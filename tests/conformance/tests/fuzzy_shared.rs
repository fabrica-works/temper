use jig_conformance::{
    Cut,
    scenarios::{self, Scenario},
};
use temper_conformance_world::Temper;

/// Crash one shared scenario at every durable commit of its baseline.
fn crashed_at_every_commit(scenario: Scenario) {
    let baseline = scenarios::run::<Temper>(scenario, Cut::None, 101).expect("baseline").expect("applicable");
    eprintln!("{scenario:?}: {} durable commit cuts", baseline.commits);
    for cut in 1..=baseline.commits {
        scenarios::run::<Temper>(scenario, Cut::AfterCommit(cut), 101)
            .unwrap_or_else(|error| panic!("{scenario:?}, cut {cut}: {error:?}"));
    }
}

#[test]
fn a_late_copy_is_crashed_at_every_commit() {
    crashed_at_every_commit(Scenario::LateCopy);
}

#[test]
fn another_hand_is_crashed_at_every_commit() {
    crashed_at_every_commit(Scenario::OtherHand);
}

#[test]
fn uncertain_restarts_are_crashed_at_every_commit() {
    crashed_at_every_commit(Scenario::UncertainRestarts);
}

#[test]
fn a_changed_judge_is_crashed_at_every_commit() {
    crashed_at_every_commit(Scenario::ChangedJudge);
}

#[test]
fn a_narrowing_policy_is_crashed_at_every_commit() {
    crashed_at_every_commit(Scenario::NarrowingPolicy);
}

#[test]
fn a_standing_task_is_crashed_at_every_commit() {
    crashed_at_every_commit(Scenario::StandingTask);
}

#[test]
fn a_run_budget_is_crashed_at_every_commit() {
    crashed_at_every_commit(Scenario::RunBudget);
}

#[test]
fn a_shrinking_pool_is_not_applicable() {
    let outcome = scenarios::run::<Temper>(Scenario::ShrinkingPool, Cut::None, 101).expect("not applicable");
    assert!(outcome.is_none(), "temper's forge has no pooled holds");
}
