//! Hosted commands observed through their pipes, exit and a service watch cue.
use crate::fixtures;
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, Spawn, Submit, Way};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
use std::{collections::BTreeMap, os::unix::ffi::OsStrExt, path::PathBuf};
#[derive(Clone, Copy)]
pub struct Dir(pub u8);
pub const D1: Dir = Dir(1);
pub enum Arg {
    Lit(&'static str),
    Task(usize),
}
impl From<&'static str> for Arg {
    fn from(word: &'static str) -> Self {
        Self::Lit(word)
    }
}
pub struct Argv(pub Vec<Arg>);
#[macro_export]
macro_rules! argv { ($($word:expr),* $(,)?) => { $crate::calls::Argv(vec![$($crate::calls::Arg::from($word)),*]) }; }
pub enum Step {
    SignIn { at: Dir },
    Call { at: Dir, argv: Argv },
    Start { at: Dir, argv: Argv },
    Join { call: usize },
    Watching { call: usize },
    Until { at: Dir, argv: Argv, holds: &'static [u8], early: &'static [u8] },
}
#[must_use]
pub fn sign_in(at: Dir) -> Step {
    Step::SignIn { at }
}
#[must_use]
pub fn call(at: Dir, argv: Argv) -> Step {
    Step::Call { at, argv }
}
#[must_use]
pub fn until(at: Dir, argv: Argv, holds: &'static [u8], early: &'static [u8]) -> Step {
    Step::Until { at, argv, holds, early }
}
#[must_use]
pub fn start(at: Dir, argv: Argv) -> Step {
    Step::Start { at, argv }
}
#[must_use]
pub fn join(call: usize) -> Step {
    Step::Join { call }
}
#[must_use]
pub fn watching(call: usize) -> Step {
    Step::Watching { call }
}
#[must_use]
pub fn task(call: usize) -> Arg {
    Arg::Task(call)
}
#[must_use]
pub fn occurrences(bytes: &[u8], needle: &[u8]) -> usize {
    bytes.windows(needle.len()).filter(|part| *part == needle).count()
}
pub struct Called {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub spawned: Time,
    pub ended: Time,
    pub tries: u32,
    pub early: u32,
}
// A complete small rendered turn fits; a pipe chunk alone could miss a split reply.
const OUTPUT: usize = 65_536;
const ROOM: u32 = 16 * fixtures::CALLS_AT_ONCE; // Spawn, pipe closes, reads and wait for each call.
#[derive(Default)]
struct Running {
    pending: usize,
    started: bool,
    pidfd: Option<Fd>,
    stdout: Option<Fd>,
    stderr: Option<Fd>,
}
pub(crate) struct Caller {
    pub(crate) released: bool,
    pub(crate) calls: Vec<Called>,
    steps: Vec<Step>,
    next: usize,
    scratch: PathBuf,
    root: Option<Fd>,
    current: Option<usize>,
    running: Vec<Option<Running>>,
    owners: BTreeMap<Token, usize>,
    pub(crate) watching: Option<usize>,
    pub(crate) watched: bool,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    serial: u64,
    memory: u64,
}
impl Caller {
    pub(crate) fn new(steps: Vec<Step>, scratch: PathBuf, root: Fd) -> Self {
        let memory = u64::try_from((steps.len() + 1) * (OUTPUT * 4 + 4096)).expect("caller memory")
            + Queue::<Complete>::worst_case(ROOM).expect("completions")
            + Queue::<Submit>::worst_case(ROOM).expect("submissions");
        let count = steps.len();
        Self {
            released: false,
            calls: Vec::with_capacity(count),
            steps,
            next: 0,
            scratch,
            root: Some(root),
            current: None,
            running: Vec::with_capacity(count),
            owners: BTreeMap::new(),
            watching: None,
            watched: false,
            completions: Queue::with_capacity(ROOM),
            submissions: Queue::with_capacity(ROOM),
            serial: 0,
            memory,
        }
    }
    fn submit(&mut self, call: usize, kind: Op) {
        self.serial += 1;
        if call != usize::MAX {
            self.running[call].as_mut().expect("running command").pending += 1;
        }
        self.owners.insert(Token::new(self.serial), call);
        self.submissions.push(Submit { op: Token::new(self.serial), kind });
    }
    fn read(&mut self, call: usize, fd: Fd) {
        self.submit(call, Op::PipeRead { fd, buf: vec![0; 4096].into_boxed_slice() });
    }
    fn words(&self) -> Vec<Box<[u8]>> {
        let (at, argv) = match &self.steps[self.next] {
            Step::SignIn { at } => {
                return vec![
                    fixtures::directory(&self.scratch, *at).as_os_str().as_bytes().into(),
                    b"sign-in".as_slice().into(),
                    self.scratch.join("link").as_os_str().as_bytes().into(),
                ];
            }
            Step::Call { at, argv } | Step::Start { at, argv } | Step::Until { at, argv, .. } => (at, argv),
            Step::Join { .. } | Step::Watching { .. } => panic!("command step"),
        };
        let mut words = vec![fixtures::directory(&self.scratch, *at).as_os_str().as_bytes().into()];
        for arg in &argv.0 {
            words.push(match arg {
                Arg::Lit(word) => word.as_bytes().into(),
                Arg::Task(call) => {
                    let text = std::str::from_utf8(&self.calls[*call].stdout).expect("call stdout");
                    let number = text
                        .split("done: task ")
                        .nth(1)
                        .expect("created task")
                        .split(|c: char| !c.is_ascii_digit())
                        .next()
                        .expect("task number");
                    assert!(!number.is_empty(), "created task number");
                    number.as_bytes().into()
                }
            });
        }
        words
    }
    fn spawn(&mut self, now: Time, index: usize) {
        let args = self.words().into_boxed_slice();
        if self.calls.len() == index {
            self.calls.push(Called {
                exit: None,
                stdout: Vec::new(),
                stderr: Vec::new(),
                spawned: now,
                ended: now,
                tries: 0,
                early: 0,
            });
            self.running.push(None);
        }
        let call = &mut self.calls[index];
        call.stdout.clear();
        call.stderr.clear();
        call.exit = None;
        call.tries += 1;
        self.running[index] = Some(Running::default());
        if matches!(self.steps[self.next], Step::Start { .. })
            && matches!(self.steps.get(self.next + 1), Some(Step::Watching { call }) if *call == index)
        {
            // Arm before the spawned host can connect; release still waits for its task delivery.
            self.watching = Some(index);
        }
        self.submit(
            index,
            Op::Spawn {
                spawn: Box::new(Spawn {
                    program: fixtures::CTL.into(),
                    args,
                    env: Box::new([]),
                    root: self.root.expect("launch root"),
                    dir: Box::new([]),
                    pipes: [(0, Way::In), (1, Way::Out), (2, Way::Out)]
                        .map(|(child, way)| Pipe { child, way, parent: None })
                        .into(),
                }),
            },
        );
    }
    fn complete(&mut self, complete: Complete, now: Time) {
        let index = self.owners.remove(&complete.op).expect("operation owner");
        if index == usize::MAX {
            assert!(matches!(complete.kind, Op::Close { .. }));
            complete.result.expect("root close");
            return;
        }
        self.running[index].as_mut().expect("running command").pending -= 1;
        match (complete.kind, complete.result.expect("caller operation")) {
            (Op::Spawn { spawn }, Done::Spawned { pidfd }) => {
                let running = self.running[index].as_mut().expect("running command");
                running.pidfd = Some(pidfd);
                running.started = true;
                for pipe in spawn.pipes {
                    let fd = pipe.parent.expect("parent pipe");
                    match pipe.child {
                        0 => self.submit(index, Op::Close { fd }),
                        1 => {
                            self.running[index].as_mut().expect("running command").stdout = Some(fd);
                            self.read(index, fd);
                        }
                        2 => {
                            self.running[index].as_mut().expect("running command").stderr = Some(fd);
                            self.read(index, fd);
                        }
                        _ => panic!("standard pipes"),
                    }
                }
            }
            (Op::PipeRead { fd, buf }, Done::Count(count)) => {
                if count == 0 {
                    if self.running[index].as_mut().expect("running command").stdout == Some(fd) {
                        self.running[index].as_mut().expect("running command").stdout = None;
                    } else {
                        assert_eq!(self.running[index].as_mut().expect("running command").stderr, Some(fd));
                        self.running[index].as_mut().expect("running command").stderr = None;
                    }
                    self.submit(index, Op::Close { fd });
                } else {
                    let stdout = self.running[index].as_ref().expect("running command").stdout == Some(fd);
                    let call = &mut self.calls[index];
                    let bytes = if stdout { &mut call.stdout } else { &mut call.stderr };
                    bytes.extend_from_slice(&buf[..usize::try_from(count).expect("read count")]);
                    assert!(bytes.len() <= OUTPUT, "bounded command output");
                    self.read(index, fd);
                }
            }
            (Op::Wait { pidfd, reap: false }, Done::Exit(_)) => {
                self.submit(index, Op::Wait { pidfd, reap: true });
            }
            (Op::Wait { pidfd, reap: true }, Done::Exit(exit)) => {
                self.calls[index].exit = Some(exit);
                self.calls[index].ended = now;
                self.submit(index, Op::Close { fd: pidfd });
            }
            (Op::Close { .. }, Done::Nothing) => {}
            (kind, done) => panic!("caller completion {kind:?}: {done:?}"),
        }
    }
    fn settle(&mut self) {
        for index in 0..self.running.len() {
            let Some(running) = &mut self.running[index] else { continue };
            if running.pending != 0 {
                continue;
            }
            if let Some(pidfd) = running.pidfd.take() {
                self.submit(index, Op::Wait { pidfd, reap: false });
            } else {
                assert!(self.calls[index].exit.is_some(), "completed command exit");
                self.running[index] = None;
            }
        }
    }
    fn advance(&mut self, now: Time) {
        let Some(step) = self.steps.get(self.next) else {
            if self.running.iter().all(Option::is_none)
                && let Some(fd) = self.root.take()
            {
                self.submit(usize::MAX, Op::Close { fd });
            }
            return;
        };
        match step {
            Step::Watching { call } => {
                assert!(self.running[*call].as_ref().is_some_and(|r| r.started), "started watch call");
                self.watching = Some(*call);
                if !self.watched {
                    return;
                }
                self.watching = None;
                self.watched = false;
            }
            Step::Join { call } => {
                if self.running[*call].is_some() {
                    return;
                }
            }
            Step::SignIn { .. } | Step::Call { .. } | Step::Start { .. } | Step::Until { .. } => {
                let Some(index) = self.current else {
                    assert!(self.running.iter().flatten().count() < fixtures::CALLS_AT_ONCE as usize);
                    let index = self.calls.len();
                    self.spawn(now, index);
                    self.current = Some(index);
                    return;
                };
                if let Step::Start { .. } = step {
                    if !self.running[index].as_ref().is_some_and(|r| r.started) {
                        return;
                    }
                } else if self.running[index].is_some() {
                    return;
                }
                let call = &mut self.calls[index];
                if let Step::Until { holds, early, .. } = step
                    && occurrences(&call.stdout, holds) == 0
                    && call.exit == Some(Exit::Code(0))
                {
                    call.early += u32::from(occurrences(&call.stdout, early) != 0);
                    self.spawn(now, index);
                    return;
                }
                self.current = None;
            }
        }
        self.next += 1;
    }
    fn can_advance(&self) -> bool {
        match self.steps.get(self.next) {
            Some(Step::Watching { .. }) => self.watched || self.watching.is_none(),
            Some(Step::Join { call }) => self.running[*call].is_none(),
            Some(Step::Start { .. }) => {
                self.current.is_none_or(|i| self.running[i].as_ref().is_some_and(|r| r.started))
            }
            Some(_) => self.current.is_none_or(|i| self.running[i].is_none()),
            None => self.root.is_some() && self.running.iter().all(Option::is_none),
        }
    }
}
impl Host for Caller {
    fn iterate(&mut self, now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.complete(complete, now);
        }
        self.settle();
        if self.released {
            self.advance(now);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.completions.is_empty() || self.released && self.can_advance()
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.released && self.root.is_none() && self.owners.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.memory
    }
    fn operations(&self) -> u32 {
        ROOM
    }
}
