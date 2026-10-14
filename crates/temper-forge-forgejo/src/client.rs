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
pub(crate) mod branches;
mod items;
pub(crate) mod reviews;
use branches::BranchKind;
use reviews::ReviewKind;
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
        Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
        | Request::Pulls { .. } => reviews::encode(request, limits),
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
    Reviews(Box<reviews::Reviews>),
    Branches(Box<branches::Branches>),
}
#[must_use]
pub fn decoder(request: &Request, limits: &Limits) -> Option<Decoder> {
    let kind = match request {
        Request::Reviews { .. } => Kind::Review(ReviewKind::Reviews),
        Request::ReviewComments { .. } => Kind::Review(ReviewKind::Comments),
        Request::Branch { .. } => Kind::Branch(BranchKind::Branch),
        Request::Branches { .. } => Kind::Branch(BranchKind::Branches),
        Request::Protection { .. } => Kind::Branch(BranchKind::Protection),
        Request::Protections { .. } => Kind::Branch(BranchKind::Protections),
        Request::Permission { .. } => Kind::Branch(BranchKind::Permission),
        Request::Collaborators { .. } => Kind::Users,
        Request::Pulls { .. } => Kind::Item(Item::Pulls),
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
        Kind::Review(_) => DecoderState::Reviews(Box::new(reviews::Reviews::new())),
        Kind::Branch(kind) => DecoderState::Branches(Box::new(branches::Branches::new(kind, limits))),
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
            DecoderState::Reviews(state) => state.step(movement, &decoder.walk),
            DecoderState::Branches(state) => state.step(movement, &decoder.walk, limits),
            DecoderState::Items(state) => state.step(movement, &mut decoder.walk),
        },
    }
}
/// Skips earlier comments, including their bodies, before the first step.
pub fn keep_after(decoder: &mut Decoder, after: u64) {
    match &mut decoder.state {
        DecoderState::Items(state) => state.keep_after(after),
        DecoderState::Start(_) | DecoderState::Reviews(_) | DecoderState::Branches(_) => {}
    }
}
/// Stops emitting rows from an unpaged thread and counts its remaining elements.
pub fn cut(decoder: &mut Decoder) -> tokenizer::Request {
    decoder.walk.cut()
}
