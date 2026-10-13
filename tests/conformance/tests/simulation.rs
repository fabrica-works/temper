use jig_conformance::{Application, Harness};
use temper_conformance_world::{Config, Temper};
#[test]
fn temper_starts_adopts_and_assigns_through_the_shared_referee() {
    let mut world = Harness::<Temper>::new(Config::new(71, None), 71);
    let result = world.drain();
    assert!(
        result.is_ok(),
        "{result:?} ready={} quiet={} trace={:?}",
        Temper::ready(&world.domain),
        Temper::quiescent(&world.domain),
        &world.trace[world.trace.len().saturating_sub(12)..]
    );
    assert!(world.peers.adopted);
    assert_eq!(world.peers.assignments.len(), 1);
}
#[test]
fn an_idempotent_forge_set_tolerates_its_late_copy() {
    jig_conformance::scenarios::run::<Temper>(
        jig_conformance::scenarios::Scenario::LateCopy,
        jig_conformance::Cut::None,
        71,
    )
    .expect("late copy")
    .expect("applicable");
}
#[test]
fn an_uncertain_status_set_keeps_its_deadline_across_restarts() {
    jig_conformance::scenarios::run::<Temper>(
        jig_conformance::scenarios::Scenario::UncertainRestarts,
        jig_conformance::Cut::None,
        71,
    )
    .expect("uncertain restart")
    .expect("applicable");
}
#[test]
fn a_standing_task_runs_once_per_funding_period() {
    jig_conformance::scenarios::run::<Temper>(
        jig_conformance::scenarios::Scenario::StandingTask,
        jig_conformance::Cut::None,
        71,
    )
    .expect("standing task")
    .expect("applicable");
}
#[test]
fn many_completions_use_one_run_allowance() {
    jig_conformance::scenarios::run::<Temper>(
        jig_conformance::scenarios::Scenario::RunBudget,
        jig_conformance::Cut::None,
        71,
    )
    .expect("run budget")
    .expect("applicable");
}

#[test]
fn remaining_shared_scenarios_use_real_forge_and_people_operations() {
    for scenario in [
        jig_conformance::scenarios::Scenario::OtherHand,
        jig_conformance::scenarios::Scenario::ChangedJudge,
        jig_conformance::scenarios::Scenario::NarrowingPolicy,
    ] {
        jig_conformance::scenarios::run::<Temper>(scenario, jig_conformance::Cut::None, 71)
            .unwrap_or_else(|error| panic!("{scenario:?}: {error:?}"))
            .expect("applicable");
    }
}

/// Crash one shared scenario at its configured entry, middle and closing
/// receipt cuts; its baseline runs in the focused cases above and in the
/// fuzzy sweep. Each chosen cut must fire.
fn crashed_at(scenario: jig_conformance::scenarios::Scenario, cuts: [u64; 3]) {
    for cut in cuts {
        let outcome = jig_conformance::scenarios::run::<Temper>(scenario, jig_conformance::Cut::AfterCommit(cut), 91)
            .unwrap_or_else(|error| panic!("{scenario:?} at {cut}: {error:?}"))
            .expect("applicable");
        assert!(outcome.crashes.contains(&cut), "{scenario:?} cut {cut} did not fire");
    }
}

#[test]
fn a_late_copy_keeps_its_promises_at_focused_crash_cuts() {
    crashed_at(jig_conformance::scenarios::Scenario::LateCopy, [1, 6, 12]);
}

#[test]
fn another_hand_keeps_its_promises_at_focused_crash_cuts() {
    crashed_at(jig_conformance::scenarios::Scenario::OtherHand, [1, 15, 30]);
}

#[test]
fn uncertain_restarts_keep_their_promises_at_focused_crash_cuts() {
    crashed_at(jig_conformance::scenarios::Scenario::UncertainRestarts, [1, 7, 15]);
}

#[test]
fn a_changed_judge_keeps_its_promises_at_focused_crash_cuts() {
    crashed_at(jig_conformance::scenarios::Scenario::ChangedJudge, [1, 22, 45]);
}

#[test]
fn a_narrowing_policy_keeps_its_promises_at_focused_crash_cuts() {
    crashed_at(jig_conformance::scenarios::Scenario::NarrowingPolicy, [1, 12, 24]);
}

#[test]
fn a_standing_task_keeps_its_promises_at_focused_crash_cuts() {
    crashed_at(jig_conformance::scenarios::Scenario::StandingTask, [1, 12, 24]);
}

#[test]
fn a_run_budget_keeps_its_promises_at_focused_crash_cuts() {
    crashed_at(jig_conformance::scenarios::Scenario::RunBudget, [1, 10, 21]);
}
