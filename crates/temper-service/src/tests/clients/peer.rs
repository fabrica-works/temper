use crate::engine::Engine;
use jig_protocol_client_wire::v7 as wire;
use skein_io as io;
use skein_lib::{Decimal, Reader, Time, Token, Wall, Writer};

pub(super) struct Peer {
    wire: skein_world::stream::Wire,
    owner: Option<Token>,
    pub(super) closed: u32,
}
impl Peer {
    pub(super) fn new(engine: &mut Engine, method: &[u8], path: &[u8], body: &[u8], bearer: Option<&[u8]>) -> Self {
        let header = b" HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n";
        let content = b"Content-Length: ";
        let length = Decimal::of(u64::try_from(body.len()).unwrap());
        let authorization = match bearer {
            Some(value) => b"Authorization: Bearer ".len().checked_add(value.len()).unwrap().checked_add(2).unwrap(),
            None => 0,
        };
        let size = method
            .len()
            .checked_add(1)
            .unwrap()
            .checked_add(path.len())
            .unwrap()
            .checked_add(header.len())
            .unwrap()
            .checked_add(content.len())
            .unwrap()
            .checked_add(length.as_bytes().len())
            .unwrap()
            .checked_add(4)
            .unwrap()
            .checked_add(body.len())
            .unwrap()
            .checked_add(authorization)
            .unwrap();
        let mut bytes = Writer::new(size);
        bytes.put(method).unwrap();
        bytes.put(b" ").unwrap();
        bytes.put(path).unwrap();
        bytes.put(b" HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n").unwrap();
        if let Some(bearer) = bearer {
            bytes.put(b"Authorization: Bearer ").unwrap();
            bytes.put(bearer).unwrap();
            bytes.put(b"\r\n").unwrap();
        }
        bytes.put(b"Content-Length: ").unwrap();
        bytes.put(Decimal::of(u64::try_from(body.len()).unwrap()).as_bytes()).unwrap();
        bytes.put(b"\r\n\r\n").unwrap();
        bytes.put(body).unwrap();
        let mut wire = skein_world::stream::Wire::new(7);
        wire.write(&bytes.finish());
        assert!(engine.clients.up(io::Event::Accepted {
            owner: Token::new(crate::tokens::CLIENTS),
            socket: Token::new(100),
            peer: address(),
        }));
        Self { wire, owner: None, closed: 0 }
    }
    pub(super) fn pump(&mut self, engine: &mut Engine, decide: bool) {
        for _ in 0_u32..256 {
            if decide {
                crate::engine::iterate(engine, Time::from_nanos(1), Wall::from_nanos(1));
            } else {
                engine.clients.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.events);
            }
            for _ in 0..engine.clients.io.len() {
                match engine.clients.io.pop().unwrap() {
                    io::Request::Listen { owner, addr } => {
                        assert!(engine.clients.up(io::Event::Listening { owner, listener: Token::new(99), addr }));
                    }
                    io::Request::Bind { socket, owner } => {
                        assert_eq!(socket, Token::new(100));
                        self.owner = Some(owner);
                    }
                    io::Request::Stream { stream, down } => {
                        assert_eq!(stream, Token::new(100));
                        self.wire.take(down);
                    }
                    io::Request::Close { .. } | io::Request::Abort { .. } => {
                        self.closed = self.closed.checked_add(1).unwrap();
                        assert!(engine.clients.up(io::Event::Closed { owner: self.owner.unwrap() }));
                    }
                    io::Request::Connect { .. }
                    | io::Request::Reject { .. }
                    | io::Request::Output { .. }
                    | io::Request::Spawn { .. }
                    | io::Request::Signal { .. }
                    | io::Request::Usage { .. }
                    | io::Request::Window { .. } => panic!("unexpected client io"),
                }
            }
            if self.closed > 0 || self.streaming() {
                break;
            }
            if let Some(owner) = self.owner
                && let Some(up) = self.wire.answer()
            {
                assert!(engine.clients.up(io::Event::Stream { owner, up }));
            }
        }
    }
    pub(super) fn sign_in_answer(&self, engine: &Engine) -> wire::SignInAnswer {
        let at = skein_lib::bytes::find(&self.wire.received, b"\r\n\r\n").unwrap().checked_add(4).unwrap();
        wire::SignInAnswer::decode(
            &engine.clients.env.limits.wire,
            &mut Reader::new(self.wire.received.get(at..).unwrap()),
        )
        .unwrap()
    }
    pub(super) fn streaming(&self) -> bool {
        skein_lib::bytes::find(&self.wire.received, b"text/event-stream").is_some()
    }
}
pub(super) fn address() -> io::Addr {
    io::Addr::Ip((::core::net::Ipv4Addr::LOCALHOST, 7100).into())
}
pub(super) fn link_body(engine: &Engine, link: &[u8]) -> Box<[u8]> {
    use skein_lib::base64::{self, Alphabet};
    let at = skein_lib::bytes::find(link, b"secret=").unwrap().checked_add(7).unwrap();
    let secret = base64::decode(Alphabet::Url, link.get(at..).unwrap(), 32).unwrap();
    let record = wire::LinkSignIn::new(&engine.clients.env.limits.wire, wire::LinkSignInParts { secret }).unwrap();
    let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut bytes).unwrap();
    bytes.finish()
}

pub(super) fn bearer(value: &[u8]) -> Box<[u8]> {
    use skein_lib::base64::{self, Alphabet};
    let mut bytes = Writer::new(
        usize::try_from(base64::encoded_len(Alphabet::Url, u32::try_from(value.len()).unwrap()).unwrap()).unwrap(),
    );
    base64::encode(Alphabet::Url, value, &mut bytes).unwrap();
    bytes.finish()
}
