//! temper's pages own their handles and navigation (client.md, 3.1).
use crate::{Address, Limits, owner::Owner};
use jig_client::{
    Client, Handle, Latest, Link, Told, Watchable,
    contract::{Delivery, Me, Subject},
};
use skein_lib::{Env, Queue, Time};
#[derive(Debug)]
pub struct Pages {
    address: Address,
    project: Option<u32>,
    watching: bool,
    shown: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Project { project: u32 },
    SignOut,
    Go { address: Address },
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
        }
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
pub fn step(pages: &mut Pages, _env: &Env<Limits>, client: &Client, event: Event, out: &mut Queue<Request>) {
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
        Event::Went { address } => pages.address = address,
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
        Event::Told { .. } => return,
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
