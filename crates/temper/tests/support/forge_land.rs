//! A person's change on the owned forge, signed in with `ChatGPT` (domain/testing.md, 2.6–2.7).
use super::{Process, sign_in};
use skein_fake_llm_domain::api::{self, Finish, Line, Script, Turn};
use skein_fake_peers::oauth;
use skein_io::kernel::Exit;
use skein_lib::Duration;
use skein_world::{Referee, real};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

/// The model side: processes a story adds beside the box and ctl.
#[expect(
    clippy::large_enum_variant,
    reason = "existing shared fake constructors pass peers by value; faces are moved once at startup"
)]
pub enum Face {
    /// Shipped issuer and Responses; no auxiliary model processes.
    Live,
    /// Skein's fake issuer and the existing TLS LLM peer playing the plan's scripts.
    Fake { issuer: oauth::Peer, peer: skein_fake_peers::llm::Peer },
}

/// Initialize three fresh private directories, then patch the config for the fakes.
#[must_use]
pub fn fake_deployment(paths: &[PathBuf; 3]) -> oauth::Peer {
    initialize(paths);
    sign_in::patch_configuration(&paths[0])
}

fn initialize(paths: &[PathBuf; 3]) {
    for path in paths {
        fs::create_dir(path).expect("fresh deployment directory");
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private deployment directory");
    }
    let initialized = std::process::Command::new(env!("CARGO_BIN_EXE_temper"))
        .arg("init")
        .args(
            ["--config", "--data", "--cache"]
                .into_iter()
                .zip(paths)
                .flat_map(|(flag, path)| [std::ffi::OsString::from(flag), path.as_os_str().to_owned()]),
        )
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .current_dir(&paths[1])
        .output()
        .expect("initialize a fresh deployment");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
}

/// The person's request and the first repository's observed identity and contents.
pub struct Plan<'a> {
    /// The person's words, carrying `line`, which cue the chat.
    pub words: &'a [u8],
    /// The unique README line, without its trailing newline.
    pub line: &'a [u8],
    /// The first repository's resource, its packed tag (`temper_engine_domain_forge::tag(3, first.id)`).
    pub repository: u64,
    /// README.md on main before the change.
    pub readme: &'a [u8],
}

/// The first box's reply, once the change has landed.
pub const LANDED: &str = "The change landed.";
/// The restarted chat's answer: the second box's reply.
pub const RESTARTED: &str = "The change is still on main.";

/// Delegate, wait, then reply as an Open chat.
#[must_use]
pub fn plan_scripts(plan: &Plan<'_>) -> Box<[Script]> {
    let line = std::str::from_utf8(plan.line).expect("README line UTF-8");
    let coding = format!("@coding Append this README line: {line}");
    assert!(!plan.words.is_empty(), "nonempty chat cue");
    assert!(!plan.line.is_empty() && !plan.line.contains(&b'\n'), "one nonempty README line");
    assert!(plan.words.windows(plan.line.len()).any(|part| part == plan.line), "request carries the line");
    // System cues match the earliest byte position. Each task's original spec
    // supplies its cue before any conversation history. Keep the coder's spec
    // distinct from the chat cue.
    assert!(!coding.as_bytes().windows(plan.words.len()).any(|part| part == plan.words), "distinct chat cue");
    let edit = format!("{line}\n");
    let mut scripts = temper_real_loop_world::landing::change_scripts(
        temper_real_loop_world::landing::ChangeAuthority { coding_kind: 2, scope: None },
        plan.repository,
        plan.readme,
        edit.as_bytes(),
        plan.words,
        &coding,
    )
    .into_vec();
    // Init has gates: [], so no reviewer runs in this story.
    scripts.truncate(2);
    // Keep the first repository's proof that the worker cloned the starter workflow.
    let workflow = Turn {
        lines: Box::new([Line::Call {
            name: b"read".as_slice().into(),
            arguments: br#"{"path":".forgejo/workflows/ci.yml"}"#.as_slice().into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    };
    let mut coder = std::mem::take(&mut scripts[1].turns).into_vec();
    coder.insert(1, workflow);
    scripts[1].turns = coder.into_boxed_slice();
    let text =
        |bytes: &[u8]| Turn { lines: Box::new([Line::Text { text: bytes.into() }]), finish: Finish::Stop, tokens: 20 };
    let chat = &mut scripts[0];
    chat.turns[0].lines = Box::new([Line::Call {
        name: b"change".as_slice().into(),
        arguments: serde_json::to_vec(&serde_json::json!({"words": coding, "repository": "work/hello"}))
            .expect("change tool JSON")
            .into_boxed_slice(),
    }]);
    // An Open chat replies with text and keeps listening (domain/agent.md, 4.6).
    // The first box stops without Close; no finish request or closing turn runs here.
    chat.turns[2] = text(LANDED.as_bytes());
    scripts.into_boxed_slice()
}

/// The fake face uses the shared plan bearer and existing test TLS root.
#[must_use]
pub fn fake_face(issuer: oauth::Peer, scripts: Box<[Script]>) -> Face {
    Face::Fake { issuer, peer: sign_in::plan_peer(scripts) }
}

/// Indexes of the face's processes, following the story's box and ctl.
pub struct FaceProcs {
    pub issuer: usize,
    pub visitor: usize,
    pub peer: usize,
}

impl Face {
    /// Add the issuer, redirect visitor and model peer to the story's world.
    pub(super) fn spawn<R: Referee<Process>>(self, world: &mut real::World<Process, R>) -> FaceProcs {
        match self {
            Self::Live => panic!("the live face runs without peers through run_live"),
            Self::Fake { issuer, peer } => FaceProcs {
                issuer: world.spawn(|| Process::Issuer(Box::new(issuer))),
                visitor: world.spawn(|| Process::Visitor(Box::default())),
                peer: world.spawn(|| Process::Peer(Box::new(peer))),
            },
        }
    }
}

/// What the model peer saw, without credential values.
pub struct Seen {
    pub requests: Vec<api::Query>,
    pub answers: Vec<Result<(), api::Error>>,
    pub schemes: Vec<Option<skein_llm::Scheme>>,
}

#[must_use]
pub(super) fn seen(procs: &[Process], at: &FaceProcs) -> Seen {
    peer_seen(procs, at.peer)
}

pub(super) fn peer_seen(procs: &[Process], index: usize) -> Seen {
    use skein_fake_peers::llm::Observation;
    let Process::Peer(peer) = &procs[index] else { panic!("LLM peer") };
    let mut seen = Seen { requests: Vec::new(), answers: Vec::new(), schemes: Vec::new() };
    for event in peer.observations() {
        match event {
            Observation::Query { query, .. } => seen.requests.push(query.clone()),
            Observation::Answered { result, .. } => seen.answers.push(*result),
            Observation::Credential { scheme, .. } => seen.schemes.push(*scheme),
            Observation::Accepted { .. } | Observation::Head { .. } | Observation::Closed { .. } => {}
        }
    }
    seen
}

/// A face and the conversation reads its box must follow before stopping.
pub struct CtlFace {
    pub face: Face,
    pub first: bool,
}

/// Follow the committed reply, paging the conversation and waiting from a read's digest.
pub(super) struct Reply {
    first: bool,
    snapshot: Option<String>,
    phase: ReplyPhase,
    replied: bool,
}
enum ReplyPhase {
    Snapshot,
    Pages,
    Done,
}
impl Reply {
    pub fn new(first: bool) -> Self {
        Self { first, snapshot: None, phase: ReplyPhase::Snapshot, replied: false }
    }
    pub fn advance(&mut self, calls: &mut super::ctl::Calls, expected_exits: &mut Vec<(usize, u8)>) {
        use super::ctl::{Call, Step, json};
        if matches!(self.phase, ReplyPhase::Done) || !calls.foreground_idle() {
            return;
        }
        let last = json(calls.ends().last().expect("reply read"));
        if matches!(self.phase, ReplyPhase::Snapshot) {
            self.snapshot = Some(last["digest"].as_str().expect("task read digest").to_owned());
            calls.push(Step::Run(Call::new(&["chat", "1", "--json"])));
            self.phase = ReplyPhase::Pages;
            return;
        }
        // The first box waits for the change's reply; the second, for the
        // restarted chat's answer, before it closes the chat.
        let reply = if self.first { LANDED } else { RESTARTED };
        self.replied |= serde_json::to_string(&last["activity"]["rows"]).expect("conversation JSON").contains(reply);
        if let Some(before) = last["activity"]["before"]["value"].as_str() {
            calls.push(Step::Run(Call::new(&["chat", "1", "--before", before, "--json"])));
        } else if !self.replied {
            // 'Change requested' plus wait already meets replied. A snapshot
            // taken before reading the conversation prevents missing its next reply.
            calls.push(Step::Run(Call::new(&[
                "wait",
                "task",
                "1",
                "--until",
                "replied",
                "--since",
                self.snapshot.as_deref().expect("snapshot before conversation"),
                "--timeout",
                "120s",
                "--json",
            ])));
            calls.push(Step::Run(Call::new(&["task", "1", "--json"])));
            self.phase = ReplyPhase::Snapshot;
        } else {
            for parent in ["1", "2"] {
                calls.push(Step::Run(Call::new(&["tasks", "--project", "1", "--parent", parent, "--ended", "--json"])));
            }
            if !self.first {
                // The six queued reads end with task 4, which must be unknown.
                expected_exits.push((calls.ends().len() + 5, 2));
                for args in [
                    &["close", "1", "--key", "close-plan", "--json"][..],
                    &["wait", "task", "1", "--until", "ended", "--timeout", "60s", "--json"],
                    &["tasks", "--project", "1", "--json"],
                    &["task", "4", "--json"],
                ] {
                    calls.push(Step::Run(Call::new(args)));
                }
            }
            calls.release();
            self.phase = ReplyPhase::Done;
        }
    }
}

#[must_use]
pub fn restart_script(chat_cue: &[u8]) -> Box<[Script]> {
    // A system cue's turns count every assistant message (fake-llm.md 2.4):
    // the first box's delegate, wait and reply are turns 0 to 2, played there.
    // The restart runs the chat's lost attempt again, which answers (turn 3);
    // Close then asks the parked run to finish, and its text ends it (turn 4,
    // domain/agent.md, 4.6). An Open run offers no finish tool.
    let text =
        |bytes: &[u8]| Turn { lines: Box::new([Line::Text { text: bytes.into() }]), finish: Finish::Stop, tokens: 20 };
    let played = b"Played in the first box.";
    Box::new([Script {
        cue: chat_cue.into(),
        turns: Box::new([
            text(played),
            text(played),
            text(played),
            text(RESTARTED.as_bytes()),
            text(b"The plan chat is now closed."),
        ]),
    }])
}

#[must_use]
pub fn landed_identity(task: &str, page: &str) -> (String, String) {
    let landed = task.lines().find(|line| line.contains("Landed · merge commit ")).expect("landed line");
    let pull = page.lines().find(|line| line.starts_with("PR #")).expect("pull identity on the change page");
    assert!(pull.contains(" · work/hello · "), "pull belongs to the first repository: {pull}");
    (landed.to_owned(), pull.to_owned())
}

#[must_use]
pub fn ended_tasks(value: &serde_json::Value, expected: u64) -> Vec<u64> {
    assert_no_live_tasks(value);
    let ended = &value["ended"]["rows"];
    assert_eq!(ended["cut"], false, "whole ended task set: {value}");
    assert!(ended["after"].is_null(), "whole ended task set: {value}");
    let mut tasks: Vec<_> = ended["rows"]
        .as_array()
        .expect("ended rows")
        .iter()
        .map(|row| row["chip"]["task"].as_u64().expect("task number"))
        .collect();
    tasks.sort_unstable();
    assert_eq!(tasks, [expected], "exact ended delegates of this parent");
    tasks
}
pub fn assert_no_live_tasks(value: &serde_json::Value) {
    assert_eq!(value["live"], serde_json::json!([]), "no live tasks in this query: {value}");
    assert!(value["live_after"].is_null(), "all live tasks read");
}
/// A project's live roots read: `tasks --project P` without `--ended`.
pub fn assert_no_live_roots(value: &serde_json::Value) {
    assert_eq!(value["tasks"]["rows"], serde_json::json!([]), "no live root tasks: {value}");
    assert_eq!(value["tasks"]["more"], false, "all live root tasks read: {value}");
}

pub fn assert_reply(calls: &[super::Ended], first: bool) {
    assert_eq!(replies(calls, LANDED), 1, "one committed final reply, across all conversation pages");
    assert_eq!(replies(calls, RESTARTED), usize::from(!first), "the restarted chat answers once");
}
fn replies(calls: &[super::Ended], text: &str) -> usize {
    let mut replies = std::collections::BTreeMap::new();
    for call in calls {
        let page = super::ctl::json(call);
        let Some(rows) = page["activity"]["rows"].as_array() else { continue };
        assert_eq!(page["activity"]["cut"], false);
        for row in rows {
            let Some(turn) = row.pointer("/event/owner/decoded/turn/decoded") else { continue };
            assert_eq!(turn["cut"], false, "whole turn");
            for message in turn["messages"].as_array().expect("turn messages") {
                if message["speaker"] != "assistant" {
                    continue;
                }
                for block in message["blocks"].as_array().expect("message blocks") {
                    if block["text"]["text"] == text {
                        replies.insert(row["position"].as_u64().expect("position"), turn.clone());
                    }
                }
            }
        }
    }
    replies.len()
}

pub fn assert_requests(seen: &Seen, count: usize, words: &[u8], first: bool) {
    let mut counts = [0; 3];
    for query in &seen.requests {
        let has = |cue: &[u8]| query.system.windows(cue.len()).any(|part| part == cue);
        let kind = if has(words) {
            0
        } else if has(b"@coding Append this README line:") {
            1
        } else if has(b"Review this change at the named head") {
            2
        } else {
            panic!("unexpected task cue")
        };
        counts[kind] += 1;
    }
    assert_eq!(counts, if first { [3, 4, 0] } else { [2, 0, 0] }, "chat, coder, review requests");
    assert_eq!(seen.requests.len(), count, "exact scripted model requests");
    assert_eq!(seen.answers, vec![Ok(()); count], "every request answered");
    assert_eq!(seen.schemes, vec![Some(skein_llm::Scheme::Bearer); count], "a bearer on every request");
}

pub fn plan_change_steps(
    steps: &mut Vec<super::ctl::Step>,
    first: bool,
    setup_calls: usize,
    words: &str,
) -> (usize, usize) {
    change_steps(steps, first, setup_calls, words, "120s", false)
}

fn change_steps(
    steps: &mut Vec<super::ctl::Step>,
    first: bool,
    setup_calls: usize,
    words: &str,
    within: &str,
    live: bool,
) -> (usize, usize) {
    use super::ctl::{Call, Step};
    steps.push(Step::Run(Call::new(&["me", "--json"])));
    if first {
        steps.extend([
            Step::Run(Call::new(&["connect", "1", "--key", "connect-plan"])),
            Step::Run(Call::new(if live {
                &["wait", "accounts", "--until", "settled", "--timeout", "600s", "--json"]
            } else {
                &["wait", "accounts", "--until", "settled", "--json"]
            })),
        ]);
    }
    let account_at = setup_calls + if first { 3 } else { 1 };
    steps.push(Step::Run(Call::new(&["accounts", "--json"])));
    if first {
        steps.push(Step::Run(Call::new(&[
            "chat",
            "--project",
            "1",
            "--model",
            "gpt-6-luna",
            "--key",
            "change-on-plan",
            words,
            "--json",
        ])));
    }
    if first {
        // Chat creation answers before its delegate exists. As in the
        // first-repository story, wait for the chat's initial reply first.
        steps.push(Step::Run(Call::new(&["wait", "task", "1", "--until", "replied", "--timeout", within, "--json"])));
    }
    let found_at = account_at + 1 + 2 * usize::from(first);
    let mut found = super::first_repository::found_steps();
    if live {
        found[0] = Step::Run(Call::new(&["wait", "task", "2", "--until", "ended", "--timeout", within, "--json"]));
    }
    steps.extend(found);
    // The referee holds ctl open, reads and pages the conversation, and
    // follows --since if the procedure's end has not yet reached the chat.
    steps.push(Step::Run(Call::new(&["task", "1", "--json"])));
    (account_at, found_at)
}

pub fn assert_plan_box<T>(run: &str, first: bool, outcome: &super::boxed::CtlOutcome<T>, stderr: &str) {
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "{run}: {stderr}");
    assert_eq!(outcome.left, Some(vec![]), "{run}: {stderr}");
    for (index, call) in outcome.calls.iter().enumerate() {
        assert_eq!(
            call.exit,
            Some(Exit::Code(if !first && index == outcome.calls.len() - 1 { 2 } else { 0 })),
            "{run} call {index}: stdout {}; stderr {}; box {stderr}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
}

#[must_use]
pub fn deployment_flags(paths: &[std::path::PathBuf; 3]) -> Vec<String> {
    ["--config", "--data", "--cache"]
        .into_iter()
        .zip(paths)
        .flat_map(|(flag, path)| [flag.into(), path.display().to_string()])
        .collect()
}

type BinaryFingerprint = (u64, [u8; 32]);

#[must_use]
pub fn cache_forgejo_binaries(cache: &std::path::Path) -> ([std::path::PathBuf; 2], [BinaryFingerprint; 2]) {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    let (forgejo, runner) = super::owned::binaries();
    let cached = [
        cache.join(format!("forgejo/{VERSION}/forgejo")),
        cache.join(format!("forgejo-runner/{RUNNER_VERSION}/forgejo-runner")),
    ];
    for (source, target) in [forgejo, runner].iter().zip(&cached) {
        cache_binary(source, target);
    }
    let before = cached.each_ref().map(|path| cached_binary_fingerprint(path));
    (cached, before)
}

#[must_use]
pub fn forgejo_setup_steps(run: &str) -> Vec<super::ctl::Step> {
    use super::ctl::{Call, Step};
    let mut steps = vec![Step::Mark("start"), Step::Run(Call::sign_in())];
    if run == "first" {
        steps.extend([
            Step::Run(Call::new(&["forges"])),
            Step::Run(Call::new(&["forges", "--json"])),
            Step::Run(Call::new(&["forge-setup", "3"])),
        ]);
    }
    // Real preparation, hashing two releases, runner registration and reconcile
    // take seconds; allow three minutes, plus ten seconds to stop and reap.
    steps.extend([
        Step::Run(Call::new(&["wait", "forges", "--until", "settled", "--timeout", "180s"])),
        Step::Run(Call::new(&["forges", "--json"])),
        Step::Run(Call::new(&["forges"])),
    ]);
    steps
}

pub fn cache_binary(source: &std::path::Path, target: &std::path::Path) {
    use std::{
        fs,
        os::unix::fs::{DirBuilderExt, PermissionsExt},
    };
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(target.parent().expect("cache parent"))
        .expect("private binary cache");
    fs::copy(source, target).expect("copy pinned executable, never link it");
    fs::set_permissions(target, fs::Permissions::from_mode(0o700)).expect("private executable");
}

#[must_use]
pub fn assert_forgejo_setup(
    run: &str,
    calls: &[super::Ended],
    stderr: &str,
    previous: Option<u64>,
    data: &std::path::Path,
) -> u64 {
    for call in calls {
        assert_eq!(
            call.exit,
            Some(Exit::Code(0)),
            "{run}: ctl stdout: {}; stderr: {}; box: {stderr}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
    assert_eq!(calls.len(), if run == "first" { 7 } else { 4 }, "all setup calls ran: {stderr}");
    if run == "first" {
        assert!(
            String::from_utf8_lossy(&calls[1].stdout)
                .lines()
                .any(|line| line == "forge 3 · Forgejo · owned · Not set up")
        );
        let offered = super::ctl::json(&calls[2]);
        let forge = offered["forges"]
            .as_array()
            .expect("forge rows")
            .iter()
            .find(|forge| forge["number"] == 3)
            .expect("forge 3 row");
        assert_eq!(forge["offer"], "offered");
        assert_eq!(forge["state"], "not_set_up");
        assert!(calls[3].stdout.starts_with(b"setting up\n"));
        let adoption = String::from_utf8_lossy(&calls[3].stdout);
        assert!(
            adoption.contains("Adopted work/hello on forge 3 as owned."),
            "ctl printed the first adoption: {adoption}"
        );
        eprintln!("{run} first repository adoption: {adoption}");
        let made = super::ctl::json(&calls[5]);
        let row =
            made["forges"].as_array().expect("forges rows").iter().find(|row| row["number"] == 3).expect("forge 3");
        eprintln!("{run} first repository read: {}", row["first"]);
        assert_eq!(row["first"]["state"], "made");
        assert_eq!(row["first"]["project"]["existing"], 1);
        assert_eq!(row["first"]["org"], "work");
        assert_eq!(row["first"]["repository"], "hello");
        assert_eq!(row["first"]["id"], 1, "the script tags forge 3, repository 1");
        assert_eq!(row["state"], "connected");
        // Preparation is silent on success; Forgejo's startup
        // and the runner's lines reach the box with their step prefixes.
        for step in ["forgejo 3: ", "runner 3: "] {
            assert!(stderr.lines().any(|line| line.starts_with(step)), "group step {step}: {stderr}");
        }
    } else {
        assert!(
            !stderr.lines().any(|line| line.starts_with("fetch ") || line.contains("download")),
            "the restarted group fetches nothing: {stderr}"
        );
    }
    let stdout = String::from_utf8_lossy(&calls.last().expect("forges read").stdout);
    let user = stdout.lines().find_map(|line| {
        line.strip_prefix("forge 3 · Forgejo · owned · Connected as temper (")?.split(')').next()?.parse::<u64>().ok()
    });
    assert!(user.is_some_and(|user| user != 0), "{run}: forges row: {stdout}; box: {stderr}");
    let user = user.expect("connected bot's ID");
    if let Some(previous) = previous {
        assert_eq!(user, previous, "the second box connects as the same bot");
    }
    for name in ["admin_token", "api_token", "git_token", "user", "webhook_secret"] {
        assert!(!data.join("forge-3").join(name).exists(), "{run}: no forge-3/{name}");
    }
    user
}

#[must_use]
pub fn cached_binary_fingerprint(path: &std::path::Path) -> BinaryFingerprint {
    use std::io::Read;
    let mut file = std::fs::File::open(path).expect("cached binary");
    let length = file.metadata().expect("cached metadata").len();
    let mut hash = skein_lib::sha256::Sha256::new();
    let mut bytes = vec![0; 65_536].into_boxed_slice();
    loop {
        let count = file.read(&mut bytes).expect("binary chunk");
        if count == 0 {
            break;
        }
        hash.update(&bytes[..count]);
    }
    (length, hash.finish())
}

/// Init's shipped issuer, Responses endpoints and charter text, on the same
/// three roots as the process story. Only sign-in and run deadlines grow.
pub fn live_deployment(paths: &[PathBuf; 3]) {
    initialize(paths);
    let path = paths[0].join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["limits"]["overrides"]["accounts.sign_in_deadline_ms"] = 600_000.into();
    for charter in value["engine"]["charters"].as_array_mut().expect("charters") {
        assert_eq!(charter["model"], "gpt-6-luna", "the shipped plan model");
        charter["time_ms"] = 240_000.into();
    }
    super::write_private(&path, &serde_json::to_vec(&value).expect("live configuration"));
}

/// Reserve a new kept deployment without reading or replacing an existing store.
pub fn reserve_live_store(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            format!(
                "kept store {} exists: reruns on a kept sign-in are not built yet; move it aside to sign in afresh",
                path.display()
            )
        } else {
            format!("create kept store {}: {error}", path.display())
        }
    })
}

/// The forge's durable identity, read from this run's private owned repository.
#[derive(Debug, PartialEq, Eq)]
pub struct ForgeIdentity {
    pub number: u64,
    pub url: String,
    pub merge: String,
    pub head: String,
    pub branches: Vec<(String, String)>,
}

pub struct LiveEvidence {
    pub task: String,
    pub reads: super::first_repository::Evidence,
    pub forge: ForgeIdentity,
    pub replies: std::collections::BTreeMap<u64, serde_json::Value>,
    pub delegates: [Vec<u64>; 2],
}

fn forge_identity(address: std::net::SocketAddr, line: &str, token: &str) -> ForgeIdentity {
    use skein_lib::base64::{self, Alphabet};
    let get = |suffix: &str| {
        super::owned::answer(address.port(), &format!("/api/v1/repos/work/hello/{suffix}"), Some(token))
            .unwrap_or_else(|why| panic!("owned forge read {suffix}: {why}"))
    };
    let pulls = get("pulls?state=all&limit=100&page=1");
    let pulls = pulls.as_array().expect("pull list");
    assert_eq!(pulls.len(), 1, "exactly one pull on work/hello");
    assert_eq!(get("pulls?state=all&limit=100&page=2"), serde_json::json!([]));
    let pull = &pulls[0];
    assert_eq!(pull["merged"], true, "the pull merged");
    let head = pull["head"]["sha"].as_str().expect("pull head").to_owned();
    let status = get(&format!("commits/{head}/status"));
    assert_eq!(status["state"], "success", "CI at the merged head");
    assert!(!status["statuses"].as_array().expect("CI jobs").is_empty(), "a CI job ran");
    let readme = get("contents/README.md?ref=main");
    assert_eq!(readme["encoding"], "base64");
    let encoded: Vec<_> = readme["content"]
        .as_str()
        .expect("README content")
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    let readme = base64::decode(Alphabet::Standard, &encoded, 1 << 20).expect("README base64");
    assert_eq!(
        std::str::from_utf8(&readme).expect("README UTF-8").lines().last(),
        Some(line),
        "main ends with the tagged line"
    );
    let branches = get("branches?limit=100&page=1");
    assert_eq!(get("branches?limit=100&page=2"), serde_json::json!([]));
    let mut branches: Vec<_> = branches
        .as_array()
        .expect("branches")
        .iter()
        .map(|branch| {
            (
                branch["name"].as_str().expect("branch name").to_owned(),
                branch["commit"]["id"].as_str().expect("branch head").to_owned(),
            )
        })
        .collect();
    branches.sort();
    let merge = pull["merge_commit_sha"].as_str().expect("merge hash").to_owned();
    assert!(branches.iter().any(|(name, head)| name == "main" && head == &merge), "main is at the pull's merge commit");
    ForgeIdentity {
        number: pull["number"].as_u64().expect("pull number"),
        url: pull["html_url"].as_str().expect("pull URL").to_owned(),
        merge,
        head,
        branches,
    }
}

/// Read the live oracle using a credential minted only for this disposable forge.
#[must_use]
pub fn live_forge(paths: &[PathBuf; 3], first: bool, address: std::net::SocketAddr, line: &str) -> ForgeIdentity {
    let token = oracle_token(paths, first);
    forge_identity(address, line, &token)
}

/// The starter is private. Mint an independent, read-only oracle credential on
/// this disposable forge, rather than reading temper's stored credentials.
fn oracle_token(paths: &[PathBuf; 3], first: bool) -> String {
    use std::{
        io::Read,
        process::{Command, Stdio},
    };
    let forge = paths[1].join("forge-3/forgejo");
    let (binary, _) = super::owned::binaries();
    let mut child = Command::new(binary)
        .arg("--work-path")
        .arg(&forge)
        .arg("--config")
        .arg(forge.join("app.ini"))
        .args([
            "admin",
            "user",
            "generate-access-token",
            "--username",
            "temper",
            "--token-name",
            if first { "live-oracle-first" } else { "live-oracle-restart" },
            "--scopes",
            "read:repository,read:issue",
            "--raw",
        ])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        // The owned forge commands use this same private home.
        .env("HOME", paths[1].join("forge-3/home"))
        .current_dir(&forge)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("oracle token command");
    let clock = skein_shell::Clock::new();
    let until = clock.now().now.saturating_add(Duration::from_secs(10));
    loop {
        if let Some(status) = child.try_wait().expect("oracle token status") {
            assert!(status.success(), "read-only oracle token creation failed");
            break;
        }
        if clock.now().now >= until {
            child.kill().expect("stop expired oracle command");
            child.wait().expect("reap oracle command");
            panic!("oracle token creation exceeded ten seconds");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let mut bytes = Vec::new();
    child.stdout.take().expect("oracle token output").take(1024).read_to_end(&mut bytes).expect("oracle token bytes");
    let token = String::from_utf8(bytes).expect("oracle token encoding");
    let token = token.trim();
    assert_eq!(token.len(), 40, "oracle token length");
    assert!(token.bytes().all(|byte| byte.is_ascii_hexdigit()), "oracle token format");
    token.to_owned()
}

/// Commands are admitted one at a time so a failure cannot start the next act.
fn live_commands(first: bool, words: &str) -> (Vec<Vec<String>>, usize) {
    let mut commands = vec![vec!["sign-in", "-"]];
    if first {
        commands.extend([vec!["forges"], vec!["forges", "--json"], vec!["forge-setup", "3"]]);
    }
    commands.extend([
        vec!["wait", "forges", "--until", "settled", "--timeout", "180s"],
        vec!["forges", "--json"],
        vec!["forges"],
        vec!["me", "--json"],
    ]);
    if first {
        commands.extend([
            vec!["connect", "1", "--key", "connect-plan"],
            vec!["wait", "accounts", "--until", "settled", "--timeout", "600s", "--json"],
        ]);
    }
    let account = commands.len();
    commands.push(vec!["accounts", "--json"]);
    if first {
        commands.extend([
            vec!["chat", "--project", "1", "--model", "gpt-6-luna", "--key", "change-on-plan", words, "--json"],
            vec!["wait", "task", "1", "--until", "replied", "--timeout", "1800s", "--json"],
        ]);
    }
    (commands.into_iter().map(|args| args.into_iter().map(str::to_owned).collect()).collect(), account)
}

pub struct LivePlan<'a> {
    pub paths: &'a [PathBuf; 3],
    pub client: &'a std::path::Path,
    pub first: bool,
    pub words: &'a str,
    pub line: &'a str,
}

/// Only committed assistant text at or after the child's end counts as its reply.
pub fn replies_after(
    activity: &serde_json::Value,
    ended_at: u64,
) -> Result<std::collections::BTreeMap<u64, serde_json::Value>, String> {
    let mut replies = std::collections::BTreeMap::new();
    for row in activity["rows"].as_array().ok_or("chat activity missing")? {
        if row["at"].as_u64().ok_or("activity timestamp missing")? < ended_at {
            continue;
        }
        let Some(turn) = row.pointer("/event/owner/decoded/turn/decoded") else { continue };
        require(turn["cut"] == false, "whole reply turn")?;
        for message in turn["messages"].as_array().ok_or("turn messages missing")? {
            if message["speaker"] == "assistant"
                && message["blocks"].as_array().ok_or("message blocks missing")?.iter().any(|block| {
                    block
                        .pointer("/text/text")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|text| !text.trim().is_empty())
                })
            {
                replies.insert(row["position"].as_u64().ok_or("reply position missing")?, turn.clone());
            }
        }
    }
    Ok(replies)
}

/// Names from actual committed call blocks, never from the model's prose.
#[must_use]
pub fn successful_tools(activity: &serde_json::Value) -> std::collections::BTreeSet<String> {
    let Some(rows) = activity["rows"].as_array() else { return std::collections::BTreeSet::new() };
    committed_tools(rows)
        .filter(|call| call["failed"] == false && !call["outcome"].is_null())
        .filter_map(|call| call["tool"].as_str().map(str::to_owned))
        .collect()
}

fn committed_tools(rows: &[serde_json::Value]) -> impl Iterator<Item = &serde_json::Value> {
    rows.iter()
        .filter_map(|row| {
            row.pointer("/event/owner/decoded/turn/decoded/messages").and_then(serde_json::Value::as_array)
        })
        .flatten()
        .filter(|message| message["speaker"] == "assistant")
        .flat_map(|message| message["blocks"].as_array().into_iter().flatten())
        .filter_map(|block| block.get("call"))
}

fn require(condition: bool, message: &str) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message.to_owned()) }
}
fn live_json(call: &super::Ended) -> Result<serde_json::Value, String> {
    serde_json::from_slice(&call.stdout).map_err(|error| format!("live ctl JSON: {error}"))
}

enum LivePhase {
    Setup,
    Waiting,
    Progress,
    Snapshot,
    Pages,
    Closing,
    Done,
    Stopping,
    Settled,
}
struct LiveJudge<'a> {
    plan: LivePlan<'a>,
    account_at: usize,
    found_at: usize,
    checked: usize,
    pending: std::collections::VecDeque<Vec<String>>,
    arguments: Vec<Vec<String>>,
    active_until: Option<skein_lib::Time>,
    wait_started: Option<skein_lib::Time>,
    wait_until: Option<skein_lib::Time>,
    wait_call: Option<usize>,
    failure: &'a std::cell::Cell<Option<String>>,
    signing_until: Option<skein_lib::Time>,
    account_checked: bool,
    linked: bool,
    cleanup: bool,
    phase: LivePhase,
    snapshot: Option<String>,
    replies: std::collections::BTreeMap<u64, serde_json::Value>,
    tasks_at: usize,
    tools: std::collections::BTreeSet<String>,
    ended_at: Option<u64>,
    pages: usize,
    evidence: &'a std::cell::Cell<Option<LiveEvidence>>,
    deadline: skein_lib::Time,
    poll: skein_lib::Time,
}
impl LiveJudge<'_> {
    fn advance(&mut self, now: skein_lib::Time, calls: &mut super::ctl::Calls) -> Result<(), String> {
        if !calls.foreground_idle() || !self.pending.is_empty() {
            return Ok(());
        }
        let last = live_json(calls.ends().last().ok_or("live read missing")?)?;
        match self.phase {
            LivePhase::Setup => {
                self.wait_started = Some(now);
                self.wait_until = Some(now.saturating_add(Duration::from_secs(1800)));
                self.wait_round(now)?;
            }
            LivePhase::Waiting => {
                self.wait_call = Some(calls.ends().len() - 1);
                if last["met"] == false {
                    self.queue(&["task", "2", "--json"]);
                    self.queue(&["tasks", "--project", "1", "--parent", "2", "--json"]);
                    self.phase = LivePhase::Progress;
                } else {
                    self.found_at = calls.ends().len() - 1;
                    self.queue(&["task", "2"]);
                    self.queue(&["task", "3", "--json"]);
                    self.queue(&["task", "1", "--json"]);
                    self.phase = LivePhase::Snapshot;
                }
            }
            LivePhase::Progress => self.progress(now, calls, &last)?,
            LivePhase::Snapshot => {
                let ended = live_json(&calls.ends()[self.found_at])?;
                require(ended["met"] == true, "change ended within its watch")?;
                self.ended_at = Some(
                    ended["task"]["chip"]["details"]["owner"]["details"]["ended_at"]
                        .as_u64()
                        .ok_or("change ended timestamp missing")?,
                );
                self.snapshot = Some(last["digest"].as_str().ok_or("chat digest missing")?.to_owned());

                self.pages = 0;
                self.queue(&["chat", "1", "--json"]);
                self.phase = LivePhase::Pages;
            }
            LivePhase::Pages => {
                self.pages += 1;
                require(self.pages <= 32, "bounded conversation pages")?;
                let activity = &last["activity"];
                require(activity["cut"] == false, "whole activity page")?;
                self.tools.extend(successful_tools(activity));
                self.replies.extend(replies_after(activity, self.ended_at.ok_or("change end missing")?)?);
                if let Some(before) = activity["before"]["value"].as_str() {
                    self.queue(&["chat", "1", "--before", before, "--json"]);
                } else if self.replies.is_empty() {
                    self.queue(&[
                        "wait",
                        "task",
                        "1",
                        "--until",
                        "replied",
                        "--since",
                        &self.snapshot.clone().ok_or("snapshot before conversation missing")?,
                        "--timeout",
                        "1800s",
                        "--json",
                    ]);
                    self.queue(&["task", "1", "--json"]);
                    self.phase = LivePhase::Snapshot;
                } else {
                    self.tasks_at = calls.ends().len();
                    for parent in ["1", "2"] {
                        self.queue(&["tasks", "--project", "1", "--parent", parent, "--ended", "--json"]);
                    }
                    if !self.plan.first {
                        for args in [
                            &["close", "1", "--key", "close-live-plan", "--json"][..],
                            &["wait", "tree", "1", "--until", "ended", "--timeout", "300s", "--json"],
                            &["tasks", "--project", "1", "--json"],
                        ] {
                            self.queue(args);
                        }
                    }
                    self.phase = LivePhase::Closing;
                }
            }
            LivePhase::Closing => {
                self.check_landing(calls)?;
                calls.release();
                self.phase = LivePhase::Done;
            }
            LivePhase::Done | LivePhase::Stopping | LivePhase::Settled => {}
        }
        Ok(())
    }
    fn progress(
        &mut self,
        now: skein_lib::Time,
        calls: &super::ctl::Calls,
        last: &serde_json::Value,
    ) -> Result<(), String> {
        let task = live_json(&calls.ends()[calls.ends().len() - 2])?;
        let children = last["tasks"]["rows"]
            .as_array()
            .ok_or("children rows missing")?
            .iter()
            .map(|row| format!("{}:{}", row["chip"]["task"], row["chip"]["phase"].as_str().unwrap_or("?")))
            .collect::<Vec<_>>()
            .join(",");
        super::live::notice(&format!(
            "PROGRESS {} task2={} children={} completions={} waits_on={} details={}",
            now.saturating_since(self.wait_started.ok_or("watch start missing")?).as_nanos() / 1_000_000_000,
            task["chip"]["phase"].as_str().unwrap_or("?"),
            children,
            super::live::deployment_completions(self.plan.paths),
            task["waits_on"],
            task["chip"]["details"]
        ));
        if now >= self.wait_until.ok_or("watch deadline missing")? {
            self.fail("change watch exceeded 1800s", calls, self.wait_call);
        } else {
            self.wait_round(now)?;
        }
        Ok(())
    }
    fn stopping(&self) -> bool {
        matches!(self.phase, LivePhase::Stopping | LivePhase::Settled)
    }

    fn check_landing(&mut self, calls: &super::ctl::Calls) -> Result<(), String> {
        let last = live_json(calls.ends().last().ok_or("closing reads missing")?)?;
        if !self.plan.first {
            require(live_json(&calls.ends()[calls.ends().len() - 2])?["met"] == true, "chat tree closed")?;
            require(
                last["tasks"]["rows"] == serde_json::json!([]) && last["tasks"]["more"] == false,
                "no live root tasks",
            )?;
        }
        let reads = super::first_repository::live(&self.plan.paths[0], &self.plan.paths[1], self.plan.client);
        require(self.tools.contains("change"), "the chat's committed change call succeeded")?;
        let mut delegates = [Vec::new(), Vec::new()];
        for (offset, tasks) in delegates.iter_mut().enumerate() {
            let value = live_json(&calls.ends()[self.tasks_at + offset])?;
            require(value["live"] == serde_json::json!([]) && value["live_after"].is_null(), "no live delegates")?;
            let ended = &value["ended"]["rows"];
            require(ended["cut"] == false, "whole delegate set")?;
            require(ended["after"].is_null(), "whole delegate set")?;
            *tasks = ended["rows"]
                .as_array()
                .ok_or("ended delegates missing")?
                .iter()
                .map(|row| row["chip"]["task"].as_u64().ok_or("delegate task missing"))
                .collect::<Result<_, _>>()?;
            tasks.sort_unstable();
        }
        require(delegates[0].contains(&2), "change task belongs to the chat that called change")?;
        require(!self.tools.contains("delegate"), "no raw delegate started the task")?;
        let forge = live_forge(self.plan.paths, self.plan.first, reads.address, self.plan.line);
        self.evidence.set(Some(LiveEvidence {
            task: String::from_utf8_lossy(&calls.ends()[self.found_at + 1].stdout).into_owned(),
            reads,
            forge,
            replies: std::mem::take(&mut self.replies),
            delegates,
        }));
        Ok(())
    }
}
impl LiveJudge<'_> {
    fn queue(&mut self, args: &[&str]) {
        self.pending.push_back(args.iter().map(|arg| (*arg).to_owned()).collect());
    }
    fn wait_round(&mut self, now: skein_lib::Time) -> Result<(), String> {
        let left = self.wait_until.ok_or("watch deadline missing")?.saturating_since(now).as_nanos() / 1_000_000_000;
        self.queue(&[
            "wait",
            "task",
            "2",
            "--until",
            "ended",
            "--timeout",
            &format!("{}s", left.clamp(1, 60)),
            "--json",
        ]);
        self.phase = LivePhase::Waiting;
        Ok(())
    }
    fn dump(&self, index: usize, call: &super::Ended) {
        super::live::notice(&format!(
            "EVIDENCE cwd={} argv=temper ctl client {:?} exit={:?}\nstdout: {}\nstderr: {}",
            self.plan.client.display(),
            self.arguments.get(index),
            call.exit,
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        ));
    }
    fn fail(&mut self, message: &str, calls: &super::ctl::Calls, failing: Option<usize>) {
        let previous = self.failure.take();
        if previous.is_some() {
            self.failure.set(previous);
            return;
        }
        let message = if let Some(index) = failing {
            format!("{message}: argv={:?} exit={:?}", self.arguments.get(index), calls.ends()[index].exit)
        } else {
            message.to_owned()
        };
        self.failure.set(Some(super::live::redact(&message)));
        super::live::notice(&format!("FAILURE {message}"));
        if let Some(index) = failing {
            self.dump(index, &calls.ends()[index]);
        } else if let Some(call) = calls.pending() {
            self.dump(calls.ends().len(), call);
        }
        // Each independent read uses ctl's existing 60s bound. Even an unknown
        // task or a broken read is evidence, never a reason to omit later reads.
        let mut task2 = false;
        for args in [
            &["task", "1", "--json"][..],
            &["task", "2", "--json"],
            &["task", "3", "--json"],
            &["tasks", "--project", "1", "--json"],
        ] {
            if let Some(read) = self.diagnostic(args) {
                task2 |= args == ["task", "2", "--json"] && read.exit == Some(Exit::Code(0));
            }
        }
        let mut before = None::<String>;
        for _ in 0..32 {
            let mut args = vec!["chat", "1", "--json"];
            if let Some(cursor) = &before {
                args.extend(["--before", cursor]);
            }
            let Some(read) = self.diagnostic(&args) else { break };
            let Ok(page) = serde_json::from_slice::<serde_json::Value>(&read.stdout) else { break };
            if let Some(rows) = page["activity"]["rows"].as_array() {
                for call in committed_tools(rows) {
                    super::live::notice(&format!(
                        "TOOL name={} failed={} outcome={}",
                        call["tool"], call["failed"], call["outcome"]
                    ));
                }
            }
            before = page["activity"]["before"]["value"].as_str().map(str::to_owned);
            if before.is_none() {
                break;
            }
        }
        if task2 {
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                super::first_repository::change_page(&self.plan.paths[0], self.plan.client)
            })) {
                Ok(page) => super::live::notice(&format!("CHANGE PAGE\n{page}")),
                Err(_) => super::live::notice("CHANGE PAGE unavailable"),
            }
        }
        self.pending.clear();
        self.phase = LivePhase::Done;
    }
    fn diagnostic(&self, args: &[&str]) -> Option<super::Ended> {
        if let Ok(read) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::ctl::alone(self.plan.client, super::ctl::Call::new(args))
        })) {
            // Do not copy conversation prose or tool arguments into notices.
            let stdout = if args.first() == Some(&"chat") {
                "(committed tools below)".into()
            } else {
                String::from_utf8_lossy(&read.stdout)
            };
            super::live::notice(&format!(
                "EVIDENCE cwd={} argv=temper ctl client {args:?} exit={:?}\nstdout: {stdout}\nstderr: {}",
                self.plan.client.display(),
                read.exit,
                String::from_utf8_lossy(&read.stderr)
            ));
            Some(read)
        } else {
            super::live::notice(&format!("EVIDENCE {args:?}: bounded read unavailable"));
            None
        }
    }
    fn drive(
        &mut self,
        now: skein_lib::Time,
        binary: &mut skein_world::end_to_end::Binary,
        calls: &mut super::ctl::Calls,
    ) {
        use skein_shell::Host;
        calls.observe_box_stderr(binary.stderr(), now);
        if !self.linked && super::printed_link(calls.stdout()).is_some() {
            super::live::client_notice(&String::from_utf8_lossy(calls.link()));
            self.linked = true;
        }
        for index in self.checked..calls.ends().len() {
            let call = &calls.ends()[index];
            let not_yet = matches!(self.phase, LivePhase::Waiting) && call.exit == Some(Exit::Code(3));
            if call.exit != Some(Exit::Code(0)) && !not_yet {
                self.fail("live ctl failed", calls, Some(index));
                break;
            }
            if self.plan.first && index == self.account_at - 2 {
                super::live::notice(&String::from_utf8_lossy(&call.stdout));
                self.signing_until = Some(now.saturating_add(Duration::from_secs(600)));
            }
        }
        if calls.ends().len() > self.checked {
            self.active_until = None;
        }
        self.checked = calls.ends().len();
        if !matches!(self.phase, LivePhase::Done | LivePhase::Stopping | LivePhase::Settled)
            && !self.account_checked
            && calls.ends().len() > self.account_at
        {
            match live_json(&calls.ends()[self.account_at]).and_then(|value| super::live::ready_account(&value)) {
                Ok(()) => self.account_checked = true,
                Err(message) => self.fail(&message, calls, Some(self.account_at)),
            }
        }
        if !self.stopping() && !matches!(self.phase, LivePhase::Done) {
            if self.signing_until.is_some_and(|until| !self.account_checked && now >= until) {
                self.fail("ChatGPT sign-in exceeded 10 minutes", calls, None);
            } else if now >= self.deadline {
                self.fail("live story exceeded its budget", calls, None);
            } else if binary.exit_status().is_some() {
                self.fail("live box stopped before its reads", calls, None);
            } else if self.active_until.is_some_and(|until| now >= until) {
                self.fail("live ctl exceeded its per-call bound", calls, None);
            } else if now >= self.poll {
                if super::live::deployment_completions(self.plan.paths) > 25 {
                    self.fail("live completion budget exceeded", calls, None);
                }
                self.poll = now.saturating_add(Duration::from_secs(1));
            }
        }
        if !self.stopping() && !matches!(self.phase, LivePhase::Done) {
            if let Err(message) = self.advance(now, calls) {
                self.fail(&message, calls, calls.ends().len().checked_sub(1));
            }
            if calls.foreground_idle()
                && let Some(args) = self.pending.pop_front()
            {
                let seconds = args
                    .windows(2)
                    .find(|pair| pair[0] == "--timeout")
                    .and_then(|pair| pair[1].strip_suffix('s'))
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
                self.active_until = Some(now.saturating_add(Duration::from_secs(seconds + 60)));
                let words = args.iter().map(String::as_str).collect::<Vec<_>>();
                calls.push(super::ctl::Step::Run(if words == ["sign-in", "-"] {
                    super::ctl::Call::sign_in()
                } else {
                    super::ctl::Call::new(&words)
                }));
                self.arguments.push(args);
            }
        }
        if !self.stopping() && matches!(self.phase, LivePhase::Done) {
            calls.release();
            calls.cancel();
            binary.signal(skein_io::kernel::Signal::Terminate);
            self.phase = LivePhase::Stopping;
        }
        if !self.cleanup && now >= self.deadline && !binary.is_empty() {
            binary.signal_tree(skein_io::kernel::Signal::Kill);
            self.cleanup = true;
        }
    }
}
impl Referee<Process> for LiveJudge<'_> {
    fn act(&mut self, now: skein_lib::Time, procs: &mut [Process]) {
        let [Process::Binary(binary), Process::Calls(calls), ..] = procs else { return };
        // Legacy oracle assertions must never unwind through the world's Tree.
        // Their recorded failure is raised by the caller after run_live returns.
        if let Err(failed) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.drive(now, binary, calls))) {
            let message = failed
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| failed.downcast_ref::<&str>().copied())
                .unwrap_or("live referee validation failed");
            self.fail(message, calls, calls.ends().len().checked_sub(1));
            calls.release();
            calls.cancel();
            binary.signal(skein_io::kernel::Signal::Terminate);
            self.phase = LivePhase::Stopping;
        }
    }
    fn observe(&mut self, _now: skein_lib::Time, procs: &[Process]) {
        use skein_shell::Host;
        if self.stopping() && procs.iter().all(Host::is_empty) {
            self.phase = LivePhase::Settled;
        }
    }
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        if matches!(self.phase, LivePhase::Settled) || self.cleanup {
            None
        } else if self.stopping() {
            Some(self.deadline)
        } else {
            Some(self.poll.min(self.deadline))
        }
    }
    fn overdue(&self, _now: skein_lib::Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        matches!(self.phase, LivePhase::Settled)
    }
}

/// The live face has only the supervised box and ctl, no issuer, visitor or peer.
pub fn run_live(
    plan: LivePlan<'_>,
    budget: Duration,
    face: &Face,
) -> Result<super::boxed::CtlOutcome<LiveEvidence>, String> {
    use skein_shell::{Clock, Host};
    use skein_world::end_to_end::{Binary, Command, Expectation, Mode};
    assert!(matches!(face, Face::Live), "live driver takes the live face");
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        Command {
            program: env!("CARGO_BIN_EXE_temper").into(),
            arguments: std::iter::once("box".into())
                .chain(deployment_flags(plan.paths).into_iter().map(Into::into))
                .collect(),
            directory: plan.paths[0].parent().expect("scratch root").into(),
            environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        },
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("live box starts");
    let streams = binary.take_streams().expect("box pipes");
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(plan.client).expect("fresh client root");
    fds.push(root);
    let (commands, account_at) = live_commands(plan.first, plan.words);
    let failure = std::cell::Cell::new(None);
    let evidence = std::cell::Cell::new(None);
    let mut world = real::World::new(LiveJudge {
        plan,
        account_at,
        found_at: 0,
        checked: 0,
        pending: commands.into(),
        arguments: Vec::new(),
        active_until: None,
        wait_started: None,
        wait_until: None,
        wait_call: None,
        failure: &failure,
        signing_until: None,
        account_checked: false,
        linked: false,
        cleanup: false,
        phase: LivePhase::Setup,
        snapshot: None,
        replies: std::collections::BTreeMap::new(),
        tasks_at: 0,
        tools: std::collections::BTreeSet::new(),
        ended_at: None,
        pages: 0,
        evidence: &evidence,
        deadline: clock
            .now()
            .now
            .saturating_add(Duration::from_nanos(budget.as_nanos().saturating_sub(Duration::from_secs(10).as_nanos()))),
        poll: clock.now().now,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        let mut calls = super::ctl::Calls::new(root, vec![], Some(&streams), None).with_box_stderr();
        calls.hold();
        Process::Calls(Box::new(calls))
    });
    // Failure diagnostics: at most 36 ctl reads (60s each), a 20s native
    // change-page read, and ten seconds to cancel, drain and reap afterwards.
    let drain_budget = Duration::from_secs(36 * 60 + 30);
    let ended = world.run(&clock, Duration::from_nanos(budget.as_nanos().saturating_add(drain_budget.as_nanos())));
    if !ended.procs.iter().all(Host::is_empty) {
        return Err("live box, descendants and ctl did not drain".into());
    }
    let [Process::Binary(binary), Process::Calls(calls)] = ended.procs.as_slice() else { panic!("live box and ctl") };
    if let Some(message) = failure.take() {
        return Err(message);
    }
    Ok(super::boxed::CtlOutcome {
        exit: binary.exit_status(),
        stdout: calls.stdout().to_vec(),
        stderr: binary.stderr().to_vec(),
        left: binary.tree_status().live_at_leader_exit.clone(),
        calls: calls
            .ends()
            .iter()
            .map(|call| super::Ended { exit: call.exit, stdout: call.stdout.clone(), stderr: call.stderr.clone() })
            .collect(),
        checked: evidence.take(),
    })
}
