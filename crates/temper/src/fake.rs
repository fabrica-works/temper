//! The document form's files and peer loop (protocol/setup.md, 8.4).
pub mod git;
use skein_fake_peers::{forge as forge_peer, llm as llm_peer};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Submit};
use skein_lib::{List, Queue, Time, Token, Wall};
use skein_shell::{Fallback, Host as _, Origin, Setting, Settings};
use std::{
    cell::RefCell,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    rc::Rc,
};
use temper_fake::{
    credential::{self, credential},
    forge, limits, llm,
};

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
pub fn configure(arguments: &[Bytes], environment: &[EnvironmentEntry]) -> Result<super::Command, String> {
    let Some((verb, arguments)) = arguments.split_first() else {
        return Err("temper fake: expected llm, forge or git\n".into());
    };
    if verb.as_ref() == b"git" {
        return Ok(super::Command::FakeGit { arguments: arguments.into() });
    }
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
    Ok(super::Command::Fake {
        arguments: Arguments {
            kind,
            document: super::path(&resolved.value(0).bytes).to_path_buf(),
            ready,
            seed,
            observations,
        },
    })
}

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
    fn checkpoint_or_llm(&self) -> bool {
        self.shared.is_none() || self.checkpoint
    }
    fn fail(&mut self, name: &str, why: &dyn std::fmt::Display) {
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
                self.fail("observations", &"drain exceeds its byte bound");
                return;
            }
            if !self.buffer.is_empty() {
                let result = self.observations.as_mut().expect("writer has file").write_all(self.buffer.as_slice());
                self.buffer.clear();
                if let Err(why) = result {
                    let path = self.observation_path.as_ref().expect("writer has path").display().to_string();
                    self.fail(&path, &why);
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
                        self.fail("signals: cancel", &format_args!("{error:?}"));
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
                            self.fail("signals", &"read failed");
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
            let mut stdout = io::stdout();
            let result =
                stdout.write_all(&prefix).and_then(|()| writeln!(stdout, "{address}")).and_then(|()| stdout.flush());
            if let Err(why) = result {
                self.fail("stdout", &why);
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

#[cfg(test)]
mod tests;
