//! One task's borrowed documents, first activity page and live run handle.
use super::Request;
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::Queue;
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
}
impl Task {
    pub(crate) fn new(number: u64, first: u64, limits: &Limits, out: &mut Queue<Request>) -> Self {
        let document = handle(first);
        let tree = handle(first.checked_add(1).expect("task handles fit"));
        let activity = handle(first.checked_add(2).expect("task handles fit"));
        for (handle, subject) in [
            (document, c::Subject::Task { task: number }),
            (tree, c::Subject::Tree { task: number, window: limits.jig.rows }),
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
        self.next
    }
    pub(crate) fn told(&mut self, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
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
            Told::Changed { handle } | Told::Text { handle } if handle == self.tree || Some(handle) == self.run => {}
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
    // The first activity page and a final task read, after the live watch ended.
    limits.jig.document_bytes.checked_mul(2)
}
