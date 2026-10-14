//! HTTP and git share one forge domain; factories take its handle from this world.
use skein_io::{
    self as io,
    kernel::{Complete, Submit},
};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_shell::Host;
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use temper_fake_forge_domain as domain;
use temper_fake_forge_protocol as protocol;
type Shared = Rc<RefCell<temper_fake_git::Forge>>;
thread_local! { static SHARED: RefCell<Option<temper_fake_git::Transport>> = const { RefCell::new(None) }; }
pub(crate) fn set(transport: Option<temper_fake_git::Transport>) {
    SHARED.with(|shared| {
        *shared.borrow_mut() = transport;
    });
}
pub(crate) fn transport() -> temper_fake_git::Transport {
    SHARED.with(|shared| shared.borrow().as_ref().expect("world's forge").clone())
}
struct Connection {
    socket: Token,
    server: protocol::Server,
    above: Queue<protocol::Event>,
    below: Queue<skein_lib::stream::Down>,
}
pub(crate) struct ForgeHost {
    shared: Shared,
    service: protocol::Service,
    io: io::Io,
    limits: io::Limits,
    protocol: protocol::Limits,
    events: Queue<io::Event>,
    requests: Queue<io::Request>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    connections: BTreeMap<Token, Connection>,
    listener: Option<Token>,
    serial: u64,
    stopping: bool,
}
impl ForgeHost {
    pub(crate) fn new() -> Self {
        let limits = io::Limits {
            sockets: 16,
            names: 0,
            refusals: 8,
            intake: 65_536,
            receive: 1024,
            output: 65_536,
            sends: 8,
            accepts: 4,
            backlog: 8,
            close_timeout: skein_lib::Duration::from_secs(1),
            retry: skein_lib::Duration::from_millis(1),
        };
        let protocol = temper_engine_protocol_forge_world::fixtures::fake_limits();
        let service = protocol::Service::new(
            temper_fake_git::fixtures::table(b"acme/app", None),
            &protocol,
            crate::fixtures::SEED,
        )
        .expect("protocol tables");
        let mut requests = Queue::with_capacity(256);
        requests.push(io::Request::Listen {
            owner: Token::new(u64::MAX),
            addr: io::Addr::Ip((std::net::Ipv4Addr::LOCALHOST, 3000).into()),
        });
        Self {
            shared: transport().forge,
            service,
            io: io::Io::new(&limits),
            limits,
            protocol,
            events: Queue::with_capacity(256),
            requests,
            completions: Queue::with_capacity(256),
            submissions: Queue::with_capacity(256),
            connections: BTreeMap::new(),
            listener: None,
            serial: 0,
            stopping: false,
        }
    }
    pub(crate) fn shutdown(&mut self) {
        self.stopping = true;
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "a server listens and receives streams; other events fail the world"
    )]
    fn event(&mut self, event: io::Event, env: &Env<protocol::Limits>) {
        match event {
            io::Event::Listening { listener, .. } => self.listener = Some(listener),
            io::Event::Accepted { socket, .. } => {
                if self.stopping {
                    self.requests.push(io::Request::Reject { socket });
                    return;
                }
                self.serial += 1;
                let owner = Token::new(self.serial);
                self.requests.push(io::Request::Bind { socket, owner });
                let mut connection = Connection {
                    socket,
                    server: protocol::Server::new(owner, &self.protocol).expect("server"),
                    above: Queue::with_capacity(32),
                    below: Queue::with_capacity(32),
                };
                protocol::start(
                    &mut connection.server,
                    &mut self.service,
                    env,
                    &mut connection.above,
                    &mut connection.below,
                );
                self.connections.insert(owner, connection);
            }
            io::Event::Stream { owner, up } => {
                if let Some(connection) = self.connections.get_mut(&owner) {
                    protocol::up(
                        &mut connection.server,
                        &mut self.service,
                        env,
                        up,
                        &mut connection.above,
                        &mut connection.below,
                    );
                }
            }
            io::Event::Closed { owner } => {
                if let Some(mut connection) = self.connections.remove(&owner) {
                    protocol::closed(
                        &mut connection.server,
                        &mut self.service,
                        env,
                        &mut connection.above,
                        &mut connection.below,
                    );
                }
            }
            other => panic!("forge io event {other:?}"),
        }
    }
    fn progress(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.protocol };
        let mut forge = self.shared.borrow_mut();
        forge.now = now;
        let domain_env = Env { now, wall, limits: forge.config };
        let mut replies = Queue::with_capacity(domain::MAX_OUT);
        for connection in self.connections.values_mut() {
            if self.stopping {
                protocol::close(
                    &mut connection.server,
                    &mut self.service,
                    &env,
                    &mut connection.above,
                    &mut connection.below,
                );
            }
            protocol::resume(
                &mut connection.server,
                &mut self.service,
                &env,
                &mut connection.above,
                &mut connection.below,
            );
            while let Some(event) = connection.above.pop() {
                match event {
                    protocol::Event::Domain(event) => {
                        domain::step(&mut forge.domain, &domain_env, event, &mut replies);
                        while let Some(reply) = replies.pop() {
                            forge.pending.push(reply);
                        }
                    }
                    protocol::Event::Close => self.requests.push(io::Request::Close { entity: connection.socket }),
                    protocol::Event::Closed => {}
                }
            }
            while let Some(down) = connection.below.pop() {
                self.requests.push(io::Request::Stream { stream: connection.socket, down });
            }
        }
        domain::fire(&mut forge.domain, &domain_env, &mut replies);
        while let Some(reply) = replies.pop() {
            forge.pending.push(reply);
        }
        for reply in std::mem::take(&mut forge.pending) {
            if let Ok((owner, reply)) = self.service.target(reply) {
                let connection = self.connections.get_mut(&owner).expect("live HTTP call");
                protocol::down(
                    &mut connection.server,
                    &mut self.service,
                    &env,
                    reply,
                    &mut connection.above,
                    &mut connection.below,
                );
            }
        }
        forge.domain.reclaim();
        self.service.reclaim();
        if self.stopping
            && let Some(listener) = self.listener.take()
        {
            self.requests.push(io::Request::Close { entity: listener });
        }
    }
}
impl Host for ForgeHost {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits };
        while let Some(complete) = self.completions.pop() {
            io::up(&mut self.io, &env, complete, &mut self.events, &mut self.submissions);
        }
        for _ in 0..64 {
            if !self.io.is_ready() {
                break;
            }
            io::resume(&mut self.io, &env, &mut self.events, &mut self.submissions);
        }
        if self.io.is_due(now) {
            io::fire(&mut self.io, &env, &mut self.events, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            self.event(event, &Env { now, wall, limits: self.protocol });
        }
        self.progress(now, wall);
        while self.io.takes() {
            let Some(request) = self.requests.pop() else {
                break;
            };
            io::down(&mut self.io, &env, request, &mut self.submissions);
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        self.io.is_ready()
            || !self.completions.is_empty()
            || !self.requests.is_empty()
            || self.connections.values().any(|c| c.server.has_work() || !c.above.is_empty() || !c.below.is_empty())
            || (self.stopping && self.listener.is_some())
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.io.next_deadline(), self.shared.borrow().domain.next_deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.io.is_empty()
            && self.connections.is_empty()
            && self.requests.is_empty()
            && self.submissions.is_empty()
            && self.completions.is_empty()
    }
    fn worst_case(&self) -> u64 {
        128 * 1024 * 1024
    }
    fn operations(&self) -> u32 {
        256
    }
}
