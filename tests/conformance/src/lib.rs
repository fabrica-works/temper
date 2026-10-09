//! Temper's engine on jig's crash harness and outside referee.
//! The forge observes native API calls, and store snapshots are independent of
//! root state (jig's domain/testing.md, sections 5–6).
mod config;
pub mod observations;
mod scenarios;
pub mod stories;
pub mod systems;

use jig_conformance::referee as r;
use jig_conformance::{Application, Clock, Input, Output};
use jig_core_accounts as accounts;
use jig_core_people as people;
use jig_core_tasks as tasks;
use jig_fake_workers as workers;
use observations::Store;
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use temper_engine_domain::{self as root, engine};
use temper_engine_domain_forge as forge;

/// Scenario identity and seed retained across cold starts.
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub seed: u64,
    pub scenario: Option<jig_conformance::scenarios::Scenario>,
    pub story: Option<stories::Story>,
    limits: engine::Limits,
    maximum: u64,
}
impl Config {
    #[must_use]
    pub fn new(seed: u64, scenario: Option<jig_conformance::scenarios::Scenario>) -> Self {
        Self::configured(seed, scenario, None)
    }
    #[must_use]
    pub fn for_story(seed: u64, story: stories::Story) -> Self {
        Self::configured(seed, None, Some(story))
    }
    fn configured(
        seed: u64,
        scenario: Option<jig_conformance::scenarios::Scenario>,
        story: Option<stories::Story>,
    ) -> Self {
        let limits = config::root(seed, scenario, story).1;
        let maximum = engine::worst_case(&limits).expect("finite root")
            + u64::try_from(std::mem::size_of::<Domain>() - std::mem::size_of::<engine::Domain>())
                .expect("driver size");
        Self { seed, scenario, story, limits, maximum }
    }
}
/// Timers enter independently from external root inputs.
#[expect(clippy::large_enum_variant, reason = "the test driver retains native events directly for the harness trace")]
pub enum Event {
    Root(engine::Event),
    Fire,
    Resume,
}
impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root(event) => {
                if let engine::Event::Loaded { owner, rows, next } = event {
                    // The store retains the exact rows; traces need the page's
                    // identity and bounds rather than another formatted copy.
                    f.debug_struct("Loaded")
                        .field("owner", owner)
                        .field("rows", &rows.len())
                        .field("next", next)
                        .finish()
                } else {
                    f.debug_tuple("Root").field(event).finish()
                }
            }
            Self::Fire => f.write_str("Fire"),
            Self::Resume => f.write_str("Resume"),
        }
    }
}
/// Send an external input through the conformance harness.
pub fn send(world: &mut jig_conformance::Harness<Temper>, event: engine::Event) {
    world.send(Event::Root(event));
}
/// The application binding.
pub struct Temper;
/// Driver progress bookkeeping around the concrete engine, outside its state.
pub struct Domain {
    root: engine::Domain,
    outputs: Queue<engine::Request>,
    probed: bool,
}

/// Independent worker, party and store evidence; retained across crashes.
#[expect(clippy::struct_excessive_bools, reason = "each startup request has independent request and reply evidence")]
pub struct Peers {
    pub policy: r::Policy,
    pub seed: u64,
    pub scenario: Option<jig_conformance::scenarios::Scenario>,
    pub story: Option<stories::Story>,
    pub worker: workers::Worker,
    pub assignments: Vec<engine::Assignment>,
    pub answers: Vec<root::CallAnswer>,
    pub sign_in: Option<u64>,
    pub adopted: bool,
    signed_in: bool,
    adopting: bool,
    chatted: bool,
    applied: u64,
    observed: Vec<r::Observed>,
    requests: std::collections::BTreeMap<u64, [u8; 16]>,
    ask_bodies: std::collections::BTreeMap<u64, (u64, [u8; 16], people::Ask)>,
    ask_retries: std::collections::BTreeMap<u64, u64>,
    pub pending_proposal: Option<u64>,
    pub saved_deadline: Option<Wall>,
    pub initial_assignments: usize,
    pub changes: Vec<u64>,
    pub branches: Vec<Box<[u8]>>,
    pub produced: std::collections::BTreeMap<u64, temper_engine_domain_forge_client::api::Commit>,
    pub reported: std::collections::BTreeMap<u64, temper_engine_domain_forge_client::api::Commit>,
    calls: std::collections::BTreeMap<u64, (u64, u64, engine::Call)>,
    retries: std::collections::BTreeMap<u64, u64>,
    generation: u64,
}
/// Released outputs, drained without inspecting the root.
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "released native requests stay directly visible in the harness trace")]
pub enum Delivery {
    Root(engine::Request),
    Resume,
}

fn env(config: &Config, clock: Clock) -> Env<engine::Limits> {
    Env { now: Time::from_nanos(clock.now), wall: Wall::from_nanos(clock.now), limits: config.limits }
}

impl Application for Temper {
    type Domain = Domain;
    type Config = Config;
    type Systems = systems::FakeForge;
    type Peers = Peers;
    type Key = root::Key;
    type Record = root::Record;
    type Event = Event;
    type Delivery = Delivery;

    fn build(config: &Config) -> Domain {
        let (mut configuration, limits) = config::root(config.seed, config.scenario, config.story);
        if config.scenario == Some(jig_conformance::scenarios::Scenario::ChangedJudge) {
            for rule in &mut configuration.landing.deployment {
                rule.enforce = false;
            }
            if let Some(rules) = configuration.landing.projects.get_mut(&1) {
                for rule in rules {
                    rule.enforce = false;
                }
            }
        }
        Domain {
            root: engine::Domain::new(configuration, &limits),
            outputs: Queue::with_capacity(engine::max_out(&limits)),
            probed: false,
        }
    }
    fn systems(config: &Config, _: u64) -> systems::FakeForge {
        let mut system = systems::FakeForge::new_with_ci(
            config.seed,
            if config.story == Some(stories::Story::RepairCi) { 0 } else { 1000 },
        );
        system.lose_merge_answer = config.story == Some(stories::Story::LostMerge);
        system.lose_issue_answer = config.story == Some(stories::Story::IssueCreation);
        system.merge_only = config.story == Some(stories::Story::OtherHand)
            || matches!(
                config.scenario,
                Some(
                    jig_conformance::scenarios::Scenario::OtherHand
                        | jig_conformance::scenarios::Scenario::ChangedJudge
                )
            );
        system
    }
    fn peers(config: &Config, _: u64) -> Peers {
        let script: Box<[workers::Script]> = if config.story.is_some() {
            Box::new([workers::Script::Wait, workers::Script::Finish { report: b"story done".as_slice().into() }])
        } else {
            match config.scenario {
                Some(jig_conformance::scenarios::Scenario::RunBudget) => {
                    let mut values = vec![workers::Script::Wait];
                    for index in 0..12 {
                        values.push(workers::Script::Turn {
                            body: format!("completion {index}").into_bytes().into(),
                            cost: if index < 8 { 2 } else { 1 },
                            read: None,
                        });
                    }
                    values.push(workers::Script::Park);
                    values.into_boxed_slice()
                }
                Some(
                    jig_conformance::scenarios::Scenario::OtherHand
                    | jig_conformance::scenarios::Scenario::ChangedJudge
                    | jig_conformance::scenarios::Scenario::StandingTask,
                ) => Box::new([
                    workers::Script::Wait,
                    workers::Script::Finish { report: b"period done".as_slice().into() },
                ]),
                _ => Box::new([workers::Script::Wait]),
            }
        };
        Peers {
            seed: config.seed,
            scenario: config.scenario,
            story: config.story,
            policy: observations::policy(config.seed, config.scenario, config.story),
            worker: workers::Worker::new(7, 2, (0..16).map(|_| script.clone()).collect()),
            assignments: Vec::new(),
            answers: Vec::new(),
            sign_in: None,
            adopted: false,
            signed_in: false,
            adopting: false,
            chatted: false,
            applied: 0,
            observed: Vec::new(),
            requests: std::collections::BTreeMap::new(),
            ask_bodies: std::collections::BTreeMap::new(),
            ask_retries: std::collections::BTreeMap::new(),
            pending_proposal: None,
            saved_deadline: None,
            initial_assignments: 0,
            changes: Vec::new(),
            branches: Vec::new(),
            produced: std::collections::BTreeMap::new(),
            reported: std::collections::BTreeMap::new(),
            calls: std::collections::BTreeMap::new(),
            retries: std::collections::BTreeMap::new(),
            generation: 0,
        }
    }
    fn policy(peers: &Peers) -> r::Policy {
        peers.policy.clone()
    }
    fn recovery(_: &Config, connector: u16, kind: u16) -> r::Recovery {
        assert_eq!(connector, 0);
        match kind {
            3 | 4 => r::Recovery::Conditional,
            6 => r::Recovery::Idempotent,
            2 | 7 | 8 | 9 => r::Recovery::Unrecoverable,
            _ => panic!("unknown forge kind {kind}"),
        }
    }
    fn start(_: &Config, _: &mut Store) -> Vec<Event> {
        vec![Event::Root(engine::Event::Start)]
    }
    fn cold(peers: &mut Peers, store: &Store, _: Clock) {
        peers.observed.push(r::Observed::ColdStore { receipts: observations::snapshot(store).receipts });
        peers.worker.restart_engine();
        peers.generation += 1;
        peers.retries.extend(peers.calls.keys().map(|call| (*call, 0)));
        peers.ask_retries.extend(peers.ask_bodies.keys().map(|call| (*call, 0)));
        if peers.sign_in.is_none() {
            peers.signed_in = false;
        }
        if !peers.adopted {
            peers.adopting = false;
        }
        if peers.assignments.is_empty() {
            peers.chatted = false;
        }
    }
    fn timers(_: &Config) -> Vec<Event> {
        vec![Event::Fire]
    }
    fn step(domain: &mut Domain, config: &Config, clock: Clock, input: Input<Event, root::Record>) {
        let environment = env(config, clock);
        domain.probed = matches!(input, Input::Event(Event::Resume | Event::Fire));
        match input {
            Input::Event(Event::Root(event)) => engine::step(&mut domain.root, &environment, event),
            Input::Event(Event::Fire) => engine::fire(&mut domain.root, &environment),
            Input::Event(Event::Resume) => root::step(&mut domain.root, &environment, root::Event::Resume),
            Input::Committed(number) => {
                engine::step(&mut domain.root, &environment, engine::Event::Committed { number });
            }
            Input::Failed(number) => {
                engine::step(&mut domain.root, &environment, engine::Event::Uncommitted { number });
            }
            Input::Restore(_) => panic!("temper restores through bounded load terminals"),
        }
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "every native nonstore request is retained unchanged as a released delivery"
    )]
    fn release(domain: &mut Domain, config: &Config, clock: Clock) -> Vec<Output<root::Key, root::Record, Delivery>> {
        let environment = env(config, clock);
        engine::release(&mut domain.root, &environment, &mut domain.outputs);
        if domain.outputs.is_empty() {
            engine::release(&mut domain.root, &environment, &mut domain.outputs);
        }
        let mut outputs = Vec::new();
        while let Some(request) = domain.outputs.pop() {
            outputs.push(match request {
                engine::Request::Commit { number, writes } => Output::Commit {
                    number,
                    writes: writes
                        .into_vec()
                        .into_iter()
                        .map(|w| match w {
                            root::Write::Save(row) => jig_fake_store::Write::Save { key: row.key(), row },
                            root::Write::Erase(key) => jig_fake_store::Write::Erase(key),
                        })
                        .collect(),
                },
                engine::Request::Stop => Output::Stop,
                other => Output::Deliver(Delivery::Root(other)),
            });
        }
        if outputs.is_empty() && !domain.root.quiescent() && !domain.probed {
            outputs.push(Output::Deliver(Delivery::Resume));
        }
        if !outputs.is_empty() && !matches!(outputs.first(), Some(Output::Deliver(Delivery::Resume))) {
            domain.probed = false;
        }
        outputs
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the peer retains calls and charge evidence without interpreting other inputs"
    )]
    fn inbound(peers: &mut Peers, _: &Store, clock: Clock, input: &mut Event) {
        let Event::Root(input) = input else { return };
        match input {
            engine::Event::Call { task, attempt, call, body, .. } => {
                peers.calls.insert(call.raw(), (*task, *attempt, copy_call(body)));
                peers.retries.insert(call.raw(), clock.now + 1_000_000_000);
            }
            engine::Event::Ask { reply_to, sign_in, key, ask } => {
                let call = destination(reply_to);
                peers.requests.insert(call, *key);
                peers.ask_bodies.insert(call, (*sign_in, *key, ask.clone()));
            }
            engine::Event::Turn { task, attempt, turn, .. } => peers.observed.push(r::Observed::Completion {
                task: *task,
                attempt: *attempt,
                cumulative: turn.cumulative,
            }),
            engine::Event::Answer { task, attempt, cumulative, .. } => {
                peers.observed.push(r::Observed::Completion {
                    task: *task,
                    attempt: *attempt,
                    cumulative: *cumulative,
                });
            }
            _ => {}
        }
    }
    #[expect(
        clippy::too_many_lines,
        clippy::wildcard_enum_match_arm,
        reason = "one boundary adapter acknowledges selected native outputs and retains their receipts"
    )]
    fn deliver(
        peers: &mut Peers,
        systems: &mut systems::FakeForge,
        store: &mut Store,
        clock: Clock,
        delivery: Delivery,
    ) -> Vec<Input<Event, root::Record>> {
        let mut events = Vec::new();
        let Delivery::Root(delivery) = delivery else {
            return vec![Input::Event(Event::Fire), Input::Event(Event::Resume)];
        };
        match delivery {
            engine::Request::ToChild(released) => events.push(engine::Event::Released(released)),
            engine::Request::Load { owner, range, after, most, .. } => {
                let rows: Vec<_> = store
                    .rows
                    .iter()
                    .filter(|(key, _)| range.contains(key) && after.as_ref().is_none_or(|previous| *key > previous))
                    .take(usize::try_from(most + 1).expect("tiny load"))
                    .map(|(_, row)| row.clone())
                    .collect();
                let more = rows.len() > usize::try_from(most).expect("tiny load");
                let mut rows = rows;
                if more {
                    rows.pop();
                }
                let next = if more { rows.last().map(root::Record::key) } else { None };
                events.push(engine::Event::Loaded { owner, rows: rows.into_boxed_slice(), next });
            }
            engine::Request::Forge { call, repository, op } => {
                events.extend(systems.call(store, clock, call, repository, op));
            }
            engine::Request::Deliver(delivery) => {
                let mut requires = Vec::new();
                match delivery {
                    root::Delivery::Assigned { channel, assignment } => {
                        assert_eq!(channel.raw(), 7);
                        peers.observed.push(r::Observed::Assigned {
                            task: assignment.task,
                            attempt: assignment.attempt,
                            budget: assignment.run.budget,
                        });
                        for word in &assignment.inbox {
                            peers.observed.push(r::Observed::Reached(r::Receipt::Word(assignment.task, word.number)));
                        }
                        peers.worker.assign(workers::Assignment {
                            task: assignment.task,
                            attempt: assignment.attempt,
                            budget: assignment.run.budget,
                            transcript: assignment.transcript.clone(),
                        });
                        if !peers
                            .assignments
                            .iter()
                            .any(|a| a.task == assignment.task && a.attempt == assignment.attempt)
                        {
                            peers.assignments.push(assignment);
                        }
                    }
                    root::Delivery::AcknowledgeTurn { task, attempt, turn, .. } => {
                        requires.push(r::Receipt::Turn(task, attempt, turn));
                        peers.worker.acknowledge_turn(task, attempt, turn);
                    }
                    root::Delivery::Acknowledge { task, attempt, .. } => {
                        requires.push(r::Receipt::Terminal(task, attempt));
                        peers.worker.acknowledge_answer(task, attempt);
                    }
                    root::Delivery::Cancel { task, attempt, .. } => peers.worker.cancel(task, attempt),
                    root::Delivery::WebReply { to, sign_in, reply } => {
                        let call = to.into_token().raw();
                        let busy = matches!(
                            &reply,
                            people::Reply::Outcome(people::Outcome::Refused(
                                people::Refusal::Busy | people::Refusal::NotReady
                            ))
                        );
                        if busy {
                            peers.ask_retries.insert(call, clock.now + 1_000_000_000);
                        } else {
                            peers.ask_bodies.remove(&call);
                        }
                        let key = peers.requests.remove(&call);
                        if matches!(&reply,people::Reply::Outcome(outcome) if !matches!(outcome,people::Outcome::Refused(people::Refusal::Busy|people::Refusal::NotReady)))
                            && let Some(key) = key
                        {
                            requires.push(r::Receipt::Party(1, key));
                        }
                        match reply {
                            people::Reply::SignedIn { .. } => {
                                peers.sign_in = sign_in;
                                requires.push(r::Receipt::SignIn(sign_in.expect("signed in")));
                            }
                            people::Reply::Outcome(people::Outcome::Adopted { .. }) => peers.adopted = true,
                            _ => {}
                        }
                    }
                    root::Delivery::CallAnswer { task, attempt, call, answer, .. } => {
                        if let root::CallAnswer::Proposed { proposal } = &answer {
                            peers.pending_proposal = Some(*proposal);
                        }
                        let (_, _, body) = peers.calls.remove(&call.raw()).expect("named caller retained its request");
                        peers.retries.remove(&call.raw());
                        if !matches!(
                            answer,
                            root::CallAnswer::ForgeEffectDenied { answer: jig_core_authority::Answer::Wait, .. }
                        ) {
                            requires.push(r::Receipt::Call(task, attempt, body.completion, body.position));
                        }
                        peers.answers.push(answer);
                    }
                    root::Delivery::Inbound { task, attempt, word, .. } => {
                        requires.push(r::Receipt::Word(task, word.number));
                        peers.observed.push(r::Observed::Reached(r::Receipt::Word(task, word.number)));
                        peers.worker.message(task, attempt, word.number, word.words);
                    }
                    root::Delivery::Result { task, person, .. } => {
                        requires.push(r::Receipt::Ended(task));
                        peers.observed.push(r::Observed::Result { task, to: r::Source::Person(person) });
                    }
                    _ => {}
                }
                if !requires.is_empty() {
                    peers.observed.push(r::Observed::Released { requires });
                }
            }
            engine::Request::Account(accounts::Request::Refresh { account, generation }) => {
                events.push(engine::Event::Refreshed { account, generation, valid: Duration::from_secs(60) });
            }
            engine::Request::CallBusy { call, .. } => {
                peers.retries.insert(call.raw(), clock.now + 1_000_000_000);
            }
            engine::Request::Host(_) => panic!("scripted workers use typed calls"),
            _ => {}
        }
        events.into_iter().map(|event| Input::Event(Event::Root(event))).collect()
    }
    fn poll(
        peers: &mut Peers,
        systems: &mut systems::FakeForge,
        _: &mut Store,
        clock: Clock,
        ready: bool,
    ) -> Vec<Input<Event, root::Record>> {
        systems.generation = peers.generation;
        let mut events = systems.poll(clock);
        if ready {
            let asks: Vec<_> =
                peers.ask_retries.iter().filter_map(|(call, at)| (*at <= clock.now).then_some(*call)).collect();
            for call in asks {
                peers.ask_retries.remove(&call);
                if let Some((sign_in, key, ask)) = peers.ask_bodies.get(&call) {
                    events.push(engine::Event::Ask {
                        reply_to: ReplyTo::new(Token::new(call)),
                        sign_in: *sign_in,
                        key: *key,
                        ask: ask.clone(),
                    });
                }
            }
            let due: Vec<_> =
                peers.retries.iter().filter_map(|(call, at)| (*at <= clock.now).then_some(*call)).collect();
            for call in due {
                peers.retries.insert(call, clock.now + 1_000_000_000);
                if let Some((task, attempt, body)) = peers.calls.get(&call) {
                    events.push(engine::Event::Call {
                        channel: Token::new(7),
                        task: *task,
                        attempt: *attempt,
                        call: Token::new(call),
                        body: copy_call(body),
                    });
                }
            }
            events.extend(peers.worker.tick(Time::from_nanos(clock.now)).into_iter().map(|up| {
                let mut event = worker_event(up);
                if let engine::Event::Answer { task, pushed, .. } = &mut event
                    && let Some(commit) = peers.reported.get(task)
                {
                    *pushed = Box::new([forge::Pushed { tag: 2, commit: *commit }]);
                }
                if let engine::Event::Answer { task, end, pushed, .. } = &mut event
                    && matches!(end, tasks::End::Finished { .. })
                    && let Some(commit) = peers.produced.get(task)
                {
                    *end = tasks::End::Finished {
                        result: tasks::TaskResult::Change {
                            connector: 0,
                            kind: 2,
                            resource: u64::from(temper_engine_forge_world::REPO.repository),
                            words: b"pushed".as_slice().into(),
                        },
                        cancel_delegates: false,
                    };
                    *pushed = Box::new([forge::Pushed { tag: 2, commit: *commit }]);
                }
                event
            }));
            if !peers.signed_in {
                peers.signed_in = true;
                events.push(engine::Event::SignedIn {
                    reply_to: ReplyTo::new(Token::new(90)),
                    identity: people::Identity {
                        key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                        login: b"owner".as_slice().into(),
                        name: b"Owner".as_slice().into(),
                    },
                });
            }
            if let Some(sign_in) = peers.sign_in {
                if !peers.adopting {
                    peers.adopting = true;
                    events.push(engine::Event::Ask {
                        reply_to: ReplyTo::new(Token::new(91)),
                        sign_in,
                        key: [91; 16],
                        ask: engine::adopt_repository_ask(
                            forge::Adoption {
                                project: 1,
                                home: true,
                                provider: temper_engine_forge_world::REPO,
                                host: b"forge.example".as_slice().into(),
                                owner: b"org".as_slice().into(),
                                name: b"repo".as_slice().into(),
                                prefix: b"temper/".as_slice().into(),
                                role: forge::Role::Owned,
                                landing: b"main".as_slice().into(),
                                ci: true,
                                checks: Box::new([]),
                            },
                            0,
                        )
                        .expect("adoption"),
                    });
                }
                if peers.adopted && !peers.chatted {
                    peers.chatted = true;
                    events.push(engine::Event::Ask {
                        reply_to: ReplyTo::new(Token::new(92)),
                        sign_in,
                        key: [92; 16],
                        ask: people::Ask::StartChat { project: 1, words: b"work".as_slice().into() },
                    });
                }
            }
        }
        events.into_iter().map(|event| Input::Event(Event::Root(event))).collect()
    }
    fn observed(peers: &mut Peers, systems: &systems::FakeForge, store: &Store, _: Clock) -> Vec<r::Observed> {
        let mut observations = Vec::new();
        if peers.applied != store.applied {
            peers.applied = store.applied;
            observations.push(r::Observed::Durable { number: store.applied, snapshot: observations::snapshot(store) });
        }
        observations.append(&mut peers.observed);
        observations.extend(systems.take_observed());
        let mut totals = std::collections::BTreeMap::<u64, (u64, u64)>::new();
        for (&(task, attempt), expense) in &peers.worker.expenses {
            if expense.lost_turns > 0 {
                observations.push(r::Observed::LostTurns {
                    task,
                    attempt,
                    turns: expense.lost_turns,
                    spent: expense.lost,
                });
            }
            let total = totals.entry(task).or_default();
            total.0 += expense.spent;
            total.1 += expense.lost;
        }
        for (task, (spent, lost_bound)) in totals {
            observations.push(r::Observed::Spend { task, spent, lost_bound });
        }
        observations
    }
    fn ready(domain: &Domain) -> bool {
        domain.root.ready()
    }
    fn quiescent(domain: &Domain) -> bool {
        domain.root.quiescent() || domain.probed
    }
    fn reclaim(domain: &mut Domain) {
        domain.root.reclaim();
    }
    fn maximum(config: &Config) -> u64 {
        config.maximum
    }
    fn configuration_heap(_: &Config) -> u64 {
        0
    }
}

fn worker_event(up: workers::Up) -> engine::Event {
    match up {
        workers::Up::Hello { channel, slots, stop_bound, hosting } => engine::Event::Hello {
            channel: Token::new(channel),
            hello: jig_core_fleet::Hello {
                slots,
                stop_bound,
                workstreams: Box::new([]),
                hosting: hosting
                    .into_vec()
                    .into_iter()
                    .map(|(task, attempt, answered)| jig_core_fleet::Hosted {
                        run: Token::new(task),
                        attempt: Token::new(attempt),
                        phase: if answered { jig_core_fleet::Phase::Answered } else { jig_core_fleet::Phase::Active },
                    })
                    .collect(),
            },
        },
        workers::Up::Lost { channel } => engine::Event::Lost { channel: Token::new(channel) },
        workers::Up::Turn { channel, task, attempt, turn, spent, read, body } => engine::Event::Turn {
            channel: Token::new(channel),
            task,
            attempt,
            turn: engine::Turn { number: turn, cumulative: spent, read, transcript: body },
        },
        workers::Up::Answer { channel, task, attempt, spent, end } => engine::Event::Answer {
            channel: Token::new(channel),
            task,
            attempt,
            cumulative: spent,
            end: match end {
                workers::End::Finished(words) => {
                    tasks::End::Finished { result: tasks::TaskResult::Report { words }, cancel_delegates: false }
                }
                workers::End::Parked => tasks::End::Parked,
                workers::End::Failed(code) => panic!("unbound host failure {code:?}"),
            },
            saved: None,
            pushed: Box::new([]),
        },
        workers::Up::Call { .. } => panic!("scenario sends typed calls"),
    }
}

fn destination(to: &mut ReplyTo) -> u64 {
    let token = std::mem::replace(to, ReplyTo::new(Token::new(0))).into_token();
    *to = ReplyTo::new(token);
    token.raw()
}

#[expect(clippy::wildcard_enum_match_arm, reason = "scenario calls outside the retained vocabulary panic explicitly")]
fn copy_call(body: &engine::Call) -> engine::Call {
    let tool = match &body.tool {
        engine::Tool::EffectForge { repository, resource, write } => {
            engine::Tool::EffectForge { repository: *repository, resource: resource.clone(), write: write.clone() }
        }
        engine::Tool::Delegate { batch } => engine::Tool::Delegate { batch: batch.clone() },
        engine::Tool::Propose { action: engine::ProposedAction::Batch(batch), reason, as_holder } => {
            engine::Tool::Propose {
                action: engine::ProposedAction::Batch(batch.clone()),
                reason: reason.clone(),
                as_holder: *as_holder,
            }
        }
        engine::Tool::Propose {
            action: engine::ProposedAction::Effect { repository, resource, write },
            reason,
            as_holder,
        } => engine::Tool::Propose {
            action: engine::ProposedAction::Effect {
                repository: *repository,
                resource: resource.clone(),
                write: write.clone(),
            },
            reason: reason.clone(),
            as_holder: *as_holder,
        },
        other => panic!("scenario call outside retained vocabulary: {other:?}"),
    };
    engine::Call { completion: body.completion, position: body.position, tool }
}
