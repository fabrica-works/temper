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
        .args(["forge", "prepare"])
        .args(support::configuration::flags(scratch.path()))
        .arg("1")
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
    let clock = skein_shell::Clock::new();
    let deadline = clock.now().now.saturating_add(support::WITHIN);
    let stat = loop {
        let stat = fs::read_to_string(format!("/proc/{sleep}/stat"));
        if stat.is_err()
            || stat.as_ref().is_ok_and(|text| text.rsplit_once(") ").is_some_and(|(_, rest)| rest.starts_with("Z ")))
        {
            break stat;
        }
        assert!(clock.now().now < deadline, "the sleep exits within bound");
        std::thread::yield_now();
    };
    assert!(stat.is_err() || stat.expect("stat").contains(") Z "), "the sleep is dead");
    let state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"))
        .expect("record");
    assert!(!state.prepared);
    scratch.remove();
}
#[test]
fn a_second_run_is_refused_while_the_first_holds_the_lock() {
    let scratch = support::Scratch::new("forge_run_held");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut first = support::owned::command(scratch.path(), "forgejo").spawn().expect("first run");
    programs.marker("forgejo", support::WITHIN);
    let refused = support::owned::command(scratch.path(), "forgejo").output().expect("second run");
    first.kill().and_then(|()| first.wait()).expect("first stopped");
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge run: forge 1: forgejo is running or being prepared already\n");
    scratch.remove();
}
#[test]
fn the_run_replaces_itself_holding_the_lock_as_standard_input() {
    let scratch = support::Scratch::new("forge_run_exec");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut run = support::owned::command(scratch.path(), "forgejo").spawn().expect("run starts");
    let marker = programs.marker("forgejo", support::WITHIN);
    let stdin = fs::read_link(format!("/proc/{}/fd/0", run.id())).expect("stdin");
    run.kill().and_then(|()| run.wait()).expect("stopped");
    assert_eq!(marker.pid, run.id());
    assert_eq!(stdin, programs.lock("forgejo"));
    assert_eq!(
        marker.arguments,
        [
            "--work-path",
            scratch.path().join("forge-1/forgejo").to_str().expect("UTF-8 path"),
            "--config",
            scratch.path().join("forge-1/forgejo/app.ini").to_str().expect("UTF-8 path"),
            "web"
        ]
    );
    scratch.remove();
}
#[test]
fn the_new_process_sees_the_specs_environment_alone() {
    let scratch = support::Scratch::new("forge_run_environment");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut run = support::owned::command(scratch.path(), "forgejo").env("LEAKED", "private").spawn().expect("run");
    let mut marker = programs.marker("forgejo", support::WITHIN);
    run.kill().and_then(|()| run.wait()).expect("stopped");
    marker.environment.sort();
    let directory = scratch.path().join("forge-1");
    let mut expected = vec![
        format!("HOME={}/home", directory.display()),
        "PATH=/usr/bin:/bin".into(),
        "TZ=UTC".into(),
        "GIT_CONFIG_NOSYSTEM=1".into(),
        format!("GIT_CONFIG_GLOBAL={}/home/gitconfig", directory.display()),
        format!("PWD={}/forgejo", directory.display()),
    ];
    expected.sort();
    assert_eq!(marker.environment, expected);
    scratch.remove();
}
#[test]
fn the_new_process_holds_no_descriptor_but_the_standard_three() {
    let scratch = support::Scratch::new("forge_run_descriptors");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut run = support::owned::command(scratch.path(), "runner").spawn().expect("run");
    let marker = programs.marker("runner", support::WITHIN);
    run.kill().and_then(|()| run.wait()).expect("stopped");
    assert_eq!(marker.descriptors, [0, 1, 2]);
    assert_eq!(
        marker.arguments,
        ["--config", scratch.path().join("forge-1/runner/config.yaml").to_str().expect("UTF-8 path"), "daemon"]
    );
    scratch.remove();
}
#[test]
fn no_secret_of_the_state_reaches_a_refusal() {
    let scratch = support::Scratch::new("forge_run_secret_refusal");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let path = scratch.path().join("forge-1/state");
    let mut state = temper_forge_setup::state::decode(&fs::read(&path).expect("state")).expect("record");
    state.prepared = false;
    support::write_private(&path, temper_forge_setup::state::encode(&state).expose());
    let refused = support::owned::command(scratch.path(), "forgejo").output().expect("refusal");
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        refused.stderr,
        format!("temper forge run: forge 1: its preparation did not end: run temper forge prepare --config {0} --data {0} --cache {0} 1\n", scratch.path().display()).as_bytes()
    );
    for secret in [&state.secrets.secret_key, &state.secrets.internal_token, &state.secrets.jwt_secret] {
        assert_eq!(skein_lib::bytes::find(&refused.stderr, secret.expose()), None);
    }
    scratch.remove();
}

#[test]
fn a_newer_recorded_forgejo_refuses_the_preparation_untouched() {
    use support::owned;
    let scratch = support::Scratch::new("forge_newer_prepare");
    let _programs = owned::scripted(scratch.path());
    assert!(owned::prepare(scratch.path(), 1).status.success());
    owned::record_versions(scratch.path(), Some(b"999.0.0"), None);
    let files = ["state", "forgejo/app.ini"];
    let before: Vec<_> = files
        .iter()
        .map(|name| fs::read(scratch.path().join("forge-1").join(name)).expect("upgrade fixture"))
        .collect();
    owned::refused(scratch.path(), "prepare", b"temper forge prepare: forge 1: forgejo is prepared at 999.0.0, newer than this temper's pin 17.0.0: its migrations only go forward\n");
    let after: Vec<_> = files
        .iter()
        .map(|name| fs::read(scratch.path().join("forge-1").join(name)).expect("upgrade fixture"))
        .collect();
    assert_eq!(before, after);
    assert_eq!(fs::read_dir(scratch.path().join("forge-1/backups")).expect("upgrade fixture").count(), 0);
    scratch.remove();
}
#[test]
fn the_run_refuses_a_newer_recorded_version_in_its_own_words() {
    use support::owned;
    let scratch = support::Scratch::new("forge_newer_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::record_versions(scratch.path(), Some(b"999.0.0"), None);
    owned::refused(scratch.path(), "run", b"temper forge run: forge 1: prepared at 999.0.0 / 13.2.0, this temper pins 17.0.0 / 13.2.0: no command of this temper runs it\n");
    scratch.remove();
}
#[test]
fn the_run_refuses_an_unreadable_recorded_version_in_its_own_words() {
    use support::owned;
    let scratch = support::Scratch::new("forge_unreadable_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::record_versions(scratch.path(), Some(b"private-looking-garbage"), None);
    owned::refused(scratch.path(), "run", b"temper forge run: forge 1: prepared at a version this temper cannot read: no command of this temper runs it\n");
    scratch.remove();
}
#[test]
fn the_run_names_prepare_for_a_runner_added() {
    use support::owned;
    let scratch = support::Scratch::new("forge_runner_added_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::recorded_without_runner(scratch.path());
    owned::refused(scratch.path(), "run", format!("temper forge run: forge 1: prepared without a runner, and the setup has one: run temper forge prepare --config {0} --data {0} --cache {0} 1\n", scratch.path().display()).as_bytes());
    scratch.remove();
}
#[test]
fn the_run_names_prepare_for_a_runner_removed() {
    use support::owned;
    let scratch = support::Scratch::new("forge_runner_removed_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::setup_without_runner(scratch.path());
    owned::refused(scratch.path(), "run", format!("temper forge run: forge 1: prepared with a runner, and the setup has none: run temper forge prepare --config {0} --data {0} --cache {0} 1\n", scratch.path().display()).as_bytes());
    scratch.remove();
}
#[test]
fn an_interrupted_upgrade_leaves_the_state_for_the_next_preparation() {
    use support::owned;
    let scratch = support::Scratch::new("forge_upgrade_interrupted");
    let programs = owned::scripted(scratch.path());
    assert!(owned::prepare(scratch.path(), 1).status.success());
    owned::record_versions(scratch.path(), Some(b"16.0.5"), None);
    let path = owned::file(scratch.path(), 1, temper_forge_setup::place::File::State);
    let before = fs::read(&path).expect("old state");
    programs.hang("forgejo", "dump");
    let run = Command::new(env!("CARGO_BIN_EXE_temper"))
        .args(["forge", "prepare"])
        .args(support::configuration::flags(scratch.path()))
        .arg("1")
        .current_dir(scratch.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("prepare");
    let _sleep = programs.hung(support::WITHIN);
    assert!(
        Command::new("/bin/sh")
            .args(["-c", &format!("kill -TERM {}", run.id())])
            .status()
            .expect("upgrade fixture")
            .success()
    );
    let ended = run.wait_with_output().expect("interrupted");
    assert_eq!(ended.status.code(), Some(1));
    assert_eq!(ended.stderr, b"temper forge prepare: forge 1: interrupted\n");
    assert_eq!(fs::read(path).expect("old state kept"), before);
    assert_eq!(fs::read_dir(scratch.path().join("forge-1/backups")).expect("upgrade fixture").count(), 0);
    scratch.remove();
}
#[test]
fn reconcile_refuses_a_tls_forge_naming_it() {
    let scratch = support::Scratch::new("reconcile_tls");
    support::owned::managed(scratch.path());
    support::owned::configuration_value(scratch.path(), |value| {
        value["application"]["forges"][0]["transport"] =
            serde_json::json!({"tls":{"server_name":"forge.example","trust":null}});
    });
    support::owned::refused(
        scratch.path(),
        "reconcile",
        b"temper forge reconcile: forge 1: its transport is TLS, and reconcile speaks plaintext to loopback alone\n",
    );
    assert!(!scratch.path().join("forge-1").exists());
    scratch.remove();
}
#[test]
fn reconcile_makes_a_missing_forge_directory_private() {
    let scratch = support::Scratch::new("reconcile_missing_directory");
    support::owned::managed(scratch.path());
    support::owned::refused(
        scratch.path(),
        "reconcile",
        b"temper forge reconcile: forge 1: Forgejo did not answer within 300 ms\n",
    );
    assert_eq!(fs::metadata(scratch.path().join("forge-1")).expect("directory").permissions().mode() & 0o777, 0o700);
    scratch.remove();
}
#[test]
fn reconcile_refuses_a_forge_directory_open_to_others() {
    let scratch = support::Scratch::new("reconcile_open_directory");
    support::owned::managed(scratch.path());
    fs::create_dir(scratch.path().join("forge-1")).expect("open directory");
    fs::set_permissions(scratch.path().join("forge-1"), fs::Permissions::from_mode(0o755)).expect("open directory");
    support::owned::refused(scratch.path(), "reconcile", format!("temper forge reconcile: forge 1: {}: must be a private directory of this user's, not reached through a link\n", scratch.path().join("forge-1").display()).as_bytes());
    assert!(!scratch.path().join("forge-1/state").exists());
    scratch.remove();
}
#[test]
fn reconcile_refuses_a_forge_the_setup_has_no_entry_for() {
    let scratch = support::Scratch::new("reconcile_no_entry");
    support::owned::managed(scratch.path());
    support::owned::configuration_value(scratch.path(), |value| {
        value["application"]["setup"] = serde_json::json!([]);
    });
    support::owned::refused(
        scratch.path(),
        "reconcile",
        b"temper forge reconcile: forge 1: the setup has no entry for it\n",
    );
    assert!(!scratch.path().join("forge-1").exists());
    scratch.remove();
}
