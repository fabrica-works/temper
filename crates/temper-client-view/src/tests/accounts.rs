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
    let mut doc = document(count);
    let mut uses = List::with_capacity(300);
    for party in 1..=300 {
        uses.push(c::OwnerAccountUse::Party(party)).unwrap();
    }
    doc.accounts.get_mut(0).unwrap().uses = Some(uses);
    deliver(&mut root, watch, doc);
    let domain = fixtures::limits();
    let words = fixtures::words();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
    let mut view = crate::View::new(&bounds, &domain, words, fixtures::FORGE);
    let tree = fixtures::build(&mut view, &root);
    let shown = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    assert!(shown.contains("Account 3"));
    assert!(shown.contains("Person 300"));
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

#[test]
fn supplied_source_states_keep_their_words() {
    use c::OwnerAccountState as S;
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    for (state, expected) in [
        (S::Asking, b"Checking".as_slice()),
        (S::Granted, b"Ready"),
        (S::Later { until: None }, b"Limited. Tasks on it wait."),
        (S::SignInAgain, b"Needs sign-in"),
        (S::Misconfigured, b"Check the configuration's registration."),
        (S::Cancelled, b"Cancelled"),
        (S::Closing, b"Closing"),
        (
            S::Unloaded { at: c::OwnerAccountPlace::File, why: c::OwnerAccountUnread::Stalled },
            b"Cannot load File: Stalled",
        ),
    ] {
        let mut doc = document(1);
        doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = state;
        deliver(&mut root, watch, doc);
        let shown = fixtures::text(fixtures::build(&mut view, &root));
        assert!(skein_lib::bytes::find(&shown, expected).is_some(), "{state:?}");
        assert!(skein_lib::bytes::find(&shown, b"Limited until").is_none(), "no invented reset");
    }
}
#[test]
fn a_supplied_reset_is_visible() {
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state =
        c::OwnerAccountState::Later { until: Some(skein_lib::Wall::from_nanos(14_000_000_000)) };
    deliver(&mut root, watch, doc);
    assert!(skein_lib::bytes::find(&fixtures::text(fixtures::build(&mut view, &root)), b"Limited until 14").is_some());
}
#[test]
fn sign_in_ends_preserve_their_typed_details() {
    use c::{OwnerAccountEnd as E, OwnerAccountFailure as F, OwnerAccountRefusal as R};
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    for (end, expected) in [
        (E::Made { account: 29, declined: true }, b"You signed in without plan usage.".as_slice()),
        (E::Renewed { account: 29, declined: false }, b"same account again; account 29"),
        (E::Refused { refusal: R::NotKept { why: c::OwnerAccountKeyFailure::TooLong } }, b"The key is too long."),
        (E::Refused { refusal: R::Limit { bound: c::OwnerAccountBound::Sources } }, b"Source limit reached."),
        (E::Refused { refusal: R::Kind { kind: c::OwnerAccountKind::Program } }, b"Source kind refused: Program login"),
        (E::Failed { failure: F::MissingScope { scope: 19 } }, b"Missing required scope 19"),
        (E::Failed { failure: F::Later { after: Some(skein_lib::Duration::from_secs(5)) } }, b"Try again after 5"),
    ] {
        let mut doc = document(0);
        doc.last_end = Some(c::OwnerAccountLastEnd { source: SOURCE, end });
        deliver(&mut root, watch, doc);
        assert!(skein_lib::bytes::find(&fixtures::text(fixtures::build(&mut view, &root)), expected).is_some());
    }
}
#[test]
fn account_bounds_follow_document_bytes() {
    let mut domain = fixtures::limits();
    let words = fixtures::words();
    let first = crate::Limits::of(&domain, &words).unwrap();
    domain.client.domain.document_bytes *= 2;
    let larger = crate::Limits::of(&domain, &words).unwrap();
    assert!(larger.tree.nodes > first.tree.nodes);
    assert_eq!(larger.tree.text_bytes, larger.tree.nodes * larger.tree.text_per_node);
    assert!(crate::worst_case(&larger, &domain, &words).is_some());
}
#[test]
fn an_owner_sees_deployment_accounts_after_their_own() {
    let limits = fixtures::limits();
    let (mut root, me_watch, _) = fixtures::root_at(d::Address::Accounts, &limits);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.deployment_role = Some(c::HeldRole::Owner);
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::with_capacity(d::max_out(&limits));
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    let watch =
        out.iter()
            .find_map(|r| {
                if let d::Request::Client {
                    request:
                        jig_client::Request::Down {
                            down:
                                jig_client::Down::Open {
                                    watch,
                                    subject: jig_client::Watchable::Jig(c::Subject::Deployment),
                                },
                        },
                } = r
                {
                    Some(*watch)
                } else {
                    None
                }
            })
            .unwrap();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let deployment = jig_client_protocol_link::translate::documents::owner_deployment(
        jig_client_test_records::deployment_rows_record(),
        &link,
    )
    .unwrap();
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::OwnerDeployment(Box::new(deployment))),
                },
            },
        },
    );
    let mut view = fixtures::view();
    let text = fixtures::text(fixtures::build(&mut view, &root));
    assert!(
        skein_lib::bytes::find(&text, b"Your accounts").unwrap()
            < skein_lib::bytes::find(&text, b"Deployment accounts").unwrap()
    );
    assert!(skein_lib::bytes::find(&text, b"Account 41").is_some());
}
#[test]
fn invalid_account_numbers_cannot_change_the_request() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(0));
    let mut view = fixtures::view();
    press(&mut view, &mut root, Role::TextField, b"Provider number", Act::Input { value: Box::from(b"17".as_slice()) });
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Account number (empty for a new account)",
        Act::Input { value: Box::from(b"not-a-number".as_slice()) },
    );
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"private-once".as_slice()) },
    );
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::Button, b"Add key").unwrap();
    assert!(crate::event(&mut view, node, Act::Press).is_none());
    assert!(root.pages().accounts().unwrap().pending.is_none());
    let before = root.pages().accounts().unwrap().key_written();
    press(&mut view, &mut root, Role::Button, b"Cancel", Act::Press);
    let tree = fixtures::build(&mut view, &root);
    for field in tree.nodes().iter().filter(|n| n.role == Role::TextField) {
        assert!(field.value.as_ref().unwrap().written > before);
    }
}
