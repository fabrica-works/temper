//! A resumed delivery's evidence, as the protocol layer decodes it: smith's
//! channel record of the delivery's terminal, which must agree with the
//! outcome the engine committed (`domain/worker.md`, 4.1).

use smith_channel as wire;
use temper_worker_domain::{Invalid, agent, host};

/// The delivery the evidence records, if it decodes and agrees with
/// `outcome`.
///
/// # Errors
///
/// [`Invalid::Evidence`] for evidence that does not decode, or that records
/// another outcome.
pub fn decode(outcome: host::DeliveryOutcome, evidence: &[u8]) -> Result<agent::Delivery, Invalid> {
    let delivery = delivery(evidence).ok_or(Invalid::Evidence)?;
    let recorded = match &delivery {
        agent::Delivery::Delivered(_) => host::DeliveryOutcome::Delivered,
        agent::Delivery::Nothing => host::DeliveryOutcome::Nothing,
        agent::Delivery::Stale => host::DeliveryOutcome::Stale,
        agent::Delivery::Refused(_) => host::DeliveryOutcome::Refused,
        agent::Delivery::Failed(_) => host::DeliveryOutcome::Failed,
    };
    if recorded == outcome { Ok(delivery) } else { Err(Invalid::Evidence) }
}

fn delivery(evidence: &[u8]) -> Option<agent::Delivery> {
    let record = wire::DeliveryReply::decode(&wire::CEILINGS, &mut skein_lib::Reader::new(evidence)).ok()?;
    Some(match record.into_parts().value {
        wire::Delivery::Delivered(delivered) => {
            let mut receipts = Vec::new();
            for row in delivered.into_parts().receipts.into_boxed() {
                let parts = row.into_parts();
                receipts.push(agent::Receipt::new(parts.directory, parts.text)?);
            }
            agent::Delivery::Delivered(agent::Delivered::new(receipts.into_boxed_slice())?)
        }
        wire::Delivery::Nothing => agent::Delivery::Nothing,
        wire::Delivery::Stale => agent::Delivery::Stale,
        wire::Delivery::Refused(refusal) => {
            let parts = refusal.into_parts();
            let marker = match parts.marker {
                Some(marker) => {
                    let fields = marker.into_parts();
                    Some(agent::Marker::new(fields.directory, fields.path)?)
                }
                None => None,
            };
            agent::Delivery::Refused(agent::DeliveryRefusal::new(marker, parts.explanation)?)
        }
        wire::Delivery::Failed(failure) => {
            let parts = failure.into_parts();
            agent::Delivery::Failed(agent::DeliveryFailure {
                directory: parts.directory,
                reason: reason(&parts.reason),
                diagnostic: agent::Diagnostic::new(&parts.diagnostic, parts.dropped),
            })
        }
    })
}

fn reason(reason: &wire::DeliveryReason) -> agent::DeliveryReason {
    match *reason {
        wire::DeliveryReason::Unreachable => agent::DeliveryReason::Unreachable,
        wire::DeliveryReason::RefusedByTarget => agent::DeliveryReason::RefusedByTarget,
        wire::DeliveryReason::TimedOut => agent::DeliveryReason::TimedOut,
        wire::DeliveryReason::Broken => agent::DeliveryReason::Broken,
        wire::DeliveryReason::TooLarge => agent::DeliveryReason::TooLarge,
        wire::DeliveryReason::Missing => agent::DeliveryReason::Missing,
        wire::DeliveryReason::Busy => agent::DeliveryReason::Busy,
        wire::DeliveryReason::Unavailable => agent::DeliveryReason::Unavailable,
        wire::DeliveryReason::Cancelled => agent::DeliveryReason::Cancelled,
        wire::DeliveryReason::Unknown => agent::DeliveryReason::Unknown,
    }
}
