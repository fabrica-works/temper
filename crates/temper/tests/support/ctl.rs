//! Executable calls beside serve, with every descriptor on one real ring.
use super::{Ended, Process, WITHIN};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, Signal, Spawn, Submit, Target, Way};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::{collections::VecDeque, path::Path};

// The service's output bound also holds the complete tools guide (4,698 bytes).
const OUTPUT: usize = 65_536;
const LINK: Token = Token::new(0);
const SPAWN: Token = Token::new(2);
const WRITE: Token = Token::new(3);
const STDOUT: Token = Token::new(4);
const STDERR: Token = Token::new(5);
const WAIT: Token = Token::new(6);
const REAP: Token = Token::new(7);
const CUT: Token = Token::new(8);
const STARTED: u64 = 1 << 31;
const OWN_CLOSE: u64 = 1 << 32;
// Poll pending promptly without holding up the interrupted call.
const PACE: Duration = Duration::from_millis(100);
// The shared engine document gives the native person's project number one.
pub const PROJECT: &str = "1";

#[derive(Clone)]
pub enum Stdin {
    Nothing,
    Link,
}
#[derive(Clone)]
pub struct Call {
    arguments: Vec<Box<[u8]>>,
    stdin: Stdin,
    directory: bool,
}
impl Call {
    #[must_use]
    pub fn new(arguments: &[&str]) -> Self {
        Self {
            arguments: arguments.iter().map(|word| word.as_bytes().into()).collect(),
            stdin: Stdin::Nothing,
            directory: true,
        }
    }
    #[must_use]
    pub fn bare(arguments: &[&str]) -> Self {
        Self { directory: false, ..Self::new(arguments) }
    }
    #[must_use]
    pub fn sign_in() -> Self {
        Self { stdin: Stdin::Link, ..Self::new(&["sign-in", "-"]) }
    }
}
pub enum Step {
    Run(Call),
    Start(Call),
    Until(Call, &'static [u8]),
    Cut,
    Pause,
    Resume,
}
pub struct Story<'a> {
    scratch: &'a Path,
    steps: Vec<Step>,
    forgejo: Option<u32>,
}
impl<'a> Story<'a> {
    #[must_use]
    pub fn new(scratch: &'a Path, steps: Vec<Step>) -> Self {
        Self { scratch, steps, forgejo: None }
    }
    #[must_use]
    pub fn on(scratch: &'a Path, forgejo: &super::forgejo::Throwaway, steps: Vec<Step>) -> Self {
        Self { scratch, steps, forgejo: Some(forgejo.pid()) }
    }
}
pub struct Seen {
    pub calls: Vec<Ended>,
    pub serve: Ended,
}

struct Child {
    base: u64,
    pidfd: Option<Fd>,
    pipes: [Option<Fd>; 3],
    input: Vec<u8>,
    written: usize,
    writing: bool,
    reading: [bool; 2],
    waiting: bool,
    closing: u32,
    ended: Ended,
}
impl Child {
    fn new(input: Vec<u8>, base: u64) -> Self {
        Self {
            base,
            pidfd: None,
            pipes: [None; 3],
            input,
            written: 0,
            writing: false,
            reading: [false; 2],
            waiting: false,
            closing: 0,
            ended: Ended { exit: None, stdout: Vec::new(), stderr: Vec::new() },
        }
    }
    fn token(&self, token: Token) -> Token {
        Token::new(self.base + token.raw())
    }
    fn close(&mut self, fd: Fd, submit: &mut Queue<Submit>) {
        self.closing += 1;
        submit.push(Submit {
            op: self.token(Token::new(100 + u64::try_from(fd.raw()).expect("descriptor"))),
            kind: Op::Close { fd },
        });
    }
    fn completed(&mut self, complete: Complete, submit: &mut Queue<Submit>) {
        let done = complete.result.expect("ctl operation succeeds");
        match Token::new(complete.op.raw() - self.base) {
            SPAWN => {
                let (Done::Spawned { pidfd }, Op::Spawn { spawn }) = (done, complete.kind) else {
                    panic!("spawned ctl")
                };
                self.pidfd = Some(pidfd);
                for (slot, pipe) in self.pipes.iter_mut().zip(spawn.pipes) {
                    *slot = pipe.parent;
                }
            }
            op if op.raw() >= 100 => self.closing -= 1,
            WRITE => {
                let Done::Count(count) = done else { panic!("stdin write count") };
                self.writing = false;
                self.written += usize::try_from(count).expect("bounded write");
            }
            STDOUT | STDERR => {
                let index = usize::from(complete.op == self.token(STDERR));
                self.reading[index] = false;
                let (Done::Count(count), Op::PipeRead { buf, .. }) = (done, complete.kind) else {
                    panic!("ctl pipe read")
                };
                let bytes = if index == 0 { &mut self.ended.stdout } else { &mut self.ended.stderr };
                bytes.extend_from_slice(&buf[..usize::try_from(count).expect("bounded read")]);
                assert!(bytes.len() <= OUTPUT, "bounded ctl output");
                if count == 0 {
                    let fd = self.pipes[index + 1].take().expect("output pipe");
                    self.close(fd, submit);
                }
            }
            WAIT => {
                let Done::Exit(exit) = done else { panic!("ctl exit") };
                self.ended.exit = Some(exit);
                submit.push(Submit {
                    op: self.token(REAP),
                    kind: Op::Wait { pidfd: self.pidfd.expect("ctl pidfd"), reap: true },
                });
            }
            REAP => {
                let fd = self.pidfd.take().expect("reaped ctl pidfd");
                self.close(fd, submit);
            }
            CUT => {}
            _ => panic!("ctl operation token"),
        }
    }
    fn drive(&mut self, submit: &mut Queue<Submit>) {
        if self.pidfd.is_none() {
            return;
        }
        if !self.writing
            && let Some(fd) = self.pipes[0]
        {
            if self.written == self.input.len() {
                self.pipes[0] = None;
                self.close(fd, submit);
            } else {
                self.writing = true;
                submit.push(Submit {
                    op: self.token(WRITE),
                    kind: Op::PipeWrite { fd, bytes: self.input[self.written..].into(), from: 0 },
                });
            }
        }
        for (index, op) in [STDOUT, STDERR].into_iter().enumerate() {
            if !self.reading[index]
                && let Some(fd) = self.pipes[index + 1]
            {
                self.reading[index] = true;
                submit.push(Submit {
                    op: self.token(op),
                    kind: Op::PipeRead { fd, buf: vec![0; 4096].into_boxed_slice() },
                });
            }
        }
        if self.pipes.iter().all(Option::is_none) && !self.waiting {
            self.waiting = true;
            submit.push(Submit {
                op: self.token(WAIT),
                kind: Op::Wait { pidfd: self.pidfd.expect("ctl pidfd"), reap: false },
            });
        }
    }
    fn finished(&self) -> bool {
        self.ended.exit.is_some() && self.pidfd.is_none() && self.closing == 0
    }
}

enum Action {
    Run,
    Until(Call, &'static [u8]),
    Signal,
}

pub struct Calls {
    root: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    closing: u32,
    link: Vec<u8>,
    steps: VecDeque<Step>,
    child: Option<Child>,
    action: Option<Action>,
    started: Option<(Child, usize)>,
    cutting: bool,
    retry: Option<Time>,
    forgejo: Option<u32>,
    ends: Vec<Ended>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
}
impl Calls {
    fn new(root: Fd, steps: Vec<Step>, streams: Option<&Streams>, forgejo: Option<u32>) -> Self {
        let mut calls = Self {
            root: Some(root),
            output: None,
            reading: false,
            closing: 0,
            link: Vec::new(),
            steps: steps.into(),
            child: None,
            action: None,
            started: None,
            cutting: false,
            retry: None,
            forgejo,
            ends: Vec::new(),
            complete: Queue::with_capacity(32),
            submit: Queue::with_capacity(32),
        };
        if let Some(Streams::Pipes { input, output }) = streams {
            calls.output = Some(*output);
            calls.close(*input);
        }
        calls
    }
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.submit.push(Submit {
            op: Token::new(OWN_CLOSE + u64::try_from(fd.raw()).expect("descriptor")),
            kind: Op::Close { fd },
        });
    }
    fn done(&self) -> bool {
        self.steps.is_empty() && self.child.is_none() && self.started.is_none() && self.action.is_none()
    }
    fn spawn(&mut self, program: Box<[u8]>, args: Vec<Box<[u8]>>, input: Vec<u8>, started: bool) {
        let child = Child::new(input, if started { STARTED } else { 0 });
        self.submit.push(Submit {
            op: child.token(SPAWN),
            kind: Op::Spawn {
                spawn: Box::new(Spawn {
                    program,
                    args: args.into_boxed_slice(),
                    env: Box::new([]),
                    root: self.root.expect("scratch root"),
                    dir: Box::new([]),
                    pipes: [Way::In, Way::Out, Way::Out]
                        .into_iter()
                        .enumerate()
                        .map(|(child, way)| Pipe {
                            child: u32::try_from(child).expect("standard descriptor"),
                            way,
                            parent: None,
                        })
                        .collect(),
                }),
            },
        });
        if started {
            assert!(self.started.is_none(), "only one started call");
            let index = self.ends.len();
            self.ends.push(Ended { exit: None, stdout: Vec::new(), stderr: Vec::new() });
            self.started = Some((child, index));
        } else {
            self.child = Some(child);
        }
    }
    fn start(&mut self, call: Call, started: bool) {
        let mut args: Vec<Box<[u8]>> = vec![b"ctl".as_slice().into()];
        if call.directory {
            args.push(b"client".as_slice().into());
        }
        args.extend(call.arguments);
        let input = match call.stdin {
            Stdin::Nothing => Vec::new(),
            Stdin::Link => {
                let mut bytes = super::printed_link(&self.link).expect("serve link").to_vec();
                bytes.push(b'\n');
                bytes
            }
        };
        self.spawn(env!("CARGO_BIN_EXE_temper").as_bytes().into(), args, input, started);
    }
    fn finish(&mut self, now: Time) {
        if self.started.as_ref().is_some_and(|(child, _)| child.finished()) {
            let (child, index) = self.started.take().expect("finished started call");
            self.ends[index] = child.ended;
            self.cutting = false;
        }
        if !self.child.as_ref().is_some_and(Child::finished) {
            return;
        }
        let ended = self.child.take().expect("finished ctl").ended;
        match self.action.take().expect("current action") {
            Action::Run => self.ends.push(ended),
            Action::Until(call, bytes) => {
                if ended.stdout.windows(bytes.len()).any(|window| window == bytes) {
                    self.ends.push(ended);
                } else {
                    self.action = Some(Action::Until(call, bytes));
                    self.retry = Some(now.saturating_add(PACE));
                }
            }
            Action::Signal => assert_eq!(
                ended.exit,
                Some(Exit::Code(0)),
                "Forgejo signal: {}",
                String::from_utf8_lossy(&ended.stderr)
            ),
        }
    }
    fn advance(&mut self, now: Time) {
        if self.child.is_some() || self.cutting || self.retry.is_some_and(|retry| retry > now) {
            return;
        }
        if let Some(Action::Until(call, _)) = &self.action {
            self.start(call.clone(), false);
            self.retry = None;
            return;
        }
        match self.steps.pop_front() {
            Some(Step::Run(call)) => {
                self.action = Some(Action::Run);
                self.start(call, false);
            }
            Some(Step::Start(call)) => self.start(call, true),
            Some(Step::Until(call, bytes)) => {
                self.action = Some(Action::Until(call.clone(), bytes));
                self.start(call, false);
            }
            Some(Step::Cut) => {
                let (child, _) = self.started.as_ref().expect("call still running at cut");
                self.cutting = true;
                self.submit.push(Submit {
                    op: child.token(CUT),
                    kind: Op::Signal {
                        pidfd: child.pidfd.expect("started call pidfd"),
                        signal: Signal::Terminate,
                        to: Target::Child,
                    },
                });
            }
            Some(step @ (Step::Pause | Step::Resume)) => {
                self.action = Some(Action::Signal);
                let signal: &[u8] = if matches!(step, Step::Pause) { b"-STOP" } else { b"-CONT" };
                self.spawn(
                    b"/usr/bin/kill".as_slice().into(),
                    vec![signal.into(), self.forgejo.expect("owned Forgejo pid").to_string().into_bytes().into()],
                    Vec::new(),
                    false,
                );
            }
            None => {}
        }
    }
}
impl Host for Calls {
    fn iterate(&mut self, now: Time, _wall: Wall) {
        while let Some(complete) = self.complete.pop() {
            if complete.op == LINK {
                self.reading = false;
                let Op::PipeRead { buf, .. } = complete.kind else { panic!("serve stdout") };
                let Done::Count(count) = complete.result.expect("serve read") else { panic!("read count") };
                self.link.extend_from_slice(&buf[..usize::try_from(count).expect("bounded read")]);
                assert!(self.link.len() <= OUTPUT, "bounded serve output");
                if count == 0 {
                    let fd = self.output.take().expect("serve stdout");
                    self.close(fd);
                }
            } else if complete.op.raw() >= OWN_CLOSE {
                complete.result.expect("fixture descriptor closed");
                self.closing -= 1;
            } else if complete.op.raw() >= STARTED {
                self.started.as_mut().expect("started call").0.completed(complete, &mut self.submit);
            } else {
                self.child.as_mut().expect("running ctl").completed(complete, &mut self.submit);
            }
        }
        self.finish(now);
        if self.closing == 0 && (self.output.is_none() || super::printed_link(&self.link).is_some()) {
            self.advance(now);
        }
        if let Some(child) = &mut self.child {
            child.drive(&mut self.submit);
        }
        if let Some((child, _)) = &mut self.started {
            child.drive(&mut self.submit);
        }
        if self.done()
            && let Some(root) = self.root.take()
        {
            self.close(root);
        }
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submit.push(Submit { op: LINK, kind: Op::PipeRead { fd, buf: vec![0; 1024].into_boxed_slice() } });
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.complete.is_empty() || !self.submit.is_empty() || self.retry.is_some_and(|retry| retry <= now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.retry
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.retry
    }
    fn is_empty(&self) -> bool {
        self.done() && self.root.is_none() && self.output.is_none() && self.closing == 0
    }
    fn worst_case(&self) -> u64 {
        2_097_152
    }
    fn operations(&self) -> u32 {
        32
    }
}

struct Judge {
    stopped: bool,
    settled: bool,
    deadline: Time,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        if procs.len() == 2 && !self.stopped {
            let Process::Calls(calls) = &procs[1] else { panic!("calls host") };
            if calls.done() {
                let Process::Binary(serve) = &mut procs[0] else { panic!("serve binary") };
                serve.signal(Signal::Terminate);
                self.stopped = true;
            }
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.settled = procs.iter().all(Host::is_empty);
        if let Process::Binary(serve) = &procs[0] {
            assert!(
                self.stopped || serve.exit_status().is_none(),
                "serve ended early: {}",
                String::from_utf8_lossy(serve.stderr())
            );
        }
        assert!(now < self.deadline || self.settled, "ctl story did not settle");
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "ctl story deadline".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
#[must_use]
pub fn run(story: Story<'_>) -> Seen {
    let clock = Clock::new();
    let mut serve = super::serve(story.scratch);
    let streams = serve.take_streams().expect("serve pipes");
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(story.scratch).expect("scratch root");
    fds.push(root);
    let mut world =
        real::World::new(Judge { stopped: false, settled: false, deadline: clock.now().now.saturating_add(WITHIN) });
    world.spawn_with_fds(serve.descriptors(), || Process::Binary(Box::new(serve)));
    world
        .spawn_with_fds(fds, || Process::Calls(Box::new(Calls::new(root, story.steps, Some(&streams), story.forgejo))));
    let mut outcome = world.run(&clock, WITHIN);
    let Process::Binary(serve) = &outcome.procs[0] else { panic!("serve binary") };
    let serve = Ended { exit: serve.exit_status(), stdout: Vec::new(), stderr: serve.stderr().to_vec() };
    assert_eq!(serve.exit, Some(Exit::Code(0)), "serve drained: {}", String::from_utf8_lossy(&serve.stderr));
    let Process::Calls(calls) = &mut outcome.procs[1] else { panic!("calls host") };
    Seen { calls: std::mem::take(&mut calls.ends), serve }
}
#[must_use]
pub fn alone(scratch: &Path, call: Call) -> Ended {
    let clock = Clock::new();
    let root = skein_shell::open_root(scratch).expect("scratch root");
    let mut world =
        real::World::new(Judge { stopped: false, settled: false, deadline: clock.now().now.saturating_add(WITHIN) });
    world.spawn_with_fds(vec![root], || Process::Calls(Box::new(Calls::new(root, vec![Step::Run(call)], None, None))));
    let mut outcome = world.run(&clock, WITHIN);
    let Process::Calls(calls) = &mut outcome.procs[0] else { panic!("calls host") };
    calls.ends.pop().expect("one ctl end")
}
#[must_use]
pub fn json(ended: &Ended) -> serde_json::Value {
    serde_json::from_slice(&ended.stdout).unwrap_or_else(|error| {
        panic!("ctl JSON: {error}; exit {:?}; stderr: {}", ended.exit, String::from_utf8_lossy(&ended.stderr))
    })
}
