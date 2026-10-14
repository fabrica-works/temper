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
    pub peers: skein_lib::Map<skein_lib::Token, Peer>,
    pub opened: u32,
    pub stopped: Option<(u16, crate::Stopped)>,
    pub closed: bool,
}
#[derive(Debug)]
pub struct Peer {
    forge: u16,
    socket: skein_lib::Token,
    wire: skein_world::stream::Wire,
    connected: bool,
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
            peers: skein_lib::Map::with_capacity(32),
            opened: 0,
            stopped: None,
            closed: false,
        };
        h.component.begin(&h.env, &mut h.up, &mut h.io);
        h.drain();
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
        self.drain();
        self.component.reclaim();
        for _ in 0..self.up.len() {
            match self.up.pop().expect("queued event") {
                Event::Answered(answer) => {
                    assert!(self.answered(answer.call.raw()).is_none(), "one terminal per call");
                    drop(self.answers.insert(answer.call.raw(), answer).expect("fixture answers fit"));
                }
                Event::Stopped { forge, why } => {
                    assert!(self.stopped.is_none(), "start stopped once");
                    self.stopped = Some((forge, why));
                }
                Event::Closed => {
                    assert!(!self.closed, "closed once");
                    self.closed = true;
                }
            }
        }
    }
    pub fn answered(&self, call: u64) -> Option<&Answered> {
        self.answers.get(&call)
    }
    pub fn start(&self, forge: u16) -> Start {
        self.component.start(forge).expect("fixture forge configured")
    }
    fn newest(&self, forge: u16) -> skein_lib::Token {
        let mut found = None;
        for (owner, peer) in &self.peers {
            if peer.forge == forge {
                found = Some(*owner);
            }
        }
        found.expect("forge has a connection")
    }
    fn owners(&self) -> List<skein_lib::Token> {
        let mut owners = List::with_capacity(32);
        for (owner, _) in &self.peers {
            owners.push(*owner).expect("fixture peers fit");
        }
        owners
    }
    fn event(&mut self, event: skein_io::Event) {
        self.component.up(&self.env, event, &mut self.up, &mut self.io);
    }
    pub fn connected(&mut self, forge: u16) {
        let owner = self.newest(forge);
        self.peers.get_mut(&owner).expect("registered connect").connected = true;
        self.event(skein_io::Event::Connected { owner });
        self.drain();
    }
    pub fn refuse(&mut self, forge: u16) {
        let owner = self.newest(forge);
        self.event(skein_io::Event::Failed { owner, error: skein_io::Error::Refused });
        self.event(skein_io::Event::Closed { owner });
        drop(self.peers.remove(&owner));
        self.drain();
    }
    pub fn answer(&mut self, forge: u16, bytes: &[u8]) {
        let owner = self.newest(forge);
        self.peers.get_mut(&owner).expect("registered connect").wire.write(bytes);
        self.drain();
    }
    pub fn reset(&mut self, forge: u16) {
        let owner = self.newest(forge);
        self.event(skein_io::Event::Stream {
            owner,
            up: skein_lib::stream::Up::Failed(skein_lib::stream::Fault::Reset),
        });
        self.drain();
    }
    pub fn request_lines(&self, forge: u16) -> List<Box<[u8]>> {
        let mut lines = List::with_capacity(16);
        for (_, peer) in &self.peers {
            if peer.forge != forge {
                continue;
            }
            let mut start = 0;
            for (at, byte) in peer.wire.received.iter().enumerate() {
                if *byte == b'\n' {
                    let line = peer.wire.received.get(start..at).expect("line positions");
                    if line.starts_with(b"GET ") {
                        lines.push(Box::from(line.strip_suffix(b"\r").unwrap_or(line))).expect("fixture lines fit");
                    }
                    start = at.saturating_add(1);
                }
            }
        }
        lines
    }
    pub fn connections_opened(&self) -> u32 {
        self.opened
    }
    pub fn stopped(&self) -> Option<(u16, crate::Stopped)> {
        self.stopped
    }
    pub fn ready() -> Self {
        let mut h = Self::new(settings());
        h.connected(FORGE);
        h.answer(FORGE, &user_ok());
        h.answer(FORGE, &settings_ok());
        h
    }
    fn drain(&mut self) {
        for _ in 0..1024_u32 {
            for _ in 0..self.io.len() {
                match self.io.pop().expect("queued request") {
                    skein_io::Request::Connect { owner, .. } => {
                        self.opened = self.opened.saturating_add(1);
                        let socket = skein_lib::Token::new(owner.raw().saturating_add(100));
                        drop(
                            self.peers
                                .insert(
                                    owner,
                                    Peer {
                                        forge: FORGE,
                                        socket,
                                        wire: skein_world::stream::Wire::new(owner.raw()),
                                        connected: false,
                                    },
                                )
                                .expect("fixture peers fit"),
                        );
                        self.event(skein_io::Event::Connecting { owner, socket });
                    }
                    skein_io::Request::Stream { stream, down } => {
                        for owner in self.owners().into_boxed() {
                            let peer = self.peers.get_mut(&owner).expect("fixture peer");
                            if peer.socket == stream {
                                peer.wire.take(down);
                                break;
                            }
                        }
                    }
                    skein_io::Request::Close { entity } | skein_io::Request::Abort { entity } => {
                        let mut owner = None;
                        for (token, peer) in &self.peers {
                            if peer.socket == entity {
                                owner = Some(*token);
                                break;
                            }
                        }
                        if let Some(owner) = owner {
                            self.event(skein_io::Event::Closed { owner });
                            drop(self.peers.remove(&owner));
                        }
                    }
                    skein_io::Request::Listen { .. }
                    | skein_io::Request::Bind { .. }
                    | skein_io::Request::Reject { .. }
                    | skein_io::Request::Output { .. }
                    | skein_io::Request::Spawn { .. }
                    | skein_io::Request::Signal { .. }
                    | skein_io::Request::Usage { .. }
                    | skein_io::Request::Window { .. } => unreachable!("forge only uses sockets"),
                }
            }
            let mut events = List::with_capacity(32);
            for owner in self.owners().into_boxed() {
                let peer = self.peers.get_mut(&owner).expect("fixture peer");
                if peer.connected
                    && let Some(up) = peer.wire.answer()
                {
                    events.push(skein_io::Event::Stream { owner, up }).expect("fixture events fit");
                }
            }
            for event in events.into_boxed() {
                self.event(event);
            }
        }
    }
}
pub fn ok(body: &[u8]) -> Box<[u8]> {
    let decimal = skein_lib::Decimal::of(u64::try_from(body.len()).expect("body length"));
    let mut out = skein_lib::Writer::new(
        body.len()
            .saturating_add(b"HTTP/1.1 200 OK\r\nContent-Length: ".len())
            .saturating_add(decimal.as_bytes().len())
            .saturating_add(b"\r\nDate: Wed, 01 Jan 2020 00:00:00 GMT\r\n\r\n".len()),
    );
    out.put(b"HTTP/1.1 200 OK\r\nContent-Length: ").expect("head fits");
    out.put(decimal.as_bytes()).expect("length fits");
    out.put(b"\r\nDate: Wed, 01 Jan 2020 00:00:00 GMT\r\n\r\n").expect("date fits");
    out.put(body).expect("body fits");
    out.finish()
}
pub fn status(code: u16) -> Box<[u8]> {
    let decimal = skein_lib::Decimal::of(u64::from(code));
    let mut out = skein_lib::Writer::new(
        b"HTTP/1.1 "
            .len()
            .saturating_add(decimal.as_bytes().len())
            .saturating_add(b" Refused\r\nContent-Length: 0\r\n\r\n".len()),
    );
    out.put(b"HTTP/1.1 ").expect("head fits");
    out.put(skein_lib::Decimal::of(u64::from(code)).as_bytes()).expect("code fits");
    out.put(b" Refused\r\nContent-Length: 0\r\n\r\n").expect("head fits");
    out.finish()
}
pub fn user_ok() -> Box<[u8]> {
    ok(b"{\"id\":2,\"login\":\"writer\"}")
}
pub fn settings_ok() -> Box<[u8]> {
    ok(b"{\"max_response_items\":2,\"default_paging_num\":30}")
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

/// One connection's scripted stream, shared by its step tests.
#[derive(Debug)]
pub struct ConnectionHarness {
    pub connection: crate::connection::Connection,
    pub env: Env<Limits>,
    pub forge: Forge,
    pub io: Queue<skein_io::Request>,
    pub sent: Box<[u8]>,
}
impl ConnectionHarness {
    pub fn new() -> Self {
        let limits = limits();
        let forge = settings().forges.into_boxed().into_iter().next().expect("one forge");
        let mut connection =
            crate::connection::Connection::new(&forge, &limits, shared(), Time::from_nanos(10_000_000_000));
        connection.socket = Some(skein_lib::Token::new(100));
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        connection.connect(&env);
        Self { connection, env, forge, io: Queue::with_capacity(32), sent: Box::from([]) }
    }
    pub fn pump(&mut self) {
        self.connection.pump(&self.env, &mut self.io, 16);
    }
    pub fn answer(&mut self, response: &[u8]) -> List<crate::connection::Heard> {
        let mut wire = skein_world::stream::Wire::new(1);
        let mut written = false;
        let mut heard = List::with_capacity(16);
        for _ in 0_u32..1024 {
            self.pump();
            for _ in 0..self.io.len() {
                let request = self.io.pop().expect("queued io request");
                match request {
                    skein_io::Request::Stream { down, .. } => wire.take(down),
                    skein_io::Request::Listen { .. }
                    | skein_io::Request::Connect { .. }
                    | skein_io::Request::Bind { .. }
                    | skein_io::Request::Reject { .. }
                    | skein_io::Request::Output { .. }
                    | skein_io::Request::Spawn { .. }
                    | skein_io::Request::Signal { .. }
                    | skein_io::Request::Usage { .. }
                    | skein_io::Request::Window { .. }
                    | skein_io::Request::Close { .. }
                    | skein_io::Request::Abort { .. } => unreachable!("one connection emits stream requests"),
                }
            }
            if !written && self.connection.phase == crate::connection::Phase::Reading {
                wire.write(response);
                written = true;
            }
            if let Some(event) = wire.answer() {
                self.connection.stream(&self.env, event);
            }
            if let Some(event) = self.connection.heard() {
                heard.push(event).expect("fixture events fit");
            }
        }
        self.sent = Box::from(wire.received.as_slice());
        heard
    }
}
