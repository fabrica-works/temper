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
fn native_stories_recover_their_focused_commit_cuts() {
    // Configured receipt cuts, including writer restoration, an already deleted
    // branch, a lost milestone comment and a retained live-run word.
    for (story, cuts) in [
        (Story::ReviewedChange, [1, 15]),
        (Story::RepairCi, [1, 21]),
        (Story::GoalIssue, [19, 23]),
        (Story::OtherHand, [1, 15]),
        (Story::LostMerge, [1, 16]),
        (Story::PersonWords, [1, 8]),
        (Story::IssueCreation, [1, 6]),
    ] {
        for cut in cuts {
            let outcome = stories::run(story, Cut::AfterCommit(cut), 101)
                .unwrap_or_else(|error| panic!("{story:?}, cut {cut}: {error:?}"));
            assert!(outcome.crashes.contains(&cut), "{story:?} cut {cut} did not fire");
        }
    }
}
