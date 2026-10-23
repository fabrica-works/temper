//! The forge family's client records (client/README.md, sections 4.6 and 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v1.rs"]
pub mod v1;

use alloc::boxed::Box;
use skein_lib::{Reader, Writer};

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
pub const CLIENT_VERSION: u32 = 7;

/// The deployment's maximum number of forges (client/README.md, 4.6).
pub const FORGES: u32 = 16;
/// The box's maximum setup failure line length (client/README.md, 4.6).
pub const SET_UP_WHY_BYTES: u32 = 256;

/// The demo's maximum number of suggested requests (client/README.md, 4.6).
pub const CUES: u32 = 8;
/// Each suggested request's maximum UTF-8 byte length (client/README.md, 4.6).
pub const CUE_BYTES: u32 = 256;

/// The pull request address's maximum byte length (client/README.md, 4.6).
pub const ADDRESS_BYTES: u32 = 2048;
/// The change panel's maximum number of gates (client/README.md, 4.6).
pub const GATES: u32 = 64;
/// The queued change's maximum number of tasks ahead (client/README.md, 4.6).
pub const TASKS_AHEAD: u32 = 64;

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
        forges_forges: FORGES,
        return_path: 512,
        set_up_failed_why: SET_UP_WHY_BYTES,
        connected_login: bounds.field_bytes,
        stopped_reason: bounds.field_bytes,
        pull_address: ADDRESS_BYTES,
        queued_ahead: TASKS_AHEAD,
        change_panel_owner: bounds.field_bytes,
        change_panel_name: bounds.field_bytes,
        change_panel_branch: bounds.field_bytes,
        change_panel_landing: bounds.field_bytes,
        change_panel_gates: GATES,
        demo_answer_owner: bounds.field_bytes,
        demo_answer_name: bounds.field_bytes,
        demo_answer_cues: CUES,
        demo_answer_cues_item: CUE_BYTES,
        set_up_org: bounds.field_bytes,
        set_up_repository: bounds.field_bytes,
        new_project_name: bounds.field_bytes,
        first_repository_org: bounds.field_bytes,
        first_repository_repository: bounds.field_bytes,
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
pub fn requesting(record: &v1::Requesting) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Encode the outcome for jig's owned record.
#[must_use]
pub fn answer(record: &v1::Answer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

#[cfg(test)]
mod tests;

/// Encode which of the family's reads is requested.
#[must_use]
pub fn reading(record: &v1::Reading) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete family reading.
#[must_use]
pub fn decode_reading(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Reading> {
    let mut reader = Reader::new(bytes);
    let record = v1::Reading::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}
/// Encode the demo's answer; no demo is represented by no record.
#[must_use]
pub fn demo_answer(record: &v1::DemoAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode a complete demo answer, refusing empty cues (client/README.md, 4.6).
#[must_use]
pub fn decode_demo_answer(bytes: &[u8], limits: &v1::Limits) -> Option<v1::DemoAnswer> {
    let mut reader = Reader::new(bytes);
    let record = v1::DemoAnswer::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    for cue in record.cues() {
        if cue.is_empty() {
            return None;
        }
    }
    Some(record)
}

/// Encode which of the family's subjects is watched (client/README.md, 4.6).
#[must_use]
pub fn watching(record: &v1::Watching) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode one complete family watch (client/README.md, 4.6).
#[must_use]
pub fn decode_watching(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Watching> {
    let mut reader = Reader::new(bytes);
    let record = v1::Watching::decode(limits, &mut reader).ok()?;
    if reader.is_empty() { Some(record) } else { None }
}
/// Encode the change's whole panel (client/README.md, 4.6; client/forge.md, 9.2).
#[must_use]
pub fn change_panel(record: &v1::ChangePanel) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}
/// Decode a complete panel, refusing a queued place before 1 (client/README.md, 4.6).
#[must_use]
pub fn decode_change_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ChangePanel> {
    let mut reader = Reader::new(bytes);
    let record = v1::ChangePanel::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    if let v1::Step::Queued(queued) = record.step()
        && queued.place() == 0
    {
        return None;
    }
    Some(record)
}

/// Decode one complete requesting (client/README.md, 4.6).
#[must_use]
pub fn decode_requesting(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Requesting> {
    let mut reader = Reader::new(bytes);
    let record = v1::Requesting::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(record)
}

/// Encode the family's forges (client/README.md, 4.6).
#[must_use]
pub fn forges(record: &v1::Forges) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
    record.encode(&mut writer).expect("measured room");
    writer.finish()
}

/// Decode one complete forges (client/README.md, 4.6).
#[must_use]
pub fn decode_forges(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Forges> {
    let mut reader = Reader::new(bytes);
    let record = v1::Forges::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let mut previous = None;
    for row in record.forges() {
        if let Some(number) = previous
            && number >= row.number()
        {
            return None;
        }
        previous = Some(row.number());
    }
    Some(record)
}
