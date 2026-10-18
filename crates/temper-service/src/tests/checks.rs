use super::fixtures;
use crate::startup::{Problems, What, checks, lines};
use jig_protocol_config::{self as config, Path};
use temper_protocol_worker_wire as payload;

fn started(input: &[u8], files: &crate::settings::Files, agent: &crate::settings::Agent) -> Problems {
    match crate::startup(
        input,
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
        br#"{"checkout.name_bytes":300,"checkout.repositories":4}"#,
    );
    let found = crate::named(&document).unwrap_err();
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
            b"engine.llm.endpoints[0]: not in the agent's configuration under the same name and account\n".as_slice(),
        ),
        (
            "model",
            true,
            false,
            b"engine.charters[0].model: not in the agent's configuration under the same name and account\n",
        ),
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
    let found = crate::named(&document).unwrap_err();
    assert_eq!(&*lines(&found), b"engine.limits.io.sockets: 49 bytes, past its bound of 48\n".as_slice());
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
        &*lines(&crate::named(&document).unwrap_err()),
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
        &*lines(&crate::named(&document).unwrap_err()),
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
    assert_eq!(&*lines(&crate::named(&document).unwrap_err()), b"application.webhooks.listener: must be unique\n");
}
#[test]
fn webhook_listeners_on_port_zero_do_not_clash() {
    let document =
        fixtures::with(&fixtures::webhook_document(), b"application.webhooks.listener", br#"{"socket":"127.0.0.1:0"}"#);
    let document = fixtures::with(&document, b"application.clients.listener", br#"{"socket":"127.0.0.1:0"}"#);
    let named = crate::named(&document).unwrap();
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
        &*lines(&crate::named(&document).unwrap_err()),
        b"engine.limits.io.sockets: 66 bytes, past its bound of 48\n"
    );
}
#[test]
fn the_shipped_profile_admits_a_webhook_listener() {
    let found = started(&fixtures::webhook_document(), &fixtures::files(), &fixtures::startup_agent());
    assert_eq!(&*lines(&found), b"");
}
