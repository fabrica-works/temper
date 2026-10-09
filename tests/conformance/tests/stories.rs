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
#[test]
fn a_repair_writer_restores_before_its_next_push() {
    stories::run(Story::RepairCi, Cut::AfterCommit(21), 101).expect("restored repair writer");
}
#[test]
fn a_milestone_release_recovers_its_already_deleted_branch() {
    stories::run(Story::GoalIssue, Cut::AfterCommit(19), 101).expect("recovered milestone release");
}
