use crate::address::{Address, Filter, Kind, Place, parse, print};

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
fn each_inbox_filter_round_trips_through_its_query() {
    let filters: [(Filter, &[u8]); 9] = [
        (Filter { project: Some(3), ..Filter::ALL }, b"/?project=3"),
        (Filter { kind: Some(Kind::Proposal), ..Filter::ALL }, b"/?kind=proposal"),
        (Filter { kind: Some(Kind::Question), ..Filter::ALL }, b"/?kind=question"),
        (Filter { kind: Some(Kind::Held), ..Filter::ALL }, b"/?kind=held"),
        (Filter { kind: Some(Kind::PersonTask), ..Filter::ALL }, b"/?kind=person-task"),
        (Filter { goal: Some(9), ..Filter::ALL }, b"/?goal=9"),
        (
            Filter { project: Some(3), kind: Some(Kind::PersonTask), goal: Some(9) },
            b"/?project=3&kind=person-task&goal=9",
        ),
        (Filter { project: Some(u32::MAX), ..Filter::ALL }, b"/?project=4294967295"),
        (Filter { goal: Some(u64::MAX), ..Filter::ALL }, b"/?goal=18446744073709551615"),
    ];
    for (filter, path) in filters {
        let address = Address::Inbox { filter };
        assert_eq!(&*print(&address), path, "{filter:?}");
        assert_eq!(parse(path), Place::Address(address), "{path:?}");
    }
}

#[test]
fn the_inbox_prints_its_set_fields_in_one_order() {
    let filters: [(Filter, &[u8]); 2] =
        [(Filter::ALL, b"/"), (Filter { project: Some(3), goal: Some(9), ..Filter::ALL }, b"/?project=3&goal=9")];
    for (filter, path) in filters {
        assert_eq!(&*print(&Address::Inbox { filter }), path, "{filter:?}");
    }
}

#[test]
fn another_pages_query_is_dropped() {
    assert_eq!(parse(b"/chats/7?x=1"), Place::Address(Address::Chat { task: 7 }));
}

#[test]
fn a_field_it_cannot_read_is_dropped() {
    let project = Filter { project: Some(3), ..Filter::ALL };
    let fields: &[(&[u8], Filter)] = &[
        (b"/?secret=s", Filter::ALL),
        (b"/?kind=reply&project=3", project),
        (b"/?kind=Question", Filter::ALL),
        (b"/?project=07", Filter::ALL),
        (b"/?project=4294967296", Filter::ALL),
        (b"/?goal=", Filter::ALL),
        (b"/?kind", Filter::ALL),
        (b"/?=3", Filter::ALL),
        (b"/?", Filter::ALL),
        (b"/?&&", Filter::ALL),
        (b"/?project=3&project=4", project),
        (b"/?project=x&project=4", Filter { project: Some(4), ..Filter::ALL }),
        (b"/?goal=9&project=3", Filter { goal: Some(9), ..project }),
        (b"/?goal=18446744073709551616", Filter::ALL),
        (b"/?kind=question=held", Filter::ALL),
        (b"/?kind=held&kind=question", Filter { kind: Some(Kind::Held), ..Filter::ALL }),
        (b"/?goal=x&goal=9&goal=10", Filter { goal: Some(9), ..Filter::ALL }),
        (b"/?kind=reply&kind=question", Filter { kind: Some(Kind::Question), ..Filter::ALL }),
    ];
    for &(path, filter) in fields {
        assert_eq!(parse(path), Place::Address(Address::Inbox { filter }), "{path:?}");
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
