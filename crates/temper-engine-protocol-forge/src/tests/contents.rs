//! Contents plans through HTTP bytes, with literal answers and exchange counts.
use super::fixtures;
use alloc::boxed::Box;
use skein_lib::Duration;
use temper_engine_domain_forge_client::api::{Answer, Error, File, Op, Read};
const HEAD: [u8; 32] = {
    let mut bytes = [0; 32];
    bytes[0] = 1;
    bytes
};
const BASE: [u8; 32] = {
    let mut bytes = [0; 32];
    bytes[0] = 3;
    bytes
};
const PULL: &[u8] = br#"{"number":1,"state":"open","head":{"repo_id":1,"ref":"work","sha":"0100000000000000000000000000000000000000"},"base":{"repo_id":1,"ref":"main","sha":"0300000000000000000000000000000000000000"},"merged":false,"merge_commit_sha":null,"merge_base":"0300000000000000000000000000000000000000","mergeable":true,"requested_reviewers":[]}"#;
#[test]
fn pull_files_at_another_head_fail_stale_reading_no_file() {
    let mut h = fixtures::Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    h.call(1, fixtures::REPO, Op::Read(Read::PullFiles { number: 1, head: [0; 32], page: 1 }));
    h.run(Duration::from_millis(1));
    let lines = h.request_lines(fixtures::FORGE).into_boxed();
    assert_eq!(lines.last().expect("pull request").as_ref(), b"GET /api/v1/repos/fixture/specimen/pulls/1 HTTP/1.1");
    h.answer(fixtures::FORGE, &fixtures::ok(PULL));
    h.run(Duration::from_millis(10));
    let answer = h.answered(1).expect("one answer");
    assert_eq!((answer.cost, answer.result.clone()), (1, Err(Error::Stale)));
}
#[test]
fn a_files_missing_side_is_absent_and_its_read_counted() {
    let mut h = fixtures::Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    h.call(1, fixtures::REPO, Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 1 }));
    h.answer(fixtures::FORGE, &fixtures::ok(PULL));
    h.answer(fixtures::FORGE, b"HTTP/1.1 200 OK\r\nContent-Length: 40\r\nX-HasMore: true\r\n\r\n[{\"filename\":\"one\",\"status\":\"modified\"}]");
    h.answer(fixtures::FORGE, &fixtures::status(404));
    h.answer(fixtures::FORGE, &fixtures::ok(b"after"));
    h.run(Duration::from_millis(10));
    let answer = h.answered(1).expect("one answer");
    assert_eq!(
        (answer.cost, answer.result.clone()),
        (
            4,
            Ok(Answer::PullFiles {
                head: HEAD,
                files: Box::new([File {
                    path: Box::from(&b"one"[..]),
                    before: None,
                    after: Some(Box::from(&b"after"[..]))
                }]),
                more: true
            })
        )
    );
    let lines = h.request_lines(fixtures::FORGE).into_boxed();
    assert_eq!(
        lines[4].as_ref(),
        b"GET /api/v1/repos/fixture/specimen/raw/one?ref=0300000000000000000000000000000000000000 HTTP/1.1"
    );
    assert_eq!(
        lines[5].as_ref(),
        b"GET /api/v1/repos/fixture/specimen/raw/one?ref=0100000000000000000000000000000000000000 HTTP/1.1"
    );
}
#[test]
fn pull_files_choose_added_deleted_and_renamed_sides() {
    for (name, document, first, file, cost) in side_cases() {
        let mut h = fixtures::Harness::ready();
        h.known(fixtures::REPO, fixtures::specimen());
        h.call(1, fixtures::REPO, Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 2 }));
        h.answer(fixtures::FORGE, &fixtures::ok(PULL));
        h.answer(fixtures::FORGE, &fixtures::ok(document));
        assert!(skein_lib::bytes::find(&h.request_bytes(fixtures::FORGE), first).is_some(), "{name}: first side");
        h.answer(fixtures::FORGE, &fixtures::ok(b"text"));
        if name == "renamed" {
            h.answer(fixtures::FORGE, &fixtures::ok(b"text"));
        }
        h.run(Duration::from_millis(10));
        let answer = h.answered(1).expect("one answer");
        assert_eq!(
            (answer.cost, answer.result.clone()),
            (cost, Ok(Answer::PullFiles { head: HEAD, files: Box::new([file]), more: false })),
            "{name}"
        );
    }
}
type SideCase = (&'static str, &'static [u8], &'static [u8], File, u32);
fn side_cases() -> [SideCase; 3] {
    [
        (
            "added",
            br#"[{"filename":"new","status":"added"}]"#,
            b"raw/new?ref=0100000000000000000000000000000000000000",
            File { path: Box::from(&b"new"[..]), before: None, after: Some(Box::from(&b"text"[..])) },
            3,
        ),
        (
            "deleted",
            br#"[{"filename":"old","status":"deleted"}]"#,
            b"raw/old?ref=0300000000000000000000000000000000000000",
            File { path: Box::from(&b"old"[..]), before: Some(Box::from(&b"text"[..])), after: None },
            3,
        ),
        (
            "renamed",
            br#"[{"filename":"new","previous_filename":"old","status":"renamed"}]"#,
            b"raw/old?ref=0300000000000000000000000000000000000000",
            File {
                path: Box::from(&b"new"[..]),
                before: Some(Box::from(&b"text"[..])),
                after: Some(Box::from(&b"text"[..])),
            },
            4,
        ),
    ]
}
#[test]
fn comparisons_read_both_directions_and_both_file_sides() {
    let mut h = fixtures::Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    h.call(1, fixtures::REPO, Op::Read(Read::Compare { before: BASE, after: HEAD }));
    h.answer(fixtures::FORGE, &fixtures::ok(br#"{"total_commits":1,"commits":[{"sha":"0100000000000000000000000000000000000000"}],"files":[{"filename":"file","status":"modified"}]}"#));
    h.answer(fixtures::FORGE, &fixtures::ok(br#"{"total_commits":0,"commits":[],"files":[]}"#));
    h.answer(fixtures::FORGE, &fixtures::ok(b"before"));
    h.answer(fixtures::FORGE, &fixtures::ok(b"after"));
    h.run(Duration::from_millis(10));
    let answer = h.answered(1).expect("comparison");
    assert_eq!(
        (answer.cost, answer.result.clone()),
        (
            4,
            Ok(Answer::Compare {
                before: BASE,
                after: HEAD,
                contains_before: true,
                files: Box::new([File {
                    path: Box::from(&b"file"[..]),
                    before: Some(Box::from(&b"before"[..])),
                    after: Some(Box::from(&b"after"[..]))
                }]),
                commits: Box::new([HEAD])
            })
        )
    );
    let lines = h.request_lines(fixtures::FORGE).into_boxed();
    assert_eq!(lines[3].as_ref(), b"GET /api/v1/repos/fixture/specimen/compare/0100000000000000000000000000000000000000...0300000000000000000000000000000000000000 HTTP/1.1");
}
#[test]
fn file_reads_send_the_range_and_keep_only_the_requested_bytes() {
    for (name, body, max, range, expected, truncated) in [
        ("short", &b"abc"[..], 4, &b"Range: bytes=0-4\r\n"[..], &b"abc"[..], false),
        ("full", &b"abcd"[..], 4, &b"Range: bytes=0-4\r\n"[..], &b"abcd"[..], false),
        ("long", &b"abcde"[..], 4, &b"Range: bytes=0-4\r\n"[..], &b"abcd"[..], true),
        ("empty", &b"x"[..], 0, &b"Range: bytes=0-0\r\n"[..], &b""[..], true),
    ] {
        let mut h = fixtures::Harness::ready();
        h.known(fixtures::REPO, fixtures::specimen());
        h.call(1, fixtures::REPO, Op::Read(Read::File { head: HEAD, path: Box::from(&b"a b/c"[..]), max_bytes: max }));
        h.run(Duration::from_millis(1));
        let request = h.request_bytes(fixtures::FORGE);
        assert!(skein_lib::bytes::find(&request, range).is_some(), "{name}: range");
        assert!(skein_lib::bytes::find(&request, b"raw/a%20b/c?").is_some(), "{name}: path");
        h.answer(fixtures::FORGE, &fixtures::ok(body));
        h.run(Duration::from_millis(10));
        let answer = h.answered(1).expect("file");
        assert_eq!(
            (answer.cost, answer.result.clone()),
            (1, Ok(Answer::File { head: HEAD, path: Box::from(&b"a b/c"[..]), bytes: Box::from(expected), truncated })),
            "{name}"
        );
    }
}
#[test]
fn raw_sides_at_the_answer_bound_fit_and_one_byte_more_is_too_large() {
    for (name, body, expected) in bounded_cases() {
        let mut shared = fixtures::shared();
        shared.client.answer_bytes = 4096;
        let mut h = fixtures::Harness::with(fixtures::limits(), shared, fixtures::settings());
        h.connected(fixtures::FORGE);
        h.answer(fixtures::FORGE, &fixtures::user_ok());
        h.answer(fixtures::FORGE, &fixtures::settings_ok());
        h.known(fixtures::REPO, fixtures::specimen());
        h.call(1, fixtures::REPO, Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 1 }));
        h.answer(fixtures::FORGE, &fixtures::ok(PULL));
        h.answer(fixtures::FORGE, &fixtures::ok(br#"[{"filename":"x","status":"added"}]"#));
        h.answer(fixtures::FORGE, &fixtures::ok(body));
        h.run(Duration::from_millis(10));
        let answer = h.answered(1).expect("bounded answer");
        assert_eq!(answer.result, expected, "{name}");
    }
}
type BoundedCase = (&'static str, &'static [u8], Result<Answer, Error>);
fn bounded_cases() -> [BoundedCase; 2] {
    [
        (
            "exact",
            &[b'x'; 4047],
            Ok(Answer::PullFiles {
                head: HEAD,
                files: Box::new([File {
                    path: Box::from(&b"x"[..]),
                    before: None,
                    after: Some(Box::from([b'x'; 4047])),
                }]),
                more: false,
            }),
        ),
        ("past", &[b'x'; 4048], Err(Error::TooLarge)),
    ]
}
#[test]
fn multiple_changed_files_each_advance_to_their_own_sides() {
    let mut h = fixtures::Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    h.call(1, fixtures::REPO, Op::Read(Read::PullFiles { number: 1, head: HEAD, page: 1 }));
    h.answer(fixtures::FORGE, &fixtures::ok(PULL));
    h.answer(
        fixtures::FORGE,
        &fixtures::ok(br#"[{"filename":"new","status":"added"},{"filename":"old","status":"deleted"}]"#),
    );
    h.answer(fixtures::FORGE, &fixtures::ok(b"new text"));
    h.answer(fixtures::FORGE, &fixtures::ok(b"old text"));
    h.run(Duration::from_millis(10));
    let answer = h.answered(1).expect("files");
    let expected = Answer::PullFiles {
        head: HEAD,
        files: Box::new([
            File { path: Box::from(&b"new"[..]), before: None, after: Some(Box::from(&b"new text"[..])) },
            File { path: Box::from(&b"old"[..]), before: Some(Box::from(&b"old text"[..])), after: None },
        ]),
        more: false,
    };
    assert_eq!((answer.cost, answer.result.clone()), (4, Ok(expected)));
}
#[test]
fn file_reads_accept_partial_content_and_preserve_raw_bytes() {
    let mut h = fixtures::Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    h.call(1, fixtures::REPO, Op::Read(Read::File { head: HEAD, path: Box::from(&b"file"[..]), max_bytes: 2 }));
    h.answer(
        fixtures::FORGE,
        b"HTTP/1.1 206 Partial Content\r\nContent-Length: 3\r\nContent-Range: bytes 0-2/9\r\n\r\n\xff\x00x",
    );
    h.run(Duration::from_millis(10));
    let answer = h.answered(1).expect("file");
    assert_eq!(
        (answer.cost, answer.result.clone()),
        (
            1,
            Ok(Answer::File {
                head: HEAD,
                path: Box::from(&b"file"[..]),
                bytes: Box::from(&b"\xff\x00"[..]),
                truncated: true
            })
        )
    );
}
#[test]
fn contents_sizes_agree_with_the_clients_measure() {
    for (name, answer, expected) in measured_contents() {
        assert_eq!(crate::reads::size::answer(&answer, 8), Ok(expected), "{name}: running measure");
        assert_eq!(
            temper_engine_domain_forge_client::answer_bytes_unbounded(&answer),
            Some(expected),
            "{name}: client measure"
        );
    }
}
fn measured_contents() -> [(&'static str, Answer, u64); 3] {
    [
        (
            "file",
            Answer::File { head: HEAD, path: Box::from(&b"x"[..]), bytes: Box::from(&b"abc"[..]), truncated: false },
            4,
        ),
        (
            "pull files",
            Answer::PullFiles {
                head: HEAD,
                files: Box::new([File {
                    path: Box::from(&b"x"[..]),
                    before: Some(Box::from(&b"a"[..])),
                    after: Some(Box::from(&b"bc"[..])),
                }]),
                more: false,
            },
            52,
        ),
        (
            "comparison",
            Answer::Compare {
                before: BASE,
                after: HEAD,
                contains_before: true,
                files: Box::new([File {
                    path: Box::from(&b"x"[..]),
                    before: Some(Box::from(&b"a"[..])),
                    after: Some(Box::from(&b"bc"[..])),
                }]),
                commits: Box::new([HEAD]),
            },
            84,
        ),
    ]
}
