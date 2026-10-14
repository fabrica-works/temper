//! Temper's ordered families under jig's application tag (store/README.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub mod keys;
pub mod records;
#[rustfmt::skip]
#[path = "generated/v1.rs"]
pub mod v1;
#[cfg(test)]
mod tests;

/// One version for every durable family (store/README.md, section 4).
pub const STORE_VERSION: u32 = 1;

/// A malformed row, which the routing reports before stopping the engine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Undecodable;

/// Schema limits checked once alongside the forge's key bounds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    limits: v1::Limits,
    forge: temper_engine_domain_forge::Limits,
}
impl Bounds {
    /// The generated decoder's limits.
    #[must_use]
    pub const fn limits(&self) -> &v1::Limits {
        &self.limits
    }
    /// The domain limits for names and key decoding.
    #[must_use]
    pub const fn forge(&self) -> &temper_engine_domain_forge::Limits {
        &self.forge
    }
}

/// Make schema bounds from the domain, refusing any ceiling overflow.
#[must_use]
pub fn bounds(forge: &temper_engine_domain_forge::Limits) -> Option<Bounds> {
    let limits = v1::Limits {
        segments_segments: forge.name_bytes.checked_mul(4)?,
        protection_branch: forge.client.answer_bytes,
        protection_contexts: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        settings_default_branch: forge.client.answer_bytes,
        adopted_host: forge.name_bytes,
        adopted_owner: forge.name_bytes,
        adopted_name: forge.name_bytes,
        adopted_prefix: forge.name_bytes,
        adopted_checks: forge.change_policy.gates,
        retargeted_before: forge.name_bytes,
        retargeted_after: forge.name_bytes,
    };
    v1::check_limits(&limits).ok()?;
    Some(Bounds { limits, forge: *forge })
}

/// Every application key, relative to jig's application tag.
#[must_use]
pub fn interval() -> skein_kv::Range {
    skein_kv::Range { start: Box::new([]), end: None }
}
