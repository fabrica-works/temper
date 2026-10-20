//! Landing spans chat, coding and review, then the chat's reply after the merge.
//! Four agent lifetimes, seven LLM turns and 23 hosted git calls exceed the 1 s aim.
use skein_world::domain::heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;

#[test]
fn a_chat_lands_its_change_on_the_forge() {
    use temper_real_loop_world::{
        Exit, argv,
        calls::{D1, call, delegate, digest, sign_in, task},
        forge, landing, run_forge,
    };
    // Sign-in and adoption precede the chat; its replied wait precedes the tree read.
    const CHAT: usize = 2;
    const TREE: usize = 4;
    // The final call waits for the parent chat to finish its reply after landing.
    const REPLIED: usize = 6;
    let ran = run_forge(
        landing::fixture(),
        |at| landing::scripts(at, u64::from(forge::ID), landing::README),
        || {
            vec![
                sign_in(D1),
                call(
                    D1,
                    argv![
                        "adopt",
                        "--project",
                        "1",
                        "--owner",
                        "acme",
                        "--name",
                        "app",
                        "--landing",
                        "main",
                        "--key",
                        "adopt-app"
                    ],
                ),
                call(D1, argv!["chat", "--project", "1", "--key", "land", "@forge_chat Land a small change"]),
                call(D1, argv!["wait", "task", task(CHAT), "--until", "replied"]),
                call(D1, argv!["task", task(CHAT), "--tree", "--json"]),
                call(D1, argv!["wait", "task", delegate(TREE), "--until", "ended"]),
                call(D1, argv!["wait", "task", task(CHAT), "--since", digest(TREE), "--until", "replied"]),
            ]
        },
    );
    assert_eq!(ran.calls()[REPLIED].exit, Some(Exit::Code(0)), "{:?}", ran.calls()[REPLIED]);
    assert!(ran.forge().merged_green() && ran.forge().main_file(b"README.md").ends_with(landing::EDIT));
}
