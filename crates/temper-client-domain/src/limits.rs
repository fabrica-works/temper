//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{Ask, Request, forge, owner::Owner};
use skein_lib::{Queue, RequestLimits, RequestTable};
/// Application-configured child and request-table bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub jig: jig_domain_client::Limits,
    pub forge: forge::Limits,
    pub table: RequestLimits,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    checked_max_out(limits).expect("root output capacities are checked")
}
fn checked_max_out(limits: &Limits) -> Option<u32> {
    jig_domain_client::max_out(&limits.jig).checked_mul(4)?.checked_add(limits.table.out)?.checked_add(16)
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    if limits.jig.kept < limits.table.requests
        || limits.jig.navigation_entries < 3
        || limits.table.out == 0
        || limits.table.first == skein_lib::Duration::ZERO
        || limits.table.first > limits.table.most
    {
        return None;
    }
    let mut total = jig_domain_client::worst_case(&limits.jig)?;
    total = total.checked_add(skein_lib::Map::<u64, u32>::worst_case(limits.table.requests)?)?;
    total = total.checked_add(forge::worst_case(&limits.forge)?)?;
    // Held parts and the PartsLeft survivor list each own their part payloads.
    total = total.checked_add(u64::from(limits.forge.parts).checked_mul(limits.jig.document_bytes)?.checked_mul(2)?)?;
    total = total.checked_add(RequestTable::<Owner, Ask>::worst_case(&limits.table)?)?;
    total = total.checked_add(limits.table.bytes.checked_mul(3)?)?;
    let jig_out = jig_domain_client::max_out(&limits.jig);
    let root_out = checked_max_out(limits)?;
    total = total.checked_add(Queue::<jig_domain_client::Request>::worst_case(jig_out)?.checked_mul(3)?)?;
    total = total.checked_add(Queue::<forge::Request>::worst_case(forge::max_out(&limits.forge))?)?;
    total = total.checked_add(u64::from(jig_out).checked_mul(limits.jig.document_bytes)?.checked_mul(3)?)?;
    total = total.checked_add(u64::from(root_out).checked_mul(limits.jig.document_bytes)?)?;
    total.checked_add(Queue::<Request>::worst_case(root_out)?)
}
