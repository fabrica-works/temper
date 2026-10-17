mod fixtures;
use std::boxed::Box;
use temper_codecs_world::forgejo;
use temper_forge_forgejo::{
    client,
    decode::Malformed,
    document::{Access, Document, Row},
    request::{Method, Request},
    server::{self, Answer},
};
#[test]
fn setup_requests_encode_as_the_tool_sent_them() {
    let limits = forgejo::profile_limits();
    for (case, request) in forgejo::setup_cases() {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &limits).expect("setup encoding");
        assert_eq!(encoded.method, transcript.method, "{case}");
        assert_eq!(&*encoded.target, transcript.target, "{case}");
        assert_eq!(encoded.body.as_deref().unwrap_or_default(), transcript.request_body, "{case}");
    }
}
#[test]
fn setup_requests_route_on_the_servers_side_to_the_same_request() {
    let limits = forgejo::profile_limits();
    for (case, request) in forgejo::setup_cases() {
        let t = forgejo::transcript(case);
        assert_eq!(
            forgejo::decode_route(t.method, &t.target, &t.request_body, &limits).expect("setup request").0,
            request,
            "{case}"
        );
    }
}
fn answers() -> Vec<(&'static str, Answer)> {
    let bot = || Answer::User(fixtures::user(fixtures::BOT, b"bot", false, b"temper:bot"));
    let person = || Answer::User(fixtures::user(fixtures::PERSON, b"person", false, b"Person"));
    vec![
        ("setup-user-create", bot()),
        ("setup-user-of", bot()),
        ("setup-user-admin", Answer::User(fixtures::user(1, b"fixture", true, b""))),
        ("setup-user-create-person", person()),
        ("setup-user-person-cleared", person()),
        ("setup-token-make", Answer::Token(fixtures::api_token(true))),
        ("setup-token-make-git", Answer::Token(fixtures::git_token(true))),
        ("setup-token-list", Answer::Tokens(Box::new([fixtures::git_token(false)]))),
        ("setup-token-list-page-2", Answer::Tokens(Box::new([fixtures::api_token(false)]))),
        ("setup-token-as-the-bot", bot()),
        ("setup-token-git-as-the-bot", bot()),
        ("setup-scope-read-user", Answer::User(fixtures::user(1, b"fixture", true, b""))),
    ]
}
fn expected(answer: Answer) -> (Document, Vec<Row>) {
    if let Answer::User(user) = answer {
        return (Document::User(user), vec![]);
    }
    if let Answer::Token(token) = answer {
        return (Document::Token(token), vec![]);
    }
    let Answer::Tokens(tokens) = answer else { panic!("a setup answer") };
    (Document::Listed { skipped: 0 }, tokens.into_vec().into_iter().map(Row::Token).collect())
}
fn request(case: &str) -> Request {
    forgejo::setup_cases().into_iter().find(|(name, _)| *name == case).expect("setup case").1
}
#[test]
fn setup_transcripts_decode_to_their_documents() {
    let limits = forgejo::profile_limits();
    for (case, answer) in answers() {
        let decoder = client::decoder(&request(case), &limits).expect("setup decoder");
        let result = forgejo::decode_answer(decoder, &forgejo::transcript(case).body, &limits);
        let (document, rows) = expected(answer);
        assert_eq!(result.document, Ok(document), "{case}");
        assert_eq!(result.rows, rows, "{case}");
    }
    for reply in org_answers().into_iter().chain(hook_answers()) {
        check_document(&reply, &forgejo::transcript(reply.case).body);
    }
}
#[test]
fn setup_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape() {
    let limits = forgejo::profile_limits();
    for (case, answer) in answers() {
        let encoded = server::encode(&answer, &limits).expect("setup encoding");
        let decoder = client::decoder(&request(case), &limits).expect("setup decoder");
        let result = forgejo::decode_answer(decoder, &encoded, &limits);
        let (document, rows) = expected(answer);
        assert_eq!(result.document, Ok(document), "{case}");
        assert_eq!(result.rows, rows, "{case}");
        assert!(forgejo::shape(&encoded).is_subset(&forgejo::shape(&forgejo::transcript(case).body)), "{case}");
    }
    for reply in org_answers().into_iter().chain(hook_answers()) {
        let encoded = server::encode(&reply.answer, &limits).expect("org encoding");
        check_document(&reply, &encoded);
        assert!(
            forgejo::shape(&encoded).is_subset(&forgejo::shape(&forgejo::transcript(reply.case).body)),
            "{}",
            reply.case
        );
    }
}
fn refused_answer(case: &str, status: u16, message: &[u8]) {
    let limits = forgejo::profile_limits();
    let transcript = forgejo::transcript(case);
    assert_eq!(transcript.status, status);
    assert_eq!(
        forgejo::decode_answer(client::error_decoder(&limits), &transcript.body, &limits).document,
        Ok(Document::Error(temper_forge_forgejo::document::ErrorBody {
            message: temper_forge_forgejo::document::Body::Kept(Box::from(message)),
        }))
    );
}
#[test]
fn creating_an_existing_user_reads_its_refusal() {
    refused_answer("setup-user-create-again", 422, b"user already exists [name: bot]");
}
#[test]
fn creating_a_user_as_a_writer_reads_the_admin_refusal() {
    refused_answer("setup-user-create-by-writer", 403, b"user should be the site admin");
}
#[test]
fn basic_auth_for_a_person_who_must_change_password_reads_the_refusal() {
    refused_answer(
        "setup-user-person-basic",
        403,
        b"You must change your password. Change it at: http://127.0.0.1:3000//user/change_password",
    );
}
#[test]
fn a_token_for_a_person_who_must_change_password_reads_the_refusal() {
    refused_answer(
        "setup-user-person-token",
        403,
        b"You must change your password. Change it at: http://127.0.0.1:3000//user/change_password",
    );
}
fn user_body() -> serde_json::Value {
    serde_json::json!({"username":"bot", "email":"bot@example.invalid", "password":"secret", "full_name":"temper:bot", "must_change_password":false})
}
fn refused(body: &serde_json::Value, expected: Malformed) {
    assert_eq!(
        forgejo::decode_refusal(
            Method::Post,
            b"/api/v1/admin/users",
            body.to_string().as_bytes(),
            &forgejo::profile_limits()
        ),
        expected
    );
}
#[test]
fn a_text_must_change_password_is_refused_naming_its_field() {
    let mut body = user_body();
    body["must_change_password"] = serde_json::json!("false");
    refused(&body, Malformed::Shape { field: b"must_change_password" });
}
#[test]
fn a_password_past_string_bytes_is_refused_naming_its_field() {
    let mut body = user_body();
    body["password"] = serde_json::json!("x".repeat(513));
    refused(&body, Malformed::TooLong { field: b"password" });
}
#[test]
fn a_missing_full_name_is_refused_naming_its_field() {
    let mut body = user_body();
    body.as_object_mut().expect("object").remove("full_name");
    refused(&body, Malformed::Missing { field: b"full_name" });
}

fn token_body() -> serde_json::Value {
    serde_json::json!({"name":"temper-api", "scopes":["read:user"]})
}
fn refused_token(body: &serde_json::Value, expected: Malformed) {
    assert_eq!(
        forgejo::decode_refusal(
            Method::Post,
            b"/api/v1/admin/users/bot/tokens",
            body.to_string().as_bytes(),
            &forgejo::profile_limits()
        ),
        expected
    );
}
#[test]
fn a_scope_list_that_is_text_is_refused_naming_scopes() {
    let mut body = token_body();
    body["scopes"] = serde_json::json!("read:user");
    refused_token(&body, Malformed::Shape { field: b"scopes" });
}
#[test]
fn a_scope_list_past_the_profiles_inner_is_refused_naming_scopes() {
    let mut body = token_body();
    body["scopes"] = serde_json::json!(vec!["read:user"; 33]);
    refused_token(&body, Malformed::Bound { field: b"scopes" });
}

#[test]
fn making_a_token_of_an_existing_name_reads_the_refusal() {
    refused_answer("setup-token-make-again", 400, b"access token name has been used already");
}
#[test]
fn a_git_token_without_read_user_reads_the_scope_refusal() {
    refused_answer("setup-token-git-alone", 403, b"token does not have at least one of required scope(s): [read:user]");
}
#[test]
fn deleting_a_missing_token_reads_the_refusal() {
    refused_answer("setup-token-delete-missing", 404, b"The target couldn't be found.");
}
#[test]
fn an_api_token_without_write_issue_reads_the_scope_refusal() {
    refused_answer(
        "setup-token-narrow-write",
        403,
        b"token does not have at least one of required scope(s): [write:issue]",
    );
}
#[test]
fn an_admin_token_without_write_admin_reads_the_scope_refusal() {
    refused_answer("setup-scope-no-admin", 403, b"token does not have at least one of required scope(s): [read:admin]");
}
#[test]
fn an_admin_token_without_a_user_scope_reads_the_refusal() {
    refused_answer("setup-scope-no-user", 403, b"token does not have at least one of required scope(s): [read:user]");
}
#[test]
fn a_deleted_tokens_refusal_is_read_with_its_masked_value() {
    refused_answer("setup-token-deleted-use", 401, b"token is malformed: token contains an invalid number of segments\naccess token does not exist [sha: 0000000000000000000000000000000000000004]\ntoken is malformed: token contains an invalid number of segments\ntask with token \"0000000000000000000000000000000000000004\": resource does not exist\nauthorized integration: parse JWT error: token is malformed: token contains an invalid number of segments");
}

struct OrgAnswer {
    case: &'static str,
    request: temper_forge_forgejo::request::Request,
    document: Document,
    rows: Vec<Row>,
    answer: Answer,
}
fn check_document(reply: &OrgAnswer, body: &[u8]) {
    let limits = forgejo::profile_limits();
    let reader = client::decoder(&reply.request, &limits).expect("org answer");
    let decoded = forgejo::decode_answer(reader, body, &limits);
    assert_eq!(decoded.document, Ok(reply.document.clone()), "{}", reply.case);
    assert_eq!(decoded.rows, reply.rows, "{}", reply.case);
}
fn org_answers() -> Vec<OrgAnswer> {
    let mut cases = Vec::new();
    for case in ["setup-org-made", "setup-org-read"] {
        cases.push(org_answer(case, Document::Org(fixtures::org()), vec![], Answer::Org(fixtures::org())));
    }
    cases.push(org_answer("setup-team-made", Document::Team(fixtures::team()), vec![], Answer::Team(fixtures::team())));
    let mut edited = fixtures::team();
    edited.spec.permission = Access::Read;
    edited.spec.includes_all_repositories = false;
    edited.spec.units.pulls = Access::Read;
    edited.spec.units.actions = Access::None;
    cases.push(org_answer("setup-team-edit", Document::Team(edited.clone()), vec![], Answer::Team(edited)));
    for (case, teams) in [
        ("setup-team-list-new-org", vec![fixtures::owners()]),
        ("setup-team-list", vec![fixtures::owners(), fixtures::team()]),
        ("setup-team-list-page-1", vec![fixtures::owners()]),
        ("setup-team-list-page-3", vec![]),
    ] {
        let rows = teams.iter().cloned().map(Row::Team).collect();
        cases.push(org_answer(case, Document::Listed { skipped: 0 }, rows, Answer::Teams(teams.into_boxed_slice())));
    }
    for (case, users) in
        [("setup-team-members-none", vec![]), ("setup-team-members", vec![fixtures::user(2, b"writer", false, b"")])]
    {
        let rows = users.iter().cloned().map(Row::User).collect();
        cases.push(org_answer(case, Document::Listed { skipped: 0 }, rows, Answer::Members(users.into_boxed_slice())));
    }
    cases
}
fn org_answer(case: &'static str, document: Document, rows: Vec<Row>, answer: Answer) -> OrgAnswer {
    let request = forgejo::setup_cases().into_iter().find(|(name, _)| *name == case).expect("org case").1;
    OrgAnswer { case, request, document, rows, answer }
}
fn hook_answers() -> Vec<OrgAnswer> {
    let mut cases = Vec::new();
    for (case, id, other, narrow) in [
        ("setup-hook-made", 1, false, false),
        ("setup-hook-made-twin", 2, false, false),
        ("setup-hook-made-other", 3, true, false),
        ("setup-hook-patched", 1, false, false),
        ("setup-hook-patched-narrow", 1, false, true),
    ] {
        let hook = fixtures::hook(id, other, narrow);
        cases.push(org_answer(case, Document::Hook(hook.clone()), vec![], Answer::Hook(hook)));
    }
    for (case, hooks) in [
        ("setup-hook-list-none", vec![]),
        (
            "setup-hook-list",
            vec![fixtures::hook(1, false, false), fixtures::hook(2, false, false), fixtures::hook(3, true, false)],
        ),
        ("setup-hook-list-page-1", vec![fixtures::hook(1, false, false)]),
        ("setup-hook-list-page-4", vec![]),
    ] {
        let rows = hooks.iter().cloned().map(Row::Hook).collect();
        cases.push(org_answer(case, Document::Listed { skipped: 0 }, rows, Answer::Hooks(hooks.into_boxed_slice())));
    }
    cases
}
#[test]
fn patching_a_missing_hook_reads_its_refusal() {
    refused_answer("setup-hook-patch-missing", 404, b"The target couldn't be found.");
}
#[test]
fn deleting_a_missing_hook_reads_its_refusal() {
    refused_answer("setup-hook-delete-missing", 404, b"The target couldn't be found.");
}
