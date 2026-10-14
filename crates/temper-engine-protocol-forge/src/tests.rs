mod connection;
mod contents;
mod fixtures;
mod reads;
mod writes;
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

use crate::{
    Answered, Event, Limits, Request, Unusable,
    limits::{self, Listing},
    tables::Tables,
};
use skein_lib::{Secret, Token};
use temper_engine_domain_forge_client::api;

#[test]
fn limits_that_disagree_with_the_client_name_both_limits() {
    let l = fixtures::limits();
    let s = fixtures::shared();
    let short_answer =
        crate::Shared { client: temper_engine_domain_forge_client::Limits { answer_bytes: 64, ..s.client }, ..s };
    let rows = [
        ("connections", Limits { connections: 3, ..l }, s, Unusable::Connections { connections: 3, calls: 2 }),
        (
            "deadline",
            Limits { deadline: Duration::from_secs(11), ..l },
            s,
            Unusable::Deadline { deadline: Duration::from_secs(11), lifetime: Duration::from_secs(10) },
        ),
        ("body", Limits { body_bytes: 1044, ..l }, s, Unusable::BodyBytes { body_bytes: 1044, needed: 1045 }),
        ("name", Limits { string_bytes: 127, ..l }, s, Unusable::StringBytes { string_bytes: 127, name_bytes: 128 }),
        ("repositories", Limits { repositories: 3, ..l }, s, Unusable::Repositories { repositories: 3, forge: 2 }),
        ("answer", l, short_answer, Unusable::PageSize { listing: Listing::Items }),
    ];
    for (name, limits, shared, expected) in rows {
        assert_eq!(limits::check(&limits, &shared, &fixtures::settings()), Err(expected), "{name}");
    }
    assert_eq!(limits::check(&l, &s, &fixtures::settings()), Ok(()), "fixture agrees");
}

#[test]
fn a_page_is_the_smallest_of_rows_cap_and_answer_room() {
    let limits = fixtures::limits();
    let shared = fixtures::shared();
    let rows = [("forge cap", 2, 65_536, 2), ("client rows", 50, 65_536, 8), ("answer room", 50, 4096, 1)];
    for (name, cap, answer_bytes, expected) in rows {
        let shared = crate::Shared {
            client: temper_engine_domain_forge_client::Limits { answer_bytes, ..shared.client },
            ..shared
        };
        assert_eq!(limits::page_size(Listing::Items, cap, &limits, &shared), expected, "{name}");
    }
}

#[test]
fn a_call_waiting_for_its_forge_is_unavailable_at_its_deadline() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    assert_eq!(h.start(fixtures::FORGE), Start::Checking { next: Duration::from_secs(1) }, "start checking");
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(10_000_000_000)), "arrival plus deadline");
    h.run(Duration::from_secs(9));
    assert_eq!(h.answered(1), None, "before deadline");
    h.run(Duration::from_secs(1));
    assert_eq!(
        h.answered(1),
        Some(&Answered { call: Token::new(1), cost: 0, result: Err(api::Error::Unavailable) }),
        "at deadline"
    );
    h.run(Duration::from_secs(1));
    assert_eq!(h.answers.len(), 1, "answered once");
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(21_000_000_000)), "only the new start waits");
    assert_eq!(h.io.len(), 0, "nothing sent");
}

#[test]
fn a_full_table_drops_the_entry_used_longest_ago() {
    let mut tables = Tables::new(2, 2);
    let rows = [
        ("repositories stay separate by forge", api::Repository { forge: 2, repository: 1 }),
        ("repositories stay separate by id", api::Repository { forge: 1, repository: 2 }),
    ];
    for (name, second) in rows {
        tables.keep_repository(fixtures::REPO, fixtures::specimen());
        tables.keep_repository(second, fixtures::specimen());
        assert_eq!(tables.repository(fixtures::REPO), Some(&fixtures::specimen()), "{name}: use first");
        tables.keep_repository(api::Repository { forge: 3, repository: 3 }, fixtures::specimen());
        assert_eq!(tables.repository(second), None, "{name}: oldest evicted");
        assert_eq!(tables.repository(fixtures::REPO), Some(&fixtures::specimen()), "{name}: recent kept");
    }
}

#[test]
fn a_full_user_table_keeps_the_most_recent_login_and_replaces_in_place() {
    let mut tables = Tables::new(2, 2);
    tables.keep_user(1, 1, Box::from(&b"first"[..]));
    tables.keep_user(2, 1, Box::from(&b"other-forge"[..]));
    assert_eq!(tables.user(1, 1), Some(&b"first"[..]), "first touched");
    tables.keep_user(1, 2, Box::from(&b"third"[..]));
    assert_eq!(tables.user(2, 1), None, "oldest evicted");
    tables.keep_user(1, 1, Box::from(&b"renamed"[..]));
    assert_eq!(tables.user(1, 1), Some(&b"renamed"[..]), "existing login refreshed");
    assert_eq!(tables.user(1, 2), Some(&b"third"[..]), "replacement evicts nothing");
}

#[test]
fn refused_answers_are_offered_once_and_returned_answers_stay_until_close() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    let answer = Answered { call: Token::new(1), cost: 0, result: Err(api::Error::Unavailable) };
    h.send(Request::Refused(answer.clone()));
    h.send(Request::Offer);
    let Some(Event::Answered(offered)) = h.up.pop() else {
        unreachable!("one offered answer");
    };
    assert_eq!(offered, answer, "same answer offered");
    h.send(Request::Returned(offered));
    h.send(Request::Offer);
    assert_eq!(h.up.len(), 0, "returned answer never offered");
    h.send(Request::Close);
    assert_eq!(h.up.len(), 0, "connections close before the terminal");
    h.run(Duration::ZERO);
    assert!(h.closed, "closed emitted after io closed");
    h.send(Request::Offer);
    h.send(Request::Close);
    assert_eq!(h.up.len(), 0, "nothing after closed");
}

#[test]
fn closing_waiting_calls_emits_only_the_close_terminal() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.send(Request::Close);
    assert_eq!(h.up.len(), 0, "close waits for io");
    h.run(Duration::ZERO);
    assert!(h.closed, "close terminal after last socket");
    h.run(Duration::from_secs(20));
    assert_eq!(h.answered(1), None, "no call answer after root stopped");
    assert_eq!(h.component.next_deadline(), None, "no live deadlines");
}

#[test]
fn unusable_forge_settings_name_the_forge() {
    let loopback = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    let remote = SocketAddr::from((Ipv4Addr::new(192, 0, 2, 1), 3000));
    let rows = [
        ("empty token", skein_io::Addr::Ip(loopback), b"".as_slice(), 2, Unusable::Token { forge: 1 }),
        ("zero user", skein_io::Addr::Ip(loopback), fixtures::TOKEN, 0, Unusable::User { forge: 1 }),
        ("remote plaintext", skein_io::Addr::Ip(remote), fixtures::TOKEN, 2, Unusable::Address { forge: 1 }),
        (
            "in process",
            skein_io::Addr::InProcess(skein_io::Name::EMPTY),
            fixtures::TOKEN,
            2,
            Unusable::Address { forge: 1 },
        ),
    ];
    for (name, address, token, user, expected) in rows {
        let mut settings = fixtures::settings();
        let forge = settings.forges.get_mut(0).expect("one forge");
        forge.address = address;
        forge.token = Secret::new(Box::from(token));
        forge.user = user;
        assert_eq!(limits::check(&fixtures::limits(), &fixtures::shared(), &settings), Err(expected), "{name}");
    }
}

#[test]
fn zero_documents_and_stacked_demands_are_refused_in_order() {
    let l = fixtures::limits();
    let s = fixtures::shared();
    let rows = [
        ("zero chunk", Limits { chunk: 0, ..l }, s, Unusable::Zero),
        (
            "document depth",
            Limits { depth: 7, ..l },
            s,
            Unusable::Documents(temper_forge_forgejo::limits::Problem::Depth),
        ),
        (
            "HTTP over TLS",
            Limits { head_bytes: 16_385, ..l },
            s,
            Unusable::Stack { stacked: limits::Stacked::HttpOverTls, demand: 16_385, cap: 16_384 },
        ),
        (
            "TLS over io",
            l,
            crate::Shared { io: skein_io::Limits { intake: 18_431, ..s.io }, ..s },
            Unusable::Stack { stacked: limits::Stacked::TlsOverIo, demand: 18_432, cap: 18_431 },
        ),
    ];
    for (name, limits, shared, expected) in rows {
        assert_eq!(limits::check(&limits, &shared, &fixtures::settings()), Err(expected), "{name}");
    }
}

#[test]
fn duplicate_forge_numbers_are_refused_before_any_connection() {
    let mut settings = fixtures::settings();
    let extra = fixtures::settings().forges.into_boxed();
    for forge in extra {
        settings.forges.push(forge).expect("two fixture forges fit");
    }
    assert_eq!(
        limits::check(&fixtures::limits(), &fixtures::shared(), &settings),
        Err(Unusable::Duplicate { forge: 1 }),
        "duplicate forge"
    );
}

#[test]
fn the_worst_case_counts_answers_and_refuses_overflow() {
    let limits = fixtures::limits();
    let shared = fixtures::shared();
    let bytes = limits::worst_case(&limits, &shared, 1).expect("fixture bound fits");
    assert!(bytes >= 262_144, "two building and two held answers alone");
    let huge = Limits { string_bytes: u32::MAX, inner: u32::MAX, connections: u32::MAX, ..limits };
    assert_eq!(limits::worst_case(&huge, &shared, 1), None, "multipliers overflow checked");
}

#[test]
fn many_due_calls_keep_the_timer_output_bound() {
    let limits = Limits { connections: 20, ..fixtures::limits() };
    let shared = fixtures::shared();
    let shared =
        crate::Shared { client: temper_engine_domain_forge_client::Limits { calls: 20, ..shared.client }, ..shared };
    let mut h = fixtures::Harness::with(limits, shared, fixtures::settings());
    for call in 0..20 {
        h.call(call, fixtures::REPO, api::Op::Read(api::Read::Settings));
    }
    for (name, span, count, deadline) in [
        ("first batch", Duration::from_secs(10), 16, Some(Time::from_nanos(10_000_000_000))),
        ("rest", Duration::ZERO, 20, Some(Time::from_nanos(11_000_000_000))),
    ] {
        h.run(span);
        assert_eq!(h.answers.len(), count, "{name}: calls ended");
        assert_eq!(h.component.next_deadline(), deadline, "{name}: earliest remaining");
    }
}

#[test]
fn many_held_answers_keep_the_offer_output_bound() {
    let limits = Limits { connections: 20, ..fixtures::limits() };
    let shared = fixtures::shared();
    let shared =
        crate::Shared { client: temper_engine_domain_forge_client::Limits { calls: 20, ..shared.client }, ..shared };
    let mut h = fixtures::Harness::with(limits, shared, fixtures::settings());
    for call in 0..20 {
        h.send(Request::Refused(Answered { call: Token::new(call), cost: 0, result: Err(api::Error::Unavailable) }));
    }
    for (name, count) in [("first batch", 16), ("second batch", 4), ("empty", 0)] {
        h.send(Request::Offer);
        assert_eq!(h.up.len(), count, "{name}: offer batch");
        for _ in 0..count {
            drop(h.up.pop());
        }
    }
}
