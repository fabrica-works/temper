//! Forge records and the bounded attempts owned by their routing (client.md, 4.6).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use jig_protocol_client_link as link;
use skein_lib::{List, Reader, Set, Token, Writer};
use temper_client_domain::forge;
use temper_protocol_client_wire::{self as wire, v1 as w};

#[derive(Debug)]
pub struct Attempts {
    set: Set<Token>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Full;
impl Attempts {
    #[must_use]
    pub const fn new(most: u32) -> Self {
        Self { set: Set::with_capacity(most) }
    }
    /// # Errors
    /// Refuses when the bounded attempts table is full.
    pub fn sent(&mut self, attempt: Token) -> Result<(), Full> {
        match self.set.insert(attempt) {
            Ok(_) => Ok(()),
            Err(_) => Err(Full),
        }
    }
    #[must_use]
    pub fn answered(&mut self, attempt: Token, last: bool) -> bool {
        if last { self.set.remove(&attempt) } else { self.set.contains(&attempt) }
    }
}
#[derive(Debug)]
pub struct Family {
    pub attempts: Attempts,
    pub wire: w::Limits,
    pub link: link::Limits,
}
#[must_use]
pub fn request(ask: &forge::Ask, limits: &w::Limits) -> Option<link::Family> {
    let forge::Ask::Adopt(a) = ask;
    let mut checks = List::with_capacity(limits.adopt_checks);
    for check in &a.checks {
        if checks.push(*check).is_err() {
            return None;
        }
    }
    let record = w::Adopt::new(
        limits,
        w::AdoptParts {
            project: a.project,
            home: a.home,
            forge: a.forge,
            owner: a.owner.clone(),
            name: a.name.clone(),
            prefix: a.prefix.clone(),
            role: role_to(a.role),
            landing: a.landing.clone(),
            ci: a.ci,
            checks,
        },
    )
    .ok()?;
    Some(link::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::adopt(&record) })
}
#[must_use]
pub fn restored(record: &link::Family, limits: &w::Limits) -> Option<forge::Ask> {
    if record.family != wire::FORGE {
        return None;
    }
    if record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = Reader::new(&record.bytes);
    let a = w::Adopt::decode(limits, &mut reader).ok()?.into_parts();
    if !reader.is_empty() {
        return None;
    }
    Some(forge::Ask::Adopt(forge::Adoption {
        project: a.project,
        home: a.home,
        forge: a.forge,
        owner: a.owner,
        name: a.name,
        prefix: a.prefix,
        role: role_from(&a.role),
        landing: a.landing,
        ci: a.ci,
        checks: a.checks.into_boxed(),
    }))
}
#[must_use]
pub fn answer(record: &link::Family, limits: &w::Limits, link: &link::Limits) -> Option<forge::Answer> {
    if record.family != wire::FORGE {
        return None;
    }
    if record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = Reader::new(&record.bytes);
    let a = w::AdoptAnswer::decode(limits, &mut reader).ok()?.into_parts();
    if !reader.is_empty() {
        return None;
    }
    Some(match a.outcome {
        w::Outcome::Adopted(a) => {
            let a = a.into_parts();
            forge::Answer::Adopted(forge::Adopted {
                forge: a.forge,
                repository: a.repository,
                owner: a.owner,
                name: a.name,
                role: role_from(&a.role),
                home: a.home,
                kinds: forge::Kinds {
                    read: a.kinds.read(),
                    push: a.kinds.push(),
                    open: a.kinds.open(),
                    land: a.kinds.land(),
                    review: a.kinds.review(),
                    status: a.kinds.status(),
                    comment: a.kinds.comment(),
                    issue: a.kinds.issue(),
                    branch: a.kinds.branch(),
                },
                protection: match a.protection {
                    w::Protection::Unknown => forge::Protection::Unknown,
                    w::Protection::Absent => forge::Protection::Absent,
                    w::Protection::Rule(rule) => {
                        let rule = rule.into_parts();
                        forge::Protection::Rule(forge::Rule {
                            branch: rule.branch,
                            contexts: rule.contexts.into_boxed(),
                            approvals: rule.approvals,
                            dismiss_stale: rule.dismiss_stale,
                        })
                    }
                },
            })
        }
        w::Outcome::CoreRefused(a) => forge::Answer::CoreRefused(core_refusal(a.refusal(), link)?),
        w::Outcome::ForgeRefused(a) => forge::Answer::ForgeRefused(match a.into_parts().reason {
            w::ForgeReason::NoSuchForge => forge::Reason::NoSuchForge,
            w::ForgeReason::NotFound => forge::Reason::NotFound,
            w::ForgeReason::NoAccess => forge::Reason::NoAccess,
            w::ForgeReason::Refused => forge::Reason::Refused,
            w::ForgeReason::InFlight => forge::Reason::InFlight,
            w::ForgeReason::TooLarge => forge::Reason::TooLarge,
            w::ForgeReason::Unreachable => forge::Reason::Unreachable,
            w::ForgeReason::InvalidAnswer => forge::Reason::InvalidAnswer,
        }),
        w::Outcome::Unknown => forge::Answer::Unknown,
    })
}
#[must_use]
pub fn core_refusal(bytes: &[u8], limits: &link::Limits) -> Option<jig_domain_client::contract::Refusal> {
    use jig_protocol_client_wire as j;
    let mut reader = Reader::new(bytes);
    let refusal = j::Refusal::decode(&limits.wire, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let final_answer = j::Final::Refused(j::Refused::new(&limits.wire, j::RefusedParts { refusal }).ok()?);
    let mut writer = Writer::new(usize::try_from(final_answer.measure()).expect("measured record fits usize"));
    final_answer.encode(&mut writer).ok()?;
    let owned = j::Owned::new(
        &limits.wire,
        j::OwnedParts { owner: j::Owner::Jig, version: j::VERSION, record: writer.finish() },
    )
    .ok()?;
    match link::translate::answers::answer(j::Answer::FinalAnswer(owned), limits).ok()? {
        jig_domain_client::contract::Answer::Refused(refusal) => Some(refusal),
        jig_domain_client::contract::Answer::Done(_) => None,
    }
}
#[must_use]
pub const fn served(versions: jig_domain_client::contract::VersionRange) -> forge::Served {
    if versions.contains(wire::CLIENT_VERSION) { forge::Served::Yes } else { forge::Served::OutOfDate }
}
const fn role_to(role: forge::Role) -> w::Role {
    match role {
        forge::Role::Owned => w::Role::Owned,
        forge::Role::Adopted => w::Role::Adopted,
        forge::Role::Fork => w::Role::Fork,
        forge::Role::Context => w::Role::Context,
    }
}
const fn role_from(role: &w::Role) -> forge::Role {
    match role {
        w::Role::Owned => forge::Role::Owned,
        w::Role::Adopted => forge::Role::Adopted,
        w::Role::Fork => forge::Role::Fork,
        w::Role::Context => forge::Role::Context,
    }
}
#[must_use]
pub fn up(request: link::Request, family: &mut Family) -> super::Up {
    super::up(request, family)
}
#[must_use]
pub fn down(request: temper_client_domain::Request, family: &mut Family) -> super::Down {
    super::down(request, family)
}
