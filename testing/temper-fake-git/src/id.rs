//! The forge's counted ids (forge.md, section 9).
/// Count big-endian, then twelve zero bytes, as forty lowercase hex digits.
#[must_use]
pub fn id(count: u64) -> [u8; 40] {
    let mut raw = [0; 20];
    raw[..8].copy_from_slice(&count.to_be_bytes());
    let mut out = [b'0'; 40];
    for (byte, digits) in raw.iter().zip(out.chunks_exact_mut(2)) {
        digits[0] = b"0123456789abcdef"[usize::from(byte >> 4)];
        digits[1] = b"0123456789abcdef"[usize::from(byte & 15)];
    }
    out
}
