#[test]
fn directory_commands_take_flags_after_their_command_words() {
    for (words, valid) in [
        (vec!["serve"], true),
        (vec!["box"], true),
        (vec!["serve", "old.json"], false),
        (vec!["program-login", "name"], true),
        (vec!["program-login"], false),
        (vec!["forge", "prepare", "1"], true),
        (vec!["forge", "run", "1", "runner"], true),
        (vec!["forge", "reconcile", "1"], true),
        (vec!["unknown"], false),
    ] {
        let mut words: Vec<Box<[u8]>> = words.iter().map(|v| Box::from(v.as_bytes())).collect();
        words
            .extend([b"--config".as_slice(), b"/c", b"--data", b"/d", b"--cache", b"/k"].iter().map(|v| Box::from(*v)));
        assert_eq!(super::configure_with(&words, &[], b"/").is_ok(), valid);
    }
}
const OPEN_LINK: &[u8] =
    b"http://127.0.0.1:7000/jig/v1/sign-in/link?secret=AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM".as_slice();
type OpenRow = (&'static str, &'static [u8], Result<(bool, &'static str), &'static str>);
const OPEN_ARGUMENTS: &[OpenRow] = &[
    ("link", OPEN_LINK, Ok((true, "127.0.0.1:7000"))),
    ("address", b"http://127.0.0.1:7000".as_slice(), Ok((false, "127.0.0.1:7000"))),
    ("slash", b"http://127.0.0.1:7000/".as_slice(), Ok((false, "127.0.0.1:7000"))),
    ("IPv6", b"http://[::1]:7000/".as_slice(), Ok((false, "[::1]:7000"))),
    ("loopback range", b"http://127.1.2.3:7000".as_slice(), Ok((false, "127.1.2.3:7000"))),
    ("scheme", b"https://127.0.0.1:7000".as_slice(), Err("not a link or an address: https://127.0.0.1:7000")),
    ("hostname", b"http://localhost:7000".as_slice(), Err("not a link or an address: http://localhost:7000")),
    ("port", b"http://127.0.0.1".as_slice(), Err("not a link or an address: http://127.0.0.1")),
    ("remote", b"http://10.0.0.1:7000".as_slice(), Err("the engine's address must be on loopback")),
    ("path", b"http://127.0.0.1:7000/other".as_slice(), Err("not a link or an address: http://127.0.0.1:7000/other")),
    (
        "short secret",
        b"http://127.0.0.1:7000/jig/v1/sign-in/link?secret=AA".as_slice(),
        Err("not a link or an address: http://127.0.0.1:7000/jig/v1/sign-in/link?secret=AA"),
    ),
];

#[test]
fn each_open_argument_reads_as_a_link_an_address_or_a_refusal() {
    use super::open::{Refused, Target, target};
    for &(row, argument, expected) in OPEN_ARGUMENTS {
        let found = target(argument)
            .map(|target| match target {
                Target::Link { engine, link } => {
                    assert_eq!(link.expose(), argument, "{row}: link bytes");
                    (true, engine.to_string())
                }
                Target::Address { engine } => (false, engine.to_string()),
            })
            .map_err(|why: Refused| why.line(std::path::Path::new("client")));
        assert_eq!(
            found.as_ref().map(|(link, engine)| (*link, engine.as_str())).map_err(String::as_str),
            expected
                .map_err(|why| format!("temper open: {why}\n"))
                .as_ref()
                .map(|(link, engine)| (*link, *engine))
                .map_err(String::as_str),
            "{row}"
        );
    }
}
#[test]
fn open_without_two_arguments_prints_its_usage() {
    for (row, arguments, expected) in [
        ("none", vec![], "temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS\n"),
        ("one", vec!["client".into()], "temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS\n"),
        (
            "three",
            vec!["client".into(), "address".into(), "extra".into()],
            "temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS\n",
        ),
    ] {
        let refused = super::open::start(&arguments, &skein_shell::Clock::new()).expect_err("usage refused");
        assert_eq!(refused.line(std::path::Path::new("client")), expected, "{row}");
    }
}

#[path = "../tests/support/configuration.rs"]
pub(crate) mod configuration;

#[test]
fn the_standalone_envelope_retains_no_descriptors() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let directory = std::env::temp_dir().join(format!("temper-envelope-{}", std::process::id()));
    fs::create_dir(&directory).expect("envelope scratch");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private scratch");
    for name in ["accounts", "secrets", "charters", "state"] {
        fs::create_dir_all(directory.join(name)).expect("fixture parents");
    }
    let write = |name: &str, bytes: &[u8]| {
        let path = directory.join(name);
        fs::write(&path, bytes).expect("fixture bytes");
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
    };
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        write(std::str::from_utf8(&name).expect("fixture path"), bytes.expose());
    }
    for (name, bytes) in [
        ("worker.secret", b"worker-secret".as_slice()),
        ("secrets/forge-1.user", b"42\n".as_slice()),
        ("secrets/forge-1.api-token", b"forge-1-api-token-value".as_slice()),
        ("secrets/forge-1.git-token", b"forge-1-git-token-value".as_slice()),
        ("root.der", skein_tls_world::pki::ROOT),
    ] {
        write(name, bytes);
    }
    write("accounts/chat.json", br#"{"access_token":"token","expires_at":4102444800,"account_id":"acc"}"#);
    let (agent, document) = configuration::documents(&directory, "/fixture/temper");
    write("agent.json", &agent);
    assert_eq!(configuration::flags(&directory).len(), 6);
    let envelope = super::envelope_in(&configuration::directories(&directory), &document).expect("standalone envelope");
    assert_no_descriptors_in(&directory);
    let mut owned = Vec::new();
    let service = super::prepare(&configuration::directories(&directory), &document, &mut owned)
        .expect("same standalone preparation");
    assert_eq!(envelope, (service.worst_case(), service.operations()));
    drop(service);
    super::close(owned);
    assert_no_descriptors_in(&directory);
    fs::remove_dir_all(directory).expect("remove envelope scratch");
}
fn assert_no_descriptors_in(directory: &std::path::Path) {
    for entry in std::fs::read_dir("/proc/self/fd").expect("open descriptor list") {
        let path = entry.expect("descriptor entry").path();
        if let Ok(target) = std::fs::read_link(&path) {
            assert!(!target.starts_with(directory), "descriptor {path:?} still holds {target:?}");
        }
    }
}

#[test]
fn ctl_is_configured() {
    for arguments in [vec!["help".into()], vec!["directory".into(), "me".into()]] {
        let words = std::iter::once("ctl".into()).chain(arguments.clone());
        assert_eq!(
            super::configure_with(
                &words.map(|v: std::ffi::OsString| Box::from(v.as_encoded_bytes())).collect::<Vec<_>>(),
                &[],
                b"/"
            ),
            Ok(super::Command::Ctl { arguments })
        );
    }
}

#[test]
fn fixed_sign_in_ports_are_checked_against_owned_listeners_only() {
    // Fixture parsing without a deployment keeps written paths.
    let example = include_bytes!("../../../docs/examples/owned.json");
    let mut value: serde_json::Value = serde_json::from_slice(example).expect("owned JSON");
    value["engine"]["llm"]["providers"] = serde_json::json!([{
        "name":"OpenAI", "number":1, "models":["gpt-6-luna"], "sources":{
            "sign_in": {"issuer":"https://issuer.example", "client":"client", "scopes":[],
                "authorization":"https://issuer.example/auth", "token":"https://issuer.example/token",
                "keys":"https://issuer.example/keys", "transport":"tls", "server_name":"issuer.example",
                "must_grant":[], "redirect_path":"callback", "redirect_port":3000}
        }
    }]);
    let bytes = serde_json::to_vec(&value).expect("owned JSON");
    assert_eq!(
        super::setup::check(&bytes, &jig_protocol_config::Directories::working()).expect_err("owned listen conflict"),
        "engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.setup[0].mode.owned.listen\n"
    );
    // The same API on a connected forge belongs to the remote peer, not this process.
    value["application"]["setup"] = serde_json::json!([]);
    assert!(
        super::setup::check(
            &serde_json::to_vec(&value).expect("connected JSON"),
            &jig_protocol_config::Directories::working()
        )
        .is_ok()
    );
}

#[test]
fn a_programs_owner_refusal_identifies_the_directory_ownership_conflict() {
    use smith_agent_shell::program::{Refusal, Unsafe};
    let other = skein_shell::effective_user().wrapping_add(1);
    let why = super::agent::program_directory_refusal(&Refusal::Unsafe {
        path: "/programs".into(),
        found: Unsafe::Owner { found: other },
    });
    assert_eq!(
        why,
        format!("workers[0].agent.configuration: programs.claude_code.directory: /programs: owned by user {other}")
    );
}

#[test]
fn program_login_refuses_arguments_outside_the_required_source_name() {
    for arguments in [
        vec!["program-login"],
        vec!["program-login", "source", "extra"],
        vec!["program-login", "config.json", "source", "extra"],
        vec!["sign-in", "source"],
    ] {
        assert_eq!(configure_program_login(&arguments), Err(format!("temper: invalid arguments\n{}", super::usage())));
    }
}
#[test]
fn program_login_accepts_exactly_the_required_source_name() {
    let directories =
        super::directories::resolve(&[], &[(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))], b"/")
            .expect("default directories")
            .0;
    assert_eq!(
        configure_program_login(&["program-login", "source"]),
        Ok(super::Command::ProgramLogin { directories, name: "source".into() })
    );
}

fn configure_program_login(arguments: &[&str]) -> Result<super::Command, String> {
    super::configure_with(
        &arguments.iter().map(|value| Box::from(value.as_bytes())).collect::<Vec<_>>(),
        &[(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))],
        b"/",
    )
}
