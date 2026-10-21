use crate::tests::fixtures;
use jig_client::contract as c;
#[test]
fn a_list_prints_its_rows_then_its_continuation() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let call = root.submit(fixtures::TASKS_CALL);
    root.serve(call, fixtures::task_page_with_cursor());
    assert_eq!(root.answer(call).expect("page answered").stdout, fixtures::golden("tasks_next"));
}
#[test]
fn ended_tasks_use_only_the_ended_group() {
    let (root, call) = fixtures::read_call(fixtures::ENDED_CALL);
    assert_eq!(
        root.reads.get(&call).expect("task query").1,
        c::Read::Tasks {
            query: c::TaskQuery {
                project: Some(fixtures::PROJECT),
                requester: None,
                root: None,
                parent: None,
                label: None,
                phase: Some(c::PhaseClass::Ended),
                executor: None,
                tracked: None,
                order: c::QueryOrder::Number
            },
            live: None,
            ended: Some(fixtures::cursor()),
        }
    );
}
#[test]
fn a_composite_task_prints_one_answer() {
    let (mut root, call) = fixtures::composite();
    assert_eq!(root.answer(call).expect("all parts answered").stdout, fixtures::golden("task_composite"));
}
#[test]
fn a_composite_task_closes_its_related_watches() {
    let (root, _) = fixtures::composite();
    assert_eq!(root.closed.len(), fixtures::RELATED_WATCHES);
}
#[test]
fn an_inbox_cursor_of_another_group_than_its_kind_exits_1() {
    assert_eq!(fixtures::parse(fixtures::WRONG_INBOX_GROUP), Err(crate::jig::argv::Usage::Kind { argument: 3 }));
}

#[test]
fn an_inbox_reads_on_in_needs_while_needs_has_more() {
    let printed = fixtures::read_answer(fixtures::INBOX_CALL, fixtures::inbox_page(true));
    assert_eq!(printed.stdout, fixtures::golden("inbox_needs_next"));
}
#[test]
fn an_inbox_cursor_selects_its_group() {
    let (root, call) = fixtures::read_call(fixtures::NEEDS_NEXT_CALL);
    assert_eq!(
        root.reads.get(&call).expect("inbox read").1,
        c::Read::Inbox {
            group: c::InboxReadGroup::Needs,
            project: None,
            kind: None,
            goal: None,
            after: Some(fixtures::cursor())
        }
    );
}

#[test]
fn inbox_prints_what_needs_the_party_then_updates() {
    let printed = fixtures::inbox_answer(fixtures::PAGE_ROWS, fixtures::PAGE_ROWS, false);
    assert_eq!(printed.stdout, fixtures::golden("inbox_both"));
}
#[test]
fn an_inbox_reads_on_in_updates_once_needs_ends() {
    let printed = fixtures::inbox_answer(fixtures::PAGE_ROWS, fixtures::PAGE_ROWS, true);
    assert_eq!(printed.stdout, fixtures::golden("inbox_updates_next"));
}
#[test]
fn an_inbox_page_whose_updates_do_not_fit_reads_on_at_the_start_of_updates() {
    let printed = fixtures::inbox_answer(fixtures::FULL_NEEDS, fixtures::PAGE_ROWS, true);
    assert_eq!(printed.stdout, fixtures::golden("inbox_updates_start"));
}
#[test]
fn an_inbox_group_byte_alone_starts_the_group() {
    let (root, call) = fixtures::read_call(fixtures::UPDATES_START_CALL);
    assert_eq!(
        root.reads.get(&call).expect("updates read").1,
        c::Read::Inbox { group: c::InboxReadGroup::Updates, project: None, kind: None, goal: None, after: None }
    );
}
#[test]
fn a_limited_activity_reads_on_with_its_page_whole() {
    let printed = fixtures::read_answer(fixtures::LIMITED_ACTIVITY_CALL, fixtures::activity_words());
    assert_eq!(printed.stdout, fixtures::golden("activity_limited"));
}
#[test]
fn a_limited_task_activity_names_its_whole_page() {
    let (mut root, call) = fixtures::read_call(fixtures::LIMITED_TASK_CALL);
    root.serve(call, fixtures::task_result(fixtures::goal(fixtures::TASK, fixtures::waiting_on_me())));
    root.serve(call, fixtures::activity_words());
    assert_eq!(root.answer(call).expect("task answered").stdout, fixtures::golden("task_limited_activity"));
}

#[test]
fn ended_tasks_keep_their_flag_in_the_continuation() {
    let printed = fixtures::read_answer(fixtures::ENDED_CALL, fixtures::ended_page());
    assert_eq!(printed.stdout, fixtures::golden("tasks_ended_next"));
}

#[test]
fn task_labels_use_jigs_label_numbers() {
    for (word, label) in fixtures::TASK_LABELS {
        let argv = fixtures::extended(fixtures::TASKS_CALL, &["--label", word]);
        let (root, call) = fixtures::read_call(argv.as_slice());
        let c::Read::Tasks { query, .. } = &root.reads.get(&call).expect("task query").1 else { panic!("tasks query") };
        assert_eq!(query.label, Some(*label), "{word}");
    }
}

#[test]
fn a_kept_activity_is_parked_in_text_and_preserved_in_json() {
    for (argv, expected) in [
        (&["activity", "41"][..], b"parked".as_slice()),
        (&["activity", "41", "--json"][..], br#""kept":{"attempt":7,"pending":[]}"#),
    ] {
        let c::ReadResult::Value(c::ReadValue::Activity(mut page)) = fixtures::activity_words() else {
            panic!("activity")
        };
        page.rows.get_mut(0).expect("row").event = c::Activity::Owner {
            source: Box::new(c::OwnerActivityLine {
                task: fixtures::TASK,
                position: 1,
                at: skein_lib::Wall::EPOCH,
                line: c::OwnerActivity::Kept { attempt: 7, pending: skein_lib::List::with_capacity(0) },
            }),
            decoded: None,
        };
        let printed = fixtures::read_answer(argv, c::ReadResult::Value(c::ReadValue::Activity(page)));
        assert!(skein_lib::bytes::find(&printed.stdout, expected).is_some());
    }
}
#[test]
fn a_met_notice_keeps_its_dependency_and_removal_flag_in_json() {
    let c::ReadResult::Value(c::ReadValue::Activity(mut page)) = fixtures::activity_words() else { panic!("activity") };
    page.rows.get_mut(0).expect("row").event = c::Activity::Owner {
        source: Box::new(c::OwnerActivityLine {
            task: fixtures::TASK,
            position: 1,
            at: skein_lib::Wall::EPOCH,
            line: c::OwnerActivity::Words {
                message: 2,
                by: c::OwnerRequester::Task(12),
                kind: c::OwnerMessageKind::Met { dependency: 12, removed: true },
                words: fixtures::act_text(b""),
            },
        }),
        decoded: None,
    };
    let printed =
        fixtures::read_answer(&["activity", "41", "--json"], c::ReadResult::Value(c::ReadValue::Activity(page)));
    assert!(skein_lib::bytes::find(&printed.stdout, br#""met":{"dependency":12,"removed":true}"#).is_some());
}
