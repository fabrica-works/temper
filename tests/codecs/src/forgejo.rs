//! Raw Forgejo transcripts and a bounded tokenizer driver.
use skein_json::tokenizer;
use skein_lib::{
    Env, Intake, Queue, Time, Wall,
    stream::{Down, Up},
};
use std::{collections::BTreeSet, fs, path::Path};
use temper_forge_forgejo::{
    client,
    decode::{Malformed, Step},
    document::{Document, Row},
    limits::Limits,
    request::{Method, Page, Repo, Request},
    server,
};
#[derive(Debug)]
pub struct Transcript {
    pub method: Method,
    pub target: Vec<u8>,
    pub request_body: Vec<u8>,
    pub status: u16,
    pub body: Vec<u8>,
}
#[must_use]
pub fn transcript(case: &str) -> Transcript {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/transcripts").join(case);
    let request = fs::read(root.join("request")).expect("request transcript");
    let response = fs::read(root.join("response")).expect("response transcript");
    let line = request.split(|b| *b == b'\r').next().expect("request line");
    let mut words = line.split(|b| *b == b' ');
    let method = match words.next().expect("method") {
        b"GET" => Method::Get,
        b"POST" => Method::Post,
        b"PATCH" => Method::Patch,
        b"DELETE" => Method::Delete,
        other => panic!("method {other:?}"),
    };
    let target = words.next().expect("target").to_vec();
    let status = std::str::from_utf8(response.split(|b| *b == b' ').nth(1).expect("status"))
        .expect("status text")
        .parse()
        .expect("status number");
    Transcript { method, target, request_body: body(&request), status, body: body(&response) }
}
fn body(bytes: &[u8]) -> Vec<u8> {
    let at = bytes.windows(4).position(|w| w == b"\r\n\r\n").expect("HTTP blank line");
    bytes[at + 4..].to_vec()
}
#[must_use]
pub fn limits() -> Limits {
    Limits {
        json: tokenizer::Limits { depth: 16, string: 65_536, number: 32, chunk: 4_096, length: 4_194_304 },
        writer: skein_json::writer::Limits { depth: 16, length: 4_194_304 },
        string_bytes: 1024,
        body_bytes: 65_536,
        inner: 64,
        target_bytes: 4_096,
    }
}
#[derive(Debug)]
pub struct Decoded {
    pub rows: Vec<Row>,
    pub document: Result<Document, Malformed>,
    pub steps: usize,
}
#[must_use]
pub fn decode(request: &Request, body: &[u8]) -> Decoded {
    let limits = limits();
    let decoder = client::decoder(request, &limits).expect("JSON answer");
    decode_answer(decoder, body, &limits)
}
#[must_use]
pub fn decode_answer(mut decoder: client::Decoder, body: &[u8], limits: &Limits) -> Decoded {
    let mut rows = Vec::new();
    let mut document = None;
    let first = client::first(&decoder);
    let steps = drive(body, limits, first, |event| match client::step(&mut decoder, limits, event) {
        Step::Demand(next) => Some(next),
        Step::Row(row, next) => {
            rows.push(row);
            Some(next)
        }
        Step::Done(value) => {
            document = Some(Ok(value));
            None
        }
        Step::Malformed(error) => {
            document = Some(Err(error));
            None
        }
    });
    Decoded { rows, document: document.expect("terminal decoder"), steps }
}
#[must_use]
pub fn route(transcript: &Transcript) -> Request {
    decode_route(transcript.method, &transcript.target, &transcript.request_body, &limits())
        .expect("transcript request")
        .0
}
/// # Errors
/// Returns the route or body refusal for malformed input.
pub fn decode_route(method: Method, target: &[u8], body: &[u8], limits: &Limits) -> Result<(Request, usize), String> {
    match server::route(method, target, limits) {
        Err(error) => Err(format!("route: {error:?}")),
        Ok(server::Route::Ready(request)) => Ok((request, 0)),
        Ok(server::Route::Body(mut decoder)) => {
            let mut result = None;
            let first = server::body_first(&decoder);
            let steps = drive(body, limits, first, |event| match server::body(&mut decoder, limits, event) {
                server::BodyStep::Demand(next) => Some(next),
                server::BodyStep::Decoded(request) => {
                    result = Some(Ok(request));
                    None
                }
                server::BodyStep::Malformed(error) => {
                    result = Some(Err(format!("body: {error:?}")));
                    None
                }
            });
            result.expect("terminal body decoder").map(|request| (request, steps))
        }
    }
}
fn drive(
    body: &[u8],
    limits: &Limits,
    mut demand: tokenizer::Request,
    mut event: impl FnMut(&tokenizer::Event) -> Option<tokenizer::Request>,
) -> usize {
    // No decoded string can exceed the complete input, including malformed
    // mutations. Keep the configured bound while reserving only useful room.
    let mut json = limits.json;
    json.string = json.string.min(u32::try_from(body.len()).expect("body fits"));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: json };
    let mut tokenizer = tokenizer::Tokenizer::new(&json);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut intake = Intake::with_capacity(
        u32::try_from(body.len()).expect("body fits").max(tokenizer::largest_demand(&limits.json)),
    );
    intake.append(body).expect("input fits");
    let mut read = None;
    for steps in 1..=body.len() * 4 + 32 {
        match read.take() {
            Some(read) => {
                let up = match intake.meet(read) {
                    Some(bytes) => Up::Bytes(bytes),
                    None => Up::End,
                };
                tokenizer::up(&mut tokenizer, &env, up, &mut above, &mut below);
            }
            None => tokenizer::down(&mut tokenizer, &env, demand, &mut above, &mut below),
        }
        match below.pop() {
            Some(Down::Demand { read: r, room: 0 }) => read = Some(r),
            None => {}
            Some(other) => panic!("unexpected demand {other:?}"),
        }
        if let Some(token) = above.pop() {
            match event(&token) {
                Some(next) => demand = next,
                None => return steps,
            }
        }
    }
    panic!("decoder exhausted its input step bound")
}
#[must_use]
pub fn specimen() -> Repo {
    Repo { owner: Box::from(*b"fixture"), name: Box::from(*b"specimen") }
}
#[must_use]
pub fn sha(n: u64) -> temper_forge_forgejo::id::ObjectId {
    temper_forge_forgejo::id::read(format!("{n:040x}").as_bytes()).expect("masked sha")
}
#[must_use]
pub fn page(page: u32, limit: u32) -> Page {
    Page { page: Some(page), limit: Some(limit) }
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Kind {
    Object,
    Array,
    Number,
    String,
    Bool,
    Null,
}
#[must_use]
pub fn shape(json: &[u8]) -> BTreeSet<(String, Kind)> {
    let value: serde_json::Value = serde_json::from_slice(json).expect("JSON shape");
    let mut set = BTreeSet::new();
    walk(&value, "", &mut set);
    set
}
fn walk(value: &serde_json::Value, path: &str, set: &mut BTreeSet<(String, Kind)>) {
    let kind = match value {
        serde_json::Value::Object(object) => {
            for (key, child) in object {
                let at = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };
                walk(child, &at, set);
            }
            Kind::Object
        }
        serde_json::Value::Array(array) => {
            for child in array {
                walk(child, &format!("{path}[]"), set);
            }
            Kind::Array
        }
        serde_json::Value::Number(_) => Kind::Number,
        serde_json::Value::String(_) => Kind::String,
        serde_json::Value::Bool(_) => Kind::Bool,
        serde_json::Value::Null => Kind::Null,
    };
    set.insert((path.to_string(), kind));
}

#[must_use]
pub fn user(id: u64) -> temper_forge_forgejo::document::User {
    temper_forge_forgejo::document::User {
        id,
        login: Box::from(if id == 1 { b"fixture".as_slice() } else { b"writer" }),
    }
}
#[must_use]
pub fn write_cases() -> Vec<(&'static str, Request)> {
    use temper_forge_forgejo::document::IssueState;
    let mut rows = vec![
        (
            "create-issue",
            Request::CreateIssue {
                at: specimen(),
                title: Box::from(*b"fixture issue"),
                body: Box::from(*b"<!-- temper:key 01000001 -->\nissue text"),
            },
        ),
        (
            "open-pull",
            Request::OpenPull {
                at: specimen(),
                head: Box::from(*b"writer-head"),
                base: Box::from(*b"landing"),
                title: Box::from(*b"writer pull"),
                body: Box::from(*b"writer text"),
            },
        ),
        (
            "open-pull-no-changes",
            Request::OpenPull {
                at: specimen(),
                head: Box::from(*b"no-changes"),
                base: Box::from(*b"main"),
                title: Box::from(*b"empty"),
                body: Box::from(*b"text"),
            },
        ),
        (
            "post-comment",
            Request::PostComment {
                at: specimen(),
                number: 3,
                body: Box::from(*b"<!-- temper:key 01000002 -->\ncomment text"),
            },
        ),
        ("post-comment-plain", Request::PostComment { at: specimen(), number: 3, body: Box::from(*b"plain comment") }),
        (
            "edit-issue-title",
            Request::EditIssue {
                at: specimen(),
                number: 3,
                title: Some(Box::from(*b"edited title")),
                body: None,
                state: None,
            },
        ),
        (
            "close-issue",
            Request::EditIssue { at: specimen(), number: 3, title: None, body: None, state: Some(IssueState::Closed) },
        ),
        (
            "reopen-issue",
            Request::EditIssue { at: specimen(), number: 3, title: None, body: None, state: Some(IssueState::Open) },
        ),
    ];
    rows.extend(write_other_cases());
    rows
}
fn write_other_cases() -> Vec<(&'static str, Request)> {
    use temper_forge_forgejo::{
        document::{MergeStyle, StatusState},
        request::{ReviewEvent, UpdateStyle},
    };
    let mut rows = vec![
        (
            "request-reviewers",
            Request::RequestReviewers {
                at: specimen(),
                number: 1,
                reviewers: Box::from([Box::<[u8]>::from(*b"stranger")]),
            },
        ),
        (
            "remove-reviewers",
            Request::RemoveReviewers {
                at: specimen(),
                number: 1,
                reviewers: Box::from([Box::<[u8]>::from(*b"stranger")]),
            },
        ),
        ("merge-pull", Request::Merge { at: specimen(), number: 5, style: MergeStyle::Merge, head: sha(5) }),
        ("update-pull", Request::Update { at: specimen(), number: 4, style: Some(UpdateStyle::Merge) }),
        (
            "post-status",
            Request::PostStatus {
                at: specimen(),
                commit: sha(4),
                state: StatusState::Success,
                context: Box::from(*b"fixture/test"),
            },
        ),
        (
            "post-status-pending",
            Request::PostStatus {
                at: specimen(),
                commit: sha(4),
                state: StatusState::Pending,
                context: Box::from(*b"fixture/other"),
            },
        ),
        ("create-branch", Request::CreateBranch { at: specimen(), branch: Box::from(*b"recorded"), from: sha(4) }),
        ("delete-branch", Request::DeleteBranch { at: specimen(), branch: Box::from(*b"recorded") }),
    ];
    for (case, event, key) in [
        ("post-review-approved", ReviewEvent::Approved, "01000003"),
        ("post-review-request-changes", ReviewEvent::RequestChanges, "01000004"),
        ("post-review-comment", ReviewEvent::Comment, "01000005"),
    ] {
        rows.push((
            case,
            Request::PostReview {
                at: specimen(),
                number: 1,
                event,
                body: format!("<!-- temper:key {key} -->\nreview text").into_bytes().into_boxed_slice(),
            },
        ));
    }
    rows
}
/// A global bound above every committed JSON transcript's tokenizer steps.
pub const STEP_BOUND: usize = 1_000_000;
#[must_use]
pub fn decode_error(body: &[u8]) -> Decoded {
    let limits = limits();
    decode_answer(client::error_decoder(&limits), body, &limits)
}
/// JSON transcripts with their request, or none when their status is an error.
/// Unread write answers also use the error decoder as a bounded JSON shape check.
#[must_use]
pub fn json_cases() -> Vec<(String, Option<Request>)> {
    let mut cases = vec![];
    for case in transcript_cases() {
        let transcript = transcript(&case);
        if serde_json::from_slice::<serde_json::Value>(&transcript.body).is_err() {
            continue;
        }
        let request = if transcript.status < 300 {
            let request = route(&transcript);
            if client::decoder(&request, &limits()).is_some() { Some(request) } else { None }
        } else {
            None
        };
        cases.push((case, request));
    }
    cases
}
#[must_use]
pub fn transcript_cases() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/forgejo-conformance/transcripts");
    let mut entries: Vec<_> = fs::read_dir(root)
        .expect("transcripts directory")
        .map(|entry| entry.expect("transcript entry"))
        .filter(|entry| entry.file_type().expect("entry type").is_dir())
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    entries.into_iter().map(|entry| entry.file_name().into_string().expect("transcript name")).collect()
}
