//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
use crate::{
    Address, Place,
    store::{Key, Record},
};
use alloc::boxed::Box;
use jig_client::contract::{Ask as JigAsk, Read, Subject};
use skein_lib::{List, RequestEnvelope, RequestKey, Token};
/// One keyed request, tagged with the child that owns its meaning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    Forge(crate::forge::Ask),
    Jig(Box<JigAsk>),
}
/// A decoded terminal tagged with its owning child.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Forge(crate::forge::Answer),
    Jig(jig_client::contract::Answer),
}
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
    Start { address: Address, kept: List<Record>, offset: jig_client_view::boundary::Offset },
    Went { place: Place },
    Act { action: Action },
    Answer { attempt: Token, envelope: RequestEnvelope, answer: Option<Answered> },
    Jig(jig_client_view::Event),
    SignedOut,
}
/// An unkeyed jig shell obligation passed through the root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JigOut {
    Read { read: Token, what: Read },
    Open { watch: Token, subject: Subject },
    Continue { watch: Token, subject: Subject },
    Close { watch: Token },
    Answered { asked: Token, answer: jig_client::contract::Answer },
    Unknown { asked: Token },
}
/// Bounded routing and child facts for the shell to observe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Jig(jig_client_view::facts::Fact),
    TableFull,
    PartsFull,
    InvalidRecord,
    InvalidAnswer,
    UnknownConnector,
    KeptAnswered,
}
/// One shell obligation; keyed sends come only from the table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Save { record: Record },
    Erase { key: Key },
    Send { attempt: Token, key: RequestKey, ask: Ask },
    Jig(JigOut),
    Show { address: Address, replace: bool },
    SignIn { provider: u16, back: Address },
    SignOut,
    Fact { fact: Fact },
}
