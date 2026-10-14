mod fixtures;
use crate::{
    client,
    decode::Malformed,
    document::{Body, Document},
    id, path,
    request::Request,
    time,
};
use alloc::boxed::Box;
use skein_lib::{Wall, Writer};
#[test]
fn names_in_paths_are_percent_encoded_one_segment_each() {
    for (name, expected) in [
        (b"main".as_slice(), b"main".as_slice()),
        (b"feature/slash", b"feature%2Fslash"),
        (b"a b", b"a%20b"),
        ("é".as_bytes(), b"%C3%A9"),
    ] {
        let mut out = Writer::new(expected.len());
        path::segment(name, &mut out).unwrap();
        assert_eq!(&*out.finish(), expected, "{name:?}");
    }
    let mut out = Writer::new(13);
    path::file(b"docs/a b.md", &mut out).unwrap();
    assert_eq!(&*out.finish(), b"docs/a%20b.md");
    let target = path::read(b"/api/v1/repos/a%2Fb/%C3%A9?q=a+b", &fixtures::limits()).unwrap();
    assert_eq!(target.segments[3].as_ref(), b"a/b");
    assert_eq!(target.query[0].value.as_ref(), b"a b");
}
#[test]
fn rfc3339_times_read_with_their_offset_and_write_in_utc() {
    for (text, seconds) in [
        (b"2020-01-01T00:00:00Z".as_slice(), 1_577_836_800),
        (b"2020-01-01T05:30:00+05:30", 1_577_836_800),
        (b"2019-12-31T17:00:00-07:00", 1_577_836_800),
        (b"2020-01-01T00:00:00.999Z", 1_577_836_800),
    ] {
        let wall = time::read(text).unwrap();
        assert_eq!(wall.as_secs(), seconds, "{text:?}");
        assert_eq!(time::write(wall), *b"2020-01-01T00:00:00Z");
    }
    for text in
        [b"1969-12-31T23:59:59Z".as_slice(), b"2020-02-30T00:00:00Z", b"2020-01-01T00:00:00", b"2554-07-21T23:34:34Z"]
    {
        assert_eq!(time::read(text), None, "{text:?}");
    }
    assert_eq!(time::write(Wall::from_nanos(u64::MAX)), *b"2554-07-21T23:34:33Z");
    assert_eq!(time::read(b"1970-01-01T00:00:00Z"), Some(Wall::EPOCH));
}
#[test]
fn object_ids_read_40_or_64_lowercase_digits_only() {
    for (length, valid) in [(40, true), (64, true), (39, false), (41, false)] {
        let digits = Box::from([b'a'; 64].get(..length).unwrap());
        let result = id::read(&digits);
        assert_eq!(result.is_some(), valid, "{length}");
        if let Some(value) = result {
            assert_eq!(id::write(&value), digits);
        }
    }
    assert_eq!(id::read(&[b'A'; 40]), None);
    assert_eq!(id::unsigned(b"18446744073709551615"), Some(u64::MAX));
    for text in [b"18446744073709551616".as_slice(), b"-1", b"1.0", b"1e0", b""] {
        assert_eq!(id::unsigned(text), None, "{text:?}");
    }
}
#[test]
fn a_start_document_missing_a_field_is_malformed_naming_it() {
    let limits = fixtures::limits();
    let rows:[(&str,Request,&[u8],Malformed);2]=[
        ("user without login",Request::User,br#"{"id":2,"is_admin":false}"#,Malformed::Missing{field:b"login"}),
        ("repository without its format",Request::RepositoryById{id:1},br#"{"id":1,"owner":{"id":1,"login":"fixture"},"name":"specimen","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":true,"allow_rebase":true,"html_url":"http://127.0.0.1:3000/fixture/specimen"}"#,Malformed::Missing{field:b"object_format_name"})];
    for (name, request, body, expected) in rows {
        let decoder = client::decoder(&request, &limits).unwrap();
        assert_eq!(fixtures::decode(decoder, body, &limits).document, Err(expected), "{name}");
    }
}
#[test]
fn a_string_past_its_cap_is_malformed_and_a_body_past_its_cap_is_long() {
    let limits = fixtures::small();
    for (text, valid) in
        [(br#"{"id":2,"login":"12345678"}"#.as_slice(), true), (br#"{"id":2,"login":"123456789"}"#, false)]
    {
        let decoder = client::decoder(&Request::User, &limits).unwrap();
        let result = fixtures::decode(decoder, text, &limits);
        if valid {
            assert_eq!(
                result.document,
                Ok(Document::User(crate::document::User { id: 2, login: Box::from(*b"12345678") }))
            );
        } else {
            assert_eq!(result.document, Err(Malformed::TooLong { field: b"login" }));
        }
    }
    let long = fixtures::capped(br#""12345678901234567""#, limits.body_bytes, &limits);
    assert_eq!(Body::read(&long, b"body"), Ok(Body::Long(17)));
    assert_eq!(
        Body::read(&fixtures::capped(br#""1234567890123456""#, limits.body_bytes, &limits), b"body"),
        Ok(Body::Kept(Box::from(*b"1234567890123456")))
    );
}
#[test]
fn start_documents_refuse_duplicate_null_and_wrongly_typed_fields() {
    let limits = fixtures::limits();
    for (body, expected) in [
        (br#"{"id":2,"id":3,"login":"writer"}"#.as_slice(), Malformed::Shape { field: b"id" }),
        (br#"{"id":null,"login":"writer"}"#, Malformed::Missing { field: b"id" }),
        (br#"{"id":"2","login":"writer"}"#, Malformed::Shape { field: b"id" }),
        (br#"{"id":-1,"login":"writer"}"#, Malformed::Range { field: b"id" }),
    ] {
        let decoder = client::decoder(&Request::User, &limits).unwrap();
        assert_eq!(fixtures::decode(decoder, body, &limits).document, Err(expected), "{body:?}");
    }
}
#[test]
fn unknown_nested_fields_are_skipped_without_retaining_their_strings() {
    let limits = fixtures::small();
    let decoder = client::decoder(&Request::User, &limits).unwrap();
    let result = fixtures::decode(
        decoder,
        br#"{"extra":{"login":"long unknown string","nested":[{}]},"id":1,"login":"fixture"}"#,
        &limits,
    );
    assert_eq!(result.document, Ok(Document::User(crate::document::User { id: 1, login: Box::from(*b"fixture") })));
    assert!(result.rows.is_empty());
}
#[test]
fn target_bounds_and_malformed_escapes_are_refused_before_routing() {
    let mut limits = fixtures::limits();
    limits.target_bytes = 12;
    assert_eq!(client::encode(&Request::User, &limits).unwrap().target.as_ref(), b"/api/v1/user");
    limits.target_bytes = 11;
    assert_eq!(client::encode(&Request::User, &limits), Err(client::Refusal::TooLong));
    assert_eq!(path::read(b"/api/v1/user", &limits), Err(path::Unread::TooLong));
    for target in [b"/bad%".as_slice(), b"/bad%2", b"/bad%GG"] {
        assert_eq!(path::read(target, &limits), Err(path::Unread::Escape), "{target:?}");
    }
    assert_eq!(
        crate::server::route(crate::request::Method::Post, b"/api/v1/user", &fixtures::limits()).unwrap_err(),
        crate::server::Unrouted::Method
    );
    assert_eq!(
        crate::server::route(crate::request::Method::Get, b"/api/v1/no-route", &fixtures::limits()).unwrap_err(),
        crate::server::Unrouted::NotFound
    );
}
