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
