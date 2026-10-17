pub mod support;
use std::{fs, os::unix::fs::PermissionsExt, process::Command};
use temper_forge_setup_shell::process;
#[test]
fn a_second_preparation_is_refused_while_the_first_holds_the_lock() {
    let scratch = support::Scratch::new("forge_prepare_held");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let _lock = process::lock(&scratch.path().join("forge-1/forgejo.lock")).expect("first holds lock");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge prepare: forge 1: forgejo is running or being prepared already\n");
    scratch.remove();
}
#[test]
fn a_preparation_cut_short_is_begun_again_from_an_empty_layout() {
    let scratch = support::Scratch::new("forge_cut_short");
    let programs = support::owned::scripted(scratch.path());
    programs.fail("forgejo", "admin");
    let first = support::owned::prepare(scratch.path(), 1);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stderr, b"temper forge prepare: forge 1: CreateAdmin ended with status 1: broken\n");
    fs::write(scratch.path().join("forge-1/forgejo/left-over"), b"x").expect("left-over");
    programs.mend("forgejo");
    let second = support::owned::prepare(scratch.path(), 1);
    assert_eq!(second.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&second.stderr));
    assert!(!scratch.path().join("forge-1/forgejo/left-over").exists());
    let state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"));
    assert!(state.expect("a state").prepared, "state ends last");
    scratch.remove();
}
#[test]
fn forgejo_files_with_no_state_are_refused_naming_the_directory() {
    let scratch = support::Scratch::new("forge_no_state");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    fs::remove_file(scratch.path().join("forge-1/state")).expect("no state");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        format!(
            "temper forge prepare: forge 1: {}: Forgejo files have no preparation state\n",
            scratch.path().join("forge-1").display()
        )
    );
    scratch.remove();
}
#[test]
fn a_later_preparation_takes_forgejos_lock_alone() {
    let scratch = support::Scratch::new("forge_later_lock");
    let _programs = support::owned::scripted(scratch.path());
    let first = support::owned::prepare(scratch.path(), 1);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    let _lock = process::lock(&scratch.path().join("forge-1/runner.lock")).expect("runner runs");
    let later = support::owned::prepare(scratch.path(), 1);
    assert_eq!(later.status.code(), Some(0), "{}", String::from_utf8_lossy(&later.stderr));
    scratch.remove();
}
#[test]
fn a_first_preparation_refuses_a_running_runner() {
    let scratch = support::Scratch::new("forge_first_runner_lock");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"))
        .expect("record");
    state.prepared = false;
    support::write_private(&scratch.path().join("forge-1/state"), temper_forge_setup::state::encode(&state).expose());
    let _lock = process::lock(&scratch.path().join("forge-1/runner.lock")).expect("runner runs");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge prepare: forge 1: runner is running or being prepared already\n");
    scratch.remove();
}
#[test]
fn a_forge_directory_open_to_others_is_refused_naming_it() {
    let scratch = support::Scratch::new("forge_directory_open");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    fs::set_permissions(scratch.path().join("forge-1"), fs::Permissions::from_mode(0o755)).expect("open directory");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        format!(
            "temper forge prepare: forge 1: {}: must be a private directory of this user's, not reached through a link\n",
            scratch.path().join("forge-1").display()
        )
    );
    scratch.remove();
}
#[test]
fn a_termination_signal_kills_the_commands_group_and_ends_the_preparation_interrupted() {
    let scratch = support::Scratch::new("forge_interrupted");
    let programs = support::owned::scripted(scratch.path());
    programs.hang("forgejo", "migrate");
    let run = Command::new(env!("CARGO_BIN_EXE_temper"))
        .args(["forge", "prepare", "config.json", "1"])
        .current_dir(scratch.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("preparation");
    let sleep = programs.hung(support::WITHIN);
    assert!(
        Command::new("/bin/sh").args(["-c", &format!("kill -TERM {}", run.id())]).status().expect("signal").success()
    );
    let ended = run.wait_with_output().expect("preparation ends");
    assert_eq!(ended.status.code(), Some(1));
    assert_eq!(ended.stderr, b"temper forge prepare: forge 1: interrupted\n");
    let stat = fs::read_to_string(format!("/proc/{sleep}/stat"));
    assert!(stat.is_err() || stat.expect("stat").contains(") Z "), "the sleep is dead");
    let state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"))
        .expect("record");
    assert!(!state.prepared);
    scratch.remove();
}
