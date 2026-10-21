//! Pure configuration fixtures shared by unit and process tests.
use std::path::Path;
#[must_use]
pub fn documents(directory: &Path, program: &str) -> (Vec<u8>, Vec<u8>) {
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":600,"read_window":256,
            "conversations":1,"calls_per_response":1},
            "policy":{"max_turns":5,"group_stop":10,"inbox":8,"unacknowledged":8}},
        "endpoints":[{"name":"fake","number":0,"dialect":3,"provider":"openai",
            "address":"127.0.0.1:34443","transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),
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
    value["workers"][0]["agent"]["program"] = program.into();
    value["workers"][0]["agent"]["configuration"] = directory.join("agent.json").to_str().expect("agent path").into();
    value["workers"][0]["agent"]["arguments"] = serde_json::json!(["agent", directory.join("agent.json")]);
    value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 598.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["connectors"] = serde_json::json!([1]);
    value["engine"]["llm"]["endpoints"][0]["dialect"] = 3.into();
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:34443"});
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["documents.rows"] = 2.into();
    value["engine"]["limits"]["overrides"]["documents.lines"] = 2.into();
    value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 128.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    value["engine"]["limits"]["overrides"]["views.watchers"] = 16.into();
    value["engine"]["limits"]["overrides"]["views.watchers_per_party"] = 16.into();
    // The native deployment account controls use the real membership in project zero.
    let mut deployment = value["engine"]["projects"][0].clone();
    deployment["number"] = 0.into();
    value["engine"]["projects"].as_array_mut().expect("projects").push(deployment);
    value["engine"]["limits"]["overrides"]["people.projects"] = 2.into();
    value["engine"]["limits"]["overrides"]["authority.projects"] = 2.into();
    value["engine"]["limits"]["overrides"]["tasks.funders"] = 4.into();
    value["application"] = serde_json::json!({
        "webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:0"},"bundle":null},
        "forges":[{"number":1,"kind":"forgejo","api":{"socket":"127.0.0.1:9"},"transport":"plaintext",
            "git":"http://127.0.0.1:9","webhook_secret":null,"api_token":"secrets/forge-1.api-token","user":{"file":"secrets/forge-1.user"},"identity":1}],
        "identities":[{"number":1,"token":"secrets/forge-1.git-token","login":"temper",
            "author":{"name":"Temper","email":"temper@example.test"}}],
        "setup":[],"demo":null,"landing":{"deployment":[],"projects":[]},"charter":1,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,
            "checkout.message_bytes":4096,"forge.name_bytes":64,"forge.resources_per_task":2,"git_component.stdout_bytes":2048}},
        "workers":[{"root":"worker-root","git":"/usr/bin/git"}]});
    (serde_json::to_vec(&agent).expect("agent JSON"), serde_json::to_vec(&value).expect("deployment JSON"))
}

/// All directories are explicit and belong to this fixture's scratch.
#[must_use]
pub fn directories(directory: &Path) -> temper::directories::Directories {
    use std::os::unix::ffi::OsStrExt;
    let values = [
        b"--config".as_slice(),
        directory.as_os_str().as_bytes(),
        b"--data",
        directory.as_os_str().as_bytes(),
        b"--cache",
        directory.as_os_str().as_bytes(),
    ];
    temper::directories::resolve(&values.iter().map(|v| Box::from(*v)).collect::<Vec<_>>(), &[], b"/")
        .expect("fixture directories")
        .0
}
#[must_use]
pub fn flags(directory: &Path) -> Vec<std::ffi::OsString> {
    use std::os::unix::ffi::OsStringExt;
    directories(directory).flags().into_vec().into_iter().map(|v| std::ffi::OsString::from_vec(v.into_vec())).collect()
}

/// Replace an owned entry's mode with the demo's, keeping its forge and paths.
pub fn fake_entry(entry: &mut serde_json::Value, listen: &str) {
    entry.as_object_mut().expect("setup entry").remove("orgs");
    entry["mode"] = serde_json::json!({"fake": {
        "listen": {"socket": listen}, "git": "demo/bin/git", "owner": "demo",
        "repositories": [{"name": "widget", "landing": "main"}]
    }});
}

/// A deployment whose forge 1 is the demo's fake, without creating its files.
#[must_use]
pub fn check_document(directory: &Path) -> serde_json::Value {
    let (_, bytes) = documents(directory, "/fixture/temper");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
    value["application"]["setup"] = serde_json::json!([{
        "forge":1, "directory":"forge-1", "bot":{"login":"temper","email":"temper@example.test"}
    }]);
    fake_entry(&mut value["application"]["setup"][0], "127.0.0.1:9");
    value["application"]["forges"][0]["api_token"] = "forge-1/api_token".into();
    value["application"]["forges"][0]["user"] = serde_json::json!({"file":"forge-1/user"});
    value["application"]["identities"][0]["token"] = "forge-1/git_token".into();
    value
}
