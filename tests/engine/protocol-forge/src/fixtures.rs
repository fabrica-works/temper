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
        tls: skein_tls::client::Limits { read: 16384, send: 16384, records: skein_tls::client::MAX_RECORD },
    }
}

use core::net::{Ipv4Addr, SocketAddr};
use skein_lib::{List, Secret};
use temper_engine_domain_forge_client::{self as client, api};
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
        client: client::Limits {
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
            token: Secret::new(Box::from(TOKEN)),
            user: WRITER,
        })
        .expect("one forge");
    Settings { forges }
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
pub fn branch() -> temper_engine_domain_forge_client::api::Op {
    api::Op::Read(api::Read::Branch { branch: Box::from(*b"main") })
}
