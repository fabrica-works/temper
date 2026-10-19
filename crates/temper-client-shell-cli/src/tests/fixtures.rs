use crate::{
    arguments,
    host::{Invocation, OneShot, Startup},
};
use jig_client_protocol_wire as w;
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Kind, Op, OpenHow, ServiceSignal, Signalled, Stat, Submit};
use skein_lib::{Duration, Time, Wall};
use skein_shell::Host;
use std::collections::{BTreeMap, VecDeque};
pub const LINK: &[u8] = b"http://127.0.0.1:4000/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const LINK_FILE: Fd = Fd::new(23);
pub const CHAT: &[&[u8]] = &[b"chat", b"hello", b"--project", b"shop", b"--key", b"start-chat"];
const DIRECTORY: Fd = Fd::new(20);
const ADDRESS: Fd = Fd::new(21);
const TOKEN: Fd = Fd::new(22);
const STDERR: Fd = Fd::new(3);
const USER: u32 = 7;
pub const TOKEN_BYTES: &[u8] = b"0123456789012345678901234567890123456789";
pub const PARTIAL: usize = 3; // More than one short write is needed for every host diagnostic.
pub struct Frame {
    pub host: OneShot,
    pub link: &'static [u8],
    pub refuse_link: Option<()>,
    hold_stores: Option<()>,
    files: BTreeMap<Fd, Vec<u8>>,
    names: BTreeMap<Box<[u8]>, Fd>,
    stored: BTreeMap<Box<[u8]>, Vec<u8>>,
    sockets: BTreeMap<Fd, Socket>,
    held: Vec<Submit>,
    opened: Vec<Box<[u8]>>,
    requests: Vec<Box<[u8]>>,
    deliver: Option<()>,
    watch: Option<u64>,
    stream: Option<Fd>,
    stdout: Vec<u8>,
    next_socket: i32,
    now: Time,
    start: Time,
    signal: Option<Submit>,
    pub busy: bool,
    pub absent: bool,
    pub refused: bool,
    pub address: &'static [u8],
    pub token: &'static [u8],
    pub locks: u32,
    pub token_reads: u32,
    pub private_opens: u32,
    stderr: Vec<u8>,
    pub write_size: usize,
    pub writes: u32,
}
impl Frame {
    pub fn act(words: &[&[u8]]) -> Self {
        let host = OneShot::new(
            Invocation {
                directory: Some(b"directory".as_slice().into()),
                parsed: arguments::parse(words).expect("fixture call"),
            },
            Startup {
                parent: Some(Fd::new(1)),
                stdin: Fd::new(2),
                stdout: Fd::new(4),
                stderr: STDERR,
                signals: Fd::new(5),
                user: USER,
                seed: 1,
            },
        )
        .expect("native limits");
        Self {
            host,
            link: LINK,
            refuse_link: None,
            hold_stores: None,
            files: BTreeMap::new(),
            names: BTreeMap::new(),
            stored: BTreeMap::new(),
            sockets: BTreeMap::new(),
            held: Vec::new(),
            opened: Vec::new(),
            requests: Vec::new(),
            deliver: None,
            watch: None,
            stream: None,
            stdout: Vec::new(),
            next_socket: 100,
            now: Time::ZERO,
            start: Time::ZERO,
            signal: None,
            busy: false,
            absent: false,
            refused: false,
            address: b"http://127.0.0.1:4000\n",
            token: TOKEN_BYTES,
            locks: 0,
            token_reads: 0,
            private_opens: 0,
            stderr: Vec::new(),
            write_size: usize::MAX,
            writes: 0,
        }
    }
    pub fn sign_in(link: &'static [u8]) -> Self {
        let mut frame = Self::act(&[b"sign-in", b"link-file"]);
        frame.link = link;
        frame.hold_stores = Some(());
        frame
    }
    pub fn answer_sign_in_then_me(&mut self) {
        self.sign_in_then_deliver_me();
    }
    pub fn store_token_with_address(&mut self) {
        self.hold_stores = None;
        self.run();
    }
    pub fn stored(&self, name: &[u8]) -> Option<&[u8]> {
        self.stored.get(name).map(Vec::as_slice)
    }
    pub fn requests(&self) -> usize {
        self.requests.len()
    }
    pub fn read(words: &[&str]) -> Self {
        Self::act(&words.iter().map(|word| word.as_bytes()).collect::<Vec<_>>())
    }
    pub fn sign_in_then_deliver_me(&mut self) {
        self.deliver = Some(());
        if self.watch.is_some() {
            self.deliver_me();
        }
        self.run();
    }
    pub fn drain(&mut self) {
        self.run();
    }
    pub fn opened(&self, path: &[u8]) -> bool {
        self.opened.iter().any(|name| name.as_ref() == path)
    }
    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }
    pub fn sent_stream_gone(&self) -> bool {
        self.requests.iter().any(|path| path.as_ref() == b"/jig/v1/stream-gone")
    }
    pub fn started(&self) -> Time {
        self.start
    }
    pub fn next_deadline(&self) -> Option<Time> {
        self.host.next_policy_deadline()
    }
    pub fn lock_busy(&mut self) {
        self.busy = true;
        self.run();
    }
    pub fn advance_past(&mut self, duration: Duration) {
        self.now = self.now.saturating_add(duration);
        self.run();
    }
    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }
    pub fn exit(&self) -> Option<Exit> {
        self.host.exit()
    }
    pub fn signal(&mut self, signal: ServiceSignal) {
        self.run();
        let Submit { op, kind } = self.signal.take().expect("armed signal read");
        self.host.completions().push(Complete {
            op,
            kind,
            result: Ok(Done::Signalled(Signalled::Termination(signal))),
        });
        self.run();
    }
    pub fn run(&mut self) {
        for _ in 0..2000 {
            for mut submit in std::mem::take(&mut self.held) {
                if matches!(submit.kind, Op::Sync { .. }) && self.hold_stores.is_none()
                    || matches!(&submit.kind, Op::Recv { fd, .. } if self.sockets.get(fd).is_some_and(|s| !s.reply.is_empty()))
                {
                    let result = self.answer(&mut submit.kind);
                    self.host.completions().push(Complete { op: submit.op, kind: submit.kind, result });
                } else {
                    self.held.push(submit);
                }
            }
            self.host.iterate(self.now, Wall::EPOCH);
            let mut answered = false;
            while let Some(mut submit) = self.host.submissions().pop() {
                if self.hold_stores.is_some() && matches!(submit.kind, Op::Sync { .. }) {
                    self.held.push(submit);
                    continue;
                }
                if matches!(submit.kind, Op::ReadSignal { .. }) {
                    self.signal = Some(submit);
                    continue;
                }
                if let Op::Recv { fd, .. } = &submit.kind
                    && self.sockets.get(fd).is_some_and(|s| s.reply.is_empty())
                {
                    self.held.push(submit);
                    continue;
                }
                let result = self.answer(&mut submit.kind);
                self.host.completions().push(Complete { op: submit.op, kind: submit.kind, result });
                answered = true;
            }
            if !answered && !self.host.work_pending(self.now) {
                return;
            }
        }
        panic!("host did not settle");
    }
    fn stat(&self, fd: Fd) -> Done {
        let (kind, size, mode, links) = if fd == DIRECTORY {
            (Kind::Directory, 0, if self.refused { 0o777 } else { 0o700 }, 2)
        } else {
            (Kind::File, u64::try_from(self.contents(fd).len()).expect("fixture size"), 0o600, 1)
        };
        Done::Stat(Stat { kind, size, mode, owner: USER, links })
    }
    fn contents(&self, fd: Fd) -> &[u8] {
        if fd == ADDRESS {
            self.address
        } else if fd == TOKEN {
            self.token
        } else if fd == LINK_FILE {
            self.link
        } else {
            self.files.get(&fd).expect("fixture file")
        }
    }
    fn read_file(&self, fd: Fd, buf: &mut [u8], at: u64) -> Done {
        let bytes = self.contents(fd);
        let start = usize::try_from(at).expect("fixture offset").min(bytes.len());
        let count = buf.len().min(bytes.len() - start);
        buf[..count].copy_from_slice(&bytes[start..start + count]);
        Done::Count(u32::try_from(count).expect("fixture count"))
    }
    fn open_file(&mut self, path: &[u8], how: OpenHow) -> Result<Done, Error> {
        self.opened.push(path.into());
        if matches!(how, OpenHow::Create { .. } | OpenHow::CreateNoFollow { .. }) {
            let fd = Fd::new(self.next_socket);
            self.next_socket += 1;
            self.files.insert(fd, Vec::new());
            self.names.insert(path.into(), fd);
            return Ok(Done::Fd(fd));
        }
        if path == b"link-file" {
            return Ok(Done::Fd(LINK_FILE));
        }
        if path == b"store" {
            return Err(Error::NotFound);
        }
        if path == b"directory" {
            if how == OpenHow::ReadNoFollow {
                self.private_opens += 1;
            }
            if self.absent {
                return Err(Error::NotFound);
            }
            Ok(Done::Fd(DIRECTORY))
        } else if path == b"address" {
            Ok(Done::Fd(ADDRESS))
        } else if path == b"token" {
            self.token_reads += 1;
            Ok(Done::Fd(TOKEN))
        } else {
            assert_eq!(path, b".");
            Ok(Done::Fd(Fd::new(19)))
        }
    }
    fn answer(&mut self, kind: &mut Op) -> Result<Done, Error> {
        match kind {
            Op::Open { path, how, .. } => self.open_file(path, *how),
            Op::Write { fd, bytes, from, at } => {
                let start = usize::try_from(*from).expect("write offset");
                let at = usize::try_from(*at).expect("file offset");
                let bytes = &bytes[start..];
                let file = self.files.get_mut(fd).expect("temporary file");
                file.resize(at + bytes.len(), 0);
                file[at..].copy_from_slice(bytes);
                Ok(Done::Count(u32::try_from(bytes.len()).expect("written size")))
            }
            Op::Rename { from, to, .. } => {
                let fd = self.names.remove(from).expect("temporary name");
                self.stored.insert(to.clone(), self.files.get(&fd).expect("temporary contents").clone());
                Ok(Done::Nothing)
            }
            Op::Stat { fd } => Ok(self.stat(*fd)),
            Op::Read { fd, buf, at } => Ok(self.read_file(*fd, buf, *at)),
            Op::Lock { fd } => {
                assert_eq!(*fd, DIRECTORY);
                self.locks += 1;
                if self.busy { Err(Error::Busy) } else { Ok(Done::Nothing) }
            }
            Op::PipeWrite { fd, bytes, from } => {
                assert!([STDERR, Fd::new(4)].contains(fd));
                let start = usize::try_from(*from).expect("fixture offset");
                let count = (bytes.len() - start).min(self.write_size);
                if *fd == STDERR {
                    self.stderr.extend_from_slice(&bytes[start..start + count]);
                } else {
                    self.stdout.extend_from_slice(&bytes[start..start + count]);
                }
                self.writes += 1;
                Ok(Done::Count(u32::try_from(count).expect("fixture count")))
            }
            Op::Socket { .. } => {
                let fd = Fd::new(self.next_socket);
                self.next_socket += 1;
                self.sockets.insert(fd, Socket::default());
                Ok(Done::Fd(fd))
            }
            Op::Sync { .. } | Op::Remove { .. } | Op::Connect { .. } | Op::Shutdown { .. } | Op::Close { .. } => {
                Ok(Done::Nothing)
            }
            Op::Send { fd, bytes, from } => {
                let start = usize::try_from(*from).expect("send offset");
                self.sockets.get_mut(fd).expect("socket").request.extend_from_slice(&bytes[start..]);
                let count = bytes.len() - start;
                self.http_request(*fd);
                Ok(Done::Count(u32::try_from(count).expect("send count")))
            }
            Op::Recv { fd, buf } => {
                let reply = &mut self.sockets.get_mut(fd).expect("socket").reply;
                let count = buf.len().min(reply.len());
                for byte in &mut buf[..count] {
                    *byte = reply.pop_front().expect("response byte");
                }
                Ok(Done::Count(u32::try_from(count).expect("recv count")))
            }
            Op::Cancel { target } => {
                if let Some(index) = self.held.iter().position(|submit| submit.op == *target) {
                    let Submit { op, kind } = self.held.remove(index);
                    self.host.completions().push(Complete { op, kind, result: Err(Error::Cancelled) });
                    return Ok(Done::Nothing);
                }
                if self.signal.as_ref().is_none_or(|signal| signal.op != *target) {
                    return Err(Error::TooLate);
                }
                let Submit { op, kind } = self.signal.take().expect("signal stays in flight");
                self.host.completions().push(Complete { op, kind, result: Err(Error::Cancelled) });
                Ok(Done::Nothing)
            }
            Op::PipeRead { .. } => Ok(Done::Count(0)),
            Op::Bind { .. }
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
            | Op::ReadSignal { .. } => panic!("unexpected kernel operation"),
        }
    }
}

#[derive(Default)]
struct Socket {
    request: Vec<u8>,
    reply: VecDeque<u8>,
    answered: bool,
}
pub type Engine = Frame;
pub type Disk = Frame;
impl Frame {
    fn http_request(&mut self, fd: Fd) {
        let socket = self.sockets.get_mut(&fd).expect("socket");
        if socket.answered {
            return;
        }
        let Some(end) = socket.request.windows(4).position(|b| b == b"\r\n\r\n") else { return };
        let head = std::str::from_utf8(&socket.request[..end]).expect("HTTP head");
        let length = head
            .lines()
            .find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length: ").map(str::to_owned))
            .map_or(0, |number| number.parse::<usize>().expect("content length"));
        if socket.request.len() < end + 4 + length {
            return;
        }
        let path = head.split_whitespace().nth(1).expect("request path").as_bytes().to_vec();
        self.requests.push(path.clone().into_boxed_slice());
        socket.answered = true;
        match path.as_slice() {
            b"/jig/v1/sign-in/link" => {
                let answer = if self.refuse_link.is_some() {
                    w::SignInAnswer::Refused
                } else {
                    w::SignInAnswer::SignedIn(
                        w::SignedIn::new(
                            &w::CEILINGS,
                            w::SignedInParts {
                                token: Some(skein_lib::Secret::new(TOKEN_BYTES.into())),
                                party: 1,
                                expires: u64::MAX,
                            },
                        )
                        .expect("signed-in answer"),
                    )
                };
                let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("answer size"));
                answer.encode(&mut writer).expect("answer fits");
                let body = writer.finish();
                socket.reply.extend(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).bytes());
                socket.reply.extend(body.iter().copied());
            }
            b"/jig/v1/stream" => {
                self.stream = Some(fd);
                socket.reply.extend(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n");
                let mut served = skein_lib::List::with_capacity(1);
                served
                    .push(
                        w::Served::new(
                            &w::CEILINGS,
                            w::ServedParts { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION },
                        )
                        .expect("served version"),
                    )
                    .expect("one family");
                let hello = w::StreamEvent::Hello(
                    w::Hello::new(
                        &w::CEILINGS,
                        w::HelloParts { stream: [1; 16], served, heartbeat: Duration::from_secs(60) },
                    )
                    .expect("hello"),
                );
                socket.reply.extend(sse(&hello));
            }
            b"/jig/v1/watch" => {
                let open = w::Open::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&socket.request[end + 4..]))
                    .expect("watch record");
                self.watch = Some(open.watch());
                let answer = w::OpenAnswer::Admitted;
                let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("answer size"));
                answer.encode(&mut writer).expect("answer fits");
                let body = writer.finish();
                socket.reply.extend(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).bytes());
                socket.reply.extend(body.iter().copied());
                if self.deliver.is_some() {
                    self.deliver_me();
                }
            }
            _ => panic!("unexpected HTTP route"),
        }
    }
    fn deliver_me(&mut self) {
        let value = w::DeliveryValue::Me(jig_client_test_records::frame_record());
        let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).expect("delivery size"));
        value.encode(&mut writer).expect("delivery fits");
        let document = w::Owned::new(
            &w::CEILINGS,
            w::OwnedParts { owner: w::Owner::Jig, version: w::VERSION, record: writer.finish() },
        )
        .expect("owned delivery");
        let event = w::StreamEvent::Delivery(
            w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: self.watch.expect("opened watch"), document })
                .expect("me delivery"),
        );
        self.sockets.get_mut(&self.stream.expect("open stream")).expect("stream socket").reply.extend(sse(&event));
    }
}
fn sse(event: &w::StreamEvent) -> Vec<u8> {
    use skein_lib::base64::{self, Alphabet};
    let mut writer = skein_lib::Writer::new(usize::try_from(event.measure()).expect("event size"));
    event.encode(&mut writer).expect("event fits");
    let bytes = writer.finish();
    let length =
        base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("event size")).expect("base64 size");
    let mut writer = skein_lib::Writer::new(usize::try_from(length).expect("base64 size"));
    base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("base64 fits");
    let mut frame = b"data: ".to_vec();
    frame.extend_from_slice(&writer.finish());
    frame.extend_from_slice(b"\n\n");
    frame
}
