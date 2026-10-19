//! The forge family's client records (client/README.md, sections 4.6 and 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v1.rs"]
pub mod v1;

use alloc::boxed::Box;
use skein_lib::Writer;

/// The forge connector's number on jig's client wire (client/README.md, 4.6).
pub const FORGE: u16 = 1;
/// The application's stable owners on jig's client wire (client/README.md, 4.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// The forge's requests and records.
    Forge,
    /// The application's pages, using jig's operations and subjects.
    Pages,
    /// Calls made by the tools runner.
    Runner,
}
/// The owner's number, never assigned to another owner.
#[must_use]
pub const fn number(owner: Owner) -> u16 {
    match owner {
        Owner::Forge => FORGE,
        Owner::Pages => 2,
        Owner::Runner => 3,
    }
}
/// The owner a number names, if this build knows it.
#[must_use]
pub const fn owner(number: u16) -> Option<Owner> {
    match number {
        FORGE => Some(Owner::Forge),
        2 => Some(Owner::Pages),
        3 => Some(Owner::Runner),
        _ => None,
    }
}
/// Every record's version, served by both ends (client/README.md, 4.6).
pub const CLIENT_VERSION: u32 = 1;

/// A side's bounds on the forge family's fields (client/README.md, 4.4 and 4.6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    pub field_bytes: u32,
    pub checks: u32,
    pub contexts: u32,
    pub refusal_bytes: u32,
}

/// Set the schema limits from their source fields, refusing past a ceiling.
#[must_use]
pub fn limits(bounds: &Bounds) -> Option<v1::Limits> {
    let limits = v1::Limits {
        adopt_owner: bounds.field_bytes,
        adopt_name: bounds.field_bytes,
        adopt_prefix: bounds.field_bytes,
        adopt_landing: bounds.field_bytes,
        adopted_owner: bounds.field_bytes,
        adopted_name: bounds.field_bytes,
        rule_branch: bounds.field_bytes,
        adopt_checks: bounds.checks,
        rule_contexts: bounds.contexts,
        rule_contexts_item: bounds.field_bytes,
        core_refused_refusal: bounds.refusal_bytes,
    };
    match v1::check_limits(&limits) {
        Ok(()) => Some(limits),
        Err(_) => None,
    }
}

/// Encode the request for jig's keyed record and the client's small store.
#[must_use]
pub fn adopt(record: &v1::Adopt) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode the outcome for jig's owned record.
#[must_use]
pub fn answer(record: &v1::AdoptAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

#[cfg(test)]
mod tests;
