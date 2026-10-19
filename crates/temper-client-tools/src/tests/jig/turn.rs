use crate::tests::fixtures;
use jig_client::{
    Readout,
    contract::{ReadNone, ReadResult},
};
#[test]
fn a_turn_past_the_read_bound_exits_0() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let call = root.submit(fixtures::TURN_CALL);
    root.serve(call, fixtures::turn_too_large(fixtures::TOO_LARGE));
    let printed = root.answer(call).expect("sized answer");
    assert_eq!((printed.status, printed.stdout), (0, fixtures::golden("turn_limit")));
}
#[test]
fn a_large_page_turn_prints_its_read_call() {
    let printed = fixtures::read_answer(fixtures::ACTIVITY_CALL, fixtures::activity_too_large());
    assert_eq!(printed.stdout, fixtures::golden("activity_turn"));
}
#[test]
fn a_turn_part_is_cut_with_its_continuation() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_text(fixtures::LONG_PART));
    assert_eq!(printed.stdout, fixtures::golden("turn_part"));
}
#[test]
fn a_turn_part_at_the_bound_is_complete() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_text(fixtures::PART_SIZE));
    assert_eq!(printed.stdout, fixtures::golden("turn_part_complete"));
}
#[test]
fn a_turn_continuation_starts_at_its_byte_offset() {
    let printed = fixtures::read_answer(fixtures::TURN_NEXT_CALL, fixtures::turn_text(fixtures::LONG_PART));
    assert_eq!(printed.stdout, fixtures::golden("turn_part_next"));
}
#[test]
fn client_document_limits_exit_1() {
    for readout in [Readout::Limit, Readout::Jig(ReadResult::None(ReadNone::TooLarge { bytes: fixtures::TOO_LARGE }))] {
        let (mut root, call) = fixtures::read_call(fixtures::TASK_CALL);
        root.serve_readout(call, readout);
        assert_eq!(root.answer(call).expect("limit answered").status, 1);
    }
}
#[test]
fn chat_prints_only_the_hosts_provisional_text() {
    let (mut root, call) = fixtures::read_call(fixtures::CHAT_CALL);
    root.serve(call, fixtures::activity_too_large());
    root.deliver_run(fixtures::no_provisional());
    assert_eq!(root.answer(call).expect("chat answered").stdout, fixtures::golden("activity_turn"));
}

#[test]
fn a_run_closes_its_watch_at_its_first_delivery() {
    let (mut root, call) = fixtures::read_call(fixtures::RUN_CALL);
    root.deliver_run(fixtures::no_provisional());
    assert_eq!(root.closed.len(), fixtures::PAGE_ROWS);
    let _printed = root.answer(call).expect("run answered");
}

#[test]
fn a_turn_prints_its_supplied_blocks() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_call(None));
    assert_eq!(printed.stdout, fixtures::golden("turn_call"));
}
#[test]
fn a_path_is_never_cut() {
    let printed = fixtures::read_answer(fixtures::TURN_CALL, fixtures::turn_call(Some(fixtures::long_path())));
    assert_eq!(printed.stdout, fixtures::golden("turn_path"));
}
#[test]
fn a_json_turn_part_carries_its_continuation() {
    let printed = fixtures::read_answer(fixtures::TURN_JSON_CALL, fixtures::turn_text(fixtures::LONG_PART));
    assert_eq!(printed.stdout, fixtures::golden("turn_part_json"));
}

#[test]
fn run_prints_the_hosts_provisional_text() {
    let (mut root, call) = fixtures::read_call(fixtures::RUN_CALL);
    root.deliver_run(fixtures::provisional());
    assert_eq!(root.answer(call).expect("run answered").stdout, fixtures::golden("run_provisional"));
}
