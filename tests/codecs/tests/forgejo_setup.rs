use std::boxed::Box;
use temper_codecs_world::forgejo;
use temper_forge_forgejo::{
    client,
    document::{Document, User},
    request::Method,
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
fn answers() -> Vec<(&'static str, User)> {
    vec![
        (
            "setup-user-create",
            User { id: 4, login: Box::from(*b"bot"), is_admin: false, full_name: Box::from(*b"temper:bot") },
        ),
        (
            "setup-user-of",
            User { id: 4, login: Box::from(*b"bot"), is_admin: false, full_name: Box::from(*b"temper:bot") },
        ),
        ("setup-user-admin", User { id: 1, login: Box::from(*b"fixture"), is_admin: true, full_name: Box::from([]) }),
        (
            "setup-user-create-person",
            User { id: 5, login: Box::from(*b"person"), is_admin: false, full_name: Box::from(*b"Person") },
        ),
        (
            "setup-user-person-cleared",
            User { id: 5, login: Box::from(*b"person"), is_admin: false, full_name: Box::from(*b"Person") },
        ),
    ]
}
#[test]
fn setup_transcripts_decode_to_their_documents() {
    let limits = forgejo::profile_limits();
    for (case, user) in answers() {
        let decoder = client::decoder(&temper_forge_forgejo::request::Request::User, &limits).expect("user");
        assert_eq!(
            forgejo::decode_answer(decoder, &forgejo::transcript(case).body, &limits).document,
            Ok(Document::User(user)),
            "{case}"
        );
    }
}
#[test]
fn setup_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape() {
    let limits = forgejo::profile_limits();
    for (case, user) in answers() {
        let encoded = server::encode(&Answer::User(user.clone()), &limits).expect("user encoding");
        let decoder = client::decoder(&temper_forge_forgejo::request::Request::User, &limits).expect("user");
        assert_eq!(forgejo::decode_answer(decoder, &encoded, &limits).document, Ok(Document::User(user)), "{case}");
        assert!(forgejo::shape(&encoded).is_subset(&forgejo::shape(&forgejo::transcript(case).body)), "{case}");
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
fn refused(body: &serde_json::Value, expected: &str) {
    assert_eq!(
        forgejo::decode_route(
            Method::Post,
            b"/api/v1/admin/users",
            body.to_string().as_bytes(),
            &forgejo::profile_limits()
        )
        .expect_err("bad body"),
        expected
    );
}
#[test]
fn a_text_must_change_password_is_refused_naming_its_field() {
    let mut body = user_body();
    body["must_change_password"] = serde_json::json!("false");
    refused(
        &body,
        "body: Shape { field: [109, 117, 115, 116, 95, 99, 104, 97, 110, 103, 101, 95, 112, 97, 115, 115, 119, 111, 114, 100] }",
    );
}
#[test]
fn a_password_past_string_bytes_is_refused_naming_its_field() {
    let mut body = user_body();
    body["password"] = serde_json::json!("x".repeat(513));
    refused(&body, "body: TooLong { field: [112, 97, 115, 115, 119, 111, 114, 100] }");
}
#[test]
fn a_missing_full_name_is_refused_naming_its_field() {
    let mut body = user_body();
    body.as_object_mut().expect("object").remove("full_name");
    refused(&body, "body: Missing { field: [102, 117, 108, 108, 95, 110, 97, 109, 101] }");
}
