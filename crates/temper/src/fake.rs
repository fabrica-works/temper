//! The document form's files and peer loop (protocol/setup.md, 8.4).
use skein_shell::{Fallback, Origin, Setting, Settings};
use std::path::PathBuf;

pub const LLM_SETTINGS: Settings = Settings {
    tool: "temper",
    table: &[
        Setting { name: "document", value: "FILE", about: "The fake LLM's JSON document.", fallback: Fallback::Refuse },
        Setting {
            name: "ready",
            value: "PREFIX",
            about: "Print PREFIX and the bound address once.",
            fallback: Fallback::Text(""),
        },
        Setting {
            name: "seed",
            value: "N",
            about: "The domain and credential seed, a decimal u64.",
            fallback: Fallback::Text("0"),
        },
    ],
};
pub const FORGE_SETTINGS: Settings = Settings {
    tool: "temper",
    table: &[
        Setting {
            name: "document",
            value: "FILE",
            about: "The fake forge's JSON document.",
            fallback: Fallback::Refuse,
        },
        Setting {
            name: "ready",
            value: "PREFIX",
            about: "Print PREFIX and the bound address once.",
            fallback: Fallback::Text(""),
        },
        Setting {
            name: "seed",
            value: "N",
            about: "The domain and credential seed, a decimal u64.",
            fallback: Fallback::Text("0"),
        },
        Setting {
            name: "observations",
            value: "FILE",
            about: "Create the forge's JSON lines observation stream.",
            fallback: Fallback::Text(""),
        },
    ],
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Llm,
    Forge,
}
impl Kind {
    fn command(self) -> &'static str {
        match self {
            Self::Llm => "temper fake llm",
            Self::Forge => "temper fake forge",
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Arguments {
    pub kind: Kind,
    pub document: PathBuf,
    pub ready: Option<Box<[u8]>>,
    pub seed: u64,
    pub observations: Option<PathBuf>,
}
type Bytes = Box<[u8]>;
type EnvironmentEntry = (Bytes, Bytes);
pub fn configure(arguments: &[Bytes], environment: &[EnvironmentEntry]) -> Result<Arguments, String> {
    let Some((verb, arguments)) = arguments.split_first() else {
        return Err("temper fake: expected llm or forge\n".into());
    };
    let (kind, table) = match verb.as_ref() {
        b"llm" => (Kind::Llm, &LLM_SETTINGS),
        b"forge" => (Kind::Forge, &FORGE_SETTINGS),
        _ => return Err(format!("temper fake: unknown fake {}\n", String::from_utf8_lossy(verb))),
    };
    let resolved = skein_shell::resolve(table, arguments, environment)
        .map_err(|why| super::prefixed(kind.command(), &why.to_string()))?;
    if let Some(word) = resolved.rest().first() {
        return Err(super::prefixed(kind.command(), &format!("unexpected argument {}", String::from_utf8_lossy(word))));
    }
    let seed = &resolved.value(2).bytes;
    let seed = std::str::from_utf8(seed)
        .ok()
        .filter(|text| !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|text| text.parse::<u64>().ok())
        .ok_or_else(|| super::prefixed(kind.command(), "--seed N (TEMPER_SEED): expected a decimal u64"))?;
    // shell.md 6.5 has no absent setting; Default alone means absent.
    // In particular --ready "" is a present, empty prefix.
    let ready = if resolved.value(1).origin == Origin::Default { None } else { Some(resolved.value(1).bytes.clone()) };
    let observations = if kind == Kind::Forge && resolved.value(3).origin != Origin::Default {
        Some(super::path(&resolved.value(3).bytes).to_path_buf())
    } else {
        None
    };
    Ok(Arguments { kind, document: super::path(&resolved.value(0).bytes).to_path_buf(), ready, seed, observations })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn configured(arguments: &[&str], environment: &[(&str, &str)]) -> Result<super::super::Command, String> {
        let arguments = arguments.iter().map(|s| s.as_bytes().into()).collect::<Vec<_>>();
        let environment =
            environment.iter().map(|(k, v)| (k.as_bytes().into(), v.as_bytes().into())).collect::<Vec<_>>();
        super::super::configure_with(&arguments, &environment, b"/work")
    }
    #[test]
    fn arguments() {
        for (words, environment, kind, document, ready, seed, observations) in [
            (vec!["fake", "llm", "--document", "llm.json"], vec![], Kind::Llm, "llm.json", None, 0, None),
            (vec!["fake", "forge", "--document", "forge.json"], vec![], Kind::Forge, "forge.json", None, 0, None),
            (
                vec!["fake", "llm", "--document", "llm.json", "--ready", "ready ", "--seed", "23"],
                vec![],
                Kind::Llm,
                "llm.json",
                Some("ready "),
                23,
                None,
            ),
            (
                vec!["fake", "forge", "--document", "forge.json", "--ready", "", "--observations", "seen"],
                vec![],
                Kind::Forge,
                "forge.json",
                Some(""),
                0,
                Some("seen"),
            ),
            (
                vec!["fake", "forge"],
                vec![
                    ("TEMPER_DOCUMENT", "env.json"),
                    ("TEMPER_READY", "up "),
                    ("TEMPER_SEED", "42"),
                    ("TEMPER_OBSERVATIONS", "seen"),
                ],
                Kind::Forge,
                "env.json",
                Some("up "),
                42,
                Some("seen"),
            ),
            (
                vec!["fake", "llm", "--document", "flag.json", "--seed", "7"],
                vec![("TEMPER_DOCUMENT", "env.json"), ("TEMPER_SEED", "42")],
                Kind::Llm,
                "flag.json",
                None,
                7,
                None,
            ),
        ] {
            let Command::Fake { arguments } = configured(&words, &environment).expect("valid fixture") else {
                panic!("fake command");
            };
            assert_eq!(
                arguments,
                Arguments {
                    kind,
                    document: document.into(),
                    ready: ready.map(|s| s.as_bytes().into()),
                    seed,
                    observations: observations.map(Into::into)
                }
            );
        }
        for (words, expected) in [
            (vec!["fake"], "temper fake: expected llm or forge\n"),
            (vec!["fake", "other"], "temper fake: unknown fake other\n"),
            (vec!["fake", "llm"], "temper fake llm: --document FILE is not given, and TEMPER_DOCUMENT is not set\n"),
            (
                vec!["fake", "llm", "--document", "doc", "--seed", "no"],
                "temper fake llm: --seed N (TEMPER_SEED): expected a decimal u64\n",
            ),
            (
                vec!["fake", "llm", "--document", "doc", "--observations", "seen"],
                "temper fake llm: unexpected argument --observations\n",
            ),
            (vec!["fake", "forge", "--document", "doc", "extra"], "temper fake forge: unexpected argument extra\n"),
        ] {
            assert_eq!(configured(&words, &[]).expect_err("refused fixture"), expected);
        }
    }
    use crate::Command;
}

use skein_fake_peers::{forge as forge_peer, llm as llm_peer};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Submit};
use skein_lib::{List, Queue, Time, Token, Wall};
use skein_shell::Host as _;
use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    rc::Rc,
};
use temper_fake::{
    credential::{self, credential},
    forge, limits, llm,
};

/// Creates every output before opening the peer or kernel. A refused start rolls
/// back only what this invocation created; a running peer leaves every file.
pub fn run(arguments: &Arguments) -> Result<u8, String> {
    let command = arguments.kind.command();
    let mut created = Created::default();
    let opened = start(arguments, &mut created);
    let (mut host, mut kernel) = match opened {
        Ok(opened) => opened,
        Err(why) => {
            created.rollback();
            return Err(super::prefixed(command, &why));
        }
    };
    let exit = skein_shell::drive(&mut kernel, &skein_shell::Clock::new(), &mut host);
    Ok(match exit {
        Exit::Code(code) => code,
        Exit::Signal(_) => 1,
    })
}
#[derive(Default)]
struct Created {
    files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}
impl Created {
    fn file(&mut self, path: &Path, bytes: &[u8]) -> Result<File, String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|why| format!("{}: {why}", path.display()))?;
        self.files.push(path.into());
        file.write_all(bytes).map_err(|why| format!("{}: {why}", path.display()))?;
        Ok(file)
    }
    fn directory(&mut self, path: &Path) -> Result<(), String> {
        let mut missing = Vec::new();
        for parent in path.ancestors() {
            match fs::symlink_metadata(parent) {
                Ok(_) => break,
                Err(why) if why.kind() == io::ErrorKind::NotFound => missing.push(parent.to_path_buf()),
                Err(why) => return Err(format!("{}: {why}", parent.display())),
            }
        }
        // Record before creation, so a partially successful mkdir is also undone.
        self.directories.extend(missing.into_iter().rev());
        super::directories::make_private(path, skein_shell::effective_user())
    }
    fn rollback(self) {
        for path in self.files.into_iter().rev() {
            let _removed = fs::remove_file(path);
        }
        for path in self.directories.into_iter().rev() {
            let _removed = fs::remove_dir(path);
        }
    }
}
fn start(arguments: &Arguments, created: &mut Created) -> Result<(Host, skein_shell::Kernel), String> {
    let bytes = skein_shell::read_file(&arguments.document, temper_fake::DOCUMENT_BYTES)
        .map_err(|why| format!("{}: {}", arguments.document.display(), super::unread(why)))?;
    let (peer, shared, observations) = match arguments.kind {
        Kind::Llm => {
            let document =
                llm::Document::decode(&bytes).map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
            let key = credential(arguments.seed, credential::KEY);
            created.file(super::path(document.key()), &key)?;
            if let Some(root) = document.root() {
                created.file(super::path(root), skein_fake_peers::transport::ROOT)?;
            }
            let observations = arguments.observations.as_ref().map(|path| created.file(path, b"")).transpose()?;
            let peer = llm::peer(llm::configuration(&document, &key, arguments.seed))
                .map_err(|why| format!("peer: {why:?}"))?;
            (Peer::Llm(peer), None, observations)
        }
        Kind::Forge => {
            let document =
                forge::Document::decode(&bytes).map_err(|why| String::from_utf8_lossy(&why.line()).into_owned())?;
            let token = credential(arguments.seed, credential::ADMIN_TOKEN);
            let path = super::path(document.admin_token());
            created.directory(path.parent().expect("absolute token file"))?;
            created.file(path, &token)?;
            let observations = arguments.observations.as_ref().map(|path| created.file(path, b"")).transpose()?;
            let (peer, shared) =
                forge::peer(&document, &token, arguments.seed).map_err(|why| format!("peer: {why:?}"))?;
            (Peer::Forge(peer), Some(shared), observations)
        }
    };
    let signals = skein_shell::open_termination_signals().map_err(|why| format!("signals: errno {why}"))?;
    let operations = peer.host().operations().checked_add(2).expect("signal read and cancel");
    let host = Host {
        peer,
        shared,
        signals,
        signal: Signal::New,
        cancel: Cancel::Idle,
        completions: Queue::with_capacity(operations),
        submissions: Queue::with_capacity(operations),
        ready: arguments.ready.clone(),
        observation_path: arguments.observations.clone(),
        observations,
        writer: arguments
            .observations
            .as_ref()
            .map(|_| skein_fake_forge_protocol::observations::Writer::new(&forge::LIMITS.limits)),
        buffer: List::with_capacity(if arguments.observations.is_some() { limits::OBSERVATION_BYTES } else { 0 }),
        stopped: false,
        peer_submissions: 0,
        checkpoint: false,
        failed: false,
        command: arguments.kind.command(),
    };
    let kernel = match skein_shell::Kernel::open(skein_shell::Config { operations: host.operations() }) {
        Ok(kernel) => kernel,
        Err(why) => {
            super::close(vec![signals]);
            return Err(why.to_string());
        }
    };
    Ok((host, kernel))
}
#[expect(
    clippy::large_enum_variant,
    reason = "one peer lives on the host stack; the 384-byte difference needs no heap indirection"
)]
enum Peer {
    Llm(llm_peer::Peer),
    Forge(forge_peer::Peer),
}
impl Peer {
    fn host(&self) -> &dyn skein_shell::Host {
        match self {
            Self::Llm(p) => p,
            Self::Forge(p) => p,
        }
    }
    fn host_mut(&mut self) -> &mut dyn skein_shell::Host {
        match self {
            Self::Llm(p) => p,
            Self::Forge(p) => p,
        }
    }
    fn address(&self) -> Option<std::net::SocketAddr> {
        match self {
            Self::Llm(p) => p.address(),
            Self::Forge(p) => p.address(),
        }
    }
    fn shutdown(&mut self) {
        match self {
            Self::Llm(p) => p.shutdown(),
            Self::Forge(p) => p.shutdown(),
        }
    }
}
// Both own tokens have slot u32::MAX, which io never issues: at skein
// 9d34791e, crates/skein-io/src/layer.rs:291-303 uses Id::token;
// crates/skein-lib/src/id.rs:41-44 puts the slot in the high half, and
// crates/skein-lib/src/slab.rs:49-56 allocates only 0..capacity (u32).
const SIGNAL: Token = Token::new(u64::MAX);
const CANCEL_SIGNAL: Token = Token::new(u64::MAX - 1);
#[derive(PartialEq, Eq)]
enum Signal {
    New,
    Reading,
    Close,
    Closing,
    Closed,
}
#[derive(PartialEq, Eq)]
enum Cancel {
    Idle,
    Waiting,
}
struct Host {
    peer: Peer,
    shared: Option<Rc<RefCell<forge_peer::Forge>>>,
    signals: Fd,
    signal: Signal,
    cancel: Cancel,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    ready: Option<Box<[u8]>>,
    observation_path: Option<PathBuf>,
    observations: Option<File>,
    writer: Option<skein_fake_forge_protocol::observations::Writer>,
    buffer: List<u8>,
    stopped: bool,
    peer_submissions: u32,
    checkpoint: bool,
    failed: bool,
    command: &'static str,
}
impl Host {
    fn failed(&mut self, name: &str, why: &dyn std::fmt::Display) {
        eprintln!("{}: {name}: {why}", self.command);
        self.failed = true;
        self.stopped = true;
        self.peer.shutdown();
        if self.signal == Signal::Reading && self.cancel == Cancel::Idle {
            self.submissions.push(Submit { op: CANCEL_SIGNAL, kind: Op::Cancel { target: SIGNAL } });
            self.cancel = Cancel::Waiting;
        }
    }
    /// The host has ended sending after an unwritten pass: `BrokenPipe`, as
    /// after this side's Shutdown (skein 9d34791e, crates/skein-io/src/kernel.rs:809-812).
    /// Unlike `Cancelled`, this reaches `broken` in `open_sent` and `Flush::Done` in
    /// `closing_sent` (crates/skein-io/src/stream.rs:1186-1190,1364-1366).
    /// The protocol closes a failed stream; `Close` on `Broken` cancels its `Recv`,
    /// settles both completions, then releases the fd (stream.rs:501,
    /// 1423-1449,1476-1497,1515-1530), even if the client keeps it open.
    fn suppress_sends(&mut self) {
        for _ in 0..self.submissions.len() {
            let submission = self.submissions.pop().expect("one pass over submissions");
            if matches!(submission.kind, Op::Send { .. }) {
                self.completions.push(Complete {
                    op: submission.op,
                    kind: submission.kind,
                    result: Err(skein_io::kernel::Error::BrokenPipe),
                });
            } else {
                self.submissions.push(submission);
            }
        }
    }
    fn observations(&mut self) {
        let Some(shared) = &self.shared else { return };
        let stopped = self.stopped && self.peer.host().is_empty();
        let mut forge = shared.borrow_mut();
        // A later checkpoint would certify observations this FILE has lost.
        // After failure every pass, including stop, only drains and drops.
        if !self.failed
            && let Some(writer) = &mut self.writer
        {
            let result = if stopped && !self.checkpoint {
                writer.stop(&mut forge.domain, &mut self.buffer)
            } else {
                writer.drain(&mut forge.domain, &mut self.buffer)
            };
            self.checkpoint |= stopped;
            drop(forge);
            if result.is_err() {
                self.buffer.clear();
                self.failed("observations", &"drain exceeds its byte bound");
                return;
            }
            if !self.buffer.is_empty() {
                let result = self.observations.as_mut().expect("writer has file").write_all(self.buffer.as_slice());
                self.buffer.clear();
                if let Err(why) = result {
                    let path = self.observation_path.as_ref().expect("writer has path").display().to_string();
                    self.failed(&path, &why);
                }
            }
        } else {
            while forge.domain.pop_observation().is_some() {}
            self.checkpoint |= stopped;
        }
    }
}
impl skein_shell::Host for Host {
    fn iterate(&mut self, now: Time, wall: Wall) {
        for _ in 0..self.completions.len() {
            let complete = self.completions.pop().expect("one pass over completions");
            if complete.op != SIGNAL && complete.op != CANCEL_SIGNAL && self.peer.host_mut().completions().room() == 0 {
                self.completions.push(complete);
                break;
            }
            if complete.op == CANCEL_SIGNAL {
                self.cancel = Cancel::Idle;
                match complete.result {
                    Ok(Done::Nothing) | Err(skein_io::kernel::Error::TooLate) => {}
                    Err(error) => {
                        // InvalidArgument/Other leave the read running (kernel.rs:45-46).
                        // Retry its cancellation; a descriptor closes only after its read.
                        self.failed("signals: cancel", &format_args!("{error:?}"));
                    }
                    Ok(_) => panic!("cancel completes with Nothing"),
                }
            } else if complete.op == SIGNAL {
                match self.signal {
                    Signal::Reading => {
                        self.signal = Signal::Close;
                        if !(matches!(complete.result, Ok(Done::Signalled(_)))
                            || self.failed && complete.result == Err(skein_io::kernel::Error::Cancelled))
                        {
                            self.failed("signals", &"read failed");
                        }
                        self.stopped = true;
                        self.peer.shutdown();
                    }
                    Signal::Closing => self.signal = Signal::Closed,
                    Signal::New | Signal::Close | Signal::Closed => panic!("one own operation"),
                }
            } else {
                self.peer.host_mut().completions().push(complete);
            }
        }
        if self.signal == Signal::New && self.failed {
            self.signal = Signal::Close;
        }
        if self.signal == Signal::New {
            self.submissions.push(Submit { op: SIGNAL, kind: Op::ReadSignal { fd: self.signals } });
            self.signal = Signal::Reading;
        }
        if self.signal == Signal::Close && self.cancel == Cancel::Idle {
            self.submissions.push(Submit { op: SIGNAL, kind: Op::Close { fd: self.signals } });
            self.signal = Signal::Closing;
        }
        self.peer.host_mut().iterate(now, wall);
        while self.submissions.room() > 0 {
            let Some(submission) = self.peer.host_mut().submissions().pop() else { break };
            self.submissions.push(submission);
        }
        self.peer_submissions = self.peer.host_mut().submissions().len();
    }
    fn drain(&mut self) {
        if let Some(address) = self.peer.address()
            && let Some(prefix) = self.ready.take()
        {
            let mut stdout = io::stdout().lock();
            let result =
                stdout.write_all(&prefix).and_then(|()| writeln!(stdout, "{address}")).and_then(|()| stdout.flush());
            if let Err(why) = result {
                self.failed("stdout", &why);
            }
        }
        self.observations();
        if self.failed {
            self.suppress_sends();
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        self.peer.host().work_pending(now)
            || self.peer_submissions > 0
            || !self.completions.is_empty()
            || !self.submissions.is_empty()
            || (self.signal == Signal::Close && self.cancel == Cancel::Idle)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.peer.host().next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.peer.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.peer.host().is_empty()
            && self.signal == Signal::Closed
            && self.cancel == Cancel::Idle
            && self.checkpoint_or_llm()
            && self.submissions.is_empty()
            && self.completions.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.peer.host().worst_case()
            + List::<u8>::worst_case(self.buffer.capacity()).expect("bounded observation buffer")
            + if self.writer.is_some() { u64::from(limits::OBSERVATION_STATE_BYTES) } else { 0 }
            + self.ready.as_ref().map_or(0, |prefix| u64::try_from(prefix.len()).expect("ready bytes"))
            + self
                .observation_path
                .as_ref()
                .map_or(0, |path| u64::try_from(path.as_os_str().len()).expect("path bytes"))
            + Queue::<Complete>::worst_case(self.completions.capacity()).expect("queue bound")
            + Queue::<Submit>::worst_case(self.submissions.capacity()).expect("queue bound")
    }
    fn operations(&self) -> u32 {
        self.peer.host().operations().checked_add(2).expect("signal read and cancel")
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(u8::from(self.failed)))
    }
}
impl Host {
    fn checkpoint_or_llm(&self) -> bool {
        self.shared.is_none() || self.checkpoint
    }
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    use std::io::Read;

    fn advance(host: &Host, content: &[u8]) {
        let mut shared = host.shared.as_ref().expect("forge").borrow_mut();
        let env = skein_lib::Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge::LIMITS };
        skein_fake_forge_domain::advance(
            &mut shared.domain,
            &env,
            b"fake/widget",
            b"main",
            b"new",
            content,
            forge::OWNER,
        )
        .expect("valid fixture");
    }

    #[test]
    fn observation_write_failure_drains_the_peer_and_keeps_its_outputs() {
        // The read-only descriptor fails writes with EBADF without damaging the
        // real FILE. Restoring write access after failure detects later writes,
        // including a misleading checkpoint at stop. The tiny buffer also checks
        // the drain's bound failure under the same refused-reply contract.
        for (name, checkpoint, bound_failure) in
            [("first", false, false), ("checkpoint", true, false), ("bound", true, true)]
        {
            let directory =
                std::env::temp_dir().join(format!("temper-fake-write-failure-{}-{name}", std::process::id()));
            super::super::directories::make_private(&directory, skein_shell::effective_user()).expect("valid fixture");
            let document = directory.join("forge.json");
            fs::write(
                &document,
                serde_json::to_vec(&serde_json::json!({"listen":{"socket":"127.0.0.1:0"},
                "admin_token":directory.join("forge/admin_token"),"owner":"fake",
                "repositories":[{"name":"widget","landing":"main","ci":null}]}))
                .expect("valid fixture"),
            )
            .expect("valid fixture");
            let path = directory.join("observations");
            let arguments = Arguments {
                kind: Kind::Forge,
                document: document.clone(),
                ready: None,
                seed: 23,
                observations: Some(path.clone()),
            };
            let (mut host, mut kernel) = start(&arguments, &mut Created::default()).expect("valid fixture");
            if checkpoint {
                advance(&host, b"before");
                host.observations();
            }
            let clock = skein_shell::Clock::new();
            let deadline = clock.now().now.saturating_add(skein_lib::Duration::from_millis(300));
            let mut client = None;
            let before = loop {
                assert!(clock.now().now < deadline, "{name}: reaches the reply pass");
                kernel.reap(host.completions());
                let skein_shell::Now { now, wall } = clock.now();
                host.iterate(now, wall);
                if client.is_none()
                    && let Some(address) = host.peer.address()
                {
                    let mut socket = std::net::TcpStream::connect(address).expect("client");
                    socket.set_read_timeout(Some(std::time::Duration::from_millis(300))).expect("timeout");
                    let token = credential(23, credential::ADMIN_TOKEN);
                    write!(socket, "GET /api/v1/repositories/1 HTTP/1.1\r\nHost: localhost\r\nAuthorization: token {}\r\nConnection: keep-alive\r\n\r\n",
                        std::str::from_utf8(&token).expect("hex")).expect("request");
                    client = Some(socket);
                }
                let reply = host.submissions.iter().any(|submission| matches!(submission.kind, Op::Send { .. }));
                if reply {
                    // A request is in this pass, with its reply still unsubmitted.
                    advance(&host, b"lost");
                    if bound_failure {
                        host.buffer = List::with_capacity(1);
                    } else {
                        host.observations = Some(File::open(&path).expect("read-only writer seam"));
                    }
                }
                let before = fs::read(&path).expect("FILE before this pass");
                host.drain();
                if reply {
                    assert!(host.failed, "{name}: the pass's write fails");
                    assert!(
                        host.completions.iter().any(|c| matches!(c.kind, Op::Send { .. })
                            && c.result == Err(skein_io::kernel::Error::BrokenPipe)),
                        "{name}: reply refused"
                    );
                    host.observations = Some(OpenOptions::new().append(true).open(&path).expect("restore writer"));
                    advance(&host, b"later");
                    break before;
                }
                kernel.submit(host.submissions(), skein_shell::Wait::No);
            };
            // The client's sending half remains open: settling must cancel Recv.
            assert_eq!(skein_shell::drive(&mut kernel, &clock, &mut host), Exit::Code(1), "{name}");
            let mut client = client.expect("open client");
            let mut bytes = Vec::new();
            match client.read_to_end(&mut bytes) {
                Ok(_) => {}
                Err(why) if why.kind() == io::ErrorKind::ConnectionReset => {}
                Err(why) => panic!("{name}: client end: {why}"),
            }
            assert!(bytes.is_empty(), "{name}: no reply bytes");
            assert!(host.is_empty());
            assert_eq!(kernel.in_flight(), 0);
            assert!(host.shared.as_ref().expect("forge").borrow_mut().domain.pop_observation().is_none());
            assert_eq!(fs::read(&path).expect("FILE"), before, "{name}: no checkpoint after loss");
            if checkpoint {
                let records = String::from_utf8(before).expect("JSON lines");
                assert_eq!(records.lines().last(), Some("{\"observations_lost\":0}"));
            } else {
                assert!(before.is_empty(), "{name}: the first write failed");
            }
            assert!(document.exists());
            assert!(directory.join("forge/admin_token").exists());
            fs::remove_dir_all(directory).expect("valid fixture");
        }
    }
}
