use crate::tests::fixtures;
use jig_client::{self as client, contract as c};
#[test]
fn two_calls_in_flight_are_answered_each_under_its_own_number() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let a = root.submit(fixtures::TASK_CALL);
    let b = root.submit(fixtures::OTHER_TASK_CALL);
    root.serve(b, fixtures::task_result(fixtures::goal(fixtures::OTHER_TASK, fixtures::waiting_on_me())));
    assert!(root.answer(a).is_none());
    assert_eq!(root.answer(b).expect("second answered").status, 0);
    root.serve(a, fixtures::task_result(fixtures::goal(fixtures::TASK, fixtures::waiting_on_me())));
    assert_eq!(root.answer(a).expect("first answered").status, 0);
}
#[test]
fn an_unknown_asker_tells_no_child() {
    for (asker, facts) in [(fixtures::UNKNOWN_ASKER, 1), (temper_client_domain::owner::Owner::Forge.asker(), 0)] {
        let mut root = fixtures::signed_in(fixtures::PARTY);
        root.client(client::Event::Watch {
            handle: client::Handle { asker, number: 1 },
            subject: client::Watchable::Jig(c::Subject::Me),
        });
        assert_eq!(root.facts, facts, "{asker:?}");
    }
}
#[test]
fn new_calls_start_above_all_kept_call_numbers() {
    let mut root = fixtures::started(fixtures::kept(fixtures::RESTORED_PREFIX));
    assert_eq!(root.root.next, Some(fixtures::RESTORED_PREFIX + 1));
    root.deliver(c::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    let call = root.submit(fixtures::TASK_CALL);
    root.serve(call, fixtures::task_result(fixtures::goal(fixtures::TASK, fixtures::waiting_on_me())));
    assert_eq!(root.answer(call).expect("new read answered").status, 0);
}
#[test]
fn call_number_exhaustion_exits_1_without_sending() {
    let mut root = fixtures::started(fixtures::kept(u32::MAX));
    root.deliver(c::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    let call = root.submit(fixtures::TASK_CALL);
    let answer = root.answer(call).expect("exhaustion answered locally");
    assert_eq!((answer.status, answer.stderr.as_ref()), (1, b"call number exhausted\n".as_slice()));
    assert_eq!(root.reads.len(), 0);
}

#[test]
fn every_call_handle_uses_its_prefix_under_one_owner() {
    use crate::jig::runner;
    let mut root = fixtures::signed_in(fixtures::PARTY);
    for prefix in [1, fixtures::RESTORED_PREFIX] {
        let mut out = skein_lib::Queue::with_capacity(fixtures::OUTPUTS);
        let env =
            skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: root.limits.tools };
        runner::call(
            &mut root.root.runner,
            &root.root.client,
            &env,
            runner::Called {
                number: fixtures::CORRELATION,
                call_number: prefix,
                call: fixtures::call(fixtures::TASK_CALL),
                json: false,
            },
            &mut out,
        );
        let runner::Out::Client { event: client::Event::Read { handle, .. } } = out.pop().expect("read once") else {
            panic!("read event")
        };
        assert_eq!(handle, client::Handle { asker: super::RUNNER.asker(), number: (u64::from(prefix) << 32u32) | 1 });
    }
}
