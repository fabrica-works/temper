use super::fixtures as f;
use crate::{
    forges,
    jig::{self, call::Parsed},
};
use jig_client::{self as client, contract as c};
use skein_lib::{Duration, List};
use temper_client_domain::forge::{
    self,
    forges::{Connection as C, Kind, Mode, Offer, Row, Step},
};
use temper_client_protocol_wire::{self as wire, v1 as w};
fn row(connection: C) -> Row {
    Row { number: 3, kind: Kind::Forgejo, mode: Mode::Owned, offer: Offer::Offered, connection }
}
fn record(connection: w::Connection) -> client::Family {
    let limits = forge::wire_limits(&f::limits_root().domain.forge).unwrap();
    let mut forges = List::with_capacity(1);
    forges
        .push(
            w::ForgeRow::new(
                &limits,
                w::ForgeRowParts {
                    number: 3,
                    kind: w::Kind::Forgejo,
                    mode: w::Mode::Owned,
                    offer: w::Offer::Offered,
                    connection,
                },
            )
            .unwrap(),
        )
        .unwrap();
    client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges }).unwrap()),
    }
}
fn deliver(root: &mut f::Harness, record: client::Family) {
    let subject = forge::forges::watch(&root.limits.domain.forge).unwrap();
    let token = root.watch_token(&subject);
    root.client(client::Event::Up { up: client::Up::FamilyDelivered { watch: token, record } });
}
#[test]
fn forge_calls_round_trip_through_argv_json_and_help() {
    for argv in [
        &["forges"][..],
        &["forge-setup", "3"],
        &["forge-setup", "3", "--key", "setup"],
        &["wait", "forges", "--until", "settled"],
        &["wait", "forges", "--since", "0123456789abcdef", "--until", "connected:3", "--timeout", "2s"],
    ] {
        let call = f::call(argv);
        assert_eq!(f::printed(&call), Parsed::Call { call: call.clone(), json: false });
        let json = jig::json::write_call(crate::table::TOOLS, &call).unwrap();
        assert_eq!(f::from_json(&json), call);
        let help = f::help(Some(call.tool));
        assert!(skein_lib::bytes::find(&help, argv[0].as_bytes()).is_some());
    }
    for argv in [
        &["forge-setup"][..],
        &["forge-setup", "oops"],
        &["forge-setup", "65536"],
        &["forge-setup", "0"],
        &["wait", "forges", "--until", "busy"],
        &["wait", "forges", "--until", "connected:x"],
        &["wait", "forges", "--until", "connected:65536"],
        &["wait", "forges", "--until", "connected:0"],
        &["wait", "forges", "--until", "connected:"],
        &["wait", "forges", "--until", "connected:18446744073709551616"],
    ] {
        assert!(f::parse(argv).is_err(), "{argv:?}");
    }
}
#[test]
fn every_state_renders_as_text_and_structured_json() {
    let cases: &[(C, &[u8], &[u8])] = &[
        (C::NotConnected, b"Not connected", br#""state":"not_connected""#),
        (C::NotSetUp, b"Not set up", br#""state":"not_set_up""#),
        (C::SettingUp { step: None }, b"Setting up", br#""state":"setting_up""#),
        (C::SettingUp { step: Some(Step::Fetch) }, b"Setting up: fetching", br#""state":"setting_up","step":"fetch""#),
        (
            C::SettingUp { step: Some(Step::Prepare) },
            b"Setting up: preparing",
            br#""state":"setting_up","step":"prepare""#,
        ),
        (C::SettingUp { step: Some(Step::Run) }, b"Setting up: starting", br#""state":"setting_up","step":"run""#),
        (
            C::SetUpFailed { why: b"fetch failed".as_slice().into() },
            b"Set up failed: fetch failed",
            br#""state":"set_up_failed","why":"fetch failed""#,
        ),
        (C::Connecting, b"Connecting", br#""state":"connecting""#),
        (
            C::Connected { id: 42, login: b"bot".as_slice().into() },
            b"Connected as bot (42)",
            br#""state":"connected","id":42,"login":"bot""#,
        ),
        (
            C::Stopped { reason: b"retired".as_slice().into() },
            b"Stopped: retired",
            br#""state":"stopped","reason":"retired""#,
        ),
    ];
    for (state, words, fields) in cases {
        let rows = [row(state.clone())];
        let plain = forges::print_rows(Some(&rows), false, None, &f::limits_root());
        let json = forges::print_rows(Some(&rows), true, None, &f::limits_root());
        assert_eq!(plain.status, 0);
        assert_eq!(json.status, 0);
        let document =
            concat(&[br#"{"forges":[{"number":3,"kind":"forgejo","mode":"owned","offer":"offered","#, fields, b"}]}"]);
        let digest = jig::digest::Digest::of(&document).hex();
        let expected = concat(&[
            b"forge 3 \xc2\xb7 Forgejo \xc2\xb7 owned \xc2\xb7 ",
            words,
            b"\ndigest: forges ",
            &digest,
            b"\n",
        ]);
        assert_eq!(&*plain.stdout, expected.as_ref(), "{state:?}");
        let expected = concat(&[
            document.get(..document.len().checked_sub(1).unwrap()).unwrap(),
            b",\"digest\":\"",
            &digest,
            b"\"}",
        ]);
        assert_eq!(&*json.stdout, expected.as_ref(), "{state:?}");
    }
    let mut without = row(C::NotSetUp);
    without.offer = Offer::NotUnderTheBox;
    let plain = forges::print_rows(Some(&[without.clone()]), false, None, &f::limits_root());
    assert!(skein_lib::bytes::find(&plain.stdout, b"temper forge fetch 3").is_some());
    let json = forges::print_rows(Some(&[without]), true, None, &f::limits_root());
    assert!(skein_lib::bytes::find(&json.stdout, b"not_under_the_box").is_some());
}
#[test]
fn settled_excludes_setting_up_and_connecting_and_connected_names_a_forge() {
    for state in [
        C::NotConnected,
        C::NotSetUp,
        C::SetUpFailed { why: Box::default() },
        C::Stopped { reason: Box::default() },
        C::Connected { id: 1, login: Box::default() },
    ] {
        assert!(forges::holds(&[row(state)], b"settled"));
    }
    for state in [C::SettingUp { step: None }, C::SettingUp { step: Some(Step::Run) }, C::Connecting] {
        assert!(!forges::holds(&[row(C::NotConnected), row(state)], b"settled"));
    }
    let rows = [row(C::Connected { id: 1, login: Box::default() })];
    assert!(forges::holds(&rows, b"connected:3"));
    assert!(!forges::holds(&rows, b"connected:2"));
    assert!(!forges::holds(&[row(C::NotConnected)], b"connected:3"));
    assert!(forges::holds(&[], b"settled"));
}
#[test]
fn the_read_closes_after_its_first_document_and_refuses_invalid_bytes() {
    for (bytes, status) in [
        (record(w::Connection::NotSetUp), 0),
        (client::Family { bytes: b"bad".as_slice().into(), ..record(w::Connection::NotSetUp) }, 1),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["forges"]);
        deliver(&mut root, bytes);
        assert_eq!(root.answer(call).unwrap().status, status);
        assert_eq!(root.closed.len(), 1);
        assert_eq!(root.watches.len(), 1, "only the runner's me remains");
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn a_wait_follows_each_setup_step_and_releases_the_watch_and_timer() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["wait", "forges", "--until", "settled"]);
    let limits = forge::wire_limits(&root.limits.domain.forge).unwrap();
    for step in [None, Some(w::SetUpStep::Prepare), Some(w::SetUpStep::Run)] {
        deliver(
            &mut root,
            record(w::Connection::SettingUp(w::SettingUp::new(&limits, w::SettingUpParts { step }).unwrap())),
        );
        assert!(root.answer(call).is_none());
    }
    deliver(&mut root, record(w::Connection::Connecting));
    assert!(root.answer(call).is_none());
    deliver(&mut root, record(w::Connection::NotConnected));
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 0);
    assert!(printed.stdout.ends_with(b"met\n"));
    assert_eq!(root.closed.len(), 1);
    assert!(crate::root::next_deadline(&root.root).is_none());
}
#[test]
fn since_uses_the_read_digest_and_plain_wait_requires_a_change() {
    for until in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let read = root.submit(&["forges", "--json"]);
        deliver(&mut root, record(w::Connection::NotSetUp));
        let printed = root.answer(read).unwrap();
        let digest = f::last_digest(&printed.stdout);
        let flags = if until { &["--until", "settled"][..] } else { &[] };
        let call = root.wait_since(&["wait", "forges"], flags, &digest);
        deliver(&mut root, record(w::Connection::NotSetUp));
        assert!(root.answer(call).is_none());
        deliver(&mut root, record(w::Connection::NotConnected));
        assert_eq!(root.answer(call).unwrap().status, 0);
    }
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["wait", "forges"]);
    deliver(&mut root, record(w::Connection::NotSetUp));
    assert!(root.answer(call).is_none());
    deliver(&mut root, record(w::Connection::NotSetUp));
    assert!(root.answer(call).is_none());
    deliver(&mut root, record(w::Connection::NotConnected));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
#[test]
fn timeout_prints_the_current_document_and_watch_refusals_keep_their_status() {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(&["wait", "forges", "--until", "connected:3", "--timeout", "1s", "--json"]);
    deliver(&mut root, record(w::Connection::NotSetUp));
    root.now = root.now.checked_add(Duration::from_secs(1)).unwrap();
    root.fire();
    let printed = root.answer(call).unwrap();
    assert_eq!(printed.status, 3);
    assert!(f::json_has(&printed, br#""state":"not_set_up""#));
    assert!(f::json_has(&printed, br#""met":false,"why":"timeout""#));
    assert_eq!(root.closed.len(), 1);
    for (end, status) in [(c::WatchEnd::NotAllowed, 2), (c::WatchEnd::Unknown, 2)] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["wait", "forges", "--until", "settled"]);
        let token = root.watch_token(&forge::forges::watch(&root.limits.domain.forge).unwrap());
        root.client(client::Event::Up { up: client::Up::WatchEnded { watch: token, end } });
        assert_eq!(root.answer(call).unwrap().status, status);
        assert!(crate::root::next_deadline(&root.root).is_none());
    }
}
#[test]
fn setup_answers_print_their_words_json_and_status() {
    let limits = forge::wire_limits(&f::limits_root().domain.forge).unwrap();
    for (outcome, expected, status) in [
        (w::Outcome::SettingUp, b"setting up\nnext: wait forges --until settled\n".as_slice(), 0),
        (w::Outcome::Unknown, b"unknown\n", 2),
        (w::Outcome::SetUpRefused(w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap()), b"refused: busy\n", 2),
        (w::Outcome::SetUpRefused(w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::NotOffered }).unwrap()), b"refused: not offered\n", 2),
        (w::Outcome::SetUpRefused(w::SetUpRefused::new(&limits, w::SetUpRefusedParts { why: w::SetUpRefusal::NotUnderTheBox }).unwrap()), b"refused: not under the box\nrun `temper forge fetch 3`, `temper forge prepare 3`, `temper forge run 3 forgejo` and `temper forge run 3 runner`, then restart `temper serve`\n", 2),
    ] {
        for json in [false, true] {
            let mut root = f::signed_in(f::PARTY);
            let call = root.submit_prepared(f::call(&["forge-setup", "3"]), json, None);
            let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("family") };
            assert_eq!(*scope, c::Scope::Deployment);
            let w::Request::SetUp(request) = wire::decode_requesting(&record.bytes, &limits).unwrap().into_parts().request else { panic!("set up"); };
            assert_eq!(request.forge(), 3);
            root.adoption_answer(call, outcome.clone());
            let printed = root.answer(call).unwrap();
            assert_eq!(printed.status, status);
            let bytes = if status == 0 { &printed.stdout } else { &printed.stderr };
            if json { assert!(bytes.starts_with(b"{")); } else { assert_eq!(bytes.as_ref(), expected); }
        }
    }
}

#[test]
fn setup_keeps_drawn_keys_and_recovers_the_answer_in_its_own_words() {
    for json in [false, true] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["forge-setup", "3"]);
        assert!(root.saved.contains_key(&call), "a drawn key is durable too");
        let key = root.sent_name(call).unwrap();
        let mut records = f::restored_adoption(f::PARTY);
        let record = records.get_mut(0).unwrap();
        record.key = key;
        record.request.request = root.adoption_bytes(call).clone();
        let mut root = f::started_with(records, f::SEED, json);
        root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
        root.adoption_answer(0, w::Outcome::SettingUp);
        let expected = if json {
            b"{\"operation\":\"forge-setup\",\"outcome\":{\"setting_up\":{},\"next\":\"wait forges --until settled\"}}\n".as_slice()
        } else {
            b"kept: forge-setup setting up\nnext: wait forges --until settled\n"
        };
        assert_eq!(root.kept_lines.as_slice(), &[Box::<[u8]>::from(expected)]);
    }
}
#[test]
fn setup_names_are_stable_and_unconfirmed_calls_wait_for_me() {
    let mut root = f::started(List::with_capacity(0));
    let call = root.submit(&["forge-setup", "3", "--key", "start-here"]);
    assert!(root.nothing_sent());
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    assert_eq!(root.sent_name(call), Some(crate::key::engine(&f::name(b"start-here"))));
    let mut other = f::signed_in(f::PARTY);
    let next = other.submit(&["forge-setup", "3", "--key", "start-here"]);
    assert_eq!(root.sent_name(call), other.sent_name(next));
    assert_eq!(root.adoption_bytes(call), other.adoption_bytes(next));
}
#[test]
fn a_set_up_rejects_an_adoption_answer_and_preserves_entrance_refusals() {
    let outcome = client::Outcome::Answered(f::family_answer(f::adopted_with_unknown_protection()));
    assert_eq!(forges::setup_print(3, &outcome, false, false, &f::limits_root()).status, 1);
    let outcome = client::Outcome::Answered(client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Role { held: None },
        at: None,
    }));
    for json in [false, true] {
        assert_eq!(forges::setup_print(3, &outcome, json, false, &f::limits_root()).status, 2);
    }
}

#[test]
fn all_children_share_the_call_budget_and_timeouts_release_it() {
    let mut root = f::started(List::with_capacity(0));
    // Calls wait for me, so no stream-table bound masks the root's call bound.
    let mut calls = List::with_capacity(root.limits.tools.rows);
    for at in 0..root.limits.tools.rows {
        let argv = if at % 2 == 0 {
            &["wait", "forges", "--timeout", "1s"][..]
        } else {
            &["wait", "accounts", "--timeout", "1s"]
        };
        calls.push(root.submit(argv)).unwrap();
    }
    let excess = root.submit(&["forge-setup", "3", "--json"]);
    let printed = root.answer(excess).unwrap();
    assert_eq!(printed.status, 1);
    assert_eq!(printed.stderr.as_ref(), br#"{"error":"call limit"}"#);
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    root.now = root.now.checked_add(Duration::from_secs(1)).unwrap();
    root.fire();
    for call in &calls {
        assert!(root.answer(*call).is_some());
    }
    let next = root.submit(&["forge-setup", "3"]);
    assert!(root.sends.contains_key(&next));
}

fn concat(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0usize;
    for part in parts {
        size = size.checked_add(part.len()).unwrap();
    }
    let mut out = skein_lib::Writer::new(size);
    for part in parts {
        out.put(part).unwrap();
    }
    out.finish()
}
