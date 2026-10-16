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
#[test]
fn a_refused_setup_write_reads_its_status_and_message() {
    let limits = forgejo::profile_limits();
    for (case, status) in [
        ("setup-user-create-again", 422),
        ("setup-user-create-by-writer", 403),
        ("setup-user-person-basic", 403),
        ("setup-user-person-token", 403),
    ] {
        let t = forgejo::transcript(case);
        assert_eq!(t.status, status, "{case}");
        let result = forgejo::decode_answer(client::error_decoder(&limits), &t.body, &limits);
        let Ok(Document::Error(message)) = result.document else { panic!("{case}: error document") };
        let expected: serde_json::Value = serde_json::from_slice(&t.body).expect("error body");
        assert_eq!(
            message.message,
            temper_forge_forgejo::document::Body::Kept(Box::from(
                expected["message"].as_str().expect("message").as_bytes()
            )),
            "{case}"
        );
    }
}
#[test]
fn setup_bodies_enforce_members_types_and_bounds() {
    let limits = forgejo::profile_limits();
    let base = serde_json::json!({"username":"bot", "email":"bot@example.invalid", "password":"secret", "full_name":"temper:bot", "must_change_password":false});
    for (key, value, expected) in [
        ("must_change_password", Some(serde_json::json!("false")), "must_change_password"),
        ("password", Some(serde_json::json!("x".repeat(513))), "password"),
        ("full_name", None, "full_name"),
    ] {
        let mut body = base.clone();
        match value {
            Some(value) => body[key] = value,
            None => {
                body.as_object_mut().expect("object").remove(key);
            }
        }
        let refusal = forgejo::decode_route(Method::Post, b"/api/v1/admin/users", body.to_string().as_bytes(), &limits)
            .expect_err("bad body");
        assert!(refusal.contains(&format!("field: {:?}", expected.as_bytes())), "{key}: {refusal}");
    }
}
