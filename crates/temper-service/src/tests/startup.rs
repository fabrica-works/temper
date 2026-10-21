// Fixture parsing without a deployment keeps written paths.
#![expect(clippy::disallowed_methods, reason = "ordinary test code edits UTF-8 configuration fixtures")]
use super::fixtures;
use crate::settings;
use crate::startup::{Problems, What, application, lines};
use jig_protocol_config::{self as config, Path, Segment};
use skein_lib::List;

const REFUSED_TEMPLATE: u32 = 7;

#[test]
fn the_clients_listener_is_read_from_the_application_member() {
    let jig = fixtures::engine().jig;
    let (members, problems) = application(fixtures::APPLICATION, &jig);
    assert_eq!(&*lines(&problems), b"", "example accepted");
    let config::Address::Addr(skein_io::Addr::Ip(listener)) = members.unwrap().clients.listener else {
        panic!("a socket address")
    };
    assert_eq!(listener, "127.0.0.1:7100".parse::<core::net::SocketAddr>().unwrap());
}
#[test]
fn a_missing_clients_member_is_named_by_its_path() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    let input = input.replace("\"clients\":{\"listener\":{\"socket\":\"127.0.0.1:7100\"},\"bundle\":null},", "");
    let (members, problems) = application(input.as_bytes(), &jig);
    assert!(members.is_none());
    assert_eq!(&*lines(&problems), b"application.clients: missing\n");
}
#[test]
fn an_in_process_clients_listener_is_refused_as_a_form() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    let input = input.replace("\"socket\":\"127.0.0.1:7100\"", "\"in_process\":\"clients\"");
    let (members, problems) = application(input.as_bytes(), &jig);
    assert!(members.is_none());
    assert_eq!(&*lines(&problems), b"application.clients.listener: invalid form\n");
}

fn application(input: &[u8], jig: &config::Engine) -> (Option<application::Members>, Problems) {
    let document =
        config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let members = application::parse(&document, 0, jig, &config::Directories::working(), &mut problems);
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
    assert_eq!(members.workers[0].git.as_deref(), Some(b"/usr/bin/git".as_slice()));
    assert_eq!(members.charter, 1);
    assert_eq!(application::files(&members, 4096, 8192).len(), 3, "all private files");
    let resolved = application::resolve(&members, &fixtures::files(), &mut Problems::empty()).unwrap();
    assert_eq!(resolved.forges[0].api_token.as_ref().unwrap().expose(), b"api-token");
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
    let problems = crate::named(input.as_bytes(), &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*lines(&problems), b"engine.templates: missing\nengine.chats: unknown member\n");
}
fn refused(input: &[u8]) -> Box<[u8]> {
    let (members, problems) = application(input, &fixtures::engine().jig);
    assert!(members.is_none());
    lines(&problems)
}
#[test]
fn an_unknown_user_member_is_named_by_its_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION)
        .unwrap()
        .replace("\"user\":{\"file\":\"secrets/forge-1.user\"}", "\"usr\":42");
    assert_eq!(
        &*refused(input.as_bytes()),
        b"application.forges[0].user: missing\napplication.forges[0].usr: unknown member\n"
    );
}
#[test]
fn a_missing_user_member_is_named_by_its_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION)
        .unwrap()
        .replace(",\"user\":{\"file\":\"secrets/forge-1.user\"}", "");
    assert_eq!(&*refused(input.as_bytes()), b"application.forges[0].user: missing\n");
}
#[test]
fn an_unknown_profile_is_refused_naming_its_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"small\"", "\"large\"");
    assert_eq!(&*refused(input.as_bytes()), b"application.limits.profile: the only profile is \"small\"\n");
}
#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary tests format input and diagnostics")]
fn an_override_by_dotted_name_is_set_or_refused() {
    let jig = fixtures::engine().jig;
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap();
    for (override_value, expected) in [
        (
            "\"overrides\":{\"forge.repositories\":3,\"forges.log_origins\":3,\"forge_component.log_origins\":7,\"forge_component.log_poll_ms\":7123,\"forge_component.log_catch_up_ms\":9321,\"forge_component.log_pages\":13,\"forge_component.delivery_bytes\":12345}",
            "",
        ),
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
            assert_eq!((members.limits.forges.log_origins, members.limits.forge_component.log_origins), (3, 7));
            assert_eq!(members.limits.forge_component.log_poll, skein_lib::Duration::from_millis(7123));
            assert_eq!(members.limits.forge_component.log_catch_up, skein_lib::Duration::from_millis(9321));
            assert_eq!(members.limits.forge_component.log_pages, 13);
            assert_eq!(members.limits.forge_component.delivery_bytes, 12_345);
        }
    }
    for (name, value) in [
        ("forges.log_origins", u64::from(u32::MAX) + 1),
        ("forge_component.log_origins", u64::from(u32::MAX) + 1),
        ("forge_component.log_poll_ms", u64::MAX / 1_000_000 + 1),
        ("forge_component.log_catch_up_ms", u64::MAX / 1_000_000 + 1),
        ("forge_component.log_pages", u64::from(u32::MAX) + 1),
        ("forge_component.delivery_bytes", u64::from(u32::MAX) + 1),
    ] {
        let override_value = format!("\"overrides\":{{\"{name}\":{value}}}");
        let (_, problems) = application(input.replace("\"overrides\":{}", &override_value).as_bytes(), &jig);
        assert_eq!(
            lines(&problems).as_ref(),
            format!("application.limits.overrides.{name}: outside its type\n").as_bytes()
        );
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
    problems.push(Path::root().member(b"application").member(b"webhooks"), What::NoWebhooks);
    problems
        .push(Path::root().member(b"engine").member(b"memory_bytes"), What::Jig(config::What::Memory { over: 4096 }));
    assert_eq!(&*lines(&problems), b"application.forges[0].user: missing\napplication.webhooks: needs application.webhooks\nengine.memory_bytes: the worst case passes memory_bytes by 4096 bytes\n".as_slice());
    for _ in 0..config::PROBLEMS {
        problems.push(Path::root(), What::NoWebhooks);
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
        (config::What::Relative, b"at: relative path must be non-empty with no .. component\n"),
    ] {
        let mut problems = Problems::empty();
        problems.push(Path::root().member(b"at"), What::Jig(what));
        assert_eq!(&*lines(&problems), expected, "row {what:?}");
    }
}
#[test]
fn members_and_references_are_checked() {
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
        ("\"git\":\"/usr/bin/git\"", "\"git\":null", ""),
        (
            "\"transport\":\"plaintext\"",
            "\"transport\":{\"tls\":{\"server_name\":\"forge\"}}",
            "application.forges[0].transport.tls.trust: missing\n",
        ),
    ] {
        let (members, problems) = application(input.replace(from, to).as_bytes(), &jig);
        assert_eq!(members.is_some(), expected.is_empty(), "row {from}: {to}");
        assert_eq!(&*lines(&problems), expected.as_bytes(), "row {from}");
        if expected.is_empty() {
            assert_eq!(members.unwrap().workers[0].git, None);
        }
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
    let named = crate::named(&fixtures::document(), &jig_protocol_config::Directories::working()).unwrap();
    assert_eq!(named.files.len(), 7, "all files listed");
    // The optional program carriers add 16 B to both RunPolicy and its model.
    // With the same run budget, this direct charter has 32 B less instruction room.
    let instruction_bytes = 16_070 - 2 * 16;
    for (name, path, max, private) in [
        (b"charters/chat.txt".as_slice(), b"engine.charters[0].instructions".as_slice(), instruction_bytes, false),
        (b"secrets/worker-a.digest", b"engine.hosts.workers[0].digest", 65, true),
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
    let named = crate::named(&fixtures::document(), &jig_protocol_config::Directories::working()).unwrap();
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
        &jig_protocol_config::Directories::working(),
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
    let problems = crate::named(input.as_bytes(), &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*lines(&problems), b"engine.memory_bytes: wrong value kind\n".as_slice());
}
#[test]
fn startup_makes_the_one_process_from_the_example() {
    let mut service = match crate::startup(
        &fixtures::document(),
        &jig_protocol_config::Directories::working(),
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
            &jig_protocol_config::Directories::working(),
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
    let mut entries = List::with_capacity(1);
    entries.push((b"worker-secret".as_slice().into(), skein_lib::Secret::new(b"four\n".as_slice().into()))).unwrap();
    let made = crate::Worker::new(
        &worker,
        &fixtures::worker_agent(),
        &fixtures::worker_limits(),
        &settings::Files { entries },
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

#[test]
fn the_webhook_listener_and_secret_file_are_read() {
    let named = crate::named(&fixtures::webhook_document(), &jig_protocol_config::Directories::working()).unwrap();
    let webhooks = named.webhooks.unwrap();
    let config::Address::Addr(listener) = webhooks.listener else { panic!("socket") };
    assert_eq!(listener.socket().unwrap(), "127.0.0.1:7200".parse().unwrap());
    let file = named.files.iter().find(|file| file.path.as_ref() == b"secrets/forge-1.webhook-secret").unwrap();
    assert_eq!(file.max, 128);
    assert!(file.private);
    let mut problems = Problems::empty();
    problems.push(file.at.clone(), What::Jig(config::What::Missing));
    assert_eq!(&*lines(&problems), b"application.forges[0].webhook_secret: missing\n");
}
#[test]
fn null_webhook_members_are_read_as_none() {
    let (members, problems) = application(fixtures::APPLICATION, &fixtures::engine().jig);
    assert_eq!(&*lines(&problems), b"");
    let members = members.unwrap();
    assert!(members.webhooks.is_none());
    assert!(members.forges[0].webhook_secret.is_none());
}
#[test]
fn a_missing_webhooks_member_is_refused_by_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"webhooks\":null,", "");
    assert_eq!(&*refused(input.as_bytes()), b"application.webhooks: missing\n");
}
#[test]
fn a_missing_webhook_secret_is_refused_by_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace("\"webhook_secret\":null,", "");
    assert_eq!(&*refused(input.as_bytes()), b"application.forges[0].webhook_secret: missing\n");
}
#[test]
fn an_in_process_webhook_listener_is_refused() {
    let input =
        fixtures::with(&fixtures::document(), b"application.webhooks", br#"{"listener":{"in_process":"hooks"}}"#);
    assert_eq!(
        &*lines(&crate::named(&input, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.listener: invalid form\n"
    );
}
#[test]
fn the_webhook_secret_excludes_its_final_newline() {
    let input = core::str::from_utf8(fixtures::APPLICATION)
        .unwrap()
        .replace("\"webhooks\":null", "\"webhooks\":{\"listener\":{\"socket\":\"127.0.0.1:7200\"}}")
        .replace("\"webhook_secret\":null", "\"webhook_secret\":\"secrets/forge-1.webhook-secret\"");
    let (members, _) = application(input.as_bytes(), &fixtures::engine().jig);
    let resolved = application::resolve(&members.unwrap(), &fixtures::files(), &mut Problems::empty()).unwrap();
    assert_eq!(resolved.forges[0].webhook_secret.as_ref().unwrap().expose(), b"fixture-hook-secret");
}
#[test]
fn the_configured_bundle_is_named_for_the_binary() {
    let document = fixtures::with(&fixtures::document(), b"application.clients.bundle", br#""/bundle""#);
    let named = crate::named(&document, &jig_protocol_config::Directories::working()).expect("named");
    assert_eq!(named.bundle.as_deref(), Some(b"/bundle".as_slice()));
    assert_eq!(named.bundle_limits, crate::limits::Clients { bundle_files: 128, bundle_bytes: 16 << 20 });
}
#[test]
fn a_missing_bundle_is_named_by_its_path() {
    let input = core::str::from_utf8(fixtures::APPLICATION).unwrap().replace(",\"bundle\":null}", "}");
    assert_eq!(&*refused(input.as_bytes()), b"application.clients.bundle: missing\n");
}
#[test]
fn a_webhook_listener_with_a_bundle_is_refused() {
    let document = fixtures::with(
        &fixtures::webhook_document(),
        b"application.webhooks",
        br#"{"listener":{"socket":"127.0.0.1:7200"},"bundle":null}"#,
    );
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.bundle: unknown member\n"
    );
}

#[test]
fn paths_are_resolved_by_kind_and_absolute_only_members_refuse_relative_values() {
    let directories = config::Directories::given(b"/configuration", b"/data", b"/cache").unwrap();
    for (member, input, expected, refused) in [
        (b"application.workers[0].root".as_slice(), br#""root""#.as_slice(), b"/data/root".as_slice(), false),
        (b"application.workers[0].root", br#""/kept/root""#, b"/kept/root", false),
        (b"engine.deployment.directory", br#""state""#, b"/data/state", false),
        (b"engine.deployment.directory", br#""/kept/state""#, b"/kept/state", false),
        (b"engine.charters[0].instructions", br#""chat.txt""#, b"/configuration/chat.txt", false),
        (b"engine.charters[0].instructions", br#""/kept/chat.txt""#, b"/kept/chat.txt", false),
        (b"workers[0].secret", br#""secret""#, b"/data/secret", false),
        (b"workers[0].secret", br#""/kept/secret""#, b"/kept/secret", false),
        (b"engine.hosts.workers[0].digest", br#""digest""#, b"/data/digest", false),
        (b"engine.hosts.workers[0].digest", br#""/kept/digest""#, b"/kept/digest", false),
        (b"application.workers[0].git", br#""git""#, b"", true),
        (b"application.clients.bundle", br#""bundle""#, b"", true),
        (b"application.clients.bundle", br#""/kept/bundle""#, b"/kept/bundle", false),
        (b"workers[0].agent.program", br#""temper""#, b"", true),
        (b"workers[0].agent.configuration", br#""agent.json""#, b"", true),
        (b"workers[0].agent.arguments", br#"["agent","agent.json"]"#, b"", true),
    ] {
        let document = fixtures::with(&fixtures::document(), member, input);
        let result = crate::named(&document, &directories);
        if refused {
            let lines = lines(&result.unwrap_err());
            let expected = [member, b": must be an absolute path\n"].concat();
            assert_eq!(lines.as_ref(), expected, "{}", core::str::from_utf8(member).expect("test path"));
        } else {
            let named = result.unwrap();
            let value = match member {
                b"application.workers[0].root" => &*named.worker_root,
                b"engine.deployment.directory" => &*named.deployment,
                b"application.clients.bundle" => named.bundle.as_deref().unwrap(),
                _ => &*named.files.iter().find(|file| file.path.as_ref() == expected).expect("resolved file key").path,
            };
            assert_eq!(value, expected);
        }
    }
    for trust in [br#""roots.der""#.as_slice(), br#""/kept/roots.der""#] {
        let tls = [br#"{"tls":{"server_name":"example.test","trust":"#.as_slice(), trust, b"}}"].concat();
        let document = fixtures::with(&fixtures::document(), b"application.forges[0].transport", &tls);
        let named = crate::named(&document, &directories).unwrap();
        let application::Transport::Tls { trust: Some(value), .. } = &named.forges[0].transport else { panic!("TLS") };
        assert_eq!(
            value.as_ref(),
            if trust == br#""roots.der""# { b"/configuration/roots.der".as_slice() } else { b"/kept/roots.der" }
        );
    }
}

fn github_application() -> Box<str> {
    github_input(core::str::from_utf8(fixtures::APPLICATION).unwrap())
}
pub(super) fn github_input(input: &str) -> Box<str> {
    input
                .replace("\"kind\":\"forgejo\"", "\"kind\":\"github\"")
        .replace("\"api\":{\"socket\":\"127.0.0.1:9\"}", "\"api\":null")
        .replace("\"transport\":\"plaintext\"", "\"transport\":null")
        .replace("\"git\":\"http://127.0.0.1:9\"", "\"git\":null,\"web\":null")
        .replace(",\"webhook_secret\":null,\"api_token\":\"secrets/forge-1.api-token\",\"user\":{\"file\":\"secrets/forge-1.user\"},\"identity\":1", "")
        .replace("\"token\":\"secrets/forge-1.git-token\",\"login\":\"temper\",\"author\":{\"name\":\"Temper\",\"email\":\"temper@example.test\"}", "\"forge\":1,\"login\":null,\"author\":null").into_boxed_str()
}
#[test]
fn load_refuses_a_record_one_byte_short_of_app_key() {
    let document = github_input(core::str::from_utf8(&fixtures::document()).unwrap());
    let needle = "\"overrides\":{\"checkout.name_bytes\"";
    assert_eq!(document.matches(needle).count(), 1);
    let document = document.replacen(needle, "\"overrides\":{\"records.value_bytes\":1196,\"checkout.name_bytes\"", 1);
    let found = crate::named(document.as_bytes(), &config::Directories::working()).unwrap_err();
    assert_eq!(
        lines(&found).as_ref(),
        b"application.limits.overrides.records.value_bytes: must agree with AppKey (forge_component.key_bytes)\n"
    );
}
#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary tests format input and diagnostics")]
fn load_refuses_log_limits_naming_both_bounds() {
    let document = fixtures::document();
    let document = core::str::from_utf8(&document).unwrap();
    let needle = "\"overrides\":{\"checkout.name_bytes\"";
    assert_eq!(document.matches(needle).count(), 1);
    for (overrides, expected) in [
        (
            "\"forges.log_origins\":0",
            "application.limits.overrides.forges.log_origins: must agree with forges.log_origins (above 0)\n",
        ),
        (
            "\"forge_component.log_origins\":0",
            "application.limits.overrides.forge_component.log_origins: must agree with forge_component.log_origins (above 0)\napplication.limits.overrides.forges.log_origins: must agree with forge_component.log_origins\n",
        ),
        (
            "\"forge_component.log_poll_ms\":0",
            "application.limits.overrides.forge_component.log_poll_ms: must agree with forge_component.log_poll_ms (above 0)\n",
        ),
        (
            "\"forge_component.log_catch_up_ms\":0",
            "application.limits.overrides.forge_component.log_catch_up_ms: must agree with forge_component.log_catch_up_ms (above 0)\napplication.limits.overrides.forge_component.log_catch_up_ms: must agree with forge_component.log_poll_ms\n",
        ),
        (
            "\"forge_component.log_pages\":0",
            "application.limits.overrides.forge_component.log_pages: must agree with forge_component.log_pages (above 0)\n",
        ),
        (
            "\"forge_component.delivery_bytes\":0",
            "application.limits.overrides.forge_component.delivery_bytes: must agree with forge_component.delivery_bytes (above 0)\n",
        ),
        (
            "\"forges.log_origins\":5,\"forge_component.log_origins\":5",
            "application.limits.overrides.forges.log_origins: must agree with forges.log_origins (at most 4)\n",
        ),
        (
            "\"forges.log_origins\":3,\"forge_component.log_origins\":2",
            "application.limits.overrides.forges.log_origins: must agree with forge_component.log_origins\n",
        ),
        (
            "\"forge_component.log_poll_ms\":86400000,\"forge_component.log_catch_up_ms\":86399999",
            "application.limits.overrides.forge_component.log_catch_up_ms: must agree with forge_component.log_poll_ms\n",
        ),
        (
            "\"forge_component.log_catch_up_ms\":86400001",
            "application.limits.overrides.forge_component.log_catch_up_ms: must agree with forge_component.log_catch_up_ms (at most 86400000 ms / 24 h)\n",
        ),
    ] {
        let input = document.replacen(needle, &format!("\"overrides\":{{{overrides},\"checkout.name_bytes\""), 1);
        let found = crate::named(input.as_bytes(), &config::Directories::working()).unwrap_err();
        assert_eq!(lines(&found).as_ref(), expected.as_bytes(), "{overrides}");
    }
    let overrides = "\"forges.log_origins\":3,\"forge_component.log_origins\":7,\"forge_component.log_poll_ms\":7123,\"forge_component.log_catch_up_ms\":9321,\"forge_component.log_pages\":13,\"forge_component.delivery_bytes\":12345";
    let input = document.replacen(needle, &format!("\"overrides\":{{{overrides},\"checkout.name_bytes\""), 1);
    crate::named(input.as_bytes(), &config::Directories::working()).unwrap();
    // Named checks all relations; inspect the same document through the existing application parser.
    let parsed =
        config::document(input.as_bytes(), &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 })
            .unwrap();
    let at = config::read::Object::new(&parsed, 0).unwrap().member(&parsed, b"application").unwrap();
    let jig = fixtures::engine().jig;
    let mut problems = Problems::empty();
    let loaded = application::parse(&parsed, at, &jig, &config::Directories::working(), &mut problems).unwrap().limits;
    assert!(problems.is_empty());
    assert_eq!((loaded.forges.log_origins, loaded.forge_component.log_origins), (3, 7));
    assert_eq!(loaded.forge_component.log_poll, skein_lib::Duration::from_millis(7123));
    assert_eq!(loaded.forge_component.log_catch_up, skein_lib::Duration::from_millis(9321));
    assert_eq!(loaded.forge_component.log_pages, 13);
    assert_eq!(loaded.forge_component.delivery_bytes, 12_345);
}
#[test]
fn github_defaults_and_the_identity_join_need_no_credential_files() {
    use crate::settings::Kind;
    let jig = fixtures::engine().jig;
    let (members, problems) = application(github_application().as_bytes(), &jig);
    assert_eq!(lines(&problems).as_ref(), b"");
    let members = members.unwrap();
    let forge = &members.forges[0];
    assert_eq!(forge.kind, Kind::GitHub);
    assert_eq!(forge.api, config::Address::Host { name: Box::from(b"api.github.com".as_slice()), port: 443 });
    match &forge.transport {
        application::Transport::Tls { server_name, trust } => {
            assert_eq!(server_name.as_ref(), b"api.github.com");
            assert!(trust.is_none());
        }
        application::Transport::Plaintext => panic!("GitHub defaults to TLS"),
    }
    assert_eq!(forge.git.as_ref(), b"https://github.com");
    assert_eq!(forge.web.as_deref(), Some(b"https://github.com".as_slice()));
    assert_eq!(forge.identity, 1);
    assert_eq!(members.identities[0].forge, Some(1));
    assert!(application::files(&members, 4096, 8192).is_empty());
    let resolved =
        application::resolve(&members, &settings::Files { entries: List::with_capacity(0) }, &mut Problems::empty())
            .unwrap();
    assert!(resolved.forges[0].api_token.is_none());
    assert_eq!(resolved.identities[0].forge, 1);
    let input = github_application()
        .replace("\"api\":null", "\"api\":{\"socket\":\"127.0.0.1:9\"}")
        .replace("\"transport\":null", "\"transport\":\"plaintext\"")
        .replace("\"web\":null", "\"web\":\"http://127.0.0.1:9/web\"");
    let (members, problems) = application(input.as_bytes(), &jig);
    assert_eq!(lines(&problems).as_ref(), b"");
    assert_eq!(members.unwrap().forges[0].web.as_deref(), Some(b"http://127.0.0.1:9/web".as_slice()));
}
#[test]
fn github_refusals_name_the_configured_member() {
    let jig = fixtures::engine().jig;
    for (needle, replacement, path) in [
        ("\"kind\":\"github\"", "\"kind\":\"other\"", "forges[0].kind"),
        ("\"web\":null", "\"web\":\"https://bad/\"", "forges[0].web"),
        ("\"web\":null", "\"web\":null,\"api_token\":\"file\"", "forges[0].api_token"),
        ("\"web\":null", "\"web\":null,\"user\":42", "forges[0].user"),
        ("\"web\":null", "\"web\":null,\"webhook_secret\":null", "forges[0].webhook_secret"),
        ("\"forge\":1", "\"forge\":1,\"token\":null", "identities[0].token"),
        ("127.0.0.1:7100", "127.0.0.1:0", "clients.listener"),
        ("\"forge\":1", "\"forge\":2", "forges[0].identity"),
        ("\"forge\":1,", "", "forges[0].identity"),
        ("\"web\":null", "\"web\":null,\"identity\":1", "forges[0].identity"),
        ("\"web\":null", "\"web\":null,\"identity\":0", "forges[0].identity"),
        (
            "\"identities\":[",
            "\"identities\":[{\"number\":2,\"forge\":1,\"login\":null,\"author\":null},",
            "forges[0].identity",
        ),
    ] {
        let input = github_application().replace(needle, replacement);
        let (members, problems) = application(input.as_bytes(), &jig);
        assert!(members.is_none(), "{path}");
        assert!(core::str::from_utf8(&lines(&problems)).unwrap().contains(path), "{path}: {:?}", lines(&problems));
    }
    let input = core::str::from_utf8(fixtures::APPLICATION)
        .unwrap()
        .replace("\"kind\":\"forgejo\"", "\"kind\":\"forgejo\",\"web\":\"https://forge.example\"");
    let (members, problems) = application(input.as_bytes(), &jig);
    assert!(members.is_none());
    assert!(core::str::from_utf8(&lines(&problems)).unwrap().contains("forges[0].web"));
}

#[test]
fn a_configured_github_forge_without_records_starts_not_connected() {
    let document = fixtures::document();
    let document = github_input(core::str::from_utf8(&document).unwrap());
    let mut service = crate::startup(
        document.as_bytes(),
        &config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    )
    .unwrap();
    assert_eq!(service.engine.workers.forges[0].kind, settings::Kind::GitHub);
    service.engine.store = fixtures::memory_store();
    let mut face = fixtures::ForgeFace::new();
    for _ in 0_u8..32 {
        face.turn(&mut service);
    }
    assert_eq!(
        service.notes().pop(),
        Some(crate::Note::ForgeNotConnected { number: 1, why: crate::NotConnected::NoRecords })
    );
    assert!(service.engine.credentials.is_empty());
    assert!(service.engine.forge.starts.is_empty());
    assert!(service.engine.forge_records.done());
    assert!(service.engine.domain.ready());
    assert_eq!(service.end(), None);
}

#[test]
fn startup_admits_the_full_setup_total_and_refuses_one_byte_short() {
    let document = github_input(core::str::from_utf8(&fixtures::document()).unwrap())
        .replace("\"api\":null", "\"api\":{\"socket\":\"127.0.0.1:9\"}")
        .replace("\"transport\":null", "\"transport\":\"plaintext\"");
    let directories = config::Directories::working();
    let files = fixtures::files();
    let agent = fixtures::startup_agent();
    let seeds = fixtures::seeds();
    let required = crate::startup::memory_requirements(
        document.as_bytes(),
        &directories,
        &files,
        &agent,
        &seeds,
        None,
        fixtures::forge_runtime(),
    )
    .unwrap();
    for short in [false, true] {
        let input = document
            .replacen("1152921504606846976", &(required.engine_bytes - u64::from(short)).to_string(), 1)
            .replace("1152921504606846976", &required.worker_bytes.to_string());
        let started = crate::startup(
            input.as_bytes(),
            &directories,
            &files,
            &agent,
            &seeds,
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        );
        if short {
            assert_eq!(
                lines(&started.err().unwrap()).as_ref(),
                b"engine.memory_bytes: the worst case passes memory_bytes by 1 bytes\n"
            );
        } else {
            assert_eq!(started.unwrap().worst_case(), required.engine_bytes + required.worker_bytes);
        }
    }
}
#[test]
fn startup_maps_refused_setup_and_total_overflow_to_memory_bound() {
    let mut engine = fixtures::engine();
    let (endpoints, _) = crate::forge::prepare(&engine, fixtures::forge_runtime()).unwrap();
    let root = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let store = crate::store::limits(&root, &engine.jig.limits.core, &root.forge, &engine.jig.limits.store).unwrap();
    let io = engine.jig.limits.io;
    // u32::MAX runs overflows setup's writes = runs + forges, after the host bound fits.
    for (runs, roots) in [(0, 0), (u32::MAX, 0), (8, u64::MAX)] {
        engine.limits.setup.runs = runs;
        assert_eq!(runs.checked_add(engine.limits.forge_component.forges).is_none(), runs == u32::MAX);
        assert_eq!(crate::setup::worst_case(&engine, &endpoints, 2).is_none(), runs != 8);
        let found = crate::system::startup_worst_case(&engine, &root, &store, None, 0, &io, (roots, 0, &endpoints))
            .unwrap_err();
        assert_eq!(lines(&Problems::jig(found, &Path::root())).as_ref(), b"engine.memory_bytes: past its bound\n");
    }
}

#[test]
fn admission_model_words_use_tokens_and_missing_names_make_no_amount_claim() {
    use jig_domain_fleet::AgentShortfall;
    for (why, expected) in [
        (
            AgentShortfall::Model { charter: 7, model: 0 },
            "model: not in the agent's configuration under the same name\n",
        ),
        (
            AgentShortfall::Window { charter: 7, model: 0 },
            "model: needs a context window of 8192 tokens; the agent offers 1024 tokens\n",
        ),
        (
            AgentShortfall::Output { charter: 7, model: 0 },
            "model: needs 8192 output tokens; the agent offers 1024 tokens\n",
        ),
    ] {
        let mut problems = Problems::empty();
        problems.push(Path::root().member(b"model"), What::Admission { why, offered: 1024, required: 8192 });
        assert_eq!(&*lines(&problems), expected.as_bytes());
    }
}

#[test]
fn a_store_reader_derives_serves_limits_or_their_problem_path() {
    let document = fixtures::document();
    let directories = config::Directories::working();
    let service = crate::startup(
        &document,
        &directories,
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    )
    .unwrap();
    let refused = core::str::from_utf8(&document).unwrap().replacen(
        "\"overrides\":{\"checkout.name_bytes\"",
        "\"overrides\":{\"root.loads.rows\":0,\"checkout.name_bytes\"",
        1,
    );
    for (name, input, expected) in [
        ("serve", document.as_ref(), Ok((service.engine.store.opening_limits(), service.engine.env.limits.forge))),
        ("limits", refused.as_bytes(), Err(b"engine.limits: past its bound\n".as_slice())),
    ] {
        let found = crate::store_limits(input, &directories);
        let found = found.map(|limits| (limits.store, limits.forge)).map_err(|problems| lines(&problems));
        assert_eq!(found.as_ref().map_err(AsRef::as_ref), expected.as_ref().map_err(AsRef::as_ref), "{name}");
    }
}
