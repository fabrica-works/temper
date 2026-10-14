//! One call's exchanges, and its single terminal answer (forge.md, 3.2).
use temper_engine_domain_forge_client::api;
use temper_forge_forgejo::request;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Exchange {
    pub request: request::Request,
    pub range: Option<u32>,
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
