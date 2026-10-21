//! A real native client driven by jig's finite semantic-tree script.
use jig_client_protocol_link::Signing;
use skein_fake_person::{Doing, Find, Next, Step, Target, TreeFace};
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
use temper_client_domain::Address;
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};
const START_WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(20);
// Measured 21 s from Start to landed + one missed slow poll of 40 s = 61 s.
const DEMO_LANDS_WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(61);
const SIGN_IN_FIRST: skein_lib::Duration = skein_lib::Duration::from_millis(100);
const SIGN_IN_MOST: skein_lib::Duration = skein_lib::Duration::from_secs(2);
const PIPE: Token = Token::new(u64::MAX);
const CLOSE: Token = Token::new(u64::MAX - 1);
#[derive(Clone, Copy, PartialEq, Eq)]
enum Begin {
    LinkWithKey,
    Token,
    Link,
}
/// The script is separate from the client's production composition.
pub struct Script {
    steps: Box<[Step<Address>]>,
    begin: Begin,
}
#[must_use]
pub fn answered_chat(words: &[u8], answer: &[u8]) -> Script {
    Script {
        begin: Begin::LinkWithKey,
        steps: Box::new([
            Step::Go { address: Address::Chats },
            Step::See { find: Find::named(skein_ui::Role::TextArea, b"Start a chat"), within: START_WITHIN },
            Step::Type { find: Find::named(skein_ui::Role::TextArea, b"Start a chat"), words: words.into() },
            Step::Press { find: Find::named(skein_ui::Role::Button, b"Start") },
            Step::See { find: Find::text(answer), within: START_WITHIN },
        ]),
    }
}
/// Signs in by the printed link, no key added: the demo's seed added it.
/// Asks `cue` in a new chat, then sees the text and the task's report.
#[must_use]
pub fn demo_chat(cue: &[u8], text: &[u8], report: &[u8]) -> Script {
    use skein_ui::Role;
    let result = || Box::new([Target::Role { role: Role::Item, name: cue.into() }]);
    Script {
        begin: Begin::Link,
        steps: Box::new([
            Step::Go { address: Address::Chats },
            Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: START_WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"demo") },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: cue.into() },
            Step::Press { find: Find::named(Role::Button, b"Start") },
            // Native's two-row log initially holds the end and result. Once
            // the report arrives, page back to the turn that carries the answer.
            Step::See { find: Find::text(report), within: START_WITHIN },
            Step::See { find: Find::named(Role::Button, b"Older"), within: START_WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Older") },
            Step::See {
                find: Find::text(text)
                    .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }])),
                within: START_WITHIN,
            },
            // A Failed's reason could contain the report too. The inbox's result
            // card must say Done, and hold this cue's report, without any log.
            Step::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
            Step::See { find: Find::exactly_named(Role::Status, b"Done").within(result()), within: START_WITHIN },
            Step::See { find: Find::text(report).within(result()), within: START_WITHIN },
        ]),
    }
}
/// The first run delegates and lands without choosing a model or answering its wait.
#[must_use]
pub fn demo_change() -> Script {
    use skein_ui::Role;
    // The empty first run's chat is task 1; its change is task 2.
    const CHANGE: u64 = 2;
    let conversation = || Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }]);
    let details = || Box::new([Target::Role { role: Role::Region, name: b"Task details".as_slice().into() }]);
    Script {
        begin: Begin::Link,
        steps: Box::new([
            Step::Go { address: Address::Chats },
            Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: START_WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Start a chat"), words: b"Add CSV export to widget".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Start") },
            Step::See {
                find: Find::text("I’ll delegate this change in demo/widget. It will push a branch, open a pull request, check CI and land into main on Demo forge.".as_bytes()).within(conversation()),
                within: START_WITHIN,
            },
            Step::See {
                find: Find::text("CSV export is available from widget’s table. The change landed on Demo forge. This was a scripted demo change.".as_bytes()).within(conversation()),
                within: DEMO_LANDS_WITHIN,
            },
            // The owner's chat log holds the result's words alone, not its ending.
            Step::Go { address: Address::Task { task: CHANGE } },
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"Export widget's table as CSV").within(details()),
                within: START_WITHIN,
            },
            // Ended, not held: a change that fails to land is held; the clone check reads the merge.
            Step::See { find: Find::exactly_named(Role::Status, b"Ended").within(details()), within: START_WITHIN },
        ]),
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectSelection {
    Pending,
    Selected,
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
    signing_until: Option<Time>,
    signing_retry: Option<Time>,
    signing_backoff: skein_lib::Duration,
    pub finished: bool,
    project: ProjectSelection,
    pub failed: Option<String>,
    begin: Begin,
    seed: u64,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Person {
    /// Every byte read from the binary's stdout, including the printed link.
    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn with_link(root: Fd, input: Fd, output: Fd, script: Script, link: &[u8]) -> Self {
        let mut person = Self::new(root, input, output, script);
        person.bytes.extend_from_slice(link);
        person.bytes.push(b'\n');
        person
    }
    #[must_use]
    pub fn new(root: Fd, input: Fd, output: Fd, script: Script) -> Self {
        let steps = if script.begin == Begin::LinkWithKey {
            use skein_ui::Role;
            let mut setup = vec![
                Step::Go { address: Address::Accounts },
                Step::See { find: Find::named(Role::Button, b"Use deployment accounts"), within: START_WITHIN },
                Step::Press { find: Find::named(Role::Button, b"Use deployment accounts") },
                Step::Type { find: Find::named(Role::TextField, b"Provider number"), words: b"1".as_slice().into() },
                Step::Type {
                    find: Find::named(Role::TextField, b"Key (pasted once)"),
                    words: jig_channel_world::fixtures::KEY.into(),
                },
                Step::Press { find: Find::named(Role::Button, b"Add key") },
                Step::See { find: Find::text(b"Key kept as source"), within: START_WITHIN },
            ];
            setup.extend(script.steps);
            setup.into_boxed_slice()
        } else {
            script.steps
        };
        Self {
            client: None,
            root: Some(root),
            actor: skein_fake_person::Person::new(steps),
            input: Some(input),
            output: Some(output),
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            wake: None,
            signing_until: None,
            signing_retry: None,
            signing_backoff: SIGN_IN_FIRST,
            finished: false,
            project: ProjectSelection::Pending,
            failed: None,
            begin: script.begin,
            // Fresh demo clients must not reuse the same request names when
            // signing the same owner in again on the same deployment.
            seed: if script.begin == Begin::Link {
                let seed = skein_shell::crypto_seed().expect("demo client's fresh seed");
                u64::from_le_bytes(seed.expose()[..8].try_into().expect("eight seed bytes"))
            } else {
                7
            },
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
    fn start(&mut self, now: Time) {
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
            self.signing_until = Some(now.saturating_add(START_WITHIN));
            self.client = Some(
                NativeClient::new(
                    Settings {
                        engine: skein_io::Addr::Ip(address),
                        directory: b"private".as_slice().into(),
                        limits: temper_client_shell_terminal::limits::native(),
                    },
                    self.root.take().expect("client root"),
                    skein_shell::effective_user(),
                    self.seed,
                    if self.begin == Begin::Token { Start::Token } else { Start::Link(link.into()) },
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
            match signing {
                Signing::SignedIn { .. } => {
                    self.signing_retry = None;
                    self.signing_until = None;
                }
                Signing::Again => {
                    let until = self.signing_until.expect("sign-in deadline");
                    assert!(now < until, "native signing failed: {signing:?}");
                    self.signing_retry = Some(now.saturating_add(self.signing_backoff).min(until));
                    self.signing_backoff = self.signing_backoff.saturating_mul(2).min(SIGN_IN_MOST);
                }
                other @ (Signing::Refused | Signing::Lost | Signing::SignedOut) => {
                    panic!("native signing failed: {other:?}");
                }
            }
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
        if self.signing_backoff != SIGN_IN_FIRST
            && let Some(until) = self.signing_until
        {
            assert!(now < until, "native signing failed: Again");
            self.wake = Some(until);
        }
        if let Some(retry) = self.signing_retry {
            if now >= retry {
                self.signing_retry = None;
                // Start::Token installs a held credential locally; only link sign-ins have replies.
                client
                    .sign_in(super::printed_link(&self.bytes).expect("printed sign-in link"))
                    .expect("printed sign-in link");
            } else {
                self.wake = Some(self.wake.map_or(retry, |at| at.min(retry)));
            }
        }
        if client.started() && self.project == ProjectSelection::Pending && select_project(client, self.begin) {
            self.project = ProjectSelection::Selected;
            self.wake = Some(now);
        }
        let signing_wake = self.wake;
        if client.started() && self.project == ProjectSelection::Selected && !self.finished && self.failed.is_none() {
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
        self.wake = [self.wake, signing_wake].into_iter().flatten().min();
        while let Some(submit) = client.submissions().pop() {
            self.submissions.push(submit);
        }
    }
}
// Channel fixtures offer project zero and one. Init offers just one, already
// selected. The demo uses that button; older scripts still choose the other one.
fn select_project(client: &mut NativeClient, begin: Begin) -> bool {
    let node = client
        .tree()
        .nodes()
        .iter()
        .find(|node| {
            node.role == skein_ui::Role::Button
                && node.name.as_deref() == Some(b"Project".as_slice())
                && (begin == Begin::Link || !node.states.as_slice().contains(&skein_ui::State::Pressed))
        })
        .map(|node| node.id);
    let Some(node) = node else { return false };
    assert!(client.act(node, skein_ui::Act::Press), "offered project selection");
    true
}
impl Host for Person {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.completed(complete);
        }
        if let Some(fd) = self.input.take() {
            self.close(fd);
        }
        self.start(now);
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
        Step::Go {
            address: Address::Inbox {
                filter: temper_client_domain::address::Filter {
                    project: Some(1),
                    ..temper_client_domain::address::Filter::ALL
                },
            },
        },
        Step::See { find: Find::named(Role::Link, b"Adopt a repository"), within: super::WITHIN },
        Step::Press { find: Find::named(Role::Link, b"Adopt a repository") },
        Step::See { find: Find::named(Role::TextField, b"Owner"), within: super::WITHIN },
        Step::Type { find: Find::named(Role::TextField, b"Owner"), words: b"acme".as_slice().into() },
        Step::Type { find: Find::named(Role::TextField, b"Name"), words: b"widget".as_slice().into() },
        Step::Type { find: Find::named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::Press { find: Find::named(Role::Button, b"Adopt") },
        Step::See { find: Find::text(b"Adopted acme/widget on forge 1 as owned."), within: super::WITHIN },
    ]);
    steps.extend([
        Step::Go { address: Address::Chats },
        Step::See { find: Find::named(Role::TextArea, b"Start a chat"), within: super::WITHIN },
        Step::Type {
            find: Find::named(Role::TextArea, b"Start a chat"),
            words: b"@forge_chat Land a small change".as_slice().into(),
        },
        Step::Press { find: Find::named(Role::Button, b"Start") },
        Step::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
        inbox_reply(b"@forge_chat Land a small change"),
    ]);
    Script { steps: steps.into_boxed_slice(), begin: Begin::LinkWithKey }
}
#[must_use]
pub fn found() -> Script {
    Script {
        begin: Begin::Token,
        steps: Box::new([
            Step::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
            inbox_reply(b"@forge_chat Land a small change"),
        ]),
    }
}
fn inbox_reply(task: &[u8]) -> Step<Address> {
    Step::See {
        find: Find::named(skein_ui::Role::Heading { level: 3 }, b"Reply")
            .within(Box::new([Target::Role { role: skein_ui::Role::Item, name: task.into() }])),
        within: super::WITHIN,
    }
}
