#![expect(dead_code, reason = "focused and fuzzy tests share one machine world")]
use skein_lib::{Duration, Queue, Secret, Time, Token};
use skein_ui::{Act, Builder, NodeId, Role, Tree};
use std::collections::VecDeque;
use temper_client_shell_terminal::{
    limits,
    terminal::{Client, Loop, Notice},
};

pub fn tree_limits() -> skein_ui::Limits {
    skein_ui::Limits {
        nodes: 64,
        depth: 8,
        text_per_node: 128,
        text_bytes: 4096,
        drawing_bytes: 16,
        scene_marks: 0,
        scene_bytes: 0,
        styles_per_node: 2,
        patches: 320,
        bindings: 16,
        slots: 4,
        markdown_depth: 2,
    }
}
fn tree(revision: u64) -> Tree {
    let mut builder = Builder::new(&tree_limits());
    builder.open(Role::Page);
    builder.open(Role::Region);
    builder.region(jig_view::regions::JigRegion::Link.region());
    builder.open(Role::Status);
    builder.text(skein_ui::Run::Plain, b"Live");
    builder.close();
    builder.close();
    builder.open(Role::Main);
    builder.open(Role::Heading { level: 1 });
    builder.text(skein_ui::Run::Plain, format!("Revision {revision}").as_bytes());
    builder.close();
    for (role, label) in
        [(Role::Button, b"Button".as_slice()), (Role::Link, b"Link"), (Role::Tab, b"Tab"), (Role::Check, b"Check")]
    {
        builder.open(role);
        builder.name(label);
        builder.bind(skein_ui::Binding { owner: jig_view::Owner::Jig.opaque(), token: Token::new(1) });
        builder.close();
    }
    builder.close();
    builder.close();
    builder.finish()
}
pub struct Script {
    tree: Tree,
    built: u64,
    next_id: u32,
    changes: VecDeque<Time>,
    notices: VecDeque<Notice>,
    acts: Vec<u32>,
    pending: bool,
    pub closed: bool,
    pub sign_ins: Vec<Time>,
    now: Time,
}
impl Script {
    fn new(signed_in: bool) -> Self {
        let mut script = Self {
            tree: Tree::new(tree_limits().nodes),
            built: 0,
            next_id: 0,
            changes: VecDeque::new(),
            notices: VecDeque::new(),
            acts: Vec::new(),
            pending: false,
            closed: false,
            sign_ins: Vec::new(),
            now: Time::ZERO,
        };
        if signed_in {
            script.rebuild();
        }
        script
    }
    fn rebuild(&mut self) {
        self.built += 1;
        let mut next = tree(self.built);
        skein_ui::diff(&self.tree, &mut next, &tree_limits(), &mut self.next_id, &mut Queue::with_capacity(320));
        self.tree = next;
    }
    pub fn change_at(&mut self, at: Duration) {
        self.changes.push_back(Time::ZERO.saturating_add(at));
    }
    pub fn notice_at_once(&mut self, notice: Notice) {
        self.notices.push_back(notice);
    }
    pub fn acts(&self) -> &[u32] {
        &self.acts
    }
    fn iterate(&mut self, now: Time) {
        self.now = now;
        if self.closed {
            return;
        }
        if self.pending {
            self.pending = false;
            self.rebuild();
        }
        while self.changes.front().is_some_and(|at| *at <= now) {
            self.changes.pop_front();
            self.rebuild();
        }
    }
}
impl Client for Script {
    fn tree(&self) -> &Tree {
        &self.tree
    }
    fn built(&self) -> u64 {
        self.built
    }
    fn act(&mut self, node: NodeId, act: Act) -> bool {
        if self.closed {
            return false;
        }
        let mut print = skein_ui_text::Print::new(&tree_limits()).expect("world print envelope");
        skein_ui_text::render(&self.tree, b"temper", Some(jig_view::regions::JigRegion::Link.region()), &mut print);
        let number =
            print.numbers.iter().position(|id| *id == node).map(|index| u32::try_from(index).expect("number fits") + 1);
        let Some(number) = number else {
            return false;
        };
        assert_eq!(act, Act::Toggle, "script's commands toggle its check");
        self.acts.push(number);
        self.pending = true;
        true
    }
    fn sign_in(&mut self, link: &[u8]) -> Result<(), jig_shell_native::Unlinked> {
        assert_eq!(link, b"private-link");
        self.sign_ins.push(self.now);
        Ok(())
    }
    fn notice(&mut self) -> Option<Notice> {
        self.notices.pop_front()
    }
    fn close(&mut self) {
        self.closed = true;
    }
}
pub struct World {
    pub terminal: Loop,
    pub client: Script,
    pub now: Time,
    held: Option<Box<[u8]>>,
    output: Vec<(Duration, Box<[u8]>)>,
    automatic: bool,
    input: VecDeque<(Time, Box<[u8]>)>,
    typed: Vec<u32>,
    stdout_every: u64,
}
impl World {
    pub fn signed_in() -> Self {
        Self::new(true)
    }
    pub fn signing_in() -> Self {
        Self::new(false)
    }
    fn new(signed_in: bool) -> Self {
        let mut world = Self {
            terminal: Loop::new(
                &limits::terminal(),
                &tree_limits(),
                Box::from(*b"client"),
                (!signed_in).then(|| Secret::new(Box::from(*b"private-link"))),
            ),
            client: Script::new(signed_in),
            now: Time::ZERO,
            held: None,
            output: Vec::new(),
            automatic: true,
            input: VecDeque::new(),
            typed: Vec::new(),
            stdout_every: 1,
        };
        world.turn();
        world
    }
    pub fn type_bytes(&mut self, bytes: &[u8]) {
        let mut taken = 0;
        while taken < bytes.len() {
            let count = self.terminal.stdin(&bytes[taken..], self.now, &mut self.client);
            taken += count;
            if count == 0 {
                self.stdout_takes();
            }
        }
        self.capture();
    }
    pub fn hold_stdout(&mut self) {
        self.automatic = false;
    }
    fn capture(&mut self) {
        if let Some(bytes) = self.terminal.output() {
            assert!(self.held.is_none(), "one stdout write in flight");
            self.output.push((self.now.saturating_since(Time::ZERO), bytes.clone()));
            self.held = Some(bytes);
        }
    }
    pub fn stdout_takes(&mut self) {
        self.capture();
        if self.held.take().is_some() {
            self.terminal.written();
        }
        self.terminal.iterate(self.now, &mut self.client);
        self.capture();
    }
    fn turn(&mut self) {
        self.client.iterate(self.now);
        while self.input.front().is_some_and(|(at, _)| *at <= self.now) {
            let (_, input) = self.input.pop_front().expect("due input");
            self.type_bytes(&input);
        }
        self.terminal.iterate(self.now, &mut self.client);
        self.capture();
        if self.automatic && (self.now.as_nanos() / 10_000_000).is_multiple_of(self.stdout_every) {
            self.stdout_takes();
        }
    }
    pub fn run_until(&mut self, span: Duration) {
        let end = Time::ZERO.saturating_add(span);
        while self.now < end {
            self.now = self.now.saturating_add(Duration::from_millis(10)).min(end);
            self.turn();
        }
    }
    pub fn prints_at(&self) -> Vec<Duration> {
        self.output.iter().filter(|(_, bytes)| bytes.starts_with(b"== ")).map(|(at, _)| *at).collect()
    }
    pub fn last_output(&self) -> &[u8] {
        &self.output.last().expect("output exists").1
    }
    pub fn shell_lines(&self) -> Vec<Box<[u8]>> {
        self.output.iter().filter(|(_, bytes)| bytes.starts_with(b"temper: ")).map(|(_, bytes)| bytes.clone()).collect()
    }
    pub fn shown_last(&self) -> bool {
        self.output.iter().rev().find(|(_, bytes)| bytes.starts_with(b"== ")).is_some_and(|(_, bytes)| {
            bytes.starts_with(format!("== Revision {} == Live\n", self.client.built()).as_bytes())
        })
    }
    pub fn typed(&self) -> &[u32] {
        &self.typed
    }
    pub fn random(mut seed: u64) -> Self {
        let mut world = Self::signed_in();
        world.stdout_every = draw(&mut seed) % 13 + 1;
        for n in 1..=24 {
            let at = n * 200;
            let split = usize::try_from(draw(&mut seed) % 3).expect("two-byte command split");
            world.input.push_back((Time::ZERO.saturating_add(Duration::from_millis(at)), Box::from(&b"4\n"[..split])));
            world
                .input
                .push_back((Time::ZERO.saturating_add(Duration::from_millis(at + 10)), Box::from(&b"4\n"[split..])));
            world.typed.push(4);
            world.client.change_at(Duration::from_millis(at + 20 + draw(&mut seed) % 100));
        }
        world
    }
}
fn draw(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
