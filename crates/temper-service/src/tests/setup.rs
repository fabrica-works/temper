use super::*;
use crate::tests::fixtures;
use skein_forge_setup::github::manifest::Target;
use skein_lib::{List, Rng};

fn service(states: u32, life: Duration) -> crate::Service {
    let mut config = fixtures::engine();
    config.forges[0].kind = crate::settings::Kind::GitHub;
    config.forges[0].api = jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()));
    config.forges[0].web = Some(Box::from(b"http://127.0.0.1:3000".as_slice()));
    config.forges[0].api_token = None;
    config.forges[0].transport = crate::settings::Transport::Plaintext;
    config.limits.setup.states = states;
    config.limits.setup.state_life = life;
    fixtures::fill_files(&mut config);
    let mut runtime = List::with_capacity(1);
    runtime
        .push(crate::ForgeRuntime {
            number: 1,
            address: ([127, 0, 0, 1], 3000).into(),
            transport: skein_forge_connection::settings::Transport::Plaintext { name: None },
            trust_roots: 0,
            trust_root_bytes: 0,
        })
        .unwrap();
    let mut service = crate::Service::engine(config, &fixtures::seeds(), None, runtime).unwrap();
    service.engine.store = fixtures::records_store();
    assert!(service.engine.setup.component.is_none(), "no host before the rows load");
    for _ in 0_u32..8 {
        crate::forge::records::advance(&mut service.engine);
    }
    advance(&mut service.engine);
    assert!(service.engine.setup.component.is_some());
    assert!(service.engine.forge_records.done());
    service
}
fn start(service: &mut crate::Service, run: u64, plan: host::Plan) {
    let run = Token::new(run);
    service.engine.setup.staged.insert(run, plan).unwrap();
    crate::route::request(&mut service.engine, root::Request::Setup { run });
}
fn terminal(service: &mut crate::Service) -> root::SetupEnd {
    for _ in 0_u32..4 {
        advance(&mut service.engine);
    }
    let Some(root::Event::Setup { end, .. }) = service.engine.events.pop() else { panic!("setup terminal") };
    end
}
fn manifest(service: &mut crate::Service, run: u64) -> Box<[u8]> {
    start(service, run, host::Plan::Manifest { forge: 1, target: Target::Personal });
    let root::SetupEnd::Done { outcome: root::SetupOutcome::Manifest { state }, .. } = terminal(service) else {
        panic!("manifest")
    };
    state
}
fn row(service: &mut crate::Service, item: Item, bytes: &[u8]) {
    service.engine.forge_records.rows.push(
        records::number(1, item),
        Secret::new(records::encode(bytes).into_boxed_slice()),
        4096,
    );
}
#[test]
fn the_host_starts_after_loading_with_temper_urls_and_the_configured_limits() {
    let mut service = service(8, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    let page = service.setup_page(1, &state, Time::ZERO).unwrap();
    for expected in [
        b"temper-".as_slice(),
        b"/forges/1/github/redirect",
        b"/forges/1/github/setup",
        b"http://127.0.0.1:3000/settings/apps/new",
    ] {
        assert!(contains(page, expected));
    }
    assert_eq!(service.engine.setup.env.limits.runs, 8);
    assert_eq!(service.engine.setup.env.limits.states, 8);
    assert!(service.engine.setup.io.is_empty(), "manifest is local");
}
#[test]
#[expect(clippy::manual_map, reason = "the step subset uses explicit matches")]
fn reads_use_the_retained_row_past_its_version_and_absence_answers_none() {
    for present in [false, true] {
        let mut service = service(8, Duration::from_secs(3600));
        if present {
            row(&mut service, Item::AppSlug, b"app-fixture");
        }
        start(&mut service, 1, host::Plan::Install { forge: 1 });
        let end = terminal(&mut service);
        let (_, host::Answer::Read(value)) = service.engine.setup.answers.pop().unwrap() else { panic!("read answer") };
        assert_eq!(
            match &value {
                Some(value) => Some(value.expose().len()),
                None => None,
            },
            if present { Some(11) } else { None }
        );
        match end {
            root::SetupEnd::Done { outcome: root::SetupOutcome::Install { url, .. }, .. } => {
                assert!(url.starts_with(b"http://127.0.0.1:3000/apps/app-fixture/"));
            }
            root::SetupEnd::Stopped { reason, .. } => assert!(reason.ends_with(b"Missing")),
            root::SetupEnd::Done { .. } => panic!("install terminal"),
        }
        assert!(service.engine.setup.io.is_empty());
    }
}
#[test]
fn pages_expire_on_their_done_clock_and_reload_does_not_consume_them() {
    let mut service = service(2, Duration::from_secs(3));
    let state = manifest(&mut service, 1);
    for seconds in [0, 2, 2] {
        assert!(service.setup_page(1, &state, Time::ZERO.saturating_add(Duration::from_secs(seconds))).is_some());
    }
    assert!(service.setup_page(2, &state, Time::ZERO).is_none());
    assert!(service.setup_page(1, &state, Time::ZERO.saturating_add(Duration::from_secs(3))).is_none());
    service.engine.env.now = Time::ZERO.saturating_add(Duration::from_secs(3));
    advance(&mut service.engine);
    assert!(service.engine.setup.pages.is_empty());
    assert_eq!(service.engine.setup.next_deadline(), None);
}
#[test]
fn the_oldest_page_is_dropped_at_capacity() {
    let mut service = service(1, Duration::from_secs(3600));
    let first = manifest(&mut service, 1);
    let second = manifest(&mut service, 2);
    assert!(service.setup_page(1, &first, Time::ZERO).is_none());
    assert!(service.setup_page(1, &second, Time::ZERO).is_some());
}
#[test]
fn an_install_can_evict_a_manifests_state_while_its_page_lives() {
    let mut service = service(1, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    row(&mut service, Item::AppSlug, b"app-fixture");
    start(&mut service, 2, host::Plan::Install { forge: 1 });
    assert!(let_done(terminal(&mut service)));
    assert!(service.setup_page(1, &state, Time::ZERO).is_some());
    start(&mut service, 3, host::Plan::Convert { forge: 1, code: code(), state });
    let root::SetupEnd::Stopped { forge, reason } = terminal(&mut service) else { panic!("evicted state refused") };
    assert_eq!(forge, 1);
    assert!(reason.ends_with(b"Unknown"));
    assert!(service.engine.setup.io.is_empty(), "evicted state sends no conversion");
}
fn let_done(end: root::SetupEnd) -> bool {
    match end {
        root::SetupEnd::Done { .. } => true,
        root::SetupEnd::Stopped { .. } => false,
    }
}
fn code() -> Secret {
    Secret::new(Box::from(skein_lib::Decimal::of(Rng::new(805).next_u64()).as_bytes()))
}

struct Face {
    owner: Option<Token>,
    wire: skein_world::stream::Wire,
}
impl Face {
    fn new() -> Self {
        Self { owner: None, wire: skein_world::stream::Wire::new(7) }
    }
    fn turn(&mut self, service: &mut crate::Service) {
        for _ in 0_u32..service.engine.setup.io.len() {
            match service.engine.setup.io.pop().unwrap() {
                skein_io::Request::Connect { owner, .. } => {
                    self.owner = Some(owner);
                    self.wire = skein_world::stream::Wire::new(7);
                    service.engine.setup.up(skein_io::Event::Connecting { owner, socket: Token::new(123) });
                    service.engine.setup.up(skein_io::Event::Connected { owner });
                }
                skein_io::Request::Stream { down, .. } => self.wire.take(down),
                skein_io::Request::Close { .. } | skein_io::Request::Abort { .. } => {
                    service.engine.setup.up(skein_io::Event::Closed { owner: self.owner.take().unwrap() });
                }
                skein_io::Request::Listen { .. }
                | skein_io::Request::Bind { .. }
                | skein_io::Request::Reject { .. }
                | skein_io::Request::Output { .. }
                | skein_io::Request::Spawn { .. }
                | skein_io::Request::Signal { .. }
                | skein_io::Request::Usage { .. }
                | skein_io::Request::Window { .. } => panic!("setup socket request"),
            }
        }
        if let Some(owner) = self.owner
            && let Some(up) = self.wire.answer()
        {
            service.engine.setup.up(skein_io::Event::Stream { owner, up });
        }
        advance(&mut service.engine);
    }
    fn reply(&mut self, service: &mut crate::Service, body: &[u8]) {
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
        self.wire.write(&writer.finish());
        for _ in 0_u32..1024 {
            self.turn(service);
        }
    }
    fn sent(&mut self, service: &mut crate::Service) {
        for _ in 0_u32..1024 {
            self.turn(service);
        }
        assert!(contains(&self.wire.received, b"\r\n\r\n"), "request sent");
    }
}
fn writing() -> (crate::Service, Face) {
    let mut service = service(8, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    start(&mut service, 2, host::Plan::Convert { forge: 1, code: code(), state });
    let mut face = Face::new();
    face.sent(&mut service);
    let der = skein_fake_forge_github::keys::KEYS[0];
    let encoded =
        skein_lib::base64::encoded_len(skein_lib::base64::Alphabet::Standard, u32::try_from(der.len()).unwrap())
            .unwrap();
    let mut base64 = skein_lib::Writer::new(usize::try_from(encoded).unwrap());
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, der, &mut base64).unwrap();
    let prefix = br#"{"id":7,"slug":"app-fixture","owner":{"id":1,"login":"owner-fixture"},"pem":"-----BEGIN RSA PRIVATE KEY-----\n"#;
    let suffix = br#"\n-----END RSA PRIVATE KEY-----\n","client_id":"fixture","client_secret":"unused","webhook_secret":"unused"}"#;
    let mut body = skein_lib::Writer::new(
        prefix.len().checked_add(usize::try_from(encoded).unwrap()).unwrap().checked_add(suffix.len()).unwrap(),
    );
    body.put(prefix).unwrap();
    body.put(&base64.finish()).unwrap();
    body.put(suffix).unwrap();
    face.reply(&mut service, &body.finish());
    face.sent(&mut service);
    face.reply(&mut service, br#"{"id":7,"slug":"app-fixture"}"#);
    face.sent(&mut service);
    face.reply(&mut service, br#"{"id":9,"login":"app-fixture[bot]"}"#);
    if service.engine.setup.writes.is_empty() {
        if let Some(root::Event::Setup { end, .. }) = service.engine.events.pop() {
            panic!("conversion terminal: {end:?}");
        }
        panic!(
            "conversion pending: owner {}, incoming {}, outgoing {}",
            face.owner.is_some(),
            service.engine.setup.above.len(),
            service.engine.setup.io.len()
        );
    }
    (service, face)
}
#[test]
fn written_is_answered_only_after_kept_and_the_routing_rows_follow_each_keep() {
    let (mut service, mut face) = writing();
    for expected in [Item::AppId, Item::AppSlug, Item::AppKey, Item::User] {
        assert!(service.engine.setup.answers.is_empty(), "Written must wait for Kept");
        assert!(
            (match service.engine.forge_records.rows.forge(1) {
                Some(rows) => rows.get(expected).ok(),
                None => None,
            })
            .is_none(),
            "unkept row is invisible"
        );
        let owner = *service.engine.setup.writes.first().unwrap().0;
        let write = service.engine.setup.writes.get(&owner).unwrap();
        assert_eq!(write.number, records::number(1, expected));
        assert_eq!(write.value.expose().first(), Some(&records::VERSION));
        answer(&mut service.engine, StoreEvent::Kept { owner });
        assert_eq!(service.engine.setup.answers.pop(), Some((Token::new(2), host::Answer::Written)));
        service.engine.forge_records.rows.forge(1).unwrap().get(expected).unwrap();
        face.turn(&mut service);
    }
    assert!(let_done(terminal(&mut service)));
}
#[test]
fn a_failed_keep_stops_the_engine_and_never_answers_written() {
    let (mut service, _face) = writing();
    let owner = *service.engine.setup.writes.first().unwrap().0;
    let Some(StoreEvent::Kept { owner: kept }) = service.engine.store.above.pop() else {
        panic!("pending keep terminal")
    };
    assert_eq!(kept, owner);
    service.engine.store.above.push(StoreEvent::Corrupt { owner });
    crate::engine::iterate(&mut service.engine, Time::ZERO, Wall::EPOCH);
    assert!(service.end().is_some());
    assert!(service.engine.setup.answers.is_empty());
}
#[test]
fn draining_closes_the_host_and_waits_for_closed() {
    let mut service = service(8, Duration::from_secs(3600));
    let state = manifest(&mut service, 1);
    start(&mut service, 2, host::Plan::Convert { forge: 1, code: code(), state });
    let mut face = Face::new();
    face.sent(&mut service);
    crate::stop::shutdown(&mut service.engine);
    crate::stop::drained(&mut service.engine);
    advance(&mut service.engine);
    assert!(!service.engine.setup.closed, "Closed waits for the live socket");
    face.turn(&mut service);
    assert!(service.engine.setup.closed);
    assert!(service.engine.setup.pages.is_empty());
    assert_eq!(service.engine.setup.next_deadline(), None);
}
#[test]
fn a_tls_forge_is_refused_by_name_without_stopping_startup() {
    let mut service = service(8, Duration::from_secs(3600));
    let mut config = fixtures::engine();
    config.forges[0].kind = crate::settings::Kind::GitHub;
    config.forges[0].api_token = None;
    config.forges[0].transport =
        crate::settings::Transport::Tls { server_name: b"forge.example".as_slice().into(), trust: None };
    let (endpoints, _) = crate::forge::prepare(&config, fixtures::forge_runtime()).unwrap();
    service.engine.setup = prepare(&config, &endpoints, &fixtures::seeds().crypto, 2).unwrap();
    advance(&mut service.engine);
    assert!(service.engine.setup.served.is_empty());
    start(&mut service, 1, host::Plan::Manifest { forge: 1, target: Target::Personal });
    let root::SetupEnd::Stopped { forge, reason } = terminal(&mut service) else { panic!("TLS refusal") };
    assert_eq!(forge, 1);
    assert!(contains(&reason, b"no TLS at this pin"));
    assert!(service.end().is_none());
    assert!(service.engine.setup.io.is_empty());
}
#[test]
fn the_host_names_the_metadata_limits_and_rejects_bad_start_limits() {
    let mut service = service(8, Duration::from_secs(3600));
    let setup = &mut service.engine.setup;
    assert_eq!(setup.env.limits.inner, 32);
    let mut found = jig_protocol_config::Problems::empty();
    settings::problems(host::Unusable::GitHubMetadata { inner: 1, string_bytes: 1 }, &mut found);
    for (index, name) in [b"forge_component.inner".as_slice(), b"forge_component.string_bytes"].into_iter().enumerate()
    {
        assert_eq!(found.problems.get(u32::try_from(index).unwrap()).unwrap().path, settings::limit_path(name));
    }
}

fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    for part in bytes.windows(needle.len()) {
        if part == needle {
            return true;
        }
    }
    false
}
#[test]
fn setup_memory_figures_remain_separate_from_the_host_admission() {
    let mut config = fixtures::engine();
    config.forges[0].kind = crate::settings::Kind::GitHub;
    config.forges[0].api_token = None;
    let (endpoints, _) = crate::forge::prepare(&config, fixtures::forge_runtime()).unwrap();
    let setup = prepare(&config, &endpoints, &fixtures::seeds().crypto, 2).unwrap();
    let (settings, _) = setup.pending.unwrap();
    assert_eq!(settings.user_agent.as_ref(), crate::forge::USER_AGENT.as_bytes());
    let given = settings.forges.get(0).unwrap().github().unwrap();
    let machines = skein_forge_setup::github::worst_case(given, 8, 8).unwrap();
    let rows = records::Rows::worst_case(config.limits.records.value_bytes).unwrap();
    let pages = 8_u64.checked_mul(u64::from(given.limits.page_bytes)).unwrap();
    assert_eq!((machines, rows, pages), (630_744, 2_710_360_832, 131_072));
    assert_eq!(accounted(8, 8), accounted(1, 1), "runtime 55: no partial Host charge");
}
#[test]
fn every_host_unusable_arm_has_an_operator_path() {
    use host::{Stacked, Unusable};
    for why in [
        Unusable::Zero,
        Unusable::TokenBytes { token_bytes: 1 },
        Unusable::Documents(skein_forge::forgejo::limits::Problem::Depth),
        Unusable::GitHubDocuments(skein_forge::github::limits::Problem::Depth),
        Unusable::GitHubMetadata { inner: 1, string_bytes: 1 },
        Unusable::Stack { stacked: Stacked::HttpOverIo, demand: 2, cap: 1 },
        Unusable::Stack { stacked: Stacked::JsonOverHttp, demand: 2, cap: 1 },
        Unusable::Forges { forges: 1 },
        Unusable::Duplicate { forge: 1 },
        Unusable::Address { forge: 1 },
        Unusable::PlaintextName { forge: 1 },
        Unusable::Request { forge: 1 },
        Unusable::Given {
            forge: 1,
            refused: skein_forge_setup::given::Refused {
                place: skein_forge_setup::given::Place::Deadline,
                other: None,
            },
        },
        Unusable::GitHubGiven {
            forge: 1,
            refused: skein_forge_setup::github::given::Refused {
                place: skein_forge_setup::github::given::Place::Backoff,
                other: Some(skein_forge_setup::github::given::Place::BackoffMax),
            },
        },
        Unusable::Reachable { forge: 1, reachable: 2, inner: 1 },
    ] {
        let mut found = jig_protocol_config::Problems::empty();
        settings::problems(why, &mut found);
        assert!(!found.is_empty());
    }
}
#[test]
fn invalid_host_limits_stop_after_load_and_setup_overrides_are_checked() {
    let mut service = service(8, Duration::from_secs(3600));
    let mut config = fixtures::engine();
    config.forges[0].kind = crate::settings::Kind::GitHub;
    config.forges[0].api_token = None;
    config.limits.forge_component.inner = 13;
    let (endpoints, _) = crate::forge::prepare(&fixtures::engine(), fixtures::forge_runtime()).unwrap();
    service.engine.setup = prepare(&config, &endpoints, &fixtures::seeds().crypto, 2).unwrap();
    advance(&mut service.engine);
    assert!(service.end().is_some());
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    for name in [b"setup.states".as_slice(), b"setup.runs", b"setup.state_life_ms", b"setup.orgs"] {
        assert_eq!(crate::limits::apply(&mut own, name, 0), Err(crate::limits::Refused::Range));
    }
    assert_eq!(crate::limits::apply(&mut own, b"setup.runs", 3), Ok(()));
    assert_eq!(own.setup.runs, 3);
}

fn accounted(runs: u32, states: u32) -> u64 {
    let mut config = fixtures::engine();
    config.forges[0].kind = crate::settings::Kind::GitHub;
    config.forges[0].api_token = None;
    config.limits.setup.runs = runs;
    config.limits.setup.states = states;
    fixtures::fill_files(&mut config);
    crate::Service::engine(config, &fixtures::seeds(), None, fixtures::forge_runtime()).unwrap().worst_case()
}
