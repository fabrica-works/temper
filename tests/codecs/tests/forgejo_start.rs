use skein_forge::forgejo::{
    client,
    document::{ApiSettings, Document, MergeStyle, Repository, Row, User},
    id::ObjectFormat,
    request::Request,
    server::{self, Answer},
};
use temper_codecs_world::forgejo;
fn repository() -> Repository {
    Repository {
        id: 1,
        owner: User { id: 1, login: Box::from(*b"fixture"), is_admin: false, full_name: Box::from([]) },
        name: Box::from(*b"specimen"),
        object_format: ObjectFormat::Sha1,
        default_branch: Box::from(*b"main"),
        default_merge_style: MergeStyle::Merge,
        allow_merge_commits: true,
        allow_squash_merge: true,
        allow_rebase: true,
        html_url: Box::from(*b"http://127.0.0.1:3000/fixture/specimen"),
    }
}
fn cases() -> [(&'static str, Request, Answer); 6] {
    [
        (
            "user",
            Request::User,
            Answer::User(User { id: 2, login: Box::from(*b"writer"), is_admin: false, full_name: Box::from([]) }),
        ),
        (
            "user-fixture",
            Request::User,
            Answer::User(User { id: 1, login: Box::from(*b"fixture"), is_admin: true, full_name: Box::from([]) }),
        ),
        (
            "settings-api",
            Request::ApiSettings,
            Answer::ApiSettings(ApiSettings { max_response_items: 2, default_paging_num: 30 }),
        ),
        ("repository-by-id", Request::RepositoryById { id: 1 }, Answer::Repository(repository())),
        ("repository-by-path", Request::Repository { at: forgejo::specimen() }, Answer::Repository(repository())),
        (
            "users-search-uid",
            Request::UserSearch { uid: 1, limit: Some(1) },
            Answer::Search {
                ok: true,
                data: Box::from([User {
                    id: 1,
                    login: Box::from(*b"fixture"),
                    is_admin: false,
                    full_name: Box::from([]),
                }]),
            },
        ),
    ]
}
fn expected(answer: Answer) -> (Document, Vec<Row>) {
    match answer {
        Answer::User(value) => (Document::User(value), vec![]),
        Answer::ApiSettings(value) => (Document::ApiSettings(value), vec![]),
        Answer::Repository(value) => (Document::Repository(value), vec![]),
        Answer::Search { ok, data } => (Document::Search { ok }, data.into_vec().into_iter().map(Row::User).collect()),
        Answer::Token(_)
        | Answer::Tokens(_)
        | Answer::Error(_)
        | Answer::Comment(_)
        | Answer::Status(_)
        | Answer::Review(_)
        | Answer::Files(_)
        | Answer::Compare { .. }
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Job { .. }
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
        | Answer::Issues(_)
        | Answer::Issue(_)
        | Answer::Comments(_)
        | Answer::Pull(_)
        | Answer::Org(_)
        | Answer::Team(_)
        | Answer::Teams(_)
        | Answer::Hook(_)
        | Answer::Hooks(_)
        | Answer::Members(_)
        | Answer::CombinedStatus { .. } => panic!("a start answer"),
    }
}
#[test]
fn start_transcripts_decode_to_their_documents() {
    for (case, request, answer) in cases() {
        let transcript = forgejo::transcript(case);
        assert_eq!(transcript.status, 200, "{case}");
        let (document, rows) = expected(answer);
        let decoded = forgejo::decode(&request, &transcript.body);
        assert_eq!(decoded.document, Ok(document), "{case}");
        assert_eq!(decoded.rows, rows, "{case}");
    }
}
#[test]
fn start_requests_encode_as_the_tool_sent_them() {
    for (case, request, _) in cases() {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &forgejo::limits()).expect("encoded request");
        assert_eq!(encoded.method, transcript.method, "{case}");
        assert_eq!(&*encoded.target, transcript.target, "{case}");
        assert_eq!(encoded.body.as_deref().unwrap_or(b""), transcript.request_body, "{case}");
    }
}
#[test]
fn start_requests_route_on_the_servers_side_to_the_same_request() {
    for (case, request, _) in cases() {
        assert_eq!(forgejo::route(&forgejo::transcript(case)), request, "{case}");
    }
}
#[test]
fn start_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape() {
    for (case, request, answer) in cases() {
        let transcript = forgejo::transcript(case);
        let bytes = server::encode(&answer, &forgejo::limits()).expect("server answer");
        assert!(forgejo::shape(&bytes).is_subset(&forgejo::shape(&transcript.body)), "{case}");
        let (document, rows) = expected(answer);
        let decoded = forgejo::decode(&request, &bytes);
        assert_eq!(decoded.document, Ok(document), "{case}");
        assert_eq!(decoded.rows, rows, "{case}");
    }
}
