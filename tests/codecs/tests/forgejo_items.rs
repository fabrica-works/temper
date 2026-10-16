use skein_lib::Wall;
use temper_codecs_world::forgejo;
use temper_forge_forgejo::{
    client,
    decode::Malformed,
    document::{
        Body, CombinedStatus, Comment, CommitStatus, Document, Issue, IssueState, PullBranch, PullRequest, Row,
        StatusState,
    },
    request::{IssueKind, Request, Sort, StateFilter},
    server::{self, Answer},
};
fn issue(number: u64, pull: bool, title: &[u8], body: &[u8], author: u64, at: u64) -> Issue {
    Issue {
        number,
        pull,
        state: IssueState::Open,
        user: forgejo::user(author),
        labels: Box::from([]),
        updated_at: Wall::from_nanos(at),
        title: Box::from(title),
        body: Body::Kept(Box::from(body)),
    }
}
fn edited() -> Issue {
    issue(3, false, b"edited title", b"<!-- temper:key 01000001 -->\nissue text", 2, 1_577_817_005_000_000_000)
}
fn feature() -> Issue {
    issue(1, true, b"feature/slash", b"fixture pull", 1, 1_577_817_009_000_000_000)
}
fn conflict() -> Issue {
    issue(2, true, b"conflict", b"fixture pull", 1, 1_577_817_007_000_000_000)
}
fn writer() -> Issue {
    issue(4, true, b"writer pull", b"writer text", 2, 1_577_817_008_000_000_000)
}
fn listing(page: u32, kind: Option<IssueKind>, since: Wall) -> Request {
    Request::Issues {
        at: forgejo::specimen(),
        state: Some(StateFilter::All),
        kind,
        since: Some(since),
        sort: Some(Sort::LeastUpdate),
        page: forgejo::page(page, 2),
    }
}
fn pull(merged: bool) -> PullRequest {
    if merged {
        PullRequest {
            number: 5,
            state: IssueState::Closed,
            head: PullBranch { repo_id: 1, name: Box::from(*b"mergeable"), sha: Some(forgejo::sha(5)) },
            base: PullBranch { repo_id: 1, name: Box::from(*b"landing"), sha: Some(forgejo::sha(7)) },
            merged: true,
            merge_commit_sha: Some(forgejo::sha(7)),
            merge_base: Some(forgejo::sha(1)),
            mergeable: true,
            requested_reviewers: Box::from([]),
        }
    } else {
        PullRequest {
            number: 1,
            state: IssueState::Open,
            head: PullBranch { repo_id: 1, name: Box::from(*b"feature/slash"), sha: Some(forgejo::sha(4)) },
            base: PullBranch { repo_id: 1, name: Box::from(*b"main"), sha: Some(forgejo::sha(3)) },
            merged: false,
            merge_commit_sha: None,
            merge_base: Some(forgejo::sha(6)),
            mergeable: true,
            requested_reviewers: Box::from([forgejo::user(2)]),
        }
    }
}
fn cases() -> Vec<(&'static str, Request, Answer)> {
    vec![
        ("issues-page-1", listing(1, None, Wall::EPOCH), Answer::Issues(Box::from([conflict(), edited()]))),
        ("issues-page-2", listing(2, None, Wall::EPOCH), Answer::Issues(Box::from([writer(), feature()]))),
        ("issues-type-issues", listing(1, Some(IssueKind::Issues), Wall::EPOCH), Answer::Issues(Box::from([edited()]))),
        (
            "issues-type-pulls",
            listing(1, Some(IssueKind::Pulls), Wall::EPOCH),
            Answer::Issues(Box::from([conflict(), writer()])),
        ),
        (
            "issues-since-2554",
            listing(1, None, Wall::from_nanos(18_446_744_073_000_000_000)),
            Answer::Issues(Box::from([])),
        ),
        ("issue", Request::Issue { at: forgejo::specimen(), number: 3 }, Answer::Issue(edited())),
        ("issue-of-pull", Request::Issue { at: forgejo::specimen(), number: 1 }, Answer::Issue(feature())),
        (
            "issue-comments",
            Request::Comments { at: forgejo::specimen(), number: 3 },
            Answer::Comments(Box::from([
                Comment {
                    id: 4,
                    user: forgejo::user(2),
                    created_at: Some(Wall::from_nanos(1_577_817_005_000_000_000)),
                    updated_at: Some(Wall::from_nanos(1_577_817_005_000_000_000)),
                    body: Body::Kept(Box::from(*b"<!-- temper:key 01000002 -->\ncomment text")),
                },
                Comment {
                    id: 5,
                    user: forgejo::user(2),
                    created_at: Some(Wall::from_nanos(1_577_817_005_000_000_000)),
                    updated_at: Some(Wall::from_nanos(1_577_817_005_000_000_000)),
                    body: Body::Kept(Box::from(*b"plain comment")),
                },
            ])),
        ),
        ("pull", Request::Pull { at: forgejo::specimen(), number: 1 }, Answer::Pull(pull(false))),
        ("pull-merged", Request::Pull { at: forgejo::specimen(), number: 5 }, Answer::Pull(pull(true))),
        (
            "pull-for-encoded-head",
            Request::PullFor { at: forgejo::specimen(), base: Box::from(*b"main"), head: Box::from(*b"feature/slash") },
            Answer::Pull(pull(false)),
        ),
        (
            "combined-status",
            Request::CombinedStatus { at: forgejo::specimen(), commit: forgejo::sha(4), page: forgejo::page(1, 1) },
            Answer::CombinedStatus {
                state: CombinedStatus { state: Some(StatusState::Pending), total_count: 1 },
                sha: Some(forgejo::sha(4)),
                statuses: Box::from([CommitStatus {
                    creator: Some(forgejo::user(2)),
                    updated_at: Wall::from_nanos(1_577_817_005_000_000_000),
                    context: Box::from(*b"fixture/other"),
                    status: StatusState::Pending,
                    description: Box::from([]),
                    target_url: Box::from([]),
                }]),
            },
        ),
        (
            "combined-status-empty",
            Request::CombinedStatus { at: forgejo::specimen(), commit: forgejo::sha(6), page: forgejo::page(1, 1) },
            Answer::CombinedStatus {
                state: CombinedStatus { state: None, total_count: 0 },
                sha: None,
                statuses: Box::from([]),
            },
        ),
    ]
}
fn expected(answer: Answer) -> (Document, Vec<Row>) {
    match answer {
        Answer::Issues(values) => {
            (Document::Listed { skipped: 0 }, values.into_vec().into_iter().map(Row::Issue).collect())
        }
        Answer::Comments(values) => {
            (Document::Listed { skipped: 0 }, values.into_vec().into_iter().map(Row::Comment).collect())
        }
        Answer::Issue(value) => (Document::Issue(value), vec![]),
        Answer::Pull(value) => (Document::Pull(value), vec![]),
        Answer::CombinedStatus { state, statuses, .. } => {
            (Document::CombinedStatus(state), statuses.into_vec().into_iter().map(Row::Status).collect())
        }
        Answer::Error(_)
        | Answer::Comment(_)
        | Answer::Status(_)
        | Answer::Review(_)
        | Answer::Files(_)
        | Answer::Compare { .. }
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Jobs(_)
        | Answer::Reviews(_)
        | Answer::ReviewComments(_)
        | Answer::Branch(_)
        | Answer::Branches(_)
        | Answer::Protection(_)
        | Answer::Protections(_)
        | Answer::Collaborators(_)
        | Answer::Permission(_)
        | Answer::Pulls(_)
        | Answer::User(_)
        | Answer::ApiSettings(_)
        | Answer::Repository(_)
        | Answer::Search { .. } => {
            panic!("an item answer")
        }
    }
}
#[test]
fn item_transcripts_decode_to_their_documents_and_rows() {
    for (case, request, answer) in cases() {
        let transcript = forgejo::transcript(case);
        assert_eq!(transcript.status, 200, "{case}");
        let (document, rows) = expected(answer);
        let result = forgejo::decode(&request, &transcript.body);
        assert_eq!(result.document, Ok(document), "{case}");
        assert_eq!(result.rows, rows, "{case}");
    }
}
#[test]
fn item_requests_encode_as_the_tool_sent_them() {
    for (case, request, _) in cases() {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &forgejo::limits()).expect("request");
        assert_eq!(encoded.method, transcript.method, "{case}");
        assert_eq!(encoded.target.as_ref(), transcript.target, "{case}");
        assert_eq!(encoded.body.as_deref().unwrap_or(b""), transcript.request_body, "{case}");
    }
}
#[test]
fn item_requests_route_on_the_servers_side_to_the_same_request() {
    for (case, request, _) in cases() {
        assert_eq!(forgejo::route(&forgejo::transcript(case)), request, "{case}");
    }
}
#[test]
fn item_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape() {
    for (case, request, answer) in cases() {
        let transcript = forgejo::transcript(case);
        let bytes = server::encode(&answer, &forgejo::limits()).expect("server answer");
        assert!(forgejo::shape(&bytes).is_subset(&forgejo::shape(&transcript.body)), "{case}");
        let (document, rows) = expected(answer);
        let result = forgejo::decode(&request, &bytes);
        assert_eq!(result.document, Ok(document), "{case}");
        assert_eq!(result.rows, rows, "{case}");
    }
}
#[test]
fn item_transcript_mutations_are_malformed_naming_the_field() {
    for (case, field, mutation, expected) in [
        ("issue", "number", 0, Malformed::Missing { field: b"number" }),
        ("issue", "state", 1, Malformed::Value { field: b"state" }),
        ("pull", "head", 2, Malformed::Id { field: b"sha" }),
        ("issue", "updated_at", 3, Malformed::Time { field: b"updated_at" }),
        ("issue", "labels", 4, Malformed::Bound { field: b"labels" }),
    ] {
        let transcript = forgejo::transcript(case);
        let mut json: serde_json::Value = serde_json::from_slice(&transcript.body).expect("transcript JSON");
        match mutation {
            0 => {
                json.as_object_mut().expect("object").remove(field);
            }
            1 => json[field] = serde_json::json!("merged"),
            2 => json[field]["sha"] = serde_json::json!("000000000000000000000000000000000000000"),
            3 => json[field] = serde_json::json!("2020-01-01T00:00:05"),
            4 => json[field] = serde_json::Value::Array(vec![serde_json::json!({"name":"label"}); 65]),
            _ => panic!("mutation"),
        }
        let request = forgejo::route(&transcript);
        let bytes = serde_json::to_vec(&json).expect("mutated JSON");
        assert_eq!(forgejo::decode(&request, &bytes).document, Err(expected), "{case}/{field}");
    }
}
