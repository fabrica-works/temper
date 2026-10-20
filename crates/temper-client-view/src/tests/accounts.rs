#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test assertions inspect semantic trees and diagnostic redaction"
)]
use super::fixtures;
use jig_client::contract as c;
use skein_lib::{List, Token};
use skein_ui::{Act, Role};
use temper_client_domain::{
    self as d,
    pages::accounts::{Act as A, Operation, Selection},
};
const PROVIDER: u32 = 17;
const SOURCE: u32 = 43;
const ADDRESS: &[u8] = b"https://issuer.example/visit";
fn source(number: u32) -> c::OwnerAccountSource {
    c::OwnerAccountSource {
        number,
        place: number,
        kind: c::OwnerAccountKind::Key,
        state: c::OwnerAccountState::Granted,
        usable: true,
        expiring: false,
        attention: false,
        spent_until: None,
    }
}
fn document(count: u32) -> c::OwnerAccounts {
    let mut accounts = List::with_capacity(count);
    let mut signing = List::with_capacity(count);
    for number in 1..=count {
        let mut sources = List::with_capacity(count);
        for n in 1..=count {
            sources.push(source(n)).unwrap();
        }
        accounts
            .push(c::OwnerAccountLine {
                number,
                place: number,
                uses: None,
                plan: true,
                provider: Some(PROVIDER),
                display: c::DocumentText::complete(c::Text::new(Box::from(b"Pat".as_slice()), 32).unwrap()),
                holder: c::OwnerAccountHolder::Party(73),
                sources,
                cut: true,
            })
            .unwrap();
        signing
            .push(c::OwnerAccountSignIn {
                source: number,
                provider: PROVIDER,
                holder: c::OwnerAccountHolder::Party(73),
            })
            .unwrap();
    }
    c::OwnerAccounts { accounts, signing, last_end: None, usable: true, settled: false, cut: true }
}
fn deliver(root: &mut d::Domain, watch: Token, doc: c::OwnerAccounts) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(doc))),
                },
            },
        },
    );
}
fn action(root: &mut d::Domain, act: A) {
    fixtures::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Accounts(act)) });
}
fn press(view: &mut crate::View, root: &mut d::Domain, role: Role, name: &[u8], act: Act) {
    let node = fixtures::find(fixtures::build(view, root), role, name).expect("offered account control");
    let event = crate::event(view, node, act).expect("account control decodes");
    fixtures::root_event(root, event);
}
#[test]
fn the_accounts_page_keeps_lists_beyond_rows() {
    let (mut root, watch) = fixtures::account_root();
    let count = fixtures::limits().client.domain.rows + 1;
    deliver(&mut root, watch, document(count));
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let shown = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    assert!(shown.contains("Account 3"));
    assert_eq!(shown.matches("Source 3").count(), usize::try_from(count).unwrap());
    assert!(shown.contains("Sign-in in progress: source 3"));
    assert!(shown.contains("Some account facts were omitted."));
    assert!(shown.find("Account 1").unwrap() < shown.find("Account 3").unwrap());
}
#[test]
fn the_key_field_clears_as_its_request_leaves() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(0));
    let mut view = fixtures::view();
    press(&mut view, &mut root, Role::TextField, b"Provider number", Act::Input { value: Box::from(b"17".as_slice()) });
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::TextField, b"Key (pasted once)").unwrap();
    let before = view.tree().nodes().iter().find(|n| n.id == node).unwrap().value.as_ref().unwrap().written;
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"private-once".as_slice()) },
    );
    assert!(!format!("{view:?}").contains("private-once"));
    press(&mut view, &mut root, Role::Button, b"Add key", Act::Press);
    let tree = fixtures::build(&mut view, &root);
    let field = tree.nodes().iter().find(|n| n.name.as_deref() == Some(b"Key (pasted once)")).unwrap();
    assert_eq!(field.entry, Some(skein_ui::EntryKind::Secret));
    assert!(field.value.as_ref().unwrap().text.is_empty());
    assert_ne!(field.value.as_ref().unwrap().written, before);
    assert!(field.states.as_slice().contains(&skein_ui::State::Disabled));
    assert!(!root.pages().accounts().unwrap().has_key());
}
#[test]
fn a_sign_in_address_has_visible_external_text() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(0));
    action(&mut root, A::Select { selection: Selection { provider: Some(PROVIDER), ..Default::default() } });
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::with_capacity(d::max_out(&limits));
    d::step(
        &mut root,
        &env,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::Accounts(A::Submit {
                name: [97; 16],
                operation: Operation::SignIn,
            })),
        },
        &mut out,
    );
    d::step(&mut root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| match request {
            d::Request::Client {
                request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            d::Request::Client { .. }
            | d::Request::Show { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Fact { .. } => None,
        })
        .expect("sign-in attempt");
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                        c::OwnerOutcome::AccountSignInStarted { source: SOURCE, address: Box::from(ADDRESS) },
                    )))),
                    why: None,
                },
            },
        },
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let link = tree.nodes().iter().find(|n| n.external.as_deref() == Some(ADDRESS)).expect("external sign-in link");
    let at = tree.nodes().iter().position(|n| n.id == link.id).unwrap();
    assert_eq!(tree.nodes()[at + 1].text.as_slice()[0].text.as_ref(), ADDRESS);
    assert!(String::from_utf8(fixtures::text(tree).into_vec()).unwrap().contains("on the engine's machine"));
}
