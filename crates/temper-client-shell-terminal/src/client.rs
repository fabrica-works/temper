//! Root, view, link and native transport as one host (client.md, 6.2).
use crate::{
    limits,
    route::{self, Down, Duty, Up},
};
use jig_protocol_client_link as link;
use jig_shell_native::{Shell, Signing};
use skein_io::kernel::{Complete, Fd, Submit};
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_shell::Host;
use temper_client_domain as domain;
use temper_client_view as view;

/// The engine, the adopted directory's last component, and fixed capacities.
#[derive(Clone, Debug)]
pub struct Settings {
    pub engine: skein_io::Addr,
    pub directory: Box<[u8]>,
    pub limits: limits::Limits,
}
/// How the caller established the sign-in; credentials remain in the shell.
#[derive(Debug)]
pub enum Start {
    Link(Box<[u8]>),
    Token,
}
/// Facts counted by the host, outside the client domain.
#[derive(Debug, Default)]
pub struct Seen {
    pub signed_in: u32,
    pub patches: u64,
    pub builds: u64,
    pub facts: u32,
}
/// Notices belong to the caller; no secret enters the root.
#[derive(Debug)]
pub enum Shown {
    Notice(route::Notice),
    SignInAsked,
    Terminal(crate::terminal::Notice),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    WaitingLink,
    WaitingToken,
    Running,
    Closing { started: bool },
}
/// The composition holds no person; its caller drives the semantic tree.
pub struct NativeClient {
    shell: Shell,
    link: link::Link,
    family: route::family::Family,
    root: domain::Domain,
    view: view::View,
    env: Env<domain::Limits>,
    out: Queue<domain::Request>,
    linked: Queue<link::Request>,
    patches: Queue<skein_ui::Patch>,
    kept: List<domain::store::Record>,
    address: domain::Address,
    phase: Phase,
    dirty: bool,
    signing_out: bool,
    notices: Queue<Shown>,
    signing: Queue<Signing>,
    seen: Seen,
    memory: u64,
    loaded: bool,
}
impl NativeClient {
    /// Adopt the caller's private directory; validate a printed link before sending.
    /// # Errors
    /// Refuses a printed link that does not name the configured engine.
    pub fn new(
        settings: Settings,
        directory: Fd,
        user: u32,
        seed: u64,
        start: Start,
    ) -> Result<Self, jig_shell_native::Unlinked> {
        let bounds = settings.limits;
        let mut memory = limits::worst_case(&bounds).expect("native capacities are checked");
        let (view_limits, jig_words, forge_words) = limits::view(&bounds).expect("native view capacities are checked");
        let link =
            link::Link::new(&limits::link(&bounds).expect("native link capacities are checked")).expect("checked link");
        let notices = link.max_out().checked_add(domain::max_out(&bounds.client)).expect("notice capacity");
        let records = bounds
            .client
            .jig
            .records
            .checked_add(bounds.client.jig.kept)
            .and_then(|n| n.checked_add(bounds.client.jig.parts))
            .expect("kept record capacity");
        memory = memory
            .checked_add(Queue::<link::Request>::worst_case(link.max_out()).expect("link output memory"))
            .and_then(|n| n.checked_add(Queue::<Shown>::worst_case(notices)?))
            .and_then(|n| n.checked_add(Queue::<Signing>::worst_case(limits::EXCHANGES)?))
            .and_then(|n| n.checked_add(List::<domain::store::Record>::worst_case(records)?))
            .and_then(|n| n.checked_add(u64::from(records).checked_mul(bounds.client.jig.document_bytes)?))
            .and_then(|n| n.checked_add(u64::from(notices).checked_mul(40)?))
            .expect("native queues fit memory");
        let linked = Queue::with_capacity(link.max_out());
        let mut shell = Shell::new(jig_shell_native::Settings {
            engine: settings.engine,
            directory: settings.directory,
            limits: bounds.connections,
            io: bounds.io,
        });
        let token_start = match start {
            Start::Link(printed) => {
                shell.sign_in(&printed)?;
                false
            }
            Start::Token => true,
        };
        shell.adopt(directory, user, seed);
        let mut root = domain::new(
            &bounds.client,
            seed,
            domain::forge::Config {
                connector: temper_client_protocol_wire::FORGE,
                version: temper_client_protocol_wire::CLIENT_VERSION,
            },
        );
        domain::configure_navigation(&mut root, domain::navigation(&bounds.client)).expect("native navigation fits");
        Ok(Self {
            shell,
            link,
            family: limits::family(&bounds).expect("native family capacities are checked"),
            root,
            view: view::View::new(
                &view_limits,
                &bounds.client,
                jig_words,
                forge_words,
                temper_client_protocol_wire::FORGE,
            ),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: bounds.client },
            out: Queue::with_capacity(domain::max_out(&bounds.client)),
            linked,
            patches: Queue::with_capacity(view_limits.tree.patches),
            kept: List::with_capacity(records),
            address: domain::Address::Jig(jig_domain_client::address::Address::Inbox { project: None }),
            phase: if token_start { Phase::WaitingToken } else { Phase::WaitingLink },
            dirty: false,
            signing_out: false,
            notices: Queue::with_capacity(notices),
            signing: Queue::with_capacity(limits::EXCHANGES),
            seen: Seen::default(),
            memory,
            loaded: false,
        })
    }
    #[must_use]
    pub fn tree(&self) -> &skein_ui::Tree {
        self.view.tree()
    }
    /// Apply an offered act; false means no binding or an already closing client.
    pub fn act(&mut self, node: skein_ui::NodeId, act: skein_ui::Act) -> bool {
        if matches!(self.phase, Phase::Closing { .. }) {
            return false;
        }
        let Some(action) = view::act(&mut self.view, node, act) else {
            return false;
        };
        self.event(domain::Event::Act { action });
        true
    }
    /// Navigation is a root event, even when its address did not change.
    pub fn go(&mut self, address: domain::Address) {
        if !matches!(self.phase, Phase::Closing { .. }) {
            self.address = address.clone();
            self.event(domain::Event::Went { place: domain::Place::Address(address) });
        }
    }
    /// # Errors
    /// Refuses a link outside the configured engine's address.
    pub fn sign_in(&mut self, printed: &[u8]) -> Result<(), jig_shell_native::Unlinked> {
        self.shell.sign_in(printed)
    }
    /// Each sign-in outcome is handed to the caller unchanged, once.
    pub fn signing(&mut self) -> Option<Signing> {
        self.signing.pop()
    }
    /// Drain the caller's notices, including credentials to show once.
    pub fn notices(&mut self) -> Option<Shown> {
        self.notices.pop()
    }
    #[must_use]
    pub const fn started(&self) -> bool {
        match self.phase {
            Phase::Running => true,
            Phase::Closing { started } => started,
            Phase::WaitingLink | Phase::WaitingToken => false,
        }
    }
    #[must_use]
    pub const fn seen(&self) -> &Seen {
        &self.seen
    }
    /// Finish the last small-store write and close; start no more exchanges.
    pub fn close(&mut self) {
        if !matches!(self.phase, Phase::Closing { .. }) {
            self.phase = Phase::Closing { started: self.started() };
            if self.started() {
                domain::take(&mut self.root, &self.env, &mut self.out);
            }
            self.outputs();
            self.shell.close();
        }
    }
    fn event(&mut self, event: domain::Event) {
        if let Some(notice) = crate::terminal::native::event(&event) {
            self.notices.push(Shown::Terminal(notice));
        }
        domain::step(&mut self.root, &self.env, event, &mut self.out);
        self.dirty = true;
    }
    fn start(&mut self) {
        if matches!(self.phase, Phase::WaitingLink | Phase::WaitingToken) {
            self.phase = Phase::Running;
            self.notices.push(Shown::Terminal(crate::terminal::Notice::SignedIn));
            let capacity = self.kept.capacity();
            let kept = std::mem::replace(&mut self.kept, List::with_capacity(capacity));
            self.event(domain::Event::Start {
                address: self.address.clone(),
                kept,
                offset: jig_domain_client::boundary::Offset { minutes: 0 },
            });
        }
    }
    fn outputs(&mut self) {
        // A link event can synchronously emit another root request. Drain the
        // two bounded queues together before admitting the next external event.
        while !self.out.is_empty() || !self.linked.is_empty() {
            while let Some(request) = self.out.pop() {
                match route::down(request, &mut self.family) {
                    Down::Link(event) => self.link.step(self.env.now, event, &mut self.linked),
                    Down::Duty(duty) => match duty {
                        Duty::Show { address, .. } => {
                            if self.address != address {
                                self.go(address);
                            }
                        }
                        Duty::SignIn { .. } => self.notices.push(Shown::SignInAsked),
                        Duty::SignOut => {
                            if !matches!(self.phase, Phase::Closing { .. }) {
                                self.signing_out = true;
                                self.shell.sign_out();
                            }
                        }
                        Duty::Fact(_) => self.seen.facts = self.seen.facts.saturating_add(1),
                    },
                    Down::Nothing => {}
                }
            }
            while let Some(request) = self.linked.pop() {
                match route::up(request, &mut self.family) {
                    Up::Root(event) => self.event(event),
                    Up::Shell(request) => {
                        if matches!(self.phase, Phase::Closing { .. })
                            && matches!(request, link::Request::Exchange { .. } | link::Request::OpenStream)
                        {
                            continue;
                        }
                        self.shell.down(request);
                    }
                    Up::Kept(record) => self.kept.push(record).expect("link's kept records fit"),
                    Up::Notice(notice) => {
                        self.seen.facts = self.seen.facts.saturating_add(1);
                        self.notices.push(Shown::Notice(notice));
                    }
                }
            }
        }
    }
}
impl Host for NativeClient {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        self.shell.iterate(now, wall);
        if matches!(self.phase, Phase::Closing { .. }) {
            while self.shell.events().pop().is_some() {}
            return;
        }
        if let Some(records) = self.shell.loaded() {
            self.loaded = true;
            let mut versions = List::with_capacity(2);
            for version in route::versions() {
                versions.push(version).expect("two store owners fit");
            }
            let mut kept = List::with_capacity(u32::try_from(records.len()).expect("shell's bounded store"));
            for record in records {
                kept.push(record).expect("exact record capacity");
            }
            self.link.step(now, link::Event::Start { versions, kept }, &mut self.linked);
            self.outputs();
            if self.phase == Phase::WaitingToken {
                self.start();
            }
        }
        while let Some(signing) = self.shell.signing() {
            if let Some(notice) = crate::terminal::native::signing(&signing, self.signing_out) {
                self.notices.push(Shown::Terminal(notice));
            }
            match &signing {
                Signing::SignedIn { .. } => {
                    self.seen.signed_in = self.seen.signed_in.saturating_add(1);
                    self.start();
                }
                Signing::SignedOut { .. } => {
                    self.signing_out = false;
                    self.event(domain::Event::SignedOut);
                }
                Signing::Again { .. } | Signing::Lost { .. } | Signing::Refused { .. } => {
                    if self.signing_out {
                        self.signing_out = false;
                        self.event(domain::Event::Jig(jig_domain_client::Event::SignOutLost));
                    }
                }
            }
            self.signing.push(signing);
            self.outputs();
        }
        while let Some(event) = self.shell.events().pop() {
            if matches!(event, link::Event::WriteFailed) {
                self.notices.push(Shown::Terminal(crate::terminal::Notice::StoreFailed));
            }
            self.link.step(now, event, &mut self.linked);
            self.outputs();
        }
        if self.started() {
            domain::fire(&mut self.root, &self.env, &mut self.out);
            self.outputs();
            domain::take(&mut self.root, &self.env, &mut self.out);
            self.outputs();
            self.link.fire(now, &mut self.linked);
            self.outputs();
            view::build(&mut self.view, &self.root, &mut self.patches);
            if !self.patches.is_empty() {
                self.seen.builds = self.seen.builds.saturating_add(1);
            }
            while self.patches.pop().is_some() {
                self.seen.patches = self.seen.patches.saturating_add(1);
            }
            self.dirty = false;
        }
        if !self.loaded
            && !self.shell.work_pending(now)
            && self.shell.next_deadline().is_none()
            && self.shell.submissions().is_empty()
            && self.shell.completions().is_empty()
        {
            self.notices.push(Shown::Terminal(crate::terminal::Notice::DirectoryRefused));
            self.close();
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.shell.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.shell.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.shell.work_pending(now)
            || !self.out.is_empty()
            || !self.linked.is_empty()
            || (!matches!(self.phase, Phase::Closing { .. }) && self.started() && self.dirty)
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.shell.next_deadline(), self.next_policy_deadline()].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        if matches!(self.phase, Phase::Closing { .. }) {
            None
        } else {
            [domain::next_deadline(&self.root), self.link.next_deadline()].into_iter().flatten().min()
        }
    }
    fn is_empty(&self) -> bool {
        matches!(self.phase, Phase::Closing { .. }) && self.shell.is_empty()
    }
    fn worst_case(&self) -> u64 {
        self.memory
    }
    fn operations(&self) -> u32 {
        self.shell.operations()
    }
}
