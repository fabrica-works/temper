//! Lent account credentials stay in the protocol layer (credentials.md, 3).
use jig_protocol_accounts::table::Table;
#[derive(Debug)]
pub struct Values {
    pub accounts: Table,
}
impl Values {
    /// Numbered account slots, including the unused zero slot.
    #[must_use]
    pub fn new(accounts: u32, token_bytes: u32) -> Self {
        Self { accounts: Table::new(accounts, token_bytes) }
    }
}
