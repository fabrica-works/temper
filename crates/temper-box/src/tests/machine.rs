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

#[test]
#[expect(clippy::too_many_lines, reason = "protocol/setup.md 13.1 group behaviours share one transition table")]
fn asked_groups_follow_their_chains_records_failures_and_stop() {
    use super::fixtures::step;
    use crate::layout::{self, Asked, Placed};
    use crate::machine::{Input, Machine, Order, Said, Why};
    use skein_lib::{Queue, Time};
    fn append(prefix: &[(Action, Box<[Order]>)], rows: &[(Action, &[Order])]) -> Box<[(Action, Box<[Order]>)]> {
        let mut script = skein_lib::List::with_capacity(u32::try_from(prefix.len() + rows.len()).unwrap());
        for row in prefix {
            script.push(row.clone()).unwrap();
        }
        for (action, expected) in rows {
            script.push((*action, Box::from(*expected))).unwrap();
        }
        script.into_boxed()
    }
    use temper_forge_setup::{Group, Stage, Steps};
    #[derive(Clone, Copy, Debug)]
    enum Action {
        Input(Input),
        Fire(u64),
    }
    use Action::{Fire as F, Input as I};
    use Input::{Ask, Closed, Exited, NotStarted, NotWritten, Started, Terminate, Written};
    use Order::{Close, Record, Say, Signal, Spawn};
    fn placed() -> Box<[Placed]> {
        layout::flatten(Steps {
            always: Box::new([step(b"anchor", true), step(b"serve", true)]),
            asked: Box::new([
                Group {
                    forge: 3,
                    before: 1,
                    steps: Box::new([
                        (Stage::Fetch, step(b"fetch 3", false)),
                        (Stage::Prepare, step(b"prepare 3", false)),
                        (Stage::Run, step(b"forgejo 3", true)),
                        (Stage::Run, step(b"runner 3", true)),
                    ]),
                },
                Group {
                    forge: 4,
                    before: 2,
                    steps: Box::new([
                        (Stage::Prepare, step(b"prepare 4", false)),
                        (Stage::Run, step(b"forgejo 4", true)),
                    ]),
                },
            ]),
        })
    }
    fn progress(step: usize) -> Order {
        Say { said: Said::Step { step } }
    }
    fn failed(forge: u16, why: Why) -> Order {
        Say { said: Said::Failed { forge, why } }
    }
    let begin = append(&[], &[(I(Ask { forge: 3 }), &[Spawn { step: 1 }, progress(1)]), (I(Started { step: 1 }), &[])]);
    let mut prepared = begin.clone();
    prepared = append(
        &prepared,
        &[
            (I(Exited { step: 1, exit: Exit::Code(0) }), &[Close { step: 1 }]),
            (I(Closed { step: 1 }), &[Spawn { step: 2 }, progress(2)]),
            (I(Started { step: 2 }), &[]),
            (I(Exited { step: 2, exit: Exit::Code(0) }), &[Close { step: 2 }]),
            (I(Closed { step: 2 }), &[Record { forge: 3 }]),
        ],
    );
    let mut running = prepared.clone();
    running = append(
        &running,
        &[
            (I(Written { forge: 3 }), &[Spawn { step: 3 }, progress(3)]),
            (I(Started { step: 3 }), &[Spawn { step: 4 }, progress(4)]),
            (I(Started { step: 4 }), &[Say { said: Said::Started { forge: 3 } }]),
        ],
    );
    let mut cases = skein_lib::List::with_capacity(16);
    cases
        .push((
            "dormant, unknown and restart across it",
            append(
                &[],
                &[
                    (I(Ask { forge: 99 }), &[failed(99, Why::NotAsked)]),
                    (I(Exited { step: 5, exit: Exit::Code(1) }), &[Close { step: 5 }]),
                    (I(Closed { step: 5 }), &[]),
                    (F(500), &[Spawn { step: 5 }]),
                ],
            ),
        ))
        .unwrap();
    let mut busy = begin.clone();
    busy = append(&busy, &[(I(Ask { forge: 3 }), &[]), (I(Ask { forge: 4 }), &[failed(4, Why::Busy)])]);
    cases.push(("busy and same starting forge", busy)).unwrap();
    let mut recorded = prepared.clone();
    recorded = append(
        &recorded,
        &[
            (I(Ask { forge: 3 }), &[]),
            (I(Ask { forge: 4 }), &[failed(4, Why::Busy)]),
            (I(NotWritten { forge: 3 }), &[failed(3, Why::NotWritten)]),
            (I(Ask { forge: 3 }), &[Spawn { step: 1 }, progress(1)]),
        ],
    );
    cases.push(("record failure and reask", recorded)).unwrap();
    let mut once_failed = begin.clone();
    once_failed = append(
        &once_failed,
        &[
            (I(Exited { step: 1, exit: Exit::Code(7) }), &[Close { step: 1 }]),
            (I(Ask { forge: 4 }), &[failed(4, Why::Busy)]),
            (I(Ask { forge: 3 }), &[]),
            (I(Closed { step: 1 }), &[failed(3, Why::Exited { step: 1, exit: Exit::Code(7) })]),
            (I(Ask { forge: 3 }), &[Spawn { step: 1 }, progress(1)]),
        ],
    );
    cases.push(("once failure closes then dormant", once_failed)).unwrap();
    cases
        .push((
            "spawn refused",
            append(
                &[],
                &[
                    (I(Ask { forge: 3 }), &[Spawn { step: 1 }, progress(1)]),
                    (I(NotStarted { step: 1 }), &[failed(3, Why::NotStarted { step: 1 })]),
                ],
            ),
        ))
        .unwrap();
    let mut long_failed = prepared.clone();
    long_failed = append(
        &long_failed,
        &[
            (I(Written { forge: 3 }), &[Spawn { step: 3 }, progress(3)]),
            (I(Started { step: 3 }), &[Spawn { step: 4 }, progress(4)]),
            (I(Exited { step: 3, exit: Exit::Signal(9) }), &[Close { step: 3 }]),
            (I(Closed { step: 3 }), &[]),
            (I(Started { step: 4 }), &[Close { step: 4 }]),
            (I(Closed { step: 4 }), &[failed(3, Why::Exited { step: 3, exit: Exit::Signal(9) })]),
            (F(1000), &[]),
        ],
    );
    cases.push(("long failure closes spawn in flight", long_failed)).unwrap();
    let mut restart = running.clone();
    restart = append(
        &restart,
        &[
            (I(Ask { forge: 3 }), &[Say { said: Said::Started { forge: 3 } }]),
            (I(Exited { step: 3, exit: Exit::Code(0) }), &[Close { step: 3 }]),
            (I(Closed { step: 3 }), &[]),
            (F(500), &[Spawn { step: 1 }]),
        ],
    );
    cases.push(("started answers and restarts its own preparation", restart)).unwrap();
    let mut stop = begin.clone();
    stop = append(
        &stop,
        &[
            (I(Terminate), &[Signal { step: 5 }]),
            (I(Ask { forge: 3 }), &[failed(3, Why::Stopping)]),
            (I(Exited { step: 1, exit: Exit::Code(1) }), &[Close { step: 1 }]),
            (I(Closed { step: 1 }), &[]),
        ],
    );
    cases.push(("stop suppresses group replies", stop)).unwrap();
    let mut give_up = running.clone();
    give_up = append(
        &give_up,
        &[(I(Exited { step: 5, exit: Exit::Code(1) }), &[Close { step: 5 }]), (I(Closed { step: 5 }), &[])],
    );
    cases.push(("started long is subject to restart policy", give_up)).unwrap();
    for (name, script) in cases.into_boxed() {
        let mut machine = Machine::new(&limits::small(), &placed()).unwrap();
        let mut out = Queue::with_capacity(128);
        machine.start(Time::ZERO, &[], &mut out);
        assert_eq!(out.pop(), Some(Spawn { step: 0 }), "{name}");
        assert!(out.is_empty());
        machine.input(Time::ZERO, Started { step: 0 }, &mut out);
        assert_eq!(out.pop(), Some(Spawn { step: 5 }), "{name}: dormant skipped");
        machine.input(Time::ZERO, Started { step: 5 }, &mut out);
        for (action, expected) in script {
            match action {
                I(input) => machine.input(Time::ZERO, input, &mut out),
                F(ms) => machine.fire(Time::ZERO.saturating_add(Duration::from_millis(ms)), &mut out),
            }
            let mut got = skein_lib::List::with_capacity(out.len());
            for _ in 0..out.len() {
                got.push(out.pop().unwrap()).unwrap();
            }
            assert_eq!(got.as_slice(), expected.as_ref(), "{name}");
        }
    }
    // The record barrier also works when there is no preparation, or no long step.
    for long in [false, true] {
        let placed = layout::flatten(Steps {
            always: Box::new([]),
            asked: Box::new([Group {
                forge: 9,
                before: 0,
                steps: Box::new([(if long { Stage::Run } else { Stage::Fetch }, step(b"only", long))]),
            }]),
        });
        let mut machine = Machine::new(&limits::small(), &placed).unwrap();
        let mut out = Queue::with_capacity(32);
        machine.start(Time::ZERO, &[], &mut out);
        assert!(out.is_empty());
        machine.input(Time::ZERO, Ask { forge: 9 }, &mut out);
        if !long {
            assert_eq!(out.pop(), Some(Spawn { step: 0 }));
            assert_eq!(out.pop(), Some(progress(0)));
            machine.input(Time::ZERO, Started { step: 0 }, &mut out);
            machine.input(Time::ZERO, Exited { step: 0, exit: Exit::Code(0) }, &mut out);
            assert_eq!(out.pop(), Some(Close { step: 0 }));
            machine.input(Time::ZERO, Closed { step: 0 }, &mut out);
        }
        assert_eq!(out.pop(), Some(Record { forge: 9 }));
        assert!(out.is_empty());
        machine.input(Time::ZERO, Written { forge: 9 }, &mut out);
        if long {
            assert_eq!(out.pop(), Some(Spawn { step: 0 }));
            assert_eq!(out.pop(), Some(progress(0)));
            machine.input(Time::ZERO, Started { step: 0 }, &mut out);
        }
        assert_eq!(out.pop(), Some(Say { said: Said::Started { forge: 9 } }));
        assert!(out.is_empty());
    }
    // A record joins the startup chain, with no ask or record barrier.
    let mut machine = Machine::new(&limits::small(), &placed()).unwrap();
    let mut out = Queue::with_capacity(128);
    machine.start(Time::ZERO, &[3], &mut out);
    out.pop();
    machine.input(Time::ZERO, Started { step: 0 }, &mut out);
    assert_eq!(out.pop(), Some(Spawn { step: 1 }));
    for (step, next) in [(1, 2), (2, 3)] {
        machine.input(Time::ZERO, Started { step }, &mut out);
        machine.input(Time::ZERO, Exited { step, exit: Exit::Code(0) }, &mut out);
        out.pop();
        machine.input(Time::ZERO, Closed { step }, &mut out);
        assert_eq!(out.pop(), Some(Spawn { step: next }));
    }
    machine.input(Time::ZERO, Started { step: 3 }, &mut out);
    assert_eq!(out.pop(), Some(Spawn { step: 4 }));
    machine.input(Time::ZERO, Started { step: 4 }, &mut out);
    assert_eq!(out.pop(), Some(Spawn { step: 5 }));
    assert!(out.is_empty());
    // Flatten includes end placements, and preserves the stage on every item.
    let placed = placed();
    assert_eq!(placed.len(), 8);
    assert_eq!(placed[6].asked, Some(Asked { forge: 4, stage: Stage::Prepare }));
}
