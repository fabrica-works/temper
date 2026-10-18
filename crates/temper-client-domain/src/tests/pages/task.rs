use super::fixtures::{self, chat as f};
use crate::pages::{
    Request,
    task::{Shape, Task},
};
use jig_client::{Event, Readable, Told, Watchable, contract as c};
use skein_lib::Queue;
const OUTPUTS: u32 = 16;
const WINDOW: u32 = 2;
const EMPTY_OFFSET: u32 = 0;
const FIRST: u64 = f::FIRST;
const PRIORITY: u32 = 1;
fn opened() -> (Task, f::Fixture) {
    let mut fixture = f::opened();
    let mut page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut Queue::with_capacity(OUTPUTS));
    while fixture.out.pop().is_some() {}
    page.told(&fixture.client, Told::Changed { handle: page.document }, &mut fixture.out);
    (page, fixture)
}
#[test]
fn opening_a_task_watches_task_and_tree_and_reads_activity() {
    let mut out = Queue::with_capacity(OUTPUTS);
    let page = Task::new(f::TASK, FIRST, &fixtures::limits(), &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: page.document,
            subject: Watchable::Jig(c::Subject::Task { task: f::TASK }),
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Watch {
            handle: page.tree,
            subject: Watchable::Jig(c::Subject::Tree { task: f::TASK, window: WINDOW }),
        }))
    );
    assert_eq!(
        out.pop(),
        Some(Request::Client(Event::Read {
            handle: page.activity,
            read: Readable::Jig(c::Read::Activity { task: f::TASK, before: None }),
        }))
    );
    assert!(out.is_empty());
}
#[test]
fn a_live_run_on_a_tasks_page_is_watched() {
    let (page, mut f) = opened();
    assert_eq!(
        f.out.pop(),
        Some(Request::Client(Event::Watch {
            handle: page.run.expect("run handle"),
            subject: Watchable::Jig(c::Subject::Run { task: f::TASK, turn: None, offset: EMPTY_OFFSET }),
        }))
    );
}
#[test]
fn a_tasks_live_run_is_unwatched_when_it_ends() {
    let (mut page, mut f) = opened();
    let run = page.run.expect("live run");
    while f.out.pop().is_some() {}
    let mut task = f::task();
    f::phase(&mut task, c::OwnerChipPhase::Active(c::RunStage::Idle));
    f::deliver(&mut f.client, f.watch, task);
    page.told(&f.client, Told::Changed { handle: page.document }, &mut f.out);
    assert_eq!(page.run, None);
    assert_eq!(f::handles(&mut f.out).as_ref(), [run]);
}
#[test]
fn leaving_a_task_unwatches_each_of_its_handles() {
    let (page, mut f) = opened();
    while f.out.pop().is_some() {}
    page.leave(&mut f.out);
    assert_eq!(f::handles(&mut f.out).as_ref(), [page.document, page.tree, page.activity, page.run.expect("run")]);
}
#[test]
fn only_a_tracked_agents_task_gets_the_full_page() {
    const CHARTER: u32 = 1;
    const PARTY: u64 = 73;
    const PROCEDURE: u32 = 2;
    const CASES: &[(Option<u32>, c::Executor, Shape)] = &[
        (Some(PRIORITY), c::Executor::Agent { charter: CHARTER }, Shape::Tracked),
        (None, c::Executor::Agent { charter: CHARTER }, Shape::HeaderAndPlan),
        (Some(PRIORITY), c::Executor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE }, Shape::HeaderAndPlan),
        (Some(PRIORITY), c::Executor::Party { party: PARTY }, Shape::HeaderAndPlan),
    ];
    for (priority, executor, expected) in CASES {
        let (mut page, mut f) = opened();
        let mut task = f::task();
        task.priority = *priority;
        task.chip.executor = *executor;
        f::deliver(&mut f.client, f.watch, task);
        page.told(&f.client, Told::Changed { handle: page.document }, &mut f.out);
        assert_eq!(page.shape, *expected, "{priority:?}, {executor:?}");
    }
}
