//! One host owns the native client, terminal pipes and termination signals.
use super::Loop;
use crate::{client::NativeClient, limits};
use skein_io::{
    self as io,
    kernel::{Complete, Done, Exit, Fd, Op, Submit},
};
use skein_lib::{
    Duration, Env, Queue, Time, Token, Wall,
    stream::{Delimiter, Down, Read, Up},
};
use skein_shell::Host;
const TAG: u64 = 1 << 62;
const ROOM: u32 = 64;
struct Writing {
    bytes: Box<[u8]>,
    sent: usize,
    taken: usize,
    asking: bool,
}
/// Native transport and terminal io share the caller's kernel and clock.
pub struct Terminal {
    client: NativeClient,
    state: Loop,
    io: io::Io,
    env: Env<io::Limits>,
    input: Option<Token>,
    output: Option<Token>,
    signals: Option<Token>,
    reading: bool,
    held: Option<Box<[u8]>>,
    offset: usize,
    writing: Option<Writing>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    raw: Queue<Submit>,
    told: Queue<io::Event>,
    memory: u64,
    operations: u32,
}
impl Terminal {
    /// # Errors
    /// Returns the descriptor io could not adopt; fixed capacities fit all three.
    pub fn new(client: NativeClient, terminal: Loop, stdin: Fd, stdout: Fd, signals: Fd) -> Result<Self, Fd> {
        let limits = io::Limits {
            sockets: 3,
            names: 0,
            refusals: 1,
            intake: limits::terminal().read,
            receive: limits::terminal().read,
            output: limits::terminal().read,
            sends: 1,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(5),
            retry: Duration::from_millis(1),
        };
        let mut io = io::Io::new(&limits);
        let input = io.adopt_read_pipe(stdin)?;
        let output = io.adopt_write_pipe(stdout, None)?;
        let signals = io.adopt_signals(signals)?;
        let operations = client
            .operations()
            .checked_add(io::operations(&limits).expect("terminal operations"))
            .expect("combined operations");
        let room = operations.checked_mul(2).expect("completion and submission room");
        let memory = client
            .worst_case()
            .checked_add(terminal.worst_case().expect("terminal envelope"))
            .and_then(|n| n.checked_add(io::worst_case(&limits)?))
            .and_then(|n| n.checked_add(Queue::<Complete>::worst_case(room)?))
            .and_then(|n| n.checked_add(Queue::<Submit>::worst_case(room)?))
            .and_then(|n| n.checked_add(Queue::<Submit>::worst_case(ROOM)?))
            .and_then(|n| n.checked_add(Queue::<io::Event>::worst_case(ROOM)?))
            .and_then(|n| n.checked_add(u64::from(limits.intake).checked_mul(2)?))
            .expect("terminal memory fits");
        Ok(Self {
            client,
            state: terminal,
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            input: Some(input),
            output: Some(output),
            signals: Some(signals),
            reading: false,
            held: None,
            offset: 0,
            writing: None,
            complete: Queue::with_capacity(room),
            submit: Queue::with_capacity(room),
            raw: Queue::with_capacity(ROOM),
            told: Queue::with_capacity(ROOM),
            memory,
            operations,
        })
    }
    /// The binary prints this stderr line after all resources have drained.
    #[must_use]
    pub fn exit_line(&self) -> Option<Box<[u8]>> {
        self.state.exit().and_then(|(_, line)| line)
    }
    fn request(&mut self, request: io::Request) {
        assert!(self.io.takes() && self.raw.room() >= io::MAX_OUT_DOWN.submissions, "terminal request room");
        io::down(&mut self.io, &self.env, request, &mut self.raw);
    }
    fn stream(&mut self, stream: Token, down: Down) {
        self.request(io::Request::Stream { stream, down });
    }
    fn complete(&mut self, mut complete: Complete) {
        if complete.op.raw() & TAG == 0 {
            self.client.completions().push(complete);
            return;
        }
        complete.op = Token::new(complete.op.raw() & !TAG);
        if let Op::Cancel { target } = &mut complete.kind {
            *target = Token::new(target.raw() & !TAG);
        }
        if matches!(complete.kind, Op::PipeWrite { .. })
            && let Ok(Done::Count(count)) = complete.result
        {
            let writing = self.writing.as_mut().expect("stdout write owned");
            writing.taken += usize::try_from(count).expect("write count fits");
            if writing.taken == writing.bytes.len() {
                self.writing = None;
                self.state.written();
            }
        }
        io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.raw);
    }
    fn event(&mut self, event: io::Event) {
        match event {
            io::Event::Stream { owner, up: Up::Bytes(bytes) } if Some(owner) == self.input => {
                self.reading = false;
                assert!(self.held.is_none(), "one stdin chunk held");
                self.held = Some(bytes);
                self.offset = 0;
            }
            io::Event::Stream { owner, up: Up::Room } if Some(owner) == self.output => {
                let writing = self.writing.as_mut().expect("stdout room requested");
                let end = (writing.sent + usize::try_from(self.env.limits.output).expect("output chunk fits"))
                    .min(writing.bytes.len());
                let bytes = Box::from(&writing.bytes[writing.sent..end]);
                writing.sent = end;
                writing.asking = false;
                self.stream(owner, Down::Send(bytes));
            }
            io::Event::Stream { owner, up: Up::End | Up::Failed(_) } => {
                if Some(owner) == self.output {
                    self.discard_output();
                }
                self.state.close(&mut self.client);
            }
            io::Event::Shutdown { .. } | io::Event::Failed { .. } => self.state.close(&mut self.client),
            io::Event::Closed { owner } => {
                if self.input == Some(owner) {
                    self.input = None;
                }
                if self.output == Some(owner) {
                    self.output = None;
                }
                if self.signals == Some(owner) {
                    self.signals = None;
                }
            }
            io::Event::Resized => {}
            io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. } => panic!("terminal owns pipes and signals only"),
        }
    }
    fn discard_output(&mut self) {
        if self.writing.take().is_some() {
            self.state.written();
        }
        self.state.close(&mut self.client);
        for _ in 0..=limits::terminal().lines {
            self.state.iterate(self.env.now, &mut self.client);
            if self.state.output().is_none() {
                break;
            }
            self.state.written();
        }
    }
    fn pump(&mut self) {
        if let Some(bytes) = &self.held {
            self.offset += self.state.stdin(&bytes[self.offset..], self.env.now, &mut self.client);
            if self.offset == bytes.len() {
                self.held = None;
            }
        }
        self.state.iterate(self.env.now, &mut self.client);
        if self.writing.is_none()
            && let Some(bytes) = self.state.output()
        {
            self.writing = Some(Writing { bytes, sent: 0, taken: 0, asking: false });
        }
        if let Some(writing) = &mut self.writing
            && !writing.asking
            && writing.taken == writing.sent
            && writing.sent < writing.bytes.len()
        {
            let room =
                u32::try_from(writing.bytes.len() - writing.sent).unwrap_or(u32::MAX).min(self.env.limits.output);
            writing.asking = true;
            self.stream(self.output.expect("stdout open"), Down::Demand { read: Read::Nothing, room });
        }
        if self.state.closing() {
            for entity in [self.input.take(), self.signals.take()].into_iter().flatten() {
                self.request(io::Request::Abort { entity });
            }
            self.held = None;
            if self.state.exit().is_some()
                && let Some(entity) = self.output.take()
            {
                self.request(io::Request::Close { entity });
            }
        } else if !self.reading
            && self.held.is_none()
            && let Some(stream) = self.input
        {
            self.reading = true;
            self.stream(
                stream,
                Down::Demand { read: Read::Scan { until: Delimiter::LF, max: limits::terminal().read }, room: 0 },
            );
        }
    }
    fn publish(&mut self) {
        while self.submit.room() > 0 {
            let Some(mut submit) = self.raw.pop() else {
                break;
            };
            assert!(submit.op.raw() & TAG == 0, "terminal token namespace");
            submit.op = Token::new(submit.op.raw() | TAG);
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::new(target.raw() | TAG);
            }
            self.submit.push(submit);
        }
        while self.submit.room() > 0 {
            let Some(submit) = self.client.submissions().pop() else {
                break;
            };
            self.submit.push(submit);
        }
    }
}
impl Host for Terminal {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        while self.io.is_ready()
            && self.raw.room() >= io::MAX_OUT_RESUME.submissions
            && self.told.room() >= io::MAX_OUT_RESUME.events
        {
            io::resume(&mut self.io, &self.env, &mut self.told, &mut self.raw);
        }
        while !self.io.is_ready()
            && self.raw.room() >= io::MAX_OUT_UP.submissions
            && self.told.room() >= io::MAX_OUT_UP.events
        {
            let Some(complete) = self.complete.pop() else {
                break;
            };
            self.complete(complete);
        }
        while self.io.is_due(now)
            && self.raw.room() >= io::MAX_OUT_FIRE.submissions
            && self.told.room() >= io::MAX_OUT_FIRE.events
        {
            io::fire(&mut self.io, &self.env, &mut self.told, &mut self.raw);
        }
        self.client.iterate(now, wall);
        while let Some(event) = self.told.pop() {
            self.event(event);
        }
        self.pump();
        self.publish();
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.complete.is_empty()
            || !self.raw.is_empty()
            || self.io.is_ready()
            || self.io.is_due(now)
            || self.client.work_pending(now)
            || !self.submit.is_empty()
            || self.state.next_deadline().is_some_and(|deadline| now >= deadline)
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.client.next_deadline(), self.io.next_deadline(), self.state.next_deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [self.client.next_policy_deadline(), self.state.next_deadline()].into_iter().flatten().min()
    }
    fn is_empty(&self) -> bool {
        self.client.is_empty()
            && self.io.is_empty()
            && self.complete.is_empty()
            && self.submit.is_empty()
            && self.raw.is_empty()
            && self.told.is_empty()
            && self.state.exit().is_some()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then(|| Exit::Code(self.state.exit().expect("drained loop").0))
    }
    fn worst_case(&self) -> u64 {
        self.memory
    }
    fn operations(&self) -> u32 {
        self.operations
    }
}
