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
