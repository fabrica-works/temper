//! HTTP refusals of JSON reads (forge.md, 3.4).
use crate::connection::{Failure, Fields};
use skein_lib::Duration;
use temper_engine_domain_forge_client::api;
pub(crate) fn read(status: u16, fields: &Fields) -> api::Error {
    match status {
        300..=399 => api::Error::Unavailable,
        401 | 403 | 423 => api::Error::Forbidden,
        404 => api::Error::Missing,
        413 => api::Error::Full,
        429 => api::Error::RateLimited { after: fields.retry_after.unwrap_or(Duration::ZERO) },
        500..=599 => api::Error::Timeout,
        _ => api::Error::InvalidAnswer,
    }
}
pub(crate) fn transport(failure: Failure) -> api::Error {
    match failure {
        Failure::Unseen => api::Error::Unavailable,
        Failure::Seen => api::Error::Timeout,
    }
}
