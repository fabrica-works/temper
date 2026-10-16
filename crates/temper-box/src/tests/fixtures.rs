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
    Step::new(bytes::copy_of(name), Box::new([]), long)
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
        let machine = Machine::new(limits, &owned_steps()).expect("valid limits");
        let out = Queue::with_capacity(machine.max_out().saturating_mul(4));
        let mut rig = Self { machine, out, now: Time::ZERO };
        rig.machine.start(rig.now, &mut rig.out);
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
