// Fixture parsing without a deployment keeps written paths.
#![expect(clippy::disallowed_methods, reason = "ordinary tests edit UTF-8 configuration fixtures")]
use super::fixtures;
use crate::startup::{Problems, What, checks, lines};
use jig_protocol_config::{self as config, Path};
use temper_protocol_worker_wire as payload;

fn started(input: &[u8], files: &crate::settings::Files, agent: &crate::settings::Agent) -> Problems {
    match crate::startup(
        input,
        &jig_protocol_config::Directories::working(),
        files,
        agent,
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    ) {
        Ok(service) => {
            drop(service);
            Problems::empty()
        }
        Err(problems) => problems,
    }
}
#[test]
fn standalone_shape_is_required_of_jigs_parts() {
    let mut engine = fixtures::engine().jig;
    let mut clients = fixtures::engine().clients;
    engine.hosts.workers = Box::new([engine.hosts.workers.into_iter().next().unwrap()]);
    let mut workers = Box::new([fixtures::worker_settings().jig, fixtures::worker_settings().jig]);
    for (change, path, what) in standalone_shapes() {
        let count = apply_shape(change, &mut engine, &mut clients, &mut *workers);
        let mut found = Problems::empty();
        checks::standalone(&engine, &clients, &workers[..count], &mut found);
        assert_eq!(found.problems.len(), 1, "row {path:?}");
        assert_eq!(found.problems.get(0).unwrap().what, what, "row {path:?}");
        assert_eq!(found.problems.get(0).unwrap().path, path, "row {change:?}");
    }
}
#[test]
fn a_remote_past_the_checkouts_names_refuses_the_start() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes":300,"checkout.repositories":4,"forge.name_bytes":256,"forge.resources_per_task":4}"#,
    );
    let found = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(
        &*lines(&found),
        b"application.limits.overrides.checkout.name_bytes: must agree with the longest remote\n".as_slice()
    );
}
#[test]
fn the_engines_items_past_the_workers_refuse_the_start() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes":600,"checkout.repositories":1,"git_component.stdout_bytes":2048}"#,
    );
    let found = started(&document, &fixtures::files(), &fixtures::startup_agent());
    assert_eq!(&*lines(&found), b"application.limits.overrides.forge.resources_per_task: must agree with checkout.repositories and checkout.name_bytes\n".as_slice());
}
#[test]
fn an_identity_past_the_grant_bound_is_refused_naming_it() {
    let mut files = fixtures::files();
    files.entries = fixtures::without(files.entries, b"secrets/forge-1.git-token");
    files
        .entries
        .push((b"secrets/forge-1.git-token".as_slice().into(), skein_lib::Secret::new(Box::new([b'x'; 8192]))))
        .unwrap();
    let found = started(&fixtures::document(), &files, &fixtures::startup_agent());
    assert_eq!(&*lines(&found), b"application.identities[0]: past its bound\n".as_slice());
}
#[test]
fn an_endpoint_or_model_missing_from_the_agent_is_named() {
    let document = fixtures::document();
    for (row, endpoints, models, expected) in [
        (
            "endpoint",
            false,
            true,
            b"engine.llm.endpoints[0]: not in the agent's configuration under the same name\n".as_slice(),
        ),
        ("model", true, false, b"engine.charters[0].model: not in the agent's configuration under the same name\n"),
    ] {
        let mut agent = fixtures::startup_agent();
        if !endpoints {
            agent.endpoints = smith_protocol_channel::Endpoints::new(skein_lib::List::with_capacity(0));
        }
        if !models {
            agent.bounds.models = Box::new([]);
        }
        let found = started(&document, &fixtures::files(), &agent);
        assert_eq!(&*lines(&found), expected, "row {row}");
    }
}
#[test]
fn git_component_limits_against_the_checkout_name_both() {
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    own.checkout.name_bytes = 600;
    let git = temper_worker_protocol_git::Limits { stdout_bytes: 2048, ..own.git_component };
    for (git, expected) in [
        (temper_worker_protocol_git::Limits { operations: 2, ..git }, b"application.limits.overrides.git_component.operations: must agree with checkout.workspaces\n".as_slice()),
        (temper_worker_protocol_git::Limits { stdout_bytes: 1024, ..git }, b"application.limits.overrides.git_component.stdout_bytes: must agree with checkout.name_bytes and checkout.conflicts * (checkout.path_bytes + 1)\n"),
        (temper_worker_protocol_git::Limits { stderr_bytes: 1, ..git }, b"application.limits.overrides.git_component.stderr_bytes: must agree with checkout.push_diagnostic\n"),
    ] {
        let mut found = Problems::empty();
        checks::limits(&own, &git, &mut found);
        assert_eq!(&*lines(&found), expected, "row {git:?}");
    }
}
#[test]
fn a_worst_case_past_memory_bytes_says_by_how_much() {
    let mut found = Problems::empty();
    checks::memory(4097, 4096, 513, 512, &mut found);
    assert_eq!(&*lines(&found), b"engine.memory_bytes: the worst case passes memory_bytes by 1 bytes\nworkers[0].memory_bytes: the worst case passes memory_bytes by 1 bytes\n".as_slice());
    let document = fixtures::with(&fixtures::document(), b"engine.memory_bytes", b"1");
    let document = fixtures::with(&document, b"workers[0].memory_bytes", b"1");
    let found = started(&document, &fixtures::files(), &fixtures::startup_agent());
    assert_eq!(found.problems.len(), 2, "both services checked before construction");
    for (index, path) in [
        (0, Path::root().member(b"engine").member(b"memory_bytes")),
        (1, Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"memory_bytes")),
    ] {
        assert_eq!(found.problems.get(index).unwrap().path, path, "row {index}");
        let What::Jig(config::What::Memory { over }) = found.problems.get(index).unwrap().what else {
            panic!("memory refusal row {index}");
        };
        assert!(over > 0, "row {index}");
    }
}
#[test]
fn a_channel_limit_past_a_ceiling_names_the_limit() {
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    own.checkout.name_bytes = 600;
    let normal = payload::Bounds {
        repositories: 1,
        name_bytes: 32,
        remote_bytes: 32,
        conflicts: 0,
        path_bytes: 0,
        token_bytes: 4096,
        text_bytes: 256,
    };
    let mut found = Problems::empty();
    let too_many = payload::Bounds { repositories: u32::MAX, ..normal };
    checks::channel(&too_many, &normal, &[], &own, &mut found);
    assert_eq!(found.problems.len(), 1);
    assert_eq!(
        found.problems.get(0).unwrap().path,
        Path::root().member(b"application").member(b"limits").member(b"overrides").member(b"forge.resources_per_task")
    );
    let What::Past { bytes, .. } = found.problems.get(0).unwrap().what else {
        panic!("repository ceiling named");
    };
    assert_eq!(bytes, 4_294_967_295);
}
#[test]
fn one_ios_entities_hold_both_services_and_their_children() {
    let document = fixtures::with(&fixtures::document(), b"application.limits.overrides", br#"{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":2048,"git_component.operations":9}"#);
    let found = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*lines(&found), b"engine.limits.io.sockets: 52 bytes, past its bound of 48\n".as_slice());
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    TwoWorkers,
    WorkerSocket,
    HostSocket,
    ClientProcess,
    ClientRemote,
    Connectors,
    Program,
    Configuration,
}
fn apply_shape(
    shape: Shape,
    engine: &mut config::Engine,
    clients: &mut crate::startup::application::Clients,
    workers: &mut [config::Worker],
) -> usize {
    use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use skein_io::{Addr, Name};
    let socket = config::Address::Addr(Addr::Ip(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7100))));
    let local = config::Address::Addr(Addr::InProcess(Name::new(b"workers").unwrap()));
    engine.hosts.listener = Box::new([local.clone()]);
    clients.listener = socket.clone();
    engine.connectors = Box::new([1]);
    for worker in workers.iter_mut() {
        worker.engine = local.clone();
        worker.agent.program = b"/opt/temper".as_slice().into();
        worker.agent.configuration = b"/opt/agent.json".as_slice().into();
    }
    match shape {
        Shape::TwoWorkers => return 2,
        Shape::WorkerSocket => workers[0].engine = socket,
        Shape::HostSocket => engine.hosts.listener = Box::new([socket]),
        Shape::ClientProcess => clients.listener = local,
        Shape::ClientRemote => {
            clients.listener =
                config::Address::Addr(Addr::Ip(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 7100))));
        }
        Shape::Connectors => engine.connectors = Box::new([1, 2]),
        Shape::Program => workers[0].agent.program = b"temper".as_slice().into(),
        Shape::Configuration => workers[0].agent.configuration = b"agent.json".as_slice().into(),
    }
    1
}

fn standalone_shapes() -> [(Shape, Path, What); 8] {
    [
        (Shape::TwoWorkers, Path::root().member(b"workers"), What::Standalone),
        (
            Shape::WorkerSocket,
            Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"engine"),
            What::Standalone,
        ),
        (Shape::HostSocket, Path::root().member(b"engine").member(b"hosts").member(b"listener"), What::Standalone),
        (
            Shape::ClientProcess,
            Path::root().member(b"application").member(b"clients").member(b"listener"),
            What::Loopback,
        ),
        (
            Shape::ClientRemote,
            Path::root().member(b"application").member(b"clients").member(b"listener"),
            What::Loopback,
        ),
        (Shape::Connectors, Path::root().member(b"engine").member(b"connectors"), What::Connectors),
        (
            Shape::Program,
            Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"agent").member(b"program"),
            What::Absolute,
        ),
        (
            Shape::Configuration,
            Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"agent").member(b"configuration"),
            What::Absolute,
        ),
    ]
}

#[test]
fn a_webhook_secret_without_a_listener_is_refused() {
    let document = fixtures::with(&fixtures::webhook_document(), b"application.webhooks", b"null");
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.forges[0].webhook_secret: needs application.webhooks\n"
    );
}
#[test]
fn a_webhook_listener_off_loopback_is_refused() {
    let document = fixtures::with(
        &fixtures::webhook_document(),
        b"application.webhooks.listener",
        br#"{"socket":"192.0.2.1:7200"}"#,
    );
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.listener: must be a socket address on loopback\n"
    );
}
#[test]
fn a_webhook_listener_equal_to_clients_is_refused() {
    let document = fixtures::with(
        &fixtures::webhook_document(),
        b"application.webhooks.listener",
        br#"{"socket":"127.0.0.1:7100"}"#,
    );
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.listener: must be unique\n"
    );
}
#[test]
fn webhook_listeners_on_port_zero_do_not_clash() {
    let document =
        fixtures::with(&fixtures::webhook_document(), b"application.webhooks.listener", br#"{"socket":"127.0.0.1:0"}"#);
    let document = fixtures::with(&document, b"application.clients.listener", br#"{"socket":"127.0.0.1:0"}"#);
    let named = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap();
    assert_eq!(named.webhooks.unwrap().listener, named.clients);
}
#[test]
fn a_webhook_secret_past_its_bound_is_refused() {
    let mut files = fixtures::files();
    files.entries = fixtures::without(files.entries, b"secrets/forge-1.webhook-secret");
    files
        .entries
        .push((b"secrets/forge-1.webhook-secret".as_slice().into(), skein_lib::Secret::new(Box::new([b'x'; 129]))))
        .unwrap();
    let found = started(&fixtures::webhook_document(), &files, &fixtures::startup_agent());
    assert_eq!(&*lines(&found), b"application.forges[0].webhook_secret: past its bound\n");
}
#[test]
fn hook_connections_past_the_process_sockets_are_refused() {
    let document =
        fixtures::with(&fixtures::webhook_document(), b"application.limits.overrides", br#"{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":2048,"forge_component.hooks":40}"#);
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"engine.limits.io.sockets: 69 bytes, past its bound of 48\n"
    );
}
#[test]
fn the_shipped_profile_admits_a_webhook_listener() {
    let found = started(&fixtures::webhook_document(), &fixtures::files(), &fixtures::startup_agent());
    assert_eq!(&*lines(&found), b"");
}
#[test]
fn a_bundle_limit_of_zero_refuses_the_start() {
    for (overrides, expected) in [
        (
            br#"{"clients.bundle_files":0}"#.as_slice(),
            "application.limits.overrides.clients.bundle_files: must agree with a nonempty bundle\n",
        ),
        (
            br#"{"clients.bundle_bytes":0}"#.as_slice(),
            "application.limits.overrides.clients.bundle_bytes: must agree with a nonempty bundle\n",
        ),
    ] {
        let document = fixtures::with(&fixtures::document(), b"application.limits.overrides", overrides);
        let found = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
        assert_eq!(&*lines(&found), expected.as_bytes(), "row {overrides:?}");
    }
}

#[test]
fn fixed_redirect_ports_reject_listeners_on_the_reserved_number() {
    use crate::startup::Listener;
    for (port, listener) in [(7100, Listener::Clients), (7101, Listener::Webhooks), (7100, Listener::Owned(2))] {
        let mut problems = Problems::empty();
        checks::redirect_ports(&fixed_redirects(), Some(port), listener, &mut problems);
        assert_eq!(problems.problems.len(), 1, "{listener:?} {port:?}");
        assert_eq!(problems.problems.get(0).unwrap().what, What::Redirect(listener));
    }
}
#[test]
fn fixed_redirect_ports_accept_listeners_without_a_reserved_number() {
    use crate::startup::Listener;
    for (port, listener) in [(Some(7102), Listener::Clients), (Some(0), Listener::Clients), (None, Listener::Webhooks)]
    {
        let mut problems = Problems::empty();
        checks::redirect_ports(&fixed_redirects(), port, listener, &mut problems);
        assert_eq!(problems.problems.len(), 0, "{listener:?} {port:?}");
    }
}
#[test]
fn provider_redirect_ports_are_checked_before_files_are_read() {
    const PROVIDERS: &[u8] = br#"[{"name":"Fake","number":1,"signed_in_plan":false,"models":["small"],"sources":{"sign_in":{"issuer":"https://issuer.example","client":"client","scopes":[],"authorization":"https://issuer.example/auth","token":"https://issuer.example/token","keys":"https://issuer.example/keys","transport":"tls","server_name":"issuer.example","must_grant":[],"redirect_path":"callback","redirect_port":7100}}}]"#;
    let input = fixtures::with(&fixtures::document(), b"engine.llm.providers", PROVIDERS);
    let input = fixtures::with(&input, b"application.limits.overrides", br#"{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":2048,"git_component.operations":2,"checkout.workspaces":2,"forge_component.hooks":1}"#);
    let input = core::str::from_utf8(&input).unwrap();
    for (listener, expected) in [
        (r#"{"socket":"127.0.0.1:7100"}"#, true),
        (r#"{"socket":"[::1]:7100"}"#, true),
        (r#"{"socket":"127.0.0.1:7102"}"#, false),
        (r#"{"socket":"127.0.0.1:0"}"#, false),
    ] {
        let document = fixtures::with(input.as_bytes(), b"application.clients.listener", listener.as_bytes());
        let result = crate::named(&document, &jig_protocol_config::Directories::working());
        if expected {
            assert_eq!(&*lines(&result.unwrap_err()), b"engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.clients.listener\n");
        } else {
            let named = result.unwrap();
            assert_eq!(named.redirects.len(), 1);
            assert_eq!(&*named.redirects[0].redirect_path, b"callback");
        }
    }
    let hooks = input.replace("\"webhooks\":null", r#""webhooks":{"listener":{"socket":"[::1]:7100"}}"#);
    let hooks = fixtures::with(hooks.as_bytes(), b"application.clients.listener", br#"{"socket":"127.0.0.1:7102"}"#);
    assert_eq!(&*lines(&crate::named(&hooks, &jig_protocol_config::Directories::working()).unwrap_err()), b"engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.webhooks.listener\n");
    let ephemeral = input.replace(",\"redirect_port\":7100", "");
    assert!(
        crate::named(ephemeral.as_bytes(), &jig_protocol_config::Directories::working()).unwrap().redirects.is_empty()
    );
    let input = input.replace("\"redirect_port\":7100", "\"redirect_port\":0");
    assert_eq!(
        &*lines(&crate::named(input.as_bytes(), &jig_protocol_config::Directories::working()).unwrap_err()),
        b"engine.llm.providers[0].sources.sign_in.redirect_port: invalid form\n"
    );
}

#[test]
fn every_offered_adapter_must_be_declared_by_the_agent() {
    let document = fixtures::with(
        &fixtures::document(),
        b"engine.llm.providers",
        br#"[{"name":"Fake","number":1,"manage_usage":"https://claude.ai/settings/usage","models":["small"],"sources":{"programs":["claude_code"]}}]"#,
    );
    let document = jig_protocol_config::document(
        &document,
        &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let engine_at = config::read::Object::new(&document, 0).unwrap().member(&document, b"engine").unwrap();
    let engine = config::engine(&document, engine_at, &jig_protocol_config::Directories::working()).unwrap();
    for declared in [false, true] {
        let mut agent = fixtures::startup_agent();
        if declared {
            agent.programs = Box::new([crate::settings::Program {
                adapter: crate::settings::Adapter::ClaudeCode,
                path: b"/bin/claude".as_slice().into(),
                directory: b"/programs".as_slice().into(),
            }]);
        }
        let mut problems = Problems::empty();
        checks::agent(&engine, &agent, &mut problems);
        assert_eq!(
            &*lines(&problems),
            if declared {
                b"".as_slice()
            } else {
                b"engine.llm.providers[0].sources.programs[0]: program adapter not declared in the agent's programs\n"
                    .as_slice()
            }
        );
        let mut no_offers = fixtures::engine().jig;
        no_offers.llm.providers = Box::new([]);
        let mut problems = Problems::empty();
        checks::agent(&no_offers, &agent, &mut problems);
        assert!(problems.is_empty(), "no offers need no declaration");
    }
}
#[test]
fn agent_model_bounds_are_checked_under_the_configured_name() {
    for window in [true, false] {
        let engine = fixtures::engine().jig;
        let mut agent = fixtures::startup_agent();
        if window {
            agent.bounds.models[0].window = 1;
        } else {
            agent.bounds.models[0].output = 1;
        }
        let mut problems = Problems::empty();
        checks::agent(&engine, &agent, &mut problems);
        assert_eq!(
            &*lines(&problems),
            b"engine.charters[0].model: not in the agent's configuration under the same name\n"
        );
    }
}

#[test]
fn credential_capacity_is_checked_on_both_sides_and_against_smith() {
    let mut engine = fixtures::engine().jig;
    let mut worker = fixtures::worker_settings().jig.limits;
    let mut agent = fixtures::worker_agent();
    let bound = agent.limits.bodies.credential_value_bytes.min(agent.limits.bodies.withdrawn_value_bearer);
    for (sent, received, accepted) in
        [(bound, bound, true), (bound - 1, bound, true), (bound, bound - 1, false), (bound + 1, bound + 1, false)]
    {
        engine.limits.token = sent;
        worker.token = received;
        let mut problems = Problems::empty();
        checks::grants(&engine, &worker, &agent, &mut problems);
        assert_eq!(problems.is_empty(), accepted, "{sent} -> {received}");
        if !accepted {
            assert!(core::str::from_utf8(&lines(&problems)).unwrap().contains("accounts.token"));
        }
    }
    engine.limits.token = bound;
    worker.token = bound;
    agent.limits.bodies.withdrawn_value_id = jig_protocol_accounts::ACCOUNT_ID_BYTES - 1;
    let mut problems = Problems::empty();
    checks::grants(&engine, &worker, &agent, &mut problems);
    assert!(!problems.is_empty());
}

#[test]
fn configured_keys_use_the_key_ceiling_instead_of_the_oauth_token_bound() {
    let input = fixtures::with(
        fixtures::ENGINE,
        b"llm.providers",
        br#"[{"name":"Fake","number":1,"models":["small"],"sources":{"keys":true}}]"#,
    );
    let document =
        config::document(&input, &config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 32 }).unwrap();
    let mut engine = config::engine(&document, 0, &jig_protocol_config::Directories::working()).unwrap();
    engine.limits.token = 1;
    let key = jig_client_protocol_wire::v10::CEILINGS.keyed_value_some;
    assert_eq!(crate::values::credential_bytes(&engine), key);
    let mut worker = fixtures::worker_settings().jig.limits;
    worker.token = 1;
    let mut agent = fixtures::worker_agent();
    for bytes in [key - 1, key, key + 1] {
        agent.limits.bodies.credential_value_bytes = bytes;
        let mut problems = Problems::empty();
        checks::grants(&engine, &worker, &agent, &mut problems);
        assert_eq!(problems.is_empty(), bytes >= key);
    }
    agent.limits.bodies.program_value_name = u32::try_from(smith_host_domain::SourceName::BYTES).unwrap() - 1;
    let mut problems = Problems::empty();
    checks::grants(&engine, &worker, &agent, &mut problems);
    assert!(core::str::from_utf8(&lines(&problems)).unwrap().contains("program_value_name"));
}

fn fixed_redirects() -> [crate::startup::RedirectPort; 2] {
    [7100, 7101].map(|port| crate::startup::RedirectPort {
        port,
        path: Path::root().member(b"redirect_port"),
        redirect_path: b"callback".as_slice().into(),
    })
}

#[test]
fn admission_names_global_and_time_requirements_with_their_units() {
    use skein_lib::Duration;
    for (time, waiting, spend, inbox, unacknowledged, expected) in [
        (
            29_999,
            60_000,
            100,
            8,
            8,
            "engine.charters[0].time_ms: needs 30000 ms of run time; the agent's profile.policy.max_time allows 29999 ms\n",
        ),
        (
            300_000,
            9_999,
            100,
            8,
            8,
            "engine.charters[0].waiting_ms: needs 10000 ms of waiting; the agent's profile.policy.max_waiting allows 9999 ms\n",
        ),
        (
            300_000,
            60_000,
            99,
            8,
            8,
            "engine.rules.maximum_run_spend: needs a spend allowance of 100; the agent's profile.policy.max_spend allows 99\n",
        ),
        (
            300_000,
            60_000,
            100,
            1,
            1,
            "engine.limits: needs 2 inbox messages (tasks.run_inbox); the agent's profile.policy.inbox allows 1\n",
        ),
        (
            300_000,
            60_000,
            100,
            2,
            0,
            "engine.limits: needs 1 unacknowledged turns (hub.turns); the agent's profile.policy.unacknowledged allows 0\n",
        ),
    ] {
        let mut agent = fixtures::startup_agent();
        agent.bounds.time = Duration::from_millis(time);
        agent.bounds.waiting = Duration::from_millis(waiting);
        agent.bounds.spend = spend;
        agent.bounds.inbox = inbox;
        agent.bounds.unacknowledged = unacknowledged;
        let found = started(&fixtures::document(), &fixtures::files(), &agent);
        assert_eq!(&*lines(&found), expected.as_bytes());
    }
}
