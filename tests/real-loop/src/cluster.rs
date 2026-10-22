//! Engine and named workers on one ring (protocol/README.md, 15.6).
use crate::{calls, fixtures, llm, service::ServiceHost, world};
use skein_io::kernel::{Complete, Done, Op};
use std::{fmt::Write as _, net::SocketAddr, os::unix::fs::DirBuilderExt, path::Path};

/// Script hold released once every named worker has received an assignment.
pub const PLACED: &[u8] = b"cluster-placed";

pub struct Cluster {
    ran: crate::Ran,
    _scratch: Vec<crate::Scratch>,
    _git: temper_fake_git_hosted::Typed,
}
impl std::ops::Deref for Cluster {
    type Target = crate::Ran;
    fn deref(&self) -> &Self::Target {
        &self.ran
    }
}
impl Cluster {
    #[must_use]
    pub fn worker(&self, name: &str) -> &ServiceHost {
        self.ran
            .outcome
            .procs
            .iter()
            .filter_map(|proc| if let world::Proc::Service(host) = proc { Some(host.as_ref()) } else { None })
            .find(|host| matches!(&host.cluster, Some(Shape::Worker { name: worker, .. }) if *worker == name))
            .expect("worker root")
    }
}
#[must_use]
pub fn run_cluster(
    workers: &[&'static str],
    recipe: impl FnOnce() -> llm::Recipe,
    steps: impl FnOnce() -> Vec<calls::Step>,
) -> Cluster {
    assert!(!workers.is_empty(), "cluster workers");
    for (position, name) in workers.iter().enumerate() {
        assert!(!workers[..position].contains(name), "distinct worker names");
    }
    let scratch: Vec<_> = std::iter::once("engine").chain(workers.iter().copied()).map(crate::Scratch::new).collect();
    let git = temper_fake_git_hosted::fixtures::scenario().transport;
    crate::git::share(Some(git.clone()));
    let outcome = world::run_cluster(workers, &scratch, recipe, steps);
    crate::git::share(None);
    let mut scratch = scratch.into_iter();
    let ran = crate::checked(crate::Ran {
        outcome: crate::Outcome::Checked(outcome),
        scratch: scratch.next().expect("engine scratch"),
    });
    let cluster = Cluster { ran, _scratch: scratch.collect(), _git: git };
    for name in workers {
        assert_eq!(cluster.worker(name).end(), Some(&crate::End::Drained), "{name} drained");
    }
    cluster
}

pub(crate) enum Shape {
    Engine(Vec<&'static str>),
    Worker { name: &'static str, engine: Option<SocketAddr> },
}
impl Shape {
    pub(crate) fn ready(&self) -> bool {
        !matches!(self, Self::Worker { engine: None, .. })
    }
    pub(crate) fn write(&self, path: &Path) {
        use serde_json::json;
        let config = path.join("config");
        let data = path.join("data");
        for name in ["config/charters", "cache"] {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .recursive(true)
                .create(path.join(name))
                .expect("cluster directories");
        }
        let mut document: serde_json::Value =
            serde_json::from_slice(&std::fs::read(data.join("temper.json")).expect("base document"))
                .expect("base JSON");
        // Link deadlines exceed PATIENCE, so load cannot lose a link or run in this story.
        // Loss and grace belong to the engine's domain world (domain/testing.md, 2.4).
        // The stop bound stays below fleet grace (protocol/README.md, 15.3).
        let ping_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000) + 1;
        let silence_ms = 3 * ping_ms;
        let [grace_ms, agent_stop_ms, delivery_ms, save_ms] = [10_u64, 5, 5, 5];
        let stop_bound_ms = ping_ms + silence_ms + grace_ms + agent_stop_ms.max(delivery_ms) + save_ms;
        match self {
            Self::Engine(workers) => {
                document["workers"] = json!([]);
                document["application"]["workers"] = json!([]);
                document["engine"]["hosts"]["listener"] = json!([{"socket":"127.0.0.1:0"}]);
                document["engine"]["hosts"]["workers"] =
                    workers.iter().map(|name| json!({"name":name,"digest":format!("secrets/{name}.digest")})).collect();
                // One attempt per one-slot worker: the fleet counts attempts across the whole fleet.
                document["engine"]["limits"]["overrides"]["fleet.attempts"] = workers.len().into();
                document["engine"]["limits"]["overrides"]["fleet.grace_ms"] = (stop_bound_ms + 1).into();
                document["engine"]["limits"]["overrides"]["hub.turns"] = 1.into();
                document["engine"]["limits"]["overrides"]["link.ping_ms"] = ping_ms.into();
                document["engine"]["limits"]["overrides"]["link.silence_ms"] = silence_ms.into();
                for name in workers {
                    let secret = format!("{name}-fixture-secret");
                    let mut digest = String::new();
                    for word in skein_io::digest::digest(secret.as_bytes()).0 {
                        write!(digest, "{word:016x}").expect("fixture digest hex");
                    }
                    fixtures::private(&data.join(format!("secrets/{name}.digest")), digest.as_bytes())
                        .expect("fixture digest");
                }
            }
            Self::Worker { name, engine } => {
                let worker = document["workers"].as_array_mut().expect("workers").first_mut().expect("one worker");
                // One retained turn, one held event, and no program-backed LLM source.
                for (limit, size) in [("hub.turns", 1), ("hub.held", 1), ("hub.program_sources", 0)] {
                    worker["limits"]["overrides"][limit] = size.into();
                }
                let mut agent: serde_json::Value =
                    serde_json::from_slice(&std::fs::read(data.join("agent.json")).expect("agent document"))
                        .expect("agent JSON");
                agent["profile"]["policy"]["unacknowledged"] = 1.into();
                fixtures::private(&data.join("agent.json"), &serde_json::to_vec(&agent).expect("agent JSON"))
                    .expect("cluster agent");
                worker["name"] = (*name).into();
                worker["secret"] = format!("secrets/{name}.secret").into();
                worker["engine"] = json!({"socket": engine.unwrap_or(([127,0,0,1],0).into()).to_string()});
                worker["graces"] = json!({"grace_ms":grace_ms,"agent_stop_ms":agent_stop_ms,"delivery_ms":delivery_ms,"save_ms":save_ms});
                worker["limits"]["overrides"]["link.ping_ms"] = ping_ms.into();
                worker["limits"]["overrides"]["link.silence_ms"] = silence_ms.into();
                document = json!({"workers":document["workers"],"application":{"workers":document["application"]["workers"],"limits":document["application"]["limits"]}});
                fixtures::private(
                    &data.join(format!("secrets/{name}.secret")),
                    format!("{name}-fixture-secret").as_bytes(),
                )
                .expect("fixture secret");
            }
        }
        fixtures::private(&config.join("temper.json"), &serde_json::to_vec(&document).expect("cluster document"))
            .expect("private cluster document");
        std::fs::copy(data.join("charters/chat.txt"), config.join("charters/chat.txt")).expect("configuration charter");
    }
}

// Observe the worker's own received assignments and successful agent spawns.
const WIRE_BYTES: usize = 65_536;
// jig's protocol/channel.md, 4.1: Assignment starts with the task's u64.
const ASSIGNMENT: u16 = 0x0180;
#[derive(Default)]
pub(crate) struct Runs {
    fd: Option<skein_io::kernel::Fd>,
    bytes: Vec<u8>,
    pending: Option<u64>,
    pub(crate) tasks: Vec<u64>,
}
impl Runs {
    pub(crate) const MEMORY: u64 = (WIRE_BYTES * 4) as u64;
    pub(crate) fn assigned(&self) -> bool {
        self.pending.is_some() || !self.tasks.is_empty()
    }
    pub(crate) fn complete(&mut self, complete: &Complete, engine: SocketAddr) {
        match (&complete.kind, &complete.result) {
            (Op::Recv { fd, buf }, Ok(Done::Count(count))) if Some(*fd) == self.fd => {
                self.bytes.extend_from_slice(&buf[..*count as usize]);
                assert!(self.bytes.len() <= WIRE_BYTES, "bounded worker wire");
                while self.bytes.len() >= 8 {
                    let length = u32::from_be_bytes(self.bytes[4..8].try_into().expect("frame length")) as usize;
                    if self.bytes.len() < 8 + length {
                        break;
                    }
                    let kind = u16::from_be_bytes(self.bytes[..2].try_into().expect("frame kind"));
                    if kind == ASSIGNMENT {
                        self.pending =
                            Some(skein_lib::Reader::new(&self.bytes[8..8 + length]).u64().expect("assignment task"));
                    }
                    self.bytes.drain(..8 + length);
                }
            }
            (Op::Spawn { spawn }, Ok(Done::Spawned { .. })) if spawn.program.as_ref() == fixtures::AGENT => {
                self.tasks.push(self.pending.take().expect("agent follows assignment"));
            }
            (Op::Connect { fd, addr }, Ok(Done::Nothing)) if *addr == engine => {
                self.fd = Some(*fd);
                self.bytes.clear();
                self.pending = None;
            }
            _ => {}
        }
    }
}

/// Resolve the three independently owned scratch roots.
#[must_use]
pub fn directories_in(roots: [&Path; 3]) -> temper::directories::Directories {
    use std::os::unix::ffi::OsStrExt;
    let options = ["--config", "--data", "--cache"].into_iter().zip(roots);
    let args: Vec<Box<[u8]>> = options
        .into_iter()
        .flat_map(|(option, path)| [option.as_bytes().into(), path.as_os_str().as_bytes().into()])
        .collect();
    temper::directories::resolve(&args, &[], b"/").expect("fixture directories").0
}
