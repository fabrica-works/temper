//! The first-run decision, using the session's forge rows (client/README.md, 3.5, 4.6).
use super::accounts;
use crate::{
    Domain,
    forge::{
        self,
        forges::{self, State as Forges},
    },
};
use jig_client::{Latest, State};
#[derive(Debug)]
pub struct Welcome {
    pub decided: bool,
}
impl Welcome {
    pub(crate) const fn new() -> Self {
        Self { decided: false }
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
    match root.forges() {
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
