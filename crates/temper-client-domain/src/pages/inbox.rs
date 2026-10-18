//! The inbox owns its reads; watched documents stay in the client (ruling 235).
use super::Request;
use crate::{
    Limits,
    address::{Filter, Kind},
    owner::Owner,
};
use jig_client::{
    Client, Handle, Latest, Readable, Readout, Told, Watchable,
    contract::{self, Delivery, InboxGroup, InboxReadGroup, ObjectKey, Read, ReadResult, ReadValue},
};
use skein_lib::{List, Queue};
#[derive(Debug)]
pub struct Card {
    pub object: ObjectKey,
    pub group: InboxGroup,
    pub project: Option<u32>,
    pub position: Option<u64>,
    pub handle: Handle,
    pub page: Option<ReadValue>,
}
#[derive(Debug)]
pub struct Inbox {
    pub filter: Filter,
    pub entries: Handle,
    pub cards: List<Card>,
    pub queries: [Handle; 2],
    pub next: u64,
    pub window: u32,
}
fn client(out: &mut Queue<Request>, event: jig_client::Event) {
    out.push(Request::Client(event));
}
impl Inbox {
    pub(super) fn new(filter: Filter, first: u64, limits: &Limits, out: &mut Queue<Request>) -> Self {
        let inbox = Self {
            filter,
            entries: handle(first),
            queries: [
                handle(first.checked_add(1).expect("inbox handles fit")),
                handle(first.checked_add(2).expect("inbox handles fit")),
            ],
            next: first.checked_add(3).expect("inbox handles fit"),
            window: limits.jig.rows.min((limits.jig.objects / 2).max(1)),
            cards: List::with_capacity(limits.jig.objects),
        };
        client(
            out,
            jig_client::Event::Watch {
                handle: inbox.entries,
                subject: Watchable::Jig(contract::Subject::Inbox { window: inbox.window }),
            },
        );
        inbox
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [self.entries, self.queries[0], self.queries[1]] {
            client(out, jig_client::Event::Unwatch { handle });
        }
        for card in &self.cards {
            client(out, jig_client::Event::Unwatch { handle: card.handle });
        }
        self.next
    }
    #[expect(clippy::manual_map, reason = "step code uses an explicit optional kind")]
    fn refresh(&mut self, source: &Client, out: &mut Queue<Request>) {
        for card in &self.cards {
            client(out, jig_client::Event::Unwatch { handle: card.handle });
        }
        self.cards.clear();
        if self.filter.goal.is_some() {
            for (handle, group) in self.queries.into_iter().zip([InboxReadGroup::Needs, InboxReadGroup::Updates]) {
                if self.filter.kind.is_some() && group == InboxReadGroup::Updates {
                    continue;
                }
                client(
                    out,
                    jig_client::Event::Read {
                        handle,
                        read: Readable::Jig(Read::Inbox {
                            group,
                            project: self.filter.project,
                            kind: match self.filter.kind {
                                Some(value) => Some(kind(value)),
                                None => None,
                            },
                            goal: self.filter.goal,
                            after: None,
                        }),
                    },
                );
            }
            return;
        }
        let latest = match source.document(self.entries) {
            Some(document) => document.latest,
            None => None,
        };
        if let Some(Latest::Jig(Delivery::Inbox(inbox))) = latest {
            for row in &inbox.entries {
                self.add(row.object, row.group, Some(row.project), Some(row.position), out);
            }
        } else if let Some(Latest::Jig(Delivery::OwnerInbox(inbox))) = latest {
            for rows in [&inbox.needs, &inbox.updates] {
                for row in rows {
                    self.add(row.card_key(), row.group(), row.project, position(row.object), out);
                }
            }
        }
    }
    fn add(
        &mut self,
        object: ObjectKey,
        group: InboxGroup,
        project: Option<u32>,
        position: Option<u64>,
        out: &mut Queue<Request>,
    ) {
        if self.cards.len() == self.cards.capacity() {
            return;
        }
        if let Some(filter) = self.filter.project
            && Some(filter) != project
        {
            return;
        }
        if let Some(filter) = self.filter.kind
            && kind(filter) != group
        {
            return;
        }
        for card in &self.cards {
            if card.object == object {
                return;
            }
        }
        let read = match object {
            ObjectKey::Task(task) => Read::Task { task },
            ObjectKey::Proposal(proposal) => Read::Proposal { proposal },
            ObjectKey::Escalation(task) => Read::Escalation { task },
            ObjectKey::Question { task, message } => Read::Question { task, message },
            ObjectKey::PersonTask(task) => Read::PersonTask { task },
            ObjectKey::Result(task) => Read::Result { task },
            ObjectKey::Note { .. } => return,
        };
        let handle = Handle { asker: Owner::Pages.asker(), number: self.next };
        self.next = self.next.checked_add(1).expect("inbox handles fit");
        self.cards.push(Card { object, group, project, position, handle, page: None }).expect("inbox cards bounded");
        client(out, jig_client::Event::Read { handle, read: Readable::Jig(read) });
    }
    pub(super) fn told(&mut self, source: &Client, told: Told, out: &mut Queue<Request>) -> bool {
        if let Told::Changed { handle } = &told
            && *handle == self.entries
        {
            self.refresh(source, out);
            return true;
        }
        let Told::Page { handle, page: Readout::Jig(ReadResult::Value(value)) } = told else {
            return false;
        };
        if self.queries.contains(&handle) && self.filter.goal.is_some() {
            if let ReadValue::Inbox(page) = value {
                for row in &page.rows {
                    self.add(row.object, row.group, Some(row.project), Some(row.position), out);
                }
            } else if let ReadValue::OwnerInbox(page) = value {
                for row in &page.rows {
                    self.add(row.card_key(), row.group(), row.project, position(row.object), out);
                }
            }
        } else {
            for index in 0..self.cards.len() {
                let card = self.cards.get_mut(index).expect("bounded card index");
                if card.handle == handle {
                    card.page = Some(value);
                    return true;
                }
            }
            return false;
        }
        true
    }
    #[must_use]
    pub fn results_through(&self) -> Option<u64> {
        let mut through = None;
        for card in &self.cards {
            if card.group == InboxGroup::Results {
                through = through.max(card.position);
            }
        }
        through
    }
}
#[must_use]
pub const fn kind(kind: Kind) -> InboxGroup {
    match kind {
        Kind::Proposal => InboxGroup::Proposals,
        Kind::Question => InboxGroup::Questions,
        Kind::Held => InboxGroup::Escalations,
        Kind::PersonTask => InboxGroup::PersonTasks,
    }
}
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    limits.jig.objects.checked_mul(2).expect("pages output fits").checked_add(8).expect("pages output fits")
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    List::<Card>::worst_case(limits.jig.objects)?
        .checked_add(u64::from(limits.jig.objects).checked_mul(limits.jig.document_bytes)?)
}

fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}
fn position(object: contract::OwnerInboxObject) -> Option<u64> {
    if let contract::OwnerInboxObject::Result { position } = object { Some(position) } else { None }
}
