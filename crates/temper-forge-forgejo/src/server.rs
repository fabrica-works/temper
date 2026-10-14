//! The fake forge's side: origin targets and JSON answers.
use crate::{
    document::{ApiSettings, Repository, User},
    limits::Limits,
    request::{Method, Request},
};
use alloc::boxed::Box;
use skein_json::writer;
mod start;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unrouted {
    NotFound,
    Method,
    Malformed,
}
#[derive(Debug)]
pub enum Route {
    Ready(Request),
}
pub fn route(method: Method, target: &[u8], limits: &Limits) -> Result<Route, Unrouted> {
    start::route(method, target, limits)
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    User(User),
    ApiSettings(ApiSettings),
    Repository(Repository),
    Search { ok: bool, data: Box<[User]> },
}
pub fn encode(answer: &Answer, limits: &Limits) -> Result<Box<[u8]>, writer::Refusal> {
    let mut measure = writer::Encoder::measure(&limits.writer);
    start::write(&mut measure, answer);
    let length = measure.measured()?;
    let mut write = writer::Encoder::write(length, &limits.writer);
    start::write(&mut write, answer);
    Ok(write.finish())
}
