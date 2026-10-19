//! Independent LLM peer configuration (skein's fake-llm.md, 6).
use skein_fake_llm_domain::{self as domain, api};
use skein_fake_llm_protocol::{documents, provider};
use skein_fake_peers::{Limits, Transport, llm::Peer};
use skein_lib::{Duration, Time};
use std::net::{Ipv4Addr, SocketAddr};

/// Every argument after the peer's address, in its own boundary types.
pub struct Recipe {
    pub transport: Transport,
    pub limits: Limits,
    pub config: provider::Config,
    pub credential: skein_llm::Credential,
    pub protocol_limits: provider::Limits,
    pub domain: domain::Domain,
    pub domain_limits: domain::Config,
}

#[must_use]
pub fn scripted(scripts: Box<[api::Script]>) -> Recipe {
    // Smith's configured peer bounds, for an agent's prompt and finish call.
    const STRINGS: u32 = 16_384;
    const REQUEST: u32 = 32_768;
    let mut config = skein_llm_world::fake::config();
    config.latency_min = Duration::ZERO;
    config.latency_max = Duration::ZERO;
    config.script_bytes = config.script_bytes.max(STRINGS);
    config.answer_bytes = config.answer_bytes.max(STRINGS);
    config.query_bytes = config.query_bytes.max(STRINGS);
    let mut protocol = skein_llm_world::fake::limits(&skein_llm_world::limits());
    protocol.documents.openai.request = REQUEST;
    protocol.documents.openai.strings = STRINGS;
    protocol.documents.openai.answer = protocol.documents.openai.answer.max(STRINGS);
    protocol.http.body = protocol.http.body.max(u64::from(REQUEST));
    protocol.sse.event = protocol.sse.event.max(STRINGS);
    protocol.documents.openai.tokens = 4096;
    protocol.documents.openai.output_items = 256;
    protocol.documents.anthropic.output_items = 256;
    Recipe {
        transport: Transport::Plaintext,
        limits: smith_agent_process_world::fake::limits(),
        config: provider::Config {
            echo: skein_llm::openai::Echo::NONE,
            usage_fields: documents::UsageFields::ALL,
            provider: documents::Provider::OpenAi,
            path: skein_llm_world::call(1).endpoint.target,
            headers: Box::new([]),
        },
        credential: skein_llm::Credential {
            access_token: crate::fixtures::TOKEN.into(),
            account_id: b"acc".as_slice().into(),
        },
        protocol_limits: protocol,
        domain: domain::Domain::try_scripted(&config, 17, scripts).expect("bounded real-loop scripts"),
        domain_limits: config,
    }
}

pub fn peer(recipe: Recipe) -> Result<Peer, skein_fake_peers::Error> {
    Peer::new(
        (Ipv4Addr::LOCALHOST, 0).into(),
        recipe.transport,
        recipe.limits,
        recipe.config,
        recipe.credential,
        recipe.protocol_limits,
        recipe.domain,
        recipe.domain_limits,
    )
}

/// Bind and drain the peer alone under exact ownership checking.
pub fn bound(recipe: impl FnOnce() -> Recipe) -> SocketAddr {
    let mut world = skein_world::real::World::new_checked(Bound { seen: false, wake: None });
    world.spawn(|| peer(recipe()).expect("configured independent peer"));
    let outcome = world.run(&skein_shell::Clock::new(), crate::PATIENCE);
    outcome.procs[0].address().expect("peer listened before shutdown")
}

struct Bound {
    seen: bool,
    wake: Option<Time>,
}
impl skein_world::Referee<Peer> for Bound {
    fn act(&mut self, _now: Time, procs: &mut [Peer]) {
        if !self.seen && procs[0].address().is_some() {
            self.seen = true;
            self.wake = None;
            procs[0].shutdown();
        }
    }
    fn observe(&mut self, now: Time, procs: &[Peer]) {
        if !self.seen && procs[0].address().is_some() {
            self.wake = Some(now);
        }
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake
    }
    fn overdue(&self, _now: Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        self.seen
    }
}
