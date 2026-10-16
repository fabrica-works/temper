use crate::{
    Outgoing, RUNNER_VERSION, Settings, Token, VERSION, binary::names_forgejo, http, instance::ini, names_runner,
};
use std::path::Path;
#[test]
fn the_app_ini_names_loopback_sqlite_and_the_page_cap() {
    let rendered = ini(Path::new("/scratch"), 3000, &Settings::small(), "fixture", "secret", "internal");
    assert_eq!(
        rendered,
        "APP_NAME = temper throwaway\nRUN_USER = fixture\nWORK_PATH = /scratch\n[server]\nPROTOCOL = http\nHTTP_ADDR = 127.0.0.1\nHTTP_PORT = 3000\nROOT_URL = http://127.0.0.1:3000/\nAPP_DATA_PATH = /scratch/data\nDISABLE_SSH = true\nOFFLINE_MODE = true\n[database]\nDB_TYPE = sqlite3\nPATH = /scratch/forgejo.db\n[repository]\nROOT = /scratch/repositories\nDEFAULT_BRANCH = main\n[security]\nINSTALL_LOCK = true\nSECRET_KEY = secret\nINTERNAL_TOKEN = internal\n[service]\nDISABLE_REGISTRATION = true\n[log]\nMODE = console\nLEVEL = Error\n[git]\nPATH = /usr/bin/git\n[actions]\nENABLED = true\n[api]\nMAX_RESPONSE_ITEMS = 2\n"
    );
}
#[test]
fn a_chunked_answer_is_read_whole_and_unframed() {
    let response = http::parse(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\ntest\r\n3;x=y\r\ning\r\n0\r\n\r\n",
        Vec::new(),
    )
    .expect("valid chunks");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"testing");
}
#[test]
fn a_recorded_request_holds_no_token() {
    let token = Token::new("0123456789abcdef0123456789abcdef01234567".into());
    let outgoing = Outgoing::get("/api/v1/user", Some(&token));
    let sent = http::request(3000, &outgoing, Some("token MASKED"));
    assert_eq!(sent, b"GET /api/v1/user HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nAuthorization: token MASKED\r\nConnection: close\r\n\r\n");
    assert_eq!(format!("{token:?}"), "Token { length: 40 }");
}

#[test]
fn forgejo_s_line_with_its_build_details_names_the_pin() {
    for line in [
        format!(
            "forgejo version {VERSION}+gitea-1.22.0 (release name {VERSION}) built with GNU Make 4.4.1, go1.26.9 : bindata, timetzdata, sqlite, sqlite_unlock_notify"
        ),
        format!("forgejo version {VERSION}"),
    ] {
        assert!(names_forgejo(&line), "{line}");
    }
}
#[test]
fn a_forgejo_version_extending_the_pin_is_refused() {
    for line in [
        format!("forgejo version 1{VERSION}+gitea-1.22.0"),
        format!("forgejo version {VERSION}0+gitea-1.22.0"),
        format!("forgejo version {VERSION}-rc1"),
        format!("gitea version {VERSION}+gitea-1.22.0"),
    ] {
        assert!(!names_forgejo(&line), "{line}");
    }
}
#[test]
fn the_runner_s_version_line_names_the_runner_pin() {
    assert!(names_runner(&format!("forgejo-runner version v{RUNNER_VERSION}")));
}
#[test]
fn a_runner_version_containing_the_pin_is_refused() {
    for line in [
        format!("forgejo-runner version v1{RUNNER_VERSION}"),
        format!("forgejo-runner version v{RUNNER_VERSION}0"),
        format!("forgejo-runner version v{RUNNER_VERSION}-dev"),
        format!("forgejo-runner version {RUNNER_VERSION}"),
    ] {
        assert!(!names_runner(&line), "{line}");
    }
}
