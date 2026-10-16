//! Lines and commands for the terminal (client.md, 6.2 and 6.5).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use crate::render::Numbers;
use skein_lib::{Decimal, List, Writer};
use skein_ui::{Act, NodeId, Role, State, Tree};

/// One ended stdin line, excluding its newline and optional final carriage return.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Line {
    Whole(Box<[u8]>),
    TooLong,
}
/// A bounded partial line; an overlong line is discarded through its newline.
#[derive(Debug)]
pub struct Lines {
    held: List<u8>,
    limit: u32,
    long: bool,
}
impl Lines {
    #[must_use]
    pub fn new(limit: u32) -> Self {
        Self { held: List::with_capacity(limit.checked_add(1).expect("line bound fits")), limit, long: false }
    }
    /// Feed one byte, yielding a whole line only at its newline.
    pub fn take(&mut self, byte: u8) -> Option<Line> {
        if byte == b'\n' {
            let line = if self.long {
                Line::TooLong
            } else {
                if self.held.last() == Some(&b'\r') {
                    self.held.truncate(self.held.len().checked_sub(1).expect("final carriage return held"));
                }
                Line::Whole(Box::from(self.held.as_slice()))
            };
            self.held.clear();
            self.long = false;
            return Some(line);
        }
        if !self.long {
            if [self.held.len() < self.limit, self.held.len() == self.limit && byte == b'\r'].contains(&true) {
                self.held.push(byte).expect("line fits bound and final carriage return");
            } else {
                self.long = true;
                self.held.clear();
            }
        }
        None
    }
    #[must_use]
    pub fn holding(&self) -> bool {
        [self.long, !self.held.is_empty()].contains(&true)
    }
}
/// A command resolved using the last print's node identities.
#[derive(PartialEq, Eq, Debug)]
pub enum Command {
    Act { node: NodeId, act: Act },
    Write { node: NodeId, value: Box<[u8]>, submit: bool },
    Print,
    Close,
    Nothing,
    Refused(Refusal),
}
/// A refused command's reason and, when applicable, the displayed number.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    NoNumber { number: u32 },
    Gone { number: u32 },
    NoWords { number: u32 },
    NoPage,
    TooLong,
    Unknown,
}
impl Refusal {
    #[must_use]
    pub fn reprints(self) -> bool {
        match self {
            Self::NoNumber { .. } | Self::Gone { .. } => true,
            Self::NoWords { .. } | Self::NoPage | Self::TooLong | Self::Unknown => false,
        }
    }
}
fn trimmed(bytes: &[u8]) -> &[u8] {
    let mut first = bytes.len();
    let mut end = 0;
    for (at, byte) in bytes.iter().enumerate() {
        if *byte != b' ' {
            first = first.min(at);
            end = at.checked_add(1).expect("line length fits");
        }
    }
    if first > end { b"" } else { bytes.get(first..end).expect("trimmed line range") }
}
fn number(bytes: &[u8]) -> Option<u32> {
    if [bytes.is_empty(), bytes.len() > 10].contains(&true) {
        return None;
    }
    let mut value = 0_u32;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}
/// Resolve a line against the last print and the current tree, without acting.
#[must_use]
pub fn read(line: &[u8], numbers: Option<&Numbers>, tree: &Tree) -> Command {
    let line = trimmed(line);
    match line {
        b"" => return Command::Nothing,
        b"p" => return Command::Print,
        b"q" => return Command::Close,
        _ => {}
    }
    let mut split = line.len();
    for (at, byte) in line.iter().enumerate() {
        if *byte == b' ' {
            split = at;
            break;
        }
    }
    let Some(number) = number(line.get(..split).expect("first word range")) else {
        return Command::Refused(Refusal::Unknown);
    };
    let Some(numbers) = numbers else {
        return Command::Refused(Refusal::NoPage);
    };
    let Some(id) = numbers.node(number) else {
        return Command::Refused(Refusal::NoNumber { number });
    };
    let Some(node) = tree.find(id) else {
        return Command::Refused(Refusal::Gone { number });
    };
    if [node.states.as_slice().contains(&State::Disabled), tree.bound(id).is_none()].contains(&true) {
        return Command::Refused(Refusal::Gone { number });
    }
    let words = trimmed(line.get(split..).expect("words range"));
    if [Role::TextField, Role::TextArea].contains(&node.role) {
        return Command::Write {
            node: id,
            value: Box::from(words),
            submit: node.states.as_slice().contains(&State::SubmitsOnEnter),
        };
    }
    if !words.is_empty() {
        return Command::Refused(Refusal::NoWords { number });
    }
    let act = if [Role::Check, Role::Choice].contains(&node.role) { Act::Toggle } else { Act::Press };
    Command::Act { node: id, act }
}
/// The refusal's complete shell line, including its newline.
#[must_use]
pub fn refusal_line(refusal: Refusal) -> Box<[u8]> {
    let (prefix, number, suffix) = match refusal {
        Refusal::NoNumber { number } => (b"temper: no [".as_slice(), Some(number), b"] on the page\n".as_slice()),
        Refusal::Gone { number } => (b"temper: [".as_slice(), Some(number), b"] is gone\n".as_slice()),
        Refusal::NoWords { number } => (b"temper: [".as_slice(), Some(number), b"] takes no words\n".as_slice()),
        Refusal::NoPage => (b"temper: no page yet\n".as_slice(), None, b"".as_slice()),
        Refusal::TooLong => (b"temper: line too long\n".as_slice(), None, b"".as_slice()),
        Refusal::Unknown => (b"temper: commands: n, n words, p, q\n".as_slice(), None, b"".as_slice()),
    };
    let decimal = match number {
        Some(number) => Decimal::of(u64::from(number)),
        None => Decimal::of(0),
    };
    let digits = if number.is_some() { decimal.as_bytes() } else { b"" };
    let length = prefix.len().checked_add(digits.len()).expect("refusal prefix fits");
    let mut writer = Writer::new(length.checked_add(suffix.len()).expect("refusal length fits"));
    writer.put(prefix).expect("measured refusal prefix");
    writer.put(digits).expect("measured refusal number");
    writer.put(suffix).expect("measured refusal suffix");
    writer.finish()
}
