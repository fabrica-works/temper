//! Lines for the terminal (client.md, 6.2 and 6.5).
#![deny(
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::unwrap_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm
)]
use skein_lib::List;

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
