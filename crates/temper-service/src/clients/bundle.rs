//! The browser's bundle, read whole at start, and its closed list of types (client/browser.md, 2.2; skein's
//! ui-web.md, 4.3 and 4.4). Copied from jig's testing/jig-test-service/src/server/bundle.rs.
use super::Clients;
use alloc::boxed::Box;
use jig_protocol_engine_clients_http as kit_http;
use skein_http::{Header, Method, server as http};
use skein_lib::{
    List, Map, Token, bytes,
    stream::{Down, Read, Up},
};

/// A bundle path's bound (jig's testing/jig-test/src/bundle.rs).
pub const NAME_BYTES: u32 = 1024;
/// The digest directory's name: sixteen lowercase hex digits.
const DIGEST_BYTES: u64 = 16;
/// The page's `ETag`: a quoted FNV-1a of sixteen hex digits.
const ETAG_BYTES: u64 = 18;

/// One digest-directory file, typed by the closed list (skein's ui-web.md, 4.4).
#[derive(Debug)]
pub struct File {
    pub kind: &'static [u8],
    pub bytes: Box<[u8]>,
}
/// The bundle read whole at start (client/browser.md, 2.2).
#[derive(Debug)]
pub struct Bundle {
    pub page: Box<[u8]>,
    pub etag: Box<[u8]>,
    pub digest: Box<[u8]>,
    pub files: Map<Box<[u8]>, File>,
}
/// A path's type by its extension, from the closed list; `None` off it.
#[must_use]
pub fn kind(path: &[u8]) -> Option<&'static [u8]> {
    let mut dot = None;
    for (at, byte) in path.iter().enumerate() {
        if *byte == b'.' {
            dot = Some(at);
        }
        if *byte == b'/' {
            dot = None;
        }
    }
    let dot = dot?;
    match path.get(dot.checked_add(1)?..)? {
        b"html" => Some(b"text/html; charset=utf-8"),
        b"js" => Some(b"text/javascript"),
        b"wasm" => Some(b"application/wasm"),
        b"css" => Some(b"text/css"),
        b"svg" => Some(b"image/svg+xml"),
        b"png" => Some(b"image/png"),
        b"webp" => Some(b"image/webp"),
        b"ico" => Some(b"image/x-icon"),
        b"woff2" => Some(b"font/woff2"),
        b"txt" => Some(b"text/plain; charset=utf-8"),
        b"webmanifest" => Some(b"application/manifest+json"),
        _ => None,
    }
}
/// The bundle's bytes at its limits: its files (the page among them), their names, the digest and the `ETag`.
#[must_use]
pub fn worst_case(limits: &crate::limits::Clients) -> Option<u64> {
    u64::from(limits.bundle_bytes)
        .checked_add(Map::<Box<[u8]>, File>::worst_case(limits.bundle_files)?)?
        .checked_add(u64::from(limits.bundle_files).checked_mul(u64::from(NAME_BYTES))?)?
        .checked_add(DIGEST_BYTES)?
        .checked_add(ETAG_BYTES)?
        .checked_add(u64::try_from(size_of::<Bundle>()).ok()?)
}

/// How the clients' listener answers a call the mapping does not claim (client/browser.md, 2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    File,
    Page,
    NotModified,
    Root,
    NotFound,
    NotAllowed,
}
/// An answer on its way: what it is, the path it serves and how much of its body is sent.
#[derive(Debug)]
pub(crate) struct Reply {
    answer: Answer,
    path: Box<[u8]>,
    offset: usize,
}
fn target(head: &http::Call) -> &[u8] {
    match bytes::find(&head.target, b"?") {
        Some(at) => head.target.get(..at).expect("query position"),
        None => &head.target,
    }
}
impl Bundle {
    fn file(&self, path: &[u8]) -> Option<&File> {
        let name = if path == b"/robots.txt" {
            b"files/robots.txt".as_slice()
        } else {
            let rest = path.strip_prefix(b"/bundle/")?;
            let slash = bytes::find(rest, b"/")?;
            if rest.get(..slash)? != self.digest.as_ref() {
                return None;
            }
            rest.get(slash.checked_add(1)?..)?
        };
        for (path, file) in &self.files {
            if path.as_ref() == name {
                return Some(file);
            }
        }
        None
    }
    fn content(&self, reply: &Reply) -> &[u8] {
        match reply.answer {
            Answer::File | Answer::Root => &self.file(&reply.path).expect("routed file").bytes,
            Answer::Page => &self.page,
            Answer::NotModified | Answer::NotFound | Answer::NotAllowed => b"",
        }
    }
}
/// The answer to a call the mapping does not claim, in the design's order.
pub(crate) fn route(bundle: Option<&Bundle>, head: &http::Call) -> Answer {
    let Some(bundle) = bundle else { return Answer::NotFound };
    if head.method != Method::Get {
        return Answer::NotAllowed;
    }
    let path = target(head);
    if path.starts_with(b"/bundle/") {
        return if bundle.file(path).is_some() { Answer::File } else { Answer::NotFound };
    }
    if path == b"/robots.txt" && bundle.file(path).is_none() {
        return Answer::NotFound;
    }
    if head.header(b"If-None-Match") == Some(&bundle.etag) {
        return Answer::NotModified;
    }
    if path == b"/robots.txt" { Answer::Root } else { Answer::Page }
}
fn field(headers: &mut List<Header>, name: &[u8], value: &[u8]) {
    headers.push(Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }).expect("answer fields");
}
impl Clients {
    /// Answers a call the mapping does not claim, and holds the answer until it is sent.
    pub(super) fn serve_call(&mut self, call: Token, head: &http::Call) {
        let answer = route(self.bundle.as_ref(), head);
        let reply = Reply { answer, path: bytes::copy_of(target(head)), offset: 0 };
        let mut headers = List::with_capacity(3);
        let mut length = 0;
        let status = match answer {
            Answer::Page | Answer::File | Answer::Root => {
                let bundle = self.bundle.as_ref().expect("routed bundle");
                let kind = match answer {
                    Answer::File | Answer::Root => bundle.file(&reply.path).expect("routed file").kind,
                    Answer::Page => b"text/html; charset=utf-8",
                    Answer::NotModified | Answer::NotFound | Answer::NotAllowed => unreachable!("content answer"),
                };
                field(&mut headers, b"Content-Type", kind);
                length = bundle.content(&reply).len();
                200
            }
            Answer::NotModified => 304,
            Answer::NotFound => 404,
            Answer::NotAllowed => 405,
        };
        match answer {
            Answer::File => field(&mut headers, b"Cache-Control", b"public, max-age=31536000, immutable"),
            Answer::Page | Answer::Root | Answer::NotModified => {
                field(&mut headers, b"Cache-Control", b"no-cache");
                field(&mut headers, b"ETag", &self.bundle.as_ref().expect("revalidated bundle").etag);
            }
            Answer::NotFound | Answer::NotAllowed => {}
        }
        if answer == Answer::NotFound && head.method == Method::Get && reply.path.as_ref() == b"/jig/v1/sign-in/link" {
            headers.push(kit_http::link_page()).expect("answer fields");
        }
        let response = http::Response {
            status,
            headers: headers.into_boxed(),
            body: if status == 304 {
                http::Body::None
            } else {
                http::Body::Length(u64::try_from(length).expect("bounded file"))
            },
            close: false,
        };
        self.server.answer(call, http::Request::Discard, &mut self.server_out);
        self.server.answer(call, http::Request::Respond(response), &mut self.server_out);
        if length != 0 {
            self.demand(call, length);
        }
        self.replies.insert(call, reply).expect("bounded server calls");
    }
    /// Takes one event of a call being answered: room for a chunk, or the answer's end.
    pub(super) fn serve_event(&mut self, call: Token, event: http::Event) {
        match event {
            http::Event::Reply(Up::Room) => self.chunk(call),
            http::Event::Done(_)
            | http::Event::Ended
            | http::Event::Refused(_)
            | http::Event::Failed(_)
            | http::Event::Closed
            | http::Event::Reply(Up::Failed(_))
            | http::Event::Body(Up::Failed(_)) => {
                self.replies.remove(&call);
                self.server.done(call, &mut self.server_out);
            }
            http::Event::Body(_) | http::Event::Reply(_) => {}
            http::Event::Call(_) => unreachable!("a head is routed, not an event"),
        }
    }
    fn demand(&mut self, call: Token, left: usize) {
        let room = self.send.min(u32::try_from(left).expect("bounded file"));
        let demand = http::Request::Reply(Down::Demand { read: Read::Nothing, room });
        self.server.answer(call, demand, &mut self.server_out);
    }
    fn chunk(&mut self, call: Token) {
        let reply = self.replies.get_mut(&call).expect("own reply");
        let body = self.bundle.as_ref().expect("body's bundle").content(reply);
        let end = reply.offset.saturating_add(usize::try_from(self.send).expect("send fits")).min(body.len());
        let bytes = bytes::copy_of(body.get(reply.offset..end).expect("reply slice"));
        let left = body.len().checked_sub(end).expect("reply cursor");
        reply.offset = end;
        self.server.answer(call, http::Request::Reply(Down::Send(bytes)), &mut self.server_out);
        if left == 0 {
            self.server.answer(call, http::Request::Reply(Down::Finish), &mut self.server_out);
        } else {
            self.demand(call, left);
        }
    }
}
