//! A chat's delegation card through the shipped terminal, against client/README.md, 5.7: a QA pass run by `--profile qa`.
pub mod support;

use skein_world::end_to_end::{Binary, Command, Mode};
use support::{Scratch, WITHIN, configuration, terminal};

#[test]
fn running_card_names_the_change_and_exposes_its_steps() {
    // Choose the demo explicitly so this check is independent of the default
    // project's first-run banner and suggested-request setup.
    let scratch = Scratch::startup("qa-delegation-running");
    let mut arguments = vec!["box".into()];
    arguments.extend(configuration::flags(scratch.path()));
    let mut boxed = Binary::start(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("the first run starts");
    let link = terminal::link(&mut boxed, WITHIN);
    let opened = terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    person.wait_for(b"Chats", WITHIN);
    person.type_on(b"Chats", b"");
    person.wait_for(b"Start a chat: (empty) (enter sends)", WITHIN);
    person.type_on(b"demo", b"");
    person.type_on(b"Start a chat: (empty) (enter sends)", b"Add CSV export to widget");
    person.type_on(b"Start", b"");
    person.wait_for("T2 · CHANGE".as_bytes(), WITHIN);
    person.wait_for(b"Running", WITHIN);
    person.wait_for(b"Export widget's table as CSV", WITHIN);
    person.wait_for(b"Steps", WITHIN);
    person.wait_for(b"Branch pushed", WITHIN);
    person.wait_for(b"Pull request opened", WITHIN);
    person.wait_for(b"CI passed", WITHIN);
    person.wait_for(b"Landed", WITHIN);
    person.type_on(b"Task 2", b"");
    person.wait_for(b"== Export widget's table as CSV ==", WITHIN);
    person.type_line(b"q");
    assert_eq!(person.exit(WITHIN), 0, "{}", person.stderr());
    scratch.remove();
}

#[test]
fn result_anchor_preserves_the_landed_words_and_task_link() {
    let scratch = Scratch::startup("qa-delegation-landed-words");
    let mut arguments = vec!["box".into()];
    arguments.extend(configuration::flags(scratch.path()));
    let mut boxed = Binary::start(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("the first run starts");
    let link = terminal::link(&mut boxed, WITHIN);
    let opened = terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    person.wait_for(b"Chats", WITHIN);
    person.type_on(b"Chats", b"");
    person.wait_for(b"Start a chat: (empty) (enter sends)", WITHIN);
    person.type_on(b"demo", b"");
    person.type_on(b"Start a chat: (empty) (enter sends)", b"Add CSV export to widget");
    person.type_on(b"Start", b"");
    person.wait_for("CSV export is available from widget’s table. The change landed on Demo forge. This was a scripted demo change.".as_bytes(), WITHIN);
    // This contiguous block checks the title and the first seven characters
    // of the result, and keeps the four stand-in steps in their design order.
    person.wait_for(
        "Task 2:\n    T2 · CHANGE\n    * Landed\n    ### Export widget's table as CSV\n    CI passed. Landed · merge commit 0000000\n    Steps:\n      - Branch pushed\n      - Pull request opened\n      - CI passed\n      - Landed\n".as_bytes(),
        WITHIN,
    );
    person.type_on(b"Task 2", b"");
    person.wait_for(b"== Export widget's table as CSV ==", WITHIN);
    person.type_line(b"q");
    assert_eq!(person.exit(WITHIN), 0, "{}", person.stderr());
    scratch.remove();
}

#[test]
#[ignore = "the presets' turn text budget (CUE_BYTES, 256 B) cuts the delegate call's acknowledgement, so its call anchor names no task until jig's thread keys a call's tasks (client README 11)"]
fn original_delegate_call_uses_the_ended_task_document() {
    let scratch = Scratch::startup("qa-delegation-original-call");
    let mut arguments = vec!["box".into()];
    arguments.extend(configuration::flags(scratch.path()));
    let mut boxed = Binary::start(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("the first run starts");
    let link = terminal::link(&mut boxed, WITHIN);
    let opened = terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut person = terminal::Typist::new(opened, boxed, link);
    person.wait_for(b"Chats", WITHIN);
    person.type_on(b"Chats", b"");
    person.wait_for(b"Start a chat: (empty) (enter sends)", WITHIN);
    person.type_on(b"demo", b"");
    person.type_on(b"Start a chat: (empty) (enter sends)", b"Add CSV export to widget");
    person.type_on(b"Start", b"");
    person.wait_for("CSV export is available from widget’s table. The change landed on Demo forge. This was a scripted demo change.".as_bytes(), WITHIN);
    // Walk past the wake, its heard result, and the wait turn to the
    // original delegate call. It must use the ended task's own document.
    person.type_on(b"Older", b"");
    person.wait_for(b"* Earlier lines omitted 3\n  0000000000000003000000000000000000000000\n  Run 3 began", WITHIN);
    person.type_on(b"Older", b"");
    person.wait_for(b"* Earlier lines omitted 5\n  Agent\n  wait\n  Run 1 ended", WITHIN);
    person.type_on(b"Older", b"");
    person.wait_for(
        "I’ll delegate this change in demo/widget. It will push a branch, open a pull request, check CI and land into main on Demo forge.\n  Task 2:\n    T2 · CHANGE\n    * Landed\n    ### Export widget's table as CSV\n    CI passed. Landed · merge commit 0000000\n".as_bytes(),
        WITHIN,
    );
    person.type_on(b"Task 2", b"");
    person.wait_for(b"== Export widget's table as CSV ==", WITHIN);
    person.type_line(b"q");
    assert_eq!(person.exit(WITHIN), 0, "{}", person.stderr());
    scratch.remove();
}
