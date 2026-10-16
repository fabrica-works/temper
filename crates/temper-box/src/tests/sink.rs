use crate::sink::Sink;
use skein_io::Request;
use skein_lib::{
    Queue, Token, bytes,
    stream::{Down, Fault, Read, Up},
};
#[test]
fn a_sink_sends_no_more_than_the_room_it_was_granted() {
    let token = Token::new(3);
    let mut sink = Sink::new(token, 1);
    let mut out = Queue::with_capacity(4);
    sink.push(bytes::copy_of(b"runner: hello\n")).expect("one queue entry");
    assert_eq!(sink.push(bytes::copy_of(b"full")), Err(bytes::copy_of(b"full")));
    sink.tick(false, &mut out);
    let Some(Request::Stream { stream, down: Down::Demand { read, room } }) = out.pop() else {
        unreachable!("a queued line demands write room");
    };
    assert_eq!((stream, read, room), (token, Read::Nothing, 14));
    assert!(!sink.up(Up::Room, &mut out), "room is no failure");
    let Some(Request::Stream { stream, down: Down::Send(bytes) }) = out.pop() else {
        unreachable!("room permits one send");
    };
    assert_eq!(stream, token);
    assert_eq!(&*bytes, b"runner: hello\n");
    assert_eq!(bytes.len(), 14, "send stays within its grant");
    assert!(out.is_empty(), "one Room sends one entry");
}
#[test]
fn a_failed_sink_is_reported_once() {
    let mut sink = Sink::new(Token::new(3), 1);
    let mut out = Queue::with_capacity(4);
    assert!(sink.up(Up::Failed(Fault::Other), &mut out), "first failure reported");
    assert!(!sink.up(Up::Failed(Fault::Other), &mut out), "duplicate failure suppressed");
    sink.tick(false, &mut out);
    assert_eq!(out.len(), 1, "failed sink closes once");
    sink.tick(false, &mut out);
    assert_eq!(out.len(), 1, "close is not repeated");
}
