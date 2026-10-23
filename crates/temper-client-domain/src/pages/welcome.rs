//! The welcome's deployment watch and first-run decision (client/README.md, 3.5).
use super::{Request, accounts};
use crate::{
    Domain,
    forge::{self, forges},
    owner::Owner,
};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, State, Told};
use skein_lib::Queue;

#[derive(Debug)]
pub enum Forges {
    Unknown,
    Rows(Box<[forges::Row]>),
    Ended,
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the watch handle and decoding limits belong to the page")]
pub struct Welcome {
    pub forges: Forges,
    pub decided: bool,
    handle: Option<Handle>,
    limits: forge::Limits,
}
impl Welcome {
    pub(crate) const fn new(limits: forge::Limits) -> Self {
        Self { forges: Forges::Unknown, decided: false, handle: None, limits }
    }
    pub(crate) fn configure(&mut self, owner: bool, next: &mut u64, out: &mut Queue<Request>) {
        if !owner {
            self.leave(out);
        } else if self.handle.is_none()
            && match self.forges {
                Forges::Unknown => true,
                Forges::Rows(_) | Forges::Ended => false,
            }
        {
            if let Some(subject) = forges::watch(&self.limits) {
                let handle = Handle { asker: Owner::Pages.asker(), number: *next };
                *next = next.checked_add(1).expect("pages handles fit");
                self.handle = Some(handle);
                out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
            } else {
                self.forges = Forges::Ended;
            }
        }
    }
    pub(crate) fn leave(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        self.forges = Forges::Unknown;
    }
    pub(crate) fn told(&mut self, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if Some(*handle) == self.handle => {
                self.forges = match client.document(*handle) {
                    Some(doc) if doc.state == State::Ended => Forges::Ended,
                    Some(doc) => match doc.latest {
                        Some(Latest::Family(record)) => match forges::rows(record, &self.limits) {
                            Some(rows) => Forges::Rows(rows),
                            None => Forges::Ended,
                        },
                        Some(Latest::Jig(_)) => Forges::Ended,
                        None => Forges::Unknown,
                    },
                    None => Forges::Unknown,
                };
                if let Forges::Ended = self.forges {
                    out.push(Request::Client(jig_client::Event::Unwatch { handle: *handle }));
                }
                true
            }
            Told::Ended { handle, .. } if Some(*handle) == self.handle => {
                self.forges = Forges::Ended;
                true
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => false,
        }
    }
}
/// None while facts are missing; otherwise whether the person's first run is over.
#[must_use]
#[expect(clippy::manual_map, reason = "step code matches optional facts without closures")]
pub fn decision(root: &Domain) -> Option<bool> {
    let me = accounts::current_me(root.client())?;
    if !root.demo_answered() {
        return None;
    }
    let document = root.client().document(accounts::handle())?;
    let accounts = if document.state == State::Ended {
        None
    } else if let Some(Latest::Jig(jig_client::contract::Delivery::OwnerAccounts(accounts))) = document.latest {
        Some(accounts.as_ref())
    } else {
        return None;
    };
    let demo = root.forge().demo();
    let provider = match demo {
        Some(demo) => Some(demo.provider),
        None => None,
    };
    let mut model = false;
    if let Some(accounts) = accounts {
        for line in &accounts.accounts {
            if forge::demo::usable_account(line, provider) {
                model = true;
                break;
            }
        }
    }
    if me.deployment_role != Some(jig_client::contract::HeldRole::Owner) {
        return Some(model);
    }
    match &root.pages().welcome()?.forges {
        Forges::Unknown => None,
        Forges::Ended => Some(false),
        Forges::Rows(rows) => {
            for row in rows {
                let own = match demo {
                    Some(demo) => row.number != demo.forge,
                    None => true,
                };
                if own && let forges::Connection::Connected { .. } = row.connection {
                    return Some(model);
                }
            }
            Some(false)
        }
    }
}
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    let wire = forge::wire_limits(&limits.forge)?;
    let rows = u64::from(temper_client_protocol_wire::FORGES);
    // Decode owns a wire document transiently, then the page's rows and their strings.
    temper_client_protocol_wire::v1::Forges::worst_case_heap(&wire)?
        .checked_mul(2)?
        .checked_add(rows.checked_mul(u64::try_from(size_of::<forges::Row>()).ok()?)?)
}
