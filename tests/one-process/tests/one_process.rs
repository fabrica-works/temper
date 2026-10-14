use skein_world::domain::Verdict;
use temper_one_process_world::{fixtures, world};
#[test]
fn a_person_signs_in_by_the_printed_link_and_a_chat_is_answered() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::Chat { words: b"Hello" });
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.chats_answered, 1);
}
