use super::fixtures;
use jig_client::contract::*;
use skein_lib::{Env, List, Queue, Time, Token, Wall};
use skein_ui::{Builder, Role};
use temper_client_domain::{Domain, Request};
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
        dependencies: List::with_capacity(EMPTY),
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
    assert!(fixtures::find(&tree, Role::Paragraph, b"3 delegates").is_some());
}
#[test]
fn a_chats_row_links_to_its_chat_by_number() {
    let (tree, bindings) = tree(&line());
    let mut bound = None;
    for node in tree.nodes() {
        if node.role == Role::Link && node.name.as_deref() == Some(b"T9") {
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
                channel: Token::new(1),
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

fn open_chats(root: &mut Domain, env: &Env<temper_client_domain::Limits>, out: &mut Queue<Request>) -> (Token, Token) {
    use temper_client_domain::{Address, Event};
    temper_client_domain::step(
        root,
        env,
        Event::Went { place: temper_client_domain::Place::Address(Address::Chats) },
        out,
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
    (watch.expect("live watch"), read.expect("closed read"))
}
fn deliver_live(
    root: &mut Domain,
    env: &Env<temper_client_domain::Limits>,
    out: &mut Queue<Request>,
    watch: Token,
    rows: List<TaskLine>,
) {
    use temper_client_domain::Event;
    temper_client_domain::step(
        root,
        env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: WatchEvent::Delivered(Delivery::Tasks(Box::new(Tasks { rows, more: false }))),
                },
            },
        },
        out,
    );
}

pub(super) fn listed_root() -> Domain {
    let mut ended = line();
    ended.chip.task = 8;
    ended.ended_at = Some(Wall::EPOCH);
    listed(&[line()], &[ended], false)
}
fn listed(live: &[TaskLine], closed: &[TaskLine], more: bool) -> Domain {
    use temper_client_domain::{Address, Event};
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
    let (watch, read) = open_chats(&mut root, &env, &mut out);

    let mut live_rows = List::with_capacity(u32::try_from(live.len()).unwrap());
    for line in live {
        live_rows.push(line.clone()).expect("live row");
    }
    deliver_live(&mut root, &env, &mut out, watch, live_rows);
    let mut closed_rows = List::with_capacity(u32::try_from(closed.len()).unwrap());
    for line in closed {
        closed_rows.push(line.clone()).expect("closed row");
    }
    // A live update refreshes the first ended page; answer the latest read.
    let mut read = read;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down {
                    down: jig_client::Down::Read { read: token, what: jig_client::Readable::Jig(Read::Tasks { .. }) },
                },
        } = request
        {
            read = token;
        }
    }
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: EndedGroup::Rows {
                            rows: closed_rows,
                            after: if more { Some(Cursor::new(Box::from(*b"next"), 8).unwrap()) } else { None },
                            cut: false,
                        },
                    })))),
                },
            },
        },
        &mut out,
    );
    root
}
#[test]
fn chats_have_one_list_or_the_empty_sentence() {
    let empty = fixtures::page_view(&fixtures::chats_root());
    assert!(fixtures::find(empty.tree(), Role::Heading { level: 2 }, b"Your chats").is_some());
    assert!(fixtures::styles(empty.tree(), Role::List).is_empty());
    assert!(fixtures::find(empty.tree(), Role::Paragraph, b"No chats yet.").is_some());
    let view = fixtures::page_view(&listed_root());
    assert_eq!(fixtures::styles(view.tree(), Role::List).as_ref(), [Box::from([crate::look::ROWS])]);
    let list = fixtures::find(view.tree(), Role::List, b"Your chats").expect("named chats list");
    let nodes = view.tree().nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.id == list {
            assert_eq!(node.role, Role::List);
            let end = index.checked_add(usize::try_from(node.size).expect("list size")).expect("list end");
            let mut child = index + 1;
            for task in [8, 9] {
                assert_eq!(nodes[child].role, Role::Item);
                assert_eq!(nodes[child].key.expect("chat line key").a, task);
                child = child.checked_add(usize::try_from(nodes[child].size).expect("line size")).expect("next line");
            }
            assert_eq!(child, end, "both lines belong to the named list");
        }
    }
}

#[test]
fn an_empty_demo_start_has_its_invitation_and_your_chats() {
    let root = fixtures::demo_root(temper_client_domain::Address::Chats, fixtures::PROJECT, true);
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Live chats").is_none());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Closed chats").is_none());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Your chats").is_some());
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"No chats yet.").is_some());
    let mut heading = None;
    for node in view.tree().nodes() {
        if node.role == (Role::Heading { level: 1 }) {
            assert!(heading.is_none(), "one level-1 heading");
            heading = node.name.as_deref();
        }
    }
    assert_eq!(heading, Some(b"What shall we change?".as_slice()));
}

#[test]
fn a_live_demo_chat_is_listed_under_your_chats() {
    use temper_client_domain::Address;
    let mut root = fixtures::demo_root(
        Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        fixtures::PROJECT,
        true,
    );
    let limits = fixtures::domain_limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits));
    let (watch, _) = open_chats(&mut root, &env, &mut out);
    let mut rows = List::with_capacity(1);
    rows.push(line()).expect("live row");
    deliver_live(&mut root, &env, &mut out, watch, rows);
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Your chats").is_some());
    assert!(fixtures::find(view.tree(), Role::Link, b"T9").is_some());
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"Hello").is_some());
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Closed chats").is_none());
    assert!(fixtures::find(view.tree(), Role::Button, b"More closed chats").is_none());
}

#[test]
fn a_chats_line_counts_the_delegates_it_waits_on() {
    let mut line = line();
    line.waits_on = WaitsOn::Delegates;
    let (tree, _) = tree(&line);
    assert!(fixtures::find(&tree, Role::Status, b"waiting on 3 delegates").is_some());
    assert_eq!(fixtures::styles(&tree, Role::Status).as_ref(), [Box::from([crate::look::STATUS_QUIET])]);
}

#[test]
fn a_chats_line_keeps_jigs_word_for_news_without_a_wake_policy() {
    let mut line = line();
    line.waits_on = WaitsOn::News;
    let ChipPhase::Owner { details, .. } = line.chip.details.as_mut() else { panic!("owner") };
    details.wake.words = WakeRule::Immediate;
    let (tree, _) = tree(&line);
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_some());
}

fn filter(root: &mut Domain, filter: temper_client_domain::pages::chats::Filter) {
    fixtures::root_event(
        root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::ChatFilter { filter }),
        },
    );
}
fn numbers(tree: &skein_ui::Tree) -> Box<[u64]> {
    let mut numbers = List::with_capacity(u32::try_from(tree.nodes().len()).expect("tree nodes fit"));
    for node in tree.nodes() {
        if node.role == Role::Item
            && let Some(key) = node.key
        {
            numbers.push(key.a).expect("one key per node");
        }
    }
    numbers.into_boxed()
}
fn timed(task: u64, activity: u64, ended: Option<u64>) -> TaskLine {
    let mut line = line();
    line.chip.task = task;
    line.last_activity = Wall::from_nanos(activity);
    if let Some(ended) = ended {
        line.ended_at = Some(Wall::from_nanos(ended));
    }
    line
}
#[test]
fn one_list_merges_activity_and_end_times_with_lower_numbers_first_on_a_tie() {
    // Closed T8's activity is newer than everything, but its end is the tie.
    let root =
        listed(&[timed(9, 90, None), timed(6, 40, None)], &[timed(8, 200, Some(40)), timed(7, 0, Some(100))], false);
    let view = fixtures::page_view(&root);
    assert_eq!(numbers(view.tree()).as_ref(), [7, 9, 6, 8]);
    assert_eq!(fixtures::styles(view.tree(), Role::List).as_ref(), [Box::from([crate::look::ROWS])]);
}
#[test]
fn each_filter_selects_its_lines_and_presses_its_bound_button() {
    use temper_client_domain::{
        Action,
        pages::{Act, chats::Filter},
    };
    for (selected, name, expected) in [
        (Filter::All, b"All".as_slice(), [8, 9].as_slice()),
        (Filter::Live, b"Live", [9].as_slice()),
        (Filter::Closed, b"Closed", [8].as_slice()),
    ] {
        let mut root = listed_root();
        filter(&mut root, selected);
        let mut view = fixtures::page_view(&root);
        assert_eq!(numbers(view.tree()).as_ref(), expected);
        for (filter, label) in [(Filter::All, b"All".as_slice()), (Filter::Live, b"Live"), (Filter::Closed, b"Closed")]
        {
            let id = fixtures::find(view.tree(), Role::Button, label).expect("filter button");
            for node in view.tree().nodes() {
                if node.id == id {
                    assert_eq!(node.states.as_slice().contains(&skein_ui::State::Pressed), label == name);
                }
            }
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::ChatFilter { filter }))
            );
        }
        for task in expected {
            let label = match *task {
                8 => b"T8",
                9 => b"T9",
                _ => panic!("fixture task"),
            };
            let id = fixtures::find(view.tree(), Role::Link, label).expect("TN link");
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address: temper_client_domain::Address::Chat { task: *task } }))
            );
        }
    }
}
#[test]
fn empty_filters_say_which_chats_are_missing() {
    use temper_client_domain::pages::chats::Filter;
    for (selected, words) in [
        (Filter::All, b"No chats yet.".as_slice()),
        (Filter::Live, b"No live chats."),
        (Filter::Closed, b"No closed chats."),
    ] {
        let mut root = listed(&[], &[], false);
        filter(&mut root, selected);
        let view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Paragraph, words).is_some());
        assert!(fixtures::styles(view.tree(), Role::List).is_empty());
    }
    for (selected, live, ended, words) in
        [(Filter::Live, false, true, b"No live chats.".as_slice()), (Filter::Closed, true, false, b"No closed chats.")]
    {
        let active = [line()];
        let closed = [timed(8, 0, Some(0))];
        let mut root = listed(if live { &active } else { &[] }, if ended { &closed } else { &[] }, false);
        filter(&mut root, selected);
        assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Paragraph, words).is_some());
    }
}
#[test]
fn more_closed_chats_is_bound_only_under_all_or_closed_with_an_older_page() {
    use temper_client_domain::{
        Action,
        pages::{Act, chats::Filter},
    };
    for more in [false, true] {
        for selected in [Filter::All, Filter::Live, Filter::Closed] {
            let mut root = listed(&[line()], &[timed(8, 0, Some(0))], more);
            filter(&mut root, selected);
            let mut view = fixtures::page_view(&root);
            let button = fixtures::find(view.tree(), Role::Button, b"More closed chats");
            assert_eq!(button.is_some(), more && selected != Filter::Live);
            if let Some(id) = button {
                assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(Act::ClosedChats)));
            }
        }
    }
}
#[test]
fn delegate_counts_are_plural_and_absent_at_zero() {
    for (count, expected) in [(0, None), (1, Some(b"1 delegate".as_slice())), (2, Some(b"2 delegates".as_slice()))] {
        let mut line = line();
        let ChipPhase::Owner { details, .. } = line.chip.details.as_mut() else { panic!("owner") };
        details.live_delegates = count;
        let (tree, _) = tree(&line);
        if let Some(words) = expected {
            assert!(fixtures::find(&tree, Role::Paragraph, words).is_some());
        } else {
            let mut paragraphs = 0_u32;
            for node in tree.nodes() {
                if node.role == Role::Paragraph {
                    paragraphs += 1;
                }
            }
            assert_eq!(paragraphs, 3, "title, spend and time");
        }
    }
}
