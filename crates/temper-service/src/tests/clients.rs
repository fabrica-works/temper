use crate::clients::translation;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_protocol_engine_clients as kit;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;

const REPLY: ReplyTo = ReplyTo::new(Token::new(9));
#[test]
fn a_chat_ask_reaches_the_root_as_a_party_ask() {
    let ask = core::Event::People(people::Event::Ask {
        reply_to: REPLY,
        sign_in: 3,
        key: [5; 16],
        ask: people::Ask::StartChat { project: 1, words: b"question".as_slice().into() },
    });
    let Ok(root::PartyInput::Ask { reply_to, sign_in, key, ask }) = translation::to_root(ask) else {
        panic!("party ask")
    };
    assert_eq!(reply_to, REPLY);
    assert_eq!(sign_in, 3);
    assert_eq!(key, [5; 16]);
    assert_eq!(ask, people::Ask::StartChat { project: 1, words: b"question".as_slice().into() });
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
fn a_request_before_decisions_open_is_answered_not_ready() {
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
                ask: people::Ask::StartChat { project: 1, words: b"question".as_slice().into() },
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
    clients.advance(&mut events);
    let Some(skein_io::Request::Listen { owner, addr }) = clients.io.pop() else { panic!("listen") };
    assert!(clients.up(skein_io::Event::Listening { owner, listener: Token::new(77), addr }));
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events);
    let link = clients.links.pop().unwrap();
    assert!(link.starts_with(b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret="));
    assert!(clients.links.is_empty());
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events);
    assert!(clients.links.is_empty());
}
#[test]
fn closing_a_listener_during_start_closes_the_component() {
    let mut clients = super::fixtures::clients();
    let mut events = skein_lib::Queue::with_capacity(64);
    clients.advance(&mut events);
    let Some(skein_io::Request::Listen { owner, addr }) = clients.io.pop() else { panic!("listen") };
    clients.down(kit::Event::Close);
    assert!(clients.up(skein_io::Event::Listening { owner, listener: Token::new(77), addr }));
    clients.advance(&mut events);
    let Some(skein_io::Request::Close { entity }) = clients.io.pop() else { panic!("late listener close") };
    assert_eq!(entity, Token::new(77));
    assert!(clients.up(skein_io::Event::Closed { owner }));
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events);
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
    let mut config = super::fixtures::engine().jig;
    config.limits.journal.commits = 1;
    let mut clients = crate::clients::Clients::new(&config, &super::fixtures::seeds(), 64).unwrap();
    clients.commit(3, &[root::Write::Erase(root::Key::Core(core::Key::People(people::Key::SignIn(7))))]);
    assert!(!clients.commit_room());
    let mut events = skein_lib::Queue::with_capacity(64);
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events);
    assert!(!clients.commit_room());
    clients.committed(3);
    clients.iterate(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, &mut events);
    assert!(clients.commit_room());
}
