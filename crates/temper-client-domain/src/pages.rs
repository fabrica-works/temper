//! temper's pages own their handles and navigation (client.md, 3.1).
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{
    Client, Handle, Latest, Link, Told, Watchable,
    contract::{Delivery, Me, Subject},
};
use skein_lib::{Env, Queue, Time};
pub mod chat;
pub mod chats;
pub mod inbox;
#[derive(Debug)]
pub struct Pages {
    address: Address,
    project: Option<u32>,
    watching: bool,
    shown: u64,
    pub(crate) inbox: Option<inbox::Inbox>,
    chats: Option<chats::Chats>,
    chat: Option<chat::Chat>,
    next: u64,
    pub(crate) serial: u64,
    seen: Option<(u64, jig_client::Key)>,
    pub(crate) confirm: Option<crate::acts::Confirm>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Project {
        project: u32,
    },
    SignOut,
    ChatWords {
        words: Box<[u8]>,
    },
    StartChat,
    MessageWords {
        words: Box<[u8]>,
    },
    Send,
    Stop,
    Close,
    DelegateFate {
        task: u64,
        fate: jig_client::shapes::Fate,
    },
    Older,
    ClosedChats,
    Filter {
        filter: crate::address::Filter,
    },
    Seen,
    Decide {
        object: jig_client::contract::ObjectKey,
        revision: Option<jig_client::contract::Revision>,
        act: jig_client::contract::ActKind,
    },
    CardWords {
        object: jig_client::contract::ObjectKey,
        words: Box<[u8]>,
    },
    Again {
        again: bool,
    },
    Confirm,
    Cancel,
    Go {
        address: Address,
    },
}
#[derive(Debug)]
pub enum Event {
    Started,
    Went { address: Address },
    Act { act: Act },
    Told { told: Told },
    Link { link: Link },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Client(jig_client::Event),
    Show { address: Address },
}
impl Default for Pages {
    fn default() -> Self {
        Self::new()
    }
}
impl Pages {
    #[must_use]
    pub fn new() -> Self {
        Self {
            address: Address::Inbox { filter: crate::address::Filter::ALL },
            project: None,
            watching: false,
            shown: 0,
            inbox: None,
            chats: None,
            chat: None,
            next: 1,
            serial: 0,
            seen: None,
            confirm: None,
        }
    }
    #[must_use]
    pub const fn confirmation(&self) -> Option<&crate::acts::Confirm> {
        self.confirm.as_ref()
    }
    #[must_use]
    pub const fn inbox(&self) -> Option<&inbox::Inbox> {
        self.inbox.as_ref()
    }
    #[must_use]
    pub const fn chats(&self) -> Option<&chats::Chats> {
        self.chats.as_ref()
    }
    #[must_use]
    pub const fn chat(&self) -> Option<&chat::Chat> {
        self.chat.as_ref()
    }
    #[must_use]
    pub fn cards(&self) -> Option<&skein_lib::List<inbox::Card>> {
        if let Some(chat) = &self.chat { Some(&chat.cards) } else { Some(&self.inbox.as_ref()?.cards) }
    }
    pub(crate) fn cards_mut(&mut self) -> Option<&mut skein_lib::List<inbox::Card>> {
        if let Some(chat) = &mut self.chat { Some(&mut chat.cards) } else { Some(&mut self.inbox.as_mut()?.cards) }
    }
    pub(crate) fn seeded(seed: u64) -> Self {
        Self { serial: skein_lib::Rng::new(seed).next_u64(), ..Self::new() }
    }
    #[must_use]
    pub const fn project(&self) -> Option<u32> {
        self.project
    }
    #[must_use]
    pub const fn address(&self) -> &Address {
        &self.address
    }
    #[must_use]
    pub fn me<'a>(&self, client: &'a Client) -> Option<&'a Me> {
        match client.document(me_handle())?.latest? {
            Latest::Jig(Delivery::Me(me)) => Some(me),
            Latest::Jig(_) | Latest::Family(_) => None,
        }
    }
}
#[must_use]
pub fn me_handle() -> Handle {
    Handle { asker: Owner::Pages.asker(), number: 0 }
}
pub fn step(pages: &mut Pages, env: &Env<Limits>, client: &Client, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Started => {
            pages.address = Address::Inbox { filter: crate::address::Filter::ALL };
            pages.project = None;
            if !pages.watching {
                pages.watching = true;
                out.push(Request::Client(jig_client::Event::Watch {
                    handle: me_handle(),
                    subject: Watchable::Jig(Subject::Me),
                }));
            }
        }
        Event::Went { address } => went(pages, env, address, out),
        Event::Act { act: Act::ChatWords { words } } => {
            if let Some(chats) = &mut pages.chats {
                chats.write(words, &env.limits);
            }
        }
        Event::Act { act: Act::StartChat } => {
            let me = pages.me(client);
            if let Some(chats) = &mut pages.chats {
                chats.start(env, me, pages.project, &mut pages.serial, out);
            }
        }
        Event::Act { act: Act::ClosedChats } => {
            if let Some(chats) = &pages.chats {
                chats.older(out);
            }
        }
        Event::Act { act: Act::Filter { filter } } => out.push(Request::Show { address: Address::Inbox { filter } }),
        Event::Act { act: act @ (Act::MessageWords { .. } | Act::Send | Act::Stop | Act::Older) } => {
            if let Some(chat) = &mut pages.chat {
                chat.act(env, client, act, &mut pages.serial, out);
            }
        }
        Event::Act { act: Act::Seen } => {
            if let Some(chat) = &mut pages.chat {
                chat.act(env, client, Act::Seen, &mut pages.serial, out);
            } else {
                seen(pages, out);
            }
        }
        Event::Act {
            act:
                act @ (Act::Close
                | Act::DelegateFate { .. }
                | Act::Decide { .. }
                | Act::CardWords { .. }
                | Act::Again { .. }
                | Act::Confirm
                | Act::Cancel),
        } => confirmation(pages, env, client, act, out),
        Event::Act { act: Act::Project { project } } => choose_project(pages, client, project),
        Event::Act { act: Act::SignOut } => out.push(Request::Client(jig_client::Event::SignOut)),
        Event::Act { act: Act::Go { address } } => out.push(Request::Show { address }),
        Event::Told { told: Told::Changed { handle } } if handle == me_handle() => {
            if let Some(me) = pages.me(client)
                && !contains(me, pages.project)
            {
                pages.project = None;
                if let Some(row) = me.projects.iter().next() {
                    pages.project = Some(row.project);
                }
            }
        }
        Event::Link { link: Link::SignedOut | Link::OutOfDate } => {
            if pages.watching {
                out.push(Request::Client(jig_client::Event::Unwatch { handle: me_handle() }));
                pages.watching = false;
            }
            pages.project = None;
        }
        Event::Link { link: Link::Starting | Link::Live | Link::Offline } => {
            if !pages.watching {
                pages.watching = true;
                out.push(Request::Client(jig_client::Event::Watch {
                    handle: me_handle(),
                    subject: Watchable::Jig(Subject::Me),
                }));
            }
        }
        Event::Told { told } => {
            if crate::acts::told(pages, &told) {
                pages.shown = pages.shown.checked_add(1).expect("pages shown fits");
                return;
            }
            if !page_told(pages, client, told, out) {
                return;
            }
        }
    }
    let me = pages.me(client);
    if let Some(chats) = &mut pages.chats {
        chats.configure(me, pages.project, &env.limits, out);
    }
    pages.shown = pages.shown.checked_add(1).expect("pages shown count fits");
}
pub fn fire(_pages: &mut Pages, _env: &Env<Limits>, _client: &Client, _out: &mut Queue<Request>) {}
#[must_use]
pub const fn next_deadline(_pages: &Pages) -> Option<Time> {
    None
}
#[must_use]
pub const fn shown(pages: &Pages) -> u64 {
    pages.shown
}

fn contains(me: &Me, project: Option<u32>) -> bool {
    for row in &me.projects {
        if Some(row.project) == project {
            return true;
        }
    }
    false
}

fn went(pages: &mut Pages, env: &Env<Limits>, address: Address, out: &mut Queue<Request>) {
    if (pages.inbox.is_none() && pages.chats.is_none() && pages.chat.is_none()) || pages.address != address {
        if let Some(inbox) = pages.inbox.take() {
            pages.next = inbox.leave(out);
        }
        if let Some(chats) = pages.chats.take() {
            pages.next = chats.leave(out);
        }
        if let Some(chat) = pages.chat.take() {
            pages.next = chat.leave(out);
        }
        if let Address::Chat { task } = address {
            pages.chat = Some(chat::Chat::new(task, pages.next, &env.limits, out));
        }
        if address == Address::Chats {
            pages.chats = Some(chats::Chats::new(pages.next));
        }
        if let Address::Inbox { filter } = address {
            pages.inbox = Some(inbox::Inbox::new(filter, pages.next, &env.limits, out));
        }
        pages.seen = None;
        pages.confirm = None;
    }
    pages.address = address;
}

fn seen(pages: &mut Pages, out: &mut Queue<Request>) {
    if let Some(through) = match &pages.inbox {
        Some(inbox) => inbox.results_through(),
        None => None,
    } && match pages.seen {
        Some((old, _)) => old < through,
        None => true,
    } {
        pages.serial = pages.serial.checked_add(1).expect("pages keys fit");
        let key = jig_client::Key { asker: Owner::Pages.asker(), number: pages.serial };
        pages.seen = Some((through, key));
        out.push(Request::Client(jig_client::Event::Send {
            key,
            request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Seen {
                through: jig_client::contract::SeenThrough::Results { through },
            })),
        }));
    }
}

fn page_told(pages: &mut Pages, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
    if let Some(chat) = &mut pages.chat {
        if !chat.told(client, told, out) {
            return false;
        }
    } else if let Some(chats) = &mut pages.chats {
        if !chats.told(told, out) {
            return false;
        }
    } else if !match &mut pages.inbox {
        Some(inbox) => inbox.told(client, told, out),
        None => false,
    } {
        return false;
    }
    true
}

fn confirmation(pages: &mut Pages, env: &Env<Limits>, client: &Client, act: Act, out: &mut Queue<Request>) {
    let closing = match &pages.confirm {
        Some(open) => open.act == jig_client::contract::ActKind::Cancel,
        None => false,
    };
    match act {
        Act::Close => {
            if let Some(chat) = &mut pages.chat {
                chat.prepare_close(client);
                if chat.closing.is_some() {
                    pages.confirm = Some(crate::acts::Confirm {
                        object: jig_client::contract::ObjectKey::Task(chat.task),
                        revision: None,
                        act: jig_client::contract::ActKind::Cancel,
                        again: None,
                    });
                }
            }
        }
        Act::DelegateFate { task, fate } => {
            if closing && let Some(chat) = &mut pages.chat {
                chat.delegate_fate(task, fate);
            }
        }
        Act::Confirm if closing => {
            pages.confirm = None;
            if let Some(chat) = &mut pages.chat {
                chat.close(&mut pages.serial, out);
            }
        }
        Act::Cancel => {
            pages.confirm = None;
            if let Some(chat) = &mut pages.chat {
                chat.closing = None;
            }
        }
        act @ (Act::Decide { .. } | Act::CardWords { .. } | Act::Again { .. } | Act::Confirm) => {
            crate::acts::step(pages, env, act, out);
        }
        Act::Project { .. }
        | Act::SignOut
        | Act::ChatWords { .. }
        | Act::StartChat
        | Act::MessageWords { .. }
        | Act::Send
        | Act::Stop
        | Act::Older
        | Act::ClosedChats
        | Act::Filter { .. }
        | Act::Seen
        | Act::Go { .. } => unreachable!("only deciding acts routed here"),
    }
}

fn choose_project(pages: &mut Pages, client: &Client, project: u32) {
    if let Some(me) = pages.me(client) {
        for row in &me.projects {
            if row.project == project {
                pages.project = Some(project);
                break;
            }
        }
    }
}
