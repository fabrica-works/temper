use super::fixtures as f;
use crate::{
    forge::{self, demo},
    pages::Act,
};
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{List, Queue};
use temper_client_protocol_wire::{self as wire, v1 as w};
const PROVIDER: u32 = 17;
const CUE: &[u8] = b"Change the title";
const DRAFT: &[u8] = b"My draft";
fn answer(cue: &[u8]) -> Box<[u8]> {
    let mut cues = List::with_capacity(1);
    cues.push(cue.into()).unwrap();
    wire::demo_answer(
        &w::DemoAnswer::new(
            &w::CEILINGS,
            w::DemoAnswerParts {
                project: f::PROJECT,
                forge: wire::FORGE,
                provider: PROVIDER,
                owner: b"acme".as_slice().into(),
                name: b"shop".as_slice().into(),
                cues,
            },
        )
        .unwrap(),
    )
}
fn page() -> Readout {
    Readout::Family { family: wire::FORGE, record: Some(answer(CUE)) }
}
fn read(side: &mut forge::Side) -> Handle {
    let mut out = Queue::with_capacity(1);
    assert!(!demo::told(side, &Told::Link { link: Link::Live }, f::limits().forge, &mut out));
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, read } }) = out.pop() else {
        panic!("demo read");
    };
    let Readable::Family { scope, window, record } = read else { panic!("family read") };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    assert_eq!(wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read(), &w::Read::Demo);
    assert_ne!(handle.number, 0, "zero belongs to me");
    assert!(out.is_empty());
    handle
}
#[test]
fn the_demo_is_read_once_per_live_link_and_replaced_pages_are_dropped() {
    for (row, gap, reads) in [
        ("live again", Link::Live, false),
        ("offline then live", Link::Offline, true),
        ("signed out then live", Link::SignedOut, true),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let first = read(&mut side);
        let mut out = Queue::with_capacity(1);
        assert!(!demo::told(&mut side, &Told::Link { link: gap }, f::limits().forge, &mut out), "{row}");
        assert!(out.is_empty(), "{row}");
        if reads {
            let next = read(&mut side);
            assert_ne!(first, next, "{row}");
            assert!(demo::told(&mut side, &Told::Page { handle: first, page: page() }, f::limits().forge, &mut out));
            assert!(side.demo().is_none(), "{row}: replaced page dropped");
            demo::told(&mut side, &Told::Page { handle: next, page: page() }, f::limits().forge, &mut out);
            assert!(side.demo().is_some(), "{row}");
        } else {
            assert!(!demo::told(&mut side, &Told::Link { link: Link::Live }, f::limits().forge, &mut out));
            assert!(out.is_empty(), "{row}");
        }
    }
}
#[test]
fn a_demo_page_replaces_the_last_answer_or_clears_it() {
    let mut another_version = answer(CUE);
    *another_version.get_mut(1).unwrap() = u8::try_from(wire::CLIENT_VERSION + 1).unwrap();
    for (row, page, present) in [
        ("demo", page(), true),
        ("none", Readout::Family { family: wire::FORGE, record: None }, false),
        ("not served", Readout::NotServed { family: Some(wire::FORGE), served: None }, false),
        ("limit", Readout::Limit, false),
        ("undecodable", Readout::Family { family: wire::FORGE, record: Some(Box::new([255])) }, false),
        ("another version", Readout::Family { family: wire::FORGE, record: Some(another_version) }, false),
        ("empty cue", Readout::Family { family: wire::FORGE, record: Some(answer(b"")) }, false),
        ("wrong family", Readout::Family { family: 99, record: Some(answer(CUE)) }, false),
        ("jig page", Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)), false),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let first = read(&mut side);
        demo::told(
            &mut side,
            &Told::Page { handle: first, page: self::page() },
            f::limits().forge,
            &mut Queue::with_capacity(1),
        );
        let shown = forge::shown(&side);
        demo::told(&mut side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.demo().is_some(), "{row}: keep the last answer across the gap");
        let handle = read(&mut side);
        assert!(
            demo::told(&mut side, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1)),
            "{row}"
        );
        assert_eq!(side.demo().is_some(), present, "{row}");
        assert_eq!(forge::shown(&side) != shown, !present, "{row}: redraw only on change");
    }
}
fn accounts(provider: Option<u32>, usable: bool, cut: bool) -> c::OwnerAccounts {
    let mut sources = List::with_capacity(1);
    sources
        .push(c::OwnerAccountSource {
            name: None,
            number: 1,
            place: 1,
            kind: c::OwnerAccountKind::Key,
            state: c::OwnerAccountState::Granted,
            usable,
            expiring: false,
            attention: false,
            spent_until: None,
        })
        .unwrap();
    let mut accounts = List::with_capacity(1);
    accounts
        .push(c::OwnerAccountLine {
            number: 1,
            place: 1,
            uses: None,
            plan: false,
            provider,
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(f::acts::text(b"Model")),
            holder: c::OwnerAccountHolder::Party(f::PARTY),
            sources,
            cut,
        })
        .unwrap();
    c::OwnerAccounts {
        program_hosts: List::with_capacity(0),
        readiness: List::with_capacity(0),
        readiness_cut: false,
        accounts,
        signing: List::with_capacity(0),
        last_end: None,
        usable,
        settled: true,
        cut: false,
    }
}
#[test]
fn the_model_half_waits_for_complete_accounts_and_no_other_usable_provider() {
    let answer = forge::family::demo(&answer(CUE), &w::CEILINGS).unwrap();
    let mut cut = accounts(Some(PROVIDER), true, false);
    cut.cut = true;
    for (row, answer, document, model) in [
        ("no answer", None, Some(accounts(Some(PROVIDER), true, false)), false),
        ("no document", Some(&answer), None, false),
        ("cut document", Some(&answer), Some(cut), false),
        ("demo provider usable", Some(&answer), Some(accounts(Some(PROVIDER), true, false)), true),
        ("other unusable", Some(&answer), Some(accounts(Some(PROVIDER + 1), false, false)), true),
        ("other usable", Some(&answer), Some(accounts(Some(PROVIDER + 1), true, false)), false),
        ("unknown provider usable", Some(&answer), Some(accounts(None, true, false)), false),
        ("other cut", Some(&answer), Some(accounts(Some(PROVIDER + 1), false, true)), false),
    ] {
        assert_eq!(demo::halves(answer, document.as_ref()), demo::Halves { model, forge: false }, "{row}");
    }
}
fn event(root: &mut crate::Domain, event: crate::Event) -> Queue<crate::Request> {
    let mut out = Queue::with_capacity(crate::max_out(&f::limits()));
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn act(root: &mut crate::Domain, act: Act) -> Queue<crate::Request> {
    event(root, crate::Event::Act { action: crate::Action::Pages(act) })
}
fn opened(cue: &[u8]) -> (crate::Domain, skein_lib::Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, account) = f::start_accounts(&mut root, List::with_capacity(0));
    for (watch, delivery) in [
        (me, c::Delivery::Me(Box::new(f::chats::me()))),
        (account, c::Delivery::OwnerAccounts(Box::new(accounts(Some(PROVIDER), true, false)))),
    ] {
        event(
            &mut root,
            crate::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
                },
            },
        );
    }
    // Install an answer through the same page handler as the routed read.
    demo::told(&mut root.forge, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
    let handle = read(&mut root.forge);
    crate::route::told(
        &mut root,
        &f::env(&f::limits()),
        Told::Page { handle, page: Readout::Family { family: wire::FORGE, record: Some(answer(cue)) } },
        &mut Queue::with_capacity(crate::max_out(&f::limits())),
    );
    event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Chats) });
    act(&mut root, Act::ChatWords { words: DRAFT.into() });
    (root, account)
}
#[test]
fn a_suggestion_starts_one_chat_with_its_cue_and_preserves_the_draft() {
    assert_eq!(f::limits().client.domain.text, 32, "the bound rows below sit at the composer's bound");
    let at_bound = [b'a'; 32];
    let past_bound = [b'a'; 33];
    for (row, words, pending, project, model, cue, count) in [
        ("demo project", CUE, false, f::PROJECT, true, 0, 1),
        ("at the composer's bound", &at_bound, false, f::PROJECT, true, 0, 1),
        ("past the composer's bound", &past_bound, false, f::PROJECT, true, 0, 0),
        ("pending", CUE, true, f::PROJECT, true, 0, 0),
        ("other project", CUE, false, f::PROJECT + 1, true, 0, 0),
        ("no model half", CUE, false, f::PROJECT, false, 0, 0),
        ("past cues", CUE, false, f::PROJECT, true, 1, 0),
    ] {
        let (mut root, account) = opened(words);
        if pending {
            act(&mut root, Act::StartChat);
        }
        if project != f::PROJECT {
            let limits = forge::wire_limits(&f::limits().forge).unwrap();
            let mut parts = wire::decode_demo_answer(&answer(CUE), &limits).unwrap().into_parts();
            parts.project = project;
            let record = wire::demo_answer(&w::DemoAnswer::new(&limits, parts).unwrap());
            demo::told(
                &mut root.forge,
                &Told::Link { link: Link::Offline },
                f::limits().forge,
                &mut Queue::with_capacity(1),
            );
            let handle = read(&mut root.forge);
            crate::route::told(
                &mut root,
                &f::env(&f::limits()),
                Told::Page { handle, page: Readout::Family { family: wire::FORGE, record: Some(record) } },
                &mut Queue::with_capacity(crate::max_out(&f::limits())),
            );
        }
        if !model {
            event(
                &mut root,
                crate::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched {
                            watch: account,
                            event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(accounts(
                                Some(PROVIDER + 1),
                                true,
                                false,
                            )))),
                        },
                    },
                },
            );
        }
        let mut out = act(&mut root, Act::Suggest { cue });
        let mut sent = List::with_capacity(1);
        while let Some(request) = out.pop() {
            if let crate::Request::Client {
                request:
                    jig_client::Request::Down { down: jig_client::Down::Send { request: jig_client::Ask::Jig(ask), .. } },
            } = request
            {
                sent.push(*ask).expect("one chat start");
            }
        }
        assert_eq!(usize::try_from(sent.len()).unwrap(), count, "{row}");
        if count == 1 {
            assert_eq!(
                sent.as_slice(),
                [jig_client::shapes::chat(&f::chats::me(), f::PROJECT, f::acts::text(words)).unwrap()],
                "{row}"
            );
        }
        assert_eq!(root.pages().chats().unwrap().words.as_ref(), DRAFT, "{row}");
    }
}
