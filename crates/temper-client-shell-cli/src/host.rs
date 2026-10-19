//! One call owns its files, pipes and signal subscription (client/tools.md, 8).
use crate::{
    arguments::{self, Class},
    directory::Directory,
    limits,
    output::Output,
};
use skein_io::{
    self as io,
    file_layer::FileIo,
    kernel::{Complete, Exit, Fd, Op, ServiceSignal, Submit},
};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_shell::Host;
use temper_client_tools::jig::call::Parsed;
const TAG: u64 = 1 << 61;
const FILES: u64 = 1 << 62;
const ROOM: u32 = 256; // io's bounded operations plus file and pipe completions.
/// Descriptors opened by the entry point; parent belongs to this host.
#[derive(Clone, Copy)]
pub struct Startup {
    pub parent: Option<Fd>,
    pub stdin: Fd,
    pub stdout: Fd,
    pub stderr: Fd,
    pub signals: Fd,
    pub user: u32,
    pub seed: u64,
}
/// One parsed call, with DIRECTORY's leaf, absent for help.
pub struct Invocation {
    pub directory: Option<Box<[u8]>>,
    pub parsed: Parsed,
}
/// The shell's one-shot host.
pub struct OneShot {
    directory: Directory,
    io: io::Io,
    env: Env<io::Limits>,
    output: Output,
    signals: Option<Token>,
    stdin: Option<Token>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    raw: Queue<Submit>,
    told: Queue<io::Event>,
    status: Option<u8>,
    closed: bool,
    memory: u64,
}
impl OneShot {
    /// # Errors
    /// Returns unusable client limits; the shipped preset is representable.
    pub fn new(call: Invocation, startup: Startup) -> Result<Self, jig_client::Unusable> {
        let limits = limits::native();
        let class = match &call.parsed {
            Parsed::Call { call, .. } => arguments::class(call),
            Parsed::File { .. } => Class::Help,
        };
        let mut io = io::Io::new(&limits.io);
        let output = Output::new(&mut io, startup.stdout, startup.stderr, limits.io.output);
        let stdin = Some(io.adopt_read_pipe(startup.stdin).expect("stdin slot"));
        let signals = Some(io.adopt_signals(startup.signals).expect("signal slot"));
        let memory = io::worst_case(&limits.io).expect("io envelope")
            + FileIo::worst_case(4, limits::ADDRESS, 1, limits::ADDRESS).expect("file envelope")
            + Queue::<Complete>::worst_case(ROOM).expect("completions")
            + Queue::<Submit>::worst_case(ROOM).expect("submissions") * 3
            + Queue::<io::Event>::worst_case(ROOM).expect("io events")
            + Queue::<io::file::Event>::worst_case(16).expect("file events")
            + u64::from(limits.io.output) * 128
            + 8192; // Output lines and DIRECTORY's bounded path.
        let mut host = Self {
            directory: Directory::new(
                startup.parent,
                call.directory.unwrap_or_default(),
                class,
                startup.user,
                startup.seed,
            ),
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.io },
            output,
            signals,
            stdin,
            complete: Queue::with_capacity(ROOM),
            submit: Queue::with_capacity(ROOM),
            raw: Queue::with_capacity(ROOM),
            told: Queue::with_capacity(ROOM),
            status: None,
            closed: false,
            memory,
        };
        if matches!(call.parsed, Parsed::File { .. }) {
            host.fail(b"usage: temper ctl DIRECTORY TOOL [ARGUMENT...]");
        } else if class == Class::Help {
            host.finish(0, false);
        }
        Ok(host)
    }
    fn request(&mut self, request: io::Request) {
        assert!(self.io.takes() && self.raw.room() >= io::MAX_OUT_DOWN.submissions, "host request room");
        io::down(&mut self.io, &self.env, request, &mut self.raw);
    }
    fn fail(&mut self, text: &[u8]) {
        if self.status.is_some() {
            return;
        }
        let mut line = b"temper ctl: ".to_vec();
        line.extend_from_slice(text);
        line.push(b'\n');
        self.output.push(true, line.into_boxed_slice());
        self.finish(1, false);
    }
    fn finish(&mut self, status: u8, signal: bool) {
        self.status = Some(status);
        self.directory.close();
        if signal {
            for request in self.output.close(true) {
                self.request(request);
            }
            self.closed = true;
        }
        for entity in [self.stdin.take(), self.signals.take()].into_iter().flatten() {
            self.request(io::Request::Abort { entity });
        }
    }
    fn complete(&mut self, mut complete: Complete) {
        let tag = if complete.op.raw() & FILES != 0 { FILES } else { TAG };
        complete.op = Token::new(complete.op.raw() & !tag);
        if let Op::Cancel { target } = &mut complete.kind {
            *target = Token::new(target.raw() & !tag);
        }
        if tag == FILES {
            self.directory.complete(complete);
        } else {
            io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.raw);
        }
    }
    fn publish(queue: &mut Queue<Submit>, tag: u64, out: &mut Queue<Submit>) {
        while out.room() > 0 {
            let Some(mut submit) = queue.pop() else {
                break;
            };
            assert!(submit.op.raw() & (TAG | FILES) == 0, "host token namespace");
            submit.op = Token::new(submit.op.raw() | tag);
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::new(target.raw() | tag);
            }
            out.push(submit);
        }
    }
}
impl Host for OneShot {
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
        while let Some(event) = self.told.pop() {
            match event {
                io::Event::Shutdown { signal } => self.finish(
                    match signal {
                        ServiceSignal::Interrupt => limits::INTERRUPTED,
                        ServiceSignal::Terminate => limits::TERMINATED,
                    },
                    true,
                ),
                io::Event::Stream { owner, up } => {
                    if !self.closed
                        && let Some(request) = self.output.event(owner, &up)
                    {
                        self.request(request);
                    }
                }
                io::Event::Closed { .. } | io::Event::Resized => {}
                io::Event::Failed { .. } => self.fail(b"output failed"),
                io::Event::Output { .. }
                | io::Event::Listening { .. }
                | io::Event::Accepted { .. }
                | io::Event::Connecting { .. }
                | io::Event::Connected { .. }
                | io::Event::Spawned { .. }
                | io::Event::Exited { .. }
                | io::Event::Usage { .. }
                | io::Event::Window { .. } => panic!("host owns only pipes and signals"),
            }
        }
        self.directory.progress(now);
        if let Some(error) = self.directory.error.take() {
            self.fail(&error);
        }
        for request in self.output.requests() {
            self.request(request);
        }
        if self.status.is_some() && self.output.is_empty() && !self.closed {
            for request in self.output.close(false) {
                self.request(request);
            }
            self.closed = true;
        }
        Self::publish(&mut self.raw, TAG, &mut self.submit);
        Self::publish(&mut self.directory.submissions, FILES, &mut self.submit);
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
            || !self.submit.is_empty()
            || self.io.is_ready()
            || self.io.is_due(now)
            || self.directory.pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.directory.deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.directory.retry()
    }
    fn is_empty(&self) -> bool {
        self.status.is_some()
            && self.closed
            && self.io.is_empty()
            && self.directory.is_empty()
            && self.complete.is_empty()
            && self.raw.is_empty()
            && self.submit.is_empty()
            && self.told.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then(|| Exit::Code(self.status.expect("terminal status")))
    }
    fn worst_case(&self) -> u64 {
        self.memory
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("io operations") + 2
    }
}
