//! Temper's members, read strictly with jig's readers (protocol/configuration.md, 3, 4, 10).
mod landing;
mod read;
use super::{Problems, What};
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind};
use skein_lib::{List, bytes};

#[derive(Debug)]
pub struct Members {
    pub clients: Clients,
    pub webhooks: Option<Webhooks>,
    pub forges: Box<[Forge]>,
    pub identities: Box<[Identity]>,
    pub landing: temper_engine_domain::LandingPolicy,
    /// The landing's projects in the document's order, which names a refused one's path.
    pub landing_projects: Box<[u32]>,
    pub charter: u32,
    pub limits: limits::Limits,
    pub overrides: Box<[(Box<[u8]>, u64)]>,
    pub workers: Box<[Worker]>,
}
/// The application's clients listener (protocol/configuration.md, section 3).
#[derive(Debug)]
pub struct Clients {
    /// The socket address used for the listener, Host guard and printed links.
    pub listener: config::Address,
    /// `application.clients.bundle`: the bundle's directory; `None` serves no browser.
    pub bundle: Option<Box<[u8]>>,
}
/// The optional webhook listener (protocol/configuration.md, section 3).
#[derive(Debug)]
pub struct Webhooks {
    pub listener: config::Address,
}
/// Read the listener with jig's address reader, refusing non-socket forms; and `bundle` when asked (the clients').
#[must_use]
pub fn clients(document: &Document, at: u32, path: &Path, bundle: bool, problems: &mut Problems) -> Option<Clients> {
    let mut object = read::object(document, at, path, problems)?;
    let at = read::required(&mut object, document, b"listener", path, problems);
    let mut directory = None;
    if bundle
        && let Some(at) = read::required(&mut object, document, b"bundle", path, problems)
        && document.token(at)?.kind != Kind::Null
    {
        match config::read::text(document, at, 1_048_576) {
            Ok(value) => {
                if !value.starts_with(b"/") {
                    problems.push(path.member(b"bundle"), What::Absolute);
                }
                directory = Some(value);
            }
            Err(what) => problems.push(path.member(b"bundle"), What::Jig(what)),
        }
    }
    read::finish(object, document, path, problems);
    let path = path.member(b"listener");
    let mut found = config::Problems::empty();
    let listener = config::address::read(document, at?, &path, &mut found);
    problems.append(found, &Path::root());
    let listener = listener?;
    let socket = match listener {
        config::Address::Addr(skein_io::Addr::Ip(_)) => true,
        config::Address::Addr(skein_io::Addr::InProcess(_)) | config::Address::Host { .. } => false,
    };
    read::form(socket, path, problems);
    Some(Clients { listener, bundle: directory })
}
#[derive(Debug)]
pub struct Forge {
    pub number: u16,
    pub kind: settings::Kind,
    pub api: config::Address,
    pub transport: Transport,
    pub git: Box<[u8]>,
    pub api_token: Box<[u8]>,
    pub webhook_secret: Option<Box<[u8]>>,
    pub user: User,
    pub identity: u32,
}
/// How a forge's account id is supplied.
#[derive(Debug, PartialEq, Eq)]
pub enum User {
    /// An explicit positive account id.
    Id(u64),
    /// A private file holding the decimal account id.
    File(Box<[u8]>),
}
/// Twenty decimal digits and one final line feed.
pub const USER_BYTES: u32 = 21;
#[derive(Debug)]
pub enum Transport {
    Plaintext,
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
}
#[derive(Debug)]
pub struct Identity {
    pub number: u32,
    pub token: Box<[u8]>,
    pub login: Box<[u8]>,
    pub author: settings::Author,
}
#[derive(Debug)]
pub struct Worker {
    pub root: Box<[u8]>,
    pub git: Option<Box<[u8]>>,
}
fn configured_limits(
    document: &Document,
    at: u32,
    path: &Path,
    problems: &mut Problems,
    handed: &mut List<(Box<[u8]>, u64)>,
) -> Option<limits::Limits> {
    let mut object = read::object(document, at, path, problems)?;
    let profile = read::text(&mut object, document, b"profile", path, problems);
    let overrides = read::required(&mut object, document, b"overrides", path, problems);
    read::finish(object, document, path, problems);
    if let Some(profile) = profile
        && profile.as_ref() != b"small"
    {
        problems.push(path.member(b"profile"), What::Profile);
    }
    let mut limits = limits::profile(limits::Profile::Small);
    let path = path.member(b"overrides");
    if let Some(at) = overrides
        && let Some(mut object) = read::object(document, at, &path, problems)
    {
        let end = config::read::after(document, at).ok()?;
        for index in at..end {
            let Some(token) = document.token(index) else { continue };
            if token.kind != Kind::Key {
                continue;
            }
            let name = document.text(token).expect("object key text");
            let Some(value_at) = object.member(document, name) else { continue };
            match config::read::integer(document, value_at, u64::MAX) {
                Ok(value) => {
                    if limits::handed_on(name) {
                        handed.push((bytes::copy_of(name), value)).expect("bounded overrides");
                    } else if let Err(what) = limits::apply(&mut limits, name, value) {
                        problems.push(path.member(name), What::Override(what));
                    }
                }
                Err(config::What::Bound) => problems.push(path.member(name), What::Override(limits::Refused::Range)),
                Err(what) => problems.push(path.member(name), What::Jig(what)),
            }
        }
        read::finish(object, document, &path, problems);
    }
    Some(limits)
}
fn transport(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Transport> {
    if document.token(at)?.kind == Kind::String {
        let value = match config::read::text(document, at, 16) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.clone(), What::Jig(what));
                return None;
            }
        };
        read::form(value.as_ref() == b"plaintext", path.clone(), problems);
        return Some(Transport::Plaintext);
    }
    let mut outer = read::object(document, at, path, problems)?;
    let tls = read::required(&mut outer, document, b"tls", path, problems);
    read::finish(outer, document, path, problems);
    let path = path.member(b"tls");
    let mut object = read::object(document, tls?, &path, problems)?;
    let server_name = read::text(&mut object, document, b"server_name", &path, problems);
    let at = read::required(&mut object, document, b"trust", &path, problems);
    let mut trust = None;
    if let Some(at) = at
        && document.token(at)?.kind != Kind::Null
    {
        match config::read::text(document, at, 1_048_576) {
            Ok(value) => trust = Some(value),
            Err(what) => problems.push(path.member(b"trust"), What::Jig(what)),
        }
    }
    read::finish(object, document, &path, problems);
    Some(Transport::Tls { server_name: server_name?, trust })
}
fn user(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<User> {
    if document.token(at)?.kind != Kind::ObjectStart {
        return match config::read::integer(document, at, u64::MAX) {
            Ok(value) => {
                read::form(value > 0, path.clone(), problems);
                Some(User::Id(value))
            }
            Err(what) => {
                problems.push(path.clone(), What::Jig(what));
                None
            }
        };
    }
    let mut object = read::object(document, at, path, problems)?;
    let file = read::text(&mut object, document, b"file", path, problems);
    read::finish(object, document, path, problems);
    if let Some(file) = &file {
        read::form(!file.is_empty(), path.member(b"file"), problems);
        if file.is_empty() {
            return None;
        }
    }
    Some(User::File(file?))
}
#[expect(clippy::disallowed_methods, reason = "the configuration protocol parses URL hosts and ports")]
fn git_base(input: &[u8]) -> bool {
    let (rest, plain) = if let Some(rest) = input.strip_prefix(b"https://") {
        (rest, false)
    } else if let Some(rest) = input.strip_prefix(b"http://") {
        (rest, true)
    } else {
        return false;
    };
    if rest.is_empty() || rest.ends_with(b"/") {
        return false;
    }
    for byte in rest {
        if byte.is_ascii_control() || byte.is_ascii_whitespace() || b"@?#\\".contains(byte) {
            return false;
        }
    }
    let mut end = rest.len();
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b'/' {
            end = index;
            break;
        }
    }
    let Ok(authority) = core::str::from_utf8(rest.get(..end).expect("host prefix")) else { return false };
    let (host, port) = if let Some(ipv6) = authority.strip_prefix('[') {
        let Some((host, suffix)) = ipv6.split_once(']') else { return false };
        if suffix.is_empty() {
            (host, None)
        } else if let Some(port) = suffix.strip_prefix(':') {
            (host, Some(port))
        } else {
            return false;
        }
    } else if let Some((host, port)) = authority.split_once(':') {
        (host, Some(port))
    } else {
        (authority, None)
    };
    if host.is_empty() {
        return false;
    }
    if let Some(port) = port
        && port.parse::<u16>().is_err()
    {
        return false;
    }
    match host.parse::<core::net::IpAddr>() {
        Ok(ip) => !plain || ip.is_loopback(),
        Err(_) => {
            for byte in host.bytes() {
                if !byte.is_ascii_alphanumeric() && byte != b'-' && byte != b'.' {
                    return false;
                }
            }
            !plain || host.eq_ignore_ascii_case("localhost")
        }
    }
}
fn forge(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Forge> {
    let mut object = read::object(document, at, path, problems)?;
    let number = read::short(&mut object, document, b"number", path, problems);
    let kind = read::text(&mut object, document, b"kind", path, problems);
    let at = read::required(&mut object, document, b"api", path, problems);
    let mut found = config::Problems::empty();
    let api = match at {
        Some(at) => config::address::read(document, at, &path.member(b"api"), &mut found),
        None => None,
    };
    problems.append(found, &Path::root());
    let at = read::required(&mut object, document, b"transport", path, problems);
    let transport = match at {
        Some(at) => transport(document, at, &path.member(b"transport"), problems),
        None => None,
    };
    let git = read::text(&mut object, document, b"git", path, problems);
    let api_token = read::text(&mut object, document, b"api_token", path, problems);
    let at = read::required(&mut object, document, b"webhook_secret", path, problems);
    let mut webhook_secret = None;
    if let Some(at) = at
        && document.token(at)?.kind != Kind::Null
    {
        match config::read::text(document, at, 1_048_576) {
            Ok(value) => webhook_secret = Some(value),
            Err(what) => problems.push(path.member(b"webhook_secret"), What::Jig(what)),
        }
    }
    let at = read::required(&mut object, document, b"user", path, problems);
    let user = match at {
        Some(at) => user(document, at, &path.member(b"user"), problems),
        None => None,
    };
    let identity = read::number(&mut object, document, b"identity", path, problems);
    read::finish(object, document, path, problems);
    if let Some(kind) = kind {
        read::form(kind.as_ref() == b"forgejo", path.member(b"kind"), problems);
    }
    if let Some(git) = &git {
        read::form(git_base(git), path.member(b"git"), problems);
    }
    let api = api?;
    let transport = transport?;
    match &api {
        config::Address::Addr(skein_io::Addr::InProcess(_)) => read::form(false, path.member(b"api"), problems),
        config::Address::Addr(skein_io::Addr::Ip(_)) | config::Address::Host { .. } => {}
    }
    match &transport {
        Transport::Plaintext => {
            let loopback = match &api {
                config::Address::Addr(skein_io::Addr::Ip(socket)) => socket.ip().is_loopback(),
                config::Address::Addr(skein_io::Addr::InProcess(_)) | config::Address::Host { .. } => false,
            };
            if !loopback {
                problems.push(path.member(b"transport"), What::Loopback);
            }
        }
        Transport::Tls { .. } => {}
    }
    Some(Forge {
        number: number?,
        kind: settings::Kind::Forgejo,
        api,
        transport,
        git: git?,
        api_token: api_token?,
        webhook_secret,
        user: user?,
        identity: identity?,
    })
}
fn identity(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Identity> {
    let mut object = read::object(document, at, path, problems)?;
    let number = read::number(&mut object, document, b"number", path, problems);
    let token = read::text(&mut object, document, b"token", path, problems);
    let login = read::text(&mut object, document, b"login", path, problems);
    let author_at = read::required(&mut object, document, b"author", path, problems);
    read::finish(object, document, path, problems);
    if let Some(login) = &login {
        let mut valid = !login.is_empty();
        for byte in login {
            if *byte == b':' || byte.is_ascii_control() {
                valid = false;
            }
        }
        read::form(valid, path.member(b"login"), problems);
    }
    let author_path = path.member(b"author");
    let mut author = read::object(document, author_at?, &author_path, problems)?;
    let name = read::text(&mut author, document, b"name", &author_path, problems);
    let email = read::text(&mut author, document, b"email", &author_path, problems);
    read::finish(author, document, &author_path, problems);
    for (field, value) in [(b"name".as_slice(), &name), (b"email".as_slice(), &email)] {
        if let Some(value) = value {
            let mut valid = !value.is_empty();
            for byte in value {
                if b"\0\n<>".contains(byte) {
                    valid = false;
                }
            }
            read::form(valid, author_path.member(field), problems);
        }
    }
    Some(Identity {
        number: number?,
        token: token?,
        login: login?,
        author: settings::Author { name: name?, email: email? },
    })
}
fn references(
    forges: &List<Forge>,
    identities: &List<Identity>,
    charter: u32,
    jig: &config::Engine,
    problems: &mut Problems,
) {
    let path = Path::root().member(b"application");
    for (index, forge) in forges.iter().enumerate() {
        let path = read::index(&path.member(b"forges"), index);
        for previous in forges.iter().take(index) {
            if previous.number == forge.number {
                problems.push(path.member(b"number"), What::Jig(config::What::Unique));
            }
        }
        let mut found = false;
        for identity in identities {
            if identity.number == forge.identity {
                found = true;
            }
        }
        if !found {
            problems.push(path.member(b"identity"), What::Jig(config::What::Refers));
        }
    }
    for (index, identity) in identities.iter().enumerate() {
        let path = read::index(&path.member(b"identities"), index);
        for previous in identities.iter().take(index) {
            if previous.number == identity.number {
                problems.push(path.member(b"number"), What::Jig(config::What::Unique));
            }
        }
        let mut count = 0_u32;
        for forge in forges {
            if forge.identity == identity.number {
                count = count.saturating_add(1);
            }
        }
        if count != 1 {
            problems.push(path.member(b"number"), What::Jig(config::What::Refers));
        }
    }
    {
        let mut found = false;
        for entry in &jig.charters {
            if entry.number == charter {
                found = true;
            }
        }
        if !found {
            problems.push(path.member(b"charter"), What::Jig(config::What::Refers));
        }
    }
}
fn workers(
    document: &Document,
    ats: Option<List<u32>>,
    count: u32,
    directories: &config::Directories,
    path: &Path,
    problems: &mut Problems,
) -> List<Worker> {
    let mut workers = List::with_capacity(count);
    if let Some(ats) = ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"workers"), index);
            let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
            let root = read::text(&mut object, document, b"root", &path, problems);
            let at = read::required(&mut object, document, b"git", &path, problems);
            let mut git = None;
            if let Some(at) = at {
                let null = match document.token(at) {
                    Some(token) => token.kind == Kind::Null,
                    None => false,
                };
                if !null {
                    match config::read::text(document, at, 1_048_576) {
                        Ok(value) => git = Some(value),
                        Err(what) => problems.push(path.member(b"git"), What::Jig(what)),
                    }
                }
            }
            read::finish(object, document, &path, problems);
            if let Some(root) = root {
                if let Some(git) = &git
                    && !git.starts_with(b"/")
                {
                    problems.push(path.member(b"git"), What::Absolute);
                }
                if let Some(root) = joined(directories, config::Directory::Data, &root, &path.member(b"root"), problems)
                {
                    workers.push(Worker { root, git }).expect("bounded workers");
                }
            }
        }
    }
    workers
}
pub fn parse(
    document: &Document,
    at: u32,
    jig: &config::Engine,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Members> {
    let path = Path::root().member(b"application");
    let mut object = read::object(document, at, &path, problems)?;
    let mut overrides = List::with_capacity(document.len());
    read::required(&mut object, document, b"setup", &path, problems);
    let limit_at = read::required(&mut object, document, b"limits", &path, problems);
    let limits = match limit_at {
        Some(at) => configured_limits(document, at, &path.member(b"limits"), problems, &mut overrides),
        None => None,
    };
    let bounds = limits.unwrap_or(limits::profile(limits::Profile::Small));
    let forge_ats = read::list(&mut object, document, b"forges", bounds.forge.client.repositories, &path, problems);
    let identity_ats =
        read::list(&mut object, document, b"identities", jig.limits.core.accounts.credentials, &path, problems);
    let workers_ats = read::list(&mut object, document, b"workers", jig.limits.core.fleet.workers, &path, problems);
    let charter = read::number(&mut object, document, b"charter", &path, problems);
    let clients_at = read::required(&mut object, document, b"clients", &path, problems);
    let clients = match clients_at {
        Some(at) => clients(document, at, &path.member(b"clients"), true, problems),
        None => None,
    };
    let landing_at = read::required(&mut object, document, b"landing", &path, problems);
    let at = read::required(&mut object, document, b"webhooks", &path, problems);
    let mut webhooks = None;
    if let Some(at) = at
        && document.token(at)?.kind != Kind::Null
        && let Some(value) = self::clients(document, at, &path.member(b"webhooks"), false, problems)
    {
        webhooks = Some(Webhooks { listener: value.listener });
    }
    read::finish(object, document, &path, problems);
    let forge_count = match &forge_ats {
        Some(ats) => ats.len(),
        None => 0,
    };
    let identity_count = match &identity_ats {
        Some(ats) => ats.len(),
        None => 0,
    };
    if forge_ats.is_some() && forge_count == 0 {
        problems.push(path.member(b"forges"), What::Jig(config::What::Bound));
    }
    if identity_ats.is_some() && forge_ats.is_some() && identity_count != forge_count {
        problems.push(path.member(b"identities"), What::Jig(config::What::Bound));
    }
    let mut forges = List::with_capacity(bounds.forge.client.repositories);
    if let Some(ats) = forge_ats {
        for (index, at) in ats.iter().enumerate() {
            if let Some(mut forge) = forge(document, *at, &read::index(&path.member(b"forges"), index), problems) {
                resolve_forge(directories, &mut forge, &read::index(&path.member(b"forges"), index), problems);
                forges.push(forge).expect("bounded forges");
            }
        }
    }
    let mut identities = List::with_capacity(jig.limits.core.accounts.credentials);
    if let Some(ats) = identity_ats {
        for (index, at) in ats.iter().enumerate() {
            if let Some(mut identity) =
                identity(document, *at, &read::index(&path.member(b"identities"), index), problems)
            {
                if let Some(value) = joined(
                    directories,
                    config::Directory::Data,
                    &identity.token,
                    &read::index(&path.member(b"identities"), index).member(b"token"),
                    problems,
                ) {
                    identity.token = value;
                }
                identities.push(identity).expect("bounded identities");
            }
        }
    }
    let workers = workers(document, workers_ats, jig.limits.core.fleet.workers, directories, &path, problems);
    if let Some(charter) = charter
        && forges.len() == forge_count
        && identities.len() == identity_count
    {
        references(&forges, &identities, charter, jig, problems);
    }
    let landing = match landing_at {
        Some(at) => landing::parse(document, at, jig, &bounds, problems),
        None => None,
    };
    if !problems.is_empty() {
        return None;
    }
    let (landing, landing_projects) = landing?;
    Some(Members {
        clients: clients?,
        webhooks,
        forges: forges.into_boxed(),
        identities: identities.into_boxed(),
        workers: workers.into_boxed(),
        charter: charter?,
        limits: limits?,
        overrides: overrides.into_boxed(),
        landing,
        landing_projects,
    })
}
pub fn files(members: &Members, api_token_bytes: u32, token_bytes: u32) -> Box<[config::File]> {
    let count = members
        .forges
        .len()
        .checked_mul(4)
        .expect("bounded forges")
        .checked_add(members.identities.len())
        .expect("bounded files");
    let mut files = List::with_capacity(u32::try_from(count).expect("bounded files"));
    for (index, forge) in members.forges.iter().enumerate() {
        let path = read::index(&Path::root().member(b"application").member(b"forges"), index);
        files
            .push(config::File {
                at: path.member(b"api_token"),
                path: forge.api_token.clone(),
                max: api_token_bytes,
                private: true,
            })
            .expect("bounded files");
        if let User::File(name) = &forge.user {
            files
                .push(config::File {
                    at: path.member(b"user").member(b"file"),
                    path: name.clone(),
                    max: USER_BYTES,
                    private: true,
                })
                .expect("bounded files");
        }
        if let Some(name) = &forge.webhook_secret {
            files
                .push(config::File {
                    at: path.member(b"webhook_secret"),
                    path: name.clone(),
                    max: members.limits.forge_component.hook_secret_bytes,
                    private: true,
                })
                .expect("bounded files");
        }
        match &forge.transport {
            Transport::Plaintext => {}
            Transport::Tls { trust, .. } => {
                if let Some(trust) = trust {
                    files
                        .push(config::File {
                            at: path.member(b"transport").member(b"tls").member(b"trust"),
                            path: trust.clone(),
                            max: 1_048_576,
                            private: false,
                        })
                        .expect("bounded files");
                }
            }
        }
    }
    for (index, identity) in members.identities.iter().enumerate() {
        files
            .push(config::File {
                at: read::index(&Path::root().member(b"application").member(b"identities"), index).member(b"token"),
                path: identity.token.clone(),
                max: token_bytes,
                private: true,
            })
            .expect("bounded files");
    }
    files.into_boxed()
}
#[derive(Debug)]
pub struct Resolved {
    pub webhooks: Option<skein_forge_connection::hooks::Webhooks>,
    pub forges: Box<[settings::Forge]>,
    pub identities: Box<[settings::Identity]>,
}
fn contents(files: &settings::Files, name: &[u8], max: u32, at: Path, problems: &mut Problems) -> Option<Box<[u8]>> {
    for (path, value) in &files.entries {
        if path.as_ref() == name {
            if value.expose().len() > usize::try_from(max).expect("file bound") {
                problems.push(at, What::Jig(config::What::Bound));
                return None;
            }
            return Some(bytes::copy_of(value.expose()));
        }
    }
    problems.push(at, What::Jig(config::What::Unreadable));
    None
}
pub fn resolve(members: &Members, files: &settings::Files, problems: &mut Problems) -> Option<Resolved> {
    let mut forges = List::with_capacity(u32::try_from(members.forges.len()).expect("bounded forges"));
    let mut identities = List::with_capacity(u32::try_from(members.identities.len()).expect("bounded identities"));
    for (index, forge) in members.forges.iter().enumerate() {
        let path = read::index(&Path::root().member(b"application").member(b"forges"), index);
        let token = contents(files, &forge.api_token, super::API_TOKEN_BYTES, path.member(b"api_token"), problems);
        let mut webhook_secret = None;
        if let Some(name) = &forge.webhook_secret
            && let Some(value) = contents(
                files,
                name,
                members.limits.forge_component.hook_secret_bytes,
                path.member(b"webhook_secret"),
                problems,
            )
        {
            webhook_secret = Some(config::secret(value));
        }
        let transport = match &forge.transport {
            Transport::Plaintext => settings::Transport::Plaintext,
            Transport::Tls { server_name, trust } => {
                let trust = match trust {
                    Some(name) => contents(
                        files,
                        name,
                        1_048_576,
                        path.member(b"transport").member(b"tls").member(b"trust"),
                        problems,
                    ),
                    None => None,
                };
                settings::Transport::Tls { server_name: server_name.clone(), trust }
            }
        };
        let user = match &forge.user {
            User::Id(value) => Some(*value),
            User::File(name) => {
                let at = path.member(b"user").member(b"file");
                match contents(files, name, USER_BYTES, at.clone(), problems) {
                    Some(value) => match read::decimal(&value) {
                        Some(value) => Some(value),
                        None => {
                            problems.push(at, What::Jig(config::What::Form));
                            None
                        }
                    },
                    None => None,
                }
            }
        };
        if let Some(token) = token
            && let Some(user) = user
        {
            forges
                .push(settings::Forge {
                    number: forge.number,
                    kind: forge.kind,
                    api: forge.api.clone(),
                    transport,
                    git: forge.git.clone(),
                    api_token: config::secret(token),
                    webhook_secret,
                    user,
                    identity: forge.identity,
                })
                .expect("bounded forges");
        }
    }
    for (index, identity) in members.identities.iter().enumerate() {
        let path = read::index(&Path::root().member(b"application").member(b"identities"), index).member(b"token");
        if let Some(token) = contents(
            files,
            &identity.token,
            jig_protocol_worker_wire::CEILINGS.application_value_secret,
            path,
            problems,
        ) {
            identities
                .push(settings::Identity {
                    number: identity.number,
                    token: config::secret(token),
                    login: identity.login.clone(),
                    author: identity.author.clone(),
                })
                .expect("bounded identities");
        }
    }
    if !problems.is_empty() {
        return None;
    }
    let mut webhooks = None;
    if let Some(value) = &members.webhooks
        && let config::Address::Addr(listener) = &value.listener
    {
        webhooks = Some(skein_forge_connection::hooks::Webhooks { listener: *listener });
    }
    Some(Resolved { forges: forges.into_boxed(), identities: identities.into_boxed(), webhooks })
}

fn joined(
    directories: &config::Directories,
    kind: config::Directory,
    value: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    match directories.join(kind, value) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.clone(), What::Jig(what));
            None
        }
    }
}

fn resolve_forge(directories: &config::Directories, forge: &mut Forge, fp: &Path, problems: &mut Problems) {
    if let Transport::Tls { trust: Some(trust), .. } = &mut forge.transport
        && let Some(value) = joined(
            directories,
            config::Directory::Config,
            trust,
            &fp.member(b"transport").member(b"tls").member(b"trust"),
            problems,
        )
    {
        *trust = value;
    }
    if let Some(value) =
        joined(directories, config::Directory::Data, &forge.api_token, &fp.member(b"api_token"), problems)
    {
        forge.api_token = value;
    }
    if let Some(value) = &mut forge.webhook_secret
        && let Some(resolved) =
            joined(directories, config::Directory::Data, value, &fp.member(b"webhook_secret"), problems)
    {
        *value = resolved;
    }
    if let User::File(value) = &mut forge.user
        && let Some(resolved) =
            joined(directories, config::Directory::Data, value, &fp.member(b"user").member(b"file"), problems)
    {
        *value = resolved;
    }
}
