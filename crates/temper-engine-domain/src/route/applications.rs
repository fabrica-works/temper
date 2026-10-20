//! The core admits each typed application before its forge connector decides it.
use super::{Domain, Work, forge, forge_route};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, ReplyTo, Token};

/// The application's admission as the core decides it, before its owner is named.
pub(super) struct Admission {
    pub(super) reply_to: ReplyTo,
    pub(super) sign_in: u64,
    pub(super) key: [u8; 16],
    pub(super) digest: [u8; 32],
    pub(super) scope: people::ApplicationScope,
    pub(super) carried: Option<core::Carried>,
}

pub(super) fn start(domain: &mut Domain, admission: Admission, request: forge::Adoption) {
    assert!(domain.application.is_none(), "one application admission value");
    let owner = Token::new(domain.next_application_owner);
    domain.next_application_owner = domain.next_application_owner.checked_add(1).expect("application owner counter");
    domain.application = Some((owner, request));
    let Admission { reply_to, sign_in, key, digest, scope, carried } = admission;
    domain.work.push(Work::Core(core::Event::Application {
        reply_to,
        sign_in,
        key,
        scope,
        connector: domain.config.forge_connector,
        digest,
        owner,
        carried,
    }));
}

pub(super) fn admitted(domain: &mut Domain, request: Token, token: Token, owner: Token, retention: Duration) {
    let (held, adoption) = domain.application.take().expect("one admitted application value");
    assert_eq!(held, owner, "the core admits the application's owner");
    domain.work.push(Work::Forge(forge::Event::Application { request, token, retention, adoption }));
}

pub(super) fn drop(domain: &mut Domain, owner: Token) {
    if let Some((held, _)) = &domain.application {
        assert_eq!(*held, owner, "drop names the application owner");
        domain.application = None;
    }
}

pub(super) fn part(
    domain: &mut Domain,
    request: Token,
    adoption: forge::Adoption,
    repository: skein_forge::api::Repository,
) {
    match forge_route::adopt_repository_ask(adoption, repository, domain.config.forge_connector) {
        Some(ask) => domain.work.push(Work::Core(core::Event::ApplicationPart { request, ask: Box::new(ask) })),
        None => domain.work.push(Work::Forge(forge::Event::ApplicationAnswered {
            request,
            outcome: people::Outcome::Refused(people::Refusal::Unknown),
        })),
    }
}

pub(super) fn finished(domain: &mut Domain, request: Token, outcome: Result<Token, people::Again>) {
    domain.work.push(Work::Core(core::Event::ApplicationFinished { request, outcome }));
}
