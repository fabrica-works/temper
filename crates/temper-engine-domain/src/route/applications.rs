//! The core admits each typed application before its forge connector decides it.
use super::{Domain, Work, forge, forge_route};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, ReplyTo, Token};

pub(super) fn start(
    domain: &mut Domain,
    reply_to: ReplyTo,
    sign_in: u64,
    key: [u8; 16],
    digest: [u8; 32],
    scope: people::ApplicationScope,
    request: forge::Adoption,
) {
    assert!(domain.application.is_none(), "one application admission value");
    let owner = Token::new(domain.next_application_owner);
    domain.next_application_owner = domain.next_application_owner.checked_add(1).expect("application owner counter");
    domain.application = Some((owner, request));
    domain.work.push(Work::Core(core::Event::Application {
        reply_to,
        sign_in,
        key,
        scope,
        connector: domain.config.forge_connector,
        digest,
        owner,
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
