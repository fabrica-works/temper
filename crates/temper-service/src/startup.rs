//! The whole standalone document and its checked startup (configuration.md, 2).
pub(crate) mod application;
mod problems;
use crate::{AccountsRuntime, Service, Worker, WorkerRuntime, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, Segment, read};
pub use problems::{Problem, Problems, What, lines};
use skein_lib::List;

/// A forge API token's bound, until the forge component supplies it.
pub const API_TOKEN_BYTES: u32 = 4096;
/// The files and directories the binary reads and opens before startup.
#[derive(Debug)]
pub struct Named {
    pub files: Box<[config::File]>,
    pub deployment: Box<[u8]>,
    pub worker_root: Box<[u8]>,
    pub agent: Box<[u8]>,
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
fn parse(input: &[u8]) -> Result<Parsed, Problems> {
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
        Some(at) => match config::engine(&document, at) {
            Ok(engine) => Some(engine),
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
        match config::worker(&document, *at) {
            Ok(worker) => workers.push(worker).expect("bounded workers"),
            Err(found) => problems.append(found, &indexed(index)),
        }
    }
    let application = match &engine {
        Some(engine) => match application_at {
            Some(at) => application::parse(&document, at, engine, &mut problems),
            None => None,
        },
        None => None,
    };
    if !problems.is_empty() {
        return Err(problems);
    }
    let application = application.expect("required application checked");
    if workers.len() != 1 {
        problems.push(root.member(b"workers"), What::Standalone);
    }
    if usize::try_from(workers.len()).expect("bounded workers") != application.workers.len() {
        problems.push(root.member(b"application").member(b"workers"), What::Jig(config::What::Bound));
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(Parsed { engine: engine.expect("required engine checked"), workers: workers.into_boxed(), application })
}
fn all_files(parsed: &Parsed) -> Box<[config::File]> {
    let engine = config::files(&parsed.engine);
    let worker = config::worker_files(parsed.workers.first().expect("one worker checked"));
    let application =
        application::files(&parsed.application, API_TOKEN_BYTES, jig_protocol_worker_wire::CEILINGS.grant_value);
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
pub fn named(document: &[u8]) -> Result<Named, Problems> {
    let parsed = parse(document)?;
    let files = all_files(&parsed);
    Ok(Named {
        files,
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
/// Build the engine and worker together from the parsed document and read files.
pub fn startup(
    document: &[u8],
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<AccountsRuntime>,
    runtime: WorkerRuntime,
) -> Result<Service, Problems> {
    let parsed = parse(document)?;
    check_files(&parsed, files)?;
    let mut problems = Problems::empty();
    let resolved = application::resolve(&parsed.application, files, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let resolved = resolved.expect("resolved files checked");
    let worker = parsed.workers.into_iter().next().expect("one worker checked");
    let own = parsed.application.workers.into_iter().next().expect("one application worker checked");
    let worker = settings::Worker { jig: worker, root: own.root, git: own.git };
    if let Err(found) = crate::worker_settings::prepare(&worker, agent, &parsed.application.limits) {
        return Err(Problems::jig(found, &Path::root()));
    }
    let bytes = crate::worker_worst_case(&worker, agent, &parsed.application.limits)
        .ok_or(problem(indexed(0).member(b"memory_bytes"), What::Jig(config::What::Bound)))?;
    let worker = match Worker::new(&worker, agent, &parsed.application.limits, files, runtime, seeds.seed) {
        Ok(worker) => worker,
        Err(found) => return Err(Problems::jig(found, &Path::root())),
    };
    let engine = settings::Engine {
        jig: parsed.engine,
        files: settings::Files { entries: files.entries.clone() },
        forges: resolved.forges,
        identities: resolved.identities,
        landing: parsed.application.landing,
        charter: parsed.application.charter,
        limits: parsed.application.limits,
    };
    match Service::new(engine, seeds, accounts, Some(worker), bytes) {
        Ok(service) => Ok(service),
        Err(found) => Err(Problems::jig(found, &Path::root())),
    }
}
