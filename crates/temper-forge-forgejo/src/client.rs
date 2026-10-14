//! The engine's side: request spelling and streaming answer decoding.
use crate::{
    decode::{
        Step,
        walk::{Item, Kind, Walk},
    },
    limits::Limits,
    request::{Method, Request},
};
use alloc::boxed::Box;
use skein_json::tokenizer;
mod items;
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
    match request {
        Request::User
        | Request::ApiSettings
        | Request::RepositoryById { .. }
        | Request::Repository { .. }
        | Request::UserSearch { .. } => start::encode(request, limits),
        Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::CombinedStatus { .. } => items::encode(request, limits),
    }
}
#[derive(Debug)]
pub struct Decoder {
    walk: Walk,
    state: DecoderState,
}
#[derive(Debug)]
enum DecoderState {
    Start(start::Start),
    Items(Box<items::Items>),
}
#[must_use]
pub fn decoder(request: &Request, limits: &Limits) -> Option<Decoder> {
    let kind = match request {
        Request::User => Kind::User,
        Request::ApiSettings => Kind::Settings,
        Request::RepositoryById { .. } | Request::Repository { .. } => Kind::Repository,
        Request::UserSearch { .. } => Kind::Search,
        Request::Issues { .. } => Kind::Item(Item::Issues),
        Request::Issue { .. } => Kind::Item(Item::Issue),
        Request::Comments { .. } => Kind::Item(Item::Comments),
        Request::Pull { .. } | Request::PullFor { .. } => Kind::Item(Item::Pull),
        Request::CombinedStatus { .. } => Kind::Item(Item::Combined),
    };
    let state = match kind {
        Kind::Item(item) => DecoderState::Items(Box::new(items::Items::new(item, limits))),
        Kind::User | Kind::Settings | Kind::Repository | Kind::Search | Kind::Users => {
            DecoderState::Start(start::Start::new(kind))
        }
    };
    Some(Decoder { walk: Walk::new(kind, limits), state })
}
#[must_use]
pub fn first(_decoder: &Decoder) -> tokenizer::Request {
    tokenizer::Request::Next
}
pub fn step(decoder: &mut Decoder, limits: &Limits, event: &tokenizer::Event) -> Step {
    match decoder.walk.step(event, limits) {
        Err(error) => Step::Malformed(error),
        Ok(movement) => match &mut decoder.state {
            DecoderState::Start(state) => state.step(movement, decoder.walk.parent()),
            DecoderState::Items(state) => state.step(movement, &mut decoder.walk),
        },
    }
}
/// Skips earlier comments, including their bodies, before the first step.
pub fn keep_after(decoder: &mut Decoder, after: u64) {
    match &mut decoder.state {
        DecoderState::Items(state) => state.keep_after(after),
        DecoderState::Start(_) => {}
    }
}
/// Stops emitting rows from an unpaged thread and counts its remaining elements.
pub fn cut(decoder: &mut Decoder) -> tokenizer::Request {
    decoder.walk.cut()
}
