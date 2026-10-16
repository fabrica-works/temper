//! Real engine root on its direct store driver, with jig's protocol correlation.
use crate::{app::Temper, fixtures, translate};
use jig_client_world::{
    harness::{ClientId, Front, FrontFault, Observed},
    joint::{packets, transport::Transport},
};
use skein_lib::{Duration, RequestEnvelope, Time, Token, Wall};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use temper_client_domain::{self as client, Event, Request};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

/// The common client Front backed by temper's own engine root.
pub struct Joint {
    pub engine: Driver,
    pub sign_ins: u32,
    pub answer_runs: bool,
    keyed_fault: Option<RequestEnvelope>,
    reads: usize,
    read_loads: BTreeSet<Token>,
    transport: Transport,
    limits: client::Limits,
    callbacks: Vec<(ClientId, Event)>,
    parked: BTreeMap<ClientId, Vec<Request>>,
    offline_until: Option<Time>,
    due: Option<Time>,
    observed: Vec<Observed>,
    lose_answers: u32,
    seed: u64,
    now: Time,
    last_commit: u64,
    sign_outs: BTreeMap<ClientId, VecDeque<Option<u64>>>,
    truth: jig_client_world::joint::truth::Truth,
}
impl Joint {
    #[must_use]
    pub fn engine_adopted(&self, forge: u16, repository: u32) -> bool {
        use temper_engine_domain_forge::{Key, Role, Stored};
        let key = engine::Key::Forge(Key::Repository(temper_engine_domain_forge_client::api::Repository {
            forge,
            repository,
        }));
        let Some(engine::Record::Forge { row }) = self.engine.store.rows.get(&key) else { return false };
        matches!(row.as_ref(), Stored::Repository(value) if value.project == 1 && value.role == Role::Owned)
    }
    #[must_use]
    pub fn applications_admitted(&self) -> usize {
        self.engine
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(record,
                    engine::Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Answer { ask, .. }))
                    if matches!(ask.as_ref(), jig_domain_people::Ask::Application { .. })
                )
            })
            .count()
    }
    #[must_use]
    pub fn new(seed: u64, limits: client::Limits) -> Joint {
        let mut engine = Driver::with_repositories(
            Store::new(),
            walking::config(seed),
            &temper_engine_domain_world::fixtures::forge_limits(),
            fixtures::REPOSITORIES,
        );
        engine.hello(Token::new(7));
        engine.settle();
        let last_commit = engine.store.header().commits;
        let truth = jig_client_world::joint::truth::Truth::after(core_rows(&engine));
        engine.delivered.clear();
        engine.immediate.clear();
        Joint {
            engine,
            sign_ins: 0,
            answer_runs: false,
            keyed_fault: None,
            reads: 0,
            read_loads: BTreeSet::new(),
            transport: Transport::new(jig_domain_client::contract::VersionRange {
                first: jig_domain_client::contract::CLIENT_VERSION,
                last: jig_domain_client::contract::CLIENT_VERSION,
            }),
            limits,
            callbacks: Vec::new(),
            parked: BTreeMap::new(),
            offline_until: None,
            due: None,
            observed: vec![Observed::Reachable { live: true }],
            lose_answers: 0,
            seed,
            now: Time::ZERO,
            last_commit,
            truth,
            sign_outs: BTreeMap::new(),
        }
    }
    fn advance(&mut self, now: Time) {
        self.now = now;
        self.engine.env.now = now;
        self.engine.env.wall = Wall::from_nanos(now.as_nanos());
    }
    fn send(&mut self, event: engine::Event) {
        if matches!(event, engine::Event::Party(engine::PartyInput::SignedIn { .. })) {
            self.sign_ins += 1;
        }
        self.engine.send(event);
    }
    fn request(&mut self, who: ClientId, request: Request) {
        if self.offline_until.is_some() || self.transport.sign_in(who).is_none() {
            self.parked.entry(who).or_default().push(request);
            return;
        }
        if let Request::Send { attempt, .. } = &request
            && let Some(envelope) = self.keyed_fault.take()
        {
            if envelope == RequestEnvelope::SignedOut {
                self.transport.signed_out(who);
            }
            self.callbacks.push((who, Event::Answer { attempt: *attempt, envelope, answer: None }));
            return;
        }
        for event in translate::request(&mut self.transport, who, &request, &self.limits.jig) {
            self.send(event);
        }
    }
    fn incoming(&mut self, packet: packets::Incoming) {
        match packet {
            packets::Incoming::Authenticated { client, .. } => {
                for request in self.parked.remove(&client).unwrap_or_default() {
                    self.request(client, request);
                }
            }
            packets::Incoming::Callback { client, event } => {
                let event = translate::callback(event);
                if matches!(event, Event::Answer { .. }) && self.lose_answers != 0 {
                    self.lose_answers -= 1;
                } else {
                    self.callbacks.push((client, event));
                }
            }
            packets::Incoming::SignedOut { client, sign_in } => {
                self.sign_outs.entry(client).or_default().push_back(Some(sign_in));
                self.callbacks.push((client, Event::SignedOut));
            }
            packets::Incoming::SignInAgain { .. } | packets::Incoming::SignInFailed { .. } => {
                panic!("the deployment owner signs in")
            }
        }
    }
    fn packet(&mut self, request: engine::Request) {
        match translate::forge::incoming(&mut self.transport, request) {
            Ok(Some((client, event))) => {
                if self.lose_answers != 0 {
                    self.lose_answers -= 1;
                } else {
                    self.callbacks.push((client, event));
                }
            }
            Ok(None) => {}
            Err(request) => {
                if let Some(packet) = translate::incoming(&mut self.transport, *request, &self.limits.jig) {
                    self.incoming(packet);
                }
            }
        }
    }
    fn settle(&mut self) {
        for _ in 0..200 {
            for event in &mut self.engine.events {
                if let engine::Event::Store(engine::StoreInput::ReadLoaded { load, result }) = event
                    && self.read_loads.insert(*load)
                    && let Some(range) = self.engine.protocol.reads.get(self.reads)
                {
                    *result = fixtures::read(&self.engine.store, range);
                    self.reads += 1;
                }
            }
            self.engine.advance(true);
            if self.engine.root.quiescent()
                && self.engine.events.is_empty()
                && self.engine.store.pending.is_empty()
                && self.engine.forge.as_ref().is_none_or(temper_engine_domain_world::forge_stand::ForgeStand::quiescent)
            {
                return;
            }
        }
        panic!("joint engine did not settle");
    }
    fn drain(&mut self) {
        self.settle();
        let commit = self.engine.store.header().commits;
        if commit != self.last_commit {
            self.observed.extend(self.truth.applied(jig_core_world::store_peer::DurableSnapshot {
                commit,
                at: self.now,
                rows: core_rows(&self.engine),
            }));
            self.last_commit = commit;
        }
        let deliveries = std::mem::take(&mut self.engine.delivered);
        for delivery in deliveries {
            if self.answer_runs
                && let engine::Delivery::Assigned { assignment, .. } = &delivery
            {
                self.engine.answer(
                    Token::new(7),
                    assignment.task,
                    assignment.attempt,
                    0,
                    jig_domain_tasks::End::Finished {
                        result: jig_domain_tasks::TaskResult::Report {
                            words: b"Answered by the run".as_slice().into(),
                        },
                        cancel_delegates: false,
                    },
                    engine::Parts::none(),
                );
            }
            self.packet(engine::Request::Party(engine::PartyRequest::Deliver(delivery)));
        }
        let immediate = std::mem::take(&mut self.engine.immediate);
        for request in immediate {
            self.packet(request);
        }
    }
    fn detach(&mut self, who: ClientId) {
        self.sign_outs.remove(&who);
        self.callbacks.retain(|(client, _)| *client != who);
        self.parked.remove(&who);
        for watcher in self.transport.detach(who) {
            self.send(translate::party(jig_domain_core::Event::Unwatch { watcher }));
        }
    }
    fn drop_watches(&mut self) {
        for watcher in self.transport.watches() {
            let packet = packets::dropped(&mut self.transport, watcher).expect("original protocol watch");
            self.send(translate::party(jig_domain_core::Event::Unwatch { watcher }));
            self.incoming(packet);
        }
    }
}
impl Front<Temper> for Joint {
    fn bind(&mut self, who: ClientId, party: u64) {
        assert_eq!(party, fixtures::PARTY, "the configured deployment owner");
        self.transport.bind(
            who,
            party,
            jig_domain_people::Identity {
                key: jig_domain_people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: b"person".as_slice().into(),
                name: b"Person".as_slice().into(),
            },
        );
        self.callbacks.push((who, Event::Jig(self.transport.hello())));
        let event = translate::party(self.transport.start_sign_in(who));
        self.send(event);
    }
    fn wire(&mut self, who: ClientId, now: Time, _wall: Wall, request: Request) {
        self.advance(now);
        self.request(who, request);
    }
    fn poll(&mut self, now: Time, _wall: Wall) -> Vec<(ClientId, Event)> {
        self.advance(now);
        if let Some(until) = self.offline_until {
            if now < until {
                return std::mem::take(&mut self.callbacks);
            }
            self.offline_until = None;
            self.observed.push(Observed::Reachable { live: true });
            for who in self.transport.clients() {
                self.callbacks.push((who, Event::Jig(self.transport.hello())));
                if self.transport.sign_in(who).is_none() {
                    self.sign_outs.entry(who).or_default().push_back(None);
                    self.callbacks.push((who, Event::SignedOut));
                }
            }
            for (who, requests) in std::mem::take(&mut self.parked) {
                for request in requests {
                    self.request(who, request);
                }
            }
        }
        self.drain();
        if self.due.is_some_and(|due| now < due) {
            return Vec::new();
        }
        self.due = None;
        std::mem::take(&mut self.callbacks)
    }
    fn next_due(&self) -> Option<Time> {
        self.offline_until.or(self.due).or_else(|| Some(self.now.saturating_add(Duration::from_millis(100))))
    }
    fn observed(&mut self) -> Vec<Observed> {
        std::mem::take(&mut self.observed)
    }
    fn fault(&mut self, now: Time, fault: FrontFault) {
        self.advance(now);
        match fault {
            FrontFault::Restart { offline } => {
                let store = std::mem::replace(&mut self.engine.store, Store::new());
                self.engine = Driver::with_repositories(
                    store,
                    walking::config(self.seed),
                    &temper_engine_domain_world::fixtures::forge_limits(),
                    fixtures::REPOSITORIES,
                );
                self.engine.settle();
                self.reads = 0;
                self.read_loads.clear();
                self.offline_until = Some(now.saturating_add(offline));
                self.observed.push(Observed::Reachable { live: false });
                self.drop_watches();
            }
            FrontFault::Slow { span } => self.due = Some(now.saturating_add(span)),
            FrontFault::NoAnswer => self.lose_answers = self.lose_answers.checked_add(1).expect("finite lost answers"),
            FrontFault::DropWatch => self.drop_watches(),
            FrontFault::Busy => self.keyed_fault = Some(RequestEnvelope::Again),
            FrontFault::SignedOut => self.keyed_fault = Some(RequestEnvelope::SignedOut),
            FrontFault::NotReady | FrontFault::Versions | FrontFault::ObjectGone => {
                panic!("not a temper joint story fault: {fault:?}")
            }
        }
    }
    fn reload(&mut self, who: ClientId, now: Time) {
        self.advance(now);
        self.detach(who);
        if self.offline_until.is_none() {
            self.callbacks.push((who, Event::Jig(self.transport.hello())));
            if self.transport.sign_in(who).is_none() {
                self.sign_outs.entry(who).or_default().push_back(None);
                self.callbacks.push((who, Event::SignedOut));
            }
        }
    }
    fn received(&mut self, who: ClientId, event: &Event) {
        if let Event::Jig(jig_domain_client::Event::Watch {
            watch,
            event: jig_domain_client::contract::WatchEvent::Delivered(_),
        }) = event
            && let Some(event) = self.transport.taken(who, *watch)
        {
            self.send(translate::party(event));
        }
    }
    fn sign_in(&mut self, who: ClientId, now: Time, _wall: Wall) {
        self.advance(now);
        self.callbacks.retain(|(client, event)| *client != who || !matches!(event, Event::SignedOut));
        self.sign_outs.remove(&who);
        self.callbacks.push((who, Event::Jig(self.transport.hello())));
        let event = translate::party(self.transport.start_sign_in(who));
        self.send(event);
    }
    fn delivers(&mut self, client: ClientId, event: &Event) -> bool {
        if !matches!(event, Event::SignedOut) {
            return true;
        }
        let sign_in = self
            .sign_outs
            .get_mut(&client)
            .and_then(VecDeque::pop_front)
            .expect("sign-out callback retains its exchange");
        self.transport.sign_in(client).is_none_or(|current| Some(current) == sign_in)
    }
    fn sign_out(&mut self, who: ClientId, now: Time, _wall: Wall) {
        self.advance(now);
        if self.offline_until.is_some() {
            self.callbacks.push((who, Event::Jig(jig_domain_client::Event::SignOutLost)));
            return;
        }
        let Some(sign_in) = self.transport.sign_in(who) else { return };
        let reply_to =
            self.transport.reply_to(jig_client_world::joint::transport::Reply::SignOut { client: who, sign_in });
        self.send(translate::party(jig_domain_core::Event::People(jig_domain_people::Event::SignOut {
            reply_to,
            sign_in,
        })));
    }
}

// The kit's truth reads the unchanged core family of the externally applied store.
// Temper's other durable families are not jig client-referee facts.
fn core_rows(engine: &Driver) -> BTreeMap<jig_test_domain::Key, jig_test_domain::Record> {
    engine
        .store
        .rows
        .iter()
        .filter_map(|(key, record)| {
            if let (engine::Key::Core(key), engine::Record::Core(record)) = (key, record) {
                Some((jig_test_domain::Key::Core(key.clone()), jig_test_domain::Record::Core(record.clone())))
            } else {
                None
            }
        })
        .collect()
}
