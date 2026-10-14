use skein_lib::Wall;
use temper_codecs_world::forgejo;
use temper_forge_forgejo::{
    client,
    decode::Malformed,
    document::{
        Body, Branch, BranchProtection, CollaboratorPermission, CombinedStatus, Document, Permission, Review,
        ReviewComment, ReviewState, Row,
    },
    request::Request,
    server::{self, Answer},
};
fn review(id: u64, state: ReviewState, official: bool, dismissed: bool, key: &[u8]) -> Review {
    Review {
        id,
        user: forgejo::user(2),
        state,
        commit_id: forgejo::sha(4),
        body: Body::Kept(Box::from(key)),
        submitted_at: Some(Wall::from_nanos(1_577_817_005_000_000_000)),
        updated_at: Some(Wall::from_nanos(1_577_817_005_000_000_000)),
        official,
        dismissed,
    }
}
fn branch(name: &[u8], sha: u64) -> Branch {
    Branch { name: Box::from(name), commit: forgejo::sha(sha), effective_branch_protection_name: None }
}
fn protection() -> BranchProtection {
    BranchProtection {
        rule_name: Box::from(*b"main"),
        enable_status_check: true,
        status_check_contexts: Box::from([Box::from(b"fixture/test".as_slice())]),
        required_approvals: 1,
        dismiss_stale_approvals: true,
    }
}
fn cases() -> Vec<(&'static str, Request, Option<Answer>)> {
    vec![
        (
            "reviews",
            Request::Reviews { at: forgejo::specimen(), number: 1, page: forgejo::page(1, 2) },
            Some(Answer::Reviews(Box::from([
                review(1, ReviewState::Approved, false, true, b"<!-- temper:key 01000003 -->\nreview text"),
                review(2, ReviewState::RequestChanges, true, false, b"<!-- temper:key 01000004 -->\nreview text"),
            ]))),
        ),
        (
            "statuses-page-2",
            Request::CombinedStatus { at: forgejo::specimen(), commit: forgejo::sha(4), page: forgejo::page(2, 2) },
            Some(Answer::CombinedStatus {
                state: CombinedStatus { state: None, total_count: 0 },
                sha: None,
                statuses: Box::from([]),
            }),
        ),
        (
            "review-comments",
            Request::ReviewComments { at: forgejo::specimen(), number: 1, review: 5 },
            Some(Answer::ReviewComments(Box::from([ReviewComment {
                id: 18,
                user: forgejo::user(1),
                path: Box::from(*b"file-0.txt"),
                position: 1,
                body: Body::Kept(Box::from(*b"remark text")),
            }]))),
        ),
        (
            "branch",
            Request::Branch { at: forgejo::specimen(), name: Box::from(*b"main") },
            Some(Answer::Branch(branch(b"main", 3))),
        ),
        (
            "branch-with-slash",
            Request::Branch { at: forgejo::specimen(), name: Box::from(*b"feature/slash") },
            Some(Answer::Branch(branch(b"feature/slash", 4))),
        ),
        (
            "branches-page-1",
            Request::Branches { at: forgejo::specimen(), page: forgejo::page(1, 2) },
            Some(Answer::Branches(Box::from([branch(b"landing", 7), branch(b"mergeable", 5)]))),
        ),
        (
            "branches-page-2",
            Request::Branches { at: forgejo::specimen(), page: forgejo::page(2, 2) },
            Some(Answer::Branches(Box::from([branch(b"writer-head", 9), branch(b"main", 3)]))),
        ),
        (
            "branch-protection",
            Request::Protection { at: forgejo::specimen(), name: Box::from(*b"main") },
            Some(Answer::Protection(protection())),
        ),
        (
            "branch-protections",
            Request::Protections { at: forgejo::specimen() },
            Some(Answer::Protections(Box::from([protection()]))),
        ),
        (
            "collaborators",
            Request::Collaborators { at: forgejo::specimen(), page: forgejo::page(1, 2) },
            Some(Answer::Collaborators(Box::from([forgejo::user(2)]))),
        ),
        (
            "collaborator-permission",
            Request::Permission { at: forgejo::specimen(), login: Box::from(*b"writer") },
            Some(Answer::Permission(CollaboratorPermission { permission: Permission::Write, user: forgejo::user(2) })),
        ),
        ("owner-permission", Request::Permission { at: forgejo::specimen(), login: Box::from(*b"fixture") }, None),
    ]
}
fn expected(answer: Answer) -> (Document, Vec<Row>) {
    match answer {
        Answer::Reviews(v) => (Document::Listed { skipped: 0 }, v.into_vec().into_iter().map(Row::Review).collect()),
        Answer::ReviewComments(v) => {
            (Document::Listed { skipped: 0 }, v.into_vec().into_iter().map(Row::ReviewComment).collect())
        }
        Answer::Branches(v) => (Document::Listed { skipped: 0 }, v.into_vec().into_iter().map(Row::Branch).collect()),
        Answer::Protections(v) => {
            (Document::Listed { skipped: 0 }, v.into_vec().into_iter().map(Row::Protection).collect())
        }
        Answer::Collaborators(v) => {
            (Document::Listed { skipped: 0 }, v.into_vec().into_iter().map(Row::User).collect())
        }
        Answer::Branch(v) => (Document::Branch(v), vec![]),
        Answer::Protection(v) => (Document::Protection(v), vec![]),
        Answer::Permission(v) => (Document::Permission(v), vec![]),
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
        | Answer::User(_)
        | Answer::ApiSettings(_)
        | Answer::Repository(_)
        | Answer::Search { .. }
        | Answer::Issues(_)
        | Answer::Issue(_)
        | Answer::Comments(_)
        | Answer::Pull(_)
        | Answer::Pulls(_) => panic!("review family answer"),
    }
}
#[test]
fn review_transcripts_decode_to_their_documents_and_rows() {
    for (case, request, answer) in cases() {
        let transcript = forgejo::transcript(case);
        if let Some(answer) = answer {
            assert_eq!(transcript.status, 200, "{case}");
            let (document, rows) = expected(answer);
            let decoded = forgejo::decode(&request, &transcript.body);
            assert_eq!(decoded.document, Ok(document), "{case}");
            assert_eq!(decoded.rows, rows, "{case}");
        } else {
            assert_eq!(transcript.status, 403, "{case}");
        }
    }
}
#[test]
fn review_requests_encode_as_the_tool_sent_them() {
    for (case, request, _) in cases() {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &forgejo::limits()).expect("request");
        assert_eq!(encoded.method, transcript.method, "{case}");
        assert_eq!(encoded.target.as_ref(), transcript.target, "{case}");
        assert_eq!(encoded.body.as_deref().unwrap_or(b""), transcript.request_body, "{case}");
    }
}
#[test]
fn review_requests_route_on_the_servers_side_to_the_same_request() {
    for (case, request, _) in cases() {
        assert_eq!(forgejo::route(&forgejo::transcript(case)), request, "{case}");
    }
}
#[test]
fn review_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape() {
    for (case, request, answer) in cases() {
        let transcript = forgejo::transcript(case);
        if let Some(answer) = answer {
            let bytes = server::encode(&answer, &forgejo::limits()).expect("answer");
            assert!(forgejo::shape(&bytes).is_subset(&forgejo::shape(&transcript.body)), "{case}");
            let (document, rows) = expected(answer);
            let decoded = forgejo::decode(&request, &bytes);
            assert_eq!(decoded.document, Ok(document), "{case}");
            assert_eq!(decoded.rows, rows, "{case}");
        } else {
            assert_eq!(transcript.status, 403, "{case}");
        }
    }
}
#[test]
fn review_transcript_mutations_are_malformed_naming_the_field() {
    for (case, field, mutation, expected) in [
        ("reviews", "state", 0, Malformed::Value { field: b"state" }),
        ("reviews", "commit_id", 1, Malformed::Id { field: b"commit_id" }),
        ("review-comments", "position", 2, Malformed::Range { field: b"position" }),
        ("branch", "commit", 3, Malformed::Missing { field: b"id" }),
        ("branch-protection", "status_check_contexts", 4, Malformed::Bound { field: b"status_check_contexts" }),
        ("collaborator-permission", "permission", 5, Malformed::Value { field: b"permission" }),
    ] {
        let transcript = forgejo::transcript(case);
        let mut json: serde_json::Value = serde_json::from_slice(&transcript.body).expect("JSON");
        match mutation {
            0 => json[0][field] = serde_json::json!("approved"),
            1 => json[0][field] = serde_json::json!("000000000000000000000000000000000000000"),
            2 => json[0][field] = serde_json::json!(4_294_967_296u64),
            3 => {
                json[field].as_object_mut().expect("commit").remove("id");
            }
            4 => json[field] = serde_json::json!(vec!["context"; 65]),
            5 => json[field] = serde_json::json!("maintainer"),
            _ => panic!("mutation"),
        }
        assert_eq!(
            forgejo::decode(&forgejo::route(&transcript), &serde_json::to_vec(&json).expect("mutation JSON")).document,
            Err(expected),
            "{case}/{field}"
        );
    }
}
#[test]
fn open_pulls_rows_are_the_pull_transcripts_documents() {
    let request = Request::Pulls { at: forgejo::specimen(), page: forgejo::page(1, 2) };
    let (body, pulls) = open_pulls();
    let expected = pulls.iter().cloned().map(Row::Pull).collect::<Vec<_>>();
    let decoded = forgejo::decode(&request, &body);
    assert_eq!(decoded.document, Ok(Document::Listed { skipped: 0 }));
    assert_eq!(decoded.rows, expected);
    let bytes = server::encode(&Answer::Pulls(pulls.into_boxed_slice()), &forgejo::limits()).expect("pull list");
    let decoded = forgejo::decode(&request, &bytes);
    assert_eq!(decoded.document, Ok(Document::Listed { skipped: 0 }));
    assert_eq!(decoded.rows, expected);
    assert!(forgejo::shape(&bytes).is_subset(&forgejo::shape(&body)));
    let encoded = client::encode(&request, &forgejo::limits()).expect("request");
    assert_eq!(encoded.method, temper_forge_forgejo::request::Method::Get);
    assert_eq!(&*encoded.target, b"/api/v1/repos/fixture/specimen/pulls?state=open&sort=oldest&page=1&limit=2");
    let server::Route::Ready(routed) =
        server::route(encoded.method, &encoded.target, &forgejo::limits()).expect("route")
    else {
        panic!("read route has no body")
    };
    assert_eq!(routed, request);
}

fn open_pulls() -> (Vec<u8>, Vec<temper_forge_forgejo::document::PullRequest>) {
    let mut body = vec![b'['];
    let mut pulls = Vec::new();
    for (i, case) in ["pull", "pull-merged"].into_iter().enumerate() {
        let transcript = forgejo::transcript(case);
        if i != 0 {
            body.push(b',');
        }
        body.extend_from_slice(&transcript.body);
        let Document::Pull(pull) =
            forgejo::decode(&forgejo::route(&transcript), &transcript.body).document.expect("pull")
        else {
            panic!("pull document");
        };
        assert_eq!(pull.head.repo_id, 1, "{case} head");
        assert_eq!(pull.base.repo_id, 1, "{case} base");
        pulls.push(pull);
    }
    body.push(b']');
    (body, pulls)
}
