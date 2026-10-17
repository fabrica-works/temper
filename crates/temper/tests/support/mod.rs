//! Shared configuration fixtures for startup and the process tier.
pub mod configuration;
pub mod owned;
pub mod sign_in;
pub mod terminal;
pub const WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(30);
pub mod forgejo;
pub mod live;
pub mod person;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub enum Forge<'a> {
    Unreached,
    Throwaway { api: std::net::SocketAddr, git: String, token: &'a str, user: u64 },
}
pub(super) fn write_private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}
pub fn configuration(directory: &Path, forge: &Forge<'_>) {
    assert!(Path::new("/usr/bin/git").is_file(), "/usr/bin/git is required");
    for name in ["accounts", "accounts/records", "secrets", "charters", "state"] {
        fs::create_dir(directory.join(name)).expect("deployment directory");
    }
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        write_private(&directory.join(std::str::from_utf8(&name).expect("fixture path")), bytes.expose());
    }
    write_private(&directory.join("worker.secret"), b"worker-secret");
    write_private(
        &directory.join("accounts/chat.json"),
        br#"{"access_token":"token","expires_at":4102444800,"account_id":"acc"}"#,
    );
    write_private(&directory.join("secrets/forge-1.user"), b"42\n");
    write_private(&directory.join("secrets/forge-1.api-token"), b"forge-1-api-token-value");
    write_private(&directory.join("secrets/forge-1.git-token"), b"forge-1-git-token-value");
    write_private(&directory.join("root.der"), skein_tls_world::pki::ROOT);
    let (agent, value) = configuration::documents(directory, env!("CARGO_BIN_EXE_temper"));
    write_private(&directory.join("agent.json"), &agent);
    let mut value: serde_json::Value = serde_json::from_slice(&value).expect("deployment JSON");
    forgejo::configuration(directory, forge, &mut value);
    write_private(&directory.join("config.json"), &serde_json::to_vec(&value).expect("deployment JSON"));
}

/// A case's scratch, retained if a test panics.
pub struct Scratch {
    path: std::path::PathBuf,
}
impl Scratch {
    #[must_use]
    pub fn new(case: &str) -> Self {
        Self::at("process", case)
    }
    #[must_use]
    pub fn startup(case: &str) -> Self {
        Self::at("startup", case)
    }
    fn at(tier: &str, case: &str) -> Self {
        let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(tier).join(case);
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove prior scratch");
        }
        fs::create_dir_all(&path).expect("scratch directory");
        Self { path }
    }
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn remove(self) {
        fs::remove_dir_all(self.path).expect("scratch cleanup");
    }
}

use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Signal, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Mode, Streams},
    real,
};

#[must_use]
pub fn serve(directory: &Path) -> Binary {
    Binary::start(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: vec!["serve".into(), "config.json".into()],
            directory: directory.into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        65_536,
    )
    .expect("temper serve starts")
}
#[must_use]
pub fn fake_llm(scripts: Box<[skein_fake_llm_domain::api::Script]>) -> skein_fake_peers::llm::Peer {
    smith_agent_process_world::fake::configured_transport(
        scripts,
        skein_llm::Credential { access_token: b"token".as_slice().into(), account_id: b"acc".as_slice().into() },
        Duration::ZERO,
        skein_fake_peers::Transport::Tls,
    )
}
#[must_use]
pub fn printed_link(stdout: &[u8]) -> Option<&[u8]> {
    let end = stdout.iter().position(|byte| *byte == b'\n')?;
    Some(&stdout[..end])
}
pub struct Ended {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
const PIPE: Token = Token::new(1);
const CLOSE: Token = Token::new(2);
struct StdoutReader {
    input: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    closing: u32,
    bytes: Vec<u8>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl StdoutReader {
    fn new(input: Fd, output: Fd) -> Self {
        Self {
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            completions: Queue::with_capacity(8),
            submissions: Queue::with_capacity(8),
        }
    }
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.submissions.push(Submit { op: CLOSE, kind: Op::Close { fd } });
    }
}
impl Host for StdoutReader {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            if complete.op == CLOSE {
                self.closing -= 1;
                complete.result.expect("pipe closed");
            } else {
                assert_eq!(complete.op, PIPE, "stdout read token");
                self.reading = false;
                let Op::PipeRead { buf, .. } = complete.kind else { panic!("stdout read") };
                let Done::Count(count) = complete.result.expect("stdout read completes") else { panic!("read count") };
                self.bytes.extend_from_slice(&buf[..usize::try_from(count).expect("bounded read")]);
                assert!(self.bytes.len() <= 65_536, "bounded printed links");
                if count == 0 {
                    let fd = self.output.take().expect("stdout pipe");
                    self.close(fd);
                }
            }
        }
        if let Some(fd) = self.input.take() {
            self.close(fd);
        }
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submissions
                .push(Submit { op: PIPE, kind: Op::PipeRead { fd, buf: vec![0; 1024].into_boxed_slice() } });
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.completions.is_empty() || self.input.is_some() || (!self.reading && self.output.is_some())
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.input.is_none() && self.output.is_none() && self.closing == 0
    }
    fn worst_case(&self) -> u64 {
        131_072
    }
    fn operations(&self) -> u32 {
        3
    }
}
enum Process {
    Binary(Box<Binary>),
    Stdout(Box<StdoutReader>),
    Person(Box<person::Person>),
    Peer(Box<skein_fake_peers::llm::Peer>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(x) => x.as_ref(),
            Self::Stdout(x) => x.as_ref(),
            Self::Person(x) => x.as_ref(),
            Self::Peer(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(x) => x.as_mut(),
            Self::Stdout(x) => x.as_mut(),
            Self::Person(x) => x.as_mut(),
            Self::Peer(x) => x.as_mut(),
        }
    }
}
impl Host for Process {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.host().worst_case()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
}
struct Judge {
    sent: bool,
    signal: Signal,
    deadline: Time,
    settled: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [Process]) {
        let Process::Stdout(stdout) = &procs[1] else { panic!("stdout reader") };
        if !self.sent && printed_link(&stdout.bytes).is_some() {
            let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
            binary.signal(self.signal);
            for proc in &mut *procs {
                if let Process::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
            self.sent = true;
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.settled = self.sent && procs.iter().all(Host::is_empty);
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        let Process::Stdout(stdout) = &procs[1] else { panic!("stdout reader") };
        assert!(
            self.sent || binary.exit_status().is_none(),
            "temper serve failed before its link: {}",
            String::from_utf8_lossy(binary.stderr())
        );
        assert!(
            now < self.deadline || self.settled,
            "signal sent: {}; stdout bytes: {}; exit: {:?}; binary empty: {}; stdout empty: {}; stderr: {}",
            self.sent,
            stdout.bytes.len(),
            binary.exit_status(),
            binary.is_empty(),
            stdout.is_empty(),
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline)
            .then(|| "temper serve did not print its link and drain within its deadline".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
fn run_signal(
    directory: &Path,
    signal: Signal,
    deadline: Duration,
    peer: Option<skein_fake_peers::llm::Peer>,
) -> Ended {
    let clock = Clock::new();
    let mut binary = serve(directory);
    let streams = binary.take_streams().expect("binary pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let mut world = real::World::new(Judge {
        sent: false,
        signal,
        deadline: clock.now().now.saturating_add(Duration::from_nanos(
            deadline.as_nanos().saturating_sub(Duration::from_secs(5).as_nanos()),
        )),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Stdout(Box::new(StdoutReader::new(input, output))));
    if let Some(peer) = peer {
        world.spawn(|| Process::Peer(Box::new(peer)));
    }
    let outcome = world.run(&clock, deadline);
    assert!(outcome.procs.iter().all(Host::is_empty), "all real io settles");
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("binary") };
    let Process::Stdout(stdout) = &outcome.procs[1] else { panic!("stdout reader") };
    Ended { exit: binary.exit_status(), stdout: stdout.bytes.clone(), stderr: binary.stderr().to_vec() }
}
#[must_use]
pub fn until_link_then(directory: &Path, signal: Signal, deadline: Duration) -> Ended {
    run_signal(directory, signal, deadline, None)
}

pub const ANSWER: &[u8] = b"Process answer";
pub const EDIT: &[u8] = b"Changed by temper.\n";
pub use forgejo::landing_scripts;
#[must_use]
pub fn chat_script(answer: &[u8]) -> skein_fake_llm_domain::api::Script {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let words = std::str::from_utf8(answer).expect("script answer text");
    Script {
        cue: Box::new([]),
        turns: Box::new([Turn {
            lines: Box::new([Line::Call {
                name: Box::from(*b"finish"),
                arguments: serde_json::to_vec(&serde_json::json!({"form":"report","text":words,"fields":{}}))
                    .expect("report JSON")
                    .into_boxed_slice(),
            }]),
            finish: Finish::ToolCalls,
            tokens: 20,
        }]),
    }
}
pub struct Seen {
    pub signed_in: u32,
    pub finished: bool,
    pub stderr: Vec<u8>,
}
struct ChatJudge<'a> {
    ended: bool,
    settled: bool,
    deadline: Time,
    forge: Option<&'a forgejo::Throwaway>,
    poll: Time,
    merged: bool,
}
impl Referee<Process> for ChatJudge<'_> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        if !self.ended && now >= self.poll {
            if let Some(forge) = self.forge {
                self.merged = forge.merged_pull().is_some();
            }
            self.poll = now.saturating_add(Duration::from_secs(2));
        }
        let Process::Person(person) = &procs[1] else {
            panic!("native person");
        };
        if !self.ended
            && person.finished
            && person.client.as_ref().is_some_and(Host::is_empty)
            && (self.forge.is_none() || self.merged)
        {
            let Process::Binary(binary) = &mut procs[0] else {
                panic!("binary");
            };
            binary.signal(Signal::Kill);
            for proc in procs {
                if let Process::Peer(peer) = proc {
                    peer.shutdown();
                }
            }
            self.ended = true;
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.settled = self.ended && procs.iter().all(Host::is_empty);
        let Process::Binary(binary) = &procs[0] else {
            panic!("binary");
        };
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        let queries = procs.iter().find_map(|proc| match proc {
            Process::Peer(peer) => Some(
                peer.observations()
                    .iter()
                    .filter(|observation| matches!(observation, skein_fake_peers::llm::Observation::Query { .. }))
                    .count(),
            ),
            Process::Binary(_) | Process::Person(_) | Process::Stdout(_) => None,
        });
        assert!(
            person.failed.is_none(),
            "{}; fake queries: {queries:?}; forge: {}; temper serve stderr: {}",
            person.failed.as_deref().unwrap_or("person finished"),
            if person.failed.is_some() {
                self.forge.map_or_else(|| "none".into(), forgejo::Throwaway::failure_state)
            } else {
                "person running".into()
            },
            String::from_utf8_lossy(binary.stderr())
        );
        assert!(
            binary.exit_status().is_none() || self.ended,
            "temper serve failed: {}",
            String::from_utf8_lossy(binary.stderr())
        );
        assert!(
            now < self.deadline || self.settled,
            "native chat did not settle: {}",
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(if self.forge.is_some() && !self.ended {
            self.poll.min(self.deadline)
        } else {
            self.deadline
        })
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "native person's chat did not finish and settle".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
#[must_use]
pub fn run(
    directory: &Path,
    script: person::Script,
    peer: Option<skein_fake_peers::llm::Peer>,
    deadline: Duration,
) -> Seen {
    run_story(directory, script, peer, deadline, None)
}
#[must_use]
pub fn run_until_landed(
    directory: &Path,
    script: person::Script,
    peer: skein_fake_peers::llm::Peer,
    forge: &forgejo::Throwaway,
) -> Seen {
    run_story(directory, script, Some(peer), Duration::from_secs(180), Some(forge))
}
fn run_story(
    directory: &Path,
    script: person::Script,
    peer: Option<skein_fake_peers::llm::Peer>,
    deadline: Duration,
    forge: Option<&forgejo::Throwaway>,
) -> Seen {
    let clock = Clock::new();
    let mut binary = serve(directory);
    let streams = binary.take_streams().expect("binary pipes");
    let mut fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else {
        panic!("pipes mode");
    };
    let root = skein_shell::open_root(directory).expect("client's parent root");
    fds.push(root);
    let mut world = real::World::new(ChatJudge {
        ended: false,
        settled: false,
        forge,
        poll: clock.now().now,
        merged: false,
        deadline: clock.now().now.saturating_add(Duration::from_nanos(
            deadline.as_nanos().saturating_sub(Duration::from_secs(5).as_nanos()),
        )),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Person(Box::new(person::Person::new(root, input, output, script))));
    if let Some(peer) = peer {
        world.spawn(|| Process::Peer(Box::new(peer)));
    }
    let outcome = world.run(&clock, deadline);
    assert!(outcome.procs.iter().all(Host::is_empty), "all real io settles");
    let Process::Binary(binary) = &outcome.procs[0] else {
        panic!("binary");
    };
    let Process::Person(person) = &outcome.procs[1] else {
        panic!("native person");
    };
    Seen {
        signed_in: person.client.as_ref().expect("native client").seen().signed_in,
        finished: person.finished,
        stderr: binary.stderr().to_vec(),
    }
}

#[must_use]
pub fn private(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o777 == 0o600)
}

/// One complete stdout line; the read finishes before the pipe changes hosts.
#[must_use]
pub fn read_line(output: Fd, within: Duration) -> Vec<u8> {
    let clock = Clock::new();
    let deadline = clock.now().now.saturating_add(within);
    let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: 1 }).expect("link reader ring");
    let mut complete = Queue::with_capacity(1);
    let mut submit = Queue::with_capacity(1);
    let mut bytes = Vec::new();
    loop {
        submit.push(Submit { op: PIPE, kind: Op::PipeRead { fd: output, buf: vec![0; 1024].into_boxed_slice() } });
        loop {
            assert!(clock.now().now < deadline, "serve prints a link within its deadline");
            kernel.submit(&mut submit, skein_shell::Wait::Until(deadline));
            kernel.reap(&mut complete);
            if !complete.is_empty() {
                break;
            }
        }
        let complete = complete.pop().expect("read completed");
        let Op::PipeRead { buf, .. } = complete.kind else { panic!("link read") };
        let Done::Count(count) = complete.result.expect("link read succeeded") else { panic!("read count") };
        assert_ne!(count, 0, "serve ended before its link");
        bytes.extend_from_slice(&buf[..usize::try_from(count).expect("read count fits")]);
        assert!(bytes.len() <= 65_536, "bounded service output");
        if let Some(link) = printed_link(&bytes) {
            return link.to_vec();
        }
    }
}
