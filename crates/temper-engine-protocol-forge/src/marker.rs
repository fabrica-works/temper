//! Key markers at the head of written bodies (forge.md, section 4).
use alloc::boxed::Box;
use skein_lib::{Writer, bytes};
/// A marker's first bytes.
pub const HEAD: &[u8] = b"<!-- temper:key ";
/// A marker's closing bytes.
pub const TAIL: &[u8] = b" -->\n";
/// Twice the key's length and its fixed framing.
#[must_use]
pub fn length(key: u32) -> Option<u32> {
    key.checked_mul(2)?.checked_add(21)
}
/// Writes a marker and text whole, within `limit`.
#[must_use]
pub fn encode(key: &[u8], text: &[u8], limit: u32) -> Option<Box<[u8]>> {
    let size = length(u32::try_from(key.len()).ok()?)?.checked_add(u32::try_from(text.len()).ok()?)?;
    if size > limit {
        return None;
    }
    let mut out = Writer::new(usize::try_from(size).ok()?);
    out.put(HEAD).ok()?;
    for byte in key {
        out.put(&[hex(byte / 16), hex(byte % 16)]).ok()?;
    }
    out.put(TAIL).ok()?;
    out.put(text).ok()?;
    Some(out.finish())
}
/// Reads only a well-formed marker at the very beginning of a body.
#[must_use]
pub fn decode(body: &[u8]) -> Option<Box<[u8]>> {
    let rest = body.strip_prefix(HEAD)?;
    let end = bytes::find(rest, TAIL)?;
    if end % 2 != 0 {
        return None;
    }
    let digits = rest.get(..end)?;
    let mut out = Writer::new(end / 2);
    for pair in digits.chunks_exact(2) {
        let high = digit(*pair.first()?)?;
        let low = digit(*pair.get(1)?)?;
        out.put(&[high.checked_mul(16)?.checked_add(low)?]).ok()?;
    }
    Some(out.finish())
}
fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10),
        _ => None,
    }
}
pub(crate) fn hex(byte: u8) -> u8 {
    *b"0123456789abcdef".get(usize::from(byte)).expect("one hexadecimal digit")
}
