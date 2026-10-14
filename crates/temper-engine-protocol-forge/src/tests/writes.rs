//! Write plans through the component's scripted byte peer.
use super::fixtures::{self, Harness};
use crate::failure::{self, Asked, Classified, Drop};
use alloc::boxed::Box;
use skein_lib::{Duration, bytes};
use temper_engine_domain_forge_client::api::{Answer, BranchCreation, Check, Error, Op, Verdict, Write};
use temper_forge_forgejo::failure::Refusal;
const ISSUE: &[u8] = br#"{"number":7,"state":"open","user":{"id":2,"login":"writer"},"labels":[],"updated_at":"2020-01-01T00:00:00Z","title":"title","body":"body"}"#;
const COMMENT: &[u8] = br#"{"id":9,"user":{"id":2,"login":"writer"},"body":"body"}"#;
const REVIEW: &[u8] = br#"{"id":11,"user":{"id":2,"login":"writer"},"state":"APPROVED","commit_id":"0100000000000000000000000000000000000000","body":"body","official":true,"dismissed":false}"#;
const STATUS: &[u8] =
    br#"{"creator":null,"status":"success","context":"gate","description":"","target_url":"","updated_at":"2020-01-01T00:00:00Z"}"#;
const BRANCH: &[u8] = br#"{"name":"work","commit":{"id":"0100000000000000000000000000000000000000"},"effective_branch_protection_name":""}"#;
fn boxed(bytes: &[u8]) -> Box<[u8]> {
    Box::from(bytes)
}
fn known() -> Harness {
    let mut h = Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    h
}
fn head() -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[0] = 1;
    bytes
}
fn creation_rows() -> [CreationRow; 3] {
    [
        (
            "issue",
            Write::CreateIssue { key: boxed(&[0xab, 0xff]), title: boxed(b"title"), body: boxed(b"body") },
            b"POST /api/v1/repos/fixture/specimen/issues ".as_slice(),
            ISSUE,
            Answer::Created(7),
        ),
        (
            "comment",
            Write::Post { number: 7, key: boxed(&[0xab, 0xff]), body: boxed(b"body") },
            b"POST /api/v1/repos/fixture/specimen/issues/7/comments ".as_slice(),
            COMMENT,
            Answer::Commented(9),
        ),
        (
            "review",
            Write::Review { number: 7, key: boxed(&[0xab, 0xff]), verdict: Verdict::Approve, body: boxed(b"body") },
            b"POST /api/v1/repos/fixture/specimen/pulls/7/reviews ".as_slice(),
            REVIEW,
            Answer::Reviewed(11),
        ),
    ]
}
#[test]
fn creations_carry_the_key_marker_at_the_head_of_their_body() {
    for (name, write, request, body, expected) in creation_rows() {
        let mut h = known();
        h.call(1, fixtures::REPO, Op::Write(write));
        h.run(Duration::from_millis(1));
        let written = h.request_bytes(fixtures::FORGE);
        assert!(bytes::find(&written, request).is_some(), "{name}: route");
        assert!(bytes::find(&written, br#""body":"<!-- temper:key abff -->\nbody""#).is_some(), "{name}: marked body");
        h.answer(fixtures::FORGE, &fixtures::response(201, body));
        h.run(Duration::ZERO);
        let answered = h.answered(1).expect("creation answered");
        assert_eq!((answered.cost, &answered.result), (1, &Ok(expected)), "{name}");
    }
}
fn write_rows() -> [WriteRow; 7] {
    [
        (
            "edit",
            Write::Edit { number: 7, title: Some(boxed(b"changed")), body: None },
            b"PATCH /api/v1/repos/fixture/specimen/issues/7 ".as_slice(),
            br#"{"title":"changed"}"#.as_slice(),
            201,
            ISSUE,
            Answer::Done,
        ),
        (
            "close",
            Write::Close { number: 7 },
            b"PATCH /api/v1/repos/fixture/specimen/issues/7 ".as_slice(),
            br#"{"state":"closed"}"#.as_slice(),
            201,
            ISSUE,
            Answer::Done,
        ),
        (
            "reopen",
            Write::Reopen { number: 7 },
            b"PATCH /api/v1/repos/fixture/specimen/issues/7 ".as_slice(),
            br#"{"state":"open"}"#.as_slice(),
            201,
            ISSUE,
            Answer::Done,
        ),
        (
            "update",
            Write::Update { number: 7 },
            b"POST /api/v1/repos/fixture/specimen/pulls/7/update?style=merge ".as_slice(),
            b"\r\n\r\n".as_slice(),
            200,
            b"".as_slice(),
            Answer::Done,
        ),
        (
            "status",
            Write::Status { commit: head(), context: boxed(b"gate"), check: Check::Passed },
            b"POST /api/v1/repos/fixture/specimen/statuses/0100000000000000000000000000000000000000 ".as_slice(),
            br#"{"state":"success","context":"gate"}"#.as_slice(),
            201,
            STATUS,
            Answer::Done,
        ),
        (
            "branch",
            Write::CreateBranch { branch: boxed(b"work/a"), commit: head() },
            b"POST /api/v1/repos/fixture/specimen/branches ".as_slice(),
            br#"{"new_branch_name":"work/a","old_ref_name":"0100000000000000000000000000000000000000"}"#.as_slice(),
            201,
            BRANCH,
            Answer::Branch(BranchCreation::Created),
        ),
        (
            "delete",
            Write::DeleteBranch { branch: boxed(b"work/a") },
            b"DELETE /api/v1/repos/fixture/specimen/branches/work%2Fa ".as_slice(),
            b"\r\n\r\n".as_slice(),
            204,
            b"".as_slice(),
            Answer::Done,
        ),
    ]
}
#[test]
fn straightforward_writes_use_their_methods_and_literal_bodies() {
    for (name, write, request, json, code, body, expected) in write_rows() {
        let mut h = known();
        h.call(1, fixtures::REPO, Op::Write(write));
        h.run(Duration::from_millis(1));
        let written = h.request_bytes(fixtures::FORGE);
        assert!(bytes::find(&written, request).is_some(), "{name}: route");
        assert!(bytes::find(&written, json).is_some(), "{name}: body");
        h.answer(fixtures::FORGE, &fixtures::response(code, body));
        h.run(Duration::ZERO);
        let answered = h.answered(1).expect("write answered");
        assert_eq!((answered.cost, &answered.result), (1, &Ok(expected)), "{name}");
    }
}
#[test]
fn text_that_is_not_utf8_is_refused_with_nothing_written() {
    let rows = [
        ("title", Write::CreateIssue { key: boxed(b"key"), title: boxed(&[0xff]), body: boxed(b"body") }),
        ("comment", Write::Post { number: 7, key: boxed(b"key"), body: boxed(&[0xff]) }),
        ("review", Write::Review { number: 7, key: boxed(b"key"), verdict: Verdict::Comment, body: boxed(&[0xff]) }),
        ("edit", Write::Edit { number: 7, title: None, body: Some(boxed(&[0xff])) }),
        ("context", Write::Status { commit: head(), context: boxed(&[0xff]), check: Check::Pending }),
        ("branch", Write::CreateBranch { branch: boxed(&[0xff]), commit: head() }),
    ];
    for (name, write) in rows {
        let mut h = Harness::ready();
        let before = h.request_bytes(fixtures::FORGE);
        h.call(1, fixtures::REPO, Op::Write(write));
        h.run(Duration::from_millis(1));
        let answered = h.answered(1).expect("refused before resolving");
        assert_eq!((answered.cost, &answered.result), (0, &Err(Error::Refused)), "{name}");
        assert_eq!(h.request_bytes(fixtures::FORGE), before, "{name}: no request");
    }
}
fn failure_rows() -> [FailureRow; 25] {
    [
        ("redirect", Asked::Read, 301, None, None, Error::Unavailable, Drop::Path),
        ("missing", Asked::Read, 404, None, None, Error::Missing, Drop::Path),
        ("missing job", Asked::Job, 404, None, None, Error::MissingJob, Drop::Path),
        (
            "rate",
            Asked::Read,
            429,
            None,
            Some(Duration::from_secs(5)),
            Error::RateLimited { after: Duration::from_secs(5) },
            Drop::Nothing,
        ),
        (
            "rate without header",
            Asked::Write,
            429,
            None,
            None,
            Error::RateLimited { after: Duration::ZERO },
            Drop::Nothing,
        ),
        ("quota", Asked::Write, 413, None, None, Error::Full, Drop::Nothing),
        ("archived", Asked::Write, 423, None, None, Error::Forbidden, Drop::Nothing),
        ("style", Asked::Merge, 405, Some(Refusal::MergeStyle), None, Error::Refused, Drop::Row),
        ("old head", Asked::Merge, 409, Some(Refusal::HeadOutOfDate), None, Error::Stale, Drop::Nothing),
        ("delete protection", Asked::DeleteBranch, 403, None, None, Error::Protected, Drop::Nothing),
        ("unknown merge", Asked::Merge, 405, None, None, Error::Refused, Drop::Nothing),
        ("unknown read", Asked::Read, 418, None, None, Error::InvalidAnswer, Drop::Nothing),
        ("before POST", Asked::BeforeWrite, 500, None, None, Error::Unavailable, Drop::Nothing),
        ("unknown before POST", Asked::BeforeWrite, 418, None, None, Error::Unavailable, Drop::Nothing),
        ("already open", Asked::OpenPull, 409, None, None, Error::Exists, Drop::Nothing),
        ("closed", Asked::Merge, 405, Some(Refusal::MergeClosed), None, Error::Closed, Drop::Nothing),
        ("merge conflict", Asked::Merge, 405, Some(Refusal::MergeConflict), None, Error::Conflict, Drop::Nothing),
        ("merge protection", Asked::Merge, 405, Some(Refusal::MergeProtected), None, Error::Protected, Drop::Nothing),
        ("update conflict", Asked::Update, 409, Some(Refusal::UpdateConflict), None, Error::Conflict, Drop::Nothing),
        ("empty", Asked::Write, 422, Some(Refusal::Empty), None, Error::Empty, Drop::Nothing),
        ("unknown 422", Asked::OpenPull, 422, None, None, Error::Refused, Drop::Nothing),
        ("token", Asked::Write, 401, None, None, Error::Forbidden, Drop::Nothing),
        ("permission", Asked::Write, 403, None, None, Error::Forbidden, Drop::Nothing),
        ("5xx", Asked::Write, 500, None, None, Error::Timeout, Drop::Nothing),
        ("unexpected success", Asked::Write, 202, None, None, Error::Timeout, Drop::Nothing),
    ]
}
#[test]
fn failures_classify_by_status_call_and_message() {
    for (name, asked, status, refusal, retry, error, drop) in failure_rows() {
        assert_eq!(failure::classify(asked, status, refusal, retry), Classified { error, drop }, "{name}");
    }
}
#[test]
fn a_branch_already_there_answers_exists_without_retry() {
    let mut h = known();
    h.call(1, fixtures::REPO, Op::Write(Write::CreateBranch { branch: boxed(b"work"), commit: head() }));
    h.answer(fixtures::FORGE, &fixtures::response(409, br#"{"message":"branch already exists"}"#));
    h.run(Duration::ZERO);
    let answered = h.answered(1).expect("branch answered");
    assert_eq!((answered.cost, &answered.result), (1, &Ok(Answer::Branch(BranchCreation::Exists))));
}
#[test]
fn a_malformed_write_answer_is_timeout() {
    let mut h = known();
    h.call(
        1,
        fixtures::REPO,
        Op::Write(Write::CreateIssue { key: boxed(b"key"), title: boxed(b"title"), body: boxed(b"body") }),
    );
    h.answer(fixtures::FORGE, &fixtures::response(201, br#"{"number":7}"#));
    h.run(Duration::ZERO);
    let answered = h.answered(1).expect("malformed creation answered");
    assert_eq!((answered.cost, &answered.result), (1, &Err(Error::Timeout)));
}
#[test]
fn write_refusals_read_only_known_entire_messages() {
    let rows = [
        ("known", br#"{"message":"[Title]: Required"}"#.as_slice(), Error::Empty),
        ("unknown", br#"{"message":"other words"}"#.as_slice(), Error::Refused),
        ("not JSON", b"arbitrary".as_slice(), Error::Refused),
    ];
    for (name, body, expected) in rows {
        let mut h = known();
        h.call(
            1,
            fixtures::REPO,
            Op::Write(Write::CreateIssue { key: boxed(b"key"), title: boxed(b"title"), body: boxed(b"body") }),
        );
        h.answer(fixtures::FORGE, &fixtures::response(422, body));
        h.run(Duration::ZERO);
        let answered = h.answered(1).expect("refusal answered");
        assert_eq!((answered.cost, &answered.result), (1, &Err(expected)), "{name}");
    }
}
type CreationRow = (&'static str, Write, &'static [u8], &'static [u8], Answer);
type WriteRow = (&'static str, Write, &'static [u8], &'static [u8], u16, &'static [u8], Answer);
type FailureRow = (&'static str, Asked, u16, Option<Refusal>, Option<Duration>, Error, Drop);
