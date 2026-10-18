use super::fixtures;
use jig_domain_accounts::{self as accounts, FromComponent};
use skein_io::kernel::{Complete, Error, Op};
use skein_lib::{Duration, Queue, Time, Wall};
use temper_engine_domain as engine;

#[test]
fn a_handed_in_account_is_granted_from_its_file() {
    let mut stand = fixtures::accounts_with(fixtures::ACCOUNT);
    let events = fixtures::want(&mut stand, 1);
    let [engine::Event::Account(accounts::Event::Component(FromComponent::Granted { account, generation, valid }))] =
        events.as_ref()
    else {
        panic!("one grant")
    };
    assert_eq!((*account, *generation, *valid), (1, 1, Duration::from_secs(119)));
    assert!(stand.accounts.io.is_empty());
    assert!(stand.accounts.submitted.is_empty());
}

#[test]
fn a_granted_token_is_lent_and_kept_out_of_the_root() {
    let mut stand = fixtures::accounts_with(fixtures::ACCOUNT);
    let events = fixtures::want(&mut stand, 1);
    let [
        engine::Event::Account(accounts::Event::Component(FromComponent::Granted {
            account: 1, generation: 1, ..
        })),
    ] = events.as_ref()
    else {
        panic!("one grant of account 1 at generation 1")
    };
    assert_eq!(fixtures::lent(&stand, 1, 1).expose(), b"test-token");
    let value = stand.values.accounts.value(1, 1, Time::ZERO).unwrap();
    assert_eq!(value.account_id.expose(), b"acc");
}

#[test]
fn an_account_whose_file_is_missing_is_refused_by_number() {
    let mut stand = fixtures::missing_account();
    assert!(fixtures::want(&mut stand, 1).is_empty());
    let wall = Wall::from_nanos(1_000_000_000);
    let mut events = Queue::with_capacity(8);
    stand.accounts.iterate(Time::ZERO, wall, &mut stand.values, &mut events);
    let submit = stand.accounts.submitted.pop().expect("private record open");
    let Op::Open { .. } = submit.kind else { panic!("opens the private account file") };
    stand.accounts.complete(Complete { op: submit.op, kind: submit.kind, result: Err(Error::NotFound) });
    stand.accounts.iterate(Time::ZERO, wall, &mut stand.values, &mut events);
    let Some(engine::Event::Account(accounts::Event::Component(FromComponent::Failed { account: 1, .. }))) =
        events.pop()
    else {
        panic!("failure names account 1")
    };
    assert!(events.is_empty());
    assert!(stand.values.accounts.value(1, 1, Time::ZERO).is_none());
    assert!(stand.accounts.io.is_empty());
}

#[test]
fn a_closed_account_keeps_its_private_root_close_runnable() {
    let mut stand = fixtures::missing_account();
    let mut events = Queue::with_capacity(8);
    stand.accounts.request(accounts::Request::Component(accounts::ToComponent::Close));
    stand.accounts.iterate(Time::ZERO, Wall::EPOCH, &mut stand.values, &mut events);
    assert!(!stand.accounts.is_empty());
    assert!(stand.accounts.work_pending(Time::ZERO), "the private root still needs closing");
    stand.accounts.iterate(Time::ZERO, Wall::EPOCH, &mut stand.values, &mut events);
    let submit = stand.accounts.submitted.pop().expect("private root close");
    assert_eq!(submit.kind, Op::Close { fd: skein_io::kernel::Fd::new(10) });
    stand.accounts.complete(Complete { op: submit.op, kind: submit.kind, result: Ok(skein_io::kernel::Done::Nothing) });
    stand.accounts.iterate(Time::ZERO, Wall::EPOCH, &mut stand.values, &mut events);
    assert!(stand.accounts.is_empty());
    assert!(!stand.accounts.work_pending(Time::ZERO));
}
