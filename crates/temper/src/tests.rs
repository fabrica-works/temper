use super::configure;

#[test]
fn serve_without_a_configuration_path_is_a_usage_error() {
    assert_eq!(
        configure(["serve".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json | temper open DIRECTORY LINK_OR_ADDRESS | temper sign-in CONFIG.json ACCOUNT | temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner\n".into())
    );
}
#[test]
fn an_unknown_command_prints_the_usage_line() {
    assert_eq!(
        configure(["unknown".into(), "config.json".into()].into_iter()),
        Err("temper: invalid arguments\nusage: temper serve CONFIG.json | temper agent AGENT.json | temper open DIRECTORY LINK_OR_ADDRESS | temper sign-in CONFIG.json ACCOUNT | temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner\n".into())
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
fn each_unsigned_account_is_refused_naming_it() {
    let (_, document) = configuration::documents(std::path::Path::new("/fixture"), "/opt/temper");
    let original: serde_json::Value = serde_json::from_slice(&document).expect("fixture JSON");
    let confidential = serde_json::json!({"sign_in":{"client":"client","secret":"secrets/client-secret",
        "authorization":"http://127.0.0.1:34544/authorize","token":"http://127.0.0.1:34544/token",
        "endpoint":{"socket":"127.0.0.1:34544"},"transport":"plaintext","redirect":"http://127.0.0.1:2345/callback","scope":"openid"}});
    for (row, number, source, expected) in [
        ("missing", 7, None, "temper sign-in: account 7: no such account\n"),
        ("handed in", 1, None, "temper sign-in: account 1: handed in, cannot sign in\n"),
        (
            "client secret",
            1,
            Some(confidential),
            "temper sign-in: account 1: has a client secret, cannot sign in at a terminal\n",
        ),
    ] {
        let mut document = original.clone();
        if let Some(source) = source {
            document["engine"]["llm"]["accounts"][0]["source"] = source;
        }
        let bytes = serde_json::to_vec(&document).expect("fixture JSON");
        assert_eq!(super::sign_in::account(&bytes, number).err().expect("account refused").line(), expected, "{row}");
    }
}
#[test]
fn sign_in_without_two_arguments_prints_its_usage() {
    for (row, arguments, expected) in [
        ("none", vec![], "temper sign-in: usage: temper sign-in CONFIG.json ACCOUNT\n"),
        ("one", vec!["config.json".into()], "temper sign-in: usage: temper sign-in CONFIG.json ACCOUNT\n"),
        (
            "three",
            vec!["config.json".into(), "1".into(), "extra".into()],
            "temper sign-in: usage: temper sign-in CONFIG.json ACCOUNT\n",
        ),
    ] {
        assert_eq!(
            super::sign_in::start(&arguments, &skein_shell::Clock::new()).expect_err("usage refused").line(),
            expected,
            "{row}"
        );
    }
}

#[test]
fn forge_without_its_arguments_prints_its_usage() {
    assert_eq!(
        super::forge::start(&[]).expect_err("usage refused").line(),
        "temper forge prepare: usage: temper forge prepare CONFIG.json FORGE | temper forge run CONFIG.json FORGE forgejo|runner\n"
    );
}
