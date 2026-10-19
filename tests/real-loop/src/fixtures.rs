//! The process-tier documents adapted to a real loop on kernel-assigned ports.
use std::{
    io::Write,
    net::SocketAddr,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};
pub const TOKEN: &[u8] = jig_channel_world::fixtures::TOKEN;
pub const SEED: u64 = jig_channel_world::fixtures::SEED;
pub const AGENT: &[u8] = b"/real-loop/temper-agent";
#[must_use]
pub fn documents(directory: &Path, llm: SocketAddr, forge: SocketAddr) -> (serde_json::Value, serde_json::Value) {
    let program = std::str::from_utf8(AGENT).expect("agent program");
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":600,"read_window":256,
            "conversations":1,"calls_per_response":1},
            "policy":{"max_turns":5,"group_stop":10,"inbox":8,"unacknowledged":8}},
        "endpoints":[{"name":"fake","number":0,"dialect":0,"account":1,"provider":"codex",
            "address":llm.to_string(),"transport":"plaintext",
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
    // A minimal positive wait parks the scripted run and settles its withheld turn.
    value["engine"]["charters"][0]["waiting_ms"] = 1.into();
    // The close story exercises the grace transition without a five-second real wait.
    value["engine"]["templates"][0]["grace_ms"] = 1.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["connectors"] = serde_json::json!([1]);
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":llm.to_string()});
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["documents.rows"] = 2.into();
    value["engine"]["limits"]["overrides"]["documents.lines"] = 2.into();
    value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 128.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    value["engine"]["limits"]["overrides"]["views.watchers"] = 16.into();
    value["engine"]["limits"]["overrides"]["views.watchers_per_party"] = 16.into();
    value["application"] = serde_json::json!({
        "webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:0"},"bundle":null},
        "forges":[{"number":1,"kind":"forgejo","api":{"socket":forge.to_string()},"transport":"plaintext",
            "git":format!("http://{forge}"),"webhook_secret":null,"api_token":"secrets/forge-1.api-token","user":{"file":"secrets/forge-1.user"},"identity":1}],
        "identities":[{"number":1,"token":"secrets/forge-1.git-token","login":"temper",
            "author":{"name":"Temper","email":"temper@example.test"}}],
        "setup":[],"landing":{"deployment":[],"projects":[]},"charter":1,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,
            "checkout.message_bytes":4096,"forge.name_bytes":64,"forge.resources_per_task":2,"git_component.stdout_bytes":2048}},
        "workers":[{"root":"worker-root","git":"/usr/bin/git"}]});
    (agent, value)
}

/// Create the standalone tree; every private directory and file has its final mode.
pub fn write(directory: &Path, llm: SocketAddr, forge: SocketAddr) -> std::io::Result<()> {
    for name in ["", "accounts", "accounts/records", "secrets", "charters", "state", "clients"] {
        std::fs::create_dir_all(directory.join(name))?;
        std::fs::set_permissions(directory.join(name), std::fs::Permissions::from_mode(0o700))?;
    }
    for (name, bytes) in jig_channel_world::fixtures::files(false).entries.into_boxed() {
        private(&directory.join(std::str::from_utf8(&name).expect("fixture path")), bytes.expose())?;
    }
    private(&directory.join("worker.secret"), b"worker-secret")?;
    private(
        &directory.join("accounts/chat.json"),
        &serde_json::to_vec(&serde_json::json!({
            "access_token":std::str::from_utf8(TOKEN).expect("token"), "expires_at":4_102_444_800_u64,"account_id":"acc"
        }))?,
    )?;
    private(&directory.join("secrets/forge-1.user"), b"42\n")?;
    private(&directory.join("secrets/forge-1.api-token"), b"forge-1-api-token-value")?;
    private(&directory.join("secrets/forge-1.git-token"), b"forge-1-git-token-value")?;
    let (agent, config) = documents(directory, llm, forge);
    private(&directory.join("agent.json"), &serde_json::to_vec(&agent)?)?;
    private(&directory.join("config.json"), &serde_json::to_vec(&config)?)
}
pub(crate) fn private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path)?.write_all(bytes)
}

/// The tools' executable is hosted once per call, with two concurrent slots.
pub const CTL: &[u8] = b"/real-loop/temper-ctl";
pub const CALLS_AT_ONCE: u32 = 2;
#[must_use]
pub fn directory(scratch: &Path, at: crate::calls::Dir) -> std::path::PathBuf {
    scratch.join("clients").join(format!("client-{}", at.0))
}
pub(crate) fn ctl_roots(spawn: &skein_io::kernel::Spawn) -> Vec<skein_world::StartupRoot> {
    use std::os::unix::ffi::OsStrExt;
    let directory = Path::new(std::ffi::OsStr::from_bytes(&spawn.args[0]));
    let mut roots = vec![skein_world::StartupRoot {
        name: b"parent".as_slice().into(),
        path: directory.parent().expect("client parent").as_os_str().as_bytes().into(),
    }];
    let parsed =
        temper_client_shell_cli::arguments::parse(&spawn.args[1..].iter().map(AsRef::as_ref).collect::<Vec<_>>())
            .expect("story arguments");
    if let Some(path) = temper_client_shell_cli::arguments::file_root(&parsed) {
        roots.push(skein_world::StartupRoot { name: b"file_parent".as_slice().into(), path });
    }
    roots
}
pub(crate) fn ctl(spawn: &skein_io::kernel::Spawn, inherited: &skein_world::Inherited) -> crate::world::Proc {
    use std::os::unix::ffi::OsStrExt;
    use temper_client_shell_cli::{
        arguments,
        host::{Invocation, OneShot, Startup},
    };
    let directory = Path::new(std::ffi::OsStr::from_bytes(&spawn.args[0]));
    let pipe = |child| inherited.pipes.iter().find(|(number, _)| *number == child).expect("standard pipe").1;
    crate::world::Proc::Ctl(Box::new(
        OneShot::new(
            Invocation {
                directory: Some(directory.file_name().expect("directory leaf").as_bytes().into()),
                parsed: arguments::parse(&spawn.args[1..].iter().map(AsRef::as_ref).collect::<Vec<_>>())
                    .expect("story arguments"),
            },
            Startup {
                parent: Some(inherited.roots[0].1),
                file_parent: inherited
                    .roots
                    .iter()
                    .find(|(name, _)| name.as_ref() == b"file_parent")
                    .map(|(_, fd)| *fd),
                stdin: pipe(0),
                stdout: pipe(1),
                stderr: pipe(2),
                signals: inherited.signal,
                user: skein_shell::effective_user(),
                seed: SEED,
            },
        )
        .expect("shipped tools limits"),
    ))
}

/// The project supplied by the standalone engine fixture.
pub const PROJECT: &str = "1";

/// A signed-in chat is the common opener for tools stories.
#[must_use]
pub fn signed_chat(key: &'static str) -> Vec<crate::calls::Step> {
    use crate::{
        argv,
        calls::{D1, call, sign_in},
        scripts,
    };
    vec![sign_in(D1), call(D1, argv!["chat", "--project", PROJECT, "--key", key, scripts::WORDS])]
}

/// Both directories ask for the same named request.
pub const LOST_NAME: &str = "lost-answer";
/// Sign-in is call zero, so the started request is call one.
pub const LOST_CALL: usize = 1;

/// Stop a sending call only after its named request is durably held.
#[must_use]
pub fn lost_chat() -> Vec<crate::calls::Step> {
    use crate::calls::{Step, held, kill, start};
    let mut steps = signed_chat(LOST_NAME);
    let Step::Call { at, argv } = steps.pop().expect("chat opener") else { panic!("chat call") };
    steps.extend([start(at, argv), held(LOST_NAME.as_bytes()), kill(LOST_CALL)]);
    steps
}
