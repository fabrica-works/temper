use crate::clients::translation;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;

const LISTENER: Token = Token::new(77);
const STARTING_LISTENER: Token = Token::new(99);
const REPLY: ReplyTo = ReplyTo::new(Token::new(9));
const PROCESS_UPTIME: skein_lib::Time = skein_lib::Time::from_nanos(600_000_000_000);

#[test]
fn a_link_printed_on_the_first_clock_tick_is_live() {
    let mut service = super::fixtures::started();
    let engine = &mut service.engine;
    engine.clients.iterate(PROCESS_UPTIME, skein_lib::Wall::EPOCH, &mut engine.events, &[]);
    let Some(skein_io::Request::Listen { owner, addr }) = engine.clients.io.pop() else { panic!("listen") };
    assert!(engine.clients.up(skein_io::Event::Listening { owner, listener: LISTENER, addr }));
    engine.clients.iterate(PROCESS_UPTIME, skein_lib::Wall::EPOCH, &mut engine.events, &[]);
    let link = engine.clients.links.pop().expect("printed link");
    let body = peer::link_body(engine, &link);
    let mut peer = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump_at(engine, false, PROCESS_UPTIME, skein_lib::Wall::EPOCH);
    assert_eq!(engine.clients.early.len(), 1, "the printed link reaches the root");
}
#[test]
fn a_chat_ask_reaches_the_root_as_a_party_ask() {
    let ask = core::Event::People(people::Event::Ask {
        reply_to: REPLY,
        sign_in: 3,
        key: [5; 16],
        ask: super::fixtures::chat(1, b"question"),
    });
    let Ok(root::PartyInput::Ask { reply_to, sign_in, key, ask }) = translation::to_root(ask) else {
        panic!("party ask")
    };
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
    assert_eq!(key, [5; 16]);
    assert_eq!(ask, super::fixtures::chat(1, b"question"));
}
#[test]
fn sign_out_preserves_its_reply_right_and_sign_in() {
    let event = core::Event::People(people::Event::SignOut { reply_to: REPLY, sign_in: 3 });
    let Ok(root::PartyInput::SignOut { reply_to, sign_in }) = translation::to_root(event) else {
        panic!("party sign out")
    };
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
}
#[test]
fn not_ready_uses_the_clients_reply_vocabulary() {
    for (row, input, expected) in [
        (
            "sign out",
            root::PartyInput::SignOut { reply_to: REPLY, sign_in: 3 },
            people::Reply::Again(people::Again::NotReady),
        ),
        (
            "ask",
            root::PartyInput::Ask {
                reply_to: REPLY,
                sign_in: 3,
                key: [5; 16],
                ask: super::fixtures::chat(1, b"question"),
            },
            people::Reply::Again(people::Again::NotReady),
        ),
    ] {
        let Some(kit::Event::Reply { to, reply }) = translation::not_ready(input) else {
            panic!("not-ready reply {row}")
        };
        assert_eq!(to, REPLY, "{row}");
        assert_eq!(reply, expected, "{row}");
    }
}
#[test]
fn a_released_reply_keeps_its_terminal_and_destination() {
    let request = root::PartyRequest::Deliver(root::Delivery::WebReply { to: REPLY, reply: people::Reply::SignedOut });
    let Some(kit::Event::Reply { to, reply }) = translation::to_component(request) else { panic!("clients reply") };
    assert_eq!(to, REPLY);
    assert_eq!(reply, people::Reply::SignedOut);
}

#[test]
fn the_component_prints_one_sign_in_link_at_start() {
    let mut clients = super::fixtures::clients();
    let mut events = skein_lib::Queue::with_capacity(64);
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    let Some(skein_io::Request::Listen { owner, addr }) = clients.io.pop() else { panic!("listen") };
    assert!(clients.up(skein_io::Event::Listening { owner, listener: LISTENER, addr }));
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    let link = clients.links.pop().unwrap();
    assert!(link.starts_with(b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret="));
    assert!(clients.links.is_empty());
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    assert!(clients.links.is_empty());
}
#[test]
fn closing_a_listener_during_start_closes_the_component() {
    let mut clients = super::fixtures::clients();
    let mut events = skein_lib::Queue::with_capacity(64);
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    let Some(skein_io::Request::Listen { owner, addr }) = clients.io.pop() else { panic!("listen") };
    clients.down(kit::Event::Close);
    assert!(clients.up(skein_io::Event::Listening { owner, listener: LISTENER, addr }));
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    let Some(skein_io::Request::Close { entity }) = clients.io.pop() else { panic!("late listener close") };
    assert_eq!(entity, LISTENER);
    assert!(clients.up(skein_io::Event::Closed { owner }));
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    assert!(clients.closed);
}

#[test]
fn a_sign_in_digest_and_expiry_are_kept_in_secrets() {
    let mut clients = super::fixtures::clients();
    let mut store = super::fixtures::memory_store();
    clients.secrets.push(kit::SecretsRequest::Keep {
        owner: kit::SecretOwner::SignIn(3),
        value: kit::Kept {
            digest: skein_lib::Secret::new(Box::new([5; 32])),
            expires: skein_lib::Wall::from_nanos(12),
        },
    });
    clients.secrets_step(&mut store);
    store.iterate();
    let Some(jig_protocol_store::Event::Kept { owner }) = store.above.pop() else { panic!("digest kept") };
    assert_eq!(owner, Token::new(crate::tokens::CLIENTS + 1));
    let Some(jig_protocol_store::Event::Secret { value: Some(value), .. }) = store.secrets(
        Token::new(9),
        jig_protocol_store::Secrets::Get { owner: jig_protocol_store::SecretOwner::SignIn, number: 3 },
    ) else {
        panic!("secret read")
    };
    assert_eq!(value.expose().get(..8), Some([0, 0, 0, 0, 0, 0, 0, 12].as_slice()));
    assert_eq!(value.expose().get(8..), Some([5; 32].as_slice()));
}
#[test]
fn a_sign_in_ending_waits_for_its_erase_to_commit() {
    let config = super::fixtures::engine().jig;
    let mut journal = super::fixtures::engine().limits.root.journal;
    journal.commits = 1;
    let mut clients = crate::clients::Clients::new(
        &config,
        &super::fixtures::engine().clients.listener,
        &super::fixtures::seeds(),
        64,
        &journal,
    )
    .unwrap();
    clients.commit(3, &[root::Write::Erase(root::Key::Core(core::Key::People(people::Key::SignIn(7))))]);
    assert!(!clients.commit_room());
    let mut events = skein_lib::Queue::with_capacity(64);
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    assert!(!clients.commit_room());
    clients.committed(3);
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events, &[]);
    assert!(clients.commit_room());
}

use super::fixtures::http as peer;
#[test]
fn the_next_clients_stream_opens_once_a_peer_ends_its_own() {
    // The small profile admits two streams; a third needs a predecessor freed.
    const CLIENTS: u32 = 3;
    let mut service = super::fixtures::ready();
    let link = service.links().pop().unwrap();
    let body = peer::link_body(&service.engine, &link);
    let mut sign_in = peer::Peer::new(&mut service.engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    sign_in.pump(&mut service.engine, true);
    let jig_client_protocol_wire::v9::SignInAnswer::SignedIn(answer) = sign_in.sign_in_answer(&service.engine) else {
        panic!("signed in")
    };
    let bearer = peer::bearer(answer.token().as_ref().expect("native credential").expose());
    let mut third_opened = false;
    for _ in 0..CLIENTS {
        let mut stream = peer::Peer::new(&mut service.engine, b"GET", b"/jig/v1/stream", &[], Some(&bearer));
        third_opened = stream.read_hello(&mut service.engine);
        stream.end(&mut service.engine);
    }
    assert!(third_opened, "the third client receives its hello before a heartbeat");
}
#[test]
fn the_engine_prints_one_sign_in_link_at_start() {
    let mut service = super::fixtures::ready();
    let link = service.links().pop().expect("sign-in link");
    assert!(link.starts_with(b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret="));
    assert!(service.links().is_empty());
    super::fixtures::turn(&mut service);
    assert!(service.links().is_empty());
}
#[test]
fn a_link_sign_in_reaches_the_root_and_answers_a_token() {
    let mut service = super::fixtures::ready();
    let link = service.links().pop().unwrap();
    let body = peer::link_body(&service.engine, &link);
    let mut peer = peer::Peer::new(&mut service.engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(&mut service.engine, true);
    let jig_client_protocol_wire::v9::SignInAnswer::SignedIn(answer) = peer.sign_in_answer(&service.engine) else {
        panic!("signed in")
    };
    assert_eq!(answer.party(), 1);
    assert!(super::fixtures::get(&mut service, core::Key::People(people::Key::SignIn(1))).is_some());
    let engine = &mut service.engine;
    let Some(jig_protocol_store::Event::Secret { value: Some(secret), .. }) = engine.store.secrets(
        Token::new(1),
        jig_protocol_store::Secrets::Get { owner: jig_protocol_store::SecretOwner::SignIn, number: 1 },
    ) else {
        panic!("digest durable in real secrets store")
    };
    assert_eq!(secret.expose().len(), 40);
    let bearer = peer::bearer(answer.token().as_ref().expect("native credential").expose());
    let mut stream = peer::Peer::new(engine, b"GET", b"/jig/v1/stream", &[], Some(&bearer));
    stream.pump(engine, true);
    assert!(stream.streaming(), "the issued bearer authenticates through the stored digest");
    assert_eq!(engine.end, None);
}
#[test]
fn a_request_before_decisions_open_is_answered_not_ready() {
    let mut service = super::fixtures::started();
    let engine = &mut service.engine;
    engine.clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut engine.events, &engine.workers.forges);
    let Some(skein_io::Request::Listen { owner, addr }) = engine.clients.io.pop() else { panic!("listener") };
    assert!(engine.clients.up(skein_io::Event::Listening { owner, listener: STARTING_LISTENER, addr }));
    engine.clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut engine.events, &engine.workers.forges);
    let link = engine.clients.links.pop().unwrap();
    let body = peer::link_body(engine, &link);
    let mut peer = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine, false);
    assert_eq!(engine.clients.early.len(), 1, "request staged during restoration");
    crate::engine::iterate(engine, skein_lib::Time::ZERO, skein_lib::Wall::EPOCH);
    peer.pump(engine, false);
    assert_eq!(peer.sign_in_answer(engine), jig_client_protocol_wire::v9::SignInAnswer::NotReady);
    assert!(!engine.domain.ready());
}
#[test]
fn a_clients_listener_failure_stops_the_engine() {
    let mut service = super::fixtures::started();
    service.told.push(skein_io::Event::Closed { owner: Token::new(crate::tokens::SERVER) });
    super::fixtures::turn(&mut service);
    assert_eq!(service.end(), Some(&crate::End::Stopped(crate::Stop::Listener)));
}
