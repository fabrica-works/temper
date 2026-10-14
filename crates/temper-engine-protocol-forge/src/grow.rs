//! Forge connection grow (forge.md, sections 3 and 8).
use alloc::boxed::Box;
use skein_lib::List;

/// A write past the cap, refused whole: nothing written.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Full;

/// Bytes of unknown length, at most `cap`.
#[derive(Debug)]
pub struct Grow {
    bytes: List<u8>,
    cap: u32,
}

impl Grow {
    /// Starts at `first` bytes (at most `cap`).
    #[must_use]
    pub fn new(first: u32, cap: u32) -> Grow {
        Grow { bytes: List::with_capacity(first.min(cap)), cap }
    }

    /// Appends `piece` whole, growing to twice the capacity or what it needs, never past `cap`.
    pub fn write(&mut self, piece: &[u8]) -> Result<(), Full> {
        let Ok(length) = u32::try_from(piece.len()) else { return Err(Full) };
        let Some(need) = self.bytes.len().checked_add(length) else { return Err(Full) };
        if need > self.cap {
            return Err(Full);
        }
        if need > self.bytes.capacity() {
            let mut grown = List::with_capacity(need.max(self.bytes.capacity().saturating_mul(2)).min(self.cap));
            for byte in self.bytes.as_slice() {
                grown.push(*byte).expect("grown past the length");
            }
            self.bytes = grown;
        }
        for byte in piece {
            self.bytes.push(*byte).expect("room checked above");
        }
        Ok(())
    }

    /// The bytes, in a box of exactly their length.
    #[must_use]
    pub fn finish(self) -> Box<[u8]> {
        self.bytes.into_boxed()
    }
}
impl Grow {
    #[cfg(test)]
    pub(crate) fn capacity(&self) -> u32 {
        self.bytes.capacity()
    }
}
