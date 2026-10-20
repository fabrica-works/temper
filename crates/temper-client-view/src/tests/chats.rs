use super::fixtures;
use jig_client::contract::*;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role};
const TASK: u64 = 9;
const TEXT: u32 = 32;
const SPENT: u64 = 7;
const BELOW: u64 = 2;
const BUDGET: u64 = 100;
const RESERVED: u64 = 0;
const LEFT: u64 = 91;
const LABEL: u32 = 1;
const DELEGATES: u32 = 3;
const EMPTY: u32 = 0;
const ACTIVITY: u64 = 1;
pub(super) fn line() -> TaskLine {
    let budget = OwnerBudget {
        numbers: OwnerNumbers { budget: BUDGET, spent: SPENT, spent_below: BELOW, reserved: RESERVED },
        left: LEFT,
        funder: Funder::Task(TASK),
        deadline: None,
    };
    let details = OwnerChipDetails {
        project: fixtures::PROJECT,
        executor: OwnerExecutor::Person(PersonAddress::Party(73)),
        phase_since: Wall::EPOCH,
        run_since: None,
        attempt: None,
        retry: None,
        wake: OwnerWakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        person_wait: None,
        hold: None,
        budget: budget.clone(),
        activity: ACTIVITY,
        resources: List::with_capacity(EMPTY),
        resources_cut: false,
        resources_observed: false,
        procedure: None,
        live_delegates: DELEGATES,
        closing_connectors: None,
        ending: None,
        ended_at: None,
        observation: None,
    };
    TaskLine {
        chip: Chip {
            task: TASK,
            title: DocumentText::complete(Text::new(Box::from(*b"Hello"), TEXT).expect("title")),
            executor: Executor::Party { party: 73 },
            phase: Phase::Idle,
            details: Box::new(ChipPhase::Owner {
                phase: OwnerChipPhase::Active(RunStage::Idle),
                details: Box::new(details),
            }),
        },
        project: fixtures::PROJECT,
        requester: OwnerRequester::Person(73),
        tracked: None,
        label: LABEL,
        flags: TaskFlags { conversation: true, listed: false, retention: Retention::Kept },
        changed: Wall::EPOCH,
        last_activity: Wall::EPOCH,
        ended_at: None,
        budget,
        waits_on: WaitsOn::Nothing,
        phases: OwnerPhaseCounts { running: DELEGATES, ..OwnerPhaseCounts::ZERO },
        held_below: EMPTY,
        conversation: None,
    }
}
pub(super) fn tree(line: &TaskLine) -> (skein_ui::Tree, crate::frame::Bindings) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::chats::row(
        &mut b,
        &fixtures::root_in_project(Some(fixtures::PROJECT)),
        line,
        &fixtures::words(),
        &mut bindings,
    );
    (b.finish(), bindings)
}
#[test]
fn a_chats_row_says_its_status_from_jigs_facts() {
    let (tree, _) = tree(&line());
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_some());
}
#[test]
fn a_chats_row_says_its_spend_and_delegates() {
    let (tree, _) = tree(&line());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Spent 7; delegates spent 2").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Live delegates 3").is_some());
}
#[test]
fn a_chats_row_links_to_its_chat_by_number() {
    let (tree, bindings) = tree(&line());
    let mut bound = None;
    for node in tree.nodes() {
        if node.role == Role::Link && node.name.as_deref() == Some(b"Chat 9") {
            bound = node.binding;
        }
    }
    let binding = bound.expect("bound link");
    assert_eq!(
        bindings.decode(binding.token, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: temper_client_domain::Address::Chat { task: TASK }
        }))
    );
}
#[test]
fn the_chat_composer_waits_for_the_person_and_project() {
    let mut root = temper_client_domain::unlinked(&fixtures::domain_limits(), fixtures::SEED);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Went {
            place: temper_client_domain::Place::Address(temper_client_domain::Address::Chats),
        },
    );
    let mut view = fixtures::view();
    assert_eq!(fixtures::find(fixtures::build(&mut view, &root), Role::TextArea, b"Start a chat"), None);
}

#[test]
fn enter_in_the_start_composer_starts_a_chat() {
    let mut view = fixtures::page_view(&fixtures::chats_root());
    let composer = fixtures::find(view.tree(), Role::TextArea, b"Start a chat").expect("composer");
    assert_eq!(
        crate::act(&mut view, composer, skein_ui::Act::Submit),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::StartChat)),
    );
}
#[test]
fn a_line_break_in_the_start_composer_is_kept() {
    let mut root = fixtures::chats_root();
    assert_eq!(&*fixtures::type_into(&mut root, b"Start a chat", b"Hello\nthere"), b"Hello\nthere");
}
#[test]
fn a_chats_row_carries_the_row_token() {
    let (tree, _) = tree(&line());
    assert_eq!(fixtures::styles(&tree, Role::Item).as_ref(), [Box::from([crate::look::ROW])]);
}

pub(super) fn status_rows() -> [(TaskLine, skein_ui::Style); 11] {
    use crate::look;
    let mut rows = [
        (line(), look::STATUS_YOURS),
        (line(), look::STATUS_HELD),
        (line(), look::STATUS_RETRYING),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_ACTIVE),
        (line(), look::STATUS_QUIET),
        (line(), look::STATUS_QUIET),
        (line(), look::STATUS_QUIET),
        (line(), look::STATUS_QUIET),
    ];
    for (index, (line, _)) in rows.iter_mut().enumerate() {
        let ChipPhase::Owner { phase, details } = line.chip.details.as_mut() else { panic!("owner") };
        details.ended_at = Some(Wall::EPOCH);
        details.retry = Some(RetryNotice::OwnerFailed { class: HoldFailure::Lost, count: 2, allowance: 3 });
        *phase = match index {
            1 => OwnerChipPhase::Held { reason: HoldCategory::Stopped, revision: 1 },
            2 => OwnerChipPhase::Active(RunStage::BackingOff { until: Wall::EPOCH }),
            3 => OwnerChipPhase::Active(RunStage::Due),
            4..=6 => OwnerChipPhase::Active(RunStage::Running { attempt: 1 }),
            9 => OwnerChipPhase::Closing(ClosingStage::Delegates),
            10 => OwnerChipPhase::Ended,
            _ => OwnerChipPhase::Active(RunStage::Idle),
        };
        if index == 0 || index == 7 {
            line.waits_on = WaitsOn::Party {
                whom: Whom::Person(if index == 0 { 73 } else { 74 }),
                entries: 1,
                task: TASK,
                kind: WaitedFor::Question,
            };
        }
        if (4..=6).contains(&index) {
            details.observation = Some(OwnerChipObservation {
                channel: skein_lib::Token::new(1),
                work: match index {
                    4 => OwnerChipWork::Writing,
                    5 => OwnerChipWork::Tool { call: Box::from(*b"call"), tool: Box::from(*b"search") },
                    _ => OwnerChipWork::Active,
                },
                since: Wall::EPOCH,
                reported_at: Wall::EPOCH,
                started: None,
            });
        }
    }
    rows
}
#[test]
fn each_status_is_styled_by_its_meaning() {
    for (line, token) in status_rows() {
        let (tree, _) = tree(&line);
        assert_eq!(fixtures::styles(&tree, Role::Status).as_ref(), [Box::from([token])]);
    }
}
#[test]
fn the_entry_contains_the_composer_hint_and_start_in_order() {
    let view = fixtures::page_view(&fixtures::chats_root());
    let nodes = view.tree().nodes();
    let mut entry = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.styles.as_slice().contains(&crate::look::ENTRY) {
            entry = Some(index);
        }
    }
    let entry = entry.expect("entry");
    let children = &nodes[entry + 1..entry + usize::try_from(nodes[entry].size).expect("size")];
    assert_eq!(nodes[entry].role, Role::Slot);
    assert_eq!(children[0].role, Role::TextArea);
    assert!(children[0].states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
    let mut hint = None;
    let mut start = None;
    for (index, node) in children.iter().enumerate() {
        if node.styles.as_slice().contains(&crate::look::HINT) {
            hint = Some(index);
        }
        if node.role == Role::Button && node.name.as_deref() == Some(b"Start") {
            start = Some(index);
        }
    }
    let hint = hint.expect("hint");
    let start = start.expect("start");
    assert!(hint < start);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Enter to start").is_some());
}
#[test]
fn a_row_shows_the_real_activity_or_end_time() {
    let mut live = line();
    live.last_activity = Wall::from_nanos(1_792_523_940_000_000_000);
    let (shown, _) = tree(&live);
    assert!(skein_lib::bytes::find(&fixtures::text(&shown), b"10-20 19:19Z").is_some());
    live.ended_at = Some(Wall::EPOCH);
    let (shown, _) = tree(&live);
    assert!(skein_lib::bytes::find(&fixtures::text(&shown), b"01-01 00:00Z").is_some());
}

pub(super) fn listed_root() -> temper_client_domain::Domain {
    use skein_lib::{Env, Queue, Time};
    use temper_client_domain::{Address, Event, Request};
    let limits = fixtures::domain_limits();
    let (mut root, me_watch, _) =
        fixtures::root_at(Address::Inbox { filter: temper_client_domain::address::Filter::ALL }, &limits);
    let template = fixtures::chat::messaging_root();
    let me = template.pages().me(template.client()).expect("me").clone();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits));
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: WatchEvent::Delivered(Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Went { place: temper_client_domain::Place::Address(Address::Chats) },
        &mut out,
    );
    let mut watch = None;
    let mut read = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down {
                    down:
                        jig_client::Down::Open { watch: token, subject: jig_client::Watchable::Jig(Subject::Tasks { .. }) },
                },
        } = request
        {
            watch = Some(token);
        } else if let Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Read { read: token, what: jig_client::Readable::Jig(Read::Tasks { .. }) },
                },
        } = request
        {
            read = Some(token);
        }
    }

    let mut live = List::with_capacity(1);
    live.push(line()).expect("live row");
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("live watch"),
                    event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows: live, more: false }))),
                },
            },
        },
        &mut out,
    );
    let mut closed = List::with_capacity(1);
    let mut ended = line();
    ended.chip.task = 8;
    ended.ended_at = Some(Wall::EPOCH);
    closed.push(ended).expect("closed row");
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read: read.expect("closed read"),
                    result: jig_client::Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: EndedGroup::Rows { rows: closed, after: None, cut: false },
                    })))),
                },
            },
        },
        &mut out,
    );
    root
}
#[test]
fn nonempty_sections_have_lists_and_empty_sections_have_none() {
    let empty = fixtures::page_view(&fixtures::chats_root());
    assert!(fixtures::styles(empty.tree(), Role::List).is_empty());
    let view = fixtures::page_view(&listed_root());
    assert_eq!(
        fixtures::styles(view.tree(), Role::List).as_ref(),
        [Box::from([crate::look::ROWS]), Box::from([crate::look::ROWS])]
    );
    let nodes = view.tree().nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::List {
            assert_eq!(nodes[index + 1].role, Role::Item);
        }
    }
}
