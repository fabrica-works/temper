//! One process, joined through io's workers channel (protocol/README.md, 8).
use crate::{
    AccountsRuntime, End, Stop, Worker, accounts, clients, engine, limits, settings, stop, store, translate, values,
    workers,
};
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
    pub(crate) engine: engine::Engine,
    pub(crate) worker: Option<Worker>,
    io: Io,
    io_env: Env<io::Limits>,
    pub(crate) completions: Queue<Complete>,
    pub(crate) submissions: Queue<Submit>,
    raw_submissions: Queue<Submit>,
    kernel: accounts::Kernel,
    pub(crate) told: Queue<io::Event>,
    signals: Option<Token>,
    signals_closing: bool,
    bytes: u64,
    links: Queue<Box<[u8]>>,
}
impl Service {
    /// Construct the root, its deployment store and its configured accounts.
    pub fn engine(
        settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: List<crate::ForgeRuntime>,
    ) -> Result<Self, Problems> {
        let io = settings.jig.limits.io;
        Self::new(settings, seeds, accounts, forges, None, 0, io)
    }
    /// Join the configured engine and worker in one io, with their checked memory envelopes.
    pub(crate) fn new(
        settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: List<crate::ForgeRuntime>,
        worker: Option<Worker>,
        worker_bytes: u64,
        io_limits: io::Limits,
    ) -> Result<Self, Problems> {
        let (forges, roots) = crate::forge::prepare(&settings, forges)?;
        let owned = skein_forge_connection::settings::owned_bytes(&forges)
            .ok_or(start_problem(b"memory_bytes", What::Bound))?;
        Self::prepared(settings, seeds, accounts, (forges, roots, owned), worker, worker_bytes, io_limits)
    }
    pub(crate) fn prepared(
        mut settings: settings::Engine,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        forges: (skein_forge_connection::Settings, u64, u64),
        worker: Option<Worker>,
        worker_bytes: u64,
        io_limits: io::Limits,
    ) -> Result<Self, Problems> {
        let limits =
            limits::engine(&settings.jig.limits, &settings.limits).ok_or(start_problem(b"limits", What::Bound))?;
        let (forge_settings, forge_roots, forge_owned) = forges;
        let shared = crate::forge::shared(&settings.limits, &settings.jig.limits.io);
        read_charters(&mut settings)?;
        let store_limits = store::limits(&limits, &settings.jig.limits.core, &limits.forge, &settings.jig.limits.store)
            .ok_or(start_problem(b"limits", What::Bound))?;
        let max = temper_engine_domain::max_out(&limits);
        let queue = max.checked_mul(2).ok_or(start_problem(b"limits", What::Bound))?;
        let worker_operations = match &worker {
            Some(worker) => worker.operations(),
            None => 0,
        };
        let operations =
            startup_operations(&io_limits, queue, worker_operations).ok_or(start_problem(b"limits", What::Bound))?;
        let bytes = startup_worst_case(
            &settings,
            &limits,
            &store_limits,
            accounts.as_ref(),
            worker_operations,
            &io_limits,
            (forge_roots, forge_owned),
        )?;

        if let Err(what) =
            jig_protocol_config::fits(settings.jig.memory, &[jig_protocol_config::Share { name: b"service", bytes }])
        {
            return Err(start_problem(b"memory_bytes", what));
        }
        let room = max.checked_mul(6).expect("account routing room");
        let accounts = accounts_driver(&settings, seeds, accounts, room, &store_limits.secrets)?;
        let mut values =
            values::Values::new(settings.jig.limits.driver.accounts, values::credential_bytes(&settings.jig));
        let wire = translate::limits(&settings.limits).ok_or(start_problem(b"limits", What::Bound))?;
        values.fill(&settings.identities, &wire)?;
        let (forge_config, identities) = crate::forge::config(&settings, seeds.deployment);
        let mut first_periods = List::with_capacity(limits.authority.projects);
        for project in &settings.jig.projects {
            first_periods
                .push((project.number, project.policy.period_spend))
                .or(Err(start_problem(b"projects", What::Bound)))?;
        }
        let client_room = max
            .checked_add(1)
            .expect("client pass headroom")
            .checked_mul(jig_protocol_engine_clients::MAX_OUT_DOWN)
            .expect("client routing room");
        let clients =
            clients::Clients::new(&settings.jig, &settings.clients.listener, seeds, client_room, &limits.journal)?;
        let forge = crate::forge::Layer::new(
            forge_settings,
            settings.limits.forge_component,
            &shared,
            max.checked_add(2).expect("forge routing headroom"),
        )
        .expect("checked forge limits");
        for configured in &mut settings.forges {
            configured.api_token = skein_lib::Secret::new(Box::new([]));
        }
        let workers = workers::Workers::configured(&mut settings, seeds.deployment, max)?;
        let config = temper_engine_domain::Config {
            core: jig_protocol_config::core(&settings.jig, seeds.deployment, seeds.seed),
            landing: settings.landing,
            forge: forge_config,
            identities,
            forge_charter: settings.charter,
        };
        let store = store::Store::new(&limits, &store_limits, &config.core.settings.models, seeds.installation);
        Ok(Self {
            worker,
            engine: engine::Engine::new(
                config,
                &limits,
                store,
                engine::Children { accounts, workers, clients, forge },
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
            bytes: bytes.checked_add(worker_bytes).ok_or(start_problem(b"memory_bytes", What::Bound))?,
            links: Queue::with_capacity(queue),
        })
    }
    /// Adopt the binary's deployment directory.
    pub fn adopt_store(&mut self, root: Fd, effective_user: u32) -> Result<(), Fd> {
        self.engine.store.adopt(root, effective_user)
    }
    /// Hold the bundle the binary read at start, served on the clients' listener.
    pub fn adopt_bundle(&mut self, bundle: clients::bundle::Bundle) {
        self.engine.clients.adopt_bundle(bundle);
    }
    /// Earliest deadline of the root and its children.
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        accounts::earliest(
            self.engine.next_deadline(),
            match &self.worker {
                Some(worker) => worker.next_deadline(),
                None => None,
            },
        )
    }
    /// Whether an iteration has work on the supplied clock.
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.engine.work_pending(now)
            || (self.engine.end.is_some()
                || self.engine.store.opened && self.engine.workers.opened() && self.engine.clients.opened())
                && match &self.worker {
                    Some(worker) => worker.work_pending(now),
                    None => false,
                }
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
        if self.engine.end.is_some()
            && let Some(worker) = &mut self.worker
        {
            worker.abort();
        }
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
            .expect("engine operations")
            .checked_add(match &self.worker {
                Some(worker) => worker.operations(),
                None => 0,
            })
            .expect("service operations")
    }
    /// Every owned child has closed and every queue and kernel route is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.end().is_some()
            && match &self.worker {
                Some(worker) => worker.is_empty(),
                None => true,
            }
            && self.engine.forge.closed
            && self.engine.forge.io.is_empty()
            && self.engine.forge.above.is_empty()
            && self.engine.store.closed()
            && self.engine.workers.closed()
            && self.engine.clients.closed
            && self.engine.clients.io.is_empty()
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
        if self.engine.store.opened || self.engine.end.is_some() {
            for _ in 0..self.engine.forge.io.len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                    break;
                }
                let request = self.engine.forge.io.pop().expect("forge io requests");
                io::down(&mut self.io, &self.io_env, crate::forge::tag(request), &mut self.raw_submissions);
            }
            for _ in 0..self.engine.clients.io.len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                    break;
                }
                let request = self.engine.clients.io.pop().expect("client io requests");
                io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
            }
            for _ in 0..self.engine.workers.io.len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                    break;
                }
                let request = self.engine.workers.io.pop().expect("worker io requests");
                io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
            }
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
        if let Some(worker) = &mut self.worker {
            for _ in 0..worker.submitted().len() {
                if self.raw_submissions.room() == 0 {
                    break;
                }
                let submit = worker.submitted().pop().expect("worker kernel submissions");
                self.raw_submissions.push(submit);
            }
            for _ in 0..worker.io().len() {
                if self.raw_submissions.room() < io::MAX_OUT_DOWN.submissions || !self.io.takes() {
                    break;
                }
                let request = worker.io().pop().expect("worker io requests");
                io::down(&mut self.io, &self.io_env, request, &mut self.raw_submissions);
            }
        }
    }
    /// Format only the roots for a world's secrets check.
    pub fn fmt_roots(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut roots = formatter.debug_list();
        roots.entry(&self.engine.domain);
        if let Some(worker) = &self.worker {
            roots.entry(&worker.root);
        }
        roots.finish()
    }
    fn complete(&mut self, complete: Complete) {
        if store::owns(complete.op) {
            self.engine.store.complete(complete);
        } else if crate::tokens::prefix(complete.op) == 3 || crate::tokens::prefix(complete.op) == 6 {
            self.worker
                .as_mut()
                .expect("worker operation prefix")
                .complete(self.io_env.now, complete)
                .expect("worker operation owner");
        } else {
            io::up(&mut self.io, &self.io_env, complete, &mut self.told, &mut self.raw_submissions);
        }
    }
}
fn start_problem(member: &[u8], what: What) -> Problems {
    let mut problems = Problems::empty();
    problems.push(Path::root().member(b"engine").member(member), what);
    problems
}
/// The accounts' driver, when the engine configures an account; its records
/// are kept in the secrets' store, at most `secret_bytes` each.
fn accounts_driver(
    settings: &settings::Engine,
    seeds: &settings::Seeds,
    runtime: Option<AccountsRuntime>,
    room: u32,
    secrets: &skein_kv::Limits,
) -> Result<Option<accounts::Accounts>, Problems> {
    if settings.jig.llm.accounts.is_empty() && settings.jig.llm.providers.is_empty() {
        return Ok(None);
    }
    Ok(Some(accounts::Accounts::new(&settings.jig, &settings.files, seeds, runtime, room, secrets)?))
}
fn read_charters(settings: &mut settings::Engine) -> Result<(), Problems> {
    for charter in &mut settings.jig.charters {
        charter.policy.instructions = settings
            .files
            .contents(&charter.policy.instructions)
            .ok_or(start_problem(b"charters", What::Unreadable))?
            .into();
    }
    Ok(())
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
            let complete = service.kernel.complete(complete, &service.raw_submissions);
            service.complete(complete);
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
    if let Some(worker) = &mut service.worker {
        if service.engine.end.is_some()
            || match &service.engine.drain {
                Some(drain) => drain.closing,
                None => false,
            }
        {
            worker.abort();
        }
        if (service.engine.end.is_some()
            || service.engine.store.opened && service.engine.workers.opened() && service.engine.clients.opened())
            && worker.room()
        {
            worker.iterate(now, wall);
        }
        if let Some(problems) = worker.refused() {
            if service.engine.drain.is_none() && service.engine.end.is_none() {
                stop::shutdown(&mut service.engine);
            }
            if let Some(End::Drained) = service.engine.end {
                service.engine.end = Some(End::Stopped(Stop::Start(problems.clone())));
            }
        }
    }
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
    for _ in 0..service.engine.clients.links.len() {
        if service.links.room() == 0 {
            break;
        }
        let link = service.engine.clients.links.pop().expect("printed sign-in links");
        service.links.push(link);
    }
    service.kernel.publish(&mut service.raw_submissions, &mut service.submissions);
    service.io.reclaim();
}
fn route_events(service: &mut Service) {
    for _ in 0..service.told.len() {
        if !service.engine.workers.room()
            || !service.engine.clients.room(1)
            || service.engine.events.room() < engine::route_out()
        {
            break;
        }
        if !service.engine.forge.room(1) {
            break;
        }
        if let Some(worker) = &service.worker
            && !worker.room()
        {
            break;
        }
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
        if let Some(worker) = &mut service.worker
            && worker.accepts(&event)
        {
            worker.up(service.io_env.now, event);
        } else if let Some(owner) = owner
            && crate::tokens::prefix(owner) == 5
        {
            service.engine.forge.up(event);
        } else if let Some(owner) = owner
            && clients::owns(owner)
        {
            if !service.engine.clients.up(event) {
                service.engine.end = Some(End::Stopped(Stop::Listener));
            }
        } else if let Some(owner) = owner
            && accounts::owns(owner)
        {
            if let Some(accounts) = &mut service.engine.accounts {
                accounts.up(accounts::untag(event));
            }
        } else if !service.engine.workers.up(service.io_env.now, event, &mut service.engine.events) {
            service.engine.end = Some(End::Stopped(Stop::Listener));
        }
    }
}

// A conservative payload envelope uses the root's admitted footprint per held
// event/page: each payload is a bounded part of that root (programming-model.md, 6).
pub(crate) fn startup_worst_case(
    settings: &settings::Engine,
    root: &temper_engine_domain::Limits,
    store: &jig_protocol_store::Limits,
    runtime: Option<&AccountsRuntime>,
    worker_operations: u32,
    io: &io::Limits,
    forge_bytes: (u64, u64),
) -> Result<u64, Problems> {
    let queue = temper_engine_domain::max_out(root).checked_mul(2).ok_or(start_problem(b"limits", What::Bound))?;
    let operations = startup_operations(io, queue, worker_operations).ok_or(start_problem(b"limits", What::Bound))?;
    let bytes = memory(settings, root, store, runtime, operations, io, forge_bytes.1)
        .ok_or(start_problem(b"memory_bytes", What::Bound))?;
    bytes.checked_add(forge_bytes.0).ok_or(start_problem(b"memory_bytes", What::Bound))
}
fn startup_operations(io: &io::Limits, queue: u32, worker_operations: u32) -> Option<u32> {
    io::operations(io)?.checked_add(queue.checked_mul(4)?)?.checked_add(worker_operations)
}
fn memory(
    settings: &settings::Engine,
    root: &temper_engine_domain::Limits,
    store: &jig_protocol_store::Limits,
    runtime: Option<&AccountsRuntime>,
    operations: u32,
    io: &io::Limits,
    forge_owned: u64,
) -> Option<u64> {
    let queue = temper_engine_domain::max_out(root).checked_mul(2)?;
    let payload = temper_engine_domain::worst_case(root)?;
    let events = queue.checked_mul(engine::route_out())?;
    let mut bytes = payload
        .checked_add(u64::try_from(size_of::<Service>()).ok()?)?
        .checked_add(skein_io::worst_case(io)?)?
        .checked_add(store::worst_case(root, store)?)?
        .checked_add(jig_protocol_accounts::table::worst_case(
            settings.jig.limits.driver.accounts,
            values::credential_bytes(&settings.jig),
        )?)?
        .checked_add(List::<(u32, u64)>::worst_case(root.authority.projects)?)?
        .checked_add(Queue::<temper_engine_domain::Event>::worst_case(events)?)?
        .checked_add(Queue::<temper_engine_domain::Request>::worst_case(temper_engine_domain::max_out(root))?)?
        .checked_add(payload.checked_mul(u64::from(events).checked_mul(2)?.checked_add(1)?)?)?
        .checked_add(Queue::<Complete>::worst_case(queue)?)?
        .checked_add(Queue::<Submit>::worst_case(queue)?.checked_mul(2)?)?
        .checked_add(Queue::<io::Event>::worst_case(queue)?)?
        .checked_add(Queue::<Box<[u8]>>::worst_case(queue)?)?
        .checked_add(u64::from(queue).checked_mul(u64::from(io.output).checked_add(512)?)?)?
        .checked_add(Map::<Token, (Token, Option<Token>)>::worst_case(operations)?)?
        .checked_add(Map::<Token, Token>::worst_case(operations)?)?;
    bytes = bytes.checked_add(workers::worst_case(
        settings,
        temper_engine_domain::max_out(root).checked_mul(jig_protocol_engine_workers::MAX_OUT_DOWN)?,
    )?)?;
    bytes = bytes.checked_add(clients::worst_case(
        &settings.jig.limits.clients,
        &limits::server(&settings.jig.limits),
        &settings.jig.limits.clients_http,
        temper_engine_domain::max_out(root).checked_add(1)?.checked_mul(jig_protocol_engine_clients::MAX_OUT_DOWN)?,
        &root.journal,
    )?)?;
    bytes = bytes.checked_add(clients::bundle::worst_case(&settings.limits.clients)?)?;
    let forges = u32::try_from(settings.forges.len()).ok()?;
    let batch = temper_engine_domain::max_out(root).checked_add(2)?;
    bytes = bytes
        .checked_add(skein_forge_connection::limits::worst_case(
            &settings.limits.forge_component,
            &crate::forge::shared(&settings.limits, &settings.jig.limits.io),
            forges,
            forge_owned,
        )?)?
        .checked_add(Queue::<skein_forge_connection::Event>::worst_case(
            batch.checked_mul(skein_forge_connection::MAX_OUT.above)?,
        )?)?
        .checked_add(
            u64::from(batch.checked_mul(skein_forge_connection::MAX_OUT.above)?)
                .checked_mul(u64::from(settings.limits.forge.client.answer_bytes))?,
        )?
        .checked_add(Queue::<io::Request>::worst_case(batch.checked_mul(skein_forge_connection::MAX_OUT.below)?)?)?;
    let identities = u32::try_from(settings.identities.len()).ok()?;
    let wire = translate::limits(&settings.limits)?;
    bytes = bytes
        .checked_add(Map::<u32, skein_lib::Secret>::worst_case(identities)?)?
        .checked_add(List::<u32>::worst_case(identities)?)?
        .checked_add(
            u64::from(identities).checked_mul(temper_protocol_worker_wire::v1::Identity::worst_case_bytes(&wire)?)?,
        )?;
    if !settings.jig.llm.accounts.is_empty() || !settings.jig.llm.providers.is_empty() {
        let driver = settings.jig.limits.driver;
        bytes = bytes.checked_add(accounts::worst_case(&driver, queue.checked_mul(3)?, store.secrets.page.rows)?)?;
    }
    bytes = bytes.checked_add(accounts::configured_bytes(&settings.jig)?)?;
    if let Some(runtime) = runtime {
        bytes = bytes.checked_add(skein_tls::roots_worst_case(runtime.trust_roots, runtime.trust_root_bytes)?)?;
    }
    Some(bytes)
}
