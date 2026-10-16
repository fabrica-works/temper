use jig_conformance::Cut;
use temper_conformance_world::stories::{self, Story};
#[test]
fn chat_change_lands_only_after_current_head_ci_and_review() {
    stories::run(Story::ReviewedChange, Cut::None, 71).expect("reviewed change");
}
#[test]
fn failed_ci_is_repaired_and_the_new_head_is_reviewed() {
    stories::run(Story::RepairCi, Cut::None, 71).expect("CI repair");
}
#[test]
fn another_hand_holds_the_change_and_preserves_its_branch() {
    stories::run(Story::OtherHand, Cut::None, 71).expect("other hand");
}
#[test]
fn lost_merge_answer_is_found_at_its_head_after_restart() {
    stories::run(Story::LostMerge, Cut::None, 71).expect("merge recovery");
}
#[test]
fn person_words_reach_the_live_coder_after_durability() {
    stories::run(Story::PersonWords, Cut::None, 71).expect("person words");
}
#[test]
fn a_goal_issue_follows_its_milestone_and_closes() {
    stories::run(Story::GoalIssue, Cut::None, 71).expect("goal milestones");
}
#[test]
fn lost_issue_creation_answer_recovers_without_a_second_issue() {
    stories::run(Story::IssueCreation, Cut::None, 71).expect("issue creation uncertainty");
}
/// Crash one native story at its configured receipt cuts, including writer
/// restoration, an already deleted branch, a lost milestone comment and a
/// retained live-run word; its baseline runs above and in the fuzzy sweep.
/// Each chosen cut must fire.
fn crashed_at(story: Story, cuts: [u64; 2]) {
    for cut in cuts {
        let outcome = stories::run(story, Cut::AfterCommit(cut), 101)
            .unwrap_or_else(|error| panic!("{story:?}, cut {cut}: {error:?}"));
        assert!(outcome.crashes.contains(&cut), "{story:?} cut {cut} did not fire");
    }
}

#[test]
fn a_reviewed_change_recovers_its_focused_commit_cuts() {
    crashed_at(Story::ReviewedChange, [1, 17]);
}

#[test]
fn a_ci_repair_recovers_its_focused_commit_cuts() {
    crashed_at(Story::RepairCi, [1, 23]);
}

#[test]
fn a_goal_issue_recovers_its_focused_commit_cuts() {
    crashed_at(Story::GoalIssue, [21, 23]);
}

#[test]
fn another_hand_recovers_its_focused_commit_cuts() {
    crashed_at(Story::OtherHand, [1, 17]);
}

#[test]
fn a_lost_merge_recovers_its_focused_commit_cuts() {
    crashed_at(Story::LostMerge, [1, 18]);
}

#[test]
fn person_words_recover_a_retained_live_run_word() {
    crashed_at(Story::PersonWords, [1, 9]);
}

#[test]
fn an_issue_creation_recovers_its_focused_commit_cuts() {
    crashed_at(Story::IssueCreation, [1, 7]);
}

#[test]
fn a_ci_repair_keeps_its_settle_wait_across_restart() {
    let outcome = stories::run(Story::RepairCi, Cut::AfterCommit(32), 101).expect("settling survives restart");
    assert!(outcome.crashes.contains(&32));
}
