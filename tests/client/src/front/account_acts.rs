//! Accounts joins use the root-only pump of front/tests.rs; the page renderer belongs to part c.
use super::{Joint, engine_limits};
use crate::{
    app::{Event, Root, Temper},
    fixtures,
};
use jig_client_world::harness::{App, ClientId, Duty, Front, FrontFault};
use jig_domain_accounts as a;
use jig_domain_authority as auth;
use skein_lib::{Duration, Env, Queue, Wall};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};
const CLIENT: ClientId = ClientId(0);
pub const PROVIDER: u32 = 17;
#[must_use]
pub fn fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, false)
}
#[must_use]
pub fn tools_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, true)
}
fn fixture_root(seed: u64, tools: bool) -> (Joint, Root) {
    let limits = if tools { temper_client_tools::limits::client() } else { fixtures::domain_limits() };
    let mut config = walking::config(seed);
    config.core.settings.llm_providers = a::Providers {
        providers: Box::new([a::Provider {
            number: PROVIDER,
            models: Box::new([Box::from(b"scripted".as_slice())]),
            offers: a::Offers { sign_in: true, key: true, programs: Box::new([]) },
        }]),
    };
    let mut engine_limits = engine_limits();
    // The pin still seeds its legacy deployment account from Model.account; leave room for this party's account.
    engine_limits.accounts.accounts = 2;
    engine_limits.accounts.credentials = 2;
    engine_limits.authority.accounts = 3;
    engine_limits.fleet.grants = 4;
    // Two extra grants add eight entries to the four fleet routing cohorts.
    engine_limits.journal.writes = engine_limits.journal.writes.checked_add(8).expect("grant cohorts fit");
    let rules = config.core.authority.rules().clone();
    let policy = config.core.authority.policy(1).expect("fixture project").clone();
    let mut authority = auth::Domain::new(rules, engine_limits.authority).expect("account authority bounds");
    let mut out = Queue::with_capacity(auth::POLICY_MAX_OUT);
    auth::step(&mut authority, auth::Event::Policy { project: 1, policy }, &mut out);
    config.core.authority = authority;
    let driver = Driver::with_repositories(Store::new(), config, &engine_limits, fixtures::REPOSITORIES);
    let mut front = Joint::with_engine(seed, &limits, driver, 1);
    front.bind(CLIENT, fixtures::PARTY);
    let mut root = if tools {
        crate::agent::new(seed).0
    } else {
        Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed)))
    };
    event(&mut front, &mut root, Temper::start(temper_client_domain::Address::Accounts, Vec::new(), 0));
    (front, root)
}
pub fn event(front: &mut Joint, root: &mut Root, event: Event) {
    let env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::with_capacity(Temper::max_out(&env.limits));
    Temper::step(root, &env, event, &mut out);
    flush(front, root, &env, &mut out);
    settle(front, root);
}
fn flush(
    front: &mut Joint,
    root: &mut Root,
    env: &Env<temper_client_domain::Limits>,
    out: &mut Queue<temper_client_domain::Request>,
) {
    Temper::take(root, env, out);
    while let Some(request) = out.pop() {
        if let Duty::Wire(request) = Temper::duty(request) {
            front.wire(CLIENT, env.now, env.wall, request);
        }
    }
}
pub fn settle(front: &mut Joint, root: &mut Root) {
    let mut env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::with_capacity(Temper::max_out(&env.limits));
    for _ in 0..32 {
        let callbacks = front.poll(env.now, env.wall);
        if callbacks.is_empty() {
            return;
        }
        for (who, event) in callbacks {
            assert_eq!(who, CLIENT);
            env.now = env.now.saturating_add(Duration::from_millis(3));
            front.received(CLIENT, &event);
            Temper::step(root, &env, event, &mut out);
            flush(front, root, &env, &mut out);
        }
    }
    panic!("account root/front did not settle");
}
pub fn lose_next(front: &mut Joint) {
    front.fault(front.now, FrontFault::NoAnswer);
}
/// The root-only fixture plays the link's whole-answer loss under its actual attempt.
pub fn lost(front: &mut Joint, root: &mut Root) {
    let (who, attempt) = front.lost_answer.take().expect("a whole answer was lost");
    assert_eq!(who, CLIENT);
    event(
        front,
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Lost,
                    answer: None,
                    why: None,
                },
            },
        }
        .into(),
    );
}
/// The fake secrets store acknowledges the key under its source after moving the carrier.
pub fn kept(front: &mut Joint) -> u32 {
    let (source, carried) = front
        .engine
        .protocol
        .accounts
        .iter()
        .find_map(|request| {
            if let a::Request::Signing(a::Signing::Keep { source, carried, .. }) = request {
                Some((*source, *carried))
            } else {
                None
            }
        })
        .expect("key reached its component");
    assert_eq!(front.transport.carried()[&carried].expose(), b"test-private-key");
    front.transport.unused(carried);
    front.engine.send(engine::Event::Account(a::Event::Component(a::FromComponent::Granted {
        account: source,
        generation: 1,
        valid: Duration::from_secs(3600),
        form: a::Form::Credential(a::Scheme::Bearer),
    })));
    front.engine.settle();
    source
}
#[must_use]
pub fn signing(front: &Joint) -> u32 {
    front
        .engine
        .protocol
        .accounts
        .iter()
        .find_map(|request| {
            if let a::Request::Signing(a::Signing::SignIn { source, .. }) = request { Some(*source) } else { None }
        })
        .expect("sign-in reached its component")
}
pub fn assert_no_carried(front: &Joint) {
    assert!(front.transport.carried().is_empty(), "adopted and unused carriers were released");
}
