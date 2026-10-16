//! Bounded line pieces, held until a sink takes them.
use crate::host::LINE_BYTES;
use alloc::boxed::Box;
use skein_lib::{List, Writer};
/// Whether a child's pipe goes to stdout unchanged.
#[must_use]
pub fn route(plain: &[u8], step: &[u8], child_fd: u32) -> bool {
    child_fd == 1 && plain == step
}
#[derive(Debug)]
pub struct Lines {
    plain: bool,
    chunk: Box<[u8]>,
    position: usize,
    piece: List<u8>,
    ended: bool,
}
impl Lines {
    #[must_use]
    pub fn new(plain: bool) -> Self {
        Self { plain, chunk: Box::new([]), position: 0, piece: List::with_capacity(LINE_BYTES), ended: false }
    }
    pub fn push(&mut self, bytes: Box<[u8]>) {
        assert!(self.chunk.is_empty(), "one chunk per pipe");
        assert!(bytes.len() <= usize::try_from(LINE_BYTES).expect("line cap fits"), "bounded child chunk");
        self.chunk = bytes;
    }
    pub fn end(&mut self) {
        self.ended = true;
    }
    #[must_use]
    pub fn wants_read(&self) -> bool {
        self.chunk.is_empty() && !self.ended
    }
    #[must_use]
    pub fn ready(&self) -> bool {
        !self.chunk.is_empty() || self.ended && !self.piece.is_empty()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunk.is_empty() && self.piece.is_empty()
    }
    pub fn take(&mut self, name: &[u8]) -> Option<Box<[u8]>> {
        if self.plain {
            if self.chunk.is_empty() {
                return None;
            }
            return Some(core::mem::replace(&mut self.chunk, Box::new([])));
        }
        let mut complete = false;
        for byte in self.chunk.iter().skip(self.position) {
            self.piece.push(*byte).expect("line cap checked after each byte");
            self.position = self.position.saturating_add(1);
            if *byte == b'\n' || self.piece.room() == 0 {
                complete = true;
                break;
            }
        }
        if self.position == self.chunk.len() {
            self.chunk = Box::new([]);
            self.position = 0;
        }
        if !complete && (!self.ended || self.piece.is_empty()) {
            return None;
        }
        let output = joined(&[name, b": ", self.piece.as_slice()]);
        self.piece.clear();
        Some(output)
    }
}
pub(crate) fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded output");
    }
    let mut writer = Writer::new(length);
    for part in parts {
        writer.put(part).expect("computed output length");
    }
    writer.finish()
}
