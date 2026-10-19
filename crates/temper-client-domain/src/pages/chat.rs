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
    Close,
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "departures are private to the chat refresh and exposed as a slice")]
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
    pub cards: List<super::inbox::Card>,
    departures: List<c::ObjectKey>,
    pub closing: Option<c::Ask>,
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
            pending: List::with_capacity(4),
            outcome: None,
            seen: 0,
            cards: List::with_capacity(limits.client.domain.reads),
            departures: List::with_capacity(limits.client.domain.reads),
            closing: None,
            final_task: None,
        }
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [Some(self.document), Some(self.activity), self.run].into_iter().flatten() {
            unwatch(handle, out);
        }
        for card in &self.cards {
            unwatch(card.handle, out);
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
                    self.proposals(task, out);
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
                    self.proposals(&task, out);
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
            Told::Page { handle, page } if self.card_handle(handle) => {
                for index in 0..self.cards.len() {
                    let card = self.cards.get_mut(index).expect("bounded card");
                    if card.handle == handle {
                        card.page = match page {
                            Readout::Jig(c::ReadResult::Value(value)) => Some(value),
                            Readout::Limit | Readout::NotServed { .. } | Readout::Jig(_) | Readout::Family { .. } => {
                                None
                            }
                        };
                        break;
                    }
                }
            }
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
    fn card_handle(&self, handle: Handle) -> bool {
        for card in &self.cards {
            if card.handle == handle {
                return true;
            }
        }
        false
    }
    fn card(&self, object: c::ObjectKey) -> Option<&super::inbox::Card> {
        for index in 0..self.cards.len() {
            let card = self.cards.get(index)?;
            if card.object == object {
                return Some(card);
            }
        }
        None
    }
    /// Proposals the last task document removed from this chat.
    #[must_use]
    pub fn departures(&self) -> &[c::ObjectKey] {
        self.departures.as_slice()
    }
    fn sending(&self, kind: Sending) -> bool {
        for (_, pending) in &self.pending {
            if *pending == kind {
                return true;
            }
        }
        false
    }
    fn proposals(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        self.departures.clear();
        for card in &self.cards {
            let c::ObjectKey::Proposal(key) = card.object else { continue };
            if !task.proposals.as_slice().contains(&key) {
                unwatch(card.handle, out);
                self.departures.push(card.object).expect("removed cards fit their window");
            }
        }
        let mut ordered = List::with_capacity(self.cards.capacity());
        for key in &task.proposals {
            let object = c::ObjectKey::Proposal(*key);
            if ordered.len() == ordered.capacity() {
                break;
            }
            if let Some(card) = self.card(object) {
                ordered.push(card.clone()).expect("retained cards fit");
                continue;
            }
            let handle = handle(self.next);
            self.next = self.next.checked_add(1).expect("chat handles fit");
            ordered
                .push(super::inbox::Card {
                    object,
                    group: c::InboxGroup::Proposals,
                    project: Some(task.project),
                    position: None,
                    handle,
                    page: None,
                    words: None,
                    key: None,
                    outcome: None,
                })
                .expect("bounded cards");
            read(handle, Some(c::Read::Proposal { proposal: *key }), out);
        }
        self.cards = ordered;
    }

    pub(crate) fn prepare_close(&mut self, client: &Client) {
        self.closing = None;
        let Some(task) = self.task(client) else { return };
        if !allowed(task, c::ActKind::Cancel) || self.sending(Sending::Close) {
            return;
        }
        let Some(Latest::Jig(c::Delivery::Me(me))) = latest(client, super::me_handle()) else { return };
        let mut delegates = List::with_capacity(task.delegates.len());
        for delegate in &task.delegates {
            delegates
                .push(jig_client::shapes::Delegate { task: delegate.chip.task, fate: jig_client::shapes::Fate::Cancel })
                .expect("delegates fit");
        }
        let reason = c::Text::new(Box::from(*b"Chat closed"), u32::MAX).expect("bounded close reason");
        self.closing = jig_client::shapes::close_chat(me, task.project, self.task, delegates.as_slice(), reason).ok();
    }
    pub(crate) fn delegate_fate(&mut self, task: u64, fate: jig_client::shapes::Fate) {
        if let Some(c::Ask::Compound { project, operations }) = &mut self.closing {
            for index in 0..operations.len() {
                let operation = operations.get_mut(index).expect("bounded operation");
                match operation {
                    c::Operation::Cancel { task: number, reason, .. }
                    | c::Operation::Move { task: number, reason, .. }
                        if *number == task && task != self.task =>
                    {
                        *operation = match fate {
                            jig_client::shapes::Fate::Cancel => {
                                c::Operation::Cancel { project: *project, task, reason: reason.clone(), grace: None }
                            }
                            jig_client::shapes::Fate::Move => {
                                c::Operation::Move { project: *project, task, reason: reason.clone() }
                            }
                        };
                        break;
                    }
                    c::Operation::Message { .. }
                    | c::Operation::Amend { .. }
                    | c::Operation::Cancel { .. }
                    | c::Operation::Release { .. }
                    | c::Operation::Stop { .. }
                    | c::Operation::Move { .. } => {}
                }
            }
        }
    }
    pub(crate) fn close(&mut self, serial: &mut u64, out: &mut Queue<Request>) {
        let Some(ask) = self.closing.take() else { return };
        if self.sending(Sending::Close) {
            return;
        }
        *serial = serial.checked_add(1).expect("chat keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending.push((key, Sending::Close)).expect("one close fits");
        self.outcome = None;
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
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
                if words.len() <= usize::try_from(env.limits.client.domain.text).expect("text bound fits") {
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
            | Act::Close
            | Act::DelegateFate { .. }
            | Act::Go { .. } => unreachable!("only conversation acts routed here"),
        }
    }
    fn send(&mut self, env: &Env<Limits>, client: &Client, kind: Sending, serial: &mut u64, out: &mut Queue<Request>) {
        let Some(task) = self.task(client) else { return };
        let ask = match kind {
            Sending::Message if !self.words.is_empty() && allowed(task, c::ActKind::Message) => c::Ask::Message {
                project: task.project,
                task: self.task,
                words: c::Text::new(self.words.clone(), env.limits.client.domain.text).expect("composer bounded"),
            },
            Sending::Stop if allowed(task, c::ActKind::Stop) => c::Ask::Stop { project: task.project, task: self.task },
            Sending::Seen if self.thread.loaded > self.seen => {
                c::Ask::Seen { through: c::SeenThrough::Conversation { task: self.task, through: self.thread.loaded } }
            }
            Sending::Message | Sending::Stop | Sending::Seen | Sending::Close => return,
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
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
    }
}
#[must_use]
pub fn thread_limits(limits: &Limits) -> jig_client::thread::Limits {
    jig_client::thread::Limits {
        lines: limits.client.domain.rows,
        bytes: limits.client.domain.document_bytes,
        text: limits.client.domain.text,
    }
}
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    jig_client::thread::worst_case(&thread_limits(limits))?
        .checked_add(List::<(Key, Sending)>::worst_case(4)?)?
        .checked_add(u64::from(limits.client.domain.text))?
        .checked_add(limits.client.domain.document_bytes)?
        // Refresh can briefly retain old cards beside their replacements.
        .checked_add(super::inbox::worst_case(limits)?)?
        .checked_add(List::<c::ObjectKey>::worst_case(limits.client.domain.reads)?)?
        // A received task's delegate list is bounded by its document bytes.
        .checked_add(List::<c::Operation>::worst_case(
            u32::try_from(limits.client.domain.document_bytes).ok()?.checked_add(1)?,
        )?)?
        .checked_add(
            u64::from(u32::try_from(limits.client.domain.document_bytes).ok()?.checked_add(1)?).checked_mul(11)?,
        )
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
