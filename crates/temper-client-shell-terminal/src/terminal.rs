//! The terminal's loop over a client, with time supplied by its host.
use crate::{
    limits,
    lines::{Line, Lines},
};
use skein_lib::{Duration, Queue, Secret, Time};
use skein_ui::{Act, NodeId, Tree};
use skein_ui_text::{Command, Print, Refusal};
pub(crate) mod native;
mod run;
pub use run::Terminal;

/// The native client, or the machine world's script, as the terminal sees it.
pub trait Client {
    fn tree(&self) -> &Tree;
    /// Number of builds that emitted patches; zero before the first tree.
    fn built(&self) -> u64;
    fn act(&mut self, node: NodeId, act: Act) -> bool;
    /// # Errors
    /// Refuses a link that is malformed or names another engine.
    fn sign_in(&mut self, link: &[u8]) -> Result<(), jig_client_shell_native::Unlinked>;
    fn notice(&mut self) -> Option<Notice>;
    fn close(&mut self);
}
/// Notices owned by the shell; credential bytes never enter the domain.
#[derive(Debug)]
pub enum Notice {
    SignedIn,
    SignInAgain,
    SignInRefused,
    SignedOut,
    Credential { party: u64, token: Secret },
    StoreFailed,
    DirectoryRefused,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Active,
    Closing,
}
/// A bounded command stream, one output in flight, and deferred whole-page prints.
pub struct Loop {
    limits: limits::Terminal,
    print: Print,
    view_limits: skein_ui::Limits,
    directory: Box<[u8]>,
    link: Option<Secret>,
    reader: Lines,
    lines: Queue<Box<[u8]>>,
    output: Option<Box<[u8]>>,
    writing: bool,
    printed: u64,
    last: Option<Time>,
    due: bool,
    changed: bool,
    acted: Option<u64>,
    retry: Option<Time>,
    backoff: Duration,
    phase: Phase,
    failure: Option<Box<[u8]>>,
}
impl Loop {
    #[must_use]
    pub fn new(
        limits: &limits::Terminal,
        view_limits: &skein_ui::Limits,
        directory: Box<[u8]>,
        link: Option<Secret>,
    ) -> Self {
        Self {
            limits: *limits,
            print: Print::new(view_limits).expect("print bounds"),
            view_limits: *view_limits,
            directory,
            link,
            reader: Lines::new(limits.line),
            lines: Queue::with_capacity(limits.lines),
            output: None,
            writing: false,
            printed: 0,
            last: None,
            due: false,
            changed: false,
            acted: None,
            retry: None,
            backoff: limits.first,
            phase: Phase::Active,
            failure: None,
        }
    }
    /// Consume bytes until shell output needs draining; the caller retains the suffix.
    pub fn stdin<C: Client>(&mut self, bytes: &[u8], now: Time, client: &mut C) -> usize {
        if self.phase == Phase::Closing {
            return bytes.len();
        }
        let mut taken = 0;
        for byte in bytes {
            if self.lines.room() == 0 {
                break;
            }
            taken += 1;
            if let Some(line) = self.reader.take(*byte) {
                let command = match line {
                    Line::Whole(line) => skein_ui_text::command(
                        &line,
                        (self.printed > 0).then(|| self.print.numbers.as_slice()),
                        client.tree(),
                        self.limits.line,
                    ),
                    Line::TooLong => Command::Refused { refusal: Refusal::TooLong, number: 0 },
                };
                self.command(command, client);
                if self.phase == Phase::Closing {
                    return bytes.len();
                }
            }
        }
        self.iterate(now, client);
        taken
    }
    fn command<C: Client>(&mut self, command: Command, client: &mut C) {
        match command {
            Command::Act { node, first, submit } => {
                let before = client.built();
                if client.act(node, first) {
                    if submit {
                        client.act(node, Act::Submit);
                    }
                    self.acted = Some(before);
                } else {
                    self.gone(node);
                }
            }
            Command::Print => self.due = true,
            Command::Close => self.close(client),
            Command::Nothing => {}
            Command::Refused { refusal, number } => {
                self.lines.push(skein_ui_text::refusal_line(b"temper", refusal, number));
                self.due |= matches!(refusal, Refusal::NoNumber | Refusal::Gone);
            }
        }
    }
    fn gone(&mut self, node: NodeId) {
        if let Some(index) = self.print.numbers.iter().position(|id| *id == node) {
            let number = u64::try_from(index).expect("printed index fits").checked_add(1).expect("number fits");
            self.lines.push(skein_ui_text::refusal_line(b"temper", Refusal::Gone, number));
            self.due = true;
        }
    }
    /// Close without signing out, retaining already queued output for its drain.
    pub fn close<C: Client>(&mut self, client: &mut C) {
        if self.phase == Phase::Active {
            self.phase = Phase::Closing;
            self.link = None;
            self.retry = None;
            self.due = false;
            self.acted = None;
            client.close();
        }
    }
    /// Whether closing has started, so the host stops reading stdin.
    #[must_use]
    pub fn closing(&self) -> bool {
        self.phase == Phase::Closing
    }
    /// The client has iterated; consume notices and prepare the next output when free.
    pub fn iterate<C: Client>(&mut self, now: Time, client: &mut C) {
        if self.phase == Phase::Active {
            while self.lines.room() > 0 {
                let Some(notice) = client.notice() else { break };
                self.notice(notice, now, client);
                if self.phase == Phase::Closing {
                    break;
                }
            }
            if self.retry.is_some_and(|deadline| now >= deadline) {
                self.retry = None;
                if let Some(link) = &self.link
                    && client.sign_in(link.expose()).is_err()
                {
                    self.notice(Notice::SignInRefused, now, client);
                }
            }
            if self.acted.is_some_and(|built| client.built() > built) {
                self.acted = None;
                self.due = true;
            }
            self.changed = client.built() > self.printed;
            if self.changed && self.last.is_none_or(|last| now >= last.saturating_add(self.limits.interval)) {
                self.due = true;
            }
        }
        if self.writing {
            return;
        }
        if let Some(line) = self.lines.pop() {
            self.output = Some(line);
            self.writing = true;
        } else if self.due && self.phase == Phase::Active && client.built() > 0 {
            skein_ui_text::render(
                client.tree(),
                b"temper",
                Some(jig_client_view::regions::JigRegion::Link.region()),
                &mut self.print,
            );
            self.output = Some(Box::from(self.print.text.as_slice()));
            self.printed = client.built();
            self.last = Some(now);
            self.due = false;
            self.changed = false;
            self.writing = true;
        }
    }
    fn notice<C: Client>(&mut self, notice: Notice, now: Time, client: &mut C) {
        match notice {
            Notice::SignedIn => {
                self.link = None;
                self.retry = None;
            }
            Notice::SignInAgain => {
                if self.link.is_some() {
                    self.lines.push(Box::from(*b"temper: signing in: no answer from the engine; trying again\n"));
                    self.retry = Some(now.saturating_add(self.backoff));
                    self.backoff = self.backoff.saturating_mul(2).min(self.limits.most);
                }
            }
            Notice::SignInRefused => {
                self.failure =
                    Some(Box::from(*b"temper open: the link was refused: use the fresh link temper serve printed\n"));
                self.close(client);
            }
            Notice::SignedOut => self.lines.push(join(&[
                b"temper: signed out; to sign in again, run temper open ",
                &self.directory,
                b" LINK with a link temper serve printed\n",
            ])),
            Notice::Credential { party, token } => {
                let size = skein_lib::base64::encoded_len(
                    skein_lib::base64::Alphabet::Url,
                    u32::try_from(token.expose().len()).expect("bounded credential"),
                )
                .expect("credential length fits");
                let mut encoded = skein_lib::Writer::new(usize::try_from(size).expect("encoded length fits"));
                skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, token.expose(), &mut encoded)
                    .expect("credential measured");
                let encoded = encoded.finish();
                self.lines.push(join(&[
                    b"temper: credential for service ",
                    skein_lib::Decimal::of(party).as_bytes(),
                    b", shown once: ",
                    &encoded,
                    b"\n",
                ]));
            }
            Notice::StoreFailed => {
                self.lines.push(join(&[b"temper: cannot write the small store in ", &self.directory, b"\n"]));
            }
            Notice::DirectoryRefused => {
                self.failure = Some(join(&[
                    b"temper open: ",
                    &self.directory,
                    b": refused (not a private directory of this user's, or its records unreadable)\n",
                ]));
                self.close(client);
            }
        }
    }
    /// Move the next output to stdout once; it stays in flight until `written`.
    pub fn output(&mut self) -> Option<Box<[u8]>> {
        self.output.take()
    }
    pub fn written(&mut self) {
        assert!(self.writing && self.output.is_none(), "stdout owns the pending output");
        self.writing = false;
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.phase == Phase::Closing {
            return None;
        }
        let print = if !self.writing && self.changed {
            self.last.map(|last| last.saturating_add(self.limits.interval))
        } else {
            None
        };
        [self.retry, print].into_iter().flatten().min()
    }
    /// The host also waits for the client's close to drain before exiting.
    #[must_use]
    pub fn exit(&self) -> Option<(u8, Option<Box<[u8]>>)> {
        if self.phase == Phase::Closing && !self.writing && self.lines.is_empty() {
            Some((u8::from(self.failure.is_some()), self.failure.clone()))
        } else {
            None
        }
    }
    /// The renderer's envelope, line buffers and shell queue, excluding allocator overhead.
    #[must_use]
    pub fn worst_case(&self) -> Option<u64> {
        let line = u64::try_from(self.directory.len().checked_add(160)?).ok()?;
        skein_ui_text::worst_case(&self.view_limits)?
            .1
            .checked_mul(2)?
            .checked_add(skein_lib::List::<NodeId>::worst_case(self.view_limits.nodes)?)?
            .checked_add(skein_lib::List::<u8>::worst_case(self.limits.line.checked_add(1)?)?)?
            .checked_add(Queue::<Box<[u8]>>::worst_case(self.limits.lines)?)?
            .checked_add(line.checked_mul(u64::from(self.limits.lines).checked_add(3)?)?)?
            .checked_add(u64::try_from(self.link.as_ref().map_or(0, |link| link.expose().len())).ok()?)
    }
}
fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let length = parts.iter().map(|part| part.len()).sum();
    let mut writer = skein_lib::Writer::new(length);
    for part in parts {
        writer.put(part).expect("measured shell line");
    }
    writer.finish()
}
