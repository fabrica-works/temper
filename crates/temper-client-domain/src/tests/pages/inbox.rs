use crate::{
    Request,
    address::{Filter, Kind},
    pages::Act,
    tests::fixtures,
};
use jig_client::{
    Readable,
    contract::{self, InboxGroup, ObjectKey, Read},
};
use skein_lib::{List, Queue};
const TASK: u64 = 9;
const MESSAGE: u64 = 3;
const PROJECT: u32 = fixtures::PROJECT;
const OTHER_PROJECT: u32 = 2;
const GOAL: u64 = 7;
const POSITION: u64 = 11;
const BELOW_TASK: u64 = 8;
const CASES: [(ObjectKey, InboxGroup, Read); 6] = [
    (
        ObjectKey::Question { task: TASK, message: MESSAGE },
        InboxGroup::Questions,
        Read::Question { task: TASK, message: MESSAGE },
    ),
    (
        ObjectKey::Proposal(contract::ProposalKey { proposer: contract::Proposer::Task(TASK), number: MESSAGE }),
        InboxGroup::Proposals,
        Read::Proposal {
            proposal: contract::ProposalKey { proposer: contract::Proposer::Task(TASK), number: MESSAGE },
        },
    ),
    (ObjectKey::Escalation(TASK), InboxGroup::Escalations, Read::Escalation { task: TASK }),
    (ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, Read::PersonTask { task: TASK }),
    (ObjectKey::Result(TASK), InboxGroup::Results, Read::Result { task: TASK }),
    (ObjectKey::Task(TASK), InboxGroup::Replies, Read::Task { task: TASK }),
];
fn rows(object: ObjectKey, group: InboxGroup, project: u32) -> List<contract::InboxEntry> {
    let mut rows = List::with_capacity(2);
    rows.push(fixtures::inbox_entry(object, group, project, POSITION)).expect("one row");
    rows
}
fn reads(out: &mut Queue<Request>) -> Box<[Read]> {
    let mut reads = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        if let Request::Client {
            request: jig_client::Request::Down { down: jig_client::Down::Read { what: Readable::Jig(read), .. } },
        } = request
        {
            reads.push(read).expect("bounded reads");
        }
    }
    reads.into_boxed()
}
#[test]
fn the_inbox_watches_inbox_and_reads_each_entrys_object() {
    let (mut root, watch) = fixtures::inbox_home(Filter::ALL);
    for (object, group, expected) in CASES {
        let mut out = fixtures::inbox_entries(&mut root, watch, rows(object, group, PROJECT));
        assert_eq!(reads(&mut out).as_ref(), [expected], "{group:?}");
    }
}
#[test]
fn a_filter_by_project_shows_only_that_projects_entries() {
    let (mut root, watch) = fixtures::inbox_home(Filter { project: Some(OTHER_PROJECT), ..Filter::ALL });
    let mut entries = rows(ObjectKey::Result(TASK), InboxGroup::Results, PROJECT);
    entries
        .push(fixtures::inbox_entry(ObjectKey::PersonTask(TASK), InboxGroup::PersonTasks, OTHER_PROJECT, POSITION))
        .expect("second entry");
    let mut out = fixtures::inbox_entries(&mut root, watch, entries);
    assert_eq!(reads(&mut out).as_ref(), [Read::PersonTask { task: TASK }]);
}
#[test]
fn a_filter_by_kind_shows_only_that_kind() {
    let (mut root, watch) = fixtures::inbox_home(Filter { kind: Some(Kind::Question), ..Filter::ALL });
    let mut out =
        fixtures::inbox_entries(&mut root, watch, rows(ObjectKey::Result(TASK), InboxGroup::Results, PROJECT));
    assert_eq!(reads(&mut out).as_ref(), []);
}
#[test]
fn a_filter_by_goal_shows_only_entries_below_it() {
    let (mut root, watch) = fixtures::inbox_home(Filter { goal: Some(GOAL), ..Filter::ALL });
    let mut out = fixtures::inbox_entries(
        &mut root,
        watch,
        rows(ObjectKey::Question { task: TASK, message: MESSAGE }, InboxGroup::Questions, PROJECT),
    );
    assert_eq!(
        reads(&mut out).as_ref(),
        [contract::InboxReadGroup::Needs, contract::InboxReadGroup::Updates].map(|group| Read::Inbox {
            group,
            project: None,
            kind: None,
            goal: Some(GOAL),
            after: None
        })
    );
    assert_eq!(root.pages().inbox().expect("home").cards.len(), 0, "owner membership comes from the filtered query");
    let handle = root.pages().inbox().expect("home").queries[0];
    let page = contract::InboxPage {
        cut: false,
        rows: rows(ObjectKey::PersonTask(BELOW_TASK), InboxGroup::PersonTasks, PROJECT),
        before: None,
        after: None,
    };
    crate::route::told(
        &mut root,
        &fixtures::env(&fixtures::limits()),
        jig_client::Told::Page {
            handle,
            page: jig_client::Readout::Jig(contract::ReadResult::Value(contract::ReadValue::Inbox(Box::new(page)))),
        },
        &mut out,
    );
    assert_eq!(
        root.pages().inbox().expect("home").cards.get(0).expect("selected card").object,
        ObjectKey::PersonTask(BELOW_TASK)
    );
}
fn result_pages() -> crate::pages::Pages {
    fixtures::acts::pages(ObjectKey::Result(TASK), fixtures::acts::question())
}
fn open_result(pages: &mut crate::pages::Pages, through: u64) -> Queue<crate::pages::Request> {
    fixtures::acts::act(pages, Act::OpenResult { task: TASK, through })
}
fn sent_seen(out: &mut Queue<crate::pages::Request>, through: u64) -> jig_client::Key {
    let Some(crate::pages::Request::Client(jig_client::Event::Send { key, request, name: None })) = out.pop() else {
        panic!("Seen precedes Show");
    };
    assert_eq!(
        request,
        jig_client::Ask::Jig(Box::new(contract::Ask::Seen { through: contract::SeenThrough::Results { through } }))
    );
    key
}
fn shown(out: &mut Queue<crate::pages::Request>) {
    assert_eq!(out.pop(), Some(crate::pages::Request::Show { address: crate::Address::Task { task: TASK } }));
    assert!(out.is_empty());
}
fn answer_seen(pages: &mut crate::pages::Pages, key: jig_client::Key, outcome: jig_client::Outcome) {
    let mut out = Queue::with_capacity(16);
    crate::pages::step(
        pages,
        &fixtures::env(&fixtures::limits()),
        crate::unlinked(&fixtures::limits(), fixtures::SEED).client(),
        crate::pages::Event::Told { told: jig_client::Told::Answered { key, outcome } },
        &mut out,
    );
    assert!(out.is_empty(), "Seen answers draw and send nothing");
}
fn answered() -> jig_client::Outcome {
    jig_client::Outcome::Answered(jig_client::Answered::Jig(contract::Answer::Done(contract::Outcome::Changed)))
}
#[test]
fn following_a_result_sends_seen_then_shows_its_task() {
    let mut pages = result_pages();
    let mut out = open_result(&mut pages, POSITION);
    sent_seen(&mut out, POSITION);
    shown(&mut out);
}
#[test]
fn result_seen_is_forward_only_and_has_one_request_in_flight() {
    let mut pages = result_pages();
    let key = sent_seen(&mut open_result(&mut pages, POSITION), POSITION);
    shown(&mut open_result(&mut pages, POSITION + 1));
    answer_seen(&mut pages, key, answered());
    shown(&mut open_result(&mut pages, POSITION));
    shown(&mut open_result(&mut pages, POSITION - 1));
    let mut out = open_result(&mut pages, POSITION + 1);
    sent_seen(&mut out, POSITION + 1);
    shown(&mut out);
}
#[test]
fn a_refused_or_failed_result_seen_is_resent_on_the_next_follow() {
    for outcome in [
        jig_client::Outcome::Unknown,
        jig_client::Outcome::Unsent,
        jig_client::Outcome::Lost,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(contract::Answer::Refused {
            refusal: contract::Refusal::NotOffered,
            at: None,
        })),
    ] {
        let mut pages = result_pages();
        let key = sent_seen(&mut open_result(&mut pages, POSITION), POSITION);
        answer_seen(&mut pages, key, answered());
        let key = sent_seen(&mut open_result(&mut pages, POSITION + 1), POSITION + 1);
        answer_seen(&mut pages, key, outcome);
        shown(&mut open_result(&mut pages, POSITION));
        let mut out = open_result(&mut pages, POSITION + 1);
        assert_ne!(sent_seen(&mut out, POSITION + 1), key);
        shown(&mut out);
    }
}
#[test]
fn going_to_a_results_task_from_another_page_sends_no_seen() {
    let mut pages = result_pages();
    shown(&mut fixtures::acts::act(&mut pages, Act::Go { address: crate::Address::Task { task: TASK } }));
    let client = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut out = Queue::with_capacity(16);
    crate::pages::step(
        &mut pages,
        &fixtures::env(&fixtures::limits()),
        client.client(),
        crate::pages::Event::Went { address: crate::Address::Chats },
        &mut out,
    );
    assert!(fixtures::acts::sent(&mut out).is_empty());
    shown(&mut fixtures::acts::act(&mut pages, Act::Go { address: crate::Address::Task { task: TASK } }));
    shown(&mut open_result(&mut pages, POSITION));
}
#[test]
fn result_seen_pending_and_answer_survive_navigation() {
    let mut pages = result_pages();
    let key = sent_seen(&mut open_result(&mut pages, POSITION), POSITION);
    let client = crate::unlinked(&fixtures::limits(), fixtures::SEED);
    for address in [crate::Address::Task { task: TASK }, crate::Address::Inbox { filter: Filter::ALL }] {
        crate::pages::step(
            &mut pages,
            &fixtures::env(&fixtures::limits()),
            client.client(),
            crate::pages::Event::Went { address },
            &mut Queue::with_capacity(16),
        );
    }
    shown(&mut open_result(&mut pages, POSITION + 1));
    answer_seen(&mut pages, key, answered());
    shown(&mut open_result(&mut pages, POSITION));
}
