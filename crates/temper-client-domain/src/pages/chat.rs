//! One chat borrows its task and retains jig's bounded conversation projection.
use super::{Act, Request};
use crate::{Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Key, Latest, Readable, Readout, Told, Watchable, contract as c, thread::Thread};
use skein_lib::{Env, List, Queue};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sending {
    Message,
    Stop,
    Seen,
}
#[derive(Debug)]
pub struct Chat {
    pub task: u64,
    pub document: Handle,
    pub activity: Handle,
    pub run: Option<Handle>,
    pub next: u64,
    pub thread: Thread,
    pub words: Box<[u8]>,
    pub pending: List<(Key, Sending)>,
    pub outcome: Option<jig_client::Outcome>,
    pub seen: u64,
    pub final_task: Option<Box<c::Task>>,
}
impl Chat {
    pub(crate) fn new(task: u64, first: u64, limits: &Limits, out: &mut Queue<Request>) -> Self {
        let document = handle(first);
        let activity = handle(first.checked_add(1).expect("chat handles fit"));
        let mut thread = Thread::new(task, &thread_limits(limits));
        thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
        out.push(Request::Client(jig_client::Event::Watch {
            handle: document,
            subject: Watchable::Jig(c::Subject::Task { task }),
        }));
        read(activity, Some(c::Read::Activity { task, before: None }), out);
        Self {
            task,
            document,
            activity,
            run: None,
            next: first.checked_add(2).expect("chat handles fit"),
            thread,
            words: Box::default(),
            pending: List::with_capacity(3),
            outcome: None,
            seen: 0,
            final_task: None,
        }
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [Some(self.document), Some(self.activity), self.run].into_iter().flatten() {
            unwatch(handle, out);
        }
        self.next
    }
    #[must_use]
    pub fn task<'a>(&'a self, client: &'a Client) -> Option<&'a c::Task> {
        if let Some(task) = &self.final_task {
            return Some(task);
        }
        match client.document(self.document)?.latest? {
            Latest::Jig(c::Delivery::Task(task)) => Some(task),
            Latest::Jig(_) | Latest::Family(_) => None,
        }
    }
    pub(crate) fn told(&mut self, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if handle == self.document => {
                if let Some(Latest::Jig(c::Delivery::Task(task))) = latest(client, handle) {
                    read(self.activity, self.thread.document(task), out);
                    self.watch_run(task, out);
                }
            }
            Told::Ended { handle, why: jig_client::Why::SubjectEnded | jig_client::Why::Unknown }
                if handle == self.document =>
            {
                read(handle, Some(c::Read::Task { task: self.task }), out);
            }
            Told::Page { handle, page } if handle == self.document => {
                let task = match page {
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(task))) => Some(task),
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Historical(value)))
                        if value.matches(c::ObjectKey::Task(self.task)) =>
                    {
                        if let c::OwnerHistoricalValue::Task { mut document, result } = value.value {
                            document.result = Some(c::TaskResult::Document(result));
                            Some(document)
                        } else {
                            None
                        }
                    }
                    Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => None,
                };
                if let Some(task) = task {
                    read(self.activity, self.thread.document(&task), out);
                    self.watch_run(&task, out);
                    self.final_task = Some(task);
                }
            }
            Told::Changed { handle } | Told::Text { handle } if Some(handle) == self.run => {
                if let Some(Latest::Jig(c::Delivery::RunState(run))) = latest(client, handle) {
                    self.thread.run(run, client.text(handle).or(run.writing.as_ref()));
                }
            }
            Told::Page { handle, page } if handle == self.activity => {
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) = page {
                    read(self.activity, self.thread.page(*page), out);
                } else {
                    self.thread.failed();
                }
            }
            Told::Answered { key, outcome } if self.pending_key(key) => {
                if let jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) = &outcome {
                    self.thread.answered(key, answer);
                } else {
                    self.thread.words.refused(key);
                }
                let mut kept = List::with_capacity(self.pending.capacity());
                for row in &self.pending {
                    if row.0 != key {
                        kept.push(*row).expect("remaining pending fits");
                    }
                }
                self.pending = kept;
                self.outcome = Some(outcome);
            }
            Told::Pending { key } if self.pending_key(key) => {}
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
        if self.run.is_none() && self.thread.loaded >= self.thread.committed {
            self.thread.end_run();
        }
        true
    }
    fn watch_run(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        if let c::ChipPhase::Owner {
            phase:
                c::OwnerChipPhase::Active(c::RunStage::Claimed { .. } | c::RunStage::Running { .. })
                | c::OwnerChipPhase::Closing(c::ClosingStage::Run { .. }),
            ..
        } = task.chip.details.as_ref()
        {
            if self.run.is_none() {
                let handle = handle(self.next);
                self.next = self.next.checked_add(1).expect("chat handles fit");
                self.run = Some(handle);
                out.push(Request::Client(jig_client::Event::Watch {
                    handle,
                    subject: Watchable::Jig(c::Subject::Run { task: self.task, turn: None, offset: 0 }),
                }));
            }
        } else if let Some(handle) = self.run.take() {
            unwatch(handle, out);
        }
    }
    fn pending_key(&self, key: Key) -> bool {
        for (pending, _) in &self.pending {
            if *pending == key {
                return true;
            }
        }
        false
    }
    pub(crate) fn act(
        &mut self,
        env: &Env<Limits>,
        client: &Client,
        act: Act,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        match act {
            Act::MessageWords { words } => {
                if words.len() <= usize::try_from(env.limits.jig.text).expect("text bound fits") {
                    self.words = words;
                }
            }
            Act::Older => read(self.activity, self.thread.older(), out),
            Act::Send => self.send(env, client, Sending::Message, serial, out),
            Act::Stop => self.send(env, client, Sending::Stop, serial, out),
            Act::Seen => self.send(env, client, Sending::Seen, serial, out),
            Act::Project { .. }
            | Act::SignOut
            | Act::ChatWords { .. }
            | Act::StartChat
            | Act::ClosedChats
            | Act::Filter { .. }
            | Act::Decide { .. }
            | Act::CardWords { .. }
            | Act::Again { .. }
            | Act::Confirm
            | Act::Cancel
            | Act::Go { .. } => unreachable!("only conversation acts routed here"),
        }
    }
    fn send(&mut self, env: &Env<Limits>, client: &Client, kind: Sending, serial: &mut u64, out: &mut Queue<Request>) {
        let Some(task) = self.task(client) else { return };
        let ask = match kind {
            Sending::Message if !self.words.is_empty() && allowed(task, c::ActKind::Message) => c::Ask::Message {
                project: task.project,
                task: self.task,
                words: c::Text::new(self.words.clone(), env.limits.jig.text).expect("composer bounded"),
            },
            Sending::Stop if allowed(task, c::ActKind::Stop) => c::Ask::Stop { project: task.project, task: self.task },
            Sending::Seen if self.thread.loaded > self.seen => {
                c::Ask::Seen { through: c::SeenThrough::Conversation { task: self.task, through: self.thread.loaded } }
            }
            Sending::Message | Sending::Stop | Sending::Seen => return,
        };
        for (_, pending) in &self.pending {
            if *pending == kind {
                return;
            }
        }
        *serial = serial.checked_add(1).expect("chat keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        if let c::Ask::Message { words, .. } = &ask {
            let Some(Latest::Jig(c::Delivery::Me(me))) = latest(client, super::me_handle()) else {
                return;
            };
            if !self.thread.sending(key, me.party.number, words.clone(), 0) {
                return;
            }
            self.words = Box::default();
        }
        if kind == Sending::Seen {
            self.seen = self.thread.loaded;
        }
        self.pending.push((key, kind)).expect("one request per chat act fits");
        self.outcome = None;
        out.push(Request::Client(jig_client::Event::Send { key, request: jig_client::Ask::Jig(Box::new(ask)) }));
    }
}
#[must_use]
pub fn thread_limits(limits: &Limits) -> jig_client::thread::Limits {
    jig_client::thread::Limits { lines: limits.jig.rows, bytes: limits.jig.document_bytes, text: limits.jig.text }
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    jig_client::thread::worst_case(&thread_limits(limits))?
        .checked_add(List::<(Key, Sending)>::worst_case(3)?)?
        .checked_add(u64::from(limits.jig.text))?
        .checked_add(limits.jig.document_bytes)
}
fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}
fn unwatch(handle: Handle, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Unwatch { handle }));
}
fn read(handle: Handle, read: Option<c::Read>, out: &mut Queue<Request>) {
    if let Some(read) = read {
        out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }));
    }
}
pub(crate) fn from_reference(root: &crate::Domain) -> Option<crate::Address> {
    let crate::Address::Jig(jig_client_view::address::Address::Task { task: number, .. }) = root.pages().address()
    else {
        return None;
    };
    let lent = root.jig().borrow(root.client());
    let object = lent.objects().get(lent.task_page().header?)?;
    let jig_client_view::objects::Body::Task(task) = object.body? else { return None };
    let me = root.pages().me(root.client())?;
    let template = jig_client::shapes::template(me, task.project, jig_client::shapes::CHAT)?;
    if task.label == template.label { Some(crate::Address::Chat { task: *number }) } else { None }
}

#[must_use]
pub fn allowed(task: &c::Task, act: c::ActKind) -> bool {
    match jig_client::facts::offer(task.acts.as_slice(), act) {
        Some(c::Admission::Allowed | c::Admission::Bounded { .. }) => true,
        Some(c::Admission::Proposed { .. } | c::Admission::Refused(_)) | None => false,
    }
}
fn latest(client: &Client, handle: Handle) -> Option<&Latest> {
    client.document(handle)?.latest
}
