//! temper's banner and navigation, in the mockups' order (client.md, 5).
use skein_lib::{List, Token};
use skein_ui::{Binding, Builder, Region, Role};
use temper_client_domain::{Address, Domain, pages::Act};
#[must_use]
pub fn owner() -> skein_ui::Owner {
    skein_ui::Owner { number: temper_client_domain::owner::Owner::Pages.asker().0 }
}
#[must_use]
pub fn link_region() -> Region {
    Region { owner: owner(), number: 0 }
}
#[derive(Debug)]
pub(crate) struct Bindings {
    entries: List<(Token, Act)>,
    next: u64,
}
impl Bindings {
    pub fn new(capacity: u32) -> Self {
        Self { entries: List::with_capacity(capacity), next: 0 }
    }
    pub fn begin(&mut self) {
        self.entries.clear();
    }
    pub(crate) fn insert(&mut self, act: Act) -> Binding {
        self.next = self.next.checked_add(1).expect("pages binding tokens fit");
        let token = Token::new(self.next);
        self.entries.push((token, act)).expect("pages bindings fit");
        Binding { owner: owner(), token }
    }
    pub fn decode(&self, token: Token, act: skein_ui::Act) -> Option<temper_client_domain::Action> {
        for (key, meaning) in &self.entries {
            if *key == token {
                let meaning = match (meaning, act) {
                    (Act::Filter { filter }, skein_ui::Act::Input { value }) => {
                        let mut filter = *filter;
                        filter.goal = if value.is_empty() { None } else { Some(skein_ui::number(&value)?) };
                        Act::Filter { filter }
                    }
                    (meaning, skein_ui::Act::Press) => meaning.clone(),
                    _ => return None,
                };
                return Some(temper_client_domain::Action::Pages(meaning));
            }
        }
        None
    }
    pub fn worst_case(capacity: u32) -> Option<u64> {
        List::<(Token, Act)>::worst_case(capacity)
    }
}
fn text(builder: &mut Builder, role: Role, words: &[u8]) {
    builder.open(role);
    builder.name(words);
    builder.text(skein_ui::Run::Plain, words);
    builder.close();
}
fn action(builder: &mut Builder, role: Role, words: &[u8], act: Act, bindings: &mut Bindings) {
    builder.open(role);
    builder.name(words);
    builder.bind(bindings.insert(act));
    builder.close();
}
pub(crate) fn banner(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    let pages = root.pages();
    let me = pages.me(root.client());
    builder.open(Role::Banner);
    text(builder, Role::Paragraph, &words.say(crate::words::Word::Brand, &[]));
    if let Some(me) = me {
        for selected in [true, false] {
            for row in &me.projects {
                if (Some(row.project) == pages.project()) == selected {
                    let name = match &row.name {
                        Some(name) => name.bytes(),
                        None => b"Project",
                    };
                    builder.open(Role::Button);
                    builder.name(name);
                    if selected {
                        builder.state(skein_ui::State::Selected);
                    }
                    builder.bind(bindings.insert(Act::Project { project: row.project }));
                    builder.close();
                }
            }
        }
    }
    builder.open(Role::Region);
    builder.region(link_region());
    let status = match root.client().link().state {
        jig_client::Link::Offline => crate::words::Word::Offline,
        jig_client::Link::Live if me.is_some() => crate::words::Word::Live,
        jig_client::Link::SignedOut
        | jig_client::Link::Starting
        | jig_client::Link::Live
        | jig_client::Link::OutOfDate => crate::words::Word::Connecting,
    };
    text(builder, Role::Status, &words.say(status, &[]));
    let me_behind = match root.client().document(temper_client_domain::pages::me_handle()) {
        Some(document) => document.state == jig_client::State::Behind,
        None => false,
    };
    let page_behind = match pages.inbox() {
        Some(inbox) => match root.client().document(inbox.entries) {
            Some(document) => document.state == jig_client::State::Behind,
            None => false,
        },
        None => false,
    };
    if root.jig().borrow(root.client()).frame_summary().behind || me_behind || page_behind {
        text(builder, Role::Status, &words.say(crate::words::Word::Behind, &[]));
    }
    builder.close();
    if let Some(me) = me {
        text(builder, Role::Status, me.party.name.bytes());
    }
    if root.client().link().sign_out_lost {
        text(builder, Role::Alert, &words.say(crate::words::Word::SignOutLost, &[]));
    }
    if me.is_some() || root.client().link().sign_out_lost {
        action(builder, Role::Button, &words.say(crate::words::Word::SignOut, &[]), Act::SignOut, bindings);
    }
    builder.close();
}
pub(crate) fn navigation(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    let name = words.say(
        crate::words::Word::Inbox,
        &[skein_ui::phrases::Hole::Number(u64::from(match root.pages().me(root.client()) {
            Some(me) => me.needs_total,
            None => 0,
        }))],
    );
    builder.open(Role::Navigation);
    action(
        builder,
        Role::Link,
        &name,
        Act::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
        bindings,
    );
    action(
        builder,
        Role::Link,
        &words.say(crate::words::Word::Chats, &[]),
        Act::Go { address: Address::Chats },
        bindings,
    );
    if let Some(project) = root.pages().project() {
        action(
            builder,
            Role::Link,
            &words.say(crate::words::Word::Adoption, &[]),
            Act::Go { address: Address::Adopt { project } },
            bindings,
        );
    }
    builder.close();
}
