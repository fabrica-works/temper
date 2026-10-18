//! One call's exchanges, and its single terminal answer (protocol/forge.md, 3.2).
use temper_engine_domain_forge_client::api;
use temper_forge_forgejo::request;
/// The bytes requested from a raw answer.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Range {
    Head(u32),
    Tail(u32),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Exchange {
    pub request: request::Request,
    pub range: Option<Range>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Step {
    Send(Exchange),
    Done(Result<api::Answer, api::Error>),
}
impl Step {
    pub fn send(request: request::Request) -> Self {
        Self::Send(Exchange { request, range: None })
    }
}
use crate::{
    connection::Heard,
    limits::{Limits, Shared},
    tables::Tables,
};
use alloc::boxed::Box;
#[derive(Debug)]
pub(crate) enum Plan {
    Read(Box<crate::reads::Plan>),
    Write(Box<crate::writes::Plan>),
}
impl Plan {
    pub fn error(&self, error: api::Error) -> api::Error {
        match self {
            Self::Read(_) => error,
            Self::Write(plan) => plan.error(error),
        }
    }
    pub fn new(repository: api::Repository, op: api::Op, cap: u32) -> Self {
        match op {
            api::Op::Read(read) => Self::Read(Box::new(crate::reads::Plan::new(repository, read, cap))),
            api::Op::Write(write) => Self::Write(Box::new(crate::writes::Plan::new(repository, write))),
        }
    }
    pub fn begin(&mut self, l: &Limits, shared: &Shared, tables: &mut Tables) -> Result<Step, api::Error> {
        match self {
            Self::Read(plan) => plan.begin(l, shared, tables),
            Self::Write(plan) => plan.begin(l, tables),
        }
    }
    pub fn heard(
        &mut self,
        heard: Heard,
        l: &Limits,
        shared: &Shared,
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        match self {
            Self::Read(plan) => plan.heard(heard, l, shared, tables),
            Self::Write(plan) => plan.heard(heard, l, tables),
        }
    }
    pub fn pending(&mut self) -> Option<Exchange> {
        match self {
            Self::Read(plan) => plan.pending(),
            Self::Write(plan) => plan.pending(),
        }
    }
    pub fn set_pending(&mut self, request: Exchange) {
        match self {
            Self::Read(plan) => plan.set_pending(request),
            Self::Write(plan) => plan.set_pending(request),
        }
    }
    pub fn after(&self) -> u64 {
        match self {
            Self::Read(plan) => plan.after(),
            Self::Write(_) => 0,
        }
    }
    pub fn raw_room(&self, shared: &Shared) -> u32 {
        match self {
            Self::Read(plan) => plan.raw_room(shared),
            Self::Write(_) => shared.client.answer_bytes,
        }
    }
    pub fn take_cut(&mut self) -> bool {
        match self {
            Self::Read(plan) => plan.take_cut(),
            Self::Write(_) => false,
        }
    }
}
