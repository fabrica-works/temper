use super::fixtures;
use skein_lib::{Duration, Time};
use temper_engine_domain as root;

#[test]
fn the_root_reports_the_earliest_credentials_expiry() {
    let (mut domain, mut env) = fixtures::root();
    env.now = Time::from_nanos(5_000_000_000);
    assert_eq!(domain.next_deadline(&env), None);
    for (number, valid, expected) in [(1, 7, 12_u64), (2, 2, 7)] {
        root::try_step(
            &mut domain,
            &env,
            root::Event::Credential { number, generation: 1, valid: Duration::from_secs(valid) },
        )
        .expect("credential admitted");
        assert_eq!(domain.next_deadline(&env), Some(Time::from_nanos(expected.checked_mul(1_000_000_000).unwrap())));
    }
}

#[test]
fn the_forge_reports_the_earliest_procedure_or_projection_wait() {
    let own = crate::limits::profile(crate::limits::Profile::Small);
    let mut domain = temper_engine_domain_forge::Domain::new(
        &own.forge,
        7,
        temper_engine_domain_forge_client::Config {
            namespace: Box::new([7; 16]),
            writers: Box::new([temper_engine_domain_forge_client::Writer { forge: 1, author: 42 }]),
        },
        &[temper_engine_domain_forge::GitIdentity { forge: 1, credential: 1 }],
    )
    .unwrap();
    let env = skein_lib::Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: own.forge };
    assert_eq!(domain.next_wall_deadline(), None);
    domain.wait_change(&env, 1, Some(skein_lib::Wall::from_nanos(7)));
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(7)));
    domain.projection_after(2, skein_lib::Wall::from_nanos(3));
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(3)));
    domain.projection_settled(2);
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(7)));
    domain.offered_change(1);
    assert_eq!(domain.next_wall_deadline(), None);
}

#[test]
fn the_restart_asks_one_load_at_a_time() {
    let mut service = fixtures::started();
    for _ in 0_u32..16 {
        fixtures::turn(&mut service);
        if !service.engine.store.above.is_empty() {
            break;
        }
    }
    let event = service.engine.store.above.pop();
    let Some(jig_protocol_store::Event::Restored { owner, rows, next }) = event else {
        panic!("first restart load: {event:?}")
    };
    assert!(rows.is_empty());
    assert_eq!(next, None);
    assert!(service.engine.store.above.is_empty());
    fixtures::turn(&mut service);
    assert!(service.engine.store.above.is_empty(), "no next load before the first answer");
    service.engine.store.above.push(jig_protocol_store::Event::Restored { owner, rows, next });
    for _ in 0_u32..16 {
        fixtures::turn(&mut service);
        if !service.engine.store.above.is_empty() {
            break;
        }
    }
    assert_eq!(service.end(), None);
    assert_eq!(service.engine.store.above.len(), 1, "one next store operation");
    fixtures::restart(&mut service);
}
#[test]
fn a_forge_call_is_answered_unavailable_at_no_cost() {
    use temper_engine_domain_forge_client as client;
    let mut service = fixtures::started();
    let call = skein_lib::Token::new(7);
    crate::route::request(
        &mut service.engine,
        root::Request::Forge {
            call,
            repository: client::api::Repository { forge: 1, repository: 2 },
            op: client::api::Op::Read(client::api::Read::Settings),
        },
    );
    let Some(root::Event::Forge(answer)) = service.engine.events.pop() else { panic!("forge answer") };
    assert_eq!(
        *answer,
        temper_engine_domain_forge::Event::Client(client::Event::Answered {
            call,
            cost: 0,
            result: Err(client::api::Error::Unavailable),
        })
    );
}
#[test]
fn a_store_answer_is_taken_with_the_journal_full() {
    let mut service = fixtures::ready();
    let engine = &mut service.engine;
    let mut number = 2;
    for _ in 0_u32..engine.env.limits.journal.commits.checked_add(1).unwrap() {
        if root::try_step(&mut engine.domain, &engine.env, fixtures::period(number)).is_err() {
            break;
        }
        number = number.checked_add(1).unwrap();
    }
    assert!(root::try_step(&mut engine.domain, &engine.env, fixtures::period(number)).is_err(), "journal is full");
    root::release(&mut engine.domain, &engine.env, &mut engine.requests);
    let Some(root::Request::Store(root::StoreRequest::Commit { number, writes })) = engine.requests.pop() else {
        panic!("commit")
    };
    assert!(engine.store.down(root::StoreRequest::Commit { number, writes }));
    crate::engine::iterate(engine, Time::from_nanos(1), skein_lib::Wall::from_nanos(1));
    for event in &engine.store.above {
        if let jig_protocol_store::Event::Committed { number: found, .. } = event {
            assert_ne!(*found, number, "original terminal taken while admission was blocked");
        }
    }
    assert!(root::try_step(&mut engine.domain, &engine.env, fixtures::period(20)).is_ok(), "answer freed journal room");
}
#[test]
fn a_failed_commit_releases_nothing_after_it() {
    let mut service = fixtures::ready();
    let engine = &mut service.engine;
    let first = engine.domain.deployment().commits.checked_add(1).unwrap();
    root::try_step(&mut engine.domain, &engine.env, fixtures::period(2)).unwrap();
    root::try_step(&mut engine.domain, &engine.env, fixtures::period(3)).unwrap();
    engine.store.above.push(jig_protocol_store::Event::Failed { number: first });
    engine.store.above.push(jig_protocol_store::Event::Committed { number: first.checked_add(1).unwrap(), fill: 0 });
    fixtures::turn(&mut service);
    assert_eq!(service.end(), Some(&crate::End::Stopped(crate::Stop::CommitFailed { number: first })));
    assert!(service.engine.requests.is_empty(), "later outputs stayed behind the barrier");
    assert!(service.engine.store.above.is_empty());
}
