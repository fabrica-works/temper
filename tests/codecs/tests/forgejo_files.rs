use skein_lib::Wall;
use temper_codecs_world::forgejo;
use temper_forge_forgejo::{
    client,
    decode::Malformed,
    document::{ChangedFile, CombinedStatus, CommitStatus, Compare, Document, FileStatus, Job, Row, Run, StatusState},
    request::Request,
    server::{self, Answer},
};
fn file(n: u32) -> ChangedFile {
    ChangedFile {
        filename: format!("file-{n}.txt").into_bytes().into_boxed_slice(),
        previous_filename: None,
        status: FileStatus::Added,
    }
}
fn run() -> Run {
    Run { id: 1, index_in_repo: 1, commit_sha: forgejo::sha(16) }
}
fn jobs() -> Box<[Job]> {
    Box::from([Job { id: 1, run_id: 1, attempt: 1 }, Job { id: 2, run_id: 1, attempt: 1 }])
}
fn status(context: &[u8], status: StatusState, description: &[u8], target_url: &[u8], at: u64) -> CommitStatus {
    CommitStatus {
        creator: None,
        updated_at: Wall::from_nanos(at),
        context: Box::from(context),
        status,
        description: Box::from(description),
        target_url: Box::from(target_url),
    }
}
fn raw() -> Request {
    Request::Raw { at: forgejo::specimen(), path: Box::from(*b"file-0.txt"), reference: forgejo::sha(4) }
}
fn log() -> Request {
    Request::JobLog { at: forgejo::specimen(), job: 2, attempt: 1 }
}
fn cases() -> Vec<(&'static str, Request, Option<Answer>, u16)> {
    vec![
        (
            "pull-files-page-1",
            Request::PullFiles { at: forgejo::specimen(), number: 1, page: forgejo::page(1, 2) },
            Some(Answer::Files(Box::from([file(0), file(1)]))),
            200,
        ),
        (
            "pull-files-page-2",
            Request::PullFiles { at: forgejo::specimen(), number: 1, page: forgejo::page(2, 2) },
            Some(Answer::Files(Box::from([file(2)]))),
            200,
        ),
        (
            "compare",
            Request::Compare { at: forgejo::specimen(), before: forgejo::sha(6), after: forgejo::sha(4) },
            Some(Answer::Compare {
                compare: Compare { total_commits: 3 },
                commits: Box::from([forgejo::sha(4), forgejo::sha(10), forgejo::sha(8)]),
                files: Box::from([file(2), file(1), file(0)]),
            }),
            200,
        ),
        (
            "compare-reverse",
            Request::Compare { at: forgejo::specimen(), before: forgejo::sha(4), after: forgejo::sha(6) },
            Some(Answer::Compare {
                compare: Compare { total_commits: 0 },
                commits: Box::from([]),
                files: Box::from([]),
            }),
            200,
        ),
        ("raw-file", raw(), None, 200),
        ("raw-file-range", raw(), None, 206),
        (
            "actions-combined-status",
            Request::CombinedStatus { at: forgejo::specimen(), commit: forgejo::sha(16), page: forgejo::page(1, 2) },
            Some(Answer::CombinedStatus {
                state: CombinedStatus { state: Some(StatusState::Failure), total_count: 2 },
                sha: Some(forgejo::sha(16)),
                statuses: Box::from([
                    status(
                        b"conformance / pass (push)",
                        StatusState::Success,
                        b"Successful in 0s",
                        b"/fixture/specimen/actions/runs/1/jobs/0",
                        1_577_817_011_000_000_000,
                    ),
                    status(
                        b"conformance / fail (push)",
                        StatusState::Failure,
                        b"Failing after 0s",
                        b"/fixture/specimen/actions/runs/1/jobs/1",
                        1_577_817_012_000_000_000,
                    ),
                ]),
            }),
            200,
        ),
        (
            "actions-runs",
            Request::Runs {
                at: forgejo::specimen(),
                head_sha: forgejo::sha(16),
                run_number: 1,
                page: forgejo::page(1, 1),
            },
            Some(Answer::Runs { total_count: 1, runs: Box::from([run()]) }),
            200,
        ),
        ("actions-run", Request::Run { at: forgejo::specimen(), run: 1 }, Some(Answer::Run(run())), 200),
        ("actions-jobs", Request::Jobs { at: forgejo::specimen(), run: 1 }, Some(Answer::Jobs(jobs())), 200),
        ("actions-job-log", log(), None, 200),
        ("actions-job-log-range", log(), None, 206),
    ]
}
fn expected(answer: Answer) -> (Document, Vec<Row>) {
    match answer {
        Answer::Files(v) => (Document::Listed { skipped: 0 }, v.into_vec().into_iter().map(Row::File).collect()),
        Answer::Compare { compare, commits, files } => {
            let mut rows = commits.into_vec().into_iter().map(Row::Commit).collect::<Vec<_>>();
            rows.extend(files.into_vec().into_iter().map(Row::File));
            (Document::Compare(compare), rows)
        }
        Answer::Runs { total_count, runs } => {
            (Document::Runs { total_count }, runs.into_vec().into_iter().map(Row::Run).collect())
        }
        Answer::Run(v) => (Document::Run(v), vec![]),
        Answer::Jobs(v) => (Document::Listed { skipped: 0 }, v.into_vec().into_iter().map(Row::Job).collect()),
        Answer::CombinedStatus { state, statuses, .. } => {
            (Document::CombinedStatus(state), statuses.into_vec().into_iter().map(Row::Status).collect())
        }
        Answer::Error(_)
        | Answer::Comment(_)
        | Answer::Status(_)
        | Answer::Review(_)
        | Answer::User(_)
        | Answer::ApiSettings(_)
        | Answer::Repository(_)
        | Answer::Search { .. }
        | Answer::Issues(_)
        | Answer::Issue(_)
        | Answer::Comments(_)
        | Answer::Pull(_)
        | Answer::Pulls(_)
        | Answer::Reviews(_)
        | Answer::ReviewComments(_)
        | Answer::Branch(_)
        | Answer::Branches(_)
        | Answer::Protection(_)
        | Answer::Protections(_)
        | Answer::Collaborators(_)
        | Answer::Permission(_) => panic!("file family answer"),
    }
}
#[test]
fn file_transcripts_decode_to_their_documents_and_rows() {
    for (case, request, answer, status) in cases() {
        let transcript = forgejo::transcript(case);
        assert_eq!(transcript.status, status, "{case}");
        if let Some(answer) = answer {
            let (document, rows) = expected(answer);
            let decoded = forgejo::decode(&request, &transcript.body);
            assert_eq!(decoded.document, Ok(document), "{case}");
            assert_eq!(decoded.rows, rows, "{case}");
        } else {
            assert!(client::decoder(&request, &forgejo::limits()).is_none(), "{case}: bytes");
        }
    }
}
#[test]
fn file_requests_encode_as_the_tool_sent_them() {
    for (case, request, _, _) in cases() {
        let transcript = forgejo::transcript(case);
        let encoded = client::encode(&request, &forgejo::limits()).expect("request");
        assert_eq!(encoded.method, transcript.method, "{case}");
        assert_eq!(encoded.target.as_ref(), transcript.target, "{case}");
        assert_eq!(encoded.body.as_deref().unwrap_or(b""), transcript.request_body, "{case}");
    }
}
#[test]
fn file_requests_route_on_the_servers_side_to_the_same_request() {
    for (case, request, _, _) in cases() {
        assert_eq!(forgejo::route(&forgejo::transcript(case)), request, "{case}");
    }
}
#[test]
fn file_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape() {
    for (case, request, answer, _) in cases() {
        if let Some(answer) = answer {
            let transcript = forgejo::transcript(case);
            let bytes = server::encode(&answer, &forgejo::limits()).expect("answer");
            assert!(forgejo::shape(&bytes).is_subset(&forgejo::shape(&transcript.body)), "{case}");
            let (document, rows) = expected(answer);
            let decoded = forgejo::decode(&request, &bytes);
            assert_eq!(decoded.document, Ok(document), "{case}");
            assert_eq!(decoded.rows, rows, "{case}");
        } else {
            assert!(client::decoder(&request, &forgejo::limits()).is_none(), "{case}: bytes");
        }
    }
}
#[test]
fn file_transcript_mutations_are_malformed_naming_the_field() {
    for (case, field, mutation, expected) in [
        ("pull-files-page-1", "status", 0, Malformed::Value { field: b"status" }),
        ("compare", "sha", 1, Malformed::Id { field: b"sha" }),
        ("actions-run", "commit_sha", 2, Malformed::Missing { field: b"commit_sha" }),
        ("actions-jobs", "attempt", 3, Malformed::Range { field: b"attempt" }),
        ("actions-combined-status", "creator", 4, Malformed::Missing { field: b"creator" }),
    ] {
        let transcript = forgejo::transcript(case);
        let mut json: serde_json::Value = serde_json::from_slice(&transcript.body).expect("JSON");
        match mutation {
            0 => json[0][field] = serde_json::json!("changed"),
            1 => json["commits"][0][field] = serde_json::json!("000000000000000000000000000000000000000"),
            2 => {
                json.as_object_mut().expect("run").remove(field);
            }
            3 => json[0][field] = serde_json::json!(4_294_967_296u64),
            4 => {
                json["statuses"][0].as_object_mut().expect("status").remove(field);
            }
            _ => panic!("mutation"),
        }
        assert_eq!(
            forgejo::decode(&forgejo::route(&transcript), &serde_json::to_vec(&json).expect("JSON")).document,
            Err(expected),
            "{case}/{field}"
        );
    }
}
#[test]
fn a_failing_actions_status_links_its_run_and_job() {
    let transcript = forgejo::transcript("actions-combined-status");
    let decoded = forgejo::decode(&forgejo::route(&transcript), &transcript.body);
    let Row::Status(failed) = &decoded.rows[1] else {
        panic!("status row");
    };
    assert_eq!(failed.status, StatusState::Failure);
    let link = temper_forge_forgejo::header::job_link(&failed.target_url, b"http://127.0.0.1:3000/fixture/specimen");
    assert_eq!(link, Some((1, 1)));
    let (run_number, job_index) = link.expect("this repository's job link");
    let transcript = forgejo::transcript("actions-runs");
    let runs = forgejo::decode(&forgejo::route(&transcript), &transcript.body);
    let Row::Run(run) = runs.rows[0] else {
        panic!("run row");
    };
    assert_eq!(run.index_in_repo, run_number);
    assert_eq!(run.commit_sha, forgejo::sha(16));
    let transcript = forgejo::transcript("actions-jobs");
    let jobs = forgejo::decode(&forgejo::route(&transcript), &transcript.body);
    let Row::Job(job) = jobs.rows[usize::try_from(job_index).expect("job index fits")] else {
        panic!("job row");
    };
    assert_eq!(job.run_id, run.id);
    assert_eq!(job.id, 2);
    assert_eq!(job.attempt, 1);
}
