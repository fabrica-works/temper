//! Borrowed account facts and one bounded form (client/README.md, 3.5–3.6).
use super::Request;
use crate::owner::Owner;
use jig_client::{Client, Handle, Latest, State, contract as c};
use skein_lib::{Queue, Secret};
mod acts;
pub use acts::{Act, Broken, Operation, Pending};
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
#[expect(clippy::partial_pub_fields, reason = "the one-shot field is private and never bound or persisted")]
pub struct Accounts {
    pub deployment: Option<Handle>,
    pub selection: Selection,
    pub pending: Option<Pending>,
    pub confirm: Option<(skein_lib::RequestKey, c::Ask)>,
    pub broken: Option<Broken>,
    value: Option<Secret>,
    written: u64,
}

impl Default for Accounts {
    fn default() -> Self {
        Self::new()
    }
}

impl Accounts {
    #[must_use]
    pub fn new() -> Self {
        Self {
            deployment: None,
            selection: Selection::default(),
            pending: None,
            confirm: None,
            broken: None,
            value: None,
            written: 0,
        }
    }

    pub(crate) fn configure(&mut self, owner: bool, next: &mut u64, out: &mut Queue<Request>) {
        if !owner && self.deployment.is_some() && self.selection.holder == c::AccountHolder::Deployment {
            self.clear_key();
            self.confirm = None;
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
        self.clear_key();
        self.confirm = None;
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
        .checked_add(limits.client.domain.document_bytes)
}

impl Accounts {
    #[must_use]
    pub const fn key_written(&self) -> u64 {
        self.written
    }
    pub(super) fn clear_key(&mut self) {
        self.value = None;
        self.written = self.written.checked_add(1).expect("account field writes fit");
    }
    #[must_use]
    pub fn has_key(&self) -> bool {
        self.value.is_some()
    }
    /// Started is an address to visit, never proof of a completed login.
    #[must_use]
    pub fn sign_in_state(&self, client: &Client) -> SignInState {
        let Some((source, _)) = self.started() else {
            return SignInState::Unknown;
        };
        let pending = self.pending.as_ref().expect("started request");
        if pending.ended {
            return SignInState::Ended { source };
        }
        if let Some(accounts) = document(client) {
            for signing in &accounts.signing {
                if signing.source == source {
                    return SignInState::Signing { source };
                }
            }
        }
        if pending.name.is_some() { SignInState::Started { source } } else { SignInState::Unknown }
    }
    pub(super) fn refresh(&mut self, client: &Client) {
        let Some((source, _)) = self.started() else {
            return;
        };
        let Some(accounts) = document(client) else {
            return;
        };
        let pending = self.pending.as_mut().expect("started request");
        let mut signing = false;
        for row in &accounts.signing {
            if row.source == source {
                signing = true;
            }
        }
        let ended = match &accounts.last_end {
            Some(end) => end.source == source,
            None => false,
        };
        pending.ended |= ended || (!signing && !accounts.cut && (pending.signing || pending.name.is_none()));
        pending.signing |= signing;
    }
    #[must_use]
    pub fn sign_in_address(&self, client: &Client) -> Option<&[u8]> {
        match self.sign_in_state(client) {
            SignInState::Started { .. } | SignInState::Signing { .. } => Some(self.started()?.1),
            SignInState::Unknown | SignInState::Ended { .. } => None,
        }
    }
    fn started(&self) -> Option<(u32, &[u8])> {
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
            c::OwnerOutcome::AccountSignInStarted { source, address },
        )))) = self.pending.as_ref()?.outcome.as_ref()?
        {
            Some((*source, address))
        } else {
            None
        }
    }
}
