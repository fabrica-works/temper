use crate::tests::fixtures;
use jig_client::contract::{Admission, ReadNone, ReadResult};
#[test]
fn a_task_read_prints_its_document() {
    let (mut root, call, task) = fixtures::task_read();
    root.serve(call, fixtures::task_result(task));
    let answer = root.answer(call).expect("answered once the read is");
    assert_eq!((answer.status, answer.stdout), (0, fixtures::golden("task_goal_waiting")));
}
#[test]
fn a_proposable_act_says_to_whom() {
    let (mut root, call, mut task) = fixtures::task_read();
    task.acts.get_mut(0).expect("first offer").admission =
        Admission::Proposed { to: jig_client::contract::Recipient::Party(fixtures::PARTY) };
    root.serve(call, fixtures::task_result(task));
    assert_eq!(root.answer(call).expect("read answered").stdout, fixtures::golden("task_proposed"));
}
#[test]
fn a_refused_act_says_why_without_a_call() {
    let (mut root, call, mut task) = fixtures::task_read();
    task.acts.get_mut(0).expect("first offer").admission =
        Admission::Refused(jig_client::contract::Refusal::NotOffered);
    root.serve(call, fixtures::task_result(task));
    assert_eq!(root.answer(call).expect("read answered").stdout, fixtures::golden("task_refused"));
}
#[test]
fn me_prints_the_live_frame() {
    for (argv, golden) in [(fixtures::ME_CALL, "me"), (fixtures::ME_JSON_CALL, "me_json")] {
        let (mut root, call) = fixtures::read_call(argv);
        let answer = root.answer(call).expect("live frame answered");
        assert_eq!((answer.status, answer.stdout), (0, fixtures::golden(golden)), "{golden}");
    }
}
#[test]
fn no_call_runs_before_me_is_live() {
    let mut root = fixtures::started(skein_lib::List::with_capacity(fixtures::EMPTY));
    let call = root.submit(fixtures::TASK_CALL);
    assert_eq!(root.reads.len(), 0);
    root.deliver(jig_client::contract::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    assert_eq!(
        root.reads.get(&call).expect("read after live me").1,
        jig_client::contract::Read::Task { task: fixtures::TASK }
    );
}
#[test]
fn each_read_none_exits_as_the_table_says() {
    for (why, status, message) in [
        (ReadNone::Unknown, 2, b"unknown\n".as_slice()),
        (ReadNone::Refused, 2, b"refused\n"),
        (ReadNone::Gone, 2, b"gone\n"),
        (ReadNone::Busy, 1, b"busy\n"),
        (ReadNone::SignedOut, 1, b"signed out\n"),
        (ReadNone::Unreachable, 1, b"unreachable\n"),
        (ReadNone::Malformed, 1, b"malformed document\n"),
        (ReadNone::TooLarge { bytes: fixtures::TOO_LARGE }, 1, b"document limit\n"),
    ] {
        let (mut root, call) = fixtures::read_call(fixtures::TASK_CALL);
        root.serve(call, ReadResult::None(why));
        let answer = root.answer(call).expect("none is terminal");
        assert_eq!((answer.status, answer.stderr.as_ref()), (status, message), "{why:?}");
    }
}
#[test]
fn a_result_past_the_text_limit_says_how_much_is_left() {
    let (mut root, call, mut task) = fixtures::task_read();
    let mut words = skein_lib::Writer::new(usize::try_from(fixtures::LONG_WORDS).expect("fixture size"));
    for _ in 0..fixtures::LONG_WORDS {
        words.put(b"x").expect("fixture length");
    }
    task.spec.specification.words = fixtures::document_text(&words.finish());
    root.serve(call, fixtures::task_result(task));
    let answer = root.answer(call).expect("cut result answered");
    assert_eq!(answer.stdout, fixtures::golden("task_cut"));
}

#[test]
fn a_bounded_offer_prints_its_whole_bounds() {
    for (beyond, golden) in [
        (
            jig_client::contract::Beyond::Proposed { to: jig_client::contract::Recipient::Party(fixtures::PARTY) },
            "task_bounded_proposed",
        ),
        (jig_client::contract::Beyond::Refused(jig_client::contract::Refusal::NotOffered), "task_bounded_refused"),
    ] {
        let (mut root, call, mut task) = fixtures::task_read();
        task.acts.get_mut(0).expect("first offer").admission = fixtures::bounded_offer(beyond);
        root.serve(call, fixtures::task_result(task));
        assert_eq!(root.answer(call).expect("read answered").stdout, fixtures::golden(golden), "{golden}");
    }
}

#[test]
fn an_owner_resource_path_is_printed_whole() {
    let (mut root, call, mut task) = fixtures::task_read();
    fixtures::long_resource(&mut task);
    root.serve(call, fixtures::task_result(task));
    assert_eq!(root.answer(call).expect("read answered").stdout, fixtures::golden("task_whole_path"));
}

#[test]
fn an_act_with_no_tool_prints_its_name() {
    let mut printed_by_admission = skein_lib::List::with_capacity(fixtures::NO_TOOL_CASES);
    let mut expected = skein_lib::List::with_capacity(fixtures::NO_TOOL_CASES);
    for (admission, golden) in [
        (Admission::Allowed, "task_no_tool"),
        (Admission::Proposed { to: jig_client::contract::Recipient::Party(fixtures::PARTY) }, "task_no_tool_proposed"),
        (Admission::Refused(jig_client::contract::Refusal::NotOffered), "task_no_tool_refused"),
        (
            fixtures::bounded_offer(jig_client::contract::Beyond::Refused(jig_client::contract::Refusal::NotOffered)),
            "task_no_tool_bounded",
        ),
    ] {
        let (mut root, call, mut task) = fixtures::task_read();
        let offer = task.acts.get_mut(0).expect("first offer");
        offer.act = jig_client::contract::ActKind::Take;
        offer.admission = admission;
        root.serve(call, fixtures::task_result(task));
        printed_by_admission.push((golden, root.answer(call).expect("read answered").stdout)).expect("every row");
        expected.push((golden, fixtures::golden(golden))).expect("every row");
    }
    assert_eq!(printed_by_admission, expected);
}
#[test]
fn task_json_names_the_nested_contract_fields() {
    let (mut root, call) = fixtures::read_call(fixtures::TASK_JSON_CALL);
    root.serve(call, fixtures::task_result(fixtures::goal(fixtures::TASK, fixtures::waiting_on_me())));
    let answer = root.answer(call).expect("JSON read answered");
    let field = br#""waits_on":{"party":{"whom":{"person":73},"entries":1,"task":42,"kind":"question"}}"#;
    let mut found = false;
    for bytes in answer.stdout.windows(field.len()) {
        if bytes == field {
            found = true;
            break;
        }
    }
    assert!(found);
}

#[test]
fn a_source_wait_keeps_its_unknown_deadline() {
    use jig_client::contract as c;
    for (state, expected) in [
        (c::TaskSourceState::Later { until: None }, b"later until unknown".as_slice()),
        (c::TaskSourceState::Limited { until: None }, b"limited until unknown".as_slice()),
    ] {
        let (mut root, call, mut task) = fixtures::task_read();
        task.run_wait =
            Some(c::TaskRunWait::Account { account: Some(43), source: Some(17), why: c::TaskWaitWhy::Source(state) });
        root.serve(call, fixtures::task_result(task));
        let answer = root.answer(call).expect("read answered");
        assert!(skein_lib::bytes::find(&answer.stdout, expected).is_some());
    }
}

#[test]
fn a_task_read_retains_its_named_payer() {
    let (mut root, call) = fixtures::read_call(fixtures::TASK_JSON_CALL);
    let mut task = fixtures::goal(fixtures::TASK, fixtures::waiting_on_me());
    task.kept_account = Some(43);

    root.serve(call, fixtures::task_result(task));
    let answer = root.answer(call).expect("read answered");
    assert!(skein_lib::bytes::find(&answer.stdout, br#""kept_account":43"#).is_some());
}

#[test]
fn a_task_read_retains_its_account_wait_reason() {
    use jig_client::contract as c;
    let (mut root, call) = fixtures::read_call(fixtures::TASK_JSON_CALL);
    let mut task = fixtures::goal(fixtures::TASK, fixtures::waiting_on_me());
    task.run_wait = Some(c::TaskRunWait::Account {
        account: Some(43),
        source: Some(17),
        why: c::TaskWaitWhy::Invalid(c::AccountWhy::Limited),
    });
    root.serve(call, fixtures::task_result(task));
    let answer = root.answer(call).expect("read answered");
    let field = br#""run_wait":{"account":{"account":43,"source":17,"why":{"invalid":"limited"}}}"#;
    assert!(skein_lib::bytes::find(&answer.stdout, field).is_some());
}

#[test]
fn a_source_wait_prints_a_null_json_deadline() {
    use jig_client::contract as c;
    for state in [c::TaskSourceState::Later { until: None }, c::TaskSourceState::Limited { until: None }] {
        let (mut root, call) = fixtures::read_call(fixtures::TASK_JSON_CALL);
        let mut task = fixtures::goal(fixtures::TASK, fixtures::waiting_on_me());
        task.run_wait =
            Some(c::TaskRunWait::Account { account: Some(43), source: Some(17), why: c::TaskWaitWhy::Source(state) });
        root.serve(call, fixtures::task_result(task));
        let answer = root.answer(call).expect("read answered");
        assert!(skein_lib::bytes::find(&answer.stdout, br#""until":null"#).is_some());
    }
}
