use crate::{Identity, Limits};
use alloc::boxed::Box;
use skein_lib::{Duration, Secret};
use temper_worker_domain_checkout::{self as checkout, git};
pub const HOME: &[u8] = b"/srv/worker/home";
pub const REMOTE: &[u8] = b"https://forge.example.test/acme/app.git";
pub const COMMIT: git::Commit = git::Commit::new([
    0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab,
    0xab, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
pub const COMMIT_256: git::Commit = git::Commit::new([0xcd; 32]);
pub fn checkout() -> checkout::Limits {
    checkout::Limits {
        workspaces: 2,
        repositories: 2,
        name_bytes: 64,
        message_bytes: 256,
        conflicts: 4,
        path_bytes: 64,
        remote_timeout: Duration::from_secs(30),
        local_timeout: Duration::from_secs(5),
        facts: 8,
    }
}
pub fn limits() -> Limits {
    Limits { operations: 2, stdout_bytes: 512, stderr_bytes: 512, read_bytes: 64, sweep_entries: 4, sweep_bytes: 255 }
}
pub fn identity() -> Identity {
    Identity {
        token: Secret::new(Box::from(&b"tok-1"[..])),
        login: Box::from(&b"temper-bot"[..]),
        name: Box::from(&b"Temper Bot"[..]),
        email: Box::from(&b"bot@example.test"[..]),
    }
}

pub const OWNER: skein_lib::Token = skein_lib::Token::new(91);
pub const WORKSPACE: skein_lib::Token = skein_lib::Token::new(7);
pub const DEADLINE: skein_lib::Time = skein_lib::Time::from_nanos(1000);
pub const PARENT: git::Commit = COMMIT_256;
pub const EXPECTED: git::Commit = COMMIT_256;
pub fn at() -> git::Place {
    git::Place { workspace: WORKSPACE, repository: Box::from(&b"app"[..]) }
}
pub fn fetch(owner: skein_lib::Token, deadline: skein_lib::Time) -> crate::Request {
    crate::Request::Io {
        owner,
        op: git::Op::Fetch { at: at(), remote: Box::from(REMOTE), want: git::Want::Default, identity: 1 },
        deadline,
        identity: Some(identity()),
    }
}
pub fn commit_op(merging: Option<git::Commit>) -> git::Op {
    git::Op::Commit {
        at: at(),
        parent: PARENT,
        merging,
        title: Box::from(&b"Title"[..]),
        body: Box::from(&b"Body"[..]),
        identity: 1,
    }
}
pub fn push_op(expected: Option<git::Commit>) -> git::Op {
    git::Op::Push {
        at: at(),
        remote: Box::from(REMOTE),
        commit: COMMIT,
        branch: Box::from(&b"main"[..]),
        expected,
        identity: 1,
    }
}
pub fn settings() -> crate::Settings {
    crate::Settings {
        git: Box::from(&b"/bin/git"[..]),
        home: Box::from(HOME),
        root: skein_io::kernel::Fd::new(3),
        prefix: 6_u64 << 60_u32,
        serial: 1000,
        largest_read: 64,
        largest_room: 16,
    }
}
#[derive(Debug)]
pub struct Below {
    pub out: crate::Outputs,
    pub io_owner: Option<skein_lib::Token>,
    pub serial: u64,
}
impl Below {
    pub fn new() -> Below {
        Below {
            out: crate::Outputs {
                events: skein_lib::Queue::with_capacity(2),
                io: skein_lib::Queue::with_capacity(20),
                kernel: skein_lib::Queue::with_capacity(2),
            },
            io_owner: None,
            serial: 0,
        }
    }
    pub fn spawned(
        &mut self,
        component: &mut crate::Component,
        now: skein_lib::Time,
    ) -> (skein_lib::Token, Box<[skein_lib::Token]>) {
        let request = self.out.io.pop().expect("spawn requested");
        let (owner, count) = match request {
            skein_io::Request::Spawn { owner, spawn } => {
                assert_eq!(&*spawn.program, b"/bin/git");
                (owner, spawn.pipes.len())
            }
            other @ (skein_io::Request::Listen { .. }
            | skein_io::Request::Connect { .. }
            | skein_io::Request::Bind { .. }
            | skein_io::Request::Reject { .. }
            | skein_io::Request::Stream { .. }
            | skein_io::Request::Output { .. }
            | skein_io::Request::Signal { .. }
            | skein_io::Request::Usage { .. }
            | skein_io::Request::Window { .. }
            | skein_io::Request::Close { .. }
            | skein_io::Request::Abort { .. }) => panic!("expected Spawn, got {other:?}"),
        };
        self.serial = self.serial.checked_add(10).expect("test serial");
        let child = skein_lib::Token::new(self.serial);
        let pipes: Box<[skein_lib::Token]> = if count == 3 {
            Box::new([
                skein_lib::Token::new(self.serial.checked_add(1).expect("test pipe")),
                skein_lib::Token::new(self.serial.checked_add(2).expect("test pipe")),
                skein_lib::Token::new(self.serial.checked_add(3).expect("test pipe")),
            ])
        } else {
            Box::new([
                skein_lib::Token::new(self.serial.checked_add(1).expect("test pipe")),
                skein_lib::Token::new(self.serial.checked_add(2).expect("test pipe")),
            ])
        };
        self.io_owner = Some(owner);
        component.up(now, skein_io::Event::Spawned { owner, child, pipes: pipes.clone() }, &mut self.out);
        (child, pipes)
    }
}
#[derive(Debug)]
pub struct Started {
    pub component: crate::Component,
    pub below: Below,
    pub now: skein_lib::Time,
}
impl Started {
    pub fn unstarted() -> Started {
        Started {
            component: crate::Component::new(&limits(), &checkout(), settings()).expect("valid limits"),
            below: Below::new(),
            now: skein_lib::Time::ZERO,
        }
    }
    pub fn new() -> Started {
        let mut started = Self::unstarted();
        started.component.start(started.now, &mut started.below.out);
        let (child, pipes) = started.spawned();
        started.stdout(pipes[0], b"git version 2.53.0\n");
        started.end(pipes[0]);
        started.end(pipes[1]);
        started.exited(child, 0);
        started.closed(child, &pipes);
        assert_eq!(started.below.out.events.pop(), Some(crate::Event::Started));
        started.component.reclaim();
        started
    }
    pub fn down(&mut self, request: crate::Request) {
        self.component.down(self.now, request, &mut self.below.out);
    }
    pub fn spawned(&mut self) -> (skein_lib::Token, Box<[skein_lib::Token]>) {
        self.below.spawned(&mut self.component, self.now)
    }
    pub fn stdout(&mut self, pipe: skein_lib::Token, bytes: &[u8]) {
        for byte in bytes {
            while self.below.out.io.pop().is_some() {}
            self.component.up(
                self.now,
                skein_io::Event::Stream { owner: pipe, up: skein_lib::stream::Up::Bytes(Box::from([*byte])) },
                &mut self.below.out,
            );
        }
    }
    pub fn end(&mut self, pipe: skein_lib::Token) {
        while self.below.out.io.pop().is_some() {}
        self.component.up(
            self.now,
            skein_io::Event::Stream { owner: pipe, up: skein_lib::stream::Up::End },
            &mut self.below.out,
        );
    }
    pub fn exited(&mut self, _child: skein_lib::Token, code: u8) {
        self.component.up(
            self.now,
            skein_io::Event::Exited {
                owner: self.below.io_owner.expect("spawned"),
                exit: skein_io::kernel::Exit::Code(code),
            },
            &mut self.below.out,
        );
    }
    pub fn closed(&mut self, _child: skein_lib::Token, pipes: &[skein_lib::Token]) {
        while self.below.out.io.pop().is_some() {}
        for pipe in pipes {
            self.component.up(self.now, skein_io::Event::Closed { owner: *pipe }, &mut self.below.out);
        }
        self.component.up(
            self.now,
            skein_io::Event::Closed { owner: self.below.io_owner.expect("spawned") },
            &mut self.below.out,
        );
    }
    pub fn fire(&mut self, now: skein_lib::Time) {
        self.now = now;
        while self.below.out.io.pop().is_some() {}
        self.component.fire(now, &mut self.below.out);
    }
    pub fn signals(&self) -> Box<[(skein_lib::Token, skein_io::kernel::Signal, skein_io::kernel::Target)]> {
        let mut values = skein_lib::List::with_capacity(4);
        for request in &self.below.out.io {
            if let skein_io::Request::Signal { child, signal, to } = request {
                values.push((*child, *signal, *to)).expect("test-sized signals");
            }
        }
        values.into_boxed()
    }
    pub fn done(&mut self) -> Option<(skein_lib::Token, git::Done)> {
        match self.below.out.events.pop() {
            Some(crate::Event::Done { owner, done }) => Some((owner, done)),
            None => None,
            Some(crate::Event::Started | crate::Event::Unusable { .. }) => panic!("unexpected start terminal"),
        }
    }
}
