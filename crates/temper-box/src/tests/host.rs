use super::fixtures::{Harness, long, once};
use crate::host::{LINE_BYTES, io_limits};
use alloc::boxed::Box;
use skein_io::kernel::{Done, Exit, Target};
use skein_lib::{Duration, List, bytes};
use skein_sim::{Event, Summary};

#[test]
fn the_steps_start_in_order_and_a_stop_leaves_no_child() {
    let steps = [
        once(b"prepare", &[b"exit", b"0"]),
        long(b"forgejo", &[b"never"]),
        long(b"runner", &[b"never"]),
        once(b"reconcile-1", &[b"exit", b"0"]),
        once(b"reconcile-2", &[b"exit", b"0"]),
        long(b"serve", &[b"never"]),
    ];
    let mut harness = Harness::new(&steps);
    harness.run_until_idle();
    let mut names = List::with_capacity(6);
    for launch in &harness.launches {
        names.push(launch.name.clone()).expect("six launches");
    }
    let expected: Box<[Box<[u8]>]> = Box::new([
        bytes::copy_of(b"prepare"),
        bytes::copy_of(b"forgejo"),
        bytes::copy_of(b"runner"),
        bytes::copy_of(b"reconcile-1"),
        bytes::copy_of(b"reconcile-2"),
        bytes::copy_of(b"serve"),
    ]);
    assert_eq!(names.as_slice(), expected.as_ref(), "the setup's step order");
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0), "a requested stop succeeds");
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
#[test]
fn a_stop_signals_each_long_step_twice_last_first() {
    let steps = [long(b"forgejo", &[b"never"]), long(b"runner", &[b"never"]), long(b"serve", &[b"never"])];
    let mut harness = Harness::with_stubborn(&steps, true);
    harness.run_until_idle();
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0), "a stop ultimately closes stubborn children");
    let mut children = List::with_capacity(3);
    let mut signals = List::with_capacity(6);
    for entry in harness.sim.trace() {
        if entry.pid == harness.pid {
            if let Event::Complete { result: Ok(Done::Spawned { pidfd }), .. } = &entry.event {
                children.push(*pidfd).expect("three children");
            }
            if let Event::Submit { kind: Summary::Signal { pidfd, signal: 15, to: Target::Child }, .. } = &entry.event {
                signals.push((*pidfd, entry.at)).expect("two termination signals per child");
            }
        }
    }
    assert_eq!(signals.len(), 6, "each stubborn child is signalled twice");
    for (index, child) in children.iter().rev().enumerate() {
        let start = u32::try_from(index).expect("three children").saturating_mul(2);
        let first = signals.get(start).expect("first signal");
        let second = signals.get(start.saturating_add(1)).expect("second signal");
        assert_eq!((first.0, second.0), (*child, *child), "last child first");
        assert!(second.1.saturating_since(first.1) >= Duration::from_secs(10), "a grace between signals");
    }
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}
#[test]
fn a_long_step_that_keeps_exiting_is_given_up_naming_it() {
    let steps = [once(b"prepare", &[b"exit", b"0"]), long(b"forgejo", &[b"exit", b"1"])];
    let mut harness = Harness::new(&steps);
    assert_eq!(harness.run_to_end(), Exit::Code(1));
    let spawns = harness.spawn_times(b"forgejo");
    assert_eq!(spawns.len(), 6, "the first run and five restarts");
    for (pair, gap) in spawns.windows(2).zip([500, 1_000, 2_000, 4_000, 8_000]) {
        assert!(pair[1].saturating_since(pair[0]) >= Duration::from_millis(gap), "the backoff doubles");
    }
    assert_eq!(harness.spawn_times(b"prepare").len(), 6, "each restart runs prepare again");
    assert!(Harness::contains(&harness.stderr(), b"box: forgejo gave up after 5 restarts\n"), "give-up names step");
}
#[test]
fn a_once_step_that_fails_ends_the_box_with_exit_one() {
    let steps = [once(b"prepare", &[b"exit", b"1"]), long(b"forgejo", &[b"never"])];
    let mut harness = Harness::new(&steps);
    assert_eq!(harness.run_to_end(), Exit::Code(1));
    assert_eq!(&*harness.stderr(), b"box: prepare gave up after 0 restarts\n");
    assert_eq!(harness.spawn_times(b"forgejo").len(), 0, "failure never starts the next step");
}
#[test]
fn every_step_is_spawned_with_the_environment_the_box_was_given() {
    let steps = [once(b"prepare", &[b"exit", b"0"]), long(b"serve", &[b"never"])];
    let mut harness = Harness::new(&steps);
    harness.run_until_idle();
    let expected: Box<[Box<[u8]>]> = Box::new([bytes::copy_of(b"HOME=/box"), bytes::copy_of(b"TOKEN=whole=value")]);
    assert_eq!(harness.launches.len(), 2, "both steps ran");
    for launch in &harness.launches {
        assert_eq!(launch.environment, expected, "every step gets the whole environment");
    }
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
}
#[test]
fn the_io_limits_are_usable_for_one_six_and_sixty_four_steps() {
    for count in [1_u32, 6, 64] {
        let limits = io_limits(count);
        assert!(limits.is_usable(), "usable for {count} steps");
        assert_eq!(limits.sockets, count.saturating_mul(3).saturating_add(3));
        assert_eq!((limits.intake, limits.receive, limits.names), (LINE_BYTES, 1, 0));
        assert_eq!((limits.sends, limits.accepts, limits.backlog), (1, 1, 1));
        assert_eq!(limits.close_timeout, Duration::from_secs(5));
        assert_eq!(limits.retry, Duration::from_millis(10));
    }
}
