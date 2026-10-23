use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store, walking};

fn config(under_box: bool) -> engine::Config {
    let mut config = walking::config(71);
    config.under_box = under_box;
    config.forges = Box::new([
        forge::Configured { number: 1, kind: forge::Kind::Forgejo, mode: forge::Mode::Fake, asked: true },
        forge::Configured { number: 2, kind: forge::Kind::Forgejo, mode: forge::Mode::Fake, asked: false },
    ]);
    config.demo =
        Some(forge::DemoConfig { provider: 1, adoption: fixtures::adoption(1), deadline: Duration::from_secs(10) });
    config
}
fn driver(connected: bool, under_box: bool) -> (Driver, u64) {
    let mut limits = fixtures::forge_limits();
    // Two starts, one account reply awaiting reclaim, and the adoption's admission.
    limits.people.pending = 4;
    limits.journal.writes += 4;
    limits.journal.deliveries += 4;
    limits.journal.held += 12;
    let store = fixtures::first_start_over(Store::new(), config(under_box), &limits);
    let mut d = Driver::cold(store, config(under_box), &limits);
    d.forge = Some(ForgeStand::new());
    d.settle();
    d.sign_in();
    d.settle();
    if connected {
        connection(&mut d, forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    }
    let sign_in = d.session();
    d.delivered.clear();
    (d, sign_in)
}
fn connection(d: &mut Driver, state: forge::Connection) {
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection { forge: 1, state })));
    d.settle();
}
fn start(d: &mut Driver, sign_in: u64, number: u8, carry: bool) {
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in,
        key: [number; 16],
        digest: [9; 32],
        scope: people::ApplicationScope::Project(999),
        request: forge::Ask::StartDemo,
        carried: carry.then_some(core::Carried { number: u64::from(number), digest: [7; 32] }),
    }));
    d.settle();
}
fn answers(d: &Driver) -> Vec<Option<forge::Answered>> {
    d.delivered
        .iter()
        .filter_map(
            |delivery| {
                if let Delivery::Application { outcome, .. } = delivery { Some(outcome.clone()) } else { None }
            },
        )
        .collect()
}
fn keeps(d: &Driver) -> usize {
    d.protocol
        .accounts
        .iter()
        .filter(|request| matches!(request, accounts::Request::Signing(accounts::Signing::Keep { .. })))
        .count()
}

#[test]
fn demo_start_adds_the_deployments_account_and_adopts_the_repository_once() {
    let (mut d, sign_in) = driver(true, false);
    start(&mut d, sign_in, 81, true);
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::Running))]);
    assert_eq!(keeps(&d), 1);
    assert!(d.store.rows.values().any(|record| matches!(record, engine::Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::Repository(row)
            if row.project == 1 && row.owner.as_ref() == b"org" && row.name.as_ref() == b"repo"))));
    assert!(d.store.rows.values().any(|record| matches!(
        record,
        engine::Record::Core(core::Record::Accounts(accounts::Stored::Account {
            holder: accounts::Holder::Deployment,
            provider: 1,
            ..
        }))
    )));
    let calls = d.forge.as_ref().expect("fake forge").calls;
    d.delivered.clear();
    start(&mut d, sign_in, 82, true);
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::Running))]);
    assert_eq!((keeps(&d), d.forge.as_ref().expect("fake forge").calls), (1, calls));
    assert!(matches!(d.delivered.first(), Some(Delivery::CarriedUnused { number: 82 })));
}

#[test]
fn demo_start_replays_its_retained_answer_without_repeating_acts() {
    let (mut d, sign_in) = driver(true, false);
    start(&mut d, sign_in, 81, true);
    let calls = d.forge.as_ref().expect("fake forge").calls;
    d.delivered.clear();
    start(&mut d, sign_in, 81, true);
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::Running))]);
    assert_eq!(keeps(&d), 1);
    assert_eq!(d.forge.as_ref().expect("fake forge").calls, calls);
    assert!(matches!(d.delivered.first(), Some(Delivery::CarriedUnused { number: 81 })));
}

#[test]
fn demo_start_fails_when_its_account_is_refused_or_the_key_is_missing() {
    for carry in [false, true] {
        let (mut d, sign_in) = driver(true, false);
        d.protocol.key_failure = Some(accounts::KeyFailure::NotStored);
        start(&mut d, sign_in, 81, carry);
        assert_eq!(
            answers(&d),
            [Some(forge::Answered::StartDemo(forge::StartAnswer::Failed(forge::StartFailure::Account)))]
        );
        assert_eq!(d.forge.as_ref().expect("fake forge").calls, 0);
    }
}

#[test]
fn demo_start_fails_when_its_adoption_is_refused() {
    let (mut d, sign_in) = driver(true, false);
    d.forge = Some(ForgeStand::with(&[(b"org", b"another", 3)]));
    start(&mut d, sign_in, 81, true);
    assert_eq!(
        answers(&d),
        [Some(forge::Answered::StartDemo(forge::StartAnswer::Failed(forge::StartFailure::Adoption)))]
    );
    assert_eq!(keeps(&d), 1);
}

#[test]
fn demo_start_names_the_groups_failure_and_truncates_its_reason() {
    let (mut d, sign_in) = driver(false, true);
    start(&mut d, sign_in, 81, true);
    assert!(answers(&d).is_empty());
    assert!(d.immediate.iter().any(|request| matches!(request, engine::Request::SetUpGroup { forge: 1, .. })));
    connection(&mut d, forge::Connection::SetUpFailed { why: vec![b'x'; 300].into_boxed_slice() });
    assert_eq!(
        answers(&d),
        [Some(forge::Answered::StartDemo(forge::StartAnswer::Failed(forge::StartFailure::Group {
            why: vec![b'x'; 256].into_boxed_slice()
        })))]
    );
}

#[test]
fn demo_start_waits_for_connection_until_the_configured_deadline() {
    let (mut d, sign_in) = driver(false, false);
    start(&mut d, sign_in, 81, true);
    assert!(answers(&d).is_empty());
    d.fire_at(Duration::from_secs(10));
    d.settle();
    assert_eq!(
        answers(&d),
        [Some(forge::Answered::StartDemo(forge::StartAnswer::Failed(forge::StartFailure::NotConnected)))]
    );
}

#[test]
fn demo_restart_asks_the_group_makes_no_act_and_forgets_the_start_answer() {
    let (mut d, sign_in) = driver(true, true);
    start(&mut d, sign_in, 81, true);
    let old_keeps = keeps(&d);
    let old_calls = d.forge.as_ref().expect("fake forge").calls;
    d.root = engine::Domain::new(config(true), &d.env.limits).expect("valid demo config");
    d.immediate.clear();
    d.delivered.clear();
    d.send(engine::Event::Restart);
    d.settle();
    assert!(d.immediate.iter().any(|request| matches!(request, engine::Request::SetUpGroup { forge: 1, .. })));
    assert_eq!((keeps(&d), d.forge.as_ref().expect("fake forge").calls), (old_keeps, old_calls));
    d.sign_in();
    d.settle();
    let sign_in = d.session();
    d.delivered.clear();
    start(&mut d, sign_in, 81, true);
    assert_eq!(answers(&d), [None]);
}

#[test]
fn demo_start_skips_an_existing_account_but_still_adopts() {
    let (mut d, sign_in) = driver(false, false);
    start(&mut d, sign_in, 81, true);
    d.fire_at(Duration::from_secs(10));
    d.settle();
    connection(&mut d, forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    d.delivered.clear();
    start(&mut d, sign_in, 82, true);
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::Running))]);
    assert_eq!(keeps(&d), 1);
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::CarriedUnused { number: 82 })));
    assert!(d.store.rows.values().any(|record| matches!(record, engine::Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::Repository(_)))));
}

#[test]
fn demo_start_skips_an_existing_repository_but_still_adds_the_account() {
    let (mut d, sign_in) = driver(true, false);
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(80)),
        sign_in,
        key: [80; 16],
        digest: [8; 32],
        scope: people::ApplicationScope::Project(1),
        request: forge::Ask::Adopt(fixtures::adoption(1)),
        carried: None,
    }));
    d.settle();
    let calls = d.forge.as_ref().expect("fake forge").calls;
    d.delivered.clear();
    start(&mut d, sign_in, 81, true);
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::Running))]);
    assert_eq!(keeps(&d), 1);
    assert_eq!(d.forge.as_ref().expect("fake forge").calls, calls);
}

#[test]
fn simultaneous_demo_starts_share_one_pair_of_acts_and_return_unused_keys() {
    let (mut d, sign_in) = driver(false, false);
    start(&mut d, sign_in, 81, true);
    start(&mut d, sign_in, 82, true);
    assert!(answers(&d).is_empty());
    connection(&mut d, forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    assert_eq!(answers(&d), vec![Some(forge::Answered::StartDemo(forge::StartAnswer::Running)); 2]);
    assert_eq!(keeps(&d), 1);
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::CarriedUnused { number: 82 })));
}

#[test]
fn a_busy_box_link_fails_the_demo_start_with_the_groups_reason() {
    let (mut d, sign_in) = driver(false, true);
    start(&mut d, sign_in, 81, true);
    let request = d
        .immediate
        .iter()
        .find_map(
            |request| if let engine::Request::SetUpGroup { request, .. } = request { Some(*request) } else { None },
        )
        .expect("demo group ask");
    d.send(engine::Event::Forge(Box::new(forge::Event::SetUpAnswered {
        forge: 1,
        request,
        answer: forge::SetUpAnswer::Refused(forge::SetUpRefusal::Busy),
    })));
    d.settle();
    assert_eq!(
        answers(&d),
        [Some(forge::Answered::StartDemo(forge::StartAnswer::Failed(forge::StartFailure::Group {
            why: b"busy".as_slice().into()
        })))]
    );
    connection(&mut d, forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    d.delivered.clear();
    start(&mut d, sign_in, 82, true);
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::Running))]);
    assert_eq!(keeps(&d), 1);
    assert!(matches!(d.delivered.first(), Some(Delivery::CarriedUnused { number: 82 })));
}

#[test]
fn another_connected_forge_ends_the_demo_persists_and_stops_once_and_restart_asks_no_group() {
    let (mut d, sign_in) = driver(true, true);
    start(&mut d, sign_in, 81, true);
    let watcher = Token::new(901);
    d.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in,
        subject: core::documents::Watched::Application {
            scope: people::ApplicationScope::Deployment,
            connector: 0,
            subject: Token::new(0),
        },
        window: core::documents::Window { rows: 2, after: None },
    }));
    d.settle();
    assert!(
        d.delivered
            .iter()
            .any(|delivery| matches!(delivery, Delivery::Forges { forges, .. } if forges.rows.len() == 2))
    );
    d.send(engine::Event::Party(engine::PartyInput::Delivered { watcher }));
    d.settle();
    d.delivered.clear();
    for _ in 0..2 {
        d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
            forge: 2,
            state: forge::Connection::Connected { id: 2, login: b"other".as_slice().into() },
        })));
        d.settle();
    }
    assert!(d.store.rows.values().any(|record| matches!(record, engine::Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::Demo(forge::Demo { forge: 1, state: forge::DemoState::Ended })))));
    assert_eq!(d.root.forges().rows.iter().map(|row| row.number).collect::<Vec<_>>(), [2]);
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::Forges { forges, .. } if forges.rows.len() == 1 && forges.rows[0].number == 2)));
    assert_eq!(
        d.immediate.iter().filter(|request| matches!(request, engine::Request::StopGroup { forge: 1 })).count(),
        1
    );
    start(&mut d, sign_in, 82, true);
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::NotOffered))]);
    d.root = engine::Domain::new(config(true), &d.env.limits).expect("valid demo config");
    d.immediate.clear();
    d.send(engine::Event::Restart);
    d.settle();
    assert_eq!(d.root.demo_state(), Some(forge::DemoState::Ended));
    assert!(d.root.forge_ended(1));
    assert!(
        !d.immediate
            .iter()
            .any(|request| matches!(request, engine::Request::SetUpGroup { .. } | engine::Request::StopGroup { .. }))
    );
}

#[test]
fn another_forge_connecting_answers_a_held_demo_start_not_offered() {
    let (mut d, sign_in) = driver(false, true);
    start(&mut d, sign_in, 81, true);
    assert!(answers(&d).is_empty());
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: 2,
        state: forge::Connection::Connected { id: 2, login: b"other".as_slice().into() },
    })));
    d.settle();
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::NotOffered))]);
    connection(&mut d, forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() });
    d.fire_at(Duration::from_secs(10));
    d.settle();
    assert_eq!(answers(&d), [Some(forge::Answered::StartDemo(forge::StartAnswer::NotOffered))]);
}
