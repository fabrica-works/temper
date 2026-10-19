//! The first eight SHA-256 bytes of the common subject document.
use crate::jig::{Limits, render::Part};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digest([u8; 8]);
impl Digest {
    #[must_use]
    pub fn of(json: &[u8]) -> Self {
        let hash = skein_lib::sha256::digest(json);
        Self(hash.get(..8).expect("SHA-256 prefix").try_into().expect("eight bytes"))
    }
    /// Already checked by the table; no second parser.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }
    #[must_use]
    pub fn hex(self) -> [u8; 16] {
        let mut hex = [0; 16];
        for (pair, byte) in hex.chunks_exact_mut(2).zip(self.0) {
            for (slot, nibble) in pair.iter_mut().zip([byte >> 4u32, byte & 15]) {
                *slot = *b"0123456789abcdef".get(usize::from(nibble)).expect("hex digit");
            }
        }
        hex
    }
}

#[must_use]
pub fn part(part: &Part<'_>, limits: &Limits) -> Digest {
    Digest::of(&crate::jig::render::document::part_bytes(part, limits))
}
