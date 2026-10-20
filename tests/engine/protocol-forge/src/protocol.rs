//! The forge connector and real component against the fake Forgejo through bytes.
use crate::fixtures;
use skein_fake_forge_domain::{self as forge, api as raw};
use skein_fake_forge_protocol::{Faults, Route, Tally};
use skein_forge_connection::Component;
use skein_forge_machine_world::joint::{self, Joint, REPO};
use skein_lib::{Duration, Env, List, Map, Queue, ReplyTo, Time, Token, Wall};
use std::collections::{BTreeMap, BTreeSet};
use temper_engine_domain_forge as top;
use temper_engine_forge_world::LIMITS;

/// A top, transport, fake service and independently scripted people.
pub struct Protocol {
    top: top::Domain,
    env: Env<top::Limits>,
    joint: Joint,
    top_out: Queue<top::Request>,
    stored: Map<top::Key, top::Stored>,
    seen: List<top::Request>,
    pending: BTreeMap<u64, (u64, Token, bool)>,
    answered: BTreeSet<u64>,
    serial: u64,
    generation: u64,
    seed: u64,
    writes: u32,
    calls: u32,
    costs: u64,
    secrets_kept: bool,
    item_pages: Vec<u64>,
    item_start: u64,
}
impl Protocol {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self::cutting(seed, 0, Faults { reset: 0, drop: 0, server_error: 0 })
    }
    #[must_use]
    pub fn cutting(seed: u64, chance: u32, faults: Faults) -> Self {
        let mut config = fixtures::protocol_config();
        config.faults = faults;
        let now = Time::from_nanos(100_000_000_000);
        let mut world = Self {
            top: Self::top(seed),
            env: Env { now, wall: Wall::from_nanos(now.as_nanos()), limits: LIMITS },
            joint: Joint::with(
                seed,
                chance,
                config,
                fixtures::limits(),
                fixtures::writer_settings(),
                fixtures::protocol_shared(),
                joint::checks(1000),
            ),
            top_out: Queue::with_capacity(top::max_out(&LIMITS)),
            stored: Map::with_capacity(128),
            seen: List::with_capacity(256),
            pending: BTreeMap::new(),
            answered: BTreeSet::new(),
            serial: 0,
            generation: 1,
            seed,
            writes: 0,
            calls: 0,
            costs: 0,
            secrets_kept: true,
            item_pages: Vec::new(),
            item_start: 0,
        };
        world.event(top::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        world
    }
    fn top(seed: u64) -> top::Domain {
        top::Domain::new(
            &LIMITS,
            seed,
            skein_forge_client::Config {
                namespace: Box::from(b"world".as_slice()),
                writers: Box::new([skein_forge_client::Writer { forge: REPO.forge, author: 1 }]),
            },
            &[top::GitIdentity { forge: REPO.forge, credential: 1 }],
        )
        .expect("configured connector")
    }
    pub fn event(&mut self, event: top::Event) {
        self.secrets_kept &= fixtures::token_absent(&event);
        top::step(&mut self.top, &self.env, event, &mut self.top_out);
        self.take_top();
        self.top.reclaim();
    }

    pub fn adopt(&mut self) {
        self.adopt_as(top::Role::Owned);
    }

    pub fn adopt_as(&mut self, role: top::Role) {
        for attempt in 0..8 {
            self.event(top::Event::Application {
                request: Token::new(100 + attempt),
                token: Token::new(100 + attempt),
                retention: Duration::from_secs(60),
                adoption: top::Adoption {
                    project: 5,
                    home: role != top::Role::Context,
                    forge: REPO.forge,
                    host: Box::from(&b"forge.example"[..]),
                    owner: Box::from(&b"org"[..]),
                    name: Box::from(&b"repo"[..]),
                    prefix: Box::from(&b"temper/"[..]),
                    role,
                    landing: Box::from(&b"main"[..]),
                    ci: true,
                    checks: Box::new([]),
                },
            });
            self.run_for(1);
            if !self.stored.contains_key(&top::Key::Repository(REPO)) {
                self.run_for(30);
            }
            if self.stored.contains_key(&top::Key::Repository(REPO)) {
                return;
            }
            self.run_for(60); // The person can retry after the durable refusal expires.
        }
        panic!("the person could not adopt the fixture repository");
    }

    #[must_use]
    pub fn seen(&self) -> &[top::Request] {
        self.seen.as_slice()
    }
    pub fn take_seen(&mut self) -> Box<[top::Request]> {
        let taken = core::mem::replace(&mut self.seen, List::with_capacity(256));
        taken.into_boxed()
    }
    #[must_use]
    pub fn writes(&self) -> u32 {
        self.writes
    }
    #[must_use]
    pub fn calls(&self) -> u32 {
        self.calls
    }
    #[must_use]
    pub fn stored(&self) -> &Map<top::Key, top::Stored> {
        &self.stored
    }
    pub fn slow_calls(&mut self, delay: Duration) {
        self.joint.fake_env.limits.latency_min = delay;
        self.joint.fake_env.limits.latency_max = delay;
    }
    /// Script a worker's pushed branch through the independent fake git.
    pub fn permission(&mut self, permission: raw::Permission) {
        forge::grant(&mut self.joint.domain, b"org/repo", 1, permission);
    }

    pub fn produce(&mut self, branch: &[u8]) {
        self.produce_file(branch, b"file", b"change");
    }
    pub fn produce_file(&mut self, branch: &[u8], path: &[u8], content: &[u8]) {
        self.external(raw::Op::Git(raw::Git::Create { branch: Box::from(branch), commit: 1 }), 2);
        self.push_file(branch, path, content);
    }
    pub fn preload_history(&mut self, count: u32) {
        for number in 0..count {
            self.external(
                raw::Op::Write(raw::Write::CreateIssue {
                    title: Box::from(&b"old"[..]),
                    body: Box::from(number.to_be_bytes()),
                    labels: Box::new([]),
                }),
                2,
            );
        }
    }
    pub fn push(&mut self, branch: &[u8], content: &[u8]) {
        if self.joint.domain.branch(b"org/repo", branch).is_none() {
            self.external(raw::Op::Git(raw::Git::Create { branch: Box::from(branch), commit: 1 }), 2);
        }
        self.push_file(branch, b"file", content);
    }
    pub fn push_file(&mut self, branch: &[u8], path: &[u8], content: &[u8]) {
        self.push_as(branch, path, content, 1);
    }
    pub fn push_as(&mut self, branch: &[u8], path: &[u8], content: &[u8], actor: u64) {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: self.joint.fake_env.limits };
        forge::advance(&mut self.joint.domain, &env, b"org/repo", branch, path, content, actor).expect("scripted push");
    }
    pub fn status(&mut self, branch: &[u8], check: raw::Check) {
        self.external(
            raw::Op::Write(raw::Write::Status {
                commit: self.branch(branch),
                context: Box::from(&b"build"[..]),
                state: check,
            }),
            1,
        );
    }
    pub fn open_pull(&mut self, head: &[u8], base: &[u8]) -> u64 {
        let raw::Answer::Created(number) = self.external(
            raw::Op::Write(raw::Write::OpenPull {
                opens_empty: false,
                title: Box::from(&b"Change"[..]),
                body: Box::from(&b"Description"[..]),
                head: Box::from(head),
                base: Box::from(base),
            }),
            2,
        ) else {
            panic!("fixture pull opened")
        };
        number
    }
    pub fn close(&mut self, number: u64) {
        self.external(raw::Op::Write(raw::Write::Close { number }), 2);
    }
    pub fn resolve(&mut self, branch: &[u8], content: &[u8]) {
        let head = self.branch(branch);
        let base = self.branch(b"main");
        let merged = forge::merge_commit(
            &mut self.joint.domain,
            &self.joint.fake_env.limits,
            head,
            base,
            Box::new([raw::File { path: Box::from(&b"file"[..]), content: Box::from(content) }]),
            b"Resolve conflict",
        )
        .expect("fixture merge commit");
        assert_eq!(
            self.external(
                raw::Op::Git(raw::Git::Push { branch: Box::from(branch), commit: merged, expected: Some(head) }),
                1,
            ),
            raw::Answer::Pushed(raw::Pushed::Pushed)
        );
    }
    #[must_use]
    pub fn branch(&self, branch: &[u8]) -> u64 {
        self.joint.domain.branch(b"org/repo", branch).expect("fixture branch")
    }

    /// Old calls reach their accounting terminal without delivering an outcome to the new top.
    pub fn restart(&mut self) {
        self.finish();
        let component = Component::new(fixtures::writer_settings(), &fixtures::limits(), &fixtures::protocol_shared())
            .expect("fresh component");
        self.joint.replace(component);
        self.top = Self::top(self.seed);
        let records: Vec<_> = self.stored.iter().map(|(_, row)| row.clone()).collect();
        for record in records {
            self.event(top::Event::Restore { record });
        }
        self.event(top::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    }
    pub fn issue(&mut self, title: &[u8], body: &[u8]) {
        self.external_at(
            raw::Op::Write(raw::Write::CreateIssue {
                title: Box::from(title),
                body: Box::from(body),
                labels: Box::from([]),
            }),
            2,
            self.env.now,
        );
    }
    /// Accounts the old process's abandoned calls at their deadlines, without delivering them to the top.
    pub fn finish(&mut self) {
        self.generation += 1;
        let mut terminal = Env { now: self.joint.env.now, wall: self.joint.env.wall, limits: self.joint.env.limits };
        terminal.now = terminal.now.saturating_add(terminal.limits.deadline);
        self.joint.component.fire(&terminal, &mut self.joint.above, &mut self.joint.below);
        while let Some(event) = self.joint.above.pop() {
            if let skein_forge_connection::Event::Answered(answer) = event {
                assert!(self.joint.answers.insert(answer.call.raw(), answer).is_none(), "one terminal");
            }
        }
        self.take_answers();
        assert!(self.pending.is_empty(), "every submitted call ended");
        assert_eq!(u64::from(self.calls), self.answered.len() as u64, "each call has one terminal");
    }
    fn external(&mut self, op: raw::Op, user: u64) -> raw::Answer {
        self.external_at(op, user, Time::ZERO)
    }
    fn external_at(&mut self, op: raw::Op, user: u64, now: Time) -> raw::Answer {
        let mut config = self.joint.fake_env.limits;
        config.latency_min = Duration::ZERO;
        config.latency_max = Duration::ZERO;
        let env = Env { now, wall: Wall::EPOCH, limits: config };
        let mut output = Queue::with_capacity(forge::MAX_OUT);
        forge::step(
            &mut self.joint.domain,
            &env,
            forge::Event::Call {
                reply_to: ReplyTo::new(Token::new(u64::MAX)),
                user,
                repository: Box::from(b"org/repo".as_slice()),
                op,
            },
            &mut output,
        );
        let mut answer = None;
        for _ in 0..4 {
            if self.joint.domain.is_due(env.now) {
                forge::fire(&mut self.joint.domain, &env, &mut output);
            }
        }
        while let Some(request) = output.pop() {
            if let forge::Request::Reply { result, .. } = request {
                answer = Some(result.expect("scripted operation"));
            }
        }
        self.joint.domain.reclaim();
        answer.expect("scripted answer")
    }
    pub fn run_for(&mut self, seconds: u64) {
        let end = self.env.now.saturating_add(Duration::from_secs(seconds));
        for _ in 0..200_000 {
            self.joint.tick();
            self.take_answers();
            if self.top.is_ready() {
                top::resume(&mut self.top, &self.env, &mut self.top_out);
                self.take_top();
            } else if self.top.next_deadline().is_some_and(|at| at <= self.env.now) {
                top::fire(&mut self.top, &self.env, &mut self.top_out);
                self.take_top();
            } else if !self.joint.busy() {
                if self.env.now >= end {
                    return;
                }
                let next = self
                    .top
                    .next_deadline()
                    .filter(|at| *at > self.env.now)
                    .unwrap_or(end)
                    .min(self.joint.component.next_deadline().filter(|at| *at > self.env.now).unwrap_or(end))
                    .min(self.joint.domain.next_deadline().filter(|at| *at > self.env.now).unwrap_or(end))
                    .min(end)
                    .max(self.env.now.saturating_add(Duration::from_nanos(1)));
                self.joint.time(next);
                self.env.now = next;
                self.env.wall = Wall::from_nanos(next.as_nanos());
            }
            self.top.reclaim();
        }
        panic!(
            "protocol world did not settle at {:?}, end {:?}, top {:?}, component {:?}, fake {:?}, busy {}",
            self.env.now,
            end,
            self.top.next_deadline(),
            self.joint.component.next_deadline(),
            self.joint.domain.next_deadline(),
            self.joint.busy()
        );
    }

    fn take_top(&mut self) {
        let mut calls = List::with_capacity(self.top_out.len());
        let mut restart = None;
        for _ in 0..self.top_out.len() {
            let request = self.top_out.pop().expect("counted top output");
            self.secrets_kept &= fixtures::token_absent(&request);
            match request {
                top::Request::ApplicationPart { request, .. } => {
                    top::step(&mut self.top, &self.env, top::Event::Adopt { request }, &mut self.top_out);
                }
                top::Request::ApplicationDone { .. } => {}
                top::Request::ApplicationAgain { .. } => panic!("application capacity available"),
                top::Request::Adopted { reply_to, result } => {
                    let outcome = match &result {
                        Ok(adopted) => jig_domain_people::Outcome::Adopted { project: adopted.repository.project },
                        Err(_) => jig_domain_people::Outcome::Refused(jig_domain_people::Refusal::Unknown),
                    };
                    top::step(
                        &mut self.top,
                        &self.env,
                        top::Event::ApplicationAnswered { request: reply_to, outcome },
                        &mut self.top_out,
                    );
                    self.seen.push(top::Request::Adopted { reply_to, result }).expect("world output capacity");
                }
                top::Request::BriefClient { event } => {
                    top::step(&mut self.top, &self.env, top::Event::Client(event), &mut self.top_out);
                }
                top::Request::ProjectionEffect { row, entry, .. } => {
                    let mut kept = Queue::with_capacity(top::max_out(&self.env.limits));
                    top::step(&mut self.top, &self.env, top::Event::KeepProjection { row, entry }, &mut kept);
                    for _ in 0..kept.len() {
                        let top::Request::Save { record, .. } = kept.pop().expect("projection keep count") else {
                            panic!("admitted projection saves only")
                        };
                        self.stored.insert(key(&record), record).expect("world store capacity");
                    }
                }
                top::Request::Save { record, .. } => {
                    let key = key(&record);
                    self.stored.insert(key, record).expect("world store capacity");
                }
                top::Request::Erase { key } => {
                    self.stored.remove(&key).expect("erased row exists");
                }
                top::Request::Call { call, repository, op } => {
                    calls.push((call, repository, op)).expect("bounded calls");
                }
                top::Request::RestartDone { stage } => {
                    restart = match stage {
                        top::RestartStage::Restored => Some(top::Event::ReadAfresh),
                        top::RestartStage::ReadAfresh => Some(top::Event::SettleOutbox),
                        top::RestartStage::Settled => None,
                    };
                    self.seen.push(top::Request::RestartDone { stage }).expect("world output capacity");
                }
                other @ (top::Request::Resource { .. }
                | top::Request::Taken { .. }
                | top::Request::Refused { .. }
                | top::Request::Outcome { .. }
                | top::Request::NotMade { .. }
                | top::Request::ContinueRelease { .. }
                | top::Request::EffectsSettled { .. }
                | top::Request::Released { .. }
                | top::Request::Retained { .. }
                | top::Request::ReleaseFailed { .. }
                | top::Request::WakeProcedure { .. }
                | top::Request::ProjectAfter { .. }
                | top::Request::ProjectionSettled { .. }
                | top::Request::GoalTopic { .. }
                | top::Request::GoalUntopic { .. }
                | top::Request::ProjectionFailed { .. }
                | top::Request::ChangeDecision { .. }
                | top::Request::News { .. }
                | top::Request::Read { .. }
                | top::Request::BriefReady { .. }
                | top::Request::BriefSized { .. }
                | top::Request::BriefTaken { .. }
                | top::Request::Drift { .. }) => {
                    self.seen.push(other).expect("world output capacity");
                }
            }
        }
        for (call, repository, op) in calls.into_boxed() {
            self.submit(call, repository, op);
        }
        if let Some(event) = restart {
            self.event(event);
        }
    }

    fn submit(&mut self, call: Token, repository: skein_forge::api::Repository, op: skein_forge::api::Op) {
        self.calls += 1;
        if matches!(op, skein_forge::api::Op::Write(_)) {
            self.writes += 1;
        }
        self.serial += 1;
        let items = matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Items { .. }));
        if matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Items { page: 1, .. })) {
            self.item_start = self.tally().routes.get(&Route::Issues).copied().unwrap_or(0);
            self.item_pages.push(0);
        }
        assert!(self.pending.insert(self.serial, (self.generation, call, items)).is_none(), "unique call");
        self.joint.call(self.serial, repository, op);
    }
    fn take_answers(&mut self) {
        for (id, answer) in core::mem::take(&mut self.joint.answers) {
            assert!(self.answered.insert(id), "call {id} answered once");
            let (generation, call, items) = self.pending.remove(&id).expect("submitted call");
            if items {
                let pages = self.tally().routes.get(&Route::Issues).copied().unwrap_or(0) - self.item_start;
                *self.item_pages.last_mut().expect("listing pass") = pages;
            }
            self.costs += u64::from(answer.cost);
            self.secrets_kept &= fixtures::token_absent(&answer);
            if generation == self.generation {
                self.event(top::Event::Client(skein_forge_client::Event::Answered {
                    call,
                    cost: answer.cost,
                    result: answer.result,
                }));
            }
        }
    }
    #[must_use]
    pub fn listing_passes(&self) -> &[u64] {
        &self.item_pages
    }
    #[must_use]
    pub fn costs(&self) -> u64 {
        self.costs
    }
    #[must_use]
    pub fn tally(&self) -> &Tally {
        self.joint.tally()
    }
    #[must_use]
    pub fn start_reads(&self) -> u64 {
        self.tally().routes.get(&Route::User).copied().unwrap_or(0)
            + self.tally().routes.get(&Route::ApiSettings).copied().unwrap_or(0)
    }
    #[must_use]
    pub fn issues_made(&self) -> u32 {
        u32::try_from(
            self.joint
                .observations
                .iter()
                .filter(|o| matches!(o, forge::Observation::Opened { kind: raw::Kind::Issue, by: 1, .. }))
                .count(),
        )
        .expect("bounded issues")
    }
    #[must_use]
    pub fn token_kept(&self) -> bool {
        self.secrets_kept && fixtures::token_absent(&self.joint.observations)
    }
}
fn key(row: &top::Stored) -> top::Key {
    match row {
        top::Stored::Call { key, .. } => top::Key::Call(*key),
        top::Stored::Repository(row) => top::Key::Repository(row.provider),
        top::Stored::Hold(row) => top::Key::Hold(row.name.clone()),
        top::Stored::Names { task, .. } => top::Key::Names(*task),
        top::Stored::Subscription(row) => top::Key::Subscription { task: row.task, topic: row.topic.clone() },
        top::Stored::BranchHead(row) => top::Key::BranchHead(row.name.clone()),
        top::Stored::PullState(row) => top::Key::PullState(row.name.clone()),
        top::Stored::Ci(row) => top::Key::Ci { repository: row.repository, head: row.head },
        top::Stored::Landed { commit, .. } => top::Key::Landed(*commit),
        top::Stored::ProposedEffect(row) => top::Key::ProposedEffect(row.number),
        top::Stored::Entry { entry, .. } => top::Key::Entry(entry.number),
        top::Stored::Client(row) => top::Key::Client(match row {
            skein_forge_client::Stored::Live(row) => skein_forge_client::Key::Live(row.watch.resource.clone()),
            skein_forge_client::Stored::Repository(row) => skein_forge_client::Key::Repository(row.repository),
            skein_forge_client::Stored::Writer(_) => {
                unreachable!("the connector keeps the client's writer as Stored::Writer")
            }
        }),
        top::Stored::Change(row) => top::Key::Change(row.task),
        top::Stored::Issue(row) => top::Key::Issue(row.goal),
        top::Stored::Release(row) => top::Key::Release(row.task),
        top::Stored::Application { token, .. } => top::Key::Application(*token),
        top::Stored::Writer(row) => top::Key::Writer(row.forge),
    }
}
