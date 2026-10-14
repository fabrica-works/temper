//! The git component: counted operations, explicit lifecycle, parent queues.
use crate::{
    Event, Identity, Limits, Outputs, Request, Settings, Unfit, Unusable, Version,
    invocation::Command,
    limits,
    operation::Operation,
    sequence::{self, Next},
};
use skein_io::{self as io, kernel};
use skein_lib::{Id, Map, Slab, Time, Token};
use temper_worker_domain_checkout::{self as checkout, git};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Start {
    Idle,
    Checking,
    Started,
    Unusable,
    Aborted,
}
/// One worker's git component; the service drives it and drains its outputs.
#[derive(Debug)]
pub struct Component {
    pub(crate) limits: Limits,
    pub(crate) checkout: checkout::Limits,
    pub(crate) settings: Settings,
    slots: Slab<Operation>,
    index: Map<Token, Id<Operation>>,
    serial: u64,
    start: Start,
    shutting_down: bool,
}
fn event(out: &mut Outputs, event: Event) {
    out.events.push(event);
}
impl Component {
    /// Refuse the limit that breaks git.md section 8 before allocating state.
    pub fn new(limits: &Limits, checkout: &checkout::Limits, settings: Settings) -> Result<Component, Unfit> {
        limits::check(limits, checkout)?;
        let capacity = limits.operations.checked_add(1).ok_or(Unfit::Memory)?;
        let slots = capacity.checked_mul(2).ok_or(Unfit::Memory)?;
        assert!(settings.largest_read > 0 && settings.largest_room > 0, "io validated usable limits");
        Ok(Component {
            limits: *limits,
            checkout: *checkout,
            settings,
            slots: Slab::with_capacity(slots),
            index: Map::with_capacity(capacity),
            serial: 0,
            start: Start::Idle,
            shutting_down: false,
        })
    }
    fn token(&mut self) -> Option<Token> {
        self.serial = self.serial.checked_add(1)?;
        if self.serial >= (1_u64 << 48_u32) {
            return None;
        }
        Some(Token::new(self.settings.prefix.checked_add(self.serial)?))
    }
    fn insert(&mut self, operation: Operation) -> Id<Operation> {
        let key = operation.io_owner;
        let id = self.slots.insert(operation).expect("live plus retired operations fit the slab");
        assert!(self.index.insert(key, id).expect("admitted index fits").is_none(), "unique invocation token");
        id
    }
    /// Start once with git's version check; the layout is added in 07.3b.
    pub fn start(&mut self, _now: Time, out: &mut Outputs) {
        assert_eq!(self.start, Start::Idle, "start once");
        self.start = Start::Checking;
        let token = self.token().expect("first invocation token");
        let mut operation = Operation::new(token, token, Time::from_nanos(u64::MAX), None, None, Command::Version);
        operation.spawn(token, Command::Version, &self.settings, out);
        self.insert(operation);
    }
    fn owned(&self, owner: Token) -> Option<Id<Operation>> {
        for (_, id) in &self.index {
            if self.slots.get(*id).expect("live index").owner == owner {
                return Some(*id);
            }
        }
        None
    }
    /// Run an operation or cancel the operation named by its echoed owner.
    pub fn down(&mut self, now: Time, request: Request, out: &mut Outputs) {
        match request {
            Request::Io { owner, op, deadline, identity } => self.begin(now, owner, op, deadline, identity, out),
            Request::CancelIo { owner } => {
                if let Some(id) = self.owned(owner) {
                    self.slots.get_mut(id).expect("live index").cancel(git::Fault::Cancelled, &self.checkout, out);
                }
            }
            Request::Shutdown => self.shutting_down = true,
        }
    }
    fn begin(
        &mut self,
        now: Time,
        owner: Token,
        op: git::Op,
        deadline: Time,
        identity: Option<Identity>,
        out: &mut Outputs,
    ) {
        let needs_identity = match op {
            git::Op::Clone { .. }
            | git::Op::Fetch { .. }
            | git::Op::Create { .. }
            | git::Op::Commit { .. }
            | git::Op::Push { .. } => true,
            git::Op::Make { .. } | git::Op::CheckOut { .. } | git::Op::Merge { .. } => false,
        };
        let unsupported = match op {
            git::Op::Make { .. } | git::Op::Commit { merging: Some(_), .. } => true,
            git::Op::Clone { .. }
            | git::Op::Fetch { .. }
            | git::Op::Create { .. }
            | git::Op::CheckOut { .. }
            | git::Op::Merge { .. }
            | git::Op::Commit { merging: None, .. }
            | git::Op::Push { .. } => false,
        };
        if self.start != Start::Started
            || self.index.len() >= self.limits.operations
            || self.slots.is_full()
            || self.owned(owner).is_some()
            || (needs_identity && identity.is_none())
            || unsupported
        {
            event(out, Event::Done { owner, done: git::Done::Failed { fault: git::Fault::Broken } });
            return;
        }
        if now >= deadline {
            event(out, Event::Done { owner, done: git::Done::Failed { fault: git::Fault::TimedOut } });
            return;
        }
        let Some(token) = self.token() else {
            event(out, Event::Done { owner, done: git::Done::Failed { fault: git::Fault::Broken } });
            return;
        };
        let command = match sequence::first(&op) {
            Next::Run(command) => command,
            Next::Make | Next::Scan { .. } | Next::End(_) => unreachable!("supported first invocation"),
        };
        let identity = if needs_identity { identity } else { None };
        let mut operation = Operation::new(owner, token, deadline, Some(op), identity, command.clone());
        operation.spawn(token, command, &self.settings, out);
        self.insert(operation);
    }
    /// Route only events accepted by the invocation or one of its pipe tokens.
    pub fn up(&mut self, now: Time, input: io::Event, out: &mut Outputs) {
        let Some(owner) = io_owner(&input) else {
            return;
        };
        let mut found = None;
        for (_, id) in &self.index {
            if self.slots.get(*id).expect("live index").accepts(owner) {
                found = Some(*id);
                break;
            }
        }
        let Some(id) = found else {
            return;
        };
        let operation = self.slots.get_mut(id).expect("live index");
        let startup = operation.op.is_none();
        let version = operation.found_version();
        if !startup && now >= operation.deadline {
            operation.cancel(git::Fault::TimedOut, &self.checkout, out);
        }
        let next = operation.up(input, &self.settings, &self.limits, &self.checkout, out);
        if let Some(next) = next {
            if startup {
                let aborted = operation.aborted;
                self.retire(id);
                if aborted {
                    self.start = Start::Aborted;
                    return;
                }
                if version >= Some(Version { major: 2, minor: 41, patch: 0 }) {
                    self.start = Start::Started;
                    event(out, Event::Started);
                } else {
                    self.start = Start::Unusable;
                    event(out, Event::Unusable { why: Unusable::Version { found: version } });
                }
            } else {
                self.advance(id, next, out);
            }
        }
    }
    fn retire(&mut self, id: Id<Operation>) {
        let operation = self.slots.get_mut(id).expect("live index");
        operation.identity = None;
        operation.op = None;
        operation.command = Command::Version;
        let key = operation.io_owner;
        self.index.remove(&key).expect("live index");
        self.slots.retire(id);
    }
    fn advance(&mut self, id: Id<Operation>, next: Next, out: &mut Outputs) {
        match next {
            Next::Run(command) => {
                let Some(token) = self.token() else {
                    self.finish(
                        id,
                        git::Done::FailedWithOutput {
                            fault: git::Fault::Broken,
                            diagnostic: git::PushDiagnostic::empty(),
                        },
                        out,
                    );
                    return;
                };
                let operation = self.slots.get_mut(id).expect("live index");
                let old = operation.io_owner;
                self.index.remove(&old).expect("live index");
                operation.spawn(token, command, &self.settings, out);
                assert!(self.index.insert(token, id).expect("same operation").is_none(), "unique token");
            }
            Next::End(done) => self.finish(id, done, out),
            Next::Make | Next::Scan { .. } => self.finish(id, git::Done::Failed { fault: git::Fault::Broken }, out),
        }
    }
    fn finish(&mut self, id: Id<Operation>, done: git::Done, out: &mut Outputs) {
        let operation = self.slots.get(id).expect("live index");
        if !operation.aborted {
            event(out, Event::Done { owner: operation.owner, done });
        }
        self.retire(id);
    }
    /// File completions are owned by the next increment's file machine.
    pub fn complete(&mut self, _now: Time, _complete: kernel::Complete, _out: &mut Outputs) {}
    /// Kill and settle at most one operation whose one deadline has elapsed.
    pub fn fire(&mut self, now: Time, out: &mut Outputs) {
        for (_, id) in &self.index {
            let operation = self.slots.get_mut(*id).expect("live index");
            if operation.op.is_some() && operation.stop.is_none() && operation.deadline <= now {
                operation.cancel(git::Fault::TimedOut, &self.checkout, out);
                break;
            }
        }
    }
    /// Abort each child and pipe; retain a pending spawn until it can settle.
    pub fn abort(&mut self, out: &mut Outputs) {
        self.start = Start::Aborted;
        self.shutting_down = true;
        for (_, id) in &self.index {
            let operation = self.slots.get_mut(*id).expect("live index");
            operation.aborted = true;
            operation.identity = None;
            operation.cancel(git::Fault::Cancelled, &self.checkout, out);
        }
    }
    #[must_use]
    pub fn accepts(&self, event: &io::Event) -> bool {
        let Some(owner) = io_owner(event) else {
            return false;
        };
        for (_, id) in &self.index {
            if self.slots.get(*id).expect("live index").accepts(owner) {
                return true;
            }
        }
        false
    }
    #[must_use]
    pub fn owns(&self, operation: Token) -> bool {
        operation.raw().checked_sub(self.settings.prefix).unwrap_or(u64::MAX) < (1_u64 << 48_u32)
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut earliest = None;
        for (_, id) in &self.index {
            let operation = self.slots.get(*id).expect("live index");
            if operation.op.is_some()
                && operation.stop.is_none()
                && (earliest.is_none() || Some(operation.deadline) < earliest)
            {
                earliest = Some(operation.deadline);
            }
        }
        earliest
    }
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.next_deadline() {
            Some(deadline) => deadline <= now,
            None => false,
        }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
    #[must_use]
    pub fn operations(&self) -> u32 {
        self.limits.operations.checked_add(1).expect("validated operation count")
    }
    /// Reclaim retired state after the loop's down pass.
    pub fn reclaim(&mut self) {
        self.slots.reclaim();
    }
}
fn io_owner(event: &io::Event) -> Option<Token> {
    match event {
        io::Event::Listening { owner, .. }
        | io::Event::Accepted { owner, .. }
        | io::Event::Connecting { owner, .. }
        | io::Event::Connected { owner, .. }
        | io::Event::Stream { owner, .. }
        | io::Event::Output { owner, .. }
        | io::Event::Spawned { owner, .. }
        | io::Event::Exited { owner, .. }
        | io::Event::Usage { owner, .. }
        | io::Event::Window { owner, .. }
        | io::Event::Failed { owner, .. }
        | io::Event::Closed { owner } => Some(*owner),
        io::Event::Resized | io::Event::Shutdown { .. } => None,
    }
}
