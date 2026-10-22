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
const HELD: u64 = 12;
const FIRST_CARD: u32 = 0;
#[test]
fn releasing_from_the_plan_names_the_holds_revision_the_card_showed() {
    use crate::pages::Act;
    let mut pages = fixtures::task::held_pages();
    let mut out = fixtures::acts::act(
        &mut pages,
        Act::Decide {
            object: c::ObjectKey::Task(HELD),
            revision: Some(fixtures::acts::REVISION),
            act: c::ActKind::Release,
        },
    );
    assert!(fixtures::acts::sent(&mut out).is_empty(), "release waits for confirmation");
    let mut out = fixtures::acts::act(&mut pages, Act::Confirm);
    assert_eq!(
        fixtures::acts::sent(&mut out).as_ref(),
        [c::Ask::Release { project: fixtures::PROJECT, task: HELD, revision: fixtures::acts::REVISION, again: None }]
    );
    assert!(fixtures::acts::sent(&mut fixtures::acts::act(&mut pages, Act::Confirm)).is_empty());
}
#[test]
fn a_moved_hold_is_refused_and_the_card_asks_to_look_again() {
    use crate::pages::Act;
    let mut pages = fixtures::task::held_pages();
    fixtures::acts::act(
        &mut pages,
        Act::Decide {
            object: c::ObjectKey::Task(HELD),
            revision: Some(fixtures::acts::REVISION),
            act: c::ActKind::Release,
        },
    );
    fixtures::acts::act(&mut pages, Act::Confirm);
    let key = pages.cards().expect("cards").get(FIRST_CARD).expect("held").key.expect("sent");
    let outcome = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Moved { now: fixtures::acts::NEW_REVISION, what: c::Moved::Hold { reason: None } },
        at: None,
    }));
    crate::acts::told(&mut pages, &Told::Answered { key, outcome: outcome.clone() });
    assert_eq!(pages.cards().expect("cards").get(FIRST_CARD).expect("held").outcome, Some(outcome));
    assert!(fixtures::acts::sent(&mut fixtures::acts::act(&mut pages, Act::Confirm)).is_empty());
}

use crate::pages::task::{State, header_state, link_state, row_state};
use skein_lib::{List, Wall};
const DEPENDENCY: u64 = 17;
const OTHER_DEPENDENCY: u64 = 23;
#[test]
fn a_tasks_state_says_what_it_waits_for() {
    let cases: &[(&[u64], bool, bool, State)] = &[
        (&[DEPENDENCY], false, false, State::WaitingFor(DEPENDENCY)),
        (&[DEPENDENCY], false, true, State::WaitingFor(DEPENDENCY)),
        (&[DEPENDENCY, OTHER_DEPENDENCY], false, false, State::WaitingForTasks { count: 2, more: false }),
        (&[DEPENDENCY], true, true, State::WaitingForTasks { count: 1, more: true }),
        (&[], true, true, State::WaitingForTasks { count: 0, more: true }),
        (&[], false, true, State::WaitingForHoldings),
        (&[], false, false, State::Phase(c::Phase::Waiting)),
    ];
    for (numbers, more, holds, expected) in cases {
        let mut task = f::task();
        let mut dependencies = List::with_capacity(WINDOW);
        for number in *numbers {
            dependencies.push(*number).expect("waiting dependencies");
        }
        f::phase(&mut task, c::OwnerChipPhase::Waiting { dependencies, more: *more, holds: *holds });
        assert_eq!(row_state(&task.chip), expected.clone());
    }
}
#[test]
fn an_ended_task_says_its_owners_ending() {
    for (ending, expected) in [
        (
            Some(c::OwnerChipEnding::Done(c::ResultValue::Report(f::document_text(b"Done")))),
            State::Ended(c::OwnerTaskStatus::Done),
        ),
        (Some(c::OwnerChipEnding::Failed(f::document_text(b"Failed"))), State::Ended(c::OwnerTaskStatus::Failed)),
        (
            Some(c::OwnerChipEnding::Cancelled {
                reason: f::document_text(b"Cancelled"),
                result: None,
                by: c::OwnerRequester::Person(fixtures::PARTY),
                at: Wall::EPOCH,
            }),
            State::Ended(c::OwnerTaskStatus::Cancelled),
        ),
        (None, State::Phase(c::Phase::Ended)),
    ] {
        let mut task = f::task();
        f::phase(&mut task, c::OwnerChipPhase::Ended);
        let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner") };
        details.ending = ending;
        assert_eq!(row_state(&task.chip), expected);
        task.waits_on = c::WaitsOn::News;
        assert_eq!(header_state(&task, None), expected);
    }
}
#[test]
fn another_phase_says_jigs_phase() {
    let mut task = f::task();
    f::phase(&mut task, c::OwnerChipPhase::Active(c::RunStage::Idle));
    let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner") };
    details.ending = Some(c::OwnerChipEnding::Failed(f::document_text(b"Old ending")));
    assert_eq!(row_state(&task.chip), State::Phase(task.chip.phase));
    task.chip.phase = c::Phase::Waiting;
    *task.chip.details = c::ChipPhase::Waiting {
        dependencies: {
            let mut dependencies = List::with_capacity(1);
            dependencies.push(DEPENDENCY).expect("dependency");
            dependencies
        },
        hold: None,
    };
    assert_eq!(row_state(&task.chip), State::Phase(c::Phase::Waiting));
}
#[test]
fn a_dependencys_state_comes_from_its_link() {
    let mut task = f::task();
    let mut dependencies = List::with_capacity(1);
    dependencies.push(DEPENDENCY).expect("dependency");
    f::phase(&mut task, c::OwnerChipPhase::Waiting { dependencies, more: false, holds: false });
    let chip = task.chip;
    let expected = State::WaitingFor(DEPENDENCY);
    assert_eq!(link_state(&c::OwnerTaskLink::Live(chip.clone())), Some(expected.clone()));
    assert_eq!(link_state(&c::OwnerTaskLink::Retained { chip, at: Some(Wall::EPOCH) }), Some(expected.clone()));
    for status in [c::OwnerTaskStatus::Done, c::OwnerTaskStatus::Failed, c::OwnerTaskStatus::Cancelled] {
        assert_eq!(
            link_state(&c::OwnerTaskLink::Stored { number: DEPENDENCY, status: Some(status) }),
            Some(State::Ended(status))
        );
    }
    assert_eq!(link_state(&c::OwnerTaskLink::Stored { number: DEPENDENCY, status: None }), None);
}

const ATTEMPT: u64 = f::ATTEMPT;
fn observation(work: c::OwnerChipWork) -> c::OwnerChipObservation {
    c::OwnerChipObservation {
        channel: skein_lib::Token::new(1),
        work,
        since: Wall::EPOCH,
        reported_at: Wall::EPOCH,
        started: None,
    }
}
#[test]
fn a_running_rows_state_says_its_observed_work() {
    for (work, expected) in [
        (None, State::Running),
        (Some(c::OwnerChipWork::Writing), State::Writing),
        (
            Some(c::OwnerChipWork::Tool { call: Box::from(*b"call"), tool: Box::from(*b"read_file") }),
            State::Tool(Box::from(*b"read_file")),
        ),
        (Some(c::OwnerChipWork::Preparing), State::Running),
        (Some(c::OwnerChipWork::Starting), State::Running),
        (Some(c::OwnerChipWork::Active), State::Running),
        (Some(c::OwnerChipWork::Waiting), State::Running),
        (Some(c::OwnerChipWork::Ending), State::Running),
    ] {
        let mut task = f::task();
        f::phase(&mut task, c::OwnerChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT }));
        let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner") };
        if let Some(work) = work {
            details.observation = Some(observation(work));
        } else {
            details.observation = None;
        }
        assert_eq!(row_state(&task.chip), expected);
        task.waits_on = c::WaitsOn::Nothing;
        assert_eq!(header_state(&task, None), expected);
    }
}
fn pending(numbers: &[u64]) -> List<u64> {
    let mut pending = List::with_capacity(WINDOW);
    for number in numbers {
        pending.push(*number).expect("pending task");
    }
    pending
}
#[test]
fn the_headers_dependencies_use_the_supplied_exact_count() {
    for (numbers, count, expected) in [
        (&[DEPENDENCY][..], 1, State::WaitingFor(DEPENDENCY)),
        (&[DEPENDENCY, OTHER_DEPENDENCY][..], 2, State::WaitingForTasks { count: 2, more: false }),
        (&[DEPENDENCY][..], 3, State::WaitingForTasks { count: 3, more: false }),
        (&[][..], 1, State::WaitingForTasks { count: 1, more: false }),
    ] {
        let mut task = f::task();
        task.waits_on = c::WaitsOn::Dependencies { pending: pending(numbers), count };
        assert_eq!(header_state(&task, None), expected);
    }
}
#[test]
fn the_headers_delegates_use_the_chips_live_count() {
    for count in [0, 1, 3] {
        let mut task = f::task();
        task.waits_on = c::WaitsOn::Delegates;
        let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner") };
        details.live_delegates = count;
        assert_eq!(header_state(&task, None), State::WaitingForDelegates(count));
    }
}
fn party_wait(whom: c::Whom) -> c::WaitsOn {
    c::WaitsOn::Party { whom, entries: 1, task: DEPENDENCY, kind: c::WaitedFor::Question }
}
#[test]
fn the_headers_party_is_you_only_for_the_person_or_a_held_project_role() {
    let mut task = f::task();
    let mut me = fixtures::chats::me();
    task.waits_on = party_wait(c::Whom::Person(me.party.number));
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForYou);
    assert_eq!(header_state(&task, None), State::WaitingForParty(c::Whom::Person(me.party.number)));
    task.waits_on = party_wait(c::Whom::Person(DEPENDENCY));
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(c::Whom::Person(DEPENDENCY)));
    for (held, number) in [
        (c::HeldRole::Owner, 0),
        (c::HeldRole::Maintainer, 1),
        (c::HeldRole::Member, 2),
        (c::HeldRole::Observer, 3),
        (c::HeldRole::Policy { role: 7 }, 7),
    ] {
        me.projects.get_mut(0).expect("project").role = held;
        let whom = c::Whom::Role { project: task.project, role: number };
        task.waits_on = party_wait(whom);
        assert_eq!(header_state(&task, Some(&me)), State::WaitingForYou);
        assert_eq!(header_state(&task, None), State::WaitingForParty(whom));
        let other_project = c::Whom::Role { project: task.project + 1, role: number };
        task.waits_on = party_wait(other_project);
        assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(other_project));
    }
    let whom = c::Whom::Role { project: task.project, role: 8 };
    task.waits_on = party_wait(whom);
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(whom));
    me.projects.clear();
    assert_eq!(header_state(&task, Some(&me)), State::WaitingForParty(whom));
}
#[test]
fn the_header_says_news_or_falls_back_to_the_row_when_nothing_is_pending() {
    let mut task = f::task();
    task.waits_on = c::WaitsOn::News;
    assert_eq!(header_state(&task, None), State::WaitingForNews);
    f::phase(&mut task, c::OwnerChipPhase::Waiting { dependencies: pending(&[DEPENDENCY]), more: false, holds: false });
    for waits_on in [c::WaitsOn::Nothing, c::WaitsOn::Dependencies { pending: pending(&[]), count: 0 }] {
        task.waits_on = waits_on;
        assert_eq!(header_state(&task, None), State::WaitingFor(DEPENDENCY));
    }
}
