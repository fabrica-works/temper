use crate::{
    fixture::encode,
    mask::{self, Mask},
    write::{self, Leak, SecretKind, Secrets},
};
use temper_throwaway_forgejo::Exchange;
#[test]
fn masking_rewrites_ports_ids_times_and_dates_by_first_appearance() {
    let rows: &[(&[u8], &[u8])] = &[
        (b"http://127.0.0.1:41823/1", b"http://127.0.0.1:3000/1"),
        (b"9fceb02d0ae598e95dc970b74767f19372d61af8", b"0000000000000000000000000000000000000001"),
        (b"\"2026-10-14T10:12:13+05:30\"", b"\"2020-01-01T00:00:01+05:30\""),
        (
            b"Date: Fri, 16 Oct 2026 00:00:00 GMT\r\nLast-Modified: yesterday\r\n",
            b"Date: Wed, 01 Jan 2020 00:00:00 GMT\r\nLast-Modified: Wed, 01 Jan 2020 00:00:00 GMT\r\n",
        ),
        (b"\"0001-01-01T00:00:00Z\" \"1970-01-01T00:00:00Z\"", b"\"0001-01-01T00:00:00Z\" \"1970-01-01T00:00:00Z\""),
    ];
    for (input, expected) in rows {
        assert_eq!(Mask::new(41823).bytes(input), *expected, "{input:?}");
    }
    let mut mask = Mask::new(3000);
    assert_eq!(mask.bytes(b"9fceb02d0ae598e95dc970b74767f19372d61af8"), b"0000000000000000000000000000000000000001");
    assert_eq!(
        mask.bytes(b"9fceb02d0ae598e95dc970b74767f19372d61af8 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        b"0000000000000000000000000000000000000001 0000000000000000000000000000000000000002"
    );
}
#[test]
fn a_chunked_response_is_written_unframed_with_its_masked_length() {
    let exchange = Exchange {
        sent: Vec::new(),
        status: 200,
        head: b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\n".to_vec(),
        body: b"127.0.0.1:41823".to_vec(),
    };
    assert_eq!(
        mask::response(&mut Mask::new(41823), &exchange),
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 14\r\n\r\n127.0.0.1:3000"
    );
}
#[test]
fn a_secret_in_a_case_stops_the_run_naming_the_case() {
    let secrets = Secrets::of(&[(SecretKind::ApiToken, "0123456789abcdef0123456789abcdef01234567")]);
    let leaked = write::check(
        "user",
        b"GET /api/v1/user HTTP/1.1\r\nAuthorization: token MASKED\r\n\r\n",
        b"HTTP/1.1 401 Unauthorized\r\n\r\n{\"message\":\"token 0123456789abcdef0123456789abcdef01234567\"}",
        &secrets,
    );
    assert_eq!(leaked, Err(Leak { case: "user".to_owned(), kind: SecretKind::ApiToken }));
}
#[test]
fn targets_encode_every_byte_but_unreserved_ones() {
    for (plain, encoded) in [
        ("ABCxyz019-._~", "ABCxyz019-._~"),
        ("feature/slash", "feature%2Fslash"),
        ("temper / label", "temper%20%2F%20label"),
        ("é+?&:", "%C3%A9%2B%3F%26%3A"),
    ] {
        assert_eq!(encode(plain), encoded, "{plain}");
    }
}
