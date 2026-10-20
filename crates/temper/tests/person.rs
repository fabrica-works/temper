//! The native person's sign-in, with a scripted HTTP peer and no real clock or kernel.
pub mod support;
use skein_io::kernel::{Complete, Done, Error, Fd, Kind, Op, Stat, Submit};
use skein_lib::{Duration, Time, Wall};
use skein_shell::Host;
use std::collections::BTreeMap;

const LINK: &[u8] = b"http://127.0.0.1:4000/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const FIRST: Duration = Duration::from_millis(100);
const BEFORE_FIRST: Duration = Duration::from_millis(99);
const START_WITHIN: Duration = Duration::from_secs(20);
const ROOT: Fd = Fd::new(1);
const PRIVATE: Fd = Fd::new(2);
const PARENT: Fd = Fd::new(5);
// jig's client-v10 schema: NotReady, then SignedIn { token: None, party: 1, expires: MAX }.
const NOT_READY: &[u8] = &[2];
const SIGNED_IN: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255, 255, 255, 255, 255, 255, 255, 255];

#[derive(Default)]
struct Socket {
    sent: Vec<u8>,
    reply: Vec<u8>,
}
struct Frame {
    person: support::person::Person,
    sockets: BTreeMap<Fd, Socket>,
    posts: Vec<Vec<u8>>,
    again: u32,
    held: Vec<Submit>,
}
impl Frame {
    fn new() -> Self {
        Self {
            person: support::person::Person::with_link(
                ROOT,
                Fd::new(3),
                Fd::new(4),
                support::person::answered_chat(b"hello", b"answer"),
                LINK,
            ),
            sockets: BTreeMap::new(),
            posts: Vec::new(),
            again: 1,
            held: Vec::new(),
        }
    }
    fn run(&mut self, now: Time) {
        for _ in 0..1000 {
            self.person.iterate(now, Wall::EPOCH);
            for submit in std::mem::take(&mut self.held) {
                self.person.submissions().push(submit);
            }
            let mut answered = false;
            while let Some(mut submit) = self.person.submissions().pop() {
                let result = match &mut submit.kind {
                    Op::Close { .. } | Op::Connect { .. } | Op::Shutdown { .. } => Ok(Done::Nothing),
                    Op::PipeRead { .. } => continue, // The service keeps stdout open.
                    Op::Open { path, .. } if path.as_ref() == b"." => Ok(Done::Fd(PARENT)),
                    Op::Open { path, .. } if path.as_ref() == b"private" => Ok(Done::Fd(PRIVATE)),
                    Op::Open { .. } => Err(Error::NotFound),
                    Op::Stat { fd } => Ok(Done::Stat(Stat {
                        kind: Kind::Directory,
                        size: 0,
                        mode: 0o700,
                        owner: skein_shell::effective_user(),
                        links: if *fd == PRIVATE { 2 } else { 1 },
                    })),
                    Op::Socket { .. } => {
                        let fd = Fd::new(100 + i32::try_from(self.sockets.len()).expect("few sockets"));
                        self.sockets.insert(fd, Socket::default());
                        Ok(Done::Fd(fd))
                    }
                    Op::Send { fd, bytes, from } => {
                        let bytes = &bytes[usize::try_from(*from).expect("send offset")..];
                        let socket = self.sockets.get_mut(fd).expect("socket");
                        socket.sent.extend_from_slice(bytes);
                        if let Some(end) = socket.sent.windows(4).position(|w| w == b"\r\n\r\n") {
                            let head = std::str::from_utf8(&socket.sent[..end]).expect("HTTP head");
                            let length = head
                                .lines()
                                .find_map(|line| {
                                    line.to_ascii_lowercase().strip_prefix("content-length: ").map(str::to_owned)
                                })
                                .map_or(0, |n| n.parse::<usize>().expect("content length"));
                            if socket.sent.len() == end + 4 + length && head.starts_with("POST /jig/v1/sign-in/link ") {
                                self.posts.push(socket.sent[end + 4..].to_vec());
                                let body = if self.again > 0 {
                                    self.again -= 1;
                                    NOT_READY
                                } else {
                                    SIGNED_IN
                                };
                                socket.reply =
                                    format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
                                socket.reply.extend_from_slice(body);
                            }
                        }
                        Ok(Done::Count(u32::try_from(bytes.len()).expect("send count")))
                    }
                    Op::Recv { fd, buf } => {
                        let reply = &mut self.sockets.get_mut(fd).expect("socket").reply;
                        if reply.is_empty() {
                            self.held.push(submit);
                            continue;
                        }
                        let count = reply.len().min(buf.len());
                        buf[..count].copy_from_slice(&reply[..count]);
                        reply.drain(..count);
                        Ok(Done::Count(u32::try_from(count).expect("recv count")))
                    }
                    Op::Cancel { .. } => Err(Error::TooLate),
                    Op::Read { .. }
                    | Op::Write { .. }
                    | Op::Sync { .. }
                    | Op::Rename { .. }
                    | Op::Remove { .. }
                    | Op::Lock { .. }
                    | Op::PipeWrite { .. }
                    | Op::Bind { .. }
                    | Op::Listen { .. }
                    | Op::Accept { .. }
                    | Op::Append { .. }
                    | Op::Truncate { .. }
                    | Op::MakeDirectory { .. }
                    | Op::List { .. }
                    | Op::Spawn { .. }
                    | Op::Wait { .. }
                    | Op::Signal { .. }
                    | Op::Usage
                    | Op::Window { .. }
                    | Op::ReadSignal { .. } => panic!("unexpected operation: {:?}", submit.kind),
                };
                self.person.completions().push(Complete { op: submit.op, kind: submit.kind, result });
                answered = true;
            }
            if !answered && !self.person.work_pending(now) {
                return;
            }
        }
        panic!("person did not settle");
    }
}
#[test]
fn a_link_answered_again_is_sent_after_backoff() {
    let mut f = Frame::new();
    f.run(Time::ZERO);
    assert_eq!(f.person.next_deadline(), Some(Time::ZERO.saturating_add(FIRST)));
    f.run(Time::ZERO.saturating_add(BEFORE_FIRST));
    assert_eq!(f.posts.len(), 1);
    f.run(Time::ZERO.saturating_add(FIRST));
    assert_eq!(f.posts.len(), 2);
    assert_eq!(f.posts[0], f.posts[1]);
    assert_eq!(f.person.client.as_ref().expect("client").seen().signed_in, 1);
}

#[test]
#[should_panic(expected = "native signing failed: Again")]
fn repeated_again_expires_at_the_original_start_deadline() {
    let mut f = Frame::new();
    f.again = u32::MAX;
    f.run(Time::ZERO);
    f.run(Time::ZERO.saturating_add(FIRST));
    f.run(Time::ZERO.saturating_add(START_WITHIN));
}
