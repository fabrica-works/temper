//! One ctl child at a time, with bounded pipes and explicit reaping.
use skein_io::kernel::{
    Complete, Done, Exit, Fd, Op, Pipe, ServiceSignal, Signal, Signalled, Spawn, Submit, Target, Way,
};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::{Clock, Config, Host, Kernel};
use std::os::unix::ffi::OsStrExt;
use temper_box::host::LINE_BYTES;
use temper_fake::seed::{self, CALL_OUTPUT, Ended, Next, Plan};

const ROOM: u32 = 32;
const INPUT: Token = Token::new(1);
const SIGNAL: Token = Token::new(2);
const SPAWN: Token = Token::new(3);
const OUT: Token = Token::new(4);
const ERR: Token = Token::new(5);
const WRITE: Token = Token::new(6);
const WAIT: Token = Token::new(7);
const REAP: Token = Token::new(8);
const SAY: Token = Token::new(9);
const KILL: Token = Token::new(10);
const CANCEL: Token = Token::new(11);
const CLOSE: Token = Token::new(12);

pub fn run(directories: &crate::Directories) -> Result<u8, String> {
    start(directories).map_err(|why| crate::prefixed("temper fake seed", &why))
}
fn start(directories: &crate::Directories) -> Result<u8, String> {
    let bytes = super::read_document(&directories.document(), crate::CONFIG_BYTES)?;
    let deployment = crate::setup::check(&bytes, &directories.jig())?;
    let demo = deployment.demo.ok_or("application.demo is null: the deployment has no demo")?;
    let program = std::env::current_exe().map_err(|why| why.to_string())?;
    let mut owned = Vec::new();
    let opened = (|| {
        let root =
            skein_shell::open_root(crate::path(&directories.data)).map_err(|why| format!("directory: errno {why}"))?;
        owned.push(root);
        let signals = skein_shell::open_termination_signals().map_err(|why| format!("signals: errno {why}"))?;
        owned.push(signals);
        let service =
            Service::new(Plan::new(&demo), demo.client().into(), program.as_os_str().as_bytes().into(), root, signals);
        let kernel = Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string())?;
        Ok::<_, String>((service, kernel))
    })();
    let (mut service, mut kernel) = match opened {
        Ok(value) => value,
        Err(why) => {
            crate::close(owned);
            return Err(why);
        }
    };
    Ok(match skein_shell::drive(&mut kernel, &Clock::new(), &mut service) {
        Exit::Code(status) => status,
        Exit::Signal(_) => 1,
    })
}
struct Child {
    spawning: bool,
    pid: Option<Fd>,
    pipes: [Option<Fd>; 3],
    reading: [bool; 2],
    writing: bool,
    written: usize,
    input: bool,
    status: Option<Exit>,
    output: [Vec<u8>; 2],
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reading {
    Idle,
    Active,
    Cancelling,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stopping {
    Running,
    Terminating,
    Killing,
}
struct Service {
    plan: Plan,
    client: Box<[u8]>,
    program: Box<[u8]>,
    root: Option<Fd>,
    signals: Option<Fd>,
    signal_read: Reading,
    stdin: Option<Fd>,
    input_read: Reading,
    link: Vec<u8>,
    child: Option<Child>,
    line: Option<Box<[u8]>>,
    saying: bool,
    said: usize,
    status: Option<u8>,
    stopping: Stopping,
    closing: u32,
    active: u32,
    serial: u64,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
    memory: u64,
}
impl Service {
    fn new(plan: Plan, client: Box<[u8]>, program: Box<[u8]>, root: Fd, signals: Fd) -> Self {
        // Parser scratch follows the CLI's per-input-byte envelope. Plan and
        // spawn each carry bounded paths; one stdin sink and two read chunks
        // coexist with the retained link and the child's two complete streams.
        let memory = u64::try_from(size_of::<Self>()).expect("host size")
            + Queue::<Complete>::worst_case(ROOM).expect("completion queue")
            + Queue::<Submit>::worst_case(ROOM).expect("submission queue")
            + u64::from(CALL_OUTPUT) * 258 // two streams plus JSON parser scratch
            + u64::from(LINE_BYTES) * 3 // link, input read, child's stdin sink
            + 4096 * 2 // concurrent stdout/stderr read chunks
            + u64::from(temper_fake::limits::PATH_BYTES) * 8 // plan/spawn paths and arguments
            + u64::try_from(program.len() * 2 + client.len() * 2).expect("command bytes")
            + 2048; // one diagnostic and its in-flight write, with metadata
        Self {
            plan,
            client,
            program,
            root: Some(root),
            signals: Some(signals),
            signal_read: Reading::Idle,
            stdin: Some(Fd::new(0)),
            input_read: Reading::Idle,
            link: Vec::with_capacity(usize::try_from(LINE_BYTES).expect("link bound") + 1),
            child: None,
            line: None,
            saying: false,
            said: 0,
            status: None,
            stopping: Stopping::Running,
            closing: 0,
            active: 0,
            serial: 0,
            complete: Queue::with_capacity(ROOM),
            submit: Queue::with_capacity(ROOM),
            memory,
        }
    }
    fn request(&mut self, op: Token, kind: Op) {
        self.active += 1;
        // Close, cancellation and signal submissions can overlap. Their low
        // bits retain the kind; each in-flight operation has its own token.
        let op = if [CLOSE, CANCEL, KILL].contains(&op) {
            self.serial = self.serial.checked_add(1).expect("seed operation names");
            Token::new(self.serial.checked_mul(16).expect("seed operation names") + op.raw())
        } else {
            op
        };
        self.submit.push(Submit { op, kind });
    }
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.request(CLOSE, Op::Close { fd });
    }
    fn say(&mut self, bytes: Box<[u8]>) {
        assert!(self.line.is_none(), "one diagnostic at a time");
        self.said = 0;
        self.line = Some(bytes);
    }
    fn fail(&mut self, why: &[u8]) {
        if self.status.is_some() {
            return;
        }
        let status = self.child.as_ref().and_then(|child| match child.status {
            Some(Exit::Code(code)) => Some(code),
            _ => None,
        });
        let stderr = self.child.as_ref().map_or(b"".as_slice(), |child| child.output[1].as_slice());
        let failed = seed::failure(self.plan.act(), why, &Ended { status, stdout: &[], stderr });
        self.say(failed.why);
        self.status = Some(1);
        self.signal_child(Signal::Kill);
    }
    fn signal_child(&mut self, signal: Signal) {
        if let Some(pidfd) = self.child.as_ref().and_then(|child| child.pid) {
            self.request(KILL, Op::Signal { pidfd, signal, to: Target::Child });
        }
    }
    fn stop(&mut self, signal: ServiceSignal) {
        if self.stopping != Stopping::Running {
            self.stopping = Stopping::Killing;
            self.signal_child(Signal::Kill);
            return;
        }
        self.stopping = Stopping::Terminating;
        self.status = Some(match signal {
            ServiceSignal::Terminate => 143,
            ServiceSignal::Interrupt => 130,
        });
        if !self.saying {
            self.line = None;
        }
        self.signal_child(Signal::Terminate);
    }
    fn spawned(&mut self, complete: Complete) {
        let child = self.child.as_mut().expect("spawning child");
        child.spawning = false;
        if let (Ok(Done::Spawned { pidfd }), Op::Spawn { spawn }) = (complete.result, complete.kind) {
            child.pid = Some(pidfd);
            for pipe in spawn.pipes {
                child.pipes[usize::try_from(pipe.child).expect("standard descriptor")] = pipe.parent;
            }
            self.request(WAIT, Op::Wait { pidfd, reap: false });
            if self.stopping != Stopping::Running {
                self.signal_child(if self.stopping == Stopping::Killing { Signal::Kill } else { Signal::Terminate });
            }
        } else {
            self.fail(b"ctl could not be spawned");
        }
    }
    fn read_input(&mut self, complete: Complete) {
        self.input_read = Reading::Idle;
        let Op::PipeRead { buf, .. } = complete.kind else { unreachable!("stdin read") };
        if self.status.is_some() {
            return;
        }
        match complete.result {
            Ok(Done::Count(0)) => {
                let fd = self.stdin.take().expect("stdin");
                self.close(fd);
                if self.link.is_empty() {
                    self.fail(b"stdin holds no link");
                }
            }
            Ok(Done::Count(count)) => {
                self.link.extend_from_slice(&buf[..usize::try_from(count).expect("read bound")]);
                if self.link.len() > usize::try_from(LINE_BYTES).expect("link bound") {
                    self.fail(b"stdin link past LINE_BYTES");
                }
            }
            _ => self.fail(b"stdin link could not be read"),
        }
    }
    fn child_read(&mut self, complete: Complete, index: usize) {
        let child = self.child.as_mut().expect("running child");
        child.reading[index] = false;
        let Op::PipeRead { buf, .. } = complete.kind else { unreachable!("child read") };
        match complete.result {
            Ok(Done::Count(0)) => {
                let fd = child.pipes[index + 1].take().expect("child pipe");
                self.close(fd);
            }
            Ok(Done::Count(count)) => {
                let count = usize::try_from(count).expect("read count");
                if child.output[index].len() + count > usize::try_from(CALL_OUTPUT).expect("output bound") {
                    self.fail(b"ctl stream past CALL_OUTPUT");
                } else if self.status.is_none() {
                    child.output[index].extend_from_slice(&buf[..count]);
                }
            }
            _ => {
                let fd = child.pipes[index + 1].take().expect("child pipe");
                self.close(fd);
                self.fail(b"ctl stream could not be read");
            }
        }
    }
    fn completed(&mut self, complete: Complete) {
        self.active -= 1;
        match Token::new(complete.op.raw() % 16) {
            INPUT => self.read_input(complete),
            SIGNAL => {
                self.signal_read = Reading::Idle;
                if let Ok(Done::Signalled(Signalled::Termination(signal))) = complete.result {
                    self.stop(signal);
                }
            }
            SPAWN => self.spawned(complete),
            OUT => self.child_read(complete, 0),
            ERR => self.child_read(complete, 1),
            WRITE => {
                let child = self.child.as_mut().expect("child stdin");
                child.writing = false;
                if let Ok(Done::Count(count)) = complete.result
                    && count != 0
                {
                    child.written += usize::try_from(count).expect("write count");
                    return;
                }
                let fd = child.pipes[0].take().expect("child stdin");
                self.close(fd);
                // The call's exit decides whether it accepted the link.
            }
            WAIT => {
                let child = self.child.as_mut().expect("waited child");
                if let Ok(Done::Exit(exit)) = complete.result {
                    child.status = Some(exit);
                } else {
                    self.fail(b"ctl could not be waited on");
                }
                let pidfd = self.child.as_ref().and_then(|child| child.pid).expect("wait pidfd");
                self.request(REAP, Op::Wait { pidfd, reap: true });
            }
            REAP => {
                let fd = self.child.as_mut().expect("reaped child").pid.take().expect("child pidfd");
                self.close(fd);
            }
            SAY => {
                self.saying = false;
                if let Ok(Done::Count(count)) = complete.result {
                    self.said += usize::try_from(count).expect("line count");
                    if count > 0
                        && self.stopping == Stopping::Running
                        && self.said < self.line.as_ref().expect("line").len()
                    {
                        return;
                    }
                }
                self.line = None;
            }
            CLOSE => {
                self.closing -= 1;
            }
            KILL | CANCEL => {}
            _ => unreachable!("seed operation"),
        }
    }
    fn spawn(&mut self) {
        let Next::Call(call) = self.plan.next() else {
            self.status = Some(0);
            return;
        };
        let input = call.link;
        let mut args = vec![b"ctl".as_slice().into(), self.client.clone()];
        args.extend_from_slice(call.words);
        let mut pipes =
            vec![Pipe { child: 1, way: Way::Out, parent: None }, Pipe { child: 2, way: Way::Out, parent: None }];
        if input {
            pipes.push(Pipe { child: 0, way: Way::In, parent: None });
        }
        let spawn = Spawn {
            program: self.program.clone(),
            args: args.into(),
            env: Box::new([]),
            root: self.root.expect("child root"),
            dir: Box::new([]),
            pipes: pipes.into(),
        };
        self.child = Some(Child {
            spawning: true,
            pid: None,
            pipes: [None; 3],
            reading: [false; 2],
            writing: false,
            written: 0,
            input,
            status: None,
            output: [Vec::new(), Vec::new()],
        });
        self.request(SPAWN, Op::Spawn { spawn: Box::new(spawn) });
    }
    fn child_work(&mut self) {
        let Some(child) = self.child.as_mut() else {
            return;
        };
        if child.spawning {
            return;
        }
        let mut writes = Vec::new();
        if !child.writing
            && let Some(fd) = child.pipes[0]
        {
            if !child.input || child.written == self.link.len() || self.status.is_some() {
                child.pipes[0] = None;
                writes.push((CLOSE, Op::Close { fd }));
                self.closing += 1;
            } else {
                child.writing = true;
                writes.push((WRITE, Op::PipeWrite { fd, bytes: self.link[child.written..].into(), from: 0 }));
            }
        }
        for (index, token) in [OUT, ERR].into_iter().enumerate() {
            if !child.reading[index]
                && let Some(fd) = child.pipes[index + 1]
            {
                child.reading[index] = true;
                // Read only the remaining bound plus one to detect overflow.
                let room =
                    usize::try_from(CALL_OUTPUT).expect("output bound").saturating_sub(child.output[index].len());
                writes.push((token, Op::PipeRead { fd, buf: vec![0; 4096.min(room + 1)].into() }));
            }
        }
        for (token, kind) in writes {
            self.request(token, kind);
        }
        if self.child.as_ref().is_some_and(|child| child.pid.is_none() && child.pipes.iter().all(Option::is_none))
            && self.closing == 0
        {
            let child = self.child.take().expect("finished child");
            if self.status.is_none() {
                let status = match child.status {
                    Some(Exit::Code(code)) => Some(code),
                    _ => None,
                };
                match self.plan.ended(&Ended { status, stdout: &child.output[0], stderr: &child.output[1] }) {
                    Ok(Some(line)) => self.say(line),
                    Ok(None) => {}
                    Err(failed) => {
                        self.say(failed.why);
                        self.status = Some(1);
                    }
                }
            }
        }
    }
    fn advance(&mut self) {
        self.child_work();
        if let Some(line) = &self.line
            && !self.saying
        {
            self.saying = true;
            self.request(SAY, Op::PipeWrite { fd: Fd::new(2), bytes: line[self.said..].into(), from: 0 });
        }
        if self.status.is_none()
            && self.stdin.is_none()
            && self.child.is_none()
            && self.line.is_none()
            && self.closing == 0
        {
            self.spawn();
        }
        if self.status.is_none()
            && self.input_read == Reading::Idle
            && let Some(fd) = self.stdin
        {
            self.input_read = Reading::Active;
            self.request(
                INPUT,
                Op::PipeRead {
                    fd,
                    buf: vec![0; usize::try_from(LINE_BYTES).expect("link bound") + 1 - self.link.len()].into(),
                },
            );
        }
        let finish = self.status.is_some() && self.child.is_none() && self.line.is_none();
        if self.status.is_some()
            && let Some(fd) = self.stdin
        {
            if self.input_read == Reading::Idle {
                self.stdin = None;
                self.close(fd);
            } else if self.input_read != Reading::Cancelling {
                self.input_read = Reading::Cancelling;
                self.request(CANCEL, Op::Cancel { target: INPUT });
            }
        }
        if finish {
            if let Some(fd) = self.root.take() {
                self.close(fd);
            }
            if self.signal_read != Reading::Idle {
                if self.signal_read != Reading::Cancelling {
                    self.signal_read = Reading::Cancelling;
                    self.request(CANCEL, Op::Cancel { target: SIGNAL });
                }
            } else if let Some(fd) = self.signals.take() {
                self.close(fd);
            }
        } else if self.signal_read == Reading::Idle
            && let Some(fd) = self.signals
        {
            self.signal_read = Reading::Active;
            self.request(SIGNAL, Op::ReadSignal { fd });
        }
    }
}
impl Host for Service {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(complete) = self.complete.pop() {
            self.completed(complete);
        }
        self.advance();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.complete.is_empty() || !self.submit.is_empty()
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.status.is_some()
            && self.child.is_none()
            && self.stdin.is_none()
            && self.signals.is_none()
            && self.root.is_none()
            && self.line.is_none()
            && self.active == 0
    }
    fn worst_case(&self) -> u64 {
        self.memory
    }
    fn operations(&self) -> u32 {
        ROOM
    }
    fn exit(&self) -> Option<Exit> {
        self.status.map(Exit::Code)
    }
}
