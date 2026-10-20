use super::fixtures;
use skein_io::kernel::{Error, Exit, Kind, Stat};
const ADD: &[&[u8]] = &[b"add-key", b"1", b"/linked-parent/link-file", b"--deployment", b"--key", b"key"];
const AT_BOUND: &[u8] = &[b'k'; 4096];
const OVER_BOUND: &[u8] = &[b'k'; 4097];
const BOUND_NEWLINE: &[u8] = &{
    let mut bytes = [b'k'; 4097];
    bytes[4096] = b'\n';
    bytes
};

#[test]
fn a_key_file_strips_only_one_final_newline() {
    for (file, expected) in [
        (b"private-value\n".as_slice(), b"private-value".as_slice()),
        (b"private-value\n\n", b"private-value\n"),
        (AT_BOUND, AT_BOUND),
        (BOUND_NEWLINE, AT_BOUND),
    ] {
        let mut f = fixtures::Frame::act(ADD);
        f.link = file;
        f.sign_in_then_deliver_me();
        assert_eq!(f.carried_keys.len(), 1);
        assert_eq!(f.carried_keys[0].expose(), expected);
        let store = f.stored(b"store").expect("kept act");
        assert!(!store.windows(expected.len()).any(|w| w == expected));
        assert!(f.stdout().is_empty());
        assert!(f.stderr().is_empty());
    }
}
#[test]
fn an_invalid_key_size_opens_no_directory() {
    for file in [b"".as_slice(), b"\n", OVER_BOUND] {
        let mut f = fixtures::Frame::act(ADD);
        f.link = file;
        f.drain();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert!(!f.opened(b"directory"));
        assert_eq!(f.requests(), 0);
    }
}
#[test]
fn a_key_file_requires_private_regular_owned_metadata() {
    for (kind, mode, owner) in [
        (Kind::Directory, 0o600, 7),
        (Kind::Other, 0o600, 7),
        (Kind::File, 0o640, 7),
        (Kind::File, 0o606, 7),
        (Kind::File, 0o600, 8),
    ] {
        let mut f = fixtures::Frame::act(ADD);
        f.link = b"secret-value";
        f.input_stat = Some(Stat { kind, mode, owner, size: f.link.len() as u64, links: 1 });
        f.drain();
        assert_eq!(f.exit(), Some(Exit::Code(1)));
        assert!(!f.opened(b"directory"));
        assert_eq!(f.requests(), 0);
        assert!(!f.stderr().windows(f.link.len()).any(|w| w == f.link));
    }
}
#[test]
fn a_key_file_refuses_a_link_at_its_name() {
    let mut f = fixtures::Frame::act(ADD);
    f.linked_leaf = Some(Error::TooManyLinks);
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert!(!f.opened(b"directory"));
    assert_eq!(f.stderr(), b"temper ctl: key file must not be a link\n");
}
#[test]
fn a_missing_key_file_names_the_file() {
    let mut f = fixtures::Frame::act(ADD);
    f.linked_leaf = Some(Error::NotFound);
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert!(!f.opened(b"directory"));
    assert_eq!(f.stderr(), b"temper ctl: key file not found\n");
}
#[test]
fn a_key_file_accepts_its_opened_parent() {
    let mut f = fixtures::Frame::act(ADD);
    f.link = b"private-key";
    f.sign_in_then_deliver_me();
    assert_eq!(f.link_root, Some(fixtures::FILE_PARENT));
    assert_eq!(f.link_leaf.as_deref(), Some(b"link-file".as_slice()));
    assert_eq!(f.carried_keys[0].expose(), f.link);
}
#[test]
fn json_cannot_name_a_key_file() {
    let mut f = fixtures::Frame::read(&["--call", "-"]);
    f.input = br#"{"tool":"add-key","arguments":{"provider":1,"file":"link-file","key":"add"}}"#;
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert!(!f.opened(b"directory"));
}
