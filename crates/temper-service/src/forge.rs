//! The forge component's boundary to the engine (protocol/README.md, 6).
use crate::{limits, settings};
use alloc::boxed::Box;
use jig_protocol_config::{Path, Problems, Segment, What};
use skein_lib::{Duration, Env, List, Queue, Time, Wall};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_protocol_forge as kit;
/// A forge's resolved endpoint and configured TLS transport, supplied by the shell.
#[derive(Debug)]
pub struct ForgeRuntime {
    pub number: u16,
    pub address: core::net::SocketAddr,
    pub transport: kit::settings::Transport,
    /// The prepared trust configuration, priced once for this forge.
    pub trust_roots: u32,
    pub trust_root_bytes: u32,
}
/// Pair the configuration and shell endpoints by forge number.
pub fn settings(
    forges: &[settings::Forge],
    runtime: List<ForgeRuntime>,
    webhooks: Option<settings::Webhooks>,
    found: &mut Problems,
) -> Option<kit::settings::Settings> {
    let mut endpoints = List::with_capacity(runtime.len());
    for endpoint in runtime.into_boxed() {
        endpoints.push(Some(endpoint)).expect("runtime capacity");
    }
    let mut configured = List::with_capacity(u32::try_from(forges.len()).expect("configured forges"));
    for (index, forge) in forges.iter().enumerate() {
        let mut endpoint = None;
        for at in 0..endpoints.len() {
            let candidate = endpoints.get_mut(at).expect("runtime entry");
            if let Some(value) = candidate.as_ref()
                && value.number == forge.number
            {
                endpoint = candidate.take();
                break;
            }
        }
        let path = forge_path(u32::try_from(index).expect("forge index")).member(b"api");
        let Some(endpoint) = endpoint else {
            found.push(path, What::Refers);
            continue;
        };
        let name = match &forge.api {
            jig_protocol_config::Address::Host { name, .. } => Some(name.as_ref()),
            jig_protocol_config::Address::Addr(skein_io::Addr::Ip(_)) => None,
            jig_protocol_config::Address::Addr(skein_io::Addr::InProcess(_)) => {
                found.push(path, What::Plaintext);
                continue;
            }
        };
        let Some(host) = skein_forge::host::host(endpoint.address, name) else {
            found.push(path, What::Bound);
            continue;
        };
        configured
            .push(kit::settings::Forge {
                number: forge.number,
                kind: kit::settings::Kind::Forgejo,
                address: skein_io::Addr::Ip(endpoint.address),
                transport: endpoint.transport,
                host,
                credential: skein_forge::credential::Credential::Token(forge.api_token.clone()),
                user: forge.user,
            })
            .expect("configured forge capacity");
    }
    for (index, endpoint) in endpoints.iter().enumerate() {
        if endpoint.is_some() {
            found.push(forge_path(u32::try_from(index).expect("runtime index")).member(b"api"), What::Refers);
        }
    }
    if found.is_empty() {
        Some(kit::settings::Settings { forges: configured, webhooks, word: Box::from(b"temper".as_slice()) })
    } else {
        None
    }
}
fn forge_path(index: u32) -> Path {
    Path::root().member(b"application").member(b"forges").child(Segment::Index(index))
}
/// Limits shared with the domain client and io.
#[must_use]
pub fn shared(limits: &limits::Limits, io: &skein_io::Limits) -> kit::Shared {
    kit::Shared {
        client: limits.forge.client,
        name_bytes: limits.forge.name_bytes,
        repositories: limits.forge.repositories,
        io: *io,
    }
}
/// Wrap the client's terminal without bringing protocol state into the root.
#[must_use]
pub fn event(answer: kit::Answered) -> root::Event {
    root::Event::Forge(Box::new(forge::Event::Client(skein_forge_client::Event::Answered {
        call: answer.call,
        cost: answer.cost,
        result: answer.result,
    })))
}
/// Hand a verified delivery hint to root admission.
#[must_use]
pub fn hinted(hinted: kit::hooks::Hinted) -> root::Event {
    root::Event::Forge(Box::new(forge::Event::Hint { hint: hinted.hint }))
}
/// Recover an answer refused by admission or returned during the drain.
pub fn back(event: Box<forge::Event>) -> Result<kit::Answered, Box<forge::Event>> {
    if let forge::Event::Client(skein_forge_client::Event::Answered { call, cost, result }) = *event {
        Ok(kit::Answered { call, cost, result })
    } else {
        Err(event)
    }
}
/// The small profile's component room, derived from the forge where they agree.
#[must_use]
pub fn small(forge: &forge::Limits) -> kit::Limits {
    kit::Limits {
        forges: 4,
        token_bytes: 512,
        connections: forge.client.calls,
        head_bytes: 16_384,
        headers: 64,
        body_bytes_in: 16_777_216,
        depth: 16,
        chunk: 4096,
        string_bytes: 512.max(forge.name_bytes),
        body_bytes: 16_384.max(forge.client.op_bytes.saturating_mul(2).saturating_add(21)),
        inner: 32,
        target_bytes: 4096,
        repositories: forge.repositories,
        users: 256,
        deadline: forge.client.lifetime,
        idle: Duration::from_secs(4),
        start_backoff: Duration::from_secs(1),
        start_backoff_max: Duration::from_secs(60),
        hooks: 4,
        hook_head_bytes: 8192,
        hook_body_bytes: 1_048_576,
        hook_deadline: Duration::from_secs(10),
        hook_secret_bytes: 128,
        tls: skein_tls::client::Limits { read: 16_384, send: 16_384, records: skein_tls::client::MAX_RECORD },
    }
}
/// Engine-owned queues and component, with its own output reservations.
pub(crate) struct Layer {
    pub(crate) component: kit::Component,
    pub(crate) env: Env<kit::Limits>,
    pub(crate) above: Queue<kit::Event>,
    pub(crate) io: Queue<skein_io::Request>,
    pub(crate) closed: bool,
    begun: bool,
}
impl Layer {
    pub(crate) fn new(
        settings: kit::settings::Settings,
        limits: kit::Limits,
        shared: &kit::Shared,
        batch: u32,
    ) -> Result<Self, kit::Unusable> {
        Ok(Self {
            component: kit::Component::new(settings, &limits, shared)?,
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::with_capacity(batch.checked_mul(kit::MAX_OUT.above).expect("forge above room")),
            io: Queue::with_capacity(batch.checked_mul(kit::MAX_OUT.below).expect("forge io room")),
            closed: false,
            begun: false,
        })
    }
    pub(crate) fn room(&self, batch: u32) -> bool {
        self.above.room() >= batch.checked_mul(kit::MAX_OUT.above).expect("forge batch above")
            && self.io.room() >= batch.checked_mul(kit::MAX_OUT.below).expect("forge batch io")
    }
    pub(crate) fn down(&mut self, request: kit::Request) {
        self.component.down(&self.env, request, &mut self.above, &mut self.io);
    }
    pub(crate) fn up(&mut self, event: skein_io::Event) {
        self.component.up(&self.env, crate::accounts::untag(event), &mut self.above, &mut self.io);
    }
    pub(crate) fn iterate(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
        if !self.room(1) {
            return;
        }
        if !self.begun {
            self.begun = true;
            self.component.begin(&self.env, &mut self.above, &mut self.io);
        } else if let Some(deadline) = self.component.next_deadline()
            && deadline <= now
        {
            self.component.fire(&self.env, &mut self.above, &mut self.io);
        }
        self.component.reclaim();
    }
    pub(crate) fn work_pending(&self, now: Time) -> bool {
        !self.begun
            || !self.above.is_empty()
            || !self.io.is_empty()
            || match self.component.next_deadline() {
                Some(deadline) => deadline <= now,
                None => false,
            }
    }
}
/// Report every member involved in the component's refusal.
pub fn problems(unusable: kit::Unusable, found: &mut Problems) {
    use kit::limits::{Stacked, Unusable};
    let names: &[&[u8]] = match unusable {
        Unusable::Connections { .. } => &[b"forge_component.connections", b"forge.client.calls"],
        Unusable::Deadline { .. } => &[b"forge_component.deadline_ms", b"forge.client.lifetime_ms"],
        Unusable::BodyBytes { .. } => &[b"forge_component.body_bytes", b"forge.client.op_bytes"],
        Unusable::StringBytes { .. } => &[b"forge_component.string_bytes", b"forge.name_bytes"],
        Unusable::Repositories { .. } => &[b"forge_component.repositories", b"forge.repositories"],
        Unusable::PageSize { .. } => &[
            b"forge_component.body_bytes",
            b"forge_component.string_bytes",
            b"forge_component.inner",
            b"forge.client.answer_bytes",
        ],
        Unusable::Documents(_) => {
            &[b"forge_component.depth", b"forge_component.body_bytes_in", b"forge_component.chunk"]
        }
        Unusable::Stack { stacked, .. } => match stacked {
            Stacked::HttpOverTls => {
                &[b"forge_component.chunk", b"forge_component.tls.read", b"forge_component.tls.send"]
            }
            Stacked::HttpOverIo => &[b"forge_component.chunk", b"io.intake", b"io.output"],
            Stacked::TlsOverIo => {
                &[b"forge_component.tls.read", b"forge_component.tls.send", b"io.intake", b"io.output"]
            }
            Stacked::JsonOverHttp | Stacked::JsonOverServer => &[b"forge_component.chunk"],
            Stacked::ServerOverIo => &[b"forge_component.hook_head_bytes", b"io.intake", b"io.output"],
        },
        Unusable::SecretBytes { .. } => &[b"forge_component.hook_secret_bytes"],
        Unusable::Secret { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"webhook_secret"), What::Bound);
            &[b"forge_component.hook_secret_bytes"]
        }
        Unusable::Listener => {
            found.push(Path::root().member(b"application").member(b"webhooks").member(b"listener"), What::Plaintext);
            return;
        }
        Unusable::Forges { .. } => &[b"forge_component.forges"],
        Unusable::Request { .. } => {
            &[b"forge_component.target_bytes", b"forge_component.token_bytes", b"forge_component.tls.send"]
        }
        Unusable::Zero => &[b"forge_component"],
        Unusable::Duplicate { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"number"), What::Unique);
            return;
        }
        Unusable::Token { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"api_token"), What::Bound);
            return;
        }
        Unusable::User { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"user"), What::Bound);
            return;
        }
        Unusable::Address { forge } => {
            found.push(forge_path(u32::from(forge)).member(b"api"), What::Plaintext);
            return;
        }
    };
    for name in names {
        found
            .push(Path::root().member(b"application").member(b"limits").member(b"overrides").member(name), What::Bound);
    }
}
pub(crate) fn tag(mut request: skein_io::Request) -> skein_io::Request {
    if let skein_io::Request::Connect { owner, .. }
    | skein_io::Request::Listen { owner, .. }
    | skein_io::Request::Bind { owner, .. } = &mut request
    {
        assert!(owner.raw() < (1_u64 << 60_u32), "forge token fits its prefix");
        *owner = skein_lib::Token::new(owner.raw() | crate::tokens::FORGE);
    }
    request
}

pub(crate) fn indexed(unusable: kit::Unusable, forges: &[settings::Forge]) -> kit::Unusable {
    let mut unusable = unusable;
    if let kit::Unusable::Duplicate { forge }
    | kit::Unusable::Secret { forge }
    | kit::Unusable::Token { forge }
    | kit::Unusable::User { forge }
    | kit::Unusable::Address { forge } = &mut unusable
    {
        for (index, configured) in forges.iter().enumerate() {
            if configured.number == *forge {
                *forge = u16::try_from(index).expect("bounded forge index");
                break;
            }
        }
    }
    unusable
}

pub(crate) fn prepare(
    config: &settings::Engine,
    runtime: List<ForgeRuntime>,
) -> Result<(kit::settings::Settings, u64), Problems> {
    let mut found = Problems::empty();
    let bytes = match roots(config, &runtime) {
        Ok(bytes) => bytes,
        Err(problems) => {
            found = problems;
            0
        }
    };
    let prepared = settings(&config.forges, runtime, config.webhooks.clone(), &mut found).ok_or(found)?;
    if let Err(why) =
        kit::limits::check(&config.limits.forge_component, &shared(&config.limits, &config.jig.limits.io), &prepared)
    {
        let mut found = Problems::empty();
        problems(indexed(why, &config.forges), &mut found);
        return Err(found);
    }
    Ok((prepared, bytes))
}
pub(crate) fn roots(config: &settings::Engine, runtime: &List<ForgeRuntime>) -> Result<u64, Problems> {
    let mut bytes = 0_u64;
    let mut found = Problems::empty();
    for endpoint in runtime {
        if let kit::settings::Transport::Tls { .. } = endpoint.transport {
            let mut index = 0_u32;
            for (at, forge) in config.forges.iter().enumerate() {
                if forge.number == endpoint.number {
                    index = u32::try_from(at).expect("forge index");
                    break;
                }
            }
            let path = forge_path(index).member(b"transport").member(b"tls").member(b"trust");
            if endpoint.trust_roots == 0 || endpoint.trust_root_bytes == 0 {
                found.push(path, What::Bound);
                continue;
            }
            let Some(roots) = skein_tls::roots_worst_case(endpoint.trust_roots, endpoint.trust_root_bytes) else {
                found.push(path, What::Bound);
                continue;
            };
            match bytes.checked_add(roots) {
                Some(total) => bytes = total,
                None => found.push(path, What::Bound),
            }
        }
    }
    if found.is_empty() { Ok(bytes) } else { Err(found) }
}

pub(crate) fn config(
    settings: &settings::Engine,
    deployment: [u8; 16],
) -> (skein_forge_client::Config, Box<[forge::GitIdentity]>) {
    let count = u32::try_from(settings.forges.len()).expect("configured forges");
    let mut writers = List::with_capacity(count);
    let mut identities = List::with_capacity(count);
    for forge in &settings.forges {
        writers.push(skein_forge_client::Writer { forge: forge.number, author: forge.user }).expect("forge writers");
        identities
            .push(forge::GitIdentity { forge: forge.number, credential: forge.identity })
            .expect("forge identities");
    }
    (
        skein_forge_client::Config { namespace: Box::new(deployment), writers: writers.into_boxed() },
        identities.into_boxed(),
    )
}
