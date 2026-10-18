//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{Request, forge};
use skein_lib::Queue;
/// Application-configured child and request-table bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub jig: jig_client_view::Limits,
    pub forge: forge::Limits,
    pub client: jig_client::Limits,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    checked_max_out(limits).expect("root output capacities are checked")
}
fn checked_max_out(limits: &Limits) -> Option<u32> {
    jig_client_view::max_out(&limits.jig)
        .checked_mul(4)?
        .checked_add(jig_client::max_out(&limits.client))?
        .checked_add(16)
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.jig.kept < limits.client.domain.requests.requests {
        return None;
    }
    let mut total = jig_client_view::worst_case(&limits.jig)?;
    total = total.checked_add(crate::pages::inbox::worst_case(limits)?)?;
    total = total.checked_add(crate::pages::chats::worst_case(limits)?)?;
    total = total.checked_add(Queue::<crate::pages::Request>::worst_case(crate::pages::inbox::max_out(limits))?)?;
    total = total.checked_add(forge::worst_case(&limits.forge)?)?;
    // A shown outcome may carry a protection rule or a whole core refusal.
    total = total.checked_add(limits.jig.document_bytes)?;
    // The forge holds the parts of its current page.
    total = total.checked_add(u64::from(limits.forge.parts).checked_mul(limits.jig.document_bytes)?)?;
    total = total.checked_add(jig_client::worst_case(&limits.client)?)?;
    let jig_out = jig_client_view::max_out(&limits.jig);
    let root_out = checked_max_out(limits)?;
    total = total.checked_add(Queue::<jig_client_view::Request>::worst_case(jig_out)?.checked_mul(3)?)?;
    total = total.checked_add(Queue::<forge::Request>::worst_case(forge::max_out(&limits.forge))?)?;
    total = total.checked_add(u64::from(jig_out).checked_mul(limits.jig.document_bytes)?.checked_mul(3)?)?;
    total = total.checked_add(u64::from(root_out).checked_mul(limits.jig.document_bytes)?)?;
    total.checked_add(Queue::<Request>::worst_case(root_out)?)
}

/// Compose the shared transport profile from the application's UI and retry bounds.
#[must_use]
pub fn client(
    jig: &jig_client_view::Limits,
    requests: skein_lib::RequestLimits,
    authority: u32,
    exchanges: u32,
    backoff: jig_client::Backoff,
    heartbeat: skein_lib::Duration,
) -> Option<jig_client::Limits> {
    const OWNERS: &[jig_client::Spoken] = &[jig_client::Spoken {
        family: temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge),
        version: temper_client_protocol_wire::CLIENT_VERSION,
    }];
    let limits = jig_client::Limits {
        domain: jig_client::DomainLimits {
            owners: OWNERS,
            requests,
            ask_bytes: requests.bytes,
            handles: jig.watches.checked_add(1)?,
            watches: jig.watches.checked_add(1)?,
            reads: jig.reads,
            turn_bytes: u32::try_from(jig.document_bytes).ok()?,
            document_bytes: jig.document_bytes,
            backoff,
            segments: 9,
            authority,
            rows: jig.rows,
            text: jig.text,
            projects: jig.projects,
            fields: jig.fields,
            delegates: jig.delegates,
            records: jig.records,
            kept: jig.kept,
            parts: jig.parts,
            heartbeat,
        },
        exchanges,
    };
    jig_client::worst_case(&limits)?;
    Some(limits)
}
