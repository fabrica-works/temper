//! Engine and worker roots with Smith's agent and fake LLM, on jig's store
//! and party fakes (jig's domain/testing.md, sections 2 and 8).
//!
//! The process adapter keeps the concrete charter, turns and result outside
//! the domains, as a codec would. It checks their opaque bytes on each crossing;
//! no scripted agent record chooses a turn, call or ending. Smith's own world
//! supplies filesystem and completion IO below its domain. Each tick advances
//! its domain once, so durable turn ACKs and relayed answers traverse both roots.
use std::collections::{BTreeMap, VecDeque};

use jig_domain_authority as authority;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_fake_parties as parties;
use jig_fake_store as store;
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use smith_agent_world::{Job, Settings, World as Agent};
use smith_domain as smith;
use smith_domain_run as run;
use smith_host_domain::channel::{self, Down, Up};
use temper_engine_domain as engine;
use temper_engine_domain::{Delivery, Key, Record, Write};
use temper_engine_domain_world::walking;
use temper_engine_forge_world::translate as api;
use temper_fake_checkout::git::Tree;
use temper_fake_forge_domain::api as fake;
use temper_worker_checkout_world::{forge::Forge, translate as git};
use temper_worker_domain::{self as worker, wire};

const CHANNEL: Token = Token::new(7);
const PROCESS: Token = Token::new(701);
const TICK: Duration = Duration::from_millis(1);

/// What crossed the independently driven roots and process channel.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Observed {
    pub assignments: usize,
    pub checkouts: usize,
    pub spawns: usize,
    pub turns: usize,
    pub turn_acks: usize,
    pub relays: usize,
    pub replies: usize,
    pub messages: Vec<(Token, Box<[u8]>)>,
    pub cancels: usize,
    pub answers: usize,
    pub gone: usize,
}

/// A composed deployment: all roots retain their own state, with IO outside.
// Independent endpoint progress flags are test observations, not domain state.
#[expect(clippy::struct_excessive_bools, reason = "independent test endpoints retain their own progress flags")]
pub struct World {
    seed: u64,
    root: engine::Domain,
    env: Env<engine::Limits>,
    root_out: Queue<engine::Request>,
    root_events: VecDeque<engine::Event>,
    pub store: store::Store<Key, Record>,
    pub party: parties::Party,
    worker: worker::Domain,
    worker_env: Env<worker::Limits>,
    worker_out: Queue<worker::Request>,
    worker_events: VecDeque<worker::Event>,
    up: VecDeque<Up>,
    reading: Option<Token>,
    waiting_process: Option<Token>,
    reaping_process: Option<Token>,
    assignment: Option<engine::Assignment>,
    history: Option<smith::Transcript>,
    agent: Option<Agent>,
    script: Script,
    repository: bool,
    adopted: bool,
    forge: Forge,
    disk: Checkout,
    opened: bool,
    started: bool,
    admitted: bool,
    sent_turns: usize,
    sent_waits: usize,
    sent_calls: Vec<Token>,
    callbacks: BTreeMap<Token, (Box<[u8]>, Token)>,
    terminal: Option<temper_engine_smith::Terminal>,
    replies: BTreeMap<Token, run::HostAnswer>,
    settled: bool,
    answer_sent: bool,
    exit_sent: bool,
    pub observed: Observed,
}

/// A two-completion report script; optional host write precedes the report.
#[must_use]
pub fn report_script(host_call: bool) -> Script {
    let finish = Turn {
        lines: Box::new([Line::Call {
            name: Box::from(&b"finish"[..]),
            arguments: Box::from(&br#"{"report":"The composed run answered."}"#[..]),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let turns = if host_call {
        vec![
            Turn {
                lines: Box::new([Line::Call {
                    name: Box::from(&b"message"[..]),
                    arguments: Box::from(&br#"{"target":1,"form":"words","words":"check the plan"}"#[..]),
                }]),
                finish: Finish::ToolCalls,
                tokens: 20,
            },
            finish,
        ]
    } else {
        vec![finish]
    };
    Script { cue: Box::from(&b"@system"[..]), turns: turns.into_boxed_slice() }
}

impl World {
    /// Start a keyed chat through jig's independently scripted party.
    #[must_use]
    pub fn chat(seed: u64, script: Script) -> Self {
        Self::configured(seed, script, false)
    }

    /// Prepare an adopted repository through the worker before starting Smith.
    #[must_use]
    pub fn with_repository(seed: u64, script: Script) -> Self {
        Self::configured(seed, script, true)
    }

    // Keep the native boundary variants together in this ordinary IO adapter.
    #[expect(clippy::too_many_lines, reason = "ordinary adapter keeps each native boundary together")]
    fn configured(seed: u64, script: Script, repository: bool) -> Self {
        let mut config = walking::config(seed);
        config.run.model.endpoint = 0;
        config.run.model.name = Box::from(&b"fake-1"[..]);
        config.run.model.input_price = 0;
        config.run.model.cached_price = 0;
        config.run.model.output_price = 0;
        config.run.model.price_unit = 1;
        config.run.model.max_tokens = 4096;
        config.run.turns = 64;
        config.run.time = Duration::from_secs(3600);
        config.resume_bytes = 32768;
        let mut limits = walking::limits();
        limits.journal.transcript_bytes = 32768;
        limits.fleet.turns = 64;
        limits.tasks.inbox_bytes = 1024;
        limits.journal.result_bytes = 4096;
        if repository {
            grant_read(&mut config, &mut limits);
        }
        let mut forge = Forge::new(seed);
        forge.repository(
            b"org/repo",
            b"main",
            Tree::from([
                (b"README.md".to_vec(), b"The answer is 42.\n".to_vec()),
                (b"AGENTS.md".to_vec(), b"Report what you found.\n".to_vec()),
                (b"src/lib.rs".to_vec(), b"pub fn answer() -> u32 { 42 }\n".to_vec()),
            ]),
        );
        forge.grant(b"org/repo", 1, fake::Permission::Admin);
        forge.grant(b"org/repo", 7, fake::Permission::Read);
        let mut worker_limits = temper_worker_domain_world::LIMITS;
        worker_limits.host.slots = 1;
        worker_limits.host.charter_bytes = 65536;
        worker_limits.agent.charter_bytes = 65536;
        worker_limits.host.turn_bytes = 32768;
        worker_limits.agent.turn_bytes = 32768;
        worker_limits.host.turns = 2;
        worker_limits.agent.turns = 2;
        worker_limits.host.turn_queue_bytes = 65536;
        worker_limits.agent.unacknowledged_bytes = 65536;
        worker_limits.host.transcript_bytes = 32768;
        worker_limits.agent.transcript_bytes = 32768;
        worker_limits.host.event_bytes = 1024;
        worker_limits.agent.message_bytes = 1024;
        worker_limits.agent.call_bytes = 4096;
        worker_limits.checkout.message_bytes = 4096;
        worker_limits.agent.no_progress = Duration::from_secs(120);
        limits.fleet.grace = worker::declared_graces(&worker_limits)
            .expect("worker bound")
            .saturating_add(worker::push_deadline(&worker_limits).expect("push bound"))
            .saturating_add(Duration::from_secs(1));
        assert!(engine::worst_case(&limits).is_some());
        assert!(worker::worst_case(&worker_limits).is_some());
        let root = engine::Domain::new(config, &limits);
        Self {
            seed,
            root,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            root_out: Queue::with_capacity(engine::max_out(&limits)),
            root_events: VecDeque::from([engine::Event::Restart]),
            store: {
                let mut store = store::Store::new();
                let header = temper_engine_domain_world::store::HEADER;
                store.rows.insert(
                    Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)),
                    Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header))),
                );
                store
            },
            party: parties::Party::new(
                1,
                parties::Role::Owner,
                400,
                Box::new([
                    parties::Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() },
                    parties::Act::Request {
                        key: [41; 16],
                        ask: parties::Ask::Chat { words: Box::from(&b"@system"[..]) },
                    },
                ]),
            ),
            worker: worker::Domain::new(&worker_limits, seed),
            worker_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: worker_limits },
            worker_out: Queue::with_capacity(worker::max_out(&worker_limits)),
            worker_events: VecDeque::new(),
            up: VecDeque::new(),
            reading: None,
            waiting_process: None,
            reaping_process: None,
            assignment: None,
            history: None,
            agent: None,
            script,
            repository,
            adopted: !repository,
            forge,
            disk: Checkout::new(),
            opened: false,
            started: false,
            admitted: false,
            sent_turns: 0,
            sent_waits: 0,
            sent_calls: Vec::new(),
            callbacks: BTreeMap::new(),
            terminal: None,
            replies: BTreeMap::new(),
            settled: false,
            answer_sent: false,
            exit_sent: false,
            observed: Observed::default(),
        }
    }

    fn root_step(&mut self, event: engine::Event) {
        engine::step(&mut self.root, &self.env, event);
        engine::release(&mut self.root, &self.env, &mut self.root_out);
        while let Some(request) = self.root_out.pop() {
            self.root_request(request);
        }
        self.root.reclaim();
    }

    fn worker_step(&mut self, event: worker::Event) {
        worker::step(&mut self.worker, &self.worker_env, event, &mut self.worker_out);
    }

    /// Drive one domain iteration and the independently observed IO terminals.
    pub fn tick(&mut self) {
        self.env.now = self.env.now.saturating_add(TICK);
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        self.worker_env.now = self.env.now;
        self.worker_env.wall = self.env.wall;
        if let Some(event) = self.root_events.pop_front() {
            self.root_step(event);
        }
        if let Some(number) = self.store.tick(false).expect("calm store") {
            self.root_step(engine::Event::Store(engine::StoreInput::Committed { number }));
        }
        engine::fire(&mut self.root, &self.env);
        for pass in 0..32 {
            if pass == 0 {
                engine::release(&mut self.root, &self.env, &mut self.root_out);
            } else {
                temper_engine_domain_world::poll(&mut self.root, &self.env, &mut self.root_out);
            }
            if self.root_out.is_empty() && pass > 0 {
                break;
            }
            while let Some(request) = self.root_out.pop() {
                self.root_request(request);
            }
        }
        self.opened |= self.root.quiescent();
        if self.opened
            && (self.adopted || self.party.sign_in.is_none())
            && let Some(up) = self.party.tick(self.env.now)
        {
            self.party_request(up);
        }
        if self.opened {
            if let Some(event) = self.worker_events.pop_front() {
                self.worker_step(event);
            }
            if self.worker.is_due(self.worker_env.now) {
                worker::fire(&mut self.worker, &self.worker_env, &mut self.worker_out);
            }
            for _ in 0..16 {
                if !self.worker.is_ready() {
                    break;
                }
                worker::resume(&mut self.worker, &self.worker_env, &mut self.worker_out);
                while let Some(request) = self.worker_out.pop() {
                    self.worker_request(request);
                }
            }
            while let Some(request) = self.worker_out.pop() {
                self.worker_request(request);
            }
        }
        self.agent_tick();
        if let Some(owner) = self.reading.take() {
            if let Some(message) = self.up.pop_front() {
                self.worker_events.push_back(worker::Event::Received { owner, message });
            } else {
                self.reading = Some(owner);
            }
        }
        if self.answer_sent
            && self.up.is_empty()
            && !self.exit_sent
            && let (Some(owner), Some(wait)) = (self.reading, self.waiting_process)
        {
            self.reading = None;
            self.waiting_process = None;
            self.worker_events.push_back(worker::Event::Hangup { owner });
            self.worker_events.push_back(worker::Event::Exited { owner: wait });
            self.exit_sent = true;
        }
        if self.exit_sent
            && let Some(owner) = self.reaping_process.take()
        {
            self.worker_events.push_back(worker::Event::Reaped { owner, detail: Box::new([]) });
            self.observed.gone += 1;
        }
        self.worker.reclaim();
        self.root.reclaim();
    }

    /// Finish one activation, including its worker answer and committed ACK.
    pub fn run(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if self.answer_sent
                && self.party.quiescent()
                && self.observed.answers != 0
                && self.observed.gone != 0
                && self.worker_events.is_empty()
                && self.root_events.is_empty()
                && self.store.pending.is_empty()
                && self.root.quiescent()
            {
                return;
            }
        }
        panic!(
            "composed run did not settle: {:?}; up={}; reading={:?}; wait={:?}; reap={:?}; terminal={:?}; started={} settled={} sent_answer={} exit={}; agent_tail={:?}",
            self.observed,
            self.up.len(),
            self.reading,
            self.waiting_process,
            self.reaping_process,
            self.terminal,
            self.started,
            self.settled,
            self.answer_sent,
            self.exit_sent,
            self.agent().trace().iter().rev().take(8).collect::<Vec<_>>()
        );
    }

    /// Resume a parked activation from concrete history whose opaque bytes the
    /// engine made durable and returned through the worker's next Start.
    pub fn resume(&mut self, words: &[u8], script: Script) {
        assert!(matches!(self.agent().answer(), run::Answer::Parked { .. }));
        let turns = self.agent().turns();
        let first = turns.first().expect("parked conversation");
        self.history = Some(smith::Transcript {
            version: first.version,
            endpoint: first.endpoint,
            dialect: first.dialect,
            turns: turns.into(),
        });
        let mut turns = self.script.turns.to_vec();
        turns.extend(script.turns);
        self.script.turns = turns.into_boxed_slice();
        self.started = false;
        self.admitted = false;
        self.sent_turns = 0;
        self.sent_waits = 0;
        self.sent_calls.clear();
        self.terminal = None;
        self.settled = false;
        self.answer_sent = false;
        self.exit_sent = false;
        self.agent = None;
        self.act(parties::Ask::Say { task: parties::Task::Last, words: words.into() }, [44; 16]);
    }

    /// Release withheld store terminals in their independently retained order.
    pub fn release_commits(&mut self) {
        while let Some(number) = self.store.release_held() {
            self.root_events.push_back(engine::Event::Store(engine::StoreInput::Committed { number }));
        }
    }

    /// Add a new authenticated action to the independent party.
    pub fn act(&mut self, ask: parties::Ask, key: [u8; 16]) {
        self.party.extend(Box::new([parties::Act::Request { key, ask }]));
    }

    /// Wait until the process receives the first Smith host operation.
    pub fn until_call(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if !self.sent_calls.is_empty() {
                return;
            }
        }
        panic!("Smith host call did not arrive");
    }

    #[must_use]
    pub fn agent(&self) -> &Agent {
        self.agent.as_ref().expect("spawned Smith process")
    }

    #[must_use]
    pub fn assignment(&self) -> &engine::Assignment {
        self.assignment.as_ref().expect("committed assignment")
    }

    fn party_request(&mut self, request: parties::Up) {
        let event = match request {
            parties::Up::SignIn { to, provider, subject } => engine::Event::Party(engine::PartyInput::SignedIn {
                reply_to: ReplyTo::new(Token::new(to)),
                identity: people::Identity {
                    key: people::IdentityKey { provider, subject },
                    login: Box::from(&b"person"[..]),
                    name: Box::from(&b"Person"[..]),
                },
            }),
            parties::Up::Request { to, sign_in, project, key, ask } => {
                let ask = match ask {
                    parties::Ask::Chat { words } => people::Ask::StartChat { project, words },
                    parties::Ask::Say { task: parties::Task::Number(task), words } => {
                        people::Ask::Say { project, task, words }
                    }
                    parties::Ask::Stop { task: parties::Task::Number(task) } => people::Ask::Stop { project, task },
                    other @ (parties::Ask::Goal { .. }
                    | parties::Ask::Say { .. }
                    | parties::Ask::Stop { .. }
                    | parties::Ask::Release { .. }
                    | parties::Ask::Take { .. }
                    | parties::Ask::Choose { .. }
                    | parties::Ask::Decide { .. }
                    | parties::Ask::Decision { .. }) => panic!("unscripted party request: {other:?}"),
                };
                engine::Event::Party(engine::PartyInput::Ask {
                    reply_to: ReplyTo::new(Token::new(to)),
                    sign_in,
                    key,
                    ask,
                })
            }
        };
        self.root_events.push_back(event);
    }

    #[expect(clippy::wildcard_enum_match_arm, reason = "the deployment rejects unscripted peer effects explicitly")]
    fn root_request(&mut self, request: engine::Request) {
        match request {
            engine::Request::ToChild(released) => self.root_events.push_back(engine::Event::Released(released)),
            engine::Request::Store(engine::StoreRequest::Commit { number, writes }) => {
                let writes = writes
                    .into_vec()
                    .into_iter()
                    .map(|write| match write {
                        Write::Save(row) => store::Write::Save { key: row.key(), row },
                        Write::Erase(key) => store::Write::Erase(key),
                    })
                    .collect();
                self.store.submit(number, writes, u32::try_from(self.seed % 3).expect("small seeded delay"));
            }
            engine::Request::Store(engine::StoreRequest::Load { owner, range, after, most, .. }) => {
                let mut rows: Vec<_> = self
                    .store
                    .rows
                    .iter()
                    .filter(|(key, _)| range.contains(key) && after.as_ref().is_none_or(|old| *key > old))
                    .take(usize::try_from(most).expect("bounded page") + 1)
                    .map(|(_, row)| row.clone())
                    .collect();
                let next = if rows.len() > usize::try_from(most).expect("bounded page") {
                    rows.pop();
                    rows.last().map(Record::key)
                } else {
                    None
                };
                self.root_events.push_back(engine::Event::Store(engine::StoreInput::Loaded {
                    owner,
                    rows: rows.into_boxed_slice(),
                    next,
                }));
            }
            engine::Request::Forge { call, repository, op } => {
                assert_eq!(repository, temper_engine_forge_world::REPO);
                self.forge.at(self.env.now);
                let result = self.forge.api(b"org/repo", 1, api::op(&op, &self.env.limits.forge.client));
                let result =
                    result.map(|answer| api::answer(&op, answer, &self.env.limits.forge.client)).map_err(|error| {
                        api::error(
                            error,
                            temper_fake_forge_domain::time(&temper_worker_checkout_world::forge::CONFIG, self.env.now),
                        )
                    });
                self.root_events.push_back(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Client(
                    temper_engine_domain_forge_client::Event::Answered { call, cost: 1, result },
                ))));
                let _moves = self.forge.moves();
            }
            engine::Request::Worker(engine::WorkerRequest::Deliver(delivery))
            | engine::Request::Party(engine::PartyRequest::Deliver(delivery)) => self.delivery(delivery),
            engine::Request::Party(engine::PartyRequest::View(_)) | engine::Request::Account(_) => {}
            other => panic!("unscripted engine output: {other:?}"),
        }
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the fixture selects the party and host records this story sends"
    )]
    // Keep the native boundary variants together in this ordinary IO adapter.
    #[expect(clippy::too_many_lines, reason = "ordinary adapter keeps each native boundary together")]
    fn delivery(&mut self, delivery: Delivery) {
        match delivery {
            Delivery::WebReply { to, sign_in, reply } => {
                let to = to.into_token();
                if to == Token::new(900) {
                    assert!(
                        matches!(reply, people::Reply::Outcome(people::Outcome::Adopted { .. })),
                        "adopted reply: {reply:?}"
                    );
                    self.adopted = true;
                    return;
                }
                let signed_in = matches!(reply, people::Reply::SignedIn { .. });
                let reply = match reply {
                    people::Reply::SignedIn { person, .. } => {
                        parties::Reply::SignedIn { person, sign_in: sign_in.expect("signed in") }
                    }
                    people::Reply::Outcome(people::Outcome::Started { task }) => parties::Reply::Started { task },
                    people::Reply::Outcome(people::Outcome::Said { .. }) => parties::Reply::Done,
                    people::Reply::Outcome(people::Outcome::Stopped { task }) => parties::Reply::Changed { task },
                    other => panic!("unscripted party reply: {other:?}"),
                };
                self.party.reply(to.raw(), reply);
                if signed_in && self.repository {
                    self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
                        reply_to: ReplyTo::new(Token::new(900)),
                        sign_in: self.party.sign_in.expect("authenticated party"),
                        key: [90; 16],
                        ask: engine::adopt_repository_ask(
                            temper_engine_domain_forge::Adoption {
                                project: 1,
                                home: true,
                                provider: temper_engine_forge_world::REPO,
                                host: Box::from(&b"forge.example"[..]),
                                owner: Box::from(&b"org"[..]),
                                name: Box::from(&b"repo"[..]),
                                prefix: Box::from(&b"temper/"[..]),
                                role: temper_engine_domain_forge::Role::Owned,
                                checks: Box::new([]),
                                ci: true,
                                landing: Box::from(&b"main"[..]),
                            },
                            0,
                        )
                        .expect("bounded adoption request"),
                    }));
                }
            }
            Delivery::Assigned { assignment, .. } => {
                assert!(
                    self.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task)))),
                    "assignment follows durable task"
                );
                self.observed.assignments += 1;
                let charter = format!("{:?}", assignment.run).into_bytes().into_boxed_slice();
                assert_eq!(assignment.workspace.repositories.len(), usize::from(self.repository));
                let repositories = assignment
                    .workspace
                    .repositories
                    .iter()
                    .map(|item| {
                        assert!(item.push.is_none(), "inspection assignment has no push authority");
                        wire::Repository {
                            tag: item.tag,
                            name: item.name.clone(),
                            remote: [item.owner.as_ref(), b"/", item.name.as_ref()].concat().into_boxed_slice(),
                            start: match &item.start {
                                engine::ForgeStart::Base(branch) => wire::Start::Base { branch: branch.clone() },
                                engine::ForgeStart::Branch(branch) => wire::Start::Branch { branch: branch.clone() },
                                engine::ForgeStart::Saved(branch) => wire::Start::Saved { branch: branch.clone() },
                                engine::ForgeStart::Merge { branch, base } => {
                                    wire::Start::Merge { branch: branch.clone(), base: *base }
                                }
                            },
                            access: wire::Access::ReadOnly,
                            identity: item.identity,
                        }
                    })
                    .collect();
                self.worker_events.push_back(worker::Event::Assign {
                    assignment: wire::Assignment {
                        assignment: wire::RunAssignment {
                            run: Token::new(assignment.task),
                            attempt: Token::new(assignment.attempt),
                            workspace: wire::Workspace { key: assignment.workspace.key.clone(), repositories },
                            save: None,
                            charter,
                            grants: Box::new([wire::Grant {
                                account: assignment.grant.account,
                                generation: assignment.grant.generation,
                                valid: assignment.grant.valid,
                            }]),
                        },
                        turns: assignment.transcript.clone(),
                        answered: Box::new([]),
                    },
                });
                self.assignment = Some(assignment);
            }
            Delivery::AcknowledgeTurn { task, attempt, turn: number, .. } => {
                self.worker_events.push_back(worker::Event::AcknowledgeTurn {
                    run: Token::new(task),
                    attempt: Token::new(attempt),
                    turn: number,
                });
            }
            Delivery::Acknowledge { task, attempt, .. } => self
                .worker_events
                .push_back(worker::Event::Acknowledged { run: Token::new(task), attempt: Token::new(attempt) }),
            Delivery::CallAnswer { task, attempt, call, answer, .. } => {
                let (name, delivery) = self.callbacks.remove(&call).expect("worker's relay callback");
                let reply = temper_engine_smith::answer(&answer);
                let encoded = format!("{reply:?}").into_bytes().into_boxed_slice();
                self.replies.insert(call, reply);
                self.worker_events.push_back(worker::Event::Relayed {
                    run: Token::new(task),
                    attempt: Token::new(attempt),
                    call: name,
                    delivery,
                    answer: encoded,
                });
            }
            Delivery::Relay { task, attempt, word, .. } | Delivery::Inbound { task, attempt, word, .. } => {
                self.worker_events.push_back(worker::Event::Inbound {
                    run: Token::new(task),
                    attempt: Token::new(attempt),
                    name: Token::new(word.number),
                    sender: format!("{:?}", word.from).into_bytes().into_boxed_slice(),
                    words: word.words,
                });
            }
            Delivery::Cancel { task, attempt, .. } => self
                .worker_events
                .push_back(worker::Event::Cancel { run: Token::new(task), attempt: Token::new(attempt) }),
            Delivery::Fleet(_) | Delivery::Result { .. } => {}
            other => panic!("unscripted released delivery: {other:?}"),
        }
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the process adapter rejects IO absent from its declared fixture"
    )]
    // Keep the native boundary variants together in this ordinary IO adapter.
    #[expect(clippy::too_many_lines, reason = "ordinary adapter keeps each native boundary together")]
    fn worker_request(&mut self, request: worker::Request) {
        match request {
            worker::Request::Dial => self.worker_events.push_back(worker::Event::Connected),
            worker::Request::Hello { hello, graces, push_deadline } => {
                self.root_events.push_back(engine::Event::Worker(engine::WorkerInput::Hello {
                    channel: CHANNEL,
                    hello: fleet::Hello {
                        stop_bound: graces.saturating_add(push_deadline),
                        slots: hello.slots,
                        workstreams: Box::new([]),
                        hosting: Box::new([]),
                    },
                }));
            }
            worker::Request::Spawn { owner, workspace, .. } => {
                assert_eq!(workspace.is_some(), self.repository);
                self.observed.spawns += 1;
                self.worker_events.push_back(worker::Event::Spawned { owner, process: PROCESS });
            }
            worker::Request::Send { owner, process, message } => {
                assert_eq!(process, PROCESS);
                self.down(message);
                self.worker_events.push_back(worker::Event::Sent { owner });
            }
            worker::Request::Read { owner, process } => {
                assert_eq!(process, PROCESS);
                assert!(self.reading.replace(owner).is_none(), "one live process read");
            }
            worker::Request::Wait { owner, .. } => {
                self.waiting_process = Some(owner);
            }
            worker::Request::Reap { owner, .. } => {
                self.reaping_process = Some(owner);
            }
            worker::Request::Turn { run, attempt, turn } => {
                self.observed.turns += 1;
                self.root_events.push_back(engine::Event::Worker(engine::WorkerInput::Turn {
                    channel: CHANNEL,
                    task: run.raw(),
                    attempt: attempt.raw(),
                    turn: engine::Turn {
                        number: turn.turn,
                        cumulative: turn.spent,
                        read: turn.read.map(Token::raw),
                        transcript: turn.body,
                    },
                }));
            }
            worker::Request::Relay { run, attempt, call, delivery, tool, input, .. } => {
                self.observed.relays += 1;
                let submission = self
                    .agent()
                    .host_submissions()
                    .iter()
                    .find(|entry| encoded_name(entry.name).as_slice() == call.as_ref())
                    .expect("Smith emitted the relayed name");
                assert_eq!(submission.tool, tool);
                assert_eq!(submission.input.bytes(), input.as_ref());
                let body = temper_engine_smith::call(submission.name, &tool, &submission.input)
                    .expect("declared engine host schema");
                let callback = submission.relay.owner;
                self.callbacks.insert(callback, (call, delivery));
                self.root_events.push_back(engine::Event::Worker(engine::WorkerInput::Call {
                    channel: CHANNEL,
                    task: run.raw(),
                    attempt: attempt.raw(),
                    call: callback,
                    body,
                }));
            }
            worker::Request::Answer { run, attempt, answer } => {
                self.observed.answers += 1;
                let terminal = self
                    .terminal
                    .as_ref()
                    .unwrap_or_else(|| panic!("Smith ended before worker answered: {answer:?}, attempt={attempt:?}"));
                assert_eq!((answer.turns, answer.spent), (terminal.turns, terminal.cumulative));
                match &answer.ending {
                    wire::Ending::Ended { outcome, work } => {
                        assert!(work.landed.is_empty() && work.saved.is_none());
                        let run::Answer::Accepted { outcome: declared, .. } = self.agent().answer() else {
                            panic!("ended follows accepted")
                        };
                        assert_eq!(&**outcome, format!("{declared:?}").as_bytes());
                    }
                    wire::Ending::Parked { work } | wire::Ending::Failed { work, .. } => {
                        assert!(work.landed.is_empty() && work.saved.is_none());
                    }
                    wire::Ending::Refused(_) => panic!("valid assignment accepted"),
                }
                self.root_events.push_back(engine::Event::Worker(engine::WorkerInput::Answer {
                    channel: CHANNEL,
                    task: run.raw(),
                    attempt: attempt.raw(),
                    pushed: Box::new([]),
                    saved: None,
                    cumulative: terminal.cumulative,
                    end: terminal.end.clone(),
                }));
            }
            worker::Request::Io { owner, op, .. } => {
                self.observed.checkouts += 1;
                self.forge.at(self.env.now);
                let done = git::perform(&mut self.forge, &mut self.disk, op);
                self.worker_events.push_back(worker::Event::Done { owner, done });
                let _moves = self.forge.moves();
            }
            worker::Request::CancelRelay { call } => {
                self.worker_events.push_back(worker::Event::RelayCancelled { call });
            }
            worker::Request::Signal { owner, .. } => self.worker_events.push_back(worker::Event::Signalled { owner }),
            other => panic!("unscripted worker output: {other:?}"),
        }
    }

    // Keep the native boundary variants together in this ordinary IO adapter.
    #[expect(clippy::too_many_lines, reason = "ordinary adapter keeps each native boundary together")]
    fn down(&mut self, message: Down) {
        match message {
            Down::Start { start, window } => {
                assert!(!self.started);
                let assignment = self.assignment();
                assert_eq!((start.logical_run.raw(), start.activation), (assignment.task, assignment.attempt));
                assert_eq!(&*start.charter, format!("{:?}", assignment.run).as_bytes());
                assert_eq!(start.directories.len(), usize::from(self.repository));
                assert_eq!(start.workspace.is_some(), self.repository);
                assert_eq!(start.transcript.as_deref().unwrap_or(&[]), assignment.transcript.as_ref());
                assert_eq!(window.turns, 2);
                if let Some(history) = &self.history {
                    let encoded: Vec<_> =
                        history.turns.iter().map(|turn| format!("{turn:?}").into_bytes().into_boxed_slice()).collect();
                    assert_eq!(
                        encoded.as_slice(),
                        assignment.transcript.as_ref(),
                        "restored concrete history is the committed opaque prefix"
                    );
                }
                let mut settings = Settings::calm(self.seed);
                let largest = smith::max_turn_bytes(&settings.limits).expect("bounded Smith turn");
                let mut disk = Checkout::new();
                let workspace = start.workspace.map(|token| {
                    let directory = &start.directories[0];
                    let path = [git::dir(token).as_slice(), b"/", directory.name.as_ref()].concat();
                    let prefix = [path.as_slice(), b"/"].concat();
                    for (file, content) in self.disk.files() {
                        if let Some(relative) = file.strip_prefix(prefix.as_slice()) {
                            disk.write(&[directory.name.as_ref(), b"/", relative].concat(), content);
                        }
                    }
                    run::Workspace {
                        directories: Box::new([run::Directory {
                            name: directory.name.clone(),
                            root: Token::new(disk.root(&directory.name)),
                            writable: directory.writable,
                            git: directory.git,
                            conflicts: directory.conflicts.clone(),
                        }]),
                    }
                });
                let smith_start = temper_engine_smith::start(
                    assignment,
                    workspace,
                    self.history.clone(),
                    ReplyTo::new(Token::new(402)),
                    start.logical_run,
                    smith::Window { turns: window.turns, bytes: largest * u64::from(window.turns) },
                );
                let smith::Event::Start { ref charter, .. } = smith_start else { panic!("Smith start") };
                settings.job = Job::Reporting;
                settings.resume = self.history.is_some();
                settings.budget = charter.budget;
                settings.limits.run.budget = charter.budget;
                settings.limits.run.host_tools = 34;
                settings.limits.run.brief_sections = 16;
                settings.limits.session.spend = charter.budget.spend;
                let mut agent =
                    Agent::with_workspace_scripts_start(settings, disk, Box::new([self.script.clone()]), smith_start);
                agent.enable_parent_host_calls();
                self.agent = Some(agent);
                self.started = true;
            }
            Down::Acknowledge { turn } => {
                self.observed.turn_acks += 1;
                let agent = self.agent.as_mut().expect("live agent");
                agent.acknowledge_at(agent.now(), turn);
            }
            Down::Answer { call, reply } => {
                self.observed.replies += 1;
                let submission = self
                    .agent()
                    .host_submissions()
                    .iter()
                    .find(|entry| entry.relay.owner == call)
                    .expect("pending host callback");
                let relay = submission.relay;
                let (error, body) = match reply {
                    channel::Reply::Host { error, body } => (error, body),
                    channel::Reply::Withdrawn => {
                        self.agent
                            .as_mut()
                            .expect("live agent")
                            .return_host_reply(relay, run::HostReply::Withdrawn)
                            .expect("withdrawn relay terminal");
                        return;
                    }
                    channel::Reply::Unavailable => {
                        self.agent
                            .as_mut()
                            .expect("live agent")
                            .return_host_reply(relay, run::HostReply::Unanswered(run::Unanswered::Lost))
                            .expect("unavailable relay terminal");
                        return;
                    }
                    other @ (channel::Reply::Delivery(_) | channel::Reply::Busy | channel::Reply::TooLarge) => {
                        panic!("unexpected host reply: {other:?}")
                    }
                };
                // The concrete answer is retained by the codec adapter under the
                // committed engine callback, and the worker forwards its bytes.
                assert!(
                    self.store.rows.values().any(|record| matches!(record,
                    Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record)))
                        if record.key.task == self.assignment().task
                        && record.key.completion == submission.name.completion
                        && record.key.position == submission.name.position)),
                    "durable host-call decision"
                );
                let answer = self.replies.remove(&call).expect("decoded committed answer");
                assert!(!error);
                assert_eq!(&*body, format!("{answer:?}").as_bytes());
                self.agent
                    .as_mut()
                    .expect("live agent")
                    .return_host_reply(relay, run::HostReply::Answered(answer))
                    .expect("one pending relay");
            }
            Down::Message { name, label, text } => {
                self.observed.messages.push((name, text.clone()));
                let agent = self.agent.as_mut().expect("live agent");
                let text = [label.as_ref(), b": ", text.as_ref()].concat().into_boxed_slice();
                agent.message_at(agent.now(), name, text);
            }
            Down::Cancel => {
                self.observed.cancels += 1;
                self.agent.as_mut().expect("live agent").cancel_run();
            }
            Down::Grant { .. } => {}
        }
    }

    fn agent_tick(&mut self) {
        if !self.started || self.settled {
            return;
        }
        let agent = self.agent.as_mut().expect("started process");
        self.settled = agent.drive(1);
        if !self.admitted
            && agent
                .messages_seen()
                .iter()
                .any(|(_, seen)| matches!(seen, smith_agent_world::messages_referee::Seen::Admitted))
        {
            self.up.push_back(Up::Admitted);
            self.admitted = true;
        }
        for submission in agent.pending_host_calls() {
            if !self.sent_calls.contains(&submission.relay.owner) {
                self.sent_calls.push(submission.relay.owner);
                self.up.push_back(Up::Call {
                    call: submission.relay.owner,
                    name: channel::CallName {
                        activation: submission.name.activation,
                        completion: submission.name.completion,
                        position: submission.name.position,
                    },
                    deadline: self.worker_env.now.saturating_add(submission.deadline.saturating_since(submission.at)),
                    ask: channel::Ask::Host {
                        tool: submission.tool.clone(),
                        effect: match submission.effect {
                            run::HostEffect::Read => channel::Effect::Read,
                            run::HostEffect::Write => channel::Effect::Write,
                        },
                        body: submission.input.bytes().into(),
                    },
                });
            }
        }
        for (number, read, spent) in &agent.turn_metadata()[self.sent_turns..] {
            let record = &agent.turns()[usize::try_from(*number - 1).expect("positive turn")];
            self.up.push_back(Up::Turn {
                turn: channel::Turn {
                    number: *number,
                    read: *read,
                    spent: spent.units,
                    body: format!("{record:?}").into_bytes().into_boxed_slice(),
                },
            });
        }
        self.sent_turns = agent.turn_metadata().len();
        for (_, read) in &agent.waiting()[self.sent_waits..] {
            self.up.push_back(Up::Waiting { read: *read });
        }
        self.sent_waits = agent.waiting().len();
        if self.settled {
            let terminal =
                temper_engine_smith::result(crate::world::copy_answer(agent.answer()), None).expect("report terminal");
            let result = match agent.answer() {
                run::Answer::Accepted { outcome, .. } => {
                    channel::RunResult::Accepted { outcome: format!("{outcome:?}").into_bytes().into_boxed_slice() }
                }
                run::Answer::Parked { .. } => channel::RunResult::Parked,
                run::Answer::Failed { failure: run::Failure::Cancelled, .. } => {
                    channel::RunResult::Failed { failure: channel::RunFailure::Cancelled }
                }
                other @ (run::Answer::Refused(_) | run::Answer::Failed { .. }) => {
                    panic!("unscripted Smith ending: {other:?}")
                }
            };
            self.up.push_back(Up::Answer {
                answer: channel::Answer { turns: terminal.turns, spent: terminal.cumulative, result },
            });
            self.terminal = Some(terminal);
            self.answer_sent = true;
        }
    }
}

fn encoded_name(name: run::CallName) -> Vec<u8> {
    [
        name.activation.to_be_bytes().as_slice(),
        name.completion.to_be_bytes().as_slice(),
        name.position.to_be_bytes().as_slice(),
    ]
    .concat()
}

fn grant_read(config: &mut engine::Config, limits: &mut engine::Limits) {
    let grant = authority::Grant {
        connector: 0,
        kind: 1,
        pattern: authority::Pattern {
            segments: Box::new([Box::from(&b"forge"[..]), Box::from(&b"forge.example"[..]), Box::from(&b"org"[..])]),
            last: authority::Last::Open(Box::from(&b"repo"[..])),
        },
    };
    limits.people.people = 4;
    limits.people.holdings = 4;
    limits.authority.roles = 4;
    limits.journal.writes += 128;
    limits.authority.segments = 9;
    limits.authority.segment_bytes = 64;
    limits.tasks.authority_segments = 9;
    limits.tasks.authority_bytes = 128;
    limits.notes.scopes = 4;
    let mut authority_rules = config.authority.rules().clone();
    authority_rules.ceiling.grants = Box::new([grant.clone()]);
    authority_rules.ceiling.tools = authority::Tools(1);
    let mut policy = config.authority.policy(1).expect("configured policy").clone();
    policy.ceiling.grants = Box::new([grant.clone()]);
    policy.ceiling.tools = authority::Tools(1);
    policy.projections = Box::new([grant.clone()]);
    policy.roles[0].authority.grants = Box::new([grant.clone()]);
    policy.roles[0].authority.tools = authority::Tools(1);
    let mut roles = policy.roles.to_vec();
    for number in 1..4 {
        let mut role = roles[0].clone();
        role.number = number;
        roles.push(role);
    }
    policy.roles = roles.into_boxed_slice();
    config.chat_authority.grants = Box::new([grant]);
    config.chat_authority.tools = authority::Tools(1);
    let mut domain = authority::Domain::new(authority_rules, limits.authority).expect("read authority");
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    config.authority = domain;
}
