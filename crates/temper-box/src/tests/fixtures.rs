use crate::{
    limits,
    machine::{Input, Machine, Order},
};
use alloc::boxed::Box;
use skein_io::kernel::Exit;
use skein_lib::{Duration, List, Queue, Time, bytes};
use temper_forge_setup::Step;
pub const PREPARE: usize = 0;
pub const FORGEJO: usize = 1;
pub const RUNNER: usize = 2;
pub const RECONCILE_1: usize = 3;
pub const RECONCILE_2: usize = 4;
pub const SERVE: usize = 5;
pub fn step(name: &[u8], long: bool) -> Step {
    command(name, &[], long)
}
pub fn owned_steps() -> Box<[Step]> {
    Box::new([
        step(b"prepare", false),
        step(b"forgejo", true),
        step(b"runner", true),
        step(b"reconcile-1", false),
        step(b"reconcile-2", false),
        step(b"serve", true),
    ])
}
#[derive(Debug)]
pub struct Rig {
    pub machine: Machine,
    pub out: Queue<Order>,
    pub now: Time,
}
impl Rig {
    pub fn new(limits: &limits::Limits) -> Self {
        let machine = Machine::new(
            limits,
            &crate::layout::flatten(temper_forge_setup::Steps { always: owned_steps(), asked: Box::new([]) }),
        )
        .expect("valid limits");
        let out = Queue::with_capacity(machine.max_out().saturating_mul(4));
        let mut rig = Self { machine, out, now: Time::ZERO };
        rig.machine.start(rig.now, &[], &mut rig.out);
        rig
    }
    pub fn running() -> Self {
        let mut rig = Self::new(&limits::small());
        rig.completed(PREPARE);
        rig.started(FORGEJO);
        rig.started(RUNNER);
        rig.completed(RECONCILE_1);
        rig.completed(RECONCILE_2);
        rig.started(SERVE);
        rig.orders();
        rig
    }
    pub fn input(&mut self, input: Input) {
        self.machine.input(self.now, input, &mut self.out);
    }
    pub fn started(&mut self, step: usize) {
        self.input(Input::Started { step });
    }
    pub fn exited(&mut self, step: usize, exit: Exit) {
        self.input(Input::Exited { step, exit });
    }
    pub fn closed(&mut self, step: usize) {
        self.input(Input::Closed { step });
    }
    pub fn completed(&mut self, step: usize) {
        self.started(step);
        self.exited(step, Exit::Code(0));
        self.closed(step);
    }
    pub fn advance(&mut self, by: Duration) {
        self.now = self.now.saturating_add(by);
        self.machine.fire(self.now, &mut self.out);
    }
    pub fn terminate(&mut self) {
        self.input(Input::Terminate);
    }
    pub fn orders(&mut self) -> Box<[Order]> {
        let mut orders = List::with_capacity(self.out.len());
        for _ in 0..self.out.len() {
            orders.push(self.out.pop().expect("counted order")).expect("counted capacity");
        }
        orders.into_boxed()
    }
    pub fn failed_long(&mut self, step: usize) {
        self.exited(step, Exit::Code(1));
        self.closed(step);
        self.orders();
    }
    pub fn restart_forgejo(&mut self) {
        self.completed(PREPARE);
        self.started(FORGEJO);
        self.orders();
    }
}

use crate::host::{Service, Settings};
use skein_fake_machine::Machine as FakeMachine;
use skein_io::kernel::{Complete, Done, Fd, Op, ServiceSignal, Submit};
use skein_lib::{Token, Wall};
use skein_shell::Host;
use skein_sim::{Answer, Ask, Call, Config, Handle, Pid, Program, Reply, Sim};

pub fn once(name: &[u8], arguments: &[&[u8]]) -> Step {
    command(name, arguments, false)
}
pub fn long(name: &[u8], arguments: &[&[u8]]) -> Step {
    command(name, arguments, true)
}
fn command(name: &[u8], arguments: &[&[u8]], long: bool) -> Step {
    let capacity = u32::try_from(arguments.len()).expect("small fixture").saturating_add(1);
    let mut args = List::with_capacity(capacity);
    for argument in arguments {
        args.push(bytes::copy_of(argument)).expect("argument slots");
    }
    args.push(bytes::copy_of(name)).expect("name last");
    Step::new(bytes::copy_of(name), args.into_boxed(), long)
}
#[derive(Debug)]
pub struct Launch {
    pub name: Box<[u8]>,
    pub environment: Box<[Box<[u8]>]>,
    pub at: Time,
}
#[derive(Debug)]
pub struct Hosted {
    pid: Pid,
    descriptors: Box<[Fd]>,
    closed: bool,
}
#[derive(Debug)]
pub struct Harness {
    pub sim: Sim,
    pub pid: Pid,
    pub service: Service,
    pub launches: List<Launch>,
    pub machine: FakeMachine,
    pub signals: Fd,
    pub stderr: Fd,
    pub calls: Queue<Call>,
    pub answers: Queue<Answer>,
    pub stubborn: bool,
    pub hosted: List<Hosted>,
    pub pending: List<Token>,
}
impl Harness {
    pub fn new(steps: &[Step]) -> Self {
        Self::with_stubborn(steps, false)
    }
    pub fn with_stubborn(steps: &[Step], stubborn: bool) -> Self {
        let mut sim = Sim::new(0x12_02, Config::calm());
        let pid = sim.spawn_process();
        let signals = sim.open_signal_source(pid);
        let stdout = sim.open_inherited_write(pid);
        let stderr = sim.open_inherited_write(pid);
        let mut machine = FakeMachine::new();
        let opened = machine.lay(&[]);
        let root = sim.root(pid, Handle::new(opened.raw()));
        let settings = Settings {
            program: bytes::copy_of(b"process_fixture"),
            environment: Box::new([bytes::copy_of(b"HOME=/box"), bytes::copy_of(b"TOKEN=whole=value")]),
            plain: bytes::copy_of(b"serve"),
            root,
            signals,
            stdout,
            stderr,
        };
        let service = Service::new(&limits::small(), Box::from(steps), settings).expect("valid limits");
        Self {
            sim,
            pid,
            service,
            launches: List::with_capacity(128),
            machine,
            signals,
            stderr,
            calls: Queue::with_capacity(64),
            answers: Queue::with_capacity(64),
            stubborn,
            hosted: List::with_capacity(64),
            pending: List::with_capacity(64),
        }
    }
    fn answer(&mut self) {
        self.sim.calls(&mut self.calls);
        for _ in 0..self.calls.len() {
            let call = self.calls.pop().expect("counted call");
            if let Ask::Spawn { args, env, .. } = &call.ask {
                self.launches
                    .push(Launch {
                        name: args.last().expect("step name last").clone(),
                        environment: env.clone(),
                        at: self.sim.now(),
                    })
                    .expect("bounded launches");
                if self.stubborn {
                    let (_, token, _) = self.sim.hosted_spawn(call.ticket);
                    self.pending.push(token).expect("bounded hosted launches");
                }
            }
            skein_fake_machine::step(&mut self.machine, call, &mut self.answers);
            if self.stubborn {
                let mut answer = self.answers.pop().expect("one answer");
                if let Ok(Reply::Program(Program::Never)) = &answer.result {
                    answer.result = Ok(Reply::Program(Program::Service));
                }
                self.answers.push(answer);
            }
            self.sim.answer(&mut self.answers);
        }
    }
    fn bind(&mut self) {
        for complete in self.service.completions().iter() {
            if self.pending.as_slice().contains(&complete.op)
                && let Ok(Done::Spawned { pidfd }) = complete.result
            {
                let (pid, inherited) = self.sim.bind_service(self.pid, pidfd);
                let mut descriptors = List::with_capacity(3);
                for (_, fd) in inherited {
                    descriptors.push(fd).expect("two inherited pipes");
                }
                descriptors.push(self.sim.open_signal_source(pid)).expect("child signal source");
                self.hosted
                    .push(Hosted { pid, descriptors: descriptors.into_boxed(), closed: false })
                    .expect("bounded hosted children");
            }
        }
    }
    fn settle_hosted(&mut self) {
        for index in 0..self.hosted.len() {
            let child = self.hosted.get_mut(index).expect("hosted index");
            if !child.closed && !self.sim.service_running(child.pid) {
                let mut submissions = Queue::with_capacity(3);
                for (index, fd) in child.descriptors.iter().enumerate() {
                    submissions.push(Submit {
                        op: Token::new(u64::try_from(index).expect("three descriptors")),
                        kind: Op::Close { fd: *fd },
                    });
                }
                self.sim.submit(child.pid, &mut submissions);
                let mut completions = Queue::<Complete>::with_capacity(3);
                self.sim.reap(child.pid, &mut completions);
                self.sim.assert_hosted_settled(child.pid);
                child.closed = true;
            }
        }
    }
    pub fn turn(&mut self) {
        self.sim.reap(self.pid, self.service.completions());
        self.bind();
        self.service.iterate(self.sim.now(), Wall::EPOCH);
        self.service.drain();
        self.sim.submit(self.pid, self.service.submissions());
        self.answer();
        self.settle_hosted();
    }
    pub fn run_until_idle(&mut self) {
        for _ in 0..10_000_u32 {
            self.turn();
            if !self.service.work_pending(self.sim.now())
                && self.sim.next_due().is_none()
                && self.sim.ready(self.pid) == 0
                && !self.sim.deferred(self.pid)
            {
                return;
            }
        }
        unreachable!("box reaches its steady state");
    }
    pub fn run_to_end(&mut self) -> Exit {
        for _ in 0..10_000_u32 {
            self.turn();
            if let Some(exit) = self.service.exit() {
                return exit;
            }
            if !self.service.work_pending(self.sim.now()) {
                let due = match (self.sim.next_due(), self.service.next_deadline()) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, None) | (None, a) => a,
                };
                if let Some(at) = due {
                    self.sim.advance_to(at);
                }
            }
        }
        unreachable!("box settles within bounded turns");
    }
    pub fn terminate(&mut self) {
        self.sim.deliver_service_signal(self.pid, self.signals, ServiceSignal::Terminate);
    }
    pub fn spawn_times(&self, name: &[u8]) -> Box<[Time]> {
        let mut times = List::with_capacity(128);
        for launch in &self.launches {
            if launch.name.as_ref() == name {
                times.push(launch.at).expect("bounded launches");
            }
        }
        times.into_boxed()
    }
    pub fn stderr(&mut self) -> Box<[u8]> {
        self.sim.peer_drain(self.pid, self.stderr, 65_536).into_boxed_slice()
    }
    pub fn contains(bytes: &[u8], needle: &[u8]) -> bool {
        for part in bytes.windows(needle.len()) {
            if part == needle {
                return true;
            }
        }
        false
    }
}
