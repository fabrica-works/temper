//! The whole standalone document and its checked startup (protocol/configuration.md, 2).
pub(crate) mod application;
pub(crate) mod checks;
mod problems;
use crate::{AccountsRuntime, Service, Worker, WorkerRuntime, settings};
use alloc::boxed::Box;
pub use application::{Forge, Identity, Transport, USER_BYTES, User};
pub use checks::redirect_ports;
use jig_protocol_config::{self as config, Path, Segment, read};
pub use problems::{Problem, Problems, What, lines};
use skein_lib::List;

/// A forge API token's bound, until the forge component supplies it.
pub const API_TOKEN_BYTES: u32 = 4096;
/// A provider's fixed sign-in redirect, with its diagnostic path; no credentials.
#[derive(Debug)]
pub struct RedirectPort {
    pub port: u16,
    pub path: Path,
    pub redirect_path: Box<[u8]>,
}
/// The standalone listeners whose fixed ports a redirect must avoid.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Listener {
    Clients,
    Webhooks,
    Owned(u32),
}
fn redirects(engine: &config::Engine) -> Box<[RedirectPort]> {
    let mut redirects = List::with_capacity(u32::try_from(engine.llm.providers.len()).expect("bounded providers"));
    for (index, provider) in engine.llm.providers.iter().enumerate() {
        if let Some(registration) = &provider.sign_in
            && let Some(port) = registration.redirect_port
        {
            redirects
                .push(RedirectPort {
                    port,
                    path: Path::root()
                        .member(b"engine")
                        .member(b"llm")
                        .member(b"providers")
                        .child(Segment::Index(u32::try_from(index).expect("bounded providers")))
                        .member(b"sources")
                        .member(b"sign_in")
                        .member(b"redirect_port"),
                    redirect_path: registration.redirect_path.clone(),
                })
                .expect("one redirect per provider");
        }
    }
    redirects.into_boxed()
}
fn check_redirects(engine: &config::Engine, application: &application::Members, problems: &mut Problems) {
    let redirects = redirects(engine);
    redirect_ports(&redirects, checks::port(&application.clients.listener), Listener::Clients, problems);
    let port = match &application.webhooks {
        Some(hooks) => checks::port(&hooks.listener),
        None => None,
    };
    redirect_ports(&redirects, port, Listener::Webhooks, problems);
}
/// The files and directories the binary reads and opens before startup.
#[derive(Debug)]
pub struct Named {
    pub files: Box<[config::File]>,
    pub redirects: Box<[RedirectPort]>,
    pub deployment: Box<[u8]>,
    pub worker_root: Box<[u8]>,
    pub agent: Box<[u8]>,
    pub forges: Box<[Forge]>,
    pub identities: Box<[Identity]>,
    pub clients: config::Address,
    /// `application.clients.bundle`, for the binary's read.
    pub bundle: Option<Box<[u8]>>,
    /// The bundle's limits, overrides applied.
    pub bundle_limits: crate::limits::Clients,
    pub webhooks: Option<application::Webhooks>,
    pub overrides: Box<[(Box<[u8]>, u64)]>,
}
#[derive(Debug)]
struct Parsed {
    engine: config::Engine,
    workers: Box<[config::Worker]>,
    application: application::Members,
}
fn problem(path: Path, what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(path, what);
    problems
}
fn indexed(index: usize) -> Path {
    Path::root().member(b"workers").child(Segment::Index(u32::try_from(index).expect("bounded workers")))
}
fn parse(input: &[u8], directories: &config::Directories) -> Result<Parsed, Problems> {
    let length = u32::try_from(input.len()).or(Err(problem(Path::root(), What::Jig(config::What::Bound))))?;
    let document =
        match config::document(input, &config::DocumentLimits { bytes: 1_048_576, tokens: length, depth: 32 }) {
            Ok(document) => document,
            Err(found) => return Err(problem(found.path, What::Jig(found.what))),
        };
    let mut problems = Problems::empty();
    let mut found = config::Problems::empty();
    let mut whole = match read::Object::new(&document, 0) {
        Ok(object) => object,
        Err(what) => return Err(problem(Path::root(), What::Jig(what))),
    };
    let root = Path::root();
    let engine_at = whole.required(&document, b"engine", &root, &mut found);
    let worker_at = whole.required(&document, b"workers", &root, &mut found);
    let application_at = whole.required(&document, b"application", &root, &mut found);
    whole.finish(&document, &root, &mut found);
    problems.append(found, &root);
    let engine = match engine_at {
        Some(at) => match config::engine(&document, at, directories) {
            Ok(mut engine) => {
                engine.limits.clients.watches_per_stream = temper_engine_domain::WATCHES_PER_STREAM;
                Some(engine)
            }
            Err(found) => {
                problems.append(found, &root.member(b"engine"));
                None
            }
        },
        None => None,
    };
    let ats = match worker_at {
        Some(at) => match read::list(&document, at, document.len()) {
            Ok(ats) => ats,
            Err(what) => {
                problems.push(root.member(b"workers"), What::Jig(what));
                List::with_capacity(0)
            }
        },
        None => List::with_capacity(0),
    };
    let mut workers = List::with_capacity(ats.len());
    for (index, at) in ats.iter().enumerate() {
        checks::raw_worker(&document, *at, &indexed(index), &mut problems);
        match config::worker(&document, *at, directories) {
            Ok(worker) => workers.push(worker).expect("bounded workers"),
            Err(found) => problems.append(found, &indexed(index)),
        }
    }
    let application = match &engine {
        Some(engine) => match application_at {
            Some(at) => application::parse(&document, at, engine, directories, &mut problems),
            None => None,
        },
        None => None,
    };
    if !problems.is_empty() {
        return Err(problems);
    }
    let application = application.expect("required application checked");
    let workers = workers.into_boxed();
    checks::standalone(
        engine.as_ref().expect("required engine checked"),
        &application.clients,
        &workers,
        &mut problems,
    );
    if workers.len() != application.workers.len() {
        problems.push(root.member(b"application").member(b"workers"), What::Jig(config::What::Bound));
    }
    checks::hub(engine.as_ref().expect("required engine checked"), &mut problems);
    checks::provider(engine.as_ref().expect("required engine checked"), &mut problems);
    checks::credentials(
        engine.as_ref().expect("required engine checked"),
        &workers,
        &application.identities,
        &mut problems,
    );
    checks::webhooks(&application, &mut problems);
    check_redirects(engine.as_ref().expect("required engine checked"), &application, &mut problems);
    checks::records(engine.as_ref().expect("required engine checked"), &application.limits, &mut problems);
    checks::limits(&application.limits, &application.limits.git_component, &mut problems);
    let mut base = 0;
    for forge in &application.forges {
        base = base.max(u64::try_from(forge.git.len()).expect("bounded git base"));
    }
    checks::names(checks::remote(base, application.limits.forge.name_bytes), &application.limits, &mut problems);
    if let Some(worker) = workers.first() {
        checks::sockets(
            engine.as_ref().expect("required engine checked"),
            worker,
            &application.limits.git_component,
            &application,
            &mut problems,
        );
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(Parsed { engine: engine.expect("required engine checked"), workers, application })
}
fn all_files(parsed: &Parsed) -> Box<[config::File]> {
    let engine = config::files(&parsed.engine);
    let worker = config::worker_files(parsed.workers.first().expect("one worker checked"));
    let application = application::files(
        &parsed.application,
        API_TOKEN_BYTES,
        jig_protocol_worker_wire::CEILINGS.application_value_secret,
    );
    let count = engine
        .len()
        .checked_add(worker.len())
        .expect("bounded files")
        .checked_add(application.len())
        .expect("bounded files");
    let mut files = List::with_capacity(u32::try_from(count).expect("bounded file count"));
    for (named, prefix) in [(engine, Path::root().member(b"engine")), (worker, indexed(0)), (application, Path::root())]
    {
        for mut file in named {
            let mut path = prefix.clone();
            for segment in &file.at.segments {
                path = path.child(segment.clone());
            }
            file.at = path;
            files.push(file).expect("capacity includes every named file");
        }
    }
    files.into_boxed()
}
/// Parse every member before the binary reads any named file.
pub fn named(document: &[u8], directories: &config::Directories) -> Result<Named, Problems> {
    let parsed = parse(document, directories)?;
    let files = all_files(&parsed);
    Ok(Named {
        files,
        redirects: redirects(&parsed.engine),
        clients: parsed.application.clients.listener,
        bundle: parsed.application.clients.bundle,
        bundle_limits: parsed.application.limits.clients,
        webhooks: parsed.application.webhooks,
        forges: parsed.application.forges,
        identities: parsed.application.identities,
        overrides: parsed.application.overrides,
        deployment: parsed.engine.deployment.directory,
        worker_root: parsed.application.workers.first().expect("one application worker checked").root.clone(),
        agent: parsed.workers.first().expect("one worker checked").agent.configuration.clone(),
    })
}
fn check_files(parsed: &Parsed, files: &settings::Files) -> Result<(), Problems> {
    let mut problems = Problems::empty();
    for file in all_files(parsed) {
        match files.contents(&file.path) {
            Some(contents) if contents.len() <= usize::try_from(file.max).expect("file bound") => {}
            Some(_) => problems.push(file.at, What::Jig(config::What::Bound)),
            None => problems.push(file.at, What::Jig(config::What::Unreadable)),
        }
    }
    if problems.is_empty() { Ok(()) } else { Err(problems) }
}
/// Checked envelopes from the same calculations startup uses, without starting services.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Requirements {
    pub engine_bytes: u64,
    pub worker_bytes: u64,
}
struct CheckedStart {
    engine: settings::Engine,
    worker: settings::Worker,
    requirements: Requirements,
    io: skein_io::Limits,
    forges: (skein_forge_connection::Settings, u64, u64),
}
/// Derive the two envelopes without enforcing the document's memory declarations.
/// File, agent, root, channel and runtime checks are identical to startup's.
pub fn memory_requirements(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<&AccountsRuntime>,
    forges: List<crate::ForgeRuntime>,
) -> Result<Requirements, Problems> {
    Ok(checked_start(document, directories, files, agent, seeds, accounts, forges)?.requirements)
}
fn checked_start(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<&AccountsRuntime>,
    forges: List<crate::ForgeRuntime>,
) -> Result<CheckedStart, Problems> {
    let parsed = parse(document, directories)?;
    check_files(&parsed, files)?;
    let mut problems = Problems::empty();
    let resolved = application::resolve(&parsed.application, files, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let resolved = resolved.expect("resolved files checked");
    let own = &parsed.application.limits;
    let remote = u32::try_from(checks::longest_remote(&resolved.forges, own.forge.name_bytes).unwrap_or(u64::MAX)).ok();
    let engine_bounds = temper_protocol_worker_wire::Bounds {
        repositories: own.forge.resources_per_task,
        name_bytes: own.forge.name_bytes,
        remote_bytes: remote.unwrap_or(u32::MAX),
        conflicts: own.checkout.conflicts,
        path_bytes: own.checkout.path_bytes,
        token_bytes: temper_protocol_worker_wire::v1::CEILINGS.identity_token,
        text_bytes: temper_protocol_worker_wire::v1::CEILINGS.identity_name,
    };
    let worker_bounds = temper_protocol_worker_wire::Bounds {
        repositories: own.checkout.repositories,
        name_bytes: own.checkout.name_bytes,
        remote_bytes: own.checkout.name_bytes,
        ..engine_bounds
    };
    checks::channel(&engine_bounds, &worker_bounds, &resolved.forges, own, &mut problems);
    if let Some(wire) = crate::translate::limits(own) {
        checks::identities(&resolved.identities, &wire, &mut problems);
    }
    checks::agent(&parsed.engine, agent, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let worker = parsed.workers.into_iter().next().expect("one worker checked");
    let own = parsed.application.workers.into_iter().next().expect("one application worker checked");
    let worker = settings::Worker { jig: worker, root: own.root, git: own.git };
    checks::worker(&crate::worker_settings::root_limits(&worker, agent, &parsed.application.limits), &mut problems);
    checks::grants(&parsed.engine, &worker.jig.limits, agent, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    if let Err(found) = crate::worker_settings::prepare(&worker, agent, &parsed.application.limits) {
        return Err(Problems::jig(found, &Path::root()));
    }
    let bytes = crate::worker_worst_case(&worker, agent, &parsed.application.limits)
        .ok_or(problem(indexed(0).member(b"memory_bytes"), What::Jig(config::What::Bound)))?;
    let worker_operations = crate::worker_settings::operations(&parsed.application.limits.git_component)
        .ok_or(problem(indexed(0).member(b"limits"), What::Jig(config::What::Bound)))?;
    let mut engine = settings::Engine {
        jig: parsed.engine,
        clients: parsed.application.clients,
        webhooks: resolved.webhooks,
        files: settings::Files { entries: files.entries.clone() },
        forges: resolved.forges,
        identities: resolved.identities,
        landing: parsed.application.landing,
        charter: parsed.application.charter,
        limits: parsed.application.limits,
    };
    engine.jig.limits.clients.wire.keyed_value_some =
        engine.jig.limits.clients.wire.keyed_value_some.min(engine.limits.records.value_bytes);
    let root = crate::limits::engine(&engine.jig.limits, &engine.limits)
        .ok_or(problem(Path::root().member(b"engine").member(b"limits"), What::Jig(config::What::Bound)))?;
    checks::root(&engine, seeds, &parsed.application.landing_projects, &root, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let store = crate::store::limits(&root, &engine.jig.limits.core, &root.forge, &engine.jig.limits.store)
        .ok_or(problem(Path::root().member(b"engine").member(b"limits"), What::Jig(config::What::Bound)))?;
    let io = checks::process_io(&engine.jig.limits.io, Some(&worker.jig.limits.io)).ok_or(problem(
        Path::root().member(b"engine").member(b"limits").member(b"io"),
        What::Jig(config::What::Bound),
    ))?;
    let prepared_forges = prepare_forges(&engine, forges)?;
    let engine_bytes = match crate::system::startup_worst_case(
        &engine,
        &root,
        &store,
        accounts,
        worker_operations,
        &io,
        (prepared_forges.1, prepared_forges.2, &prepared_forges.0),
    ) {
        Ok(bytes) => bytes,
        Err(found) => return Err(Problems::jig(found, &Path::root())),
    };
    Ok(CheckedStart {
        engine,
        worker,
        requirements: Requirements { engine_bytes, worker_bytes: bytes },
        io,
        forges: prepared_forges,
    })
}
/// Build the engine and worker together from the parsed document and read files.
#[expect(clippy::too_many_arguments, reason = "startup receives the same directories as named plus the runtime facts")]
pub fn startup(
    document: &[u8],
    directories: &config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: WorkerRuntime,
    forges: List<crate::ForgeRuntime>,
) -> Result<Service, Problems> {
    let CheckedStart { engine, worker, requirements, io, forges } =
        checked_start(document, directories, files, agent, seeds, accounts.as_ref(), forges)?;
    let mut problems = Problems::empty();
    checks::memory(
        requirements.engine_bytes,
        engine.jig.memory,
        requirements.worker_bytes,
        worker.jig.memory,
        &mut problems,
    );
    if !problems.is_empty() {
        return Err(problems);
    }
    let worker = match Worker::new(&worker, agent, &engine.limits, files, runtime, seeds.seed) {
        Ok(worker) => worker,
        Err(found) => return Err(Problems::jig(found, &Path::root())),
    };
    match Service::prepared(engine, seeds, accounts, forges, Some(worker), requirements.worker_bytes, io) {
        Ok(service) => Ok(service),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}

fn prepare_forges(
    engine: &settings::Engine,
    forges: List<crate::ForgeRuntime>,
) -> Result<(skein_forge_connection::Settings, u64, u64), Problems> {
    let (settings, roots) = match crate::forge::prepare(engine, forges) {
        Ok(prepared) => prepared,
        Err(found) => return Err(Problems::jig(found, &Path::root())),
    };
    let owned = skein_forge_connection::settings::owned_bytes(&settings)
        .ok_or(problem(Path::root().member(b"engine").member(b"memory_bytes"), What::Jig(config::What::Bound)))?;
    Ok((settings, roots, owned))
}
