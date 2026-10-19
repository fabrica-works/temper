//! Forge component settings (protocol/forge.md, sections 2, 3.5, 7 and 8).
use alloc::boxed::Box;
use skein_io::Addr;
use skein_lib::List;

/// What the component is told at start (protocol/forge.md, 2): one entry per
/// configured forge.
#[derive(Debug)]
pub struct Settings {
    /// At most `Limits::forges`, numbers unique.
    pub forges: List<Forge>,
    /// The listener and secrets, present together (protocol/forge.md, 5).
    pub webhooks: Option<crate::hooks::Webhooks>,
    /// Owner word used in markers and webhook hints, at most 16 bytes.
    pub word: Box<[u8]>,
}

/// One forge, from `application.forges` (protocol/configuration.md, 3) and
/// what the binary resolved.
#[derive(Debug)]
pub struct Forge {
    /// The domain's forge number: a repository's `forge`.
    pub number: u16,
    pub kind: Kind,
    /// The API's socket address, resolved before the loop; never an in-process name.
    pub address: Addr,
    pub transport: Transport,
    /// `Host`'s value, made by `skein_forge::host::host`.
    pub host: Box<[u8]>,
    /// The API token, in `Authorization: token <t>`: never logged, granted or stored.
    pub credential: skein_forge::credential::Credential,
    /// The forge user the token belongs to, checked by the start's `GET /api/v1/user`.
    pub user: u64,
}

/// The documents a forge speaks.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    /// The pinned Forgejo's API under `/api/v1`.
    Forgejo,
}

/// A forge's connections.
#[derive(Debug)]
pub enum Transport {
    /// Only to a socket address on loopback.
    Plaintext,
    /// TLS with the configured server name and trust.
    Tls { server_name: skein_tls::Name, trust: skein_tls::Config },
}
