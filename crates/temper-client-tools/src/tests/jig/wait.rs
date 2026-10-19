use crate::tests::fixtures as f;
use jig_client::{Watchable, contract as c};
use skein_lib::Time;
#[test]
fn a_wait_until_replied_answers_at_the_parked_delivery_with_its_digest() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
    let parked = f::parked();
    let expected = f::subject_json(&crate::jig::render::Part::delivery(&parked).expect("task form"));
    root.deliver_subject(&f::task_watch(), parked);
    let answer = root.answer(call).expect("met at the parked delivery");
    assert_eq!(answer.status, 0);
    assert!(f::json_has(&answer, &expected));
    assert!(answer.stdout.ends_with(b",\"met\":true}"));
}
#[test]
fn a_read_matches_a_wait_digest() {
    let mut root = f::signed_in(f::PARTY);
    for row in f::wait_cases(&mut root) {
        let digest = row.read_digest;
        let call = root.wait_since(row.argv, &[], &digest);
        root.deliver_subject(&row.watch, row.delivery.clone());
        assert!(root.answer(call).is_none(), "unchanged {:?}", row.argv);
        let changed = f::changed_delivery(row.delivery);
        root.deliver_subject(&row.watch, changed);
        let answer = root.answer(call).expect("changed subject answers");
        assert_eq!(answer.status, 0, "{:?}", row.argv);
        assert!(!f::json_has(&answer, &digest), "{:?}", row.argv);
    }
}
#[test]
fn since_compares_the_first_delivery() {
    let mut root = f::signed_in(f::PARTY);
    let parked = f::parked();
    let different = f::task_digest(&f::writing_chat()).hex();
    let call = root.wait_since(f::WAIT_TASK, &[], &different);
    root.deliver_subject(&f::task_watch(), parked);
    assert_eq!(root.answer(call).expect("first differs").status, 0);
}
#[test]
fn an_unqualified_wait_uses_its_first_delivery_as_the_baseline() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &[]);
    root.deliver_subject(&f::task_watch(), f::parked());
    assert!(root.answer(call).is_none());
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert_eq!(root.answer(call).expect("first change").status, 0);
}
#[test]
fn both_requirements_must_hold_on_the_same_delivery() {
    let parked = f::parked();
    let mut root = f::signed_in(f::PARTY);
    let call = root.wait_since(f::WAIT_TASK, f::UNTIL_REPLIED, &f::task_digest(&parked).hex());
    for delivery in [parked.clone(), f::writing_chat(), parked.clone()] {
        root.deliver_subject(&f::task_watch(), delivery);
        assert!(root.answer(call).is_none());
    }
    root.deliver_subject(&f::task_watch(), f::changed_delivery(parked));
    assert_eq!(root.answer(call).expect("both hold now").status, 0);
}
#[test]
fn timeout_exits_3_with_the_last_subject() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::SHORT_TIMEOUT);
    let parked = f::parked();
    let expected = f::subject_json(&crate::jig::render::Part::delivery(&parked).expect("task form"));
    root.deliver_subject(&f::task_watch(), parked);
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("deadline");
    root.fire();
    let answer = root.answer(call).expect("timed out");
    assert_eq!(answer.status, 3);
    assert!(f::json_has(&answer, &expected));
    assert!(f::json_has(&answer, b"\"why\":\"timeout\",\"met\":false"));
}
#[test]
fn a_timeout_does_not_fire_early() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::SHORT_TIMEOUT);
    assert_eq!(crate::root::next_deadline(&root.root), Time::ZERO.checked_add(f::WAIT_SPAN));
    root.fire();
    assert!(root.answer(call).is_none());
}
#[test]
fn a_me_change_rechecks_the_task_without_changing_its_digest() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_WAITING_ON_ME);
    let task = f::goal(
        f::TASK,
        f::party_wait(c::Whom::Role { project: f::PROJECT, role: f::POLICY_ROLE }, c::WaitedFor::Question),
    );
    let delivery = c::Delivery::Task(Box::new(task));
    let digest = f::task_digest(&delivery).hex();
    root.deliver_subject(&f::task_watch(), delivery);
    assert!(root.answer(call).is_none());
    let mut me = f::me(f::PARTY);
    me.projects.get_mut(0).expect("project").role = c::HeldRole::Policy { role: f::POLICY_ROLE };
    root.deliver(c::Delivery::Me(Box::new(me)));
    let answer = root.answer(call).expect("role now holds");
    assert_eq!(answer.status, 0);
    assert!(f::json_has(&answer, &digest));
}
#[test]
fn a_meeting_delivery_wins_over_timeout_in_the_same_iteration() {
    let flags = f::extended(f::SHORT_TIMEOUT, f::UNTIL_REPLIED);
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, flags.as_slice());
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("deadline");
    root.deliver_subject(&f::task_watch(), f::parked());
    root.fire();
    assert_eq!(root.answer(call).expect("delivery won").status, 0);
}
#[test]
fn a_wait_keeps_its_baseline_while_behind() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &[]);
    root.deliver_subject(&f::task_watch(), f::parked());
    root.watch_event(&f::task_watch(), c::WatchEvent::Behind);
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    assert!(root.answer(call).is_none());
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert_eq!(root.answer(call).expect("new live delivery").status, 0);
}
#[test]
fn coalesced_deliveries_answer_from_the_current_document() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert!(root.answer(call).is_none());
    let changed = f::changed_delivery(f::parked());
    let expected = f::subject_json(&crate::jig::render::Part::delivery(&changed).expect("task"));
    root.deliver_subject(&f::task_watch(), changed);
    assert!(f::json_has(&root.answer(call).expect("current parked document"), &expected));
}
#[test]
fn run_text_changes_are_deliveries() {
    let (mut root, call) = f::waiting_call(f::WAIT_RUN, &[]);
    root.deliver_run(f::no_provisional());
    root.deliver_run(f::provisional());
    let answer = root.answer(call).expect("text changed");
    assert_eq!(answer.status, 0);
    assert!(f::json_has(&answer, b"draft"));
    assert!(root.answer(call).is_none());
}
#[test]
fn an_answer_releases_only_the_calls_handle() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
    let flags = f::extended(f::WAIT_TASK, &["--until", "phase:running", "--json"]);
    let other = root.submit(flags.as_slice());
    root.deliver_subject(&f::task_watch(), f::parked());
    assert_eq!(root.answer(call).expect("first call met").status, 0);
    assert!(root.answer(other).is_none());
    assert_eq!(root.watches.len(), f::TWO_CALLERS);
    root.deliver_subject(&f::task_watch(), f::writing_chat());
    assert_eq!(root.answer(other).expect("second call met").status, 0);
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
    for subject in root.watches.values() {
        assert_eq!(*subject, Watchable::Jig(c::Subject::Me));
    }
}

#[test]
fn a_wait_text_uses_the_subjects_read_form() {
    let expected = f::read_answer(f::TASK_CALL, f::task_result(f::parked_task()));
    let (mut root, call) = f::plain_wait(f::UNTIL_REPLIED);
    root.deliver_subject(&f::task_watch(), f::parked());
    assert_eq!(root.answer(call).expect("text answer").stdout, f::appended(&expected.stdout, b"met\n"));
}
#[test]
fn a_timeout_text_names_why_it_was_not_met() {
    let expected = f::read_answer(f::TASK_CALL, f::task_result(f::parked_task()));
    let (mut root, call) = f::plain_wait(f::SHORT_TIMEOUT);
    root.deliver_subject(&f::task_watch(), f::parked());
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("deadline");
    root.fire();
    assert_eq!(root.answer(call).expect("timeout text").stdout, f::appended(&expected.stdout, b"not met: timeout\n"));
}

#[test]
fn a_task_watch_ending_reads_how_the_task_ended_before_answering() {
    let mut root = f::signed_in(f::PARTY);
    for phase in f::ENDED_PHASES {
        let argv = f::extended(f::WAIT_TASK, f::UNTIL_ENDED);
        let argv = f::extended(argv.as_slice(), &["--json"]);
        let call = root.submit(argv.as_slice());
        root.deliver_subject(&f::task_watch(), f::writing_chat());
        root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
        assert!(root.answer(call).is_none(), "{phase:?}: final read is pending");
        assert_eq!(root.reads.get(&call).expect("final read").1, c::Read::Task { task: f::TASK });
        let task = f::ended_task(*phase);
        let read = root.submit(f::TASK_JSON_CALL);
        root.serve(read, f::task_result(task.clone()));
        let expected = root.answer(read).expect("ended task read");
        root.serve(call, f::task_result(task));
        let answer = root.answer(call).expect("final task read");
        assert_eq!(answer.status, 0, "{phase:?}");
        assert_eq!(answer.stdout, f::met_json(&expected.stdout), "{phase:?}");
    }
}
#[test]
fn an_already_ended_delivery_reads_the_final_task() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_ENDED);
    let task = f::ended_task(c::Phase::Done);
    root.final_delivery(call, task.clone());
    assert!(root.answer(call).is_none());
    assert_eq!(crate::root::next_deadline(&root.root), None);
    root.serve(call, f::task_result(task));
    assert_eq!(root.answer(call).expect("final read").status, 0);
}
#[test]
fn a_task_ended_text_uses_the_final_reads_part() {
    let (mut root, call) = f::plain_wait(f::UNTIL_ENDED);
    root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
    let task = f::ended_task(c::Phase::Cancelled);
    let expected = f::read_answer(f::TASK_CALL, f::task_result(task.clone()));
    root.serve(call, f::task_result(task));
    assert_eq!(root.answer(call).expect("final text").stdout, f::appended(&expected.stdout, b"met\n"));
}
#[test]
fn a_run_or_tree_ending_meets_ended() {
    let mut root = f::signed_in(f::PARTY);
    for (argv, watch, delivery) in [
        (f::WAIT_RUN, f::run_watch(), f::no_provisional()),
        (f::WAIT_TREE, f::tree_watch(), c::Delivery::Tree(Box::new(f::composite_tree()))),
    ] {
        for initial in [None, Some(delivery)] {
            let call = root.wait_since(argv, f::UNTIL_ENDED, &f::task_digest(&f::parked()).hex());
            if let Some(delivery) = initial {
                root.deliver_subject(&watch, delivery);
            }
            assert!(root.answer(call).is_none());
            root.end(call, &watch, c::WatchEnd::SubjectEnded);
            let answer = root.answer(call).expect("ended meets despite since");
            assert_eq!(answer.status, 0, "{argv:?}");
            assert!(answer.stdout.ends_with(b",\"met\":true}"));
            assert!(root.reads.is_empty());
            assert_eq!(root.watches.len(), f::SHARED_WATCHES);
        }
    }
}
#[test]
fn a_subject_ending_unmet_keeps_its_last_digest() {
    let mut root = f::signed_in(f::PARTY);
    for flags in [&[][..], f::UNTIL_RUNNING] {
        let argv = f::extended(f::WAIT_TASK, flags);
        let argv = f::extended(argv.as_slice(), &["--json"]);
        let call = root.submit(argv.as_slice());
        let parked = f::parked();
        let expected = f::subject_json(&crate::jig::render::Part::delivery(&parked).expect("task"));
        root.deliver_subject(&f::task_watch(), parked);
        root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
        let answer = root.answer(call).expect("ended unmet");
        assert_eq!(answer.status, 3);
        assert!(f::json_has(&answer, &expected));
        assert!(answer.stdout.ends_with(b",\"why\":\"ended\",\"met\":false}"));
    }
}
#[test]
fn an_end_before_delivery_invents_no_document() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_REPLIED);
    root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
    let answer = root.answer(call).expect("ended without delivery");
    assert_eq!(answer.status, 3);
    assert_eq!(answer.stdout.as_ref(), b"{\"task\":null,\"why\":\"ended\",\"met\":false}");
}
#[test]
fn a_subject_no_longer_visible_exits_2() {
    let mut root = f::signed_in(f::PARTY);
    for (end, reason) in [(c::WatchEnd::Unknown, b"unknown".as_slice()), (c::WatchEnd::NotAllowed, b"not_allowed")] {
        let argv = f::extended(f::WAIT_TASK, &["--json"]);
        let call = root.submit(argv.as_slice());
        root.end(call, &f::task_watch(), end);
        let answer = root.answer(call).expect("refused");
        assert_eq!(answer.status, 2);
        assert!(skein_lib::bytes::find(&answer.stderr, reason).is_some());
        assert_eq!(crate::root::next_deadline(&root.root), None);
    }
}
#[test]
fn client_watch_failures_exit_1() {
    let mut root = f::signed_in(f::PARTY);
    for end in [c::WatchEnd::SignedOut, c::WatchEnd::NotServed] {
        let argv = f::extended(f::WAIT_TASK, &["--json"]);
        let call = root.submit(argv.as_slice());
        root.end(call, &f::task_watch(), end);
        assert_eq!(root.answer(call).expect("client failure").status, 1);
    }
}
#[test]
fn final_read_failures_keep_the_reads_status() {
    let mut root = f::signed_in(f::PARTY);
    for (result, status) in [
        (jig_client::Readout::Limit, 1),
        (jig_client::Readout::NotServed { family: None, served: None }, 1),
        (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut)), 1),
        (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)), 1),
        (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)), 2),
        (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)), 2),
    ] {
        let argv = f::extended(f::WAIT_TASK, f::UNTIL_ENDED);
        let call = root.submit(argv.as_slice());
        root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
        root.serve_readout(call, result);
        assert_eq!(root.answer(call).expect("final read failed").status, status);
        assert_eq!(crate::root::next_deadline(&root.root), None);
    }
}
#[test]
fn sign_out_of_required_me_ends_the_wait() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, f::UNTIL_WAITING_ON_ME);
    root.end(call, &Watchable::Jig(c::Subject::Me), c::WatchEnd::SignedOut);
    assert_eq!(root.answer(call).expect("me signed out").status, 1);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}
#[test]
fn a_stopping_engine_reopens_before_answering() {
    let mut root = f::signed_in(f::PARTY);
    for end in [c::WatchEnd::Stopping, c::WatchEnd::Dropped, c::WatchEnd::Busy] {
        let argv = f::extended(f::WAIT_TASK, f::UNTIL_REPLIED);
        let call = root.submit(argv.as_slice());
        root.deliver_subject(&f::task_watch(), f::writing_chat());
        root.end(call, &f::task_watch(), end);
        assert!(root.answer(call).is_none());
        root.now = crate::root::next_deadline(&root.root).expect("reopen deadline");
        root.fire();
        root.deliver_subject(&f::task_watch(), f::parked());
        assert_eq!(root.answer(call).expect("reopened parked task").status, 0);
    }
}
#[test]
fn a_finishing_read_releases_the_waits_timer() {
    let flags = f::extended(f::UNTIL_ENDED, f::SHORT_TIMEOUT);
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, flags.as_slice());
    root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
    root.now = Time::ZERO.checked_add(f::WAIT_SPAN).expect("old deadline");
    root.fire();
    assert!(root.answer(call).is_none());
    assert_eq!(crate::root::next_deadline(&root.root), None);
    root.serve(call, f::task_result(f::ended_task(c::Phase::Done)));
    assert_eq!(root.answer(call).expect("read after old timeout").status, 0);
}
#[test]
fn an_answered_wait_ignores_late_terminals() {
    let (mut root, call) = f::finishing_call();
    let token = root.reads.get(&call).expect("final read").0;
    root.serve(call, f::task_result(f::ended_task(c::Phase::Done)));
    assert_eq!(root.answer(call).expect("single answer").status, 0);
    root.client(jig_client::Event::Up {
        up: jig_client::Up::ReadDone { read: token, result: jig_client::Readout::Limit },
    });
    root.fire();
    assert!(root.answer(call).is_none());
    assert_eq!(root.watches.len(), f::SHARED_WATCHES);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}

#[test]
fn an_end_without_delivery_names_the_text_subject() {
    let mut root = f::signed_in(f::PARTY);
    let argv = f::extended(f::WAIT_RUN, f::UNTIL_ENDED);
    let call = root.submit(argv.as_slice());
    root.end(call, &f::run_watch(), c::WatchEnd::SubjectEnded);
    assert_eq!(root.answer(call).expect("ended run text").stdout.as_ref(), b"run ended\nmet\n");
}
#[test]
fn an_unchanged_ended_delivery_still_waits_for_the_terminal() {
    let task = f::ended_task(c::Phase::Done);
    let delivery = c::Delivery::Task(Box::new(task.clone()));
    let mut root = f::signed_in(f::PARTY);
    let call = root.wait_since(f::WAIT_TASK, f::UNTIL_ENDED, &f::task_digest(&delivery).hex());
    root.deliver_subject(&f::task_watch(), delivery);
    assert!(root.answer(call).is_none());
    assert!(root.reads.is_empty());
    root.end(call, &f::task_watch(), c::WatchEnd::SubjectEnded);
    root.serve(call, f::task_result(task));
    assert_eq!(root.answer(call).expect("terminal overrides since").status, 0);
}
#[test]
fn a_client_handle_limit_exits_1() {
    let mut root = f::signed_in_with_handle_limit();
    let call = root.submit(f::WAIT_TASK);
    assert_eq!(root.answer(call).expect("client handle limit").status, 1);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}
#[test]
fn sign_out_during_the_final_read_exits_1() {
    let (mut root, call) = f::finishing_call();
    root.client(jig_client::Event::Up { up: jig_client::Up::SignedOut });
    assert_eq!(root.answer(call).expect("signed out final read").status, 1);
    assert_eq!(crate::root::next_deadline(&root.root), None);
}
