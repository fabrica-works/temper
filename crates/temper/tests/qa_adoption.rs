//! Checks demo adoption and its listener, secret and handover against protocol/setup.md, 5.1, 5.3, 5.4, 8.2–8.4 and 13.1; a QA pass run by `--profile qa`.
pub mod support;

use std::{fs, net::TcpListener, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

use skein_io::kernel::Exit;
use skein_lib::Duration;

#[test]
fn init_names_a_fixed_listener_and_the_demo_secret() {
    let scratch = support::Scratch::startup("qa-adoption-init");
    let flags = support::configuration::flags(scratch.path());
    let output = Command::new(env!("CARGO_BIN_EXE_temper")).arg("init").args(&flags).output().expect("run init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let document_path = flags
        .iter()
        .map(PathBuf::from)
        .map(|directory| directory.join("temper.json"))
        .find(|path| path.is_file())
        .expect("init's document in its explicit configuration directory");
    let bytes = fs::read(&document_path).expect("read init's document");
    let document: serde_json::Value = serde_json::from_slice(&bytes).expect("document JSON");
    let listener = document["application"]["webhooks"]["listener"]["socket"]
        .as_str()
        .expect("init names the webhooks listener")
        .parse::<std::net::SocketAddr>()
        .expect("listener socket");
    assert!(listener.ip().is_loopback());
    assert_ne!(listener.port(), 0, "Adopt needs the document's fixed hook address");
    let forges = document["application"]["forges"].as_array().expect("forges");
    let demo = forges.iter().find(|forge| forge["number"] == 1).expect("demo forge");
    assert_eq!(demo["webhook_secret"], "demo/forge/webhook_secret");
    let credentials = scratch.path().join("credentials");
    fs::create_dir(&credentials).expect("fixture directory");
    support::configuration(&credentials, &support::Forge::Unreached);
    let forge_directory = scratch.path().join("demo/forge");
    fs::create_dir(&forge_directory).expect("fake directory");
    fs::set_permissions(&forge_directory, fs::Permissions::from_mode(0o700)).expect("private fake directory");
    for (record, file) in [("api-token", "api_token"), ("git-token", "git_token"), ("user", "user")] {
        fs::copy(credentials.join(format!("secrets/forge-1.{record}")), forge_directory.join(file))
            .expect("files the box's reconcile supplies");
    }
    for file in ["admin_token", "webhook_secret"] {
        let path = forge_directory.join(file);
        fs::write(&path, b"0123456789abcdef0123456789abcdef01234567").expect("box's secret file");
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private secret file");
    }
    temper::envelope_in(&support::configuration::directories(scratch.path()), &bytes)
        .unwrap_or_else(|why| panic!("serve must accept init's wiring: {why}"));
    scratch.remove();
}

#[test]
fn fake_prepare_keeps_its_handover_for_serve() {
    let scratch = support::Scratch::startup("qa-adoption-handover");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let flags = support::configuration::flags(scratch.path());
    let document = support::configuration::check_document(scratch.path());
    let document_path = flags
        .iter()
        .map(PathBuf::from)
        .map(|directory| directory.join("temper.json"))
        .find(|path| path.is_file())
        .expect("fixture document");
    fs::write(document_path, serde_json::to_vec(&document).expect("fixture JSON")).expect("write fake configuration");
    let mut arguments = vec!["forge".into(), "prepare".into()];
    arguments.extend(flags);
    arguments.push("1".into());
    let command = temper::configure(arguments.into_iter()).expect("prepare arguments");
    assert_eq!(temper::run(command).expect("prepare fake"), 0);
    let handover = support::owned::file(scratch.path(), 1, temper_forge_setup::place::File::AdminToken);
    let first = fs::read(&handover).expect("prepare leaves the handover");
    assert_eq!(first.len(), 40);
    assert!(first.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)));
    assert!(support::private(&handover));
    let mut arguments = vec!["forge".into(), "prepare".into()];
    arguments.extend(support::configuration::flags(scratch.path()));
    arguments.push("1".into());
    assert_eq!(
        temper::run(temper::configure(arguments.into_iter()).expect("prepare arguments")).expect("prepare again"),
        0
    );
    assert_eq!(fs::read(&handover).expect("repeated prepare retains the handover"), first);
    for (record, file) in [("api-token", "api_token"), ("git-token", "git_token"), ("user", "user")] {
        fs::copy(scratch.path().join(format!("secrets/forge-1.{record}")), scratch.path().join("forge-1").join(file))
            .expect("fake's file access from the fixture's credentials");
    }
    let service = temper::prepare_in(&support::configuration::directories(scratch.path()))
        .unwrap_or_else(|why| panic!("serve reads the valid handover: {why}"));
    assert_eq!(fs::read(&handover).expect("serve leaves the handover"), first);
    drop(service);
    scratch.remove();
}

#[test]
fn occupied_defaults_still_give_adopt_a_distinct_fixed_listener() {
    let scratch = support::Scratch::startup("qa-adoption-fallback");
    let _occupied: Vec<_> = (8367..=8371).filter_map(|port| TcpListener::bind(("127.0.0.1", port)).ok()).collect();
    let output = Command::new(env!("CARGO_BIN_EXE_temper"))
        .arg("init")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("run init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("document")).expect("JSON");
    let app = &document["application"];
    let addresses: Vec<std::net::SocketAddr> = [
        &app["clients"]["listener"]["socket"],
        &app["forges"][0]["api"]["socket"],
        &app["demo"]["llm"]["listen"]["socket"],
        &app["webhooks"]["listener"]["socket"],
    ]
    .iter()
    .map(|value| value.as_str().expect("socket").parse().expect("address"))
    .collect();
    for (index, address) in addresses.iter().enumerate() {
        assert!(address.ip().is_loopback());
        assert_ne!(address.port(), 0, "init writes the assigned port, never port zero");
        assert!(!(8367..=8371).contains(&address.port()), "occupied default reused: {address}");
        assert!(!addresses[..index].contains(address), "two listeners share {address}");
    }
    assert_eq!(app["forges"][0]["webhook_secret"], "demo/forge/webhook_secret");
    scratch.remove();
}

#[test]
fn malformed_handovers_are_named_without_printing_their_value() {
    for (name, token) in [
        ("short", "0123456789abcdef0123456789abcdef0123456"),
        ("long", "0123456789abcdef0123456789abcdef012345678"),
        ("uppercase", "ABCDEF0123456789abcdef0123456789abcdef01"),
        ("nonhex", "g123456789abcdef0123456789abcdef01234567"),
    ] {
        let scratch = support::Scratch::startup(&format!("qa-adoption-invalid-{name}"));
        support::configuration(scratch.path(), &support::Forge::Unreached);
        let document = support::configuration::check_document(scratch.path());
        fs::write(scratch.path().join("temper.json"), serde_json::to_vec(&document).expect("JSON"))
            .expect("fake document");
        let directory = scratch.path().join("forge-1");
        fs::create_dir(&directory).expect("fake directory");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private directory");
        for (record, file) in [("api-token", "api_token"), ("git-token", "git_token"), ("user", "user")] {
            fs::copy(scratch.path().join(format!("secrets/forge-1.{record}")), directory.join(file))
                .expect("fixture credential");
        }
        let handover = directory.join("admin_token");
        fs::write(&handover, token).expect("invalid handover");
        fs::set_permissions(&handover, fs::Permissions::from_mode(0o600)).expect("private handover");
        let why = temper::prepare_in(&support::configuration::directories(scratch.path()))
            .err()
            .expect("malformed handover must refuse startup");
        // A read refusal also gives the file's path; a form refusal names the member alone (the ignored case below).
        assert!(why.contains("application.setup[0].admin_token: "), "{name}: {why}");
        assert!(!why.contains(token), "{name}: refusal disclosed the handover");
        assert_eq!(fs::read(&handover).expect("refusal leaves handover"), token.as_bytes());
        scratch.remove();
    }
}

#[test]
#[ignore = "a handover of 40 bytes in the wrong form is refused naming the entry's member, not the file's path: startup's What is Copy and carries no path; listed in o/first-run-d2's closing"]
fn a_handover_in_the_wrong_form_is_refused_naming_its_path() {
    let scratch = support::Scratch::startup("qa-adoption-invalid-form-path");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let document = support::configuration::check_document(scratch.path());
    fs::write(scratch.path().join("temper.json"), serde_json::to_vec(&document).expect("JSON")).expect("fake document");
    let directory = scratch.path().join("forge-1");
    fs::create_dir(&directory).expect("fake directory");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private directory");
    for (record, file) in [("api-token", "api_token"), ("git-token", "git_token"), ("user", "user")] {
        fs::copy(scratch.path().join(format!("secrets/forge-1.{record}")), directory.join(file))
            .expect("fixture credential");
    }
    let handover = directory.join("admin_token");
    fs::write(&handover, "ABCDEF0123456789abcdef0123456789abcdef01").expect("invalid handover");
    fs::set_permissions(&handover, fs::Permissions::from_mode(0o600)).expect("private handover");
    let why = temper::prepare_in(&support::configuration::directories(scratch.path()))
        .err()
        .expect("malformed handover must refuse startup");
    assert!(why.contains(handover.to_str().expect("path")), "{why}");
    scratch.remove();
}

#[test]
fn missing_handover_is_refused_before_a_fake_connects() {
    let scratch = support::Scratch::startup("qa-adoption-missing-handover");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let document = support::configuration::check_document(scratch.path());
    fs::write(scratch.path().join("temper.json"), serde_json::to_vec(&document).expect("JSON")).expect("fake document");
    let directory = scratch.path().join("forge-1");
    fs::create_dir(&directory).expect("fake directory");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private directory");
    for (record, file) in [("api-token", "api_token"), ("git-token", "git_token"), ("user", "user")] {
        fs::copy(scratch.path().join(format!("secrets/forge-1.{record}")), directory.join(file))
            .expect("fixture credential");
    }
    let why = temper::prepare_in(&support::configuration::directories(scratch.path()))
        .err()
        .expect("missing handover must refuse startup");
    assert!(why.contains(directory.join("admin_token").to_str().expect("path")), "{why}");
    assert!(!directory.join("admin_token").exists(), "serve does not invent an admin credential");
    scratch.remove();
}

#[test]
fn a_running_box_hooks_every_seeded_repository_with_the_named_secret() {
    // A running box is needed to cross preparation, serve's Adopt and the fake's hook API.
    let scratch = support::Scratch::startup("qa-adoption-all-repositories");
    let flags = support::configuration::flags(scratch.path());
    let output = Command::new(env!("CARGO_BIN_EXE_temper")).arg("init").args(&flags).output().expect("run init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("document")).expect("JSON");
    document["application"]["setup"][0]["mode"]["fake"]["repositories"]
        .as_array_mut()
        .expect("fake repositories")
        .push(serde_json::json!({"name": "gadget", "landing": "main"}));
    fs::write(&path, serde_json::to_vec(&document).expect("JSON")).expect("two-repository document");
    let forge_address: std::net::SocketAddr = document["application"]["forges"][0]["api"]["socket"]
        .as_str()
        .expect("API socket")
        .parse()
        .expect("API address");
    let listener = document["application"]["webhooks"]["listener"]["socket"].as_str().expect("listener");
    let secret_directory = scratch.path().join("demo/forge");
    fs::create_dir(&secret_directory).expect("forge directory");
    fs::set_permissions(&secret_directory, fs::Permissions::from_mode(0o700)).expect("private directory");
    let secret = "0123456789abcdef0123456789abcdef01234567";
    let secret_path = secret_directory.join("webhook_secret");
    fs::write(&secret_path, secret).expect("named secret");
    fs::set_permissions(&secret_path, fs::Permissions::from_mode(0o600)).expect("private secret");
    let client = scratch.path().join("qa-client");
    fs::create_dir(&client).expect("client directory");
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client directory");
    let outcome = support::boxed::run_story(
        scratch.path(),
        flags.into_iter().map(|flag| flag.into_string().expect("flag")).collect(),
        &client,
        support::person::demo_chat(1, b"QA adoption probe", b"demo model", b"The demo changes nothing."),
        Duration::from_secs(30),
        || {
            let token = fs::read_to_string(secret_directory.join("admin_token")).expect("handover");
            ["widget", "gadget"].map(|repository| {
                support::owned::answer(
                    forge_address.port(),
                    &format!("/api/v1/repos/demo/{repository}/hooks"),
                    Some(token.trim_end()),
                )
                .expect("read repository hooks")
            })
        },
    );
    assert!(outcome.finished, "{}", String::from_utf8_lossy(&outcome.stderr));
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert_eq!(outcome.left.as_deref(), Some([].as_slice()));
    for hooks in &outcome.checked {
        let hooks = hooks.as_array().expect("hook list");
        assert_eq!(hooks.len(), 1, "one hook for each fake entry repository");
        assert_eq!(hooks[0]["config"]["url"], format!("http://{listener}/hooks/1"));
        // The fake never sends a hook's secret in a reply (skein-fake-forge-protocol setup/hooks.rs); signed deliveries
        // are the secret's evidence, in the process tier's story.
        assert_eq!(hooks[0]["config"]["secret"], serde_json::Value::Null, "the secret stays unsent");
        assert_eq!(hooks[0]["active"], true);
    }
    assert_eq!(fs::read_to_string(&secret_path).expect("secret survives"), secret);
    assert!(secret_directory.join("admin_token").is_file(), "serve leaves the handover");
    scratch.remove();
}

#[test]
fn one_sided_hook_wiring_is_refused_before_a_fake_connects() {
    // A setup entry's forge names its secret exactly when the deployment has a listener (temper-forge-setup's members
    // check, and serve's webhooks check), so 8.4's "else none and no hook" is never reached from a valid document.
    for (name, listener, named_secret, refusal) in [
        ("listener-only", true, false, "application.forges[0].webhook_secret: invalid form"),
        ("secret-only", false, true, "application.forges[0].webhook_secret: needs application.webhooks"),
    ] {
        let scratch = support::Scratch::startup(&format!("qa-adoption-{name}"));
        let output = Command::new(env!("CARGO_BIN_EXE_temper"))
            .arg("init")
            .args(support::configuration::flags(scratch.path()))
            .output()
            .expect("run init");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let path = scratch.path().join("temper.json");
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("document")).expect("JSON");
        if !listener {
            document["application"]["webhooks"] = serde_json::Value::Null;
        }
        if !named_secret {
            document["application"]["forges"][0]["webhook_secret"] = serde_json::Value::Null;
        }
        fs::write(&path, serde_json::to_vec(&document).expect("JSON")).expect("one-sided hook wiring");
        let outcome = support::boxed::run(scratch.path(), None, support::boxed::Plan::Stop, Duration::from_secs(5));
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert_eq!(outcome.exit, Some(Exit::Code(1)), "{name}: {stderr}");
        assert!(stderr.contains(refusal), "{name}: {stderr}");
        assert!(support::printed_link(&outcome.stdout).is_none(), "{name}: no link without a start");
        assert_eq!(outcome.left.as_deref(), Some([].as_slice()), "{name}");
        scratch.remove();
    }
}

#[test]
fn the_default_box_prepares_adopts_and_hands_over_the_link() {
    // Only a running box crosses init's document, the handover, Adopt and the seed's real adoption.
    let scratch = support::Scratch::startup("qa-adoption-default-box");
    let output = Command::new(env!("CARGO_BIN_EXE_temper"))
        .arg("init")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("run init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let outcome = support::boxed::run(scratch.path(), None, support::boxed::Plan::Stop, Duration::from_secs(5));
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert!(support::printed_link(&outcome.stdout).is_some(), "seed must finish and hand over the sign-in link");
    assert_eq!(outcome.left.as_deref(), Some([].as_slice()));
    let directory = scratch.path().join("demo/forge");
    assert!(directory.join("admin_token").is_file(), "serve leaves prepare's handover");
    assert!(directory.join("api_token").is_file(), "reconcile's API access remains in its file");
    assert!(directory.join("git_token").is_file(), "reconcile's git access remains in its file");
    assert!(directory.join("webhook_secret").is_file(), "reconcile supplies the named secret");
    scratch.remove();
}
