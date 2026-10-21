//! Steps, exits and timers in; process orders out (protocol/setup.md, 7.2).
use crate::layout::Placed;
use crate::limits::{Limits, Unfit, check};
use alloc::boxed::Box;
use skein_io::kernel::Exit;
use skein_lib::{List, Queue, Time};
use temper_forge_setup::Kind;
/// Inputs referring only to a step's index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Input {
    Started { step: usize },
    NotStarted { step: usize },
    Exited { step: usize, exit: Exit },
    Closed { step: usize },
    Terminate,
    Ask { forge: u16 },
    Written { forge: u16 },
    NotWritten { forge: u16 },
}
/// Effects requested of the host.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    Spawn { step: usize },
    Signal { step: usize },
    Close { step: usize },
    Record { forge: u16 },
    Say { said: Said },
}
/// Pipe replies name a placed step; formatting belongs to the host.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Said {
    Step { step: usize },
    Started { forge: u16 },
    Failed { forge: u16, why: Why },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    Busy,
    NotAsked,
    Stopping,
    Exited { step: usize, exit: Exit },
    NotStarted { step: usize },
    NotWritten,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Dormant,
    Starting,
    Recording { next: Option<usize> },
    Started,
    Failing { why: Why },
}
#[derive(Debug)]
pub(crate) struct GroupState {
    forge: u16,
    first: usize,
    end: usize,
    chained: bool,
    status: Status,
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
    group: Option<usize>,
    phase: Phase,
    exits: Queue<Time>,
}
/// The box's policy, independent of process handles and step names.
#[derive(Debug)]
pub struct Machine {
    states: Box<[State]>,
    groups: Box<[GroupState]>,
    limits: Limits,
    outcome: Option<Outcome>,
    forced: bool,
    begun: bool,
}
impl Machine {
    /// Checks limits and allocates each step's bounded exit history.
    pub fn new(limits: &Limits, steps: &[Placed]) -> Result<Self, Unfit> {
        check(limits)?;
        let mut states = List::with_capacity(u32::try_from(steps.len()).expect("steps fit u32"));
        let mut groups = List::with_capacity(u32::try_from(steps.len()).expect("bounded groups"));
        for (index, step) in steps.iter().enumerate() {
            let group = match step.asked {
                Some(asked) => {
                    let last = groups.len().checked_sub(1);
                    let mut found = None;
                    if let Some(last) = last {
                        let previous: &mut GroupState = groups.get_mut(last).expect("last group");
                        if previous.forge == asked.forge && previous.end == index {
                            previous.end = index.saturating_add(1);
                            found = Some(usize::try_from(last).expect("group index"));
                        }
                    }
                    match found {
                        Some(index) => Some(index),
                        None => {
                            let group = usize::try_from(groups.len()).expect("group index");
                            groups
                                .push(GroupState {
                                    forge: asked.forge,
                                    first: index,
                                    end: index.saturating_add(1),
                                    chained: false,
                                    status: Status::Dormant,
                                })
                                .expect("bounded groups");
                            Some(group)
                        }
                    }
                }
                None => None,
            };
            states
                .push(State {
                    long: step.step.kind() == Kind::Long,
                    group,
                    phase: Phase::Dormant,
                    exits: Queue::with_capacity(limits.restarts.saturating_add(1)),
                })
                .expect("one state per step");
        }
        Ok(Self {
            states: states.into_boxed(),
            groups: groups.into_boxed(),
            limits: *limits,
            outcome: None,
            forced: false,
            begun: false,
        })
    }
    /// Reserves spawns, closes, records and replies for every entry point.
    #[must_use]
    pub fn max_out(&self) -> u32 {
        u32::try_from(self.states.len()).expect("steps fit u32").saturating_mul(3).saturating_add(4)
    }
    /// Starts the first step once.
    pub fn start(&mut self, _now: Time, recorded: &[u16], orders: &mut Queue<Order>) {
        if !self.begun && self.outcome.is_none() {
            self.begun = true;
            for group in &mut self.groups {
                if recorded.contains(&group.forge) {
                    group.chained = true;
                    group.status = Status::Started;
                }
            }
            for first in 0..self.states.len() {
                if self.chain(first).is_none() {
                    self.spawn(first, self.successor(first), orders);
                    break;
                }
            }
        }
    }
    fn chain(&self, step: usize) -> Option<usize> {
        let group = self.states.get(step).expect("step index").group?;
        if self.groups.get(group).expect("group index").chained { None } else { Some(group) }
    }
    #[expect(clippy::manual_find, reason = "the programming model uses explicit loops rather than closures")]
    fn successor(&self, step: usize) -> Option<usize> {
        for next in step.saturating_add(1)..self.states.len() {
            if self.chain(step) == self.chain(next) {
                return Some(next);
            }
        }
        None
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
            if let Some(group) = state.group
                && self.groups.get(group).expect("group index").status == Status::Starting
            {
                orders.push(Order::Say { said: Said::Step { step } });
            }
        }
    }
    /// Takes one lifecycle or termination input.
    pub fn input(&mut self, now: Time, input: Input, orders: &mut Queue<Order>) {
        match input {
            Input::Terminate => self.stop(now, orders),
            Input::Ask { forge } => self.ask(forge, orders),
            Input::Written { forge } => self.written(forge, orders),
            Input::NotWritten { forge } => {
                if self.outcome.is_none()
                    && let Some(group) = self.group(forge)
                    && match self.groups.get(group).expect("group").status {
                        Status::Recording { .. } => true,
                        Status::Dormant | Status::Starting | Status::Started | Status::Failing { .. } => false,
                    }
                {
                    self.fail(group, Why::NotWritten, orders);
                }
            }
            Input::Started { step } => self.started(now, step, orders),
            Input::NotStarted { step } => {
                if self.outcome.is_none()
                    && let Some(group) = self.starting_group(step)
                {
                    self.states.get_mut(step).expect("step").phase = Phase::Stopped;
                    self.fail(group, Why::NotStarted { step }, orders);
                    self.failed_if_closed(group, orders);
                    return;
                }
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
            Input::Exited { step, exit } => {
                if self.outcome.is_none()
                    && let Some(group) = self.starting_group(step)
                    && (self.states.get(step).expect("step").long || exit != Exit::Code(0))
                {
                    self.fail(group, Why::Exited { step, exit }, orders);
                } else {
                    self.exited(step, exit == Exit::Code(0), orders);
                }
            }
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
                } else if let Some(group) = self.starting_group(step) {
                    match self.groups.get(group).expect("group").status {
                        Status::Failing { .. } => {
                            self.states.get_mut(step).expect("step").phase =
                                Phase::Closing { next: None, success: false };
                            orders.push(Order::Close { step });
                        }
                        Status::Starting => {
                            if self.states.get(step).expect("step").long {
                                self.states.get_mut(step).expect("step").phase = Phase::Live { next: None };
                                self.advance_group(group, next, orders);
                            }
                        }
                        Status::Dormant | Status::Recording { .. } | Status::Started => {}
                    }
                } else if self.states.get(step).expect("step").long {
                    self.states.get_mut(step).expect("step").phase = Phase::Live { next: None };
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
        let group = state.group;
        let phase = core::mem::replace(&mut state.phase, Phase::Stopped);
        match phase {
            Phase::Closing { next, success } => {
                if self.outcome.is_some() {
                    self.stop_next(now, orders);
                } else if let Some(group) = group
                    && self.group_starting(group)
                {
                    match self.groups.get(group).expect("group").status {
                        Status::Failing { .. } => self.failed_if_closed(group, orders),
                        Status::Starting => self.advance_group(group, next, orders),
                        Status::Dormant | Status::Recording { .. } | Status::Started => {}
                    }
                } else if self.states.get(step).expect("step").long {
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
    fn group(&self, forge: u16) -> Option<usize> {
        for (index, group) in self.groups.iter().enumerate() {
            if group.forge == forge {
                return Some(index);
            }
        }
        None
    }
    fn group_starting(&self, group: usize) -> bool {
        match self.groups.get(group).expect("group").status {
            Status::Starting | Status::Recording { .. } | Status::Failing { .. } => true,
            Status::Dormant | Status::Started => false,
        }
    }
    fn starting_group(&self, step: usize) -> Option<usize> {
        let group = self.states.get(step)?.group?;
        if self.group_starting(group) { Some(group) } else { None }
    }
    fn ask(&mut self, forge: u16, orders: &mut Queue<Order>) {
        let Some(index) = self.group(forge) else {
            orders.push(Order::Say { said: Said::Failed { forge, why: Why::NotAsked } });
            return;
        };
        if self.groups.get(index).expect("group").status == Status::Started {
            orders.push(Order::Say { said: Said::Started { forge } });
            return;
        }
        if self.outcome.is_some() {
            orders.push(Order::Say { said: Said::Failed { forge, why: Why::Stopping } });
            return;
        }
        if self.group_starting(index) {
            return;
        }
        for group in 0..self.groups.len() {
            if self.group_starting(group) {
                orders.push(Order::Say { said: Said::Failed { forge, why: Why::Busy } });
                return;
            }
        }
        let group = self.groups.get_mut(index).expect("group");
        group.status = Status::Starting;
        let first = group.first;
        self.advance_group(index, Some(first), orders);
    }
    fn advance_group(&mut self, index: usize, next: Option<usize>, orders: &mut Queue<Order>) {
        // The first long step is the write barrier; once written, a live long
        // predecessor proves the barrier has already passed.
        let group = self.groups.get(index).expect("group");
        let mut written = false;
        for step in group.first..group.end {
            let state = self.states.get(step).expect("group step");
            if state.long
                && match state.phase {
                    Phase::Live { .. } => true,
                    Phase::Dormant
                    | Phase::Waiting { .. }
                    | Phase::Starting { .. }
                    | Phase::Closing { .. }
                    | Phase::Backoff { .. }
                    | Phase::Grace { .. }
                    | Phase::Stopped => false,
                }
            {
                written = true;
            }
        }
        let needs_record = match next {
            Some(step) => self.states.get(step).expect("step").long && !written,
            None => !written,
        };
        if needs_record {
            let group = self.groups.get_mut(index).expect("group");
            group.status = Status::Recording { next };
            orders.push(Order::Record { forge: group.forge });
        } else if let Some(step) = next {
            self.spawn(step, self.successor(step), orders);
        } else {
            self.group_started(index, orders);
        }
    }
    fn group_started(&mut self, index: usize, orders: &mut Queue<Order>) {
        let group = self.groups.get_mut(index).expect("group");
        group.status = Status::Started;
        orders.push(Order::Say { said: Said::Started { forge: group.forge } });
    }
    fn written(&mut self, forge: u16, orders: &mut Queue<Order>) {
        if self.outcome.is_some() {
            return;
        }
        let Some(index) = self.group(forge) else {
            return;
        };
        let group = self.groups.get_mut(index).expect("group");
        match group.status {
            Status::Recording { next } => {
                group.status = Status::Starting;
                if let Some(step) = next {
                    self.spawn(step, self.successor(step), orders);
                } else {
                    self.group_started(index, orders);
                }
            }
            Status::Dormant | Status::Starting | Status::Started | Status::Failing { .. } => {}
        }
    }
    fn fail(&mut self, index: usize, why: Why, orders: &mut Queue<Order>) {
        let group = self.groups.get_mut(index).expect("group");
        if match group.status {
            Status::Failing { .. } => true,
            Status::Dormant | Status::Starting | Status::Recording { .. } | Status::Started => false,
        } {
            return;
        }
        group.status = Status::Failing { why };
        for step in group.first..group.end {
            let state = self.states.get_mut(step).expect("group step");
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
        self.failed_if_closed(index, orders);
    }
    fn failed_if_closed(&mut self, index: usize, orders: &mut Queue<Order>) {
        let group = self.groups.get_mut(index).expect("group");
        let why = match group.status {
            Status::Failing { why } => why,
            Status::Dormant | Status::Starting | Status::Recording { .. } | Status::Started => return,
        };
        for step in group.first..group.end {
            match self.states.get(step).expect("group step").phase {
                Phase::Dormant | Phase::Stopped => {}
                Phase::Waiting { .. }
                | Phase::Starting { .. }
                | Phase::Live { .. }
                | Phase::Closing { .. }
                | Phase::Backoff { .. }
                | Phase::Grace { .. } => return,
            }
        }
        for step in group.first..group.end {
            let state = self.states.get_mut(step).expect("group step");
            state.phase = Phase::Dormant;
            for _ in 0..state.exits.len() {
                state.exits.pop();
            }
        }
        group.status = Status::Dormant;
        orders.push(Order::Say { said: Said::Failed { forge: group.forge, why } });
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
                        if self.chain(before) != self.chain(step) {
                            continue;
                        }
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
