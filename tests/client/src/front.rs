//! Real engine root on its direct store driver, with jig's protocol correlation.
pub mod account_acts;
use crate::app::Event;
use crate::{app::Temper, fixtures, translate};
use jig_client_world::{
    harness::{ClientId, Front, FrontFault, Observed},
    joint::{packets, transport::Transport},
};
use skein_lib::{Duration, RequestEnvelope, Time, Token, Wall};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use temper_client_domain::{self as client, Event as RootEvent, Request};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

/// The common client Front backed by temper's own engine root.
pub struct Joint {
    pub engine: Driver,
    pub sign_ins: u32,
    pub answer_runs: bool,
    pub park_runs: bool,
    keyed_fault: Option<RequestEnvelope>,
    reads: usize,
    read_loads: BTreeSet<Token>,
    transport: Transport,
    limits: client::Limits,
    client_limits: BTreeMap<ClientId, client::Limits>,
    next_limits: Option<client::Limits>,
    callbacks: Vec<(ClientId, Event)>,
    parked: BTreeMap<ClientId, Vec<Request>>,
    offline_until: Option<Time>,
    due: Option<Time>,
    observed: Vec<Observed>,
    lose_answers: u32,
    lost_answer: Option<(ClientId, Token)>,
    hosting: BTreeMap<u64, u64>,
    seed: u64,
    now: Time,
    last_commit: u64,
    sign_outs: BTreeMap<ClientId, VecDeque<Option<u64>>>,
    truth: jig_client_world::joint::truth::Truth,
}
fn engine_limits() -> engine::Limits {
    let mut limits = temper_engine_domain_world::fixtures::forge_limits();
    let tools = temper_client_tools::limits::client().client.domain;
    // Frame project allocations must also fit the smaller tools project bound.
    limits.documents.lines = limits.documents.lines.min(tools.projects);
    limits
}
impl Joint {
    #[must_use]
    pub fn answers_to_lose(&self) -> u32 {
        self.lose_answers
    }
    #[must_use]
    pub fn engine_adopted(&self, forge: u16, repository: u32) -> bool {
        use temper_engine_domain_forge::{Key, Role, Stored};
        let key = engine::Key::Forge(Key::Repository(skein_forge::api::Repository { forge, repository }));
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
        let engine =
            Driver::with_repositories(Store::new(), walking::config(seed), &engine_limits(), fixtures::REPOSITORIES);
        Self::with_engine(seed, &limits, engine, 1)
    }
    /// The small profile carries a whole turn while an activity page can size it.
    #[must_use]
    pub fn with_large_turns(seed: u64, limits: &client::Limits) -> Self {
        let profile = temper_service::limits::profile(temper_service::limits::Profile::Small).root;
        let mut engine_limits = engine_limits();
        // tools.md, 11.2: preserve the profile's byte bounds for the large turn.
        engine_limits.journal.transcript_bytes = profile.journal.transcript_bytes;
        engine_limits.loads.bytes = profile.loads.bytes;
        engine_limits.loads.reply_bytes = profile.loads.reply_bytes;
        engine_limits.loads.transcript_bytes = profile.loads.transcript_bytes;
        let engine = Driver::with_repositories(
            Store::with_transcript_bytes(profile.journal.transcript_bytes),
            walking::config(seed),
            &engine_limits,
            fixtures::REPOSITORIES,
        );
        Self::with_engine(seed, limits, engine, 1)
    }
    /// A chat's allotment leaves pool funding for the goal it proposes.
    #[must_use]
    pub fn with_goal_proposals(seed: u64, limits: client::Limits) -> Self {
        let mut config = walking::config(seed);
        let rules = config.core.authority.rules().clone();
        let mut policy = config.core.authority.policy(1).expect("fixture project").clone();
        policy.roles[0].authority.budget.spend = 150;
        let mut domain =
            jig_domain_authority::Domain::new(rules, *config.core.authority.limits()).expect("goal authority");
        let mut out = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
        jig_domain_authority::step(&mut domain, jig_domain_authority::Event::Policy { project: 1, policy }, &mut out);
        assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: 1 }));
        config.core.authority = domain;
        let mut engine_limits = engine_limits();
        engine_limits.people.sign_ins = 3;
        engine_limits.fleet.slots = 2;
        engine_limits.fleet.attempts = 4;
        engine_limits.journal.writes = 4096;
        engine_limits.journal.deliveries = 64;
        engine_limits.journal.held = 192;
        let engine = Driver::with_repositories(Store::new(), config, &engine_limits, fixtures::REPOSITORIES);
        Self::with_engine(seed, &limits, engine, 2)
    }
    pub(crate) fn with_questions(seed: u64, limits: &client::Limits) -> Self {
        use jig_domain_authority as authority;
        let mut config = walking::config(seed);
        let mut rules = config.core.authority.rules().clone();
        rules.ceiling.delegation.depth = 2;
        let mut policy = config.core.authority.policy(fixtures::QUESTION_PROJECT).expect("fixture project").clone();
        policy.ceiling.delegation.depth = 2;
        policy.roles[0].authority.delegation.depth = 2;
        let mut domain = authority::Domain::new(rules, *config.core.authority.limits()).expect("question authority");
        let mut out = skein_lib::Queue::with_capacity(authority::POLICY_MAX_OUT);
        authority::step(
            &mut domain,
            authority::Event::Policy { project: fixtures::QUESTION_PROJECT, policy },
            &mut out,
        );
        assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: fixtures::QUESTION_PROJECT }));
        config.core.authority = domain;
        let mut engine_limits = engine_limits();
        engine_limits.tasks.inbox_bytes = 512;
        engine_limits.fleet.slots = 2;
        engine_limits.fleet.attempts = 4;
        engine_limits.journal.writes = 4096;
        engine_limits.journal.deliveries = 64;
        engine_limits.journal.held = 192;
        Self::with_engine(
            seed,
            limits,
            Driver::with_repositories(Store::new(), config, &engine_limits, fixtures::REPOSITORIES),
            2,
        )
    }
    /// A real forge procedure is admitted by this scenario's policy.
    #[must_use]
    pub(crate) fn with_procedures(seed: u64, limits: &client::Limits) -> Self {
        use jig_domain_authority as a;
        const PROJECT: u32 = 1;
        const PROCEDURE: u32 = 2;
        const EXECUTORS: u32 = 1;
        const POLICY_OUT: u32 = a::POLICY_MAX_OUT;
        const SLOTS: u32 = 1;
        let mut config = walking::config(seed);
        let mut rules = config.core.authority.rules().clone();
        let mut policy = config.core.authority.policy(PROJECT).expect("project").clone();
        let kinds: Box<[a::Executor]> = Box::new([a::Executor::Procedure(PROCEDURE)]);
        rules.ceiling.delegation.kinds.clone_from(&kinds);
        policy.ceiling.delegation.kinds.clone_from(&kinds);
        policy.roles[0].authority.delegation.kinds = kinds;
        let mut authority_limits = *config.core.authority.limits();
        authority_limits.executors = EXECUTORS;
        let mut domain = a::Domain::new(rules, authority_limits).expect("procedure policy limits");
        let mut out = skein_lib::Queue::with_capacity(POLICY_OUT);
        a::step(&mut domain, a::Event::Policy { project: PROJECT, policy }, &mut out);
        assert_eq!(out.pop(), Some(a::PolicyFact::Added { project: PROJECT }));
        config.core.authority = domain;
        let mut engine_limits = engine_limits();
        engine_limits.authority = authority_limits;
        let engine = Driver::with_repositories(Store::new(), config, &engine_limits, fixtures::REPOSITORIES);
        Self::with_engine(seed, limits, engine, SLOTS)
    }
    fn with_engine(seed: u64, limits: &client::Limits, mut engine: Driver, slots: u32) -> Self {
        engine.send(temper_engine_domain_world::protocol::hello_slots(fixtures::QUESTION_WORKER, slots, Box::new([])));
        engine.settle();
        let last_commit = engine.store.header().commits;
        let truth = jig_client_world::joint::truth::Truth::after(core_rows(&engine));
        engine.delivered.clear();
        engine.immediate.clear();
        Joint {
            engine,
            sign_ins: 0,
            answer_runs: false,
            park_runs: false,
            keyed_fault: None,
            reads: 0,
            read_loads: BTreeSet::new(),
            transport: Transport::new(jig_client::contract::VersionRange {
                first: jig_client_protocol_wire::VERSION,
                last: jig_client_protocol_wire::VERSION,
            }),
            limits: *limits,
            client_limits: BTreeMap::new(),
            next_limits: None,
            callbacks: Vec::new(),
            parked: BTreeMap::new(),
            offline_until: None,
            due: None,
            observed: vec![Observed::Reachable { live: true }],
            lose_answers: 0,
            lost_answer: None,
            hosting: BTreeMap::new(),
            seed,
            now: Time::ZERO,
            last_commit,
            truth,
            sign_outs: BTreeMap::new(),
        }
    }
    pub fn next_client_limits(&mut self, limits: client::Limits) {
        self.next_limits = Some(limits);
    }
    fn hello(&mut self, who: ClientId) {
        self.callbacks.push((
            who,
            translate::callback(jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: self.transport.hello() },
            })
            .into(),
        ));
        self.callbacks.push((
            who,
            RootEvent::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::FamilyServed {
                        family: fixtures::FORGE,
                        versions: Some(jig_client::contract::VersionRange {
                            first: temper_client_protocol_wire::CLIENT_VERSION,
                            last: temper_client_protocol_wire::CLIENT_VERSION,
                        }),
                    },
                },
            }
            .into(),
        ));
    }
    fn advance(&mut self, now: Time) {
        self.now = now;
        self.engine.env.now = now;
        self.engine.env.wall = Wall::from_nanos(now.as_nanos());
        if let Some(deadline) = self.engine.root.next_deadline(&self.engine.env)
            && deadline <= now
        {
            engine::fire(&mut self.engine.root, &self.engine.env);
        }
    }
    fn send(&mut self, mut event: engine::Event) {
        // The deployment front serves a window within the engine's configured page bound.
        if let engine::Event::Party(engine::PartyInput::Watch { window, .. }) = &mut event {
            window.rows = window.rows.min(self.engine.env.limits.documents.rows);
        }
        if matches!(event, engine::Event::Party(engine::PartyInput::SignedIn { .. })) {
            self.sign_ins += 1;
        }
        self.engine.send(event);
    }
    fn request(&mut self, who: ClientId, mut request: Request) {
        if self.offline_until.is_some() || self.transport.sign_in(who).is_none() {
            if let Request::Client {
                request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, value: Some(_), .. } },
            } = &request
            {
                self.callbacks.push((
                    who,
                    RootEvent::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt: *attempt,
                                envelope: RequestEnvelope::Again,
                                answer: None,
                                why: None,
                            },
                        },
                    }
                    .into(),
                ));
                return;
            }
            self.parked.entry(who).or_default().push(request);
            return;
        }
        if let Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } =
            &request
            && let Some(envelope) = self.keyed_fault.take()
        {
            if envelope == RequestEnvelope::SignedOut {
                self.transport.signed_out(who);
                self.sign_outs.entry(who).or_default().push_back(None);
                self.callbacks.push((
                    who,
                    RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }.into(),
                ));
            }
            self.callbacks.push((
                who,
                RootEvent::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer { attempt: *attempt, envelope, answer: None, why: None },
                    },
                }
                .into(),
            ));
            return;
        }
        for event in translate::request(&mut self.transport, who, &mut request, &self.client_limits[&who].client.domain)
        {
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
                if matches!(
                    event,
                    RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::Answer { .. } } }
                ) && self.lose_answers != 0
                {
                    if let RootEvent::Client {
                        event: jig_client::Event::Up { up: jig_client::Up::Answer { attempt, .. } },
                    } = &event
                    {
                        self.lost_answer = Some((client, *attempt));
                    }
                    self.lose_answers -= 1;
                } else {
                    self.callbacks.push((client, event.into()));
                }
            }
            packets::Incoming::SignedOut { client, sign_in } => {
                self.observed.push(Observed::SignInEnded { client });
                self.sign_outs.entry(client).or_default().push_back(Some(sign_in));
                self.callbacks.push((
                    client,
                    Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }),
                ));
            }
            packets::Incoming::SignInAgain { why, .. } => panic!("the deployment owner signs in again: {why:?}"),
            packets::Incoming::SignInFailed { failure, .. } => panic!("the deployment owner signs in: {failure:?}"),
        }
    }
    fn packet(&mut self, request: engine::Request) {
        match translate::forge::incoming(&mut self.transport, request) {
            Ok(Some((client, event))) => {
                if self.lose_answers != 0 {
                    self.lose_answers -= 1;
                } else {
                    self.callbacks.push((client, event.into()));
                }
            }
            Ok(None) => {}
            Err(request) => {
                let mut who = None;
                if let engine::Request::Party(engine::PartyRequest::Deliver(
                    engine::Delivery::WebReply { to, .. } | engine::Delivery::Read { reply: to, .. },
                )) = request.as_ref()
                {
                    who = self
                        .transport
                        .reply_tokens()
                        .into_iter()
                        .find(|token| *to == skein_lib::ReplyTo::new(*token))
                        .and_then(|token| self.transport.reply(token))
                        .map(|reply| match reply {
                            jig_client_world::joint::transport::Reply::SignIn { client }
                            | jig_client_world::joint::transport::Reply::SignOut { client, .. }
                            | jig_client_world::joint::transport::Reply::Keyed { client, .. }
                            | jig_client_world::joint::transport::Reply::Read { client, .. }
                            | jig_client_world::joint::transport::Reply::ApplicationRead { client, .. } => *client,
                        });
                }
                if let engine::Request::Party(
                    engine::PartyRequest::Deliver(
                        engine::Delivery::Document { watcher, .. } | engine::Delivery::WatchEnded { watcher, .. },
                    )
                    | engine::PartyRequest::WatchRefused { watcher, .. }
                    | engine::PartyRequest::RunText { watcher, .. },
                ) = request.as_ref()
                {
                    who = self.transport.watch(*watcher).map(|watch| watch.client);
                }
                let limits = who.map_or(self.limits, |who| self.client_limits[&who]);
                let codec =
                    if limits == self.limits { fixtures::codec() } else { fixtures::codec_for(&limits.client.domain) };
                if let Some(packet) = translate::incoming(&mut self.transport, *request, &limits.client.domain, &codec)
                {
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
        panic!(
            "joint engine did not settle: quiescent={}; returned={}; events={:?}; pending={:?}",
            self.engine.root.quiescent(),
            self.engine.returned,
            self.engine.events,
            self.engine.store.pending
        );
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
            if let engine::Delivery::Link { channel, down: jig_domain_host::LinkDown::Cancel { run, attempt } } =
                &delivery
            {
                self.hosting.remove(&run.raw());
                self.engine.answer(
                    *channel,
                    run.raw(),
                    attempt.raw(),
                    0,
                    jig_domain_tasks::End::Parked,
                    engine::Parts::none(),
                );
            }
            if self.park_runs
                && let engine::Delivery::Assigned { assignment, .. } = &delivery
            {
                if let Some(word) = assignment.messages.last() {
                    self.engine.send(temper_engine_domain_world::protocol::turn(
                        Token::new(7),
                        assignment.task,
                        assignment.attempt,
                        1,
                        0,
                        Some(word.number),
                        b"read messages",
                    ));
                    self.engine.settle();
                }
                self.engine.answer(
                    Token::new(7),
                    assignment.task,
                    assignment.attempt,
                    0,
                    jig_domain_tasks::End::Parked,
                    engine::Parts::none(),
                );
            }
            if !self.answer_runs
                && let engine::Delivery::Assigned { assignment, .. } = &delivery
            {
                self.hosting.insert(assignment.task, assignment.attempt);
            }
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
        self.client_limits.insert(who, self.next_limits.take().unwrap_or(self.limits));
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
        self.callbacks.push((
            who,
            Event::Root(translate::callback(jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: self.transport.hello() },
            })),
        ));
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
                self.hello(who);
                if self.transport.sign_in(who).is_none() {
                    self.sign_outs.entry(who).or_default().push_back(None);
                    self.callbacks.push((
                        who,
                        Event::Root(RootEvent::Client {
                            event: jig_client::Event::Up { up: jig_client::Up::SignedOut },
                        }),
                    ));
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
                let limits = self.engine.env.limits;
                let hosting = self
                    .hosting
                    .iter()
                    .map(|(task, attempt)| {
                        temper_engine_domain_world::protocol::hosting(*task, *attempt, jig_domain_host::Phase::Active)
                    })
                    .collect();
                let store = std::mem::replace(&mut self.engine.store, Store::new());
                self.engine =
                    Driver::with_repositories(store, walking::config(self.seed), &limits, fixtures::REPOSITORIES);
                self.engine.send(temper_engine_domain_world::protocol::hello_slots(
                    fixtures::QUESTION_WORKER,
                    limits.fleet.slots,
                    hosting,
                ));
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
        // A terminal restart has no retained address, unlike a browser reload.
        self.callbacks.push((who, RootEvent::Went { place: client::Place::Address(fixtures::inbox()) }.into()));
        if self.offline_until.is_none() {
            if self.transport.sign_in(who).is_some() {
                self.callbacks.push((
                    who,
                    RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } }.into(),
                ));
            }
            self.hello(who);
            if self.transport.sign_in(who).is_none() {
                self.sign_outs.entry(who).or_default().push_back(None);
                self.callbacks.push((
                    who,
                    Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } }),
                ));
            }
        }
    }
    fn received(&mut self, who: ClientId, event: &Event) {
        if let Event::Root(RootEvent::Client {
            event:
                jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: jig_client::contract::WatchEvent::Delivered(_) },
                },
        }) = event
            && let Some(event) = self.transport.taken(who, *watch)
        {
            self.send(translate::party(event));
        }
    }
    fn sign_in(&mut self, who: ClientId, now: Time, _wall: Wall) {
        self.advance(now);
        self.callbacks.retain(|(client, event)| {
            *client != who
                || !matches!(
                    event,
                    Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } })
                )
        });
        self.sign_outs.remove(&who);
        self.callbacks.push((
            who,
            Event::Root(translate::callback(jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: self.transport.hello() },
            })),
        ));
        let event = translate::party(self.transport.start_sign_in(who));
        self.send(event);
    }
    fn delivers(&mut self, client: ClientId, event: &Event) -> bool {
        if !matches!(
            event,
            Event::Root(RootEvent::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } })
        ) {
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
            self.callbacks.push((who, Event::Root(RootEvent::Client { event: jig_client::Event::SignOutLost })));
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

#[cfg(test)]
mod tests;

mod accounts;
