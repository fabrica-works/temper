//! The tools' real pipes meet the standalone service and hosted agent.
use skein_world::domain::heap::Counting;
#[global_allocator]
static HEAP: Counting = Counting;
#[test]
fn a_signed_in_chat_shows_its_reply_only_once_committed() {
    use temper_real_loop_world::{
        Exit, argv,
        calls::{D1, call, occurrences, sign_in, task, until},
        run, scripts,
    };
    let ran = run(
        || scripts::chat_scripts(1),
        || {
            vec![
                sign_in(D1),
                call(D1, argv!["chat", "--project", "1", "--key", "first", scripts::WORDS]),
                until(D1, argv!["chat", task(1)], scripts::REPLY, scripts::EARLY),
            ]
        },
    );
    let [signed, chat, read] = ran.calls() else { panic!("three calls") };
    assert_eq!(
        [signed.exit, chat.exit, read.exit],
        [Some(Exit::Code(0)); 3],
        "{}",
        String::from_utf8_lossy(&chat.stderr)
    );
    assert_eq!((read.early, occurrences(&read.stdout, scripts::REPLY)), (0, 1), "{} reads", read.tries);
}
#[test]
fn a_signed_in_directory_is_private_to_its_user() {
    use std::os::unix::fs::MetadataExt;
    use temper_real_loop_world::{
        Exit,
        calls::{D1, sign_in},
        fixtures, run, scripts,
    };
    let ran = run(|| scripts::chat_scripts(1), || vec![sign_in(D1)]);
    assert_eq!(ran.calls()[0].exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&ran.calls()[0].stderr));
    let directory = fixtures::directory(ran.path(), D1);
    for (path, mode) in
        [(directory.clone(), 0o700), (directory.join("token"), 0o600), (directory.join("address"), 0o600)]
    {
        let metadata = std::fs::metadata(&path).expect("signed-in metadata");
        assert_eq!(
            (metadata.mode() & 0o777, metadata.uid()),
            (mode, skein_shell::effective_user()),
            "{}",
            path.display()
        );
    }
}
