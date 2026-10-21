#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust for table fixtures and assertions (programming-model.md, 10.2)"
)]
use super::fixtures;
use crate::{
    check, limits, problem,
    setup::{Fetch, Mirror, Mode, Origin, Owned, Setup},
};

fn origin(host: &[u8], port: u16) -> Origin {
    Origin { host: host.into(), port }
}
fn read_owned(owned: &str) -> Result<Owned, Box<[u8]>> {
    let source = core::str::from_utf8(fixtures::TWO).unwrap();
    let start = source.find("{\"forgejo\"").unwrap();
    let end = source.rfind("}}}").unwrap().checked_add(1).unwrap();
    let source = format!("{}{}{}", &source[..start], owned, &source[end..]);
    let mut ps = problem::Problems::empty();
    let value = Setup::read(
        &fixtures::document(source.as_bytes()),
        0,
        &limits::small(),
        &jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap(),
        &mut ps,
    );
    if !ps.is_empty() {
        return Err(problem::lines(&ps));
    }
    match value.unwrap().entries[1].mode.clone() {
        Mode::Owned(owned) => Ok(owned),
        Mode::Managed { .. } | Mode::Fake(_) => panic!("owned"),
    }
}
fn owned_json(forgejo: &str, runner: Option<&str>, mirror: Option<&str>) -> String {
    let runner = runner.map_or("null".into(), |binary| format!(r#"{{"binary":"{binary}","labels":["temper-host:host"],"capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"}}"#));
    let mirror = mirror.map_or(String::new(), |mirror| format!(",\"mirror\":{mirror}"));
    format!(
        r#"{{"forgejo":"{forgejo}","runner":{runner},"listen":{{"socket":"127.0.0.1:3000"}},"person":{{"login":"ada","email":"ada@example.invalid"}}{mirror}}}"#
    )
}
#[test]
fn owned_binaries_and_mirror() {
    let base = fixtures::owned();
    let mirror = r#"{"forgejo":"https://mirror.test","runner":"https://127.0.0.1:8443"}"#;
    for (forgejo, runner, mirrors) in [
        ("fetched", Some("/opt/forgejo/forgejo-runner"), None),
        ("/opt/forgejo/forgejo", Some("fetched"), None),
        ("fetched", Some("fetched"), None),
        ("/opt/forgejo/forgejo", Some("/opt/forgejo/forgejo-runner"), None),
        ("fetched", None, None),
        ("fetched", Some("fetched"), Some(mirror)),
        ("/opt/forgejo/forgejo", None, Some(mirror)),
    ] {
        let mut expected = base.clone();
        expected.fetch = Fetch {
            forgejo: forgejo == "fetched",
            runner: runner == Some("fetched"),
            mirror: mirrors
                .map(|_| Mirror { forgejo: origin(b"mirror.test", 443), runner: origin(b"127.0.0.1", 8443) }),
        };
        expected.forgejo = if expected.fetch.forgejo {
            b"/cache/forgejo/17.0.0/forgejo".as_slice().into()
        } else {
            forgejo.as_bytes().into()
        };
        expected.runner = runner.map(|binary| {
            let mut value = base.runner.clone().unwrap();
            value.binary = if binary == "fetched" {
                b"/cache/forgejo-runner/13.2.0/forgejo-runner".as_slice().into()
            } else {
                binary.as_bytes().into()
            };
            value
        });
        assert_eq!(read_owned(&owned_json(forgejo, runner, mirrors)), Ok(expected));
    }
    for bad in [
        "http://mirror.test",
        "https://mirror.test/",
        "https://mirror.test/releases",
        "https://u@mirror.test",
        "https://mirror.test?a",
        "https://mirror.test#a",
        "https://",
        "https://mirror.test:0",
        "https://mirror.test:65536",
    ] {
        let mirror = format!(r#"{{"forgejo":"{bad}","runner":"https://mirror.test"}}"#);
        assert_eq!(
            read_owned(&owned_json("fetched", None, Some(&mirror))),
            Err(b"application.setup[1].mode.owned.mirror.forgejo: invalid form\n".as_slice().into()),
            "{bad}"
        );
    }
    for (mirror, expected) in [
        (r#"{"forgejo":"https://mirror.test"}"#, "application.setup[1].mode.owned.mirror.runner: missing\n"),
        (
            r#"{"forgejo":"https://mirror.test","runner":"https://mirror.test","third":1}"#,
            "application.setup[1].mode.owned.mirror.third: unknown member\n",
        ),
    ] {
        assert_eq!(read_owned(&owned_json("fetched", None, Some(mirror))), Err(expected.as_bytes().into()));
    }
}
#[test]
fn fetch_bound_through_checked() {
    for (value, expected) in [
        (None, Ok(268_435_456)),
        (Some(1), Ok(1)),
        (Some(u64::MAX), Ok(u64::MAX)),
        (Some(0), Err(b"application.limits.overrides.setup.fetch_bytes: outside its type\n".as_slice())),
    ] {
        let overrides = value.map_or(Vec::new(), |value| vec![(b"setup.fetch_bytes".as_slice().into(), value)]);
        let got = check::checked(
            &fixtures::document(fixtures::TWO),
            0,
            &jig_protocol_config::Directories::working(),
            &fixtures::members(),
            None,
            &overrides,
        );
        match expected {
            Ok(expected) => assert_eq!(got.unwrap().fetch_bytes, expected),
            Err(expected) => assert_eq!(problem::lines(&got.unwrap_err()).as_ref(), expected),
        }
    }
}
