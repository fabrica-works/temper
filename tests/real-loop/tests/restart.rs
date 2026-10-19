//! A new standalone preparation reads the work the drained service kept.
use skein_world::domain::heap::Counting;
use temper_real_loop_world::{
    Exit, argv,
    calls::{D1, call, number, occurrences, sign_in, task, until},
    fixtures, llm, run, run_on, scripts,
};
#[global_allocator]
static HEAP: Counting = Counting;
// One chat is created after sign-in; its last read follows the restarted sign-in.
const CHAT: usize = 1;
const READ: usize = 1;
// Only that chat's first run consumes a scripted turn; the restart consumes none.
const CHATS: usize = 1;
#[test]
fn a_restarted_service_keeps_the_chat_with_its_reply() {
    let first = run(
        || scripts::chat_scripts(CHATS),
        || {
            let mut steps = fixtures::signed_chat("kept");
            steps.push(until(D1, argv!["chat", task(CHAT)], scripts::REPLY, scripts::EARLY));
            steps
        },
    );
    let chat = first.number(CHAT);
    let second = run_on(
        first.into_scratch(),
        || llm::scripted(scripts::chat_scripts(CHATS)),
        move || vec![sign_in(D1), call(D1, argv!["chat", number(chat)])],
    );
    let read = &second.calls()[READ];
    assert_eq!(
        (read.exit, occurrences(&read.stdout, scripts::REPLY)),
        (Some(Exit::Code(0)), 1),
        "{}",
        String::from_utf8_lossy(&read.stderr)
    );
    let asked = second.llm().iter().any(|seen| matches!(seen, skein_fake_peers::llm::Observation::Query { .. }));
    assert!(!asked, "the restarted service asked the model again");
}
