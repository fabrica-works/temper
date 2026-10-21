//! temper's banner, with navigation after the projects (client/README.md, 5).
use skein_lib::{List, Token};
use skein_ui::{Binding, Builder, Region, Role, Run};
use temper_client_domain::{Address, Domain, pages::Act};
#[must_use]
pub const fn owner() -> skein_ui::Owner {
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
                    (Act::ChatWords { .. }, skein_ui::Act::Input { value }) => Act::ChatWords { words: value },
                    (Act::MessageWords { .. }, skein_ui::Act::Input { value }) => Act::MessageWords { words: value },
                    (Act::CardWords { object, .. }, skein_ui::Act::Input { value }) => {
                        Act::CardWords { object: *object, words: value }
                    }
                    (Act::ChatWords { .. }, skein_ui::Act::Submit) => Act::StartChat,
                    (Act::MessageWords { .. }, skein_ui::Act::Submit) => Act::Send,
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
fn text(builder: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    builder.open(role);
    if let Some(style) = style {
        builder.style(style);
    }
    builder.name(words);
    builder.text(Run::Plain, words);
    builder.close();
}
fn action(builder: &mut Builder, role: Role, words: &[u8], act: Act, current: bool, bindings: &mut Bindings) {
    builder.open(role);
    builder.name(words);
    if role == Role::Link {
        builder.text(Run::Plain, words);
    }
    if current {
        builder.state(skein_ui::State::Current);
    }
    builder.bind(bindings.insert(act));
    builder.close();
}
pub(crate) fn unavailable(root: &Domain) -> Option<crate::words::Word> {
    match root.client().link().state {
        jig_client::Link::SignedOut => Some(crate::words::Word::SignedOut),
        jig_client::Link::OutOfDate => Some(crate::words::Word::OutOfDate),
        jig_client::Link::Starting | jig_client::Link::Live | jig_client::Link::Offline => None,
    }
}
pub(crate) fn banner(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    let pages = root.pages();
    let me = pages.me(root.client());
    builder.open(Role::Banner);
    builder.style(crate::look::BANNER);
    text(builder, Role::Paragraph, &words.say(crate::words::Word::Brand, &[]), Some(crate::look::BRAND));
    if let Some(word) = unavailable(root) {
        builder.open(Role::Slot);
        builder.style(crate::look::CORNER);
        builder.open(Role::Region);
        builder.region(link_region());
        text(builder, Role::Status, &words.say(word, &[]), None);
        builder.close();
        builder.close();
        builder.close();
        return;
    }
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
                        builder.state(skein_ui::State::Pressed);
                    }
                    builder.bind(bindings.insert(Act::Project { project: row.project }));
                    builder.close();
                }
            }
        }
    }
    navigation(builder, root, words, bindings);
    builder.open(Role::Slot);
    builder.style(crate::look::CORNER);
    builder.open(Role::Region);
    builder.region(link_region());
    let (status, style) = match root.client().link().state {
        jig_client::Link::Offline => (crate::words::Word::Offline, crate::look::OFFLINE),
        jig_client::Link::Live if me.is_some() => (crate::words::Word::Live, crate::look::LIVE),
        jig_client::Link::SignedOut
        | jig_client::Link::Starting
        | jig_client::Link::Live
        | jig_client::Link::OutOfDate => (crate::words::Word::Connecting, crate::look::CONNECTING),
    };
    text(builder, Role::Status, &words.say(status, &[]), Some(style));
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
    if me_behind || page_behind {
        text(builder, Role::Status, &words.say(crate::words::Word::Behind, &[]), Some(crate::look::BEHIND));
    }
    builder.close();
    if let Some(me) = me {
        text(builder, Role::Status, me.party.name.bytes(), None);
    }
    if root.client().link().sign_out_lost {
        text(builder, Role::Alert, &words.say(crate::words::Word::SignOutLost, &[]), None);
    }
    if me.is_some() || root.client().link().sign_out_lost {
        action(builder, Role::Button, &words.say(crate::words::Word::SignOut, &[]), Act::SignOut, false, bindings);
    }
    builder.close();
    builder.close();
}
pub(crate) fn navigation(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    if unavailable(root).is_some() {
        return;
    }
    let (inbox_current, adopt_current) = match root.pages().address() {
        Address::Inbox { .. } => (true, false),
        Address::Adopt { .. } => (false, true),
        Address::Chats | Address::Chat { .. } | Address::Task { .. } | Address::Accounts => (false, false),
    };
    let name = words.say(
        crate::words::Word::Inbox,
        &[skein_ui::phrases::Hole::Number(u64::from(match root.pages().me(root.client()) {
            Some(me) => me.needs_total,
            None => 0,
        }))],
    );
    builder.open(Role::Navigation);
    builder.style(crate::look::NAVIGATION);
    action(
        builder,
        Role::Link,
        &name,
        Act::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
        inbox_current,
        bindings,
    );
    action(
        builder,
        Role::Link,
        &words.say(crate::words::Word::Chats, &[]),
        Act::Go { address: Address::Chats },
        *root.pages().address() == Address::Chats,
        bindings,
    );
    if let Some(project) = root.pages().project() {
        action(
            builder,
            Role::Link,
            &words.say(crate::words::Word::Adoption, &[]),
            Act::Go { address: Address::Adopt { project } },
            adopt_current,
            bindings,
        );
    }
    action(
        builder,
        Role::Link,
        &crate::accounts::words::say(words, crate::accounts::words::Word::Accounts, &[]),
        Act::Go { address: Address::Accounts },
        *root.pages().address() == Address::Accounts,
        bindings,
    );
    builder.close();
}

/// The model half's banner, after the frame and before the page's main.
pub(crate) fn demo_banner(b: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    if unavailable(root).is_some() || *root.pages().address() == Address::Accounts || root.demo().is_none() {
        return;
    }
    b.open(Role::Region);
    b.style(crate::look::ACCOUNTS);
    b.name(&words.say(crate::words::Word::Demo, &[]));
    text(b, Role::Paragraph, &words.say(crate::words::Word::DemoModel, &[]), None);
    action(
        b,
        Role::Link,
        &words.say(crate::words::Word::ConnectModel, &[]),
        Act::Go { address: Address::Accounts },
        false,
        bindings,
    );
    b.close();
}
