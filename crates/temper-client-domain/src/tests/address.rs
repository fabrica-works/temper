use crate::address::{Address, Filter, Kind, Place, parse, print};

const INBOX: Place = Place::Address(Address::Inbox { filter: Filter::ALL });

#[test]
fn each_page_round_trips_through_its_path() {
    let pages: [(Address, &[u8]); 5] = [
        (Address::Inbox { filter: Filter::ALL }, b"/"),
        (Address::Chats, b"/chats"),
        (Address::Chat { task: 7 }, b"/chats/7"),
        (Address::Task { task: 7 }, b"/tasks/7"),
        (Address::Adopt { project: 3 }, b"/projects/3/adopt"),
    ];
    for (address, path) in pages {
        assert_eq!(&*print(&address), path, "{address:?}");
        assert_eq!(parse(path), Place::Address(address), "{path:?}");
    }
}

#[test]
fn the_inbox_prints_slash_whatever_its_filter() {
    let filters = [
        ("project", Filter { project: Some(3), ..Filter::ALL }),
        ("kind", Filter { kind: Some(Kind::Question), ..Filter::ALL }),
        ("goal", Filter { goal: Some(9), ..Filter::ALL }),
    ];
    for (name, filter) in filters {
        assert_eq!(&*print(&Address::Inbox { filter }), b"/", "{name}");
    }
}

#[test]
fn a_query_is_ignored() {
    let chat = Place::Address(Address::Chat { task: 7 });
    for (path, expected) in [(&b"/chats/7?x=1"[..], chat), (b"/?secret=s", INBOX)] {
        assert_eq!(parse(path), expected, "{path:?}");
    }
}

#[test]
fn one_trailing_slash_is_ignored() {
    for (path, expected) in [
        (&b"/chats/7/"[..], Place::Address(Address::Chat { task: 7 })),
        (b"/chats/", Place::Address(Address::Chats)),
        (b"/chats/7//", Place::Missing),
    ] {
        assert_eq!(parse(path), expected, "{path:?}");
    }
}

#[test]
fn a_path_that_is_no_page_is_missing() {
    let paths: [&[u8]; 12] = [
        b"/jig/v1/sign-in/link",
        b"/bundle/x",
        b"/test/panic",
        b"/robots.txt",
        b"chats",
        b"/chats/x",
        b"/tasks",
        b"/projects/1",
        b"/projects/1/board",
        b"/projects/3/adopt/x",
        b"//",
        b"/chats/%37",
    ];
    for path in paths {
        assert_eq!(parse(path), Place::Missing, "{path:?}");
    }
}

#[test]
fn a_number_not_in_canonical_decimal_is_missing() {
    let paths: [&[u8]; 5] = [b"/chats/07", b"/chats/+7", b"/chats/-7", b"/chats/7a", b"/chats/123456789012345678901"];
    for path in paths {
        assert_eq!(parse(path), Place::Missing, "{path:?}");
    }
}

#[test]
fn each_number_spans_its_types_range() {
    let task = Place::Address(Address::Chat { task: u64::MAX });
    let project = Place::Address(Address::Adopt { project: u32::MAX });
    assert_eq!(parse(b"/chats/18446744073709551615"), task);
    assert_eq!(parse(b"/projects/4294967295/adopt"), project);
    assert_eq!(&*print(&Address::Chat { task: u64::MAX }), b"/chats/18446744073709551615");
    assert_eq!(&*print(&Address::Adopt { project: u32::MAX }), b"/projects/4294967295/adopt");
    assert_eq!(parse(b"/chats/18446744073709551616"), Place::Missing);
    assert_eq!(parse(b"/projects/4294967296/adopt"), Place::Missing);
}
