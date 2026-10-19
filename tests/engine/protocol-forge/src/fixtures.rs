//! The component's small limits and configured forge.
use skein_lib::Duration;
use temper_engine_protocol_forge::Limits;
pub const WRITER: u64 = 2;
#[must_use]
pub fn limits() -> Limits {
    Limits {
        forges: 2,
        token_bytes: 64,
        connections: 2,
        head_bytes: 4096,
        headers: 32,
        body_bytes_in: 1_048_576,
        depth: 16,
        chunk: 1024,
        string_bytes: 128,
        body_bytes: 1045,
        inner: 8,
        target_bytes: 1024,
        repositories: 2,
        users: 8,
        deadline: Duration::from_secs(10),
        idle: Duration::from_secs(5),
        start_backoff: Duration::from_secs(1),
        start_backoff_max: Duration::from_secs(8),
        hooks: 4,
        hook_head_bytes: 8192,
        hook_body_bytes: 1_048_576,
        hook_deadline: Duration::from_secs(10),
        hook_secret_bytes: 128,
        tls: skein_tls::client::Limits { read: 16384, send: 16384, records: skein_tls::client::MAX_RECORD },
    }
}

use core::net::{Ipv4Addr, SocketAddr};
use skein_forge::api;
use skein_lib::{List, Secret};
use temper_engine_protocol_forge::{
    Settings, Shared,
    settings::{Forge, Kind, Transport},
};
pub const FORGE: u16 = 1;
pub const REPO: api::Repository = api::Repository { forge: 1, repository: 1 };
pub const TOKEN: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
#[must_use]
pub fn shared() -> Shared {
    Shared {
        client: skein_forge_client::Limits {
            pending: 4,
            calls: 2,
            rate: 100,
            reserve: 0,
            window: Duration::from_secs(10),
            op_bytes: 512,
            answer_bytes: 65_536,
            rows: 8,
            inbox: 16,
            read_attempts: 3,
            backoff: Duration::from_secs(1),
            backoff_max: Duration::from_secs(4),
            entries: 8,
            write_attempts: 3,
            lifetime: Duration::from_secs(10),
            clock_margin: Duration::from_secs(1),
            resources: 8,
            repositories: 2,
            poll: Duration::from_secs(5),
            poll_max: Duration::from_secs(30),
            hinted: Duration::from_secs(1),
            slow: Duration::from_secs(40),
            settle: Duration::from_secs(30),
            facts: 16,
        },
        name_bytes: 128,
        repositories: 2,
        io: skein_io::Limits {
            sockets: 8,
            names: 0,
            refusals: 2,
            intake: 32_768,
            receive: 4096,
            output: 32_768,
            sends: 4,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(1),
            retry: Duration::from_millis(10),
        },
    }
}
#[must_use]
pub fn settings() -> Settings {
    let mut forges = List::with_capacity(2);
    forges
        .push(Forge {
            number: FORGE,
            kind: Kind::Forgejo,
            address: skein_io::Addr::Ip(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))),
            transport: Transport::Plaintext,
            host: Box::from(&b"127.0.0.1:3000"[..]),
            credential: skein_forge::credential::Credential::Token(Secret::new(Box::from(TOKEN))),
            user: WRITER,
        })
        .expect("one forge");
    Settings { forges, webhooks: None, word: Box::from(b"temper".as_slice()) }
}
/// Makes a literal HTTP answer for scenarios independent of recorded cases.
#[must_use]
pub fn response(code: u16, body: &[u8]) -> Vec<u8> {
    let mut bytes = format!("HTTP/1.1 {code} Answer\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
/// One branch read after its repository resolution.
#[must_use]
pub fn branch() -> skein_forge::api::Op {
    api::Op::Read(api::Read::Branch { branch: Box::from(*b"main") })
}

/// The fake's HTTP and document bounds, matching the service's small profile.
#[must_use]
pub fn fake_limits() -> skein_fake_forge_protocol::Limits {
    skein_fake_forge_protocol::Limits {
        calls: 4,
        connections: 4,
        raw_bytes: 4096,
        http: skein_http::server::Limits {
            head: 4096,
            headers: 16,
            body: 16384,
            read: 1024,
            response: 4096,
            send: 1024,
        },
        documents: skein_forge::forgejo::limits::Limits {
            json: skein_json::tokenizer::Limits { chunk: 1024, string: 16384, number: 32, depth: 16, length: 65536 },
            writer: skein_json::writer::Limits { depth: 16, length: 65536 },
            string_bytes: 512,
            body_bytes: 16384,
            inner: 32,
            target_bytes: 4096,
        },
    }
}
/// The token's writer is user 1; the scenario's repository is id 2.
#[must_use]
pub fn protocol_config() -> skein_fake_forge_protocol::Config {
    use skein_fake_forge_protocol::{Config, Faults, Repository};
    let mut tokens = List::with_capacity(1);
    tokens.push((Secret::new(Box::from(TOKEN)), 1)).expect("one token");
    let mut users = List::with_capacity(2);
    users.push((1, Box::from(b"engine".as_slice()))).expect("user");
    users.push((2, Box::from(b"person".as_slice()))).expect("user");
    let mut repositories = List::with_capacity(1);
    repositories.push(Repository { id: 2, full: Box::from(b"org/repo".as_slice()), protected: None }).expect("repo");
    Config {
        base: Box::from(b"http://127.0.0.1:3000".as_slice()),
        tokens,
        users,
        repositories,
        max_response_items: 2,
        default_paging_num: 30,
        hooks: None,
        faults: Faults { reset: 0, drop: 0, server_error: 0 },
    }
}

/// The connector world's writer, matching the fake's API token.
#[must_use]
pub fn writer_settings() -> Settings {
    let mut settings = settings();
    settings.forges.get_mut(0).expect("forge").user = 1;
    settings
}
/// Component bounds use exactly the top's client limits.
#[must_use]
pub fn protocol_shared() -> Shared {
    Shared { client: temper_engine_forge_world::LIMITS.client, ..shared() }
}
#[must_use]
pub fn goal_view() -> temper_engine_domain_forge_issues::GoalView {
    use temper_engine_domain_forge_issues::{GoalView, Phase};
    GoalView {
        phase: Phase::Waiting,
        goal: 42,
        repository: 2,
        title: Box::from("Goal"),
        goal_text: Box::from("Plan"),
        plan: Box::new([]),
        milestones: Box::new([]),
        finished: None,
    }
}

/// Both byte-valued records' Debug and text-valued facts must hide the API token.
#[must_use]
pub fn token_absent(value: &impl core::fmt::Debug) -> bool {
    let text = format!("{value:?}");
    let bytes = format!("{TOKEN:?}");
    !text.as_bytes().windows(TOKEN.len()).any(|part| part == TOKEN) && !text.contains(&bytes[1..bytes.len() - 1])
}

/// Hook bounds are the shipped profile's five hook limits.
#[must_use]
pub fn hook_limits() -> Limits {
    let profile = temper_service::limits::profile(temper_service::limits::Profile::Small).forge_component;
    Limits {
        hooks: profile.hooks,
        hook_head_bytes: profile.hook_head_bytes,
        hook_body_bytes: profile.hook_body_bytes,
        hook_deadline: profile.hook_deadline,
        hook_secret_bytes: profile.hook_secret_bytes,
        ..limits()
    }
}
pub const HOOK_SECRET: &[u8] = b"fixture-hook-secret";
pub const PING: &[u8] = b"{\"zen\":\"a delivery of no row\"}";
#[must_use]
pub fn hook_settings() -> Settings {
    let mut settings = settings();
    let mut secrets = List::with_capacity(2);
    secrets.push((FORGE, Secret::new(Box::from(HOOK_SECRET)))).expect("secret");
    settings.webhooks = Some(temper_engine_protocol_forge::hooks::Webhooks {
        listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7200).into()),
        secrets,
    });
    settings
}
#[must_use]
pub fn hooked_protocol_config() -> skein_fake_forge_protocol::Config {
    let mut config = protocol_config();
    config.hooks = Some(skein_fake_forge_protocol::delivery::Endpoint {
        forge: FORGE,
        host: Box::from(b"127.0.0.1:7200".as_slice()),
        secret: Secret::new(Box::from(HOOK_SECRET)),
    });
    config
}
#[must_use]
pub fn push_hook(branch: &[u8], commit: u64) -> skein_fake_forge_domain::Request {
    use skein_fake_forge_domain::{Request, api::Change};
    Request::Hook {
        repository: Box::from(b"org/repo".as_slice()),
        change: Change::Push,
        number: None,
        branch: Some(Box::from(branch)),
        commit: Some(commit),
        by: Some(2),
        body: None,
        ended: None,
    }
}
