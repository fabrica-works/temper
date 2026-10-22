//! One task's borrowed documents, first activity page and live run handle.
use super::{Request, inbox::Card};
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{List, Queue};
/// A task's state as a goal's page says it (client/README.md, 5.8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// jig's word for its phase.
    Phase(c::Phase),
    /// Waiting on one listed dependency: this task.
    WaitingFor(u64),
    /// Waiting on `count` listed dependencies, and on more when the list was cut.
    WaitingForTasks { count: u32, more: bool },
    /// Waiting on no dependency, for its holdings.
    WaitingForHoldings,
    /// Waiting for the supplied number of live delegates.
    WaitingForDelegates(u32),
    /// Waiting for an answer from the reader.
    WaitingForYou,
    /// Waiting for an answer from another person or role.
    WaitingForParty(c::Whom),
    /// Waiting for news.
    WaitingForNews,
    /// Running without more specific observed work.
    Running,
    /// Observed writing.
    Writing,
    /// Observed using this tool.
    Tool(Box<[u8]>),
    /// Ended, with its owner's ending.
    Ended(c::OwnerTaskStatus),
}
/// A task's state from its chip.
#[must_use]
pub fn row_state(chip: &c::Chip) -> State {
    if chip.phase == c::Phase::Running {
        if let c::ChipPhase::Owner { details, .. } = chip.details.as_ref()
            && let Some(observation) = &details.observation
        {
            match &observation.work {
                c::OwnerChipWork::Writing => return State::Writing,
                c::OwnerChipWork::Tool { tool, .. } => return State::Tool(tool.clone()),
                c::OwnerChipWork::Preparing
                | c::OwnerChipWork::Starting
                | c::OwnerChipWork::Active
                | c::OwnerChipWork::Waiting
                | c::OwnerChipWork::Ending => {}
            }
        }
        return State::Running;
    }
    match chip.details.as_ref() {
        c::ChipPhase::Owner { phase: c::OwnerChipPhase::Waiting { dependencies, more, holds }, .. } => {
            if dependencies.len() == 1 && !more {
                State::WaitingFor(*dependencies.as_slice().first().expect("one dependency"))
            } else if !dependencies.is_empty() || *more {
                State::WaitingForTasks { count: dependencies.len(), more: *more }
            } else if *holds {
                State::WaitingForHoldings
            } else {
                State::Phase(chip.phase)
            }
        }
        c::ChipPhase::Owner { phase: c::OwnerChipPhase::Ended, details } => match &details.ending {
            Some(c::OwnerChipEnding::Done(_)) => State::Ended(c::OwnerTaskStatus::Done),
            Some(c::OwnerChipEnding::Failed(_)) => State::Ended(c::OwnerTaskStatus::Failed),
            Some(c::OwnerChipEnding::Cancelled { .. }) => State::Ended(c::OwnerTaskStatus::Cancelled),
            None => State::Phase(chip.phase),
        },
        c::ChipPhase::Owner {
            phase:
                c::OwnerChipPhase::Active(_)
                | c::OwnerChipPhase::Person { .. }
                | c::OwnerChipPhase::Closing(_)
                | c::OwnerChipPhase::Held { .. },
            ..
        }
        | c::ChipPhase::Active(_)
        | c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Idle { .. }
        | c::ChipPhase::Starting
        | c::ChipPhase::Running { .. }
        | c::ChipPhase::WaitingForWords { .. }
        | c::ChipPhase::BackingOff { .. }
        | c::ChipPhase::Procedure { .. }
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_)
        | c::ChipPhase::Held { .. }
        | c::ChipPhase::Ended { .. } => State::Phase(chip.phase),
    }
}
/// A goal's header state from its exact wait, falling back to its chip.
#[must_use]
pub fn header_state(task: &c::Task, me: Option<&c::Me>) -> State {
    match task.chip.phase {
        c::Phase::Ended | c::Phase::Done | c::Phase::Failed | c::Phase::Cancelled => return row_state(&task.chip),
        c::Phase::Waiting
        | c::Phase::Idle
        | c::Phase::Due
        | c::Phase::Preparing
        | c::Phase::Claimed
        | c::Phase::Running
        | c::Phase::Active
        | c::Phase::BackingOff
        | c::Phase::Closing
        | c::Phase::Held => {}
    }
    match &task.waits_on {
        c::WaitsOn::Dependencies { pending, count } => {
            if *count == 1 && pending.len() == 1 {
                State::WaitingFor(*pending.as_slice().first().expect("one pending task"))
            } else if *count != 0 {
                State::WaitingForTasks { count: *count, more: false }
            } else {
                row_state(&task.chip)
            }
        }
        c::WaitsOn::Delegates => match task.chip.details.as_ref() {
            c::ChipPhase::Owner { details, .. } => State::WaitingForDelegates(details.live_delegates),
            c::ChipPhase::Active(_)
            | c::ChipPhase::Waiting { .. }
            | c::ChipPhase::Idle { .. }
            | c::ChipPhase::Starting
            | c::ChipPhase::Running { .. }
            | c::ChipPhase::WaitingForWords { .. }
            | c::ChipPhase::BackingOff { .. }
            | c::ChipPhase::Procedure { .. }
            | c::ChipPhase::Person { .. }
            | c::ChipPhase::Closing(_)
            | c::ChipPhase::Held { .. }
            | c::ChipPhase::Ended { .. } => row_state(&task.chip),
        },
        c::WaitsOn::Party { whom, .. } => {
            if let Some(me) = me
                && reader(*whom, task.project, me)
            {
                State::WaitingForYou
            } else {
                State::WaitingForParty(*whom)
            }
        }
        c::WaitsOn::News => State::WaitingForNews,
        c::WaitsOn::Nothing => row_state(&task.chip),
    }
}
// Jig's facts::reader is private; keep its person and project-role identity rule.
fn reader(whom: c::Whom, project: u32, me: &c::Me) -> bool {
    match whom {
        c::Whom::Person(person) => person == me.party.number,
        c::Whom::Role { project: held_project, role } => {
            if held_project != project {
                return false;
            }
            for held in &me.projects {
                let number = match held.role {
                    c::HeldRole::Owner => 0,
                    c::HeldRole::Maintainer => 1,
                    c::HeldRole::Member => 2,
                    c::HeldRole::Observer => 3,
                    c::HeldRole::Policy { role } => role,
                };
                if held.project == project && number == role {
                    return true;
                }
            }
            false
        }
    }
}
/// A dependency's state from its link; none when a stored link has no status.
#[must_use]
pub fn link_state(link: &c::OwnerTaskLink) -> Option<State> {
    match link {
        c::OwnerTaskLink::Live(chip) | c::OwnerTaskLink::Retained { chip, .. } => Some(row_state(chip)),
        c::OwnerTaskLink::Stored { status: Some(status), .. } => Some(State::Ended(*status)),
        c::OwnerTaskLink::Stored { status: None, .. } => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Unknown,
    Tracked,
    HeaderAndPlan,
}
#[derive(Debug)]
pub struct Task {
    pub number: u64,
    pub document: Handle,
    pub tree: Handle,
    pub activity: Handle,
    pub run: Option<Handle>,
    pub shape: Shape,
    pub latest: Option<Box<c::ActivityPage>>,
    pub final_task: Option<Box<c::Task>>,
    pub next: u64,
    pub through: Option<u64>,
    pub cards: List<Card>,
}
impl Task {
    pub(crate) fn new(number: u64, first: u64, limits: &Limits, out: &mut Queue<Request>) -> Self {
        let document = handle(first);
        let tree = handle(first.checked_add(1).expect("task handles fit"));
        let activity = handle(first.checked_add(2).expect("task handles fit"));
        for (handle, subject) in [
            (document, c::Subject::Task { task: number }),
            (tree, c::Subject::Tree { task: number, window: limits.client.domain.rows }),
        ] {
            out.push(Request::Client(jig_client::Event::Watch { handle, subject: Watchable::Jig(subject) }));
        }
        read(activity, c::Read::Activity { task: number, before: None }, out);
        Self {
            number,
            document,
            tree,
            activity,
            run: None,
            shape: Shape::Unknown,
            latest: None,
            final_task: None,
            next: first.checked_add(3).expect("task handles fit"),
            through: None,
            cards: List::with_capacity(limits.client.domain.reads),
        }
    }
    #[must_use]
    pub fn document<'a>(&'a self, client: &'a Client) -> Option<&'a c::Task> {
        if let Some(task) = &self.final_task {
            return Some(task);
        }
        match client.document(self.document)?.latest? {
            Latest::Jig(c::Delivery::Task(task)) => Some(task),
            Latest::Jig(_) | Latest::Family(_) => None,
        }
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [Some(self.document), Some(self.tree), Some(self.activity), self.run].into_iter().flatten() {
            unwatch(handle, out);
        }
        for card in &self.cards {
            unwatch(card.handle, out);
        }
        self.next
    }
    pub(crate) fn told(&mut self, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
        if let Told::Page { handle, page: Readout::Jig(c::ReadResult::Value(value)) } = &told {
            for index in 0..self.cards.len() {
                let card = self.cards.get_mut(index).expect("bounded held card");
                if card.handle == *handle {
                    if let c::ReadValue::Task(task) = value {
                        card.project = Some(task.project);
                    }
                    card.page = Some(value.clone());
                    return true;
                }
            }
        }
        match told {
            Told::Changed { handle } if handle == self.document => {
                if let Some(Latest::Jig(c::Delivery::Task(task))) = latest(client, handle) {
                    self.changed(task, out);
                }
            }
            Told::Ended { handle, why: jig_client::Why::SubjectEnded | jig_client::Why::Unknown }
                if handle == self.document =>
            {
                read(handle, c::Read::Task { task: self.number }, out);
            }
            Told::Page { handle, page } if handle == self.document => {
                let task = match page {
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(task))) => Some(task),
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Historical(value))) => {
                        if let c::OwnerHistoricalValue::Task { document, .. } = value.value
                            && document.chip.task == self.number
                        {
                            Some(document)
                        } else {
                            None
                        }
                    }
                    Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
                };
                if let Some(task) = task {
                    self.changed(&task, out);
                    self.final_task = Some(task);
                }
            }
            Told::Page { handle, page } if handle == self.activity => {
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) = page {
                    self.latest = Some(page);
                }
            }
            Told::Changed { handle } if handle == self.tree => match latest(client, handle) {
                Some(Latest::Jig(c::Delivery::OwnerTree(tree))) => {
                    self.held(&tree.root, out);
                    for row in &tree.rows {
                        self.held(&row.chip, out);
                    }
                }
                Some(Latest::Jig(c::Delivery::Tree(tree))) => {
                    for node in &tree.nodes {
                        self.held(&node.chip, out);
                    }
                }
                Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
            },
            Told::Text { handle } if handle == self.tree => {}
            Told::Changed { handle } | Told::Text { handle } if Some(handle) == self.run => {}
            Told::Ended { handle, .. } if handle == self.tree || Some(handle) == self.run => {}
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => return false,
        }
        true
    }
    fn held(&mut self, chip: &c::Chip, out: &mut Queue<Request>) {
        if chip.phase != c::Phase::Held || self.cards.len() == self.cards.capacity() {
            return;
        }
        let object = c::ObjectKey::Task(chip.task);
        for card in &self.cards {
            if card.object == object {
                return;
            }
        }
        let handle = handle(self.next);
        self.next = self.next.checked_add(1).expect("task handles fit");
        self.cards
            .push(Card {
                object,
                group: c::InboxGroup::Escalations,
                project: None,
                position: None,
                handle,
                page: None,
                words: None,
                key: None,
                outcome: None,
            })
            .expect("held cards bounded");
        read(handle, c::Read::Task { task: chip.task }, out);
    }
    fn changed(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        self.shape = match task.chip.executor {
            c::Executor::Agent { .. } if task.priority.is_some() => Shape::Tracked,
            c::Executor::Agent { .. }
            | c::Executor::Procedure { .. }
            | c::Executor::Party { .. }
            | c::Executor::Role { .. } => Shape::HeaderAndPlan,
        };
        if let Some(through) = self.through
            && through != task.activity
        {
            read(self.activity, c::Read::Activity { task: self.number, before: None }, out);
        }
        self.through = Some(task.activity);
        let running = match task.chip.details.as_ref() {
            c::ChipPhase::Owner {
                phase:
                    c::OwnerChipPhase::Active(c::RunStage::Claimed { .. } | c::RunStage::Running { .. })
                    | c::OwnerChipPhase::Closing(c::ClosingStage::Run { .. }),
                ..
            }
            | c::ChipPhase::Running { .. }
            | c::ChipPhase::WaitingForWords { .. } => true,
            c::ChipPhase::Owner { .. }
            | c::ChipPhase::Active(_)
            | c::ChipPhase::Waiting { .. }
            | c::ChipPhase::Idle { .. }
            | c::ChipPhase::Starting
            | c::ChipPhase::BackingOff { .. }
            | c::ChipPhase::Procedure { .. }
            | c::ChipPhase::Person { .. }
            | c::ChipPhase::Closing(_)
            | c::ChipPhase::Held { .. }
            | c::ChipPhase::Ended { .. } => false,
        };
        if running && self.run.is_none() {
            let handle = handle(self.next);
            self.next = self.next.checked_add(1).expect("task handles fit");
            self.run = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Run { task: self.number, turn: None, offset: 0 }),
            }));
        } else if !running && let Some(handle) = self.run.take() {
            unwatch(handle, out);
        }
    }
    pub(crate) fn chat(&self, client: &Client, me: Option<&c::Me>) -> Option<Address> {
        let task = self.document(client)?;
        let template = jig_client::shapes::template(me?, task.project, jig_client::shapes::CHAT)?;
        if task.label == template.label { Some(Address::Chat { task: self.number }) } else { None }
    }
}
fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}
fn unwatch(handle: Handle, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Unwatch { handle }));
}
fn read(handle: Handle, read: c::Read, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }));
}
fn latest(client: &Client, handle: Handle) -> Option<&Latest> {
    client.document(handle)?.latest
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    // Activity, the final task, and bounded held-task cards and their words.
    limits
        .client
        .domain
        .document_bytes
        .checked_mul(u64::from(limits.client.domain.reads).checked_add(2)?)?
        .checked_add(List::<Card>::worst_case(limits.client.domain.reads)?)?
        .checked_add(u64::from(limits.client.domain.reads).checked_mul(u64::from(limits.client.domain.text))?)
}
