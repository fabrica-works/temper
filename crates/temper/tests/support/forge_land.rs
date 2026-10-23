//! A person's change on the owned forge, signed in with `ChatGPT` (domain/testing.md, 2.6).
use super::{Process, sign_in};
use skein_fake_llm_domain::api::{self, Finish, Line, Script, Turn};
use skein_fake_peers::oauth;
use skein_world::{Referee, real};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

/// The model side: processes a story adds beside the box and ctl.
pub enum Face {
    /// Skein's fake issuer and the existing TLS LLM peer playing the plan's scripts.
    Fake { issuer: oauth::Peer, peer: skein_fake_peers::llm::Peer },
}

/// Initialize three fresh private directories, then patch the config for the fakes.
#[must_use]
pub fn fake_deployment(paths: &[PathBuf; 3]) -> oauth::Peer {
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
    sign_in::patch_configuration(&paths[0])
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
