mod fixtures;
use crate::{
    StartRead, Stopped, conventions, marker,
    start::{self, Heard, Start},
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use skein_lib::{Duration, Time};

#[test]
fn a_marker_round_trips_its_key_and_keeps_the_text() {
    let sixty_four = [0xab; 64];
    let rows: [(&str, &[u8], &[u8]); 4] = [
        ("empty key", b"", b"text"),
        ("one byte", &[0x01], b"text"),
        ("64 bytes", &sixty_four, b"text"),
        ("text holds a marker", &[0xff], b"<!-- temper:key 01 -->\ntext"),
    ];
    for (name, key, text) in rows {
        let encoded = marker::encode(key, text, 1024).expect("fixture fits");
        assert_eq!(marker::decode(&encoded), Some(Box::from(key)), "{name}");
        assert!(encoded.ends_with(text), "{name}: text kept whole");
        let size = u32::try_from(encoded.len()).expect("fixture length");
        assert_eq!(marker::encode(key, text, size), Some(encoded), "{name}: exact limit");
        assert_eq!(marker::encode(key, text, size.saturating_sub(1)), None, "{name}: one short");
    }
    assert_eq!(marker::encode(&[0xab, 0x01], b"body", 29), Some(Box::from(&b"<!-- temper:key ab01 -->\nbody"[..])));
    assert_eq!(marker::length(u32::MAX), None);
}

#[test]
fn a_body_without_a_well_formed_marker_has_no_key() {
    let rows: [(&str, &[u8]); 5] = [
        ("uppercase", b"<!-- temper:key AB -->\n"),
        ("odd", b"<!-- temper:key a -->\n"),
        ("no line feed", b"<!-- temper:key ab -->"),
        ("leading space", b" <!-- temper:key ab -->\n"),
        ("after text", b"text<!-- temper:key ab -->\n"),
    ];
    for (name, body) in rows {
        assert_eq!(marker::decode(body), None, "{name}");
    }
}

#[test]
fn the_start_stops_on_another_user_or_a_4xx_and_waits_otherwise() {
    let limits = fixtures::limits();
    let now = Time::from_nanos(1_000_000_000);
    let checking = Start::Checking { next: limits.start_backoff };
    let settings = Start::Settings { next: limits.start_backoff };
    let again = Start::Waiting { until: Time::from_nanos(2_000_000_000), next: Duration::from_secs(2) };
    let rows = [
        ("configured user", checking, Heard::User { id: fixtures::WRITER }, settings),
        ("another user", checking, Heard::User { id: 1 }, Start::Stopped(Stopped::OtherUser { user: 1 })),
        (
            "refused token",
            checking,
            Heard::Status { status: 401, retry: None },
            Start::Stopped(Stopped::Refused { read: StartRead::User, status: 401 }),
        ),
        (
            "no API",
            settings,
            Heard::Status { status: 404, retry: None },
            Start::Stopped(Stopped::Refused { read: StartRead::ApiSettings, status: 404 }),
        ),
        ("page cap", settings, Heard::ApiSettings { max_response_items: 2 }, Start::Ready { cap: 2 }),
        ("5xx", checking, Heard::Status { status: 503, retry: None }, again),
        (
            "429",
            checking,
            Heard::Status { status: 429, retry: Some(Duration::from_secs(5)) },
            Start::Waiting { until: Time::from_nanos(6_000_000_000), next: Duration::from_secs(2) },
        ),
        ("unreachable", checking, Heard::Unanswered, again),
        ("empty cap", settings, Heard::ApiSettings { max_response_items: 0 }, again),
        ("wrong document", settings, Heard::User { id: fixtures::WRITER }, again),
        ("redirect", checking, Heard::Status { status: 301, retry: None }, again),
        ("ready stays ready", Start::Ready { cap: 2 }, Heard::Unanswered, Start::Ready { cap: 2 }),
    ];
    for (name, state, heard, expected) in rows {
        assert_eq!(start::heard(state, heard, fixtures::WRITER, now, &limits), expected, "{name}");
    }
}

#[test]
fn host_names_include_only_nondefault_ports_and_bracket_ipv6() {
    type HostRow = (&'static str, SocketAddr, Option<&'static [u8]>, &'static [u8]);
    let rows: [HostRow; 5] = [
        ("plain default", SocketAddr::from((Ipv4Addr::LOCALHOST, 80)), None, b"127.0.0.1"),
        ("plain custom", SocketAddr::from((Ipv4Addr::LOCALHOST, 3000)), None, b"127.0.0.1:3000"),
        ("TLS default", SocketAddr::from((Ipv4Addr::LOCALHOST, 443)), Some(b"forge.example"), b"forge.example"),
        ("TLS custom", SocketAddr::from((Ipv4Addr::LOCALHOST, 3000)), Some(b"forge.example"), b"forge.example:3000"),
        ("IPv6", SocketAddr::from((Ipv6Addr::LOCALHOST, 80)), None, b"[0:0:0:0:0:0:0:1]"),
    ];
    for (name, address, server, expected) in rows {
        assert_eq!(conventions::host(address, server), Some(Box::from(expected)), "{name}");
    }
}
