use super::fixtures;
use jig_client::{contract as c, thread::Thread};
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role};
const TASK: u64 = 9;
const ATTEMPT: u64 = 23;
const TURN: u32 = 1;
const RETRY: u32 = 0;
const OFFSET: u32 = 0;
const THROUGH: u32 = 5;
const COMMITTED: u32 = 0;
const LINES: u32 = 4;
const BYTES: u64 = 65536;
const TEXT: u32 = 32;
const EXCHANGES: u32 = 8;
const SPENT: u64 = 7;
const BELOW: u64 = 2;
fn task() -> c::Task {
    use jig_client_protocol_wire::v8 as w;
    let mut record = jig_client_test_records::task_record().into_parts();
    record.chip = jig_client_test_records::owner_waiting_chip_record();
    let record = w::Task::new(&w::CEILINGS, record).expect("checked fixture");
    let link = jig_client_protocol_link::Limits::new(fixtures::limits().client.domain, EXCHANGES).expect("link");
    let mut task = jig_client_protocol_link::translate::documents::task(record, &link).expect("task");
    let c::ChipPhase::Owner { phase, .. } = task.chip.details.as_mut() else { panic!("owner chip") };
    *phase = c::OwnerChipPhase::Active(c::RunStage::Idle);
    task.budget.numbers.spent = SPENT;
    task.budget.numbers.spent_below = BELOW;
    task
}
#[test]
fn a_chats_header_says_its_status_and_spend() {
    let mut b = Builder::new(&fixtures::tree_limits());
    let root = fixtures::root_in_project(Some(fixtures::PROJECT));
    crate::pages::chat::header(&mut b, &task(), root.pages().me(root.client()), &fixtures::words());
    let tree = b.finish();
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Spent 7; delegates spent 2").is_some());
}
fn provisional() -> Thread {
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT });
    let text = c::RunText {
        attempt: ATTEMPT,
        began: Some(Wall::EPOCH),
        turn: c::ProvisionalTurnName { attempt: ATTEMPT, number: TURN, retry: RETRY },
        from: OFFSET,
        through: THROUGH,
        text: c::Text::new(Box::from(*b"Hello"), TEXT).expect("text"),
        cut: false,
        call: None,
    };
    thread.run(
        &c::RunState { task: TASK, attempt: Some(ATTEMPT), committed_turn: COMMITTED, writing: None, call: None },
        Some(&text),
    );
    thread
}
fn tree(thread: &Thread) -> skein_ui::Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::pages::chat::conversation(
        &mut b,
        thread,
        &fixtures::root_in_project(None),
        &fixtures::words(),
        &fixtures::jig_words(),
        &mut jig_client_view::JigBindings::new(fixtures::tree_limits().bindings, TEXT),
    );
    b.finish()
}
#[test]
fn provisional_text_is_marked_until_its_turn_commits() {
    let mut thread = provisional();
    assert!(fixtures::find(&tree(&thread), Role::Status, b"Provisional").is_some());
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(TURN);
    rows.push(c::ActivityLine {
        position: TURN.into(),
        at: Wall::EPOCH,
        event: c::Activity::Turn {
            name: c::TurnName { attempt: ATTEMPT, number: TURN },
            turn: c::TurnRead::Undecodable { why: c::TurnProblem::Malformed },
        },
    })
    .expect("turn");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    assert_eq!(fixtures::find(&tree(&thread), Role::Status, b"Provisional"), None);
}

const MESSAGE: u64 = 3;
const PARTY: u64 = 73;
#[test]
fn an_owner_activity_message_is_shown_in_the_chat() {
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(TURN);
    rows.push(c::ActivityLine {
        position: TURN.into(),
        at: Wall::EPOCH,
        event: c::Activity::Owner {
            source: Box::new(c::OwnerActivityLine {
                task: TASK,
                position: TURN.into(),
                at: Wall::EPOCH,
                line: c::OwnerActivity::Words {
                    message: MESSAGE,
                    by: c::OwnerRequester::Person(PARTY),
                    kind: c::OwnerMessageKind::Words,
                    words: c::Text::new(Box::from(*b"An owner message"), TEXT).expect("words"),
                },
            }),
            decoded: None,
        },
    })
    .expect("line");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    assert!(skein_lib::bytes::find(&fixtures::text(&tree(&thread)), b"An owner message").is_some());
}
const DELEGATE: u64 = 12;
const OTHER_DELEGATE: u64 = 13;
const CLOSE_GRACE: skein_lib::Duration = skein_lib::Duration::from_secs(5);
const CLOSE_OPERATIONS: u32 = 3;
fn closing(grace: Option<skein_lib::Duration>) -> skein_ui::Tree {
    let mut operations = List::with_capacity(CLOSE_OPERATIONS);
    let reason = c::Text::new(Box::from(*b"Chat closed"), TEXT).expect("reason");
    operations
        .push(c::Operation::Cancel { project: fixtures::PROJECT, task: DELEGATE, reason: reason.clone(), grace: None })
        .expect("delegate cancel");
    operations
        .push(c::Operation::Move { project: fixtures::PROJECT, task: OTHER_DELEGATE, reason: reason.clone() })
        .expect("delegate move");
    operations
        .push(c::Operation::Cancel { project: fixtures::PROJECT, task: TASK, reason, grace })
        .expect("chat close");
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::confirm::close_choices(
        &mut b,
        TASK,
        &operations,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
}
#[test]
fn the_close_confirmation_names_each_live_delegates_fate() {
    let tree = closing(Some(CLOSE_GRACE));
    let text = fixtures::text(&tree);
    assert!(skein_lib::bytes::find(&text, b"Delegate 12 will be cancelled").is_some());
    assert!(skein_lib::bytes::find(&text, b"Delegate 13 will be moved to you").is_some());
    assert!(fixtures::find(&tree, Role::Button, b"Cancel delegate 12").is_some());
    assert!(fixtures::find(&tree, Role::Button, b"Take delegate 13").is_some());
}
const FRACTIONAL_GRACE: skein_lib::Duration = skein_lib::Duration::from_millis(125);
const GRACES: [(Option<skein_lib::Duration>, &[u8]); 3] = [
    (Some(CLOSE_GRACE), b"allow 5 seconds to finish, then cancel"),
    (Some(FRACTIONAL_GRACE), b"allow 125000000 nanoseconds to finish, then cancel"),
    (None, b"Close chat now"),
];
#[test]
fn the_close_confirmation_names_the_templates_grace() {
    for (grace, expected) in GRACES {
        assert!(skein_lib::bytes::find(&fixtures::text(&closing(grace)), expected).is_some(), "{grace:?}");
    }
}

const CHAT_LABEL: u32 = 7;
const CHAT_TEMPLATE: u32 = 1;
const CHAT_CHARTER: u32 = 1;
const SINGLE_TEMPLATE: u32 = 1;
const EMPTY_PARAMETERS: u32 = 0;
fn chat_root() -> temper_client_domain::Domain {
    use temper_client_domain::{Address, Event};
    let (mut root, me_watch, task_watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
    let mut me = root.pages().me(root.client()).expect("me").clone();
    let member = me.projects.get_mut(EMPTY_PARAMETERS).expect("project");
    member.templates = List::with_capacity(SINGLE_TEMPLATE);
    member
        .templates
        .push(c::Template {
            number: CHAT_TEMPLATE,
            name: c::DocumentText::complete(c::Text::new(Box::from(*b"Chat"), TEXT).expect("name")),
            executor: c::TemplateExecutor::Charter(CHAT_CHARTER),
            contract: c::TaskContractDocument::Report { words: TEXT },
            shallower: SINGLE_TEMPLATE,
            budget: c::TemplateBudget::AsAllowed { cap: None },
            deadline: c::TemplateDeadline::None,
            tracked: None,
            flags: c::TaskFlags { conversation: true, listed: false, retention: c::Retention::Kept },
            label: CHAT_LABEL,
            grace: Some(CLOSE_GRACE),
        })
        .expect("template");
    let mut task = task();
    task.chip.task = TASK;
    task.project = fixtures::PROJECT;
    task.acts = List::with_capacity(SINGLE_TEMPLATE);
    task.acts
        .push(c::Offered {
            act: c::ActKind::Cancel,
            admission: c::Admission::Allowed,
            effect: Some(c::Effect::Cancel(c::Impact {
                tasks: c::PhaseCounts {
                    waiting: 0,
                    running: 1,
                    held: 0,
                    closing: 0,
                    ended: 0,
                    done: 0,
                    failed: 0,
                    cancelled: 0,
                },
                live_runs: 1,
                proposals: 0,
            })),
        })
        .expect("close");
    for (watch, document) in [
        (me_watch, c::Delivery::Me(Box::new(me))),
        (task_watch.expect("chat watch"), c::Delivery::Task(Box::new(task))),
    ] {
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(document) },
                },
            },
        );
    }
    root
}
#[test]
fn pressing_close_opens_the_chat_confirmation() {
    let mut root = chat_root();
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
        },
    );
    assert!(root.pages().confirmation().is_some(), "Close prepared the compound request");
    let mut view = fixtures::view();
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Dialog, b"Close").is_some());
}

#[test]
fn the_close_confirmation_says_the_cancellations_facts() {
    let mut root = chat_root();
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
        },
    );
    let mut view = fixtures::view();
    let said = fixtures::text(fixtures::build(&mut view, &root));
    let dangerous = fixtures::jig_words().say(jig_client_view::phrases::jig::JigPhrase::Dangerous, &[]);
    assert!(skein_lib::bytes::find(&said, &dangerous).is_some(), "irreversible cancellation");
    let impact = fixtures::jig_words()
        .say(jig_client_view::phrases::jig::JigPhrase::TasksRunning, &[skein_ui::phrases::Hole::Number(1)]);
    assert!(skein_lib::bytes::find(&said, &impact).is_some(), "supplied running-task impact");
}

const PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(DELEGATE), number: 3 };
const PROPOSAL_OUTPUTS: u32 = 256;
const PROPOSAL_NOW: skein_lib::Time = skein_lib::Time::ZERO;
const PROPOSAL_OFFERS: u32 = 2;
const EMPTY_BATCH: u32 = 0;
fn proposal_root() -> temper_client_domain::Domain {
    use temper_client_domain::Event;
    let limits = fixtures::domain_limits();
    let (mut root, _, watch) = fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &limits);
    let mut task = task();
    task.chip.task = TASK;
    task.proposals = List::with_capacity(SINGLE_TEMPLATE);
    task.proposals.push(PROPOSAL).expect("one proposal");
    let mut out = skein_lib::Queue::with_capacity(PROPOSAL_OUTPUTS);
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: PROPOSAL_NOW, wall: Wall::EPOCH, limits },
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    let mut acts = List::with_capacity(PROPOSAL_OFFERS);
    for act in [c::ActKind::Accept, c::ActKind::Reject] {
        acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).expect("offer");
    }
    let page = c::ReadValue::OwnerProposal(Box::new(c::OwnerProposal {
        key: PROPOSAL,
        project: fixtures::PROJECT,
        reason: c::DocumentText::complete(c::Text::new(Box::from(*b"Make a goal"), TEXT).expect("reason")),
        action: c::OwnerProposalAction::Batch(List::with_capacity(EMPTY_BATCH)),
        as_holder: true,
        waiting: c::OwnerProposalWaiting { holder: c::OwnerHistoryHolder::Person(PARTY), since: Wall::EPOCH },
        funding: List::with_capacity(EMPTY_BATCH),
        acts,
    }));
    let mut answered = false;
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Client {
            request:
                jig_client::Request::Down {
                    down:
                        jig_client::Down::Read {
                            read,
                            what: jig_client::Readable::Jig(c::Read::Proposal { proposal: PROPOSAL }),
                        },
                },
        } = request
        {
            fixtures::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(page)),
                        },
                    },
                },
            );
            answered = true;
            break;
        }
    }
    assert!(answered, "proposal read emitted");
    root
}
#[test]
fn a_proposal_waiting_in_the_chat_is_shown_after_its_conversation() {
    let root = proposal_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let mut conversation_end = None;
    let mut accept = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Log {
            conversation_end = Some(index + usize::try_from(node.size).expect("log size"));
        }
        if node.role == Role::Button && node.name.as_deref() == Some(b"Accept".as_slice()) {
            accept = Some(index);
        }
    }
    assert!(
        accept.expect("proposal's accept act") >= conversation_end.expect("conversation"),
        "card after conversation"
    );
    assert!(fixtures::find(tree, Role::Button, b"Reject with reason").is_some());
}
#[test]
fn rejecting_in_the_chat_shows_the_reason_in_its_confirmation() {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = proposal_root();
    let object = c::ObjectKey::Proposal(PROPOSAL);
    for act in [
        Act::CardWords { object, words: Box::from(*b"Too costly") },
        Act::Decide { object, act: c::ActKind::Reject, revision: None },
    ] {
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(act) });
    }
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Reject with reason").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(tree), b"Too costly").is_some());
}
