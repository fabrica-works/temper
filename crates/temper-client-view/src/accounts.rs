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
}
pub(crate) const fn owner() -> skein_ui::Owner {
    // View-local owner, outside the u16 wire family/asker namespace.
    skein_ui::Owner { number: 65536 }
}
impl Bindings {
    pub fn new(capacity: u32) -> Self {
        Self { entries: List::with_capacity(capacity), next: 0 }
    }
    pub fn begin(&mut self) {
        self.entries.clear();
    }
    pub fn insert(&mut self, meaning: Meaning) -> Binding {
        self.next = self.next.checked_add(1).expect("account bindings fit");
        let token = Token::new(self.next);
        self.entries.push((token, meaning)).expect("account bindings fit");
        Binding { owner: owner(), token }
    }
    pub fn decode(&self, token: Token, act: Act) -> Option<Event> {
        for (key, meaning) in &self.entries {
            if *key != token {
                continue;
            }
            let action = match (meaning, act) {
                (Meaning::Key, Act::Input { value }) => return Some(Event::AccountKey { value: Secret::new(value) }),
                (Meaning::Field(field, selection), Act::Input { value }) => {
                    let number =
                        if value.is_empty() { None } else { Some(u32::try_from(skein_ui::number(&value)?).ok()?) };
                    let mut selection = *selection;
                    match field {
                        Field::Provider => selection.provider = number,
                        Field::Account => selection.account = number,
                        Field::Source => selection.source = number,
                    }
                    AccountAct::Select { selection }
                }
                (Meaning::Action(action), Act::Press) => action.clone(),
                (_, Act::Submit | Act::Toggle | Act::Drawing { .. })
                | (Meaning::Action(_), Act::Input { .. })
                | (Meaning::Key | Meaning::Field(..), Act::Press) => return None,
            };
            return Some(Event::Act { action: Action::Pages(pages::Act::Accounts(action)) });
        }
        None
    }
    pub fn worst_case(capacity: u32) -> Option<u64> {
        List::<(Token, Meaning)>::worst_case(capacity)
    }
}
pub(crate) fn submit(name: RequestKey, operation: pages::accounts::Operation) -> Meaning {
    Meaning::Action(AccountAct::Submit { name, operation })
}
