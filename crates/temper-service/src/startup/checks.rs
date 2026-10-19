//! Checks across standalone's parts before any service is made.
use super::{Problems, What};
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use jig_protocol_config::{self as config, Path, Segment};
use skein_io::Addr;
use skein_lib::{List, Map};
use temper_engine_domain as domain;
use temper_protocol_worker_wire as payload;
use temper_worker_domain as worker_domain;

fn at(index: usize, path: Path) -> Path {
    path.child(Segment::Index(u32::try_from(index).expect("bounded configuration")))
}
fn own(name: &[u8]) -> Path {
    Path::root().member(b"application").member(b"limits").member(b"overrides").member(name)
}
fn against(problems: &mut Problems, name: &[u8], other: &'static [u8]) {
    problems.push(own(name), What::Against { other });
}
fn in_process(address: &config::Address) -> bool {
    match address {
        config::Address::Addr(Addr::InProcess(name)) => name.as_bytes() == b"workers",
        config::Address::Addr(Addr::Ip(_)) | config::Address::Host { .. } => false,
    }
}
/// Standalone's requirements on jig's parts, including absolute agent paths.
pub fn standalone(
    engine: &config::Engine,
    clients: &super::application::Clients,
    workers: &[config::Worker],
    problems: &mut Problems,
) {
    let root = Path::root();
    if workers.len() != 1 {
        problems.push(root.member(b"workers"), What::Standalone);
    }
    if engine.hosts.listener.len() != 1 || !in_process(engine.hosts.listener.first().expect("parsed listeners")) {
        problems.push(root.member(b"engine").member(b"hosts").member(b"listener"), What::Standalone);
    }
    let hosts_match = match (engine.hosts.workers.first(), workers.first()) {
        (Some(host), Some(worker)) => host.name == worker.name,
        (None, _) | (_, None) => false,
    };
    if engine.hosts.workers.len() != 1 || !hosts_match {
        problems.push(root.member(b"engine").member(b"hosts").member(b"workers"), What::Standalone);
    }
    match clients.listener {
        config::Address::Addr(Addr::Ip(socket)) if socket.ip().is_loopback() => {}
        config::Address::Addr(Addr::Ip(_) | Addr::InProcess(_)) | config::Address::Host { .. } => {
            problems.push(root.member(b"application").member(b"clients").member(b"listener"), What::Loopback);
        }
    }
    if engine.connectors.as_ref() != [1] {
        problems.push(root.member(b"engine").member(b"connectors"), What::Connectors);
    }
    for (index, worker) in workers.iter().enumerate() {
        let path = at(index, root.member(b"workers"));
        if !in_process(&worker.engine) {
            problems.push(path.member(b"engine"), What::Standalone);
        }
        for (name, value) in
            [(b"program".as_slice(), &worker.agent.program), (b"configuration", &worker.agent.configuration)]
        {
            if !value.starts_with(b"/") {
                problems.push(path.member(b"agent").member(name), What::Absolute);
            }
        }
    }
}
/// The root has no hub child: the engine runs no slot, and every charter's
/// runs are placed on the workers (domain/root.md, 2).
pub fn hub(engine: &config::Engine, problems: &mut Problems) {
    let path = Path::root().member(b"engine");
    if engine.limits.core.fleet.engine_slots != 0 {
        problems.push(path.member(b"limits").member(b"overrides").member(b"fleet.engine_slots"), What::NoHub);
    }
    for (index, charter) in engine.charters.iter().enumerate() {
        match charter.hosts {
            fleet::Kinds::Workers => {}
            fleet::Kinds::Engine | fleet::Kinds::Both => {
                problems.push(at(index, path.member(b"charters")).member(b"hosts"), What::NoHub);
            }
        }
    }
}
/// The deployment's provider is not 0, which the root keeps for the forge's
/// sign-in (domain/root.md, 6).
pub fn provider(engine: &config::Engine, problems: &mut Problems) {
    for (index, provider) in engine.providers.iter().enumerate() {
        match provider.kind {
            config::ProviderKind::Deployment if provider.number == 0 => {
                problems.push(
                    at(index, Path::root().member(b"engine").member(b"providers")).member(b"number"),
                    What::Provider,
                );
            }
            config::ProviderKind::Deployment | config::ProviderKind::Link | config::ProviderKind::Application => {}
        }
    }
}
/// Each git identity's number is one both the engine's and the worker's
/// accounts tables hold.
pub fn credentials(
    engine: &config::Engine,
    workers: &[config::Worker],
    identities: &[super::Identity],
    problems: &mut Problems,
) {
    let mut bound = engine.limits.core.accounts.credentials;
    for worker in workers {
        bound = bound.min(worker.limits.core.accounts.credentials);
    }
    for (index, identity) in identities.iter().enumerate() {
        if identity.number == 0 || identity.number > bound {
            let path = at(index, Path::root().member(b"application").member(b"identities"));
            problems.push(path.member(b"number"), What::Credentials);
        }
    }
}
/// Check the optional listener and the secret paths that require it.
pub fn webhooks(members: &super::application::Members, problems: &mut Problems) {
    let path = Path::root().member(b"application");
    if let Some(webhooks) = &members.webhooks {
        match &webhooks.listener {
            config::Address::Addr(Addr::Ip(socket)) if socket.ip().is_loopback() => {
                if socket.port() != 0 && webhooks.listener == members.clients.listener {
                    problems.push(path.member(b"webhooks").member(b"listener"), What::Jig(config::What::Unique));
                }
            }
            config::Address::Addr(_) | config::Address::Host { .. } => {
                problems.push(path.member(b"webhooks").member(b"listener"), What::Loopback);
            }
        }
    } else {
        for (index, forge) in members.forges.iter().enumerate() {
            if forge.webhook_secret.is_some() {
                problems.push(at(index, path.member(b"forges")).member(b"webhook_secret"), What::NoWebhooks);
            }
        }
    }
}
/// Every git rule is checked independently, so one start reports them all.
pub fn limits(limits: &limits::Limits, git: &temper_worker_protocol_git::Limits, problems: &mut Problems) {
    use temper_worker_protocol_git::{Unfit, check};
    let mut checked = *git;
    // Repair just the reported rule in the local copy, then ask for the next.
    for _ in 0_u8..7 {
        match check(&checked, &limits.checkout) {
            Ok(()) => break,
            Err(Unfit::Operations) => {
                against(problems, b"git_component.operations", b"checkout.workspaces");
                checked.operations = limits.checkout.workspaces;
            }
            Err(Unfit::StdoutBytes) => {
                against(
                    problems,
                    b"git_component.stdout_bytes",
                    b"checkout.name_bytes and checkout.conflicts * (checkout.path_bytes + 1)",
                );
                let Some(bytes) = temper_worker_protocol_git::limits::largest_output(&limits.checkout) else { break };
                checked.stdout_bytes = bytes;
            }
            Err(Unfit::StderrBytes) => {
                against(problems, b"git_component.stderr_bytes", b"checkout.push_diagnostic");
                checked.stderr_bytes = u32::try_from(temper_worker_domain_checkout::git::PushDiagnostic::CAPACITY)
                    .expect("diagnostic bound");
            }
            Err(Unfit::ReadBytes) => {
                against(problems, b"git_component.read_bytes", b"a nonzero read");
                checked.read_bytes = 1;
            }
            Err(Unfit::SweepEntries) => {
                against(problems, b"git_component.sweep_entries", b"a nonempty sweep page");
                checked.sweep_entries = 1;
            }
            Err(Unfit::SweepBytes) => {
                against(problems, b"git_component.sweep_bytes", b"the kernel's longest name");
                checked.sweep_bytes = u32::try_from(skein_io::kernel::LONGEST_NAME).expect("kernel name bound");
            }
            Err(Unfit::Memory) => {
                problems.push(own(b"git_component.operations"), What::Jig(config::What::Bound));
                break;
            }
        }
    }
}
/// The longest routed remote: base, two separators, owner, name and `.git`.
pub fn longest_remote(forges: &[settings::Forge], name_bytes: u32) -> Option<u64> {
    let mut base = 0;
    for forge in forges {
        base = base.max(u64::try_from(forge.git.len()).ok()?);
    }
    remote(base, name_bytes)
}
pub(super) fn remote(base: u64, name_bytes: u32) -> Option<u64> {
    base.checked_add(u64::from(name_bytes).checked_mul(2)?)?.checked_add(6)
}
pub(super) fn names(remote: Option<u64>, limits: &limits::Limits, problems: &mut Problems) {
    if remote.unwrap_or(u64::MAX) > u64::from(limits.checkout.name_bytes) {
        against(problems, b"checkout.name_bytes", b"the longest remote");
    }
    if limits.forge.name_bytes > limits.checkout.name_bytes {
        against(problems, b"checkout.name_bytes", b"forge.name_bytes");
    }
}
fn bounded(bounds: &payload::Bounds, engine: bool, problems: &mut Problems) -> Option<payload::v1::Limits> {
    let ceilings = payload::v1::CEILINGS;
    let repositories: &[u8] = if engine { b"forge.resources_per_task" } else { b"checkout.repositories" };
    let names: &[u8] = if engine { b"forge.name_bytes" } else { b"checkout.name_bytes" };
    for (name, bytes, bound) in [
        (
            repositories,
            bounds.repositories,
            ceilings.items_repositories.min(ceilings.left_landed).min(ceilings.saved_repositories),
        ),
        (
            names,
            bounds.name_bytes,
            ceilings
                .repository_name
                .min(ceilings.merge_branch)
                .min(ceilings.branch_branch)
                .min(ceilings.writable_push)
                .min(ceilings.items_save_some),
        ),
        (b"checkout.name_bytes", bounds.remote_bytes, ceilings.repository_remote),
        (b"checkout.conflicts", bounds.conflicts, ceilings.files_files),
        (b"checkout.path_bytes", bounds.path_bytes, ceilings.files_files_item),
        (b"identity.token_bytes", bounds.token_bytes, ceilings.identity_token),
        (
            b"identity.text_bytes",
            bounds.text_bytes,
            ceilings.identity_name.min(ceilings.identity_login).min(ceilings.identity_email),
        ),
    ] {
        if bytes > bound {
            problems.push(own(name), What::Past { bytes: u64::from(bytes), bound: u64::from(bound) });
        }
    }
    payload::limits(bounds)
}
/// Each side's application records fit jig's fields and the worker's admission.
pub fn channel(
    engine: &payload::Bounds,
    worker: &payload::Bounds,
    forges: &[settings::Forge],
    limits: &limits::Limits,
    problems: &mut Problems,
) {
    names(longest_remote(forges, limits.forge.name_bytes), limits, problems);
    // The engine's terms take the smaller of the two repository bounds
    // (translate::limits), so a checkout's larger one makes the worker's
    // parts past those terms and its hello never opens the link.
    let repository_shape = engine.repositories == worker.repositories;
    let engine = bounded(engine, true, problems);
    let worker = bounded(worker, false, problems);
    let ceilings = jig_protocol_worker_wire::CEILINGS;
    let mut sizes = [None, None];
    for (index, side) in [engine, worker].iter().enumerate() {
        if let Some(side) = side {
            *sizes.get_mut(index).expect("two sides") = payload::largest(side);
            if let Some(largest) = *sizes.get(index).expect("two sides") {
                for (name, bytes, bound) in [
                    (b"workspace_items".as_slice(), largest.items, ceilings.workspace_items),
                    (b"work_left_some", largest.left, ceilings.work_left_some),
                    (b"work_saved_some", largest.saved, ceilings.work_saved_some),
                    (b"resource_resource_some", largest.resource, ceilings.resource_resource_some),
                    (b"grant_value", largest.grant, ceilings.grant_value),
                ] {
                    if bytes > bound {
                        let prefix: &[u8] = if index == 0 { b"engine" } else { b"workers" };
                        problems.push(
                            Path::root().member(prefix).member(b"limits").member(name),
                            What::Past { bytes: u64::from(bytes), bound: u64::from(bound) },
                        );
                    }
                }
            }
        }
    }
    if let [Some(engine), Some(worker)] = sizes
        && (!repository_shape || engine.items > worker.items)
    {
        against(problems, b"forge.resources_per_task", b"checkout.repositories and checkout.name_bytes");
    }
}
/// The identity's complete encoded value must fit one grant.
pub fn identities(identities: &[settings::Identity], limits: &payload::v1::Limits, problems: &mut Problems) {
    for (index, identity) in identities.iter().enumerate() {
        let path = at(index, Path::root().member(b"application").member(b"identities"));
        match crate::translate::identity(identity, limits) {
            Ok(value) => {
                let bytes = u64::try_from(value.expose().len()).expect("bounded identity");
                let bound = u64::from(jig_protocol_worker_wire::CEILINGS.grant_value);
                if bytes > bound {
                    problems.push(path, What::Past { bytes, bound });
                }
            }
            Err(_) => problems.push(path, What::Jig(config::What::Bound)),
        }
    }
}
fn worker_limit(member: &[u8]) -> Path {
    at(0, Path::root().member(b"workers")).member(b"limits").member(b"overrides").member(member)
}
/// The worker's rules against the agent's configuration, each naming both
/// limits (temper-worker-domain's `worst_case`).
pub fn worker(limits: &worker_domain::Limits, problems: &mut Problems) {
    use worker_domain::Unfit;
    let worker = at(0, Path::root().member(b"workers"));
    for unfit in &worker_domain::unfit(limits) {
        let (path, other): (Path, &'static [u8]) = match unfit {
            Unfit::Agent | Unfit::Unacknowledged => {
                problems.push(worker.member(b"agent").member(b"configuration"), What::Jig(config::What::Bound));
                continue;
            }
            Unfit::Directories => (own(b"checkout.repositories"), b"the agent's directories"),
            Unfit::Conflicts => (own(b"checkout.conflicts"), b"the agent's conflicts"),
            Unfit::Paths => (own(b"checkout.path_bytes"), b"the agent's path_bytes"),
            Unfit::Names => (own(b"checkout.name_bytes"), b"the agent's name_bytes"),
            Unfit::Key => (own(b"checkout.name_bytes"), b"a workstream's 8-byte key"),
            Unfit::Accounts => (worker_limit(b"hub.accounts"), b"the agent's accounts"),
            Unfit::Workspaces => (worker.member(b"slots"), b"checkout.workspaces"),
            Unfit::Agents => (worker.member(b"slots"), b"the agent's agents"),
            Unfit::Turns => (worker_limit(b"hub.turns"), b"the agent's turns"),
            Unfit::TurnQueue => (worker_limit(b"hub.turn_queue_bytes"), b"the agent's turns * hub.turn_bytes"),
            Unfit::Calls => (worker_limit(b"hub.run_calls"), b"the agent's calls"),
            Unfit::Events => {
                (worker_limit(b"hub.event_bytes"), b"the agent's message_bytes less smith's 2-byte separator")
            }
            Unfit::Held => (worker_limit(b"hub.held"), b"the agent's messages"),
            Unfit::Inbox => (worker_limit(b"hub.inbox"), b"the agent's messages"),
            Unfit::CallBytes => (own(b"checkout.message_bytes"), b"the agent's call_bytes"),
            Unfit::Saved => (own(b"checkout.message_bytes"), b"the save's commit message"),
            Unfit::BoundsInbox => (worker_limit(b"hub.inbox"), b"the agent's hello's inbox"),
            Unfit::BoundsMessage => (worker_limit(b"hub.event_bytes"), b"the agent's hello's message_bytes"),
        };
        problems.push(path, What::Against { other });
    }
}
/// The landing rules' limits, one of which a refused rule breaks.
const LANDING: &[u8] =
    b"root.journal.transcript_bytes, authority.requirements, forge.judge_criteria and forge.change_policy.gates";
/// The index of `number` in a list of numbered entries, which a check found there.
fn index_of(numbers: &[u32], number: u32) -> usize {
    let mut index = 0;
    for (at, listed) in numbers.iter().enumerate() {
        if *listed == number {
            index = at;
        }
    }
    index
}
/// What the root's construction asserts, refused before it is made, each
/// naming the configuration that breaks it (domain/root.md, 6).
pub fn root(
    engine: &settings::Engine,
    seeds: &settings::Seeds,
    landing_projects: &[u32],
    limits: &domain::Limits,
    problems: &mut Problems,
) {
    let mut core = config::core(&engine.jig, seeds.deployment, seeds.seed);
    for charter in &mut core.settings.charters {
        if let Some(contents) = engine.files.contents(&charter.policy.instructions) {
            charter.policy.instructions = Box::from(contents);
        }
    }
    let mut projects = Map::with_capacity(engine.landing.projects.capacity());
    for (&project, rules) in &engine.landing.projects {
        projects.insert(project, rules.clone()).expect("the same capacity");
    }
    let landing = domain::LandingPolicy { deployment: engine.landing.deployment.clone(), projects };
    let (forge, identities) = crate::forge::config(engine, seeds.deployment);
    let config = domain::Config { core, landing, forge, identities, forge_charter: engine.charter };
    let mut numbers = List::with_capacity(engine.jig.limits.core.authority.projects);
    for project in &engine.jig.projects {
        numbers.push(project.number).expect("bounded projects");
    }
    let numbers = numbers.into_boxed();
    let engine_path = Path::root().member(b"engine");
    let landing_path = Path::root().member(b"application").member(b"landing");
    for refused in domain::refused(config, limits) {
        let (path, what) = match refused {
            domain::Refused::Run { charter, bytes } => (
                at(charter, engine_path.member(b"charters")),
                match bytes {
                    Some(bytes) => What::Within {
                        bytes,
                        limit: b"root.journal.run_bytes",
                        bound: u64::from(limits.journal.run_bytes),
                    },
                    None => What::Jig(config::What::Bound),
                },
            ),
            domain::Refused::Resume => (
                engine_path.member(b"limits").member(b"overrides").member(b"core.resume_bytes"),
                What::Against { other: b"root.journal.transcript_bytes, above 0" },
            ),
            domain::Refused::JudgeProjects => {
                (own(b"forge.judge_projects"), What::Against { other: b"authority.projects in engine.limits" })
            }
            domain::Refused::JudgeCriteria => {
                (own(b"forge.judge_criteria"), What::Against { other: b"authority.requirements in engine.limits" })
            }
            domain::Refused::Escalation { project } => (
                at(index_of(&numbers, project), engine_path.member(b"projects"))
                    .member(b"policy")
                    .member(b"escalation_role"),
                What::Escalation,
            ),
            domain::Refused::Permissions { project, bytes } => (
                at(index_of(&numbers, project), engine_path.member(b"projects")).member(b"roles"),
                match bytes {
                    Some(bytes) => What::Within {
                        bytes,
                        limit: b"root.journal.transcript_bytes",
                        bound: u64::from(limits.journal.transcript_bytes),
                    },
                    None => What::Jig(config::What::Bound),
                },
            ),
            domain::Refused::Landing { project: None } => {
                (landing_path.member(b"deployment"), What::Against { other: LANDING })
            }
            domain::Refused::Landing { project: Some(project) } => (
                at(index_of(landing_projects, project), landing_path.member(b"projects")).member(b"rules"),
                What::Against { other: LANDING },
            ),
            domain::Refused::Forges => (
                Path::root().member(b"application").member(b"forges"),
                What::Against { other: b"forge.repositories and forge.client.op_bytes" },
            ),
        };
        problems.push(path, what);
    }
}
/// Endpoint names and accounts, and every charter model, agree with the agent.
pub fn agent(engine: &config::Engine, agent: &settings::Agent, problems: &mut Problems) {
    for (index, endpoint) in engine.llm.endpoints.iter().enumerate() {
        let agrees = match agent.endpoints.resolve(&endpoint.name) {
            Some(found) => found.account == endpoint.account,
            None => false,
        };
        if !agrees {
            problems.push(at(index, Path::root().member(b"engine").member(b"llm").member(b"endpoints")), What::Agent);
        }
    }
    for (index, charter) in engine.charters.iter().enumerate() {
        let path = at(index, Path::root().member(b"engine").member(b"charters"));
        model(&charter.policy.model, &agent.bounds, path.member(b"model"), problems);
        for (index, alternative) in charter.policy.alternatives.iter().enumerate() {
            model(alternative, &agent.bounds, at(index, path.member(b"alternatives")), problems);
        }
    }
}
fn model(model: &jig_domain_core::Model, bounds: &jig_domain_host::Bounds, path: Path, problems: &mut Problems) {
    let mut found = false;
    for bound in &bounds.models {
        found |= bound.name == model.name && bound.window >= model.window && bound.output >= model.output;
    }
    if !found {
        problems.push(path, What::Agent);
    }
}
/// One io holds engine links, the worker's link, and four entities per child.
pub(super) fn sockets(
    engine: &config::Engine,
    worker: &config::Worker,
    git: &temper_worker_protocol_git::Limits,
    members: &super::application::Members,
    problems: &mut Problems,
) {
    match (socket_count(engine, worker, git, members), process_io(&engine.limits.io, Some(&worker.limits.io))) {
        (Some(bytes), Some(io)) if bytes <= u64::from(io.sockets) => {}
        (Some(bytes), Some(io)) => problems.push(
            Path::root().member(b"engine").member(b"limits").member(b"io").member(b"sockets"),
            What::Past { bytes, bound: u64::from(io.sockets) },
        ),
        (None, _) | (_, None) => problems.push(
            Path::root().member(b"engine").member(b"limits").member(b"io").member(b"sockets"),
            What::Jig(config::What::Bound),
        ),
    }
}
fn socket_count(
    engine: &config::Engine,
    worker: &config::Worker,
    git: &temper_worker_protocol_git::Limits,
    members: &super::application::Members,
) -> Option<u64> {
    let mut required = u64::from(limits::server(&engine.limits).connections).checked_add(1)?.checked_add(2)?;
    if members.webhooks.is_some() {
        required = required.checked_add(1)?.checked_add(u64::from(members.limits.forge_component.hooks))?;
    }
    if !engine.llm.accounts.is_empty() {
        required = required.checked_add(u64::try_from(engine.llm.accounts.len()).ok()?)?;
        for account in &engine.llm.accounts {
            match &account.source {
                config::Source::SignIn(_) => {
                    required = required
                        .checked_add(u64::from(engine.limits.driver.exchanges))?
                        .checked_add(u64::from(engine.limits.driver.listeners))?;
                    break;
                }
                config::Source::HandedIn { .. } => {}
            }
        }
    }
    required
        .checked_add(1)?
        .checked_add(u64::from(worker.slots).checked_mul(4)?)?
        .checked_add(u64::from(git.operations).checked_mul(4)?)
}
/// Compose engine and worker io exactly as jig's reference startup does.
pub(crate) fn process_io(engine: &skein_io::Limits, worker: Option<&skein_io::Limits>) -> Option<skein_io::Limits> {
    let mut io = *engine;
    if let Some(worker) = worker {
        io.names = io.names.checked_add(worker.names)?;
        io.intake = io.intake.max(worker.intake);
        io.receive = io.receive.max(worker.receive);
        io.output = io.output.max(worker.output);
        io.sends = io.sends.max(worker.sends);
    }
    io.sockets = limits::SOCKETS;
    Some(io)
}
/// Each service's sum is checked against its own memory admission.
pub fn memory(engine: u64, engine_memory: u64, worker: u64, worker_memory: u64, problems: &mut Problems) {
    for (path, bytes, bound) in [
        (Path::root().member(b"engine"), engine, engine_memory),
        (at(0, Path::root().member(b"workers")), worker, worker_memory),
    ] {
        if let Err(what) = config::fits(bound, &[config::Share { name: b"service", bytes }]) {
            problems.push(path.member(b"memory_bytes"), What::Jig(what));
        }
    }
}
