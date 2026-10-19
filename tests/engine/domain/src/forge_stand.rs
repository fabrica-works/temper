//! An independent fake forge beneath the direct root fixture.
use skein_forge::api;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use temper_engine_domain as engine;
use temper_engine_domain_forge_client::{self as client};
use temper_engine_forge_world::translate;
use temper_fake_forge_domain::{self as fake, api as raw};

pub struct ForgeStand {
    domain: fake::Domain,
    env: Env<fake::Config>,
    out: Queue<fake::Request>,
    pending: std::collections::BTreeMap<Token, api::Op>,
    limits: client::Limits,
    repositories: std::collections::BTreeMap<u32, Box<[u8]>>,
    pub calls: u32,
}

impl Default for ForgeStand {
    fn default() -> Self {
        Self::new()
    }
}

impl ForgeStand {
    #[must_use]
    pub fn quiescent(&self) -> bool {
        self.pending.is_empty() && self.out.is_empty()
    }
    #[must_use]
    pub fn new() -> ForgeStand {
        Self::with(&[(b"org", b"repo", 2), (b"org", b"private", 3)])
    }
    #[must_use]
    pub fn with(repositories: &[(&[u8], &[u8], u32)]) -> ForgeStand {
        let mut config = temper_engine_forge_world::fake_config();
        config.limits.repositories = u32::try_from(repositories.len()).expect("bounded repositories");
        config.latency_min = skein_lib::Duration::ZERO;
        config.latency_max = skein_lib::Duration::ZERO;
        let mut domain = fake::Domain::new(&config, 71);
        let mut names = std::collections::BTreeMap::new();
        for (owner, repository, id) in repositories {
            let mut name = owner.to_vec();
            name.push(b'/');
            name.extend_from_slice(repository);
            fake::repository(
                &mut domain,
                &config,
                raw::Setup {
                    id: *id,
                    name: name.clone().into(),
                    default: b"main".as_slice().into(),
                    tree: Box::new([]),
                    labels: Box::new([]),
                    checks: raw::Checks {
                        contexts: Box::new([]),
                        latency_min: skein_lib::Duration::ZERO,
                        latency_max: skein_lib::Duration::ZERO,
                        silent: 0,
                        passes: 1000,
                        reruns: 0,
                        cue: None,
                    },
                    protection: None,
                    hooked: false,
                },
            );
            if name != b"org/private" {
                for (user, permission) in
                    [(1, raw::Permission::Admin), (2, raw::Permission::Write), (7, raw::Permission::Read)]
                {
                    fake::grant(&mut domain, &name, user, permission);
                }
            }
            assert!(names.insert(*id, name.into_boxed_slice()).is_none());
        }
        ForgeStand {
            domain,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config },
            out: Queue::with_capacity(fake::MAX_OUT),
            pending: std::collections::BTreeMap::new(),
            limits: crate::fixtures::forge_limits().forge.client,
            repositories: names,
            calls: 0,
        }
    }

    pub fn call(&mut self, call: Token, repository: api::Repository, op: api::Op, limits: &client::Limits) {
        self.limits = *limits;
        self.calls = self.calls.checked_add(1).expect("stand calls bound");
        assert_eq!(repository.forge, 1);
        let name: Box<[u8]> = if let Some(name) = translate::locate_name(&op) {
            name
        } else {
            self.repositories.get(&repository.repository).expect("stand repository is configured").clone()
        };
        let raw = translate::op(&op, limits);
        assert!(self.pending.insert(call, op).is_none());
        fake::step(
            &mut self.domain,
            &self.env,
            fake::Event::Call { reply_to: ReplyTo::new(call), user: 1, repository: name, op: raw },
            &mut self.out,
        );
    }

    pub fn replies(&mut self) -> Vec<engine::Event> {
        let mut replies = Vec::new();
        for _ in 0..8 {
            if !self.domain.is_due(self.env.now) {
                break;
            }
            fake::fire(&mut self.domain, &self.env, &mut self.out);
        }
        for _ in 0..self.out.len() {
            if let fake::Request::Reply { to, result } = self.out.pop().expect("stand output") {
                let call = to.into_token();
                let op = self.pending.remove(&call).expect("one stand terminal");
                let result = match result {
                    Ok(answer) => Ok(translate::answer(&op, answer, &self.limits)),
                    Err(error) => Err(translate::error(error, self.env.now)),
                };
                replies.push(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Client(
                    client::Event::Answered { call, cost: 1, result },
                ))));
            }
        }
        self.domain.reclaim();
        replies
    }
}
