//! The machine on skein-io, with one bounded chunk per child pipe.
use crate::{
    limits::{Limits, Unfit},
    lines::{self, Lines},
    machine::{Input, Machine, Order, Outcome},
    sink::Sink,
};
use alloc::boxed::Box;
use skein_io::{
    self as io, Event, Io, Request,
    kernel::{self, Complete, Exit, Fd, Submit},
};
use skein_lib::{
    Decimal, Duration, Env, List, Queue, Time, Token, Wall,
    stream::{Down, Read, Up},
};
use temper_forge_setup::Step;
/// The descriptors and command inherited from the binary.
#[derive(Debug)]
pub struct Settings {
    pub program: Box<[u8]>,
    pub environment: Box<[Box<[u8]>]>,
    pub plain: Box<[u8]>,
    pub root: Fd,
    pub signals: Fd,
    pub stdout: Fd,
    pub stderr: Fd,
}
/// A longer line is emitted in pieces, each prefixed.
pub const LINE_BYTES: u32 = 4096;
const ROOT_CLOSE: Token = Token::new(u64::MAX);
#[derive(Debug)]
struct Pipe {
    token: Option<Token>,
    reading: bool,
    lines: Lines,
}
impl Pipe {
    fn new(plain: bool) -> Self {
        Self { token: None, reading: false, lines: Lines::new(plain) }
    }
    fn settled(&self) -> bool {
        self.token.is_none() && self.lines.is_empty()
    }
}
#[derive(Debug)]
struct Child {
    owner: Option<Token>,
    process: Option<Token>,
    pipes: [Pipe; 2],
    spawn: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    New,
    Running,
    Closing,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Root {
    Open,
    Closing,
    Closed,
}
/// All io and policy state owned by one box process.
#[derive(Debug)]
pub struct Service {
    io: Io,
    env: Env<io::Limits>,
    machine: Machine,
    steps: Box<[Step]>,
    settings: Settings,
    children: Box<[Child]>,
    sinks: [Sink; 2],
    signals: Token,
    phase: Phase,
    failed: bool,
    serial: u64,
    root: Root,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    told: Queue<Event>,
    requests: Queue<Request>,
    orders: Queue<Order>,
    memory: u64,
}
impl Service {
    /// Checks the policy limits and adopts the binary's descriptors.
    pub fn new(limits: &Limits, steps: Box<[Step]>, settings: Settings) -> Result<Self, Unfit> {
        let machine = Machine::new(limits, &steps)?;
        let count = u32::try_from(steps.len()).expect("steps fit u32");
        let mut bounds = io_limits(count);
        let mut children = List::with_capacity(count);
        let mut inputs = u64::try_from(settings.plain.len()).expect("plain name");
        for step in &steps {
            bounds.output = bounds.output.max(
                LINE_BYTES
                    .checked_add(u32::try_from(step.name().len()).expect("bounded name"))
                    .expect("bounded line")
                    .checked_add(3)
                    .expect("prefix"),
            );
            children
                .push(Child {
                    owner: None,
                    process: None,
                    pipes: [Pipe::new(lines::route(&settings.plain, step.name(), 1)), Pipe::new(false)],
                    spawn: false,
                })
                .expect("one child per step");
            inputs = inputs.saturating_add(u64::try_from(step.name().len()).expect("name length"));
            inputs = inputs.saturating_add(u64::try_from(size_of_val(step.arguments())).expect("argument slots"));
            for argument in step.arguments() {
                inputs = inputs.saturating_add(u64::try_from(argument.len()).expect("argument length"));
            }
        }
        inputs = inputs.saturating_add(
            u64::try_from(size_of_val(settings.environment.as_ref()))
                .expect("environment slots")
                .saturating_mul(u64::from(count).saturating_add(1)),
        );
        for value in &settings.environment {
            inputs = inputs.saturating_add(
                u64::try_from(value.len())
                    .expect("environment length")
                    .saturating_mul(u64::from(count).saturating_add(1)),
            );
        }
        inputs = inputs.saturating_add(
            u64::try_from(settings.program.len())
                .expect("program length")
                .saturating_mul(u64::from(count).saturating_add(1)),
        );
        let mut io = Io::new(&bounds);
        let signals = io.adopt_signals(settings.signals).expect("three adopted slots");
        let stdout = io.adopt_write_pipe(settings.stdout, None).expect("stdout slot");
        let stderr = io.adopt_write_pipe(settings.stderr, None).expect("stderr slot");
        let capacity = io::operations(&bounds).expect("bounded operations").saturating_mul(4).max(64);
        let entries = count.saturating_mul(2).saturating_add(1);
        // Each queued record can own a launch or a chunk. Count even the impossible
        // case where every slot holds the largest launch and the largest chunk.
        let payload = inputs
            .saturating_add(u64::from(bounds.output))
            .saturating_add(u64::try_from(size_of::<[kernel::Pipe; 2]>()).expect("pipe specs"));
        let queues = Queue::<Complete>::worst_case(capacity)
            .expect("completion queue")
            .saturating_add(Queue::<Submit>::worst_case(capacity).expect("submission queue"))
            .saturating_add(Queue::<Event>::worst_case(capacity).expect("event queue"))
            .saturating_add(Queue::<Request>::worst_case(capacity).expect("request queue"))
            .saturating_add(Queue::<Order>::worst_case(capacity).expect("order queue"));
        let memory = io::worst_case(&bounds)
            .expect("io memory")
            .saturating_add(crate::limits::worst_case(limits, count).expect("policy memory"))
            .saturating_add(inputs.saturating_mul(3))
            .saturating_add(queues)
            .saturating_add(payload.saturating_mul(u64::from(capacity).saturating_mul(4)))
            .saturating_add(List::<Child>::worst_case(count).expect("child states"))
            .saturating_add(List::<Step>::worst_case(count).expect("step states"))
            .saturating_add(u64::from(count).saturating_mul(u64::from(LINE_BYTES).saturating_mul(4)))
            .saturating_add(Sink::worst_case(entries, bounds.output).saturating_mul(2));
        Ok(Self {
            io,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: bounds },
            machine,
            steps,
            settings,
            children: children.into_boxed(),
            sinks: [Sink::new(stdout, entries), Sink::new(stderr, entries)],
            signals,
            phase: Phase::New,
            failed: false,
            serial: 1_u64 << 63,
            root: Root::Open,
            completions: Queue::with_capacity(capacity),
            submissions: Queue::with_capacity(capacity),
            told: Queue::with_capacity(capacity),
            requests: Queue::with_capacity(capacity),
            orders: Queue::with_capacity(capacity),
            memory,
        })
    }
    fn room(&self, bound: io::MaxOut) -> bool {
        self.told.room() >= bound.events && self.submissions.room() >= bound.submissions
    }
    fn input(&mut self, input: Input) {
        self.machine.input(self.env.now, input, &mut self.orders);
    }
    fn event(&mut self, event: Event) {
        match event {
            Event::Shutdown { .. } => self.input(Input::Terminate),
            Event::Spawned { owner, child, pipes } => {
                for (step, state) in self.children.iter_mut().enumerate() {
                    if state.owner == Some(owner) {
                        state.process = Some(child);
                        for (pipe, token) in state.pipes.iter_mut().zip(&pipes) {
                            pipe.token = Some(*token);
                        }
                        self.machine.input(self.env.now, Input::Started { step }, &mut self.orders);
                        break;
                    }
                }
            }
            Event::Exited { owner, exit } => {
                for (step, state) in self.children.iter().enumerate() {
                    if state.owner == Some(owner) {
                        self.machine.input(self.env.now, Input::Exited { step, exit }, &mut self.orders);
                        break;
                    }
                }
            }
            Event::Closed { owner } => {
                for sink in &mut self.sinks {
                    if sink.token == owner {
                        sink.closed();
                    }
                }
                for (step, state) in self.children.iter_mut().enumerate() {
                    if state.owner == Some(owner) {
                        let input = if state.process.take().is_some() {
                            Input::Closed { step }
                        } else {
                            Input::NotStarted { step }
                        };
                        state.owner = None;
                        self.machine.input(self.env.now, input, &mut self.orders);
                    }
                    for pipe in &mut state.pipes {
                        if pipe.token == Some(owner) {
                            pipe.token = None;
                            pipe.reading = false;
                            pipe.lines.end();
                        }
                    }
                }
            }
            Event::Stream { owner, up } => self.stream(owner, up),
            Event::Failed { owner, .. } => {
                if owner == self.signals {
                    self.failed = true;
                    self.input(Input::Terminate);
                }
            }
            Event::Listening { .. }
            | Event::Accepted { .. }
            | Event::Connecting { .. }
            | Event::Connected { .. }
            | Event::Output { .. }
            | Event::Usage { .. }
            | Event::Resized
            | Event::Window { .. } => {}
        }
    }
    fn stream(&mut self, owner: Token, up: Up) {
        for sink in &mut self.sinks {
            if sink.token == owner {
                if sink.up(up, &mut self.requests) {
                    self.failed = true;
                    self.input(Input::Terminate);
                }
                return;
            }
        }
        for state in &mut self.children {
            for pipe in &mut state.pipes {
                if pipe.token == Some(owner) {
                    pipe.reading = false;
                    match up {
                        Up::Bytes(bytes) => pipe.lines.push(bytes),
                        Up::End | Up::Failed(_) => {
                            pipe.lines.end();
                            self.requests.push(Request::Close { entity: owner });
                        }
                        Up::Room => {}
                    }
                    return;
                }
            }
        }
    }
    fn order(&mut self, order: Order) {
        match order {
            Order::Spawn { step } => self.children.get_mut(step).expect("machine step").spawn = true,
            Order::Signal { step } => {
                if let Some(child) = self.children.get(step).expect("machine step").process {
                    self.requests.push(Request::Signal {
                        child,
                        signal: kernel::Signal::Terminate,
                        to: kernel::Target::Child,
                    });
                }
            }
            Order::Close { step } => {
                if let Some(child) = self.children.get(step).expect("machine step").process {
                    self.requests.push(Request::Close { entity: child });
                }
            }
        }
    }
    fn children(&mut self) {
        for (step, state) in self.children.iter_mut().enumerate() {
            if self.requests.room() < 6 {
                break;
            }
            let description = self.steps.get(step).expect("step state");
            for (fd, pipe) in state.pipes.iter_mut().enumerate() {
                let plain = fd == 0 && lines::route(&self.settings.plain, description.name(), 1);
                let sink = self.sinks.get_mut(usize::from(!plain)).expect("two sinks");
                if sink.takes() {
                    if let Some(line) = pipe.lines.take(description.name()) {
                        sink.push(line).expect("checked sink room");
                    }
                } else if !sink.room() {
                    // Broken sinks discard bounded pending pieces during teardown.
                    drop(pipe.lines.take(description.name()));
                }
                if pipe.lines.wants_read()
                    && !pipe.reading
                    && sink.takes()
                    && let Some(stream) = pipe.token
                {
                    self.requests.push(Request::Stream {
                        stream,
                        down: Down::Demand { read: Read::Arrived { max: LINE_BYTES }, room: 0 },
                    });
                    pipe.reading = true;
                }
            }
            let mut settled = true;
            for pipe in &state.pipes {
                settled &= pipe.settled();
            }
            if state.spawn && settled {
                state.spawn = false;
                self.serial = self.serial.checked_add(1).expect("spawn serial");
                let owner = Token::new(self.serial);
                state.owner = Some(owner);
                for (fd, pipe) in state.pipes.iter_mut().enumerate() {
                    *pipe = Pipe::new(fd == 0 && lines::route(&self.settings.plain, description.name(), 1));
                }
                self.requests.push(Request::Spawn {
                    owner,
                    spawn: kernel::Spawn {
                        program: self.settings.program.clone(),
                        args: Box::from(description.arguments()),
                        env: self.settings.environment.clone(),
                        root: self.settings.root,
                        dir: Box::new([]),
                        pipes: Box::new([
                            kernel::Pipe { child: 1, way: kernel::Way::Out, parent: None },
                            kernel::Pipe { child: 2, way: kernel::Way::Out, parent: None },
                        ]),
                    },
                });
            }
        }
    }
    fn finish(&mut self) {
        if !self.machine.is_idle() {
            return;
        }
        for state in &self.children {
            for pipe in &state.pipes {
                if !pipe.settled() {
                    return;
                }
            }
        }
        if self.phase != Phase::Closing {
            if self.requests.room() == 0 {
                return;
            }
            match self.machine.outcome() {
                Some(Outcome::GaveUp { step, restarts }) => {
                    let sink = self.sinks.get_mut(1).expect("stderr");
                    if sink.room() && !sink.takes() {
                        return;
                    }
                    if sink.takes() {
                        let digits = Decimal::of(u64::from(restarts));
                        sink.push(lines::joined(&[
                            b"box: ",
                            self.steps.get(step).expect("failed step").name(),
                            b" gave up after ",
                            digits.as_bytes(),
                            b" restarts\n",
                        ]))
                        .expect("sink room");
                    }
                }
                Some(Outcome::Stopped) | None => {}
            }

            self.requests.push(Request::Close { entity: self.signals });
            self.phase = Phase::Closing;
        }
        if self.root == Root::Open && self.submissions.room() > 0 {
            self.submissions.push(Submit { op: ROOT_CLOSE, kind: kernel::Op::Close { fd: self.settings.root } });
            self.root = Root::Closing;
        }
    }
}
/// Three entities per step, and the three inherited streams; names extend output at construction.
#[must_use]
pub fn io_limits(steps: u32) -> io::Limits {
    io::Limits {
        sockets: steps.checked_mul(3).expect("bounded step count").checked_add(3).expect("inherited streams"),
        names: 0,
        refusals: steps.max(1),
        intake: LINE_BYTES,
        receive: 1,
        output: LINE_BYTES.saturating_add(3),
        sends: 1,
        accepts: 1,
        backlog: 1,
        close_timeout: Duration::from_secs(5),
        retry: Duration::from_millis(10),
    }
}
impl skein_shell::Host for Service {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        for _ in 0..self.env.limits.sockets.saturating_add(self.env.limits.refusals) {
            if !self.io.is_ready() || !self.room(io::MAX_OUT_RESUME) {
                break;
            }
            io::resume(&mut self.io, &self.env, &mut self.told, &mut self.submissions);
        }
        if !self.io.is_ready() {
            for _ in 0..self.completions.len() {
                if !self.room(io::MAX_OUT_UP) {
                    break;
                }
                let complete = self.completions.pop().expect("counted completion");
                if complete.op == ROOT_CLOSE {
                    self.root = Root::Closed;
                } else {
                    io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.submissions);
                }
            }
            if self.completions.is_empty() {
                for _ in 0..self.env.limits.sockets.saturating_mul(2) {
                    if !self.io.is_due(now) || !self.room(io::MAX_OUT_FIRE) {
                        break;
                    }
                    io::fire(&mut self.io, &self.env, &mut self.told, &mut self.submissions);
                }
            }
        }
        if self.phase == Phase::New && self.orders.room() >= self.machine.max_out() {
            self.machine.start(now, &mut self.orders);
            self.phase = Phase::Running;
        }
        for _ in 0..self.told.len() {
            if self.orders.room() < self.machine.max_out() || self.requests.room() < 4 {
                break;
            }
            let event = self.told.pop().expect("counted event");
            self.event(event);
        }
        if self.told.is_empty() && self.orders.room() >= self.machine.max_out() {
            self.machine.fire(now, &mut self.orders);
        }
        for _ in 0..self.orders.len() {
            if self.requests.room() == 0 {
                break;
            }
            let order = self.orders.pop().expect("counted order");
            self.order(order);
        }
        self.children();
        self.finish();
        for sink in &mut self.sinks {
            if self.requests.room() > 0 {
                sink.tick(self.phase == Phase::Closing, &mut self.requests);
            }
        }
        for _ in 0..self.requests.len() {
            if !self.io.takes() || self.submissions.room() < io::MAX_OUT_DOWN.submissions {
                break;
            }
            let request = self.requests.pop().expect("counted request");
            io::down(&mut self.io, &self.env, request, &mut self.submissions);
        }
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        if self.phase == Phase::New
            || !self.completions.is_empty()
            || !self.told.is_empty()
            || !self.requests.is_empty()
            || !self.orders.is_empty()
            || self.io.is_ready()
            || self.io.is_due(now)
        {
            return true;
        }
        if let Some(at) = self.machine.next_deadline()
            && at <= now
        {
            return true;
        }
        for sink in &self.sinks {
            if sink.pending() {
                return true;
            }
        }
        for (step, state) in self.children.iter().enumerate() {
            let description = self.steps.get(step).expect("step state");
            for (fd, pipe) in state.pipes.iter().enumerate() {
                let plain = fd == 0 && lines::route(&self.settings.plain, description.name(), 1);
                let sink = self.sinks.get(usize::from(!plain)).expect("two sinks");
                if (pipe.lines.ready() && (sink.takes() || !sink.room()))
                    || (pipe.lines.wants_read() && !pipe.reading && pipe.token.is_some() && sink.takes())
                {
                    return true;
                }
            }
            let mut settled = true;
            for pipe in &state.pipes {
                settled &= pipe.settled();
            }
            if state.spawn && settled {
                return true;
            }
        }
        false
    }
    fn next_deadline(&self) -> Option<Time> {
        match (self.io.next_deadline(), self.machine.next_deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, None) | (None, a) => a,
        }
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.machine.next_deadline()
    }
    fn is_empty(&self) -> bool {
        self.phase == Phase::Closing
            && self.root == Root::Closed
            && self.machine.is_idle()
            && self.io.is_empty()
            && self.requests.is_empty()
            && self.orders.is_empty()
            && self.told.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
    }
    fn exit(&self) -> Option<Exit> {
        if !self.is_empty() {
            return None;
        }
        Some(Exit::Code(u8::from(self.failed || self.machine.outcome() != Some(Outcome::Stopped))))
    }
    fn worst_case(&self) -> u64 {
        self.memory
    }
    fn operations(&self) -> u32 {
        io::operations(&self.env.limits).expect("io operations").saturating_add(1)
    }
}
