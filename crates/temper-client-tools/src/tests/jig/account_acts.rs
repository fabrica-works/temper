use crate::{jig::call::Parsed, tests::fixtures as f};
use jig_client::contract as c;
use skein_lib::{List, Secret};
const KEY: &[u8] = b"one-use-private-key";
const ADD: &[&str] = &["add-key", "9", "key-file", "--deployment", "--key", "add"];
const FORMS: &[&[&str]] = &[
    &["connect", "9", "--key", "connect"],
    &["connect", "0", "--key", "zero-provider"],
    &["add-key", "0", "file", "--key", "zero-provider"],
    &["disconnect", "43", "--source", "sign_in"],
    ADD,
    &["add-key", "9", "file", "--account", "43", "--key", "join"],
    &["replace-key", "43", "file"],
    &["forget", "0"],
    &["rank", "0", "1"],
    &["add-key", "9", "file", "--account", "0", "--key", "k"],
    &["forget", "43"],
    &["forget", "43", "--source", "key"],
    &["rank", "43", "2", "--source", "program"],
    &["retry", "43"],
];
#[test]
fn account_forms_round_trip_through_the_table() {
    for words in FORMS {
        let call = f::call(words);
        assert_eq!(f::back(&call), f::owned(words));
        assert_eq!(f::printed(&call), Parsed::Call { call, json: false });
    }
    for words in [
        &["connect", "9"][..],
        &["add-key", "9", "file"],
        &["add-key", "9", "-", "--key", "add"],
        &["rank", "43", "0"],
        &["retry", "4294967296"],
    ] {
        assert!(f::parse(words).is_err(), "{words:?}");
    }
}
#[test]
fn account_acts_preserve_the_resolved_source() {
    for (words, change) in [
        (&["forget", "43"][..], c::AccountChange::Remove { account: f::ACCOUNT }),
        (&["forget", "43", "--source", "key"], c::AccountChange::RemoveKey { source: f::SOURCE }),
        (&["rank", "43", "2"], c::AccountChange::MoveAccount { account: f::ACCOUNT, place: 2 }),
        (
            &["rank", "43", "2", "--source", "key"],
            c::AccountChange::MoveSource { account: f::ACCOUNT, source: f::SOURCE, place: 2 },
        ),
        (&["retry", "43"], c::AccountChange::Resume { source: f::SOURCE }),
    ] {
        for owner in [false, true] {
            let mut root = f::account_root(owner);
            let call = root.submit(words);
            let mut accounts = f::accounts();
            if owner {
                accounts.accounts = List::with_capacity(0);
            }
            f::deliver_accounts(&mut root, accounts);
            if owner {
                f::deliver_deployment(&mut root, f::deployment());
            }
            let (_, _, ask) = root.sends.get(&call).expect("resolved act");
            let holder = if owner { c::AccountHolder::Deployment } else { c::AccountHolder::Own };
            assert_eq!(ask, &jig_client::Ask::Jig(Box::new(c::Ask::AccountChange { holder, change: change.clone() })));
        }
    }
}
#[test]
fn ambiguous_sources_are_refused_before_sending() {
    for cut in [false, true] {
        let mut root = f::account_root(false);
        let call = root.submit(&["retry", "43", "--source", "key"]);
        let mut accounts = f::accounts();
        let row = accounts.accounts.get_mut(0).expect("row");
        let source = row.sources.get(0).expect("source").clone();
        row.sources = List::with_capacity(2);
        row.sources.push(source.clone()).expect("first");
        row.sources.push(source).expect("second");
        row.cut = cut;
        f::deliver_accounts(&mut root, accounts);
        assert_eq!(root.answer(call).expect("local refusal").status, 1);
        assert!(root.sends.is_empty());
    }
}
#[test]
fn a_key_value_never_enters_the_request_store() {
    let mut root = f::account_root(false);
    let call = root.submit_value(ADD, Some(Secret::new(KEY.into())));
    assert_eq!(root.carried.get(&call).expect("transient value").expose(), KEY);
    let (_, name, ask) = root.sends.get(&call).expect("sent once");
    assert_eq!(*name, f::named("add"));
    assert_eq!(
        ask,
        &jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Deployment,
            change: c::AccountChange::AddKey { provider: 9, join: None, digest: skein_lib::sha256::digest(KEY) },
        }))
    );
    assert!(root.saved.is_empty());
    root.done(call, c::Outcome::Owner(c::OwnerOutcome::AccountKeyKept { source: f::SOURCE }));
    assert_eq!(root.answer(call).expect("durable answer").status, 0);
}
#[test]
fn connect_prints_the_watch_that_follows_its_address() {
    let mut root = f::account_root(false);
    let call = root.submit(&["connect", "0", "--key", "connect"]);
    assert_eq!(
        root.sends.get(&call).expect("configured provider zero").2,
        jig_client::Ask::Jig(Box::new(c::Ask::SignIn { holder: c::AccountHolder::Own, provider: 0 }))
    );
    root.done(
        call,
        c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted {
            source: f::SOURCE,
            address: Box::from(b"http://engine/visit".as_slice()),
        }),
    );
    let printed = root.answer(call).expect("durable start");
    assert_eq!(printed.status, 0);
    assert!(
        printed.stdout.windows(b"wait accounts --until settled".len()).any(|w| w == b"wait accounts --until settled")
    );
}
#[test]
fn unconfirmed_disconnect_names_the_provider_settings() {
    let mut root = f::account_root(false);
    let call = root.submit(&["disconnect", "43"]);
    f::deliver_accounts(&mut root, f::accounts());
    assert_eq!(
        root.sends.get(&call).expect("disconnect").2,
        jig_client::Ask::Jig(Box::new(c::Ask::SignOut { holder: c::AccountHolder::Own, source: f::SOURCE }))
    );
    root.done(call, c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut { confirmed: false }));
    let printed = root.answer(call).expect("removed locally");
    assert_eq!(printed.status, 0);
    assert!(printed.stdout.windows(b"provider's settings".len()).any(|w| w == b"provider's settings"));
}
#[test]
fn a_value_request_is_not_retried_automatically() {
    for envelope in [skein_lib::RequestEnvelope::Again, skein_lib::RequestEnvelope::Lost] {
        let mut root = f::account_root(false);
        let call = root.submit_value(ADD, Some(Secret::new(KEY.into())));
        let (attempt, name, ask) = root.sends.get(&call).expect("one attempt").clone();
        root.client(jig_client::Event::Up {
            up: jig_client::Up::Answer { attempt, envelope, answer: None, why: Some(jig_client::Retry::Busy) },
        });
        let printed = root.answer(call).expect("caller must resupply");
        assert_eq!(printed.status, 1);
        assert!(root.saved.is_empty());
        root.sends.clear();
        root.now = root.now.saturating_add(skein_lib::Duration::from_secs(60));
        root.fire();
        assert!(root.sends.is_empty());
        let retry = root.submit_value(ADD, Some(Secret::new(KEY.into())));
        let (_, next_name, next_ask) = root.sends.get(&retry).expect("explicit re-entry");
        assert_eq!((*next_name, next_ask), (name, &ask));
    }
}
#[test]
fn an_engine_key_refusal_exits_two() {
    let mut root = f::account_root(false);
    let call = root.submit_value(ADD, Some(Secret::new(KEY.into())));
    root.answered(call, c::Answer::Refused { refusal: c::Refusal::KeyConflict, at: None });
    let printed = root.answer(call).expect("engine refusal");
    assert_eq!(printed.status, 2);
    assert!(!printed.stderr.windows(KEY.len()).any(|w| w == KEY));
}
#[test]
fn replacement_draws_an_ephemeral_name() {
    let mut root = f::account_root(false);
    let call = root.submit_value(&["replace-key", "43", "file"], Some(Secret::new(KEY.into())));
    f::deliver_accounts(&mut root, f::accounts());
    assert!(root.sent_name(call).is_some());
    assert!(root.saved.is_empty());
    assert_eq!(
        root.sends.get(&call).expect("replacement").2,
        jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
            holder: c::AccountHolder::Own,
            change: c::AccountChange::ReplaceKey { source: f::SOURCE, digest: skein_lib::sha256::digest(KEY) },
        }))
    );
}

#[test]
fn out_of_bound_secrets_are_refused_at_call_admission() {
    for value in [Box::from([]), Box::from([0; 4097])] {
        let mut root = f::account_root(false);
        let call = root.submit_value(ADD, Some(Secret::new(value)));
        assert_eq!(root.answer(call).expect("client bound").status, 1);
        assert!(root.sends.is_empty());
    }
}
