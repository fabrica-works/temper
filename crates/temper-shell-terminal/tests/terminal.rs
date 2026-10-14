mod support;
use skein_lib::{Duration, Time};
use temper_shell_terminal::terminal::Notice;

#[test]
fn a_change_within_the_interval_prints_once_when_it_ends() {
    let mut world = support::World::signed_in();
    world.client.change_at(Duration::from_millis(200));
    world.client.change_at(Duration::from_millis(700));
    world.run_until(Duration::from_millis(1500));
    assert_eq!(world.prints_at(), vec![Duration::ZERO, Duration::from_secs(1)]);
    assert!(world.shown_last(), "latest tree printed");
    assert_eq!(world.terminal.next_deadline(), None, "settled tree has no print timer");
}
#[test]
fn an_act_prints_at_once_after_the_next_build() {
    let mut world = support::World::signed_in();
    world.run_until(Duration::from_millis(100));
    world.type_bytes(b"4\n");
    assert_eq!(world.prints_at(), vec![Duration::ZERO], "waits for next build");
    world.run_until(Duration::from_millis(110));
    assert_eq!(world.prints_at(), vec![Duration::ZERO, Duration::from_millis(110)]);
    assert_eq!(world.client.acts(), &[4]);
}
#[test]
fn a_due_print_waits_for_stdout_then_shows_the_tree_then() {
    let mut world = support::World::signed_in();
    world.hold_stdout();
    world.type_bytes(b"p\n");
    world.client.change_at(Duration::from_millis(200));
    world.client.change_at(Duration::from_millis(1200));
    world.run_until(Duration::from_millis(1500));
    assert_eq!(world.prints_at(), vec![Duration::ZERO, Duration::ZERO], "old output stays in flight");
    world.stdout_takes();
    assert_eq!(world.prints_at(), vec![Duration::ZERO, Duration::ZERO, Duration::from_millis(1500)]);
    assert!(world.shown_last(), "waited print uses latest tree");
}
#[test]
fn a_command_cut_at_any_byte_acts_once() {
    for split in 0..=2 {
        let mut world = support::World::signed_in();
        world.type_bytes(&b"4\n"[..split]);
        world.type_bytes(&b"4\n"[split..]);
        world.run_until(Duration::from_millis(10));
        assert_eq!(world.client.acts(), &[4], "split {split}");
    }
}
#[test]
fn the_end_of_stdin_closes_and_exits_zero_after_the_drain() {
    for (name, command) in [("eof", None), ("q", Some(b"q\n".as_slice()))] {
        let mut world = support::World::signed_in();
        world.hold_stdout();
        world.type_bytes(b"p\n");
        if let Some(command) = command {
            world.type_bytes(command);
        } else {
            world.terminal.close(&mut world.client);
        }
        assert!(world.client.closed, "{name}: client closes");
        assert_eq!(world.terminal.exit(), None, "{name}: output drains first");
        world.stdout_takes();
        assert_eq!(world.terminal.exit(), Some((0, None)), "{name}: successful drain");
    }
}
#[test]
fn an_interrupt_closes_without_signing_out() {
    let mut world = support::World::signed_in();
    world.terminal.close(&mut world.client);
    world.stdout_takes();
    assert!(world.client.closed);
    assert!(world.client.acts().is_empty(), "closing offers no sign-out act");
    assert_eq!(world.terminal.exit(), Some((0, None)));
}
#[test]
fn signing_in_without_an_answer_tries_again_after_its_backoff() {
    let mut world = support::World::signing_in();
    for (notice_at, retry_at) in
        [(0, 250), (250, 750), (750, 1750), (1750, 3750), (3750, 7750), (7750, 15750), (15750, 23750)]
    {
        world.client.notice_at_once(Notice::SignInAgain);
        world.terminal.iterate(Time::ZERO.saturating_add(Duration::from_millis(notice_at)), &mut world.client);
        world.run_until(Duration::from_millis(retry_at - 10));
        assert!(
            world
                .client
                .sign_ins
                .last()
                .is_none_or(|time| *time < Time::ZERO.saturating_add(Duration::from_millis(retry_at))),
            "retry at {retry_at}: waits"
        );
        world.run_until(Duration::from_millis(retry_at));
        assert_eq!(world.client.sign_ins.last(), Some(&world.now), "retry at {retry_at}");
    }
    assert_eq!(world.last_output(), b"temper: signing in: no answer from the engine; trying again\n");
}
#[test]
fn a_refused_link_exits_one_with_its_line() {
    let mut world = support::World::signing_in();
    world.client.notice_at_once(Notice::SignInRefused);
    world.terminal.iterate(Time::ZERO, &mut world.client);
    assert!(world.client.closed);
    assert_eq!(
        world.terminal.exit(),
        Some((1, Some(Box::from(*b"temper open: the link was refused: use the fresh link temper serve printed\n"))))
    );
}
#[test]
fn a_credential_and_store_notice_print_once_in_order() {
    let mut world = support::World::signed_in();
    world.client.notice_at_once(Notice::Credential { party: 7, token: skein_lib::Secret::new(Box::from(*b"abc")) });
    world.client.notice_at_once(Notice::StoreFailed);
    world.run_until(Duration::from_millis(30));
    assert_eq!(
        world.shell_lines(),
        vec![
            Box::<[u8]>::from(*b"temper: credential for service 7, shown once: YWJj\n"),
            Box::from(*b"temper: cannot write the small store in client\n"),
        ]
    );
}
#[test]
fn signing_in_cancels_the_pending_link_retry() {
    let mut world = support::World::signing_in();
    world.client.notice_at_once(Notice::SignInAgain);
    world.run_until(Duration::from_millis(10));
    world.client.notice_at_once(Notice::SignedIn);
    world.run_until(Duration::from_secs(1));
    assert!(world.client.sign_ins.is_empty(), "no link exchange after signed in");
    assert_eq!(world.terminal.next_deadline(), None, "link retry removed");
}
#[test]
fn a_full_shell_queue_preserves_the_unconsumed_input() {
    let mut world = support::World::signed_in();
    let input = b"x\n".repeat(80);
    world.type_bytes(&input);
    world.run_until(Duration::from_secs(1));
    assert_eq!(world.shell_lines(), vec![Box::<[u8]>::from(*b"temper: commands: n, n words, p, q\n"); 80]);
}
