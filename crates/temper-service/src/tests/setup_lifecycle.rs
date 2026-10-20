use super::*;
use crate::setup::lifecycle;
use skein_forge_connection::{self as connection, Change};
use temper_engine_domain_forge as domain;

fn keep_next(service: &mut crate::Service, item: Item) {
    let owner = *service.engine.setup.writes.first().unwrap().0;
    assert_eq!(service.engine.setup.writes.get(&owner).unwrap().number, records::number(1, item));
    answer(&mut service.engine, StoreEvent::Kept { owner });
    advance(&mut service.engine);
}
fn converted() -> crate::Service {
    let (mut service, mut face) = writing();
    for item in [Item::AppId, Item::AppSlug, Item::AppKey, Item::User] {
        keep_next(&mut service, item);
        assert_eq!(service.engine.setup.answers.pop(), Some((Token::new(2), host::Answer::Written)));
        face.turn(&mut service);
    }
    service
}
fn app(service: &mut crate::Service) {
    for (item, value) in [
        (Item::AppId, Box::from(7_u64.to_le_bytes())),
        (Item::AppSlug, Box::from(b"app-fixture".as_slice())),
        (Item::AppKey, Box::from(skein_fake_forge_github::keys::KEYS[0])),
        (Item::User, Box::from(9_u64.to_le_bytes())),
    ] {
        row(service, item, &value);
    }
    let origin = records::origin(connection::settings::Kind::GitHub, &service.engine.workers.forges[0].api).encode();
    row(service, Item::Origin, &origin);
}
fn accepted(service: &mut crate::Service, ids: &[u64]) {
    row(service, Item::Accepted, skein_forge_setup::item::accepted_item(ids).expose());
}
fn values(service: &crate::Service) -> Box<[u64]> {
    lifecycle::accepted(&service.engine, 1)
}
fn installation(service: &mut crate::Service, id: u64, change: Change) {
    service.engine.forge.above.push(connection::Event::Installation {
        forge: 1,
        installation: id,
        account: Some(connection::Account {
            id: 100_u64.checked_add(id).unwrap(),
            login: Box::from(b"account-fixture".as_slice()),
        }),
        change,
    });
    crate::engine::forge_pass(&mut service.engine);
}
fn run_events(service: &mut crate::Service) {
    for _ in 0..service.engine.events.len() {
        let event = service.engine.events.pop().unwrap();
        if let root::Event::Forge(_) = &event {
            service.engine.step(event);
        }
    }
    for _ in 0_u32..8 {
        service.engine.step(root::Event::Resume);
    }
}
#[test]
fn conversion_keeps_origin_then_installs_answers_the_browser_and_connects_from_records() {
    let mut service = converted();
    assert!(service.engine.forge.starts.is_empty(), "no Connect before Origin Kept");
    for event in &service.engine.events {
        if let root::Event::Setup { .. } = event {
            panic!("browser still waiting");
        }
    }
    let owner = *service.engine.setup.writes.first().unwrap().0;
    let write = service.engine.setup.writes.get(&owner).unwrap();
    assert_eq!(write.number, records::number(1, Item::Origin));
    assert!(service.engine.setup.answers.is_empty(), "Origin is the routing's write");
    // The records, rather than the conversion outcome, determine access.
    row(&mut service, Item::User, &19_u64.to_le_bytes());
    keep_next(&mut service, Item::Origin);
    assert!(service.engine.forge.starts.is_empty(), "Install runs before Connect");
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Install { state, url }, .. } = terminal(&mut service)
    else {
        panic!("browser install link")
    };
    assert!(!state.is_empty());
    assert!(url.starts_with(b"http://127.0.0.1:3000/apps/app-fixture/"));
    let (forge, access) = service.engine.forge.starts.get(0).unwrap();
    assert_eq!((*forge, access.user), (1, 19));
    let skein_forge::credential::Credential::App { id, key } = &access.credential else { panic!("App access") };
    assert_eq!(*id, 7);
    assert_eq!(key.expose().len(), skein_fake_forge_github::keys::KEYS[0].len());
    assert_eq!(service.engine.forge.accepted.get(0).unwrap().1.as_ref(), []);
}
#[test]
fn an_install_failure_answers_the_conversion_but_the_kept_app_still_connects() {
    let mut service = converted();
    let owner = *service.engine.setup.writes.first().unwrap().0;
    answer(&mut service.engine, StoreEvent::Kept { owner });
    // Exercise the routing boundary's terminal independently of the local Install machine.
    for _ in 0..service.engine.setup.above.len() {
        let _discard = service.engine.setup.above.pop();
    }
    service.engine.setup.above.push(host::Event::Stop {
        run: Token::new(2),
        failure: reconcile::Failure {
            forge: 1,
            step: reconcile::Stage::State,
            subject: Box::from(b"install".as_slice()),
            why: reconcile::Why::Crowded,
        },
    });
    let root::SetupEnd::Stopped { reason, .. } = terminal(&mut service) else { panic!("install failure") };
    assert!(reason.ends_with(b"Crowded"));
    assert_eq!(service.engine.forge.starts.len(), 1);
}
#[test]
fn a_spent_conversion_code_names_the_possibly_orphaned_app_without_connecting() {
    let mut service = service(8, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    start(&mut service, 2, host::Plan::Convert { forge: 1, code: code(), state });
    for _ in 0..service.engine.setup.above.len() {
        let _discard = service.engine.setup.above.pop();
    }
    service.engine.setup.above.push(host::Event::Stop {
        run: Token::new(2),
        failure: reconcile::Failure {
            forge: 1,
            step: reconcile::Stage::App,
            subject: Box::from(b"convert".as_slice()),
            why: reconcile::Why::Refused(404),
        },
    });
    let root::SetupEnd::Stopped { reason, .. } = terminal(&mut service) else { panic!("spent code") };
    assert!(contains(&reason, b"App may exist on GitHub without temper holding its key"));
    assert!(service.engine.forge.starts.is_empty());
}
#[test]
fn manifest_and_convert_refuse_even_partial_app_records_and_preserve_the_accepted_set() {
    for item in [Item::AppId, Item::AppSlug, Item::AppKey] {
        let mut service = service(8, Duration::from_secs(3600));
        let state = manifest(&mut service, 1);
        row(&mut service, item, b"partial-record");
        accepted(&mut service, &[11, 22]);
        for (run, plan) in [
            (2, host::Plan::Manifest { forge: 1, target: Target::Personal }),
            (3, host::Plan::Convert { forge: 1, code: code(), state }),
        ] {
            start(&mut service, run, plan);
            let root::SetupEnd::Stopped { reason, .. } = terminal(&mut service) else { panic!("existing App refused") };
            assert!(contains(&reason, b"Forget"));
        }
        assert_eq!(values(&service).as_ref(), [11, 22]);
        assert!(service.engine.setup.io.is_empty());
    }
}
#[test]
fn connected_delivers_the_records_identity_to_the_root_then_reconciles_once() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    service.engine.forge.above.push(connection::Event::Connected { forge: 1 });
    crate::engine::forge_pass(&mut service.engine);
    let Some(root::Event::Forge(event)) = service.engine.events.pop() else { panic!("connected reaches root") };
    assert_eq!(
        *event,
        domain::Event::Connection {
            forge: 1,
            state: domain::Connection::Connected { id: 9, login: Box::from(b"app-fixture[bot]".as_slice()) }
        }
    );
    assert!(service.engine.setup.runs.is_empty(), "root learns Connected first");
    advance(&mut service.engine);
    assert!(service.engine.setup.runs.is_empty(), "reconcile waits for the root to give the writer");
    service.engine.step(root::Event::Forge(event));
    advance(&mut service.engine);
    assert_eq!(service.engine.setup.runs.len(), 1);
    let (_, run) = service.engine.setup.runs.first().unwrap();
    assert_eq!(run.kind, Kind::Reconcile);
    assert!(run.internal);
    advance(&mut service.engine);
    assert_eq!(service.engine.setup.runs.len(), 1, "one pending reconcile per forge");
}
#[test]
fn connected_holds_one_reconcile_until_the_writing_run_ends() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    start(&mut service, 1, host::Plan::Install { forge: 1 });
    for _ in 0_u32..2 {
        service.engine.forge.above.push(connection::Event::Connected { forge: 1 });
        crate::engine::forge_pass(&mut service.engine);
    }
    run_events(&mut service);
    assert_eq!(service.engine.setup.runs.len(), 1);
    assert!(!service.engine.setup.runs.first().unwrap().1.internal);
    advance(&mut service.engine);
    assert_eq!(service.engine.setup.runs.len(), 1, "Install still reading");
    for _ in 0_u32..8 {
        advance(&mut service.engine);
    }
    assert_eq!(service.engine.setup.runs.len(), 1);
    assert!(service.engine.setup.runs.first().unwrap().1.internal);
}
fn accepting(ids: &[u64], installation: u64) -> (crate::Service, Face) {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    accepted(&mut service, ids);
    start(&mut service, 1, host::Plan::Install { forge: 1 });
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Install { state, .. }, .. } = terminal(&mut service) else {
        panic!("install state")
    };
    start(
        &mut service,
        2,
        host::Plan::Accept { forge: 1, installation: core::num::NonZeroU64::new(installation).unwrap(), state },
    );
    let mut face = Face::new();
    face.sent(&mut service);
    (service, face)
}
fn accept_reply(service: &mut crate::Service, face: &mut Face) {
    face.reply(service, br#"{"id":22,"app_id":7,"account":{"id":122,"login":"account-fixture","type":"Organization"},"target_type":"Organization","created_at":"1970-01-01T00:00:00Z","suspended_at":null}"#);
    assert!(!service.engine.setup.writes.is_empty(), "Accept ended before its write: {:?}", terminal(service));
}
#[test]
fn accept_hands_the_kept_set_to_the_component_only_after_done() {
    let (mut service, mut face) = accepting(&[11], 22);
    installation(&mut service, 22, Change::Created);
    accept_reply(&mut service, &mut face);
    assert!(service.engine.forge.accepted.is_empty(), "no Installations before Kept");
    assert_eq!(values(&service).as_ref(), [11]);
    keep_next(&mut service, Item::Accepted);
    assert!(let_done(terminal(&mut service)));
    assert!(!service.engine.forge.accepted.is_empty(), "Accept Done must send Installations");
    assert_eq!(service.engine.forge.accepted.get(0).unwrap().1.as_ref(), [11, 22]);
    let info = service.engine.setup.lifecycle.installations.get(&(1, 22)).unwrap();
    assert!(!info.stray, "Accept ends the stray mark");
    assert!(service.engine.forge.starts.is_empty(), "Accept never reconnects");
}
#[test]
fn deletion_keeps_the_rewritten_set_before_handing_it_down() {
    let mut service = service(8, Duration::from_secs(3600));
    accepted(&mut service, &[11, 22]);
    installation(&mut service, 11, Change::Deleted);
    advance(&mut service.engine);
    assert_eq!(values(&service).as_ref(), [11, 22], "old set until Kept");
    assert!(service.engine.forge.accepted.is_empty());
    keep_next(&mut service, Item::Accepted);
    assert_eq!(values(&service).as_ref(), [22]);
    assert_eq!(service.engine.forge.accepted.get(0).unwrap().1.as_ref(), [22]);
    installation(&mut service, 11, Change::Deleted);
    advance(&mut service.engine);
    assert!(service.engine.setup.writes.is_empty(), "same deletion writes nothing twice");
}
#[test]
fn deletion_during_accept_waits_then_rewrites_from_the_newly_kept_set() {
    let (mut service, mut face) = accepting(&[11], 22);
    installation(&mut service, 11, Change::Deleted);
    advance(&mut service.engine);
    assert!(service.engine.setup.writes.is_empty(), "deletion must wait for Accept");
    accept_reply(&mut service, &mut face);
    assert_eq!(service.engine.setup.writes.len(), 1);
    keep_next(&mut service, Item::Accepted);
    for _ in 0_u32..4 {
        advance(&mut service.engine);
    }
    assert_eq!(values(&service).as_ref(), [11, 22], "Accept's kept set is the rewrite's source");
    assert_eq!(service.engine.setup.writes.len(), 1, "deferred deletion now writes");
    keep_next(&mut service, Item::Accepted);
    assert_eq!(values(&service).as_ref(), [22], "neither Accept nor deletion is lost");
    assert_eq!(service.engine.forge.accepted.get(1).unwrap().1.as_ref(), [22]);
}
#[test]
fn a_first_listing_deletion_suspension_and_stray_are_idempotent() {
    let mut service = service(8, Duration::from_secs(3600));
    accepted(&mut service, &[11, 22]);
    for _ in 0_u32..2 {
        installation(&mut service, 11, Change::Deleted);
        installation(&mut service, 22, Change::Suspended);
        installation(&mut service, 33, Change::Created);
    }
    assert_eq!(service.engine.events.len(), 3, "same deleted, suspended and stray states reach root only once");
    assert!(service.engine.setup.lifecycle.installations.get(&(1, 22)).unwrap().suspended);
    assert!(service.engine.setup.lifecycle.installations.get(&(1, 33)).unwrap().stray);
    advance(&mut service.engine);
    assert_eq!(service.engine.setup.writes.len(), 1);
    keep_next(&mut service, Item::Accepted);
    assert_eq!(values(&service).as_ref(), [22]);
    installation(&mut service, 33, Change::Deleted);
    assert!(!service.engine.setup.lifecycle.installations.contains_key(&(1, 33)));
}
#[test]
fn a_stopped_event_reaches_the_root_and_a_later_connected_clears_it() {
    let mut service = fixtures::ready();
    let why = connection::Stopped::KeyRefused;
    service.engine.forge.above.push(connection::Event::Stopped { forge: 1, why });
    crate::engine::forge_pass(&mut service.engine);
    run_events(&mut service);
    assert_eq!(
        service.engine.domain.forge_connection(1),
        domain::Connection::Stopped { reason: Box::from(b"KeyRefused".as_slice()) }
    );
    service.engine.forge.above.push(connection::Event::Connected { forge: 1 });
    crate::engine::forge_pass(&mut service.engine);
    run_events(&mut service);
    assert_eq!(
        service.engine.domain.forge_connection(1),
        domain::Connection::Connected { id: 42, login: Box::from(b"temper".as_slice()) }
    );
}
#[test]
fn a_reconcile_failure_names_its_stop_without_connecting_or_changing_the_connection() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    start(&mut service, 1, host::Plan::Reconcile { forge: 1, adopted: Box::new([]) });
    for _ in 0..service.engine.setup.above.len() {
        let _discard = service.engine.setup.above.pop();
    }
    service.engine.setup.above.push(host::Event::Stop {
        run: Token::new(1),
        failure: reconcile::Failure {
            forge: 1,
            step: reconcile::Stage::App,
            subject: Box::from(b"app".as_slice()),
            why: reconcile::Why::Refused(401),
        },
    });
    let root::SetupEnd::Stopped { reason, .. } = terminal(&mut service) else { panic!("reconcile stop") };
    assert!(contains(&reason, b"Refused(401)"));
    assert!(service.engine.forge.starts.is_empty());
    assert_eq!(service.engine.domain.forge_connection(1), domain::Connection::NotConnected);
}

fn connection_turn(service: &mut crate::Service, face: &mut Face) {
    service.engine.forge.iterate(Time::ZERO, Wall::EPOCH);
    for _ in 0..service.engine.forge.io.len() {
        match service.engine.forge.io.pop().unwrap() {
            skein_io::Request::Connect { owner, .. } => {
                face.owner = Some(owner);
                face.wire = skein_world::stream::Wire::new(7);
                service.engine.forge.up(skein_io::Event::Connecting { owner, socket: Token::new(321) });
                service.engine.forge.up(skein_io::Event::Connected { owner });
            }
            skein_io::Request::Stream { down, .. } => face.wire.take(down),
            skein_io::Request::Close { .. } | skein_io::Request::Abort { .. } => {
                service.engine.forge.up(skein_io::Event::Closed { owner: face.owner.take().unwrap() });
            }
            skein_io::Request::Listen { .. }
            | skein_io::Request::Bind { .. }
            | skein_io::Request::Reject { .. }
            | skein_io::Request::Output { .. }
            | skein_io::Request::Spawn { .. }
            | skein_io::Request::Signal { .. }
            | skein_io::Request::Usage { .. }
            | skein_io::Request::Window { .. } => panic!("connection socket request"),
        }
    }
    if let Some(owner) = face.owner
        && let Some(up) = face.wire.answer()
    {
        service.engine.forge.up(skein_io::Event::Stream { owner, up });
    }
}
fn locate(service: &mut crate::Service, call: u64) {
    service.engine.forge.down(connection::Request::Call {
        call: Token::new(call),
        repository: skein_forge::api::Repository { forge: 1, repository: 0 },
        op: skein_forge::api::Op::Read(skein_forge::api::Read::Locate {
            owner: Box::from(b"account-fixture".as_slice()),
            name: Box::from(b"repository-fixture".as_slice()),
        }),
    });
}
#[test]
fn calls_are_not_connected_before_connect_then_wait_until_connected() {
    let mut service = converted();
    locate(&mut service, 1);
    let Some(connection::Event::Answered(answer)) = service.engine.forge.above.pop() else {
        panic!("NotConnected before Connect")
    };
    assert_eq!((answer.cost, answer.result), (0, Err(skein_forge::api::Error::NotConnected)));
    keep_next(&mut service, Item::Origin);
    assert!(let_done(terminal(&mut service)));
    locate(&mut service, 2);
    assert!(service.engine.forge.above.is_empty(), "a call after Connect waits for the start");
    let mut face = Face::new();
    for _ in 0_u32..1024 {
        connection_turn(&mut service, &mut face);
    }
    assert!(contains(&face.wire.received, b"GET /app/installations"));
    face.wire.write(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n[]");
    for _ in 0_u32..1024 {
        connection_turn(&mut service, &mut face);
    }
    let mut connected = false;
    let mut answered = false;
    for _ in 0..service.engine.forge.above.len() {
        match service.engine.forge.above.pop().unwrap() {
            connection::Event::Connected { forge: 1 } => connected = true,
            connection::Event::Answered(answer) => {
                assert!(connected, "the waiting call is answered after Connected");
                assert_eq!(
                    (answer.call, answer.cost, answer.result),
                    (Token::new(2), 0, Err(skein_forge::api::Error::NotInstalled))
                );
                answered = true;
            }
            connection::Event::Installation { .. }
            | connection::Event::Fact { .. }
            | connection::Event::GitToken { .. }
            | connection::Event::Hint(_)
            | connection::Event::ListenerLost
            | connection::Event::Connected { .. }
            | connection::Event::Stopped { .. }
            | connection::Event::Closed => panic!("connection start event"),
        }
    }
    assert!(connected && answered);
}

#[test]
fn a_real_reconcile_prunes_accepted_after_kept_and_never_connects_or_writes_origin() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    accepted(&mut service, &[11]);
    start(&mut service, 1, host::Plan::Reconcile { forge: 1, adopted: Box::new([]) });
    let mut face = Face::new();
    face.sent(&mut service);
    face.reply(&mut service, br#"{"id":7,"slug":"app-fixture","permissions":{},"events":[]}"#);
    face.sent(&mut service);
    face.reply(&mut service, br#"{"url":"https://app-fixture.invalid/"}"#);
    face.sent(&mut service);
    face.reply(&mut service, b"[]");
    assert_eq!(values(&service).as_ref(), [11]);
    assert_eq!(service.engine.setup.writes.len(), 1);
    keep_next(&mut service, Item::Accepted);
    assert!(let_done(terminal(&mut service)), "Lacking is a successful reconcile");
    let Some(crate::Note::ForgeLacking { number, names }) = service.engine.notes.pop() else {
        panic!("Lacking names logged")
    };
    assert_eq!(number, 1);
    assert!(contains(&names, b"permission contents=write;"));
    assert!(contains(&names, b"event push;"));
    assert_eq!(u64::try_from(names.len()).unwrap(), diagnostics::worst_case().unwrap());
    assert!(values(&service).is_empty());
    assert!(service.engine.setup.writes.is_empty(), "reconcile writes no Origin");
    assert!(service.engine.forge.starts.is_empty(), "reconcile connects nothing");
    assert!(service.engine.forge.accepted.get(0).unwrap().1.is_empty());
}
fn connection_reply(service: &mut crate::Service, face: &mut Face, body: &[u8]) {
    let length = skein_lib::Decimal::of(u64::try_from(body.len()).unwrap());
    let head = b"HTTP/1.1 200 OK\r\nContent-Length: ";
    let mut writer = skein_lib::Writer::new(
        head.len()
            .checked_add(length.as_bytes().len())
            .unwrap()
            .checked_add(4)
            .unwrap()
            .checked_add(body.len())
            .unwrap(),
    );
    writer.put(head).unwrap();
    writer.put(length.as_bytes()).unwrap();
    writer.put(b"\r\n\r\n").unwrap();
    writer.put(body).unwrap();
    face.wire.write(&writer.finish());
    for _ in 0_u32..1024 {
        connection_turn(service, face);
    }
}
#[test]
fn the_components_first_listing_routes_gone_suspended_and_stray_installations() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    accepted(&mut service, &[11, 22]);
    let expected = records::origin(connection::settings::Kind::GitHub, &service.engine.workers.forges[0].api);
    let start = records::access(
        connection::settings::Kind::GitHub,
        &expected,
        service.engine.forge_records.rows.forge(1).unwrap(),
        &service.engine.forge_records.bounds,
        false,
    )
    .unwrap();
    service.engine.forge.down(connection::Request::Connect { forge: 1, access: start.access });
    service.engine.forge.down(connection::Request::Installations { forge: 1, accepted: start.accepted });
    let mut face = Face::new();
    for _ in 0_u32..1024 {
        connection_turn(&mut service, &mut face);
    }
    connection_reply(&mut service, &mut face, br#"[{"id":22,"account":{"id":122,"login":"suspended-account","type":"Organization"},"created_at":"1970-01-01T00:00:00Z","suspended_at":"2026-01-01T00:00:00Z"},{"id":33,"account":{"id":133,"login":"stray-account","type":"User"},"created_at":"1970-01-01T00:00:00Z","suspended_at":null}]"#);
    crate::engine::forge_pass(&mut service.engine);
    assert!(service.engine.setup.lifecycle.installations.get(&(1, 22)).unwrap().suspended);
    assert!(service.engine.setup.lifecycle.installations.get(&(1, 33)).unwrap().stray);
    assert_eq!(values(&service).as_ref(), [11, 22]);
    advance(&mut service.engine);
    keep_next(&mut service, Item::Accepted);
    assert_eq!(values(&service).as_ref(), [22], "the missing accepted id is pruned exactly once");
    assert_eq!(service.engine.forge.accepted.get(1).unwrap().1.as_ref(), [22]);
}

#[test]
fn a_manifest_already_started_without_records_is_unaffected_by_records_arriving() {
    let mut service = service(8, Duration::from_secs(3600));
    start(&mut service, 1, host::Plan::Manifest { forge: 1, target: Target::Personal });
    row(&mut service, Item::AppId, &7_u64.to_le_bytes());
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Manifest { state }, .. } = terminal(&mut service) else {
        panic!("admitted Manifest finishes")
    };
    assert!(service.setup_page(1, &state, Time::ZERO).is_some());
    start(&mut service, 2, host::Plan::Manifest { forge: 1, target: Target::Personal });
    let root::SetupEnd::Stopped { reason, .. } = terminal(&mut service) else { panic!("later Manifest refused") };
    assert!(contains(&reason, b"Forget"));
}
#[test]
fn new_app_admission_changes_neither_a_connected_nor_a_stopped_forges_state() {
    for state in [
        domain::Connection::Connected { id: 42, login: Box::from(b"temper".as_slice()) },
        domain::Connection::Stopped { reason: Box::from(b"key refused".as_slice()) },
    ] {
        let mut service = fixtures::ready();
        // Use the real startup host for this forge's loopback GitHub setup boundary.
        let mut setup = super::service(8, Duration::from_secs(3600));
        app(&mut setup);
        service.engine.setup = setup.engine.setup;
        service.engine.forge_records.rows = setup.engine.forge_records.rows;
        service.engine.step(root::Event::Forge(Box::new(domain::Event::Connection { forge: 1, state: state.clone() })));
        start(&mut service, 1, host::Plan::Manifest { forge: 1, target: Target::Personal });
        let root::SetupEnd::Stopped { reason, .. } = terminal(&mut service) else { panic!("new App refused") };
        assert!(contains(&reason, b"Forget"));
        assert_eq!(service.engine.domain.forge_connection(1), state);
        assert!(service.engine.forge.starts.is_empty());
    }
}
#[test]
fn pending_reconcile_wakes_the_loop_only_when_a_run_slot_is_available() {
    let mut service = service(8, Duration::from_secs(3600));
    app(&mut service);
    lifecycle::connected(&mut service.engine, 1);
    run_events(&mut service);
    assert!(service.engine.setup.lifecycle_ready(&service.engine), "Connected after the setup pass needs a new pass");
    service.engine.setup.env.limits.runs = 1;
    let token = Token::new(99);
    service
        .engine
        .setup
        .runs
        .insert(
            token,
            Run { forge: 2, kind: Kind::Install, internal: false, previous: domain::Connection::NotConnected },
        )
        .unwrap();
    assert!(!service.engine.setup.lifecycle_ready(&service.engine), "no busy loop while all run slots wait for IO");
    service.engine.setup.runs.remove(&token).unwrap();
    assert!(service.engine.setup.lifecycle_ready(&service.engine));
    advance(&mut service.engine);
    assert!(!service.engine.setup.lifecycle_ready(&service.engine), "the reconcile now owns the forge");
}
