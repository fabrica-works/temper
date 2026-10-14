//! Shared configuration fixtures for startup and the process tier.
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub enum Forge {
    Unreached,
}
fn private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}
pub fn configuration(directory: &Path, forge: &Forge) {
    let Forge::Unreached = forge;
    assert!(Path::new("/usr/bin/git").is_file(), "/usr/bin/git is required");
    for name in ["accounts", "accounts/records", "secrets", "charters", "state"] {
        fs::create_dir(directory.join(name)).expect("deployment directory");
    }
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        private(&directory.join(std::str::from_utf8(&name).expect("fixture path")), bytes.expose());
    }
    private(&directory.join("worker.secret"), b"worker-secret");
    private(
        &directory.join("accounts/chat.json"),
        br#"{"access_token":"token","expires_at":4102444800,"account_id":"acc"}"#,
    );
    private(&directory.join("secrets/forge-1.api-token"), b"forge-1-api-token-value");
    private(&directory.join("secrets/forge-1.git-token"), b"forge-1-git-token-value");
    private(&directory.join("root.der"), skein_tls_world::pki::ROOT);
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":600,"read_window":256,
            "conversations":1,"calls_per_response":1},
            "policy":{"max_turns":5,"group_stop":10,"inbox":8,"unacknowledged":8}},
        "endpoints":[{"name":"fake","number":0,"dialect":0,"account":1,"provider":"codex",
            "address":"127.0.0.1:34443","transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),
            "identity":"plain","connect":1000,"handshake":1000,
            "models":[{"name":"fake","window":1_048_576,"output":1024,"reasoning_item":64,"head":60000,"idle":30000}]}],
        "environment":[]});
    private(&directory.join("agent.json"), &serde_json::to_vec(&agent).expect("agent JSON"));
    let mut value: serde_json::Value =
        serde_json::from_slice(&jig_channel_world::fixtures::engine_document()).expect("engine JSON");
    value.as_object_mut().expect("whole object").remove("service");
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_world::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"]["program"] = env!("CARGO_BIN_EXE_temper").into();
    value["workers"][0]["agent"]["configuration"] = directory.join("agent.json").to_str().expect("agent path").into();
    value["workers"][0]["agent"]["arguments"] = serde_json::json!(["agent", directory.join("agent.json")]);
    value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 598.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["clients"]["listener"] = serde_json::json!({"socket":"127.0.0.1:0"});
    value["engine"]["connectors"] = serde_json::json!([1]);
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:34443"});
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    value["application"] = serde_json::json!({
        "forges":[{"number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:9"},"transport":"plaintext",
            "git":"http://127.0.0.1:9","api_token":"secrets/forge-1.api-token","user":42,"identity":1}],
        "identities":[{"number":1,"token":"secrets/forge-1.git-token","login":"temper",
            "author":{"name":"Temper","email":"temper@example.test"}}],
        "landing":{"deployment":[],"projects":[]},"charter":1,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,
            "checkout.message_bytes":4096,"forge.name_bytes":64,"forge.resources_per_task":2,"git_component.stdout_bytes":2048}},
        "workers":[{"root":"worker-root","git":"/usr/bin/git"}]});
    private(&directory.join("config.json"), &serde_json::to_vec(&value).expect("deployment JSON"));
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
    Peer(Box<skein_fake_peers::llm::Peer>),
}
impl Process {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(x) => x.as_ref(),
            Self::Stdout(x) => x.as_ref(),
            Self::Peer(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(x) => x.as_mut(),
            Self::Stdout(x) => x.as_mut(),
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
fn run(directory: &Path, signal: Signal, deadline: Duration, peer: Option<skein_fake_peers::llm::Peer>) -> Ended {
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
    world.spawn_with_fds(fds, || {
        Process::Stdout(Box::new(StdoutReader {
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            completions: Queue::with_capacity(8),
            submissions: Queue::with_capacity(8),
        }))
    });
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
    run(directory, signal, deadline, None)
}
