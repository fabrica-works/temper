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
                    (Act::AmendWords { .. }, skein_ui::Act::Input { value }) => Act::AmendWords { words: value },
                    (Act::AmendPriority { .. }, skein_ui::Act::Input { value }) => {
                        Act::AmendPriority { priority: u32::try_from(skein_ui::number(&value)?).ok()? }
                    }
                    (Act::AmendTracked, skein_ui::Act::Toggle) => Act::AmendTracked,
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
    if role == Role::Link {
        crate::link(builder, words, bindings.insert(act));
        builder.text(Run::Plain, words);
    } else {
        builder.open(role);
        builder.name(words);
        builder.bind(bindings.insert(act));
    }
    if current {
        builder.state(skein_ui::State::Current);
    }
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
    if let Address::Welcome { .. } = pages.address()
        && unavailable(root).is_none()
    {
        builder.open(Role::Slot);
        builder.style(crate::look::CORNER);
        if let Some(me) = me {
            text(builder, Role::Status, me.party.name.bytes(), None);
            action(builder, Role::Button, &words.say(crate::words::Word::SignOut, &[]), Act::SignOut, false, bindings);
        }
        builder.close();
        builder.close();
        return;
    }
    builder.open(Role::Disclosure);
    builder.name(&words.say(crate::words::Word::Menu, &[]));
    builder.style(crate::look::MENU);
    if let Some(word) = unavailable(root) {
        builder.open(Role::Slot);
        builder.style(crate::look::CORNER);
        builder.open(Role::Region);
        builder.region(link_region());
        text(builder, Role::Status, &words.say(word, &[]), None);
        builder.close();
        builder.close();
        builder.close();
        builder.close();
        return;
    }
    if let Some(me) = me {
        for selected in [true, false] {
            for row in &me.projects {
                if (Some(row.project) == pages.project()) == selected {
                    // Until jig gives projects names, a project `me` leaves unnamed is "Project N" (5.1).
                    let fallback = words
                        .say(crate::words::Word::Project, &[skein_ui::phrases::Hole::Number(u64::from(row.project))]);
                    let name = match &row.name {
                        Some(name) => name.bytes(),
                        None => &fallback,
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
    builder.close();
}
pub(crate) fn navigation(builder: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    if unavailable(root).is_some() {
        return;
    }
    let (inbox_current, adopt_current) = match root.pages().address() {
        Address::Inbox { .. } => (true, false),
        Address::Adopt { .. } => (false, true),
        Address::Welcome { .. } | Address::Chats | Address::Chat { .. } | Address::Task { .. } | Address::Accounts => {
            (false, false)
        }
    };
    let count = match root.pages().me(root.client()) {
        Some(me) => me.needs_total,
        None => 0,
    };
    let name = if count == 0 {
        words.say(crate::words::Word::InboxTitle, &[])
    } else {
        words.say(crate::words::Word::Inbox, &[skein_ui::phrases::Hole::Number(u64::from(count))])
    };
    builder.open(Role::Navigation);
    builder.name(&words.say(crate::words::Word::MainNavigation, &[]));
    builder.style(crate::look::NAVIGATION);
    crate::link(
        builder,
        &name,
        bindings.insert(Act::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } }),
    );
    if inbox_current {
        builder.state(skein_ui::State::Current);
    }
    if count == 0 {
        builder.text(Run::Plain, &name);
    } else {
        let number = skein_lib::Decimal::of(u64::from(count));
        let prefix = name.len().checked_sub(number.as_bytes().len()).expect("count is the phrase suffix");
        builder.text(Run::Plain, name.get(..prefix).expect("prefix fits"));
        builder.open(Role::Text);
        builder.style(crate::look::NAV_COUNT);
        builder.run(Run::Plain, number.as_bytes());
        builder.close();
    }
    builder.close();
    action(
        builder,
        Role::Link,
        &words.say(crate::words::Word::Chats, &[]),
        Act::Go { address: Address::Chats },
        match root.pages().address() {
            Address::Chats | Address::Chat { .. } => true,
            Address::Welcome { .. }
            | Address::Inbox { .. }
            | Address::Task { .. }
            | Address::Adopt { .. }
            | Address::Accounts => false,
        },
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

/// The demo's one banner, after the accounts' banner and before the page and its chatbar (client/README.md, 5.1).
pub(crate) fn demo_banner(b: &mut Builder, root: &Domain, words: &crate::words::Words, bindings: &mut Bindings) {
    if unavailable(root).is_some() {
        return;
    }
    match root.pages().address() {
        Address::Accounts | Address::Welcome { .. } => return,
        Address::Chats
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Inbox { .. }
        | Address::Adopt { .. } => {}
    }
    let Some((_, halves)) = root.demo() else { return };
    b.open(Role::Region);
    b.style(crate::look::ACCOUNTS);
    b.name(&words.say(crate::words::Word::Demo, &[]));
    let word = if halves.model && halves.forge {
        crate::words::Word::DemoBoth
    } else if halves.model {
        crate::words::Word::DemoModel
    } else {
        crate::words::Word::DemoForge
    };
    text(b, Role::Paragraph, &words.say(word, &[]), None);
    if halves.model {
        action(
            b,
            Role::Link,
            &words.say(crate::words::Word::ConnectModel, &[]),
            Act::Go { address: Address::Accounts },
            false,
            bindings,
        );
    }
    if halves.forge {
        b.open(Role::Button);
        b.name(&words.say(crate::words::Word::ConnectForge, &[]));
        b.state(skein_ui::State::Disabled);
        b.close();
    }
    b.close();
}
