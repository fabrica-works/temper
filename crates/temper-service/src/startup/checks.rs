//! Checks across standalone's parts before any service is made.
use super::{Problems, What};
use crate::{limits, settings};
use jig_protocol_config::{self as config, Path, Segment};
use skein_io::Addr;
use temper_protocol_worker_wire as payload;

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
pub fn standalone(engine: &config::Engine, workers: &[config::Worker], problems: &mut Problems) {
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
    match engine.clients.listener {
        config::Address::Addr(Addr::Ip(socket)) if socket.ip().is_loopback() => {}
        config::Address::Addr(Addr::Ip(_) | Addr::InProcess(_)) | config::Address::Host { .. } => {
            problems.push(root.member(b"engine").member(b"clients").member(b"listener"), What::Loopback);
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
    let repository_shape = engine.repositories <= worker.repositories;
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
    problems: &mut Problems,
) {
    match (socket_count(engine, worker, git), process_io(&engine.limits.io, Some(&worker.limits.io))) {
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
) -> Option<u64> {
    let mut required = u64::from(limits::server(&engine.limits).connections).checked_add(1)?.checked_add(2)?;
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
        io.sockets = io.sockets.checked_add(worker.sockets)?;
        io.names = io.names.checked_add(worker.names)?;
        io.intake = io.intake.max(worker.intake);
        io.receive = io.receive.max(worker.receive);
        io.output = io.output.max(worker.output);
        io.sends = io.sends.max(worker.sends);
    }
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
