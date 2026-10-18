use crate::{
    Action, Event, Request,
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
#[test]
fn marking_seen_sends_one_seen_over_results() {
    let limits = fixtures::limits();
    let (mut root, watch) = fixtures::inbox_home(Filter::ALL);
    fixtures::inbox_entries(&mut root, watch, rows(ObjectKey::Result(TASK), InboxGroup::Results, PROJECT));
    let mut out = Queue::with_capacity(crate::max_out(&limits));
    for _ in 0_u32..2 {
        crate::step(&mut root, &fixtures::env(&limits), Event::Act { action: Action::Pages(Act::Seen) }, &mut out);
    }
    let mut sent = List::with_capacity(2);
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: jig_client::Down::Send { request: jig_client::Ask::Jig(ask), .. } },
        } = request
        {
            sent.push(*ask).expect("two acts at most");
        }
    }
    assert_eq!(
        sent.as_slice(),
        [contract::Ask::Seen { through: contract::SeenThrough::Results { through: POSITION } }]
    );
}
