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
    &["declare", "9", "helper", "work-login", "--key", "declare"],
    &["declare", "9", "helper", "work-login", "--account", "43", "--key", "join"],
    &["verify", "43"],
    &["verify", "43", "--source", "program", "--key", "check"],
];
#[test]
fn account_forms_round_trip_through_the_table() {
    for words in FORMS {
        let call = f::call(words);
        assert_eq!(f::back(&call), f::owned(words));
        assert_eq!(f::printed(&call), Parsed::Call { call, json: false });
    }
}
#[test]
fn account_usage_is_refused_before_sending() {
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
        (&["rank", "43", "2"], c::AccountChange::MoveAccount { account: f::ACCOUNT, place: 1 }),
        (
            &["rank", "43", "2", "--source", "key"],
            c::AccountChange::MoveSource { account: f::ACCOUNT, source: f::SOURCE, place: 1 },
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
    for (words, holder, join) in [
        (ADD, c::AccountHolder::Deployment, None),
        (&["add-key", "9", "file", "--key", "add"][..], c::AccountHolder::Own, None),
        (&["add-key", "9", "file", "--account", "43", "--key", "add"], c::AccountHolder::Own, Some(f::ACCOUNT)),
    ] {
        let mut root = f::account_root(false);
        let call = root.submit_value(words, Some(Secret::new(KEY.into())));
        assert_eq!(root.carried.get(&call).expect("transient value").expose(), KEY);
        let (_, name, ask) = root.sends.get(&call).expect("sent once");
        assert_eq!(*name, f::named("add"));
        assert_eq!(
            ask,
            &jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                holder,
                change: c::AccountChange::AddKey {
                    provider: f::PROVIDER,
                    join,
                    digest: skein_lib::sha256::digest(KEY)
                },
            }))
        );
        assert!(root.saved.is_empty());
        root.done(call, c::Outcome::Owner(c::OwnerOutcome::AccountKeyKept { source: f::SOURCE }));
        assert_eq!(root.answer(call).expect("durable answer").status, 0);
    }
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
    assert!(skein_lib::bytes::find(&printed.stdout, b"wait accounts --until settled").is_some());
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
    root.done(
        call,
        c::Outcome::Owner(c::OwnerOutcome::AccountSignedOut { confirmed: false, failed_hosts: Box::new([]) }),
    );
    let printed = root.answer(call).expect("removed locally");
    assert_eq!(printed.status, 0);
    assert!(skein_lib::bytes::find(&printed.stdout, b"provider's settings").is_some());
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
    assert!(skein_lib::bytes::find(&printed.stderr, KEY).is_none());
}
#[test]
fn replacement_draws_an_ephemeral_name() {
    let mut root = f::account_root(false);
    let call = root.submit_value(&["replace-key", "43", "file"], Some(Secret::new(KEY.into())));
    f::deliver_accounts(&mut root, f::accounts());
    let first = root.sent_name(call).expect("drawn name");
    root.done(call, c::Outcome::Owner(c::OwnerOutcome::AccountKeyKept { source: f::SOURCE }));
    let next = root.submit_value(&["replace-key", "43", "file"], Some(Secret::new(KEY.into())));
    f::deliver_accounts(&mut root, f::accounts());
    assert_ne!(first, root.sent_name(next).expect("second drawn name"));
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

#[test]
fn source_shorthand_resolves_the_supplied_kind() {
    for (word, kind) in [("sign_in", c::OwnerAccountKind::SignIn), ("program", c::OwnerAccountKind::Program)] {
        for tool in ["disconnect", "forget"] {
            let mut root = f::account_root(false);
            let call = root.submit(&[tool, "43", "--source", word]);
            let mut accounts = f::accounts();
            accounts.accounts.get_mut(0).expect("account").sources.get_mut(0).expect("source").kind = kind;
            f::deliver_accounts(&mut root, accounts);
            assert_eq!(
                root.sends.get(&call).expect("resolved sign-out").2,
                jig_client::Ask::Jig(Box::new(c::Ask::SignOut { holder: c::AccountHolder::Own, source: f::SOURCE }))
            );
        }
    }
}

#[test]
fn a_named_replacement_uses_the_supplied_name() {
    let mut root = f::account_root(false);
    let call = root.submit_value(&["replace-key", "43", "file", "--key", "replacement"], Some(Secret::new(KEY.into())));
    f::deliver_accounts(&mut root, f::accounts());
    assert_eq!(root.sent_name(call), Some(f::named("replacement")));
}
#[test]
fn account_zero_reaches_its_holder() {
    let mut root = f::account_root(false);
    let call = root.submit(&["forget", "0"]);
    let mut accounts = f::accounts();
    accounts.accounts.get_mut(0).expect("account").number = 0;
    f::deliver_accounts(&mut root, accounts);
    assert!(root.sends.contains_key(&call));
}
#[test]
fn a_lost_replacement_prints_its_recoverable_name() {
    for envelope in [skein_lib::RequestEnvelope::Again, skein_lib::RequestEnvelope::Lost] {
        let mut root = f::account_root(false);
        let call = root.submit_value(&["replace-key", "43", "file"], Some(Secret::new(KEY.into())));
        f::deliver_accounts(&mut root, f::accounts());
        let (attempt, name, _) = root.sends.get(&call).expect("first replacement").clone();
        root.client(jig_client::Event::Up {
            up: jig_client::Up::Answer { attempt, envelope, answer: None, why: Some(jig_client::Retry::Busy) },
        });
        let printed = root.answer(call).expect("recovery instruction");
        assert_eq!(printed.status, 1);
        let start = skein_lib::bytes::find(&printed.stderr, b"supply --key ").expect("printed name");
        let tail = printed.stderr.get(start.checked_add(13).expect("prefix fits")..).expect("name follows");
        let end = skein_lib::bytes::find(tail, b" ").expect("name end");
        let drawn = crate::jig::key::Name::new(tail.get(..end).expect("printed name")).expect("recoverable name");
        assert!(skein_lib::bytes::find(tail, b" and the same file again").is_some());
        assert_eq!(crate::jig::key::named(crate::key::TOOLS, &drawn), name);
    }
}

#[test]
fn declare_validates_the_offer_and_sends_only_the_contract_fields() {
    for join in [false, true] {
        let mut root = f::account_root(false);
        let mut me = f::me(f::PARTY);
        me.providers = f::provider_offers();
        root.deliver(c::Delivery::Me(Box::new(me)));
        let words = if join {
            &["declare", "9", "helper", "work-login", "--account", "43", "--key", "declaration"][..]
        } else {
            &["declare", "9", "helper", "work-login", "--key", "declaration"][..]
        };
        let call = root.submit(words);
        let (_, name, ask) = root.sends.get(&call).expect("declaration");
        assert_eq!(*name, f::named("declaration"));
        assert_eq!(
            ask,
            &jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                holder: c::AccountHolder::Own,
                change: c::AccountChange::DeclareProgram {
                    provider: f::PROVIDER,
                    join: if join { Some(f::ACCOUNT) } else { None },
                    name: b"work-login".as_slice().into(),
                },
            }))
        );
        root.done(
            call,
            c::Outcome::Owner(c::OwnerOutcome::AccountProgramDeclared {
                source: f::SOURCE,
                name: b"work-login".as_slice().into(),
            }),
        );
        let printed = root.answer(call).expect("declared");
        assert_eq!(printed.status, 0);
        assert!(f::has(&printed, b"Run on each host: temper program-login work-login"));
    }
}
#[test]
fn declare_refuses_unoffered_adapters_and_providers_locally() {
    for words in [
        &["declare", "9", "absent", "work-login", "--key", "declaration"][..],
        &["declare", "99", "helper", "work-login", "--key", "declaration"],
    ] {
        let mut root = f::account_root(false);
        let mut me = f::me(f::PARTY);
        me.providers = f::provider_offers();
        root.deliver(c::Delivery::Me(Box::new(me)));
        let call = root.submit(words);
        let printed = root.answer(call).expect("local refusal");
        assert_eq!(printed.status, 1);
        assert!(skein_lib::bytes::find(&printed.stderr, b"adapter not offered").is_some());
        assert!(root.sends.is_empty());
    }
}
#[test]
fn declare_refuses_unchecked_adapters_when_provider_offers_are_cut() {
    for (providers_cut, programs_cut, name_cut) in [(true, false, false), (false, true, false), (false, false, true)] {
        let mut root = f::account_root(false);
        let mut me = f::me(f::PARTY);
        me.providers = f::provider_offers();
        me.providers_cut = providers_cut;
        let offer = me.providers.get_mut(0).expect("offer");
        offer.programs_cut = programs_cut;
        offer.name.cut = name_cut;
        if providers_cut {
            me.providers = List::with_capacity(0);
        }
        root.deliver(c::Delivery::Me(Box::new(me)));
        let call = root.submit(&["declare", "9", "absent", "work-login", "--key", "declaration"]);
        let printed = root.answer(call).expect("local refusal");
        assert_eq!(printed.status, 1);
        assert!(skein_lib::bytes::find(&printed.stderr, b"provider offers cut in me; adapter unchecked").is_some());
        assert!(skein_lib::bytes::find(&printed.stderr, b"adapter not offered").is_none());
        assert!(root.sends.is_empty());
    }
}
#[test]
fn declare_checks_the_portable_name_bound_in_argv_and_json() {
    for name in ["", "UPPER", "with_space ", "../escape", "abcdefghijklmnopqrstuvwxyz1234567"] {
        assert!(f::parse(&["declare", "9", "helper", name, "--key", "k"]).is_err(), "{name}");
    }
    for name in ["a", "abcdefghijklmnopqrstuvwxyz123456"] {
        let call = f::call(&["declare", "9", "helper", name, "--key", "k"]);
        assert_eq!(f::printed(&call), Parsed::Call { call, json: false });
    }
    assert!(f::parse(&["declare", "9", "helper", "work-login"]).is_err());
    assert!(f::parse(&["declare", "4294967296", "helper", "work-login", "--key", "k"]).is_err());
}
#[test]
fn verify_asks_the_supplied_program_source_then_names_the_wait() {
    for owner in [false, true] {
        for json in [false, true] {
            let mut root = f::account_root(owner);
            let words =
                if json { &["verify", "43", "--source", "program", "--json"][..] } else { &["verify", "43"][..] };
            let call = root.submit(words);
            let mut accounts = f::accounts();
            accounts.accounts.get_mut(0).expect("account").sources.get_mut(0).expect("source").kind =
                c::OwnerAccountKind::Program;
            if owner {
                accounts.accounts = List::with_capacity(0);
            }
            f::deliver_accounts(&mut root, accounts);
            if owner {
                let mut deployment = f::deployment();
                deployment.accounts.get_mut(0).expect("account").sources.get_mut(0).expect("source").kind =
                    c::OwnerAccountKind::Program;
                f::deliver_deployment(&mut root, deployment);
            }
            assert_eq!(
                root.sends.get(&call).expect("check request").2,
                jig_client::Ask::Jig(Box::new(c::Ask::AccountChange {
                    holder: if owner { c::AccountHolder::Deployment } else { c::AccountHolder::Own },
                    change: c::AccountChange::CheckProgram { source: f::SOURCE },
                }))
            );
            assert!(root.answer(call).is_none(), "wait for durable answer");
            root.done(call, c::Outcome::Owner(c::OwnerOutcome::AccountChanged));
            let printed = root.answer(call).expect("asked");
            assert_eq!(printed.status, 0);
            assert!(f::has(&printed, b"wait accounts --until settled"));
        }
    }
}
#[test]
fn verify_refuses_sources_without_a_program_login_locally() {
    for kind in [c::OwnerAccountKind::Key, c::OwnerAccountKind::SignIn, c::OwnerAccountKind::None] {
        let mut root = f::account_root(false);
        let call = root.submit(&["verify", "43"]);
        let mut accounts = f::accounts();
        accounts.accounts.get_mut(0).expect("account").sources.get_mut(0).expect("source").kind = kind;
        f::deliver_accounts(&mut root, accounts);
        let printed = root.answer(call).expect("local refusal");
        assert_eq!(printed.status, 1);
        assert!(skein_lib::bytes::find(&printed.stderr, b"verify needs a program source").is_some());
        assert!(root.sends.is_empty());
    }
}
