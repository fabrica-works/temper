//! Named operations and one-shot secret transport; jig owns admission and durability.
use super::{Accounts, KEY_BYTES, Request, Selection, Way};
use crate::owner::Owner;
use alloc::boxed::Box;
use jig_client::{Client, Key, Told, contract as c};
use skein_lib::{Queue, RequestKey, Secret};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    SignIn,
    SignOut,
    AddKey,
    AddAccountKey,
    DeclareProgram,
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
    Choose { name: RequestKey, way: Way },
    ProgramName { value: Box<[u8]> },
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
    pub(crate) fn key(&mut self, value: Secret) {
        self.value = None;
        if let Some(pending) = &self.pending
            && pending.outcome.is_none()
        {
            return;
        }
        self.broken = if value.is_empty() {
            Some(Broken::Empty)
        } else if value.len() > KEY_BYTES {
            Some(Broken::TooLong)
        } else {
            self.value = Some(value);
            None
        };
    }
    pub(crate) fn act(&mut self, client: &Client, act: Act, serial: &mut u64, out: &mut Queue<Request>) {
        match act {
            Act::Select { selection } => {
                self.clear_key();
                self.confirm = None;
                self.selection = selection;
                self.broken = None;
                self.way = None;
                self.program = Box::new([]);
            }
            Act::Choose { name, way } => {
                if let Some(pending) = &self.pending
                    && pending.outcome.is_none()
                {
                    return;
                }
                let Some(me) = super::current_me(client) else { return };
                if !super::ways(&me.providers, None, self.selection.holder).as_slice().contains(&way) {
                    return;
                }
                self.clear_key();
                self.confirm = None;
                self.broken = None;
                self.selection =
                    Selection { provider: Some(way.provider()), holder: self.selection.holder, ..Default::default() };
                self.way = Some(way);
                self.program =
                    if let Way::Program { .. } = way { program_default(me.party.name.bytes()) } else { Box::new([]) };
                if let Way::SignIn { .. } = way {
                    self.submit(client, name, Operation::SignIn, serial, out);
                }
            }
            Act::ProgramName { value } => {
                if let Some(pending) = &self.pending
                    && pending.outcome.is_none()
                {
                    return;
                }
                self.program = Box::new([]);
                self.broken = Some(Broken::ProgramName);
                if value.len() <= 32 {
                    self.program = value;
                    if c::valid_program_name(&self.program) {
                        self.broken = None;
                    }
                }
            }
            Act::Cancel => {
                self.confirm = None;
                self.clear_key();
                self.broken = None;
                if let Some(pending) = &self.pending
                    && (pending.outcome == Some(jig_client::Outcome::Lost)
                        || pending.outcome == Some(jig_client::Outcome::Unsent))
                {
                    self.pending = None;
                }
            }
            Act::Confirm => {
                if let Some((name, ask)) = self.confirm.take() {
                    self.send(name, ask, false, serial, out);
                }
            }
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
        if let Some(pending) = &self.pending
            && pending.outcome.is_none()
        {
            return;
        }
        if operation.carries() && self.value.is_none() {
            if self.broken != Some(Broken::Empty) && self.broken != Some(Broken::TooLong) {
                self.broken = Some(Broken::ValueAgain);
            }
            return;
        }
        let Some(ask) = self.ask(client, operation) else {
            return;
        };
        if let Some(pending) = &self.pending
            && (pending.outcome == Some(jig_client::Outcome::Unsent)
                || pending.outcome == Some(jig_client::Outcome::Lost))
            && (pending.name != Some(name) || pending.ask != ask)
        {
            self.clear_key();
            self.broken = Some(Broken::ValueAgain);
            return;
        }
        if operation.confirms() {
            self.confirm = Some((name, ask));
        } else {
            self.send(name, ask, operation.carries(), serial, out);
        }
    }
    fn ask(&mut self, client: &Client, operation: Operation) -> Option<c::Ask> {
        self.broken = Some(Broken::Selection);
        let selection = self.selection;
        let me = super::current_me(client)?;
        if selection.holder == c::AccountHolder::Deployment && me.deployment_role != Some(c::HeldRole::Owner) {
            return None;
        }
        // Explicit numbers are forwarded; account/source policy remains the engine's.
        let holder = selection.holder;
        let change = match operation {
            Operation::SignIn => {
                let provider = selection.provider?;
                self.broken = None;
                return Some(c::Ask::SignIn { holder, provider });
            }
            Operation::SignOut => {
                let source = selection.source?;
                self.broken = None;
                return Some(c::Ask::SignOut { holder, source });
            }
            Operation::AddAccountKey => {
                let Way::Key { provider } = self.way? else { return None };
                c::AccountChange::AddKey { provider, join: None, digest: self.digest()? }
            }
            Operation::DeclareProgram => {
                let Way::Program { provider } = self.way? else { return None };
                if holder != c::AccountHolder::Own || !c::valid_program_name(&self.program) {
                    self.broken = Some(Broken::ProgramName);
                    return None;
                }
                c::AccountChange::DeclareProgram { provider, join: None, name: self.program.clone() }
            }
            Operation::AddKey => c::AccountChange::AddKey {
                provider: selection.provider?,
                join: Some(selection.account?),
                digest: self.digest()?,
            },
            Operation::ReplaceKey => c::AccountChange::ReplaceKey { source: selection.source?, digest: self.digest()? },
            Operation::RemoveKey => c::AccountChange::RemoveKey { source: selection.source? },
            Operation::RemoveAccount => c::AccountChange::Remove { account: selection.account? },
            Operation::Resume => c::AccountChange::Resume { source: selection.source? },
            Operation::MoveAccount { place } => {
                if place == 0 {
                    self.broken = Some(Broken::Rank);
                    return None;
                }
                c::AccountChange::MoveAccount { account: selection.account?, place }
            }
            Operation::MoveSource { place } => {
                if place == 0 {
                    self.broken = Some(Broken::Rank);
                    return None;
                }
                c::AccountChange::MoveSource { account: selection.account?, source: selection.source?, place }
            }
        };
        self.broken = None;
        Some(c::Ask::AccountChange { holder, change })
    }
    fn digest(&mut self) -> Option<[u8; 32]> {
        if self.value.is_none() {
            self.broken = Some(Broken::ValueAgain);
        }
        Some(skein_lib::sha256::digest(self.value.as_ref()?.expose()))
    }
    fn send(&mut self, name: RequestKey, ask: c::Ask, carrying: bool, serial: &mut u64, out: &mut Queue<Request>) {
        if let Some(pending) = &self.pending
            && pending.outcome.is_none()
        {
            return;
        }
        let value = self.value.take();
        self.clear_key();
        if carrying && value.is_none() {
            self.broken = Some(Broken::ValueAgain);
            return;
        }
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending =
            Some(Pending { key, name: Some(name), ask: ask.clone(), outcome: None, signing: false, ended: false });
        self.broken = None;
        let request = jig_client::Ask::Jig(Box::new(ask));
        let event = if carrying {
            jig_client::Event::Carry { key, name, request, value: value.expect("checked value") }
        } else {
            jig_client::Event::Send { key, name: Some(name), request }
        };
        out.push(Request::Client(event));
    }
    pub(crate) fn told(&mut self, client: &Client, told: &Told) -> bool {
        match told {
            Told::Kept { key, request } => {
                let jig_client::Ask::Jig(ask) = request else {
                    return false;
                };
                if let c::Ask::SignIn { .. } | c::Ask::SignOut { .. } | c::Ask::AccountChange { .. } = ask.as_ref() {
                    if self.pending.is_none() {
                        self.pending = Some(Pending {
                            key: *key,
                            name: None,
                            ask: *ask.clone(),
                            outcome: None,
                            signing: false,
                            ended: false,
                        });
                    }
                    true
                } else {
                    false
                }
            }
            Told::Answered { key, outcome } => {
                if let Some(pending) = &mut self.pending
                    && pending.key == *key
                {
                    pending.outcome = Some(outcome.clone());
                    self.refresh(client);
                    true
                } else {
                    false
                }
            }
            Told::Pending { key } => match &self.pending {
                Some(pending) => pending.key == *key,
                None => false,
            },
            Told::Changed { handle } if *handle == super::handle() => {
                self.refresh(client);
                true
            }
            Told::Link { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => false,
        }
    }
}
impl Operation {
    const fn carries(self) -> bool {
        match self {
            Self::AddKey | Self::AddAccountKey | Self::ReplaceKey => true,
            Self::SignIn
            | Self::DeclareProgram
            | Self::SignOut
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
            Self::SignIn
            | Self::AddKey
            | Self::AddAccountKey
            | Self::DeclareProgram
            | Self::ReplaceKey
            | Self::Resume
            | Self::MoveAccount { .. }
            | Self::MoveSource { .. } => false,
        }
    }
}

fn program_default(person: &[u8]) -> Box<[u8]> {
    let mut length = 0;
    for (index, byte) in person.iter().take(25).enumerate() {
        if !byte.is_ascii_alphanumeric() && *byte != b'-' {
            break;
        }
        length = index.checked_add(1).expect("program prefix fits");
    }
    let mut name = skein_lib::Writer::new(7_usize.checked_add(length).expect("program name fits"));
    name.put(b"claude-").expect("measured prefix fits");
    for byte in person.iter().take(length) {
        name.put(&[byte.to_ascii_lowercase()]).expect("measured name fits");
    }
    name.finish()
}
