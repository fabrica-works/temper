//! The deployment's demo read and its two independent halves (client/README.md, 4.8).
use super::{Demo, Limits, Request, Side, family, wire_limits};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Link, Readable, Readout, Told, contract as c};
use skein_lib::{Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent session lifecycle facts (client/README.md, 4.8)")]
pub(super) struct State {
    pub answer: Option<Demo>,
    pub pending: Option<Handle>,
    pub live: bool,
    pub answered: bool,
    pub start: Option<Starting>,
    pub failure: Option<StartFailure>,
    pub sent: bool,
    pub removed: bool,
    pub removal: Option<(Handle, u32)>,
    pub model_ended: bool,
    pub forge_ended: bool,
}
#[derive(Debug)]
pub(super) struct Starting {
    pub request: Token,
    pub silent: bool,
}
impl State {
    pub const fn new() -> Self {
        Self {
            answer: None,
            pending: None,
            live: false,
            answered: false,
            start: None,
            failure: None,
            sent: false,
            removed: false,
            removal: None,
            model_ended: false,
            forge_ended: false,
        }
    }
}
/// A link tell remains available to the other children; every forge read page is consumed.
pub(crate) fn told(side: &mut Side, told: &Told, limits: Limits, out: &mut Queue<Request>) -> bool {
    match told {
        Told::Link { link } => {
            if *link == Link::Live && !side.demo.live {
                side.demo.live = true;
                read(side, out);
            } else if *link != Link::Live {
                side.demo.live = false;
            }
            false
        }
        Told::Page { handle, page } if handle.asker == crate::owner::Owner::Forge.asker() => {
            if Some(*handle) == side.demo.pending {
                side.demo.pending = None;
                let answer = match page {
                    Readout::Family { family: number, record: Some(record) } if *number == wire::FORGE => {
                        match wire_limits(&limits) {
                            Some(limits) => family::demo(record, &limits),
                            None => None,
                        }
                    }
                    Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit | Readout::Jig(_) => None,
                };
                let first = !side.demo.answered;
                side.demo.answered = true;
                if first || side.demo.answer != answer {
                    side.demo.answer = answer;
                    side.changed();
                }
                configure(side, limits, out);
            }
            true
        }
        Told::Answered { key, outcome } if start_key(side, *key) => {
            let result = match outcome {
                jig_client::Outcome::Answered(value) => answer(value, &limits),
                jig_client::Outcome::Unknown
                | jig_client::Outcome::Limit
                | jig_client::Outcome::Held
                | jig_client::Outcome::NotKept
                | jig_client::Outcome::Unsent
                | jig_client::Outcome::Lost
                | jig_client::Outcome::Unconfirmed => Some(StartAnswer::Unknown),
            };
            side.demo.start = None;
            side.demo.failure = match result {
                Some(StartAnswer::Failed(why)) => Some(why),
                Some(
                    StartAnswer::Running
                    | StartAnswer::NotOffered
                    | StartAnswer::Unknown
                    | StartAnswer::EntranceRefused(_),
                )
                | None => None,
            };
            read(side, out);
            side.changed();
            true
        }
        Told::Page { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => false,
    }
}
/// Whether the demo's model and forge still supply the deployment's work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Halves {
    pub model: bool,
    pub forge: bool,
}
#[must_use]
pub fn halves(answer: Option<&Demo>, accounts: Option<&c::OwnerAccounts>, forges: &super::forges::State) -> Halves {
    let mut halves = Halves { model: false, forge: false };
    let Some(answer) = answer else { return halves };
    if let super::forges::State::Rows(rows) = forges {
        halves.forge = true;
        for row in rows {
            if row.number != answer.forge
                && let super::forges::Connection::Connected { .. } = row.connection
            {
                halves.forge = false;
                break;
            }
        }
    }
    let Some(accounts) = accounts else { return halves };
    if accounts.cut {
        return halves;
    }
    for line in &accounts.accounts {
        if (line.provider != Some(answer.provider) && line.cut) || usable_account(line, Some(answer.provider)) {
            return halves;
        }
    }
    halves.model = true;
    halves
}

pub(crate) fn current(side: &Side, accounts: Option<&c::OwnerAccounts>) -> Halves {
    let mut halves = halves(side.demo.answer.as_ref(), accounts, &side.forges);
    halves.model &= !side.demo.model_ended;
    halves.forge &= !side.demo.forge_ended;
    halves
}

/// A usable source of a provider other than the demo's, or any provider without a demo.
#[must_use]
pub fn usable_account(line: &c::OwnerAccountLine, demo_provider: Option<u32>) -> bool {
    if demo_provider.is_some() && line.provider == demo_provider {
        return false;
    }
    for source in &line.sources {
        if source.usable {
            return true;
        }
    }
    false
}

/// Observe positive evidence of an end; unavailable or cut documents are not ends.
/// Session latches keep a lost credential from bringing the model half back.
pub(crate) fn progress(
    side: &mut Side,
    accounts: Option<&c::OwnerAccounts>,
    client: &Client,
    out: &mut Queue<Request>,
) {
    observe_end(side, accounts, client, out);
    remove(side, client, out);
}

fn observe_end(side: &mut Side, accounts: Option<&c::OwnerAccounts>, client: &Client, out: &mut Queue<Request>) {
    let Some(demo) = side.demo.answer.as_ref() else { return };
    if demo.state != DemoState::Running {
        return;
    }
    let mut model_ended = false;
    if let Some(accounts) = accounts {
        for line in &accounts.accounts {
            model_ended |= usable_account(line, Some(demo.provider));
        }
    }
    let mut forge_ended = false;
    if let super::forges::State::Rows(rows) = &side.forges {
        for row in rows {
            if row.number != demo.forge
                && let super::forges::Connection::Connected { .. } = row.connection
            {
                forge_ended = true;
                break;
            }
        }
    }
    let reread = forge_ended && !side.demo.forge_ended;
    let changed = (model_ended && !side.demo.model_ended) || reread;
    let first_end = !side.demo.model_ended && !side.demo.forge_ended && (model_ended || forge_ended);
    side.demo.model_ended |= model_ended;
    side.demo.forge_ended |= forge_ended;
    if first_end && !side.demo.removed && side.demo.removal.is_none() && owner(client) {
        side.serial = side.serial.checked_add(1).expect("forge handles fit");
        let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
        side.demo.removal = Some((handle, demo.provider));
        out.push(Request::Client {
            asked: jig_client::Event::Watch { handle, subject: jig_client::Watchable::Jig(c::Subject::Deployment) },
        });
    }
    if reread {
        read(side, out);
    }
    if changed {
        side.changed();
    }
}

fn owner(client: &Client) -> bool {
    let Some(document) = client.document(super::me::handle()) else { return false };
    if document.state != jig_client::State::Live {
        return false;
    }
    let Some(Latest::Jig(c::Delivery::Me(me))) = document.latest else { return false };
    me.deployment_role == Some(c::HeldRole::Owner)
}

/// The end holds this watch even when the forge's reread has already answered none.
fn remove(side: &mut Side, client: &Client, out: &mut Queue<Request>) {
    let Some((handle, provider)) = side.demo.removal else { return };
    let Some(document) = client.document(handle) else { return };
    if document.state != jig_client::State::Live {
        return;
    }
    let Some(Latest::Jig(c::Delivery::OwnerDeployment(deployment))) = document.latest else { return };
    for account in &deployment.accounts {
        if account.provider == Some(provider) && account.holder == c::OwnerAccountHolder::Deployment {
            side.serial = side.serial.checked_add(1).expect("forge request serial fits");
            side.demo.removed = true;
            // No pending page operation owns this key: every outcome is dropped.
            out.push(Request::Client {
                asked: jig_client::Event::Send {
                    key: jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: side.serial },
                    name: None,
                    request: jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                        holder: c::AccountHolder::Deployment,
                        change: c::AccountChange::Remove { account: account.number },
                    })),
                },
            });
            break;
        }
    }
    side.demo.removal = None;
    out.push(Request::Client { asked: jig_client::Event::Unwatch { handle } });
}

fn read(side: &mut Side, out: &mut Queue<Request>) {
    side.serial = side.serial.checked_add(1).expect("forge handles fit");
    let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    side.demo.pending = Some(handle);
    let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Demo }).expect("fieldless demo read");
    out.push(Request::Client {
        asked: jig_client::Event::Read {
            handle,
            read: Readable::Family {
                scope: c::Scope::Deployment,
                // Window has no minimum row count; this read has no paged rows.
                window: c::Window { rows: 0, after: None },
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::reading(&record),
                },
            },
        },
    });
}

/// The engine's offered demo state, independent of a start sent by this session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoState {
    NotStarted,
    Running,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartFailure {
    Group(Box<[u8]>),
    Account,
    Adoption,
    NotConnected,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartAnswer {
    Running,
    NotOffered,
    Failed(StartFailure),
    Unknown,
    EntranceRefused(c::EntranceRefusal),
}
#[must_use]
pub fn ask(limits: &Limits) -> Option<jig_client::Ask> {
    let limits = wire_limits(limits)?;
    let record = w::Requesting::new(&limits, w::RequestingParts { request: w::Request::StartDemo }).ok()?;
    Some(jig_client::Ask::Family {
        scope: c::Scope::Deployment,
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::requesting(&record),
        },
    })
}
/// Outcomes belonging to another operation are undecodable here.
#[must_use]
pub fn answer(value: &jig_client::Answered, limits: &Limits) -> Option<StartAnswer> {
    let record = match value {
        jig_client::Answered::Family(record) => record,
        jig_client::Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            return Some(StartAnswer::EntranceRefused(c::EntranceRefusal::new(refusal.clone()).ok()?));
        }
        jig_client::Answered::Jig(c::Answer::Done(_)) => return None,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let value = w::Answer::decode(&wire_limits(limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match value.into_parts().outcome {
        w::Outcome::Running => StartAnswer::Running,
        w::Outcome::Unknown => StartAnswer::Unknown,
        w::Outcome::StartRefused(value) => match value.into_parts().why {
            w::StartRefusal::NotOffered => StartAnswer::NotOffered,
        },
        w::Outcome::StartFailed(value) => StartAnswer::Failed(match value.into_parts().why {
            w::StartFailure::Group(value) => StartFailure::Group(value.into_parts().why),
            w::StartFailure::Account => StartFailure::Account,
            w::StartFailure::Adoption => StartFailure::Adoption,
            w::StartFailure::NotConnected => StartFailure::NotConnected,
        }),
        w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_) => return None,
    })
}
fn start_key(side: &Side, key: jig_client::Key) -> bool {
    if key.asker != crate::owner::Owner::Forge.asker() {
        return false;
    }
    match &side.demo.start {
        Some(start) => start.request.raw() == key.number,
        None => false,
    }
}
pub(crate) fn configure(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if !side.demo.sent
        && let Some(demo) = side.demo.answer.as_ref()
        && demo.state == DemoState::Running
    {
        start(side, limits, true, out);
    }
}
pub(crate) fn start(side: &mut Side, limits: Limits, silent: bool, out: &mut Queue<Request>) {
    if side.party.is_none() || side.served == super::Served::OutOfDate || side.demo.start.is_some() {
        return;
    }
    let Some(demo) = side.demo.answer.as_ref() else { return };
    if !silent && demo.state == DemoState::Running && side.demo.failure.is_none() {
        return;
    }
    let Some(ask) = ask(&limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.demo.start = Some(Starting { request, silent });
    side.demo.failure = None;
    side.demo.sent = true;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
