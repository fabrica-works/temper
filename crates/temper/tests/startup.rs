pub mod support;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

#[test]
fn a_configuration_file_past_its_bound_is_refused() {
    let scratch = support::Scratch::startup("configuration-bound");
    let directory = scratch.path();
    fs::File::create(directory.join("temper.json"))
        .expect("configuration file")
        .set_len(1_048_577)
        .expect("one byte past bound");
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(directory)).err().expect("refused"),
        format!("temper serve: configuration {}: past 1048576 bytes\n", directory.join("temper.json").display())
    );
    scratch.remove();
}
#[test]
fn a_secret_file_readable_by_others_refuses_the_start_naming_it() {
    let scratch = support::Scratch::startup("public-token");
    let directory = scratch.path();
    support::configuration(directory, &support::Forge::Unreached);
    fs::set_permissions(directory.join("secrets/forge-1.git-token"), fs::Permissions::from_mode(0o644))
        .expect("public permissions");
    let refused = temper::prepare_in(&support::configuration::directories(directory)).err().expect("start refused");
    assert_eq!(
        refused,
        format!(
            "temper serve: application.identities[0].token: {}: readable by others or reached through a link\n",
            scratch.path().join("secrets/forge-1.git-token").display()
        )
    );
    assert!(!refused.contains("forge-1-git-token-value"), "no secret in a diagnostic");
    scratch.remove();
}
#[test]
fn a_worker_root_reached_through_a_link_is_refused() {
    let scratch = support::Scratch::startup("linked-root");
    let directory = scratch.path();
    support::configuration(directory, &support::Forge::Unreached);
    fs::create_dir(directory.join("real-root")).expect("root directory");
    symlink(directory.join("real-root"), directory.join("worker-root")).expect("linked root");
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(directory)).err().expect("refused"),
        "temper serve: application.workers[0].root: must be a private directory of this user's, not reached through a link\n"
    );
    scratch.remove();
}
#[test]
fn problems_reach_stderr_one_line_each() {
    let scratch = support::Scratch::startup("problem-lines");
    let directory = scratch.path();
    fs::write(directory.join("temper.json"), b"{}").expect("configuration");
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(directory)).err().expect("refused"),
        "temper serve: engine: missing\ntemper serve: workers: missing\ntemper serve: application: missing\n"
    );
    scratch.remove();
}

#[test]
fn an_address_with_no_token_in_the_directory_is_not_signed_in() {
    for (row, token, expected) in [
        ("absent", None, "temper open: not signed in: give the link temper serve printed\n"),
        ("empty", Some(b"".as_slice()), "temper open: not signed in: give the link temper serve printed\n"),
    ] {
        let scratch = support::Scratch::startup(&format!("open-{row}"));
        if let Some(token) = token {
            fs::write(scratch.path().join("token"), token).expect("token file");
            fs::set_permissions(scratch.path().join("token"), fs::Permissions::from_mode(0o600))
                .expect("private token");
        }
        assert_eq!(
            temper::open::check_token(scratch.path()).expect_err("not signed in").line(scratch.path()),
            expected,
            "{row}"
        );
        scratch.remove();
    }
}

#[test]
fn a_user_file_readable_by_others_refuses_the_start_naming_it() {
    let scratch = support::Scratch::startup("public-user");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    fs::set_permissions(scratch.path().join("secrets/forge-1.user"), fs::Permissions::from_mode(0o644))
        .expect("fixture operation");
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(scratch.path())).err().expect("configuration refused"),
        format!(
            "temper serve: application.forges[0].user.file: {}: readable by others or reached through a link\n",
            scratch.path().join("secrets/forge-1.user").display()
        )
    );
    scratch.remove();
}
fn managed(directory: &std::path::Path, move_files: bool) -> serde_json::Value {
    support::configuration(directory, &support::Forge::Unreached);
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("temper.json")).expect("configuration bytes"))
            .expect("fixture operation");
    value["application"]["setup"] = serde_json::json!([{
        "forge":1,"directory":"forge-1","bot":{"login":"temper","email":"temper@example.test"},
        "orgs":["acme"],"mode":{"managed":{"admin_token":"secrets/forgejo-admin"}}
    }]);
    if move_files {
        fs::create_dir(directory.join("forge-1")).expect("fixture operation");
        for (from, to) in [
            ("secrets/forge-1.api-token", "api_token"),
            ("secrets/forge-1.git-token", "git_token"),
            ("secrets/forge-1.user", "user"),
        ] {
            fs::rename(directory.join(from), directory.join("forge-1").join(to)).expect("fixture operation");
        }
        value["application"]["forges"][0]["api_token"] = "forge-1/api_token".into();
        value["application"]["forges"][0]["user"] = serde_json::json!({"file":"forge-1/user"});
        value["application"]["identities"][0]["token"] = "forge-1/git_token".into();
    }
    support::write_private(&directory.join("temper.json"), &serde_json::to_vec(&value).expect("configuration JSON"));
    value
}
#[test]
fn a_setup_naming_another_api_token_file_refuses_temper_serve_naming_its_path() {
    let scratch = support::Scratch::startup("setup-token");
    let mut value = managed(scratch.path(), true);
    value["application"]["forges"][0]["api_token"] = "another/api_token".into();
    support::write_private(
        &scratch.path().join("temper.json"),
        &serde_json::to_vec(&value).expect("configuration JSON"),
    );
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(scratch.path())).err().expect("configuration refused"),
        "temper serve: application.forges[0].api_token: invalid form\n"
    );
    scratch.remove();
}
#[test]
fn a_setup_limit_past_its_check_refuses_temper_serve_naming_both_limits() {
    let scratch = support::Scratch::startup("setup-limits");
    let mut value = managed(scratch.path(), true);
    value["application"]["limits"]["overrides"]["setup.backoff_ms"] = 6000.into();
    support::write_private(
        &scratch.path().join("temper.json"),
        &serde_json::to_vec(&value).expect("configuration JSON"),
    );
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(scratch.path())).err().expect("configuration refused"),
        "temper serve: application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_managed_entrys_absent_admin_token_file_does_not_stop_temper_serve() {
    let scratch = support::Scratch::startup("setup-no-admin");
    managed(scratch.path(), true);
    assert!(!scratch.path().join("secrets/forgejo-admin").exists());
    assert!(temper::prepare_in(&support::configuration::directories(scratch.path())).is_ok());
    scratch.remove();
}
fn box_document(scratch: &support::Scratch) -> serde_json::Value {
    support::configuration(scratch.path(), &support::Forge::Unreached);
    serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("configuration")).expect("JSON")
}
fn refused_box(scratch: &support::Scratch, value: &serde_json::Value) -> String {
    support::write_private(&scratch.path().join("temper.json"), &serde_json::to_vec(value).expect("JSON"));
    temper::run(temper::Command::Box { directories: support::configuration::directories(scratch.path()) })
        .expect_err("box refused")
}
#[test]
fn a_box_configuration_that_is_absent_is_refused_naming_it() {
    assert_eq!(
        temper::run(temper::Command::Box {
            directories: support::configuration::directories(std::path::Path::new("/absent-temper"))
        }),
        Err("temper box: configuration /absent-temper/temper.json: absent; temper init writes one\n".into())
    );
}
#[test]
fn a_box_configuration_missing_members_is_refused_one_line_each() {
    let scratch = support::Scratch::startup("box-members");
    assert_eq!(
        refused_box(&scratch, &serde_json::json!({})),
        "temper box: engine: missing\ntemper box: workers: missing\ntemper box: application: missing\n"
    );
    scratch.remove();
}
#[test]
fn a_setup_naming_no_forge_refuses_the_box_before_it_starts() {
    let scratch = support::Scratch::startup("box-forge");
    let mut value = managed(scratch.path(), false);
    value["application"]["setup"][0]["forge"] = 7.into();
    assert_eq!(refused_box(&scratch, &value), "temper box: application.setup[0].forge: unknown reference\n");
    assert!(!scratch.path().join("forge-7").exists());
    scratch.remove();
}
#[test]
fn a_box_maximum_backoff_below_its_base_is_refused() {
    let scratch = support::Scratch::startup("box-backoff");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 100.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.backoff_ms: must agree with box.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_maximum_backoff_reaching_its_window_is_refused() {
    let scratch = support::Scratch::startup("box-window");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 300_000.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.backoff_max_ms: must agree with box.window_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_restart_count_past_its_bound_is_refused() {
    let scratch = support::Scratch::startup("box-restarts");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 65.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.restarts: past its bound\n"
    );
    scratch.remove();
}
#[test]
fn a_box_restart_count_outside_its_type_is_refused() {
    let scratch = support::Scratch::startup("box-restarts-type");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 4_294_967_296_u64.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.restarts: outside its type\n"
    );
    scratch.remove();
}
#[test]
fn an_unknown_box_limit_is_refused() {
    let scratch = support::Scratch::startup("box-unknown");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.nope"] = 1.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper box: application.limits.overrides.box.nope: no limit of temper's has this name\n"
    );
    scratch.remove();
}
#[test]
fn a_box_limit_past_its_check_refuses_temper_serve_too() {
    let scratch = support::Scratch::startup("box-serve-refused");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 100.into();
    support::write_private(&scratch.path().join("temper.json"), &serde_json::to_vec(&value).expect("JSON"));
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(scratch.path())).err().expect("serve refused"),
        "temper serve: application.limits.overrides.box.backoff_ms: must agree with box.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_limit_within_its_check_is_accepted_by_temper_serve() {
    let scratch = support::Scratch::startup("box-serve-accepted");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 3.into();
    support::write_private(&scratch.path().join("temper.json"), &serde_json::to_vec(&value).expect("JSON"));
    assert!(temper::prepare_in(&support::configuration::directories(scratch.path())).is_ok());
    scratch.remove();
}

#[test]
fn a_webhook_secret_readable_by_others_refuses_startup() {
    let scratch = support::Scratch::startup("public-hook-secret");
    let _ = support::webhooks::configuration(scratch.path());
    fs::set_permissions(scratch.path().join("secrets/forge-1.webhook-secret"), fs::Permissions::from_mode(0o644))
        .expect("public permissions");
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(scratch.path())).err().expect("configuration refused"),
        format!(
            "temper serve: application.forges[0].webhook_secret: {}: readable by others or reached through a link\n",
            scratch.path().join("secrets/forge-1.webhook-secret").display()
        )
    );
    scratch.remove();
}

fn owned_example(case: &str) -> support::Scratch {
    let scratch = support::Scratch::startup(case);
    let owned = support::owned::Owned::example();
    support::owned::lay(scratch.path(), &owned);
    fs::write(scratch.path().join("temper.json"), support::owned::example_document(scratch.path(), &owned))
        .expect("example configuration");
    scratch
}
#[test]
fn the_owned_example_asks_for_reconciles_files() {
    let scratch = owned_example("owned-example-refused");
    assert_eq!(
        temper::prepare_in(&support::configuration::directories(scratch.path())).err().expect("start refused"),
        "temper serve: application.forges[0].api_token: forge-1/api_token: absent\n\
         temper serve: application.forges[0].user.file: forge-1/user: absent\n\
         temper serve: application.forges[0].webhook_secret: forge-1/webhook_secret: absent\n\
         temper serve: application.identities[0].token: forge-1/git_token: absent\n"
            .replace(": forge-1/", &format!(": {}/forge-1/", scratch.path().display()))
    );
    scratch.remove();
}
#[test]
fn serve_prepares_once_on_the_owned_example() {
    const TOKEN_BYTES: usize = 512;
    const BOT_USER: &[u8] = b"42";
    let scratch = support::Scratch::startup("owned-example-prepared");
    let owned = support::owned::Owned::example();
    support::owned::lay(scratch.path(), &owned);
    support::write_private(
        &scratch.path().join("temper.json"),
        &support::owned::example_document(scratch.path(), &owned),
    );
    let forge = scratch.path().join("forge-1");
    fs::create_dir(&forge).expect("forge directory");
    fs::set_permissions(&forge, fs::Permissions::from_mode(0o700)).expect("private forge");
    for (name, bytes) in [
        ("api_token", [b't'; TOKEN_BYTES].as_slice()),
        ("user", BOT_USER),
        ("webhook_secret", b"webhook-secret"),
        ("git_token", b"git-token"),
    ] {
        support::write_private(&forge.join(name), bytes);
    }
    let prepared = temper::prepare_in(&support::configuration::directories(scratch.path()));
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    scratch.remove();
}

#[test]
fn the_owned_example_is_named_without_a_problem() {
    // Only naming a fixture document; no deployment or files are opened.
    let named = temper_service::named(support::owned::EXAMPLE, &jig_protocol_config::Directories::working())
        .expect("owned example named");
    assert_eq!(named.agent.as_ref(), b"/home/me/temper/owned/agent.json");
    assert_eq!(named.worker_root.as_ref(), b"workspaces");
    assert_eq!(named.deployment.as_ref(), b"state");
    for path in [
        b"forge-1/api_token".as_slice(),
        b"forge-1/user",
        b"forge-1/webhook_secret",
        b"forge-1/git_token",
        b"accounts/llm.json",
    ] {
        assert_eq!(
            named.files.iter().find(|file| file.path.as_ref() == path).map(|file| file.private),
            Some(true),
            "{path:?}"
        );
    }
}

#[test]
fn every_command_checks_setup_before_reading_files() {
    let scratch = owned_example("every-command-setup");
    let path = scratch.path().join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["limits"]["overrides"]["setup.backoff_ms"] = 6000.into();
    let bytes = serde_json::to_vec(&value).expect("configuration JSON");
    support::write_private(&path, &bytes);
    let line = "application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n";
    for (command, refused) in [
        (
            "serve",
            temper::prepare_in(&support::configuration::directories(scratch.path())).err().expect("serve refused"),
        ),
        (
            "box",
            temper::run(temper::Command::Box { directories: support::configuration::directories(scratch.path()) })
                .expect_err("box refused"),
        ),
    ] {
        assert_eq!(refused, format!("temper {command}: {line}"), "{command}");
    }
    for verb in ["prepare", "reconcile", "run"] {
        support::owned::refused(scratch.path(), verb, format!("temper forge {verb}: forge 1: {line}").as_bytes());
    }
    let runner = support::owned::command(scratch.path(), "runner").output().expect("runner refuses");
    assert_eq!(runner.status.code(), Some(1));
    assert_eq!(runner.stderr, format!("temper forge run: forge 1: {line}").as_bytes());
    scratch.remove();
}

const PAGE: &[u8] = b"<!doctype html><title>t</title>";
const DIGEST: &str = "bundle/0123456789abcdef";

/// A refusal's row: its name, what spoils the bundle, and the start's refusal.
type Spoiled<'a> = (&'a str, fn(&std::path::Path), &'a str);
/// A limit's row: its name, the configuration's change, and the start's refusal.
type Limited<'a> = (&'a str, fn(&mut serde_json::Value), &'a str);

fn fixture_bundle(directory: &std::path::Path) {
    fs::create_dir_all(directory.join(DIGEST).join("files")).expect("bundle directory");
    fs::write(directory.join("bundle/index.html"), PAGE).expect("bundle page");
    fs::write(directory.join(DIGEST).join("boot.js"), b"boot();").expect("bundle script");
    fs::write(directory.join(DIGEST).join("files/robots.txt"), b"User-agent: *\nDisallow:\n")
        .expect("bundle root file");
}
fn bundle_configuration(directory: &std::path::Path) {
    support::configuration(directory, &support::Forge::Unreached);
    let document = directory.join("temper.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&document).expect("configuration")).expect("JSON");
    value["application"]["clients"]["bundle"] = directory.join("bundle").to_str().expect("bundle path").into();
    support::write_private(&document, &serde_json::to_vec(&value).expect("JSON"));
}
#[test]
fn a_bundle_directory_is_read_with_its_one_digest_directory() {
    let scratch = support::Scratch::startup("bundle-read");
    let directory = scratch.path();
    fixture_bundle(directory);
    let limits = temper_service::limits::Clients { bundle_files: 128, bundle_bytes: 16 << 20 };
    let bundle =
        temper::bundle::read(directory.join("bundle").as_os_str().as_encoded_bytes(), limits).expect("bundle read");
    assert_eq!(bundle.page.as_ref(), PAGE);
    assert_eq!(bundle.etag.as_ref(), b"\"edaa28e315690734\"");
    assert_eq!(bundle.digest.as_ref(), b"0123456789abcdef");
    assert_eq!(bundle.files.len(), 2);
    let script = bundle.files.get(b"boot.js".as_slice()).expect("script");
    assert_eq!((script.kind, script.bytes.as_ref()), (b"text/javascript".as_slice(), b"boot();".as_slice()));
    let robots = bundle.files.get(b"files/robots.txt".as_slice()).expect("root file");
    assert_eq!(
        (robots.kind, robots.bytes.as_ref()),
        (b"text/plain; charset=utf-8".as_slice(), b"User-agent: *\nDisallow:\n".as_slice())
    );
    scratch.remove();
}
#[test]
fn a_directory_that_is_no_bundle_refuses_the_start() {
    let rows: [Spoiled<'_>; 4] = [
        (
            "no-page",
            |directory| fs::remove_file(directory.join("bundle/index.html")).expect("page removed"),
            "temper serve: application.clients.bundle: bundle/index.html: absent\n",
        ),
        (
            "no-digest",
            |directory| fs::remove_dir_all(directory.join(DIGEST)).expect("digest removed"),
            "temper serve: application.clients.bundle: bundle: no digest directory\n",
        ),
        (
            "two-digests",
            |directory| fs::create_dir(directory.join("bundle/fedcba9876543210")).expect("second digest"),
            "temper serve: application.clients.bundle: bundle/fedcba9876543210: more than one digest directory\n",
        ),
        (
            "off-the-list",
            |directory| fs::write(directory.join(DIGEST).join("files/data.bin"), b"x").expect("odd file"),
            "temper serve: application.clients.bundle: bundle/0123456789abcdef/files/data.bin: file type is not on the closed list\n",
        ),
    ];
    for (name, spoil, expected) in rows {
        let scratch = support::Scratch::startup(&format!("bundle-{name}"));
        let directory = scratch.path();
        bundle_configuration(directory);
        fixture_bundle(directory);
        spoil(directory);
        assert_eq!(
            temper::prepare_in(&support::configuration::directories(directory)).err().expect("refused"),
            expected.replace(": bundle", &format!(": {}/bundle", directory.display())),
            "row {name}"
        );
        scratch.remove();
    }
}
#[test]
fn a_bundle_past_a_limit_refuses_the_start() {
    let rows: [Limited<'_>; 2] = [
        (
            "files",
            |value| value["application"]["limits"]["overrides"]["clients.bundle_files"] = 2.into(),
            "temper serve: application.clients.bundle: bundle/0123456789abcdef/files/robots.txt: past clients.bundle_files\n",
        ),
        (
            "bytes",
            |value| value["application"]["limits"]["overrides"]["clients.bundle_bytes"] = 61.into(),
            "temper serve: application.clients.bundle: bundle/0123456789abcdef/files/robots.txt: past clients.bundle_bytes\n",
        ),
    ];
    for (name, limit, expected) in rows {
        let scratch = support::Scratch::startup(&format!("bundle-limit-{name}"));
        let directory = scratch.path();
        bundle_configuration(directory);
        fixture_bundle(directory);
        support::owned::configuration_value(directory, limit);
        assert_eq!(
            temper::prepare_in(&support::configuration::directories(directory)).err().expect("refused"),
            expected.replace(": bundle", &format!(": {}/bundle", directory.display())),
            "row {name}"
        );
        scratch.remove();
    }
}

#[test]
fn programs_directories_are_checked_at_the_name_before_startup_effects() {
    for row in ["missing", "private", "link", "linked-parent", "mode", "file"] {
        let scratch = support::Scratch::startup(&format!("programs-{row}"));
        let directory = scratch.path();
        support::configuration(directory, &support::Forge::Unreached);
        let target = directory.join("programs");
        if row != "missing" {
            if row == "file" {
                fs::write(&target, b"not a directory").expect("file");
            } else {
                fs::create_dir(&target).expect("programs directory");
                fs::set_permissions(&target, fs::Permissions::from_mode(if row == "mode" { 0o750 } else { 0o700 }))
                    .expect("directory mode");
            }
        }
        let path = match row {
            "link" => {
                let link = directory.join("linked");
                symlink(&target, &link).expect("leaf link");
                link
            }
            "linked-parent" => {
                let parent = directory.join("parent");
                symlink(directory, &parent).expect("parent link");
                parent.join("programs")
            }
            _ => target.clone(),
        };
        let agent_path = directory.join("agent.json");
        let mut agent: serde_json::Value =
            serde_json::from_slice(&fs::read(&agent_path).expect("agent")).expect("JSON");
        agent["programs"] = serde_json::json!({"claude_code":{"path":"/nonexistent/claude", "directory":path}});
        fs::write(agent_path, serde_json::to_vec(&agent).expect("agent JSON")).expect("agent");
        // A later missing secret proves that the preflight neither creates nor inspects a login.
        fs::remove_file(directory.join("secrets/forge-1.api-token")).expect("absent peer");
        let refused = temper::prepare_in(&support::configuration::directories(directory)).err().expect("refused");
        if ["missing", "private", "linked-parent"].contains(&row) {
            assert!(refused.contains("application.forges[0].api_token"), "{row}: {refused}");
            assert!(!refused.contains("programs.claude_code.directory"), "{row}: {refused}");
        } else {
            let reason = match row {
                "link" => "symbolic link",
                "mode" => "nonprivate mode bits 0o50",
                "file" => "not a directory",
                _ => unreachable!(),
            };
            assert_eq!(
                refused,
                format!(
                    "temper serve: workers[0].agent.configuration: programs.claude_code.directory: {}: {reason}\n",
                    path.display()
                )
            );
        }
        assert!(!directory.join("worker-root").exists(), "{row}: no worker root");
        assert!(!target.exists() || row != "missing", "missing programs stay missing");
        scratch.remove();
    }
}

#[test]
fn owned_memory_figures_match_startup_requirements_at_bounded_strings() {
    let scratch = support::Scratch::startup("owned-memory-figures");
    let bounded = support::owned::bounded(scratch.path());
    let required = bounded.requirements();
    support::owned::check_memory(required);
    eprintln!("owned requirements: engine {}, worker {} bytes", required.engine_bytes, required.worker_bytes);
    let mut value: serde_json::Value = serde_json::from_slice(&bounded.document).expect("bounded JSON");
    value["engine"]["memory_bytes"] = required.engine_bytes.into();
    value["workers"][0]["memory_bytes"] = required.worker_bytes.into();
    let document = serde_json::to_vec(&value).expect("exact envelopes JSON");
    let started = bounded.startup(&document);
    assert!(
        started.is_ok(),
        "{}",
        String::from_utf8_lossy(&temper_service::startup::lines(&started.err().expect("startup refusal")))
    );
    for (engine, path) in [(true, "engine.memory_bytes"), (false, "workers[0].memory_bytes")] {
        let mut short = value.clone();
        if engine {
            short["engine"]["memory_bytes"] = (required.engine_bytes - 1).into();
        } else {
            short["workers"][0]["memory_bytes"] = (required.worker_bytes - 1).into();
        }
        let refused = bounded
            .startup(&serde_json::to_vec(&short).expect("one-byte-short JSON"))
            .err()
            .expect("refused before construction");
        assert_eq!(
            String::from_utf8_lossy(&temper_service::startup::lines(&refused)),
            format!("{path}: the worst case passes memory_bytes by 1 bytes\n")
        );
    }
    scratch.remove();
}

#[test]
fn program_login_refuses_invalid_input_before_creation() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    for row in ["no-program", "malformed", "missing-members", "oversized", "bad-agent", "bad-name"] {
        let scratch = support::Scratch::startup(&format!("login-{row}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        configuration(
            directory,
            (row != "no-program").then_some(programs.as_path()),
            std::path::Path::new("/nonexistent/claude"),
        );
        match row {
            "malformed" => fs::write(directory.join("temper.json"), b"{").expect("malformed JSON"),
            "missing-members" => fs::write(directory.join("temper.json"), b"{}").expect("missing members"),
            "oversized" => fs::File::create(directory.join("temper.json"))
                .expect("configuration")
                .set_len(1_048_577)
                .expect("past bound"),
            "bad-agent" => fs::write(directory.join("agent.json"), b"{}").expect("bad agent"),
            _ => {}
        }
        let why = refused(directory, if row == "bad-name" { "../escape" } else { "source" }.as_ref());
        assert!(why.starts_with("temper program-login: "), "{row}: {why}");
        assert_eq!(why.lines().count(), 1, "{row}: {why}");
        assert!(!why.contains("exec:"), "{row}: {why}");
        if row == "no-program" {
            assert!(why.contains("programs.claude_code: not configured"), "{why}");
        }
        if row == "oversized" {
            assert!(why.contains("past 1048576 bytes"), "{why}");
        }
        if row == "bad-name" {
            assert!(why.contains("source name must be"), "{why}");
        }
        assert!(!programs.exists());
        no_deployment_resources(directory);
        scratch.remove();
    }
}

#[test]
fn program_login_checks_both_directory_names_before_creating_anything() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    for row in
        ["program-link", "program-mode", "program-file", "source-link", "source-mode", "source-file", "source-dangling"]
    {
        let scratch = support::Scratch::startup(&format!("login-{row}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        let source = programs.join("source");
        if row.starts_with("source-") {
            fs::create_dir(&programs).expect("programs");
            fs::set_permissions(&programs, fs::Permissions::from_mode(0o700)).expect("private");
        }
        let target = if row.starts_with("source-") { &source } else { &programs };
        let expected = if row.ends_with("link") || row.ends_with("dangling") {
            symlink(directory.join("missing"), target).expect("linked leaf");
            "symbolic link"
        } else if row.ends_with("mode") {
            fs::create_dir(target).expect("directory");
            fs::set_permissions(target, fs::Permissions::from_mode(0o750)).expect("unsafe mode");
            "nonprivate mode bits 0o50"
        } else {
            fs::write(target, b"file").expect("wrong kind");
            "not a directory"
        };
        configuration(directory, Some(&programs), std::path::Path::new("/nonexistent/claude"));
        let why = refused(directory, "source".as_ref());
        assert!(why.contains(expected), "{row}: {why}");
        assert!(why.contains(&target.display().to_string()), "{row}: {why}");
        assert!(!directory.join("missing").exists(), "linked peer stays absent");
        if row.starts_with("program-") {
            assert!(fs::symlink_metadata(&source).is_err(), "missing source stays absent");
        }
        no_deployment_resources(directory);
        scratch.remove();
    }
}

#[test]
fn program_login_prepares_private_directories_through_each_supported_parent() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    for row in ["missing", "private", "linked-parent"] {
        let scratch = support::Scratch::startup(&format!("login-create-{row}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        if row == "private" {
            fs::create_dir(&programs).expect("programs");
            fs::set_permissions(&programs, fs::Permissions::from_mode(0o700)).expect("private");
        }
        let configured = if row == "linked-parent" {
            symlink(directory, directory.join("parent")).expect("linked parent");
            directory.join("parent/programs")
        } else {
            programs.clone()
        };
        configuration(directory, Some(&configured), std::path::Path::new("/nonexistent/claude"));
        let why = refused(directory, "source".as_ref());
        assert!(why.starts_with("temper program-login: exec: "), "{row}: {why}");
        assert_eq!(why.lines().count(), 1);
        for path in [&programs, &programs.join("source")] {
            assert_eq!(fs::metadata(path).expect("created directory").permissions().mode() & 0o777, 0o700);
        }
        no_deployment_resources(directory);
        scratch.remove();
    }
}
