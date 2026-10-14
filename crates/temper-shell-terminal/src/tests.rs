use crate::{
    limits,
    route::{self, Down, Notice, Up},
};
use jig_domain_client::contract;
use jig_protocol_client_link as link;
use skein_lib::{List, Queue, Secret, Time};
use temper_client_domain as client;
mod family;
mod fixtures;
mod render;

#[test]
fn a_jig_read_reaches_the_link_with_its_own_token() {
    let down = route::down(
        client::Request::Jig(client::JigOut::Read { read: fixtures::READ, what: fixtures::inbox_read() }),
        &mut fixtures::family(),
    );
    let Down::Link(link::Event::Read { read, what: link::Readable::Jig(what) }) = down else {
        panic!("jig read reaches its link");
    };
    assert_eq!(read, fixtures::READ);
    assert_eq!(what, fixtures::inbox_read());
}
#[test]
fn a_jig_watch_event_reaches_the_root_unchanged() {
    for (name, event, expected) in [
        ("opened", contract::WatchEvent::Opened, contract::WatchEvent::Opened),
        ("alive", contract::WatchEvent::Alive, contract::WatchEvent::Alive),
    ] {
        let up = route::up(link::Request::Watch { watch: fixtures::WATCH, event }, &mut fixtures::family());
        let Up::Root(client::Event::Jig(jig_domain_client::Event::Watch { watch, event: found })) = up else {
            panic!("{name}: jig watch reaches the root");
        };
        assert_eq!(watch, fixtures::WATCH, "{name}: token");
        assert_eq!(found, expected, "{name}: event");
    }
}
#[test]
fn a_table_record_saved_then_kept_comes_back_the_same() {
    let limits = limits::link(&limits::native()).expect("usable native limits");
    for (name, record) in
        [("jig", client::store::Record::Jig(fixtures::jig_record())), ("table", fixtures::table_record())]
    {
        let mut link = link::Link::new(&limits).expect("bounded store codec");
        let mut out = Queue::with_capacity(link.max_out());
        link.step(
            Time::ZERO,
            link::Event::Save { record: route::saved(record.clone(), &fixtures::family()).expect("bounded fixture") },
            &mut out,
        );
        let Some(link::Request::Write { key, record: bytes }) = out.pop() else {
            panic!("{name}: save writes");
        };
        let mut kept = List::with_capacity(1);
        kept.push(link::StoreRead { key, record: bytes }).expect("one record fits");
        let mut versions = List::with_capacity(2);
        for version in route::versions() {
            versions.push(version).expect("two versions fit");
        }
        let mut restarted = link::Link::new(&limits).expect("bounded restore");
        restarted.step(Time::ZERO, link::Event::Start { kept, versions }, &mut out);
        let Some(link::Request::Kept { record: saved }) = out.pop() else {
            panic!("{name}: record restored");
        };
        assert_eq!(route::kept(saved, &fixtures::family()), Some(record), "{name}: unchanged record");
        assert!(out.is_empty(), "{name}: only one restored record");
    }
}
#[test]
fn exchanges_and_store_writes_go_to_jigs_shell() {
    let requests = [
        (
            "write",
            link::Request::Write { key: Box::from(*b"key"), record: Box::from(*b"record") },
            link::Request::Write { key: Box::from(*b"key"), record: Box::from(*b"record") },
        ),
        (
            "remove",
            link::Request::Remove { key: Box::from(*b"key") },
            link::Request::Remove { key: Box::from(*b"key") },
        ),
        (
            "exchange",
            link::Request::Exchange { exchange: fixtures::ATTEMPT, path: b"/read", body: Box::from(*b"body") },
            link::Request::Exchange { exchange: fixtures::ATTEMPT, path: b"/read", body: Box::from(*b"body") },
        ),
        (
            "abandon",
            link::Request::Abandon { exchange: fixtures::ATTEMPT },
            link::Request::Abandon { exchange: fixtures::ATTEMPT },
        ),
        ("open", link::Request::OpenStream, link::Request::OpenStream),
        ("close", link::Request::CloseStream, link::Request::CloseStream),
    ];
    for (name, request, expected) in requests {
        let Up::Shell(found) = route::up(request, &mut fixtures::family()) else {
            panic!("{name}: shell request");
        };
        assert_eq!(found, expected, "{name}: unchanged transport");
    }
}
#[test]
fn a_service_credential_is_a_notice_never_a_root_event() {
    let credential = Secret::new(Box::from(fixtures::CREDENTIAL));
    let up =
        route::up(link::Request::ServiceCredential { party: fixtures::SERVICE, credential }, &mut fixtures::family());
    let Up::Notice(Notice::Credential { party, credential }) = up else {
        panic!("a service's credential goes to the shell");
    };
    assert_eq!(party, fixtures::SERVICE);
    assert_eq!(credential.expose(), fixtures::CREDENTIAL);
}
#[test]
fn a_family_readout_or_delivery_nobody_asked_for_is_a_notice() {
    for (name, request) in [
        (
            "read",
            link::Request::Read {
                read: fixtures::READ,
                result: link::Readout::Family { family: 1, record: Some(Box::from(*b"record")) },
            },
        ),
        (
            "delivery",
            link::Request::FamilyDelivery {
                watch: fixtures::WATCH,
                record: link::Family { family: 1, version: 1, bytes: Box::from(*b"record") },
            },
        ),
    ] {
        assert!(
            matches!(route::up(request, &mut fixtures::family()), Up::Notice(Notice::Unasked)),
            "{name}: unasked notice"
        );
    }
}
#[test]
fn the_native_limits_pass_each_worst_case() {
    let limits = limits::native();
    let (view, jig, forge) = limits::view(&limits).expect("usable view");
    let root = client::worst_case(&limits.client).expect("usable root");
    let view = temper_client_view::worst_case(&view, &limits.client, &jig, &forge).expect("usable view");
    let link = link::worst_case(&limits::link(&limits).expect("usable link")).expect("usable link memory");
    let shell = jig_shell_native::Shell::new(jig_shell_native::Settings {
        engine: skein_io::Addr::Ip("127.0.0.1:7100".parse().expect("loopback")),
        directory: Box::from(*b"client"),
        limits: limits.connections,
        io: limits.io,
    });
    assert_eq!(limits::shell(&limits), Some(shell.worst_case()), "checked shell formula");
    assert_eq!(
        limits::worst_case(&limits),
        Some(
            root + view
                + link
                + shell.worst_case()
                + skein_lib::Set::<skein_lib::Token>::worst_case(limits.client.table.requests).expect("attempts fit")
        ),
        "sum of all owners"
    );
    assert!(skein_io::worst_case(&limits.io).is_some(), "usable io");
}
