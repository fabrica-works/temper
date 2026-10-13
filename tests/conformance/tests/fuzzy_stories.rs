use jig_conformance::Cut;
use temper_conformance_world::stories::{self, Story};

/// Crash one native story at every durable commit of its baseline.
fn crashed_at_every_commit(story: Story) {
    let baseline = stories::run(story, Cut::None, 101).expect("baseline");
    eprintln!("{story:?}: {} durable commit cuts, {} baseline iterations", baseline.commits, baseline.iterations);
    for cut in 1..=baseline.commits {
        stories::run(story, Cut::AfterCommit(cut), 101)
            .unwrap_or_else(|error| panic!("{story:?}, cut {cut}: {error:?}"));
    }
}

#[test]
fn a_reviewed_change_is_crashed_at_every_commit() {
    crashed_at_every_commit(Story::ReviewedChange);
}

#[test]
fn a_ci_repair_is_crashed_at_every_commit() {
    crashed_at_every_commit(Story::RepairCi);
}

#[test]
fn a_goal_issue_is_crashed_at_every_commit() {
    crashed_at_every_commit(Story::GoalIssue);
}

#[test]
fn another_hand_is_crashed_at_every_commit() {
    crashed_at_every_commit(Story::OtherHand);
}

#[test]
fn a_lost_merge_is_crashed_at_every_commit() {
    crashed_at_every_commit(Story::LostMerge);
}

#[test]
fn person_words_are_crashed_at_every_commit() {
    crashed_at_every_commit(Story::PersonWords);
}

#[test]
fn an_issue_creation_is_crashed_at_every_commit() {
    crashed_at_every_commit(Story::IssueCreation);
}
