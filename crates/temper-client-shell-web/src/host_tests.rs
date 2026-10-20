//! 16:52Z ruling: the host owns secret/clearing/target evidence; Chromium owns navigation and re-entry.
use super::*;
use jig_client::contract as c;
use skein_ui::{EntryKind, Role};
const KEY: &[u8] = b"private-host-key";
const ADDRESS: &[u8] = b"http://127.0.0.1:9/authorize?state=test";
fn fixture() -> Host {
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH);
    host.root = root::unlinked(&host.env.limits, 11);
    host.event(Event::Start { address: Address::Accounts, kept: List::with_capacity(0) });
    host.client(jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    host.client(jig_client::Event::SignIn { with: jig_client::With::Transport });
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } });
    let me = host
        .out
        .iter()
        .find_map(|request| match request {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) },
                    },
            } => Some(*watch),
            Request::Client { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => None,
        })
        .expect("me watch");
    let link =
        jig_client_protocol_link::Limits::new(host.env.limits.client.domain, limits::EXCHANGES).expect("link bounds");
    let frame = jig_client_protocol_link::translate::documents::frame(jig_client_test_records::frame_record(), &link)
        .expect("frame");
    host.client(jig_client::Event::Up {
        up: jig_client::Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) },
    });
    host.requests();
    host
}
fn build(host: &mut Host) -> Queue<Patch> {
    let mut patches = Queue::with_capacity(host.bounds.tree.patches);
    host.build(&mut patches);
    patches
}
fn node(host: &Host, role: Role, name: &[u8]) -> NodeId {
    host.view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .expect("account control")
        .id
}
fn input(host: &mut Host, name: &[u8], value: &[u8]) -> Queue<Patch> {
    let field = node(host, Role::TextField, name);
    host.act(field, Act::Input { value: Box::from(value) });
    build(host)
}
fn send(host: &mut Host, name: &[u8]) -> Token {
    let button = node(host, Role::Button, name);
    host.act(button, Act::Press);
    host.client(jig_client::Event::Stored);
    host.out
        .iter()
        .find_map(|request| match request {
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } => {
                Some(*attempt)
            }
            Request::Client { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => None,
        })
        .expect("account request left the host")
}
#[test]
fn the_key_patch_declares_a_secret_entry() {
    let mut host = fixture();
    let patches = build(&mut host);
    let key = node(&host, Role::TextField, b"Key (pasted once)");
    let made =
        patches
            .iter()
            .find_map(|patch| {
                if let Patch::Insert { nodes, .. } = patch { nodes.iter().find(|made| made.id == key) } else { None }
            })
            .expect("inserted key field");
    assert_eq!(made.attribute.entry, Some(EntryKind::Secret));
    assert!(made.value.as_ref().expect("empty field").text.is_empty());
    input(&mut host, b"Provider number", b"17");
    let patches = input(&mut host, b"Key (pasted once)", KEY);
    assert!(!format!("{patches:?} {:?}", host.view).contains("private-host-key"));
}
#[test]
fn sending_a_key_emits_the_empty_value_patch() {
    let mut host = fixture();
    build(&mut host);
    input(&mut host, b"Provider number", b"17");
    input(&mut host, b"Key (pasted once)", KEY);
    let key = node(&host, Role::TextField, b"Key (pasted once)");
    send(&mut host, b"Add key");
    let patches = build(&mut host);
    assert!(
        patches
            .iter()
            .any(|patch| matches!(patch,Patch::Value {node,value:Some(value)} if *node==key && value.text.is_empty()))
    );
}
#[test]
fn a_key_never_reaches_the_small_store_output() {
    let mut host = fixture();
    build(&mut host);
    input(&mut host, b"Provider number", b"17");
    input(&mut host, b"Key (pasted once)", KEY);
    send(&mut host, b"Add key");
    let requests = host.requests();
    assert!(
        !requests.iter().any(|request| matches!(request, Asked::Save { .. })),
        "carried requests are not small-store writes"
    );
}
#[test]
fn the_external_href_is_the_supplied_sign_in_address() {
    let mut host = fixture();
    build(&mut host);
    input(&mut host, b"Provider number", b"17");
    let attempt = send(&mut host, b"Sign in with provider");
    host.client(jig_client::Event::Up {
        up: jig_client::Up::Answer {
            attempt,
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                c::OwnerOutcome::AccountSignInStarted { source: 43, address: Box::from(ADDRESS) },
            )))),
            why: None,
        },
    });
    build(&mut host);
    let link = host
        .view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.external.as_deref() == Some(ADDRESS))
        .expect("sign-in address")
        .id;
    assert_eq!(host.href(link).as_deref(), Some(ADDRESS));
}
#[test]
fn the_accounts_href_navigates_inside_the_app() {
    let mut host = fixture();
    build(&mut host);
    let link = node(&host, Role::Link, b"Accounts");
    assert_eq!(host.href(link).as_deref(), Some(b"/accounts".as_slice()));
    host.went(b"/chats");
    build(&mut host);
    let link = node(&host, Role::Link, b"Accounts");
    host.act(link, Act::Press);
    host.requests();
    assert_eq!(host.show, Some((Box::from(b"/accounts".as_slice()), false)));
}

#[test]
#[cfg(target_pointer_width = "64")]
fn the_web_preset_has_a_literal_price() {
    assert_eq!(limits::worst_case(&limits::client()), Some(119_483_766_472));
}
