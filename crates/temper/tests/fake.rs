//! The QA runner starts the shipped binary, then reads files and serves requests.
pub mod support;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::Duration,
};

fn command(kind: &str, document: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_temper"));
    command
        .env_clear()
        .args(["fake", kind, "--document"])
        .arg(document)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}
fn llm_document(directory: &Path, tls: bool) -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"key":directory.join("key"),"tls":if tls {json!({"root":directory.join("root.der")})}else{Value::Null},
        "models":[{"name":"qa","window":4096,"output":1024}],"cued":[],
        "asked":[{"cue":"say hello","turns":["hello from QA"]}],"otherwise":["fallback"]})
}
fn forge_document(directory: &Path) -> Value {
    json!({"listen":{"socket":"127.0.0.1:0"},"admin_token":directory.join("forge/admin_token"),"owner":"fake",
        "repositories":[{"name":"widget","landing":"main","ci":null}]})
}
fn document(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).expect("valid fixture")).expect("valid fixture");
}
struct Running {
    child: Option<Child>,
    address: SocketAddr,
}
impl Running {
    fn start(kind: &str, document: &Path, observations: Option<&Path>) -> Self {
        let mut command = command(kind, document);
        command.args(["--ready", "ready ", "--seed", "23"]);
        if let Some(path) = observations {
            command.arg("--observations").arg(path);
        }
        let child = command.spawn().expect("valid fixture");
        let mut run = Self { child: Some(child), address: "127.0.0.1:0".parse().expect("valid fixture") };
        let mut bytes = Vec::new();
        let stdout = run.child.as_mut().expect("running fake").stdout.as_mut().expect("stdout pipe");
        for _ in 0..128 {
            let mut byte = [0_u8; 1];
            stdout.read_exact(&mut byte).expect("ready line");
            bytes.push(byte[0]);
            if byte[0] == b'\n' {
                break;
            }
        }
        let line = String::from_utf8(bytes).expect("ready line is UTF-8");
        assert!(line.starts_with("ready 127.0.0.1:"), "ready line: {line}");
        run.address = line.strip_prefix("ready ").expect("valid fixture").trim_end().parse().expect("valid fixture");
        assert_ne!(run.address.port(), 0);
        run
    }
    fn stop(mut self) {
        let child = self.child.take().expect("valid fixture");
        assert!(
            Command::new("/bin/kill")
                .args(["-TERM", &child.id().to_string()])
                .status()
                .expect("valid fixture")
                .success()
        );
        let ended = child.wait_with_output().expect("valid fixture");
        assert_eq!(ended.status.code(), Some(0), "{}", String::from_utf8_lossy(&ended.stderr));
        assert!(ended.stdout.is_empty(), "stdout holds only the ready line");
        assert!(ended.stderr.is_empty(), "{}", String::from_utf8_lossy(&ended.stderr));
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _killed = child.kill();
            let _ended = child.wait();
        }
    }
}
fn stream(address: SocketAddr) -> TcpStream {
    let stream = TcpStream::connect_timeout(&address, Duration::from_millis(300)).expect("valid fixture");
    stream.set_read_timeout(Some(Duration::from_millis(300))).expect("valid fixture");
    stream.set_write_timeout(Some(Duration::from_millis(300))).expect("valid fixture");
    stream
}
fn http(method: &str, target: &str, authorization: &str, body: &[u8]) -> Vec<u8> {
    let mut request=format!("{method} {target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",body.len()).into_bytes();
    if !authorization.is_empty() {
        request.extend_from_slice(format!("Authorization: {authorization}\r\n").as_bytes());
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(body);
    request
}
fn request(address: SocketAddr, method: &str, target: &str, authorization: &str, body: &[u8]) -> Vec<u8> {
    let mut stream = stream(address);
    stream.write_all(&http(method, target, authorization, body)).expect("valid fixture");
    let mut response = Vec::new();
    stream.read_to_end(&mut response).expect("valid fixture");
    response
}
fn body(response: &[u8]) -> Value {
    let boundary = response.windows(4).position(|part| part == b"\r\n\r\n").expect("valid fixture") + 4;
    serde_json::from_slice(&response[boundary..]).expect("valid fixture")
}
fn private_hex(path: &Path, expected: &[u8]) {
    let bytes = fs::read(path).expect("valid fixture");
    assert_eq!(bytes.len(), 40);
    assert!(bytes.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
    assert_eq!(bytes, expected);
    assert_eq!(fs::metadata(path).expect("valid fixture").permissions().mode() & 0o777, 0o600);
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table covers refused startup and rollback for every output")]
fn refused_start() {
    struct Row {
        name: &'static str,
        kind: &'static str,
        document: Option<Value>,
        files: &'static [&'static str],
        directories: &'static [(&'static str, u32)],
        observations: Option<&'static str>,
        path: Option<&'static str>,
        reason: &'static str,
    }
    let scratch = support::Scratch::new("fake_refused");
    let llm = |name| Some(llm_document(&scratch.path().join(name), true));
    let forge = |name| Some(forge_document(&scratch.path().join(name)));
    let rows = [
        Row {
            name: "key",
            kind: "llm",
            document: llm("key"),
            files: &["key"],
            directories: &[],
            observations: None,
            path: Some("key"),
            reason: "File exists (os error 17)",
        },
        Row {
            name: "admin_token",
            kind: "forge",
            document: forge("admin_token"),
            files: &["forge/admin_token"],
            directories: &[("forge", 0o700)],
            observations: None,
            path: Some("forge/admin_token"),
            reason: "File exists (os error 17)",
        },
        Row {
            name: "root",
            kind: "llm",
            document: llm("root"),
            files: &["root.der"],
            directories: &[],
            observations: None,
            path: Some("root.der"),
            reason: "File exists (os error 17)",
        },
        Row {
            name: "observations",
            kind: "forge",
            document: forge("observations"),
            files: &["observations"],
            directories: &[],
            observations: Some("observations"),
            path: Some("observations"),
            reason: "File exists (os error 17)",
        },
        Row {
            name: "directory",
            kind: "forge",
            document: forge("directory"),
            files: &[],
            directories: &[("forge", 0o755)],
            observations: None,
            path: Some("forge"),
            reason: "must be a private directory of this user's, not reached through a link",
        },
        Row {
            name: "missing",
            kind: "llm",
            document: None,
            files: &[],
            directories: &[],
            observations: None,
            path: Some("document.json"),
            reason: "absent",
        },
        Row {
            name: "refused",
            kind: "llm",
            document: Some(json!({})),
            files: &[],
            directories: &[],
            observations: None,
            path: None,
            reason: "listen: missing",
        },
    ];
    for row in rows {
        let dir = scratch.path().join(row.name);
        fs::create_dir(&dir).expect("case directory");
        let doc = dir.join("document.json");
        if let Some(value) = row.document {
            document(&doc, &value);
        }
        for (name, mode) in row.directories {
            fs::create_dir(dir.join(name)).expect("existing directory");
            fs::set_permissions(dir.join(name), fs::Permissions::from_mode(*mode)).expect("directory mode");
        }
        for name in row.files {
            fs::write(dir.join(name), b"existing").expect("existing output");
        }
        let mut command = command(row.kind, &doc);
        if let Some(path) = row.observations {
            command.arg("--observations").arg(dir.join(path));
        }
        let before = snapshot(&dir);
        let output = command.output().expect("refused fake");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let location = row.path.map_or_else(String::new, |path| format!("{}: ", dir.join(path).display()));
        let expected = format!("temper fake {}: {location}{}\n", row.kind, row.reason);
        assert_eq!(String::from_utf8(output.stderr).expect("UTF-8 refusal"), expected, "{}", row.name);
        assert_eq!(snapshot(&dir), before, "{}: refused startup restores the files", row.name);
    }
    scratch.remove();
}

fn snapshot(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, (u32, Option<Vec<u8>>)> {
    let mut found = std::collections::BTreeMap::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).expect("valid fixture") {
            let entry = entry.expect("valid fixture");
            let meta = entry.metadata().expect("valid fixture");
            let path = entry.path();
            let bytes = if meta.is_dir() {
                directories.push(path.clone());
                None
            } else {
                Some(fs::read(&path).expect("valid fixture"))
            };
            found.insert(path, (meta.permissions().mode() & 0o777, bytes));
        }
    }
    found
}
#[test]
fn forge_run() {
    let scratch = support::Scratch::new("fake_forge_run");
    let dir = scratch.path();
    let doc = dir.join("forge.json");
    let mut config = forge_document(dir);
    config["repositories"][0]["landing"] = json!("trunk");
    document(&doc, &config);
    let observations = dir.join("observations");
    let run = Running::start("forge", &doc, Some(&observations));
    let token = temper_fake::credential::credential(23, temper_fake::credential::ADMIN_TOKEN);
    private_hex(&dir.join("forge/admin_token"), &token);
    assert_eq!(fs::metadata(dir.join("forge")).expect("valid fixture").permissions().mode() & 0o777, 0o700);
    assert_eq!(fs::metadata(&observations).expect("valid fixture").permissions().mode() & 0o777, 0o600);
    assert!(fs::read(&observations).expect("valid fixture").is_empty(), "no seed observations");
    let auth = format!("token {}", std::str::from_utf8(&token).expect("valid fixture"));
    let user = request(run.address, "GET", "/api/v1/user", &auth, b"");
    assert!(user.starts_with(b"HTTP/1.1 200"));
    assert_eq!(body(&user)["login"], "temper-admin");
    let repository = request(run.address, "GET", "/api/v1/repositories/1", &auth, b"");
    assert!(repository.starts_with(b"HTTP/1.1 200"));
    let repository = body(&repository);
    for (member, expected) in
        [("default_branch", "trunk".to_owned()), ("html_url", format!("http://{}/fake/widget", run.address))]
    {
        assert_eq!(repository[member], expected, "{member}");
    }
    let objects = request(run.address, "POST", "/.fake/git/objects", "", br#"{"commits":[1]}"#);
    assert!(objects.starts_with(b"HTTP/1.1 200"));
    let objects = body(&objects);
    assert_eq!(objects["objects"][0]["commit"], 1);
    assert!(objects["objects"][0]["parent"].is_null());
    let tree = objects["objects"][0]["tree"].as_array().expect("valid fixture");
    assert_eq!(tree.len(), temper_fake::forge::FILES.len());
    let path = b".forgejo/workflows/check.yml";
    let len = skein_lib::base64::encoded_len(
        skein_lib::base64::Alphabet::Standard,
        u32::try_from(path.len()).expect("valid fixture"),
    )
    .expect("valid fixture");
    let mut expected_path = skein_lib::Writer::new(usize::try_from(len).expect("valid fixture"));
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Standard, path, &mut expected_path).expect("valid fixture");
    let expected_path = expected_path.finish();
    assert!(tree.iter().any(|file| file["path"].as_str().expect("valid fixture").as_bytes() == expected_path.as_ref()));
    // Setup's admin flag and the domain's repository permission are separate.
    // Grant through the API, as a runner would, rather than seeding a grant.
    let grant = request(
        run.address,
        "PUT",
        "/api/v1/repos/fake/widget/collaborators/temper-admin",
        &auth,
        br#"{"permission":"admin"}"#,
    );
    assert!(grant.starts_with(b"HTTP/1.1 204"), "{}", String::from_utf8_lossy(&grant));
    let branch = request(
        run.address,
        "POST",
        "/api/v1/repos/fake/widget/branches",
        &auth,
        br#"{"new_branch_name":"work","old_ref_name":"0000000000000001000000000000000000000000"}"#,
    );
    assert!(branch.starts_with(b"HTTP/1.1 201"), "{}", String::from_utf8_lossy(&branch));
    let lines = fs::read_to_string(&observations).expect("valid fixture");
    let records =
        lines.lines().map(|line| serde_json::from_str::<Value>(line).expect("valid fixture")).collect::<Vec<_>>();
    assert!(records.iter().any(|line| line["observation"] == "moved" && line["branch"] == "work"));
    assert!(records.last().expect("valid fixture").get("observations_lost").is_some(), "checkpoint precedes the reply");
    run.stop();
    let lines = fs::read_to_string(&observations).expect("valid fixture");
    assert_eq!(
        serde_json::from_str::<Value>(lines.lines().last().expect("valid fixture")).expect("valid fixture"),
        json!({"observations_lost":0})
    );
    assert!(dir.join("forge/admin_token").exists());
    assert!(observations.exists());
    assert!(doc.exists());
    scratch.remove();
}
const CHAT:&[u8]=br#"{"model":"qa","store":false,"instructions":"","input":[{"role":"user","content":[{"type":"input_text","text":"say hello"}]}],"stream":true}"#;
#[test]
fn llm_run() {
    for tls in [false, true] {
        let scratch = support::Scratch::new(if tls { "fake_llm_tls" } else { "fake_llm_plain" });
        let dir = scratch.path();
        let doc = dir.join("llm.json");
        document(&doc, &llm_document(dir, tls));
        let run = Running::start("llm", &doc, None);
        let key = temper_fake::credential::credential(23, temper_fake::credential::KEY);
        private_hex(&dir.join("key"), &key);
        let auth = format!("Bearer {}", std::str::from_utf8(&key).expect("valid fixture"));
        for (credential, status, text) in [
            (&*auth, b"HTTP/1.1 200".as_slice(), "hello from QA"),
            ("Bearer wrong", b"HTTP/1.1 401", "invalid_api_key"),
        ] {
            let response = if tls {
                assert_eq!(fs::read(dir.join("root.der")).expect("valid fixture"), skein_fake_peers::transport::ROOT);
                assert_eq!(
                    fs::metadata(dir.join("root.der")).expect("valid fixture").permissions().mode() & 0o777,
                    0o600
                );
                let mut roots = rustls::RootCertStore::empty();
                roots
                    .add(rustls::pki_types::CertificateDer::from(
                        fs::read(dir.join("root.der")).expect("valid fixture"),
                    ))
                    .expect("valid fixture");
                let config =
                    rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                        .with_safe_default_protocol_versions()
                        .expect("valid fixture")
                        .with_root_certificates(roots)
                        .with_no_client_auth();
                let connection = rustls::ClientConnection::new(
                    Arc::new(config),
                    rustls::pki_types::ServerName::try_from("skein.test").expect("valid fixture"),
                )
                .expect("valid fixture");
                let mut stream = rustls::StreamOwned::new(connection, stream(run.address));
                stream.write_all(&http("POST", "/v1/responses", credential, CHAT)).expect("valid fixture");
                let mut response = Vec::new();
                match stream.read_to_end(&mut response) {
                    Ok(_) => {}
                    Err(why) if why.kind() == std::io::ErrorKind::UnexpectedEof => {}
                    Err(why) => panic!("TLS response: {why}"),
                }
                response
            } else {
                request(run.address, "POST", "/v1/responses", credential, CHAT)
            };
            assert!(response.starts_with(status), "{}", String::from_utf8_lossy(&response));
            assert!(String::from_utf8_lossy(&response).contains(text), "{}", String::from_utf8_lossy(&response));
        }
        if !tls {
            let mut other: Value = serde_json::from_slice(CHAT).expect("valid fixture");
            other["model"] = json!("another-model");
            let response = request(
                run.address,
                "POST",
                "/v1/responses",
                &auth,
                &serde_json::to_vec(&other).expect("valid fixture"),
            );
            assert!(response.starts_with(b"HTTP/1.1 200"));
            assert!(String::from_utf8_lossy(&response).contains("hello from QA"));
        }
        run.stop();
        assert!(dir.join("key").exists());
        if tls {
            assert!(dir.join("root.der").exists());
        }
        scratch.remove();
    }
}
#[test]
fn ready_write_failure_stops_and_keeps_outputs() {
    let scratch = support::Scratch::new("fake_ready_failure");
    let doc = scratch.path().join("llm.json");
    document(&doc, &llm_document(scratch.path(), false));
    let output = command("llm", &doc)
        .args(["--ready", ""])
        .stdout(fs::OpenOptions::new().write(true).open("/dev/full").expect("valid fixture"))
        .output()
        .expect("valid fixture");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"temper fake llm: stdout: No space left on device (os error 28)\n");
    assert!(scratch.path().join("key").exists());
    scratch.remove();
}
