//! The engine's side: request spelling and streaming answer decoding.
use crate::{
    decode::{
        Step,
        walk::{Kind, Walk},
    },
    limits::Limits,
    request::{Method, Request},
};
use alloc::boxed::Box;
use skein_json::tokenizer;
mod start;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Encoded {
    pub method: Method,
    pub target: Box<[u8]>,
    pub body: Option<Box<[u8]>>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    Text,
    TooLong,
}
pub fn encode(request: &Request, limits: &Limits) -> Result<Encoded, Refusal> {
    start::encode(request, limits)
}
#[derive(Debug)]
pub struct Decoder {
    walk: Walk,
    start: start::Start,
}
#[must_use]
pub fn decoder(request: &Request, limits: &Limits) -> Option<Decoder> {
    let kind = match request {
        Request::User => Kind::User,
        Request::ApiSettings => Kind::Settings,
        Request::RepositoryById { .. } | Request::Repository { .. } => Kind::Repository,
        Request::UserSearch { .. } => Kind::Search,
    };
    Some(Decoder { walk: Walk::new(kind, limits), start: start::Start::new(kind) })
}
#[must_use]
pub fn first(_decoder: &Decoder) -> tokenizer::Request {
    tokenizer::Request::Next
}
pub fn step(decoder: &mut Decoder, limits: &Limits, event: &tokenizer::Event) -> Step {
    match decoder.walk.step(event, limits) {
        Err(error) => Step::Malformed(error),
        Ok(movement) => decoder.start.step(movement, decoder.walk.parent(), event),
    }
}
