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
        assert_eq!(limits.sockets, count.saturating_mul(3).saturating_add(5));
        assert_eq!((limits.intake, limits.receive, limits.names), (LINE_BYTES, 1, 0));
        assert_eq!((limits.sends, limits.accepts, limits.backlog), (1, 1, 1));
        assert_eq!(limits.close_timeout, Duration::from_secs(5));
        assert_eq!(limits.retry, Duration::from_millis(10));
    }
}

fn grouped_steps() -> temper_forge_setup::Steps {
    use temper_forge_setup::{Group, Stage, Steps};
    Steps {
        always: Box::new([long(b"serve", &[b"never"])]),
        asked: Box::new([Group {
            forge: 3,
            before: 0,
            steps: Box::new([
                (Stage::Prepare, once(b"prepare 3", &[b"exit", b"0"])),
                (Stage::Run, long(b"forgejo 3", &[b"never"])),
            ]),
        }]),
    }
}
#[test]
fn serves_pipe_lines_are_bounded_named_and_answered() {
    let mut too_long = List::with_capacity(257);
    for _ in 0_u32..256 {
        too_long.push(b'x').unwrap();
    }
    too_long.push(b'\n').unwrap();
    for (name, input, expected, diagnostic) in [
        ("ask", b"start 3\n".as_slice(), b"step 3 prepare\nstep 3 run\nstarted 3\n".as_slice(), b"".as_slice()),
        ("unknown forge", b"start 99\n", b"failed 99 not asked\n", b""),
        ("unknown line", b"secret unknown\n", b"", b"box: serve: unknown pipe line\n"),
        ("too long", too_long.as_slice(), b"", b"box: serve: pipe line too long\n"),
    ] {
        let mut harness = Harness::grouped(grouped_steps(), &[], true);
        harness.run_until_idle();
        assert_eq!(harness.launches.len(), 1, "{name}: dormant until asked");
        assert_eq!(
            harness.launches.get(0).unwrap().pipes.as_ref(),
            &[
                skein_io::kernel::Pipe { child: 1, way: skein_io::kernel::Way::Out, parent: None },
                skein_io::kernel::Pipe { child: 2, way: skein_io::kernel::Way::Out, parent: None },
                skein_io::kernel::Pipe { child: 3, way: skein_io::kernel::Way::Out, parent: None },
                skein_io::kernel::Pipe { child: 4, way: skein_io::kernel::Way::In, parent: None },
            ]
        );
        harness.ask(input);
        harness.run_until_idle();
        assert_eq!(&*harness.reply_bytes(), expected, "{name}");
        assert_eq!(&*harness.stderr(), diagnostic, "{name}");
        if !diagnostic.is_empty() {
            harness.ask(b"start 99\n");
            harness.run_until_idle();
            assert_eq!(&*harness.reply_bytes(), b"failed 99 not asked\n", "{name}: next line recovered");
        }
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}
#[test]
fn records_are_probed_before_start_and_written_private_or_reported() {
    use skein_fake_machine::{How, Item};
    let present =
        [Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/3", b"").mode(0o600)];
    let unreadable =
        [Item::directory(b"box"), Item::directory(b"box/started"), Item::file(b"box/started/3", b"").mode(0)];
    let blocked = [Item::file(b"box", b"kept")];
    let denied = [Item::directory(b"box"), Item::directory(b"box/started").mode(0o500)];
    for (name, items, recorded, writable, why) in [
        ("absent", &[][..], false, true, b"".as_slice()),
        ("present", present.as_slice(), true, true, b""),
        ("unreadable", unreadable.as_slice(), false, true, b"permission denied"),
        ("directory blocked", blocked.as_slice(), false, false, b"not a directory"),
        ("create denied", denied.as_slice(), false, false, b""),
    ] {
        let mut harness = Harness::grouped(grouped_steps(), items, true);
        harness.run_until_idle();
        assert_eq!(harness.spawn_times(b"prepare 3").len(), usize::from(recorded), "{name}: startup record");
        let diagnostics = harness.stderr();
        if !why.is_empty() {
            assert!(Harness::contains(&diagnostics, b"box/started/3"), "{name}");
            assert!(Harness::contains(&diagnostics, why), "{name}");
        }
        harness.ask(b"start 3\n");
        harness.run_until_idle();
        let replies = harness.reply_bytes();
        if recorded {
            assert_eq!(&*replies, b"started 3\n");
        } else if writable {
            assert_eq!(&*replies, b"step 3 prepare\nstep 3 run\nstarted 3\n");
        } else {
            assert_eq!(&*replies, b"step 3 prepare\nfailed 3 record not written\n");
            assert!(Harness::contains(&harness.stderr(), b"box/started/3"));
        }
        if writable && name != "unreadable" {
            let root = harness.machine.reopen_root(0);
            let file = harness.machine.open(root, b"box/started/3", How::Read).unwrap();
            assert_eq!(harness.machine.stat(file).mode & 0o777, 0o600, "{name}: private record");
            assert!(harness.machine.read(file, 0, 1).unwrap().is_empty());
            harness.machine.close(file);
            harness.machine.close(root);
        }
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
        assert_eq!(harness.machine.open_handles(), 0, "{name}: every opened record closed");
    }
}
#[test]
fn an_exited_serve_loses_its_answers_and_gets_fresh_pipes() {
    let mut harness = Harness::grouped(grouped_steps(), &[], true);
    harness.run_until_idle();
    let old = harness.serve_index();
    let child = harness.hosted.get(old).unwrap();
    let old_output = child.output.unwrap();
    let old_input = child.input.unwrap();
    let pid = child.pid;
    harness.hold_prepare = true;
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    assert!(harness.held_prepare.is_some());
    harness.sim.finish_service(pid, Exit::Code(1));
    harness.run_until_idle();
    harness.replies.clear();
    harness.release_prepare();
    harness.run_until_idle();
    assert!(harness.replies.is_empty(), "answers to the exited serve are dropped");
    harness.sim.advance_to(harness.sim.now().saturating_add(Duration::from_millis(500)));
    harness.run_until_idle();
    assert_eq!(harness.spawn_times(b"serve").len(), 2);
    let fresh = harness.hosted.get(harness.serve_index()).unwrap();
    assert_ne!(fresh.pid, pid);
    assert!(fresh.output.is_some() && fresh.input.is_some());
    assert_ne!(harness.control_fds.get(0), harness.control_fds.get(1));
    assert!(old_output.raw() > 0_i32 && old_input.raw() > 0_i32);
    assert!(harness.replies.is_empty(), "old pending answers were dropped");
    harness.ask(b"start 3\n");
    harness.run_until_idle();
    assert_eq!(&*harness.reply_bytes(), b"started 3\n");
    harness.terminate();
    assert_eq!(harness.run_to_end(), Exit::Code(0));
    harness.sim.assert_quiescent(harness.pid);
    harness.sim.assert_no_open_fds(harness.pid);
}

#[test]
fn a_serve_that_does_not_read_holds_the_next_ask_until_replies_are_written() {
    for buffer in [8, 16] {
        let mut harness = Harness::buffered(grouped_steps(), &[], true, buffer);
        harness.run_until_idle();
        harness.ask(b"start 3\n");
        harness.run_until_idle();
        harness.ask(b"start 4\n");
        harness.run_until_idle();
        assert!(!Harness::contains(&harness.reply_bytes(), b"failed 4"), "buffer {buffer}: next ask waits");
        let expected = b"step 3 prepare\nstep 3 run\nstarted 3\nfailed 4 not asked\n";
        let mut received = List::with_capacity(u32::try_from(expected.len()).unwrap());
        for _ in 0..expected.len() {
            if usize::try_from(received.len()).unwrap() == expected.len() {
                break;
            }
            let remaining = u32::try_from(expected.len()).unwrap().saturating_sub(received.len());
            for byte in harness.read_reply(remaining) {
                received.push(byte).unwrap();
            }
            harness.run_until_idle();
        }
        assert_eq!(received.as_slice(), expected, "buffer {buffer}: replies stay in order");
        harness.terminate();
        assert_eq!(harness.run_to_end(), Exit::Code(0));
        harness.sim.assert_quiescent(harness.pid);
        harness.sim.assert_no_open_fds(harness.pid);
    }
}
