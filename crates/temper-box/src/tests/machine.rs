use super::fixtures::{FORGEJO, PREPARE, RECONCILE_1, RECONCILE_2, RUNNER, Rig, SERVE};
use crate::{
    limits,
    machine::{Input, Order, Outcome},
};
use skein_io::kernel::Exit;
use skein_lib::Duration;
#[test]
fn a_once_step_must_exit_zero_before_the_next_starts() {
    let mut rig = Rig::new(&limits::small());
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: PREPARE }]);
    rig.started(PREPARE);
    assert_eq!(&*rig.orders(), &[]);
    rig.exited(PREPARE, Exit::Code(0));
    assert_eq!(&*rig.orders(), &[Order::Close { step: PREPARE }]);
    rig.closed(PREPARE);
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: FORGEJO }]);
}
#[test]
fn a_long_step_lets_the_next_start_once_it_has_started() {
    let mut rig = Rig::new(&limits::small());
    rig.completed(PREPARE);
    rig.orders();
    rig.started(FORGEJO);
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: RUNNER }]);
}
#[test]
fn a_long_step_that_exits_runs_its_once_steps_again_before_it() {
    let mut rig = Rig::running();
    rig.exited(FORGEJO, Exit::Code(1));
    assert_eq!(&*rig.orders(), &[Order::Close { step: FORGEJO }]);
    rig.closed(FORGEJO);
    rig.advance(Duration::from_millis(500));
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: PREPARE }]);
    rig.completed(PREPARE);
    assert_eq!(&*rig.orders(), &[Order::Close { step: PREPARE }, Order::Spawn { step: FORGEJO }]);
    rig.started(FORGEJO);
    assert_eq!(&*rig.orders(), &[]);
}
#[test]
fn a_long_step_that_exits_runs_every_once_step_just_before_it_again() {
    let mut rig = Rig::running();
    rig.failed_long(SERVE);
    rig.advance(Duration::from_millis(500));
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: RECONCILE_1 }]);
    rig.completed(RECONCILE_1);
    assert_eq!(&*rig.orders(), &[Order::Close { step: RECONCILE_1 }, Order::Spawn { step: RECONCILE_2 }]);
    rig.completed(RECONCILE_2);
    assert_eq!(&*rig.orders(), &[Order::Close { step: RECONCILE_2 }, Order::Spawn { step: SERVE }]);
}
#[test]
fn a_long_step_after_a_long_step_restarts_alone() {
    let mut rig = Rig::running();
    rig.failed_long(RUNNER);
    rig.advance(Duration::from_millis(500));
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: RUNNER }]);
    rig.started(RUNNER);
    assert_eq!(&*rig.orders(), &[]);
}
#[test]
fn the_backoff_doubles_and_stops_doubling_at_its_maximum() {
    // Eight retries reach the cap; the shipped five stop before it.
    let mut rig = Rig::new(&limits::Limits { restarts: 8, ..limits::small() });
    rig.completed(PREPARE);
    rig.started(FORGEJO);
    rig.orders();
    for millis in [500, 1_000, 2_000, 4_000, 8_000, 16_000, 30_000, 30_000] {
        rig.failed_long(FORGEJO);
        assert_eq!(
            rig.machine.next_deadline(),
            Some(rig.now.saturating_add(Duration::from_millis(millis))),
            "gap {millis}"
        );
        rig.advance(Duration::from_millis(millis));
        rig.orders();
        rig.restart_forgejo();
    }
}
#[test]
fn the_backoff_starts_again_after_a_window_without_an_exit() {
    let mut rig = Rig::running();
    rig.failed_long(FORGEJO);
    rig.advance(Duration::from_millis(500));
    rig.orders();
    rig.restart_forgejo();
    rig.advance(Duration::from_secs(301));
    rig.failed_long(FORGEJO);
    assert_eq!(rig.machine.next_deadline(), Some(rig.now.saturating_add(Duration::from_millis(500))));
}
#[test]
fn more_exits_than_restarts_within_the_window_give_up_naming_the_step() {
    let mut rig = Rig::running();
    for millis in [500, 1_000, 2_000, 4_000, 8_000] {
        rig.failed_long(FORGEJO);
        rig.advance(Duration::from_millis(millis));
        rig.orders();
        rig.restart_forgejo();
    }
    rig.exited(FORGEJO, Exit::Code(1));
    rig.closed(FORGEJO);
    assert_eq!(rig.machine.outcome(), Some(Outcome::GaveUp { step: FORGEJO, restarts: 5 }));
    assert_eq!(&*rig.orders(), &[Order::Close { step: FORGEJO }, Order::Signal { step: SERVE }]);
}
#[test]
fn a_once_step_that_fails_gives_up_and_stops_what_runs() {
    for not_started in [false, true] {
        let mut rig = Rig::running();
        rig.failed_long(SERVE);
        rig.advance(Duration::from_millis(500));
        rig.orders();
        if not_started {
            rig.input(Input::NotStarted { step: RECONCILE_1 });
        } else {
            rig.started(RECONCILE_1);
            rig.exited(RECONCILE_1, Exit::Code(1));
            rig.closed(RECONCILE_1);
        }
        assert_eq!(
            rig.machine.outcome(),
            Some(Outcome::GaveUp { step: RECONCILE_1, restarts: 0 }),
            "not started {not_started}"
        );
        let expected: &[Order] = if not_started {
            &[Order::Signal { step: RUNNER }]
        } else {
            &[Order::Close { step: RECONCILE_1 }, Order::Signal { step: RUNNER }]
        };
        assert_eq!(&*rig.orders(), expected, "not started {not_started}");
    }
}
#[test]
fn a_stop_signals_the_long_steps_last_first_one_at_a_time() {
    let mut rig = Rig::running();
    rig.terminate();
    for (step, next) in [(SERVE, Some(RUNNER)), (RUNNER, Some(FORGEJO)), (FORGEJO, None)] {
        assert_eq!(&*rig.orders(), &[Order::Signal { step }]);
        rig.exited(step, Exit::Code(0));
        assert_eq!(&*rig.orders(), &[Order::Close { step }]);
        rig.closed(step);
        if next.is_none() {
            assert_eq!(&*rig.orders(), &[]);
        }
    }
    assert!(rig.machine.is_idle(), "all children closed");
    assert_eq!(rig.machine.outcome(), Some(Outcome::Stopped));
}
#[test]
fn every_long_step_is_signalled_twice_with_a_grace_after_each() {
    let mut rig = Rig::running();
    rig.terminate();
    for step in [SERVE, RUNNER, FORGEJO] {
        assert_eq!(&*rig.orders(), &[Order::Signal { step }]);
        rig.advance(Duration::from_secs(10));
        assert_eq!(&*rig.orders(), &[Order::Signal { step }]);
        rig.advance(Duration::from_secs(10));
        assert_eq!(&*rig.orders(), &[Order::Close { step }]);
        rig.closed(step);
    }
    assert!(rig.machine.is_idle(), "all children closed");
}
#[test]
fn a_stop_during_a_backoff_starts_nothing() {
    let mut rig = Rig::running();
    rig.failed_long(FORGEJO);
    rig.terminate();
    assert_eq!(&*rig.orders(), &[Order::Signal { step: SERVE }]);
    rig.advance(Duration::from_millis(500));
    assert_eq!(&*rig.orders(), &[]);
}
#[test]
fn a_step_still_starting_is_stopped_once_started() {
    let mut rig = Rig::new(&limits::small());
    rig.completed(PREPARE);
    rig.orders();
    rig.terminate();
    assert_eq!(&*rig.orders(), &[]);
    rig.started(FORGEJO);
    assert_eq!(&*rig.orders(), &[Order::Signal { step: FORGEJO }]);
}
#[test]
fn a_second_termination_signal_closes_every_child_at_once() {
    let mut rig = Rig::running();
    rig.terminate();
    rig.orders();
    rig.terminate();
    assert_eq!(
        &*rig.orders(),
        &[Order::Close { step: FORGEJO }, Order::Close { step: RUNNER }, Order::Close { step: SERVE }]
    );
    for step in [FORGEJO, RUNNER, SERVE] {
        rig.closed(step);
    }
    assert!(rig.machine.is_idle(), "forced stop leaves no child");
}

#[test]
fn a_long_step_refused_before_start_retries_and_continues_the_order() {
    let mut rig = Rig::new(&limits::small());
    rig.completed(PREPARE);
    rig.orders();
    rig.input(Input::NotStarted { step: FORGEJO });
    assert_eq!(&*rig.orders(), &[]);
    rig.advance(Duration::from_millis(500));
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: PREPARE }]);
    rig.completed(PREPARE);
    assert_eq!(&*rig.orders(), &[Order::Close { step: PREPARE }, Order::Spawn { step: FORGEJO }]);
    rig.started(FORGEJO);
    assert_eq!(&*rig.orders(), &[Order::Spawn { step: RUNNER }]);
}
