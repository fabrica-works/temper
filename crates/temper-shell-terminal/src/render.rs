//! Bounded text and print numbers for the terminal (client.md, 6.4–6.8).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use jig_view::{Node, NodeId, Role, Run, State, Tree};
use skein_lib::{Decimal, List, Writer};

/// The most lines, bytes and numbered controls of one whole page print.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    pub lines: u32,
    pub bytes: u64,
    pub numbers: u32,
    pub depth: u32,
}
/// Words of the frame's behind status, from jig's phrasebook.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Frame {
    pub behind: Box<[u8]>,
}
/// The last print's enabled bound controls, in preorder.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Numbers {
    nodes: List<NodeId>,
}
impl Numbers {
    #[must_use]
    pub fn node(&self, number: u32) -> Option<NodeId> {
        self.nodes.get(number.checked_sub(1)?).copied()
    }
    #[must_use]
    pub fn count(&self) -> u32 {
        self.nodes.len()
    }
}
/// One complete page print and the numbers it displayed.
#[derive(Debug)]
pub struct Print {
    pub text: Box<[u8]>,
    pub numbers: Numbers,
}
/// Checked upper envelope: text replacement, run marks, state words and indentation.
#[must_use]
pub fn bounds(tree: &jig_view::Limits) -> Option<Bounds> {
    let lines = tree.nodes.checked_add(tree.text_bytes)?.checked_add(1)?;
    let indentation = tree.depth.checked_mul(2)?.checked_add(6)?;
    let bytes = u64::from(lines)
        .checked_mul(u64::from(indentation.checked_add(1)?))?
        .checked_add(u64::from(tree.text_bytes).checked_mul(5)?)?
        .checked_add(u64::from(tree.nodes).checked_mul(400)?)?
        .checked_add(32)?;
    Some(Bounds { lines, bytes, numbers: tree.bindings, depth: tree.depth })
}
/// The text, number table and bounded rendering scratch, excluding allocator overhead.
#[must_use]
pub fn worst_case(bounds: &Bounds) -> Option<u64> {
    bounds
        .bytes
        .checked_add(List::<NodeId>::worst_case(bounds.numbers)?)?
        .checked_add(List::<Open>::worst_case(bounds.depth)?)
}

/// Measure and then write the complete page, retaining only this print's numbers.
#[must_use]
pub fn print(tree: &Tree, frame: &Frame, bounds: &Bounds) -> Print {
    assert!(tree.len() < bounds.lines, "tree fits print envelope");
    let mut numbers = Numbers { nodes: List::with_capacity(bounds.numbers) };
    let mut open = List::with_capacity(bounds.depth);
    let mut output = Output { writer: None, bytes: 0, lines: 0, indent: 0, quote: false };
    walk(tree, frame, &mut output, &mut numbers, &mut open);
    assert!(output.bytes <= usize::try_from(bounds.bytes).expect("print bytes fit"), "print fits byte bound");
    assert!(output.lines <= bounds.lines, "print fits line bound");
    numbers.nodes.clear();
    open.clear();
    output.writer = Some(Writer::new(output.bytes));
    output.bytes = 0;
    output.lines = 0;
    output.indent = 0;
    output.quote = false;
    walk(tree, frame, &mut output, &mut numbers, &mut open);
    Print { text: output.writer.expect("writing pass has a writer").finish(), numbers }
}

#[derive(Debug)]
struct Open {
    end: u32,
    indent: u32,
    bullet: bool,
    takes: bool,
}
fn end(index: u32, node: &Node) -> u32 {
    index.checked_add(node.size).expect("subtree end fits")
}
fn words(tree: &Tree, index: u32, output: &mut Output) {
    let node = tree.get(index).expect("word owner exists");
    runs(node, output);
    let mut skipped = 0;
    for at in index.checked_add(1).expect("child index fits")..end(index, node) {
        if at < skipped {
            continue;
        }
        let child = tree.get(at).expect("word child exists");
        if child.role == Role::Text {
            runs(child, output);
        } else {
            skipped = end(at, child);
        }
    }
}
fn runs(node: &Node, output: &mut Output) {
    for run in &node.text {
        let code = match run.run {
            Run::Code => true,
            Run::Plain | Run::Emphasis | Run::Strong | Run::Link { .. } => false,
        };
        if code {
            output.put(b"`");
        }
        output.printable(&run.text);
        if code {
            output.put(b"`");
        }
    }
}
fn name(tree: &Tree, index: u32, output: &mut Output) {
    match &tree.get(index).expect("named node exists").name {
        Some(name) => output.printable(name),
        None => words(tree, index, output),
    }
}
fn header(tree: &Tree, frame: &Frame, output: &mut Output) -> (u32, Option<u32>, Option<u32>, Option<u32>) {
    let frame_end = match tree.get(1) {
        Some(node) => end(1, node),
        None => 1,
    };
    let mut title = None;
    let mut link = None;
    let mut behind = None;
    let mut statuses = 0_u32;
    for index in 1..tree.len() {
        let node = tree.get(index).expect("header node exists");
        if index < frame_end && node.role == Role::Status {
            statuses = statuses.checked_add(1).expect("frame status count fits");
            if link.is_none() {
                link = Some(index);
            } else if statuses == 2 && equal_words(tree, index, &frame.behind) {
                behind = Some(index);
            }
        }
        if index >= frame_end && title.is_none() && node.role == (Role::Heading { level: 1 }) {
            title = Some(index);
        }
    }
    output.put(b"== ");
    match title {
        Some(index) => words(tree, index, output),
        None => output.put(b"temper"),
    }
    output.put(b" == ");
    if let Some(index) = link {
        words(tree, index, output);
    }
    if let Some(index) = behind {
        output.put(b"; ");
        words(tree, index, output);
    }
    output.newline();
    (frame_end, title, link, behind)
}
fn equal_words(tree: &Tree, index: u32, expected: &[u8]) -> bool {
    let mut position = 0_usize;
    let mut skipped = 0;
    let node = tree.get(index).expect("status exists");
    for at in index..end(index, node) {
        if at < skipped {
            continue;
        }
        let child = tree.get(at).expect("status child exists");
        if at != index && child.role != Role::Text {
            skipped = end(at, child);
            continue;
        }
        for run in &child.text {
            let next = position.checked_add(run.text.len()).expect("status length fits");
            if expected.get(position..next) != Some(run.text.as_ref()) {
                return false;
            }
            position = next;
        }
    }
    position == expected.len()
}

fn walk(tree: &Tree, frame: &Frame, output: &mut Output, numbers: &mut Numbers, open: &mut List<Open>) {
    let (_, title, link, behind) = header(tree, frame, output);
    let mut skipped = 0;
    for index in 0..tree.len() {
        if index < skipped {
            continue;
        }
        for _ in 0..open.len() {
            if open.last().expect("open ancestor exists").end > index {
                break;
            }
            open.truncate(open.len().checked_sub(1).expect("ancestor exists"));
        }
        let node = tree.get(index).expect("preorder node exists");
        if [title, link, behind].contains(&Some(index)) {
            skipped = end(index, node);
            continue;
        }
        if node.role == Role::Text
            && match open.last() {
                Some(parent) => parent.takes,
                None => false,
            }
        {
            continue;
        }
        let parent_indent = match open.last() {
            Some(parent) => parent.indent,
            None => 0,
        };
        let (visible, extra, owns) = shape(node);
        output.indent = parent_indent.checked_add(extra).expect("indentation fits");
        output.quote = node.role == Role::Quote;
        if visible {
            let mut column = 0;
            for position in 0..open.len() {
                let ancestor = open.get_mut(position).expect("ancestor exists");
                if ancestor.bullet {
                    let spaces = ancestor
                        .indent
                        .checked_sub(2)
                        .expect("item indentation includes its mark")
                        .checked_sub(column)
                        .expect("nested mark follows its ancestor");
                    for _ in 0..spaces {
                        output.put(b" ");
                    }
                    output.put(b"- ");
                    ancestor.bullet = false;
                    column = ancestor.indent;
                }
            }
            output.indent = output.indent.checked_sub(column).expect("mark precedes content");
            output.padding();
            output.indent = parent_indent.checked_add(extra).expect("continuation indentation fits");
            line(tree, index, node, output, numbers);
            states(node, output);
            output.newline();
        }
        if node.role == Role::Row {
            skipped = end(index, node);
            continue;
        }
        let indent = parent_indent.checked_add(child_indent(node)).expect("child indentation fits");
        open.push(Open {
            end: end(index, node),
            indent,
            bullet: [Role::Item, Role::TreeItem].contains(&node.role),
            takes: owns,
        })
        .expect("ancestors fit view depth");
    }
}
fn shape(node: &Node) -> (bool, u32, bool) {
    match node.role {
        Role::Page | Role::Item | Role::TreeItem | Role::Table | Role::Slot => (false, 0, false),
        Role::Region | Role::Figure | Role::Dialog | Role::List | Role::Tree => (node.name.is_some(), 0, false),
        Role::ChoiceGroup | Role::Tabs | Role::Log | Role::Drawing | Role::Text => (true, 0, false),
        Role::Code => (true, 4, true),
        Role::Description => (true, 2, true),
        Role::Heading { .. }
        | Role::Paragraph
        | Role::Time
        | Role::Cell
        | Role::Term
        | Role::Button
        | Role::Tab
        | Role::Link
        | Role::TextField
        | Role::TextArea
        | Role::Choice
        | Role::Check
        | Role::Status
        | Role::Alert
        | Role::Progress
        | Role::Quote
        | Role::Separator
        | Role::Row => (true, 0, true),
    }
}
fn child_indent(node: &Node) -> u32 {
    match node.role {
        Role::Item | Role::TreeItem | Role::ChoiceGroup | Role::Tabs | Role::Log | Role::Drawing => 2,
        Role::Region | Role::Figure | Role::Dialog | Role::List | Role::Tree => {
            if node.name.is_some() {
                2
            } else {
                0
            }
        }
        Role::Page
        | Role::Heading { .. }
        | Role::Paragraph
        | Role::Text
        | Role::Table
        | Role::Row
        | Role::Cell
        | Role::Term
        | Role::Description
        | Role::Button
        | Role::Link
        | Role::TextField
        | Role::TextArea
        | Role::Choice
        | Role::Check
        | Role::Status
        | Role::Alert
        | Role::Tab
        | Role::Progress
        | Role::Time
        | Role::Code
        | Role::Quote
        | Role::Separator
        | Role::Slot => 0,
    }
}
fn numbered(node: &Node) -> bool {
    match node.role {
        Role::Button | Role::Link | Role::TextField | Role::TextArea | Role::Choice | Role::Check | Role::Tab => {
            node.external.is_none()
        }
        Role::Page
        | Role::Region
        | Role::Heading { .. }
        | Role::Paragraph
        | Role::Text
        | Role::List
        | Role::Item
        | Role::Table
        | Role::Row
        | Role::Cell
        | Role::Term
        | Role::Description
        | Role::ChoiceGroup
        | Role::Dialog
        | Role::Status
        | Role::Alert
        | Role::Log
        | Role::Tree
        | Role::TreeItem
        | Role::Tabs
        | Role::Progress
        | Role::Time
        | Role::Code
        | Role::Quote
        | Role::Separator
        | Role::Figure
        | Role::Slot
        | Role::Drawing => false,
    }
}
fn line(tree: &Tree, index: u32, node: &Node, output: &mut Output, numbers: &mut Numbers) {
    if numbered(node) {
        if tree.bound(node.id).is_some() && !node.states.as_slice().contains(&State::Disabled) {
            numbers.nodes.push(node.id).expect("numbers fit view bindings");
            output.put(b"[");
            output.decimal(numbers.count());
            output.put(b"] ");
        } else {
            output.put(b"[-] ");
        }
    }
    match node.role {
        Role::Heading { level } => {
            for _ in 0..level {
                output.put(b"#");
            }
            output.put(b" ");
            words(tree, index, output);
        }
        Role::Region
        | Role::Figure
        | Role::Dialog
        | Role::List
        | Role::Tree
        | Role::ChoiceGroup
        | Role::Tabs
        | Role::Log
        | Role::Drawing => {
            name(tree, index, output);
            output.put(b":");
        }
        Role::Button | Role::Tab => name(tree, index, output),
        Role::Link => {
            name(tree, index, output);
            if let Some(url) = &node.external {
                output.put(b" <");
                output.printable(url);
                output.put(b">");
            }
        }
        Role::TextField | Role::TextArea => {
            name(tree, index, output);
            output.put(b": ");
            match &node.value {
                Some(value) if !value.text.is_empty() => output.printable(&value.text),
                Some(_) | None => output.put(b"(empty)"),
            }
        }
        Role::Choice | Role::Check => {
            let selected = pressed(node);
            if node.role == Role::Choice {
                output.put(if selected { b"(*) " } else { b"( ) " });
            } else {
                output.put(if selected { b"[x] " } else { b"[ ] " });
            }
            name(tree, index, output);
        }
        Role::Status => {
            output.put(b"*");
            if !equal_words(tree, index, b"") {
                output.put(b" ");
                words(tree, index, output);
            } else if let Some(name) = &node.name {
                output.put(b" ");
                output.printable(name);
            }
        }
        Role::Alert => {
            output.put(b"! ");
            words(tree, index, output);
        }
        Role::Progress => {
            name(tree, index, output);
            output.put(b": ");
            words(tree, index, output);
        }
        Role::Term => {
            words(tree, index, output);
            output.put(b":");
        }
        Role::Separator => output.put(b"--"),
        Role::Row => {
            let mut first = true;
            for at in index.checked_add(1).expect("row child index fits")..end(index, node) {
                if tree.get(at).expect("row descendant exists").role == Role::Cell {
                    if !first {
                        output.put(b" | ");
                    }
                    first = false;
                    words(tree, at, output);
                }
            }
        }
        Role::Paragraph | Role::Text | Role::Time | Role::Cell | Role::Description | Role::Code | Role::Quote => {
            words(tree, index, output);
        }
        Role::Page | Role::Item | Role::TreeItem | Role::Table | Role::Slot => {}
    }
}
fn pressed(node: &Node) -> bool {
    for state in &node.states {
        if [State::Pressed, State::Selected].contains(state) {
            return true;
        }
    }
    false
}
fn states(node: &Node, output: &mut Output) {
    for (state, text) in [
        (State::Disabled, b" (disabled)".as_slice()),
        (State::Expanded, b" (expanded)"),
        (State::Busy, b" (busy)"),
        (State::Current, b" (current)"),
        (State::Invalid, b" (invalid)"),
        (State::Pressed, b" (pressed)"),
        (State::Selected, b" (selected)"),
        (State::Provisional, b" (provisional)"),
        (State::SubmitsOnEnter, b" (enter sends)"),
    ] {
        let marked =
            [Role::Check, Role::Choice].contains(&node.role) && [State::Pressed, State::Selected].contains(&state);
        if node.states.as_slice().contains(&state) && !marked {
            output.put(text);
        }
    }
}

struct Output {
    writer: Option<Writer>,
    bytes: usize,
    lines: u32,
    indent: u32,
    quote: bool,
}
impl Output {
    fn put(&mut self, bytes: &[u8]) {
        self.bytes = self.bytes.checked_add(bytes.len()).expect("print length fits");
        if let Some(writer) = &mut self.writer {
            writer.put(bytes).expect("print measured first");
        }
    }
    fn padding(&mut self) {
        for _ in 0..self.indent {
            self.put(b" ");
        }
        if self.quote {
            self.put(b"> ");
        }
    }
    fn newline(&mut self) {
        self.put(b"\n");
        self.lines = self.lines.checked_add(1).expect("line count fits");
    }
    fn printable(&mut self, bytes: &[u8]) {
        let mut skip = 0;
        for at in 0..bytes.len() {
            if at < skip {
                continue;
            }
            let tail = bytes.get(at..).expect("text position exists");
            let valid = match core::str::from_utf8(tail) {
                Ok(text) => text,
                Err(error) => core::str::from_utf8(tail.get(..error.valid_up_to()).expect("valid prefix"))
                    .expect("UTF-8 prefix checked"),
            };
            skip = at.checked_add(valid.len()).expect("text offset fits");
            for character in valid.chars() {
                match character {
                    '\n' => {
                        self.newline();
                        self.padding();
                    }
                    '\t' => self.put(b" "),
                    '\u{0}'..='\u{1f}' | '\u{7f}'..='\u{9f}' => self.put("�".as_bytes()),
                    _ => {
                        let mut buffer = [0; 4];
                        self.put(character.encode_utf8(&mut buffer).as_bytes());
                    }
                }
            }
            if valid.is_empty() {
                self.put("�".as_bytes());
            }
        }
    }
    fn decimal(&mut self, number: u32) {
        self.put(Decimal::of(u64::from(number)).as_bytes());
    }
}
