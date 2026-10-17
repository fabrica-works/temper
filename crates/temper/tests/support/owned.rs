//! Fixtures for the owned forge commands, shared with the process tier.
use skein_lib::Duration;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use temper_throwaway_forgejo::{RUNNER_VERSION, VERSION};
pub struct Names<'a> {
    pub forgejo: &'a Path,
    pub runner: Option<&'a Path>,
    pub port: u16,
    pub bot: &'a str,
    pub person: &'a str,
    pub orgs: &'a [&'a str],
}
pub fn document(value: &mut serde_json::Value, names: &Names<'_>) {
    let api = format!("127.0.0.1:{}", names.port);
    value["application"]["forges"][0] = serde_json::json!({"number":1,"kind":"forgejo","api":{"socket":api},
        "transport":"plaintext","git":format!("http://{api}"),"api_token":"forge-1/api_token",
        "user":{"file":"forge-1/user"},"identity":1});
    value["application"]["identities"][0]["token"] = "forge-1/git_token".into();
    value["application"]["identities"][0]["login"] = names.bot.into();
    value["application"]["setup"] = serde_json::json!([{"forge":1,"directory":"forge-1",
        "bot":{"login":names.bot,"email":format!("{}@example.invalid", names.bot)},"orgs":names.orgs,
        "mode":{"owned":{"forgejo":names.forgejo,"runner":names.runner.map(|binary| serde_json::json!({"binary":binary,
            "labels":["temper-host:host"],"capacity":1,"path":"/usr/bin:/bin"})),
            "listen":{"socket":api},"person":{"login":names.person,"email":format!("{}@example.invalid", names.person)}}}}]);
}
pub fn configuration(directory: &Path, names: &Names<'_>) {
    super::configuration(directory, &super::Forge::Unreached);
    let path = directory.join("config.json");
    let mut value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    document(&mut value, names);
    super::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
}
#[must_use]
pub fn prepare(directory: &Path, forge: u16) -> Output {
    Command::new(env!("CARGO_BIN_EXE_temper"))
        .args(["forge", "prepare", "config.json", &forge.to_string()])
        .current_dir(directory)
        .output()
        .expect("temper forge prepare")
}
#[must_use]
pub fn files(directory: &Path) -> Vec<PathBuf> {
    ["state", "admin_token", "person_password", "runner/secret", "runner/config.yaml"]
        .iter()
        .map(|name| directory.join("forge-1").join(name))
        .collect()
}
pub struct Programs {
    directory: PathBuf,
}
impl Programs {
    pub fn fail(&self, program: &str, command: &str) {
        fs::write(self.directory.join(format!("{program}.fail")), command).expect("failure");
    }
    pub fn mend(&self, program: &str) {
        fs::remove_file(self.directory.join(format!("{program}.fail"))).expect("mended");
    }
    pub fn hang(&self, program: &str, command: &str) {
        fs::write(self.directory.join(format!("{program}.hang")), command).expect("hang");
    }
    #[must_use]
    pub fn hung(&self, within: Duration) -> u32 {
        let clock = skein_shell::Clock::new();
        let deadline = clock.now().now.saturating_add(within);
        loop {
            if let Ok(pid) = fs::read_to_string(self.directory.join("hung")) {
                return pid.trim().parse().expect("sleep pid");
            }
            assert!(clock.now().now < deadline, "command hung within bound");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
#[must_use]
pub fn scripted(directory: &Path) -> Programs {
    for (program, version) in [
        ("forgejo", format!("forgejo version {VERSION}")),
        ("runner", format!("forgejo-runner version v{RUNNER_VERSION}")),
    ] {
        let body = format!(
            r#"#!/bin/sh
if [ "$1" = --version ]; then echo '{version}'; exit; fi
if [ "$1" = --work-path ]; then shift 4; fi
base='{directory}'
if [ -f "$base/{program}.fail" ] && [ "$1" = "$(cat "$base/{program}.fail")" ]; then echo broken >&2; exit 1; fi
if [ -f "$base/{program}.hang" ] && [ "$1" = "$(cat "$base/{program}.hang")" ]; then sleep 60 & echo $! > "$base/hung"; wait; fi
case "$1 $3" in
    'admin generate-access-token') echo fixture-admin-token;;
    'admin create') echo fixture-person-password;;
    'forgejo-cli register') echo fixture-runner-uuid;;
esac
"#,
            directory = directory.display()
        );
        let path = directory.join(program);
        fs::write(&path, body).expect("program");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable");
    }
    configuration(
        directory,
        &Names {
            forgejo: &directory.join("forgejo"),
            runner: Some(&directory.join("runner")),
            port: 3000,
            bot: "temper-bot",
            person: "ada",
            orgs: &["workshop"],
        },
    );
    Programs { directory: directory.into() }
}
/// Writes a complete pinned preparation with credentials that must never be printed.
pub fn finished(directory: &Path) {
    use skein_lib::Secret;
    use temper_forge_setup::state::{self, Secrets, State};
    let root = directory.join("forge-1");
    fs::create_dir_all(root.join("forgejo")).expect("work path");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("private directory");
    let state = State {
        forgejo: VERSION.as_bytes().into(),
        runner: Some(RUNNER_VERSION.as_bytes().into()),
        prepared: true,
        secrets: Secrets {
            secret_key: Secret::new(b"secret-key-never-print".as_slice().into()),
            internal_token: Secret::new(b"internal-token-never-print".as_slice().into()),
            jwt_secret: Secret::new(b"jwt-secret-never-print".as_slice().into()),
        },
    };
    super::write_private(&root.join("state"), state::encode(&state).expose());
}
