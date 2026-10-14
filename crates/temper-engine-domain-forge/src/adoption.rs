//! A core-admitted keyed adoption and its retained terminal (domain/forge.md, 4).
use crate::{
    Adopted, AdoptedRepository, Adoption, AdoptionRefusal, ApplicationOutcome, Domain, Key, Limits, Request, Stored,
    reason,
};
use jig_domain_people as people;
use skein_lib::{Duration, Env, Map, Queue, Token, Wall};
use temper_engine_domain_forge_client::api;

#[derive(Debug)]
pub(crate) struct Pending {
    token: Token,
    retention: Duration,
    observed: Option<ApplicationOutcome>,
}

#[derive(Debug)]
pub(crate) struct State {
    pending: Map<Token, Pending>,
    kept: Map<Token, (Wall, ApplicationOutcome)>,
}

impl State {
    pub(crate) fn restore(&mut self, token: u64, until: Wall, outcome: ApplicationOutcome) {
        assert!(self.kept.insert(Token::new(token), (until, outcome)).is_ok(), "restored application outcome fits");
    }
    pub(crate) fn new(limits: &Limits) -> Self {
        Self { pending: Map::with_capacity(limits.adoptions), kept: Map::with_capacity(limits.applications) }
    }
}

impl Domain {
    /// The forge's outcome under the people child's token, while retained.
    #[must_use]
    pub fn application_outcome(&self, token: Token) -> Option<&ApplicationOutcome> {
        match self.application.kept.get(&token) {
            Some((_, outcome)) => Some(outcome),
            None => None,
        }
    }
}

pub(crate) fn start(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    retention: Duration,
    adoption: Adoption,
    out: &mut Queue<Request>,
) {
    let mut expired = None;
    for (token, (until, _)) in &d.application.kept {
        if *until <= env.wall {
            expired = Some(*token);
            break;
        }
    }
    if let Some(token) = expired {
        d.application.kept.remove(&token);
        out.push(Request::Erase { key: Key::Application(token.raw()) });
    }
    if d.application.kept.len().saturating_add(d.application.pending.len()) >= env.limits.applications
        || d.application.pending.len() >= env.limits.adoptions
    {
        return again(request, people::Again::Busy, out);
    }
    let pending = Pending { token, retention, observed: None };
    if !d.writers.contains_key(&adoption.provider.forge) {
        return finish(d, env, request, pending, ApplicationOutcome::ForgeRefused(AdoptionRefusal::NoSuchForge), out);
    }
    assert!(d.application.pending.insert(request, pending).is_ok(), "reserved application flight");
    out.push(Request::ApplicationPart { request, adoption });
}

/// Record exactly what the adoption reads decided before forwarding to the core.
pub(crate) fn adopted(d: &mut Domain, out: &mut Queue<Request>, reply_to: Token, result: Result<Adopted, api::Error>) {
    if let Some(pending) = d.application.pending.get_mut(&reply_to) {
        pending.observed = Some(match &result {
            Ok(adopted) => {
                let row = &adopted.repository;
                ApplicationOutcome::Adopted(AdoptedRepository {
                    repository: row.provider,
                    owner: row.owner.clone(),
                    name: row.name.clone(),
                    role: row.role,
                    home: row.home,
                    kinds: row.kinds,
                    protection: row.protection.clone(),
                })
            }
            Err(error) => ApplicationOutcome::ForgeRefused(reason(*error)),
        });
    }
    out.push(Request::Adopted { reply_to, result });
}

pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    outcome: people::Outcome,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.application.pending.remove(&request) else { return };
    let outcome = if let Some(ApplicationOutcome::ForgeRefused(why)) = &pending.observed {
        ApplicationOutcome::ForgeRefused(*why)
    } else if let people::Outcome::Refused(why) = outcome {
        ApplicationOutcome::CoreRefused(why)
    } else if let people::Outcome::Again(why) = outcome {
        return again(request, why, out);
    } else if let people::Outcome::Adopted { .. } = outcome {
        pending.observed.take().expect("adoption reads recorded the repository")
    } else {
        unreachable!("the core's adoption part ends with an adoption outcome");
    };
    finish(d, env, request, pending, outcome, out);
}

fn finish(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    pending: Pending,
    outcome: ApplicationOutcome,
    out: &mut Queue<Request>,
) {
    let until = Wall::from_nanos(env.wall.as_nanos().saturating_add(pending.retention.as_nanos()));
    let row = Stored::Application { token: pending.token.raw(), until, outcome: outcome.clone() };
    assert!(d.application.kept.insert(pending.token, (until, outcome)).is_ok(), "reserved application outcome");
    out.push(Request::Save { record: row, release: None, read_afresh: None });
    out.push(Request::ApplicationDone { request, token: pending.token });
}

fn again(request: Token, why: people::Again, out: &mut Queue<Request>) {
    out.push(Request::ApplicationAgain { request, why });
}

pub(crate) fn worst_case(limits: &Limits) -> Option<u64> {
    let payload = u64::from(limits.name_bytes).checked_mul(2)?.checked_add(u64::from(limits.client.answer_bytes))?;
    Map::<Token, Pending>::worst_case(limits.adoptions)?
        .checked_add(Map::<Token, (Wall, ApplicationOutcome)>::worst_case(limits.applications)?)?
        .checked_add(payload.checked_mul(u64::from(limits.adoptions).checked_add(u64::from(limits.applications))?)?)
}
