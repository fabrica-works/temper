//! Accounts bindings contain selections and operation names, never a key value.
use skein_lib::{List, RequestKey, Secret, Token};
use skein_ui::{Act, Binding};
use temper_client_domain::{
    Action, Event,
    pages::{
        self,
        accounts::{Act as AccountAct, Selection},
    },
};
pub(crate) mod words;
#[derive(Clone, Copy, Debug)]
pub(crate) enum Field {
    Provider,
    Account,
    Source,
}
#[derive(Clone, Debug)]
pub(crate) enum Meaning {
    Action(AccountAct),
    Field(Field, Selection),
    Key,
}
#[derive(Debug)]
pub(crate) struct Bindings {
    entries: List<(Token, Meaning)>,
    next: u64,
    invalid: u8,
}
pub(crate) const fn owner() -> skein_ui::Owner {
    // View-local owner, outside the u16 wire family/asker namespace.
    skein_ui::Owner { number: 65536 }
}
impl Bindings {
    pub fn new(capacity: u32) -> Self {
        Self { entries: List::with_capacity(capacity), next: 0, invalid: 0 }
    }
    pub fn begin(&mut self, reset: bool) {
        if reset {
            self.invalid = 0;
        }
        self.entries.clear();
    }
    pub fn insert(&mut self, meaning: Meaning) -> Binding {
        self.next = self.next.checked_add(1).expect("account bindings fit");
        let token = Token::new(self.next);
        self.entries.push((token, meaning)).expect("account bindings fit");
        Binding { owner: owner(), token }
    }
    pub fn decode(&mut self, token: Token, act: Act) -> Option<Event> {
        for (key, meaning) in &self.entries {
            if *key != token {
                continue;
            }
            let action = match (meaning, act) {
                (Meaning::Key, Act::Input { value }) => return Some(Event::AccountKey { value: Secret::new(value) }),
                (Meaning::Field(field, selection), Act::Input { value }) => {
                    let bit = field.bit();
                    self.invalid &= !bit;
                    let number = if value.is_empty() {
                        None
                    } else {
                        let parsed = match skein_ui::number(&value) {
                            Some(number) => u32::try_from(number).ok(),
                            None => None,
                        };
                        if parsed.is_none() {
                            self.invalid |= bit;
                        }
                        parsed
                    };
                    let mut selection = *selection;
                    match field {
                        Field::Provider => selection.provider = number,
                        Field::Account => selection.account = number,
                        Field::Source => selection.source = number,
                    }
                    AccountAct::Select { selection }
                }
                (Meaning::Action(action), Act::Press) => {
                    match action {
                        AccountAct::Select { .. } | AccountAct::Cancel => self.invalid = 0,
                        AccountAct::Submit { .. } | AccountAct::Confirm => {
                            if self.invalid != 0 {
                                return None;
                            }
                        }
                    }
                    action.clone()
                }
                (_, Act::Submit | Act::Toggle | Act::Drawing { .. })
                | (Meaning::Action(_), Act::Input { .. })
                | (Meaning::Key | Meaning::Field(..), Act::Press) => return None,
            };
            return Some(Event::Act { action: Action::Pages(pages::Act::Accounts(action)) });
        }
        None
    }
    pub fn invalid(&self) -> bool {
        self.invalid != 0
    }
    pub fn invalid_field(&self, field: Field) -> bool {
        self.invalid & field.bit() != 0
    }
    pub fn worst_case(capacity: u32) -> Option<u64> {
        List::<(Token, Meaning)>::worst_case(capacity)?.checked_add(u64::try_from(size_of::<Self>()).ok()?)
    }
}
pub(crate) fn submit(name: RequestKey, operation: pages::accounts::Operation) -> Meaning {
    Meaning::Action(AccountAct::Submit { name, operation })
}

impl Field {
    const fn bit(self) -> u8 {
        match self {
            Self::Provider => 1,
            Self::Account => 2,
            Self::Source => 4,
        }
    }
}
