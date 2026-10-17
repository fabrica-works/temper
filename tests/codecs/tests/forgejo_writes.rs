use temper_codecs_world::forgejo;
use temper_forge_forgejo::client;
#[test]
fn write_requests_encode_as_the_tool_sent_them() {
    for (case, request) in forgejo::write_cases() {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &forgejo::limits()).expect("write encoding");
        assert_eq!(encoded.method, transcript.method, "{case}");
        assert_eq!(&*encoded.target, transcript.target, "{case}");
        assert_eq!(encoded.body.as_deref().unwrap_or_default(), transcript.request_body, "{case}");
    }
}
#[test]
fn write_requests_route_on_the_servers_side_to_the_same_request() {
    for (case, request) in forgejo::write_cases() {
        assert_eq!(forgejo::route(&forgejo::transcript(case)), request, "{case}");
    }
}
use skein_lib::Wall;
use temper_forge_forgejo::{
    document::{
        Body, Branch, Comment, CommitStatus, Document, Issue, IssueState, PullBranch, PullRequest, Review, ReviewState,
        StatusState,
    },
    server::{self, Answer},
};
fn answers() -> Vec<(&'static str, Answer)> {
    let at = Wall::from_nanos(1_577_817_005_000_000_000);
    let mut rows = vec![];
    for (case, title, state) in [
        ("create-issue", b"fixture issue".as_slice(), IssueState::Open),
        ("edit-issue-title", b"edited title", IssueState::Open),
        ("close-issue", b"edited title", IssueState::Closed),
        ("reopen-issue", b"edited title", IssueState::Open),
    ] {
        rows.push((
            case,
            Answer::Issue(Issue {
                number: 3,
                pull: false,
                state,
                user: forgejo::user(2),
                labels: Box::from([]),
                updated_at: at,
                title: Box::from(title),
                body: Body::Kept(Box::from(*b"<!-- temper:key 01000001 -->\nissue text")),
            }),
        ));
    }
    for (case, number, head, head_sha, base, base_sha) in
        [("open-pull", 4, "writer-head", 2, "landing", 1), ("open-pull-no-changes", 6, "no-changes", 3, "main", 3)]
    {
        rows.push((
            case,
            Answer::Pull(PullRequest {
                number,
                state: IssueState::Open,
                head: PullBranch { name: Box::from(head.as_bytes()), sha: Some(forgejo::sha(head_sha)), repo_id: 1 },
                base: PullBranch { name: Box::from(base.as_bytes()), sha: Some(forgejo::sha(base_sha)), repo_id: 1 },
                merged: false,
                merge_commit_sha: None,
                merge_base: Some(forgejo::sha(3)),
                mergeable: true,
                requested_reviewers: Box::from([]),
            }),
        ));
    }
    rows.extend(posted_answers());
    rows
}
fn posted_answers() -> Vec<(&'static str, Answer)> {
    let at = Wall::from_nanos(1_577_817_005_000_000_000);
    let mut rows = vec![];
    for (case, id, body) in
        [("post-comment", 4, "<!-- temper:key 01000002 -->\ncomment text"), ("post-comment-plain", 5, "plain comment")]
    {
        rows.push((
            case,
            Answer::Comment(Comment {
                id,
                user: forgejo::user(2),
                created_at: Some(at),
                updated_at: Some(at),
                body: Body::Kept(Box::from(body.as_bytes())),
            }),
        ));
    }
    for (case, id, state, official, body) in [
        ("post-review-approved", 1, ReviewState::Approved, true, "<!-- temper:key 01000003 -->\nreview text"),
        (
            "post-review-request-changes",
            2,
            ReviewState::RequestChanges,
            true,
            "<!-- temper:key 01000004 -->\nreview text",
        ),
        ("post-review-comment", 3, ReviewState::Comment, false, "<!-- temper:key 01000005 -->\nreview text"),
    ] {
        rows.push((
            case,
            Answer::Review(Review {
                id,
                user: forgejo::user(2),
                state,
                commit_id: forgejo::sha(4),
                body: Body::Kept(Box::from(body.as_bytes())),
                submitted_at: Some(at),
                updated_at: Some(at),
                official,
                dismissed: false,
            }),
        ));
    }
    for (case, context, status) in [
        ("post-status", b"fixture/test".as_slice(), StatusState::Success),
        ("post-status-pending", b"fixture/other", StatusState::Pending),
    ] {
        rows.push((
            case,
            Answer::Status(CommitStatus {
                creator: Some(forgejo::user(2)),
                updated_at: Wall::from_nanos(1_577_817_005_000_000_000),
                context: Box::from(context),
                status,
                description: Box::from([]),
                target_url: Box::from([]),
            }),
        ));
    }
    rows.push((
        "create-branch",
        Answer::Branch(Branch {
            name: Box::from(*b"recorded"),
            commit: forgejo::sha(4),
            effective_branch_protection_name: None,
        }),
    ));
    rows
}
fn expected(answer: Answer) -> Document {
    match answer {
        Answer::Issue(value) => Document::Issue(value),
        Answer::Pull(value) => Document::Pull(value),
        Answer::Comment(value) => Document::Comment(value),
        Answer::Review(value) => Document::Review(value),
        Answer::Status(value) => Document::Status(value),
        Answer::Branch(value) => Document::Branch(value),
        Answer::Error(_)
        | Answer::Token(_)
        | Answer::Tokens(_)
        | Answer::User(_)
        | Answer::ApiSettings(_)
        | Answer::Repository(_)
        | Answer::Search { .. }
        | Answer::Issues(_)
        | Answer::Comments(_)
        | Answer::Files(_)
        | Answer::Compare { .. }
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Job { .. }
        | Answer::Jobs(_)
        | Answer::Pulls(_)
        | Answer::Reviews(_)
        | Answer::ReviewComments(_)
        | Answer::Branches(_)
        | Answer::Protection(_)
        | Answer::Protections(_)
        | Answer::Collaborators(_)
        | Answer::Permission(_)
        | Answer::Org(_)
        | Answer::Team(_)
        | Answer::Teams(_)
        | Answer::Members(_)
        | Answer::CombinedStatus { .. } => panic!("unexpected write answer"),
    }
}
#[test]
fn write_transcripts_decode_to_their_documents_and_rows() {
    let requests = forgejo::write_cases();
    for (case, answer) in answers() {
        let request = &requests.iter().find(|(name, _)| *name == case).expect("request case").1;
        let transcript = forgejo::transcript(case);
        let decoded = forgejo::decode(request, &transcript.body);
        assert_eq!(decoded.document, Ok(expected(answer)), "{case}");
        assert!(decoded.rows.is_empty(), "{case}: single document");
    }
}
#[test]
fn write_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape() {
    let requests = forgejo::write_cases();
    for (case, answer) in answers() {
        let request = &requests.iter().find(|(name, _)| *name == case).expect("request case").1;
        let encoded = server::encode(&answer, &forgejo::limits()).expect("answer encoding");
        let shape = forgejo::shape(&forgejo::transcript(case).body);
        for field in forgejo::shape(&encoded) {
            assert!(shape.contains(&field), "{case}: {field:?}");
        }
        let decoded = forgejo::decode(request, &encoded);
        assert_eq!(decoded.document, Ok(expected(answer)), "{case}");
        assert!(decoded.rows.is_empty(), "{case}");
    }
}
#[test]
fn write_transcript_mutations_are_malformed_naming_the_field() {
    let request =
        forgejo::write_cases().into_iter().find(|(case, _)| *case == "post-comment").expect("fixture value").1;
    let body = forgejo::transcript("post-comment").body;
    let mut value: serde_json::Value = serde_json::from_slice(&body).expect("fixture value");
    value.as_object_mut().expect("fixture value").remove("id");
    let decoded = forgejo::decode(&request, &serde_json::to_vec(&value).expect("fixture value"));
    assert_eq!(decoded.document, Err(temper_forge_forgejo::decode::Malformed::Missing { field: b"id" }));
}
#[test]
fn write_bodies_enforce_fields_types_and_bounds() {
    let mut limits = forgejo::limits();
    limits.inner = 1;
    limits.string_bytes = 8;
    limits.body_bytes = 16;
    let rows: [(&[u8], &[u8], &str); 7] = [
        (b"issues", br#"{"body":"a"}"#, "title"),
        (b"issues", br#"{"title":"a","title":"b","body":"a"}"#, "title"),
        (b"issues", br#"{"title":2,"body":"a"}"#, "title"),
        (b"issues", br#"{"title":"123456789","body":"a"}"#, "title"),
        (b"issues", br#"{"title":"a","body":"12345678901234567"}"#, "body"),
        (b"pulls/1/requested_reviewers", br#"{"reviewers":["a","b"]}"#, "reviewers"),
        (b"pulls/1/requested_reviewers", br#"{"reviewers":["123456789"]}"#, "reviewers"),
    ];
    for (tail, body, field) in rows {
        let target = [b"/api/v1/repos/a/b/".as_slice(), tail].concat();
        let error = forgejo::decode_route(temper_forge_forgejo::request::Method::Post, &target, body, &limits)
            .expect_err("malformed request");
        assert!(error.contains(&format!("{:?}", field.as_bytes())), "{field}: {error}");
    }
}
use temper_forge_forgejo::{
    document::ErrorBody,
    failure::{self, Refusal},
};
fn failure_cases() -> Vec<(&'static str, u16, &'static [u8], Option<Refusal>)> {
    vec![
        ("merge-stale-head", 409, b"head out of date", Some(Refusal::HeadOutOfDate)),
        ("update-conflict", 409, b"merge failed because of conflict", Some(Refusal::UpdateConflict)),
        ("merge-closed", 405, b"", Some(Refusal::MergeClosed)),
        ("merge-conflict", 405, b"Please try again later", Some(Refusal::MergeConflict)),
        ("merge-protected", 405, b"not allowed to merge [reason: Does not have enough approvals]", Some(Refusal::MergeProtected)),
        ("merge-style-refused", 405, b"squash is not an allowed merge style for this repository", Some(Refusal::MergeStyle)),
        ("create-issue-empty-title", 422, b"[Title]: Required", Some(Refusal::Empty)),
        ("create-branch-exists", 409, b"The branch already exists.", None),
        ("no-token", 401, b"token is required", None),
        ("wrong-token", 401, b"token is malformed: token contains an invalid number of segments\naccess token does not exist [sha: 000000000000000000000000000000000000000c]\ntoken is malformed: token contains an invalid number of segments\ntask with token \"000000000000000000000000000000000000000c\": resource does not exist\nauthorized integration: parse JWT error: token is malformed: token contains an invalid number of segments", None),
        ("owner-permission", 403, b"Only admins can query all permissions, repo admins can query all repo permissions, collaborators can query only their own", None),
        ("writer-branch-protection", 403, b"user should be an owner or a collaborator with admin write of a repository", None),
        ("writer-branch-protections", 403, b"user should be an owner or a collaborator with admin write of a repository", None),
        ("delete-protected-branch", 403, b"can not delete default branch", None),
        ("archived-write", 423, b"<Repository 3:fixture/archived> is archived", None),
        ("missing-issue", 404, b"IsErrIssueNotExist", None),
        ("actions-job-log-past-last-step", 404, b"step index out of range", None),
        ("raw-file-missing", 404, b"The target couldn't be found.", None),
        ("stranger-repository-by-path", 404, b"The target couldn't be found.", None),
        ("stranger-repository-by-id", 404, b"The target couldn't be found.", None),
        ("delete-missing-branch", 500, b"", None),
        ("moved-repository", 301, b"", None),
        ("open-pull-exists", 409, b"pull request already exists for these targets [id: 1, issue_id: 1, head_repo_id: 1, base_repo_id: 1, head_branch: feature/slash, base_branch: main]", None),
    ]
}
#[test]
fn failure_transcripts_name_their_refusal_by_status_and_message() {
    for (case, status, message, expected) in failure_cases() {
        let transcript = forgejo::transcript(case);
        assert_eq!(transcript.status, status, "{case}");
        let decoded = forgejo::decode_error(&transcript.body);
        if case == "moved-repository" {
            assert!(decoded.document.is_err(), "{case}: no JSON message");
        } else {
            assert_eq!(
                decoded.document,
                Ok(Document::Error(ErrorBody { message: Body::Kept(Box::from(message)) })),
                "{case}"
            );
        }
        assert!(decoded.rows.is_empty(), "{case}");
        assert_eq!(failure::refusal(status, message), expected, "{case}");
    }
}
#[test]
fn error_answers_meet_the_client_with_the_transcripts_shape() {
    for (case, _, message, _) in failure_cases() {
        if case == "wrong-token" || case == "moved-repository" {
            continue;
        }
        let expected = ErrorBody { message: Body::Kept(Box::from(message)) };
        let encoded = server::encode(&Answer::Error(expected.clone()), &forgejo::limits()).expect("error encoding");
        let shape = forgejo::shape(&forgejo::transcript(case).body);
        for field in forgejo::shape(&encoded) {
            assert!(shape.contains(&field), "{case}: {field:?}");
        }
        assert_eq!(forgejo::decode_error(&encoded).document, Ok(Document::Error(expected)), "{case}");
    }
}
