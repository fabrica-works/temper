//! The process tier's reader of lab/cluster (protocol/README.md, 15.5).
use super::{StdoutReader, WITHIN};
use serde_json::Value;
use skein_io::kernel::{Complete, Exit, Signal, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Expectation, Method, Mode, Streams},
    real,
};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::DirBuilderExt,
    path::{Component, Path, PathBuf},
};

// qa.md, 10.1: temper-m1-calibration, the approved class the cluster runs under until it has its own.
const READY_SECONDS: u64 = 60;
const STOP_SECONDS: u64 = 10;
// lab/cluster/engine.json's fleet.grace_ms: above each worker's stop bound, 2300 (jig-protocol-config worker.rs:38), which a hello must be under (jig-domain-fleet channel.rs:174); reached only on a lost link.
pub(crate) const FLEET_GRACE_MS: u64 = 3000;
const _: () = assert!(FLEET_GRACE_MS < STOP_SECONDS * 1000);
const CGROUPS: &str = "the cluster's tests need cgroups delegated by the user's service manager (temper docs/design/protocol/README.md, 15.5): run them through heavy";

/// lab/cluster/ read, rendered for one test's scratch, and its processes' state.
pub struct Cluster {
    experiment: PathBuf,
    specs: Vec<Value>,
    bindings: BTreeMap<String, String>,
    /// The settled cgroup preflight's wall time, including its world.
    pub probe_cost: Duration,
}
fn private_directory(path: &Path) {
    fs::DirBuilder::new().recursive(true).mode(0o700).create(path).expect("private cluster directory");
}
fn contained(base: &Path, path: &str) -> PathBuf {
    assert!(
        !path.is_empty() && Path::new(path).components().all(|c| matches!(c, Component::Normal(_))),
        "cluster path: {path}"
    );
    base.join(path)
}
fn string(value: &Value) -> &str {
    value.as_str().expect("cluster string")
}
fn list(value: &Value) -> &[Value] {
    value.as_array().expect("cluster list")
}
impl Cluster {
    /// Read the description and require delegated cgroups before any member starts.
    #[must_use]
    pub fn read(scratch: &Path) -> Self {
        let probe_cost = probe(scratch);
        let mut cluster = Self::description(scratch);
        cluster.probe_cost = probe_cost;
        cluster
    }
    fn description(scratch: &Path) -> Self {
        let experiment =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lab/cluster").canonicalize().expect("lab/cluster");
        let value: Value =
            serde_json::from_slice(&fs::read(experiment.join("experiment.json")).expect("cluster description"))
                .expect("cluster JSON");
        assert_eq!(value["format"], "lab-experiment/1");
        assert_eq!(value["name"], "cluster");
        assert_eq!(value["cost"], "temper-m1-calibration");
        assert!(list(&value["dirs"]).is_empty(), "cluster has no run directories");
        let specs = list(&value["processes"]).to_vec();
        let mut bindings = BTreeMap::from([
            ("artifact.temper".into(), env!("CARGO_BIN_EXE_temper").into()),
            ("experiment".into(), experiment.to_str().expect("experiment path").into()),
        ]);
        for spec in &specs {
            let name = string(&spec["name"]);
            let dir = contained(scratch, name);
            private_directory(&dir);
            assert!(
                bindings.insert(format!("proc.{name}.dir"), dir.to_str().expect("process path").into()).is_none(),
                "duplicate process {name}"
            );
            assert!(
                spec["memory_bytes"].is_null() || spec["memory_bytes"].as_u64().is_some_and(|n| n > 0),
                "{name}'s memory bound"
            );
        }
        Self { experiment, specs, bindings, probe_cost: Duration::ZERO }
    }
    /// The process's directory, `{proc.P.dir}`.
    #[must_use]
    pub fn dir(&self, process: &str) -> PathBuf {
        self.bindings.get(&format!("proc.{process}.dir")).expect("named process directory").into()
    }
    fn index(&self, name: &str) -> usize {
        self.specs.iter().position(|s| s["name"] == name).unwrap_or_else(|| panic!("unknown cluster process {name}"))
    }
    fn render(&self, text: &str) -> String {
        let mut chars = text.chars().peekable();
        let mut out = String::new();
        while let Some(c) = chars.next() {
            match c {
                '{' if chars.peek() == Some(&'{') => {
                    chars.next();
                    out.push('{');
                }
                '}' if chars.peek() == Some(&'}') => {
                    chars.next();
                    out.push('}');
                }
                '{' => {
                    let mut name = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) if c != '{' => name.push(c),
                            _ => panic!("malformed cluster binding {{{name}"),
                        }
                    }
                    out.push_str(
                        self.bindings
                            .get(&name)
                            .unwrap_or_else(|| panic!("unknown or unavailable cluster binding {{{name}}}")),
                    );
                }
                '}' => panic!("unescaped cluster brace"),
                c => out.push(c),
            }
        }
        out
    }
    /// Render just the named files, also used by the cheap configuration check.
    pub fn render_files(&self, process: &str) {
        let spec = &self.specs[self.index(process)];
        for file in spec.get("files").map_or(&[][..], list) {
            let source = contained(&self.experiment, string(&file["from"]));
            let target = contained(&self.dir(process), string(&file["path"]));
            private_directory(target.parent().expect("file parent"));
            let bytes = self.render(&fs::read_to_string(&source).expect("cluster template"));
            super::write_private(&target, bytes.as_bytes());
        }
    }
    fn command(&self, process: &str) -> Command {
        self.render_files(process);
        let spec = &self.specs[self.index(process)];
        let argv: Vec<_> = list(&spec["argv"]).iter().map(|v| self.render(string(v))).collect();
        let (program, arguments) = argv.split_first().expect("cluster argv");
        assert!(Path::new(program).is_absolute(), "absolute cluster executable: {program}");
        Command {
            program: program.into(),
            arguments: arguments.iter().map(Into::into).collect(),
            directory: self.dir(process),
            environment: spec["env"]
                .as_object()
                .expect("cluster environment")
                .iter()
                .map(|(k, v)| (k.into(), self.render(string(v)).into()))
                .collect(),
        }
    }
    /// Render the documents without a process or cgroup probe, with explicit ready captures.
    #[must_use]
    pub fn documents(scratch: &Path, captures: &[(&str, &str, &str)]) -> Self {
        let mut cluster = Self::description(scratch);
        for (process, capture, value) in captures {
            cluster.bindings.insert(format!("proc.{process}.ready.{capture}"), (*value).into());
        }
        for process in ["engine", "w1", "w2"] {
            cluster.render_files(process);
        }
        cluster
    }
    /// Reserve a world root for each member before running the world.
    #[must_use]
    pub fn members(self) -> (Members, Vec<Root>) {
        let count = self.specs.len();
        let hosts = (0..count).map(|_| Root::Process(Box::new(Member::idle()))).collect();
        (
            Members {
                cluster: self,
                requested: vec![false; count],
                states: (0..count).map(|_| State::Waiting).collect(),
                started: Vec::new(),
                stopping: false,
                stopped: Vec::new(),
                failure: None,
            },
            hosts,
        )
    }
}

/// A cluster world root: one described process or the story's client calls.
pub enum Root {
    Process(Box<Member>),
    Client { calls: Option<Box<super::ctl::Calls>>, completions: Queue<Complete>, submissions: Queue<Submit> },
}
impl Root {
    #[must_use]
    pub fn client() -> Self {
        Self::Client { calls: None, completions: Queue::with_capacity(0), submissions: Queue::with_capacity(0) }
    }
    fn member(&self) -> &Member {
        let Self::Process(member) = self else { panic!("described process root") };
        member
    }
    fn member_mut(&mut self) -> &mut Member {
        let Self::Process(member) = self else { panic!("described process root") };
        member
    }
    fn host(&self) -> Option<&dyn Host> {
        match self {
            Self::Process(member) => Some(member.as_ref()),
            Self::Client { calls, .. } => calls.as_deref().map(|c| c as &dyn Host),
        }
    }
    fn host_mut(&mut self) -> Option<&mut dyn Host> {
        match self {
            Self::Process(member) => Some(member.as_mut()),
            Self::Client { calls, .. } => calls.as_deref_mut().map(|c| c as &mut dyn Host),
        }
    }
}
impl Host for Root {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if let Some(host) = self.host_mut() {
            host.iterate(now, wall);
        }
    }
    fn drain(&mut self) {
        if let Some(host) = self.host_mut() {
            host.drain();
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        match self {
            Self::Process(member) => member.completions(),
            Self::Client { calls: Some(calls), .. } => calls.completions(),
            Self::Client { completions, .. } => completions,
        }
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        match self {
            Self::Process(member) => member.submissions(),
            Self::Client { calls: Some(calls), .. } => calls.submissions(),
            Self::Client { submissions, .. } => submissions,
        }
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().is_some_and(|h| h.work_pending(now))
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().and_then(Host::next_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().and_then(Host::next_policy_deadline)
    }
    fn is_empty(&self) -> bool {
        self.host().is_none_or(Host::is_empty)
    }
    fn worst_case(&self) -> u64 {
        self.host().map_or(2_097_152, Host::worst_case)
    }
    fn operations(&self) -> u32 {
        self.host().map_or(32, Host::operations)
    }
}

/// A world root: waiting, then the described Binary and its output reader, then ended.
pub struct Member {
    command: Option<Command>,
    binary: Option<Binary>,
    stdout: Option<StdoutReader>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Member {
    fn idle() -> Self {
        Self {
            command: None,
            binary: None,
            stdout: None,
            completions: Queue::with_capacity(16),
            submissions: Queue::with_capacity(16),
        }
    }
}
impl Host for Member {
    fn iterate(&mut self, now: Time, wall: Wall) {
        if let Some(command) = self.command.take() {
            let mut binary = Binary::start_with_tree(command, Mode::Pipes, 65_536, Expectation::Supervise, false)
                .expect("cluster member starts");
            let Streams::Pipes { input, output } = binary.take_streams().expect("cluster pipes") else {
                panic!("pipes")
            };
            self.stdout = Some(StdoutReader::new(input, output));
            self.binary = Some(binary);
        }
        if let (Some(binary), Some(stdout)) = (&mut self.binary, &mut self.stdout) {
            // Two children share this root; the low token bit routes their completions.
            while let Some(mut c) = self.completions.pop() {
                let reader = c.op.raw() & 1 == 1;
                c.op = Token::new(c.op.raw() >> 1);
                if reader {
                    stdout.completions().push(c);
                } else {
                    binary.completions().push(c);
                }
            }
            binary.iterate(now, wall);
            stdout.iterate(now, wall);
            for (host, bit) in [(binary as &mut dyn Host, 0), (stdout as &mut dyn Host, 1)] {
                while let Some(mut s) = host.submissions().pop() {
                    s.op = Token::new(s.op.raw().checked_mul(2).expect("member operation token fits") | bit);
                    self.submissions.push(s);
                }
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.command.is_some()
            || !self.completions.is_empty()
            || self.binary.as_ref().is_some_and(|b| b.work_pending(now))
            || self.stdout.as_ref().is_some_and(|s| s.work_pending(now))
    }
    fn next_deadline(&self) -> Option<Time> {
        self.binary.as_ref().and_then(Host::next_deadline)
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.command.is_none()
            && self.binary.as_ref().is_none_or(Host::is_empty)
            && self.stdout.as_ref().is_none_or(Host::is_empty)
    }
    fn worst_case(&self) -> u64 {
        1_048_576
    }
    fn operations(&self) -> u32 {
        16
    }
}

enum State {
    Waiting,
    Starting { deadline: Time, captures: Vec<Option<String>> },
    Ready,
    Stopping { deadline: Time, killed: bool, survivors: Vec<u32> },
    Ended,
}
/// Every cluster story embeds this half in its own referee.
pub struct Members {
    cluster: Cluster,
    requested: Vec<bool>,
    states: Vec<State>,
    started: Vec<usize>,
    stopping: bool,
    stopped: Vec<Stopped>,
    failure: Option<String>,
}
/// The recorded exit of one member, in order of exit.
pub struct Stopped {
    pub name: String,
    pub exit: Option<Exit>,
    pub killed: bool,
    pub survivors: Vec<u32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
impl Members {
    /// Start these processes and their predecessors, ties in name order.
    pub fn up(&mut self, processes: &[&str]) {
        assert!(!self.stopping, "up before stop");
        for process in processes {
            self.request(self.cluster.index(process), &mut Vec::new());
        }
    }
    fn request(&mut self, index: usize, visiting: &mut Vec<usize>) {
        assert!(!visiting.contains(&index), "cluster after cycle");
        if self.requested[index] {
            return;
        }
        visiting.push(index);
        let after = self.after(index);
        for predecessor in after {
            self.request(predecessor, visiting);
        }
        visiting.pop();
        self.requested[index] = true;
    }
    fn after(&self, index: usize) -> Vec<usize> {
        self.cluster.specs[index]
            .get("after")
            .map_or(&[][..], list)
            .iter()
            .map(|p| self.cluster.index(string(p)))
            .collect()
    }
    #[must_use]
    pub fn is_up(&self) -> bool {
        !self.stopping
            && self.requested.iter().zip(&self.states).all(|(wanted, state)| !wanted || matches!(state, State::Ready))
    }
    #[must_use]
    pub fn capture(&self, process: &str, capture: &str) -> Option<&str> {
        self.cluster.bindings.get(&format!("proc.{process}.ready.{capture}")).map(String::as_str)
    }
    /// Terminate started leaders, one at a time, in reverse start order.
    pub fn stop(&mut self) {
        self.stopping = true;
    }
    #[must_use]
    pub fn stopped(&self) -> Option<&[Stopped]> {
        (self.stopping && self.stopped.len() == self.started.len()).then_some(&self.stopped)
    }
    pub fn act(&mut self, now: Time, hosts: &mut [Root]) {
        for &i in &self.started {
            let binary = hosts[i].member_mut().binary.as_mut().expect("started member iterated");
            let survivors = binary.tree_status().live_at_leader_exit.as_deref().unwrap_or(&[]);
            let timed_out =
                matches!(&self.states[i], State::Stopping { deadline, killed: false, .. } if now >= *deadline);
            let left_tree =
                binary.exit_status().is_some() && (!survivors.is_empty() || !binary.tree_status().live_pids.is_empty());
            if (left_tree || timed_out)
                && !matches!(self.states[i], State::Stopping { killed: true, .. } | State::Ended)
            {
                let mut survivors = survivors.to_vec();
                survivors.extend_from_slice(&binary.tree_status().live_pids);
                survivors.sort_unstable();
                survivors.dedup();
                self.states[i] = State::Stopping { deadline: now, killed: true, survivors };
                binary.signal_tree(Signal::Kill);
                self.stopping = true;
            }
        }
        if self.stopping {
            if let Some(&i) = self.started.iter().rev().find(|&&i| !matches!(self.states[i], State::Ended))
                && !matches!(self.states[i], State::Stopping { .. })
            {
                let binary = hosts[i].member_mut().binary.as_mut().expect("started binary");
                if binary.exit_status().is_none() {
                    binary.signal(Signal::Terminate);
                }
                self.states[i] = State::Stopping {
                    deadline: now.saturating_add(Duration::from_secs(STOP_SECONDS)),
                    killed: false,
                    survivors: Vec::new(),
                };
            }
            return;
        }
        let mut pending: Vec<_> = (0..self.states.len())
            .filter(|&i| {
                self.requested[i]
                    && matches!(self.states[i], State::Waiting)
                    && self.after(i).iter().all(|&p| matches!(self.states[p], State::Ready))
            })
            .collect();
        pending.sort_by_key(|&i| string(&self.cluster.specs[i]["name"]));
        for i in pending {
            let name = string(&self.cluster.specs[i]["name"]);
            hosts[i].member_mut().command = Some(self.cluster.command(name));
            self.states[i] = State::Starting {
                deadline: now.saturating_add(Duration::from_secs(READY_SECONDS)),
                captures: vec![None; list(&self.cluster.specs[i]["ready"]).len()],
            };
            self.started.push(i);
        }
    }
    pub fn observe(&mut self, now: Time, hosts: &[Root]) {
        for &i in &self.started {
            let host = hosts[i].member();
            let Some(binary) = &host.binary else { continue };
            let name = string(&self.cluster.specs[i]["name"]);
            assert!(!binary.stderr_overflowed(), "{name}'s stderr exceeded its bound");
            if let State::Starting { deadline, captures } = &mut self.states[i] {
                let rules = list(&self.cluster.specs[i]["ready"]);
                for (rule, captured) in rules.iter().zip(captures.iter_mut()) {
                    if captured.is_some() {
                        continue;
                    }
                    let bytes = match string(&rule["stream"]) {
                        "stdout" => &host.stdout.as_ref().expect("stdout").bytes,
                        "stderr" => binary.stderr(),
                        other => panic!("cluster ready stream {other}"),
                    };
                    // split_inclusive leaves a partial last line unmatched.
                    for line in bytes.split_inclusive(|b| *b == b'\n') {
                        if let Some(line) = line.strip_suffix(b"\n") {
                            let line = line.strip_suffix(b"\r").unwrap_or(line);
                            if let Some(suffix) = line.strip_prefix(string(&rule["prefix"]).as_bytes()) {
                                *captured = Some(std::str::from_utf8(suffix).expect("ready text").into());
                                break;
                            }
                        }
                    }
                }
                if captures.iter().all(Option::is_some) {
                    for (rule, captured) in rules.iter().zip(captures.iter()) {
                        if let Some(capture) = rule["capture"].as_str() {
                            self.cluster.bindings.insert(
                                format!("proc.{name}.ready.{capture}"),
                                captured.as_ref().expect("ready capture").clone(),
                            );
                        }
                    }
                    self.states[i] = State::Ready;
                } else if now >= *deadline {
                    self.failure = Some(format!(
                        "{name} did not become ready within {READY_SECONDS}s: {}",
                        String::from_utf8_lossy(binary.stderr())
                    ));
                    self.stopping = true;
                }
            }
            if binary.exit_status().is_some() && !matches!(self.states[i], State::Stopping { .. } | State::Ended) {
                self.failure =
                    Some(format!("{name} exited unexpectedly: {}", String::from_utf8_lossy(binary.stderr())));
                self.stopping = true;
                self.states[i] = State::Stopping {
                    deadline: now.saturating_add(Duration::from_secs(STOP_SECONDS)),
                    killed: false,
                    survivors: Vec::new(),
                };
            }
            if host.is_empty()
                && let State::Stopping { killed, survivors, .. } = &self.states[i]
            {
                assert_eq!(binary.counts().expect("settled tree").method, Method::Cgroup, "{CGROUPS}");
                self.stopped.push(Stopped {
                    name: name.into(),
                    exit: binary.exit_status(),
                    killed: *killed,
                    survivors: survivors.clone(),
                    stdout: host.stdout.as_ref().expect("stdout").bytes.clone(),
                    stderr: binary.stderr().to_vec(),
                });
                self.states[i] = State::Ended;
            }
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.states
            .iter()
            .filter_map(|s| match s {
                State::Starting { deadline, .. } if !self.stopping => Some(*deadline),
                State::Stopping { deadline, killed: false, .. } => Some(*deadline),
                State::Waiting | State::Starting { .. } | State::Ready | State::Stopping { .. } | State::Ended => None,
            })
            .min()
    }
    #[must_use]
    pub fn overdue(&self, _now: Time) -> Option<String> {
        None
    } // act escalates, then awaits settlement.
    #[must_use]
    pub fn passed(&self) -> bool {
        if self.stopped().is_none() {
            return false;
        }
        assert!(self.failure.is_none(), "{}", self.failure.as_deref().unwrap_or(""));
        for stopped in &self.stopped {
            assert!(!stopped.killed && stopped.survivors.is_empty(), "{} needed tree cleanup", stopped.name);
        }
        true
    }
}

struct Probe {
    settled: bool,
}
impl Referee<Binary> for Probe {
    fn act(&mut self, _now: Time, _hosts: &mut [Binary]) {}
    fn observe(&mut self, _now: Time, hosts: &[Binary]) {
        self.settled = hosts.iter().all(Host::is_empty);
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
fn probe(scratch: &Path) -> Duration {
    let clock = Clock::new();
    let began = clock.now().now;
    let binary = Binary::start_with_tree(
        Command {
            program: "/bin/true".into(),
            arguments: Vec::new(),
            environment: Vec::new(),
            directory: scratch.into(),
        },
        Mode::Pipes,
        1024,
        Expectation::Supervise,
        false,
    )
    .unwrap_or_else(|why| panic!("{CGROUPS}: {why:?}"));
    let mut world = real::World::new(Probe { settled: false });
    world.spawn_with_fds(binary.descriptors(), || binary);
    let outcome = world.run(&clock, WITHIN);
    let binary = &outcome.procs[0];
    assert_eq!(binary.exit_status(), Some(Exit::Code(0)), "cgroup probe exits successfully");
    assert_eq!(binary.counts().expect("probe settled").method, Method::Cgroup, "{CGROUPS}");
    clock.now().now.saturating_since(began)
}
