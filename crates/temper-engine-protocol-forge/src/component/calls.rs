//! Calls share the component's connections and entry-point output reservations.
use super::{Body, Component, Heard, Phase, Plan, Start, Step, Waiting, api};
use crate::{
    boundary::{Answered, Event},
    limits::Limits,
};
use skein_lib::{Env, Id, List, Queue, Token};
impl Component {
    pub(super) fn expire(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) {
        let waiting = self.waiting.get(id).expect("indexed call");
        let error = if waiting.cost == 0 { api::Error::Unavailable } else { api::Error::Timeout };
        let error = match &waiting.plan {
            Some(plan) => plan.error(error),
            None => error,
        };
        if let Some(token) = waiting.connection {
            self.connection_slab
                .get_mut(*self.connections.get(&token).expect("holder"))
                .expect("connection")
                .close(true, io);
        }
        self.finish(call, id, Err(error), up);
    }
    pub(super) fn finish(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        result: Result<api::Answer, api::Error>,
        up: &mut Queue<Event>,
    ) {
        let waiting = self.waiting.get(id).expect("indexed call");
        let cost = waiting.cost;
        if let Some(token) = waiting.connection {
            let _holder = self.holders.remove(&token);
        }
        self.waiting.retire(id);
        let _call = self.calls.remove(&call);
        up.push(Event::Answered(Answered { call, cost, result }));
    }
    pub(super) fn drive_calls(
        &mut self,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
        above_room: u32,
        below_room: u32,
    ) {
        let above = up.len();
        let below = io.len();
        let mut calls = List::with_capacity(self.calls.capacity());
        for (call, id) in &self.calls {
            calls.push((*call, *id)).expect("bounded calls");
        }
        for (call, id) in calls.into_boxed() {
            if up.len().saturating_sub(above) >= above_room
                || io.len().saturating_sub(below).saturating_add(1) >= below_room
            {
                self.wake = Some(env.now);
                break;
            }
            if self.closed {
                break;
            }
            let waiting = self.waiting.get(id).expect("indexed call");
            if waiting.deadline <= env.now {
                self.expire(call, id, up, io);
                continue;
            }
            let forge = waiting.repository.forge;
            let mut at = None;
            for n in 0..self.settings.forges.len() {
                if self.settings.forges.get(n).expect("forge").number == forge {
                    at = Some(n);
                    break;
                }
            }
            let Some(at) = at else {
                self.finish(call, id, Err(api::Error::Unavailable), up);
                continue;
            };
            if !super::ready(*self.starts.get(at).expect("forge start")) {
                continue;
            }
            if !self.assign(call, id, at, env, up, io) {
                continue;
            }
            self.pump_call(
                (call, id),
                at,
                env,
                up,
                io,
                (
                    above_room.saturating_sub(up.len().saturating_sub(above)),
                    below_room.saturating_sub(io.len().saturating_sub(below)),
                ),
            );
        }
        for (_, id) in &self.connections {
            if io.len().saturating_sub(below) >= below_room {
                self.wake = Some(env.now);
                break;
            }
            let c = self.connection_slab.get_mut(*id).expect("indexed connection");
            if self.closed || (c.phase == Phase::Idle && c.deadline <= Some(env.now)) || c.phase == Phase::Closed {
                c.close(c.in_flight(), io);
            }
        }
    }
    fn assign(
        &mut self,
        call: Token,
        id: Id<Waiting>,
        at: u32,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
    ) -> bool {
        let waiting = self.waiting.get(id).expect("call");
        if waiting.connection.is_some() {
            return true;
        }
        let forge = waiting.repository.forge;
        let cap = match *self.starts.get(at).expect("start") {
            Start::Ready { cap } => cap,
            Start::Checking { .. } | Start::Settings { .. } | Start::Waiting { .. } | Start::Stopped(_) => {
                return false;
            }
        };
        let mut token = None;

        let mut other = None;
        for (owner, connection) in &self.connections {
            let c = self.connection_slab.get(*connection).expect("indexed connection");
            if c.phase == Phase::Idle && !c.closing && !self.holders.contains_key(owner) {
                if c.forge == forge {
                    token = Some(*owner);
                    break;
                }
                let mut start = false;
                for start_token in &self.start_tokens {
                    if *start_token == Some(*owner) {
                        start = true;
                    }
                }
                if !start {
                    other = Some(*connection);
                }
            }
        }
        if token.is_none() && self.connections.len() < self.connections.capacity() {
            token = Some(self.open_start(at, env, io));
        }
        if token.is_none() {
            if let Some(other) = other {
                self.connection_slab.get_mut(other).expect("idle other forge").close(false, io);
            }
            return false;
        }
        let token = token.expect("selected connection");
        *self.start_tokens.get_mut(at).expect("forge slot") = None;
        let _old = self.holders.insert(token, call).expect("one holder per connection");
        let waiting = self.waiting.get_mut(id).expect("indexed call");
        waiting.connection = Some(token);
        if waiting.plan.is_some() {
            return true;
        }
        let op = waiting.op.take().expect("waiting operation");
        let mut plan = Plan::new(waiting.repository, op, cap);
        let step = plan.begin(&env.limits, &self.shared, &mut self.tables);
        waiting.plan = Some(plan);
        match step {
            Ok(step) => {
                if self.call_step(id, step, env, up, io, call) {
                    return false;
                }
            }
            Err(error) => {
                self.finish(call, id, Err(error), up);
                return false;
            }
        }
        true
    }
    fn pump_call(
        &mut self,
        holder: (Token, Id<Waiting>),
        at: u32,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
        budget: (u32, u32),
    ) {
        let (call, id) = holder;
        let above = up.len();
        let below = io.len();
        let token = self.waiting.get(id).expect("call").connection.expect("holder");
        let connection = *self.connections.get(&token).expect("indexed connection");
        for _ in 0..64_u32 {
            if up.len().saturating_sub(above) >= budget.0
                || io.len().saturating_sub(below).saturating_add(1) >= budget.1
            {
                self.wake = Some(env.now);
                break;
            }
            let c = self.connection_slab.get_mut(connection).expect("holding connection");
            if c.phase == Phase::Idle {
                let request =
                    self.waiting.get_mut(id).expect("indexed call").plan.as_mut().expect("call plan").pending();
                if let Some(request) = request {
                    c.deadline = Some(self.waiting.get(id).expect("call").deadline);
                    c.raw_room(self.waiting.get(id).expect("call").plan.as_ref().expect("plan").raw_room(&self.shared));
                    match c.start(env, self.settings.forges.get(at).expect("forge"), request.request, request.range) {
                        Ok(()) => {
                            c.keep_after(self.waiting.get(id).expect("call").plan.as_ref().expect("plan").after());
                        }
                        Err(_) => {
                            self.finish(call, id, Err(api::Error::Refused), up);
                            break;
                        }
                    }
                    self.waiting.get_mut(id).expect("call").charged = false;
                }
            }
            c.pump(env, io, budget.1.saturating_sub(io.len().saturating_sub(below)).saturating_sub(1));
            let waiting = self.waiting.get_mut(id).expect("indexed call");
            if c.written && !waiting.charged {
                waiting.cost = waiting.cost.checked_add(1).expect("bounded exchanges");
                waiting.charged = true;
            }
            if let Some(heard) = c.heard() {
                if crate::reads::long(&heard) {
                    self.diagnostics.long_bodies = self.diagnostics.long_bodies.saturating_add(1);
                }
                let malformed = match &heard {
                    Heard::Done(Body::Malformed(_)) => true,
                    Heard::Done(Body::Document(_) | Body::Raw { .. } | Body::Empty)
                    | Heard::Head { .. }
                    | Heard::Row(_)
                    | Heard::Failed(_) => false,
                };
                if malformed {
                    self.diagnostics.malformed = self.diagnostics.malformed.saturating_add(1);
                }
                let step =
                    waiting.plan.as_mut().expect("call plan").heard(heard, &env.limits, &self.shared, &mut self.tables);
                if waiting.plan.as_mut().expect("plan").take_cut() {
                    c.cut(env);
                }
                match step {
                    Ok(Some(step)) => {
                        if self.call_step(id, step, env, up, io, call) {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        c.close(true, io);
                        self.finish(call, id, Err(error), up);
                        break;
                    }
                }
                continue;
            }
            if c.phase == Phase::Closed {
                c.close(true, io);
            }
            if !c.ready() {
                break;
            }
            self.wake = Some(env.now);
        }
    }
    fn call_step(
        &mut self,
        id: Id<Waiting>,
        step: Step,
        env: &Env<Limits>,
        up: &mut Queue<Event>,
        io: &mut Queue<skein_io::Request>,
        call: Token,
    ) -> bool {
        match step {
            Step::Send(exchange) => {
                self.waiting.get_mut(id).expect("indexed call").plan.as_mut().expect("call plan").set_pending(exchange);
                let token = self.waiting.get(id).expect("call").connection.expect("holder");
                let c = self
                    .connection_slab
                    .get_mut(*self.connections.get(&token).expect("connection"))
                    .expect("connection");
                if c.phase == Phase::Closed {
                    c.close(false, io);
                    let _holder = self.holders.remove(&token);
                    self.waiting.get_mut(id).expect("call").connection = None;
                    self.wake = Some(env.now);
                    return true;
                }
                false
            }
            Step::Done(result) => {
                let token = self.waiting.get(id).expect("call").connection.expect("holder");
                let c = self
                    .connection_slab
                    .get_mut(*self.connections.get(&token).expect("connection"))
                    .expect("connection");
                if c.phase == Phase::Closed {
                    c.close(false, io);
                } else {
                    c.deadline = Some(env.now.saturating_add(env.limits.idle));
                }
                self.finish(call, id, result, up);
                true
            }
        }
    }
}
