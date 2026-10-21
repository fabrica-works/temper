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
