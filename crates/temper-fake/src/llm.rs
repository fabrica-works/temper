//! The QA model's document and played scripts (protocol/setup.md, 8.4).
use crate::{Refusal, read, turn};
use alloc::boxed::Box;
use core::net::SocketAddr;
use jig_protocol_config::Path;
use skein_fake_llm_domain::{self as domain, api};
use skein_json::Document as Json;
use skein_lib::{Duration, List};

/// One checked advertised model. The peer has no model-name admission table.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Model {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}
/// A standalone model document, checked before any output is created.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document {
    listen: SocketAddr,
    key: Box<[u8]>,
    root: Option<Box<[u8]>>,
    models: Box<[Model]>,
    scripts: api::Scripts,
}
impl Document {
    pub fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let d = read::document(bytes)?;
        let p = Path::root();
        let mut o = read::object(&d, 0, &p)?;
        let listen_at = read::required(&mut o, &d, b"listen", &p)?;
        let key = read::field(&mut o, &d, b"key", crate::limits::PATH_BYTES, &p)?;
        let tls_at = read::required(&mut o, &d, b"tls", &p)?;
        let models_at = read::required(&mut o, &d, b"models", &p)?;
        let cued_at = read::required(&mut o, &d, b"cued", &p)?;
        let asked_at = read::required(&mut o, &d, b"asked", &p)?;
        let otherwise_at = read::required(&mut o, &d, b"otherwise", &p)?;
        read::finish(o, &d, &p)?;
        let listen = read::listen(&d, listen_at, &p.member(b"listen"))?;
        read::absolute(&key, &p.member(b"key"))?;
        let root = if read::is_null(&d, tls_at) {
            None
        } else {
            let p = p.member(b"tls");
            let mut o = read::object(&d, tls_at, &p)?;
            let root = read::field(&mut o, &d, b"root", crate::limits::PATH_BYTES, &p)?;
            read::finish(o, &d, &p)?;
            read::absolute(&root, &p.member(b"root"))?;
            Some(root)
        };
        let models = models(&d, models_at, &p.member(b"models"))?;
        let scripts = api::Scripts {
            cued: scripts(&d, cued_at, &p.member(b"cued"))?,
            asked: scripts(&d, asked_at, &p.member(b"asked"))?,
            otherwise: turns(&d, otherwise_at, &p.member(b"otherwise"))?,
        };
        Self::checked(listen, key, root, models, scripts, &p)
    }
    pub(crate) fn checked(
        listen: SocketAddr,
        key: Box<[u8]>,
        root: Option<Box<[u8]>>,
        models: Box<[Model]>,
        scripts: api::Scripts,
        p: &Path,
    ) -> Result<Self, Refusal> {
        check_scripts(&scripts, &p.member(b"otherwise"))?;
        Ok(Self { listen, key, root, models, scripts })
    }
    #[must_use]
    pub fn listen(&self) -> SocketAddr {
        self.listen
    }
    #[must_use]
    pub fn key(&self) -> &[u8] {
        &self.key
    }
    #[must_use]
    pub fn root(&self) -> Option<&[u8]> {
        self.root.as_deref()
    }
    #[must_use]
    pub fn models(&self) -> &[Model] {
        &self.models
    }
    /// No menu, failure or latency. Unknown model names receive the same scripts:
    /// skein's provider retains no model list and performs no per-model admission.
    #[must_use]
    pub fn domain(&self, seed: u64) -> domain::Domain {
        domain::Domain::played(&LIMITS, seed, self.scripts.clone(), empty_menu()).expect("checked scripts")
    }
}
/// `OpenAI`'s public-route configuration, accepting exactly the supplied key.
/// Typed wire codecs are constructed by the peer factory; the document's advertised
/// models stay here beside the played domain, which has no per-model admission.
#[derive(Debug)]
pub struct Configuration {
    pub listen: SocketAddr,
    pub tls: bool,
    pub key: skein_lib::Secret,
    pub models: Box<[Model]>,
    pub domain: domain::Domain,
}
/// The public `OpenAI` Responses route; no Codex account id is configured.
pub const ROUTE: &[u8] = b"/v1/responses";
/// Builds the played domain without a menu, failures or latency, and its one key.
#[must_use]
pub fn configuration(document: &Document, key: &[u8], seed: u64) -> Configuration {
    Configuration {
        listen: document.listen,
        tls: document.root.is_some(),
        key: skein_lib::Secret::new(skein_lib::bytes::copy_of(key)),
        models: document.models.clone(),
        domain: document.domain(seed),
    }
}
fn empty_menu() -> api::Menu {
    api::Menu { arguments: Box::new([]), invalid: Box::new([]) }
}
fn models(d: &Json, at: u32, p: &Path) -> Result<Box<[Model]>, Refusal> {
    let items = read::list(d, at, 1, p)?;
    let mut out = List::<Model>::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let p = read::index(p, i);
        let mut o = read::object(d, *at, &p)?;
        let name = read::field(&mut o, d, b"name", crate::limits::MODEL_NAME_BYTES, &p)?;
        let window_at = read::required(&mut o, d, b"window", &p)?;
        let output_at = read::required(&mut o, d, b"output", &p)?;
        read::finish(o, d, &p)?;
        let window = read::integer(d, window_at, u64::from(u32::MAX), &p.member(b"window"))?;
        let output = read::integer(d, output_at, u64::from(u32::MAX), &p.member(b"output"))?;
        out.push(Model {
            name,
            window: u32::try_from(window).expect("u32 bound"),
            output: u32::try_from(output).expect("u32 bound"),
        })
        .expect("eight models");
    }
    check_models(out.as_slice(), p, true)?;
    Ok(out.into_boxed())
}
/// The same model bounds for the document and a deployment's derivation.
/// The latter reports every model problem at its provider member.
pub(crate) fn check_models(models: &[Model], p: &Path, indexed: bool) -> Result<(), Refusal> {
    if models.len() > usize::try_from(crate::ENTRIES).expect("eight models") {
        return Err(read::why(p, b"past its bound"));
    }
    if models.is_empty() {
        return Err(read::why(p, b"must be non-empty"));
    }
    for (i, model) in models.iter().enumerate() {
        let here = if indexed { read::index(p, i) } else { p.clone() };
        let name = if indexed { here.member(b"name") } else { here.clone() };
        let window = if indexed { here.member(b"window") } else { here.clone() };
        let output = if indexed { here.member(b"output") } else { here.clone() };
        read::nonempty(&model.name, &name)?;
        if model.name.len() > usize::try_from(crate::limits::MODEL_NAME_BYTES).expect("name bound") {
            return Err(read::why(&name, b"past its bound"));
        }
        if model.window == 0 {
            return Err(read::why(&window, b"must be at least 1"));
        }
        if model.output == 0 {
            return Err(read::why(&output, b"must be at least 1"));
        }
        if model.output > model.window {
            return Err(read::why(&output, b"must not exceed window"));
        }
        for prev in models.iter().take(i) {
            if prev.name == model.name {
                return Err(read::why(&name, b"must be unique"));
            }
        }
    }
    Ok(())
}
pub(crate) fn check_scripts(scripts: &api::Scripts, p: &Path) -> Result<(), Refusal> {
    // The domain owns the joint byte accounting, including every wrapper.
    match domain::Domain::played(&LIMITS, 0, scripts.clone(), empty_menu()) {
        Ok(_) => Ok(()),
        Err(_) => Err(read::why(p, b"scripts past their joint byte bound")),
    }
}
pub(crate) fn scripts(d: &Json, at: u32, p: &Path) -> Result<Box<[api::Script]>, Refusal> {
    let items = read::list(d, at, 0, p)?;
    let mut out = List::<api::Script>::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let p = read::index(p, i);
        let mut o = read::object(d, *at, &p)?;
        let cue = read::field(&mut o, d, b"cue", crate::limits::CUE_BYTES, &p)?;
        let turns_at = read::required(&mut o, d, b"turns", &p)?;
        read::finish(o, d, &p)?;
        read::nonempty(&cue, &p.member(b"cue"))?;
        for prev in &out {
            if prev.cue == cue {
                return Err(read::why(&p.member(b"cue"), b"must be unique"));
            }
        }
        let turns = turns(d, turns_at, &p.member(b"turns"))?;
        out.push(api::Script { cue, turns }).expect("eight scripts");
    }
    Ok(out.into_boxed())
}
pub(crate) fn turns(d: &Json, at: u32, p: &Path) -> Result<Box<[api::Turn]>, Refusal> {
    let items = read::list(d, at, 1, p)?;
    let mut out = List::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let turn = turn::Turn::read(d, *at, &read::index(p, i))?;
        out.push(turn::into_skein(turn)).expect("eight turns");
    }
    Ok(out.into_boxed())
}
/// Live-domain limits: joint script bytes include all 17 scripts' wrappers.
pub const LIMITS: domain::Config = domain::Config {
    calls: 8,
    query_bytes: 1 << 20,
    script_bytes: crate::SCRIPT_BYTES,
    answer_bytes: crate::ANSWER_BYTES,
    latency_min: Duration::ZERO,
    latency_max: Duration::ZERO,
    overloaded: 0,
    rate_limited: 0,
    retry_after: Duration::ZERO,
    unavailable: 0,
    too_long: 0,
    unauthorized: 0,
    refused: 0,
    no_calls: 0,
    answer_tokens: 16_384,
    calls_per_answer: 8,
    malformed: 0,
    outside_choice: 0,
    tool_rounds: 0,
    cache_lifetime: Duration::ZERO,
    cache_entries: 0,
    unscoped_reads: 0,
};

/// Constructs the checked configuration's peer without io (protocol/setup.md, 8.4).
pub fn peer(configuration: Configuration) -> Result<skein_fake_peers::llm::Peer, skein_fake_peers::Error> {
    crate::wire::llm(configuration)
}
