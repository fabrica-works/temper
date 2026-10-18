//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{Address, Place};
use skein_lib::List;
/// A view act routed exhaustively to its child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Forge(crate::forge::Act),
    Pages(crate::pages::Act),
    Jig(jig_client_view::action::Act),
}
/// One external fact or act for the root to route.
#[derive(Debug)]
pub enum Event {
    Forge(crate::forge::Event),
    Start { address: Address, kept: List<jig_client::Record>, offset: jig_client_view::boundary::Offset },
    Went { place: Place },
    Act { action: Action },
    Client { event: jig_client::Event },
    Jig(jig_client_view::Event),
    SignedOut,
}
/// Bounded routing and child facts for the shell to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    PartsFull,
    InvalidRecord,
    UnknownConnector,
    UnknownAsker,
}
/// One shell obligation; keyed sends come only from the table.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Client { request: jig_client::Request },
    Show { address: Address, replace: bool },
    SignOut,
    Dropped { count: u32 },
    Fact { fact: Fact },
}
