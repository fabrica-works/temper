//! Connection machinery, before the component wires it in 13.2b.
use super::fixtures;
use crate::{
    connection::{Failure, Heard, Phase},
    grow::{Full, Grow},
};
use alloc::boxed::Box;
use skein_lib::{
    Wall,
    stream::{Fault, Up},
};
use temper_forge_forgejo::request;

#[test]
fn the_capped_buffer_doubles_and_refuses_a_write_past_its_cap() {
    let mut grow = Grow::new(4, 10);
    let rows: [(&str, &[u8], u32); 3] = [("first piece", b"abc", 4), ("doubled", b"def", 8), ("capped", b"ghij", 10)];
    for (name, piece, expected) in rows {
        assert_eq!(grow.write(piece), Ok(()), "{name}: piece fits");
        assert_eq!(grow.capacity(), expected, "{name}: capacity");
    }
    assert_eq!(grow.write(b"k"), Err(Full), "one past refused whole");
    assert_eq!(grow.finish().as_ref(), b"abcdefghij", "nothing changed by refusal");
}

#[test]
fn a_reset_before_the_first_byte_is_unseen_and_after_it_seen() {
    for (name, send, expected) in [("before room", false, Failure::Unseen), ("after send", true, Failure::Seen)] {
        let mut h = fixtures::ConnectionHarness::new();
        h.connection.start(&h.env, &h.forge, request::Request::User, None).expect("valid request");
        h.pump();
        if send {
            h.connection.stream(&h.env, Up::Room);
            h.pump();
        }
        h.connection.stream(&h.env, Up::Failed(Fault::Reset));
        h.pump();
        assert_eq!(h.connection.heard(), Some(Heard::Failed(expected)), "{name}");
        assert_eq!(h.connection.heard(), None, "{name}: one failure");
    }
}

#[test]
fn a_json_response_is_decoded_through_its_body_end() {
    let mut h = fixtures::ConnectionHarness::new();
    h.connection.start(&h.env, &h.forge, request::Request::User, None).expect("valid request");
    let heard = h.answer(b"HTTP/1.1 200 OK\r\nContent-Length: 25\r\nDate: Wed, 01 Jan 2020 00:00:00 GMT\r\n\r\n{\"id\":2,\"login\":\"writer\"}");
    assert_eq!(heard.len(), 2, "head then document");
    assert_eq!(
        heard.get(0),
        Some(&Heard::Head {
            status: 200,
            fields: crate::connection::Fields {
                date: Some(Wall::from_nanos(1_577_836_800_000_000_000)),
                retry_after: None,
                has_more: None,
                content_range: None,
                close: false
            }
        }),
        "response fields"
    );
    assert_eq!(
        heard.get(1),
        Some(&Heard::Done(crate::connection::Body::Document(temper_forge_forgejo::document::Document::User(
            temper_forge_forgejo::document::User { id: 2, login: Box::from(&b"writer"[..]) }
        )))),
        "decoded whole"
    );
    assert_eq!(h.connection.phase, Phase::Idle, "reusable");
}

#[test]
fn a_malformed_document_closes_without_reading_more() {
    let mut h = fixtures::ConnectionHarness::new();
    h.connection.start(&h.env, &h.forge, request::Request::User, None).expect("valid request");
    let heard = h.answer(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}");
    let Some(Heard::Done(crate::connection::Body::Malformed(_))) = heard.last() else {
        unreachable!("malformed terminal");
    };
    assert_eq!(h.connection.phase, Phase::Closed, "malformed closes");
}
