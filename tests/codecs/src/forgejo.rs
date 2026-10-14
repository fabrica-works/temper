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
    let mut decoder = client::decoder(request, &limits).expect("JSON answer");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.json };
    let mut tokenizer = tokenizer::Tokenizer::new(&limits.json);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut intake = Intake::with_capacity(
        u32::try_from(body.len()).expect("body fits").max(tokenizer::largest_demand(&limits.json)),
    );
    intake.append(body).expect("input fits");
    let mut rows = Vec::new();
    let mut demand = client::first(&decoder);
    let mut read = None;
    for steps in 1..=body.len() * 4 + 32 {
        match read.take() {
            Some(read) => {
                let event = match intake.meet(read) {
                    Some(bytes) => Up::Bytes(bytes),
                    None => Up::End,
                };
                tokenizer::up(&mut tokenizer, &env, event, &mut above, &mut below);
            }
            None => tokenizer::down(&mut tokenizer, &env, demand, &mut above, &mut below),
        }
        match below.pop() {
            Some(Down::Demand { read: r, room: 0 }) => read = Some(r),
            None => {}
            Some(other) => panic!("unexpected demand {other:?}"),
        }
        if let Some(event) = above.pop() {
            match client::step(&mut decoder, &limits, &event) {
                Step::Demand(next) => demand = next,
                Step::Row(row, next) => {
                    rows.push(row);
                    demand = next;
                }
                Step::Done(document) => return Decoded { rows, document: Ok(document), steps },
                Step::Malformed(error) => return Decoded { rows, document: Err(error), steps },
            }
        }
    }
    panic!("decoder exhausted its input step bound")
}
#[must_use]
pub fn route(transcript: &Transcript) -> Request {
    let server::Route::Ready(request) =
        server::route(transcript.method, &transcript.target, &limits()).expect("transcript route");
    request
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
