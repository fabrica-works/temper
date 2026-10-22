//! The deployment's configured forges and their set up (client/README.md, 4.6).
use super::{family, wire_limits};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Forgejo,
    GitHub,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Connected,
    Managed,
    Owned,
    Fake,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Fetch,
    Prepare,
    Run,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offer {
    NotOffered,
    NotUnderTheBox,
    Offered,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Connection {
    NotConnected,
    NotSetUp,
    SettingUp { step: Option<Step> },
    SetUpFailed { why: Box<[u8]> },
    Connecting,
    Connected { id: u64, login: Box<[u8]> },
    Stopped { reason: Box<[u8]> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    pub connection: Connection,
    pub offer: Offer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetUpOutcome {
    SettingUp,
    Refused(SetUpRefusal),
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetUpRefusal {
    NotOffered,
    NotUnderTheBox,
    Busy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetUpAnswer {
    Outcome(SetUpOutcome),
    EntranceRefused(c::EntranceRefusal),
}
#[must_use]
pub fn watch(limits: &super::Limits) -> Option<Watchable> {
    let limits = wire_limits(limits)?;
    let watching = w::Watching::new(&limits, w::WatchingParts { watch: w::Watch::Forges }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Deployment,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) },
    })
}
#[must_use]
pub fn ask(forge: u16, limits: &super::Limits) -> Option<Ask> {
    Some(Ask::Family { scope: c::Scope::Deployment, record: family::set_up(forge, &wire_limits(limits)?)? })
}
#[must_use]
pub fn answer(answer: &Answered, limits: &super::Limits) -> Option<SetUpAnswer> {
    match answer {
        Answered::Family(record) => Some(SetUpAnswer::Outcome(family::set_up_answer(record, &wire_limits(limits)?)?)),
        Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            Some(SetUpAnswer::EntranceRefused(c::EntranceRefusal::new(refusal.clone()).ok()?))
        }
        Answered::Jig(c::Answer::Done(_)) => None,
    }
}
/// Invalid documents are refused by the watch's owner, which unwatches its handle.
#[must_use]
pub fn rows(record: &Family, limits: &super::Limits) -> Option<Box<[Row]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    family::forges(&record.bytes, &wire_limits(limits)?)
}
