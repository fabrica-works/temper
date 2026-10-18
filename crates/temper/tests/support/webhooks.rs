//! Signed deliveries against the shipped listener.
use super::{Process, StdoutReader};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{Referee, end_to_end::Streams, real};
use std::fmt::Write as _;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
};

pub const SECRET: &[u8] = b"fixture-hook-secret";
pub const PING: &[u8] = b"ping";

#[must_use]
pub fn free_address() -> SocketAddr {
    TcpListener::bind("127.0.0.1:0").expect("free socket").local_addr().expect("socket address")
}
#[must_use]
pub fn configuration(directory: &Path) -> SocketAddr {
    super::configuration(directory, &super::Forge::Unreached);
    let address = free_address();
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["webhooks"] = serde_json::json!({"listener":{"socket":address.to_string()}});
    value["application"]["forges"][0]["webhook_secret"] = "secrets/forge-1.webhook-secret".into();
    super::write_private(&directory.join("secrets/forge-1.webhook-secret"), SECRET);
    super::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
    address
}
#[must_use]
pub fn signature(secret: &[u8], body: &[u8]) -> String {
    let mut hmac = skein_lib::sha256::Hmac::new(secret);
    hmac.update(body);
    let mut signature = String::with_capacity(64);
    for byte in hmac.finish() {
        write!(signature, "{byte:02x}").expect("signature text");
    }
    signature
}
pub struct Post<'a> {
    path: &'a str,
    body: &'a [u8],
    event: &'a [u8],
    signed: bool,
}
impl<'a> Post<'a> {
    #[must_use]
    pub fn signed(path: &'a str, body: &'a [u8], event: &'a [u8]) -> Self {
        Self { path, body, event, signed: true }
    }
    #[must_use]
    pub fn unsigned(path: &'a str, body: &'a [u8], event: &'a [u8]) -> Self {
        Self { path, body, event, signed: false }
    }
    fn send(&self, listener: SocketAddr) -> u16 {
        let mut stream = TcpStream::connect_timeout(&listener, std::time::Duration::from_secs(5)).expect("listener");
        stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("read deadline");
        stream.set_write_timeout(Some(std::time::Duration::from_secs(5))).expect("write deadline");
        write!(stream, "POST {} HTTP/1.1\r\nHost: {listener}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Forgejo-Event: {}\r\n",
            self.path, self.body.len(), std::str::from_utf8(self.event).expect("event")).expect("request head");
        if self.signed {
            write!(stream, "X-Forgejo-Signature: {}\r\n", signature(SECRET, self.body)).expect("signature");
        }
        stream.write_all(b"\r\n").expect("head end");
        stream.write_all(self.body).expect("body");
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).expect("response");
        line.split_whitespace().nth(1).expect("status").parse().expect("status number")
    }
}
pub struct Ended {
    pub statuses: Vec<u16>,
    pub exit: Option<Exit>,
    pub stderr: Vec<u8>,
}
struct Judge<'a> {
    listener: SocketAddr,
    posts: &'a [Post<'a>],
    statuses: &'a mut Vec<u16>,
    sent: bool,
    settled: bool,
    deadline: Time,
}
impl Referee<Process> for Judge<'_> {
    fn act(&mut self, _: Time, procs: &mut [Process]) {
        let Process::Stdout(stdout) = &procs[1] else { panic!("stdout reader") };
        if !self.sent && super::printed_link(&stdout.bytes).is_some() {
            self.statuses.extend(self.posts.iter().map(|post| post.send(self.listener)));
            let Process::Binary(binary) = &mut procs[0] else { panic!("binary") };
            binary.signal(Signal::Terminate);
            self.sent = true;
        }
    }
    fn observe(&mut self, _: Time, procs: &[Process]) {
        self.settled = self.sent && procs.iter().all(Host::is_empty);
        let Process::Binary(binary) = &procs[0] else { panic!("binary") };
        assert!(
            self.sent || binary.exit_status().is_none(),
            "serve failed: {}",
            String::from_utf8_lossy(binary.stderr())
        );
    }
    fn next_deadline(&self) -> Option<Time> {
        (!self.settled).then_some(self.deadline)
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (!self.settled && now >= self.deadline).then(|| "webhook delivery or drain did not finish".into())
    }
    fn passed(&self) -> bool {
        self.settled
    }
}
pub fn served(directory: &Path, listener: SocketAddr, posts: &[Post<'_>]) -> Ended {
    let clock = Clock::new();
    let mut binary = super::serve(directory);
    let streams = binary.take_streams().expect("pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("pipes") };
    let within = Duration::from_secs(30);
    let mut statuses = Vec::new();
    let mut world = real::World::new(Judge {
        listener,
        posts,
        statuses: &mut statuses,
        sent: false,
        settled: false,
        deadline: clock.now().now.saturating_add(within),
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Process::Stdout(Box::new(StdoutReader::new(input, output))));
    let outcome = world.run(&clock, within);
    assert!(outcome.procs.iter().all(Host::is_empty), "all io settles");
    let Process::Binary(binary) = &outcome.procs[0] else { panic!("binary") };
    Ended { statuses, exit: binary.exit_status(), stderr: binary.stderr().to_vec() }
}
