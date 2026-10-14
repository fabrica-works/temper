//! Forge connection connection (forge.md, sections 3 and 8).
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};
use temper_forge_forgejo::{decode, document, header};

/// A forge connection's state; the component arms its deadline.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    /// Nothing held.
    Closed,
    /// io's connect, until the call's deadline (or a start read's).
    Connecting,
    /// TLS's handshake, until the same deadline.
    Handshaking,
    /// The request going down; `written` once its first byte went.
    Sending { written: bool },
    /// The head, then the body as the decoder demands.
    Reading,
    /// Kept for its forge until `idle`.
    Idle,
}

/// The answer's fields the plans read beside its body.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fields {
    /// `Date`: the time the answer was made.
    pub date: Option<Wall>,
    /// `Retry-After` of a `429`, from `Date` when it is a date.
    pub retry_after: Option<Duration>,
    /// `X-HasMore`.
    pub has_more: Option<bool>,
    /// `Content-Range` of a `206`.
    pub content_range: Option<header::ContentRange>,
    /// `Connection: close`: Closed after this answer, not Idle.
    pub close: bool,
}

/// What the body was.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Body {
    /// A JSON answer, whole; its rows went up before it.
    Document(document::Document),
    /// A raw answer within its cap; `more` when bytes followed the cap.
    Raw { bytes: Box<[u8]>, more: bool },
    /// No body, or one not read (a failure's status needs no message).
    Empty,
    /// Not the document its request gets: the connection closes.
    Malformed(decode::Malformed),
}

/// Why an exchange failed (forge.md, 3.4, "Unavailable is provable only before the first byte").
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Failure {
    /// The server never saw the request: no byte written (a refused connect, a failed
    /// handshake, the deadline while waiting), or skein-http's `Closed`: `Unavailable`.
    Unseen,
    /// It may have been read: a reset, the stream's end, `Truncated`, `Stream`, the
    /// deadline in flight, an answer HTTP refuses: `Timeout`.
    Seen,
}

/// What a connection tells the call it serves, in order: `Head`, `Row`s, `Done`; or `Failed`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Heard {
    Head { status: u16, fields: Fields },
    Row(document::Row),
    Done(Body),
    Failed(Failure),
}

use crate::{
    limits::{self, Limits, Shared},
    settings::{Forge, Transport},
};
use skein_http::client as http;
use skein_json::tokenizer;
use skein_lib::{
    Env, List, Queue, Time, Token, Writer, bytes,
    stream::{Down, Up},
};
use temper_forge_forgejo::{client as documents, request};
/// One exchange's response body state.
#[derive(Debug)]
enum Receiving {
    Json(documents::Decoder),
    Empty,
}
/// One connection's machines, socket identity, request and pending output.
pub(crate) struct Connection {
    pub(crate) forge: u16,
    pub(crate) phase: Phase,
    pub(crate) deadline: Option<Time>,
    pub(crate) socket: Option<Token>,
    pub(crate) closing: bool,
    pub(crate) written: bool,
    http: http::Client,
    tls: Option<skein_tls::client::Client>,
    json: tokenizer::Tokenizer,
    shared: Shared,
    http_up: Queue<http::Event>,
    http_down: Queue<Down>,
    tls_up: Queue<skein_tls::client::Event>,
    tls_down: Queue<Down>,
    json_up: Queue<tokenizer::Event>,
    json_down: Queue<Down>,
    heard: Queue<Heard>,
    request: Option<request::Request>,
    receive: Receiving,
    done: Option<Body>,
    after: u64,
}
impl Connection {
    pub fn new(forge: &Forge, l: &Limits, shared: Shared, deadline: Time) -> Self {
        let tls = match &forge.transport {
            Transport::Plaintext => None,
            Transport::Tls { trust, server_name } => {
                Some(skein_tls::client::Client::new(trust, server_name.clone(), &l.tls))
            }
        };
        Self {
            forge: forge.number,
            phase: Phase::Connecting,
            deadline: Some(deadline),
            socket: None,
            closing: false,
            written: false,
            http: http::Client::new(&limits::http(l, tls.is_some(), &shared.io)),
            tls,
            json: tokenizer::Tokenizer::new(&limits::documents(l).json),
            shared,
            http_up: Queue::with_capacity(8),
            http_down: Queue::with_capacity(8),
            tls_up: Queue::with_capacity(8),
            tls_down: Queue::with_capacity(8),
            json_up: Queue::with_capacity(8),
            json_down: Queue::with_capacity(8),
            heard: Queue::with_capacity(16),
            request: None,
            receive: Receiving::Empty,
            done: None,
            after: 0,
        }
    }
    pub fn start(
        &mut self,
        env: &Env<Limits>,
        forge: &Forge,
        request: request::Request,
        range: Option<u32>,
    ) -> Result<(), documents::Refusal> {
        let encoded = documents::encode(&request, &limits::documents(&env.limits))?;
        if encoded.body.is_some() {
            return Err(documents::Refusal::TooLong);
        }
        let mut headers = List::with_capacity(5);
        headers.push(field(b"Host", &forge.host)).expect("five header fields");
        let mut auth = Writer::new(forge.token.len().saturating_add(6));
        auth.put(b"token ").expect("prefix measured");
        auth.put(forge.token.expose()).expect("token measured");
        headers
            .push(skein_http::Header { name: Box::from(&b"Authorization"[..]), value: auth.finish() })
            .expect("five header fields");
        headers.push(field(b"User-Agent", crate::conventions::USER_AGENT.as_bytes())).expect("five header fields");
        if let Some(max) = range {
            let decimal = skein_lib::Decimal::of(u64::from(max));
            let mut value = Writer::new(decimal.as_bytes().len().saturating_add(8));
            value.put(b"bytes=0-").expect("range prefix measured");
            value.put(decimal.as_bytes()).expect("range measured");
            headers
                .push(skein_http::Header { name: Box::from(&b"Range"[..]), value: value.finish() })
                .expect("five header fields");
        }
        self.request = Some(request);
        self.done = None;
        self.written = false;
        self.after = 0;
        self.phase = Phase::Sending { written: false };
        let method = match encoded.method {
            request::Method::Get => skein_http::Method::Get,
            request::Method::Post => skein_http::Method::Post,
            request::Method::Patch => skein_http::Method::Patch,
            request::Method::Delete => skein_http::Method::Delete,
        };
        self.http_down(
            env,
            http::Request::Call(http::Call {
                method,
                target: encoded.target,
                headers: headers.into_boxed(),
                body: http::Body::None,
                close: false,
            }),
        );
        Ok(())
    }
    pub fn connect(&mut self, env: &Env<Limits>) {
        match &mut self.tls {
            Some(tls) => {
                self.phase = Phase::Handshaking;
                let tls_env = Env { now: env.now, wall: env.wall, limits: env.limits.tls };
                skein_tls::client::down(
                    tls,
                    &tls_env,
                    skein_tls::client::Request::Handshake,
                    &mut self.tls_up,
                    &mut self.tls_down,
                );
            }
            None => {
                self.phase = Phase::Idle;
                self.deadline = None;
            }
        }
    }
    pub fn stream(&mut self, env: &Env<Limits>, event: Up) {
        if self.closing || self.phase == Phase::Closed {
            return;
        }
        match &mut self.tls {
            Some(tls) => {
                let tls_env = Env { now: env.now, wall: env.wall, limits: env.limits.tls };
                skein_tls::client::up(tls, &tls_env, event, &mut self.tls_up, &mut self.tls_down);
            }
            None => {
                let h = self.http_env(env);
                http::up(&mut self.http, &h, event, &mut self.http_up, &mut self.http_down);
            }
        }
    }
    fn http_env(&self, env: &Env<Limits>) -> Env<http::Limits> {
        Env { now: env.now, wall: env.wall, limits: limits::http(&env.limits, self.tls.is_some(), &self.shared.io) }
    }
    fn http_down(&mut self, env: &Env<Limits>, request: http::Request) {
        let h = self.http_env(env);
        http::down(&mut self.http, &h, request, &mut self.http_up, &mut self.http_down);
    }
    fn json_down(&mut self, env: &Env<Limits>, request: tokenizer::Request) {
        let j = Env { now: env.now, wall: env.wall, limits: limits::documents(&env.limits).json };
        tokenizer::down(&mut self.json, &j, request, &mut self.json_up, &mut self.json_down);
    }
    /// Routes at most a fixed number of child-machine movements. Remaining work stays owned.
    pub fn pump(&mut self, env: &Env<Limits>, io: &mut Queue<skein_io::Request>, budget: u32) {
        let mut emitted = 0_u32;
        for _ in 0_u32..64 {
            if emitted >= budget || !self.heard.is_empty() || self.closing || self.phase == Phase::Closed {
                break;
            }
            if let Some(down) = self.tls_down.pop() {
                self.send(down, io);
                emitted = emitted.saturating_add(1);
                continue;
            }
            if let Some(event) = self.tls_up.pop() {
                self.tls_event(env, event);
                continue;
            }
            if let Some(down) = self.http_down.pop() {
                match &mut self.tls {
                    Some(tls) => {
                        let e = Env { now: env.now, wall: env.wall, limits: env.limits.tls };
                        skein_tls::client::down(
                            tls,
                            &e,
                            skein_tls::client::Request::Stream(down),
                            &mut self.tls_up,
                            &mut self.tls_down,
                        );
                    }
                    None => {
                        self.send(down, io);
                        emitted = emitted.saturating_add(1);
                    }
                }
                continue;
            }
            if let Some(down) = self.json_down.pop() {
                self.http_down(env, http::Request::Body(down));
                continue;
            }
            if let Some(event) = self.json_up.pop() {
                self.json_event(env, event);
                continue;
            }
            if let Some(event) = self.http_up.pop() {
                self.http_event(env, event);
                continue;
            }
            break;
        }
    }
    fn send(&mut self, down: Down, io: &mut Queue<skein_io::Request>) {
        if let Down::Send(_) = &down
            && self.request.is_some()
        {
            self.written = true;
            self.phase = Phase::Reading;
        }
        if let Some(socket) = self.socket {
            io.push(skein_io::Request::Stream { stream: socket, down });
        }
    }
    fn tls_event(&mut self, env: &Env<Limits>, event: skein_tls::client::Event) {
        match event {
            skein_tls::client::Event::Ready(_) => {
                self.phase = Phase::Idle;
                self.deadline = None;
            }
            skein_tls::client::Event::Stream(event) => {
                let h = self.http_env(env);
                http::up(&mut self.http, &h, event, &mut self.http_up, &mut self.http_down);
            }
            skein_tls::client::Event::Failed(_) => {
                self.failed(if self.written { Failure::Seen } else { Failure::Unseen });
            }
            skein_tls::client::Event::Closed => {}
        }
    }
    fn http_event(&mut self, env: &Env<Limits>, event: http::Event) {
        match event {
            http::Event::Response(response) => self.response(env, response),
            http::Event::Body(event) => match &mut self.receive {
                Receiving::Json(_) => {
                    let j = Env { now: env.now, wall: env.wall, limits: limits::documents(&env.limits).json };
                    tokenizer::up(&mut self.json, &j, event, &mut self.json_up, &mut self.json_down);
                }
                Receiving::Empty => {
                    drop(event);
                }
            },
            http::Event::Upload(event) => {
                drop(event);
            }
            http::Event::Done(reuse) => {
                let done = match core::mem::replace(&mut self.receive, Receiving::Empty) {
                    Receiving::Json(_) => {
                        self.done.take().unwrap_or(Body::Malformed(decode::Malformed::Missing { field: b"" }))
                    }
                    Receiving::Empty => self.done.take().unwrap_or(Body::Empty),
                };
                let close = reuse == http::Reuse::Close
                    || match &done {
                        Body::Malformed(_) => true,
                        Body::Document(_) | Body::Raw { .. } | Body::Empty => false,
                    };
                self.heard.push(Heard::Done(done));
                self.request = None;
                self.phase = if close { Phase::Closed } else { Phase::Idle };
                self.deadline = if close { None } else { Some(env.now.saturating_add(env.limits.idle)) };
            }
            http::Event::Failed(error) => self.failed(match error {
                http::Error::Closed(_) | http::Error::Refused(_) => Failure::Unseen,
                http::Error::Truncated { .. }
                | http::Error::Stream(_)
                | http::Error::Status
                | http::Error::Version
                | http::Error::Header
                | http::Error::HeadTooLong
                | http::Error::TooManyHeaders
                | http::Error::Framing
                | http::Error::ChunkSize
                | http::Error::Chunk
                | http::Error::Trailer
                | http::Error::Upgrade => Failure::Seen,
            }),
            http::Event::Closed => {}
        }
    }
    fn response(&mut self, env: &Env<Limits>, response: http::Response) {
        let date = match response.header(b"Date") {
            Some(date) => temper_forge_forgejo::time::http_date(date),
            None => None,
        };
        let retry_after = match response.header(b"Retry-After") {
            Some(value) => header::retry_after(value, date.unwrap_or(env.wall)),
            None => None,
        };
        let has_more = match response.header(b"X-HasMore") {
            Some(value) => header::has_more(value),
            None => None,
        };
        let content_range = match response.header(b"Content-Range") {
            Some(value) => header::content_range(value),
            None => None,
        };
        let close = match response.header(b"Connection") {
            Some(value) => value.eq_ignore_ascii_case(b"close"),
            None => false,
        };
        self.heard.push(Heard::Head {
            status: response.status,
            fields: Fields { date, retry_after, has_more, content_range, close },
        });
        let l = limits::documents(&env.limits);
        if response.status == 401
            || response.status == 403
            || response.framing == http::Framing::Empty
            || response.framing == http::Framing::Length(0)
        {
            self.receive = Receiving::Empty;
            self.http_down(env, http::Request::Discard);
            return;
        }
        let request = self.request.as_ref().expect("head belongs to a request");
        let decoder =
            if response.status >= 300 { Some(documents::error_decoder(&l)) } else { documents::decoder(request, &l) };
        match decoder {
            Some(mut decoder) => {
                documents::keep_after(&mut decoder, self.after);
                self.receive = Receiving::Json(decoder);
                self.json = tokenizer::Tokenizer::new(&l.json);
                self.json_down(env, tokenizer::Request::Next);
            }
            None => {
                self.receive = Receiving::Empty;
                self.http_down(env, http::Request::Discard);
            }
        }
    }
    fn json_event(&mut self, env: &Env<Limits>, event: tokenizer::Event) {
        let Receiving::Json(decoder) = &mut self.receive else {
            return;
        };
        let step = documents::step(decoder, &limits::documents(&env.limits), &event);
        match step {
            decode::Step::Demand(request) => self.json_down(env, request),
            decode::Step::Row(row, request) => {
                self.heard.push(Heard::Row(row));
                self.json_down(env, request);
            }
            decode::Step::Done(document) => {
                self.done = Some(Body::Document(document));
            }
            decode::Step::Malformed(error) => {
                self.receive = Receiving::Empty;
                self.phase = Phase::Closed;
                self.deadline = None;
                self.heard.push(Heard::Done(Body::Malformed(error)));
            }
        }
    }
    pub fn failed(&mut self, failure: Failure) {
        if self.phase == Phase::Closed {
            return;
        }
        self.phase = Phase::Closed;
        self.deadline = None;
        self.heard.push(Heard::Failed(failure));
    }
    pub fn timeout(&mut self) {
        self.failed(if self.written { Failure::Seen } else { Failure::Unseen });
    }
    pub fn close(&mut self, abort: bool, io: &mut Queue<skein_io::Request>) {
        if self.closing {
            return;
        }
        self.closing = true;
        self.phase = Phase::Closed;
        self.deadline = None;
        if let Some(socket) = self.socket {
            io.push(if abort {
                skein_io::Request::Abort { entity: socket }
            } else {
                skein_io::Request::Close { entity: socket }
            });
        }
    }
    pub fn heard(&mut self) -> Option<Heard> {
        self.heard.pop()
    }
    pub fn ready(&self) -> bool {
        if self.phase == Phase::Closed {
            return !self.heard.is_empty();
        }
        !self.http_up.is_empty()
            || !self.http_down.is_empty()
            || !self.tls_up.is_empty()
            || !self.tls_down.is_empty()
            || !self.json_up.is_empty()
            || !self.json_down.is_empty()
            || !self.heard.is_empty()
    }
    pub fn keep_after(&mut self, after: u64) {
        self.after = after;
        if let Receiving::Json(decoder) = &mut self.receive {
            documents::keep_after(decoder, after);
        }
    }
    pub fn cut(&mut self, env: &Env<Limits>) {
        if let Receiving::Json(decoder) = &mut self.receive {
            let request = documents::cut(decoder);
            self.json_down(env, request);
        }
    }
}

fn field(name: &[u8], value: &[u8]) -> skein_http::Header {
    skein_http::Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }
}

impl core::fmt::Debug for Connection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Connection")
            .field("forge", &self.forge)
            .field("phase", &self.phase)
            .field("deadline", &self.deadline)
            .finish_non_exhaustive()
    }
}
/// The fixed routing queues, allocated once per connection.
pub(crate) fn routing_bytes() -> Option<u64> {
    Queue::<http::Event>::worst_case(8)?
        .checked_add(Queue::<Down>::worst_case(8)?.checked_mul(3)?)?
        .checked_add(Queue::<skein_tls::client::Event>::worst_case(8)?)?
        .checked_add(Queue::<tokenizer::Event>::worst_case(8)?)?
        .checked_add(Queue::<Heard>::worst_case(16)?)
}
