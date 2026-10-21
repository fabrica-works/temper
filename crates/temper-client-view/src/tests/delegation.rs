use super::fixtures;
use crate::look;
use jig_client::contract as c;
use skein_fake_person::{Face, Find, Found, Target, TreeFace};
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::Role;
use temper_client_domain::{Address, Domain, Event, Request};
const CHANGE: u64 = 2;
const CHAT: u64 = 1;
const EXCHANGES: u32 = 64;
const STEPS: u32 = 4;
const TITLE: &[u8] = b"Export widget's table as CSV";
const MERGE: &[u8] = b"1a2b3c4abcdef0123456789abcdef0123456789abcd";
const WAKE_SUMMARY: &[u8] =
    b"Begin the work your brief describes.\n\n2: Result from task 2:\n0000000000000003000000000000000000000000";
fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(Box::from(bytes), 256).expect("fixture text")
}
fn document(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(text(bytes))
}
fn change() -> c::Task {
    let mut task = fixtures::task::document();
    task.chip.task = CHANGE;
    task.chip.title = document(TITLE);
    task.chip.phase = c::Phase::Running;
    task.hold = None;
    task.result = None;
    task.result_contract = c::TaskContractDocument::Change { connector: fixtures::FORGE, kind: 1, words: 256 };
    let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner chip") };
    details.ending = None;
    task
}
fn landed() -> c::Task {
    let mut task = change();
    task.requester = c::OwnerRequester::Task(CHAT);
    task.result = Some(c::TaskResult::Ending(Box::new(c::OwnerChipEnding::Done(c::ResultValue::Change {
        kind: 1,
        part: c::OwnerPart {
            connector: fixtures::FORGE,
            kind: c::OwnerPartKind::Result,
            key: c::OwnerPartKey::Number(CHANGE),
        },
        words: document(MERGE),
    }))));
    task
}
fn limits() -> temper_client_domain::Limits {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.rows = 8;
    limits.client.domain.text = 256;
    limits
}
fn event(root: &mut Domain, event: Event, out: &mut Queue<Request>) {
    temper_client_domain::step(root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, event, out);
}
fn root(documents: &[c::Task], called: &[u64], heard: bool) -> Domain {
    root_activity(documents, activity(called, heard))
}
fn root_activity(documents: &[c::Task], activity: c::ActivityPage) -> Domain {
    let (mut root, _, _) = fixtures::root_at(Address::Chats, &limits());
    let mut out = Queue::with_capacity(temper_client_domain::max_out(&limits()));
    event(
        &mut root,
        Event::Went { place: temper_client_domain::Place::Address(Address::Chat { task: CHAT }) },
        &mut out,
    );
    for _ in 0..EXCHANGES {
        let Some(request) = out.pop() else { break };
        match request {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { .. }) },
                    },
            } => {
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(
                                    Box::new(activity.clone()),
                                ))),
                            },
                        },
                    },
                    &mut out,
                );
            }
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down:
                            jig_client::Down::Open {
                                watch,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { task: CHAT }),
                            },
                    },
            } => {
                let mut chat = change();
                chat.chip.task = CHAT;
                chat.activity = activity.rows.get(0).expect("newest activity").position;
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(chat))),
                            },
                        },
                    },
                    &mut out,
                );
            }
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down:
                            jig_client::Down::Open {
                                watch,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { task: number }),
                            },
                    },
            } => {
                for task in documents {
                    if task.chip.task != number {
                        continue;
                    }
                    event(
                        &mut root,
                        Event::Client {
                            event: jig_client::Event::Up {
                                up: jig_client::Up::Watched {
                                    watch,
                                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
                                },
                            },
                        },
                        &mut out,
                    );
                }
            }
            Request::Client { .. }
            | Request::Show { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => {}
        }
    }
    assert!(out.is_empty(), "delegation fixture settles");
    root
}
fn activity(called: &[u64], heard: bool) -> c::ActivityPage {
    let mut objects = List::with_capacity(u32::try_from(called.len()).expect("tasks fit"));
    for task in called {
        objects.push(c::ObjectKey::Task(*task)).expect("task");
    }
    let mut blocks = List::with_capacity(3);
    blocks.push(c::Block::Text(text(b"I'll delegate this change in demo/widget."))).expect("agent words");
    blocks
        .push(c::Block::Call(c::Call {
            number: 7,
            tool: text(b"delegate"),
            input: text(b"{}"),
            outcome: Some(text(b"Delegated tasks: 2")),
            failed: false,
            objects,
            parts: List::with_capacity(0),
        }))
        .expect("call");
    blocks.push(c::Block::Result { call: 7, summary: text(b"Delegated tasks: 2"), failed: false }).expect("result");
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).expect("message");
    let mut rows = List::with_capacity(4);
    if heard {
        for (position, from, words) in [(5, CHANGE, b"Carded result".as_slice()), (4, 3, b"Another task's result")] {
            rows.push(c::ActivityLine {
                position,
                at: Wall::EPOCH,
                event: c::Activity::Owner {
                    source: Box::new(c::OwnerActivityLine {
                        task: CHAT,
                        position,
                        at: Wall::EPOCH,
                        line: c::OwnerActivity::Heard(c::OwnerWord {
                            number: position,
                            from: c::OwnerRequester::Task(from),
                            kind: c::OwnerMessageKind::Result(c::OwnerWordResult::Report),
                            words: text(words),
                            at: Wall::EPOCH,
                            hits: 0,
                            eligible: false,
                        }),
                    }),
                    decoded: None,
                },
            })
            .expect("heard line");
        }
        let task = change();
        rows.push(c::ActivityLine {
            position: 3,
            at: Wall::EPOCH,
            event: c::Activity::Result {
                result: c::Result {
                    task: task.chip,
                    ending: c::Ending::Done,
                    value: Some(c::ResultValue::Report(document(b"Duplicate result"))),
                    at: Some(Wall::EPOCH),
                    details: None,
                    acts: List::with_capacity(0),
                },
            },
        })
        .expect("result line");
    }
    rows.push(c::ActivityLine {
        position: 2,
        at: Wall::EPOCH,
        event: c::Activity::Turn {
            name: c::TurnName { attempt: 1, number: 1 },
            turn: c::TurnRead::Decoded(c::Turn {
                messages,
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                cut: false,
            }),
        },
    })
    .expect("turn");
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
fn view(task: Option<c::Task>, called: &[u64], heard: bool) -> crate::View {
    view_documents(task.as_slice(), called, heard)
}
fn view_documents(documents: &[c::Task], called: &[u64], heard: bool) -> crate::View {
    let root = root(documents, called, heard);
    view_root(&root)
}
fn view_root(root: &Domain) -> crate::View {
    let mut view = crate::View::new(
        &crate::Limits { tree: fixtures::tree_limits() },
        &limits(),
        fixtures::words(),
        fixtures::FORGE,
    );
    fixtures::build(&mut view, root);
    view
}
fn find(view: &crate::View, target: Find) -> Option<skein_ui::NodeId> {
    let find = target.within(Box::new([
        Target::Role { role: Role::Log, name: Box::from(b"Conversation".as_slice()) },
        Target::Role { role: Role::Region, name: Box::from(b"Task 2".as_slice()) },
    ]));
    match TreeFace::new(view.tree()).find(&find) {
        Found::Node(id) => Some(id),
        Found::Absent | Found::Pending => None,
    }
}
fn status(view: &crate::View, name: &[u8], token: skein_ui::Style) {
    let id = find(view, Find::exactly_named(Role::Status, name)).expect("card status");
    assert_eq!(view.tree().find(id).expect("status").styles.as_slice(), [token]);
}
fn steps(view: &crate::View, done: bool) {
    let id = find(view, Find::exactly_named(Role::List, b"Steps")).expect("steps");
    let mut face = TreeFace::subtree(view.tree(), id).expect("list");
    let mut count = 0_u32;
    for node in view.tree().nodes() {
        if node.styles.as_slice().contains(&look::STEP) {
            count = count.checked_add(1).expect("step count");
            assert_eq!(node.styles.as_slice().contains(&look::STEP_DONE), done);
        }
    }
    assert_eq!(count, STEPS);
    for name in [b"Branch pushed".as_slice(), b"Pull request opened", b"CI passed", b"Landed"] {
        if let Found::Node(_) = face.find(&Find::exactly_named(Role::Item, name)) {
        } else {
            panic!("step absent");
        }
    }
}
#[test]
fn a_running_change_shows_its_card_at_its_call() {
    let mut view = view(Some(change()), &[CHANGE], false);
    let region = fixtures::find(view.tree(), Role::Region, b"Task 2").expect("task card");
    assert_eq!(view.tree().find(region).expect("card").styles.as_slice(), [look::CARD, look::TASK_CARD]);
    assert!(find(&view, Find::text("T2 · CHANGE".as_bytes())).is_some());
    status(&view, b"Running", look::STATUS_ACTIVE);
    assert!(find(&view, Find::exactly_named(Role::Heading { level: 3 }, TITLE)).is_some());
    steps(&view, false);
    let link = find(&view, Find::exactly_named(Role::Link, b"Task 2")).expect("task link");
    assert_eq!(
        crate::act(&mut view, link, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: Address::Task { task: CHANGE }
        }))
    );
    assert_eq!(work_slots(&view), 0);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Delegated tasks: 2").is_none());
    let said = fixtures::text(view.tree());
    assert!(
        skein_lib::bytes::find(&said, b"I'll delegate").expect("agent")
            < skein_lib::bytes::find(&said, "T2 · CHANGE".as_bytes()).expect("card")
    );
}
#[test]
fn a_landed_change_ticks_its_steps_and_says_its_merge_commit() {
    let view = view(Some(landed()), &[CHANGE], false);
    status(&view, b"Landed", look::STATUS_DONE);
    assert!(find(&view, Find::text("CI passed. Landed · merge commit 1a2b3c4".as_bytes())).is_some());
    steps(&view, true);
}
fn wake(called: bool, summary: &[u8]) -> c::ActivityPage {
    let mut blocks = List::with_capacity(1);
    blocks.push(c::Block::Result { call: 8, summary: text(summary), failed: false }).expect("wake result");
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Person, blocks }).expect("wake message");
    let mut rows = List::with_capacity(2);
    rows.push(c::ActivityLine {
        position: 3,
        at: Wall::EPOCH,
        event: c::Activity::Turn {
            name: c::TurnName { attempt: 2, number: 1 },
            turn: c::TurnRead::Decoded(c::Turn {
                messages,
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                cut: false,
            }),
        },
    })
    .expect("wake turn");
    if called {
        for line in &activity(&[CHANGE], false).rows {
            rows.push(line.clone()).expect("call turn");
        }
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}
#[test]
fn a_landed_change_shows_its_card_at_its_result_once_its_call_has_left() {
    let view = view_root(&root_activity(&[landed()], wake(false, WAKE_SUMMARY)));
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_some());
    assert!(find(&view, Find::text("T2 · CHANGE".as_bytes())).is_some());
    status(&view, b"Landed", look::STATUS_DONE);
    assert_eq!(work_slots(&view), 0);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Begin the work your brief describes.").is_none());
}
#[test]
fn a_card_shows_once_at_its_call_when_its_result_shares_the_window() {
    let view = view_root(&root_activity(&[landed()], wake(true, WAKE_SUMMARY)));
    let mut cards = 0_u32;
    for node in view.tree().nodes() {
        if node.role == Role::Region && node.name.as_deref() == Some(b"Task 2") {
            cards = cards.checked_add(1).expect("card count");
        }
    }
    assert_eq!(cards, 1);
    let said = fixtures::text(view.tree());
    let call = skein_lib::bytes::find(&said, b"I'll delegate").expect("call turn");
    let card = skein_lib::bytes::find(&said, "T2 · CHANGE".as_bytes()).expect("card");
    let wake = skein_lib::bytes::find(&said, b"Person").expect("wake turn");
    assert!(call < card && card < wake, "the card stays at its call");
    assert_eq!(work_slots(&view), 0);
}
#[test]
fn a_result_without_its_document_keeps_its_work() {
    let view = view_root(&root_activity(&[], wake(false, WAKE_SUMMARY)));
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_none());
    assert_eq!(work_slots(&view), 1);
    assert!(fixtures::find(view.tree(), Role::Paragraph, WAKE_SUMMARY).is_some());
}
#[test]
fn a_persons_result_keeps_its_work() {
    let summary = b"...\n2: Result from task 2:\nwords";
    let mut task = change();
    task.requester = c::OwnerRequester::Person(CHANGE);
    let view = view_root(&root_activity(&[task], wake(false, summary)));
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_none());
    assert_eq!(work_slots(&view), 1);
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"Tool completed").is_some());
    assert!(fixtures::find(view.tree(), Role::Paragraph, summary).is_some());
}
#[test]
fn a_partly_carded_result_keeps_its_work() {
    let summary = b"Result from task 2:\nlanded\n3: Result from task 3:\nwaiting";
    let view = view_root(&root_activity(&[landed()], wake(false, summary)));
    assert_eq!(work_slots(&view), 1);
    assert!(fixtures::find(view.tree(), Role::Paragraph, summary).is_some());
}
#[test]
fn each_state_says_its_words_in_its_status_look() {
    for (ending, held, name, token) in [
        (None, true, b"Held".as_slice(), look::STATUS_HELD),
        (Some(c::Ending::Done), false, b"Completed", look::STATUS_DONE),
        (Some(c::Ending::Failed { reason: None }), false, b"Failed", look::STATUS_HELD),
        (
            Some(c::Ending::Cancelled { reason: None, by: c::Actor::Party(1), at: None }),
            false,
            b"Cancelled",
            look::STATUS_QUIET,
        ),
        (Some(c::Ending::Deleted), false, b"Ended", look::STATUS_QUIET),
    ] {
        let mut task = change();
        if held {
            task.chip.phase = c::Phase::Held;
        }
        if let Some(ending) = ending {
            task.chip.details = Box::new(c::ChipPhase::Ended { how: ending, at: Wall::EPOCH });
        }
        let view = view(Some(task), &[CHANGE], false);
        status(&view, name, token);
        steps(&view, false);
    }
}
#[test]
fn a_report_has_no_steps() {
    let mut task = change();
    task.result_contract = c::TaskContractDocument::Report { words: 256 };
    task.result = Some(c::TaskResult::Ending(Box::new(c::OwnerChipEnding::Done(c::ResultValue::Report(document(
        b"A **useful** report",
    ))))));
    let view = view(Some(task), &[CHANGE], false);
    assert!(find(&view, Find::text("T2 · REPORT".as_bytes())).is_some());
    assert!(find(&view, Find::exactly_named(Role::Paragraph, b"A useful report")).is_some());
    assert!(find(&view, Find::named(Role::List, b"Steps")).is_none());
}
#[test]
fn a_failed_change_says_its_reason() {
    let mut task = change();
    task.result = Some(c::TaskResult::Ending(Box::new(c::OwnerChipEnding::Failed(document(b"Checks failed")))));
    let view = view(Some(task), &[CHANGE], false);
    status(&view, b"Failed", look::STATUS_HELD);
    assert!(find(&view, Find::text(b"Checks failed")).is_some());
}
#[test]
fn a_delegation_without_its_document_shows_its_call_work() {
    let view = view(None, &[CHANGE], false);
    assert!(fixtures::find(view.tree(), Role::Region, b"Task 2").is_none());
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"delegate").is_some());
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"Delegated tasks: 2").is_some());
    assert_eq!(work_slots(&view), 2);
}
#[test]
fn a_carded_task_hides_its_heard_result() {
    let view = view(Some(landed()), &[CHANGE], true);
    let text = fixtures::text(view.tree());
    assert!(skein_lib::bytes::find(&text, b"Carded result").is_none());
    assert!(skein_lib::bytes::find(&text, b"Duplicate result").is_none());
    assert!(skein_lib::bytes::find(&text, b"Another task's result").is_some());
}
#[test]
fn a_partly_carded_call_keeps_its_work() {
    let view = view(Some(change()), &[CHANGE, 3], false);
    assert!(find(&view, Find::text("T2 · CHANGE".as_bytes())).is_some());
    assert_eq!(work_slots(&view), 2);
}

pub(super) fn step_roles() {
    let view = view(Some(landed()), &[CHANGE], false);
    for (token, role) in [(look::STEPS, Role::List), (look::STEP, Role::Item), (look::STEP_DONE, Role::Item)] {
        let mut found = false;
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&token) {
                assert_eq!(node.role, role);
                found = true;
            }
        }
        assert!(found);
    }
}

#[test]
fn a_calls_cards_keep_its_task_order() {
    let mut other = change();
    other.chip.task = 3;
    let view = view_documents(&[change(), other], &[3, CHANGE], false);
    let mut cards = List::with_capacity(2);
    for node in view.tree().nodes() {
        if node.styles.as_slice().contains(&look::TASK_CARD) {
            cards.push(node.name.as_deref().expect("card name")).expect("two cards");
        }
        assert!(!node.styles.as_slice().contains(&look::WORK));
    }
    assert_eq!(cards.as_slice(), [b"Task 3".as_slice(), b"Task 2"]);
}

fn work_slots(view: &crate::View) -> u32 {
    let mut count = 0_u32;
    for node in view.tree().nodes() {
        if node.styles.as_slice().contains(&look::WORK) {
            count = count.checked_add(1).expect("work slot count");
        }
    }
    count
}
