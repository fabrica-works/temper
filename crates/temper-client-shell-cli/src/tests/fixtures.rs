use crate::{
    arguments,
    host::{Invocation, OneShot, Startup},
};
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Kind, Op, OpenHow, ServiceSignal, Signalled, Stat, Submit};
use skein_lib::{Duration, Time, Wall};
use skein_shell::Host;
pub const CHAT: &[&[u8]] = &[b"chat", b"hello", b"--project", b"shop", b"--key", b"start-chat"];
const DIRECTORY: Fd = Fd::new(20);
const ADDRESS: Fd = Fd::new(21);
const TOKEN: Fd = Fd::new(22);
const STDERR: Fd = Fd::new(3);
const USER: u32 = 7;
const TOKEN_BYTES: &[u8] = b"0123456789012345678901234567890123456789";
pub const PARTIAL: usize = 3; // More than one short write is needed for every host diagnostic.
pub struct Frame {
    pub host: OneShot,
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
            self.host.iterate(self.now, Wall::EPOCH);
            let mut answered = false;
            while let Some(mut submit) = self.host.submissions().pop() {
                if matches!(submit.kind, Op::ReadSignal { .. }) {
                    self.signal = Some(submit);
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
    fn answer(&mut self, kind: &mut Op) -> Result<Done, Error> {
        match kind {
            Op::Open { path, how, .. } => {
                if path.as_ref() == b"directory" {
                    if *how == OpenHow::ReadNoFollow {
                        self.private_opens += 1;
                    }
                    if self.absent {
                        return Err(Error::NotFound);
                    }
                    Ok(Done::Fd(DIRECTORY))
                } else if path.as_ref() == b"address" {
                    Ok(Done::Fd(ADDRESS))
                } else if path.as_ref() == b"token" {
                    self.token_reads += 1;
                    Ok(Done::Fd(TOKEN))
                } else {
                    assert_eq!(path.as_ref(), b".");
                    Ok(Done::Fd(Fd::new(19)))
                }
            }
            Op::Stat { fd } => {
                let (kind, size, mode, links) = if *fd == DIRECTORY {
                    (Kind::Directory, 0, if self.refused { 0o777 } else { 0o700 }, 2)
                } else {
                    (
                        Kind::File,
                        u64::try_from(if *fd == ADDRESS { self.address.len() } else { self.token.len() })
                            .expect("fixture size"),
                        0o600,
                        1,
                    )
                };
                Ok(Done::Stat(Stat { kind, size, mode, owner: USER, links }))
            }
            Op::Read { fd, buf, at } => {
                let bytes = if *fd == ADDRESS {
                    self.address
                } else {
                    assert_eq!(*fd, TOKEN);
                    self.token
                };
                let start = usize::try_from(*at).expect("fixture offset").min(bytes.len());
                let count = buf.len().min(bytes.len() - start);
                buf[..count].copy_from_slice(&bytes[start..start + count]);
                Ok(Done::Count(u32::try_from(count).expect("fixture count")))
            }
            Op::Lock { fd } => {
                assert_eq!(*fd, DIRECTORY);
                self.locks += 1;
                if self.busy { Err(Error::Busy) } else { Ok(Done::Nothing) }
            }
            Op::PipeWrite { fd, bytes, from } => {
                assert_eq!(*fd, STDERR);
                let start = usize::try_from(*from).expect("fixture offset");
                let count = (bytes.len() - start).min(self.write_size);
                self.stderr.extend_from_slice(&bytes[start..start + count]);
                self.writes += 1;
                Ok(Done::Count(u32::try_from(count).expect("fixture count")))
            }
            Op::Cancel { target } => {
                if self.signal.as_ref().is_none_or(|signal| signal.op != *target) {
                    return Err(Error::TooLate);
                }
                let Submit { op, kind } = self.signal.take().expect("signal stays in flight");
                self.host.completions().push(Complete { op, kind, result: Err(Error::Cancelled) });
                Ok(Done::Nothing)
            }
            Op::Close { .. } => Ok(Done::Nothing),
            Op::PipeRead { .. } => Ok(Done::Count(0)),
            Op::Socket { .. }
            | Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Write { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
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
