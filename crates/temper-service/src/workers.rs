//! Worker channel routing, with temper's parts decoded beside an answer.
use jig_domain_host as host;
use jig_protocol_worker_wire::Part;
use skein_lib::{Reader, Token};
use temper_engine_domain as root;
use temper_protocol_worker_wire::v1 as wire;

/// Decode the answer's named parts; any missing or bad part discards all parts.
#[must_use]
pub fn parts(up: &host::LinkUp, parts: &[Part], limits: &wire::Limits) -> Option<root::Parts> {
    let (work, preparation) = match up {
        host::LinkUp::Answer { answer, .. } => match &answer.ending {
            host::Ending::Ended { work, .. } | host::Ending::Parked { work } => (Some(work), None),
            host::Ending::Failed { work, failure, .. } => (
                Some(work),
                match failure {
                    host::Failure::Unprepared(host::Preparation::Permanent { resource }) => *resource,
                    host::Failure::Unprepared(host::Preparation::Transient)
                    | host::Failure::Run(_)
                    | host::Failure::Agent(_)
                    | host::Failure::Cancelled(_) => None,
                },
            ),
            host::Ending::Refused(_) => (None, None),
        },
        host::LinkUp::Activity { .. }
        | host::LinkUp::Rejected { .. }
        | host::LinkUp::Exhausted { .. }
        | host::LinkUp::Turn { .. }
        | host::LinkUp::Call { .. } => (None, None),
    };
    let left = match work {
        Some(work) => left(work.left, parts, limits).ok()?,
        None => None,
    };
    let saved = match work {
        Some(work) => saved(work.saved, parts, limits).ok()?,
        None => None,
    };
    let preparation = preparation_part(preparation, parts, limits).ok()?;
    Some(crate::translate::parts(left, saved, preparation))
}
fn bytes(token: Token, parts: &[Part]) -> Option<&[u8]> {
    for part in parts {
        if part.token == token {
            return Some(&part.bytes);
        }
    }
    None
}
fn left(token: Option<Token>, parts: &[Part], limits: &wire::Limits) -> Result<Option<wire::Left>, ()> {
    let Some(token) = token else {
        return Ok(None);
    };
    let mut reader = Reader::new(bytes(token, parts).ok_or(())?);
    let value = wire::Left::decode(limits, &mut reader).or(Err(()))?;
    if !reader.is_empty() {
        return Err(());
    }
    Ok(Some(value))
}
fn saved(token: Option<Token>, parts: &[Part], limits: &wire::Limits) -> Result<Option<wire::Saved>, ()> {
    let Some(token) = token else {
        return Ok(None);
    };
    let mut reader = Reader::new(bytes(token, parts).ok_or(())?);
    let value = wire::Saved::decode(limits, &mut reader).or(Err(()))?;
    if !reader.is_empty() {
        return Err(());
    }
    Ok(Some(value))
}
fn preparation_part(
    token: Option<Token>,
    parts: &[Part],
    limits: &wire::Limits,
) -> Result<Option<wire::Preparation>, ()> {
    let Some(token) = token else {
        return Ok(None);
    };
    let mut reader = Reader::new(bytes(token, parts).ok_or(())?);
    let value = wire::Preparation::decode(limits, &mut reader).or(Err(()))?;
    if !reader.is_empty() {
        return Err(());
    }
    Ok(Some(value))
}
