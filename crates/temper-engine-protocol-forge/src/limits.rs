//! Forge component limits (forge.md, sections 2, 3.5, 7 and 8).
use skein_lib::Duration;
use temper_engine_domain_forge_client as client;
use temper_forge_forgejo::limits as documents;

/// The component's limits (forge.md, 7), in temper's `limits` member as
/// `forge_component.<field>`; the profile `small`'s values are 14.1's.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// The forges served.
    pub forges: u32,
    /// The longest API token.
    pub token_bytes: u32,
    /// Forge connections in all: the client's `calls`.
    pub connections: u32,
    /// The largest response head read (HTTP's `head`).
    pub head_bytes: u32,
    /// The most fields of a response head (HTTP's own).
    pub headers: u32,
    /// The largest response body read (JSON's `length`): time and bytes, not memory.
    pub body_bytes_in: u32,
    /// The JSON nesting followed.
    pub depth: u32,
    /// The most bytes of a string's text demanded at once (JSON's own).
    pub chunk: u32,
    /// The longest name, title, context, description, link or path kept.
    pub string_bytes: u32,
    /// The longest body kept whole: at least twice the client's `op_bytes` and 21.
    pub body_bytes: u32,
    /// The most elements kept of a list inside one document (06's `inner`).
    pub inner: u32,
    /// The longest request target (06's `target_bytes`).
    pub target_bytes: u32,
    /// The repositories' table: at most the forge's `repositories`.
    pub repositories: u32,
    /// The users' table.
    pub users: u32,
    /// A call's, at most the client's `lifetime`; also each start read's.
    pub deadline: Duration,
    /// How long an idle connection is kept, below the forge's keep-alive timeout.
    pub idle: Duration,
    /// A forge's start asked again after this, doubling to `start_backoff_max`.
    pub start_backoff: Duration,
    pub start_backoff_max: Duration,
    /// TLS's own.
    pub tls: skein_tls::client::Limits,
}

/// What the component is given of its neighbours' limits (forge.md, 12, "Shared limits").
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Shared {
    /// The forge client's: `calls`, `rows`, `answer_bytes`, `op_bytes`, `lifetime`.
    pub client: client::Limits,
    /// The forge's `name_bytes` (`temper_engine_domain_forge::Limits`).
    pub name_bytes: u32,
    /// The forge's `repositories`.
    pub repositories: u32,
    /// io's caps, which the stacked machines' demands must fit.
    pub io: skein_io::Limits,
}

/// The listings whose page size the component chooses (forge.md, 3.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Listing {
    Items,
    Reviews,
    Statuses,
    Remarks,
    Branches,
    Collaborators,
    PullFiles,
}

/// A machine stacked on another, whose demand must fit the other's cap.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Stacked {
    HttpOverTls,
    HttpOverIo,
    TlsOverIo,
    JsonOverHttp,
}

/// Why the component cannot start (configuration.md, 7: "each failure naming both limits").
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unusable {
    /// A limit is 0, or the worst case passes a `u64`.
    Zero,
    /// `connections` is not the client's `calls`.
    Connections { connections: u32, calls: u32 },
    /// `deadline` passes the client's `lifetime`.
    Deadline { deadline: Duration, lifetime: Duration },
    /// `body_bytes` below the largest body temper writes, its marker included.
    BodyBytes { body_bytes: u32, needed: u64 },
    /// `string_bytes` below the forge's `name_bytes`.
    StringBytes { string_bytes: u32, name_bytes: u32 },
    /// `repositories` above the forge's.
    Repositories { repositories: u32, forge: u32 },
    /// One of the listing's largest rows does not fit the client's `answer_bytes`.
    PageSize { listing: Listing },
    /// The documents' limits refused (06's `limits::check`).
    Documents(documents::Problem),
    /// A stacked machine's demand past the cap of the stream below.
    Stack { stacked: Stacked, demand: u32, cap: u32 },
    /// More forges than `forges`.
    Forges { forges: u32 },
    /// Two forges with one number.
    Duplicate { forge: u16 },
    /// An API token empty or past `token_bytes`.
    Token { forge: u16 },
    /// A user of 0.
    User { forge: u16 },
    /// Plaintext to an address not on loopback, or an in-process name.
    Address { forge: u16 },
    /// The longest request head (target, `Host`, token, agent) past HTTP's `request`.
    Request { forge: u16 },
}
