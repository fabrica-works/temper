//! The shipped binary, on a scratch deployment, drained on a signal.
pub mod support;
use skein_io::kernel::{Exit, Signal};
use skein_lib::Duration;

#[test]
fn temper_serve_prints_its_link_and_drains_on_a_signal() {
    let scratch = support::Scratch::new("drain");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let ended = support::until_link_then(scratch.path(), Signal::Terminate, Duration::from_secs(60));
    let link = support::printed_link(&ended.stdout).expect("one printed line");
    assert!(link.starts_with(b"http://127.0.0.1:"), "a loopback link: {}", String::from_utf8_lossy(link));
    assert!(skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some(), "the deployment's sign-in link");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "drained: {}", String::from_utf8_lossy(&ended.stderr));
    scratch.remove();
}

#[test]
fn temper_open_signs_in_prints_the_inbox_and_exits_on_q() {
    let scratch = support::Scratch::new("temper_open_inbox");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let mut serve = support::serve(scratch.path());
    let link = support::terminal::link(&mut serve, support::WITHIN);
    let open = support::terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut typist = support::terminal::Typist::new(open, serve, link);
    typist.wait_for(b"== Inbox == Live", support::WITHIN);
    typist.type_line(b"q");
    assert_eq!(typist.exit(support::WITHIN), 0, "stderr: {}", typist.stderr());
    assert!(support::private(&scratch.path().join("client/token")), "the bearer token is kept 0o600");
    scratch.remove();
}

#[test]
fn temper_open_adopts_a_repository_by_commands() {
    let scratch = support::Scratch::new("temper_open_adopt");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let mut serve = support::serve(scratch.path());
    let link = support::terminal::link(&mut serve, support::WITHIN);
    let open = support::terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut typist = support::terminal::Typist::new(open, serve, link);
    typist.wait_for(b"== Inbox == Live", support::WITHIN);
    typist.wait_for(b"All goals\n", support::WITHIN);
    typist.wait_for(b"[4] Project 1\n", support::WITHIN);
    typist.type_line(b"4");
    typist.type_on(b"Adopt a repository", b"");
    for (control, words, printed) in [
        (b"Owner: (empty)".as_slice(), b"acme".as_slice(), b"Owner: acme".as_slice()),
        (b"Name: (empty)", b"widget", b"Name: widget"),
        (b"Landing branch: (empty)", b"main", b"Landing branch: main"),
        (
            b"Checks: gate numbers, separated by spaces or commas: (empty)",
            b"1",
            b"Checks: gate numbers, separated by spaces or commas: 1",
        ),
    ] {
        typist.type_on(control, words);
        typist.wait_for(printed, support::WITHIN);
    }
    typist.type_on(b"[x] It runs CI", b"");
    typist.wait_for(b"[ ] It runs CI", support::WITHIN);
    typist.type_on(b"Adopt", b"");
    typist.wait_for(b"Adopted acme/widget on forge 1 as owned.", support::WITHIN);
    typist.type_line(b"q");
    assert_eq!(typist.exit(support::WITHIN), 0, "stderr: {}", typist.stderr());
    forgejo.stop();
    scratch.remove();
}

#[test]
fn a_persons_change_lands_on_the_forge_and_is_found_after_a_restart() {
    let scratch = support::Scratch::new("landed");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let peer = support::fake_llm(support::landing_scripts(support::ANSWER, &forgejo));
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let first = support::run_until_landed(scratch.path(), support::person::adopt_and_change(&forgejo), peer, &forgejo);
    assert_eq!((first.signed_in, first.finished), (1, true), "{}", String::from_utf8_lossy(&first.stderr));
    eprintln!("stage job: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    let merged = forgejo.merged_pull().expect("the change's pull request is merged");
    assert_eq!(forgejo.job_state(merged), "success");
    assert!(forgejo.main_readme().ends_with(support::EDIT), "main holds the agent's edit");
    let second = support::run(scratch.path(), support::person::found(), None, Duration::from_secs(60));
    assert!(second.finished, "the work is found after the restart");
    assert_eq!(forgejo.merged_pull(), Some(merged), "the forge's state is unchanged by the restart");
    forgejo.stop();
    scratch.remove();
}

#[test]
fn temper_sign_in_prints_the_visit_and_keeps_the_record() {
    let scratch = support::Scratch::new("temper_sign_in_record");
    support::sign_in::configuration(scratch.path());
    let issuer = support::sign_in::issuer();
    let mut child = support::sign_in::start(scratch.path(), support::sign_in::ACCOUNT);
    let visit = child.line(support::WITHIN);
    assert!(visit.starts_with(support::sign_in::AUTHORIZATION), "the visit is the issuer's: {visit:?}");
    let browser = support::sign_in::browse(&issuer, &visit);
    assert_eq!(child.exit(issuer, browser, None, support::WITHIN), 0, "stderr: {}", child.stderr());
    assert!(support::private(&scratch.path().join("accounts/records/1")), "the record is kept 0o600");
    scratch.remove();
}
#[test]
fn a_running_engine_lends_the_record_temper_sign_in_kept() {
    let scratch = support::Scratch::new("temper_sign_in_running");
    support::sign_in::configuration(scratch.path());
    let engine = support::sign_in::running(scratch.path());
    let issuer = support::sign_in::issuer();
    let mut child = support::sign_in::start(scratch.path(), support::sign_in::ACCOUNT);
    let visit = child.line(support::WITHIN);
    let browser = support::sign_in::browse(&issuer, &visit);
    assert_eq!(child.exit(issuer, browser, Some(engine), support::WITHIN), 0, "stderr: {}", child.stderr());
    scratch.remove();
}

#[test]
fn forge_prepare_makes_an_owned_forge_then_remakes_nothing() {
    let scratch = support::Scratch::new("forge_prepare_real");
    let _port = support::owned::prepared_processes(scratch.path());
    let before: Vec<_> = support::owned::files(scratch.path())
        .iter()
        .map(|path| {
            assert!(support::private(path), "{} is private", path.display());
            fs_read(path)
        })
        .collect();
    let secret = fs_read(&scratch.path().join("forge-1/runner/secret"));
    assert_eq!(secret.len(), 40);
    assert!(secret.iter().all(u8::is_ascii_hexdigit));
    let second = support::owned::prepare(scratch.path(), 1);
    assert_eq!(second.status.code(), Some(0), "{}", String::from_utf8_lossy(&second.stderr));
    let after: Vec<_> = support::owned::files(scratch.path()).iter().map(|path| fs_read(path)).collect();
    assert_eq!(after, before);
    let ini = scratch.path().join("forge-1/forgejo/app.ini");
    let settings = fs_read(&ini);
    let admin_token = scratch.path().join("forge-1/admin_token");
    std::fs::remove_file(&admin_token).expect("removed token");
    let third = support::owned::prepare(scratch.path(), 1);
    assert_eq!(third.status.code(), Some(0), "{}", String::from_utf8_lossy(&third.stderr));
    let token = fs_read(&admin_token);
    let line = token.split(|byte| *byte == b'\n').next().expect("token line");
    assert_eq!(line.len(), 40);
    assert!(line.iter().all(u8::is_ascii_hexdigit));
    assert_eq!(fs_read(&ini), settings);
    scratch.remove();
}
fn fs_read(path: &std::path::Path) -> Vec<u8> {
    std::fs::read(path).expect("prepared file")
}

#[test]
fn a_prepared_forge_runs_forgejo_and_its_runner_once_each() {
    use support::owned;
    let scratch = support::Scratch::new("forge_run_real");
    let port = owned::prepared_processes(scratch.path());
    let forgejo = owned::Running::start(scratch.path(), "forgejo");
    let version = owned::await_answer(port, "/api/v1/version", None);
    assert_eq!(
        version["version"].as_str().expect("version").split('+').next(),
        Some(temper_throwaway_forgejo::VERSION)
    );
    let token = std::fs::read_to_string(scratch.path().join("forge-1/admin_token")).expect("admin token");
    let admin = owned::await_answer(port, "/api/v1/user", Some(token.trim()));
    assert_eq!(admin["login"], "temper-admin");
    assert_eq!(admin["is_admin"], true);
    let mut runner = owned::Running::start(scratch.path(), "runner");
    runner.declared();
    let refused = owned::command(scratch.path(), "forgejo").output().expect("second run");
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge run: forge 1: forgejo is running or being prepared already\n");
    runner.stop();
    forgejo.stop();
    assert!(owned::answer(port, "/api/v1/version", None).is_err());
    scratch.remove();
}

#[test]
fn an_older_recorded_forgejo_is_dumped_then_upgraded() {
    use std::fs;
    use support::owned;
    use temper_forge_setup::{place::File, state};
    let scratch = support::Scratch::new("forge_upgrade_real");
    let _port = owned::prepared_processes(scratch.path());
    owned::record_versions(scratch.path(), Some(b"16.0.5"), None);
    let partial = owned::file(scratch.path(), 1, File::DumpPartial);
    fs::write(&partial, b"interrupted dump").expect("partial");
    let upgraded = owned::prepare(scratch.path(), 1);
    assert!(upgraded.status.success(), "{}", String::from_utf8_lossy(&upgraded.stderr));
    let (backup, before, meta) = owned::dump(scratch.path());
    assert!(!partial.exists());
    let record = state::decode(&fs::read(owned::file(scratch.path(), 1, File::State)).expect("upgrade fixture"))
        .expect("upgrade fixture");
    assert_eq!(&*record.forgejo, temper_throwaway_forgejo::VERSION.as_bytes());
    owned::record_versions(scratch.path(), Some(b"16.0.5"), None);
    let repeated = owned::prepare(scratch.path(), 1);
    assert!(repeated.status.success(), "{}", String::from_utf8_lossy(&repeated.stderr));
    assert_eq!(fs::read(&backup).expect("backup kept"), before);
    assert_eq!(
        fs::metadata(&backup).expect("upgrade fixture").modified().expect("upgrade fixture"),
        meta.modified().expect("upgrade fixture")
    );
    scratch.remove();
}
#[test]
fn an_older_recorded_runner_has_its_files_prepared_again_without_a_dump() {
    use std::fs;
    use support::owned;
    use temper_forge_setup::place::File;
    let scratch = support::Scratch::new("runner_upgrade_real");
    let _port = owned::prepared_processes(scratch.path());
    owned::record_versions(scratch.path(), None, Some(b"13.1.0"));
    let configuration = owned::file(scratch.path(), 1, File::RunnerConfiguration);
    let before = fs::read(&configuration).expect("connection");
    fs::remove_file(&configuration).expect("remove configuration");
    let upgraded = owned::prepare(scratch.path(), 1);
    assert!(upgraded.status.success(), "{}", String::from_utf8_lossy(&upgraded.stderr));
    assert_eq!(fs::read(configuration).expect("configuration remade"), before);
    assert_eq!(fs::read_dir(scratch.path().join("forge-1/backups")).expect("upgrade fixture").count(), 0);
    scratch.remove();
}

#[test]
fn reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing() {
    use std::{
        fs,
        os::unix::fs::{MetadataExt, PermissionsExt},
    };
    use support::owned;
    let scratch = support::Scratch::new("owned_reconcile");
    let port = owned::reconcile_configuration(scratch.path());
    let running = owned::start(scratch.path());
    let first = owned::reconcile(scratch.path(), 1);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(first.stdout, b"");
    let instance = owned::attach(scratch.path(), port, running);
    owned::assert_converged(scratch.path(), &instance);
    let files: Vec<_> =
        ["api_token", "git_token", "user"].iter().map(|name| scratch.path().join("forge-1").join(name)).collect();
    let before: Vec<_> = files.iter().map(|path| fs::metadata(path).expect("written file")).collect();
    for meta in &before {
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }
    let second = owned::reconcile(scratch.path(), 1);
    assert_eq!(second.status.code(), Some(0), "{}", String::from_utf8_lossy(&second.stderr));
    for (path, before) in files.iter().zip(before) {
        let after = fs::metadata(path).expect("kept file");
        assert_eq!(
            (after.ino(), after.mtime(), after.mtime_nsec()),
            (before.ino(), before.mtime(), before.mtime_nsec())
        );
    }
    instance.stop().expect("forge stops");
    scratch.remove();
}

#[test]
fn temper_box_prints_the_link_and_a_stop_leaves_no_process() {
    let scratch = support::Scratch::new("box_stop");
    let forgejo = support::owned::tier_configuration(scratch.path());
    let ended = support::boxed::run(scratch.path(), forgejo, support::boxed::Plan::Stop, Duration::from_secs(120));
    assert_box_streams(&ended);
    assert_eq!(ended.forgejo_at_link, Some(200), "Forgejo answered its version when the link came");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    assert_eq!(support::boxed::status(forgejo, "/api/v1/version"), None, "Forgejo is gone");
    scratch.remove();
}
fn assert_box_streams(ended: &support::boxed::Outcome) {
    let link = support::printed_link(&ended.stdout).expect("the box's stdout carries the link");
    assert!(skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some(), "the sign-in link");
    let alone = |line: &[u8]| line.is_empty() || line.starts_with(b"http://127.0.0.1:");
    assert!(ended.stdout.split(|byte| *byte == b'\n').all(alone), "stdout holds links alone");
    let steps: [&[u8]; 6] = [b"prepare 1: ", b"forgejo 1: ", b"runner 1: ", b"reconcile 1: ", b"serve: ", b"box: "];
    let lines = || ended.stderr.split(|byte| *byte == b'\n').filter(|line| !line.is_empty());
    assert!(lines().all(|line| steps.iter().any(|step| line.starts_with(step))), "each stderr line names its step");
    assert!(lines().any(|line| line.starts_with(b"runner 1: ")), "the runner's lines arrive prefixed");
}

#[test]
fn temper_box_restarts_a_killed_forgejo_while_the_engine_serves() {
    let scratch = support::Scratch::new("box_kill");
    let forgejo = support::owned::tier_configuration(scratch.path());
    let kill = support::boxed::Plan::KillForgejo;
    let ended = support::boxed::run(scratch.path(), forgejo, kill, Duration::from_secs(120));
    let (killed, restarted) = ended.forgejo_pids.expect("Forgejo was killed and came back");
    assert_ne!(killed, restarted, "another Forgejo process answers");
    assert_eq!(ended.engine_served, Some(true), "the same engine process answered while Forgejo was back");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    scratch.remove();
}
