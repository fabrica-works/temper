//! A real native client driven by jig's finite semantic-tree script.
use skein_fake_person::{Doing, Find, Next, Step, Target, TreeFace};
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::Host;
use temper_client_domain::Address;
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};
const PIPE: Token = Token::new(u64::MAX);
const CLOSE: Token = Token::new(u64::MAX - 1);
/// The script is separate from the client's production composition.
pub struct Script {
    steps: Box<[Step<Address>]>,
    resume: bool,
}
#[must_use]
pub fn answered_chat(words: &[u8], answer: &[u8]) -> Script {
    Script {
        resume: false,
        steps: Box::new([
            Step::Go { address: Address::Jig(jig_domain_client::address::Address::Conversations { project: None }) },
            Step::See { find: Find::named(skein_ui::Role::TextArea, b"First words"), within: Duration::from_secs(20) },
            Step::Type { find: Find::named(skein_ui::Role::TextArea, b"First words"), words: words.into() },
            Step::Press { find: Find::named(skein_ui::Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(answer), within: Duration::from_secs(20) },
        ]),
    }
}
/// Stdout and the client share one host; the link comes only from the binary.
pub struct Person {
    pub client: Option<NativeClient>,
    root: Option<Fd>,
    actor: skein_fake_person::Person<Address>,
    input: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    closing: u32,
    bytes: Vec<u8>,
    wake: Option<Time>,
    pub finished: bool,
    pub failed: Option<String>,
    resume: bool,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Person {
    #[must_use]
    pub fn with_link(root: Fd, input: Fd, output: Fd, script: Script, link: &[u8]) -> Self {
        let mut person = Self::new(root, input, output, script);
        person.bytes.extend_from_slice(link);
        person.bytes.push(b'\n');
        person
    }
    #[must_use]
    pub fn new(root: Fd, input: Fd, output: Fd, script: Script) -> Self {
        Self {
            client: None,
            root: Some(root),
            actor: skein_fake_person::Person::new(script.steps),
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            wake: None,
            finished: false,
            failed: None,
            resume: script.resume,
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
        }
    }
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.submissions.push(Submit { op: CLOSE, kind: Op::Close { fd } });
    }
    fn completed(&mut self, complete: Complete) {
        if complete.op == CLOSE {
            self.closing -= 1;
            complete.result.expect("pipe closed");
        } else if complete.op == PIPE {
            self.reading = false;
            let Op::PipeRead { buf, .. } = complete.kind else {
                panic!("stdout read");
            };
            let Done::Count(count) = complete.result.expect("stdout read completes") else {
                panic!("read count");
            };
            self.bytes.extend_from_slice(&buf[..usize::try_from(count).expect("bounded read")]);
            assert!(self.bytes.len() <= 65_536, "bounded printed links");
            if count == 0 {
                let fd = self.output.take().expect("stdout");
                self.close(fd);
            }
        } else {
            self.client.as_mut().expect("native client").completions().push(complete);
        }
    }
    fn start(&mut self) {
        if self.client.is_none()
            && let Some(link) = super::printed_link(&self.bytes)
        {
            let address = std::str::from_utf8(link)
                .expect("printed URL")
                .strip_prefix("http://")
                .expect("loopback URL")
                .split('/')
                .next()
                .expect("URL address")
                .parse()
                .expect("socket address");
            self.client = Some(
                NativeClient::new(
                    Settings {
                        engine: skein_io::Addr::Ip(address),
                        directory: b"private".as_slice().into(),
                        limits: temper_client_shell_terminal::limits::native(),
                    },
                    self.root.take().expect("client root"),
                    skein_shell::effective_user(),
                    7,
                    if self.resume { Start::Token } else { Start::Link(link.into()) },
                )
                .expect("printed sign-in link"),
            );
        }
    }
    fn drive(&mut self, now: Time, wall: Wall) {
        let Some(client) = &mut self.client else {
            return;
        };
        client.iterate(now, wall);
        while let Some(signing) = client.signing() {
            assert!(
                matches!(signing, jig_client_protocol_link::Signing::SignedIn { .. }),
                "native signing failed: {signing:?}"
            );
        }
        while let Some(notice) = client.notices() {
            assert!(
                !matches!(
                    notice,
                    temper_client_shell_terminal::client::Shown::Notice(
                        temper_client_shell_terminal::route::Notice::Fact(_)
                    )
                ),
                "native person step {} protocol fact: {notice:?}",
                self.actor.at()
            );
            assert!(
                !matches!(
                    notice,
                    temper_client_shell_terminal::client::Shown::Terminal(
                        temper_client_shell_terminal::terminal::Notice::DirectoryRefused
                    )
                ),
                "private client directory refused"
            );
        }
        self.wake = None;
        if client.started() && !self.finished && self.failed.is_none() {
            let before = self.actor.at();
            match self.actor.next(&mut TreeFace::new(client.tree()), now) {
                Next::Do(Doing::Act { node, act }) => assert!(client.act(node, act), "offered person act"),
                Next::Do(Doing::Go { address }) => client.go(address),
                Next::Do(Doing::Reload) => panic!("chat script has no reload"),
                Next::Wait { until } => self.wake = until,
                Next::Done => {
                    self.finished = true;
                    client.close();
                }
                Next::Failed { step, why } => {
                    self.failed = Some(format!("native person step {step}: {why:?}"));
                }
            }
            if self.actor.at() != before {
                self.wake = Some(now);
            }
        }
        while let Some(submit) = client.submissions().pop() {
            self.submissions.push(submit);
        }
    }
}
impl Host for Person {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.completed(complete);
        }
        if let Some(fd) = self.input.take() {
            self.close(fd);
        }
        self.start();
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submissions
                .push(Submit { op: PIPE, kind: Op::PipeRead { fd, buf: vec![0; 1024].into_boxed_slice() } });
        }
        self.drive(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.input.is_some()
            || (!self.reading && self.output.is_some())
            || self.wake.is_some_and(|at| at <= now)
            || self.client.as_ref().is_some_and(|client| client.work_pending(now))
    }
    fn next_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_deadline), self.wake].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_policy_deadline), self.wake].into_iter().flatten().min()
    }
    fn is_empty(&self) -> bool {
        self.output.is_none() && self.closing == 0 && self.client.as_ref().is_some_and(Host::is_empty)
    }
    fn worst_case(&self) -> u64 {
        self.client.as_ref().map_or(131_072, |client| client.worst_case() + 131_072)
    }
    fn operations(&self) -> u32 {
        512
    }
}

#[must_use]
pub fn adopt_and_change(_forgejo: &super::forgejo::Throwaway) -> Script {
    use skein_ui::Role;
    let mut steps = answered_chat(b"Hello", super::ANSWER).steps.into_vec();
    steps.extend([
        Step::Go { address: Address::Jig(jig_domain_client::address::Address::Inbox { project: Some(1) }) },
        Step::See { find: Find::named(Role::Link, b"Adopt a repository"), within: super::WITHIN },
        Step::Press { find: Find::named(Role::Link, b"Adopt a repository") },
        Step::See { find: Find::named(Role::TextField, b"Owner"), within: super::WITHIN },
        Step::Type { find: Find::named(Role::TextField, b"Owner"), words: b"acme".as_slice().into() },
        Step::Type { find: Find::named(Role::TextField, b"Name"), words: b"widget".as_slice().into() },
        Step::Type { find: Find::named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::Press { find: Find::named(Role::Check, b"It runs CI") },
        Step::Type {
            find: Find::named(Role::TextField, b"Checks: gate numbers, separated by spaces or commas"),
            words: b"1".as_slice().into(),
        },
        Step::Press { find: Find::named(Role::Button, b"Adopt") },
        Step::See { find: Find::text(b"Adopted acme/widget on forge 1 as owned."), within: super::WITHIN },
    ]);
    steps.extend([
        Step::Go { address: Address::Jig(jig_domain_client::address::Address::Conversations { project: None }) },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within: super::WITHIN },
        Step::Type {
            find: Find::named(Role::TextArea, b"First words"),
            words: b"@forge_chat Land a small change".as_slice().into(),
        },
        Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
        Step::Go { address: Address::Jig(jig_domain_client::address::Address::Inbox { project: None }) },
        inbox_reply(b"Task 2:"),
    ]);
    Script { steps: steps.into_boxed_slice(), resume: false }
}
#[must_use]
pub fn found() -> Script {
    Script {
        resume: true,
        steps: Box::new([
            Step::Go { address: Address::Jig(jig_domain_client::address::Address::Inbox { project: None }) },
            inbox_reply(b"Task 2:"),
        ]),
    }
}
fn inbox_reply(task: &[u8]) -> Step<Address> {
    Step::See {
        find: Find::text(b"Reply message:")
            .within(Box::new([Target::Role { role: skein_ui::Role::Item, name: task.into() }])),
        within: super::WITHIN,
    }
}
