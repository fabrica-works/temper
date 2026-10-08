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

#[test]
fn shared_scenarios_keep_their_promises_at_focused_crash_cuts() {
    for scenario in jig_conformance::scenarios::ALL {
        let baseline =
            jig_conformance::scenarios::run::<Temper>(scenario, jig_conformance::Cut::None, 91).expect("baseline");
        let Some(baseline) = baseline else {
            assert_eq!(scenario, jig_conformance::scenarios::Scenario::ShrinkingPool);
            continue;
        };
        for cut in [1, baseline.commits / 2, baseline.commits] {
            jig_conformance::scenarios::run::<Temper>(scenario, jig_conformance::Cut::AfterCommit(cut), 91)
                .unwrap_or_else(|error| panic!("{scenario:?} at {cut}: {error:?}"));
        }
    }
}
