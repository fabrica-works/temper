//! Temper's ordered families under jig's application tag (store/README.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub mod keys;
#[cfg(test)]
mod tests;

/// One version for every durable family (store/README.md, section 4).
pub const STORE_VERSION: u32 = 1;

/// A malformed row, which the routing reports before stopping the engine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Undecodable;

/// Every application key, relative to jig's application tag.
#[must_use]
pub fn interval() -> skein_kv::Range {
    skein_kv::Range { start: Box::new([]), end: None }
}
