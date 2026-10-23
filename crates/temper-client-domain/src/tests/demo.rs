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
pub(super) fn answer(cue: &[u8]) -> Box<[u8]> {
    let mut cues = List::with_capacity(1);
    cues.push(cue.into()).unwrap();
    wire::demo_answer(
        &w::DemoAnswer::new(
            &w::CEILINGS,
            w::DemoAnswerParts {
                state: w::DemoState::Running,
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
        assert!(!side.demo_answered(), "{row}: no answer yet");
        demo::told(
            &mut side,
            &Told::Page { handle: first, page: self::page() },
            f::limits().forge,
            &mut Queue::with_capacity(1),
        );
        let shown = forge::shown(&side);
        demo::told(&mut side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.demo().is_some(), "{row}: keep the last answer across the gap");
        assert!(side.demo_answered(), "{row}: keep that the read answered across the gap");
        let handle = read(&mut side);
        assert!(
            demo::told(&mut side, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1)),
            "{row}"
        );
        assert_eq!(side.demo().is_some(), present, "{row}");
        assert_eq!(forge::shown(&side) != shown, !present, "{row}: redraw only on change");
    }
}
pub(super) fn accounts(provider: Option<u32>, usable: bool, cut: bool) -> c::OwnerAccounts {
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
fn the_model_half_waits_for_accounts_and_yields_to_another_provider() {
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
        let forge = answer.is_some();
        assert_eq!(
            demo::halves(answer, document.as_ref(), &forge::forges::State::Rows(Box::new([]))),
            demo::Halves { model, forge },
            "{row}"
        );
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
    let mut at_bound = skein_lib::bytes::zeroed(usize::try_from(wire::CUE_BYTES).unwrap());
    at_bound.fill(b'a');
    let above_receiving_fixture = [b'a'; 33];
    for (row, words, pending, project, model, cue, count) in [
        ("demo project", CUE, false, f::PROJECT, true, 0, 1),
        ("at the composer's bound", &at_bound, false, f::PROJECT, true, 0, 1),
        ("above the receiving fixture", &above_receiving_fixture, false, f::PROJECT, true, 0, 1),
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
                [jig_client::shapes::chat(
                    &f::chats::me(),
                    f::PROJECT,
                    c::Text::new(words.into(), wire::CUE_BYTES).unwrap()
                )
                .unwrap()],
                "{row}"
            );
        }
        assert_eq!(root.pages().chats().unwrap().words.as_ref(), DRAFT, "{row}");
    }
}

#[test]
fn the_first_demo_read_answer_counts_even_when_it_has_no_record() {
    for page in [
        Readout::Family { family: wire::FORGE, record: None },
        Readout::NotServed { family: Some(wire::FORGE), served: None },
        Readout::Limit,
        Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
    ] {
        let mut side = forge::Side::new(&f::limits().forge, f::SEED);
        let handle = read(&mut side);
        let before = forge::shown(&side);
        demo::told(&mut side, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.demo_answered());
        assert!(side.demo().is_none());
        assert!(forge::shown(&side) > before, "an answered absence redraws the welcome");
        demo::told(&mut side, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
        assert!(side.demo_answered());
    }
}

fn demo_side(state: w::DemoState) -> forge::Side {
    let mut side = forge::Side::new(&f::limits().forge, f::SEED);
    let mut parts = wire::decode_demo_answer(&answer(CUE), &w::CEILINGS).unwrap().into_parts();
    parts.state = state;
    let handle = read(&mut side);
    demo::told(
        &mut side,
        &Told::Page {
            handle,
            page: Readout::Family {
                family: wire::FORGE,
                record: Some(wire::demo_answer(&w::DemoAnswer::new(&w::CEILINGS, parts).unwrap())),
            },
        },
        f::limits().forge,
        &mut Queue::with_capacity(4),
    );
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, _) = f::start_accounts(&mut root, List::with_capacity(0));
    event(
        &mut root,
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(f::chats::me()))),
                },
            },
        },
    );
    forge::me::told(
        &mut side,
        root.client(),
        &Told::Changed { handle: forge::me::handle() },
        &mut Queue::with_capacity(4),
    );
    side
}
fn started(side: &mut forge::Side, silent: bool) -> skein_lib::Token {
    let mut out = Queue::with_capacity(4);
    demo::start(side, f::limits().forge, silent, &mut out);
    let Some(forge::Request::Send { request, ask: jig_client::Ask::Family { scope, record }, .. }) = out.pop() else {
        panic!("start sent")
    };
    assert_eq!(scope, c::Scope::Deployment);
    assert_eq!(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request(), &w::Request::StartDemo);
    assert!(out.is_empty());
    request
}
fn answered(side: &mut forge::Side, request: skein_lib::Token, outcome: w::Outcome) {
    let value = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap();
    let mut out = Queue::with_capacity(4);
    demo::told(
        side,
        &Told::Answered {
            key: jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() },
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&value),
            })),
        },
        f::limits().forge,
        &mut out,
    );
    let Some(forge::Request::Client { asked: jig_client::Event::Read { .. } }) = out.pop() else {
        panic!("start answer rereads the demo")
    };
    assert!(out.is_empty());
}
#[test]
fn demo_state_decodes_and_start_is_single_across_pages_with_a_new_retry_key() {
    let mut side = demo_side(w::DemoState::NotStarted);
    assert_eq!(side.demo().unwrap().state, demo::DemoState::NotStarted);
    assert!(!side.demo_running());
    let first = started(&mut side, false);
    assert!(side.demo_running());
    assert!(side.demo_starting());
    let mut out = Queue::with_capacity(4);
    forge::step(
        &mut side,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: f::limits().forge },
        forge::Event::Left,
        &mut out,
    );
    demo::start(&mut side, f::limits().forge, false, &mut out);
    assert!(out.is_empty());
    answered(
        &mut side,
        first,
        w::Outcome::StartFailed(
            w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::Account }).unwrap(),
        ),
    );
    assert_eq!(side.demo_failure(), Some(&demo::StartFailure::Account));
    assert!(side.demo_running());
    assert!(!side.demo_starting());
    let retry = started(&mut side, false);
    assert_ne!(first, retry);
    assert!(side.demo_failure().is_none());
}
#[test]
fn once_per_session_running_start_is_silent_and_its_failure_is_shown() {
    let mut side = demo_side(w::DemoState::Running);
    let mut out = Queue::with_capacity(4);
    demo::configure(&mut side, f::limits().forge, &mut out);
    let Some(forge::Request::Send { request, .. }) = out.pop() else { panic!("silent start") };
    assert!(!side.demo_starting());
    answered(
        &mut side,
        request,
        w::Outcome::StartFailed(
            w::StartFailed::new(&w::CEILINGS, w::StartFailedParts { why: w::StartFailure::NotConnected }).unwrap(),
        ),
    );
    demo::configure(&mut side, f::limits().forge, &mut out);
    assert!(out.is_empty());
    assert_eq!(side.demo_failure(), Some(&demo::StartFailure::NotConnected));
}
#[test]
fn running_unknown_and_not_offered_each_read_the_demo_again() {
    for outcome in [
        w::Outcome::Running,
        w::Outcome::Unknown,
        w::Outcome::StartRefused(
            w::StartRefused::new(&w::CEILINGS, w::StartRefusedParts { why: w::StartRefusal::NotOffered }).unwrap(),
        ),
    ] {
        let mut side = demo_side(w::DemoState::NotStarted);
        let request = started(&mut side, false);
        answered(&mut side, request, outcome);
        assert!(!side.demo_starting());
        assert!(side.demo_failure().is_none());
    }
}
#[test]
fn another_operations_outcome_is_not_a_start_answer() {
    let value = jig_client::Answered::Family(jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::SettingUp }).unwrap()),
    });
    assert!(demo::answer(&value, &f::limits().forge).is_none());
}

fn up(root: &mut crate::Domain, up: jig_client::Up) -> Queue<crate::Request> {
    event(root, crate::Event::Client { event: jig_client::Event::Up { up } })
}
fn deliver_accounts(
    root: &mut crate::Domain,
    watch: skein_lib::Token,
    document: c::OwnerAccounts,
) -> Queue<crate::Request> {
    up(
        root,
        jig_client::Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::OwnerAccounts(Box::new(document))),
        },
    )
}
fn ending_accounts(other_usable: bool) -> c::OwnerAccounts {
    accounts(Some(PROVIDER + 1), other_usable, false)
}
fn deployment(demo_account: bool) -> c::OwnerDeployment {
    let link = jig_client_protocol_link::Limits::new(f::limits().client.domain, 8).unwrap();
    let mut document = jig_client_protocol_link::translate::documents::owner_deployment(
        jig_client_test_records::owner_deployment_record(),
        &link,
    )
    .unwrap();
    document.accounts = List::with_capacity(1);
    if demo_account {
        document
            .accounts
            .push(c::OwnerAccount {
                number: 42,
                place: 0,
                provider: Some(PROVIDER),
                provider_name: None,
                manage_usage: None,
                display: c::DocumentText::complete(f::acts::text(b"Demo")),
                holder: c::OwnerAccountHolder::Deployment,
                uses: None,
                plan: false,
                sources: List::with_capacity(0),
                generation: None,
                state: c::OwnerAccountState::Granted,
                usable: true,
                expiring: false,
                attention: false,
                spent_until: None,
                runs: List::with_capacity(0),
                waiting: List::with_capacity(0),
                cut: false,
            })
            .unwrap();
    }
    document
}
fn deliver_deployment(
    root: &mut crate::Domain,
    watch: skein_lib::Token,
    document: c::OwnerDeployment,
) -> Queue<crate::Request> {
    up(
        root,
        jig_client::Up::Watched {
            watch,
            event: c::WatchEvent::Delivered(c::Delivery::OwnerDeployment(Box::new(document))),
        },
    )
}
fn deployment_watch(out: &mut Queue<crate::Request>) -> Option<skein_lib::Token> {
    let mut watch = None;
    let mut rest = Queue::with_capacity(out.len());
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down:
                        jig_client::Down::Open { watch: token, subject: jig_client::Watchable::Jig(c::Subject::Deployment) },
                },
        } = request
        {
            assert!(watch.replace(token).is_none(), "one deployment watch");
        } else {
            rest.push(request);
        }
    }
    *out = rest;
    watch
}
fn closed(out: &Queue<crate::Request>, watch: skein_lib::Token) -> bool {
    for request in out {
        if let crate::Request::Client {
            request: jig_client::Request::Down { down: jig_client::Down::Close { watch: token } },
        } = request
            && *token == watch
        {
            return true;
        }
    }
    false
}
fn ending_root(role: Option<c::HeldRole>) -> (crate::Domain, skein_lib::Token, skein_lib::Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, account) = f::start_accounts(&mut root, List::with_capacity(0));
    let mut frame = f::chats::me();
    frame.deployment_role = role;
    let mut out = up(
        &mut root,
        jig_client::Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) },
    );
    let mut forges = None;
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { .. } },
                },
        } = request
        {
            forges = Some(watch);
        }
    }
    demo::told(&mut root.forge, &Told::Link { link: Link::Offline }, f::limits().forge, &mut Queue::with_capacity(1));
    let handle = read(&mut root.forge);
    crate::route::told(
        &mut root,
        &f::env(&f::limits()),
        Told::Page { handle, page: page() },
        &mut Queue::with_capacity(crate::max_out(&f::limits())),
    );
    (root, account, forges.expect("deployment forges"))
}
fn forge_rows(root: &mut crate::Domain, watch: skein_lib::Token, connected: bool) -> Queue<crate::Request> {
    let mut forges = List::with_capacity(1);
    let connection = if connected {
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"bot".as_slice().into() }).unwrap(),
        )
    } else {
        w::Connection::Connecting
    };
    forges
        .push(
            w::ForgeRow::new(
                &w::CEILINGS,
                w::ForgeRowParts {
                    first: None,
                    number: wire::FORGE + 1,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Owned,
                    connection,
                    offer: w::Offer::NotOffered,
                },
            )
            .unwrap(),
        )
        .unwrap();
    up(
        root,
        jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges }).unwrap()),
            },
        },
    )
}
fn removed(out: &mut Queue<crate::Request>) -> List<skein_lib::Token> {
    let mut attempts = List::with_capacity(4);
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
        {
            assert_eq!(
                *ask,
                c::Ask::AccountChange {
                    holder: c::AccountHolder::Deployment,
                    change: c::AccountChange::Remove { account: 42 }
                }
            );
            attempts.push(attempt).unwrap();
        }
    }
    attempts
}
#[test]
fn the_model_end_removes_the_deployment_demo_account_once_and_drops_its_answer() {
    for answer in [
        None,
        Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountRemoved {
            confirmed: true,
            failed_hosts: Box::new([]),
        })))),
        Some(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Role { held: Some(c::HeldRole::Member) },
            at: None,
        })),
    ] {
        let (mut root, account, forges) = ending_root(Some(c::HeldRole::Owner));
        assert!(deployment_watch(&mut deliver_accounts(&mut root, account, ending_accounts(false))).is_none());
        forge_rows(&mut root, forges, false);
        assert!(root.demo().unwrap().1.model);
        let mut out = deliver_accounts(&mut root, account, ending_accounts(true));
        let watch = deployment_watch(&mut out).expect("owner watches deployment at the first end");
        assert!(removed(&mut out).is_empty(), "wait for deployment delivery");
        assert!(deliver_accounts(&mut root, account, ending_accounts(true)).is_empty(), "watch opened once");
        let mut out = deliver_deployment(&mut root, watch, deployment(true));
        assert!(closed(&out, watch), "remove drops the deployment watch");
        let attempts = removed(&mut out);
        assert_eq!(attempts.len(), 1);
        assert!(!root.demo().unwrap().1.model);
        let shown = crate::shown(&root);
        let attempt = *attempts.get(0).unwrap();
        up(
            &mut root,
            jig_client::Up::Answer { attempt, envelope: skein_lib::RequestEnvelope::Final, answer, why: None },
        );
        assert_eq!(crate::shown(&root), shown, "cleanup outcome has no view");
        assert!(removed(&mut deliver_accounts(&mut root, account, ending_accounts(false))).is_empty());
        assert!(!root.demo().unwrap().1.model, "the ended half never returns");
        assert!(removed(&mut forge_rows(&mut root, forges, true)).is_empty(), "the second end sends no remove");
    }
}
#[test]
fn the_forge_end_rereads_once_removes_the_account_and_none_clears_the_demo() {
    let (mut root, account, forges) = ending_root(Some(c::HeldRole::Owner));
    assert!(deployment_watch(&mut deliver_accounts(&mut root, account, ending_accounts(false))).is_none());
    assert!(forge_rows(&mut root, forges, false).is_empty());
    let mut out = forge_rows(&mut root, forges, true);
    let mut read = None;
    let mut remaining = Queue::with_capacity(out.len());
    while let Some(request) = out.pop() {
        if let crate::Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Read { read: token, what: Readable::Family { record, .. } },
                },
        } = &request
        {
            assert_eq!(wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read(), &w::Read::Demo);
            assert!(read.replace(*token).is_none());
        } else {
            remaining.push(request);
        }
    }
    let watch = deployment_watch(&mut remaining).expect("forge end opens deployment watch");
    assert!(removed(&mut remaining).is_empty(), "wait for deployment");
    assert!(forge_rows(&mut root, forges, true).is_empty(), "unchanged connected rows never repeat the read");
    up(
        &mut root,
        jig_client::Up::ReadDone {
            read: read.expect("Demo reread"),
            result: Readout::Family { family: wire::FORGE, record: None },
        },
    );
    assert!(root.forge().demo().is_none());
    assert!(root.demo().is_none());
    let mut out = deliver_deployment(&mut root, watch, deployment(true));
    assert!(closed(&out, watch));
    assert_eq!(removed(&mut out).len(), 1, "kept provider survives the none read");
    assert!(removed(&mut deliver_accounts(&mut root, account, ending_accounts(true))).is_empty());
}
#[test]
fn cleanup_watches_only_for_an_owner_at_an_end_and_drops_a_document_without_the_account() {
    for role in [None, Some(c::HeldRole::Member), Some(c::HeldRole::Maintainer), Some(c::HeldRole::Owner)] {
        let (mut root, account, forges) = ending_root(role);
        assert!(
            deployment_watch(&mut deliver_accounts(&mut root, account, ending_accounts(false))).is_none(),
            "before end"
        );
        let mut out = deliver_accounts(&mut root, account, ending_accounts(true));
        let watch = deployment_watch(&mut out);
        assert!(removed(&mut out).is_empty());
        if role == Some(c::HeldRole::Owner) {
            let watch = watch.expect("owner watch");
            let mut out = deliver_deployment(&mut root, watch, deployment(false));
            assert!(closed(&out, watch), "absent account drops watch too");
            assert!(removed(&mut out).is_empty());
            assert!(deployment_watch(&mut deliver_accounts(&mut root, account, ending_accounts(true))).is_none());
            assert!(deployment_watch(&mut forge_rows(&mut root, forges, true)).is_none(), "second end cannot reopen");
        } else {
            assert!(watch.is_none(), "non-owner never watches deployment");
        }
    }
}
#[test]
fn cleanup_needs_a_running_answer_and_positive_evidence_of_an_end() {
    for case in ["none", "not started", "missing", "cut"] {
        let (mut root, account, _) = ending_root(Some(c::HeldRole::Owner));
        if case == "none" || case == "not started" {
            demo::told(
                &mut root.forge,
                &Told::Link { link: Link::Offline },
                f::limits().forge,
                &mut Queue::with_capacity(1),
            );
            let handle = read(&mut root.forge);
            let page = if case == "none" {
                Readout::Family { family: wire::FORGE, record: None }
            } else {
                let mut parts = wire::decode_demo_answer(&answer(CUE), &w::CEILINGS).unwrap().into_parts();
                parts.state = w::DemoState::NotStarted;
                Readout::Family {
                    family: wire::FORGE,
                    record: Some(wire::demo_answer(&w::DemoAnswer::new(&w::CEILINGS, parts).unwrap())),
                }
            };
            demo::told(&mut root.forge, &Told::Page { handle, page }, f::limits().forge, &mut Queue::with_capacity(1));
        }
        let mut document = ending_accounts(case != "cut");
        document.cut = case == "cut";
        let mut out = if case == "missing" {
            event(&mut root, crate::Event::Went { place: crate::Place::Address(crate::Address::Chats) })
        } else {
            deliver_accounts(&mut root, account, document)
        };
        assert!(deployment_watch(&mut out).is_none(), "{case}");
        assert!(removed(&mut out).is_empty(), "{case}");
    }
}
