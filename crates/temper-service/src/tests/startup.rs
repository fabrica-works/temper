#![expect(clippy::disallowed_methods, reason = "ordinary test code edits UTF-8 configuration fixtures")]
use super::fixtures;
use crate::startup::{Problems, What, application, lines};
use jig_protocol_config::{self as config, Path, Segment};

const REFUSED_TEMPLATE: u32 = 7;

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
    assert_eq!(members.forges[0].user, application::User::File(b"secrets/forge-1.user".as_slice().into()));
    assert_eq!(members.forges[0].git.as_ref(), b"http://127.0.0.1:9");
    assert_eq!(members.identities[0].author.name.as_ref(), b"Temper");
    assert_eq!(members.workers[0].git.as_ref(), b"/usr/bin/git");
    assert_eq!(members.charter, 1);
    assert_eq!(application::files(&members, 4096, 8192).len(), 3, "all private files");
    let resolved = application::resolve(&members, &fixtures::files(), &mut Problems::empty()).unwrap();
    assert_eq!(resolved.forges[0].api_token.expose(), b"api-token");
    assert_eq!(resolved.identities[0].token.expose(), b"git-token");
    assert!(!fixtures::document().is_empty(), "complete example bytes");
    let document = fixtures::parsed();
    assert!(!document.is_empty(), "full standalone example is JSON");
}
#[test]
fn the_example_engine_keeps_jigs_chat_and_goal_templates() {
    let jig = fixtures::engine().jig;
    let labels: Box<[u32]> = jig.templates.iter().map(|template| template.label).collect();
    assert_eq!(*labels, [1, 2], "jig's chat and goal templates, in order");
    assert_eq!(jig.templates[0].number, 1);
}
#[test]
fn a_chats_member_is_refused_as_unknown_naming_its_path() {
    let input = fixtures::document();
    let input = core::str::from_utf8(&input).unwrap().replacen("\"templates\": [", "\"chats\": [", 1);
    let problems = crate::named(input.as_bytes()).unwrap_err();
    assert_eq!(&*lines(&problems), b"engine.templates: missing\nengine.chats: unknown member\n");
}
#[test]
fn an_unknown_missing_or_reserved_member_is_named_by_its_path() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    for (from, to, expected) in [
        (
            "\"user\":{\"file\":\"secrets/forge-1.user\"}",
            "\"usr\":42",
            "application.forges[0].user: missing\napplication.forges[0].usr: unknown member\n",
        ),
        (",\"user\":{\"file\":\"secrets/forge-1.user\"}", "", "application.forges[0].user: missing\n"),
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
        (config::What::Grace { template: REFUSED_TEMPLATE }, b"at: grace does not fit template 7\n"),
        (
            config::What::Operations { operations: 4, delegates: 4, goals: 8 },
            b"at: operations do not fit compound requests: operations 4, delegates 4, goals 8\n",
        ),
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
        ("\"user\":{\"file\":\"secrets/forge-1.user\"}", "\"user\":0", "application.forges[0].user: invalid form\n"),
        (
            "\"user\":{\"file\":\"secrets/forge-1.user\"}",
            "\"user\":{\"file\":\"secrets/forge-1.user\"},\"user\":43",
            "application.forges[0].user: duplicate\n",
        ),
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
        (1, b"secrets/forge-1.user".as_slice(), 21, true),
        (2, b"root.der".as_slice(), 1_048_576, false),
        (3, b"secrets/forge-1.git-token".as_slice(), 8192, true),
    ] {
        assert_eq!(files[index].path.as_ref(), path, "file {index}");
        assert_eq!(files[index].max, max, "bound {index}");
        assert_eq!(files[index].private, private, "privacy {index}");
    }
    let mut problems = Problems::empty();
    assert!(application::resolve(&members, &fixtures::files(), &mut problems).is_none(), "trust file must be read");
    assert_eq!(&*lines(&problems), b"application.forges[0].transport.tls.trust: not read\n".as_slice());
}
#[test]
fn named_lists_each_file_with_its_bound_and_privacy() {
    let named = crate::named(&fixtures::document()).unwrap();
    assert_eq!(named.files.len(), 8, "all files listed");
    for (name, path, max, private) in [
        (b"charters/chat.txt".as_slice(), b"engine.charters[0].instructions".as_slice(), 16_219, false),
        (b"secrets/worker-a.digest", b"engine.hosts.workers[0].digest", 65, true),
        (b"accounts/chat.json", b"engine.llm.accounts[0].source.handed_in.file", 65_536, true),
        (b"worker-secret", b"workers[0].secret", 257, true),
        (fixtures::AGENT_JSON, b"workers[0].agent.configuration", 1_048_576, false),
        (b"secrets/forge-1.api-token", b"application.forges[0].api_token", 4096, true),
        (b"secrets/forge-1.git-token", b"application.identities[0].token", 8192, true),
    ] {
        let file = named.files.iter().find(|file| file.path.as_ref() == name).expect("listed file");
        assert_eq!(file.max, max, "bound {name:?}");
        assert_eq!(file.private, private, "privacy {name:?}");
        let mut problems = Problems::empty();
        problems.push(file.at.clone(), What::Jig(config::What::Missing));
        let expected = [path, b": missing\n"].concat();
        assert_eq!(&*lines(&problems), expected.as_slice(), "path {name:?}");
    }
}
#[test]
fn named_returns_the_worker_root_and_the_agent_configuration() {
    let named = crate::named(&fixtures::document()).unwrap();
    assert_eq!(named.deployment.as_ref(), b"state");
    assert_eq!(named.worker_root.as_ref(), b"worker-root");
    assert_eq!(named.agent.as_ref(), fixtures::AGENT_JSON);
}
#[test]
fn a_file_named_but_not_given_refuses_the_start_naming_it() {
    let mut files = fixtures::files();
    files.entries = fixtures::without(files.entries, b"secrets/forge-1.git-token");
    let refused = crate::startup(
        &fixtures::document(),
        &files,
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    );
    let Err(problems) = refused else { panic!("a missing token refuses the start") };
    assert_eq!(&*lines(&problems), b"application.identities[0].token: not read\n".as_slice());
}
#[test]
fn jigs_problems_keep_their_paths_under_engine() {
    let input = fixtures::document();
    let input = core::str::from_utf8(&input)
        .unwrap()
        .replace("\"memory_bytes\": 1152921504606846976,", "\"memory_bytes\": \"bad\",");
    let problems = crate::named(input.as_bytes()).unwrap_err();
    assert_eq!(&*lines(&problems), b"engine.memory_bytes: wrong value kind\n".as_slice());
}
#[test]
fn startup_makes_the_one_process_from_the_example() {
    let mut service = match crate::startup(
        &fixtures::document(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    ) {
        Ok(service) => service,
        Err(problems) => panic!("startup: {}", core::str::from_utf8(&lines(&problems)).unwrap()),
    };
    service.engine.store = fixtures::memory_store();
    let mut stand = fixtures::system::SystemStand { service, kernel: fixtures::Kernel::new() };
    fixtures::until_hosting(&mut stand);
    let (channel, hello) = fixtures::hello(&stand.service).expect("worker linked through bytes");
    assert_eq!(channel, skein_lib::Token::new(1));
    assert_eq!(hello.slots, 1);
    for _ in 0_u16..256 {
        if stand.service.engine.domain.ready() {
            break;
        }
        stand.turn();
    }
    assert!(stand.service.engine.domain.ready(), "engine restored its store");
    assert_eq!(stand.service.end(), None);
}
#[test]
fn a_workers_missing_or_oversized_secret_keeps_its_absolute_path() {
    let document = fixtures::document();
    let agent = fixtures::startup_agent();
    let seeds = fixtures::seeds();
    for (row, bytes, expected) in [
        ("missing", None, b"workers[0].secret: not read\n".as_slice()),
        ("past", Some(Box::from([b'x'; 257])), b"workers[0].secret: past its bound\n"),
    ] {
        let mut files = fixtures::files();
        files.entries = fixtures::without(files.entries, b"worker-secret");
        if let Some(bytes) = bytes {
            files.entries.push((b"worker-secret".as_slice().into(), skein_lib::Secret::new(bytes))).unwrap();
        }
        let found = match crate::startup(
            &document,
            &files,
            &agent,
            &seeds,
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        ) {
            Ok(service) => {
                drop(service);
                Problems::empty()
            }
            Err(problems) => problems,
        };
        assert_eq!(&*lines(&found), expected, "row {row}");
    }
}
#[test]
fn a_worker_secret_at_its_bound_excludes_its_final_newline() {
    let mut worker = fixtures::worker_settings();
    worker.jig.limits.link.secret = 4;
    let mut entries = skein_lib::List::with_capacity(1);
    entries.push((b"worker-secret".as_slice().into(), skein_lib::Secret::new(b"four\n".as_slice().into()))).unwrap();
    let made = crate::Worker::new(
        &worker,
        &fixtures::worker_agent(),
        &fixtures::worker_limits(),
        &crate::settings::Files { entries },
        fixtures::worker_runtime(),
        7,
    );
    assert!(made.is_ok(), "four secret bytes plus their final newline fit four");
}

#[test]
fn a_user_file_member_reads_as_a_private_file_of_21_bytes() {
    let (members, problems) = application(fixtures::APPLICATION, &fixtures::engine().jig);
    assert_eq!(&*lines(&problems), b"");
    let files = application::files(&members.unwrap(), 4096, 8192);
    let file = files.iter().find(|file| file.path.as_ref() == b"secrets/forge-1.user").unwrap();
    assert_eq!(file.max, 21);
    assert!(file.private);
    let mut problems = Problems::empty();
    problems.push(file.at.clone(), What::Jig(config::What::Missing));
    assert_eq!(&*lines(&problems), b"application.forges[0].user.file: missing\n");
}
#[test]
fn a_user_file_resolves_to_the_decimal_id_it_holds() {
    let (members, _) = application(fixtures::APPLICATION, &fixtures::engine().jig);
    let mut problems = Problems::empty();
    let resolved = application::resolve(&members.unwrap(), &fixtures::files(), &mut problems).unwrap();
    assert_eq!(resolved.forges[0].user, 42);
    assert_eq!(&*lines(&problems), b"");
}
#[test]
fn a_user_file_holding_no_positive_decimal_is_refused_by_path() {
    let (members, _) = application(fixtures::APPLICATION, &fixtures::engine().jig);
    let members = members.unwrap();
    for value in
        [b"".as_slice(), b"0", b"042", b"-1", b"4 2", b"42\n\n", b"123456789012345678901", b"18446744073709551616"]
    {
        let mut files = fixtures::files();
        files.entries = fixtures::without(files.entries, b"secrets/forge-1.user");
        files.entries.push((b"secrets/forge-1.user".as_slice().into(), skein_lib::Secret::new(value.into()))).unwrap();
        let mut problems = Problems::empty();
        assert!(application::resolve(&members, &files, &mut problems).is_none(), "{value:?}");
        assert_eq!(&*lines(&problems), b"application.forges[0].user.file: invalid form\n", "{value:?}");
    }
}
fn refused_user(value: &str) -> Box<[u8]> {
    let input =
        core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("{\"file\":\"secrets/forge-1.user\"}", value);
    let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
    assert!(members.is_none());
    lines(&problems)
}
#[test]
fn a_user_object_with_a_missing_file_is_refused() {
    assert_eq!(&*refused_user("{}"), b"application.forges[0].user.file: missing\n");
}
#[test]
fn a_user_object_with_a_stray_member_is_refused() {
    assert_eq!(&*refused_user("{\"file\":\"x\",\"stray\":1}"), b"application.forges[0].user.stray: unknown member\n");
}
#[test]
fn a_user_object_with_an_empty_path_is_refused() {
    assert_eq!(&*refused_user("{\"file\":\"\"}"), b"application.forges[0].user.file: invalid form\n");
}
#[test]
fn a_missing_setup_is_refused_naming_its_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"setup\":[],", "");
    let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
    assert!(members.is_none());
    assert_eq!(&*lines(&problems), b"application.setup: missing\n");
}
#[test]
fn a_setup_or_box_override_is_handed_on_with_its_value() {
    let input = core::str::from_utf8(fixtures::APPLICATION)
        .unwrap()
        .replace("\"overrides\":{}", "\"overrides\":{\"setup.orgs\":2,\"box.steps\":3}");
    let (members, problems) = application(input.as_bytes(), &fixtures::engine().jig);
    assert_eq!(&*lines(&problems), b"");
    assert_eq!(
        &*members.unwrap().overrides,
        &[(b"setup.orgs".as_slice().into(), 2), (b"box.steps".as_slice().into(), 3)]
    );
}
