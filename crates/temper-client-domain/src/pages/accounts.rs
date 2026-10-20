//! Borrowed account facts and one bounded form (client/README.md, 3.5–3.6).
use super::Request;
use crate::owner::Owner;
use jig_client::{Client, Handle, Latest, State, contract as c};
use skein_lib::Queue;

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
    match doc.latest? {
        Latest::Jig(c::Delivery::OwnerAccounts(accounts)) => Some(accounts),
        _ => None,
    }
}

#[derive(Debug, Default)]
pub struct Accounts {
    pub deployment: Option<Handle>,
}

impl Accounts {
    pub(crate) fn configure(&mut self, owner: bool, next: &mut u64, out: &mut Queue<Request>) {
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
        match doc.latest? {
            Latest::Jig(c::Delivery::OwnerDeployment(deployment)) => Some(deployment),
            _ => None,
        }
    }
}

/// Current identity only; old owner facts cannot keep a deployment watch open.
#[must_use]
pub fn current_me(client: &Client) -> Option<&c::Me> {
    let doc = client.document(super::me_handle())?;
    if doc.state != State::Live {
        return None;
    }
    match doc.latest? {
        Latest::Jig(c::Delivery::Me(me)) => Some(me),
        _ => None,
    }
}
#[must_use]
pub fn worst_case() -> Option<u64> {
    u64::try_from(size_of::<Accounts>()).ok()
}
