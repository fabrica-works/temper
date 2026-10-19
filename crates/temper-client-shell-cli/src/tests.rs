mod fixtures;
use crate::{directory, limits};
use skein_io::kernel::{Exit, ServiceSignal};
#[test]
fn a_busy_directory_is_refused_after_60_s_of_retries() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.lock_busy();
    assert_eq!(f.next_deadline(), Some(f.started().saturating_add(limits::LOCK_TRY)));
    f.advance_past(limits::LOCK);
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is busy with another call\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn calls_that_send_nothing_take_no_lock() {
    let cases: &[&[&[u8]]] = &[&[b"me"], &[b"wait", b"task", b"41"], &[b"pending"], &[b"help"]];
    for words in cases {
        let mut f = fixtures::Frame::act(words);
        f.run();
        assert_eq!(f.locks, 0, "{words:?}");
    }
}
#[test]
fn a_name_address_is_refused() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.address = b"http://localhost:4000\n";
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: the engine's address must be on loopback\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn numeric_loopback_addresses_are_accepted() {
    for address in [b"http://127.3.4.5:4000\n".as_slice(), b"http://[::1]:4000\n"] {
        assert!(directory::address(address).is_ok(), "{address:?}");
    }
}
#[test]
fn a_non_loopback_address_exits_1_before_the_token() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.address = b"http://192.0.2.1:4000\n";
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: the engine's address must be on loopback\n");
    assert_eq!(f.token_reads, 0);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_refused_directory_exits_1() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.refused = true;
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is not private: Mode(63)\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sigterm_exits_143_printing_nothing_more() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.signal(ServiceSignal::Terminate);
    assert_eq!(f.stderr(), b"");
    assert_eq!(f.exit(), Some(Exit::Code(limits::TERMINATED)));
}
#[test]
fn sigint_exits_130() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.signal(ServiceSignal::Interrupt);
    assert_eq!(f.exit(), Some(Exit::Code(limits::INTERRUPTED)));
}
#[test]
fn partial_writes_finish_before_exit() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.absent = true;
    f.write_size = fixtures::PARTIAL;
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY does not exist\n");
    assert!(f.writes > 1);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn an_absent_directory_is_never_made() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.absent = true;
    f.run();
    assert_eq!(f.private_opens, 0);
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY does not exist\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_zero_byte_token_is_not_signed_in() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.token = b"";
    f.run();
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is not signed in\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
