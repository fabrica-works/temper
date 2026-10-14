//! The component's small limits and configured forge.
use crate::Limits;
use skein_lib::Duration;
pub const WRITER: u64 = 2;
pub fn limits() -> Limits {
    Limits {
        forges: 2,
        token_bytes: 64,
        connections: 2,
        head_bytes: 4096,
        headers: 32,
        body_bytes_in: 1_048_576,
        depth: 16,
        chunk: 1024,
        string_bytes: 128,
        body_bytes: 1045,
        inner: 8,
        target_bytes: 1024,
        repositories: 2,
        users: 8,
        deadline: Duration::from_secs(10),
        idle: Duration::from_secs(5),
        start_backoff: Duration::from_secs(1),
        start_backoff_max: Duration::from_secs(8),
        tls: skein_tls::client::Limits { read: 16384, send: 16384, records: skein_tls::client::MAX_RECORD },
    }
}

use crate::{
    Answered, Component, Event, Settings, Shared,
    settings::{Forge, Kind, Transport},
    start::Start,
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, SocketAddr};
use skein_lib::{Env, List, Queue, Secret, Time, Wall};
use temper_engine_domain_forge_client::{self as client, api};
pub const FORGE: u16 = 1;
pub const REPO: api::Repository = api::Repository { forge: 1, repository: 1 };
pub const TOKEN: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
pub fn shared() -> Shared {
    Shared {
        client: client::Limits {
            pending: 4,
            calls: 2,
            rate: 100,
            reserve: 0,
            window: Duration::from_secs(10),
            op_bytes: 512,
            answer_bytes: 65_536,
            rows: 8,
            inbox: 16,
            read_attempts: 3,
            backoff: Duration::from_secs(1),
            backoff_max: Duration::from_secs(4),
            entries: 8,
            write_attempts: 3,
            lifetime: Duration::from_secs(10),
            clock_margin: Duration::from_secs(1),
            resources: 8,
            repositories: 2,
            poll: Duration::from_secs(5),
            poll_max: Duration::from_secs(30),
            hinted: Duration::from_secs(1),
            slow: Duration::from_secs(40),
            facts: 16,
        },
        name_bytes: 128,
        repositories: 2,
        io: skein_io::Limits {
            sockets: 8,
            names: 0,
            refusals: 2,
            intake: 32_768,
            receive: 4096,
            output: 32_768,
            sends: 4,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(1),
            retry: Duration::from_millis(10),
        },
    }
}
pub fn settings() -> Settings {
    let mut forges = List::with_capacity(2);
    forges
        .push(Forge {
            number: FORGE,
            kind: Kind::Forgejo,
            address: skein_io::Addr::Ip(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))),
            transport: Transport::Plaintext,
            host: Box::from(&b"127.0.0.1:3000"[..]),
            token: Secret::new(Box::from(TOKEN)),
            user: WRITER,
        })
        .expect("one forge");
    Settings { forges }
}
#[derive(Debug)]
pub struct Harness {
    pub component: Component,
    pub env: Env<Limits>,
    pub up: Queue<Event>,
    pub io: Queue<skein_io::Request>,
    pub answers: skein_lib::Map<u64, Answered>,
}
impl Harness {
    pub fn new(settings: Settings) -> Self {
        Self::with(limits(), shared(), settings)
    }
    pub fn with(limits: Limits, shared: Shared, settings: Settings) -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut h = Self {
            component: Component::new(settings, &env.limits, &shared).expect("fixture limits agree"),
            env,
            up: Queue::with_capacity(32),
            io: Queue::with_capacity(32),
            answers: skein_lib::Map::with_capacity(32),
        };
        h.component.begin(&h.env, &mut h.up, &mut h.io);
        h
    }
    pub fn call(&mut self, call: u64, repository: api::Repository, op: api::Op) {
        self.send(crate::Request::Call { call: skein_lib::Token::new(call), repository, op });
    }
    pub fn send(&mut self, request: crate::Request) {
        self.component.down(&self.env, request, &mut self.up, &mut self.io);
    }
    pub fn run(&mut self, span: Duration) {
        self.env.now = self.env.now.saturating_add(span);
        self.component.fire(&self.env, &mut self.up, &mut self.io);
        self.component.reclaim();
        for _ in 0..self.up.len() {
            match self.up.pop().expect("queued event") {
                Event::Answered(answer) => {
                    assert!(self.answered(answer.call.raw()).is_none(), "one terminal per call");
                    drop(self.answers.insert(answer.call.raw(), answer).expect("fixture answers fit"));
                }
                Event::Stopped { .. } | Event::Closed => unreachable!("run collects call answers only"),
            }
        }
    }
    pub fn answered(&self, call: u64) -> Option<&Answered> {
        self.answers.get(&call)
    }
    pub fn start(&self, forge: u16) -> Start {
        self.component.start(forge).expect("fixture forge configured")
    }
}
pub fn specimen() -> crate::tables::Repository {
    crate::tables::Repository {
        owner: Box::from(&b"fixture"[..]),
        name: Box::from(&b"specimen"[..]),
        object_format: temper_forge_forgejo::id::ObjectFormat::Sha1,
        merge_style: temper_forge_forgejo::document::MergeStyle::Squash,
        html_url: Box::from(&b"http://127.0.0.1:3000/fixture/specimen"[..]),
    }
}
