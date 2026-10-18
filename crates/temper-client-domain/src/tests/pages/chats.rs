use super::fixtures;
use crate::pages::{Request, chats::Chats};
use jig_client::{Event, Readable, Readout, Told, Watchable, contract::*};
use skein_lib::{List, Queue};
const FIRST: u64 = 1;
const OUTPUTS: u32 = 16;
const CURSOR_BYTES: u32 = 8;
const SERIAL: u64 = 13;
#[test]
fn starting_a_chat_sends_the_chat_template_filled_once() {
    let limits = fixtures::limits();
    let me = fixtures::chats::me();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut serial = SERIAL;
    chats.write(Box::from(*b"Hello"), &limits);
    chats.start(&fixtures::env(&limits), Some(&me), Some(fixtures::PROJECT), &mut serial, &mut out);
    chats.start(&fixtures::env(&limits), Some(&me), Some(fixtures::PROJECT), &mut serial, &mut out);
    let expected =
        jig_client::shapes::chat(&me, fixtures::PROJECT, fixtures::acts::text(b"Hello"), fixtures::env(&limits).wall)
            .expect("template");
    assert_eq!(fixtures::acts::sent(&mut out).as_ref(), [expected]);
}
#[test]
fn the_chats_watch_live_lines_by_last_activity() {
    let limits = fixtures::limits();
    let me = fixtures::chats::me();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    chats.configure(Some(&me), Some(fixtures::PROJECT), &limits, &mut out);
    let query = TaskQuery {
        project: Some(fixtures::PROJECT),
        requester: Some(OwnerRequester::Person(fixtures::PARTY)),
        label: Some(7),
        root: None,
        parent: None,
        phase: None,
        executor: None,
        tracked: None,
        order: QueryOrder::Activity,
    };
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: chats.live,
            subject: Watchable::Jig(Subject::Tasks { query, window: limits.client.domain.rows })
        }))
    );
}
#[test]
fn closed_chats_are_read_a_page_at_a_time() {
    let limits = fixtures::limits();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    chats.configure(Some(&fixtures::chats::me()), Some(fixtures::PROJECT), &limits, &mut out);
    out.pop();
    let Some(Request::Client(Event::Read { handle, read: Readable::Jig(Read::Tasks { query, live, ended }) })) =
        out.pop()
    else {
        panic!("closed read")
    };
    assert_eq!(
        (handle, query.phase, query.order, live, ended),
        (chats.closed, Some(PhaseClass::Ended), QueryOrder::Ended, None, None)
    );
    let cursor = Cursor::new(Box::from(*b"next"), CURSOR_BYTES).expect("cursor");
    chats.told(
        Told::Page {
            handle,
            page: Readout::Jig(ReadResult::Value(ReadValue::Tasks(Box::new(TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: EndedGroup::Rows { rows: List::with_capacity(0), after: Some(cursor.clone()), cut: false },
            })))),
        },
        &mut out,
    );
    chats.older(&mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle,
            read: Readable::Jig(Read::Tasks { query, live: None, ended: Some(cursor) })
        }))
    );
}
#[test]
fn starting_a_chat_waits_for_the_first_words() {
    let limits = fixtures::limits();
    let me = fixtures::chats::me();
    let mut chats = Chats::new(FIRST);
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut serial = SERIAL;
    chats.start(&fixtures::env(&limits), Some(&me), Some(fixtures::PROJECT), &mut serial, &mut out);
    assert_eq!(fixtures::acts::sent(&mut out).len(), 0);
}
#[test]
fn a_started_chat_opens_its_conversation_activity() {}
