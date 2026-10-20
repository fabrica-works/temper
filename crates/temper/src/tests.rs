use super::configure;

#[test]
fn serve_without_a_configuration_path_is_a_usage_error() {
    assert_eq!(
        configure(["serve".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json | temper open DIRECTORY LINK_OR_ADDRESS | temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner | temper forge reconcile CONFIG.json FORGE | temper box CONFIG.json | temper ctl DIRECTORY TOOL [ARGUMENT...] | temper ctl help [TOOL]\n".into())
    );
}
#[test]
fn an_unknown_command_prints_the_usage_line() {
    assert_eq!(
        configure(["unknown".into(), "config.json".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json | temper open DIRECTORY LINK_OR_ADDRESS | temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner | temper forge reconcile CONFIG.json FORGE | temper box CONFIG.json | temper ctl DIRECTORY TOOL [ARGUMENT...] | temper ctl help [TOOL]\n".into())
    );
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
mod configuration;
#[test]
fn forge_without_its_arguments_prints_its_usage() {
    assert_eq!(
        super::forge::start(&[]).expect_err("usage refused").line(),
        "temper forge prepare: usage: temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner | temper forge reconcile CONFIG.json FORGE\n"
    );
}

#[test]
fn box_with_one_path_is_the_box_command() {
    assert_eq!(
        configure(["box".into(), "config.json".into()].into_iter()),
        Ok(super::Command::Box { configuration: "config.json".into() })
    );
}
#[test]
fn box_without_a_path_prints_the_usage() {
    assert_eq!(configure(["box".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json | temper open DIRECTORY LINK_OR_ADDRESS | temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner | temper forge reconcile CONFIG.json FORGE | temper box CONFIG.json | temper ctl DIRECTORY TOOL [ARGUMENT...] | temper ctl help [TOOL]\n".into()));
}
#[test]
fn box_with_two_paths_prints_the_usage() {
    assert_eq!(configure(["box".into(), "one.json".into(), "two.json".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json | temper open DIRECTORY LINK_OR_ADDRESS | temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner | temper forge reconcile CONFIG.json FORGE | temper box CONFIG.json | temper ctl DIRECTORY TOOL [ARGUMENT...] | temper ctl help [TOOL]\n".into()));
}

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
    let envelope = super::envelope_in(&directory, &document).expect("standalone envelope");
    assert_no_descriptors_in(&directory);
    let mut owned = Vec::new();
    let service = super::prepare(&directory, &document, &mut owned).expect("same standalone preparation");
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
        assert_eq!(configure(words), Ok(super::Command::Ctl { arguments }));
    }
}
