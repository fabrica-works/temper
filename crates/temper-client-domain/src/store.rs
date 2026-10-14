//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use skein_lib::{RequestKey, RequestRecord};
pub const VERSION: u16 = 1;
/// A versioned child or request-table record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Jig(jig_domain_client::store::Record),
    Table { version: u16, record: RequestRecord<crate::Ask> },
}
/// The store key tagged with its owning child or table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Jig(jig_domain_client::store::StoreKey),
    Table(RequestKey),
}
