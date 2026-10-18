use super::commit_text;

const SHA1: [u8; 32] = [
    0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const SHA256: [u8; 32] = [0xab; 32];
const COMMITS: &[([u8; 32], &[u8])] = &[
    (SHA1, b"abcdef0123456789abcdef0123456789abcdef01"),
    (SHA256, b"abababababababababababababababababababababababababababababababab"),
];

#[test]
fn commit_text_uses_lowercase_digits_at_the_identifier_width() {
    for (commit, digits) in COMMITS {
        assert_eq!(commit_text(commit).as_ref(), *digits, "commit {commit:?}");
    }
}
