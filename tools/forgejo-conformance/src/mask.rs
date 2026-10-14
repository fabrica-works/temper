use std::collections::BTreeMap;
use temper_throwaway_forgejo::Exchange;
#[derive(Debug)]
pub struct Mask {
    port: u16,
    ids: BTreeMap<Vec<u8>, usize>,
    times: BTreeMap<Vec<u8>, usize>,
}
impl Mask {
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self { port, ids: BTreeMap::new(), times: BTreeMap::new() }
    }
    #[must_use]
    pub fn bytes(&mut self, bytes: &[u8]) -> Vec<u8> {
        let needle = format!("127.0.0.1:{}", self.port).into_bytes();
        let mut out = Vec::new();
        let mut at = 0;
        while at < bytes.len() {
            if bytes[at..].starts_with(&needle) {
                out.extend_from_slice(b"127.0.0.1:3000");
                at += needle.len();
                continue;
            }
            if (at == 0 || bytes[at - 1] == b'\n')
                && let Some(end) = bytes[at..].windows(2).position(|w| w == b"\r\n")
            {
                let line = &bytes[at..at + end];
                if let Some(colon) = line.iter().position(|b| *b == b':')
                    && (line[..colon].eq_ignore_ascii_case(b"date")
                        || line[..colon].eq_ignore_ascii_case(b"last-modified"))
                {
                    out.extend_from_slice(&line[..=colon]);
                    out.extend_from_slice(b" Wed, 01 Jan 2020 00:00:00 GMT\r\n");
                    at += end + 2;
                    continue;
                }
            }
            if bytes[at] == b'"'
                && let Some(end) = bytes[at + 1..].iter().position(|b| *b == b'"')
            {
                let time = &bytes[at + 1..at + 1 + end];
                if time.len() >= 20
                    && time[..4].iter().all(u8::is_ascii_digit)
                    && time[4] == b'-'
                    && time[10] == b'T'
                    && (&b"2000"[..]..=&b"2999"[..]).contains(&&time[..4])
                {
                    let next = self.times.len() + 1;
                    let n = *self.times.entry(time.to_vec()).or_insert(next);
                    let offset = if time.ends_with(b"Z") {
                        "Z"
                    } else {
                        std::str::from_utf8(&time[time.len() - 6..]).expect("RFC3339 offset")
                    };
                    out.extend_from_slice(
                        format!("\"2020-01-01T{:02}:{:02}:{:02}{offset}\"", n / 3600, n / 60 % 60, n % 60).as_bytes(),
                    );
                    at += end + 2;
                    continue;
                }
            }
            if bytes[at].is_ascii_digit() || (b'a'..=b'f').contains(&bytes[at]) {
                let end =
                    at + bytes[at..].iter().take_while(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)).count();
                let id = &bytes[at..end];
                if id.len() == 40 || id.len() == 64 {
                    let next = self.ids.len() + 1;
                    let n = *self.ids.entry(id.to_vec()).or_insert(next);
                    out.extend_from_slice(format!("{n:0width$x}", width = id.len()).as_bytes());
                    at = end;
                    continue;
                }
                out.extend_from_slice(id);
                at = end;
                continue;
            }
            out.push(bytes[at]);
            at += 1;
        }
        out
    }
}
#[must_use]
pub fn request(mask: &mut Mask, exchange: &Exchange) -> Vec<u8> {
    mask.bytes(&exchange.sent)
}
#[must_use]
pub fn response(mask: &mut Mask, exchange: &Exchange) -> Vec<u8> {
    let head = mask.bytes(&exchange.head);
    let body = mask.bytes(&exchange.body);
    let mut out = Vec::new();
    for line in head.split(|b| *b == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            continue;
        }
        let key = line.split(|b| *b == b':').next().unwrap_or_default();
        if key.eq_ignore_ascii_case(b"content-length") || key.eq_ignore_ascii_case(b"transfer-encoding") {
            continue;
        }
        out.extend_from_slice(line);
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
    out.extend_from_slice(&body);
    out
}
