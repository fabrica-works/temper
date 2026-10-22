//! The shipped binary, on a scratch deployment, drained on a signal.
pub mod support;
use skein_io::kernel::{Exit, Signal};
use skein_lib::Duration;

// Each whole-example box story is bounded at 120 seconds.
const BOX_DEADLINE: Duration = Duration::from_secs(120);

fn ctl_scratch(case: &str) -> support::Scratch {
    let scratch = support::Scratch::new(case);
    support::configuration(scratch.path(), &support::Forge::Unreached);
    scratch
}

#[test]
fn temper_ctl_help_prints_the_guide() {
    let scratch = ctl_scratch("ctl_help");
    let ended = support::ctl::alone(scratch.path(), support::ctl::Call::bare(&["help"]));
    assert_eq!(ended.exit, Some(Exit::Code(0)));
    assert!(ended.stdout.starts_with(b"Agent tools\n"));
    assert_eq!(ended.stderr, b"");
    scratch.remove();
}

#[test]
fn temper_ctl_without_a_tool_exits_1() {
    let scratch = ctl_scratch("ctl_usage");
    let ended = support::ctl::alone(scratch.path(), support::ctl::Call::new(&[]));
    assert_eq!(ended.exit, Some(Exit::Code(1)));
    assert!(ended.stderr.starts_with(b"temper ctl: "));
    let line = ended.stderr.strip_suffix(b"\n").expect("terminated diagnostic");
    assert!(!line.contains(&b'\n'), "one diagnostic line");
    scratch.remove();
}

#[test]
fn temper_ctl_signs_in_by_the_link_read_on_stdin() {
    use support::ctl::{Call, Step, Story};
    let scratch = ctl_scratch("ctl_sign_in");
    let steps = vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))];
    let seen = support::ctl::run(Story::new(scratch.path(), steps));
    let [signed, me] = seen.calls.as_slice() else { panic!("two ctl ends") };
    assert_eq!(signed.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&signed.stderr));
    let document = support::ctl::json(me);
    assert_eq!((me.exit, document["party"]["number"].is_u64()), (Some(Exit::Code(0)), true), "{document}");
    scratch.remove();
}

#[test]
fn a_wait_past_its_timeout_exits_3() {
    use support::ctl::{Call, Step, Story};
    let scratch = ctl_scratch("ctl_timeout");
    let seen = support::ctl::run(Story::new(
        scratch.path(),
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["me", "--json"])),
            Step::Run(Call::new(&["wait", "me", "--timeout", "1", "--json"])),
        ],
    ));
    let [_, me, wait] = seen.calls.as_slice() else { panic!("three ctl ends") };
    assert_eq!(wait.exit, Some(Exit::Code(3)), "{}", String::from_utf8_lossy(&wait.stderr));
    assert_eq!(support::ctl::json(wait)["me"]["digest"], support::ctl::json(me)["digest"]);
    scratch.remove();
}

#[test]
fn an_adopt_cut_by_a_signal_is_made_once() {
    use support::ctl::{Call, PROJECT, Step, Story, json};
    let scratch = support::Scratch::new("ctl_cut");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let adopt = &[
        "adopt",
        "--project",
        PROJECT,
        "--owner",
        "acme",
        "--name",
        "widget",
        "--landing",
        "main",
        "--key",
        "widget",
        "--json",
    ];
    let seen = support::ctl::run(Story::on(
        scratch.path(),
        &forgejo,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["me", "--json"])),
            Step::Pause,
            Step::Start(Call::new(adopt)),
            Step::Until(Call::new(&["pending"]), b"adopt"),
            Step::Cut,
            Step::Run(Call::new(&["pending", "--json"])),
            Step::Resume,
            Step::Run(Call::new(adopt)),
            Step::Run(Call::new(&["pending"])),
        ],
    ));
    let [_, me, cut, _, listed, again, empty] = seen.calls.as_slice() else { panic!("seven ctl ends") };
    assert_eq!((cut.exit, cut.stdout.as_slice()), (Some(Exit::Code(143)), b"".as_slice()), "128 + SIGTERM");
    let (party, listed) = (json(me)["party"]["number"].as_u64().expect("signed in"), json(listed));
    let first_sent = listed["pending"][0]["first_sent"].clone();
    assert!(first_sent.is_u64() || first_sent.is_null(), "{listed}");
    let row = serde_json::json!({"operation": "adopt", "party": party, "first_sent": first_sent});
    assert_eq!(listed, serde_json::json!({"pending": [row], "unread": 0}));
    assert_eq!(again.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&again.stderr));
    let adopted = json(again);
    let kept: serde_json::Value = serde_json::from_slice(&again.stderr).expect("one kept line, JSON");
    assert_eq!(kept, serde_json::json!({"operation": "adopt", "outcome": adopted}), "the first answer");
    assert_eq!((empty.exit, empty.stdout.as_slice()), (Some(Exit::Code(0)), b"".as_slice()));
    forgejo.stop();
    scratch.remove();
}

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
    typist.wait_for(b"All goals (pressed)\n", support::WITHIN);
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
fn a_push_is_delivered_signed_with_the_secret_reconcile_drew() {
    let scratch = support::Scratch::new("hook_delivery");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    let listener = std::net::TcpListener::bind(forgejo.webhooks()).expect("nothing holds the address");
    forgejo.push("hook.txt");
    let delivery = loop {
        let delivery = support::webhooks::receive(&listener, support::WITHIN);
        if skein_lib::bytes::find(&delivery.body, b"Add hook.txt").is_some() {
            break delivery;
        }
    };
    assert_eq!(delivery.event, "push", "Forgejo names the event");
    assert!(skein_lib::bytes::find(&delivery.body, b"refs/heads/main").is_some(), "a push to main");
    assert_eq!(delivery.signature, support::webhooks::signature(&forgejo.secret(), &delivery.body));
    forgejo.stop();
    scratch.remove();
}

#[test]
fn a_persons_change_lands_on_the_forge_and_is_found_after_a_restart() {
    landing("landed", false);
}

#[test]
fn a_change_lands_by_deliveries_with_every_poll_raised() {
    landing("hinted", true);
}

/// The landing story; the hinted case raises four limits and pushes the slow job marker.
fn landing(case: &str, raised: bool) {
    let scratch = support::Scratch::new(case);
    let forgejo = if raised {
        support::forgejo::Throwaway::start_with_job_sleep(&scratch, 8)
    } else {
        support::forgejo::Throwaway::start(&scratch)
    };
    if raised {
        forgejo.push("slow");
    }
    support::configuration(scratch.path(), &forgejo.forge());
    if raised {
        support::raise_paces(scratch.path());
    }
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
        Some(skein_forge::forgejo::pin::VERSION)
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
    assert_eq!(&*record.forgejo, skein_forge::forgejo::pin::VERSION.as_bytes());
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
fn temper_box_prints_its_link_and_stops_its_processes() {
    let scratch = support::Scratch::new("box_stop");
    let owned = boxed_example(&scratch);
    let forgejo = owned.forgejo;
    let ended = support::boxed::run(scratch.path(), Some(forgejo), support::boxed::Plan::Stop, BOX_DEADLINE);
    assert_box_streams(&ended);
    assert_eq!(ended.forgejo_at_link, Some(200), "Forgejo answered its version when the link came");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    assert_eq!(support::boxed::status(forgejo, "/api/v1/version"), None, "Forgejo is gone");
    scratch.remove();
}
fn boxed_example(scratch: &support::Scratch) -> support::owned::Owned {
    let owned = support::owned::Owned::tier();
    support::owned::lay(scratch.path(), &owned);
    std::fs::write(scratch.path().join("temper.json"), support::owned::example_document(scratch.path(), &owned))
        .expect("box example");
    owned
}
fn assert_box_streams(ended: &support::boxed::Outcome) {
    let link = support::printed_link(&ended.stdout).unwrap_or_else(|| {
        panic!("the box's stdout carries the link: {}", String::from_utf8_lossy(&ended.stderr));
    });
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
    let owned = boxed_example(&scratch);
    let forgejo = owned.forgejo;
    let kill = support::boxed::Plan::KillForgejo;
    let ended = support::boxed::run(scratch.path(), Some(forgejo), kill, BOX_DEADLINE);
    let (killed, restarted) = ended.forgejo_pids.unwrap_or_else(|| {
        panic!("Forgejo was killed and came back: {}", String::from_utf8_lossy(&ended.stderr));
    });
    assert_ne!(killed, restarted, "another Forgejo process answers");
    assert_eq!(ended.engine_served, Some(true), "the same engine process answered while Forgejo was back");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "stopped as asked: {}", String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.left, Some(Vec::new()), "no process the box started outlived it");
    scratch.remove();
}

#[test]
fn temper_serve_answers_a_signed_delivery_204() {
    use support::webhooks::{self, KNOWN_FORGE, PING, Post};
    let scratch = support::Scratch::new("signed_hook");
    let ended = webhooks::delivered(&scratch, Post::signed(KNOWN_FORGE, b"ping", PING));
    assert_eq!(ended.statuses, [204]);
    assert_eq!(ended.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&ended.stderr));
    scratch.remove();
}
#[test]
fn temper_serve_answers_an_unsigned_delivery_403() {
    use support::webhooks::{self, KNOWN_FORGE, PING, Post};
    let scratch = support::Scratch::new("unsigned_hook");
    let ended = webhooks::delivered(&scratch, Post::unsigned(KNOWN_FORGE, b"ping", PING));
    assert_eq!(ended.statuses, [403]);
    assert_eq!(ended.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&ended.stderr));
    scratch.remove();
}
#[test]
fn temper_serve_answers_an_unknown_forge_404() {
    use support::webhooks::{self, PING, Post, UNKNOWN_FORGE};
    let scratch = support::Scratch::new("unknown_hook");
    let ended = webhooks::delivered(&scratch, Post::signed(UNKNOWN_FORGE, b"ping", PING));
    assert_eq!(ended.statuses, [404]);
    assert_eq!(ended.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&ended.stderr));
    scratch.remove();
}

#[test]
fn a_native_chat_reaches_the_model() {
    let scratch = support::Scratch::new("native-chat");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let peer = support::fake_llm(vec![support::chat_script(support::ANSWER)].into_boxed_slice());
    let seen = support::run(
        scratch.path(),
        support::person::answered_chat(b"Hello", support::ANSWER),
        Some(peer),
        support::WITHIN,
    );
    assert!(seen.finished);
    forgejo.stop();
    scratch.remove();
}

#[test]
fn program_login_execs_the_configured_fake_claude() {
    program_login_story("success", smith_fake_claude::Login::Immediate("claude.ai".into()), 0);
}

#[test]
fn program_login_propagates_the_fake_claudes_refusal() {
    program_login_story("refused", smith_fake_claude::Login::Refused, 1);
    program_login_story("exec-refused", smith_fake_claude::Login::default(), 1);
}

fn program_login_story(row: &str, login: smith_fake_claude::Login, exit: u8) {
    use smith_fake_claude::{LOGIN_FILE, OBSERVATIONS_FILE, STORY_FILE, Story, observations};
    let scratch = support::Scratch::new(&format!("program-login-{row}"));
    let directory = scratch.path();
    let executable = std::path::PathBuf::from(
        std::env::var_os("TEMPER_TEST_CLAUDE")
            .expect("build smith's fake-claude through heavy and set its absolute TEMPER_TEST_CLAUDE path"),
    );
    assert!(executable.is_absolute() && executable.is_file(), "configured fake binary: {}", executable.display());
    let programs = directory.join("programs");
    let configured = if row == "exec-refused" { directory.join("absent-claude") } else { executable };
    support::program_login::configuration(directory, Some(&programs), &configured);
    let source = programs.join("source");
    if row != "exec-refused" {
        smith_agent_shell::program::prepare(&programs, "source").expect("story directory");
        std::fs::write(source.join(STORY_FILE), Story { login, ..Story::default() }.encode()).expect("fake story");
    }
    let ended = support::program_login::run(directory);
    assert_eq!(ended.exit, Some(Exit::Code(exit)), "{}", String::from_utf8_lossy(&ended.stderr));
    let stderr = String::from_utf8_lossy(&ended.stderr);
    if row == "exec-refused" {
        let lines = stderr.lines().collect::<Vec<_>>();
        let (last, before) = lines.split_last().expect("temper's refusal line");
        assert!(last.starts_with("temper program-login: exec: "), "{stderr}");
        assert!(before.iter().all(|line| line.starts_with("smith: ")), "{stderr}");
        assert!(!source.join(OBSERVATIONS_FILE).exists());
        assert!(!source.join(LOGIN_FILE).exists());
        support::program_login::no_deployment_resources(directory);
        scratch.remove();
        return;
    }
    assert!(stderr.lines().all(|line| !line.starts_with("temper program-login:")), "{stderr}");
    let records = observations(&std::fs::read(source.join(OBSERVATIONS_FILE)).expect("observations"));
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].arguments, ["auth", "login"]);
    assert_eq!(records[0].environment, ["CLAUDE_CONFIG_DIR", "HOME", "PATH", "TEMPER_BENIGN"]);
    if exit == 0 {
        assert_eq!(std::fs::read(source.join(LOGIN_FILE)).expect("login written"), b"claude.ai");
        assert_eq!(ended.stdout, b"Signed in.\n");
    } else {
        assert!(!source.join(LOGIN_FILE).exists());
        assert!(ended.stderr.windows(b"sign-in refused".len()).any(|bytes| bytes == b"sign-in refused"));
    }
    support::program_login::no_deployment_resources(directory);
    scratch.remove();
}

#[test]
fn a_first_run_answers_on_the_demo() {
    first_run(false);
}

#[test]
#[ignore = "jig seeds no deployment owner in init's configuration; ctl omits the kept deployment account"]
fn a_first_run_answers_on_the_demo_and_a_second_finds_it_seeded() {
    first_run(true);
}
fn first_run(restart: bool) {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("first-run");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    for path in &paths {
        fs::create_dir(path).expect("empty first-run directory");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private first-run directory");
    }
    let flags: Vec<String> = ["--config", "--data", "--cache"]
        .into_iter()
        .zip(&paths)
        .flat_map(|(flag, path)| [flag.into(), path.display().to_string()])
        .collect();
    for (run, person, key) in [
        (
            "first",
            support::person::demo_change(),
            temper_fake::seed::KEY_ADDED_LINE,
        ),
        (
            "second",
            // After the first run's chat, task 1, and its change, task 2.
            support::person::demo_chat(
                3,
                b"Add a dark theme to widget",
                b"I would give widget a dark theme that follows the system's preference, with its colours as variables in one place so both themes stay in step. I am the demo model and play this from a script, so nothing is changed: connect a model from Accounts to have the work done.",
                b"The demo changes nothing.",
            ),
            temper_fake::seed::KEY_FOUND_LINE,
        ),
    ].into_iter().take(if restart { 2 } else { 1 }) {
        let client = scratch.path().join(run);
        fs::create_dir(&client).expect("fresh client root");
        fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client root");
        let clock = skein_shell::Clock::new();
        let began = clock.now().now;
        let ended = support::boxed::run_story(
            scratch.path(),
            flags.clone(),
            &client,
            person,
            BOX_DEADLINE,
            || demo_main(&paths[0], &paths[1], &client, run == "first"),
        );
        eprintln!("{run} run: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
        let stderr = String::from_utf8_lossy(&ended.stderr);
        assert_eq!(ended.exit, Some(Exit::Code(0)), "{run}: {stderr}");
        assert!(ended.finished, "{run}: {stderr}");
        assert_eq!(ended.left, Some(Vec::new()), "{run}: no process outlived the box: {stderr}");
        let wrote = format!("box: wrote the configuration, {}\n", paths[0].join("temper.json").display());
        assert_eq!(stderr.contains(&wrote), run == "first", "{run}: {stderr}");
        assert_demo_streams(&ended.stdout, &ended.stderr, key);
        if run == "second" {
            assert_eq!(ended.checked, fake_forge_commit_id(DEMO_SEEDED_HEAD_COUNT), "a restarted fake holds the seed");
        }
    }
    scratch.remove();
}
fn assert_demo_streams(stdout: &[u8], stderr: &[u8], key: &[u8]) {
    let link = support::printed_link(stdout).expect("the person's sign-in link");
    assert!(skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some());
    assert_eq!(stdout, [link, b"\n"].concat(), "exactly one stdout line");
    let stderr = std::str::from_utf8(stderr).expect("demo diagnostics");
    let adopt = std::str::from_utf8(temper_fake::seed::ADOPT_LINE).expect("act 4");
    let lines = [
        "temper fake seed: act 1: signed in\n".to_string(),
        std::str::from_utf8(key).expect("act 2").into(),
        "temper fake seed: act 3: accounts are usable\n".into(),
        adopt.into(),
        "temper fake seed: act 5: signed out\n".into(),
        "temper fake seed: act 6: the spent link was refused\n".into(),
    ];
    let mut remaining = stderr;
    for line in lines {
        let expected = format!("seed: {line}");
        let (_, after) =
            remaining.split_once(&expected).unwrap_or_else(|| panic!("missing ordered {expected}: {stderr}"));
        remaining = after;
        assert_eq!(stderr.matches(&expected).count(), 1, "one act: {expected}");
    }
    let steps = ["fake llm: ", "prepare 1: ", "fake forge 1: ", "reconcile 1: ", "serve: ", "seed: ", "box: "];
    let bundle = std::path::Path::new(env!("CARGO_BIN_EXE_temper"))
        .parent()
        .expect("binary parent")
        .join("../share/temper/bundle");
    let no_bundle = format!("init: no bundle at {}: the browser is not served", bundle.display());
    let no_git = "init: no git on PATH: a real forge's repositories need git 2.41 or later, set as workers[].git";
    for line in stderr.lines() {
        assert!(
            steps.iter().any(|prefix| line.starts_with(prefix)) || line == no_git || line == no_bundle,
            "unprefixed demo diagnostic: {line}"
        );
    }
}
// forge_seed gives repository 1 the counted head 1 at every fake start.
const DEMO_SEEDED_HEAD_COUNT: u64 = 1;

/// The demo's main, cloned as the worker clones it, after landing or a fresh fake.
fn demo_main(config: &std::path::Path, data: &std::path::Path, root: &std::path::Path, landed: bool) -> String {
    use std::{fs, os::unix::ffi::OsStrExt, process::Command};
    use temper_worker_protocol_git::invocation::{self, Command as GitCommand};
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(config.join("temper.json")).expect("written configuration")).expect("JSON");
    let forge = &document["application"]["forges"][0];
    let address: std::net::SocketAddr =
        forge["api"]["socket"].as_str().expect("chosen forge address").parse().expect("loopback");
    let token = fs::read_to_string(data.join("demo/forge/api_token")).expect("reconcile's API token");
    let user =
        support::owned::answer(address.port(), "/api/v1/user", Some(token.trim())).expect("reconciled bot answers");
    assert_eq!(user["login"], "temper", "reconcile made and authenticated forge 1's bot before serve");
    let branch = support::owned::answer(address.port(), "/api/v1/repos/demo/widget/branches/main", Some(token.trim()))
        .expect("main's API head");
    let head = branch["commit"]["id"].as_str().expect("landing head").to_string();
    let seeded = fake_forge_commit_id(DEMO_SEEDED_HEAD_COUNT);
    if landed {
        assert_ne!(head, seeded, "the change moved main");
        let compared = support::owned::answer(
            address.port(),
            &format!("/api/v1/repos/demo/widget/compare/{seeded}...{head}"),
            Some(token.trim()),
        )
        .expect("seed to main comparison");
        assert_eq!(compared["total_commits"], 1, "one commit past the seed: {compared}");
        assert_eq!(compared["commits"], serde_json::json!([{"sha": head}]), "only the landed commit");
    }
    let identity = temper_worker_protocol_git::Identity {
        token: skein_lib::Secret::new(
            fs::read(data.join("demo/forge/git_token")).expect("reconcile's git token").into_boxed_slice(),
        ),
        login: b"temper".as_slice().into(),
        name: b"Temper".as_slice().into(),
        email: b"temper@localhost".as_slice().into(),
    };
    let remote = format!("{}/demo/widget.git", forge["git"].as_str().expect("demo git URL"));
    let clone = GitCommand::Clone { remote: remote.as_bytes().into(), directory: b"widget".as_slice().into() };
    for (operation, kind, at) in [
        (clone, temper_worker_domain_checkout::git::Kind::Clone, root.to_path_buf()),
        (
            GitCommand::CheckOut { commit: invocation::object(head.as_bytes()).expect("API object id") },
            temper_worker_domain_checkout::git::Kind::CheckOut,
            root.join("widget"),
        ),
    ] {
        let mut command = Command::new(data.join("demo/bin/git"));
        command
            .env_clear()
            .current_dir(at)
            .args(invocation::arguments(&operation).iter().map(|word| std::ffi::OsStr::from_bytes(word)));
        for entry in invocation::environment(root.as_os_str().as_bytes(), Some(kind), Some(&identity)) {
            let split = entry.iter().position(|b| *b == b'=').expect("environment entry");
            command.env(std::ffi::OsStr::from_bytes(&entry[..split]), std::ffi::OsStr::from_bytes(&entry[split + 1..]));
        }
        let cloned = command.output().expect("demo's git link");
        assert!(cloned.status.success(), "git: {}", String::from_utf8_lossy(&cloned.stderr));
    }
    let checkout = root.join("widget");
    for (path, bytes) in temper_fake::forge::FILES {
        let expected = if landed && path == b"src/main.rs" {
            let script = document["application"]["demo"]["llm"]["cued"]
                .as_array()
                .expect("demo scripts")
                .iter()
                .find(|script| script["cue"] == "Export widget's table as CSV")
                .expect("coding script");
            let edit = &script["turns"][1]["lines"][0]["arguments"]["edits"][0];
            let old = edit["old"].as_str().expect("old main");
            let new = edit["new"].as_str().expect("new main");
            let seed = std::str::from_utf8(bytes).expect("seeded source");
            assert_eq!(seed.matches(old).count(), 1, "one coding edit in the seed");
            seed.replacen(old, new, 1).into_bytes()
        } else {
            bytes.to_vec()
        };
        assert_eq!(fs::read(checkout.join(std::ffi::OsStr::from_bytes(path))).expect("cloned demo file"), expected);
    }
    assert_eq!(fs::read(checkout.join(".git/temper-remote")).expect("cloned remote"), remote.as_bytes());
    let count = u64::from_le_bytes(
        fs::read(checkout.join(".git/temper-head")).expect("cloned head").try_into().expect("counted id"),
    );
    assert_eq!(fake_forge_commit_id(count), head, "clone holds the API's landing head");
    let admin_token = fs::read_to_string(data.join("demo/forge/admin_token")).expect("preparation's handover token");
    let hooks = support::owned::answer(address.port(), "/api/v1/repos/demo/widget/hooks", Some(admin_token.trim()))
        .expect("the handover's admin can read repository hooks");
    let hooks = hooks.as_array().expect("repository hooks");
    assert_eq!(hooks.len(), 1, "one hook for demo/widget");
    let listener: std::net::SocketAddr = document["application"]["webhooks"]["listener"]["socket"]
        .as_str()
        .expect("chosen webhooks address")
        .parse()
        .expect("loopback");
    assert_eq!(hooks[0]["config"]["url"], format!("http://127.0.0.1:{}/hooks/1", listener.port()));
    head
}

fn fake_forge_commit_id(count: u64) -> String {
    String::from_utf8(skein_forge::forgejo::id::write(&skein_fake_forge_protocol::routes::object(count)).into_vec())
        .expect("fake commit hex")
}

// protocol/README.md, 15.6: the description's first process stories.
struct ClusterJudge {
    members: support::cluster::Members,
    deadline: skein_lib::Time,
    workers: bool,
}
impl skein_world::Referee<support::cluster::Root> for ClusterJudge {
    fn act(&mut self, now: skein_lib::Time, hosts: &mut [support::cluster::Root]) {
        if self.members.is_up() {
            if self.workers {
                for worker in ["w1", "w2"] {
                    assert_eq!(
                        self.members.capture(worker, "engine"),
                        self.members.capture("engine", "workers"),
                        "{worker} linked to this engine"
                    );
                }
            }
            self.members.stop();
        }
        self.members.act(now, hosts);
    }
    fn observe(&mut self, now: skein_lib::Time, hosts: &[support::cluster::Root]) {
        self.members.observe(now, hosts);
        if let Some(stopped) = self.members.stopped() {
            if self.workers {
                assert_eq!(stopped.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["w2", "w1", "engine", "llm"]);
            } else {
                let [engine, llm] = stopped else { panic!("engine and llm stopped") };
                assert_eq!(engine.name, "engine");
                assert_eq!(llm.name, "llm");
                engine_lines(engine);
            }
            for member in stopped {
                clean_cluster_exit(member);
            }
        }
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: skein_lib::Time) -> Option<String> {
        self.members.overdue(now).or_else(|| {
            (now >= self.deadline && !self.members.passed()).then(|| "cluster did not settle within WITHIN".into())
        })
    }
    fn passed(&self) -> bool {
        self.members.passed()
    }
}
fn cluster_story(scratch: &std::path::Path, workers: bool) {
    let cluster = support::cluster::Cluster::read(scratch);
    let (mut members, hosts) = cluster.members();
    members.up(if workers { &["w1", "w2"] } else { &["engine"] });
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(ClusterJudge {
        members,
        deadline: clock.now().now.saturating_add(support::WITHIN),
        workers,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    let outcome = world.run(&clock, support::WITHIN);
    assert!(outcome.procs.iter().all(skein_shell::Host::is_empty), "every cluster root settled");
}
fn clean_cluster_exit(stopped: &support::cluster::Stopped) {
    assert_eq!(stopped.exit, Some(Exit::Code(0)), "{}: {}", stopped.name, String::from_utf8_lossy(&stopped.stderr));
    assert!(!stopped.killed, "{} stopped before STOP_SECONDS", stopped.name);
    assert!(stopped.survivors.is_empty(), "{} left no survivor", stopped.name);
}
#[test]
fn an_engine_alone_prints_its_lines_and_drains_on_a_signal() {
    let scratch = support::Scratch::new("cluster_engine_alone");
    cluster_story(scratch.path(), false);
    scratch.remove();
}
fn engine_lines(engine: &support::cluster::Stopped) {
    let stderr = std::str::from_utf8(&engine.stderr).expect("engine lines");
    let lines: Vec<_> = stderr.lines().collect();
    let [_, _] = lines.as_slice() else { panic!("exactly the two listening lines: {stderr}") };
    let clients: std::net::SocketAddr = lines
        .iter()
        .find_map(|line| line.strip_prefix("listening clients "))
        .expect("clients line")
        .parse()
        .expect("clients address");
    let workers: std::net::SocketAddr = lines
        .iter()
        .find_map(|line| line.strip_prefix("listening workers "))
        .expect("workers line")
        .parse()
        .expect("workers address");
    for address in [clients, workers] {
        assert_eq!(address.ip(), std::net::Ipv4Addr::LOCALHOST);
        assert_ne!(address.port(), 0, "a bound port");
    }
    assert_ne!(clients.port(), workers.port(), "distinct listeners");
    let stdout = std::str::from_utf8(&engine.stdout).expect("link text");
    let link = stdout.strip_suffix('\n').expect("terminated sign-in link");
    assert!(!link.contains('\n'), "exactly one stdout line");
    let secret = link
        .strip_prefix(&format!("http://{clients}/jig/v1/sign-in/link?secret="))
        .expect("sign-in link at the clients listener");
    assert!(!secret.is_empty(), "a sign-in secret");
}

#[test]
fn a_mixed_document_exits_1_naming_its_path() {
    use skein_world::end_to_end::{Binary, Command, Expectation, Mode};
    struct Refusal {
        deadline: skein_lib::Time,
        settled: bool,
    }
    impl skein_world::Referee<Binary> for Refusal {
        fn act(&mut self, _now: skein_lib::Time, _hosts: &mut [Binary]) {}
        fn observe(&mut self, _now: skein_lib::Time, hosts: &[Binary]) {
            self.settled = hosts.iter().all(skein_shell::Host::is_empty);
        }
        fn next_deadline(&self) -> Option<skein_lib::Time> {
            Some(self.deadline)
        }
        fn overdue(&self, now: skein_lib::Time) -> Option<String> {
            (now >= self.deadline && !self.settled).then(|| "mixed document did not exit".into())
        }
        fn passed(&self) -> bool {
            self.settled
        }
    }
    let scratch = support::Scratch::new("cluster_mixed_document");
    let cluster = support::cluster::Cluster::documents(
        scratch.path(),
        &[("llm", "address", "127.0.0.1:1"), ("engine", "workers", "127.0.0.1:2")],
    );
    let path = cluster.dir("engine").join("config/temper.json");
    let mut engine: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("engine document")).expect("engine JSON");
    let worker: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cluster.dir("w1").join("config/temper.json")).expect("worker document"))
            .expect("worker JSON");
    engine["workers"] = worker["workers"].clone();
    std::fs::write(&path, serde_json::to_vec(&engine).expect("mixed JSON")).expect("mixed document");
    let directory = cluster.dir("engine");
    let binary = Binary::start_with_tree(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: vec![
                "serve".into(),
                "--config".into(),
                directory.join("config").into(),
                "--data".into(),
                directory.join("data").into(),
                "--cache".into(),
                directory.join("cache").into(),
            ],
            environment: Vec::new(),
            directory,
        },
        Mode::Pipes,
        65_536,
        Expectation::Supervise,
        false,
    )
    .expect("mixed serve starts");
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(Refusal {
        deadline: clock.now().now.saturating_add(support::WITHIN),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || binary);
    let outcome = world.run(&clock, support::WITHIN);
    let [binary] = outcome.procs.as_slice() else { panic!("one serve") };
    assert_eq!(binary.exit_status(), Some(Exit::Code(1)));
    assert_eq!(binary.stderr(), b"temper serve: workers: an engine alone takes no local worker, one plaintext loopback listener and workers with distinct digest paths\n");
    scratch.remove();
}

#[test]
fn the_cluster_stops_in_reverse_each_exiting_0() {
    let scratch = support::Scratch::new("cluster_reverse_stop");
    cluster_story(scratch.path(), true);
    scratch.remove();
}

// Both the idle threshold and ordinary time allowance hold an open chat's slot.
// Past the story's deadline, a run that parks has already failed it.
const CHAT_HOLDS_ITS_SLOT: Duration = support::WITHIN.saturating_mul(2);

// A lost link heals: a stall past the 1 s silence loses it. The worker's contact
// grace, graces.grace_ms 1000 (temper-service worker_settings.rs:145; jig 8f1acb2
// worker-link component.rs:629), outlasts two redials (125–250 ms, doubling:
// backoff.rs:16; profile/small.rs:239); a re-hello clears it (component.rs:455).
// The engine keeps that channel's attempts adrift for fleet.grace_ms 3000
// (jig-domain-fleet channel.rs:250); re-hello lists them back (attempt.rs:49),
// so nothing is placed twice.
#[test]
fn a_task_runs_on_each_worker() {
    cluster_chats("cluster_each_worker", ClusterStory::Each);
}

// A waiting reply is uncommitted: killing w1 loses it. The retry on w2
// resumes without an assistant message, so the same cue replies and waits
// again. Only after close/end is that reply committed, exactly once; the
// old attempt ends lost and the replacement finishes done. This covers loss
// and retry; the hung-worker story will test fencing a surviving old run.
#[test]
fn a_worker_killed_mid_run_is_lost_and_its_run_answered_once_on_another() {
    cluster_chats("cluster_killed", ClusterStory::Kill);
}

// Jig admits only with an available placement (e7a93cd, jig-domain-core
// routing/run_preparation.rs:466–492, run_choice.rs:73–77): the second task
// is due, unclaimed and unplaced; no attempt exists to be placed twice.
#[test]
fn a_worker_started_later_takes_the_waiting_task() {
    cluster_chats("cluster_started_later", ClusterStory::Later);
}

#[test]
fn a_worker_restarted_on_its_directories_links_at_its_position() {
    cluster_chats("cluster_restarted", ClusterStory::Restart);
}

#[test]
fn an_unexpected_member_exit_is_reported_after_its_client_settles() {
    let failure = std::panic::catch_unwind(|| cluster_chats("cluster_unexpected", ClusterStory::Unexpected))
        .expect_err("unexpected member fails the story");
    let message = failure.downcast_ref::<String>().expect("member diagnostic");
    assert!(message.contains("w1 exited unexpectedly: Some(Signal(9))"), "{message}");
    assert!(message.contains("linked w1 "), "stderr tail: {message}");
    assert!(!message.contains("EBUSY") && !message.contains("ResourceBusy"), "{message}");
}

fn cluster_chats(case: &str, story: ClusterStory) {
    let scratch = support::Scratch::new(case);
    let cluster = support::cluster::Cluster::read(scratch.path());
    let engine = cluster.dir("engine").join("config/temper.json");
    let key = cluster.dir("llm").join("llm.key");
    let (mut members, hosts) = cluster.members();
    members.up(if story == ClusterStory::Each { &["w1", "w2"] } else { &["w1"] });
    let clock = skein_shell::Clock::new();
    let mut world = skein_world::real::World::new(ClusterChatJudge {
        members,
        deadline: clock.now().now.saturating_add(support::WITHIN),
        engine,
        key,
        client: scratch.path().join("client"),
        stage: ClusterChatStage::Start,
        tasks: Vec::new(),
        workers: std::collections::BTreeSet::new(),
        story,
        reading: None,
        checking: std::collections::VecDeque::new(),
        last_read: String::new(),
        lost_attempt: None,
    });
    for host in hosts {
        world.spawn(|| host);
    }
    world.spawn(support::cluster::Root::client);
    let outcome = world.run(&clock, support::WITHIN);
    assert!(outcome.procs.iter().all(skein_shell::Host::is_empty), "every cluster root settled");
    eprintln!("cluster chats settled in {:?}", outcome.end.saturating_since(outcome.start));
    scratch.remove();
}

// lab/cluster has two chats, one per slot, and two lives in the restart story.
const CLUSTER_CHATS: usize = 2;
const FIRST_LIFE: usize = 0;
const SECOND_LIFE: usize = 1;
const RESTART_LINKS: usize = 2;
const REPLY_ONCE: usize = 1;
const FIRST_LINK: usize = 1;
const LOST_AND_RETRY: usize = 2; // the killed run and its replacement
const KILL_SIGNAL: u32 = 9; // Linux SIGKILL, as reported by wait
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClusterStory {
    Each,
    Unexpected,
    Kill,
    Later,
    Restart,
}
#[derive(Debug)]
enum ClusterChatStage {
    Start,
    Created,
    Replied,
    WaitingSlot,
    Crashed,
    Armed,
    Killed,
    StoppedOne,
    Relinking,
    Closed,
    Verifying,
    Stopping,
}
struct ClusterChatJudge {
    members: support::cluster::Members,
    deadline: skein_lib::Time,
    engine: std::path::PathBuf,
    key: std::path::PathBuf,
    client: std::path::PathBuf,
    stage: ClusterChatStage,
    tasks: Vec<String>,
    workers: std::collections::BTreeSet<String>,
    story: ClusterStory,
    reading: Option<ClusterRead>,
    checking: std::collections::VecDeque<String>,
    last_read: String,
    lost_attempt: Option<u64>,
}
impl ClusterChatJudge {
    fn queue(&self, host: &mut support::cluster::Root, calls: Vec<support::ctl::Call>) {
        let link = format!("http://{}", self.members.capture("engine", "link").expect("engine link"));
        let mut client = support::ctl::Calls::linked(&link, &self.client);
        for call in calls {
            client.push(support::ctl::Step::Run(call));
        }
        let support::cluster::Root::Client { calls, .. } = host else { panic!("client root") };
        *calls = Some(Box::new(client));
    }
    fn begin(&mut self, client: &mut support::cluster::Root) {
        use support::ctl::{Call, PROJECT};
        let engine: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&self.engine).expect("rendered engine document"))
                .expect("engine JSON");
        let charter = engine["engine"]["charters"]
            .as_array()
            .expect("charters")
            .iter()
            .find(|charter| charter["number"] == engine["application"]["charter"])
            .expect("chat charter");
        assert!(
            Duration::from_millis(charter["waiting_ms"].as_u64().expect("chat waiting_ms")) >= CHAT_HOLDS_ITS_SLOT,
            "lab/cluster/engine.json: chat charter waiting_ms must hold its slot past the story deadline"
        );
        let time = charter["time_ms"].as_u64().expect("chat time_ms");
        let reserve = charter["reserve_time_ms"].as_u64().expect("chat reserve_time_ms");
        assert!(
            time.checked_sub(reserve).is_some_and(|ordinary| Duration::from_millis(ordinary) >= CHAT_HOLDS_ITS_SLOT),
            "lab/cluster/engine.json: chat charter time_ms less its reserve must hold its slot past the story deadline"
        );
        self.queue(
            client,
            vec![
                Call::sign_in(),
                Call::new(&[
                    "add-key",
                    "1",
                    self.key.to_str().expect("LLM key path"),
                    "--deployment",
                    "--key",
                    "cluster-llm",
                ]),
                Call::new(&["wait", "accounts", "--until", "usable"]),
                Call::new(&["chat", "--project", PROJECT, "--key", "first", "cluster-first-chat", "--json"]),
            ],
        );
        self.stage = ClusterChatStage::Created;
    }
    fn create_second(&mut self, client: &mut support::cluster::Root) {
        use support::ctl::{Call, PROJECT};
        self.queue(
            client,
            vec![Call::new(&["chat", "--project", PROJECT, "--key", "second", "cluster-second-chat", "--json"])],
        );
        self.stage = ClusterChatStage::Created;
    }
    fn current(&self) -> &str {
        self.checking.front().or_else(|| self.tasks.last()).expect("current chat")
    }
    fn poll(&self, client: &mut support::cluster::Root) {
        use support::ctl::Call;
        let task = self.current();
        let calls = if matches!(self.stage, ClusterChatStage::Verifying) {
            vec![Call::new(&["task", task, "--activity", &temper_client_tools::limits::ROWS.to_string(), "--json"])]
        } else if matches!(self.stage, ClusterChatStage::Replied) {
            // The live reply watch returns a task; read.waiting() still checks its exact work and placement.
            vec![Call::new(&["wait", "task", task, "--until", "replied", "--json"]), Call::new(&["accounts", "--json"])]
        } else {
            vec![Call::new(&["task", task, "--json"]), Call::new(&["accounts", "--json"])]
        };
        self.queue(client, calls);
    }
    fn read(&mut self, client: &mut support::cluster::Root, ends: &[support::Ended]) -> Option<ClusterRead> {
        use support::ctl::{Call, json};
        if self.reading.as_ref().is_some_and(|read| read.turns) {
            let mut read = self.reading.take().expect("turn reads");
            let turns = read.rows.iter_mut().filter(|row| row["event"]["owner"]["source"]["line"]["turn"].is_object());
            for (row, ended) in turns.zip(ends) {
                let turn = json(ended);
                complete_cluster_json(&turn);
                assert!(turn["turn"]["decoded"].is_object(), "committed turn decoded: {turn}");
                row["event"]["owner"]["decoded"] = turn;
            }
            return Some(read);
        }
        let (mut read, page) = match ends {
            [task, accounts] => {
                let task = json(task);
                let task = if matches!(self.stage, ClusterChatStage::Replied) { task["task"].clone() } else { task };
                return Some(ClusterRead::new(task, json(accounts)));
            }
            [activity] if self.reading.is_some() => (self.reading.take().expect("continued activity"), json(activity)),
            [packet] => {
                let packet = json(packet);
                (ClusterRead::new(packet["task"].clone(), serde_json::Value::Null), packet["activity"].clone())
            }
            _ => panic!("task/accounts, task with activity, or continued activity"),
        };
        complete_cluster_json(&page);
        read.rows.extend(page["rows"].as_array().expect("activity rows").iter().cloned());
        let task = self.current();
        if let Some(before) = page["before"]["value"].as_str() {
            self.queue(client, vec![Call::new(&["activity", task, "--before", before, "--json"])]);
        } else {
            let calls: Vec<_> = read
                .rows
                .iter()
                .filter_map(|row| {
                    let turn = row.pointer("/event/owner/source/line/turn")?;
                    let attempt = turn["attempt"].as_u64().expect("turn attempt");
                    let number = turn["turn"].as_u64().expect("turn number");
                    Some(Call::new(&["turn", task, &format!("{attempt}.{number}"), "--json"]))
                })
                .collect();
            if calls.is_empty() {
                return Some(read);
            }
            self.queue(client, calls);
            read.turns = true;
        }
        self.reading = Some(read);
        None
    }
    fn advance_read(&mut self, client: &mut support::cluster::Root, read: &ClusterRead) {
        self.last_read = read.owner().to_string();
        match self.stage {
            ClusterChatStage::WaitingSlot => {
                if read.owner()["phase"] == serde_json::json!({"active": "due"})
                    && read.owner()["details"]["attempt"].is_null()
                    && read.owner()["details"]["observation"].is_null()
                {
                    self.members.up(&["w2"]);
                    self.stage = ClusterChatStage::Replied;
                }
                self.poll(client);
            }
            ClusterChatStage::Relinking => {
                if self.members.linked("w1") == RESTART_LINKS && read.connected("w1") {
                    self.create_second(client);
                } else {
                    self.poll(client);
                }
            }
            ClusterChatStage::Replied => self.replied(client, read),
            ClusterChatStage::Armed => {
                if self.members.is_up() && self.members.linked("w2") >= FIRST_LINK && read.connected("w2") {
                    self.members.kill("w1");
                    self.stage = ClusterChatStage::Killed;
                }
                self.poll(client);
            }
            ClusterChatStage::Killed => {
                if self.members.ended("w1")
                    && read.worker() == Some("w2")
                    && read.waiting()
                    && Some(read.attempt()) != self.lost_attempt
                {
                    self.close(client, false);
                } else {
                    self.poll(client);
                }
            }
            ClusterChatStage::Verifying => {
                assert!(read.owner()["details"]["ending"]["done"].is_object(), "chat ended done: {}", read.task);
                let text = if self.tasks.first().is_some_and(|task| task == self.current()) {
                    "First worker replied."
                } else {
                    "Second worker replied."
                };
                assert_eq!(read.replies(text), REPLY_ONCE, "reply once across complete committed turns");
                if self.story == ClusterStory::Kill {
                    self.loss(read);
                }
                self.checking.pop_front().expect("verified chat");
                if self.checking.is_empty() {
                    if self.story == ClusterStory::Restart && self.tasks.len() < CLUSTER_CHATS {
                        self.members.stop_one("w1");
                        self.stage = ClusterChatStage::StoppedOne;
                    } else {
                        self.members.stop();
                        self.stage = ClusterChatStage::Stopping;
                    }
                } else {
                    self.poll(client);
                }
            }
            ClusterChatStage::Start
            | ClusterChatStage::Created
            | ClusterChatStage::StoppedOne
            | ClusterChatStage::Crashed
            | ClusterChatStage::Closed
            | ClusterChatStage::Stopping => unreachable!("read stage"),
        }
    }
    fn replied(&mut self, client: &mut support::cluster::Root, read: &ClusterRead) {
        let first = self.tasks.len() < CLUSTER_CHATS;
        if let Some(worker) = read.worker().filter(|_| read.waiting()) {
            eprintln!(
                "client placement: task {} attempt {} worker {worker}",
                self.tasks.last().expect("chat"),
                read.attempt()
            );
            if self.story == ClusterStory::Each {
                assert!(self.workers.insert(worker.into()), "each chat on its own worker");
            } else {
                assert_eq!(worker, if first || self.story == ClusterStory::Restart { "w1" } else { "w2" });
            }
            if self.story == ClusterStory::Unexpected {
                self.queue(
                    client,
                    vec![
                        support::ctl::Call::new(&["wait", "task", self.current(), "--until", "ended"]),
                        support::ctl::Call::new(&["unknown"]),
                    ],
                );
                self.members.signal("w1", Signal::Kill);
                self.stage = ClusterChatStage::Crashed;
            } else if self.story == ClusterStory::Kill {
                self.lost_attempt = Some(read.attempt());
                self.members.up(&["w2"]);
                self.stage = ClusterChatStage::Armed;
                self.poll(client);
            } else if first {
                if self.story == ClusterStory::Restart {
                    self.close(client, true);
                } else {
                    self.create_second(client);
                }
            } else {
                if self.story == ClusterStory::Each {
                    assert_eq!(self.workers.iter().map(String::as_str).collect::<Vec<_>>(), ["w1", "w2"]);
                }
                self.close(client, false);
            }
        } else {
            self.poll(client);
        }
    }
    fn loss(&self, read: &ClusterRead) {
        let old = self.lost_attempt.expect("killed attempt");
        let replacement = read.attempt();
        assert_ne!(replacement, old, "replacement attempt on w2");
        let lines: Vec<_> = read.rows.iter().map(|row| &row["event"]["owner"]["source"]["line"]).collect();
        assert_eq!(lines.iter().filter(|line| line["run_began"].is_object()).count(), LOST_AND_RETRY);
        for (attempt, end) in
            [(old, serde_json::json!({"failed": {"class": "lost"}})), (replacement, serde_json::json!("finished"))]
        {
            assert!(lines.iter().any(|line| line["run_began"]["attempt"] == attempt), "attempt began");
            let ended = lines.iter().find(|line| line["run_ended"]["attempt"] == attempt).expect("attempt ended");
            assert_eq!(ended["run_ended"]["end"], end);
        }
        assert!(!lines.iter().any(|line| line["turn"]["attempt"] == old), "killed reply was never committed");
    }
    fn close(&mut self, client: &mut support::cluster::Root, first_only: bool) {
        use support::ctl::Call;
        let mut close = Vec::new();
        for (task, key) in self.tasks.iter().zip(["close-first", "close-second"]) {
            if self.story == ClusterStory::Restart && !first_only && self.tasks.first() == Some(task) {
                continue;
            }
            self.checking.push_back(task.clone());
            close.push(Call::new(&["close", task, "--key", key]));
            close.push(Call::new(&["wait", "task", task, "--until", "ended"]));
        }
        self.queue(client, close);
        self.stage = ClusterChatStage::Closed;
    }
}
struct ClusterRead {
    task: serde_json::Value,
    accounts: serde_json::Value,
    rows: Vec<serde_json::Value>,
    turns: bool,
}
impl ClusterRead {
    fn new(task: serde_json::Value, accounts: serde_json::Value) -> Self {
        Self { task, accounts, rows: Vec::new(), turns: false }
    }
    fn owner(&self) -> &serde_json::Value {
        &self.task["chip"]["details"]["owner"]
    }
    fn waiting(&self) -> bool {
        self.owner()["details"]["observation"]["work"] == "waiting"
    }
    fn attempt(&self) -> u64 {
        self.owner()["details"]["attempt"].as_u64().expect("chat attempt")
    }
    fn hosts(&self) -> &[serde_json::Value] {
        self.accounts["accounts"]["deployment_facts"]["program_hosts"].as_array().expect("deployment hosts")
    }
    fn connected(&self, name: &str) -> bool {
        self.hosts().iter().any(|host| host["name"] == name && host["connected"] == true)
    }
    fn worker(&self) -> Option<&str> {
        let channel = self.owner()["details"]["observation"]["channel"].as_u64()?;
        self.hosts().iter().find(|host| host["channel"] == channel && host["connected"] == true)?["name"].as_str()
    }
    fn replies(&self, text: &str) -> usize {
        self.rows
            .iter()
            .filter_map(|row| row["event"]["owner"]["decoded"]["turn"]["decoded"]["messages"].as_array())
            .flatten()
            .filter(|message| message["speaker"] == "assistant")
            .flat_map(|message| message["blocks"].as_array().expect("message blocks"))
            .filter_map(|block| block["text"]["text"].as_str())
            .map(|words| words.matches(text).count())
            .sum()
    }
}
fn complete_cluster_json(value: &serde_json::Value) {
    if let Some(cut) = value.get("cut") {
        assert_eq!(cut, false, "complete committed transcript");
    }
    if let Some(left) = value.get("bytes_left") {
        assert_eq!(left, 0, "complete turn text");
    }
    for item in
        value.as_object().into_iter().flat_map(serde_json::Map::values).chain(value.as_array().into_iter().flatten())
    {
        complete_cluster_json(item);
    }
}

impl skein_world::Referee<support::cluster::Root> for ClusterChatJudge {
    fn act(&mut self, now: skein_lib::Time, hosts: &mut [support::cluster::Root]) {
        use skein_shell::Host;
        use support::ctl::json;
        self.members.act(now, hosts);
        if self.members.failed() {
            self.stage = ClusterChatStage::Stopping;
            return;
        }
        let (client, _) = hosts.split_last_mut().expect("cluster client root");
        if matches!(self.stage, ClusterChatStage::StoppedOne) && self.members.ended("w1") {
            self.members.restart("w1");
            self.stage = ClusterChatStage::Relinking;
            self.poll(client);
        }
        if self.members.is_up() || self.members.ended("w1") {
            if matches!(self.stage, ClusterChatStage::Start) {
                self.begin(client);
            } else if client.is_empty() && !matches!(self.stage, ClusterChatStage::Start | ClusterChatStage::Stopping) {
                let support::cluster::Root::Client { calls, .. } = client else { panic!("client root") };
                let calls = calls.take().expect("completed cluster calls");
                for ended in calls.ends() {
                    assert_eq!(ended.exit, Some(Exit::Code(0)), "ctl: {}", String::from_utf8_lossy(&ended.stderr));
                }
                match self.stage {
                    ClusterChatStage::Created => {
                        let created = json(calls.ends().last().expect("created chat"));
                        let task = created["outcome"]["done"]["owner"]["created"]["first"]
                            .as_u64()
                            .unwrap_or_else(|| panic!("created chat number: {created}"))
                            .to_string();
                        self.tasks.push(task);
                        self.stage = if self.story == ClusterStory::Later && self.tasks.len() == CLUSTER_CHATS {
                            ClusterChatStage::WaitingSlot
                        } else {
                            ClusterChatStage::Replied
                        };
                        self.poll(client);
                    }
                    ClusterChatStage::Replied
                    | ClusterChatStage::WaitingSlot
                    | ClusterChatStage::Relinking
                    | ClusterChatStage::Verifying
                    | ClusterChatStage::Armed
                    | ClusterChatStage::Killed => {
                        if let Some(read) = self.read(client, calls.ends()) {
                            self.advance_read(client, &read);
                        }
                    }
                    ClusterChatStage::Closed => {
                        self.stage = ClusterChatStage::Verifying;
                        self.poll(client);
                    }
                    ClusterChatStage::Start
                    | ClusterChatStage::Stopping
                    | ClusterChatStage::StoppedOne
                    | ClusterChatStage::Crashed => {
                        unreachable!("active calls")
                    }
                }
            }
        }
    }
    fn observe(&mut self, now: skein_lib::Time, hosts: &[support::cluster::Root]) {
        use skein_shell::Host;
        let (client, hosts) = hosts.split_last().expect("cluster client root");
        self.members.observe(now, hosts);
        if self.members.failed() {
            if self.story != ClusterStory::Unexpected {
                return;
            }
            let support::cluster::Root::Client { calls: Some(calls), .. } = client else {
                panic!("failed client's calls")
            };
            if self.story == ClusterStory::Unexpected && matches!(self.stage, ClusterChatStage::Crashed) {
                assert!(!calls.is_empty(), "in-flight child not yet reaped");
            }
            if self.story == ClusterStory::Unexpected && self.members.stopped().is_some() {
                assert!(calls.is_empty(), "client settled before remaining members stop");
                let [cancelled] = calls.ends() else { panic!("one in-flight call; queued unknown never started") };
                assert_eq!(cancelled.exit, Some(Exit::Signal(KILL_SIGNAL)), "client killed, waited and reaped");
            }
            return;
        }
        if let Some(stopped) = self.members.stopped() {
            let expected = if self.story == ClusterStory::Restart {
                vec![("w1", FIRST_LIFE), ("w1", SECOND_LIFE), ("engine", FIRST_LIFE), ("llm", FIRST_LIFE)]
            } else if self.story == ClusterStory::Kill {
                vec![("w1", FIRST_LIFE), ("w2", FIRST_LIFE), ("engine", FIRST_LIFE), ("llm", FIRST_LIFE)]
            } else {
                vec![("w2", FIRST_LIFE), ("w1", FIRST_LIFE), ("engine", FIRST_LIFE), ("llm", FIRST_LIFE)]
            };
            assert_eq!(stopped.iter().map(|s| (s.name.as_str(), s.life)).collect::<Vec<_>>(), expected);
            for member in stopped {
                if self.story == ClusterStory::Kill && member.name == "w1" {
                    assert_eq!(member.ending, support::cluster::Ending::Kill);
                    assert_eq!(member.exit, Some(Exit::Signal(KILL_SIGNAL)));
                    assert!(!member.killed && member.survivors.is_empty(), "intentional kill needed no cleanup");
                    continue;
                }
                let alone = self.story == ClusterStory::Restart && member.name == "w1" && member.life == FIRST_LIFE;
                assert_eq!(
                    member.ending,
                    if alone { support::cluster::Ending::Alone } else { support::cluster::Ending::Cluster }
                );
                clean_cluster_exit(member);
            }
        }
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        Some(self.members.next_deadline().map_or(self.deadline, |at| at.min(self.deadline)))
    }
    fn overdue(&self, now: skein_lib::Time) -> Option<String> {
        (now >= self.deadline && !self.members.passed()).then(|| {
            format!(
                "cluster {:?} waited in {:?}: {}; last read: {}",
                self.story,
                self.stage,
                match self.stage {
                    ClusterChatStage::Start => "requested members ready",
                    ClusterChatStage::Created => "chat creation",
                    ClusterChatStage::Replied => "waiting run and connected placement",
                    ClusterChatStage::WaitingSlot => "due second chat without attempt or observation",
                    ClusterChatStage::Crashed => "unexpected exit with a call in flight",
                    ClusterChatStage::Armed => "w2 linked and connected before the kill",
                    ClusterChatStage::Killed => "w1 tree settled and a new attempt waiting on w2",
                    ClusterChatStage::Closed => "closed chats ended",
                    ClusterChatStage::Verifying => "ended done and complete committed turns",
                    ClusterChatStage::StoppedOne => "w1's first life settled",
                    ClusterChatStage::Relinking => "w1's second linked line and connected host",
                    ClusterChatStage::Stopping => "every process tree settled",
                },
                self.last_read
            )
        })
    }
    fn passed(&self) -> bool {
        matches!(self.stage, ClusterChatStage::Stopping) && self.members.passed()
    }
}

#[test]
#[ignore = "until accounts_runtime resolves sign-in endpoints at connect (o/accounts-jp13b's M3)"]
fn a_default_box_with_a_sign_in_registration_starts_without_dns() {
    use std::{ffi::OsString, fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("first-run-no-dns");
    let hosts = scratch.path().join("hosts");
    let nsswitch = scratch.path().join("nsswitch.conf");
    fs::write(&hosts, b"").expect("empty names");
    fs::write(&nsswitch, b"hosts: files\npasswd: files\ngroup: files\n").expect("no DNS resolver");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    for path in &paths {
        fs::create_dir(path).expect("empty deployment directories");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private deployment directories");
    }
    let mut init = std::process::Command::new(env!("CARGO_BIN_EXE_temper"));
    init.arg("init");
    for (flag, path) in ["--config", "--data", "--cache"].into_iter().zip(&paths) {
        init.arg(flag).arg(path);
    }
    let initialized = init.output().expect("initialize the default document without resolving names");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
    // Smith also resolves the agent's endpoints at preflight. Supply local endpoint
    // facts there so this regression reaches accounts_runtime; forge defaults stay intact.
    let agent_path = paths[0].join("agent.json");
    let mut agent: serde_json::Value =
        serde_json::from_slice(&fs::read(&agent_path).expect("agent fixture")).expect("agent JSON");
    for endpoint in agent["endpoints"].as_array_mut().expect("endpoints") {
        if endpoint["address"].as_str().expect("endpoint address").parse::<std::net::SocketAddr>().is_err() {
            endpoint["address"] = "127.0.0.1:9".into();
        }
    }
    fs::write(agent_path, serde_json::to_vec(&agent).expect("local agent fixture")).expect("local preflight endpoints");
    // The shipped providers offer keys only. Add a registration solely to reach
    // the separate sign-in startup resolver while retaining all default forges.
    let document_path = paths[0].join("temper.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&document_path).expect("default document")).expect("JSON");
    document["engine"]["llm"]["providers"][0]["sources"]["sign_in"] = serde_json::json!({
        "issuer":"https://issuer.example", "client":"client", "scopes":[],
        "authorization":"https://issuer.example/auth", "token":"https://issuer.example/token",
        "keys":"https://issuer.example/keys", "transport":"tls", "server_name":"issuer.example",
        "must_grant":[], "redirect_path":"callback", "redirect_port":3000
    });
    document["application"]["limits"]["overrides"]["git_component.operations"] = 1.into();
    document["application"]["limits"]["overrides"]["checkout.workspaces"] = 1.into();
    fs::write(document_path, serde_json::to_vec(&document).expect("sign-in fixture")).expect("registered sign-in");
    // The private network has only loopback. Files-only name lookup and a private /run
    // exclude DNS and host resolver daemons; the demo peers alone use its sockets.
    let script = r#"set -eu
/usr/bin/mount --make-rprivate /
/usr/bin/mount --bind "$1" /etc/hosts
/usr/bin/mount --bind "$2" /etc/nsswitch.conf
/usr/bin/mount -t tmpfs tmpfs /run
/usr/bin/ip link set lo up
shift 2
exec "$@"
"#;
    let mut arguments: Vec<OsString> = ["-Urnm", "/bin/sh", "-c", script, "no-dns"].map(OsString::from).into();
    arguments.extend([
        hosts.into_os_string(),
        nsswitch.into_os_string(),
        env!("CARGO_BIN_EXE_temper").into(),
        "box".into(),
    ]);
    for (flag, path) in ["--config", "--data", "--cache"].into_iter().zip(&paths) {
        arguments.extend([flag.into(), path.as_os_str().into()]);
    }
    let ended = support::boxed::run_command(
        skein_world::end_to_end::Command {
            program: "/usr/bin/unshare".into(),
            arguments,
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        None,
        support::boxed::Plan::Stop,
        Duration::from_secs(3),
    );
    let diagnostic = String::from_utf8_lossy(&ended.stderr).into_owned();
    let linked = support::printed_link(&ended.stdout).is_some();
    scratch.remove();
    assert!(!diagnostic.contains("application.forges"), "forge preparation refused: {diagnostic}");
    assert!(!diagnostic.contains("workers[0].agent"), "agent preflight fixture refused: {diagnostic}");
    assert!(
        linked || diagnostic.contains("failed to lookup address information"),
        "unexpected startup refusal: {diagnostic}"
    );
    assert!(
        linked,
        "no-DNS start must reach its printed link; the current sign-in endpoint resolution refuses it: {diagnostic}"
    );
    assert_eq!(ended.exit, Some(Exit::Code(0)), "{diagnostic}");
    assert_eq!(ended.left, Some(Vec::new()), "no process outlived the box: {diagnostic}");
}
