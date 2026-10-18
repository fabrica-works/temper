//! Steps, exits and timers in; process orders out (protocol/setup.md, 7.2).
use crate::limits::{Limits, Unfit, check};
use alloc::boxed::Box;
use skein_io::kernel::Exit;
use skein_lib::{List, Queue, Time};
use temper_forge_setup::Step;
/// Inputs referring only to a step's index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Input {
    Started { step: usize },
    NotStarted { step: usize },
    Exited { step: usize, exit: Exit },
    Closed { step: usize },
    Terminate,
}
/// Effects requested of the host.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    Spawn { step: usize },
    Signal { step: usize },
    Close { step: usize },
}
/// The result once all children are closed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Stopped,
    GaveUp { step: usize, restarts: u32 },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Dormant,
    Waiting { next: Option<usize> },
    Starting { next: Option<usize> },
    Live { next: Option<usize> },
    Closing { next: Option<usize>, success: bool },
    Backoff { at: Time, next: Option<usize> },
    Grace { at: Time, second: bool },
    Stopped,
}
#[derive(Debug)]
pub(crate) struct State {
    long: bool,
    phase: Phase,
    exits: Queue<Time>,
}
/// The box's policy, independent of process handles and step names.
#[derive(Debug)]
pub struct Machine {
    states: Box<[State]>,
    limits: Limits,
    outcome: Option<Outcome>,
    forced: bool,
    begun: bool,
}
impl Machine {
    /// Checks limits and allocates each step's bounded exit history.
    pub fn new(limits: &Limits, steps: &[Step]) -> Result<Self, Unfit> {
        check(limits)?;
        let mut states = List::with_capacity(u32::try_from(steps.len()).expect("steps fit u32"));
        for step in steps {
            states
                .push(State {
                    long: step.is_long(),
                    phase: Phase::Dormant,
                    exits: Queue::with_capacity(limits.restarts.saturating_add(1)),
                })
                .expect("one state per step");
        }
        Ok(Self { states: states.into_boxed(), limits: *limits, outcome: None, forced: false, begun: false })
    }
    /// One order per step is enough for every entry point.
    #[must_use]
    pub fn max_out(&self) -> u32 {
        u32::try_from(self.states.len()).expect("steps fit u32").max(1)
    }
    /// Starts the first step once.
    pub fn start(&mut self, _now: Time, orders: &mut Queue<Order>) {
        if !self.begun && self.outcome.is_none() {
            self.begun = true;
            self.spawn(0, self.successor(0), orders);
        }
    }
    fn successor(&self, step: usize) -> Option<usize> {
        let next = step.checked_add(1)?;
        if next < self.states.len() { Some(next) } else { None }
    }
    fn continuation(&self, step: usize) -> Option<usize> {
        match self.states.get(step)?.phase {
            Phase::Waiting { next } => next,
            Phase::Dormant
            | Phase::Starting { .. }
            | Phase::Live { .. }
            | Phase::Closing { .. }
            | Phase::Backoff { .. }
            | Phase::Grace { .. }
            | Phase::Stopped => self.successor(step),
        }
    }
    fn spawn(&mut self, step: usize, next: Option<usize>, orders: &mut Queue<Order>) {
        if let Some(state) = self.states.get_mut(step) {
            state.phase = Phase::Starting { next };
            orders.push(Order::Spawn { step });
        }
    }
    /// Takes one lifecycle or termination input.
    pub fn input(&mut self, now: Time, input: Input, orders: &mut Queue<Order>) {
        match input {
            Input::Terminate => self.stop(now, orders),
            Input::Started { step } => self.started(now, step, orders),
            Input::NotStarted { step } => {
                if let Some(state) = self.states.get_mut(step) {
                    let next = match state.phase {
                        Phase::Starting { next } => next,
                        Phase::Dormant
                        | Phase::Waiting { .. }
                        | Phase::Live { .. }
                        | Phase::Closing { .. }
                        | Phase::Backoff { .. }
                        | Phase::Grace { .. }
                        | Phase::Stopped => return,
                    };
                    state.phase = Phase::Closing { next, success: false };
                }
                self.closed(now, step, orders);
            }
            Input::Exited { step, exit } => self.exited(step, exit == Exit::Code(0), orders),
            Input::Closed { step } => self.closed(now, step, orders),
        }
    }
    fn started(&mut self, now: Time, step: usize, orders: &mut Queue<Order>) {
        let Some(state) = self.states.get_mut(step) else {
            return;
        };
        match state.phase {
            Phase::Starting { next } => {
                state.phase = Phase::Live { next };
                if self.outcome.is_some() {
                    if self.forced {
                        state.phase = Phase::Closing { next: None, success: false };
                        orders.push(Order::Close { step });
                    } else {
                        self.stop_next(now, orders);
                    }
                } else if state.long {
                    state.phase = Phase::Live { next: None };
                    if let Some(next) = next {
                        self.spawn(next, self.continuation(next), orders);
                    }
                }
            }
            Phase::Dormant
            | Phase::Waiting { .. }
            | Phase::Live { .. }
            | Phase::Closing { .. }
            | Phase::Backoff { .. }
            | Phase::Grace { .. }
            | Phase::Stopped => {}
        }
    }
    fn exited(&mut self, step: usize, success: bool, orders: &mut Queue<Order>) {
        let Some(state) = self.states.get_mut(step) else {
            return;
        };
        match state.phase {
            Phase::Starting { next } | Phase::Live { next } => {
                state.phase = Phase::Closing { next, success };
                orders.push(Order::Close { step });
            }
            Phase::Grace { .. } => {
                state.phase = Phase::Closing { next: None, success };
                orders.push(Order::Close { step });
            }
            Phase::Dormant | Phase::Waiting { .. } | Phase::Closing { .. } | Phase::Backoff { .. } | Phase::Stopped => {
            }
        }
    }
    fn closed(&mut self, now: Time, step: usize, orders: &mut Queue<Order>) {
        let Some(state) = self.states.get_mut(step) else {
            return;
        };
        let phase = core::mem::replace(&mut state.phase, Phase::Stopped);
        match phase {
            Phase::Closing { next, success } => {
                if self.outcome.is_some() {
                    self.stop_next(now, orders);
                } else if state.long {
                    self.restart(now, step, next, orders);
                } else if success {
                    if let Some(next) = next {
                        self.spawn(next, self.continuation(next), orders);
                    }
                } else {
                    self.give_up(now, step, 0, orders);
                }
            }
            Phase::Dormant
            | Phase::Waiting { .. }
            | Phase::Starting { .. }
            | Phase::Live { .. }
            | Phase::Backoff { .. }
            | Phase::Grace { .. }
            | Phase::Stopped => {
                state.phase = phase;
            }
        }
    }
    fn restart(&mut self, now: Time, step: usize, next: Option<usize>, orders: &mut Queue<Order>) {
        let state = self.states.get_mut(step).expect("known step");
        let count = state.exits.len();
        for _ in 0..count {
            let at = state.exits.pop().expect("counted exit");
            if now.saturating_since(at) <= self.limits.window {
                state.exits.push(at);
            }
        }
        state.exits.push(now);
        let count = state.exits.len();
        if count > self.limits.restarts {
            self.give_up(now, step, self.limits.restarts, orders);
        } else {
            let mut delay = self.limits.backoff;
            for _ in 1..count {
                delay = delay.saturating_mul(2).min(self.limits.backoff_max);
            }
            self.states.get_mut(step).expect("known step").phase =
                Phase::Backoff { at: now.saturating_add(delay), next };
        }
    }
    fn give_up(&mut self, now: Time, step: usize, restarts: u32, orders: &mut Queue<Order>) {
        self.outcome = Some(Outcome::GaveUp { step, restarts });
        self.disarm();
        self.stop_next(now, orders);
    }
    fn disarm(&mut self) {
        for state in &mut self.states {
            match state.phase {
                Phase::Backoff { .. } => state.phase = Phase::Stopped,
                Phase::Dormant
                | Phase::Waiting { .. }
                | Phase::Starting { .. }
                | Phase::Live { .. }
                | Phase::Closing { .. }
                | Phase::Grace { .. }
                | Phase::Stopped => {}
            }
        }
    }
    fn stop(&mut self, now: Time, orders: &mut Queue<Order>) {
        if self.outcome.is_some() {
            self.forced = true;
            for (step, state) in self.states.iter_mut().enumerate() {
                match state.phase {
                    Phase::Live { .. } | Phase::Grace { .. } => {
                        state.phase = Phase::Closing { next: None, success: false };
                        orders.push(Order::Close { step });
                    }
                    Phase::Dormant
                    | Phase::Waiting { .. }
                    | Phase::Starting { .. }
                    | Phase::Closing { .. }
                    | Phase::Backoff { .. }
                    | Phase::Stopped => {}
                }
            }
        } else {
            self.outcome = Some(Outcome::Stopped);
            self.disarm();
            self.stop_next(now, orders);
        }
    }
    fn stop_next(&mut self, now: Time, orders: &mut Queue<Order>) {
        for (step, state) in self.states.iter_mut().enumerate().rev() {
            match state.phase {
                Phase::Live { .. } => {
                    state.phase = Phase::Grace { at: now.saturating_add(self.limits.grace), second: false };
                    orders.push(Order::Signal { step });
                    return;
                }
                Phase::Starting { .. } | Phase::Closing { .. } | Phase::Grace { .. } => return,
                Phase::Dormant | Phase::Waiting { .. } | Phase::Backoff { .. } | Phase::Stopped => {}
            }
        }
    }
    /// Fires all expired waits; the caller reserves `max_out` orders.
    pub fn fire(&mut self, now: Time, orders: &mut Queue<Order>) {
        for step in 0..self.states.len() {
            let phase = self.states.get(step).expect("bounded step").phase;
            match phase {
                Phase::Backoff { at, next: following } if at <= now => {
                    let mut first = step;
                    for before in (0..step).rev() {
                        if self.states.get(before).expect("bounded step").long {
                            break;
                        }
                        first = before;
                    }
                    self.states.get_mut(step).expect("bounded step").phase = Phase::Waiting { next: following };
                    let next = if first == step { following } else { self.successor(first) };
                    self.spawn(first, next, orders);
                }
                Phase::Grace { at, second } if at <= now => {
                    let state = self.states.get_mut(step).expect("bounded step");
                    if second {
                        state.phase = Phase::Closing { next: None, success: false };
                        orders.push(Order::Close { step });
                    } else {
                        state.phase = Phase::Grace { at: now.saturating_add(self.limits.grace), second: true };
                        orders.push(Order::Signal { step });
                    }
                }
                Phase::Dormant
                | Phase::Waiting { .. }
                | Phase::Starting { .. }
                | Phase::Live { .. }
                | Phase::Closing { .. }
                | Phase::Backoff { .. }
                | Phase::Grace { .. }
                | Phase::Stopped => {}
            }
        }
    }
    /// Earliest backoff or grace still armed.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for state in &self.states {
            match state.phase {
                Phase::Backoff { at, .. } | Phase::Grace { at, .. } => next = Some(next.unwrap_or(at).min(at)),
                Phase::Dormant
                | Phase::Waiting { .. }
                | Phase::Starting { .. }
                | Phase::Live { .. }
                | Phase::Closing { .. }
                | Phase::Stopped => {}
            }
        }
        next
    }
    /// The chosen outcome; meaningful to the host once idle.
    #[must_use]
    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }
    /// No starting or live child remains.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        for state in &self.states {
            match state.phase {
                Phase::Dormant | Phase::Waiting { .. } | Phase::Stopped => {}
                Phase::Starting { .. }
                | Phase::Live { .. }
                | Phase::Closing { .. }
                | Phase::Backoff { .. }
                | Phase::Grace { .. } => return false,
            }
        }
        true
    }
}
