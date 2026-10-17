//! One bounded table for the last tree, with typed form input (client.md, 5.2).
use alloc::boxed::Box;
use skein_lib::{List, Token};
use skein_ui::{Act, Binding};
use temper_client_domain::{Action, forge};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meaning {
    Field(forge::Text),
    Role(forge::Role),
    Home,
    Ci,
    Adopt,
    Navigate { project: u32 },
    Resources { project: u32 },
}
#[derive(Debug)]
struct Entry {
    token: Token,
    meaning: Meaning,
}
#[derive(Debug)]
pub struct Bindings {
    entries: List<Entry>,
    next: u64,
    connector: u16,
    checks: u32,
}
impl Bindings {
    #[must_use]
    pub fn new(capacity: u32, checks: u32, connector: u16) -> Self {
        Self { entries: List::with_capacity(capacity), next: 0, connector, checks }
    }
    pub fn begin(&mut self) {
        self.entries.clear();
    }
    #[must_use]
    pub fn insert(&mut self, meaning: Meaning) -> Binding {
        self.next = self.next.checked_add(1).expect("binding tokens fit");
        let token = Token::new(self.next);
        self.entries.push(Entry { token, meaning }).expect("form bindings fit");
        Binding { owner: (jig_client_view::Owner::Application { number: self.connector }).opaque(), token }
    }
}
#[must_use]
pub fn decode(bindings: &Bindings, token: Token, act: Act) -> Option<Action> {
    for entry in &bindings.entries {
        if entry.token != token {
            continue;
        }
        return match (entry.meaning, act) {
            (Meaning::Field(field), Act::Input { value }) => {
                let typed = match field {
                    forge::Text::Forge => forge::Typed::Forge(forge_number(&value)),
                    forge::Text::Checks => forge::Typed::Checks(checks(&value, bindings.checks)),
                    forge::Text::Owner | forge::Text::Name | forge::Text::Prefix | forge::Text::Landing => {
                        forge::Typed::Bytes
                    }
                };
                Some(Action::Forge(forge::Act::Write { field, text: value, typed }))
            }
            (meaning, Act::Press | Act::Toggle) => Some(match meaning {
                Meaning::Role(role) => Action::Forge(forge::Act::Choose { role }),
                Meaning::Home => Action::Forge(forge::Act::Home),
                Meaning::Ci => Action::Forge(forge::Act::Ci),
                Meaning::Adopt => Action::Forge(forge::Act::Adopt),
                Meaning::Navigate { project } => {
                    Action::Forge(forge::Act::Go { address: forge::Address::Adopt { project } })
                }
                Meaning::Resources { project } => Action::Jig(jig_client_view::action::Act::Go {
                    address: jig_client_view::address::Address::Project {
                        project,
                        section: jig_client_view::address::ProjectSection::Resources,
                    },
                }),
                Meaning::Field(_) => return None,
            }),
            (_, Act::Input { .. } | Act::Submit | Act::Drawing { .. }) => None,
        };
    }
    None
}
#[must_use]
pub fn forge_number(text: &[u8]) -> Option<u16> {
    let number = u16::try_from(skein_ui::number(text)?).ok()?;
    if number == 0 { None } else { Some(number) }
}
#[must_use]
pub fn checks(text: &[u8], most: u32) -> Option<Box<[u32]>> {
    let capacity = most.min(u32::try_from(text.len()).ok()?);
    let mut values = List::with_capacity(capacity);
    let mut start = None;
    for (index, byte) in text.iter().enumerate() {
        if *byte == b' ' || *byte == b',' {
            if let Some(from) = start {
                check(&mut values, text.get(from..index)?)?;
                start = None;
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(from) = start {
        check(&mut values, text.get(from..)?)?;
    }
    Some(values.into_boxed())
}
fn check(values: &mut List<u32>, text: &[u8]) -> Option<()> {
    let value = u32::try_from(skein_ui::number(text)?).ok()?;
    if value == 0 || values.as_slice().contains(&value) {
        return None;
    }
    values.push(value).ok()
}
#[must_use]
pub fn worst_case(capacity: u32) -> Option<u64> {
    List::<Entry>::worst_case(capacity)
}
