//! A person's change on the owned forge, signed in with `ChatGPT` (domain/testing.md, 2.6).
use super::{Process, sign_in};
use skein_fake_llm_domain::api::{self, Finish, Line, Script, Turn};
use skein_fake_peers::oauth;
use skein_world::{Referee, real};
use std::{fs, net::SocketAddr, os::unix::fs::PermissionsExt, path::PathBuf};

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
    /// The forge API address used in the grants' segments.
    pub forge: SocketAddr,
    /// The repository's organization (`work`), the grants' `last`.
    pub org: &'a str,
    /// README.md on main before the change.
    pub readme: &'a [u8],
}

/// Delegate, wait, reply as an Open chat, then answer its person's Close.
#[must_use]
pub fn plan_scripts(plan: &Plan<'_>) -> Box<[Script]> {
    const REVIEW: &[u8] = b"Review this change at the named head";
    let line = std::str::from_utf8(plan.line).expect("README line UTF-8");
    let coding = format!("@coding Append this README line: {line}");
    assert!(!plan.words.is_empty(), "nonempty chat cue");
    assert!(!plan.line.is_empty() && !plan.line.contains(&b'\n'), "one nonempty README line");
    assert!(plan.words.windows(plan.line.len()).any(|part| part == plan.line), "request carries the line");
    // System cues match the earliest byte position. Each task's original spec
    // supplies its cue before any conversation history. Do not let the coder's
    // or review's spec also carry the chat cue (or the review cue in the coder).
    assert!(!coding.as_bytes().windows(plan.words.len()).any(|part| part == plan.words), "distinct chat cue");
    assert!(!REVIEW.windows(plan.words.len()).any(|part| part == plan.words), "distinct review cue");
    assert!(!coding.as_bytes().windows(REVIEW.len()).any(|part| part == REVIEW), "distinct coder cue");
    let edit = format!("{line}\n");
    let mut scripts = temper_real_loop_world::landing::change_scripts(
        plan.forge,
        plan.repository,
        plan.readme,
        edit.as_bytes(),
        plan.org,
        plan.words,
        &coding,
    );
    let text =
        |bytes: &[u8]| Turn { lines: Box::new([Line::Text { text: bytes.into() }]), finish: Finish::Stop, tokens: 20 };
    let chat = &mut scripts[0];
    chat.turns[2] = text(b"The change landed.");
    let mut turns = std::mem::take(&mut chat.turns).into_vec();
    turns.push(text(b"The plan chat is now closed."));
    chat.turns = turns.into_boxed_slice();
    scripts
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
