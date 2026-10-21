//! Drives the box through its pipes, signals and supervised process tree.
use super::{Process, StdoutReader, printed_link};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Expectation, Mode, Streams},
    real,
};
use std::{
    cell::Cell,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    rc::Rc,
};

/// What the referee does after the link arrives.
#[derive(Clone, Copy, Debug)]
pub enum Plan {
    /// Probe Forgejo and ask the box to stop.
    Stop,
    /// Kill Forgejo, wait for its replacement, and check the engine before stopping.
    KillForgejo,
}
/// Evidence retained after every process and pipe settles.
pub struct Outcome {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub forgejo_at_link: Option<u16>,
    pub left: Option<Vec<u32>>,
    pub forgejo_pids: Option<(u32, u32)>,
    pub engine_served: Option<bool>,
}
#[derive(Clone, Copy)]
enum Phase {
    Linked,
    Killed { engine: u32, forgejo: u32 },
    Stopping,
}
struct Judge {
    plan: Plan,
    forgejo: Option<SocketAddr>,
    forgejo_at_link: Rc<Cell<Option<u16>>>,
    forgejo_pids: Rc<Cell<Option<(u32, u32)>>>,
    engine_served: Rc<Cell<Option<bool>>>,
    temper_program: PathBuf,
    forgejo_program: Option<PathBuf>,
    phase: Phase,
    next_probe: Time,
    cleanup: bool,
    deadline: Time,
    settled: bool,
}
impl Judge {
    fn advance(&mut self, binary: &mut Binary, address: SocketAddr) {
        let pids = &binary.tree_status().live_pids;
        match self.phase {
            Phase::Linked => {
                self.forgejo_at_link.set(self.forgejo.and_then(|address| status(address, "/api/v1/version")));
                match self.plan {
                    Plan::Stop => self.stop(binary),
                    Plan::KillForgejo => {
                        if let Some(engine) = find(pids, &self.temper_program, b"serve")
                            && let Some(forgejo) =
                                find(pids, self.forgejo_program.as_deref().expect("Forgejo restart plan"), b"web")
                        {
                            let killed = std::process::Command::new("/usr/bin/kill")
                                .args(["-KILL", &forgejo.to_string()])
                                .status()
                                .expect("kill Forgejo");
                            assert!(killed.success(), "Forgejo's pid accepted the kill");
                            self.phase = Phase::Killed { engine, forgejo };
                        }
                    }
                }
            }
            Phase::Killed { engine, forgejo } => {
                if let Some(restarted) =
                    find(pids, self.forgejo_program.as_deref().expect("Forgejo restart plan"), b"web")
                    && restarted != forgejo
                    && self.forgejo.and_then(|address| status(address, "/api/v1/version")) == Some(200)
                {
                    self.forgejo_pids.set(Some((forgejo, restarted)));
                    self.engine_served.set(Some(
                        find(pids, &self.temper_program, b"serve") == Some(engine) && status(address, "/").is_some(),
                    ));
                    self.stop(binary);
                }
            }
            Phase::Stopping => {}
        }
    }
    fn stop(&mut self, binary: &mut Binary) {
        binary.signal(Signal::Terminate);
        self.phase = Phase::Stopping;
    }
}
fn find(pids: &[u32], program: &Path, argument: &[u8]) -> Option<u32> {
    pids.iter().copied().find(|pid| {
        std::fs::read_link(format!("/proc/{pid}/exe")).ok().as_deref() == Some(program)
            && std::fs::read(format!("/proc/{pid}/cmdline"))
                .is_ok_and(|bytes| bytes.split(|b| *b == 0).any(|arg| arg == argument))
    })
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let Process::Stdout(stdout) = &procs[1] else { panic!("stdout reader") };
        let engine = printed_link(&stdout.bytes)
            .and_then(|link| std::str::from_utf8(link).ok()?.strip_prefix("http://")?.split('/').next()?.parse().ok());
        let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
        if let Some(engine) = engine
            && now >= self.next_probe
        {
            self.advance(binary, engine);
            self.next_probe = now.saturating_add(Duration::from_millis(200));
        }
        let left = binary.tree_status().live_at_leader_exit.as_ref().is_some_and(|pids| !pids.is_empty());
        if !self.cleanup && (left || now >= self.deadline) && !binary.is_empty() {
            binary.signal_tree(Signal::Kill);
            self.cleanup = true;
        }
    }
    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.settled = procs.iter().all(Host::is_empty);
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.cleanup && !self.settled).then_some(self.deadline)
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
/// Starts the shipped box and drives its referee within the supplied deadline.
#[must_use]
pub fn run(directory: &Path, forgejo: Option<SocketAddr>, plan: Plan, deadline: Duration) -> Outcome {
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: std::iter::once("box".into()).chain(super::configuration::flags(directory)).collect(),
            directory: directory.into(),
            environment: Vec::new(),
        },
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("temper box starts");
    let streams = binary.take_streams().expect("binary streams");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let forgejo_at_link = Rc::new(Cell::new(None));
    let forgejo_pids = Rc::new(Cell::new(None));
    let engine_served = Rc::new(Cell::new(None));
    let forgejo_program = if matches!(plan, Plan::KillForgejo) {
        Some(std::fs::canonicalize(super::owned::binaries().0).expect("Forgejo path"))
    } else {
        None
    };
    let mut world = real::World::new(Judge {
        plan,
        forgejo,
        forgejo_at_link: forgejo_at_link.clone(),
        forgejo_pids: forgejo_pids.clone(),
        engine_served: engine_served.clone(),
        temper_program: std::fs::canonicalize(env!("CARGO_BIN_EXE_temper")).expect("temper path"),
        forgejo_program,
        phase: Phase::Linked,
        next_probe: Time::ZERO,
        cleanup: false,
        deadline: clock.now().now.saturating_add(Duration::from_nanos(
            deadline.as_nanos().saturating_sub(Duration::from_secs(5).as_nanos()).max(deadline.as_nanos() / 2),
        )),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Stdout(Box::new(StdoutReader::new(input, output))));
    let ended = world.run(&clock, deadline);
    let Process::Binary(binary) = &ended.procs[0] else { panic!("binary") };
    let Process::Stdout(stdout) = &ended.procs[1] else { panic!("stdout reader") };
    Outcome {
        exit: binary.exit_status(),
        stdout: stdout.bytes.clone(),
        stderr: binary.stderr().to_vec(),
        forgejo_at_link: forgejo_at_link.get(),
        left: binary.tree_status().live_at_leader_exit.clone(),
        forgejo_pids: forgejo_pids.get(),
        engine_served: engine_served.get(),
    }
}
/// The status line of a plain request, or none when no server answers.
#[must_use]
pub fn status(address: SocketAddr, target: &str) -> Option<u16> {
    let timeout = std::time::Duration::from_millis(100);
    let mut socket = TcpStream::connect_timeout(&address, timeout).ok()?;
    socket.set_read_timeout(Some(timeout)).ok()?;
    socket.set_write_timeout(Some(timeout)).ok()?;
    write!(socket, "GET {target} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n").ok()?;
    let mut bytes = [0_u8; 128];
    let length = socket.read(&mut bytes).ok()?;
    std::str::from_utf8(&bytes[..length]).ok()?.split_whitespace().nth(1)?.parse().ok()
}

/// The first run's evidence, including a check made while the box still serves.
pub struct StoryOutcome<T> {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub left: Option<Vec<u32>>,
    pub finished: bool,
    pub checked: T,
}
struct StoryJudge<'a, F, T> {
    check: Option<F>,
    checked: &'a Cell<Option<T>>,
    stopping: bool,
    cleanup: bool,
    settled: bool,
    deadline: Time,
}
impl<F: FnOnce() -> T, T> Referee<Process> for StoryJudge<'_, F, T> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        let finished = person.finished && person.client.as_ref().is_some_and(Host::is_empty);
        let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
        if !self.stopping && finished {
            self.checked.set(Some(self.check.take().expect("one check before stopping")()));
            binary.signal(Signal::Terminate);
            self.stopping = true;
        }
        let left = binary.tree_status().live_at_leader_exit.as_ref().is_some_and(|pids| !pids.is_empty());
        if !self.cleanup && (left || now >= self.deadline) && !binary.is_empty() {
            binary.signal_tree(Signal::Kill);
            self.cleanup = true;
        }
    }
    fn observe(&mut self, now: Time, procs: &[Process]) {
        self.settled = self.stopping && procs.iter().all(Host::is_empty);
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        let Process::Person(person) = &procs[1] else { panic!("native person") };
        let stderr = String::from_utf8_lossy(binary.stderr());
        if person.failed.is_some() {
            for node in person.client.as_ref().expect("native client").tree().nodes() {
                eprintln!(
                    "{:?}: {} {}",
                    node.role,
                    String::from_utf8_lossy(node.name.as_deref().unwrap_or_default()),
                    node.text.iter().map(|run| String::from_utf8_lossy(&run.text)).collect::<String>()
                );
            }
        }
        assert!(person.failed.is_none(), "{:?}; temper box stderr: {stderr}", person.failed);
        assert!(binary.exit_status().is_none() || self.stopping, "temper box ended before the person: {stderr}");
        assert!(binary.exit_status().is_none_or(|exit| exit == Exit::Code(0)), "temper box failed: {stderr}");
        assert!(!binary.stderr_overflowed(), "temper box overflowed stderr: {stderr}");
        assert!(now < self.deadline || self.settled, "native person did not settle: {stderr}");
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "box's person did not finish and settle".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
/// Runs the shipped box on its pipes with a fresh client rooted at `client`.
/// Checks the forge after the person finishes and their client settles, before SIGTERM.
#[must_use]
pub fn run_story<T>(
    directory: &Path,
    flags: Vec<String>,
    client: &Path,
    script: super::person::Script,
    deadline: Duration,
    check: impl FnOnce() -> T,
) -> StoryOutcome<T> {
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: std::iter::once("box".into()).chain(flags.into_iter().map(Into::into)).collect(),
            directory: directory.into(),
            // Init's optional git/claude discovery is exercised without PATH.
            // The demo uses the written link to this binary, and needs neither.
            environment: Vec::new(),
        },
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("temper box starts");
    let streams = binary.take_streams().expect("binary pipes");
    let mut fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes mode") };
    let root = skein_shell::open_root(client).expect("client's parent root");
    fds.push(root);
    let checked = Cell::new(None);
    let mut world = real::World::new(StoryJudge {
        check: Some(check),
        checked: &checked,
        stopping: false,
        cleanup: false,
        settled: false,
        deadline: clock.now().now.saturating_add(Duration::from_nanos(
            deadline.as_nanos().saturating_sub(Duration::from_secs(5).as_nanos()),
        )),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Person(Box::new(super::person::Person::new(root, input, output, script))));
    let ended = world.run(&clock, deadline);
    assert!(ended.procs.iter().all(Host::is_empty), "all real io settles");
    let Process::Binary(binary) = &ended.procs[0] else { panic!("binary") };
    let Process::Person(person) = &ended.procs[1] else { panic!("native person") };
    StoryOutcome {
        exit: binary.exit_status(),
        stdout: person.stdout().to_vec(),
        stderr: binary.stderr().to_vec(),
        left: binary.tree_status().live_at_leader_exit.clone(),
        finished: person.finished,
        checked: checked.take().expect("the finished person's forge check ran"),
    }
}
