use super::fixtures;
use crate::store::route::{self, Up};
use ::core::mem::size_of;
use jig_domain_core as core;
use jig_protocol_store as store;
use skein_lib::{Queue, Token};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

fn load(range: engine::Range, after: Option<engine::Key>, most: u32, bytes: u32) -> engine::StoreRequest {
    engine::StoreRequest::Load { owner: fixtures::OWNER, range, after, most, bytes }
}
fn forge_row(commit: u8) -> store::Row {
    let row = forge::Stored::Landed { commit: [commit; 32], task: 3 };
    store::Row::Application {
        key: temper_protocol_store::keys::encode(&forge::Key::Landed([commit; 32]), u32::MAX).unwrap(),
        value: temper_protocol_store::records::encode(&row),
    }
}
fn page(out: &mut Queue<Up>) -> (Token, Box<[engine::Record]>, Option<engine::Key>) {
    let Some(Up::Root(event)) = out.pop() else { panic!("one root event") };
    let engine::Event::Store(engine::StoreInput::Loaded { owner, rows, next }) = *event else { panic!("loaded") };
    (owner, rows, next)
}

#[test]
fn each_root_range_asks_the_store_as_the_table_maps_it() {
    let ranges = [
        engine::Range::Core(engine::CoreRange::Deployment),
        engine::Range::Core(engine::CoreRange::People),
        engine::Range::Core(engine::CoreRange::Tasks),
        engine::Range::Core(engine::CoreRange::Calls),
        engine::Range::Core(engine::CoreRange::RunProofs),
        engine::Range::Census(core::Census { results: Box::new([]) }),
        engine::Range::Forge,
        engine::Range::Core(engine::CoreRange::TaskResult { task: 7 }),
        engine::Range::Core(engine::CoreRange::ProposalDecision {
            proposer: jig_domain_people::Proposer::Party(8),
            proposal: 9,
        }),
        engine::Range::Core(engine::CoreRange::EscalationDecision { task: 7, revision: 2 }),
        engine::Range::Core(engine::CoreRange::TaskTranscript { task: 7, from: (2, 3) }),
        engine::Range::Core(engine::CoreRange::Turns { task: 7, attempt: 2 }),
        engine::Range::Core(engine::CoreRange::Notes(jig_domain_notes::Range::Entry { name: 5 })),
    ];
    for (index, range) in ranges.into_iter().enumerate() {
        let request = route::down(&mut fixtures::routes(), load(range, None, 4, 4096)).unwrap();
        check_range(index, request);
    }
}
fn check_range(index: usize, request: store::Request) {
    match index {
        0..=5 => {
            let store::Request::Restore {
                range: store::Restore::Jig(ranges), after: None, rows: 4, bytes: 4096, ..
            } = request
            else {
                panic!("jig row {index}")
            };
            let expected = [
                core::RestoreRange::Deployment,
                core::RestoreRange::People,
                core::RestoreRange::Tasks,
                core::RestoreRange::Core,
                core::RestoreRange::Core,
                core::RestoreRange::Census(core::Census { results: Box::new([]) }),
            ];
            if index == 2 {
                assert_eq!(ranges.as_ref(), [core::RestoreRange::Core, core::RestoreRange::Tasks]);
            } else {
                assert_eq!(ranges.as_ref(), [expected[index].clone()], "{index}");
            }
        }
        6 => {
            let store::Request::Restore { range: store::Restore::Application(ranges), .. } = request else {
                panic!("application")
            };
            assert_eq!(ranges.as_ref(), [skein_kv::Range { start: Box::new([]), end: None }]);
        }
        7..=9 => {
            let store::Request::Get { key: store::Get::Jig(key), .. } = request else { panic!("get {index}") };
            let expected = [
                core::Key::Tasks(jig_domain_tasks::Key::Ended(7)),
                core::Key::Core(core::CoreKey::ProposalDecision {
                    proposer: jig_domain_people::Proposer::Party(8),
                    proposal: 9,
                }),
                core::Key::Core(core::CoreKey::EscalationDecision { task: 7, revision: 2 }),
            ];
            assert_eq!(key, expected[index.checked_sub(7).unwrap()], "{index}");
        }
        10..=11 => {
            let store::Request::Transcript { read, rows: 4, bytes: 4096, .. } = request else { panic!("transcript") };
            let expected = [
                store::TranscriptRead::Window { task: 7, attempt: 2, turn: 3, after: None },
                store::TranscriptRead::Page { task: 7, backwards: false, after: None },
            ];
            assert_eq!(read, expected[index.checked_sub(10).unwrap()]);
        }
        12 => {
            let store::Request::Load {
                range: store::Load::Notes(jig_domain_notes::Range::Entry { name: 5 }),
                after: None,
                rows: 4,
                bytes: 4096,
                ..
            } = request
            else {
                panic!("notes")
            };
        }
        _ => panic!("unknown table row"),
    }
}

#[test]
fn a_forge_commit_puts_its_encoded_key_and_row() {
    let mut routes = fixtures::routes();
    let row = forge::Stored::Landed { commit: [9; 32], task: 3 };
    let request = engine::StoreRequest::Commit {
        number: 7,
        writes: Box::new([
            engine::Write::Save(engine::Record::Forge { row: Box::new(row) }),
            engine::Write::Erase(engine::Key::Forge(forge::Key::Landed([8; 32]))),
        ]),
    };
    let Some(store::Request::Commit { number: 7, writes }) = route::down(&mut routes, request) else {
        panic!("commit")
    };
    let store::Write::Put { key, value } = &writes[0] else { panic!("put") };
    let store::Row::Application { key: expected_key, value: expected_value } = forge_row(9) else { panic!("fixture") };
    assert_eq!(key, &expected_key);
    assert_eq!(value, &expected_value);
    let store::Write::Remove { key } = &writes[1] else { panic!("remove") };
    assert_eq!(key, &temper_protocol_store::keys::encode(&forge::Key::Landed([8; 32]), u32::MAX).unwrap());
}

#[test]
fn a_forge_page_reaches_the_root_decoded_with_its_last_key() {
    let mut routes = fixtures::routes();
    let Some(store::Request::Restore { owner, .. }) =
        route::down(&mut routes, load(engine::Range::Forge, None, 4, 4096))
    else {
        panic!("restore")
    };
    let mut out = Queue::with_capacity(2);
    route::up(
        &mut routes,
        store::Event::Restored {
            owner,
            rows: Box::new([forge_row(9)]),
            next: Some(store::Cursor { interval: 0, after: Box::new([2, 9]) }),
        },
        &mut out,
    );
    let (root, rows, next) = page(&mut out);
    assert_eq!(root, fixtures::OWNER);
    assert_eq!(
        rows.as_ref(),
        [engine::Record::Forge { row: Box::new(forge::Stored::Landed { commit: [9; 32], task: 3 }) }]
    );
    assert_eq!(next, Some(engine::Key::Forge(forge::Key::Landed([9; 32]))));
    assert!(routes.idle());
}

#[test]
fn a_continued_forge_load_asks_from_its_tagged_cursor() {
    let last = forge::Key::Landed([9; 32]);
    let Some(store::Request::Restore {
        range: store::Restore::Application(intervals),
        after: Some(cursor),
        rows: 4,
        ..
    }) = route::down(
        &mut fixtures::routes(),
        load(engine::Range::Forge, Some(engine::Key::Forge(last.clone())), 4, 4096),
    )
    else {
        panic!("forge restore")
    };
    assert_eq!(intervals.as_ref(), [temper_protocol_store::interval()]);
    let relative = temper_protocol_store::keys::encode(&last, u32::MAX).unwrap();
    let tagged = store::keys::application(&relative, u32::MAX).unwrap();
    assert_eq!(cursor, store::Cursor { interval: 0, after: tagged });
}

#[test]
fn a_page_keeps_the_longest_prefix_within_the_roots_bounds() {
    let mut routes = fixtures::routes();
    // Landed has no deep payload under the root's byte projection.
    let one_row = u32::try_from(size_of::<engine::Record>()).unwrap();
    for (most, bytes, expected) in [(1, 8192, 1), (4, one_row, 1), (4, 8192, 2)] {
        let Some(store::Request::Restore { owner, .. }) =
            route::down(&mut routes, load(engine::Range::Forge, None, most, bytes))
        else {
            panic!("restore")
        };
        let mut out = Queue::with_capacity(2);
        route::up(
            &mut routes,
            store::Event::Restored { owner, rows: Box::new([forge_row(8), forge_row(9)]), next: None },
            &mut out,
        );
        let (_, rows, next) = page(&mut out);
        assert_eq!(rows.len(), expected, "{most}/{bytes}");
        let expected_next = if expected == 1 { Some(engine::Key::Forge(forge::Key::Landed([8; 32]))) } else { None };
        assert_eq!(next, expected_next, "{most}/{bytes}");
    }
}

#[test]
fn a_row_that_does_not_decode_stops_the_engine() {
    let rows = [
        store::Row::Application { key: Box::new([0]), value: Box::new([255]) },
        store::Row::Application {
            key: temper_protocol_store::keys::encode(&forge::Key::Landed([8; 32]), u32::MAX).unwrap(),
            value: temper_protocol_store::records::encode(&forge::Stored::Landed { commit: [9; 32], task: 3 }),
        },
    ];
    for row in rows {
        let mut routes = fixtures::routes();
        route::down(&mut routes, engine::StoreRequest::Commit { number: 7, writes: Box::new([]) }).unwrap();
        let Some(store::Request::Restore { owner, .. }) =
            route::down(&mut routes, load(engine::Range::Forge, None, 4, 4096))
        else {
            panic!("restore")
        };
        let mut out = Queue::with_capacity(2);
        route::up(&mut routes, store::Event::Restored { owner, rows: Box::new([row]), next: None }, &mut out);
        let Some(Up::Root(event)) = out.pop() else { panic!("root event") };
        let engine::Event::Store(engine::StoreInput::Uncommitted { number: 7 }) = *event else {
            panic!("failed commit")
        };
        assert!(routes.idle());
    }
}

fn call_row() -> core::Record {
    core::Record::Core(core::CoreRecord::Call(core::CallRecord {
        key: core::CallKey { task: 1, attempt: 1, completion: 1, position: 1 },
        part: core::CallPart::Unavailable,
        settled: None,
    }))
}
#[test]
fn a_task_page_keeps_projections_then_live_tasks_and_drops_calls() {
    let projection = projection_row();
    let task = core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(fixtures::task_record(1))));
    let mut routes = fixtures::routes();
    let Some(store::Request::Restore { owner, .. }) =
        route::down(&mut routes, load(engine::Range::Core(engine::CoreRange::Tasks), None, 4, 8192))
    else {
        panic!("tasks")
    };
    let mut out = Queue::with_capacity(2);
    route::up(
        &mut routes,
        store::Event::Restored {
            owner,
            rows: Box::new([
                store::Row::Jig(projection.clone()),
                store::Row::Jig(call_row()),
                store::Row::Jig(task.clone()),
            ]),
            next: None,
        },
        &mut out,
    );
    let (_, rows, next) = page(&mut out);
    assert_eq!(rows.as_ref(), [engine::Record::Core(projection), engine::Record::Core(task)]);
    assert_eq!(next, None);
    let after = engine::Key::Core(core::Key::Tasks(jig_domain_tasks::Key::Live(1)));
    let Some(store::Request::Restore { after: Some(cursor), .. }) =
        route::down(&mut routes, load(engine::Range::Core(engine::CoreRange::Tasks), Some(after), 4, 8192))
    else {
        panic!("task continuation")
    };
    assert_eq!(cursor.interval, 1);
}

#[test]
fn a_filtered_page_that_keeps_nothing_asks_again_from_jigs_cursor() {
    let mut routes = fixtures::routes();
    let Some(store::Request::Restore { owner, .. }) =
        route::down(&mut routes, load(engine::Range::Core(engine::CoreRange::RunProofs), None, 4, 8192))
    else {
        panic!("proofs")
    };
    let cursor = store::Cursor { interval: 0, after: store::keys::encode(&call_row().key(), u32::MAX).unwrap() };
    let mut out = Queue::with_capacity(2);
    route::up(
        &mut routes,
        store::Event::Restored { owner, rows: Box::new([store::Row::Jig(call_row())]), next: Some(cursor.clone()) },
        &mut out,
    );
    let Some(Up::Ask(store::Request::Restore { owner: continued, after, rows: 4, bytes: 8192, .. })) = out.pop() else {
        panic!("continue filtered page")
    };
    assert_eq!(continued, owner);
    assert_eq!(after, Some(cursor));
    assert!(out.is_empty());
    assert!(!routes.idle());
    route::up(&mut routes, store::Event::Restored { owner, rows: Box::new([]), next: None }, &mut out);
    let (_, rows, next) = page(&mut out);
    assert!(rows.is_empty());
    assert_eq!(next, None);
    assert!(routes.idle());
}

#[test]
fn a_census_cursor_is_its_index_rows_key_and_interval() {
    let census = core::Census {
        results: Box::new([core::ReadResults { party: 4, after: 0 }, core::ReadResults { party: 8, after: 0 }]),
    };
    let index = core::Key::Tasks(jig_domain_tasks::Key::Result { party: 8, position: 2 });
    let mut routes = fixtures::routes();
    let Some(store::Request::Restore { owner, after: Some(cursor), .. }) =
        route::down(&mut routes, load(engine::Range::Census(census), Some(engine::Key::Core(index.clone())), 4, 8192))
    else {
        panic!("census continuation")
    };
    assert_eq!(cursor.interval, 2);
    assert_eq!(cursor.after, store::keys::encode(&index, u32::MAX).unwrap());
    let rows = Box::new([ended_row(5), ended_row(5), ended_row(3)]);
    let last = core::Key::Tasks(jig_domain_tasks::Key::Result { party: 8, position: 5 });
    let mut out = Queue::with_capacity(2);
    route::up(
        &mut routes,
        store::Event::Restored {
            owner,
            rows,
            next: Some(store::Cursor { interval: 2, after: store::keys::encode(&last, u32::MAX).unwrap() }),
        },
        &mut out,
    );
    let (_, rows, next) = page(&mut out);
    assert_eq!(rows.len(), 3);
    for (row, number) in rows.iter().zip([5, 5, 3]) {
        assert_eq!(row.key(), engine::Key::Core(core::Key::Tasks(jig_domain_tasks::Key::Ended(number))));
    }
    assert_eq!(next, Some(engine::Key::Core(last)));
}

fn ended_row(number: u64) -> store::Row {
    store::Row::Jig(core::Record::Tasks(jig_domain_tasks::Stored::Ended(Box::new(fixtures::task_record(number)))))
}

fn projection_row() -> core::Record {
    core::Record::Core(core::CoreRecord::Projection(core::ProjectionRecord::Task {
        goal: 1,
        task: core::ProjectionTask {
            number: 1,
            requester: jig_domain_tasks::Party::Deployment { project: 1 },
            words: Box::new([]),
            phase: jig_domain_tasks::Phase::Waiting,
        },
    }))
}

const TASK: u64 = 7;
const PROPOSAL: u64 = 19;
const REVISION: u64 = 3;
const ONE_ROW: u32 = 1;
const LOAD_BYTES: u32 = 8192;
const LOAD_OUTPUTS: u32 = 2;

struct DecidedRows {
    live: Option<jig_domain_tasks::TaskRecord>,
    history: Option<core::Record>,
    expected: Box<[engine::Record]>,
}
#[test]
fn a_task_proposal_load_follows_the_live_owners_history_pointer() {
    let history = fixtures::withdrawn_history(TASK, PROPOSAL);
    assert_decided_load(DecidedRows {
        live: Some(proposal_owner()),
        history: Some(history.clone()),
        expected: Box::new([engine::Record::Core(history)]),
    });
}
#[test]
fn a_task_proposal_load_without_its_owner_is_empty() {
    assert_decided_load(DecidedRows {
        live: None,
        history: Some(fixtures::withdrawn_history(TASK, PROPOSAL)),
        expected: Box::new([]),
    });
}
#[test]
fn a_task_proposal_load_without_its_pointer_is_empty() {
    assert_decided_load(DecidedRows {
        live: Some(fixtures::task_record(TASK)),
        history: Some(fixtures::withdrawn_history(TASK, PROPOSAL)),
        expected: Box::new([]),
    });
}
#[test]
fn a_task_proposal_load_without_its_history_is_empty() {
    assert_decided_load(DecidedRows { live: Some(proposal_owner()), history: None, expected: Box::new([]) });
}
fn proposal_owner() -> jig_domain_tasks::TaskRecord {
    let mut live = fixtures::task_record(TASK);
    live.proposal_rows = Box::new([jig_domain_tasks::ProposalRow { proposal: PROPOSAL, revision: REVISION }]);
    live
}
fn assert_decided_load(rows: DecidedRows) {
    use jig_domain_tasks as tasks;
    let mut routes = fixtures::routes();
    let range = engine::Range::Core(engine::CoreRange::TaskProposal { task: TASK, proposal: PROPOSAL });
    let Some(store::Request::Get { owner, key }) = route::down(&mut routes, load(range, None, ONE_ROW, LOAD_BYTES))
    else {
        panic!("live lookup")
    };
    let store::Get::Jig(key) = key else { panic!("jig owner") };
    assert_eq!(key, core::Key::Tasks(tasks::Key::Live(TASK)));
    let mut out = Queue::with_capacity(LOAD_OUTPUTS);
    let live = stored_live(rows.live);
    route::up(&mut routes, store::Event::Got { owner, row: live }, &mut out);
    if let Some(Up::Ask(store::Request::Get { owner: history_owner, key })) = out.iter().next() {
        assert_eq!(*history_owner, owner, "retained route");
        let store::Get::Jig(key) = key else { panic!("jig history") };
        assert_eq!(*key, core::Key::Tasks(tasks::Key::History { task: TASK, revision: REVISION }));
        out.pop().expect("history lookup");
        let history = stored_history(rows.history);
        route::up(&mut routes, store::Event::Got { owner, row: history }, &mut out);
    }
    let (load_owner, loaded, next) = page(&mut out);
    assert_eq!((load_owner, loaded.as_ref(), next), (fixtures::OWNER, rows.expected.as_ref(), None));
    assert!(out.is_empty());
    assert!(routes.idle());
}

fn stored_live(row: Option<jig_domain_tasks::TaskRecord>) -> Option<store::Row> {
    Some(store::Row::Jig(core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(row?)))))
}
fn stored_history(row: Option<core::Record>) -> Option<store::Row> {
    Some(store::Row::Jig(row?))
}
