//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
/// An application destination tagged with its owning child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Address {
    Forge(crate::forge::Address),
    Jig(jig_domain_client::address::Address),
}
/// A decoded location or a missing address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    Address(Address),
    Missing,
}
