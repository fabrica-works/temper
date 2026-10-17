//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{Address, Place};
use skein_lib::List;
/// A view act routed exhaustively to its child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Forge(crate::forge::Act),
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
/// An unkeyed jig shell obligation passed through the root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JigOut {
    Answered { asked: skein_lib::Token, answer: jig_client::contract::Answer },
    Unknown { asked: skein_lib::Token },
}
/// Bounded routing and child facts for the shell to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Jig(jig_client_view::facts::Fact),
    PartsFull,
    InvalidRecord,
    InvalidAnswer,
    UnknownConnector,
    UnknownAsker,
}
/// One shell obligation; keyed sends come only from the table.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Client { request: jig_client::Request },
    Jig(JigOut),
    Show { address: Address, replace: bool },
    SignIn { provider: u16, back: Address },
    SignOut,
    Fact { fact: Fact },
}
