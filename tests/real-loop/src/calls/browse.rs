//! A bounded browser on the caller's loop submissions; only local fake HTTP.
use skein_io::kernel::{Complete, Done, Family, Op};
use std::net::SocketAddr;

const RESPONSE: usize = 65_536;
const REDIRECTS: usize = 4;

#[derive(Debug)]
pub struct Browsed {
    pub call: usize,
    pub hops: Vec<u16>,
    pub body: Vec<u8>,
}

pub(super) struct Browser {
    pub(super) seen: Browsed,
    address: SocketAddr,
    target: String,
    bytes: Vec<u8>,
    next: Option<String>,
    pub(super) done: bool,
}
impl Browser {
    pub(super) const MEMORY: u64 = (RESPONSE * 3) as u64;
    pub(super) fn new(call: usize, stdout: &[u8]) -> Self {
        let output = std::str::from_utf8(stdout).expect("install stdout");
        let mut lines = output.lines();
        lines
            .find(|line| line.ends_with("Open this address in a browser on the engine's machine:"))
            .expect("ctl's browser instruction");
        let url = lines.next().expect("ctl's installation address");
        let (address, target) = local(url);
        Self {
            seen: Browsed { call, hops: Vec::with_capacity(REDIRECTS + 1), body: Vec::new() },
            address,
            target,
            bytes: Vec::with_capacity(RESPONSE),
            next: None,
            done: false,
        }
    }
    pub(super) fn start(&self) -> Op {
        Op::Socket { family: Family::of(&self.address) }
    }
    pub(super) fn complete(&mut self, complete: Complete) -> Option<Op> {
        match (complete.kind, complete.result.expect("browser operation")) {
            (Op::Socket { .. }, Done::Fd(fd)) => Some(Op::Connect { fd, addr: self.address }),
            (Op::Connect { fd, .. }, Done::Nothing) => Some(Op::Send {
                fd,
                bytes: format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", self.target, self.address)
                    .into_bytes()
                    .into_boxed_slice(),
                from: 0,
            }),
            (Op::Send { fd, bytes, from }, Done::Count(count)) => {
                assert!(count > 0, "browser send progresses");
                let from = from + count;
                Some(if from as usize == bytes.len() {
                    Op::Recv { fd, buf: vec![0; 4096].into_boxed_slice() }
                } else {
                    Op::Send { fd, bytes, from }
                })
            }
            (Op::Recv { fd, buf }, Done::Count(count)) => {
                self.bytes.extend_from_slice(&buf[..count as usize]);
                assert!(self.bytes.len() <= RESPONSE, "bounded browser response");
                if self.response() {
                    Some(Op::Close { fd })
                } else {
                    assert!(count > 0, "browser response ended before its Content-Length");
                    Some(Op::Recv { fd, buf })
                }
            }
            (Op::Close { .. }, Done::Nothing) => {
                if let Some(url) = self.next.take() {
                    (self.address, self.target) = local(&url);
                    self.bytes.clear();
                    Some(self.start())
                } else {
                    self.done = true;
                    None
                }
            }
            (op, done) => panic!("browser completion {op:?}: {done:?}"),
        }
    }
    fn response(&mut self) -> bool {
        let Some(end) = self.bytes.windows(4).position(|part| part == b"\r\n\r\n") else { return false };
        let header = std::str::from_utf8(&self.bytes[..end]).expect("HTTP headers");
        let mut lines = header.split("\r\n");
        let status: u16 = lines
            .next()
            .expect("HTTP status")
            .split_whitespace()
            .nth(1)
            .expect("status code")
            .parse()
            .expect("numeric status");
        let mut length = None;
        let mut location = None;
        for line in lines {
            let (name, value) = line.split_once(':').expect("HTTP header");
            if name.eq_ignore_ascii_case("content-length") {
                length = Some(value.trim().parse::<usize>().expect("Content-Length"));
            } else if name.eq_ignore_ascii_case("location") {
                location = Some(value.trim().to_owned());
            }
        }
        let length = length.expect("browser peers frame with Content-Length");
        assert!(end + 4 + length <= RESPONSE, "bounded framed response");
        if self.bytes.len() < end + 4 + length {
            return false;
        }
        self.seen.hops.push(status);
        if matches!(status, 302 | 303) {
            assert!(self.seen.hops.len() <= REDIRECTS, "at most four redirects");
            let location = location.expect("redirect Location");
            self.next =
                Some(if location.starts_with('/') { format!("http://{}{location}", self.address) } else { location });
        } else {
            self.seen.body = self.bytes[end + 4..end + 4 + length].to_vec();
        }
        true
    }
}
fn local(url: &str) -> (SocketAddr, String) {
    let rest = url.strip_prefix("http://").expect("fake browser uses plaintext HTTP");
    let (authority, target) = rest.split_once('/').expect("browser URL path");
    let address: SocketAddr = authority.parse().expect("browser's numeric address");
    assert!(address.ip().is_loopback(), "browser stays on the local fake and engine");
    (address, format!("/{target}"))
}
