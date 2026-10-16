//! The forge family's records at the clients' boundary (client.md, 8).
use alloc::boxed::Box;
use jig_protocol_engine_clients as kit;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
mod refusals;

/// Decode the keyed request, preserving its admission envelope and supplied digest.
pub fn to_root(
    exchange: kit::ToApplication,
    forges: &[crate::settings::Forge],
    limits: &w::Limits,
) -> Result<root::Event, Box<kit::Event>> {
    match exchange {
        kit::ToApplication::Request { reply_to, sign_in, key, family, version, record, digest } => {
            let mut reader = Reader::new(&record);
            let request = if family == wire::FORGE && version == wire::CLIENT_VERSION {
                w::Adopt::decode(limits, &mut reader).ok()
            } else {
                None
            };
            let Some(request) = request else {
                return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable { to: reply_to })));
            };
            if !reader.is_empty() {
                return Err(Box::new(kit::Event::Application(kit::FromApplication::Undecodable { to: reply_to })));
            }
            let a = request.into_parts();
            let mut host = Box::default();
            for forge in forges {
                if forge.number == a.forge {
                    host = self::host(&forge.git);
                    break;
                }
            }
            Ok(root::Event::Party(root::PartyInput::Application {
                reply_to,
                sign_in,
                key,
                digest,
                request: forge::Adoption {
                    project: a.project,
                    home: a.home,
                    forge: a.forge,
                    host,
                    owner: a.owner,
                    name: a.name,
                    prefix: a.prefix,
                    role: role(a.role),
                    landing: a.landing,
                    ci: a.ci,
                    checks: a.checks.into_boxed(),
                },
            }))
        }
        kit::ToApplication::Read { reply, family, .. } => {
            Err(Box::new(kit::Event::Application(kit::FromApplication::Read { reply, family, record: None })))
        }
        kit::ToApplication::Watch { watcher, .. } => {
            Err(Box::new(kit::Event::WatchRefused { watcher, why: jig_domain_core::WatchRefusal::Unknown }))
        }
    }
}
/// Keep the authority of an already checked git URL, including its explicit port.
#[must_use]
pub fn host(git: &[u8]) -> Box<[u8]> {
    let Some(start) = skein_lib::bytes::find(git, b"://") else { return Box::default() };
    let rest = git.get(start.checked_add(3).expect("URL separator")..).expect("separator within URL");
    let mut first = 0;
    let mut end = rest.len();
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b'/' || *byte == b'?' || *byte == b'#' {
            end = index;
            break;
        }
        if *byte == b'@' {
            first = index.checked_add(1).expect("URL authority separator");
        }
    }
    rest.get(first..end).expect("host within URL authority").into()
}
fn role(role: w::Role) -> forge::Role {
    match role {
        w::Role::Owned => forge::Role::Owned,
        w::Role::Adopted => forge::Role::Adopted,
        w::Role::Fork => forge::Role::Fork,
        w::Role::Context => forge::Role::Context,
    }
}
fn encoded_role(role: forge::Role) -> w::Role {
    match role {
        forge::Role::Owned => w::Role::Owned,
        forge::Role::Adopted => w::Role::Adopted,
        forge::Role::Fork => w::Role::Fork,
        forge::Role::Context => w::Role::Context,
    }
}
/// Encode one durable terminal, or the unknown terminal past its retention.
#[must_use]
pub fn answer(
    outcome: Option<forge::ApplicationOutcome>,
    limits: &w::Limits,
    jig: &jig_protocol_client_wire::v7::Limits,
) -> Option<Box<[u8]>> {
    let outcome = match outcome {
        Some(forge::ApplicationOutcome::Adopted(a)) => w::Outcome::Adopted(
            w::Adopted::new(
                limits,
                w::AdoptedParts {
                    forge: a.repository.forge,
                    repository: a.repository.repository,
                    owner: a.owner,
                    name: a.name,
                    role: encoded_role(a.role),
                    home: a.home,
                    kinds: w::Kinds::new(
                        limits,
                        w::KindsParts {
                            skein_bools: [
                                a.kinds.read,
                                a.kinds.push,
                                a.kinds.open,
                                a.kinds.land,
                                a.kinds.review,
                                a.kinds.status,
                                a.kinds.comment,
                                a.kinds.issue,
                                a.kinds.branch,
                            ],
                        },
                    )
                    .ok()?,
                    protection: protection(a.protection, limits)?,
                },
            )
            .ok()?,
        ),
        Some(forge::ApplicationOutcome::CoreRefused(value)) => {
            let value = refusals::refusal(value, jig).ok()?;
            let mut bytes = Writer::new(usize::try_from(value.measure()).ok()?);
            value.encode(&mut bytes).ok()?;
            w::Outcome::CoreRefused(w::CoreRefused::new(limits, w::CoreRefusedParts { refusal: bytes.finish() }).ok()?)
        }
        Some(forge::ApplicationOutcome::ForgeRefused(reason)) => w::Outcome::ForgeRefused(
            w::ForgeRefused::new(
                limits,
                w::ForgeRefusedParts {
                    reason: match reason {
                        forge::AdoptionRefusal::NoSuchForge => w::ForgeReason::NoSuchForge,
                        forge::AdoptionRefusal::NotFound => w::ForgeReason::NotFound,
                        forge::AdoptionRefusal::NoAccess => w::ForgeReason::NoAccess,
                        forge::AdoptionRefusal::Refused => w::ForgeReason::Refused,
                        forge::AdoptionRefusal::InFlight => w::ForgeReason::InFlight,
                        forge::AdoptionRefusal::TooLarge => w::ForgeReason::TooLarge,
                        forge::AdoptionRefusal::Unreachable => w::ForgeReason::Unreachable,
                        forge::AdoptionRefusal::InvalidAnswer => w::ForgeReason::InvalidAnswer,
                    },
                },
            )
            .ok()?,
        ),
        None => w::Outcome::Unknown,
    };
    let answer = w::AdoptAnswer::new(limits, w::AdoptAnswerParts { outcome }).ok()?;
    Some(wire::answer(&answer))
}
fn protection(value: forge::Protection, limits: &w::Limits) -> Option<w::Protection> {
    Some(match value {
        forge::Protection::Unknown => w::Protection::Unknown,
        forge::Protection::Absent => w::Protection::Absent,
        forge::Protection::Rule(rule) => {
            let mut contexts = List::with_capacity(limits.rule_contexts);
            for context in rule.contexts {
                contexts.push(context).ok()?;
            }
            w::Protection::Rule(
                w::Rule::new(
                    limits,
                    w::RuleParts {
                        branch: rule.branch,
                        contexts,
                        approvals: rule.approvals,
                        dismiss_stale: rule.dismiss_stale,
                    },
                )
                .ok()?,
            )
        }
    })
}
