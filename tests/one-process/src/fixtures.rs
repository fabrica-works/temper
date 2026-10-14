//! Configuration bytes, private files and scripts shared by the stories.
use crate::world::Settings;
use jig_channel_world::fixtures as channel;
use skein_lib::{Duration, List, Secret};
pub const SEED: u64 = 7;
pub const ANSWER: &[u8] = b"World answer";
pub const BRANCH_HEAD: u64 = 2;
pub const MOVED_BASE: u64 = 3;
pub(crate) const MERGE_BRANCH: &[u8] = b"temper/1/c2";
pub(crate) const PREPARED_REPOSITORY: &[u8] = b"workspaces/0/app";
pub(crate) const AGENTS: u32 = 2;
#[must_use]
pub fn calm(seed: u64) -> Settings {
    Settings { seed, config: skein_sim::Config::calm() }
}
pub(crate) fn document() -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(&channel::engine_document()).expect("engine JSON");
    value.as_object_mut().expect("object").remove("service");
    let worker: serde_json::Value = serde_json::from_slice(channel::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"] = serde_json::json!({"program":"/opt/temper/bin/temper","configuration":"/opt/temper/agent.json","arguments":["agent","/opt/temper/agent.json"]});
    value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 1024.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["connectors"] = serde_json::json!([1]);
    for (name, count) in [
        ("core.load_slots", 8),
        ("documents.rows", 2),
        ("documents.text_bytes", 128),
        ("people.sign_ins", 2),
        ("views.watchers", 16),
        ("views.watchers_per_party", 16),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    value["application"] = serde_json::json!({
        "forges":[{"number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:3000"},"transport":"plaintext",
            "git":"http://127.0.0.1:3000","api_token":"secrets/forge.api-token","user":1,"identity":1}],
        "identities":[{"number":1,"token":"secrets/forge.git-token","login":"worker","author":{"name":"Worker","email":"worker@example.invalid"}}],
        "landing":{"deployment":[],"projects":[]},"charter":1,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,"checkout.message_bytes":4096,"forge.name_bytes":64,"forge.resources_per_task":2,"git_component.stdout_bytes":2048}},
        "workers":[{"root":"worker","git":"/usr/bin/git"}]});
    crate::stories::configure(&mut value);
    serde_json::to_vec(&value).expect("deployment JSON")
}
pub(crate) fn files() -> temper_service::settings::Files {
    let mut entries = List::with_capacity(8);
    for source in [channel::files(false), channel::files(true)] {
        for (name, value) in source.entries.into_boxed() {
            let name = if name.as_ref() == b"agent.json" { b"/opt/temper/agent.json".as_slice().into() } else { name };
            entries.push((name, value)).expect("fixture files");
        }
    }
    for name in [b"secrets/forge.api-token".as_slice(), b"secrets/forge.git-token"] {
        entries.push((name.into(), Secret::new(temper_fake_git::fixtures::TOKEN.into()))).expect("forge files");
    }
    temper_service::settings::Files { entries }
}
pub(crate) fn seeds() -> temper_service::settings::Seeds {
    let seed = channel::seeds();
    temper_service::settings::Seeds { deployment: seed.deployment, seed: seed.seed, crypto: seed.crypto }
}
pub(crate) fn agent() -> temper_service::settings::Agent {
    let mut agent = channel::agent();
    let limits = agent_limits();
    agent.host_limits =
        smith_local_service::profile::host_limits(&agent_profile(), &limits).expect("agent host limits");
    agent.host_limits.agents = AGENTS;
    let channel = limits.channel;
    agent.limits = smith_host_protocol::Limits {
        bodies: channel.bodies,
        charter: channel.charter,
        transcript: channel.transcript,
        endpoints: channel.endpoints,
        channel: channel.channel,
        start_bytes: channel.start_bytes,
        calls: agent.host_limits.calls,
    };
    agent.limits.channel.output_bytes = smith_host_protocol::output_bytes(&agent.limits).expect("host frames");
    agent.bounds.message_bytes = 1024;
    agent.bounds.checks = Duration::from_secs(5);
    temper_service::settings::Agent {
        endpoints: agent.endpoints,
        limits: agent.limits,
        bounds: agent.bounds,
        host_limits: agent.host_limits,
    }
}
pub(crate) fn agent_profile() -> smith_agent_service::profile::Profile {
    let mut profile = smith_agent_process_world::profile();
    profile.policy.host_tools =
        u32::try_from(temper_engine_smith::tools(Duration::from_secs(10)).len()).expect("tool count");
    profile.declared.tool_payload = 1400;
    profile.declared.read_window = 256;
    profile.declared.conversations = 1;
    profile.declared.calls_per_response = 1;
    profile.policy.check_timeout = Duration::from_secs(5);
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    profile.policy.inbox = hub.held.max(hub.inbox);
    profile.policy.sections = 8;
    profile
}
pub(crate) fn agent_limits() -> smith_agent_service::Limits {
    let mut limits =
        smith_agent_service::profile::derive(&agent_profile(), &agent_declarations()).expect("chat agent limits");
    limits.llm.accounts = 2;
    limits
}
pub(crate) fn agent_declarations() -> smith_agent_service::profile::Configuration {
    let mut declarations = smith_agent_process_world::declarations(0);
    declarations.endpoints[0].models[0].output = 2048;
    declarations
}
pub(crate) fn agent_configuration() -> smith_agent_service::Config {
    let mut config = channel::agent_configuration();
    config.limits = agent_limits();
    let (destinations, _) = config.llm_endpoints.into_parts();
    let mut destination = destinations.into_boxed().into_vec().pop().expect("one LLM endpoint");
    destination.limits = config.limits.llm.adapter.client;
    config.llm_endpoints = smith_protocol_llm::Endpoints::new(
        Box::new([smith_protocol_llm::ConfiguredEndpoint {
            name: smith_domain::llm::Endpoint(0),
            destination,
            account: 1,
            connect: Duration::from_secs(1),
            handshake: Duration::from_secs(1),
            reasoning_effort: None,
            identity: smith_protocol_llm::IdentityProfile::Plain,
            models: Box::default(),
        }]),
        1,
        2,
    )
    .expect("derived provider endpoint");
    config
}
pub(crate) fn forges() -> List<temper_service::ForgeRuntime> {
    let mut forges = List::with_capacity(1);
    forges
        .push(temper_service::ForgeRuntime {
            number: 1,
            address: ([127, 0, 0, 1], 3000).into(),
            transport: temper_engine_protocol_forge::settings::Transport::Plaintext,
            trust_roots: 0,
            trust_root_bytes: 0,
        })
        .expect("one loopback forge");
    forges
}
pub(crate) fn provider() -> skein_fake_peers::llm::Peer {
    let mut script = channel::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"World answer","fields":{}}"#.as_slice().into();
    smith_agent_process_world::fake::configured(
        Box::new([skein_fake_llm_domain::api::Script { cue: Box::default(), turns: channel::provider_turns(&script) }]),
        skein_llm::Credential { access_token: channel::TOKEN.into(), account_id: b"acc".as_slice().into() },
        Duration::ZERO,
    )
}
pub(crate) fn read_then_wait_provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    smith_agent_process_world::fake::configured(
        Box::new([Script {
            cue: Box::default(),
            turns: Box::new([
                Turn {
                    lines: Box::new([Line::Call {
                        name: b"read".as_slice().into(),
                        arguments: br#"{"path":"README.md"}"#.as_slice().into(),
                    }]),
                    finish: Finish::ToolCalls,
                    tokens: 20,
                },
                Turn {
                    lines: Box::new([
                        Line::Text { text: ANSWER.into() },
                        Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                    ]),
                    finish: Finish::ToolCalls,
                    tokens: 20,
                },
            ]),
        }]),
        skein_llm::Credential { access_token: channel::TOKEN.into(), account_id: b"acc".as_slice().into() },
        Duration::ZERO,
    )
}
pub(crate) fn codec() -> jig_protocol_store::codec::Limits {
    let bytes = document();
    let size = u32::try_from(bytes.len()).expect("fixture document");
    let doc = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: size, tokens: size, depth: 32 },
    )
    .expect("document");
    let mut object = jig_protocol_config::read::Object::new(&doc, 0).expect("object");
    let at = object.member(&doc, b"engine").expect("engine member");
    let engine = jig_protocol_config::engine(&doc, at).expect("engine");
    let limits = jig_test_service_limits(&engine.limits);
    jig_protocol_store::codec::limits(&limits.root.core).expect("codec")
}
fn jig_test_service_limits(overrides: &jig_protocol_config::Limits) -> jig_test_service::limits::Limits {
    jig_test_service::limits::for_profile(jig_protocol_config::Profile::Small, overrides)
}
