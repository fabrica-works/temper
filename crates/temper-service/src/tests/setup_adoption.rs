use super::*;
use temper_engine_domain_forge as domain;
use temper_forge_setup::setup::{Account, Entry, Fake, Mode, Repository};

const HANDOVER: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
const HOOK: &[u8] = b"file-hook-secret";
fn configured(hooks: bool) -> crate::settings::Engine {
    let mut config = fixtures::engine();
    config.forges[0].api = jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 9).into()));
    // A listener alone must not make Adopt draw a secret the connection cannot read.
    config.webhooks =
        Some(skein_forge_connection::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7101).into()) });
    if hooks {
        config.forges[0].webhook_secret = Some(Secret::new(HOOK.into()));
    }
    config.setup = Box::new([crate::settings::Setup {
        entry: Entry {
            forge: 1,
            directory: b"/data/forge".as_slice().into(),
            bot: Account { login: b"temper".as_slice().into(), email: b"temper@example.test".as_slice().into() },
            orgs: Box::new([]),
            asked: false,
            mode: Mode::Fake(Fake {
                listen: ([127, 0, 0, 1], 9).into(),
                git: b"/data/git".as_slice().into(),
                owner: b"demo".as_slice().into(),
                repositories: Box::new([
                    Repository { name: b"widget".as_slice().into(), landing: b"main".as_slice().into() },
                    Repository { name: b"other".as_slice().into(), landing: b"main".as_slice().into() },
                ]),
            }),
        },
        handover: Some(Secret::new(HANDOVER.into())),
    }]);
    fixtures::fill_files(&mut config);
    config
}
fn service(hooks: bool) -> crate::Service {
    let config = configured(hooks);
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None, fixtures::forge_runtime()).unwrap();
    service.engine.as_mut().expect("engine").store = fixtures::records_store();
    advance(service.engine.as_mut().expect("engine"));
    assert!(service.engine.as_ref().expect("engine").setup.runs.is_empty(), "wait for records");
    service.engine.as_mut().expect("engine").forge.iterate(Time::ZERO, Wall::EPOCH);
    for _ in 0..service.engine.as_mut().expect("engine").forge.io.len() {
        let Some(skein_io::Request::Listen { .. }) = service.engine.as_mut().expect("engine").forge.io.pop() else {
            panic!("no API connection without access")
        };
    }
    for _ in 0_u32..8 {
        crate::forge::records::advance(service.engine.as_mut().expect("engine"));
    }
    advance(service.engine.as_mut().expect("engine"));
    assert!(service.engine.as_ref().expect("engine").forge_records.done());
    assert_eq!(service.engine.as_ref().expect("engine").setup.runs.len(), 1);
    service
}
fn sent(service: &mut crate::Service, face: &mut Face, request: &[u8]) {
    face.sent(service);
    assert!(contains(&face.wire.received, request), "wanted {request:?}, got {:?}", face.wire.received);
    assert!(contains(&face.wire.received, HANDOVER), "setup uses the handover");
}
fn repository(service: &mut crate::Service, face: &mut Face, second: bool) {
    sent(service, face, b"GET /api/v1/user ");
    let run = *service.engine.as_mut().expect("engine").setup.runs.first().unwrap().0;
    assert_eq!(service.engine.as_ref().expect("engine").setup.runs.get(&run).unwrap().kind, Kind::Adopt);
    assert!(service.engine.as_ref().expect("engine").setup.runs.get(&run).unwrap().internal);
    assert_eq!(
        service.engine.as_ref().expect("engine").setup.lifecycle.adoptions.get(&1).unwrap().next.get(),
        if second { 2 } else { 1 }
    );
    assert_eq!(
        service.engine.as_mut().expect("engine").setup.answers.pop(),
        Some((run, host::Answer::Read(Some(Secret::new(HANDOVER.into())))))
    );
    face.reply(service, br#"{"id":1,"login":"temper-admin","is_admin":true}"#);
    sent(service, face, if second { b"GET /api/v1/repositories/2 " } else { b"GET /api/v1/repositories/1 " });
    face.reply(
        service,
        if second {
            br#"{"id":2,"owner":{"id":1,"login":"demo"},"name":"other","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":true,"allow_rebase":true,"html_url":"http://127.0.0.1:9/demo/widget"}"#
        } else {
            br#"{"id":1,"owner":{"id":1,"login":"demo"},"name":"widget","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":true,"allow_rebase":true,"html_url":"http://127.0.0.1:9/demo/widget"}"#
        },
    );
    sent(service, face, b"/collaborators/temper/permission ");
    face.reply(service, br#"{"permission":"read","user":{"id":42,"login":"temper"}}"#);
    sent(
        service,
        face,
        if second {
            b"PUT /api/v1/repos/demo/other/collaborators/temper "
        } else {
            b"PUT /api/v1/repos/demo/widget/collaborators/temper "
        },
    );
    assert!(contains(&face.wire.received, br#""permission":"write""#));
    assert!(service.engine.as_ref().expect("engine").forge.starts.is_empty());
    face.wire.write(b"HTTP/1.1 204 No Content\r\n\r\n");
    for _ in 0_u32..1024 {
        face.turn(service);
    }
}
#[test]
fn fake_repositories_are_adopted_in_order_then_connected_with_the_files_access() {
    for hooks in [false, true] {
        let mut service = service(hooks);
        let mut face = Face::new();
        for second in [false, true] {
            repository(&mut service, &mut face, second);
            if hooks {
                let run = *service.engine.as_mut().expect("engine").setup.runs.first().unwrap().0;
                assert_eq!(
                    service.engine.as_mut().expect("engine").setup.answers.pop(),
                    Some((run, host::Answer::Read(Some(Secret::new(HOOK.into())))))
                );
                sent(&mut service, &mut face, b"GET /api/v1/settings/api ");
                face.reply(&mut service, br#"{"max_response_items":50,"default_paging_num":30}"#);
                sent(&mut service, &mut face, b"/hooks?page=1&limit=32 ");
                face.reply(&mut service, b"[]");
                sent(
                    &mut service,
                    &mut face,
                    if second {
                        b"POST /api/v1/repos/demo/other/hooks "
                    } else {
                        b"POST /api/v1/repos/demo/widget/hooks "
                    },
                );
                assert!(contains(&face.wire.received, b"http://127.0.0.1:7101/hooks/1"));
                assert!(contains(&face.wire.received, HOOK));
                assert!(service.engine.as_ref().expect("engine").forge.starts.is_empty());
                face.reply(
                    &mut service,
                    br#"{"id":1,"config":{"url":"http://127.0.0.1:7101/hooks/1"},"events":["push"],"active":true}"#,
                );
            }
            assert_eq!(service.engine.as_ref().expect("engine").forge.starts.len(), u32::from(second));
        }
        assert!(service.engine.as_ref().expect("engine").setup.lifecycle.adoptions.is_empty());
        assert!(service.engine.as_ref().expect("engine").setup.runs.is_empty());
        let (forge, access) = service.engine.as_mut().expect("engine").forge.starts.get(0).unwrap();
        assert_eq!((*forge, access.user), (1, 42));
        assert_eq!(
            access.credential,
            skein_forge::credential::Credential::Token(Secret::new(b"api-token".as_slice().into()))
        );
        assert_eq!(access.webhook_secret, if hooks { Some(Secret::new(HOOK.into())) } else { None });
        assert!(!service.engine.as_ref().expect("engine").forge.io.is_empty(), "Connect starts the Forgejo user check");
        lifecycle::connected(service.engine.as_mut().expect("engine"), 1);
        advance(service.engine.as_mut().expect("engine"));
        assert!(service.engine.as_ref().expect("engine").setup.runs.is_empty(), "no fake Reconcile");
        let Some(root::Event::Forge(_)) = service.engine.as_mut().expect("engine").events.pop() else {
            panic!("Forgejo connected event")
        };
        assert!(service.engine.as_ref().expect("engine").workers.forges[0].api_token.is_none());
    }
}
#[test]
fn a_stopped_fake_adoption_names_its_step_and_drops_the_remaining_repositories_and_access() {
    let mut service = service(false);
    let mut face = Face::new();
    sent(&mut service, &mut face, b"GET /api/v1/user ");
    face.reply(&mut service, br#"{"id":1,"login":"person","is_admin":false}"#);
    let root::SetupEnd::Stopped { forge, reason } = terminal(&mut service) else { panic!("stopped adoption") };
    assert_eq!(forge, 1);
    assert_eq!(reason.as_ref(), b"setup Admin temper-admin: NotAdmin");
    assert!(service.engine.as_ref().expect("engine").setup.lifecycle.adoptions.is_empty());
    assert!(service.engine.as_ref().expect("engine").setup.runs.is_empty());
    assert!(service.engine.as_ref().expect("engine").forge.starts.is_empty());
    assert!(service.engine.as_ref().expect("engine").setup.io.is_empty());
    assert_eq!(service.engine.as_ref().expect("engine").domain.forge_connection(1), domain::Connection::NotConnected);
    assert!(service.end().is_none(), "the engine runs on");
}
#[test]
fn fake_given_uses_the_webhooks_listener_only_with_a_file_secret() {
    for hooks in [false, true] {
        let config = configured(hooks);
        let (endpoints, _) = crate::forge::prepare(&config, fixtures::forge_runtime()).unwrap();
        let (settings, _) = settings::fixed(&config, &endpoints).unwrap();
        let host::Kind::Forgejo(given) = &settings.forges.get(0).unwrap().kind else { panic!("Forgejo Given") };
        assert_eq!(given.listener, if hooks { Some(([127, 0, 0, 1], 7101).into()) } else { None });
        assert_eq!(given.bot_login.as_ref(), b"temper");
        assert!(given.adopted.is_empty());
    }
}
#[test]
fn owned_gets_forgejo_given_and_managed_stays_unserved_without_start_time_plans() {
    for managed in [false, true] {
        let mut config = configured(false);
        config.setup[0].handover = None;
        config.setup[0].entry.mode = if managed {
            Mode::Managed { admin_token: b"/data/admin".as_slice().into() }
        } else {
            Mode::Owned(temper_forge_setup::setup::Owned {
                orgs: Box::new([]),
                fetch: temper_forge_setup::setup::Fetch { forgejo: false, runner: false, mirror: None },
                forgejo: b"/bin/forgejo".as_slice().into(),
                runner: None,
                listen: ([127, 0, 0, 1], 9).into(),
                person: Account { login: b"owner".as_slice().into(), email: b"owner@example.test".as_slice().into() },
            })
        };
        let (mut endpoints, _) = crate::forge::prepare(&config, fixtures::forge_runtime()).unwrap();
        let setup = prepare(&config, &mut endpoints, &fixtures::seeds().crypto, 2).unwrap();
        assert!(setup.lifecycle.adoptions.is_empty());
        assert!(setup.runs.is_empty());
        assert!(endpoints.forges.get(0).unwrap().access.is_some());
        if managed {
            assert!(setup.served.is_empty());
            assert_eq!(*setup.unserved.get(&1).unwrap(), b"managed Forgejo setup is not staged yet");
        } else {
            let (settings, _) = setup.pending.unwrap();
            let host::Kind::Forgejo(_) = &settings.forges.get(0).unwrap().kind else { panic!("owned Forgejo Given") };
        }
        let mut service = crate::Service::engine(config, &fixtures::seeds(), None, fixtures::forge_runtime()).unwrap();
        service.engine.as_mut().expect("engine").store = fixtures::records_store();
        for _ in 0_u32..8 {
            crate::forge::records::advance(service.engine.as_mut().expect("engine"));
        }
        advance(service.engine.as_mut().expect("engine"));
        assert!(
            service.engine.as_ref().expect("engine").setup.runs.is_empty(),
            "no start-time writing plan for owned or managed"
        );
        assert!(service.engine.as_ref().expect("engine").setup.io.is_empty());
    }
}

#[test]
fn an_unserved_fake_drops_its_adoptions_and_names_the_exclusion_without_connecting() {
    let config = configured(false);
    let (mut endpoints, _) = crate::forge::prepare(&config, fixtures::forge_runtime()).unwrap();
    endpoints.forges.get_mut(0).unwrap().address = skein_io::Addr::Ip(([192, 0, 2, 7], 9).into());
    let setup = prepare(&config, &mut endpoints, &fixtures::seeds().crypto, 2).unwrap();
    assert!(endpoints.forges.get(0).unwrap().access.is_none());
    let mut service = service(false);
    service.engine.as_mut().expect("engine").setup = setup;
    let root::SetupEnd::Stopped { forge, reason } = terminal(&mut service) else { panic!("unserved fake") };
    assert_eq!(forge, 1);
    assert_eq!(reason.as_ref(), b"setup host serves plaintext only on loopback");
    assert!(service.engine.as_ref().expect("engine").setup.lifecycle.adoptions.is_empty());
    assert!(service.engine.as_ref().expect("engine").setup.runs.is_empty());
    assert!(service.engine.as_ref().expect("engine").forge.starts.is_empty());
    assert!(service.engine.as_ref().expect("engine").setup.io.is_empty());
}

#[test]
fn an_unresolved_fake_keeps_its_entry_but_suppresses_file_access_and_adoption() {
    let mut config = configured(false);
    config.forges[0].api = jig_protocol_config::Address::Host { name: b"fake.example".as_slice().into(), port: 9 };
    let mut runtime = fixtures::forge_runtime();
    runtime.get_mut(0).unwrap().address = crate::forge::HELD_ADDRESS;
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None, runtime).unwrap();
    let engine = service.engine.as_mut().unwrap();
    assert!(engine.setup.served.contains_key(&1));
    assert!(engine.setup.lifecycle.adoptions.is_empty());
    engine.forge_records.loaded(Box::new([]), None);
    for _ in 0_u32..8 {
        crate::forge::records::advance(engine);
    }
    assert_eq!(
        engine.notes.pop(),
        Some(crate::Note::ForgeNotConnected {
            number: 1,
            why: crate::NotConnected::Unreachable { host: b"fake.example".as_slice().into() }
        })
    );
    engine.forge.iterate(Time::ZERO, Wall::EPOCH);
    advance(engine);
    assert!(engine.forge.starts.is_empty());
    assert!(engine.setup.runs.is_empty());
    assert!(engine.setup.io.is_empty());
    for request in &engine.forge.io {
        if let skein_io::Request::Connect { .. } = request {
            panic!("the held forge never dials");
        }
    }
}

#[test]
fn an_unresolved_forgejos_asked_check_is_refused_before_its_unstaged_connected_path() {
    let mut config = configured(false);
    config.forges[0].api = jig_protocol_config::Address::Host { name: b"fake.example".as_slice().into(), port: 9 };
    let mut runtime = fixtures::forge_runtime();
    runtime.get_mut(0).unwrap().address = crate::forge::HELD_ADDRESS;
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None, runtime).unwrap();
    let engine = service.engine.as_mut().unwrap();
    engine.forge_records.loaded(Box::new([]), None);
    for _ in 0_u32..8 {
        crate::forge::records::advance(engine);
    }
    advance(engine);
    start(&mut service, 1, host::Plan::Check { forge: 1 });
    let root::SetupEnd::Stopped { forge, reason } = terminal(&mut service) else { panic!("held Check refused") };
    assert_eq!(forge, 1);
    assert_eq!(reason.as_ref(), b"unreachable: fake.example");
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.forge.starts.is_empty());
    assert!(engine.setup.runs.is_empty());
    assert!(engine.setup.io.is_empty());
}
