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
#[test]
fn a_read_never_opens_the_store() {
    let mut f = fixtures::Engine::read(&["me", "--json"]);
    f.sign_in_then_deliver_me();
    f.drain();
    assert!(!f.opened(b"store"));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_read_waits_for_me() {
    let mut f = fixtures::Engine::read(&["me", "--json"]);
    f.drain();
    assert_eq!(f.exit(), None);
    assert_eq!(f.stdout(), b"");
    f.sign_in_then_deliver_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(f.stdout().starts_with(b"{\""));
}
#[test]
fn an_unheard_read_exits_1_at_the_link_deadline() {
    let mut f = fixtures::Engine::read(&["me"]);
    f.drain();
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.stderr(), b"temper ctl: the engine at http://127.0.0.1:4000 did not answer within 30 s\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_read_sends_no_stream_gone() {
    let mut f = fixtures::Engine::read(&["me"]);
    f.sign_in_then_deliver_me();
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(!f.sent_stream_gone());
}
#[test]
fn memory_saves_are_stored_in_order_without_file_operations() {
    use jig_client::{Event, Request};
    use skein_lib::{List, Time};
    let limits = limits::native();
    let mut memory = crate::transport::Memory::new(
        directory::address(b"http://127.0.0.1:4000\n").expect("fixture address"),
        &limits.connections,
    );
    for request in [
        Request::Save { records: List::with_capacity(0) },
        Request::Erase { records: List::with_capacity(0) },
        Request::Save { records: List::with_capacity(0) },
    ] {
        memory.down(Time::ZERO, request);
        assert_eq!(memory.events.pop(), Some(Event::Stored));
    }
    assert_eq!(memory.events.pop(), None);
    assert!(memory.progress(Time::ZERO).is_empty());
}
#[test]
fn host_bounds_cover_its_holders() {
    use skein_shell::Host;
    let f = fixtures::Engine::read(&["me"]);
    let limits = limits::native();
    let holders = limits::transport(&limits).expect("native transport bound")
        + jig_client::worst_case(&limits.root.domain.client).expect("native root bound")
        + skein_io::worst_case(&limits.io).expect("native io bound");
    assert!(f.host.worst_case() > holders);
    assert_eq!(f.host.operations(), skein_io::operations(&limits.io).expect("native operation bound") * 2 + 5);
}
#[test]
fn sign_in_keeps_its_record_before_success() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), None);
    f.store_token_with_address();
    assert_eq!(f.stored(b"address"), Some(b"http://127.0.0.1:4000\n".as_slice()));
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
#[test]
fn a_non_loopback_link_exits_1_before_sending() {
    let mut f = fixtures::Disk::sign_in(
        b"http://192.0.2.1:4000/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    f.drain();
    assert_eq!(f.stderr(), b"temper ctl: the engine's address must be on loopback\n");
    assert_eq!(f.requests(), 0);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn a_refused_link_never_prints_its_secret() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.refuse_link = Some(());
    f.answer_sign_in_then_me();
    assert_eq!(f.stdout(), b"");
    assert_eq!(f.stderr(), b"temper ctl: the sign-in link was refused\n");
    assert_eq!(f.stored(b"token"), None);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sign_in_takes_the_lock() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.lock_busy();
    assert_eq!(f.locks, 1);
    assert!(!f.opened(b"link-file"));
    assert_eq!(f.requests(), 0);
    f.advance_past(limits::LOCK);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
#[test]
fn sign_in_waits_for_live_me() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.store_token_with_address();
    assert_eq!(f.stored(b"token"), Some(fixtures::TOKEN_BYTES));
    assert_eq!(f.exit(), None);
    assert_eq!(f.stdout(), b"");
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(!f.stdout().is_empty());
}
