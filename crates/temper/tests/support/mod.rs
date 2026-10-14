//! Shared configuration fixtures for startup and the process tier.
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub enum Forge {
    Unreached,
}
fn private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}
pub fn configuration(directory: &Path, forge: &Forge) {
    let Forge::Unreached = forge;
    assert!(Path::new("/usr/bin/git").is_file(), "/usr/bin/git is required");
    for name in ["accounts", "accounts/records", "secrets", "charters", "state"] {
        fs::create_dir(directory.join(name)).expect("deployment directory");
    }
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        private(&directory.join(std::str::from_utf8(&name).expect("fixture path")), bytes.expose());
    }
    private(&directory.join("worker.secret"), b"worker-secret");
    private(
        &directory.join("accounts/chat.json"),
        br#"{"access_token":"token","expires_at":4102444800,"account_id":"acc"}"#,
    );
    private(&directory.join("secrets/forge-1.api-token"), b"forge-1-api-token-value");
    private(&directory.join("secrets/forge-1.git-token"), b"forge-1-git-token-value");
    private(&directory.join("root.der"), skein_tls_world::pki::ROOT);
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":512,"read_window":256},
            "policy":{"max_turns":5,"group_stop":10}},
        "endpoints":[{"name":"fake","number":0,"dialect":0,"account":1,"provider":"codex",
            "address":"127.0.0.1:34443","transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),
            "identity":"plain","connect":1000,"handshake":1000,
            "models":[{"name":"fake","window":1_048_576,"output":1024,"reasoning_item":64,"head":60000,"idle":30000}]}],
        "environment":[]});
    private(&directory.join("agent.json"), &serde_json::to_vec(&agent).expect("agent JSON"));
    let mut value: serde_json::Value =
        serde_json::from_slice(&jig_channel_world::fixtures::engine_document()).expect("engine JSON");
    value.as_object_mut().expect("whole object").remove("service");
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_world::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"]["program"] = env!("CARGO_BIN_EXE_temper").into();
    value["workers"][0]["agent"]["configuration"] = directory.join("agent.json").to_str().expect("agent path").into();
    value["workers"][0]["agent"]["arguments"] = serde_json::json!(["agent", directory.join("agent.json")]);
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["clients"]["listener"] = serde_json::json!({"socket":"127.0.0.1:0"});
    value["engine"]["connectors"] = serde_json::json!([1]);
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:34443"});
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    value["application"] = serde_json::json!({
        "forges":[{"number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:9"},"transport":"plaintext",
            "git":"http://127.0.0.1:9","api_token":"secrets/forge-1.api-token","user":42,"identity":1}],
        "identities":[{"number":1,"token":"secrets/forge-1.git-token","login":"temper",
            "author":{"name":"Temper","email":"temper@example.test"}}],
        "landing":{"deployment":[],"projects":[]},"charter":1,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":2048}},
        "workers":[{"root":"worker-root","git":"/usr/bin/git"}]});
    private(&directory.join("config.json"), &serde_json::to_vec(&value).expect("deployment JSON"));
}
