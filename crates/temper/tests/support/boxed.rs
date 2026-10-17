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
    path::Path,
    rc::Rc,
};

/// What the referee does after the link arrives.
#[derive(Clone, Copy, Debug)]
pub enum Plan {
    /// Probe Forgejo and ask the box to stop.
    Stop,
}
/// Evidence retained after every process and pipe settles.
pub struct Outcome {
    pub exit: Option<Exit>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub forgejo_at_link: Option<u16>,
    pub left: Option<Vec<u32>>,
}
struct Judge {
    plan: Plan,
    forgejo: SocketAddr,
    forgejo_at_link: Rc<Cell<Option<u16>>>,
    sent: bool,
    cleanup: bool,
    deadline: Time,
    settled: bool,
}
impl Referee<Process> for Judge {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let Process::Stdout(stdout) = &procs[1] else { panic!("stdout reader") };
        let linked = printed_link(&stdout.bytes).is_some();
        let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
        if !self.sent && linked {
            match self.plan {
                Plan::Stop => {
                    self.forgejo_at_link.set(status(self.forgejo, "/api/v1/version"));
                    binary.signal(Signal::Terminate);
                    self.sent = true;
                }
            }
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
pub fn run(directory: &Path, forgejo: SocketAddr, plan: Plan, deadline: Duration) -> Outcome {
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: vec!["box".into(), "config.json".into()],
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
    let mut world = real::World::new(Judge {
        plan,
        forgejo,
        forgejo_at_link: forgejo_at_link.clone(),
        sent: false,
        cleanup: false,
        deadline: clock.now().now.saturating_add(Duration::from_nanos(
            deadline.as_nanos().saturating_sub(Duration::from_secs(5).as_nanos()),
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
