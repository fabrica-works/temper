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
#[test]
fn a_fact_line_is_holds_fails_or_unchecked_with_its_reason() {
    use crate::facts::{Fact, Outcome};
    for (outcome, expected) in [
        (Outcome::Holds, "holds example\n"),
        (Outcome::Fails("HTTP 500".into()), "fails example: HTTP 500\n"),
        (Outcome::Unchecked("runner absent".into()), "unchecked example: runner absent\n"),
    ] {
        assert_eq!(Fact { name: "example", outcome }.line(), expected);
    }
}
#[test]
fn a_fact_read_from_a_case_compares_its_status_and_message() {
    let mut exchange = Exchange {
        sent: Vec::new(),
        status: 409,
        head: b"HTTP/1.1 409 Conflict\r\n".to_vec(),
        body: b"{\"message\":\"head out of date\"}".to_vec(),
    };
    assert!(crate::facts::matches(&exchange, 409, "head out of date"));
    exchange.status = 405;
    assert!(!crate::facts::matches(&exchange, 409, "head out of date"));
}
#[test]
fn the_component_block_is_replaced_and_other_lines_kept() {
    use crate::facts::{self, Fact, Outcome};
    let facts = [Fact { name: "component-read-items", outcome: Outcome::Fails("TooLarge".to_owned()) }];
    for (before, expected) in [
        (
            "# forgejo pinned\n# runner none\nholds merge\nholds component-read-items\n",
            "# forgejo pinned\n# runner none\nholds merge\nfails component-read-items: TooLarge\n",
        ),
        (
            "holds component-start-ready\nholds merge\nunchecked component-read-job: none\n",
            "fails component-read-items: TooLarge\nholds merge\n",
        ),
        ("holds merge", "holds merge\nfails component-read-items: TooLarge\n"),
    ] {
        assert_eq!(facts::replace_block(before, "component-", &facts), expected, "{before}");
    }
}
#[test]
fn a_sent_request_is_counted_by_its_request_line() {
    for (name, bytes, expected) in [
        ("head", b"GET /api/v1/user HTTP/1.1\r\nAuthorization: token MASKED\r\n\r\n".as_slice(), 1),
        ("body", b"{\"body\":\"GET /api/v1/user\"}", 0),
        ("two heads", b"POST /api/v1/repos/a/b/issues HTTP/1.1\r\n\r\nGET /api/v1/user HTTP/1.1\r\n", 2),
    ] {
        assert_eq!(crate::component::hosted::sent_requests(bytes), expected, "{name}");
    }
}

#[test]
fn a_password_is_read_between_its_first_and_last_quote() {
    for (text, expected) in [
        ("generated random password is 'a'b'\nnext", Some("a'b")),
        ("no quotes", None),
        ("one'", None),
        ("''", Some("")),
    ] {
        assert_eq!(crate::cli::password_of(text), expected, "{text}");
    }
}
#[test]
fn a_raw_token_is_forty_lowercase_digits_and_a_line_feed() {
    for (text, expected) in [
        ("0123456789abcdef0123456789abcdef01234567\n", Some("0123456789abcdef0123456789abcdef01234567")),
        ("0123456789abcdef0123456789abcdef01234567", None),
        ("0123456789abcdef0123456789abcdef0123456\n", None),
        ("0123456789ABCDEF0123456789abcdef01234567\n", None),
        ("0123456789abcdef0123456789abcdef01234567\n\n", None),
    ] {
        assert_eq!(crate::cli::raw_token(text), expected, "{text:?}");
    }
}
