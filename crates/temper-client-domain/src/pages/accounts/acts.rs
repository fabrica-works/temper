//! Named operations and one-shot secret transport; jig owns admission and durability.
use super::{Accounts, Request, Selection};
use jig_client::{Client, Key, contract as c};
use skein_lib::{Queue, RequestKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    SignOut,
    AddKey,
    ReplaceKey,
    RemoveKey,
    RemoveAccount,
    Resume,
    MoveAccount { place: u32 },
    MoveSource { place: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Select { selection: Selection },
    Submit { name: RequestKey, operation: Operation },
    Confirm,
    Cancel,
}
#[derive(Debug)]
pub struct Pending {
    pub key: Key,
    pub name: Option<RequestKey>,
    pub ask: c::Ask,
    pub outcome: Option<jig_client::Outcome>,
    pub signing: bool,
    pub ended: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Broken {
    Empty,
    TooLong,
    Selection,
    Rank,
    ValueAgain,
    ProgramName,
}

impl Accounts {
    pub(crate) fn act(&mut self, client: &Client, act: Act, serial: &mut u64, out: &mut Queue<Request>) {
        match act {
            Act::Select { selection } => {
                self.request.clear_key();
                self.request.confirm = None;
                self.selection = selection;
                self.request.broken = None;
            }
            Act::Cancel => self.request.cancel(),
            Act::Confirm => self.request.confirm(serial, out),
            Act::Submit { name, operation } => {
                self.submit(client, name, operation, serial, out);
            }
        }
    }
    fn submit(
        &mut self,
        client: &Client,
        name: RequestKey,
        operation: Operation,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if let Some(pending) = &self.request.pending
            && pending.outcome.is_none()
        {
            return;
        }
        if operation.carries() && !self.request.has_key() {
            if self.request.broken != Some(Broken::Empty) && self.request.broken != Some(Broken::TooLong) {
                self.request.broken = Some(Broken::ValueAgain);
            }
            return;
        }
        let Some(ask) = self.ask(client, operation) else {
            return;
        };
        if let Some(pending) = &self.request.pending
            && (pending.outcome == Some(jig_client::Outcome::Unsent)
                || pending.outcome == Some(jig_client::Outcome::Lost))
            && (pending.name != Some(name) || pending.ask != ask)
        {
            self.request.clear_key();
            self.request.broken = Some(Broken::ValueAgain);
            return;
        }
        if operation.confirms() {
            self.request.confirm = Some((name, ask));
        } else {
            self.request.send(name, ask, operation.carries(), serial, out);
        }
    }
    fn ask(&mut self, client: &Client, operation: Operation) -> Option<c::Ask> {
        self.request.broken = Some(Broken::Selection);
        let selection = self.selection;
        let me = super::current_me(client)?;
        if selection.holder == c::AccountHolder::Deployment && me.deployment_role != Some(c::HeldRole::Owner) {
            return None;
        }
        // Explicit numbers are forwarded; account/source policy remains the engine's.
        let holder = selection.holder;
        let change = match operation {
            Operation::SignOut => {
                let source = selection.source?;
                self.request.broken = None;
                return Some(c::Ask::SignOut { holder, source });
            }
            Operation::AddKey => c::AccountChange::AddKey {
                provider: selection.provider?,
                join: Some(selection.account?),
                digest: self.request.digest()?,
            },
            Operation::ReplaceKey => {
                c::AccountChange::ReplaceKey { source: selection.source?, digest: self.request.digest()? }
            }
            Operation::RemoveKey => c::AccountChange::RemoveKey { source: selection.source? },
            Operation::RemoveAccount => c::AccountChange::Remove { account: selection.account? },
            Operation::Resume => c::AccountChange::Resume { source: selection.source? },
            Operation::MoveAccount { place } => {
                if place == 0 {
                    self.request.broken = Some(Broken::Rank);
                    return None;
                }
                c::AccountChange::MoveAccount { account: selection.account?, place }
            }
            Operation::MoveSource { place } => {
                if place == 0 {
                    self.request.broken = Some(Broken::Rank);
                    return None;
                }
                c::AccountChange::MoveSource { account: selection.account?, source: selection.source?, place }
            }
        };
        self.request.broken = None;
        Some(c::Ask::AccountChange { holder, change })
    }
}

impl Operation {
    const fn carries(self) -> bool {
        match self {
            Self::AddKey | Self::ReplaceKey => true,
            Self::SignOut
            | Self::RemoveKey
            | Self::RemoveAccount
            | Self::Resume
            | Self::MoveAccount { .. }
            | Self::MoveSource { .. } => false,
        }
    }
    const fn confirms(self) -> bool {
        match self {
            Self::RemoveAccount | Self::RemoveKey | Self::SignOut => true,
            Self::AddKey | Self::ReplaceKey | Self::Resume | Self::MoveAccount { .. } | Self::MoveSource { .. } => {
                false
            }
        }
    }
}
