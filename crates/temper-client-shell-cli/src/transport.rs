//! Native HTTP connections on the host's io, with volatile records.
use jig_client as client;
use jig_client_shell_native::{Conn, Connections};
use skein_io::{self as io, Addr};
use skein_lib::{
    Queue, Time, Token,
    stream::{Down, Fault, Up},
};
use std::collections::{BTreeMap, VecDeque};
const OWNER: u64 = 1 << 60;
struct Socket {
    entity: Option<Token>,
    connected: bool,
    below: VecDeque<Down>,
}
pub(crate) struct Memory {
    connections: Connections,
    engine: Addr,
    sockets: BTreeMap<Conn, Socket>,
    below: Queue<(Conn, Down)>,
    pub(crate) events: Queue<client::Event>,
    closing: bool,
    max: u32,
}
impl Memory {
    pub(crate) fn new(engine: Addr, limits: &jig_client_shell_native::Limits) -> Self {
        let max = limits.exchanges.checked_add(1).expect("connections fit");
        Self {
            connections: Connections::new(limits, &engine.socket().expect("numeric engine").to_string()),
            engine,
            sockets: BTreeMap::new(),
            below: Queue::with_capacity(max * 64),
            events: Queue::with_capacity(max * 64),
            closing: false,
            max,
        }
    }
    pub(crate) fn down(&mut self, now: Time, request: client::Request) {
        if matches!(request, client::Request::Save { .. } | client::Request::Erase { .. }) {
            self.events.push(client::Event::Stored);
        }
        self.connections.down(now, request, &mut self.below);
    }
    pub(crate) fn owns(owner: Token) -> bool {
        owner.raw() & OWNER != 0
    }
    pub(crate) fn event(&mut self, now: Time, event: io::Event) {
        match event {
            io::Event::Connecting { owner, socket } => {
                self.sockets.get_mut(&Self::conn(owner)).expect("connecting socket").entity = Some(socket);
            }
            io::Event::Connected { owner } => {
                self.sockets.get_mut(&Self::conn(owner)).expect("connected socket").connected = true;
            }
            io::Event::Stream { .. } | io::Event::Failed { .. } if self.closing => {}
            io::Event::Stream { owner, up } => {
                self.connections.up(now, Self::conn(owner), up, &mut self.events, &mut self.below);
            }
            io::Event::Failed { owner, .. } => {
                self.connections.up(
                    now,
                    Self::conn(owner),
                    Up::Failed(Fault::Reset),
                    &mut self.events,
                    &mut self.below,
                );
            }
            io::Event::Closed { owner } => {
                self.sockets.remove(&Self::conn(owner));
            }
            io::Event::Output { .. }
            | io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Spawned { .. }
            | io::Event::Exited { .. }
            | io::Event::Usage { .. }
            | io::Event::Resized
            | io::Event::Window { .. }
            | io::Event::Shutdown { .. } => panic!("memory owns sockets"),
        }
    }
    fn conn(owner: Token) -> Conn {
        Conn(u32::try_from(owner.raw() & !OWNER).expect("connection name"))
    }
    pub(crate) fn close(&mut self) {
        self.closing = true;
        while self.events.pop().is_some() {}
    }
    pub(crate) fn progress(&mut self, now: Time) -> Vec<io::Request> {
        let mut requests = Vec::new();
        if self.closing {
            while self.below.pop().is_some() {}
            for socket in self.sockets.values_mut() {
                socket.below.clear();
                if let Some(entity) = socket.entity.take() {
                    requests.push(io::Request::Abort { entity });
                }
            }
            return requests;
        }
        self.connections.take(now, &mut self.events, &mut self.below);
        while let Some((conn, down)) = self.below.pop() {
            if !self.sockets.contains_key(&conn) {
                if matches!(down, Down::Finish) {
                    continue;
                }
                assert!(self.sockets.len() < usize::try_from(self.max).expect("socket bound"), "bounded sockets");
                self.sockets.insert(conn, Socket { entity: None, connected: false, below: VecDeque::new() });
                requests.push(io::Request::Connect { owner: Token::new(OWNER | u64::from(conn.0)), addr: self.engine });
            }
            let queue = &mut self.sockets.get_mut(&conn).expect("socket").below;
            assert!(queue.len() < 64, "bounded socket work");
            queue.push_back(down);
        }
        for socket in self.sockets.values_mut().filter(|socket| socket.connected) {
            while let Some(down) = socket.below.pop_front() {
                let stream = socket.entity.expect("socket entity");
                requests.push(match down {
                    Down::Finish => io::Request::Close { entity: stream },
                    down @ (Down::Demand { .. } | Down::Send(_)) => io::Request::Stream { stream, down },
                });
            }
        }
        requests
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        if self.closing { None } else { self.connections.next_deadline() }
    }
    pub(crate) fn pending(&self) -> bool {
        !self.events.is_empty()
            || !self.below.is_empty()
            || self.sockets.values().any(|s| s.connected && !s.below.is_empty() || self.closing && s.entity.is_some())
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.sockets.is_empty()
    }
}
