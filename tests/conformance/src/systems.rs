//! Independent fake Forgejo and the native calls it received.
use crate::observations::{self, Store};
use jig_conformance::{Clock, referee as r};
use jig_test_system::{Copy, Observed, SystemKey, ValueObserved};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use std::cell::RefCell;
use std::collections::BTreeMap;
use temper_engine_domain::{self as root, engine};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_client::api;
use temper_engine_forge_world::translate;
use temper_fake_forge_domain::{self as fake, api as raw};

struct Pending {
    root_call: Token,
    generation: u64,
    op: api::Op,
    effect: Option<(SystemKey, raw::Write, Copy, ValueObserved)>,
}
/// Forge state and transport faults persist when the engine crashes.
#[expect(clippy::struct_excessive_bools, reason = "independent transport faults and CI modes are orthogonal")]
pub struct FakeForge {
    pub domain: fake::Domain,
    config: fake::Config,
    client_limits: temper_engine_domain_forge_client::Limits,
    out: Queue<fake::Request>,
    pending: BTreeMap<Token, Pending>,
    late: Vec<(Token, api::Repository, api::Op, SystemKey)>,
    pub hold: bool,
    pub merge_only: bool,
    pub failed: bool,
    pub late_next: bool,
    pub effects: Vec<Observed>,
    pub native_observations: Vec<fake::Observation>,
    owners: BTreeMap<Vec<u8>, u64>,
    new_observations: RefCell<Vec<r::Observed>>,
    seen: BTreeMap<SystemKey, u32>,
    serial: u64,
    pub generation: u64,
    pub head: api::Commit,
}
impl FakeForge {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut config = temper_engine_forge_world::fake_config();
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        let mut domain = fake::Domain::new(&config, seed);
        fake::repository(
            &mut domain,
            &config,
            raw::Setup {
                name: b"org/repo".as_slice().into(),
                default: b"main".as_slice().into(),
                tree: Box::new([]),
                labels: Box::new([]),
                checks: raw::Checks {
                    contexts: Box::new([b"build".as_slice().into()]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
                protection: None,
                hooked: true,
            },
        );
        fake::grant(&mut domain, b"org/repo", 1, raw::Permission::Admin);
        fake::grant(&mut domain, b"org/repo", 2, raw::Permission::Write);
        fake::grant(&mut domain, b"org/repo", 4, raw::Permission::Write);
        let head = translate::commit(domain.branch(b"org/repo", b"main").expect("main head"));
        Self {
            domain,
            config,
            client_limits: super::config::root(seed, None).1.forge.client,
            out: Queue::with_capacity(fake::MAX_OUT),
            pending: BTreeMap::new(),
            late: Vec::new(),
            hold: false,
            merge_only: false,
            failed: false,
            late_next: false,
            effects: Vec::new(),
            native_observations: Vec::new(),
            owners: BTreeMap::new(),
            new_observations: RefCell::new(Vec::new()),
            seen: BTreeMap::new(),
            serial: 1,
            generation: 0,
            head,
        }
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        clippy::match_wildcard_for_single_variants,
        reason = "only forge entry rows associate the released write with its durable key"
    )]
    pub fn call(
        &mut self,
        store: &Store,
        clock: Clock,
        call: Token,
        repository: api::Repository,
        op: api::Op,
    ) -> Vec<engine::Event> {
        let key = if let api::Op::Write(write) = &op {
            let entry = store
                .rows
                .values()
                .find_map(|row| match row {
                    root::Record::Forge { row } => match row.as_ref() {
                        forge::Stored::Entry(entry) if &entry.effect.write == write => Some(entry),
                        _ => None,
                    },
                    _ => None,
                })
                .expect("released write has a durable forge entry");
            let key = observations::key(store, entry);
            let attempt = entry.attempt.expect("released write has a durable attempt");
            self.new_observations.borrow_mut().push(r::Observed::Released {
                requires: vec![r::Receipt::Outbox(
                    0,
                    key,
                    u32::try_from(attempt.sent.as_nanos() / 1_000_000).expect("bounded attempt timestamp"),
                )],
            });
            Some(key)
        } else {
            None
        };
        if let Some(key) = key
            && (self.hold || self.late_next)
            && (!self.merge_only || matches!(op, api::Op::Write(api::Write::Merge { .. })))
        {
            self.late_next = false;
            self.late.push((call, repository, op, key));
            return vec![engine::Event::ForgeAnswered { call, cost: 1, result: Err(api::Error::Timeout) }];
        }
        self.send(clock, call, op, key, false);
        Vec::new()
    }
    fn send(&mut self, clock: Clock, call: Token, op: api::Op, key: Option<SystemKey>, late: bool) {
        let native = translate::op(&op, &self.client_limits);
        let effect = match (&native, key) {
            (raw::Op::Write(write), Some(key)) => {
                let count = self.seen.entry(key).or_default();
                *count += 1;
                let copy = if late {
                    Copy::Late
                } else if *count > 1 {
                    Copy::Retried
                } else {
                    Copy::First
                };
                Some((key, write.clone(), copy, self.before(write, key)))
            }
            _ => None,
        };
        let fake_call = Token::new(self.serial);
        self.serial += 1;
        self.pending.insert(
            fake_call,
            Pending { root_call: call, generation: if late { u64::MAX } else { self.generation }, op, effect },
        );
        let env = Env { now: Time::from_nanos(clock.now), wall: Wall::from_nanos(clock.now), limits: self.config };
        fake::step(
            &mut self.domain,
            &env,
            fake::Event::Call {
                reply_to: ReplyTo::new(fake_call),
                user: 1,
                repository: b"org/repo".as_slice().into(),
                op: native,
            },
            &mut self.out,
        );
        self.collect_native();
    }
    pub fn delayed_merge(&self) -> bool {
        self.late.iter().any(|(_, _, op, _)| matches!(op, api::Op::Write(api::Write::Merge { .. })))
    }
    pub fn release_late(&mut self, clock: Clock) {
        self.hold = false;
        for (call, _, op, key) in std::mem::take(&mut self.late) {
            self.send(clock, call, op, Some(key), true);
        }
    }
    pub fn poll(&mut self, clock: Clock) -> Vec<engine::Event> {
        let env = Env { now: Time::from_nanos(clock.now), wall: Wall::from_nanos(clock.now), limits: self.config };
        if self.domain.is_due(env.now) {
            fake::fire(&mut self.domain, &env, &mut self.out);
        }
        let mut events = Vec::new();
        while let Some(request) = self.out.pop() {
            if let fake::Request::Hook { change, number, branch, commit, by, .. } = &request {
                let change = match change {
                    raw::Change::Push => branch.as_ref().map(|branch| match (commit, by) {
                        (Some(commit), Some(actor)) => api::Change::BranchMoved {
                            branch: branch.clone(),
                            head: translate::commit(*commit),
                            actor: *actor,
                        },
                        _ => api::Change::Branch(branch.clone()),
                    }),
                    raw::Change::Status => commit.map(|commit| api::Change::Commit(translate::commit(commit))),
                    raw::Change::Issue | raw::Change::Pull | raw::Change::Comment | raw::Change::Review => {
                        number.map(api::Change::Item)
                    }
                    raw::Change::Wiki => None,
                };
                if let Some(change) = change {
                    events.push(engine::Event::ForgeHint {
                        hint: api::Hint { repository: temper_engine_forge_world::REPO, change, key: None },
                    });
                }
            }
            if let fake::Request::Reply { to, result } = request {
                let call = to.into_token();
                let Some(pending) = self.pending.remove(&call) else { continue };
                if let (
                    api::Op::Read(api::Read::Statuses { commit, .. } | api::Read::Checks { commit }),
                    Ok(raw::Answer::Statuses(statuses)),
                ) = (&pending.op, &result)
                    && self
                        .domain
                        .branches(b"org/repo")
                        .iter()
                        .any(|(branch, head)| branch.as_ref() != b"main" && translate::commit(*head) == *commit)
                {
                    let mut resource = observations::repository();
                    resource.path.extend([b"branch".to_vec(), b"main".to_vec()]);
                    self.new_observations.borrow_mut().push(r::Observed::Read {
                        resource,
                        state: if !statuses.is_empty()
                            && statuses.iter().all(|status| status.state == raw::Check::Passed)
                        {
                            Some(observations::value(commit))
                        } else {
                            None
                        },
                        at: clock.now,
                    });
                }
                if let Some((key, write, copy, before)) = pending.effect {
                    let (kind, resources, condition, target) = raw_parts(&write);
                    let observation = Observed {
                        kind,
                        key,
                        resources: resources.into_iter().map(|name| jig_test_system::Name(name.path)).collect(),
                        condition,
                        target,
                        before: vec![before],
                        applied: result.is_ok(),
                        copy,
                    };
                    self.effects.push(observation.clone());
                    self.new_observations.borrow_mut().push(r::Observed::Effect { connector: 0, effect: observation });
                }
                if pending.generation == self.generation {
                    events.push(engine::Event::ForgeAnswered {
                        call: pending.root_call,
                        cost: 1,
                        result: match result {
                            Ok(answer) => Ok(translate::answer(&pending.op, answer, &self.client_limits)),
                            Err(error) => Err(translate::error(error, fake::time(&self.config, env.now))),
                        },
                    });
                }
            }
        }
        // Native changes are retained independently too, including refusals.
        self.collect_native();
        assert_eq!(self.domain.observations_lost(), 0, "forge observation capacity");
        self.domain.reclaim();
        events
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "all observations are retained while branch ownership uses only movement and deletion"
    )]
    fn collect_native(&mut self) {
        while let Some(observation) = self.domain.pop_observation() {
            match &observation {
                fake::Observation::Moved { branch, by, .. } => {
                    self.owners.insert(branch.to_vec(), *by);
                }
                fake::Observation::Deleted { branch, .. } => {
                    self.owners.remove(branch.as_ref());
                }
                _ => {}
            }
            self.native_observations.push(observation);
        }
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "unsupported writes panic and absent read answers provide no evidence"
    )]
    fn before(&self, write: &raw::Write, key: SystemKey) -> ValueObserved {
        let (state, owned) = match write {
            raw::Write::Merge { number, .. } => (
                match self.domain.inspect(&self.config, b"org/repo", &raw::Read::Pull { number: *number }) {
                    Ok(raw::Answer::Pull(pull)) => Some(observations::value(&translate::commit(pull.commit))),
                    _ => None,
                },
                false,
            ),
            raw::Write::DeleteBranch { branch } => (
                self.domain.branch(b"org/repo", branch).map(|head| observations::value(&translate::commit(head))),
                self.owners.get(branch.as_ref()).is_some_and(|by| matches!(by, 1 | 2)),
            ),
            raw::Write::Status { commit, context, .. } => {
                let found =
                    match self.domain.inspect(&self.config, b"org/repo", &raw::Read::Statuses { commit: *commit }) {
                        Ok(raw::Answer::Statuses(statuses)) => {
                            statuses.into_vec().into_iter().find(|s| s.context == *context)
                        }
                        _ => None,
                    };
                found.map_or((None, false), |status| {
                    (
                        Some(observations::value(format!("{context:?}:{:?}", status.state).as_bytes())),
                        status.author == 1,
                    )
                })
            }
            raw::Write::OpenPull { .. } | raw::Write::CreateIssue { .. } | raw::Write::Comment { .. } => (None, false),
            other => panic!("pre-write evidence outside scenario vocabulary: {other:?}"),
        };
        ValueObserved { state, owner: owned.then_some(key) }
    }
    #[expect(
        clippy::match_wildcard_for_single_variants,
        reason = "hook outputs are retained while this helper finds its own reply"
    )]
    pub fn external(&mut self, clock: Clock, op: raw::Op) -> raw::Answer {
        let token = Token::new(self.serial);
        self.serial += 1;
        let env = Env { now: Time::from_nanos(clock.now), wall: Wall::from_nanos(clock.now), limits: self.config };
        let mut queue = Queue::with_capacity(fake::MAX_OUT);
        fake::fire(&mut self.domain, &env, &mut queue);
        fake::step(
            &mut self.domain,
            &env,
            fake::Event::Call { reply_to: ReplyTo::new(token), user: 1, repository: b"org/repo".as_slice().into(), op },
            &mut queue,
        );
        fake::fire(&mut self.domain, &env, &mut queue);
        let mut answer = None;
        while let Some(request) = queue.pop() {
            match request {
                fake::Request::Reply { to, result } => {
                    let t = to.into_token();
                    if t == token {
                        answer = Some(result.expect("external forge operation"));
                    } else {
                        self.out.push(fake::Request::Reply { to: ReplyTo::new(t), result });
                    }
                }
                other => {
                    self.out.push(other);
                }
            }
        }
        self.collect_native();
        answer.expect("external reply")
    }
    pub fn fail(&mut self, clock: Clock, commit: u64) {
        self.external(
            clock,
            raw::Op::Write(raw::Write::Status {
                commit,
                context: b"build".as_slice().into(),
                state: raw::Check::Failed,
            }),
        );
    }
    pub fn create(&mut self, clock: Clock, branch: &[u8]) {
        self.external(
            clock,
            raw::Op::Git(raw::Git::Create {
                branch: branch.into(),
                commit: self.domain.branch(b"org/repo", b"main").expect("main"),
            }),
        );
    }
    pub fn advance(&mut self, clock: Clock, branch: &[u8], body: &[u8]) -> api::Commit {
        let env = Env { now: Time::from_nanos(clock.now), wall: Wall::from_nanos(clock.now), limits: self.config };
        let commit = fake::advance(
            &mut self.domain,
            &env,
            b"org/repo",
            branch,
            b"file",
            body,
            if body == b"other hand" { 4 } else { 1 },
        )
        .expect("external push");
        self.collect_native();
        if self.failed {
            self.fail(clock, commit);
        }
        translate::commit(commit)
    }
    pub fn take_observed(&self) -> Vec<r::Observed> {
        std::mem::take(&mut self.new_observations.borrow_mut())
    }
}
#[expect(clippy::wildcard_enum_match_arm, reason = "unsupported native writes fail the outside translation explicitly")]
fn raw_parts(write: &raw::Write) -> (u16, Vec<r::Name>, Option<u64>, u64) {
    let mut name = observations::repository();
    let (kind, condition, target) = match write {
        raw::Write::Status { context, state, .. } => {
            (6, None, observations::value(format!("{context:?}:{state:?}").as_bytes()))
        }
        raw::Write::CreateIssue { title, body, .. } => {
            let unmarked = body.as_ref().splitn(2, |x| *x == b'\n').nth(1).unwrap_or(body);
            (8, None, observations::value(format!("{title:?}:{unmarked:?}").as_bytes()))
        }
        raw::Write::Comment { number, body } => {
            name.path.extend([b"issue".to_vec(), number.to_string().into_bytes()]);
            let unmarked = body.as_ref().splitn(2, |x| *x == b'\n').nth(1).unwrap_or(body);
            (7, None, observations::value(unmarked))
        }
        raw::Write::Merge { number, head } => {
            name.path.extend([b"branch".to_vec(), b"main".to_vec()]);
            (4, Some(observations::value(&translate::commit(*head))), *number)
        }
        raw::Write::OpenPull { title, body, head, base } => {
            (3, None, observations::value(format!("{title:?}:{body:?}:{head:?}:{base:?}").as_bytes()))
        }
        raw::Write::DeleteBranch { branch } => {
            name.path.push(b"branch".to_vec());
            name.path.extend(branch.split(|byte| *byte == b'/').map(<[u8]>::to_vec));
            (2, None, 0)
        }
        other => panic!("unbound native forge write {other:?}"),
    };
    (kind, vec![name], condition, target)
}
