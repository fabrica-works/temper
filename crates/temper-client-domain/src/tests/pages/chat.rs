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
fn closing_a_chat_with_a_live_delegate_is_one_compound_request() {
    let mut f = f::with_delegate();
    f.chat.prepare_close(&f.client);
    assert!(fixtures::acts::sent(&mut f.out).is_empty(), "said before sent");
    f.chat.delegate_fate(f::DELEGATE, jig_client::shapes::Fate::Move);
    let mut serial = SERIAL;
    f.chat.close(&mut serial, &mut f.out);
    f.chat.close(&mut serial, &mut f.out);
    let expected = jig_client::shapes::close_chat(
        &fixtures::chats::me(),
        fixtures::PROJECT,
        f::TASK,
        &[jig_client::shapes::Delegate { task: f::DELEGATE, fate: jig_client::shapes::Fate::Move }],
        fixtures::acts::text(b"Chat closed"),
    )
    .expect("chat shape");
    assert_eq!(fixtures::acts::sent(&mut f.out).as_ref(), [expected], "jig's compound, once");
}
#[test]
fn the_confirmed_close_keeps_the_delegate_fates_and_grace_shown() {
    let mut f = f::with_delegate();
    f.chat.prepare_close(&f.client);
    let shown = f.chat.closing.clone().expect("confirmation request");
    let mut changed = f::task();
    changed.acts = fixtures::acts::offered(c::ActKind::Cancel, c::Admission::Allowed);
    f::deliver(&mut f.client, f.watch, changed);
    let mut serial = SERIAL;
    f.chat.close(&mut serial, &mut f.out);
    assert_eq!(fixtures::acts::sent(&mut f.out).as_ref(), [shown], "confirm the shown request");
}

const FIRST_CARD: u32 = 0;
const SECOND_CARD: u32 = 1;
const PROPOSAL: c::ProposalKey = fixtures::acts::PROPOSAL;
const DELEGATE_PROPOSAL: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(f::DELEGATE), number: 4 };
const PROPOSALS: u32 = 2;
fn proposals(f: &mut f::Fixture, keys: &[c::ProposalKey]) {
    let mut task = f::task();
    task.proposals = skein_lib::List::with_capacity(PROPOSALS);
    for key in keys {
        task.proposals.push(*key).expect("proposals fit");
    }
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(&f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
}
fn reads(out: &mut Queue<Request>) -> Box<[(jig_client::Handle, c::ProposalKey)]> {
    let mut reads = skein_lib::List::with_capacity(PROPOSALS);
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Read { handle, read: Readable::Jig(c::Read::Proposal { proposal }) }) = request {
            reads.push((handle, proposal)).expect("bounded reads");
        }
    }
    reads.into_boxed()
}
#[test]
fn a_chats_proposals_are_read_once_and_kept_in_the_tasks_order() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL, DELEGATE_PROPOSAL]);
    let first = reads(&mut f.out);
    assert_eq!(first.len(), 2);
    assert_eq!([first[0].1, first[1].1], [PROPOSAL, DELEGATE_PROPOSAL]);
    proposals(&mut f, &[DELEGATE_PROPOSAL, PROPOSAL]);
    assert!(reads(&mut f.out).is_empty(), "a retained proposal is not read again");
    assert_eq!(
        [f.chat.cards.get(FIRST_CARD).expect("first").handle, f.chat.cards.get(SECOND_CARD).expect("second").handle],
        [first[1].0, first[0].0],
        "delegate's proposal kept, ordered by the task"
    );
}
#[test]
fn a_proposal_decided_elsewhere_leaves_the_chats_card() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL]);
    let handle = reads(&mut f.out)[0].0;
    proposals(&mut f, &[]);
    assert!(f.chat.cards.is_empty());
    assert_eq!(f::handles(&mut f.out).as_ref(), [handle], "drop the card's handle too");
}
fn rejecting_pages() -> crate::pages::Pages {
    let mut pages = crate::pages::Pages::new();
    crate::pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        crate::unlinked(&fixtures::limits(), fixtures::SEED).client(),
        crate::pages::Event::Went { address: crate::Address::Chat { task: f::TASK } },
        &mut Queue::with_capacity(OUTPUTS),
    );
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: f::FIRST };
    pages
        .cards_mut()
        .expect("chat cards")
        .push(crate::pages::inbox::Card {
            object: c::ObjectKey::Proposal(PROPOSAL),
            group: c::InboxGroup::Proposals,
            project: Some(fixtures::PROJECT),
            position: None,
            handle,
            page: Some(fixtures::acts::proposal(c::ActKind::Reject, c::Admission::Allowed)),
            words: None,
            key: None,
            outcome: None,
        })
        .expect("card fits");
    pages
}
#[test]
fn rejecting_a_proposal_in_place_is_said_before_it_is_sent() {
    let mut pages = rejecting_pages();
    let object = c::ObjectKey::Proposal(PROPOSAL);
    fixtures::acts::act(&mut pages, Act::CardWords { object, words: Box::from(*b"Too costly") });
    let mut said = fixtures::acts::act(&mut pages, fixtures::acts::decide(object, c::ActKind::Reject));
    assert!(fixtures::acts::sent(&mut said).is_empty());
    assert_eq!(pages.confirmation().expect("confirmation").act, c::ActKind::Reject);
    assert_eq!(pages.chat().expect("chat").cards.get(FIRST_CARD).expect("card").key, None);
    assert_eq!(
        fixtures::acts::sent(&mut fixtures::acts::act(&mut pages, Act::Confirm)).as_ref(),
        [c::Ask::Decide {
            project: fixtures::PROJECT,
            on: c::DecideOn::Proposal {
                proposal: PROPOSAL,
                decision: c::Decision::Rejected { reason: fixtures::acts::text(b"Too costly") }
            }
        }]
    );
}

#[test]
fn a_departed_proposal_keeps_its_identity_for_the_explanation() {
    let mut f = f::opened();
    proposals(&mut f, &[PROPOSAL]);
    proposals(&mut f, &[]);
    assert_eq!(f.chat.departures(), &[c::ObjectKey::Proposal(PROPOSAL)]);
}

const DELEGATED: u64 = 2;
fn live_delegate() -> c::Task {
    let mut task = f::task();
    task.delegates = skein_lib::List::with_capacity(1);
    let mut chip = task.chip.clone();
    chip.task = DELEGATED;
    task.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).expect("live delegate");
    task
}
#[test]
fn a_live_delegate_is_watched_before_its_call_names_it() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: f.chat.next };
    f::deliver(&mut f.client, f.watch, live_delegate());
    f.chat.told(&f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(watches(&mut f.out).as_ref(), [(handle, DELEGATED)]);
}
#[test]
fn a_delegate_that_ends_is_unwatched_once_nothing_names_it() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    f::deliver(&mut f.client, f.watch, live_delegate());
    f.chat.told(&f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    let handle = watches(&mut f.out)[0].0;
    let mut task = f::task();
    task.delegates = skein_lib::List::with_capacity(0);
    f::deliver(&mut f.client, f.watch, task);
    f.chat.told(&f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(f::handles(&mut f.out).as_ref(), [handle]);
}
#[test]
fn a_chat_watches_a_delegated_task_when_only_its_result_remains() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let next = f.chat.next;
    let mut blocks = skein_lib::List::with_capacity(1);
    blocks
        .push(c::Block::Result {
            call: 8,
            summary: c::Text::new(Box::from(f::WAKE_SUMMARY), 128).expect("wake summary"),
            failed: false,
        })
        .expect("wake result");
    f::activity_blocks(&mut f, c::Speaker::Person, blocks, f::POSITION);
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: next };
    assert_eq!(watches(&mut f.out).as_ref(), [(handle, DELEGATED)]);
}

fn watches(out: &mut Queue<Request>) -> Box<[(jig_client::Handle, u64)]> {
    let mut watches = skein_lib::List::with_capacity(OUTPUTS);
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Watch { handle, subject: Watchable::Jig(c::Subject::Task { task }) }) = request {
            watches.push((handle, task)).expect("bounded watches");
        }
    }
    watches.into_boxed()
}
#[test]
fn a_chat_watches_each_delegated_task_as_its_call_arrives() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let next = f.chat.next;
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: next };
    assert_eq!(watches(&mut f.out).as_ref(), [(handle, DELEGATED)], "fresh handle watches the delegated task");
    assert!(f.chat.delegation(&f.client, DELEGATED).is_none(), "wait for its document");
    let mut emitted = Queue::with_capacity(OUTPUTS);
    f.client.step(
        skein_lib::Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Watch { handle, subject: Watchable::Jig(c::Subject::Task { task: DELEGATED }) },
        &mut emitted,
    );
    let mut watch = None;
    while let Some(request) = emitted.pop() {
        if let jig_client::Request::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Task { task: DELEGATED }) },
        } = request
        {
            watch = Some(token);
        }
    }
    let mut task = f::change();
    task.chip.task = DELEGATED;
    f::deliver(&mut f.client, watch.expect("delegation opened"), task.clone());
    assert!(f.chat.told(&f.client, Told::Changed { handle }, &mut f.out));
    assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task));
    assert!(f.out.is_empty(), "a document refresh leaves its watch alone");
    assert!(f.chat.delegation(&f.client, 999).is_none(), "only held delegations are exposed");
}
#[test]
fn a_delegation_requested_by_another_owner_is_not_this_chats() {
    let mut f = f::opened();
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    for (requester, requested) in
        [(c::OwnerRequester::Person(DELEGATED), false), (c::OwnerRequester::Task(f::TASK), true)]
    {
        let mut task = f::change();
        task.chip.task = DELEGATED;
        task.requester = requester;
        f.chat.delegations.get_mut(0).expect("delegation").final_task = Some(Box::new(task.clone()));
        assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task));
        assert_eq!(f.chat.requested(&f.client, DELEGATED), if requested { Some(&task) } else { None });
    }
}
#[test]
fn a_chat_watches_the_newest_delegations_up_to_its_bound() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let bound = crate::pages::delegation::bound(&fixtures::limits());
    let mut tasks = skein_lib::List::with_capacity(bound + 1);
    for task in 1..=u64::from(bound + 1) {
        tasks.push(task).expect("calls");
    }
    f::activity(&mut f, tasks.as_slice(), f::POSITION);
    let watched = watches(&mut f.out);
    assert_eq!(watched.len(), usize::try_from(bound).expect("bound fits"));
    for (index, (_, task)) in watched.iter().enumerate() {
        assert_eq!(*task, u64::from(bound + 1) - u64::try_from(index).expect("bounded index"));
    }
    for delegation in &f.chat.delegations {
        assert_ne!(delegation.task, 1, "oldest is past the bound");
    }
}
#[test]
fn a_delegation_is_unwatched_as_its_call_leaves_the_thread() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    let handle = watches(&mut f.out)[0].0;
    f::activity(&mut f, &[], f::POSITION + 1);
    assert_eq!(f::handles(&mut f.out).as_ref(), [handle]);
    assert!(f.chat.delegations.is_empty());
    assert!(f.chat.delegation(&f.client, DELEGATED).is_none());
}
#[test]
fn a_newer_delegation_unwatches_the_one_past_the_bound() {
    let mut f = f::opened();
    while f.out.pop().is_some() {}
    let bound = crate::pages::delegation::bound(&fixtures::limits());
    let mut tasks = skein_lib::List::with_capacity(bound + 1);
    for task in 1..=u64::from(bound) {
        tasks.push(task).expect("calls");
    }
    f::activity(&mut f, tasks.as_slice(), f::POSITION);
    let old = watches(&mut f.out);
    let oldest = old.last().expect("oldest").0;
    tasks.push(u64::from(bound + 1)).expect("new call");
    f::activity(&mut f, tasks.as_slice(), f::POSITION + 1);
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Read {
            handle: f.chat.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None })
        }))
    );
    assert_eq!(f.out.pop(), Some(Request::Client(Event::Unwatch { handle: oldest })), "make watch room first");
    let new = watches(&mut f.out);
    assert_eq!(new.len(), 1);
    assert_eq!(new[0].1, u64::from(bound + 1));
    for (delegation, (handle, task)) in f.chat.delegations.iter().skip(1).zip(old.iter()) {
        assert_eq!((delegation.handle, delegation.task), (*handle, *task), "retained handles stay the same");
    }
}
#[test]
fn leaving_a_chat_unwatches_its_delegations() {
    let mut f = f::with_delegate();
    f::activity(&mut f, &[DELEGATED, 3], f::POSITION);
    let watched = watches(&mut f.out);
    f.chat.leave(&mut f.out);
    let left = f::handles(&mut f.out);
    assert_eq!(left.len(), 5, "document, activity, run and both delegations");
    assert_eq!(&left[3..], [watched[0].0, watched[1].0]);
}
#[test]
fn an_ended_delegation_reads_its_task_and_keeps_its_result() {
    let mut f = f::opened();
    f::activity(&mut f, &[DELEGATED], f::POSITION);
    let handle = watches(&mut f.out)[0].0;
    for why in [jig_client::Why::SubjectEnded, jig_client::Why::Unknown] {
        assert!(f.chat.told(&f.client, Told::Ended { handle, why }, &mut f.out));
        assert_eq!(
            f.out.pop(),
            Some(Request::Client(Event::Read { handle, read: Readable::Jig(c::Read::Task { task: DELEGATED }) }))
        );
        assert!(f.out.is_empty());
    }
    let mut task = f::change();
    task.chip.task = DELEGATED;
    let result = f::result(&task, c::Ending::Done, Some(f::change_value(b"abcdef0123456789")));
    let historical = c::OwnerHistorical {
        key: c::ObjectKey::Task(DELEGATED),
        value: c::OwnerHistoricalValue::Task { document: Box::new(task.clone()), result: Box::new(result.clone()) },
        at: None,
        by: None,
    };
    assert!(f.chat.told(
        &f.client,
        Told::Page {
            handle,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Historical(Box::new(historical))))
        },
        &mut f.out
    ));
    task.result = Some(c::TaskResult::Document(Box::new(result)));
    assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task));
    f.chat.told(&f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    assert_eq!(f.chat.delegation(&f.client, DELEGATED), Some(&task), "refresh keeps the final task");
    assert!(f.out.is_empty(), "no new requests for retained calls");
}

#[test]
fn the_chat_composer_keeps_the_send_bound_when_receiving_grows() {
    let mut limits = fixtures::limits();
    limits.client.domain.text = 2048;
    limits.client.domain.turn_text = 2048;
    let mut f = f::opened();
    let mut serial = SERIAL;
    let bound = usize::try_from(temper_client_protocol_wire::CUE_BYTES).unwrap();
    let mut words = skein_lib::bytes::zeroed(bound);
    words.fill(b'a');
    let mut over = skein_lib::bytes::zeroed(bound.checked_add(1).unwrap());
    over.fill(b'b');
    f.chat.act(&fixtures::env(&limits), &f.client, Act::MessageWords { words: words.clone() }, &mut serial, &mut f.out);
    assert_eq!(f.chat.words, words);
    f.chat.act(&fixtures::env(&limits), &f.client, Act::MessageWords { words: over }, &mut serial, &mut f.out);
    assert_eq!(f.chat.words, words, "an oversized edit keeps the previous words");
}

fn window_page(chat: &mut Chat, client: &jig_client::Client, out: &mut Queue<Request>, page: u32, pages: u32) {
    let mut rows = skein_lib::List::with_capacity(2);
    for offset in 0..2 {
        rows.push(c::ActivityLine {
            position: u64::from(pages.checked_sub(page).unwrap().checked_mul(2).unwrap().checked_sub(offset).unwrap()),
            at: skein_lib::Wall::EPOCH,
            event: c::Activity::RunBegan {
                attempt: 1,
                how: c::RunBeginning::Resumed,
                why: fixtures::acts::text(b"resume"),
            },
        })
        .unwrap();
    }
    let cursor = c::Cursor::new(Box::from(*b"older"), 5).unwrap();
    chat.told(
        client,
        Told::Page {
            handle: chat.activity,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows,
                before: if page.checked_add(1).unwrap() < pages { Some(cursor) } else { None },
                after: None,
                cut: false,
            })))),
        },
        out,
    );
}
#[test]
fn a_chat_fills_its_window_to_the_start_without_offering_older() {
    let limits = fixtures::limits();
    let client = jig_client::Client::unlinked(&limits.client, fixtures::SEED).unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut chat = Chat::new(f::TASK, f::FIRST, &limits, &mut out);
    while out.pop().is_some() {}
    for page in 0..5 {
        window_page(&mut chat, &client, &mut out, page, 5);
        assert_eq!(!out.is_empty(), page < 4, "page {page}");
        while out.pop().is_some() {}
        assert!(!chat.offers_older());
    }
    assert_eq!(chat.thread.lines.rows.len(), 10);
    assert!(chat.thread.reading.is_none());
}
#[test]
fn a_full_chat_window_offers_older_and_reads_it_when_asked() {
    let limits = fixtures::limits();
    let client = jig_client::Client::unlinked(&limits.client, fixtures::SEED).unwrap();
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut chat = Chat::new(f::TASK, f::FIRST, &limits, &mut out);
    while out.pop().is_some() {}
    for page in 0..8 {
        window_page(&mut chat, &client, &mut out, page, 10);
        assert_eq!(!out.is_empty(), page < 7, "page {page}");
        while out.pop().is_some() {}
    }
    assert_eq!(chat.thread.lines.rows.len(), crate::pages::chat::LINES);
    assert!(chat.offers_older());
    let mut serial = SERIAL;
    chat.act(&fixtures::env(&limits), &client, Act::Older, &mut serial, &mut out);
    let Some(Request::Client(Event::Read { read: Readable::Jig(c::Read::Activity { before: Some(_), .. }), .. })) =
        out.pop()
    else {
        panic!("Older reads the preceding page");
    };
    window_page(&mut chat, &client, &mut out, 8, 10);
    assert!(out.is_empty());
    assert!(chat.offers_older());
}
