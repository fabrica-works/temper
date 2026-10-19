//! A table row's typed arguments (tools contract, 5.1).
use super::key::Name;
use alloc::boxed::Box;
use jig_client::contract::{InboxGroup, QueryOrder, TurnName};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub tool: u8,
    pub form: u8,
    pub values: Box<[Option<Values>]>,
    pub key: Option<Name>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Values {
    One(Value),
    Many(Box<[Value]>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Number(u64),
    Word(u8),
    Text(Box<[u8]>),
    Stdin,
    File(Box<[u8]>),
    Switch,
    Project(Project),
    Turn(TurnName),
    Cursor(Box<[u8]>),
    Group(InboxGroup),
    Order(QueryOrder),
    Tool(u8),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Project {
    Number(u32),
    Name(Box<[u8]>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    Call { call: Call, json: bool },
    File { file: Source, json: bool },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Path(Box<[u8]>),
    Stdin,
}
