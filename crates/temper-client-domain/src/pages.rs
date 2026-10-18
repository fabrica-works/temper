//! temper's pages own their handles and navigation (client.md, 3.1).
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{
    Client, Handle, Latest, Link, Told, Watchable,
    contract::{Delivery, Me, Subject},
};
use skein_lib::{Env, Queue, Time};
pub mod inbox;
#[derive(Debug)]
pub struct Pages {
    address: Address,
    project: Option<u32>,
    watching: bool,
    shown: u64,
    pub(crate) inbox: Option<inbox::Inbox>,
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
        Event::Went { address } => {
            if pages.inbox.is_none() || pages.address != address {
                if let Some(inbox) = pages.inbox.take() {
                    pages.next = inbox.leave(out);
                }
                if let Address::Inbox { filter } = address {
                    pages.inbox = Some(inbox::Inbox::new(filter, pages.next, &env.limits, out));
                }
                pages.seen = None;
                pages.confirm = None;
            }
            pages.address = address;
        }
        Event::Act { act: Act::Filter { filter } } => out.push(Request::Show { address: Address::Inbox { filter } }),
        Event::Act { act: Act::Seen } => {
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
        Event::Act {
            act: act @ (Act::Decide { .. } | Act::CardWords { .. } | Act::Again { .. } | Act::Confirm | Act::Cancel),
        } => crate::acts::step(pages, env, act, out),
        Event::Act { act: Act::Project { project } } => {
            if let Some(me) = pages.me(client) {
                for row in &me.projects {
                    if row.project == project {
                        pages.project = Some(project);
                        break;
                    }
                }
            }
        }
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
            if !match &mut pages.inbox {
                Some(inbox) => inbox.told(client, told, out),
                None => false,
            } {
                return;
            }
        }
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
