use crate::tests::fixtures as f;
use jig_client::{Watchable, contract as c};
use skein_lib::Time;
#[test]
fn a_wait_until_replied_answers_at_the_parked_delivery_with_its_digest() {
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &["--until", "replied"]);
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
    let call = root.wait_since(f::WAIT_TASK, &["--until", "replied"], &f::task_digest(&parked).hex());
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
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &["--until", "waiting-on-me"]);
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
    let flags = f::extended(f::SHORT_TIMEOUT, &["--until", "replied"]);
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
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &["--until", "replied"]);
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
    let (mut root, call) = f::waiting_call(f::WAIT_TASK, &["--until", "replied"]);
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
    let (mut root, call) = f::plain_wait(&["--until", "replied"]);
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
