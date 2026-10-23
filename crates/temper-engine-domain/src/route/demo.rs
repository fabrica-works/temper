//! Each act is the person's own request, as the client would send it,
//! admitted by the root under their sign-in and a key derived from the
//! start's (protocol/setup.md, 8.4).
use super::{Decision, Delivery, Domain, Env, Limits, Work, applications, core, emit, forge, internal, people};
use alloc::boxed::Box;
use skein_lib::{ReplyTo, Time, Token};

// Zero is never a task number. One start and one act at a time use this reply.
const REPLY: u64 = 0;
#[derive(Debug)]
pub(crate) struct Context {
    pub sign_in: u64,
    pub key: [u8; 16],
    pub carried: Option<core::Carried>,
}
#[derive(Debug)]
pub(crate) struct Start {
    pub request: Token,
    pub phase: Phase,
    pub until: Time,
}
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum Phase {
    Waiting,
    Account,
    Connecting,
    Adoption,
}

pub(crate) fn owns(to: Token) -> bool {
    to == Token::new(REPLY)
}

fn key(original: [u8; 16], act: &[u8]) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper demo start\0");
    hash.update(&original);
    hash.update(act);
    let digest = hash.finish();
    digest.get(..16).expect("digest prefix").try_into().expect("16-byte derived key")
}

pub(crate) fn start(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, request: Token) {
    assert!(domain.demo_start.is_none(), "one start at a time");
    let demo = domain.forge.demo().expect("offered demo");
    let until = env.now.saturating_add(demo.deadline);
    domain.demo_start = Some(Start { request, phase: Phase::Waiting, until });
    if domain.demo_reply.is_none() {
        begin(domain, env, decision);
    }
}

fn begin(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    let Some(start) = &mut domain.demo_start else { return };
    if start.phase != Phase::Waiting {
        return;
    }
    start.phase = Phase::Account;
    let request = start.request;
    let provider = domain.forge.demo().expect("offered demo").provider;
    let mut held = false;
    for fact in &domain.core.accounts.account_views() {
        if let Some(account) = domain.core.accounts.account(fact.account) {
            held |= account.provider() == provider && account.holder() == jig_domain_accounts::Holder::Deployment;
        }
    }
    let context = domain.demo_contexts.get_mut(&request).expect("admitted start envelope");
    let carried = context.carried.take();

    if held {
        if let Some(carried) = carried {
            emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
        }
        domain.demo_start.as_mut().expect("start").phase = Phase::Connecting;
        progress(domain, env, decision);
    } else if let Some(carried) = carried {
        domain.demo_reply = Some((request, Phase::Account));
        domain.work.push(Work::Core(core::Event::Key {
            reply_to: internal(REPLY),
            sign_in: context.sign_in,
            key: key(context.key, b"account"),
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Deployment,
                change: people::AccountChange::AddKey { provider, join: None, digest: carried.digest },
            }),
            carried,
        }));
    } else {
        finish(domain, forge::StartAnswer::Failed(forge::StartFailure::Account));
    }
}

pub(crate) fn progress(domain: &mut Domain, env: &Env<Limits>, _decision: &mut Decision) {
    let Some(start) = &domain.demo_start else { return };
    let demo = domain.forge.demo().expect("offered demo");
    let connection = domain.forge.connection(demo.adoption.forge);
    let connected = connected(&connection);
    if !connected && env.now >= start.until {
        finish(domain, forge::StartAnswer::Failed(forge::StartFailure::NotConnected));
        return;
    }
    if start.phase != Phase::Connecting || !connected {
        return;
    }
    if domain.forge.demo_adopted() {
        finish(domain, forge::StartAnswer::Running);
        return;
    }
    let adoption = demo.adoption.clone();
    let request = start.request;
    let context = domain.demo_contexts.get(&request).expect("active start");
    let admission = applications::Admission {
        reply_to: internal(REPLY),
        sign_in: context.sign_in,
        key: key(context.key, b"adoption"),
        digest: skein_lib::sha256::digest(b"temper demo adoption"),
        scope: people::ApplicationScope::Project(adoption.project),
        carried: None,
    };
    domain.demo_start.as_mut().expect("start").phase = Phase::Adoption;
    domain.demo_reply = Some((request, Phase::Adoption));
    applications::start(domain, admission, crate::Application::Forge(forge::Ask::Adopt(adoption)));
}

fn finish(domain: &mut Domain, answer: forge::StartAnswer) {
    domain.demo_start = None;
    domain.work.push(Work::Forge(forge::Event::DemoAnswered { answer }));
}

pub(crate) fn reply(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    _to: ReplyTo,
    reply: people::Reply,
) {
    let Some((request, phase)) = domain.demo_reply.take() else { return };
    if !active(domain, request) {
        begin(domain, env, decision);
        return;
    }
    let kept = match reply {
        people::Reply::Outcome(people::Outcome::AccountKeyKept { .. }) => true,
        people::Reply::Outcome(_)
        | people::Reply::SignedIn { .. }
        | people::Reply::SignedOut
        | people::Reply::Again(_)
        | people::Reply::Refused(_) => false,
    };
    if phase == Phase::Account && kept {
        domain.demo_start.as_mut().expect("active start").phase = Phase::Connecting;
        progress(domain, env, decision);
    } else {
        let why = if phase == Phase::Account { forge::StartFailure::Account } else { forge::StartFailure::Adoption };
        finish(domain, forge::StartAnswer::Failed(why));
    }
}

pub(crate) fn adopted(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    outcome: Option<forge::Answered>,
) {
    let Some((request, _)) = domain.demo_reply.take() else { return };
    if !active(domain, request) {
        begin(domain, env, decision);
        return;
    }
    let answer = match outcome {
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::Adopted(_))) => forge::StartAnswer::Running,
        Some(
            forge::Answered::Adopt(
                forge::ApplicationOutcome::CoreRefused(_) | forge::ApplicationOutcome::ForgeRefused(_),
            )
            | forge::Answered::SetUp(_)
            | forge::Answered::StartDemo(_),
        )
        | None => forge::StartAnswer::Failed(forge::StartFailure::Adoption),
    };
    finish(domain, answer);
}

pub(crate) fn connection(domain: &mut Domain, forge: u16, why: &[u8]) {
    if domain.demo_start.is_some()
        && let Some(demo) = domain.forge.demo()
        && demo.adoption.forge == forge
    {
        finish(
            domain,
            forge::StartAnswer::Failed(forge::StartFailure::Group {
                why: why.get(..why.len().min(256)).expect("bounded reason").into(),
            }),
        );
    }
}

fn active(domain: &Domain, request: Token) -> bool {
    match &domain.demo_start {
        Some(start) => start.request == request,
        None => false,
    }
}

pub(crate) fn connected(connection: &forge::Connection) -> bool {
    match connection {
        forge::Connection::Connected { .. } => true,
        forge::Connection::Connecting
        | forge::Connection::NotConnected
        | forge::Connection::NotSetUp
        | forge::Connection::SettingUp { .. }
        | forge::Connection::SetUpFailed { .. }
        | forge::Connection::Stopped { .. } => false,
    }
}

pub(crate) fn done(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, request: Token, success: bool) {
    if let Some(context) = domain.demo_contexts.remove(&request)
        && success
        && let Some(carried) = context.carried
    {
        emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
    }
    if let Some(start) = &domain.demo_start
        && start.request == request
    {
        domain.demo_start = None;
    }
}
