//! One loopback GitHub layer for the manifest page's browser smoke (client/browser.md, 4.4).
use skein_fake_forge_domain as domain;
use skein_fake_forge_github as layer;
use skein_fake_peers::forge::{Forge, Peer};
use skein_lib::{Duration, List, Secret};
use std::{cell::RefCell, net::SocketAddr, rc::Rc};

pub(super) fn peer(address: SocketAddr) -> Peer {
    let profile = jig_protocol_config::profile::profile(jig_protocol_config::profile::Profile::Small);
    let config = domain_config();
    let forge = Rc::new(RefCell::new(Forge::new(domain::Domain::new(&config, 23), config)));
    let mut accounts = List::with_capacity(1);
    accounts
        .push(layer::service::Account { login: b"acme".as_slice().into(), id: 10, organization: false })
        .expect("one account");
    let config = layer::Config {
        deliveries: layer::delivery::Config { log_entries: 1, ..layer::delivery::Config::standard() },
        rate_quota: 5000,
        rate_window: Duration::from_secs(3600),
        token_life: layer::installations::TOKEN_LIFE,
        base: b"http://127.0.0.1".as_slice().into(),
        blob: b"http://127.0.0.1".as_slice().into(),
        third_origin: None,
        blob_redirect: None,
        app: layer::app::App {
            id: 42,
            slug: b"unused".as_slice().into(),
            owner: 10,
            permissions: Box::new([]),
            events: Box::new([]),
            key: layer::keys::KEYS[0],
            webhook: Secret::new(b"fake-hook".as_slice().into()),
        },
        app_facts: layer::service::AppFacts::Complete,
        bot: 11,
        shown: layer::service::Shown::Ruleset,
        ruleset_id: 1,
        pull_requests_only: false,
        required_signatures: false,
        hook: b"https://unused.invalid/".as_slice().into(),
        repositories: List::with_capacity(0),
        accounts,
        person: b"acme".as_slice().into(),
        tokens: List::with_capacity(0),
        faults: layer::Faults { reset: 0, drop: 0, server_error: 0 },
    };
    let http = skein_http::server::Limits { body: 32768, response: 32768, ..profile.driver.server };
    let limits = layer::Limits {
        deliveries: layer::delivery::Limits { entries: 1, payload_bytes: 32768 },
        http,
        github: skein_forge::github::limits::Limits {
            json: skein_json::tokenizer::Limits { chunk: 1024, string: 32768, number: 32, depth: 16, length: 32768 },
            writer: skein_json::writer::Limits { depth: 16, length: 32768 },
            string_bytes: 1024,
            body_bytes: 32768,
            inner: 32,
            target_bytes: 4096,
        },
        installations: layer::installations::Limits { installations: 1, tokens: 1, listed: 1 },
        pages: layer::pages::Limits { states: 1, manifest_bytes: 8192 },
        page: 1,
        body: 32768,
        response: 32768,
        blob_records: 1,
        git_records: 1,
    };
    Peer::github(
        address,
        ([127, 0, 0, 1], 0).into(),
        skein_fake_peers::Limits {
            io: profile.io,
            connections: 2,
            queue: 64,
            plaintext: 65536,
            ciphertext: 65536,
            observations: 16,
            // The history story retains both manifest POSTs.
            observation_bytes: 8192,
        },
        config,
        limits,
        forge,
        23,
    )
    .expect("loopback GitHub peer")
}

// No repositories or git are used by this story; keep their admitted bounds small too.
fn domain_config() -> domain::Config {
    domain::Config {
        limits: domain::Limits {
            repositories: 1,
            users: 4,
            labels: 1,
            items: 1,
            comments: 1,
            dependencies: 1,
            reviews: 1,
            branches: 1,
            commits: 1,
            files: 1,
            statuses: 1,
            contexts: 1,
            bypassing_users: 0,
            pages: 1,
            name_bytes: 256,
            title_bytes: 16,
            body_bytes: 32,
            content_bytes: 32,
            page_size: 1,
            calls: 1,
            hooks: 1,
            observations: 16,
        },
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        settle_min: Duration::from_secs(2),
        settle_max: Duration::from_secs(2),
        late: 0,
        late_min: Duration::ZERO,
        late_max: Duration::ZERO,
        unavailable: 0,
        timeouts: 0,
        landing: 0,
        land_min: Duration::ZERO,
        land_max: Duration::ZERO,
        rate_limit: 0,
        rate_window: Duration::ZERO,
        ci: 12,
        hook_min: Duration::ZERO,
        hook_max: Duration::ZERO,
        hooks_late: 0,
        hooks_lost: 0,
        resolution: Duration::ZERO,
        skew: domain::Skew::None,
        status_updates: false,
        edit_updates: false,
    }
}
