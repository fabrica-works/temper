use skein_io as io;
use skein_lib::{Time, Token, Wall};
use temper_engine_protocol_forge as kit;
/// A forge's component face, with each connection's demand answered through Wire.
pub struct ForgeFace {
    owner: Option<Token>,
    wire: skein_world::stream::Wire,
    now: Time,
    worker_owner: Option<Token>,
}
impl ForgeFace {
    pub fn new() -> Self {
        Self { owner: None, wire: skein_world::stream::Wire::new(7), now: Time::from_nanos(1), worker_owner: None }
    }
    pub fn connected(&mut self, service: &mut crate::Service) {
        for _ in 0_u32..256 {
            self.turn(service);
            if self.owner.is_some() && self.wire.received.ends_with(b"\r\n\r\n") {
                return;
            }
        }
        panic!("start request sent");
    }
    pub fn answer(&mut self, service: &mut crate::Service, bytes: &[u8]) {
        self.wire.write(bytes);
        for _ in 0_u32..512 {
            self.turn(service);
        }
    }
    pub fn refuse(&mut self, service: &mut crate::Service) {
        if let Some(owner) = self.owner {
            service.engine.forge.up(io::Event::Failed { owner, error: io::Error::Refused });
        }
    }
    pub fn at(&mut self, service: &mut crate::Service, now: Time) {
        self.now = now;
        self.turn(service);
    }
    pub fn turn(&mut self, service: &mut crate::Service) {
        let engine = &mut service.engine;
        for _ in 0_u32..engine.clients.io.len() {
            match engine.clients.io.pop().unwrap() {
                io::Request::Listen { owner, addr } => {
                    assert!(engine.clients.up(io::Event::Listening { owner, listener: Token::new(77), addr }));
                }
                io::Request::Close { .. } | io::Request::Abort { .. } => {
                    assert!(engine.clients.up(io::Event::Closed { owner: Token::new(crate::tokens::CLIENTS) }));
                }
                io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("unexpected client listener request"),
            }
        }
        for _ in 0_u32..engine.workers.io.len() {
            match engine.workers.io.pop().unwrap() {
                io::Request::Listen { owner, addr } => {
                    self.worker_owner = Some(owner);
                    assert!(engine.workers.up(
                        self.now,
                        io::Event::Listening { owner, listener: Token::new(78), addr },
                        &mut engine.events
                    ));
                }
                io::Request::Close { .. } | io::Request::Abort { .. } => {
                    assert!(engine.workers.up(
                        self.now,
                        io::Event::Closed { owner: self.worker_owner.unwrap() },
                        &mut engine.events
                    ));
                }
                io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("unexpected worker listener request"),
            }
        }
        crate::engine::iterate(engine, self.now, Wall::EPOCH);
        for _ in 0_u32..engine.forge.io.len() {
            match engine.forge.io.pop().unwrap() {
                io::Request::Connect { owner, .. } => {
                    self.owner = Some(owner);
                    self.wire = skein_world::stream::Wire::new(7);
                    engine.forge.up(io::Event::Connecting { owner, socket: Token::new(99) });
                    engine.forge.up(io::Event::Connected { owner });
                }
                io::Request::Stream { down, .. } => self.wire.take(down),
                io::Request::Close { .. } | io::Request::Abort { .. } => {
                    engine.forge.up(io::Event::Closed { owner: self.owner.take().unwrap() });
                }
                io::Request::Listen { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("unexpected forge request"),
            }
        }
        if let Some(up) = self.wire.answer()
            && let Some(owner) = self.owner
        {
            engine.forge.up(io::Event::Stream { owner, up });
        }
    }
    pub fn pending(service: &crate::Service) -> u32 {
        service.engine.forge.above.len()
    }
    pub fn returned(service: &mut crate::Service, answer: kit::Answered) {
        let temper_engine_domain::Event::Forge(event) = crate::forge::event(answer) else {
            panic!("forge event");
        };
        crate::route::request(&mut service.engine, temper_engine_domain::Request::Busy(event));
    }
}
