//! Sitting-only policy and redaction (domain/testing.md, 2.7; protocol/setup.md, 17.2).
use super::relay::{
    Guard,
    managed_transport::{Answer, Request, Transport},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    io::{self, Write},
    net::{SocketAddr, TcpListener},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

const REPO: &str = "/repos/freeekanayaka/temper-probe-2";
fn invalid() -> io::Error {
    io::Error::other("managed relay invalid request or response")
}
#[derive(Default)]
struct State {
    converted: bool,
    app: Option<u64>,
    slug: String,
    installations: BTreeSet<u64>,
    tokens: Vec<String>,
    checked: bool,
    repository: Option<u64>,
    pulls: BTreeSet<u64>,
    commits: BTreeSet<u64>,
    count: usize,
}
struct Shared {
    fake: bool,
    transport: Transport,
    guard: Guard,
    state: Mutex<State>,
    journal: Mutex<File>,
    evidence: Mutex<File>,
}
impl State {
    fn jwt(&self, authorization: &str) -> bool {
        let Some(encoded) = authorization.strip_prefix("Bearer ").and_then(|s| s.split('.').nth(1)) else {
            return false;
        };
        let Ok(bytes) = skein_lib::base64::decode(skein_lib::base64::Alphabet::Url, encoded.as_bytes(), 2048) else {
            return false;
        };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else { return false };
        self.app.is_some_and(|id| {
            value["iss"] == id || value["iss"].as_str().and_then(|s| s.parse::<u64>().ok()) == Some(id)
        })
    }
    fn token(&self, authorization: &str, git: bool) -> bool {
        if git {
            let Some(encoded) = authorization.strip_prefix("Basic ") else { return false };
            let Ok(bytes) = skein_lib::base64::decode(skein_lib::base64::Alphabet::Standard, encoded.as_bytes(), 8192)
            else {
                return false;
            };
            return self.tokens.iter().any(|token| bytes.as_ref() == format!("x-access-token:{token}").as_bytes());
        }
        authorization
            .strip_prefix("Bearer ")
            .or_else(|| authorization.strip_prefix("token "))
            .is_some_and(|token| self.tokens.iter().any(|known| known == token))
    }
}
fn positive(text: &str) -> Option<u64> {
    text.parse().ok().filter(|n| *n > 0)
}
fn installation(path: &str) -> Option<u64> {
    positive(path.strip_prefix("/app/installations/")?)
}
impl Shared {
    fn git_allowed(&self, state: &State, method: &str, target: &str, body: &[u8], auth: &str) -> bool {
        let path = target.split('?').next().unwrap_or("");
        if !state.checked || !state.token(auth, true) {
            return false;
        }
        if self.fake && method == "POST" && path.starts_with("/.fake/git/") {
            let value: Value = match serde_json::from_slice(body) {
                Ok(value) => value,
                Err(_) => return false,
            };
            let known = |name: &str| value[name].as_u64().is_some_and(|id| state.commits.contains(&id));
            return match path {
                "/.fake/git/clone" => value == json!({"repository":"freeekanayaka/temper-probe-2"}),
                "/.fake/git/objects" => value["commits"].as_array().is_some_and(|ids| {
                    !ids.is_empty() && ids.iter().all(|id| id.as_u64().is_some_and(|id| state.commits.contains(&id)))
                }),
                "/.fake/git/commit" => known("parent") && value["tree"].is_array() && value["message"].is_string(),
                "/.fake/git/create" | "/.fake/git/push" => {
                    value["repository"] == "freeekanayaka/temper-probe-2"
                        && value["branch"] == format!("{}managed", self.guard.prefix)
                        && known("commit")
                }
                _ => false,
            };
        }
        self.guard.allows("git", method, target, Some(body), None)
    }
    fn allows(&self, request: &Request, git: bool) -> io::Result<bool> {
        let (method, target) = request.route()?;
        let path = target.split('?').next().ok_or_else(invalid)?;
        // Reject encoded separators, control bytes, dot segments and ambiguous paths before any dispatch.
        if path.replace("%5Bbot%5D", "[bot]").contains(['%', '\\', '#'])
            || path.contains("//")
            || path.split('/').any(|s| matches!(s, "." | ".."))
            || !path.starts_with('/')
            || target.bytes().any(|b| b <= 32 || b == 127)
            || request.header("content-encoding")?.is_some()
        {
            return Ok(false);
        }
        let body = request.body()?;
        let auth = request.header("authorization")?.unwrap_or("");
        let mut state = self.state.lock().map_err(|_| invalid())?;
        if state.count >= 400 {
            return Ok(false);
        }
        state.count += 1;
        if git {
            return Ok(self.git_allowed(&state, method, target, &body, auth));
        }
        if method == "POST" && path.starts_with("/app-manifests/") && path.ends_with("/conversions") {
            let code = &path[15..path.len() - 12];
            if state.converted || code.is_empty() || !code.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Ok(false);
            }
            state.converted = true; // Includes failed attempts: the relay admits conversion exactly once.
            return Ok(true);
        }
        if method == "GET"
            && auth.is_empty()
            && (path == format!("/users/{}[bot]", state.slug) || path == format!("/users/{}%5Bbot%5D", state.slug))
        {
            return Ok(state.app.is_some());
        }
        if state.jwt(auth) {
            if method == "GET" {
                return Ok(matches!(path, "/app" | "/app/installations" | "/app/hook/config" | "/app/hook/deliveries")
                    || installation(path).is_some()
                    || path.strip_prefix("/app/hook/deliveries/").and_then(positive).is_some()
                    || path == format!("/users/{}[bot]", state.slug)
                    || path == format!("/users/{}%5Bbot%5D", state.slug));
            }
            if method == "POST"
                && let Some(id) = path
                    .strip_prefix("/app/installations/")
                    .and_then(|s| s.strip_suffix("/access_tokens"))
                    .and_then(positive)
            {
                return Ok(state.installations.contains(&id)
                    && (body.is_empty()
                        || serde_json::from_slice::<Value>(&body)
                            .is_ok_and(|v| v == json!({}) || v == json!({"permissions":{"contents":"write"}}))));
            }
            return Ok(false);
        }
        if !state.token(auth, false) {
            return Ok(false);
        }
        if method == "GET" && path == "/installation/repositories" {
            return Ok(true);
        }
        if method == "DELETE" && path == "/installation/token" {
            return Ok(true);
        }
        if method == "POST"
            && path == "/graphql"
            && serde_json::from_slice::<Value>(&body).is_ok_and(|v| {
                v["query"].as_str().is_some_and(|q| q.as_bytes() == skein_forge::github::document::VIEWER)
            })
        {
            return Ok(true);
        }
        if !state.checked {
            return Ok(false);
        }
        if method == "GET"
            && path.strip_prefix("/repositories/").and_then(positive).is_some_and(|id| state.repository == Some(id))
        {
            return Ok(true);
        }
        if method == "GET" && (path == REPO || path.starts_with(&format!("{REPO}/"))) {
            return Ok(true);
        }
        let value = serde_json::from_slice::<Value>(&body).unwrap_or(Value::Null);
        if method == "POST" && path == format!("{REPO}/pulls") {
            return Ok(value["head"] == format!("{}managed", self.guard.prefix) && value["base"] == "main");
        }
        if method == "PATCH"
            && let Some(number) = path
                .strip_prefix(&format!("{REPO}/pulls/"))
                .or_else(|| path.strip_prefix(&format!("{REPO}/issues/")))
                .and_then(positive)
        {
            return Ok(state.pulls.contains(&number) && value == json!({"state":"closed"}));
        }
        Ok(method == "DELETE" && path == format!("{REPO}/git/refs/heads/{}managed", self.guard.prefix))
    }
    fn learn(&self, request: &Request, answer: &Answer) -> io::Result<()> {
        let (method, target) = request.route()?;
        let path = target.split('?').next().ok_or_else(invalid)?;
        if !matches!(answer.status()?, 200 | 201) {
            return Ok(());
        }
        let value: Value = match serde_json::from_slice(&answer.body()?) {
            Ok(value) => value,
            Err(_) => return Ok(()),
        };
        let mut state = self.state.lock().map_err(|_| invalid())?;
        if self.fake
            && method == "POST"
            && path == "/.fake/git/clone"
            && let Some(rows) = value["branches"].as_array()
        {
            for row in rows {
                if let Some(id) = row["commit"].as_u64() {
                    state.commits.insert(id);
                }
            }
        }
        if self.fake && method == "POST" && path == "/.fake/git/commit" {
            state.commits.insert(value["commit"].as_u64().ok_or_else(invalid)?);
        }
        if method == "POST" && path.starts_with("/app-manifests/") {
            if value["owner"]["login"] != "freeekanayaka" {
                return Err(invalid());
            }
            state.app = Some(value["id"].as_u64().ok_or_else(invalid)?);
            state.slug = value["slug"].as_str().ok_or_else(invalid)?.into();
            let sanitized = redact(&value, "");
            let mut file = self.evidence.lock().map_err(|_| invalid())?;
            serde_json::to_writer_pretty(&mut *file, &sanitized)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
        }
        if method == "GET"
            && installation(path).is_some()
            && value["account"]["login"] == "freeekanayaka"
            && (value.get("app_id").is_none() || value["app_id"].as_u64() == state.app)
        {
            state.installations.insert(value["id"].as_u64().ok_or_else(invalid)?);
        }
        if method == "POST" && path.ends_with("/access_tokens") {
            state.tokens.push(value["token"].as_str().ok_or_else(invalid)?.into());
        }
        if method == "GET" && path == "/installation/repositories" {
            state.checked = false;
            let rows = value["repositories"].as_array().ok_or_else(invalid)?;
            if value["total_count"] != 1
                || rows.len() != 1
                || rows[0]["owner"]["login"] != "freeekanayaka"
                || rows[0]["name"] != "temper-probe-2"
                || rows[0].get("full_name").is_some_and(|name| name != "freeekanayaka/temper-probe-2")
            {
                return Err(invalid());
            }
            state.repository = Some(rows[0]["id"].as_u64().ok_or_else(invalid)?);
            state.checked = true;
        }
        if method == "GET"
            && path == format!("{REPO}/pulls")
            && let Some(rows) = value.as_array()
        {
            for row in rows {
                if row["head"]["ref"] == format!("{}managed", self.guard.prefix)
                    && (row["head"]["repo"]["full_name"] == "freeekanayaka/temper-probe-2"
                        || self.fake && row["head"].get("repo").is_none())
                {
                    state.pulls.insert(row["number"].as_u64().ok_or_else(invalid)?);
                }
            }
        }
        if method == "POST" && path == format!("{REPO}/pulls") {
            state.pulls.insert(value["number"].as_u64().ok_or_else(invalid)?);
        }
        Ok(())
    }
    fn serve(&self, mut socket: std::net::TcpStream, git: bool) -> io::Result<()> {
        let request = Request::read(&mut socket)?;
        let (method, target) = request.route()?;
        let accepted = self.allows(&request, git)?;
        let answer = if accepted {
            let answer = self.transport.exchange(&request, git)?;
            let status = answer.status()?;
            // A redirected client could otherwise leave this private guard entirely.
            if ((300..400).contains(&status) && status != 304) || self.learn(&request, &answer).is_err() {
                Answer::refuse()
            } else {
                answer
            }
        } else {
            Answer::refuse()
        };
        let path = if target.starts_with("/app-manifests/") {
            "/app-manifests/[code]/conversions"
        } else {
            target.split('?').next().ok_or_else(invalid)?
        };
        let mut journal = self.journal.lock().map_err(|_| invalid())?;
        serde_json::to_writer(
            &mut *journal,
            &json!({"listener":if git {"git"} else {"api"}, "method":method, "path":path, "status":answer.status()?, "refused":!accepted || answer.status()? == 403}),
        )?;
        journal.write_all(b"\n")?;
        journal.flush()?;
        answer.send(&mut socket, method)
    }
}
/// Each node keeps its JSON type. Only the owner's explicit allowlist keeps values.
fn redact(value: &Value, path: &str) -> Value {
    let allowed =
        matches!(path, "id" | "slug" | "name" | "html_url" | "owner.login" | "owner.type" | "permissions" | "events")
            || path.starts_with("permissions.")
            || path.starts_with("events.");
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(name, value)| {
                    let next = if path.is_empty() { name.clone() } else { format!("{path}.{name}") };
                    (name.clone(), redact(value, &next))
                })
                .collect(),
        ),
        Value::Array(rows) => Value::Array(rows.iter().map(|v| redact(v, &format!("{path}.[]"))).collect()),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            let kind = match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) | Value::Object(_) => unreachable!(),
            };
            if allowed { json!({"type":kind,"value":value}) } else { json!({"type":kind}) }
        }
    }
}
#[expect(clippy::disallowed_methods, reason = "the sitting's bounded socket relay runs on owned threads")]
pub fn run() -> io::Result<()> {
    let directory = PathBuf::from(std::env::var_os("TEMPER_MANAGED_SCRATCH").ok_or_else(invalid)?);
    let prefix = std::env::var("TEMPER_MANAGED_PREFIX").map_err(|_| invalid())?;
    let mode = std::env::var("TEMPER_MANAGED_MODE").map_err(|_| invalid())?;
    let fake = match mode.as_str() {
        "fake" => Some(
            std::env::var("TEMPER_MANAGED_UPSTREAM")
                .map_err(|_| invalid())?
                .parse::<SocketAddr>()
                .map_err(|_| invalid())?,
        ),
        "live" => None,
        _ => return Err(invalid()),
    };
    let evidence = PathBuf::from(std::env::var_os("TEMPER_MANAGED_EVIDENCE").ok_or_else(invalid)?);
    let shared = Arc::new(Shared {
        fake: fake.is_some(),
        transport: Transport::new(fake)?,
        guard: Guard { owner: "freeekanayaka".into(), repository: "temper-probe-2".into(), installation: 0, prefix },
        state: Mutex::new(State::default()),
        journal: Mutex::new(
            OpenOptions::new().create_new(true).write(true).mode(0o600).open(directory.join("relay.jsonl"))?,
        ),
        evidence: Mutex::new(OpenOptions::new().create_new(true).write(true).mode(0o600).open(evidence)?),
    });
    let api = TcpListener::bind("127.0.0.1:0")?;
    let git = TcpListener::bind("127.0.0.1:0")?;
    let addresses = json!({"api":api.local_addr()?.to_string(), "git":git.local_addr()?.to_string()});
    super::write_private(&directory.join("relay-ready.json"), &serde_json::to_vec(&addresses)?);
    for (listener, git) in [(api, false), (git, true)] {
        let shared = shared.clone();
        thread::spawn(move || {
            for socket in listener.incoming().flatten() {
                let shared = shared.clone();
                thread::spawn(move || {
                    if shared.serve(socket, git).is_err() {
                        // Neither errors nor request dumps can expose keys, JWTs, codes or bodies.
                        if let Ok(mut journal) = shared.journal.lock() {
                            let _ = journal.write_all(b"{\"error\":\"transport or framing failed\"}\n");
                        }
                    }
                });
            }
        });
    }
    // The script owns this child and terminates it before removing scratch.
    loop {
        thread::park();
    }
}

#[test]
fn conversion_evidence_keeps_only_the_owners_allowlisted_values() {
    let value = json!({"id":42,"slug":"temper","pem":"private PEM","client_id":"oauth-id",
        "client_secret":"oauth-secret","webhook_secret":"hook-secret",
        "owner":{"login":"freeekanayaka","type":"User","id":123,"nested":{"secret":false}},
        "permissions":{"contents":"write"},"events":["push"],"unknown":[{"key":"secret"},true,null,42]});
    let result = redact(&value, "");
    assert_eq!(result["id"], json!({"type":"number","value":42}));
    assert_eq!(result["pem"], json!({"type":"string"}));
    assert_eq!(result["client_id"], json!({"type":"string"}));
    assert_eq!(result["owner"]["login"]["value"], "freeekanayaka");
    assert_eq!(result["owner"]["id"], json!({"type":"number"}));
    assert_eq!(result["permissions"]["contents"]["value"], "write");
    assert_eq!(result["events"][0]["value"], "push");
    assert_eq!(result["unknown"][0]["key"], json!({"type":"string"}));
    assert_eq!(result["unknown"][1], json!({"type":"boolean"}));
    assert_eq!(result["unknown"][2], json!({"type":"null"}));
    for secret in ["private PEM", "oauth-id", "oauth-secret", "hook-secret"] {
        assert!(!result.to_string().contains(secret));
    }
}

#[test]
fn guard_binds_conversion_credentials_repository_and_run_writes() {
    fn request(method: &str, path: &str, auth: &str, body: &Value) -> Request {
        let body = serde_json::to_vec(&body).expect("request JSON");
        let mut wire = format!(
            "{method} {path} HTTP/1.1\r\nHost: localhost\r\nAuthorization: {auth}\r\nContent-Length: {}\r\n\r\n",
            body.len()
        )
        .into_bytes();
        wire.extend(body);
        Request::from_bytes(&wire).expect("test framing")
    }
    let shared = Shared {
        fake: true,
        transport: Transport::new(Some("127.0.0.1:9".parse().expect("loopback"))).expect("plain transport"),
        guard: Guard {
            owner: "freeekanayaka".into(),
            repository: "temper-probe-2".into(),
            installation: 0,
            prefix: "temper/live-test/".into(),
        },
        state: Mutex::new(State::default()),
        journal: Mutex::new(OpenOptions::new().write(true).open("/dev/null").expect("discard test journal")),
        evidence: Mutex::new(OpenOptions::new().write(true).open("/dev/null").expect("discard evidence")),
    };
    let allowed = |method: &str, path: &str, auth: &str, body: Value, git: bool| {
        shared.allows(&request(method, path, auth, &body), git).expect("guard")
    };
    assert!(allowed("POST", "/app-manifests/code/conversions", "", json!({}), false));
    assert!(!allowed("POST", "/app-manifests/other/conversions", "", json!({}), false));
    assert!(!allowed("GET", REPO, "Bearer unknown", json!({}), false));
    {
        let mut state = shared.state.lock().expect("state");
        state.tokens.push("fake-minted".into());
        state.repository = Some(1);
        state.commits.insert(1);
    }
    assert!(!allowed("GET", REPO, "Bearer fake-minted", json!({}), false));
    assert!(allowed("GET", "/installation/repositories", "Bearer fake-minted", json!({}), false));
    shared.state.lock().expect("state").checked = true;
    assert!(allowed("GET", REPO, "Bearer fake-minted", json!({}), false));
    assert!(!allowed("GET", "/repos/freeekanayaka/temper-probe", "Bearer fake-minted", json!({}), false));
    assert!(!allowed(
        "GET",
        "/repos/freeekanayaka/temper-probe-2/%2e%2e/other",
        "Bearer fake-minted",
        json!({}),
        false
    ));
    assert!(!allowed("GET", "/repositories/2", "Bearer fake-minted", json!({}), false));
    assert!(allowed(
        "POST",
        &format!("{REPO}/pulls"),
        "Bearer fake-minted",
        json!({"head":"temper/live-test/managed","base":"main"}),
        false
    ));
    assert!(!allowed(
        "POST",
        &format!("{REPO}/pulls"),
        "Bearer fake-minted",
        json!({"head":"main","base":"main"}),
        false
    ));
    assert!(!allowed("PATCH", &format!("{REPO}/pulls/1"), "Bearer fake-minted", json!({"state":"closed"}), false));
    assert!(!allowed("DELETE", &format!("{REPO}/git/refs/heads/main"), "Bearer fake-minted", json!({}), false));
    let basic = "Basic eC1hY2Nlc3MtdG9rZW46ZmFrZS1taW50ZWQ=";
    assert!(allowed(
        "POST",
        "/.fake/git/create",
        basic,
        json!({"repository":"freeekanayaka/temper-probe-2","branch":"temper/live-test/managed","commit":1}),
        true
    ));
    assert!(!allowed(
        "POST",
        "/.fake/git/create",
        basic,
        json!({"repository":"freeekanayaka/temper-probe-2","branch":"main","commit":1}),
        true
    ));
    assert!(!allowed(
        "POST",
        "/.fake/git/create",
        basic,
        json!({"repository":"freeekanayaka/temper-probe","branch":"temper/live-test/managed","commit":1}),
        true
    ));
}
