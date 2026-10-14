//! Root forge stories with its durable store and the independent fake Forgejo.
#![expect(clippy::wildcard_enum_match_arm, reason = "the fixture selects only the forge rows relevant to each story")]
use jig_domain_authority as authority;
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, List, Queue, ReplyTo, Time, Token, Wall};
use std::collections::{BTreeMap, VecDeque};
use temper_engine_domain as engine;
use temper_engine_domain::{Delivery, Key, Record};
use temper_engine_domain_forge as forge_top;
use temper_engine_domain_forge_client as client;
use temper_engine_domain_world::protocol::{self, Protocol};
use temper_engine_domain_world::{store::Store, walking};
use temper_engine_forge_world::{self as forge_world, translate};
use temper_fake_forge_domain::{self as fake, api as raw};

struct World {
    root: engine::Domain,
    env: Env<engine::Limits>,
    out: Queue<engine::Request>,
    events: VecDeque<engine::Event>,
    store: Store,
    fake: fake::Domain,
    fake_env: Env<fake::Config>,
    fake_out: Queue<fake::Request>,
    pending: BTreeMap<Token, client::api::Op>,
    forge_calls: u32,
    adopted: Vec<people::Outcome>,
    signed_in: Option<u64>,
    assigned: Vec<engine::Assignment>,
    answers: Vec<temper_engine_domain::CallAnswer>,
    protocol: Protocol,
    immediate: VecDeque<engine::Event>,
    /// The funding period sent to this process's root, once it is ready.
    period: Option<u64>,
    fail_job_reads: bool,
    slow_brief_reads: bool,
    owner_gate: bool,
}

#[derive(Clone, Copy, Debug)]
enum Until {
    Ready,
    SignedIn,
    Adopted,
    Assigned,
    Subscribed,
    News,
    Read,
    Effect,
    Delegated,
    SecondAssignment,
}

impl World {
    fn new() -> Self {
        Self::configured(false, false)
    }

    fn configured(with_read: bool, with_change: bool) -> Self {
        Self::configured_ci(with_read, with_change, false)
    }

    fn configured_ci(with_read: bool, with_change: bool, silent_ci: bool) -> Self {
        Self::configured_checks(with_read, with_change, silent_ci, 1000)
    }

    fn configured_checks(with_read: bool, with_change: bool, silent_ci: bool, passes: u32) -> Self {
        Self::configured_policy(with_read, with_change, silent_ci, passes, None, false, false)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one fixture configures the policy, fake, and bounds for root forge stories"
    )]
    #[expect(clippy::fn_params_excessive_bools, reason = "the fixture varies independent forge and gate conditions")]
    fn configured_policy(
        with_read: bool,
        with_change: bool,
        silent_ci: bool,
        passes: u32,
        approval: Option<engine::Freshness>,
        agent_gate: bool,
        owner_gate: bool,
    ) -> Self {
        let mut limits = walking::limits();
        limits.people.people = 4;
        limits.people.holdings = 4;
        if owner_gate {
            limits.people.amendment_bytes = 2048;
        }
        limits.journal.writes += (people::max_out(&limits.people) - people::max_out(&walking::limits().people)) * 4;
        let mut config = walking::config(71);
        if approval.is_some() || agent_gate || owner_gate {
            limits.authority.roles = 3;
        }
        if with_change {
            limits.authority.requirements = 8;
            limits.authority.facts = 8;
        }
        if with_change {
            limits.brief.gather = Duration::from_secs(5);
            limits.brief.read_bytes = 512;
            limits.brief.budgets.pull = 384;
            limits.brief.budgets.ci = 384;
            limits.brief.brief_bytes = 768;
            limits.tasks.tasks = 12;
            limits.tasks.project_tasks = 12;
            limits.tasks.delegates = 4;
            limits.tasks.tree_tasks = 12;
            limits.tasks.depth = 3;
            limits.tasks.parameters = 8;
            limits.tasks.spec_bytes = 512;
            limits.tasks.executor_kinds = 2;
            limits.tasks.authority_grants = 10;
            limits.journal.run_bytes = 8192;
            limits.tasks.authority_segments = 9;
            limits.tasks.authority_bytes = 512;
            limits.tasks.contract_choices = 2;
            limits.tasks.inbox_messages = 8;
            limits.tasks.inbox_bytes = 1024;
            limits.authority.executors = 2;
            limits.authority.batch = 2;
            limits.authority.grants = 10;
            limits.authority.segments = 9;
            limits.authority.segment_bytes = 64;
            limits.authority.writes = 2;
            limits.journal.writes += 64;
            // The tasks child's whole route grew with jig's task families.
            limits.journal.writes = 8192;
            limits.journal.deliveries = 256;
            limits.journal.held = 1024;
            limits.journal.result_bytes = 4096;
            limits.journal.transcript_bytes = 32_768;
            limits.fleet.slots = 2;
            limits.fleet.workstreams = 2;
            // The core's inbox census and waiting entries cover every live task.
            limits.people.inbox_tasks = 24;
            limits.people.inbox_recipients = 32;
        }
        if with_read {
            limits.tasks.inbox_bytes = limits.tasks.inbox_bytes.max(256);
            if !with_change {
                limits.authority.grants = 4;
            }
            limits.authority.segments = 9;
            limits.authority.segment_bytes = 64;
            if !with_change {
                limits.tasks.authority_grants = 2;
            }
            limits.tasks.authority_segments = 9;
            if !with_change {
                limits.tasks.authority_bytes = 64;
            }
            let grant = authority::Grant {
                connector: 0,
                kind: 1,
                pattern: authority::Pattern {
                    segments: Box::new([
                        Box::from(&b"forge"[..]),
                        Box::from(&b"forge.example"[..]),
                        Box::from(&b"org"[..]),
                    ]),
                    last: authority::Last::Open(Box::from(&b"repo"[..])),
                },
            };
            let effect_grant = authority::Grant { kind: 8, ..grant.clone() };
            let mut rules = config.core.authority.rules().clone();
            if with_change {
                // The chat's 100-unit allotment must retain room for its 20-unit delegate
                // after the run reserves its capped allowance at claim.
                rules.maximum_run_spend = 60;
            }
            rules.ceiling.tools = authority::Tools(1023);
            let grants: Box<[authority::Grant]> = if with_change {
                Box::new([
                    grant.clone(),
                    effect_grant.clone(),
                    authority::Grant { kind: 2, ..grant.clone() },
                    authority::Grant { kind: 3, ..grant.clone() },
                    authority::Grant { kind: 4, ..grant.clone() },
                    authority::Grant { kind: 10, ..grant.clone() },
                    authority::Grant { kind: 11, ..grant.clone() },
                    authority::Grant { kind: 12, ..grant.clone() },
                ])
            } else {
                Box::new([grant.clone(), effect_grant.clone()])
            };
            let mut ceiling_grants = List::with_capacity(limits.authority.grants);
            for grant in &grants {
                ceiling_grants.push(grant.clone()).expect("configured task grants");
            }
            ceiling_grants.push(authority::Grant { kind: 7, ..grant.clone() }).expect("projection comments");
            ceiling_grants.push(authority::Grant { kind: 13, ..grant.clone() }).expect("projection issue sets");
            rules.ceiling.grants = ceiling_grants.into_boxed();
            if approval.is_some() || agent_gate || owner_gate {
                let landing: Box<[engine::LandingRule]> = Box::new([engine::LandingRule {
                    connector: 0,
                    kind: 4,
                    enforce: !owner_gate,
                    pattern: authority::Pattern {
                        segments: Box::new([
                            Box::from(&b"forge"[..]),
                            Box::from(&b"forge.example"[..]),
                            Box::from(&b"org"[..]),
                            Box::from(&b"repo"[..]),
                            Box::from(&b"branch"[..]),
                        ]),
                        last: authority::Last::Exact(Box::from(&b"main"[..])),
                    },
                    ci: true,
                    up_to_date: true,
                    gates: if agent_gate || owner_gate {
                        Box::new([engine::Gate { number: 1, blocking: true, freshness: engine::Freshness::Exact }])
                    } else {
                        Box::new([])
                    },
                    approvals: match approval {
                        Some(freshness) => Box::new([engine::Approval { role: 2, people: 1, freshness }]),
                        None => Box::new([]),
                    },
                }]);
                if !owner_gate {
                    config.landing.deployment.clone_from(&landing);
                }
                assert!(config.landing.projects.insert(1, landing).is_ok());
            }
            if with_change {
                rules.ceiling.delegation.kinds =
                    Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(2)]);
                rules.ceiling.delegation.tasks = 12;
                rules.ceiling.delegation.depth = 3;
            }
            let mut policy = config.core.authority.policy(1).expect("walk policy").clone();
            policy.ceiling.tools = authority::Tools(1023);
            policy.ceiling.grants.clone_from(&rules.ceiling.grants);
            policy.projections.clone_from(&rules.ceiling.grants);
            if approval.is_some() || agent_gate || owner_gate {
                let mut maintainer = policy.roles[0].clone();
                maintainer.number = 1;
                let mut member = maintainer.clone();
                member.number = 2;
                policy.roles = Box::new([policy.roles[0].clone(), maintainer, member]);
            }
            policy.roles[0].authority.tools = authority::Tools(1023);
            policy.roles[0].authority.grants.clone_from(&grants);
            if with_change {
                policy.ceiling.delegation = rules.ceiling.delegation.clone();
                policy.roles[0].authority.delegation = rules.ceiling.delegation.clone();
            }
            let mut domain = authority::Domain::new(rules, limits.authority).expect("read authority rules");
            let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
            authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut facts);
            assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
            // A chat's authority is its role's, one level shallower (jig a2616181).
            config.core.authority = domain;
        }
        limits.notes.scopes = 3 + limits.authority.grants;
        let mut fake_config = forge_world::fake_config();
        fake_config.limits.repositories = 2;
        let mut fake = fake::Domain::new(&fake_config, 71);
        fake::repository(
            &mut fake,
            &fake_config,
            raw::Setup {
                id: 2,
                name: Box::from(&b"org/repo"[..]),
                default: Box::from(&b"main"[..]),
                tree: Box::new([]),
                labels: Box::new([]),
                checks: raw::Checks {
                    contexts: Box::new([Box::from(&b"build"[..])]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: if silent_ci { 1000 } else { 0 },
                    passes,
                    reruns: 0,
                    cue: None,
                },
                protection: None,
                hooked: false,
            },
        );
        fake::grant(&mut fake, b"org/repo", 1, raw::Permission::Admin);
        fake::grant(&mut fake, b"org/repo", 2, raw::Permission::Write);
        fake::grant(&mut fake, b"org/repo", 7, raw::Permission::Read);
        let fake_env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fake_config };
        Self {
            root: engine::Domain::new(config, &limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::with_capacity(engine::max_out(&limits)),
            events: VecDeque::from([engine::Event::Restart]),
            store: Store::new(),
            fake,
            fake_env,
            fake_out: Queue::with_capacity(fake::MAX_OUT),
            pending: BTreeMap::new(),
            forge_calls: 0,
            adopted: Vec::new(),
            signed_in: None,
            assigned: Vec::new(),
            answers: Vec::new(),
            protocol: Protocol::default(),
            immediate: VecDeque::new(),
            period: None,
            fail_job_reads: false,
            slow_brief_reads: false,
            owner_gate,
        }
    }

    fn send(&mut self, event: engine::Event) {
        self.immediate.push_back(event);
        self.flush();
    }

    /// Step every input waiting at once: the one sent, then what the
    /// protocol layer answered at once, as the root's door gives them.
    fn flush(&mut self) {
        while let Some(event) = self.immediate.pop_front() {
            if let Err(event) = engine::try_step(&mut self.root, &self.env, event) {
                self.events.push_back(event);
            }
            engine::release(&mut self.root, &self.env, &mut self.out);
            self.collect();
            self.root.reclaim();
        }
    }

    /// Lose the process while retaining only the store, external forge and
    /// worker's hosted attempt. Outstanding API terminals belong to the old
    /// process; startup has to rediscover the outbox by its durable key.
    fn restart(&mut self, with_read: bool, with_change: bool) {
        assert!(self.store.pending.is_empty(), "the chosen cut resolves store uncertainty first");
        assert!(self.pending.is_empty(), "old API terminals are drained at this cut");
        let mut fresh = if self.owner_gate {
            Self::configured_policy(with_read, with_change, false, 0, None, false, true)
        } else {
            Self::configured(with_read, with_change)
        };
        std::mem::swap(&mut self.root, &mut fresh.root);
        std::mem::swap(&mut self.out, &mut fresh.out);
        self.events.clear();
        self.answers.clear();
        self.adopted.clear();
        self.period = None;
        self.events.push_back(engine::Event::Restart);
        if !self.assigned.is_empty() {
            self.events.push_back(protocol::hello_slots(
                Token::new(7),
                if with_change { 2 } else { 1 },
                self.assigned
                    .iter()
                    .map(|assignment| {
                        protocol::hosting(assignment.task, assignment.attempt, jig_domain_host::Phase::Active)
                    })
                    .collect(),
            ));
        }
    }

    fn external(&mut self, user: u64, op: raw::Op) -> raw::Answer {
        let mut config = self.fake_env.limits;
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        let env = Env { now: self.env.now, wall: self.env.wall, limits: config };
        fake::step(
            &mut self.fake,
            &env,
            fake::Event::Call {
                reply_to: ReplyTo::new(Token::new(10_000)),
                user,
                repository: Box::from(&b"org/repo"[..]),
                op,
            },
            &mut self.fake_out,
        );
        for _ in 0..8 {
            if self.fake.is_due(env.now) {
                fake::fire(&mut self.fake, &env, &mut self.fake_out);
            }
            while let Some(output) = self.fake_out.pop() {
                if let fake::Request::Reply { result, .. } = output {
                    return result.expect("external fake call");
                }
            }
        }
        panic!("external fake call did not return")
    }

    #[expect(clippy::wildcard_enum_match_arm, reason = "the world records only its scripted root deliveries")]
    fn collect(&mut self) {
        for _ in 0..self.out.len() {
            let request = match self.protocol.play(self.out.pop().expect("counted root output")) {
                // The protocol layer answers at once (testing-strategy.md, 4).
                Ok(events) => {
                    self.immediate.extend(events);
                    continue;
                }
                Err(request) => *request,
            };
            match request {
                engine::Request::ToChild(released) => self.events.push_back(engine::Event::Released(released)),
                engine::Request::Store(engine::StoreRequest::Commit { number, writes }) => {
                    self.store.pending.push_back((number, writes));
                }
                engine::Request::Store(engine::StoreRequest::Load { owner, range, after, most, .. }) => {
                    let (rows, next) = self.store.page(&range, after.as_ref(), most);
                    self.events.push_back(engine::Event::Store(engine::StoreInput::Loaded { owner, rows, next }));
                }
                engine::Request::Forge { call, repository, op } => {
                    self.forge_calls += 1;
                    if self.fail_job_reads && matches!(op, client::api::Op::Read(client::api::Read::Job { .. })) {
                        self.events.push_back(engine::Event::Forge(Box::new(
                            temper_engine_domain_forge::Event::Client(
                                temper_engine_domain_forge_client::Event::Answered {
                                    call,
                                    cost: 1,
                                    result: Err(client::api::Error::MissingJob),
                                },
                            ),
                        )));
                        continue;
                    }
                    let name: &[u8] = if repository == forge_world::REPO {
                        b"org/repo"
                    } else if repository == (client::api::Repository { forge: 1, repository: 3 }) {
                        b"org/extra"
                    } else {
                        panic!("unknown test repository {repository:?}");
                    };
                    let slow =
                        self.slow_brief_reads && matches!(op, client::api::Op::Read(client::api::Read::Job { .. }));
                    let raw = translate::op(&op, &self.env.limits.forge.client);
                    self.pending.insert(call, op);
                    let mut limits = self.fake_env.limits;
                    if slow {
                        limits.latency_min = Duration::from_secs(20);
                        limits.latency_max = Duration::from_secs(20);
                    }
                    let fake_env = Env { now: self.fake_env.now, wall: self.fake_env.wall, limits };
                    fake::step(
                        &mut self.fake,
                        &fake_env,
                        fake::Event::Call {
                            reply_to: ReplyTo::new(call),
                            user: 1,
                            repository: Box::from(name),
                            op: raw,
                        },
                        &mut self.fake_out,
                    );
                }
                engine::Request::Worker(engine::WorkerRequest::Deliver(delivery))
                | engine::Request::Party(engine::PartyRequest::Deliver(delivery)) => match delivery {
                    Delivery::WebReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } => {
                        self.signed_in = Some(sign_in);
                    }
                    Delivery::Assigned { assignment, .. } => self.assigned.push(assignment),
                    // The run has its answer once the hub relays it down; the
                    // world renders an answer as its first call's name, and a
                    // replayed call carries that same text.
                    Delivery::Link {
                        down: host::LinkDown::Relayed { answer: host::Relayed::Answer { text, .. }, .. },
                        ..
                    } => {
                        let (_, answer) = self
                            .protocol
                            .answers
                            .iter()
                            .find(|(name, _)| *name == text)
                            .expect("the world rendered every relayed answer");
                        self.answers.push(answer.clone());
                    }
                    Delivery::WebReply { reply: people::Reply::Outcome(outcome), .. }
                        if matches!(outcome, people::Outcome::Adopted { .. }) =>
                    {
                        self.adopted.push(outcome);
                    }
                    _ => {}
                },
                engine::Request::Account(_)
                | engine::Request::Party(_)
                | engine::Request::Store(_)
                | engine::Request::Busy(_)
                | engine::Request::Drained => {}
                engine::Request::Worker(other) => panic!("forge fixture received an unexpected request {other:?}"),
                engine::Request::Stop => panic!("root startup or store failure"),
            }
        }
    }

    fn tick(&mut self) {
        self.env.now = self.env.now.saturating_add(Duration::from_millis(100));
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        self.fake_env.now = self.env.now;
        self.fake_env.wall = self.env.wall;
        if let Some(event) = self.events.pop_front() {
            self.send(event);
        }
        if !self.store.pending.is_empty() {
            let number = self.store.apply();
            self.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        }
        if self.period.is_none() && self.root.ready() {
            self.period = Some(1);
            let period = engine::PartyInput::Period { project: 1, period: 1, budget: 1000, next: None };
            self.send(engine::Event::Party(period));
            // The repository's git identity, an application credential from
            // temper's own source (domain/engine.md, 12).
            self.send(engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) });
        }
        if self.fake.is_due(self.fake_env.now) {
            fake::fire(&mut self.fake, &self.fake_env, &mut self.fake_out);
        }
        for _ in 0..self.fake_out.len() {
            match self.fake_out.pop().expect("counted fake output") {
                fake::Request::Reply { to, result } => {
                    let call = to.into_token();
                    let op = self.pending.remove(&call).expect("one pending forge call");
                    let result = match result {
                        Ok(answer) => Ok(translate::answer(&op, answer, &self.env.limits.forge.client)),
                        Err(error) => {
                            Err(translate::error(error, fake::time(&self.fake_env.limits, self.fake_env.now)))
                        }
                    };
                    self.events.push_back(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Client(
                        temper_engine_domain_forge_client::Event::Answered { call, cost: 1, result },
                    ))));
                }
                fake::Request::Hook { .. } => {}
            }
        }
        temper_engine_domain_world::poll(&mut self.root, &self.env, &mut self.out);
        self.collect();
        self.flush();
        engine::fire(&mut self.root, &self.env);
        engine::release(&mut self.root, &self.env, &mut self.out);
        self.collect();
        self.flush();
        self.root.reclaim();
        self.fake.reclaim();
    }

    fn until(&mut self, target: Until) {
        for _ in 0..200 {
            self.tick();
            let done = match target {
                Until::Ready => self.root.quiescent() && self.events.is_empty() && self.store.pending.is_empty(),
                Until::SignedIn => self.signed_in.is_some(),
                Until::Adopted => !self.adopted.is_empty(),
                Until::Assigned => !self.assigned.is_empty(),
                Until::Subscribed => self
                    .answers
                    .iter()
                    .any(|answer| matches!(answer, temper_engine_domain::CallAnswer::Subscribed { .. })),
                Until::News => {
                    let task = self.assigned.first().expect("assigned subscriber").task;
                    match self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)))) {
                        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => {
                            row.inbox.iter().any(|word| matches!(word.kind, tasks::MessageKind::News { .. }))
                        }
                        _ => false,
                    }
                }
                Until::Read => {
                    self.answers.iter().any(|answer| matches!(answer, temper_engine_domain::CallAnswer::ForgeRead(_)))
                }
                Until::Effect => self
                    .answers
                    .iter()
                    .any(|answer| matches!(answer, temper_engine_domain::CallAnswer::ForgeEffect { .. })),
                Until::Delegated => {
                    self.answers.iter().any(|answer| matches!(answer, temper_engine_domain::CallAnswer::Delegated(_)))
                }
                Until::SecondAssignment => self.assigned.len() >= 2,
            };
            if done {
                return;
            }
        }
        panic!(
            "forge root did not reach {target:?}; answers={:?}, assignments={:?}, task2={:?}, task3={:?}, forge={:?}",
            self.answers,
            self.assigned,
            self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(2)))),
            self.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(3)))),
            self.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
        );
    }

    fn adopt(&mut self) {
        self.adopt_with_checks(true, Box::new([]));
    }

    fn adopt_with_checks(&mut self, ci: bool, checks: Box<[u32]>) {
        self.until(Until::Ready);
        self.send(engine::Event::Party(engine::PartyInput::SignedIn {
            reply_to: ReplyTo::new(Token::new(90)),
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: Box::from(&b"owner"[..]),
                name: Box::from(&b"Owner"[..]),
            },
        }));
        self.until(Until::SignedIn);
        self.send(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: self.signed_in.expect("signed in"),
            key: [91; 16],
            ask: engine::adopt_repository_ask(
                forge_top::Adoption {
                    project: 1,
                    home: true,
                    provider: client::api::Repository {
                        forge: forge_world::REPO.forge,
                        repository: forge_world::REPO.repository,
                    },
                    host: Box::from(&b"forge.example"[..]),
                    owner: Box::from(&b"org"[..]),
                    name: Box::from(&b"repo"[..]),
                    prefix: Box::from(&b"temper/"[..]),
                    role: forge_top::Role::Owned,
                    landing: Box::from(&b"main"[..]),
                    ci,
                    checks,
                },
                0,
            )
            .expect("forge adoption shape"),
        }));
        self.until(Until::Adopted);
        assert!(matches!(self.adopted.as_slice(), [people::Outcome::Adopted { .. }]));
    }
}

fn change_world(
    silent_ci: bool,
    passes: u32,
    approval: Option<engine::Freshness>,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    change_world_with_gate(silent_ci, passes, approval, false)
}

fn change_world_with_gate(
    silent_ci: bool,
    passes: u32,
    approval: Option<engine::Freshness>,
    agent_gate: bool,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    change_world_with_policy(silent_ci, passes, approval, agent_gate, false, false, false)
}

#[expect(clippy::too_many_lines, reason = "the root change fixture builds both producer and review routes")]
#[expect(clippy::fn_params_excessive_bools, reason = "independent CI, gate and policy settings select test worlds")]
fn change_world_with_policy(
    silent_ci: bool,
    passes: u32,
    approval: Option<engine::Freshness>,
    agent_gate: bool,
    no_ci: bool,
    owner_gate: bool,
    tracked: bool,
) -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_policy(true, true, silent_ci, passes, approval, agent_gate, owner_gate);
    if no_ci {
        world.adopt_with_checks(false, Box::new([1]));
    } else {
        world.adopt();
    }
    if owner_gate {
        let branch = people::Pattern {
            segments: Box::new([
                Box::from(&b"forge"[..]),
                Box::from(&b"forge.example"[..]),
                Box::from(&b"org"[..]),
                Box::from(&b"repo"[..]),
                Box::from(&b"branch"[..]),
            ]),
            last: people::Last::Exact(Box::from(&b"main"[..])),
        };
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: ReplyTo::new(Token::new(900)),
            sign_in: world.signed_in.expect("owner session"),
            key: [90; 16],
            ask: people::Ask::ChangePolicy {
                project: 1,
                change: people::PolicyChange::Requirements {
                    requirements: Box::new([
                        people::Requirement {
                            connector: 0,
                            kind: 4,
                            pattern: branch.clone(),
                            judge: people::Judge { connector: 0, requirement: 1, parameters: 0x8000_0000 },
                            guard: people::Guard::Guarded,
                            must_be_guarded: true,
                        },
                        people::Requirement {
                            connector: 0,
                            kind: 4,
                            pattern: branch.clone(),
                            judge: people::Judge { connector: 0, requirement: 2, parameters: 0x8000_0001 },
                            guard: people::Guard::Observed { freshness: Duration::ZERO },
                            must_be_guarded: false,
                        },
                        people::Requirement {
                            connector: 0,
                            kind: 4,
                            pattern: branch,
                            judge: people::Judge { connector: 0, requirement: 3, parameters: 0x8000_0002 },
                            guard: people::Guard::Guarded,
                            must_be_guarded: true,
                        },
                    ]),
                },
            },
        }));
        world.until(Until::Ready);
        assert!(
            world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::People(people::Key::Policy(1)))),
            "owner policy committed"
        );
        world.restart(true, true);
        world.until(Until::Ready);
    }
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = if tracked {
        people::Ask::SetGoal {
            project: 1,
            spec: Box::from(&b"make the fix"[..]),
            executor: people::GoalExecutor::Charter(1),
            budget: 100,
            priority: 3,
            deadline: None,
        }
    } else {
        people::Ask::StartChat { project: 1, words: Box::from(&b"make the fix"[..]) }
    };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask,
    }));
    world.until(Until::Assigned);
    let chat = world.assigned[0].clone();
    let grant = tasks::Grant {
        connector: 0,
        kind: 2,
        pattern: tasks::Pattern {
            segments: Box::new([Box::from(&b"forge"[..]), Box::from(&b"forge.example"[..]), Box::from(&b"org"[..])]),
            last: tasks::Last::Open(Box::from(&b"repo"[..])),
        },
    };
    let child_authority = tasks::Authority {
        tools: tasks::Tools(1023),
        grants: [3, 4, 10, 11, 12].map(|kind| tasks::Grant { kind, ..grant.clone() }).into(),
        delegation: tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 4, depth: 1 },
        // Each run's spend must fund its finishing reserve, six with the
        // world's charter (jig a020bf1c): room for five attempts of five.
        budget: tasks::Budget { spend: 40, deadline: None },
        notes: tasks::Scopes(0),
        note_resources: Box::new([]),
    };
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Delegate {
                batch: Box::new([engine::Delegate {
                    executor: tasks::Executor::Procedure { connector: 0, code: 2 },
                    spec: tasks::Spec {
                        words: Box::from(&b"Small fix"[..]),
                        parameters: Box::new([
                            tasks::Parameter::Resource {
                                name: 1,
                                connector: 0,
                                resource: u64::from(forge_world::REPO.repository),
                            },
                            tasks::Parameter::Bytes { name: 2, value: Box::from(&b"main"[..]) },
                            tasks::Parameter::Bytes { name: 6, value: Box::from(&b"file"[..]) },
                        ]),
                        inputs: Box::new([]),
                    },
                    contract: tasks::Contract::Change { connector: 0, kind: 1, words: 32 },
                    authority: child_authority,
                    symbolic_grants: Box::new([tasks::Grant {
                        connector: 0,
                        kind: 2,
                        pattern: tasks::Pattern {
                            segments: Box::new([
                                Box::from(&b"forge"[..]),
                                Box::from(&b"forge.example"[..]),
                                Box::from(&b"org"[..]),
                                Box::from(&b"repo"[..]),
                                Box::from(&b"branch"[..]),
                                Box::from(&b"temper"[..]),
                                chat.task.to_string().into_bytes().into_boxed_slice(),
                            ]),
                            last: tasks::Last::Open(Box::from(&b"c"[..])),
                        },
                    }]),
                    dependencies: Box::new([]),
                    wake: tasks::WakePolicy::DEFAULT,
                }]),
            },
        },
    );
    world.send(event);
    world.until(Until::Delegated);
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    let change = world
        .answers
        .iter()
        .find_map(|answer| match answer {
            temper_engine_domain::CallAnswer::Delegated(numbers) => numbers.first().copied(),
            _ => None,
        })
        .expect("change task number");
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
                if task.number == producer.task && task.authority.grants.iter().any(|grant|
                    grant.connector == 0 && grant.kind == 2
                        && grant.pattern.segments.last().map(AsRef::as_ref)
                            == Some(chat.task.to_string().as_bytes())
                        && matches!(&grant.pattern.last, tasks::Last::Exact(prefix)
                            if prefix.as_ref() == format!("c{change}").as_bytes()))
        )),
        "producer inherits the change task's numbered branch grant"
    );
    let branch = producer.workspace.repositories[0].push.clone().expect("writable change branch");
    (world, chat, producer, branch)
}

#[test]
fn a_pushed_change_is_recovered_before_its_worker_reports_the_head() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let pushed = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("worker push reached the forge");
    assert!(world.store.pending.is_empty(), "push precedes the worker's answer decision");
    world.restart(true, true);
    for _ in 0..200 {
        world.tick();
        if world.root.ready() {
            break;
        }
    }
    assert!(world.root.ready(), "restored root did not become ready");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
        )),
        "recovered push did not land"
    );
    let raw::Answer::Commit(found) = world.external(1, raw::Op::Read(raw::Read::Branch { branch: branch.clone() }))
    else {
        panic!("pushed branch remains readable")
    };
    assert_eq!(found, pushed, "the recovered procedure used the pushed head");
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) if task.number == chat.task
        )),
        "the chat still owns the change result"
    );
}

#[test]
fn a_change_whose_ci_never_reports_is_stalled_and_held() {
    let (mut world, chat, producer, branch) = change_world(true, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed its branch");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..450 {
        world.tick();
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Held {
                    why: temper_engine_domain_forge_change::Hold::Stalled, .. }))
        )),
        "CI silence did not stall the change: {:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    assert!(
        !world.store.rows.values().any(|stored| matches!(stored,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) if row.requester == tasks::Party::Task(chat.task)
        )),
        "stalled change must not report a landing"
    );
}

#[test]
fn an_agent_review_gate_runs_at_the_head_and_its_approval_lands_the_change() {
    check_gate_landing(false, false);
}

#[test]
fn a_change_to_a_repository_without_ci_lands_on_its_checks() {
    check_gate_landing(true, false);
}

#[test]
fn an_owner_adds_a_branch_requirement_and_later_changes_wait_for_it() {
    check_gate_landing(false, true);
}

fn check_gate_landing(no_ci: bool, owner_gate: bool) {
    let (mut world, _chat, producer, branch) =
        change_world_with_policy(no_ci, 1000, None, !owner_gate, no_ci, owner_gate, false);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"review this diff", 1)
        .expect("producer pushed review head");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let gate = world.assigned.get(2).expect("agent gate assigned").clone();
    if owner_gate {
        assert!(
            world.store.rows.values().all(|stored| !matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )),
            "the owner-added review holds landing until its verdict"
        );
    }
    assert!(
        gate.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"review this diff".len()).any(|part| part == b"review this diff"))),
        "sections={:?}",
        gate.sections
    );
    assert_eq!(gate.workspace.repositories[0].start, engine::ForgeStart::Branch(branch));
    let event = world.protocol.answer(
        Token::new(7),
        gate.task,
        gate.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: Box::from(&b"approved"[..]) },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                && change.verdicts.iter().any(|report| Some(report.head) == change.change.last_head
                    && report.status == temper_engine_domain_forge_change::Status::Passed))
    )));
}

#[test]
fn a_review_asking_for_changes_is_repaired_with_its_remarks_and_reviewed_again() {
    check_gate_repair(false);
}

#[test]
fn a_failing_check_is_repaired_and_checked_again() {
    check_gate_repair(true);
}

#[expect(clippy::too_many_lines, reason = "the check repair story spans both gate heads and their worker reports")]
fn check_gate_repair(no_ci: bool) {
    let (mut world, _chat, producer, branch) = change_world_with_policy(no_ci, 1000, None, true, no_ci, false, false);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first version", 1)
        .expect("first head pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let first = world.assigned.get(2).expect("first review assigned").clone();
    let event = world.protocol.answer(
        Token::new(7),
        first.task,
        first.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict {
                code: 2,
                fields: Box::new([]),
                words: Box::from(&b"Please fix the unsafe edge"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 4 {
            break;
        }
    }
    let repair = world.assigned.get(3).expect("review repair assigned").clone();
    assert!(
        repair.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Reviews)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"Please fix the unsafe edge".len()).any(|part| part == b"Please fix the unsafe edge"))),
        "review remarks in repair brief: {:?}",
        repair.sections
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"safe revision", 1)
        .expect("repair pushed second head");
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"repaired"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 5 {
            break;
        }
    }
    let second = world.assigned.get(4).expect("second review assigned").clone();
    assert_ne!(first.task, second.task);
    let event = world.protocol.answer(
        Token::new(7),
        second.task,
        second.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict {
                code: 1,
                fields: Box::new([]),
                words: Box::from(&b"Approved revision"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })))
        }) {
            break;
        }
    }
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                && change.change.repairs == 1))));
}

#[test]
fn a_failed_ci_repair_brief_contains_the_bounded_job_log() {
    let (mut world, _chat, producer, branch) = change_world(false, 0, None);
    world.external(
        1,
        raw::Op::Write(raw::Write::Status { commit: 1, context: Box::from(&b"build"[..]), state: raw::Check::Passed }),
    );
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first", 1)
        .expect("producer pushed its branch");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Failed,
        }),
    );
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    assert!(
        world.assigned.len() >= 3,
        "failed CI did not assign repair: task2={:?}; task3={:?}; task4={:?}; forge={:?}",
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(2)))),
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(3)))),
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(4)))),
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    let repair = world.assigned[2].clone();
    assert!(
        matches!(repair.workspace.repositories[0].start, engine::ForgeStart::Branch(ref name) if name.as_ref() == branch.as_ref())
    );
    assert!(
        repair.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Ci)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"failing check build 3".len()).any(|part| part == b"failing check build 3")
                && words.windows(b"[job log truncated]".len()).any(|part| part == b"[job log truncated]"))),
        "CI sections: {:?}",
        repair.sections
    );
}

#[test]
fn an_unreadable_failed_job_log_is_named_and_repair_still_runs() {
    let (mut world, _chat, producer, branch) = change_world(false, 0, None);
    world.fail_job_reads = true;
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first", 1)
        .expect("producer pushed its branch");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Failed,
        }),
    );
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let repair = world.assigned.get(2).expect("repair assigned despite missing job log");
    assert!(
        repair.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Ci)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"[job log could not be read]".len()).any(|part| part == b"[job log could not be read]")
                && words.windows(b"Description:".len()).any(|part| part == b"Description:")
                && words.windows(b"Link:".len()).any(|part| part == b"Link:"))),
        "CI sections: {:?}",
        repair.sections
    );
}

fn failed_ci_with_delayed_brief() -> (World, u64, u64) {
    let (mut world, chat, producer, branch) = change_world(false, 0, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first", 1)
        .expect("producer pushed its branch");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Failed,
        }),
    );
    world.slow_brief_reads = true;
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    (world, chat.task, producer.task)
}

#[test]
fn a_required_section_missing_sends_the_task_back_to_due_without_spending_a_try() {
    let (mut world, chat, producer) = failed_ci_with_delayed_brief();
    let mut repair = None;
    for _ in 0..150 {
        world.tick();
        for stored in world.store.rows.values() {
            let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = stored else { continue };
            if row.number != chat
                && row.number != producer
                && matches!(row.executor, tasks::Executor::Agent { .. })
                && row.refusals > 0
            {
                repair = Some(row.clone());
                break;
            }
        }
        if repair.is_some() {
            break;
        }
    }
    let repair = repair.unwrap_or_else(|| {
        panic!(
            "required forge section timed out while preparing: tasks={:?}; pending={:?}; assigned={:?}",
            world
                .store
                .rows
                .values()
                .filter(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
                .collect::<Vec<_>>(),
            world.pending,
            world.assigned,
        )
    });
    assert_eq!(repair.attempt, 0, "no claim was made");
    assert_eq!(repair.tries, tasks::Tries::NONE, "preparation consumed no try");
    assert_eq!(world.assigned.len(), 2, "repair was never assigned");
}

#[test]
fn an_amendment_while_gathering_discards_the_old_brief_without_claiming() {
    let (mut world, chat, producer) = failed_ci_with_delayed_brief();
    let mut repair = None;
    for _ in 0..150 {
        world.tick();
        for stored in world.store.rows.values() {
            let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = stored else { continue };
            if row.number != chat
                && row.number != producer
                && matches!(row.executor, tasks::Executor::Agent { .. })
                && row.phase == tasks::Phase::Active(tasks::Active::Preparing)
                && world.pending.values().any(|op| matches!(op, client::api::Op::Read(client::api::Read::Job { .. })))
            {
                repair = Some(row.number);
                break;
            }
        }
        if repair.is_some() {
            break;
        }
    }
    let repair = repair.unwrap_or_else(|| {
        panic!(
            "repair gathering its required section: phases={:?}; pending={:?}",
            world
                .store
                .rows
                .values()
                .filter_map(|stored| match stored {
                    Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) =>
                        Some((row.number, row.phase.clone())),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            world.pending
        )
    });
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(790)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [79; 16],
        ask: people::Ask::Amend {
            project: 1,
            task: repair,
            // The person names the specification revision they amend (jig 00de0b8c).
            revision: spec_revision(&world, repair),
            release: None,
            amendment: people::Amendment {
                addressed: None,
                instructions: None,
                spec: Some(people::Spec {
                    words: b"revised repair".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                }),
                wake: None,
                dependencies: None,
                authority: None,
                reason: b"new failure detail".as_slice().into(),
            },
        },
    }));
    for _ in 0..40 {
        world.tick();
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
            world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(repair))))
        else {
            continue;
        };
        if row.refusals > 0 && row.last_message == 2 {
            assert_eq!(row.spec.words.as_ref(), b"revised repair");
            assert_eq!(row.attempt, 0, "old preparation was not claimed");
            assert_eq!(row.tries, tasks::Tries::NONE);
            assert_eq!(world.assigned.len(), 2);
            for _ in 0..250 {
                world.tick();
            }
            assert_eq!(world.assigned.len(), 2, "an abandoned brief cannot assign after its delayed read");
            return;
        }
    }
    panic!(
        "amended repair did not abandon its pending brief: {:?}",
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(repair))))
    );
}

#[test]
fn a_conflicting_update_is_resolved_from_a_merge_in_progress() {
    let (mut world, _chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"branch change", 1)
        .expect("producer pushed branch");
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base change", 1)
        .expect("another change moved the base");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    assert!(
        world.assigned.len() >= 3,
        "conflict did not assign a resolver: task2={:?}; forge={:?}",
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(2)))),
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    let resolver = world.assigned[2].clone();
    assert!(matches!(resolver.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: ref source, base: expected }
            if source.as_ref() == branch.as_ref() && expected == translate::commit(base)));
    assert!(
        resolver.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"What landed in the base".len()).any(|part| part == b"What landed in the base")
                && words.windows(b"Conflicting file".len()).any(|part| part == b"Conflicting file"))),
        "resolution sections: {:?}",
        resolver.sections
    );
    let old = world.fake.branch(b"org/repo", &branch).expect("existing branch head");
    let merge = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        old,
        base,
        Box::new([raw::File { path: Box::from(&b"file"[..]), content: Box::from(&b"resolved"[..]) }]),
        b"Resolve conflict",
    )
    .expect("resolved merge commit");
    assert!(matches!(
        world.external(1, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: merge, expected: Some(old) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    ));
    let event = world.protocol.answer(
        Token::new(7),
        resolver.task,
        resolver.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"resolved"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                    && change.change.resolutions == 1)
        )),
        "resolved change did not land"
    );
}

#[test]
#[expect(clippy::too_many_lines, reason = "the story checks both clean-update carryover and a new review after repair")]
fn an_approval_carries_over_a_clean_update_and_is_asked_again_after_a_repair() {
    let (mut world, _chat, producer, branch) = change_world(false, 1000, Some(engine::Freshness::Clean));
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let first = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fix", 1)
        .expect("producer pushed branch");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..100 {
        world.tick();
        if world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change) if change.pull.is_some())
        )) { break; }
    }
    let pull = world
        .store
        .rows
        .values()
        .find_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(change) => change.pull,
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("opened pull before review");
    assert!(
        !world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
        )),
        "approval requirement holds the first head"
    );
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: Box::from(&b"Approved"[..]),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    let moved = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"unrelated", 1)
        .expect("base advanced cleanly");
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: client::api::Hint {
            repository: forge_world::REPO,
            change: client::api::Change::Branch(Box::from(&b"main"[..])),
            key: None,
        },
    })));
    for _ in 0..240 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                    && change.change.updates == 1 && change.change.last_head != Some(translate::commit(first)))
        )),
        "old approval did not carry across the clean update; base={moved:?}; forge={:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );

    let (mut world, _chat, producer, branch) = change_world(false, 0, Some(engine::Freshness::Clean));
    world.external(
        1,
        raw::Op::Write(raw::Write::Status { commit: 1, context: Box::from(&b"build"[..]), state: raw::Check::Passed }),
    );
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let first = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"broken", 1)
        .expect("producer pushed branch");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: first,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Failed,
        }),
    );
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let repair = world.assigned.get(2).expect("CI failure assigned repair").clone();
    let pull = world
        .store
        .rows
        .values()
        .find_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(change) => change.pull,
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("opened pull for repair");
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: Box::from(&b"Old head approved"[..]),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    let repaired = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"repaired", 1)
        .expect("repair pushed new head");
    for _ in 0..5 {
        world.tick();
    }
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: repaired,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Passed,
        }),
    );
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"repaired"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..100 {
        world.tick();
    }
    assert!(
        !world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
        )),
        "old review must not approve repaired head"
    );
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: Box::from(&b"Repaired head approved"[..]),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: client::api::Hint { repository: forge_world::REPO, change: client::api::Change::Item(pull), key: None },
    })));
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                    && change.change.repairs == 1)
        )),
        "new review must approve repaired head; forge={:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
}

#[test]
fn a_worker_frozen_past_its_grace_resumes_with_a_push_in_hand_and_lands_nothing_twice() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let pushed = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"already pushed", 1)
        .expect("worker pushed before freezing");
    world.send(protocol::lost(Token::new(7)));
    world.send(protocol::hello_slots(Token::new(8), 2, Box::new([])));
    for _ in 0..150 {
        world.tick();
        if world.assigned.iter().any(|run| run.task == producer.task && run.attempt != producer.attempt) {
            break;
        }
    }
    let resumed = world
        .assigned
        .iter()
        .find(|run| run.task == producer.task && run.attempt != producer.attempt)
        .expect("lost producer reassigned")
        .clone();
    assert_eq!(resumed.task, producer.task);
    assert_ne!(resumed.attempt, producer.attempt);
    assert!(
        matches!(resumed.workspace.repositories[0].start, engine::ForgeStart::Branch(ref name)
        if name.as_ref() == branch.as_ref()),
        "resumed worker sees its pushed branch: {:?}",
        resumed.workspace.repositories[0].start
    );
    assert_eq!(world.fake.branch(b"org/repo", &branch), Some(pushed));
    let event = world.protocol.answer(
        Token::new(8),
        resumed.task,
        resumed.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
    }
    let landed = world
        .store
        .rows
        .values()
        .filter(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        })
        .count();
    assert_eq!(landed, 1, "one landing after a frozen worker resumes");
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|stored| matches!(stored,
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) if row.requester == tasks::Party::Task(chat.task)
                    && matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. })))
            ))
            .count(),
        1,
        "one result after recovery"
    );
}

#[test]
#[expect(clippy::too_many_lines, reason = "the base repair story drives two consecutive change landings")]
fn a_broken_landing_branch_gets_one_deployment_repair_before_waiting_changes_land() {
    let (mut world, _chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: 1,
                context: Box::from(&b"build"[..]),
                state: raw::Check::Failed,
            })
        ),
        raw::Answer::Done
    ));
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fix", 1)
        .expect("original change pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let repair = world
        .assigned
        .get(2)
        .unwrap_or_else(|| {
            panic!(
                "base failure assigned deployment repair producer; task4={:?}; task5={:?}; assignments={:?}",
                world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(4)))),
                world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(5)))),
                world.assigned
            )
        })
        .clone();
    let repair_branch = repair.workspace.repositories[0].push.as_ref().expect("repair branch").clone();
    assert_ne!(repair_branch, branch);
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if change.base_repair && change.queue_repair.is_none())
    )));
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: repair_branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &repair_branch, b"base", b"fixed", 1)
        .expect("base repair pushed");
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"repaired"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..1000 {
        world.tick();
        if world
            .store
            .rows
            .values()
            .filter(|stored| {
                matches!(stored,
                    Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                        if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
                )
            })
            .count()
            >= 2
        {
            break;
        }
    }
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|stored| matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            ))
            .count(),
        2,
        "deployment repair and waiting change land; forge={:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the story selects persisted people rows")]
fn a_repository_adopted_seeds_its_collaborators_into_roles() {
    let mut world = World::new();
    world.adopt();
    let mut roles = None;
    let mut people = BTreeMap::new();
    for row in world.store.rows.values() {
        match row {
            Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Person { number, identity })) => {
                let subject: [u8; 8] =
                    identity.key.subject.as_ref().try_into().expect("forge subject is a user number");
                people.insert(u64::from_be_bytes(subject), *number);
            }
            Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Roles {
                project: 1,
                holdings,
            })) => {
                roles = Some(holdings.clone());
            }
            _ => {}
        }
    }
    let roles = roles.expect("durable project roles");
    assert!(roles.contains(&people::Holding { person: people[&7], role: people::Role::Owner }));
    assert!(roles.contains(&people::Holding { person: people[&1], role: people::Role::Maintainer }));
    assert!(roles.contains(&people::Holding { person: people[&2], role: people::Role::Member }));
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the story selects persisted people rows")]
fn policy_maps_connector_permissions_to_seeded_roles_after_restart() {
    let mut world = World::new();
    world.until(Until::Ready);
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        reply_to: ReplyTo::new(Token::new(90)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: Box::from(&b"owner"[..]),
            name: Box::from(&b"Owner"[..]),
        },
    }));
    world.until(Until::SignedIn);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [94; 16],
        ask: people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::Permissions {
                mappings: Box::new([
                    people::PermissionRole { connector: 0, permission: 3, role: 3 },
                    people::PermissionRole { connector: 0, permission: 2, role: 1 },
                ]),
            },
        },
    }));
    world.until(Until::Ready);
    assert!(
        world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::People(people::Key::Policy(1)))),
        "permission policy committed"
    );
    world.restart(false, false);
    world.signed_in = None;
    world.adopt();
    let mut roles = None;
    let mut people = BTreeMap::new();
    for row in world.store.rows.values() {
        match row {
            Record::Core(jig_domain_core::Record::People(people::Stored::Person { number, identity })) => {
                let subject: [u8; 8] =
                    identity.key.subject.as_ref().try_into().expect("forge subject is a user number");
                people.insert(u64::from_be_bytes(subject), *number);
            }
            Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: 1, holdings })) => {
                roles = Some(holdings.clone());
            }
            _ => {}
        }
    }
    let roles = roles.expect("durable project roles");
    assert!(roles.contains(&people::Holding { person: people[&7], role: people::Role::Owner }));
    assert!(roles.contains(&people::Holding { person: people[&1], role: people::Role::Observer }));
    assert!(roles.contains(&people::Holding { person: people[&2], role: people::Role::Maintainer }));
}

#[test]
fn an_adoption_sent_twice_across_a_restart_is_made_once() {
    let mut world = World::new();
    world.adopt();
    let first = world.adopted[0];
    let calls = world.forge_calls;
    world.restart(false, false);
    world.until(Until::Ready);
    world.signed_in = None;
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        reply_to: ReplyTo::new(Token::new(92)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: Box::from(&b"owner"[..]),
            name: Box::from(&b"Owner"[..]),
        },
    }));
    world.until(Until::SignedIn);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(93)),
        sign_in: world.signed_in.expect("owner signed in again"),
        key: [91; 16],
        ask: engine::adopt_repository_ask(
            forge_top::Adoption {
                project: 1,
                home: true,
                provider: client::api::Repository {
                    forge: forge_world::REPO.forge,
                    repository: forge_world::REPO.repository,
                },
                host: Box::from(&b"forge.example"[..]),
                owner: Box::from(&b"org"[..]),
                name: Box::from(&b"repo"[..]),
                prefix: Box::from(&b"temper/"[..]),
                role: forge_top::Role::Owned,
                landing: Box::from(&b"main"[..]),
                ci: true,
                checks: Box::new([]),
            },
            0,
        )
        .expect("forge adoption shape"),
    }));
    world.until(Until::Adopted);
    assert_eq!(world.adopted, [first], "the committed answer replays after restart");
    assert_eq!(world.forge_calls, calls, "a replay never asks the forge to adopt again");
}

#[test]
fn a_saved_repository_tag_becomes_a_concrete_checkout_in_the_next_attempt() {
    let mut world = World::configured(true, false);
    world.adopt();
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            id: 3,
            name: Box::from(&b"org/extra"[..]),
            default: Box::from(&b"main"[..]),
            tree: Box::new([]),
            labels: Box::new([]),
            checks: raw::Checks {
                contexts: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
            protection: None,
            hooked: false,
        },
    );
    fake::grant(&mut world.fake, b"org/extra", 1, raw::Permission::Admin);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(98)),
        sign_in: world.signed_in.expect("owner session"),
        key: [98; 16],
        ask: engine::adopt_repository_ask(
            forge_top::Adoption {
                project: 1,
                home: false,
                provider: client::api::Repository { forge: 1, repository: 3 },
                host: Box::from(&b"forge.example"[..]),
                owner: Box::from(&b"org"[..]),
                name: Box::from(&b"extra"[..]),
                prefix: Box::from(&b"temper/"[..]),
                role: forge_top::Role::Owned,
                landing: Box::from(&b"main"[..]),
                ci: true,
                checks: Box::new([]),
            },
            0,
        )
        .expect("forge adoption shape"),
    }));
    for _ in 0..100 {
        world.tick();
        if world.adopted.len() == 2 {
            break;
        }
    }
    assert!(
        matches!(world.adopted.get(1), Some(people::Outcome::Adopted { .. })),
        "second repository adopted: {:?}",
        world.adopted
    );
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(99)),
        sign_in: world.signed_in.expect("owner session"),
        key: [99; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"Work in two repositories"[..]) },
    }));
    world.until(Until::Assigned);
    let first = world.assigned[0].clone();
    let event = world.protocol.answer(
        Token::new(7),
        first.task,
        first.attempt,
        0,
        tasks::End::Parked,
        engine::Parts { saved: Some(Box::new([3])), pushed: Box::new([]), preparation: None },
    );
    world.send(event);
    for _ in 0..30 {
        world.tick();
    }
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(100)),
        sign_in: world.signed_in.expect("owner session"),
        key: [100; 16],
        ask: people::Ask::Say { project: 1, task: first.task, words: Box::from(&b"Continue"[..]), release: false },
    }));
    for _ in 0..150 {
        world.tick();
        if world.assigned.len() >= 2 {
            break;
        }
    }
    let second = world.assigned.get(1).expect("saved task reassigned");
    assert_eq!(second.saved.as_ref(), [3]);
    assert!(second.workspace.repositories.iter().any(|repository| repository.tag == 3
        && matches!(repository.start, engine::ForgeStart::Saved(ref branch)
            if branch.as_ref() == b"temper/1/s1")));
}

#[test]
fn a_goal_ending_while_its_issue_is_opening_eventually_closes_that_issue() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner session"),
        key: [94; 16],
        ask: people::Ask::SetGoal {
            project: 1,
            spec: Box::from(&b"Finish the goal"[..]),
            executor: people::GoalExecutor::Charter(1),
            budget: 50,
            priority: 3,
            deadline: None,
        },
    }));
    world.until(Until::Assigned);
    let goal = world.assigned[0].clone();
    let event = world.protocol.answer(
        Token::new(7),
        goal.task,
        goal.attempt,
        1,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: Box::from(&b"done"[..]) },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..300 {
        world.tick();
        if world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Issue(issue) if issue.state.closed)
        )) { break; }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Issue(issue) if issue.state.closed)
        )),
        "ended goal's issue did not close"
    );
}

#[test]
fn a_goal_subscription_receives_the_connectors_landing_news() {
    let mut world = World::new();
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"watch the landing"[..]) },
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::SubscribeForge {
                topic: forge_top::Topic::Landings { repository: forge_world::REPO, branch: Box::from(&b"main"[..]) },
                own_change: None,
                paths: Box::new([Box::from(&b"file"[..])]),
            },
        },
    );
    world.send(event);
    world.until(Until::Subscribed);
    for _ in 0..10 {
        world.tick();
    }
    let result = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"changed", 1);
    assert!(result.is_ok(), "external landing in the fake forge");
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: client::api::Hint {
            repository: forge_world::REPO,
            change: client::api::Change::Branch(Box::from(&b"main"[..])),
            key: None,
        },
    })));
    world.until(Until::News);
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task))))
    else {
        panic!("subscriber task remains live")
    };
    assert!(
        row.inbox
            .iter()
            .any(|word| matches!(word.kind, tasks::MessageKind::News { class: tasks::NewsClass::Wakes, .. }))
    );
}

fn subscribe_chat(world: &mut World, topic: forge_top::Topic, key: u8) -> u64 {
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"watch the forge"[..]) },
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        u64::from(key) + 1,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::SubscribeForge { topic, own_change: None, paths: Box::new([]) },
        },
    );
    world.send(event);
    world.until(Until::Subscribed);
    assignment.task
}

fn news_count(world: &World, task: u64) -> usize {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => {
            let mut hits = 0;
            for word in &row.inbox {
                if matches!(word.kind, tasks::MessageKind::News { .. }) {
                    hits += usize::try_from(word.hits).expect("bounded news hits");
                }
            }
            hits
        }
        Some(_) | None => 0,
    }
}

fn await_new_forge_news(world: &mut World, task: u64, before: usize) {
    for _ in 0..120 {
        world.tick();
        if news_count(world, task) > before {
            return;
        }
    }
    panic!("connector news did not reach the subscribed root task");
}

#[test]
fn a_pull_subscription_receives_the_connectors_state_news() {
    let mut world = World::new();
    world.adopt();
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(&b"work"[..]), commit: 1 })),
        raw::Answer::Branch(_)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"work", b"file", b"change", 1).expect("head");
    let raw::Answer::Created(pull) = world.external(
        2,
        raw::Op::Write(raw::Write::OpenPull {
            title: Box::from(&b"change"[..]),
            body: Box::new([]),
            head: Box::from(&b"work"[..]),
            base: Box::from(&b"main"[..]),
        }),
    ) else {
        panic!("pull opened")
    };
    let task = subscribe_chat(&mut world, forge_top::Topic::Pull { repository: forge_world::REPO, number: pull }, 94);
    for _ in 0..10 {
        world.tick();
    }
    let before = news_count(&world, task);
    assert!(matches!(world.external(2, raw::Op::Write(raw::Write::Close { number: pull })), raw::Answer::Done));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: client::api::Hint { repository: forge_world::REPO, change: client::api::Change::Item(pull), key: None },
    })));
    await_new_forge_news(&mut world, task, before);
}

fn ci_subscription_story(restart: bool) {
    let mut world = World::new();
    world.adopt();
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(&b"work"[..]), commit: 1 })),
        raw::Answer::Branch(_)
    ));
    let head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"work", b"file", b"change", 1).expect("head");
    let raw::Answer::Created(_pull) = world.external(
        2,
        raw::Op::Write(raw::Write::OpenPull {
            title: Box::from(&b"change"[..]),
            body: Box::new([]),
            head: Box::from(&b"work"[..]),
            base: Box::from(&b"main"[..]),
        }),
    ) else {
        panic!("pull opened")
    };
    let task = subscribe_chat(
        &mut world,
        forge_top::Topic::Ci { repository: forge_world::REPO, head: translate::commit(head) },
        96,
    );
    for _ in 0..10 {
        world.tick();
    }
    if restart {
        world.restart(false, false);
        for _ in 0..20 {
            world.tick();
        }
    }
    let before = news_count(&world, task);
    assert!(matches!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: Box::from(&b"build"[..]),
                state: raw::Check::Failed
            })
        ),
        raw::Answer::Done
    ));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: client::api::Hint {
            repository: forge_world::REPO,
            change: client::api::Change::Commit(translate::commit(head)),
            key: None,
        },
    })));
    await_new_forge_news(&mut world, task, before);
}

#[test]
fn a_ci_subscription_receives_the_connectors_verdict_news() {
    ci_subscription_story(false);
}

#[test]
fn a_durable_ci_subscription_receives_verdict_news_after_a_cold_restart() {
    ci_subscription_story(true);
}

#[test]
fn a_participation_subscription_receives_the_connectors_comment_news() {
    let mut world = World::new();
    world.adopt();
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(&b"work"[..]), commit: 1 })),
        raw::Answer::Branch(_)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"work", b"file", b"change", 1).expect("head");
    let raw::Answer::Created(pull) = world.external(
        2,
        raw::Op::Write(raw::Write::OpenPull {
            title: Box::from(&b"change"[..]),
            body: Box::new([]),
            head: Box::from(&b"work"[..]),
            base: Box::from(&b"main"[..]),
        }),
    ) else {
        panic!("pull opened")
    };
    let task =
        subscribe_chat(&mut world, forge_top::Topic::Participation { repository: forge_world::REPO, number: pull }, 98);
    for _ in 0..10 {
        world.tick();
    }
    let before = news_count(&world, task);
    assert!(matches!(
        world.external(2, raw::Op::Write(raw::Write::Comment { number: pull, body: Box::from(&b"please check"[..]) })),
        raw::Answer::Commented(_)
    ));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: client::api::Hint { repository: forge_world::REPO, change: client::api::Change::Item(pull), key: None },
    })));
    await_new_forge_news(&mut world, task, before);
}

#[test]
fn an_authorized_forge_read_returns_a_bounded_typed_answer() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"read forge"[..]) },
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::ReadForge { repository: forge_world::REPO, read: client::api::Read::Branches },
        },
    );
    world.send(event);
    decoded(&mut world);
    assert!(world.store.pending.is_empty(), "an agent read crosses the door without a commit");
    world.until(Until::Read);
    assert!(
        matches!(world.answers.as_slice(), [temper_engine_domain::CallAnswer::ForgeRead(result)] if result.is_ok())
    );
    world.answers.clear();
    let file_head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"contents", 1)
        .expect("file head");
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        94,
        engine::Call {
            completion: 2,
            position: 1,
            tool: engine::Tool::ReadForge {
                repository: forge_world::REPO,
                read: client::api::Read::File {
                    head: translate::commit(file_head),
                    path: Box::from(&b"file"[..]),
                    max_bytes: 2,
                },
            },
        },
    );
    world.send(event);
    decoded(&mut world);
    assert!(world.store.pending.is_empty(), "an agent read crosses the door without a commit");
    world.until(Until::Read);
    assert!(matches!(world.answers.as_slice(), [temper_engine_domain::CallAnswer::ForgeRead(result)]
        if matches!(result.as_ref(), Ok(client::api::Answer::File { head, bytes, truncated: true, .. })
            if *head == translate::commit(file_head) && bytes.len() == 2)));
    for completion in [1, 2] {
        let key = temper_engine_domain::CallKey {
            task: assignment.task,
            attempt: assignment.attempt,
            completion,
            position: 1,
        };
        assert!(
            !world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))),
            "read answers are never durable call records"
        );
    }
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the story prints an unexpected typed effect answer")]
fn a_named_forge_effect_is_committed_before_its_write_and_made_once() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"create an issue"[..]) },
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::EffectForge {
                repository: forge_world::REPO,
                resource: forge_top::What::Repository,
                write: Box::new(client::api::Write::CreateIssue {
                    key: Box::new([]),
                    title: Box::from(&b"Small finding"[..]),
                    body: Box::from(&b"Fix this"[..]),
                }),
            },
        },
    );
    world.send(event);
    world.until(Until::Effect);
    let entry = match world.answers[0] {
        temper_engine_domain::CallAnswer::ForgeEffect {
            entry, outcome: Some(client::Outcome::Made { .. }), ..
        } => entry,
        ref other => panic!("unexpected effect answer: {other:?}"),
    };
    for _ in 0..30 {
        world.tick();
    }
    let key = temper_engine_domain::CallKey {
        task: assignment.task,
        attempt: assignment.attempt,
        completion: 1,
        position: 1,
    };
    assert!(
        matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))), Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(row)))) if matches!(row.part, jig_domain_core::CallPart::Effect { entry: observed, outcome: Some(jig_domain_core::connector::OutboxOutcome::Made), .. } if observed == entry))
    );
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        94,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::EffectForge {
                repository: forge_world::REPO,
                resource: forge_top::What::Repository,
                write: Box::new(client::api::Write::CreateIssue {
                    key: Box::new([]),
                    title: Box::from(&b"Small finding"[..]),
                    body: Box::from(&b"Fix this"[..]),
                }),
            },
        },
    );
    world.send(event);
    for _ in 0..10 {
        world.tick();
    }
    assert!(
        matches!(world.answers.last(), Some(temper_engine_domain::CallAnswer::ForgeEffect { entry: observed, outcome: Some(client::Outcome::Made { .. }), .. }) if *observed == entry)
    );
}

#[test]
fn an_explicit_forge_effect_proposal_is_retained_until_its_holder_accepts() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"propose an issue"[..]) },
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Propose {
                action: engine::ProposedAction::Effect {
                    repository: forge_world::REPO,
                    resource: forge_top::What::Repository,
                    write: Box::new(client::api::Write::CreateIssue {
                        key: Box::new([]),
                        title: b"Proposed finding".as_slice().into(),
                        body: b"Fix this".as_slice().into(),
                    }),
                },
                reason: b"please approve".as_slice().into(),
                as_holder: false,
            },
        },
    );
    world.send(event);
    world.until(Until::Ready);
    let proposal = match world.answers.last().expect("proposal answer") {
        temper_engine_domain::CallAnswer::Proposed { proposal } => *proposal,
        other => panic!("unexpected proposal answer: {other:?}"),
    };
    assert!(world.store.rows.values().any(|record| matches!(record, Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::ProposedEffect(effect) if effect.number == proposal))));
    assert_eq!(issue_count(&mut world), 0);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner session"),
        key: [94; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(assignment.task),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    }));
    world.until(Until::Ready);
    assert!(!world.store.rows.values().any(|record| matches!(record, Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::ProposedEffect(effect) if effect.number == proposal))));
    for _ in 0..30 {
        world.tick();
    }
    assert_eq!(issue_count(&mut world), 1);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: world.signed_in.expect("owner session"),
        key: [94; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(assignment.task),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    }));
    world.until(Until::Ready);
    for _ in 0..30 {
        world.tick();
    }
    assert_eq!(issue_count(&mut world), 1);
}

fn issue_effect_call(world: &mut World, assignment: &engine::Assignment) -> engine::Event {
    world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::EffectForge {
                repository: forge_world::REPO,
                resource: forge_top::What::Repository,
                write: Box::new(client::api::Write::CreateIssue {
                    key: Box::new([]),
                    title: Box::from(&b"Small finding"[..]),
                    body: Box::from(&b"Fix this"[..]),
                }),
            },
        },
    )
}

fn pending_issue_effect() -> (World, engine::Assignment) {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::from(&b"create an issue"[..]) },
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = issue_effect_call(&mut world, &assignment);
    world.send(event);
    decoded(&mut world);
    assert!(!world.store.pending.is_empty(), "effect decision awaits durability");
    (world, assignment)
}

/// Step the protocol layer's decode of the call just sent, before any other input.
fn decoded(world: &mut World) {
    temper_engine_domain_world::poll(&mut world.root, &world.env, &mut world.out);
    world.collect();
    assert!(
        world
            .immediate
            .iter()
            .any(|event| matches!(event, engine::Event::Worker(engine::WorkerInput::DecodedHostCall { .. }))),
        "the root asked to decode the call"
    );
    world.flush();
}

fn issue_count(world: &mut World) -> usize {
    let raw::Answer::Items { items, more: false, .. } = world.external(
        1,
        raw::Op::Read(raw::Read::Items {
            state: None,
            kind: Some(raw::Kind::Issue),
            labels: Box::new([]),
            author: None,
            since: Time::ZERO,
            page: 1,
            limit: 10,
        }),
    ) else {
        panic!("one bounded issue page")
    };
    items.len()
}

// From the stop on, the engine takes nothing that would make a commit, and a
// committed forge effect stays with the connector (jig's `domain/engine.md`, 6.1).
#[test]
fn a_stop_with_a_committed_forge_effect_makes_no_commit_and_no_write() {
    let (mut world, _) = pending_issue_effect();
    let calls = world.forge_calls;
    world.send(engine::Event::Stop);
    let number = world.store.apply();
    world.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
    let commits = world.store.header().commits;
    for _ in 0..30 {
        world.tick();
    }
    assert_eq!(world.store.header().commits, commits);
    assert!(world.store.pending.is_empty());
    assert_eq!(world.forge_calls, calls, "no forge call after the stop");
}

#[test]
fn effect_survives_each_commit_and_outbox_cut_without_a_second_issue() {
    for cut in 0..4 {
        let (mut world, assignment) = pending_issue_effect();
        if cut == 0 {
            // The decision was sent, but no record became durable.
            world.store.pending.clear();
        } else {
            let number = world.store.apply();
            if cut >= 2 {
                world.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
                for _ in 0..60 {
                    if world.events.iter().any(|event| {
                        matches!(
                            event,
                            engine::Event::Forge(event) if matches!(event.as_ref(),
                                temper_engine_domain_forge::Event::Client(
                                    temper_engine_domain_forge_client::Event::Answered {
                                        result: Ok(client::api::Answer::Created(_)), ..
                                    }
                                ))
                        )
                    }) {
                        break;
                    }
                    world.tick();
                }
                assert_eq!(
                    issue_count(&mut world),
                    1,
                    "the external write completed before the process cut: cut={cut} pending={:?} events={:?} answers={:?}",
                    world.pending,
                    world.events,
                    world.answers
                );
                if cut == 3 {
                    let index = world
                        .events
                        .iter()
                        .position(|event| {
                            matches!(
                                event,
                                engine::Event::Forge(event) if matches!(event.as_ref(),
                                    temper_engine_domain_forge::Event::Client(
                                        temper_engine_domain_forge_client::Event::Answered {
                                            result: Ok(client::api::Answer::Created(_)), ..
                                        }
                                    ))
                            )
                        })
                        .expect("external outcome queued for the old process");
                    let event = world.events.remove(index).expect("queued outcome");
                    world.send(event);
                    assert!(!world.store.pending.is_empty(), "outcome waits for its commit");
                    world.store.pending.clear();
                }
            }
        }
        world.restart(true, false);
        for _ in 0..40 {
            world.tick();
        }
        let event = issue_effect_call(&mut world, &assignment);
        world.send(event);
        for _ in 0..40 {
            world.tick();
        }
        assert_eq!(issue_count(&mut world), 1, "cut {cut} created one keyed issue");
        let key = temper_engine_domain::CallKey {
            task: assignment.task,
            attempt: assignment.attempt,
            completion: 1,
            position: 1,
        };
        assert!(
            matches!(
                world.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))),
                Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(row))))
                    if matches!(row.part, jig_domain_core::CallPart::Effect {
                        outcome: Some(jig_domain_core::connector::OutboxOutcome::Made), ..
                    })
            ),
            "cut {cut} committed the settled named answer"
        );
    }
}

#[test]
fn a_tracked_goals_changes_install_landing_and_current_head_ci_topics_without_agent_calls() {
    let (mut world, goal, producer, branch) = change_world_with_policy(true, 1000, None, false, false, false, true);
    let subscriptions = |world: &World| -> Vec<forge_top::Subscriber> {
        world
            .store
            .rows
            .values()
            .filter_map(|stored| match stored {
                Record::Forge { row, .. } => match row.as_ref() {
                    forge_top::Stored::Subscription(subscriber) if subscriber.task == goal.task => {
                        Some(subscriber.clone())
                    }
                    _ => None,
                },
                Record::Core(_) => None,
            })
            .collect()
    };
    assert!(
        subscriptions(&world).iter().any(|subscriber| matches!(&subscriber.topic,
        forge_top::Topic::Landings { branch, .. } if branch.as_ref() == b"main")
            && subscriber.goal_tasks.as_ref().is_some_and(|tasks| !tasks.is_empty())),
        "goal subscribed as its plan names the landing branch"
    );
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let pushed = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: u64::from(forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts {
            saved: None,
            pushed: Box::new([forge_top::Pushed {
                tag: forge_world::REPO.repository,
                commit: translate::commit(pushed),
            }]),
            preparation: None,
        },
    );
    world.send(event);
    for _ in 0..180 {
        world.tick();
        if subscriptions(&world).iter().any(|subscriber| {
            subscriber.topic == forge_top::Topic::Ci { repository: forge_world::REPO, head: translate::commit(pushed) }
        }) {
            break;
        }
    }
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::BranchHead(head) if head.commit == translate::commit(pushed)))));
    let current = subscriptions(&world);
    let ci = current
        .iter()
        .find(|subscriber| {
            subscriber.topic == forge_top::Topic::Ci { repository: forge_world::REPO, head: translate::commit(pushed) }
        })
        .expect("goal's own CI topic");
    assert!(matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(goal.task)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) if task.subscriptions.iter().any(|sub| sub.number == ci.number)));
    world.restart(true, true);
    for _ in 0..100 {
        world.tick();
    }
    assert_eq!(
        subscriptions(&world)
            .iter()
            .filter(|subscriber| matches!(subscriber.topic, forge_top::Topic::Landings { .. }))
            .count(),
        1,
        "one durable interest for the whole subtree"
    );
}

/// The task's current specification revision, as its durable row shows it.
fn spec_revision(world: &World, task: u64) -> u64 {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => row.revisions.spec,
        _ => panic!("live task {task}"),
    }
}
