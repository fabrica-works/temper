//! The deployment's configured forges and their set up (client/README.md, 4.6).
use super::{family, wire_limits};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};

/// The wire bounds the session's forge rows.
pub const ROWS: u32 = wire::FORGES;
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
/// The project selected for the first repository (protocol/setup.md, 6.7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetUpProject {
    Existing(u32),
    New(Box<[u8]>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateStep {
    Admin,
    Org,
    Repository,
    File,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstState {
    Making,
    Made { id: u64 },
    Failed { step: CreateStep },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct First {
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub project: SetUpProject,
    pub state: FirstState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    pub connection: Connection,
    pub offer: Offer,
    pub first: Option<First>,
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
    NewProject,
    Invalid,
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
pub fn ask(
    forge: u16,
    org: Box<[u8]>,
    repository: Box<[u8]>,
    project: SetUpProject,
    limits: &super::Limits,
) -> Option<Ask> {
    Some(Ask::Family {
        scope: c::Scope::Deployment,
        record: family::set_up(forge, org, repository, project, &wire_limits(limits)?)?,
    })
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

/// The session keeps an ended watch until the next sign-in (client/README.md, 4.6).
#[derive(Debug)]
pub enum State {
    Unknown,
    Rows(Box<[Row]>),
    Ended,
}
/// The last set up sent by this client, retained across page changes.
#[derive(Debug)]
pub struct SetUp {
    pub forge: u16,
    pub request: skein_lib::Token,
    pub progress: Option<skein_lib::RequestProgress>,
    pub answer: Option<SetUpAnswer>,
}
pub(crate) fn handle() -> jig_client::Handle {
    jig_client::Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::FORGES }
}
pub(crate) fn configure(side: &mut super::Side, limits: super::Limits, out: &mut skein_lib::Queue<super::Request>) {
    use jig_client::Link;
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        if side.forges_open {
            side.forges_open = false;
            out.push(super::Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
        }
        side.forges = State::Unknown;
        side.changed();
        return;
    }
    if side.party.is_some()
        && !side.forges_open
        && let State::Unknown = side.forges
    {
        if let Some(subject) = watch(&limits) {
            side.forges_open = true;
            out.push(super::Request::Client { asked: jig_client::Event::Watch { handle: handle(), subject } });
        } else {
            side.forges = State::Ended;
        }
        side.changed();
    }
}
pub(crate) fn set_up(
    side: &mut super::Side,
    limits: super::Limits,
    forge: u16,
    out: &mut skein_lib::Queue<super::Request>,
) {
    if side.party.is_none() || side.served == super::Served::OutOfDate {
        return;
    }
    if let Some(last) = &side.set_up
        && last.progress.is_some()
    {
        return;
    }
    let Some(ask) =
        ask(forge, b"work".as_slice().into(), b"hello".as_slice().into(), SetUpProject::Existing(1), &limits)
    else {
        return;
    };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = skein_lib::Token::new(side.serial);
    side.set_up = Some(SetUp { forge, request, progress: Some(skein_lib::RequestProgress::Parked), answer: None });
    out.push(super::Request::Send { request, ask });
    side.changed();
}
pub(crate) fn told(
    side: &mut super::Side,
    client: &jig_client::Client,
    value: &jig_client::Told,
    limits: super::Limits,
    out: &mut skein_lib::Queue<super::Request>,
) -> bool {
    use jig_client::{Latest, Outcome, Progress, Told};
    match value {
        Told::Changed { handle: changed } if *changed == handle() => {
            side.forges = match client.document(*changed) {
                Some(document) if document.state == jig_client::State::Ended => State::Ended,
                Some(document) => match document.latest {
                    Some(Latest::Family(record)) => match rows(record, &limits) {
                        Some(rows) => State::Rows(rows),
                        None => State::Ended,
                    },
                    Some(Latest::Jig(_)) => State::Ended,
                    None => State::Unknown,
                },
                None => State::Unknown,
            };
            if let State::Ended = side.forges {
                end(side, out);
            }
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => {
            side.forges = State::Ended;
            end(side, out);
        }
        Told::Pending { key } if setup_key(side, *key) => {
            let Some(last) = &mut side.set_up else { return true };
            let Some(pending) = client.pending(*key) else { return true };
            let progress = match pending.progress {
                Progress::InFlight => skein_lib::RequestProgress::InFlight,
                Progress::Retrying { .. } => skein_lib::RequestProgress::Retrying,
                Progress::NotServed | Progress::Parked => skein_lib::RequestProgress::Parked,
                Progress::Unknown => skein_lib::RequestProgress::Unknown,
            };
            if progress == skein_lib::RequestProgress::Unknown {
                last.progress = None;
                last.answer = Some(SetUpAnswer::Outcome(SetUpOutcome::Unknown));
            } else {
                last.progress = Some(if side.served == super::Served::OutOfDate {
                    skein_lib::RequestProgress::Parked
                } else {
                    progress
                });
            }
        }
        Told::Answered { key, outcome } if setup_key(side, *key) => {
            let Some(last) = &mut side.set_up else { return true };
            last.progress = None;
            last.answer = Some(match outcome {
                Outcome::Answered(answered) => {
                    answer(answered, &limits).unwrap_or(SetUpAnswer::Outcome(SetUpOutcome::Unknown))
                }
                Outcome::Unknown
                | Outcome::Limit
                | Outcome::Held
                | Outcome::NotKept
                | Outcome::Unsent
                | Outcome::Lost
                | Outcome::Unconfirmed => SetUpAnswer::Outcome(SetUpOutcome::Unknown),
            });
        }
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    }
    side.changed();
    true
}
fn setup_key(side: &super::Side, key: jig_client::Key) -> bool {
    let Some(last) = &side.set_up else { return false };
    key.asker == crate::owner::Owner::Forge.asker() && key.number == last.request.raw()
}
fn end(side: &mut super::Side, out: &mut skein_lib::Queue<super::Request>) {
    if side.forges_open {
        side.forges_open = false;
        out.push(super::Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
}
#[must_use]
pub fn worst_case(limits: &super::Limits) -> Option<u64> {
    let wire = wire_limits(limits)?;
    let rows = u64::from(temper_client_protocol_wire::FORGES);
    w::Forges::worst_case_heap(&wire)?
        .checked_mul(2)?
        .checked_add(rows.checked_mul(u64::try_from(size_of::<Row>()).ok()?)?)
}
