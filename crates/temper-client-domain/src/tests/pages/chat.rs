use super::fixtures::{self, chat as f};
use crate::pages::{
    Act, Request,
    chat::{Chat, Sending},
};
use jig_client::{Event, Readable, Told, Watchable, contract as c};
use skein_lib::Queue;
const OUTPUTS: u32 = 16;
const SERIAL: u64 = 37;
const EMPTY: u32 = 0;
#[test]
fn opening_a_chat_watches_task_and_reads_its_first_activity_page() {
    let mut out = Queue::with_capacity(OUTPUTS);
    let chat = Chat::new(f::TASK, f::FIRST, &fixtures::limits(), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: chat.document,
            subject: Watchable::Jig(c::Subject::Task { task: f::TASK })
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle: chat.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None })
        }))
    );
    assert!(out.is_empty());
}
#[test]
fn a_live_run_is_watched_and_dropped_when_it_ends() {
    let mut f = f::opened();
    let run = f.chat.run.expect("live run watched");
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Watch {
            handle: run,
            subject: Watchable::Jig(c::Subject::Run { task: f::TASK, turn: None, offset: EMPTY })
        }))
    );
    let mut task = f::task();
    f::phase(&mut task, c::OwnerChipPhase::Active(c::RunStage::Idle));
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(&f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(f.chat.run, None);
    assert_eq!(f::handles(&mut f.out).as_ref(), [run]);
}
#[test]
fn leaving_a_chat_unwatches_its_task_activity_and_run() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    f.chat.leave(&mut f.out);
    assert_eq!(f::handles(&mut f.out).as_ref(), [f.chat.document, f.chat.activity, f.chat.run.expect("run")]);
}
#[test]
fn stop_sends_one_stop_and_shows_it_pending_until_answered() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let mut serial = SERIAL;
    for _ in [false, true] {
        f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::Stop, &mut serial, &mut f.out);
    }
    assert_eq!(fixtures::acts::sent(&mut f.out).as_ref(), [c::Ask::Stop { project: fixtures::PROJECT, task: f::TASK }]);
    assert_eq!(
        f.chat.pending.as_slice(),
        [(jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: SERIAL + 1 }, Sending::Stop)]
    );
    let key = f.chat.pending.as_slice()[0].0;
    f.chat.told(&f.client, Told::Answered { key, outcome: jig_client::Outcome::Unknown }, &mut f.out);
    assert!(f.chat.pending.is_empty());
    assert_eq!(f.chat.outcome, Some(jig_client::Outcome::Unknown));
}
#[test]
fn seen_over_a_chats_turns_is_one_keyed_request() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    f.chat.thread.loaded = f::POSITION;
    let mut serial = SERIAL;
    for _ in [false, true] {
        f.chat.act(&fixtures::env(&fixtures::limits()), &f.client, Act::Seen, &mut serial, &mut f.out);
    }
    assert_eq!(
        fixtures::acts::sent(&mut f.out).as_ref(),
        [c::Ask::Seen { through: c::SeenThrough::Conversation { task: f::TASK, through: f::POSITION } }]
    );
}
