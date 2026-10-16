use crate::{Exchange, Failure, Outgoing, Range};
use std::{
    fmt::Write as FmtWrite,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};
pub(crate) fn request(port: u16, outgoing: &Outgoing<'_>, auth: Option<&str>) -> Vec<u8> {
    let mut head = format!("{} {} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n", outgoing.method, outgoing.target);
    if let Some(auth) = auth {
        write!(head, "Authorization: {auth}\r\n").expect("string write");
    }
    if let Some(body) = outgoing.body {
        write!(head, "Content-Type: application/json\r\nContent-Length: {}\r\n", body.len()).expect("string write");
    }
    match outgoing.range {
        Some(Range::Head(end)) => write!(head, "Range: bytes=0-{end}\r\n").expect("string write"),
        Some(Range::Tail(length)) => write!(head, "Range: bytes=-{length}\r\n").expect("string write"),
        None => {}
    }
    head.push_str("Connection: close\r\n\r\n");
    let mut bytes = head.into_bytes();
    if let Some(body) = outgoing.body {
        bytes.extend_from_slice(body);
    }
    bytes
}
pub(crate) fn exchange(
    port: u16,
    outgoing: &Outgoing<'_>,
    auth: Option<&str>,
    cap: usize,
) -> Result<Exchange, Failure> {
    let mut socket = TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().expect("loopback address"),
        Duration::from_millis(100),
    )?;
    socket.set_read_timeout(Some(Duration::from_secs(10)))?;
    socket.set_write_timeout(Some(Duration::from_secs(10)))?;
    socket.write_all(&request(port, outgoing, auth))?;
    let mut received = Vec::new();
    socket.take(u64::try_from(cap).expect("cap fits") + 1).read_to_end(&mut received)?;
    if received.len() > cap {
        return Err(Failure::Io("answer exceeds answer_bytes".into()));
    }
    parse(
        &received,
        request(
            port,
            outgoing,
            auth.map(|value| if value.starts_with("Basic ") { "Basic MASKED" } else { "token MASKED" }),
        ),
    )
}
pub(crate) fn parse(bytes: &[u8], sent: Vec<u8>) -> Result<Exchange, Failure> {
    let bad = || Failure::Io("malformed HTTP answer".into());
    let end = bytes.windows(4).position(|w| w == b"\r\n\r\n").ok_or_else(bad)?;
    let head = &bytes[..end + 2];
    let text = std::str::from_utf8(head).map_err(|_| bad())?;
    let status = text.split_whitespace().nth(1).ok_or_else(bad)?.parse().map_err(|_| bad())?;
    let mut body = bytes[end + 4..].to_vec();
    for line in text.lines().skip(1) {
        let Some((key, value)) = line.split_once(':') else {
            return Err(bad());
        };
        if key.eq_ignore_ascii_case("transfer-encoding") && value.trim().eq_ignore_ascii_case("chunked") {
            let mut rest = body.as_slice();
            let mut plain = Vec::new();
            loop {
                let end = rest.windows(2).position(|w| w == b"\r\n").ok_or_else(bad)?;
                let size = usize::from_str_radix(
                    std::str::from_utf8(&rest[..end]).map_err(|_| bad())?.split(';').next().ok_or_else(bad)?,
                    16,
                )
                .map_err(|_| bad())?;
                rest = &rest[end + 2..];
                if size == 0 {
                    if !rest.ends_with(b"\r\n") {
                        return Err(bad());
                    }
                    break;
                }
                let chunk = rest.get(..size).ok_or_else(bad)?;
                if rest.get(size..size + 2) != Some(b"\r\n") {
                    return Err(bad());
                }
                plain.extend_from_slice(chunk);
                rest = &rest[size + 2..];
            }
            body = plain;
        }
        if key.eq_ignore_ascii_case("content-length") && value.trim().parse::<usize>().map_err(|_| bad())? != body.len()
        {
            return Err(bad());
        }
    }
    Ok(Exchange { sent, status, head: head.to_vec(), body })
}
pub fn basic(value: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in value.as_bytes().chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        for index in [a >> 2, ((a & 3) << 4) | (b >> 4), ((b & 15) << 2) | (c >> 6), c & 63] {
            out.push(char::from(TABLE[usize::from(index)]));
        }
        if chunk.len() < 3 {
            out.pop();
            out.push('=');
        }
        if chunk.len() < 2 {
            out.pop();
            out.pop();
            out.push('=');
            out.push('=');
        }
    }
    out
}
