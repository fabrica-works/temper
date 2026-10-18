//! Forge records and the bounded attempts owned by their routing (client/README.md, 4.6).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use crate::forge;
use jig_client_protocol_link as link;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};
#[must_use]
pub fn request(a: &forge::Adoption, limits: &w::Limits) -> Option<jig_client::Family> {
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
    Some(jig_client::Family {
        family: wire::number(wire::Owner::Forge),
        version: wire::CLIENT_VERSION,
        bytes: wire::adopt(&record),
    })
}
#[must_use]
pub fn answer(record: &jig_client::Family, limits: &w::Limits, link: &link::Limits) -> Option<forge::Answer> {
    if record.family != wire::number(wire::Owner::Forge) {
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
pub fn core_refusal(bytes: &[u8], limits: &link::Limits) -> Option<jig_client::contract::Refusal> {
    use jig_client_protocol_wire as j;
    let mut reader = Reader::new(bytes);
    let refusal = j::Refusal::decode(&limits.wire, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let final_answer = j::Final::Refused(j::Refused::new(&limits.wire, j::RefusedParts { refusal, at: None }).ok()?);
    let mut writer = Writer::new(usize::try_from(final_answer.measure()).expect("measured record fits usize"));
    final_answer.encode(&mut writer).ok()?;
    let owned = j::Owned::new(
        &limits.wire,
        j::OwnedParts { owner: j::Owner::Jig, version: j::VERSION, record: writer.finish() },
    )
    .ok()?;
    match link::translate::answers::answer(j::KeyedAnswer::FinalAnswer(owned), limits).ok()? {
        jig_client::contract::Answer::Refused { refusal, .. } => Some(refusal),
        jig_client::contract::Answer::Done(_) => None,
    }
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
