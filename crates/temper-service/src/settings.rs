//! Typed engine and worker settings; startup supplies the values.
use alloc::boxed::Box;
use skein_lib::{List, Secret};

/// A forge temper works on (configuration.md, section 3, `forges`).
#[derive(Debug)]
pub struct Forge {
    pub number: u16,
    pub kind: Kind,
    /// Where the forge component connects, never an in-process name.
    pub api: jig_protocol_config::Address,
    pub transport: Transport,
    /// The base of each remote and the host of the forge's resource paths.
    pub git: Box<[u8]>,
    /// The API token, read from its private file: the forge component's alone.
    pub api_token: Secret,
    /// The forge user the API token belongs to: the forge writer.
    pub user: u64,
    /// The number of this forge's git identity.
    pub identity: u32,
}

/// The documents a forge speaks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Forgejo,
}

/// A forge component's connections.
#[derive(Debug)]
pub enum Transport {
    /// Only to a socket address on loopback.
    Plaintext,
    /// The trust file's bytes, or none for the machine's roots.
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
}

/// A git identity (configuration.md, section 3, `identities`).
#[derive(Debug)]
pub struct Identity {
    pub number: u32,
    pub token: Secret,
    pub login: Box<[u8]>,
    pub author: Author,
}

/// The author and committer of the commits made with an identity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Author {
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
}

/// The private files jig's parts name, by path, as jig's reference takes them.
#[derive(Debug)]
pub struct Files {
    pub entries: List<(Box<[u8]>, Secret)>,
}

/// The seeds the shell draws at start, as jig's reference takes them.
#[derive(Debug)]
pub struct Seeds {
    pub deployment: [u8; 16],
    pub seed: u64,
    pub crypto: Secret,
}

/// The engine's settings: jig's part, typed by jig-protocol-config, and
/// temper's members (configuration.md, sections 2 and 3).
#[derive(Debug)]
pub struct Engine {
    pub jig: jig_protocol_config::Engine,
    pub files: Files,
    pub forges: Box<[Forge]>,
    pub identities: Box<[Identity]>,
    pub landing: temper_engine_domain::LandingPolicy,
    pub charter: u32,
    pub limits: super::limits::Limits,
}

/// The worker's agent profile, which the binary reads from smith's
/// agent configuration, as jig's reference service's `Agent`
/// (`lib.rs:36-41`); 09 builds the worker's host half from it.
#[derive(Debug)]
pub struct Agent {
    pub endpoints: smith_protocol_channel::Endpoints,
    pub limits: smith_host_protocol::Limits,
    pub bounds: jig_domain_host::Bounds,
    pub host_limits: smith_host_domain::Limits,
}

/// A worker's settings: jig's part and temper's (configuration.md,
/// section 4); 09 builds the worker from it.
#[derive(Debug)]
pub struct Worker {
    pub jig: jig_protocol_config::Worker,
    /// The worker's root directory: workspaces, trash and home under it.
    pub root: Box<[u8]>,
    /// The absolute path of the git program.
    pub git: Box<[u8]>,
}
