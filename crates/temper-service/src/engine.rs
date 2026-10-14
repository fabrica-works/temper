//! One root input per pass, with store answers always accepted (protocol.md, 6).
use crate::{End, route, stop, store::route::Up};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Time, Wall};
use temper_engine_domain as root;

pub(crate) struct Engine {
    pub(crate) domain: root::Domain,
    pub(crate) env: Env<root::Limits>,
    pub(crate) store: crate::store::Store,
    pub(crate) accounts: Option<crate::accounts::Accounts>,
    pub(crate) workers: crate::workers::Workers,
    pub(crate) facts: u64,
    pub(crate) values: crate::values::Values,
    pub(crate) events: Queue<root::Event>,
    pub(crate) requests: Queue<root::Request>,
    pub(crate) end: Option<End>,
    pub(crate) first_periods: List<(u32, u64)>,
    pub(crate) drain: Option<stop::Drain>,
    pub(crate) blocked: bool,
    pending: Option<Box<root::Event>>,
    begin: bool,
    pub(crate) credentials: List<u32>,
    ready: bool,
    aborted: Option<()>,
    max_out: u32,
}
impl Engine {
    pub(crate) fn new(
        config: root::Config,
        limits: &root::Limits,
        store: crate::store::Store,
        accounts: Option<crate::accounts::Accounts>,
        workers: crate::workers::Workers,
        values: crate::values::Values,
        first_periods: List<(u32, u64)>,
    ) -> Self {
        let max_out = root::max_out(limits);
        let capacity = max_out.checked_mul(2).expect("root routing room");
        let mut credentials = List::with_capacity(values.identities.len());
        for (number, _value) in &values.identities {
            credentials.push(*number).expect("configured identities");
        }
        Self {
            credentials,
            domain: root::Domain::new(config, limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            store,
            accounts,
            workers,
            facts: 0,
            values,
            events: Queue::with_capacity(capacity.checked_mul(route_out()).expect("account event room")),
            requests: Queue::with_capacity(max_out),
            end: None,
            first_periods,
            drain: None,
            blocked: false,
            pending: None,
            begin: true,
            ready: true,
            aborted: None,
            max_out,
        }
    }
    pub(crate) fn room(&self) -> bool {
        self.store.room()
            && self.workers.batch_room(self.max_out)
            && self.events.room() >= self.max_out.checked_mul(route_out()).expect("translated root outputs")
            && match &self.drain {
                Some(drain) => drain.busy.room() >= self.max_out,
                None => true,
            }
            && match &self.accounts {
                Some(accounts) => accounts.batch_room(self.max_out),
                None => true,
            }
    }
    pub(crate) fn step(&mut self, event: root::Event) {
        if let Err(event) = root::try_step(&mut self.domain, &self.env, event) {
            self.pending = Some(Box::new(event));
            self.blocked = true;
        }
        self.drain_facts();
    }
    fn drain_facts(&mut self) {
        for _ in 0..root::FACTS {
            if self.domain.pop_fact().is_none() {
                break;
            }
            self.facts = self.facts.saturating_add(1);
        }
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let mut next = crate::accounts::earliest(self.store.next_deadline(), self.workers.next_deadline());
        if self.end.is_none() {
            next = crate::accounts::earliest(next, self.domain.next_deadline(&self.env));
            next = crate::accounts::earliest(next, self.values.accounts.next_expiry());
        }
        crate::accounts::earliest(
            next,
            match &self.accounts {
                Some(accounts) => accounts.next_deadline(),
                None => None,
            },
        )
    }
    pub(crate) fn work_pending(&self, now: Time) -> bool {
        let components = self.store.work_pending(now)
            || (self.store.opened || self.end.is_some()) && self.workers.work_pending(now)
            || (self.store.opened || self.end.is_some())
                && match &self.accounts {
                    Some(accounts) => accounts.work_pending(now),
                    None => false,
                };
        if self.end.is_some() || !self.store.opened {
            return components;
        }
        let due = match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        };
        components
            || self.room()
                && (self.begin
                    || self.ready
                    || !self.blocked
                        && (self.pending.is_some()
                            || !self.events.is_empty()
                            || due
                            || self.domain.ready() && !self.first_periods.is_empty()))
    }
    pub(crate) fn abort_if_ended(&mut self) {
        if let Some(End::Aborted | End::Stopped(_)) = self.end
            && self.aborted.is_none()
        {
            self.aborted = Some(());
            self.store.close();
            self.workers.abort(self.env.now, &mut self.events);
            if let Some(accounts) = &mut self.accounts {
                accounts.abort();
            }
        }
    }
}

pub(crate) fn iterate(engine: &mut Engine, now: Time, wall: Wall) {
    engine.env.now = now;
    engine.env.wall = wall;
    engine.store.clock(now, wall);
    engine.store.iterate();
    answer(engine);
    if !engine.store.opened && engine.end.is_none() {
        return;
    }
    if let Some(accounts) = &mut engine.accounts {
        accounts.iterate(now, wall, &mut engine.values, &mut engine.events);
    }
    engine.workers.iterate(now, &mut engine.events);
    stop::advance(engine);
    if engine.end.is_some() {
        engine.abort_if_ended();
        return;
    }
    if !engine.room() {
        return;
    }
    if engine.begin {
        if engine.credentials.is_empty() {
            engine.begin = false;
            engine.step(root::Event::Restart);
        } else {
            let last = engine.credentials.len().checked_sub(1).expect("first credential");
            let number = *engine.credentials.get(last).expect("configured identity");
            engine.credentials.truncate(last);
            engine.step(root::Event::Credential {
                number,
                generation: 1,
                valid: skein_lib::Duration::from_nanos(u64::MAX),
            });
        }
    } else if !engine.blocked && engine.pending.is_some() {
        let event = engine.pending.take().expect("held input");
        engine.step(*event);
    } else if !engine.blocked && engine.domain.ready() && !engine.first_periods.is_empty() {
        let last = engine.first_periods.len().checked_sub(1).expect("first period");
        let (project, budget) = *engine.first_periods.get(last).expect("configured project");
        engine.first_periods.truncate(last);
        engine.step(root::Event::Party(root::PartyInput::Period { project, period: 1, budget, next: None }));
    } else if !engine.blocked && !engine.events.is_empty() {
        let event = engine.events.pop().expect("ready input");
        engine.step(event);
    } else if !engine.blocked
        && let Some(deadline) = engine.domain.next_deadline(&engine.env)
        && deadline <= now
    {
        root::fire(&mut engine.domain, &engine.env);
        engine.drain_facts();
    }
    release(engine);
}
fn release(engine: &mut Engine) {
    engine.ready = true;
    let mut empty = 0_u32;
    for _ in 0..root::max_out(&engine.env.limits) {
        if !engine.room() || engine.end.is_some() {
            break;
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        engine.drain_facts();
        if !engine.requests.is_empty() || engine.domain.quiescent() {
            engine.blocked = false;
        }
        if engine.requests.is_empty() {
            empty = empty.checked_add(1).expect("two empty passes");
        } else {
            empty = 0;
        }
        for _ in 0..engine.requests.len() {
            let request = engine.requests.pop().expect("root outputs");
            route::request(engine, request);
            if engine.end.is_some() {
                break;
            }
        }
        if empty == 2 {
            engine.ready = false;
            break;
        }
    }
    engine.domain.reclaim();
    stop::advance(engine);
}
fn answer(engine: &mut Engine) {
    if engine.end.is_some() {
        for _ in 0..engine.store.above.len() {
            let _late = engine.store.above.pop();
        }
        for _ in 0..engine.store.routed.len() {
            let _late = engine.store.routed.pop();
        }
        return;
    }
    for _ in 0..engine.store.above.len() {
        if engine.store.routed.room() < jig_protocol_store::MAX_OUT_DOWN {
            break;
        }
        let event = engine.store.above.pop().expect("store terminals");
        match event {
            jig_protocol_store::Event::Opened { .. } => engine.store.opened = true,
            jig_protocol_store::Event::OpenRefused { .. } => stop::unavailable(engine, b"deployment"),
            event @ (jig_protocol_store::Event::Kept { .. }
            | jig_protocol_store::Event::Erased { .. }
            | jig_protocol_store::Event::Secret { .. }
            | jig_protocol_store::Event::SecretsLoaded { .. }) => engine.store.secrets.push(event),
            event @ (jig_protocol_store::Event::Committed { .. }
            | jig_protocol_store::Event::Failed { .. }
            | jig_protocol_store::Event::Restored { .. }
            | jig_protocol_store::Event::Loaded { .. }
            | jig_protocol_store::Event::Read { .. }
            | jig_protocol_store::Event::Got { .. }
            | jig_protocol_store::Event::Turns { .. }
            | jig_protocol_store::Event::Unread { .. }
            | jig_protocol_store::Event::Corrupt { .. }) => {
                crate::store::route::up(&mut engine.store.routes, event, &mut engine.store.routed);
            }
        }
        if engine.end.is_some() {
            return;
        }
    }
    for _ in 0..engine.store.routed.len() {
        match engine.store.routed.pop().expect("store routing") {
            Up::Root(event) => {
                if let root::Event::Store(root::StoreInput::Loaded { rows, .. }) = event.as_ref() {
                    for row in rows {
                        if row.key()
                            == root::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment))
                        {
                            engine.first_periods.truncate(0);
                        }
                    }
                }
                if let root::Event::Store(root::StoreInput::Uncommitted { number }) = *event {
                    stop::failed(engine, number);
                } else {
                    engine.blocked = false;
                    engine.step(*event);
                }
            }
            Up::Ask(request) => engine.store.ask(request),
        }
        if engine.end.is_some() {
            break;
        }
    }
}

pub(crate) fn route_out() -> u32 {
    skein_oauth_accounts::MAX_OUT_DOWN.above.max(jig_protocol_engine_workers::MAX_OUT_DOWN)
}
