//! Startup for temper's standalone binary (configuration.md, section 2).
use jig_protocol_config::{self as config, read};
use skein_io::kernel::{Error, Exit, Fd};
use skein_lib::{List, Secret};
use skein_shell::{Clock, Config, Kernel};
use std::{
    ffi::OsString,
    fmt::Write,
    net::ToSocketAddrs,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Path, PathBuf},
};
use temper_service::{
    self as service,
    settings::{Agent, Files, Seeds},
};

mod host;
pub mod open;
mod setup;
pub mod sign_in;
pub use host::Host;
pub const USAGE: &str = "usage: temper serve CONFIG.json | temper agent AGENT.json | temper open DIRECTORY LINK_OR_ADDRESS | temper sign-in CONFIG.json ACCOUNT";
const CONFIG_BYTES: u32 = 1 << 20;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Serve { configuration: PathBuf },
    Agent { configuration: PathBuf },
    Open { arguments: Vec<OsString> },
    SignIn { arguments: Vec<OsString> },
}
fn prefixed(command: &str, why: &str) -> String {
    let mut lines = String::new();
    for line in why.lines() {
        writeln!(lines, "{command}: {line}").expect("writing into a string");
    }
    lines
}
pub(crate) fn problems(found: &service::startup::Problems) -> String {
    prefixed("temper serve", &String::from_utf8_lossy(&service::startup::lines(found)))
}
pub fn configure(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let usage = || format!("temper: invalid arguments\n{USAGE}\n");
    let command = args.next().ok_or_else(usage)?;
    if command == "open" {
        return Ok(Command::Open { arguments: args.collect() });
    }
    if command == "sign-in" {
        return Ok(Command::SignIn { arguments: args.collect() });
    }
    let configuration = args.next().ok_or_else(usage)?.into();
    if args.next().is_some() {
        return Err(usage());
    }
    if command == "serve" {
        Ok(Command::Serve { configuration })
    } else if command == "agent" {
        Ok(Command::Agent { configuration })
    } else {
        Err(usage())
    }
}
pub fn run(command: Command) -> Result<u8, String> {
    let exit = match command {
        Command::Serve { configuration } => serve(&configuration).map_err(|why| prefixed("temper serve", &why))?,
        Command::Agent { configuration } => smith_agent_shell::run(&configuration, Box::new(std::io::stderr()))
            .map_err(|why| prefixed("temper agent", &why))?,
        Command::Open { arguments } => return Ok(open::run(&arguments, &Clock::new())),
        Command::SignIn { arguments } => return Ok(sign_in::run(&arguments, &Clock::new())),
    };
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}
fn path(bytes: &[u8]) -> &Path {
    Path::new(std::ffi::OsStr::from_bytes(bytes))
}
fn member(document: &skein_json::Document, at: u32, key: &[u8]) -> u32 {
    read::Object::new(document, at).expect("named checked object").member(document, key).expect("named checked member")
}
fn errno(error: Error) -> i32 {
    match error {
        Error::Refused => 111,
        Error::Reset => 104,
        Error::BrokenPipe => 32,
        Error::NotConnected => 107,
        Error::AddressInUse => 98,
        Error::AddressNotAvailable => 99,
        Error::Unreachable => 101,
        Error::TimedOut => 110,
        Error::TooManyOpenFiles => 24,
        Error::NoBufferSpace => 12,
        Error::Cancelled => 125,
        Error::TooLate | Error::NotFound => 2,
        Error::Exists => 17,
        Error::NotADirectory => 20,
        Error::IsADirectory => 21,
        Error::NotEmpty => 39,
        Error::Permission => 13,
        Error::NoSpace => 28,
        Error::ReadOnly => 30,
        Error::TooManyLinks => 40,
        Error::NameTooLong => 36,
        Error::Escape => 18,
        Error::NotAFile => 6,
        Error::InvalidArgument => 22,
        Error::Other(number) => number,
    }
}
fn unread(why: skein_shell::Unread) -> String {
    match why {
        skein_shell::Unread::Absent => "absent".into(),
        skein_shell::Unread::NotAFile => "not a file".into(),
        skein_shell::Unread::TooLarge { max } => format!("past {max} bytes"),
        skein_shell::Unread::Unsafe(_) => "readable by others or reached through a link".into(),
        skein_shell::Unread::Kernel(error) => format!("errno {}", errno(error)),
    }
}
fn file_path(at: config::Path) -> String {
    let mut found = service::startup::Problems::empty();
    found.push(at, service::startup::What::Jig(config::What::Unreadable));
    let bytes = service::startup::lines(&found);
    String::from_utf8_lossy(&bytes).trim_end().strip_suffix(": not read").expect("path and reason").into()
}
fn read_files(directory: &Path, names: Box<[config::File]>, user: u32) -> Result<Files, String> {
    let mut entries = List::with_capacity(u32::try_from(names.len()).expect("bounded files"));
    let mut refused = String::new();
    for file in names {
        let resolved = directory.join(path(&file.path));
        let value = if file.private {
            skein_shell::read_private_file(&resolved, file.max, user)
        } else {
            skein_shell::read_file(&resolved, file.max).map(Secret::new)
        };
        match value {
            Ok(value) => entries.push((file.path, value)).expect("named capacity"),
            Err(why) => {
                writeln!(refused, "{}: {}: {}", file_path(file.at), path(&file.path).display(), unread(why))
                    .expect("writing into a string");
            }
        }
    }
    if refused.is_empty() { Ok(Files { entries }) } else { Err(refused) }
}
fn agent(configuration: &Path) -> Result<Agent, String> {
    let configuration = smith_agent_shell::config::read(configuration)?;
    let limits = &configuration.service.limits;
    let host_limits = smith_local_service::profile::host_limits(&configuration.profile, limits)
        .map_err(|_| "agent host limits refused")?;
    let channel = limits.channel;
    let mut protocol = smith_host_protocol::Limits {
        bodies: channel.bodies,
        charter: channel.charter,
        transcript: channel.transcript,
        endpoints: channel.endpoints,
        channel: channel.channel,
        start_bytes: channel.start_bytes,
        calls: host_limits.calls,
    };
    protocol.channel.output_bytes = smith_host_protocol::output_bytes(&protocol).ok_or("agent output bound")?;
    let mut models = Vec::new();
    for endpoint in &configuration.declarations.endpoints {
        for model in &endpoint.models {
            models.push(jig_domain_host::ModelBound {
                name: model.name.clone(),
                window: model.window,
                output: model.output,
            });
        }
    }
    let policy = configuration.profile.policy;
    let message_bytes = limits
        .domain
        .run
        .message_bytes
        .checked_sub(u32::try_from(temper_worker_domain::SEPARATOR).expect("message separator fits u32"))
        .ok_or("agent message_bytes: too small for the rendered separator")?;
    Ok(Agent {
        endpoints: configuration.service.channel_endpoints,
        limits: protocol,
        host_limits,
        bounds: jig_domain_host::Bounds {
            models: models.into_boxed_slice(),
            turns: policy.max_turns,
            spend: policy.max_spend,
            time: policy.max_time,
            waiting: policy.max_waiting,
            checks: policy.check_timeout,
            message_bytes,
            inbox: policy.inbox,
            unacknowledged: policy.unacknowledged,
        },
    })
}
fn address(configured: &config::Address) -> Result<std::net::SocketAddr, String> {
    match configured {
        config::Address::Addr(address) => address.socket().ok_or_else(|| "requires a socket address".into()),
        config::Address::Host { name, port } => (std::str::from_utf8(name).map_err(|why| why.to_string())?, *port)
            .to_socket_addrs()
            .map_err(|why| why.to_string())?
            .next()
            .ok_or_else(|| "endpoint has no address".into()),
    }
}
fn trust(directory: &Path, file: Option<&[u8]>) -> Result<(skein_tls::Config, u32, u32), String> {
    let source = file.map_or(skein_shell::Trust::Machine { ssl_cert_file: None }, |file| {
        skein_shell::Trust::Der(directory.join(path(file)))
    });
    let roots = skein_shell::read_trust_roots(
        &source,
        &skein_shell::RootBounds { bundle_bytes: CONFIG_BYTES, certificates: 1024, certificate_bytes: 65_536 },
    )
    .map_err(|why| match why {
        skein_shell::RootsRefusal::NoBundle => "no trust bundle".into(),
        skein_shell::RootsRefusal::Unread { path, why } => format!("{}: {}", path.display(), unread(why)),
        skein_shell::RootsRefusal::TooMany { bound } => format!("past {bound} trust roots"),
        skein_shell::RootsRefusal::TooLong { index, bound } => format!("trust root {index}: past {bound} bytes"),
    })?;
    let count = u32::try_from(roots.certificates.len()).expect("bounded roots");
    let each =
        roots.certificates.iter().map(|root| u32::try_from(root.len()).expect("bounded root")).max().unwrap_or(0);
    let (trust, _) = skein_tls::Config::from_der(&roots.certificates, &[]).map_err(|why| match why {
        skein_tls::Refusal::Roots => "no supported trust roots",
        skein_tls::Refusal::Alpn => "protocol names refused",
        skein_tls::Refusal::Provider => "TLS provider refused",
    })?;
    Ok((trust, count, each))
}
fn server_name(bytes: &[u8]) -> Result<skein_tls::Name, String> {
    skein_tls::Name::new(std::str::from_utf8(bytes).map_err(|why| why.to_string())?)
        .ok_or_else(|| "invalid server name".into())
}
fn accounts(
    directory: &Path,
    engine: &config::Engine,
    owned: &mut Vec<Fd>,
    user: u32,
) -> Result<service::AccountsRuntime, String> {
    let mut issuers = List::with_capacity(u32::try_from(engine.llm.accounts.len()).expect("account count"));
    let mut trust_roots = 0_u32;
    let mut trust_root_bytes = 0_u32;
    for account in &engine.llm.accounts {
        if let config::Source::SignIn(source) = &account.source {
            let address = address(&source.endpoint)?;
            let transport = match &source.transport {
                config::Transport::Plaintext => skein_oauth_accounts::Transport::Plaintext,
                config::Transport::Tls { server_name: name, trust: file } => {
                    let (trust, count, each) = trust(directory, file.as_deref())?;
                    trust_roots = trust_roots.checked_add(count).ok_or("trust count")?;
                    trust_root_bytes = trust_root_bytes.max(each);
                    skein_oauth_accounts::Transport::Tls { server_name: server_name(name)?, trust }
                }
            };
            issuers
                .push((account.number, skein_oauth_accounts::Endpoint { address, transport }))
                .expect("account count");
        }
    }
    let root = skein_shell::open_root(directory).map_err(|errno| format!("account root: errno {errno}"))?;
    owned.push(root);
    Ok(service::AccountsRuntime { root, effective_user: user, issuers, trust_roots, trust_root_bytes })
}
fn forges(directory: &Path, document: &skein_json::Document) -> Result<List<service::ForgeRuntime>, String> {
    use temper_engine_protocol_forge::settings::Transport;
    let application = member(document, 0, b"application");
    let list =
        read::list(document, member(document, application, b"forges"), document.len()).expect("named checked forges");
    let mut runtime = List::with_capacity(list.len());
    for (index, at) in list.iter().enumerate() {
        let prefix = format!("application.forges[{index}]");
        let mut found = config::Problems::empty();
        let configured =
            config::address::read(document, member(document, *at, b"api"), &config::Path::root(), &mut found)
                .expect("named checked address");
        let address = address(&configured).map_err(|why| format!("{prefix}.api: {why}"))?;
        let number =
            read::integer(document, member(document, *at, b"number"), u64::from(u16::MAX)).expect("checked number");
        let at = member(document, *at, b"transport");
        let (transport, trust_roots, trust_root_bytes) = if read::text(document, at, 32).is_ok() {
            (Transport::Plaintext, 0, 0)
        } else {
            let tls = member(document, at, b"tls");
            let name = read::text(document, member(document, tls, b"server_name"), 253).expect("checked TLS name");
            let file = read::text(document, member(document, tls, b"trust"), CONFIG_BYTES).ok();
            let (trust, count, each) =
                trust(directory, file.as_deref()).map_err(|why| format!("{prefix}.transport.tls.trust: {why}"))?;
            (Transport::Tls { server_name: server_name(&name)?, trust }, count, each)
        };
        runtime
            .push(service::ForgeRuntime {
                number: u16::try_from(number).expect("checked number"),
                address,
                transport,
                trust_roots,
                trust_root_bytes,
            })
            .expect("forge count");
    }
    Ok(runtime)
}
fn private_root(path: &Path, user: u32) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|why| why.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.uid() != user || metadata.mode() & 0o077 != 0
    {
        return Err("must be a private directory of this user's, not reached through a link".into());
    }
    Ok(())
}
fn prepare(directory: &Path, bytes: &[u8], owned: &mut Vec<Fd>) -> Result<service::Service, String> {
    let (named, _) = setup::check(bytes)?;
    let document =
        config::document(bytes, &config::DocumentLimits { bytes: CONFIG_BYTES, tokens: CONFIG_BYTES, depth: 32 })
            .expect("named checked document");
    let engine = config::engine(&document, member(&document, 0, b"engine")).expect("named checked engine");
    let user = skein_shell::effective_user();
    let files = read_files(directory, named.files, user)?;
    let agent =
        agent(&directory.join(path(&named.agent))).map_err(|why| format!("workers[0].agent.configuration: {why}"))?;
    let root_path = directory.join(path(&named.worker_root));
    if std::fs::symlink_metadata(&root_path).is_ok() {
        private_root(&root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    }
    let root = skein_shell::make_root(&root_path, 0o700)
        .map_err(|errno| format!("application.workers[0].root: errno {errno}"))?;
    owned.push(root);
    private_root(&root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
    let accounts = accounts(directory, &engine, owned, user)?;
    let forges = forges(directory, &document)?;
    let seed = skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?;
    let crypto = skein_shell::crypto_seed().map_err(|errno| format!("cryptographic seed: errno {errno}"))?;
    let drawn = skein_shell::crypto_seed().map_err(|errno| format!("deployment id: errno {errno}"))?;
    let mut deployment = [0_u8; 16];
    deployment.copy_from_slice(drawn.expose().get(..16).expect("32-byte seed"));
    let mut service = service::startup(
        bytes,
        &files,
        &agent,
        &Seeds { deployment, seed, crypto },
        Some(accounts),
        service::WorkerRuntime { root, path: root_path.as_os_str().as_bytes().into() },
        forges,
    )
    .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    let root = skein_shell::make_root(&directory.join(path(&named.deployment)), 0o700)
        .map_err(|errno| format!("engine.deployment.directory: errno {errno}"))?;
    owned.push(root);
    service.adopt_store(root, user).map_err(|_| "store refused its deployment root")?;
    Ok(service)
}
fn close(owned: Vec<Fd>) {
    for fd in owned {
        skein_shell::close_keeper_fd(fd);
    }
}
pub fn prepare_in(directory: &Path) -> Result<service::Service, String> {
    let directory = std::path::absolute(directory).map_err(|why| prefixed("temper serve", &why.to_string()))?;
    let bytes = skein_shell::read_file(&directory.join("config.json"), CONFIG_BYTES)
        .map_err(|why| prefixed("temper serve", &format!("configuration config.json: {}", unread(why))))?;
    let mut owned = Vec::new();
    let result = prepare(&directory, &bytes, &mut owned);
    if result.is_err() {
        close(owned);
    }
    result.map_err(|why| prefixed("temper serve", &why))
}
fn serve(path: &Path) -> Result<Exit, String> {
    let directory = std::env::current_dir().map_err(|why| why.to_string())?;
    let bytes = skein_shell::read_file(path, CONFIG_BYTES)
        .map_err(|why| format!("configuration {}: {}", path.display(), unread(why)))?;
    let mut owned = Vec::new();
    let prepared = prepare(&directory, &bytes, &mut owned);
    let mut service = match prepared {
        Ok(service) => service,
        Err(why) => {
            close(owned);
            return Err(why);
        }
    };
    let opened = skein_shell::open_termination_signals()
        .map_err(|errno| format!("signals: errno {errno}"))
        .and_then(|signals| {
            owned.push(signals);
            service.adopt_signals(signals).map_err(|_| "signals refused".into())
        })
        .and_then(|()| Kernel::open(Config { operations: service.operations() }).map_err(|why| why.to_string()));
    let mut kernel = match opened {
        Ok(kernel) => kernel,
        Err(why) => {
            close(owned);
            return Err(why);
        }
    };
    Ok(skein_shell::drive(&mut kernel, &Clock::new(), &mut Host::new(service)))
}
#[cfg(test)]
mod tests;
