#![expect(clippy::disallowed_types, reason = "ordinary tests build configuration documents")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests construct and inspect protocol fixtures"
)]
use super::fixtures;
use crate::startup::{self, Problems, application};
use jig_protocol_config as config;
const SIGN_IN: &str = r#"{"google":{"provider":3,"client":"temper-test","secret":"google.secret","endpoints":{"issuer":"http://127.0.0.1:31999","authorization":"http://127.0.0.1:31999/auth","token":"http://127.0.0.1:31999/token","keys":"http://127.0.0.1:31999/keys","transport":"plaintext"}},"members":[{"address":"Sam@example.com","login":"sam"}]}"#;
fn jig() -> config::Engine {
    let mut jig = fixtures::engine().jig;
    let mut providers = skein_lib::List::with_capacity(3);
    for provider in jig.providers {
        providers.push(provider).unwrap();
    }
    providers.push(config::Provider { number: 3, kind: config::ProviderKind::Application }).unwrap();
    jig.providers = providers.into_boxed();
    jig.owners[0].identity =
        jig_domain_people::IdentityKey { provider: 3, subject: b"Sam@example.com".as_slice().into() };
    jig
}
fn parse(input: &str, jig: &config::Engine) -> (Option<application::Members>, Problems) {
    let document =
        config::document(input.as_bytes(), &config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&document, 0, jig, &config::Directories::working(), &mut problems);
    (members, problems)
}
fn input(sign_in: &str) -> String {
    core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"sign_in\":null", &format!("\"sign_in\":{sign_in}"))
}
#[test]
fn a_google_registration_reads_its_members_and_private_file() {
    let (members, problems) = parse(&input(SIGN_IN), &jig());
    assert!(problems.is_empty(), "{:?}", startup::lines(&problems));
    let members = members.unwrap();
    let sign_in = members.sign_in.as_ref().unwrap();
    assert_eq!(sign_in.provider, 3);
    assert_eq!(sign_in.members[0].address.as_ref(), b"Sam@example.com");
    let files = application::files(&members, 4096, 4096);
    let secret = files.iter().find(|file| file.path.as_ref() == b"google.secret").unwrap();
    assert!(secret.private);
    assert_eq!(secret.max, 512);
}
#[test]
fn google_configuration_refusals_name_the_member() {
    for (from, to, expected) in [
        ("\"client\":\"temper-test\"", "\"client\":\"\"", "google.client"),
        ("\"client\":\"temper-test\"", "\"client\":\"has space\"", "google.client"),
        ("Sam@example.com", "not-an-address", "members[0].address"),
        ("Sam@example.com", "@example.com", "members[0].address"),
        ("\"login\":\"sam\"", "\"login\":\"-sam\"", "members[0].login"),
        ("\"login\":\"sam\"", "\"login\":\"sam_\"", "members[0].login"),
        ("\"login\":\"sam\"", "\"login\":\"s/am\"", "members[0].login"),
        ("\"provider\":3", "\"provider\":1", "google.provider"),
        ("\"provider\":3", "\"provider\":99", "google.provider"),
        ("http://127.0.0.1:31999/token", "http://example.com/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "https://user@127.0.0.1/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "http://127.0.0.1:0/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "http://127.0.0.1:+31999/token", "google.endpoints.token"),
        ("http://127.0.0.1:31999/token", "http://127.0.0.1:31999/token#fragment", "google.endpoints.token"),
        ("\"login\":\"sam\"}", "\"login\":\"sam\",\"unknown\":1}", "members[0].unknown"),
        ("\"members\":[{\"address\":\"Sam@example.com\",\"login\":\"sam\"}]", "\"members\":[]", "members"),
        (
            "\"members\":[{\"address\":\"Sam@example.com\",\"login\":\"sam\"}]",
            "\"members\":[{\"address\":\"Sam@example.com\",\"login\":\"sam\"},{\"address\":\"SAM@example.com\",\"login\":\"SAM\"}]",
            "members[1].address",
        ),
    ] {
        let (members, problems) = parse(&input(&SIGN_IN.replace(from, to)), &jig());
        assert!(members.is_none(), "{expected}");
        let lines = startup::lines(&problems);
        assert!(
            core::str::from_utf8(&lines).unwrap().contains(expected),
            "{expected}: {:?}",
            String::from_utf8_lossy(&lines)
        );
    }
}
#[test]
fn owners_must_be_members_and_at_least_one_google_owner_is_required() {
    let mut jig = jig();
    jig.owners[0].identity.subject = b"another@example.com".as_slice().into();
    let (_, problems) = parse(&input(SIGN_IN), &jig);
    assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("engine.owners[0]"));
    jig.owners[0].identity.provider = 1;
    let (_, problems) = parse(&input(SIGN_IN), &jig);
    assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("engine.owners:"));
}
#[test]
fn sign_in_is_required_and_each_zero_limit_is_refused() {
    let missing = input("null").replace("\"sign_in\":null,", "");
    let (_, problems) = parse(&missing, &jig());
    assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("application.sign_in: missing"));
    for name in ["members", "pending", "deadline_ms", "fetches", "body_bytes", "url_bytes", "secret_bytes"] {
        let mut limits = crate::limits::profile(crate::limits::Profile::Small);
        crate::limits::apply(&mut limits, format!("sign_in.{name}").as_bytes(), 0).unwrap();
        let mut problems = Problems::empty();
        startup::checks::limits(&limits, &limits.git_component, &mut problems);
        assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains(&format!("sign_in.{name}")));
    }
}
#[test]
fn filling_the_bundle_sets_the_ways_and_changes_its_etag() {
    for google in [false, true] {
        let mut bundle = fixtures::bundle::bundle();
        assert!(bundle.fill_sign_in(google).is_none(), "missing head refused");
        bundle.page = b"<!doctype html><html><head><title>t</title></head><body></body></html>".as_slice().into();
        let before = bundle.etag.clone();
        bundle.fill_sign_in(google).unwrap();
        let meta = if google {
            b"<meta name=\"temper-sign-in\" content=\"google\"></head>".as_slice()
        } else {
            b"<meta name=\"temper-sign-in\" content=\"\"></head>".as_slice()
        };
        assert!(bundle.page.windows(meta.len()).any(|part| part == meta));
        assert_ne!(bundle.etag, before);
        let hash = bundle
            .page
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3));
        assert_eq!(bundle.etag.as_ref(), format!("\"{hash:016x}\"").as_bytes());
    }
}
#[test]
fn every_members_fallback_identity_fits_both_jig_consumers() {
    for clients in [false, true] {
        let mut jig = jig();
        if clients {
            jig.limits.clients.identity_bytes = 5;
        } else {
            jig.limits.core.people.identity_bytes = 5;
        }
        let (members, problems) = parse(&input(SIGN_IN), &jig);
        assert!(members.is_none());
        assert!(core::str::from_utf8(&startup::lines(&problems)).unwrap().contains("application.sign_in.members[0]:"));
    }
}
