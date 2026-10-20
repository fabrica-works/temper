use skein_io as io;
use skein_lib::{Time, Token, Wall};

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
    pub fn sent(&self) -> &[u8] {
        &self.wire.received
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
    pub fn returned(service: &mut crate::Service, answer: skein_forge_connection::Answered) {
        let temper_engine_domain::Event::Forge(event) = crate::forge::event(answer) else {
            panic!("forge event");
        };
        crate::route::request(&mut service.engine, temper_engine_domain::Request::Busy(event));
    }
}
/// Install a real listener connection and return its first verified hint.
pub fn hook(service: &mut crate::Service) -> skein_forge_connection::hooks::Hinted {
    use skein_lib::{Secret, Writer};
    const SECRET: &[u8] = b"fixture-hook-secret";
    const BODY: &[u8] = b"{\"repository\":{\"id\":2},\"number\":7}";
    let config = super::engine();
    let mut found = jig_protocol_config::Problems::empty();
    let mut settings =
        crate::forge::settings(&config.forges, super::forge_runtime(), None, &mut found).expect("settings");
    settings.forges.get_mut(0).expect("forge").access.as_mut().expect("access").webhook_secret =
        Some(Secret::new(Box::from(SECRET)));
    settings.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: io::Addr::Ip(([127, 0, 0, 1], 7200).into()) });
    let layer = &mut service.engine.forge;
    layer.component = skein_forge_connection::Component::new(
        settings,
        &layer.env.limits,
        &crate::forge::shared(&config.limits, &config.jig.limits.io),
    )
    .expect("component");
    while layer.io.pop().is_some() {}
    layer.component.begin(&layer.env, &mut layer.above, &mut layer.io);
    let listener = Token::new(1_u64 << 59_u32);
    let mut owner = None;
    let mut wire = skein_world::stream::Wire::new(7);
    let mut mac = skein_lib::sha256::Hmac::new(SECRET);
    mac.update(BODY);
    let mut head = Writer::new(180);
    head.put(b"POST /hooks/1 HTTP/1.1\r\nHost: loopback\r\nContent-Length: 34\r\nX-Forgejo-Event: pull_request\r\nX-Forgejo-Signature: ").expect("head");
    for byte in mac.finish() {
        for nibble in [byte / 16, byte % 16] {
            head.put(&[b"0123456789abcdef"[usize::from(nibble)]]).expect("signature");
        }
    }
    head.put(b"\r\n\r\n").expect("head");
    wire.write(&head.finish());
    wire.write(BODY);
    layer.up(io::Event::Listening {
        owner: listener,
        listener: Token::new(88),
        addr: io::Addr::Ip(([127, 0, 0, 1], 7200).into()),
    });
    layer.up(io::Event::Accepted {
        owner: listener,
        socket: Token::new(89),
        peer: io::Addr::Ip(([127, 0, 0, 1], 9000).into()),
    });
    for _ in 0_u32..2048 {
        while let Some(request) = layer.io.pop() {
            match request {
                io::Request::Bind { owner: bound, .. } => owner = Some(bound),
                io::Request::Stream { stream, down } if stream == Token::new(89) => wire.take(down),
                io::Request::Listen { .. } | io::Request::Connect { .. } | io::Request::Stream { .. } => {}
                io::Request::Close { .. }
                | io::Request::Abort { .. }
                | io::Request::Reject { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("hook delivery only"),
            }
        }
        if let Some(up) = wire.answer() {
            layer.up(io::Event::Stream { owner: owner.expect("bound hook"), up });
        }
        if let Some(skein_forge_connection::Event::Hint(hint)) = layer.above.pop() {
            return hint;
        }
    }
    panic!("verified hint");
}
