use skein_world::domain::Verdict;
use temper_one_process_world::{fixtures, world};

#[test]
fn a_chat_reads_then_replies_and_waits() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::ChatReadThenWait { words: b"Hello" });
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.chats_answered, 1);
}

#[test]
fn a_merge_in_progress_is_pushed_with_both_parents() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::Merge);
    assert_eq!(outcome.verdict, Verdict::Passed);
    let [pushed] = outcome.pushed.as_slice() else { panic!("one push: {:?}", outcome.pushed) };
    assert_eq!(pushed.parents, [fixtures::BRANCH_HEAD, fixtures::MOVED_BASE]);
}
