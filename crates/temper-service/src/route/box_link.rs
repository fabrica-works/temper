//! The routing's box link: serve's side of the box's pipe (protocol/forge.md, 2.8).
#![cfg_attr(not(test), expect(dead_code, reason = "task C (o/forgejo-setup-m2) wires the box link"))]

use alloc::boxed::Box;
use skein_lib::{Decimal, List, Queue, Writer, bytes};

/// The box's word for the step it runs (protocol/setup.md, 7.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Fetch,
    Prepare,
    Run,
}
/// Why the link refuses an ask (protocol/forge.md, 2.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Refused {
    NotUnderTheBox,
    Busy,
}
/// One outcome of a link input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Out {
    /// A line for fd 3, its line feed included.
    Write(Box<[u8]>),
    /// The ask's one answer: setting up, or its refusal.
    Answered { forge: u16, answer: Result<(), Refused> },
    /// The forge setting up: `None` at the ask, then each `step` line's step.
    SettingUp { forge: u16, step: Option<Step> },
    /// `started FORGE`: the ask ended, the group started.
    Started { forge: u16 },
    /// `failed FORGE WHY`, or the pipe closed during the ask.
    Failed { forge: u16, why: Box<[u8]> },
    /// One log line naming what the link dropped, or the pipe's end.
    Named(Box<[u8]>),
}

pub(crate) struct Link {
    piped: bool,
    asked: Option<u16>,
    line_bytes: u32,
    partial: List<u8>,
    skipping: bool,
}
impl Link {
    /// With no pipe, every ask is refused `NotUnderTheBox`.
    pub(crate) fn new(line_bytes: u32, piped: bool) -> Link {
        Link { piped, asked: None, line_bytes, partial: List::with_capacity(line_bytes), skipping: false }
    }
    /// The caller reserves three output slots before asking.
    pub(crate) fn ask(&mut self, forge: u16, out: &mut Queue<Out>) {
        let refusal = if !self.piped {
            Some(Refused::NotUnderTheBox)
        } else if self.asked.is_some() {
            Some(Refused::Busy)
        } else {
            None
        };
        if let Some(reason) = refusal {
            let text = match reason {
                Refused::NotUnderTheBox => b"not under the box".as_slice(),
                Refused::Busy => b"busy".as_slice(),
            };
            out.push(Out::Answered { forge, answer: Err(reason) });
            out.push(Out::Named(joined(&[
                b"box link: refused forge ",
                Decimal::of(u64::from(forge)).as_bytes(),
                b": ",
                text,
            ])));
            return;
        }
        self.asked = Some(forge);
        out.push(Out::Write(joined(&[b"start ", Decimal::of(u64::from(forge)).as_bytes(), b"\n"])));
        out.push(Out::SettingUp { forge, step: None });
        out.push(Out::Answered { forge, answer: Ok(()) });
    }
    /// Any chunking is accepted. The caller reserves one output slot per LF in
    /// this chunk (including an LF completing a previous partial line).
    /// No output is emitted before LF, so a bounded read bounds its outputs.
    pub(crate) fn read(&mut self, input: &[u8], out: &mut Queue<Out>) {
        if !self.piped {
            return;
        }
        for byte in input {
            if self.skipping || self.partial.room() == 0 {
                self.skipping = true;
                if *byte == b'\n' {
                    self.skipping = false;
                    out.push(Out::Named(joined(&[
                        b"box link: dropped a line past box.line_bytes (",
                        Decimal::of(u64::from(self.line_bytes)).as_bytes(),
                        b")",
                    ])));
                }
                continue;
            }
            // As in temper-box's Reader, the LF is inside line_bytes. The
            // partial buffer is allocated once at new, and never grows.
            self.partial.push(*byte).expect("room checked per byte");
            if *byte == b'\n' {
                line(&mut self.asked, self.partial.as_slice(), out);
                self.partial.clear();
            } else if self.partial.room() == 0 {
                self.partial.clear();
                self.skipping = true;
            }
        }
    }
    /// fd 4 ended, or a read/write failed. The caller reserves two output slots.
    pub(crate) fn closed(&mut self, out: &mut Queue<Out>) {
        if !self.piped {
            return;
        }
        self.piped = false;
        self.partial.clear();
        self.skipping = false;
        out.push(Out::Named(Box::from(b"box link: the box closed the pipe".as_slice())));
        if let Some(forge) = self.asked.take() {
            out.push(Out::Failed { forge, why: Box::from(b"the box closed the pipe".as_slice()) });
        }
    }
}

fn line(asked: &mut Option<u16>, line: &[u8], out: &mut Queue<Out>) {
    match decoded(line) {
        Some((forge, word, event)) => {
            if *asked != Some(forge) {
                out.push(Out::Named(joined(&[
                    b"box link: dropped ",
                    word,
                    b" for forge ",
                    Decimal::of(u64::from(forge)).as_bytes(),
                    b", not asked",
                ])));
                return;
            }
            match &event {
                Out::Started { .. } | Out::Failed { .. } => *asked = None,
                Out::SettingUp { .. } => {}
                Out::Write(_) | Out::Answered { .. } | Out::Named(_) => unreachable!("decoded box line"),
            }
            out.push(event);
        }
        None => out.push(Out::Named(Box::from(b"box link: dropped an unknown line".as_slice()))),
    }
}
fn decoded(line: &[u8]) -> Option<(u16, &'static [u8], Out)> {
    let line = line.strip_suffix(b"\n")?;
    if let Some(digits) = line.strip_prefix(b"started ") {
        let forge = number(digits)?;
        return Some((forge, b"started", Out::Started { forge }));
    }
    let (word, rest) = if let Some(rest) = line.strip_prefix(b"step ") {
        (b"step".as_slice(), rest)
    } else {
        (b"failed".as_slice(), line.strip_prefix(b"failed ")?)
    };
    let mut space = None;
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b' ' {
            space = Some(index);
            break;
        }
    }
    let (digits, tail) = rest.split_at_checked(space?)?;
    let forge = number(digits)?;
    let tail = tail.strip_prefix(b" ")?;
    if word == b"step" {
        let step = match tail {
            b"fetch" => Step::Fetch,
            b"prepare" => Step::Prepare,
            b"run" => Step::Run,
            _ => return None,
        };
        Some((forge, b"step", Out::SettingUp { forge, step: Some(step) }))
    } else if tail.is_empty() {
        None
    } else {
        Some((forge, b"failed", Out::Failed { forge, why: bytes::copy_of(tail) }))
    }
}
fn number(digits: &[u8]) -> Option<u16> {
    if digits.is_empty() {
        return None;
    }
    let mut number = 0_u16;
    for byte in digits {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded line pieces");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("computed line length");
    }
    writer.finish()
}
