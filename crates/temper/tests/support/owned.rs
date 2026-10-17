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
pub fn run(directory: &Path, program: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_temper"));
    command.args(["forge", "run", "config.json", "1", program]).current_dir(directory);
    command
}
/// A process proof's child, reaped even when an assertion fails.
pub struct Running {
    child: std::process::Child,
    log: PathBuf,
}
impl Running {
    #[must_use]
    pub fn start(directory: &Path, program: &str) -> Self {
        let log = directory.join(format!("{program}.log"));
        let output = fs::File::create(&log).expect("log");
        let child =
            run(directory, program).stdout(output.try_clone().expect("log clone")).stderr(output).spawn().expect("run");
        Self { child, log }
    }
    pub fn declared(&mut self) {
        let clock = skein_shell::Clock::new();
        let deadline = clock.now().now.saturating_add(Duration::from_secs(30));
        loop {
            let text = fs::read_to_string(&self.log).expect("runner log");
            if text.contains("declared successfully") {
                return;
            }
            assert!(self.child.try_wait().expect("status").is_none(), "runner exited: {text}");
            assert!(clock.now().now < deadline, "runner did not declare: {text}");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    pub fn stop(mut self) {
        self.kill();
    }
    fn kill(&mut self) {
        let _ = self.child.kill();
        self.child.wait().expect("reaped");
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
            bot: "temper-bot",
            person: "ada",
            orgs: &["workshop"],
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
    } else {
        run(directory, "forgejo").output().expect("run refuses")
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
