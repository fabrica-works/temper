//! Temper's engine and worker protocol services.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod limits;
pub mod settings;
pub mod store;
#[cfg(test)]
mod tests;
pub mod tokens;

pub mod accounts;
pub mod values;
pub use accounts::AccountsRuntime;
mod engine;
mod route;
mod stop;
/// How the service ended (protocol.md, 6).
#[derive(PartialEq, Eq, Debug)]
pub enum End {
    Drained,
    Aborted,
    Stopped(Stop),
}
/// Why the service stopped; failed startup names its configuration paths.
#[derive(PartialEq, Eq, Debug)]
pub enum Stop {
    CommitFailed { number: u64 },
    Start(Problems),
    Listener,
}
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, What};
use skein_io::{
    self as io, Io,
    kernel::{Complete, Fd, Submit},
};
use skein_lib::{Env, List, Map, Queue, Time, Token, Wall};
/// The engine root, its protocol children and the kernel queues.
#[expect(missing_debug_implementations, reason = "the service owns account token records")]
pub struct Service {
    engine: engine::Engine,
    io: Io,
    io_env: Env<io::Limits>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    raw_submissions: Queue<Submit>,
    kernel: accounts::Kernel,
    told: Queue<io::Event>,
    signals: Option<Token>,
    signals_closing: bool,
    bytes: u64,
    links: Queue<Box<[u8]>>,
}
impl Service {
    /// Construct the root, its deployment store and its configured accounts.
    pub fn engine(
        mut settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
    ) -> Result<Self, Problems> {
        let limits =
            limits::engine(&settings.jig.limits, &settings.limits).ok_or(start_problem(b"limits", What::Bound))?;
        if temper_engine_domain::worst_case(&limits).is_none() {
            return Err(start_problem(b"limits", What::Bound));
        }
        for charter in &mut settings.jig.charters {
            charter.policy.instructions = settings
                .files
                .contents(&charter.policy.instructions)
                .ok_or(start_problem(b"charters", What::Unreadable))?
                .into();
        }
        let store_limits = store::limits(&limits, &settings.jig.limits.core, &limits.forge, &settings.jig.limits.store)
            .ok_or(start_problem(b"limits", What::Bound))?;
        let max = temper_engine_domain::max_out(&limits);
        let queue = max.checked_mul(2).ok_or(start_problem(b"limits", What::Bound))?;
        let operations = io::operations(&settings.jig.limits.io)
            .ok_or(start_problem(b"limits", What::Bound))?
            .checked_add(queue.checked_mul(4).ok_or(start_problem(b"limits", What::Bound))?)
            .ok_or(start_problem(b"limits", What::Bound))?;
        let bytes = memory(&settings, &limits, &store_limits, accounts.as_ref(), queue, operations)
            .ok_or(start_problem(b"memory_bytes", What::Bound))?;
        if let Err(what) =
            jig_protocol_config::fits(settings.jig.memory, &[jig_protocol_config::Share { name: b"service", bytes }])
        {
            return Err(start_problem(b"memory_bytes", what));
        }
        let io_limits = settings.jig.limits.io;
        let accounts = if settings.jig.llm.accounts.is_empty() {
            None
        } else {
            Some(accounts::Accounts::new(
                &settings.jig,
                &settings.files,
                seeds,
                accounts,
                max.checked_mul(6).expect("account routing room"),
            )?)
        };
        let values = values::Values::new(
            limits.accounts.accounts.checked_add(1).expect("numbered accounts"),
            settings.jig.limits.driver.client.document.token_bytes,
        );
        let count = u32::try_from(settings.forges.len()).expect("configured forges");
        let mut writers = List::with_capacity(count);
        let mut identities = List::with_capacity(count);
        for forge in &settings.forges {
            writers
                .push(temper_engine_domain_forge_client::Writer { forge: forge.number, author: forge.user })
                .expect("forge writers");
            identities
                .push(temper_engine_domain_forge::GitIdentity { forge: forge.number, credential: forge.identity })
                .expect("forge identities");
        }
        let mut first_periods = List::with_capacity(limits.authority.projects);
        for project in &settings.jig.projects {
            first_periods
                .push((project.number, project.policy.period_spend))
                .or(Err(start_problem(b"projects", What::Bound)))?;
        }
        let config = temper_engine_domain::Config {
            core: jig_protocol_config::core(&settings.jig, seeds.deployment, seeds.seed),
            landing: settings.landing,
            forge: temper_engine_domain_forge_client::Config {
                namespace: Box::new(seeds.deployment),
                writers: writers.into_boxed(),
            },
            identities: identities.into_boxed(),
            forge_charter: settings.charter,
        };
        Ok(Self {
            engine: engine::Engine::new(
                config,
                &limits,
                store::Store::new(&limits, &store_limits),
                accounts,
                values,
                first_periods,
            ),
            io: Io::new(&io_limits),
            io_env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: io_limits },
            completions: Queue::with_capacity(queue),
            submissions: Queue::with_capacity(queue),
            raw_submissions: Queue::with_capacity(queue),
            told: Queue::with_capacity(queue),
            kernel: accounts::Kernel::new(operations),
            signals: None,
            signals_closing: false,
            bytes,
            links: Queue::with_capacity(queue),
        })
    }
    /// Adopt the binary's deployment directory.
    pub fn adopt_store(&mut self, root: Fd, effective_user: u32) -> Result<(), Fd> {
        self.engine.store.adopt(root, effective_user)
    }
    /// Earliest deadline of the root and its children.
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        self.engine.next_deadline()
    }
    /// Whether an iteration has work on the supplied clock.
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.engine.work_pending(now)
            || self.io.is_ready()
            || self.io.is_due(now)
            || !self.completions.is_empty()
            || !self.told.is_empty()
            || !self.raw_submissions.is_empty()
            || self.end().is_some() && self.signals.is_some() && !self.signals_closing
    }
    /// A first call drains; another aborts.
    pub fn shutdown(&mut self) {
        stop::shutdown(&mut self.engine);
    }
    /// How the engine ended, once it has.
    #[must_use]
    pub const fn end(&self) -> Option<&End> {
        self.engine.end.as_ref()
    }
    /// Adopt the shell's termination-signal source.
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> {
        if self.signals.is_some() {
            return Err(descriptor);
        }
        match self.io.adopt_signals_for(descriptor, Token::new(1)) {
            Ok(source) => {
                self.signals = Some(source);
                Ok(())
            }
            Err(descriptor) => Err(descriptor),
        }
    }
    /// Kernel completions, filled by the shell.
    pub const fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    /// Kernel submissions, taken by the shell.
    pub const fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    /// Printed sign-in links, taken by the binary.
    pub const fn links(&mut self) -> &mut Queue<Box<[u8]>> {
        &mut self.links
    }
    /// Root, component and I/O deadlines on the injected clock.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        accounts::earliest(self.next_policy_deadline(), self.io.next_deadline())
    }
    /// The checked memory envelope for everything this service owns.
    #[must_use]
    pub const fn worst_case(&self) -> u64 {
        self.bytes
    }
    /// Maximum simultaneous kernel operations, for the shell's ring.
    #[must_use]
    pub fn operations(&self) -> u32 {
        io::operations(&self.io_env.limits)
            .expect("checked io operations")
            .checked_add(self.raw_submissions.capacity().checked_mul(4).expect("component operations"))
            .expect("service operations")
    }
    /// Every owned child has closed and every queue and kernel route is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.end().is_some()
            && self.engine.store.closed()
            && self.io.is_empty()
            && self.completions.is_empty()
            && self.submissions.is_empty()
            && self.raw_submissions.is_empty()
            && self.told.is_empty()
            && self.kernel.is_empty()
            && match &self.engine.accounts {
                Some(accounts) => accounts.is_empty(),
                None => true,
            }
    }
    fn room(&self, max: io::MaxOut) -> bool {
        self.told.room() >= max.events && self.raw_submissions.room() >= max.submissions
    }
    fn route_components(&mut self) {
        for _ in 0..self.engine.store.submitted.len() {
            if self.raw_submissions.room() == 0 {
                break;
            }
            self.raw_submissions.push(self.engine.store.submitted.pop().expect("store submissions"));
        }
        if (self.engine.store.opened || self.engine.end.is_some())
            && let Some(accounts) = &mut self.engine.accounts
        {
            for _ in 0..accounts.io.len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                    break;
                }
                let request = accounts::tag(accounts.io.pop().expect("account io requests"));
                io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
            }
        }
    }
}
fn start_problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(member), what);
    problems
}
/// Drive store terminals, account events, one root input and released outputs.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    service.io_env.now = now;
    service.io_env.wall = wall;
    if let Some(accounts) = &mut service.engine.accounts {
        accounts.clock(now, wall);
    }
    for _ in 0..service.io_env.limits.sockets.checked_add(service.io_env.limits.refusals).expect("io entries") {
        if !service.io.is_ready() || !service.room(io::MAX_OUT_RESUME) {
            break;
        }
        io::resume(&mut service.io, &service.io_env, &mut service.told, &mut service.raw_submissions);
    }
    if !service.io.is_ready() {
        for _ in 0..service.completions.len() {
            if !service.room(io::MAX_OUT_UP) {
                break;
            }
            let complete = service.completions.pop().expect("completion count");
            let files = pending_files(service.engine.accounts.as_ref());
            let (files, complete) = service.kernel.complete(complete, &service.raw_submissions, files);
            if files {
                service.engine.accounts.as_mut().expect("file owner").complete(complete);
            } else if store::owns(complete.op) {
                service.engine.store.complete(complete);
            } else {
                io::up(&mut service.io, &service.io_env, complete, &mut service.told, &mut service.raw_submissions);
            }
        }
        if service.completions.is_empty() {
            for _ in 0..service.io_env.limits.sockets.checked_mul(2).expect("io timers") {
                if !service.io.is_due(now) || !service.room(io::MAX_OUT_FIRE) {
                    break;
                }
                io::fire(&mut service.io, &service.io_env, &mut service.told, &mut service.raw_submissions);
            }
        }
    }
    route_events(service);
    engine::iterate(&mut service.engine, now, wall);
    if service.end().is_some()
        && !service.signals_closing
        && service.raw_submissions.room() >= io::MAX_OUT_DOWN.submissions
        && let Some(entity) = service.signals
    {
        service.signals_closing = true;
        io::down(&mut service.io, &service.io_env, io::Request::Close { entity }, &mut service.raw_submissions);
    }
    service.engine.abort_if_ended();
    service.route_components();
    service.kernel.publish(false, &mut service.raw_submissions, &mut service.submissions);
    if let Some(accounts) = &mut service.engine.accounts {
        service.kernel.publish(true, &mut accounts.submitted, &mut service.submissions);
    }
    service.io.reclaim();
}
fn route_events(service: &mut Service) {
    for _ in 0..service.told.len() {
        if let Some(accounts) = &service.engine.accounts
            && !accounts.up_room()
        {
            break;
        }
        let event = service.told.pop().expect("io event count");
        if let io::Event::Shutdown { .. } = event {
            service.shutdown();
            continue;
        }
        let owner = match &event {
            io::Event::Listening { owner, .. }
            | io::Event::Accepted { owner, .. }
            | io::Event::Connecting { owner, .. }
            | io::Event::Connected { owner }
            | io::Event::Stream { owner, .. }
            | io::Event::Output { owner, .. }
            | io::Event::Spawned { owner, .. }
            | io::Event::Exited { owner, .. }
            | io::Event::Usage { owner, .. }
            | io::Event::Window { owner, .. }
            | io::Event::Failed { owner, .. }
            | io::Event::Closed { owner } => Some(*owner),
            io::Event::Resized | io::Event::Shutdown { .. } => None,
        };
        if owner == Some(Token::new(1)) {
            if let io::Event::Closed { .. } = event {
                service.signals = None;
            }
            continue;
        }
        if let Some(owner) = owner
            && accounts::owns(owner)
        {
            if let Some(accounts) = &mut service.engine.accounts {
                accounts.up(accounts::untag(event));
            }
        } else {
            service.engine.end = Some(End::Stopped(Stop::Listener));
        }
    }
}

// A conservative payload envelope uses the root's admitted footprint per held
// event/page: each payload is a bounded part of that root (programming-model.md, 6).
fn memory(
    settings: &settings::Engine,
    root: &temper_engine_domain::Limits,
    store: &jig_protocol_store::Limits,
    runtime: Option<&AccountsRuntime>,
    queue: u32,
    operations: u32,
) -> Option<u64> {
    let payload = temper_engine_domain::worst_case(root)?;
    let events = queue.checked_mul(skein_oauth_accounts::MAX_OUT_DOWN.above.max(1))?;
    let mut bytes = payload
        .checked_add(u64::try_from(size_of::<Service>()).ok()?)?
        .checked_add(skein_io::worst_case(&settings.jig.limits.io)?)?
        .checked_add(store::worst_case(root, store)?)?
        .checked_add(jig_protocol_accounts::table::worst_case(
            root.accounts.accounts.checked_add(1)?,
            settings.jig.limits.driver.client.document.token_bytes,
        )?)?
        .checked_add(List::<(u32, u64)>::worst_case(root.authority.projects)?)?
        .checked_add(Queue::<temper_engine_domain::Event>::worst_case(events)?)?
        .checked_add(Queue::<temper_engine_domain::Request>::worst_case(temper_engine_domain::max_out(root))?)?
        .checked_add(Queue::<Box<temper_engine_domain_forge::Event>>::worst_case(events)?)?
        .checked_add(payload.checked_mul(u64::from(events).checked_mul(2)?.checked_add(1)?)?)?
        .checked_add(Queue::<Complete>::worst_case(queue)?)?
        .checked_add(Queue::<Submit>::worst_case(queue)?.checked_mul(2)?)?
        .checked_add(Queue::<io::Event>::worst_case(queue)?)?
        .checked_add(Queue::<Box<[u8]>>::worst_case(queue)?)?
        .checked_add(u64::from(queue).checked_mul(u64::from(settings.jig.limits.io.output).checked_add(512)?)?)?
        .checked_add(Map::<Token, (bool, Token, Option<Token>)>::worst_case(operations)?)?
        .checked_add(Map::<(bool, Token), Token>::worst_case(operations)?)?;
    if !settings.jig.llm.accounts.is_empty() {
        let mut driver = settings.jig.limits.driver;
        driver.accounts = root.accounts.accounts.checked_add(1)?;
        bytes = bytes.checked_add(accounts::worst_case(&driver, queue.checked_mul(3)?)?)?;
    }
    if let Some(runtime) = runtime {
        bytes = bytes.checked_add(skein_tls::roots_worst_case(runtime.trust_roots, runtime.trust_root_bytes)?)?;
    }
    Some(bytes)
}
fn pending_files(accounts: Option<&accounts::Accounts>) -> Option<&Queue<Submit>> {
    let accounts = accounts?;
    Some(&accounts.submitted)
}
