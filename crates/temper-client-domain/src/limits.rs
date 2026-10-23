//! The pages and forge share jig's receiving and request bounds.
use crate::{Request, forge};
use alloc::boxed::Box;
use skein_lib::Queue;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub forge: forge::Limits,
    pub client: jig_client::Limits,
    pub accounts: AccountLimits,
}
/// Interim view counts until jig owns receiving counts (owner, 17:51Z; x1g).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccountLimits {
    pub accounts: u32,
    pub sources: u32,
    pub uses: u32,
    pub sign_ins: u32,
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    jig_client::max_out(&limits.client)
        .checked_mul(4)
        .expect("root output bound fits")
        .checked_add(1)
        .expect("welcome output bound fits")
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let mut total = crate::pages::task::worst_case(limits)?;
    total = total.checked_add(crate::pages::choice::worst_case(limits)?)?;
    total = total.checked_add(crate::pages::accounts::worst_case(limits)?)?;
    total = total
        .checked_add(u64::try_from(size_of::<crate::pages::connect_model::ConnectModel>()).ok()?)?
        .checked_add(u64::try_from(crate::pages::accounts::KEY_BYTES).ok()?)?
        .checked_add(limits.client.domain.document_bytes)?
        .checked_add(32)?;
    total = total.checked_add(crate::pages::inbox::worst_case(limits)?)?;
    total = total.checked_add(crate::pages::chats::worst_case(limits)?)?;
    total = total.checked_add(crate::pages::chat::worst_case(limits)?)?;
    total = total.checked_add(Queue::<crate::pages::Request>::worst_case(crate::pages::max_out(limits))?)?;
    total = total.checked_add(forge::worst_case(&limits.forge)?)?;
    total = total.checked_add(limits.client.domain.document_bytes)?;
    total = total.checked_add(jig_client::worst_case(&limits.client)?)?;
    total = total.checked_add(Queue::<forge::Request>::worst_case(forge::max_out(&limits.forge))?)?;
    total = total.checked_add(u64::from(max_out(limits)).checked_mul(limits.client.domain.document_bytes)?)?;
    total = total.checked_add(limits.client.domain.document_bytes)?;
    total = total.checked_add(u64::try_from(size_of::<Option<(jig_client::Key, Box<[u8]>)>>()).ok()?)?;
    total.checked_add(Queue::<Request>::worst_case(max_out(limits))?)
}
