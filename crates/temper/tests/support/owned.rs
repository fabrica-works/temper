//! Fixtures for the owned forge commands, shared with the process tier.
use skein_lib::Duration;
use std::{
    fs,
    net::SocketAddr,
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
    let mut owned = Owned::example();
    owned.forgejo = ([127, 0, 0, 1], names.port).into();
    owned.forgejo_binary = names.forgejo.into();
    let mut members = application(&owned);
    members["forges"][0]["webhook_secret"] = serde_json::Value::Null;
    members["webhooks"] = serde_json::Value::Null;
    members["identities"][0]["login"] = names.bot.into();
    let setup = &mut members["setup"][0];
    setup["bot"] = serde_json::json!({"login":names.bot,"email":format!("{}@example.invalid", names.bot)});
    setup["orgs"] = serde_json::json!(names.orgs);
    let mode = &mut setup["mode"]["owned"];
    mode["person"] = serde_json::json!({"login":names.person,"email":format!("{}@example.invalid", names.person)});
    if let Some(binary) = names.runner {
        mode["runner"]["binary"] = serde_json::json!(binary);
    } else {
        mode["runner"] = serde_json::Value::Null;
    }
    for name in ["forges", "identities", "setup"] {
        value["application"][name] = members[name].take();
    }
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
            // The shell creates the file before it writes the pid: an empty file is not yet an answer.
            if let Some(pid) =
                fs::read_to_string(self.directory.join("hung")).ok().and_then(|text| text.trim().parse().ok())
            {
                return pid;
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
if [ "$1" = web ] || [ "$3" = daemon ] || [ "$5" = web ]; then
    {{ echo $$; printf '%s\n' "$@"; echo ENVIRONMENT; env; }} > '{directory}/{program}.marker'
    exec sleep 60
fi
if [ "$1" = --work-path ]; then shift 4; fi
base='{directory}'
if [ -f "$base/{program}.fail" ] && [ "$1" = "$(cat "$base/{program}.fail")" ]; then echo broken >&2; exit 1; fi
if [ -f "$base/{program}.hang" ] && [ "$1" = "$(cat "$base/{program}.hang")" ]; then sleep 60 & echo $! > "$base/hung"; wait; fi
case "$1 $3" in
    'admin generate-access-token') echo fixture-admin-token;;
    'admin create') echo "generated random password is 'fixture-person-password'";;
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
            bot: "temper",
            person: "ada",
            orgs: &["acme"],
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

/// Whether the process is blocked in its `sleep`. The kernel names the process `sleep` as the exec begins, and
/// the dynamic loader holds a descriptor (3) until it has mapped the libraries, so only a process asleep has
/// closed every descriptor the exec and its startup opened.
fn asleep(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .is_ok_and(|stat| stat.rsplit_once(") ").is_some_and(|(_, rest)| rest.starts_with("S ")))
}

pub struct Marker {
    pub pid: u32,
    pub arguments: Vec<String>,
    pub environment: Vec<String>,
    pub descriptors: Vec<u32>,
}
impl Programs {
    #[must_use]
    pub fn lock(&self, program: &str) -> PathBuf {
        self.directory.join(format!("forge-1/{program}.lock"))
    }
    pub fn marker(&self, program: &str, within: Duration) -> Marker {
        let clock = skein_shell::Clock::new();
        let deadline = clock.now().now.saturating_add(within);
        loop {
            if let Ok(text) = fs::read_to_string(self.directory.join(format!("{program}.marker"))) {
                let mut lines = text.lines();
                if let Some(pid) = lines.next().and_then(|line| line.parse::<u32>().ok())
                    && fs::read_to_string(format!("/proc/{pid}/comm")).is_ok_and(|name| name.trim() == "sleep")
                    && asleep(pid)
                {
                    let arguments =
                        lines.by_ref().take_while(|line| *line != "ENVIRONMENT").map(String::from).collect();
                    let environment = lines.map(String::from).collect();
                    let mut descriptors: Vec<_> = fs::read_dir(format!("/proc/{pid}/fd"))
                        .expect("descriptors")
                        .map(|entry| {
                            entry.expect("fd").file_name().to_str().expect("number").parse().expect("fd number")
                        })
                        .collect();
                    descriptors.sort_unstable();
                    return Marker { pid, arguments, environment, descriptors };
                }
            }
            assert!(clock.now().now < deadline, "process execs sleep within bound");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}
#[must_use]
pub fn command(directory: &Path, program: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_temper"));
    command.args(["forge", "run", "config.json", "1", program]).current_dir(directory);
    command
}
/// A process proof's child, reaped even when an assertion fails.
pub struct Running {
    child: Option<std::process::Child>,
    log: PathBuf,
}
impl Running {
    #[must_use]
    pub fn start(directory: &Path, program: &str) -> Self {
        let log = directory.join(if program == "forgejo" { "server.log" } else { "runner.log" });
        let output = fs::File::create(&log).expect("log");
        let child = command(directory, program)
            .stdout(output.try_clone().expect("log clone"))
            .stderr(output)
            .spawn()
            .expect("run");
        Self { child: Some(child), log }
    }
    #[must_use]
    pub fn into_child(mut self) -> std::process::Child {
        self.child.take().expect("owned child")
    }
    pub fn declared(&mut self) {
        let clock = skein_shell::Clock::new();
        let deadline = clock.now().now.saturating_add(Duration::from_secs(30));
        loop {
            let text = fs::read_to_string(&self.log).expect("runner log");
            if text.contains("declared successfully") {
                return;
            }
            assert!(
                self.child.as_mut().expect("owned child").try_wait().expect("status").is_none(),
                "runner exited: {text}"
            );
            assert!(clock.now().now < deadline, "runner did not declare: {text}");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    pub fn stop(mut self) {
        self.kill();
    }
    fn kill(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            child.wait().expect("reaped");
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        self.kill();
    }
}
pub fn answer(port: u16, path: &str, token: Option<&str>) -> std::io::Result<serde_json::Value> {
    use std::io::{Read, Write};
    let mut socket =
        std::net::TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), std::time::Duration::from_millis(100))?;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;
    let auth = token.map_or_else(String::new, |token| format!("Authorization: token {token}\r\n"));
    write!(socket, "GET {path} HTTP/1.0\r\nHost: 127.0.0.1\r\n{auth}Connection: close\r\n\r\n")?;
    let mut text = String::new();
    socket.take(1 << 20).read_to_string(&mut text)?;
    let (head, body) = text.split_once("\r\n\r\n").ok_or(std::io::ErrorKind::InvalidData)?;
    if !head.starts_with("HTTP/1.0 200 ") && !head.starts_with("HTTP/1.1 200 ") {
        return Err(std::io::ErrorKind::InvalidData.into());
    }
    serde_json::from_str(body).map_err(std::io::Error::other)
}
#[must_use]
pub fn await_answer(port: u16, path: &str, token: Option<&str>) -> serde_json::Value {
    let clock = skein_shell::Clock::new();
    let deadline = clock.now().now.saturating_add(Duration::from_secs(30));
    loop {
        if let Ok(value) = answer(port, path, token) {
            return value;
        }
        assert!(clock.now().now < deadline, "{path} did not answer");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
#[must_use]
pub fn prepared_processes(directory: &Path) -> u16 {
    let forgejo = std::env::var_os("TEMPER_FORGEJO").expect("TEMPER_FORGEJO");
    let runner = std::env::var_os("TEMPER_FORGEJO_RUNNER").expect("TEMPER_FORGEJO_RUNNER");
    let port = std::net::TcpListener::bind("127.0.0.1:0").expect("free port").local_addr().expect("address").port();
    configuration(
        directory,
        &Names {
            forgejo: Path::new(&forgejo),
            runner: Some(Path::new(&runner)),
            port,
            bot: "temper",
            person: "ada",
            orgs: &["acme"],
        },
    );
    let prepared = prepare(directory, 1);
    assert_eq!(prepared.status.code(), Some(0), "{}", String::from_utf8_lossy(&prepared.stderr));
    port
}

/// The setup's typed file under a numbered forge directory.
#[must_use]
pub fn file(directory: &Path, forge: u16, file: temper_forge_setup::place::File) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    let place = temper_forge_setup::place::Place {
        directory: directory.join(format!("forge-{forge}")).as_os_str().as_bytes().into(),
        user: Box::new([]),
    };
    PathBuf::from(std::ffi::OsStr::from_bytes(&place.path(file)))
}
pub fn record_versions(directory: &Path, forgejo: Option<&[u8]>, runner: Option<&[u8]>) {
    let path = file(directory, 1, temper_forge_setup::place::File::State);
    let mut state = temper_forge_setup::state::decode(&fs::read(&path).expect("state")).expect("record");
    if let Some(version) = forgejo {
        state.forgejo = version.into();
    }
    if let Some(version) = runner {
        state.runner = Some(version.into());
    }
    super::write_private(&path, temper_forge_setup::state::encode(&state).expose());
}
pub fn recorded_without_runner(directory: &Path) {
    let path = file(directory, 1, temper_forge_setup::place::File::State);
    let mut state = temper_forge_setup::state::decode(&fs::read(&path).expect("state")).expect("record");
    state.runner = None;
    super::write_private(&path, temper_forge_setup::state::encode(&state).expose());
}
pub fn setup_without_runner(directory: &Path) {
    let path = directory.join("config.json");
    let mut config: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("config")).expect("JSON");
    config["application"]["setup"][0]["mode"]["owned"]["runner"] = serde_json::Value::Null;
    super::write_private(&path, &serde_json::to_vec(&config).expect("JSON"));
}
pub fn refused(directory: &Path, verb: &str, expected: &[u8]) {
    let result = if verb == "prepare" {
        prepare(directory, 1)
    } else if verb == "reconcile" {
        reconcile(directory, 1)
    } else {
        command(directory, "forgejo").output().expect("run refuses")
    };
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(result.stderr, expected);
}

/// Checks the dump archive's recorded evidence and keeps its bytes and timestamp for a repeated upgrade.
#[must_use]
pub fn dump(directory: &Path) -> (PathBuf, Vec<u8>, fs::Metadata) {
    let backups: Vec<_> = fs::read_dir(directory.join("forge-1/backups"))
        .expect("backups")
        .map(|entry| entry.expect("backup entry").path())
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = backups[0].clone();
    assert_eq!(backup.file_name().expect("backup name"), format!("forgejo-16.0.5-to-{VERSION}.zip").as_str());
    let bytes = fs::read(&backup).expect("backup");
    let meta = fs::metadata(&backup).expect("metadata");
    assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    assert!(bytes.starts_with(b"PK"));
    assert!(skein_lib::bytes::find(&bytes, b"forgejo-db.sql").is_some());
    assert!(skein_lib::bytes::find(&bytes, b"app.ini").is_some());
    (backup, bytes, meta)
}
/// Starts one owned process with its output retained for a failing proof.
#[must_use]
pub fn run(directory: &Path, program: &str) -> Running {
    Running::start(directory, program)
}
/// Prepares and starts, leaving readiness to reconcile's first exchange.
#[must_use]
pub fn start(directory: &Path) -> Running {
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let prepared = prepare(directory, 1);
    assert_eq!(prepared.status.code(), Some(0), "{}", String::from_utf8_lossy(&prepared.stderr));
    eprintln!("stage prepare: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    let began = clock.now().now;
    let running = run(directory, "forgejo");
    eprintln!("stage start: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    running
}
#[must_use]
pub fn reconcile(directory: &Path, forge: u16) -> Output {
    Command::new(env!("CARGO_BIN_EXE_temper"))
        .args(["forge", "reconcile", "config.json", &forge.to_string()])
        .current_dir(directory)
        .output()
        .expect("temper forge reconcile")
}
/// Attaches the read-only proof handle with preparation's private admin token.
#[must_use]
pub fn attach(directory: &Path, port: u16, running: Running) -> temper_throwaway_forgejo::Instance {
    use temper_throwaway_forgejo::{Instance, Settings, Token};
    let token = fs::read_to_string(directory.join("forge-1/admin_token")).expect("admin token");
    Instance::attach(running.into_child(), port, directory, Token::new(token.trim().into()), &Settings::small())
        .expect("owned forge answers")
}
/// Reads the converged bot, its id, and its private organization's team.
pub fn assert_converged(directory: &Path, instance: &temper_throwaway_forgejo::Instance) {
    use temper_throwaway_forgejo::{Outgoing, Token};
    let api = fs::read_to_string(directory.join("forge-1/api_token")).expect("API token");
    let token = Token::new(api.trim().into());
    let response = instance.exchange(&Outgoing::get("/api/v1/user", Some(&token))).expect("bot answers");
    assert_eq!(response.status, 200);
    let bot: serde_json::Value = serde_json::from_slice(&response.body).expect("bot JSON");
    assert_eq!(bot["login"], "temper");
    let id = fs::read_to_string(directory.join("forge-1/user")).expect("user").trim().parse::<u64>().expect("id");
    assert_eq!(bot["id"], id);
    let admin = &instance.admin().token;
    let account = instance.exchange(&Outgoing::get("/api/v1/users/temper", Some(admin))).expect("account answers");
    let account: serde_json::Value = serde_json::from_slice(&account.body).expect("account JSON");
    assert_eq!(account["is_admin"], false);
    assert!(account["full_name"].as_str().expect("full name").contains("temper:bot"));
    let org = instance.exchange(&Outgoing::get("/api/v1/orgs/acme", Some(admin))).expect("org answers");
    let org: serde_json::Value = serde_json::from_slice(&org.body).expect("org JSON");
    assert_eq!(org["visibility"], "private");
    let teams = instance.exchange(&Outgoing::get("/api/v1/orgs/acme/teams", Some(admin))).expect("teams answer");
    let teams: serde_json::Value = serde_json::from_slice(&teams.body).expect("teams JSON");
    assert!(teams.as_array().expect("teams").iter().any(|team| team["name"] == "temper"));
}
/// A managed forge whose directory reconcile must make itself.
pub fn managed(directory: &Path) {
    configuration(
        directory,
        &Names {
            forgejo: Path::new("/usr/bin/false"),
            runner: None,
            port: 9,
            bot: "temper",
            person: "ada",
            orgs: &["acme"],
        },
    );
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["setup"][0]["mode"] = serde_json::json!({"managed":{"admin_token":"admin_token"}});
    value["application"]["limits"]["overrides"]["setup.deadline_ms"] = 300.into();
    value["application"]["limits"]["overrides"]["setup.backoff_ms"] = 20.into();
    value["application"]["limits"]["overrides"]["setup.backoff_max_ms"] = 50.into();
    super::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
    super::write_private(&directory.join("admin_token"), b"managed-admin-token");
}
pub fn configuration_value(directory: &Path, change: fn(&mut serde_json::Value)) {
    let path = directory.join("config.json");
    let mut value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    change(&mut value);
    super::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
}

/// A minimal owned document for the reconcile proof, with no runner.
#[must_use]
pub fn reconcile_configuration(directory: &Path) -> u16 {
    let binary = std::env::var_os("TEMPER_FORGEJO").expect("TEMPER_FORGEJO");
    let port = std::net::TcpListener::bind("127.0.0.1:0").expect("free port").local_addr().expect("address").port();
    configuration(
        directory,
        &Names { forgejo: Path::new(&binary), runner: None, port, bot: "temper", person: "ada", orgs: &["acme"] },
    );
    port
}

/// Both pinned executables are required before the process tier starts anything.
#[must_use]
pub fn binaries() -> (PathBuf, PathBuf) {
    let forgejo = std::env::var_os("TEMPER_FORGEJO").expect("TEMPER_FORGEJO must name the pinned Forgejo");
    let runner = std::env::var_os("TEMPER_FORGEJO_RUNNER").expect("TEMPER_FORGEJO_RUNNER must name the pinned runner");
    (forgejo.into(), runner.into())
}
/// Writes the process tier's owned setup and engine configuration without starting a process.
#[must_use]
pub fn tier_configuration(directory: &Path) -> std::net::SocketAddr {
    let owned = Owned::tier();
    super::configuration(
        directory,
        &super::Forge::Throwaway {
            forgejo: &owned.forgejo_binary,
            api: owned.forgejo,
            git: format!("http://{}", owned.forgejo),
        },
    );
    owned.forgejo
}

/// The complete owned deployment and its non-secret files, kept with the guide.
pub const EXAMPLE: &[u8] = include_bytes!("../../../../docs/examples/owned.json");
pub const SHIPPED: [(&str, &[u8]); 3] = [
    ("owned/agent.json", include_bytes!("../../../../docs/examples/owned/agent.json")),
    ("owned/charters/chat.txt", include_bytes!("../../../../docs/examples/owned/charters/chat.txt")),
    ("owned/charters/coding.txt", include_bytes!("../../../../docs/examples/owned/charters/coding.txt")),
];
/// The example's three listeners, provider, binaries and optional agent review.
pub struct Owned {
    pub forgejo: SocketAddr,
    pub clients: SocketAddr,
    pub webhooks: SocketAddr,
    pub llm: SocketAddr,
    pub forgejo_binary: PathBuf,
    pub runner_binary: PathBuf,
    pub review: bool,
}
impl Owned {
    #[must_use]
    pub fn example() -> Self {
        Self {
            forgejo: "127.0.0.1:3000".parse().expect("example Forgejo"),
            clients: "127.0.0.1:7100".parse().expect("example clients"),
            webhooks: "127.0.0.1:7101".parse().expect("example webhooks"),
            llm: "127.0.0.1:34443".parse().expect("fake peer"),
            forgejo_binary: "/opt/forgejo/forgejo".into(),
            runner_binary: "/opt/forgejo/forgejo-runner".into(),
            review: true,
        }
    }
    #[must_use]
    pub fn tier() -> Self {
        let (forgejo_binary, runner_binary) = binaries();
        Self {
            forgejo: super::webhooks::free_address(),
            clients: super::webhooks::free_address(),
            webhooks: super::webhooks::free_address(),
            forgejo_binary,
            runner_binary,
            review: false,
            ..Self::example()
        }
    }
}
fn edited(bytes: &[u8], owned: &Owned) -> String {
    std::str::from_utf8(bytes)
        .expect("example UTF-8")
        .replace("127.0.0.1:3000", &owned.forgejo.to_string())
        .replace("127.0.0.1:7100", &owned.clients.to_string())
        .replace("127.0.0.1:7101", &owned.webhooks.to_string())
        .replace("chatgpt.com:443", &owned.llm.to_string())
        .replace("/opt/forgejo/forgejo-runner", owned.runner_binary.to_str().expect("runner path"))
        .replace("/opt/forgejo/forgejo", owned.forgejo_binary.to_str().expect("Forgejo path"))
}
fn example_value(owned: &Owned) -> serde_json::Value {
    let mut value: serde_json::Value = serde_json::from_str(&edited(EXAMPLE, owned)).expect("owned JSON");
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":owned.llm.to_string()});
    if !owned.review {
        value["application"]["landing"]["projects"][0]["rules"][0]["gates"] = serde_json::json!([]);
    }
    value
}
#[must_use]
pub fn application(owned: &Owned) -> serde_json::Value {
    let mut value = example_value(owned)["application"].take();
    value["charter"] = 1.into();
    value
}
#[must_use]
pub fn example_document(directory: &Path, owned: &Owned) -> Vec<u8> {
    serde_json::to_string(&example_value(owned))
        .expect("example JSON")
        .replace("/home/me/temper", directory.to_str().expect("deployment path"))
        .replace("/usr/local/bin/temper", env!("CARGO_BIN_EXE_temper"))
        .into_bytes()
}
/// The guide's copied files and private credentials, with the fake peer's account.
pub fn lay(directory: &Path, owned: &Owned) {
    for name in ["owned/charters", "secrets", "accounts/records"] {
        let path = directory.join(name);
        fs::create_dir_all(&path).expect("example directory");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private directory");
    }
    for (name, bytes) in SHIPPED {
        let text = edited(bytes, owned).replace("/home/me/temper", directory.to_str().expect("deployment path"));
        fs::write(directory.join(name), text).expect("shipped file");
    }
    let path = directory.join("owned/agent.json");
    let mut agent: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("agent")).expect("agent JSON");
    agent["endpoints"][0]["server_name"] = "skein.test".into();
    agent["endpoints"][0]["trust_der"] = serde_json::json!(directory.join("root.der"));
    fs::write(path, serde_json::to_vec(&agent).expect("agent JSON")).expect("fake agent");
    super::write_private(&directory.join("root.der"), skein_tls_world::pki::ROOT);
    super::write_private(&directory.join("secrets/worker.secret"), b"worker-secret\n");
    super::write_private(
        &directory.join("secrets/worker.digest"),
        b"6fb46f7a92742970166379ed5195e79c4493a7cc5664280c039cfd4095ba5faf\n",
    );
    super::write_private(
        &directory.join("accounts/llm.json"),
        br#"{"access_token":"token","expires_at":4102444800,"account_id":"acc"}"#,
    );
}
