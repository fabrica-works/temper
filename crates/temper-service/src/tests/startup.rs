#![expect(clippy::disallowed_methods, reason = "ordinary test code edits UTF-8 configuration fixtures")]
use super::fixtures;
use crate::startup::{Problems, What, application, lines};
use jig_protocol_config::{self as config, Path, Segment};

fn application(input: &[u8], jig: &config::Engine) -> (Option<application::Members>, Problems) {
    let document =
        config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&document, 0, jig, &mut problems);
    (members, problems)
}
#[test]
fn the_standalone_example_reads_into_its_members() {
    let jig = fixtures::engine().jig;
    let (members, problems) = application(fixtures::APPLICATION, &jig);
    assert_eq!(&*lines(&problems), b"", "example accepted");
    let members = members.unwrap();
    assert_eq!(members.forges[0].user, 42);
    assert_eq!(members.forges[0].git.as_ref(), b"http://127.0.0.1:9");
    assert_eq!(members.identities[0].author.name.as_ref(), b"Temper");
    assert_eq!(members.workers[0].git.as_ref(), b"/usr/bin/git");
    assert_eq!(members.charter, 1);
    assert_eq!(application::files(&members, 4096, 8192).len(), 2, "both private files");
    let resolved = application::resolve(&members, &fixtures::files(), &mut Problems::empty()).unwrap();
    assert_eq!(resolved.forges[0].api_token.expose(), b"api-token");
    assert_eq!(resolved.identities[0].token.expose(), b"git-token");
    assert!(!fixtures::document().is_empty(), "complete example bytes");
    let document = fixtures::parsed();
    assert!(!document.is_empty(), "full standalone example is JSON");
}
#[test]
fn an_unknown_missing_or_reserved_member_is_named_by_its_path() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    for (from, to, expected) in [
        (
            "\"user\":42",
            "\"usr\":42",
            "application.forges[0].user: missing\napplication.forges[0].usr: unknown member\n",
        ),
        (",\"user\":42", "", "application.forges[0].user: missing\n"),
        ("\"charter\":1", "\"webhooks\":null,\"charter\":1", "application.webhooks: reserved: not built yet\n"),
        ("\"small\"", "\"large\"", "application.limits.profile: the only profile is \"small\"\n"),
    ] {
        let (members, problems) = application(input.replace(from, to).as_bytes(), &jig);
        assert!(members.is_none(), "refused {from}");
        assert_eq!(&*lines(&problems), expected.as_bytes(), "row {from}");
    }
}
#[test]
fn an_override_by_dotted_name_is_set_or_refused() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    for (override_value, expected) in [
        ("\"overrides\":{\"forge.repositories\":3}", ""),
        (
            "\"overrides\":{\"forge.nothing\":3}",
            "application.limits.overrides.forge.nothing: no limit of temper's has this name\n",
        ),
        (
            "\"overrides\":{\"checkout.name_bytes\":4294967296}",
            "application.limits.overrides.checkout.name_bytes: outside its type\n",
        ),
    ] {
        let (members, problems) = application(input.replace("\"overrides\":{}", override_value).as_bytes(), &jig);
        assert_eq!(&*lines(&problems), expected.as_bytes(), "row {override_value}");
        if let Some(members) = members {
            assert_eq!(members.limits.forge.repositories, 3, "set {override_value}");
        }
    }
}
#[test]
fn a_landing_rule_reads_jigs_pattern_form() {
    let jig = fixtures::engine().jig;
    let (members, problems) = application(fixtures::APPLICATION, &jig);
    assert_eq!(&*lines(&problems), b"", "landing accepted");
    let members = members.unwrap();
    let rule = &members.landing.projects.get(&1).unwrap()[0];
    assert_eq!(rule.pattern.segments[0].as_ref(), b"forge");
    assert_eq!(rule.pattern.last, jig_domain_authority::Last::Open(b"main".as_slice().into()));
    assert_eq!(rule.gates[0].number, 3);
    assert_eq!(rule.approvals[0].role, 0);
    assert_eq!(rule.approvals[0].freshness, temper_engine_domain::Freshness::Clean);
}
#[test]
fn a_git_base_with_a_user_or_a_final_slash_is_refused() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    for (git, expected) in [
        ("http://user@127.0.0.1:9", "application.forges[0].git: invalid form\n"),
        ("http://127.0.0.1:9/", "application.forges[0].git: invalid form\n"),
        ("http://forge.example", "application.forges[0].git: invalid form\n"),
        ("https://forge.example?token=1", "application.forges[0].git: invalid form\n"),
    ] {
        let (members, problems) = application(input.replace("http://127.0.0.1:9", git).as_bytes(), &jig);
        assert!(members.is_none(), "refused {git}");
        assert_eq!(&*lines(&problems), expected.as_bytes(), "row {git}");
    }
}
#[test]
fn problems_print_one_line_each_with_their_path() {
    let mut problems = Problems::empty();
    let forge = Path::root().member(b"application").member(b"forges").child(Segment::Index(0));
    problems.push(forge.member(b"user"), What::Jig(config::What::Missing));
    problems.push(Path::root().member(b"application").member(b"webhooks"), What::NotBuilt);
    problems
        .push(Path::root().member(b"engine").member(b"memory_bytes"), What::Jig(config::What::Memory { over: 4096 }));
    assert_eq!(&*lines(&problems), b"application.forges[0].user: missing\napplication.webhooks: reserved: not built yet\nengine.memory_bytes: the worst case passes memory_bytes by 4096 bytes\n".as_slice());
    for _ in 0..config::PROBLEMS {
        problems.push(Path::root(), What::NotBuilt);
    }
    assert_eq!(problems.past, 3);
    assert!(lines(&problems).ends_with(b"and 3 more\n"), "capped diagnostics name overflow");
}
#[test]
fn jig_refusals_print_words() {
    for (what, expected) in [
        (config::What::Json, b"at: invalid JSON\n".as_slice()),
        (config::What::Unknown, b"at: unknown member\n"),
        (config::What::Missing, b"at: missing\n"),
        (config::What::Duplicate, b"at: duplicate\n"),
        (config::What::Kind, b"at: wrong value kind\n"),
        (config::What::Bound, b"at: past its bound\n"),
        (config::What::Form, b"at: invalid form\n"),
        (config::What::Unique, b"at: must be unique\n"),
        (config::What::Refers, b"at: unknown reference\n"),
        (config::What::Plaintext, b"at: plaintext requires loopback or in-process\n"),
        (config::What::Digest, b"at: invalid digest\n"),
        (config::What::HandedIn, b"at: invalid handed-in account\n"),
        (config::What::Reserve { charter: 7 }, b"at: reserve does not fit charter 7\n"),
        (config::What::Budget { charter: 7 }, b"at: budget does not fit charter 7\n"),
        (config::What::Accounts { charter: 7 }, b"at: accounts do not fit charter 7\n"),
        (config::What::Message, b"at: message bounds do not agree\n"),
        (config::What::Silence, b"at: heartbeat does not fit silence bound\n"),
        (config::What::StopBound, b"at: stop bound does not fit grace\n"),
        (config::What::Memory { over: 7 }, b"at: the worst case passes memory_bytes by 7 bytes\n"),
        (config::What::Override, b"at: invalid override\n"),
        (config::What::Unreadable, b"at: not read\n"),
    ] {
        let mut problems = Problems::empty();
        problems.push(Path::root().member(b"at"), What::Jig(what));
        assert_eq!(&*lines(&problems), expected, "row {what:?}");
    }
}
#[test]
fn malformed_members_and_references_are_named() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    for (from, to, expected) in [
        ("\"charter\":1", "\"charter\":2", "application.charter: unknown reference\n"),
        ("\"kind\":4", "\"kind\":3", "application.landing.projects[0].rules[0].kind: invalid form\n"),
        ("\"role\":0", "\"role\":1", "application.landing.projects[0].rules[0].approvals[0].role: unknown reference\n"),
        ("\"project\":1", "\"project\":2", "application.landing.projects[0].project: unknown reference\n"),
        ("\"login\":\"temper\"", "\"login\":\"bad:login\"", "application.identities[0].login: invalid form\n"),
        ("\"name\":\"Temper\"", "\"name\":\"<Temper>\"", "application.identities[0].author.name: invalid form\n"),
        ("\"user\":42", "\"user\":0", "application.forges[0].user: invalid form\n"),
        ("\"user\":42", "\"user\":42,\"user\":43", "application.forges[0].user: duplicate\n"),
        ("\"git\":\"/usr/bin/git\"", "\"git\":\"git\"", "application.workers[0].git: must be an absolute path\n"),
        (
            "\"transport\":\"plaintext\"",
            "\"transport\":{\"tls\":{\"server_name\":\"forge\"}}",
            "application.forges[0].transport.tls.trust: missing\n",
        ),
    ] {
        let (members, problems) = application(input.replace(from, to).as_bytes(), &jig);
        assert!(members.is_none(), "refused {from}");
        assert_eq!(&*lines(&problems), expected.as_bytes(), "row {from}");
    }
}
#[test]
fn application_files_keep_paths_bounds_privacy_and_resolution_refusals() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace(
        "\"transport\":\"plaintext\"",
        "\"transport\":{\"tls\":{\"server_name\":\"forge\",\"trust\":\"root.der\"}}",
    );
    let (members, problems) = application(input.as_bytes(), &jig);
    assert_eq!(&*lines(&problems), b"", "TLS file form accepted");
    let members = members.unwrap();
    let files = application::files(&members, 4096, 8192);
    for (index, path, max, private) in [
        (0, b"secrets/forge-1.api-token".as_slice(), 4096, true),
        (1, b"root.der".as_slice(), 1_048_576, false),
        (2, b"secrets/forge-1.git-token".as_slice(), 8192, true),
    ] {
        assert_eq!(files[index].path.as_ref(), path, "file {index}");
        assert_eq!(files[index].max, max, "bound {index}");
        assert_eq!(files[index].private, private, "privacy {index}");
    }
    let mut problems = Problems::empty();
    assert!(application::resolve(&members, &fixtures::files(), &mut problems).is_none(), "trust file must be read");
    assert_eq!(&*lines(&problems), b"application.forges[0].transport.tls.trust: not read\n".as_slice());
}
