//! One hosted command at a time, observed through its pipes and exit.
use crate::fixtures;
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, Spawn, Submit, Way};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
use std::{os::unix::ffi::OsStrExt, path::PathBuf};
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
const ROOM: u32 = 16; // Spawn, three pipe closes, two reads, wait and their completions.
pub(crate) struct Caller {
    pub(crate) released: bool,
    pub(crate) calls: Vec<Called>,
    steps: Vec<Step>,
    next: usize,
    scratch: PathBuf,
    root: Option<Fd>,
    active: bool,
    pending: usize,
    pidfd: Option<Fd>,
    stdout: Option<Fd>,
    stderr: Option<Fd>,
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
        Self {
            released: false,
            calls: Vec::with_capacity(steps.len()),
            steps,
            next: 0,
            scratch,
            root: Some(root),
            active: false,
            pending: 0,
            pidfd: None,
            stdout: None,
            stderr: None,
            completions: Queue::with_capacity(ROOM),
            submissions: Queue::with_capacity(ROOM),
            serial: 0,
            memory,
        }
    }
    fn submit(&mut self, kind: Op) {
        self.serial += 1;
        self.pending += 1;
        self.submissions.push(Submit { op: Token::new(self.serial), kind });
    }
    fn read(&mut self, fd: Fd) {
        self.submit(Op::PipeRead { fd, buf: vec![0; 4096].into_boxed_slice() });
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
            Step::Call { at, argv } | Step::Until { at, argv, .. } => (at, argv),
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
    fn spawn(&mut self, now: Time) {
        let args = self.words().into_boxed_slice();
        if self.calls.len() == self.next {
            self.calls.push(Called {
                exit: None,
                stdout: Vec::new(),
                stderr: Vec::new(),
                spawned: now,
                ended: now,
                tries: 0,
                early: 0,
            });
        }
        let call = &mut self.calls[self.next];
        call.stdout.clear();
        call.stderr.clear();
        call.exit = None;
        call.tries += 1;
        self.active = true;
        self.submit(Op::Spawn {
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
        });
    }
    fn complete(&mut self, complete: Complete, now: Time) {
        self.pending -= 1;
        match (complete.kind, complete.result.expect("caller operation")) {
            (Op::Spawn { spawn }, Done::Spawned { pidfd }) => {
                self.pidfd = Some(pidfd);
                for pipe in spawn.pipes {
                    let fd = pipe.parent.expect("parent pipe");
                    match pipe.child {
                        0 => self.submit(Op::Close { fd }),
                        1 => {
                            self.stdout = Some(fd);
                            self.read(fd);
                        }
                        2 => {
                            self.stderr = Some(fd);
                            self.read(fd);
                        }
                        _ => panic!("standard pipes"),
                    }
                }
            }
            (Op::PipeRead { fd, buf }, Done::Count(count)) => {
                if count == 0 {
                    if self.stdout == Some(fd) {
                        self.stdout = None;
                    } else {
                        assert_eq!(self.stderr, Some(fd));
                        self.stderr = None;
                    }
                    self.submit(Op::Close { fd });
                } else {
                    let call = &mut self.calls[self.next];
                    let bytes = if self.stdout == Some(fd) { &mut call.stdout } else { &mut call.stderr };
                    bytes.extend_from_slice(&buf[..usize::try_from(count).expect("read count")]);
                    assert!(bytes.len() <= OUTPUT, "bounded command output");
                    self.read(fd);
                }
            }
            (Op::Wait { pidfd, reap: false }, Done::Exit(_)) => {
                self.submit(Op::Wait { pidfd, reap: true });
            }
            (Op::Wait { pidfd, reap: true }, Done::Exit(exit)) => {
                self.calls[self.next].exit = Some(exit);
                self.calls[self.next].ended = now;
                self.submit(Op::Close { fd: pidfd });
            }
            (Op::Close { .. }, Done::Nothing) => {}
            (kind, done) => panic!("caller completion {kind:?}: {done:?}"),
        }
    }
    fn advance(&mut self, now: Time) {
        if self.active {
            if let Some(pidfd) = self.pidfd.take() {
                self.submit(Op::Wait { pidfd, reap: false });
                return;
            }
            let call = &mut self.calls[self.next];
            assert!(call.exit.is_some(), "completed command exit");
            if let Step::Until { holds, early, .. } = &self.steps[self.next]
                && occurrences(&call.stdout, holds) == 0
                && call.exit == Some(Exit::Code(0))
            {
                call.early += u32::from(occurrences(&call.stdout, early) != 0);
                self.spawn(now);
                return;
            }
            self.active = false;
            self.next += 1;
        }
        if self.next < self.steps.len() {
            self.spawn(now);
        } else if let Some(fd) = self.root.take() {
            self.submit(Op::Close { fd });
        }
    }
}
impl Host for Caller {
    fn iterate(&mut self, now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.complete(complete, now);
        }
        if self.released && self.pending == 0 {
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
        !self.completions.is_empty() || self.released && self.pending == 0 && self.root.is_some()
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.released && self.root.is_none() && self.pending == 0
    }
    fn worst_case(&self) -> u64 {
        self.memory
    }
    fn operations(&self) -> u32 {
        ROOM
    }
}
