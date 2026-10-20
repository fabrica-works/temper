use super::fixtures;
use jig_client_protocol_wire::v10 as wire;
use jig_protocol_engine_clients as kit;
use skein_lib::{Env, List, Queue, Reader, Secret, Time, Token, Wall, Writer};

const STREAM: Token = Token::new(101);
const OPEN: Token = Token::new(102);
const CLOSE: Token = Token::new(103);
const WATCH: u64 = 17;
const ITERATIONS: u8 = 64;
const SIGN_IN: u64 = 41;
const EXPIRY: Wall = Wall::from_nanos(86_400_000_000_000);

struct Stand {
    component: kit::Component,
    env: Env<kit::Limits>,
    out: Queue<kit::Request>,
    credential: Secret,
}
impl Stand {
    fn new() -> Self {
        let config = fixtures::engine().jig;
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config.limits.clients };
        let mut served = List::with_capacity(1);
        served
            .push(kit::Served {
                family: None,
                first: jig_client_protocol_wire::VERSION,
                last: jig_client_protocol_wire::VERSION,
            })
            .unwrap();
        let settings = kit::Settings {
            deployment_provider: 2,
            link: None,
            heartbeat: config.clients.heartbeat,
            rows: 32,
            served,
            providers: List::with_capacity(0),
            seed: Secret::new(Box::new([7; 32])),
            stream: 31,
            models: jig_protocol_config::clients::models(&config.llm),
        };
        let mut out = Queue::with_capacity(64);
        let component = kit::Component::new(settings, &env, &mut out).unwrap();
        while out.pop().is_some() {}
        let mut credential = Writer::new(40);
        credential.put(&SIGN_IN.to_be_bytes()).unwrap();
        credential.put(&[7; 32]).unwrap();
        Self { component, env, out, credential: Secret::new(credential.finish()) }
    }
    fn down(&mut self, event: kit::Event) {
        self.component.down(&self.env, event, &mut self.out);
    }
    fn authenticate(&mut self) {
        let Some(kit::Request::Secrets(kit::SecretsRequest::Get { owner })) = self.out.pop() else {
            panic!("credential digest requested")
        };
        assert_eq!(owner, kit::SecretOwner::SignIn(SIGN_IN));
        self.down(kit::Event::Secrets(kit::SecretsAnswer::Got {
            owner,
            value: Some(kit::Kept {
                digest: Secret::new(Box::new(skein_lib::sha256::digest(self.credential.expose()))),
                expires: EXPIRY,
            }),
        }));
        assert!(self.component.up(&self.env, &mut self.out));
    }
    fn exchange(&mut self, exchange: Token, message: kit::Message) {
        self.down(kit::Event::Transport(kit::FromTransport::Exchange {
            exchange,
            credential: Some(self.credential.clone()),
            message,
        }));
        self.authenticate();
    }
    fn stream(&mut self) -> [u8; 16] {
        self.down(kit::Event::Transport(kit::FromTransport::StreamOpen {
            stream: STREAM,
            credential: Some(self.credential.clone()),
        }));
        self.authenticate();
        assert!(self.component.up(&self.env, &mut self.out));
        let Some(kit::Request::Transport(kit::ToTransport::Streamed {
            stream: STREAM,
            message: kit::Streamed::Record(record),
        })) = self.out.pop()
        else {
            panic!("stream hello")
        };
        let wire::StreamEvent::Hello(hello) =
            wire::StreamEvent::decode(&self.env.limits.wire, &mut Reader::new(&record)).unwrap()
        else {
            panic!("hello record")
        };
        *hello.stream()
    }
    fn watch(&mut self, stream: [u8; 16]) -> Token {
        let mut subject = Writer::new(1);
        wire::Subject::Me.encode(&mut subject).unwrap();
        let subject = wire::Owned::new(
            &self.env.limits.wire,
            wire::OwnedParts {
                owner: wire::Owner::Jig,
                version: jig_client_protocol_wire::VERSION,
                record: subject.finish(),
            },
        )
        .unwrap();
        let value = wire::Open::new(&self.env.limits.wire, wire::OpenParts { stream, watch: WATCH, subject }).unwrap();
        let mut bytes = Writer::new(usize::try_from(value.measure()).unwrap());
        value.encode(&mut bytes).unwrap();
        self.exchange(OPEN, kit::Message::Watch(bytes.finish()));
        let Some(kit::Request::Core(event)) = self.out.pop() else { panic!("core watch") };
        let jig_domain_core::Event::Watch { watcher, .. } = *event else { panic!("frame watch") };
        self.down(kit::Event::Admitted { watcher });
        while self.component.up(&self.env, &mut self.out) {}
        while self.out.pop().is_some() {}
        watcher
    }
    fn unwatch(&mut self, stream: [u8; 16]) {
        let value =
            wire::WatchClose::new(&self.env.limits.wire, wire::WatchCloseParts { stream, watch: WATCH }).unwrap();
        let mut bytes = Writer::new(usize::try_from(value.measure()).unwrap());
        value.encode(&mut bytes).unwrap();
        self.exchange(CLOSE, kit::Message::Unwatch(bytes.finish()));
    }
}

#[test]
fn an_unwatch_after_its_watch_ends_is_answered_when_the_stream_goes() {
    let mut stand = Stand::new();
    let stream = stand.stream();
    let watcher = stand.watch(stream);
    stand.down(kit::Event::WatchEnded { watcher, why: jig_domain_views::End::Gone });
    stand.unwatch(stream);
    stand.down(kit::Event::Transport(kit::FromTransport::StreamGone { stream: STREAM }));
    let mut answer = None;
    for _ in 0..ITERATIONS {
        stand.component.up(&stand.env, &mut stand.out);
        while let Some(request) = stand.out.pop() {
            if let kit::Request::Transport(kit::ToTransport::Reply { exchange: CLOSE, reply }) = request {
                answer = Some(reply);
            }
        }
    }
    let Some(kit::Reply::Record(record)) = answer else { panic!("unwatch answer after the stream closes") };
    assert_eq!(
        wire::CloseAnswer::decode(&stand.env.limits.wire, &mut Reader::new(&record)),
        Ok(wire::CloseAnswer::Closed)
    );
}
