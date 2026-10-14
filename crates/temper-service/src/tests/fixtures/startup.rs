#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary fixture code builds and edits configuration bytes"
)]
use crate::settings;
use skein_lib::{List, Secret};

pub const APPLICATION: &[u8] = br#"{
"forges":[{"number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:9"},"transport":"plaintext","git":"http://127.0.0.1:9","api_token":"secrets/forge-1.api-token","user":42,"identity":1}],
"identities":[{"number":1,"token":"secrets/forge-1.git-token","login":"temper","author":{"name":"Temper","email":"temper@example.test"}}],
"landing":{"deployment":[],"projects":[{"project":1,"rules":[{"connector":1,"kind":4,"pattern":{"segments":["forge","127.0.0.1:9","acme","app"],"last":"open","value":"main"},"enforce":true,"ci":false,"up_to_date":true,"gates":[{"number":3,"blocking":true,"freshness":"exact"}],"approvals":[{"role":0,"people":1,"freshness":"clean"}]}]}]},
"charter":1,"limits":{"profile":"small","overrides":{}},"workers":[{"root":"worker-root","git":"/usr/bin/git"}]
}"#;
pub fn document() -> Box<[u8]> {
    let mut document = Vec::from(b"{\"engine\":".as_slice());
    let engine = core::str::from_utf8(super::ENGINE)
        .unwrap()
        .replace(",\n{\"name\": \"worker-b\", \"digest\": \"secrets/worker-b.digest\"}", "");
    document.extend_from_slice(engine.as_bytes());
    document.extend_from_slice(br#", "workers":[{
"engine":{"in_process":"workers"},"name":"worker-a","secret":"worker-secret","slots":1,
"graces":{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000},
"agent":{"program":"/opt/temper/bin/temper","configuration":"/opt/temper/agent.json","arguments":["agent","/opt/temper/agent.json"]},
"limits":{"profile":"small","overrides":{"hub.slots":1,"hub.run_calls":1,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}},
"memory_bytes":1152921504606846976}],"application":"#);
    document.extend_from_slice(APPLICATION);
    document.extend_from_slice(b"}");
    document.into_boxed_slice()
}
pub fn parsed() -> skein_json::Document {
    jig_protocol_config::document(
        &document(),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap()
}
pub fn files() -> settings::Files {
    let mut entries = List::with_capacity(7);
    for (path, value) in [
        (b"secrets/forge-1.api-token".as_slice(), b"api-token\n".as_slice()),
        (b"secrets/forge-1.git-token".as_slice(), b"git-token\n".as_slice()),
        (b"worker-secret".as_slice(), super::worker::WORKER_SECRET),
        (
            b"secrets/worker-a.digest".as_slice(),
            b"8f7be71db3941de11249a5e3fba7750a41c7d9958083ef522cb79e9fd7822878".as_slice(),
        ),
        (b"charters/chat.txt".as_slice(), b"Answer briefly".as_slice()),
        (b"accounts/chat.json".as_slice(), super::ACCOUNT),
        (super::AGENT_JSON, b"{}".as_slice()),
    ] {
        entries.push((path.into(), Secret::new(value.into()))).unwrap();
    }
    settings::Files { entries }
}
pub fn without(entries: List<(Box<[u8]>, Secret)>, name: &[u8]) -> List<(Box<[u8]>, Secret)> {
    let mut kept = List::with_capacity(entries.len());
    for entry in entries.into_boxed() {
        if entry.0.as_ref() != name {
            kept.push(entry).unwrap();
        }
    }
    kept
}
pub fn worker_runtime() -> crate::WorkerRuntime {
    crate::WorkerRuntime { root: skein_io::kernel::Fd::new(9), path: b"/worker".as_slice().into() }
}
pub fn startup_agent() -> settings::Agent {
    let mut agent = super::agent();
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    agent.host_limits.directories = 3;
    agent.host_limits.name_bytes = 256;
    agent.host_limits.accounts = hub.accounts;
    agent.host_limits.messages = hub.held.max(hub.inbox);
    agent.host_limits.call_bytes = 256;
    agent.host_limits.message_bytes = 65_538;
    agent.bounds.message_bytes = 512;
    agent
}
