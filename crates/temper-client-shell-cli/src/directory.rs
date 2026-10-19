//! DIRECTORY's private files and its engine address (client/tools.md, 8.1).
use crate::{arguments::Class, limits};
use skein_io::{
    Addr, file,
    file_layer::{self, FileIo},
    kernel::{Complete, Error, Fd, Submit},
};
use skein_lib::{Duration, Queue, Time, Token};
use std::net::SocketAddr;
const OWNER: Token = Token::new(1);
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Start,
    Exists,
    Checked,
    Private,
    Lock,
    Address,
    Token,
    Ready,
    Closing,
}
pub(crate) struct Directory {
    io: FileIo,
    root: Option<Token>,
    private: Option<Token>,
    check: Option<Token>,
    leaf: Box<[u8]>,
    class: Class,
    stage: Stage,
    started: Option<Time>,
    retry: Option<Time>,
    closing: bool,
    pub(crate) engine: Option<Addr>,
    pub(crate) token: Option<Box<[u8]>>,
    pub(crate) error: Option<Box<[u8]>>,
    pub(crate) events: Queue<file::Event>,
    pub(crate) submissions: Queue<Submit>,
}
impl Directory {
    pub(crate) fn new(parent: Option<Fd>, leaf: Box<[u8]>, class: Class, user: u32, seed: u64) -> Self {
        let mut io = FileIo::new(4, limits::ADDRESS, 1, Duration::from_secs(30)).with_effective_user(user);
        io.seed_randomness(seed);
        let root = parent.map(|fd| io.adopt_root(fd).expect("parent slot"));
        Self {
            io,
            root,
            private: None,
            check: None,
            leaf,
            class,
            stage: Stage::Start,
            started: None,
            retry: None,
            closing: false,
            engine: None,
            token: None,
            error: None,
            events: Queue::with_capacity(16),
            submissions: Queue::with_capacity(16),
        }
    }
    pub(crate) fn complete(&mut self, complete: Complete) {
        file_layer::up(&mut self.io, complete, &mut self.events, &mut self.submissions);
    }
    pub(crate) fn close(&mut self) {
        self.closing = true;
        self.retry = None;
        if !self.io.takes() {
            file_layer::cancel(&mut self.io, OWNER, &mut self.submissions);
        }
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.io.open_files() == 0 && self.io.takes() && self.submissions.is_empty()
    }
    pub(crate) fn deadline(&self) -> Option<Time> {
        [self.retry, self.io.next_deadline()].into_iter().flatten().min()
    }
    pub(crate) fn pending(&self, now: Time) -> bool {
        !self.events.is_empty()
            || self.io.is_due(now)
            || (self.io.takes()
                && (self.closing && self.io.open_files() > 0
                    || !self.closing
                        && self.root.is_some()
                        && self.stage != Stage::Ready
                        && self.retry.is_none_or(|at| now >= at)))
    }
    pub(crate) fn retry(&self) -> Option<Time> {
        self.retry
    }
    fn fail(&mut self, text: &[u8]) {
        self.error = Some(text.into());
        self.close();
    }
    fn answer(&mut self, now: Time, event: file::Event) {
        if let file::Event::Opened { file, .. } = &event {
            if self.stage == Stage::Exists {
                self.check = Some(*file);
            } else {
                self.private = Some(*file);
            }
        }
        if self.closing {
            return;
        }
        if let file::Event::Refused { found, .. } = event {
            let part = match self.stage {
                Stage::Address => "address",
                Stage::Token => "token",
                Stage::Start
                | Stage::Exists
                | Stage::Checked
                | Stage::Private
                | Stage::Lock
                | Stage::Ready
                | Stage::Closing => "DIRECTORY",
            };
            self.fail(format!("{part} is not private: {found:?}").as_bytes());
            return;
        }
        match (self.stage, event) {
            (Stage::Exists, file::Event::Opened { .. }) => self.stage = Stage::Checked,
            (Stage::Checked, file::Event::Closed { .. }) => {
                self.check = None;
                self.stage = Stage::Private;
            }
            (Stage::Private, file::Event::Opened { .. }) => {
                self.stage = if self.class.locks() { Stage::Lock } else { Stage::Address }
            }
            (Stage::Lock, file::Event::Locked { .. }) => {
                self.retry = None;
                self.stage = Stage::Address;
            }
            (Stage::Lock, file::Event::Busy { .. }) => self.retry = Some(now.saturating_add(limits::LOCK_TRY)),
            (Stage::Address, file::Event::Loaded { bytes, .. }) => match address(&bytes) {
                Ok(engine) => {
                    self.engine = Some(engine);
                    self.stage = Stage::Token;
                }
                Err(error) => self.fail(error),
            },
            (Stage::Token, file::Event::Loaded { bytes, .. }) if bytes.is_empty() => {
                self.fail(b"DIRECTORY is not signed in");
            }
            (Stage::Token, file::Event::Loaded { bytes, .. }) => {
                self.token = Some(bytes);
                self.stage = Stage::Ready;
            }
            (Stage::Exists, file::Event::Failed { error: Error::NotFound, .. }) => {
                self.fail(b"DIRECTORY does not exist");
            }
            (Stage::Token, file::Event::Failed { error: Error::NotFound, .. }) => {
                self.fail(b"DIRECTORY is not signed in");
            }
            (_, event) => self.fail(format!("DIRECTORY could not be read: {event:?}").as_bytes()),
        }
    }
    pub(crate) fn progress(&mut self, now: Time) {
        if self.io.is_due(now) {
            file_layer::expire(&mut self.io, now, &mut self.submissions);
        }
        while let Some(event) = self.events.pop() {
            self.answer(now, event);
        }
        if !self.io.takes() {
            return;
        }
        if self.closing {
            if let Some(file) = self.check.take().or_else(|| self.private.take()).or_else(|| self.root.take()) {
                self.stage = Stage::Closing;
                self.request(now, file::Request::Close { owner: OWNER, file });
            }
            return;
        }
        let Some(root) = self.root else {
            return;
        };
        if self.stage == Stage::Start {
            self.stage = if self.class == Class::SignIn { Stage::Private } else { Stage::Exists };
        }
        let request = match self.stage {
            Stage::Exists => file::Request::OpenDirectory {
                owner: OWNER,
                root: self.io.descriptor(root).expect("parent"),
                name: self.leaf.clone(),
                no_follow: true,
            },
            Stage::Checked => file::Request::Close { owner: OWNER, file: self.check.expect("checked directory") },
            Stage::Private => file::Request::OpenPrivate { owner: OWNER, root, path: self.leaf.clone() },
            Stage::Lock => {
                let started = *self.started.get_or_insert(now);
                if now >= started.saturating_add(limits::LOCK) {
                    self.fail(b"DIRECTORY is busy with another call");
                    return;
                }
                if self.retry.is_some_and(|at| now < at) {
                    return;
                }
                self.retry = None;
                file::Request::Lock { owner: OWNER, root: self.private.expect("private directory") }
            }
            Stage::Address if self.class == Class::SignIn => {
                self.stage = Stage::Ready;
                return;
            }
            Stage::Address | Stage::Token => {
                let (path, max) = if self.stage == Stage::Address {
                    (b"address".as_slice(), limits::ADDRESS)
                } else {
                    (b"token".as_slice(), 40)
                };
                file::Request::Load {
                    owner: OWNER,
                    root: self.private.expect("private directory"),
                    path: path.into(),
                    max,
                    no_follow: true,
                }
            }
            Stage::Start | Stage::Ready | Stage::Closing => return,
        };
        self.request(now, request);
    }
    fn request(&mut self, now: Time, request: file::Request) {
        file_layer::down(&mut self.io, now, request, &mut self.events, &mut self.submissions);
    }
}
/// The address is numeric HTTP, one newline-terminated line, on loopback.
pub(crate) fn address(bytes: &[u8]) -> Result<Addr, &'static [u8]> {
    let bad = b"the engine's address must be on loopback".as_slice();
    let line = bytes.strip_suffix(b"\n").ok_or(bad)?;
    let host = line.strip_prefix(b"http://").ok_or(bad)?;
    let socket: SocketAddr = std::str::from_utf8(host).map_err(|_| bad)?.parse().map_err(|_| bad)?;
    if !socket.ip().is_loopback() {
        return Err(bad);
    }
    Ok(Addr::Ip(socket))
}
