use crate::{
    host::LINE_BYTES,
    lines::{Lines, route},
};
use skein_lib::{List, bytes};
#[test]
fn a_line_is_written_with_its_step_s_name_before_it() {
    let mut lines = Lines::new(false);
    lines.push(bytes::copy_of(b"first\nsecond\n"));
    assert_eq!(lines.take(b"forgejo").as_deref(), Some(b"forgejo: first\n".as_slice()));
    assert_eq!(lines.take(b"forgejo").as_deref(), Some(b"forgejo: second\n".as_slice()));
    assert_eq!(lines.take(b"forgejo"), None);
}
#[test]
fn a_line_longer_than_the_cap_is_written_in_pieces_each_named() {
    let mut lines = Lines::new(false);
    let mut chunk = List::with_capacity(LINE_BYTES);
    for _ in 0..LINE_BYTES {
        chunk.push(b'x').expect("cap");
    }
    lines.push(chunk.into_boxed());
    let first = lines.take(b"runner").expect("full piece");
    assert_eq!(first.len(), 4_104, "name, colon, space and 4096 bytes");
    assert_eq!(first.get(..8), Some(b"runner: ".as_slice()));
    for byte in first.iter().skip(8) {
        assert_eq!(*byte, b'x', "piece preserves each byte");
    }
    lines.push(bytes::copy_of(b"tail\n"));
    assert_eq!(lines.take(b"runner").as_deref(), Some(b"runner: tail\n".as_slice()));
}
#[test]
fn an_unterminated_last_fragment_is_written_when_the_pipe_ends() {
    let mut lines = Lines::new(false);
    lines.push(bytes::copy_of(b"unterminated"));
    assert_eq!(lines.take(b"prepare"), None, "wait for LF or end");
    lines.end();
    assert_eq!(lines.take(b"prepare").as_deref(), Some(b"prepare: unterminated".as_slice()));
    assert_eq!(lines.take(b"prepare"), None);
}
#[test]
fn only_the_plain_steps_stdout_is_written_as_is() {
    for (step, fd, expected) in [(b"serve".as_slice(), 1_u32, true), (b"serve", 2, false), (b"runner", 1, false)] {
        assert_eq!(route(b"serve", step, fd), expected, "plain routing");
    }
    let mut lines = Lines::new(true);
    lines.push(bytes::copy_of(b"without prefix\nor a final LF"));
    assert_eq!(lines.take(b"serve").as_deref(), Some(b"without prefix\nor a final LF".as_slice()));
}
