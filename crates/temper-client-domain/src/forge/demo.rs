//! The deployment's demo read and its two independent halves (client/README.md, 4.8).
use super::{Demo, Limits, Request, Side, family, wire_limits};
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::Queue;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
pub(super) struct State {
    pub answer: Option<Demo>,
    pub pending: Option<Handle>,
    pub live: bool,
    pub answered: bool,
}
impl State {
    pub const fn new() -> Self {
        Self { answer: None, pending: None, live: false, answered: false }
    }
}
/// A link tell remains available to the other children; every forge read page is consumed.
pub(crate) fn told(side: &mut Side, told: &Told, limits: Limits, out: &mut Queue<Request>) -> bool {
    match told {
        Told::Link { link } => {
            if *link == Link::Live && !side.demo.live {
                side.demo.live = true;
                side.serial = side.serial.checked_add(1).expect("forge handles fit");
                let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
                side.demo.pending = Some(handle);
                let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Demo })
                    .expect("fieldless demo read");
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
            }
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
pub fn halves(answer: Option<&Demo>, accounts: Option<&c::OwnerAccounts>) -> Halves {
    // Departure: the forge half holds while answered, pending the deployment's
    // part (client/README.md, 4.8 and section 11).
    let mut halves = Halves { model: false, forge: false };
    let (Some(answer), Some(accounts)) = (answer, accounts) else { return halves };
    if accounts.cut {
        return halves;
    }
    halves.forge = true;
    for line in &accounts.accounts {
        if (line.provider != Some(answer.provider) && line.cut) || usable_account(line, Some(answer.provider)) {
            return halves;
        }
    }
    halves.model = true;
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
