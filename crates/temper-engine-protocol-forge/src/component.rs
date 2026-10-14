//! Waiting calls and returned answers (forge.md, sections 2 and 8).
use crate::{
    boundary::{Answered, Event, Request},
    limits::{self, Limits, Shared, Unusable},
    settings::Settings,
    start::Start,
    tables::Tables,
};
use skein_lib::{Env, Id, List, Map, Queue, Slab, Time, Token};
use temper_engine_domain_forge_client::api;
/// Output reservations for each entry point.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub above: u32,
    pub below: u32,
}
/// Reserve these slots before an entry point runs.
pub const MAX_OUT: MaxOut = MaxOut { above: 16, below: 16 };
/// Malformed answers and long bodies counted by the component.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Diagnostics {
    pub long_bodies: u64,
    pub malformed: u64,
}
#[derive(Debug)]
#[expect(dead_code, reason = "13.3 consumes waiting operations after a forge starts")]
pub(crate) struct Waiting {
    pub repository: api::Repository,
    pub op: api::Op,
    pub deadline: Time,
}
#[derive(Debug)]
pub(crate) struct Held {
    pub answer: Answered,
    pub offer: bool,
}
/// One component per deployment, owning every configured token and start.
#[derive(Debug)]
pub struct Component {
    settings: Settings,
    starts: List<Start>,
    tables: Tables,
    waiting: Slab<Waiting>,
    calls: Map<Token, Id<Waiting>>,
    held: Map<Token, Held>,
    closed: bool,
    diagnostics: Diagnostics,
}
impl Component {
    pub fn new(settings: Settings, limits: &Limits, shared: &Shared) -> Result<Self, Unusable> {
        limits::check(limits, shared, &settings)?;
        let mut starts = List::with_capacity(limits.forges);
        for _ in &settings.forges {
            starts.push(Start::Checking { next: limits.start_backoff }).expect("checked forge count");
        }
        Ok(Self {
            settings,
            starts,
            tables: Tables::new(limits.repositories, limits.users),
            waiting: Slab::with_capacity(shared.client.calls),
            calls: Map::with_capacity(shared.client.calls),
            held: Map::with_capacity(shared.client.calls),
            closed: false,
            diagnostics: Diagnostics { long_bodies: 0, malformed: 0 },
        })
    }
    /// Starts each forge's pure check; connections arrive in 13.2.
    pub fn begin(&mut self, env: &Env<Limits>, _up: &mut Queue<Event>, _io: &mut Queue<skein_io::Request>) {
        if self.closed {
            return;
        }
        for at in 0..self.starts.len() {
            *self.starts.get_mut(at).expect("configured start") = Start::Checking { next: env.limits.start_backoff };
        }
    }
    pub fn down(
        &mut self,
        env: &Env<Limits>,
        request: Request,
        up: &mut Queue<Event>,
        _io: &mut Queue<skein_io::Request>,
    ) {
        if self.closed {
            return;
        }
        match request {
            Request::Call { call, repository, op } => {
                assert!(!self.calls.contains_key(&call), "one live call per token");
                let id = self
                    .waiting
                    .insert(Waiting { repository, op, deadline: env.now.saturating_add(env.limits.deadline) })
                    .expect("client bounds admitted calls");
                let _previous: Option<Id<Waiting>> = self.calls.insert(call, id).expect("call index matches its slab");
            }
            Request::Refused(answer) => {
                drop(
                    self.held
                        .insert(answer.call, Held { answer, offer: true })
                        .expect("at most one held answer per client call"),
                );
            }
            Request::Returned(answer) => {
                drop(
                    self.held
                        .insert(answer.call, Held { answer, offer: false })
                        .expect("at most one held answer per client call"),
                );
            }
            Request::Offer => {
                for _ in 0..MAX_OUT.above {
                    let mut next = None;
                    for (call, held) in &self.held {
                        if held.offer {
                            next = Some(*call);
                            break;
                        }
                    }
                    let Some(call) = next else {
                        break;
                    };
                    if let Some(held) = self.held.remove(&call) {
                        up.push(Event::Answered(held.answer));
                    }
                }
            }
            Request::Close => {
                for _ in 0..self.calls.len() {
                    let Some((call, id)) = self.calls.first() else {
                        break;
                    };
                    let (call, id) = (*call, *id);
                    self.waiting.retire(id);
                    let _removed: Option<Id<Waiting>> = self.calls.remove(&call);
                }
                self.held = Map::with_capacity(self.held.capacity());
                self.closed = true;
                up.push(Event::Closed);
            }
        }
    }
    /// No connection exists until 13.2; late io events have no effect.
    pub fn up(
        &mut self,
        _env: &Env<Limits>,
        event: skein_io::Event,
        _up: &mut Queue<Event>,
        _io: &mut Queue<skein_io::Request>,
    ) {
        drop(event);
    }
    pub fn fire(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, _io: &mut Queue<skein_io::Request>) {
        if self.closed {
            return;
        }
        for _ in 0..MAX_OUT.above {
            let mut next = None;
            for (call, id) in &self.calls {
                if self.waiting.get(*id).expect("indexed live call").deadline <= env.now {
                    next = Some((*call, *id));
                    break;
                }
            }
            let Some((call, id)) = next else {
                break;
            };
            self.waiting.retire(id);
            let _removed: Option<Id<Waiting>> = self.calls.remove(&call);
            up.push(Event::Answered(Answered { call, cost: 0, result: Err(api::Error::Unavailable) }));
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut earliest = None;
        for (_, id) in &self.calls {
            let time = self.waiting.get(*id).expect("indexed live call").deadline;
            if earliest.is_none() || Some(time) < earliest {
                earliest = Some(time);
            }
        }
        earliest
    }
    #[must_use]
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }
    pub fn reclaim(&mut self) {
        self.waiting.reclaim();
    }
    /// The bounded resolution tables owned by this component.
    pub fn tables(&mut self) -> &mut Tables {
        &mut self.tables
    }
    /// The pure start, in configured forge order.
    #[must_use]
    pub fn start(&self, forge: u16) -> Option<Start> {
        for (at, settings) in self.settings.forges.as_slice().iter().enumerate() {
            if settings.number == forge {
                return self.starts.as_slice().get(at).copied();
            }
        }
        None
    }
}
