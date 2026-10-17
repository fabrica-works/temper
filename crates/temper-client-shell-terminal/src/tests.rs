use crate::{
    limits,
    route::{self, Down, Notice, Up},
};
use jig_client::contract;
use jig_client_protocol_link as link;
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
    let Down::Link(link::Event::Read { read, what: jig_client::Readable::Jig(what) }) = down else {
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
        let Up::Root(client::Event::Jig(jig_client_view::Event::Watch { watch, event: found })) = up else {
            panic!("{name}: jig watch reaches the root");
        };
        assert_eq!(watch, fixtures::WATCH, "{name}: token");
        assert_eq!(found, expected, "{name}: event");
    }
}
#[test]
fn a_table_record_saved_then_kept_comes_back_the_same() {
    let limits = limits::link(&limits::native()).expect("usable native limits");
    {
        let (name, record) = ("table", fixtures::table_record());
        let mut link = link::Link::new(&limits).expect("bounded store codec");
        let mut out = Queue::with_capacity(link.max_out());
        link.step(
            Time::ZERO,
            link::Event::Save { record: route::saved(record.clone(), &fixtures::family()).expect("bounded fixture") },
            &mut out,
        );
        let Some(link::Request::Save { record: link::Record { key, bytes } }) = out.pop() else {
            panic!("{name}: save writes");
        };
        let mut kept = List::with_capacity(1);
        kept.push(link::Record { key, bytes }).expect("one record fits");
        let mut restarted = link::Link::new(&limits).expect("bounded restore");
        restarted.step(Time::ZERO, link::Event::Start { kept }, &mut out);
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
            link::Request::Save { record: link::Record { key: Box::from(*b"key"), bytes: Box::from(*b"record") } },
            link::Request::Save { record: link::Record { key: Box::from(*b"key"), bytes: Box::from(*b"record") } },
        ),
        ("remove", link::Request::Erase { key: Box::from(*b"key") }, link::Request::Erase { key: Box::from(*b"key") }),
        (
            "exchange",
            link::Request::Exchange {
                exchange: fixtures::ATTEMPT,
                credential: Some(fixtures::credential()),
                message: link::Message::Read(Box::from(*b"body")),
            },
            link::Request::Exchange {
                exchange: fixtures::ATTEMPT,
                credential: Some(fixtures::credential()),
                message: link::Message::Read(Box::from(*b"body")),
            },
        ),
        (
            "open",
            link::Request::OpenStream { credential: Some(fixtures::credential()) },
            link::Request::OpenStream { credential: Some(fixtures::credential()) },
        ),
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
                result: jig_client::Readout::Family { family: 1, record: Some(Box::from(*b"record")) },
            },
        ),
        (
            "delivery",
            link::Request::FamilyDelivery {
                watch: fixtures::WATCH,
                record: jig_client::Family { family: 1, version: 1, bytes: Box::from(*b"record") },
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
    let shell = jig_client_shell_native::Shell::new(jig_client_shell_native::Settings {
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

#[test]
fn an_equal_line_waits_once_in_the_order_it_came() {
    let mut lines = crate::terminal::waiting::Waiting::new(19);
    for line in [b"sign-in again".as_slice(), b"store failed", b"sign-in again", b"credential one", b"credential two"] {
        lines.push(Box::from(line));
    }
    let mut said = Vec::<Box<[u8]>>::new();
    lines.say(1, |line| said.push(Box::from(line)));
    assert_eq!(said, [Box::<[u8]>::from(b"sign-in again".as_slice())]);
    said.clear();
    lines.say(8, |line| said.push(Box::from(line)));
    assert_eq!(said, [b"store failed".as_slice(), b"credential one", b"credential two"].map(Box::<[u8]>::from));
    assert!(lines.is_empty());
}

#[test]
fn the_native_limits_make_the_terminal() {
    let (view, _, _) = limits::view(&limits::native()).expect("native view limits");
    let bounds = limits::terminal();
    let settings = skein_ui_text::Settings {
        name: Box::from(*b"temper"),
        longest: bounds.line,
        interval: bounds.interval,
        status: Some(jig_client_view::regions::JigRegion::Link.region()),
        lines: bounds.lines,
        line_bytes: 166,
    };
    assert!(skein_ui_text::Terminal::worst_case(&settings, &view.tree).is_some(), "native terminal memory fits");
    assert!(skein_ui_text::Terminal::operations(&settings, &view.tree).is_some(), "native terminal operations fit");
    assert!(skein_ui_text::Terminal::new(settings, &view.tree).is_some(), "native terminal settings fit");
}

#[test]
fn a_signed_in_signing_keeps_the_credential_before_any_notice() {
    let ups = route::signing(&link::Signing::SignedIn { party: fixtures::PARTY }, Some(&fixtures::credential()));
    let [Up::Keep(credential), Up::Notice(Notice::SignedIn)] = ups.as_slice() else {
        panic!("credential precedes the signed-in notice");
    };
    assert_eq!(*credential, Some(fixtures::credential()));
}
#[test]
fn a_signed_out_signing_drops_the_kept_credential() {
    let ups = route::signing(&link::Signing::SignedOut, None);
    let [Up::Keep(credential), Up::Notice(Notice::SignedOut)] = ups.as_slice() else {
        panic!("credential removal precedes the signed-out notice");
    };
    assert_eq!(*credential, None);
}
#[test]
fn a_kept_credential_signs_in_held_with_no_exchange() {
    let mut link = link::Link::new(&limits::link(&limits::native()).expect("native bounds")).expect("bounded link");
    let mut out = Queue::with_capacity(link.max_out());
    link.step(Time::ZERO, link::Event::SignIn { with: link::With::Held(fixtures::credential()) }, &mut out);
    assert_eq!(link.credential(), Some(&fixtures::credential()));
    assert!(out.is_empty(), "a held credential sends no sign-in exchange");
}
