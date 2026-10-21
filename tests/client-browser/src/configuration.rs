//! The process tier's deployment, copied: `documents` from `crates/temper/tests/support/configuration.rs`
//! and `write` from `support::configuration` with `Forge::Unreached`, which are test modules of package
//! `temper` and cannot be imported. Three changes: the LLM endpoint is unreached at `127.0.0.1:9`, beside
//! the forge; `application.clients.bundle` names the bundle; and every relative path is made absolute
//! under the scratch, as jig's `tests/client-browser` does, since `temper serve` joins paths to its working
//! directory and the harness starts it in the test's.
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

/// A closed loopback port: the forge's and the LLM endpoint's, which no story reaches.
pub(crate) const UNREACHED: &str = "127.0.0.1:9";
/// The paths the deployment names, each relative to its directory.
const RELATIVE: &[&str] = &[
    "charters/chat.txt",
    "secrets/worker-a.digest",
    "app.secret",
    "worker.secret",
    "secrets/forge-1.user",
    "secrets/forge-1.api-token",
    "secrets/forge-1.git-token",
    "state",
    "worker-root",
];

pub(crate) fn documents(directory: &Path, program: &Path, bundle: &Path) -> (Box<[u8]>, Box<[u8]>) {
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":600,"read_window":256,
            "conversations":1,"calls_per_response":1},
            "policy":{"max_turns":5,"group_stop":10,"inbox":8,"unacknowledged":8}},
        "endpoints":[{"name":"fake","number":0,"dialect":3,"provider":"openai",
            "address":UNREACHED,"transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),
            "identity":"plain","connect":1000,"handshake":1000,
            "models":[{"name":"fake","prices":{"input":1,"cached":1,"output":1,"unit":1_000_000},"window":1_048_576,"output":1024,"reasoning_item":64,"head":60000,"idle":30000}]}],
        "environment":[]});
    let mut value: serde_json::Value =
        serde_json::from_slice(&jig_channel_world::fixtures::engine_document()).expect("engine JSON");
    value.as_object_mut().expect("whole object").remove("service");
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_world::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"]["program"] = program.to_str().expect("program path").into();
    value["workers"][0]["agent"]["configuration"] = directory.join("agent.json").to_str().expect("agent path").into();
    value["workers"][0]["agent"]["arguments"] = serde_json::json!(["agent", directory.join("agent.json")]);
    value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 598.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["connectors"] = serde_json::json!([1]);
    value["engine"]["llm"]["endpoints"][0]["dialect"] = 3.into();
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":UNREACHED});
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["documents.rows"] = 2.into();
    value["engine"]["limits"]["overrides"]["documents.lines"] = 2.into();
    value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 128.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    value["engine"]["limits"]["overrides"]["views.watchers"] = 16.into();
    value["engine"]["limits"]["overrides"]["views.watchers_per_party"] = 16.into();
    value["application"] = serde_json::json!({
        "webhooks":null,
        "clients":{"listener":{"socket":"127.0.0.1:0"},"bundle":bundle},
        "forges":[{"number":1,"kind":"forgejo","api":{"socket":UNREACHED},"transport":"plaintext",
            "git":format!("http://{UNREACHED}"),"webhook_secret":null,"api_token":"secrets/forge-1.api-token","user":{"file":"secrets/forge-1.user"},"identity":1}],
        "identities":[{"number":1,"token":"secrets/forge-1.git-token","login":"temper",
            "author":{"name":"Temper","email":"temper@example.test"}}],
        "setup":[],"landing":{"deployment":[],"projects":[]},"charter":1,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,
            "checkout.message_bytes":4096,"forge.name_bytes":64,"forge.resources_per_task":2,"git_component.stdout_bytes":2048}},
        "workers":[{"root":"worker-root","git":"/usr/bin/git"}]});
    absolute_paths(&mut value, directory);
    (
        serde_json::to_vec(&agent).expect("agent JSON").into_boxed_slice(),
        serde_json::to_vec(&value).expect("deployment JSON").into_boxed_slice(),
    )
}
/// Every string that is one of the deployment's relative paths becomes that path under `directory`.
fn absolute_paths(value: &mut serde_json::Value, directory: &Path) {
    match value {
        serde_json::Value::String(text) if RELATIVE.contains(&text.as_str()) => {
            directory.join(&*text).to_str().expect("fixture path").clone_into(text);
        }
        serde_json::Value::Array(items) => {
            for item in items {
                absolute_paths(item, directory);
            }
        }
        serde_json::Value::Object(items) => {
            for item in items.values_mut() {
                absolute_paths(item, directory);
            }
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
    }
}
pub(crate) fn private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}
/// The deployment's files and its two documents, `agent.json` and `temper.json`, in `directory`.
pub(crate) fn write(directory: &Path, program: &Path, bundle: &Path) {
    assert!(Path::new("/usr/bin/git").is_file(), "/usr/bin/git is required");
    for name in ["", "accounts", "secrets", "charters", "state"] {
        fs::create_dir_all(directory.join(name)).expect("deployment directory");
        fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o700)).expect("private directory");
    }
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        private(&directory.join(std::str::from_utf8(&name).expect("fixture path")), bytes.expose());
    }
    private(&directory.join("worker.secret"), b"worker-secret");
    private(&directory.join("secrets/forge-1.user"), b"42\n");
    private(&directory.join("secrets/forge-1.api-token"), b"forge-1-api-token-value");
    private(&directory.join("secrets/forge-1.git-token"), b"forge-1-git-token-value");
    private(&directory.join("root.der"), skein_tls_world::pki::ROOT);
    let (agent, value) = documents(directory, program, bundle);
    private(&directory.join("agent.json"), &agent);
    private(&directory.join("temper.json"), &value);
}
