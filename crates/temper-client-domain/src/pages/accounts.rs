//! Borrowed account facts and one bounded form (client/README.md, 3.5–3.6).
use super::Request;
use crate::owner::Owner;
use jig_client::{Client, Handle, Latest, State, contract as c};
use skein_lib::Queue;
mod acts;
pub mod request;
mod ways;
pub use acts::{Act, Broken, Operation, Pending};
pub use ways::{Way, ways};
/// The carried-value bound (client/README.md, 3.6); the engine catalogue is staged.
pub const KEY_BYTES: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub holder: c::AccountHolder,
    pub provider: Option<u32>,
    pub account: Option<u32>,
    pub source: Option<u32>,
}
impl Default for Selection {
    fn default() -> Self {
        Self { holder: c::AccountHolder::Own, provider: None, account: None, source: None }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignInState {
    Unknown,
    Started { source: u32 },
    Signing { source: u32 },
    Ended { source: u32 },
}

/// A fixed global handle outside the increasing page namespace; zero belongs to `me`.
#[must_use]
pub fn handle() -> Handle {
    Handle { asker: Owner::Pages.asker(), number: u64::MAX }
}

/// Missing, behind and ended watches are unknown, rather than false availability.
#[must_use]
pub fn document(client: &Client) -> Option<&c::OwnerAccounts> {
    current_me(client)?;
    let doc = client.document(handle())?;
    if doc.state != State::Live {
        return None;
    }
    if let Latest::Jig(c::Delivery::OwnerAccounts(accounts)) = doc.latest? { Some(accounts) } else { None }
}

#[derive(Debug)]
pub struct Accounts {
    pub deployment: Option<Handle>,
    pub selection: Selection,
    pub request: request::Request,
}

impl Default for Accounts {
    fn default() -> Self {
        Self::new()
    }
}

impl Accounts {
    #[must_use]
    pub fn new() -> Self {
        Self { deployment: None, selection: Selection::default(), request: request::Request::default() }
    }

    pub(crate) fn configure(&mut self, owner: bool, next: &mut u64, out: &mut Queue<Request>) {
        if !owner && self.deployment.is_some() && self.selection.holder == c::AccountHolder::Deployment {
            self.request.clear_key();
            self.request.confirm = None;
            self.selection.provider = None;
            self.selection.account = None;
            self.selection.source = None;
        }
        if owner && self.deployment.is_none() {
            let handle = Handle { asker: Owner::Pages.asker(), number: *next };
            *next = next.checked_add(1).expect("page handles fit");
            self.deployment = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: jig_client::Watchable::Jig(c::Subject::Deployment),
            }));
        } else if !owner && let Some(handle) = self.deployment.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }

    pub(crate) fn leave(&mut self, out: &mut Queue<Request>) {
        self.request.clear_key();
        self.request.confirm = None;
        self.selection = Selection::default();
        if let Some(handle) = self.deployment.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }

    #[must_use]
    pub fn deployment<'a>(&self, client: &'a Client) -> Option<&'a c::OwnerDeployment> {
        if current_me(client)?.deployment_role != Some(c::HeldRole::Owner) {
            return None;
        }
        let doc = client.document(self.deployment?)?;
        if doc.state != State::Live {
            return None;
        }
        if let Latest::Jig(c::Delivery::OwnerDeployment(deployment)) = doc.latest? { Some(deployment) } else { None }
    }
}

/// Current identity only; old owner facts cannot keep a deployment watch open.
#[must_use]
pub fn current_me(client: &Client) -> Option<&c::Me> {
    let doc = client.document(super::me_handle())?;
    if doc.state != State::Live {
        return None;
    }
    if let Latest::Jig(c::Delivery::Me(me)) = doc.latest? { Some(me) } else { None }
}
#[must_use]
pub fn worst_case(limits: &crate::Limits) -> Option<u64> {
    u64::try_from(size_of::<Accounts>())
        .ok()?
        .checked_add(u64::try_from(KEY_BYTES).ok()?)?
        .checked_add(limits.client.domain.document_bytes)?
        .checked_add(32)?
        .checked_add(skein_lib::List::<Way>::worst_case(limits.client.domain.rows.checked_mul(3)?)?)
}
