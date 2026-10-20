pub mod support;
use skein_lib::Duration;
use skein_shell::Clock;
use support::{ADMIN, Peer};
use temper_forge_setup_shell::reconcile::{bound, reconcile};

fn done(settings: temper_forge_setup_shell::reconcile::Settings, peer: Peer) {
    const BOT: &[u8] = br#"{"id":42,"login":"temper-bot","is_admin":false,"full_name":"temper:bot"}"#;
    for file in ["api_token", "git_token"] {
        temper_forge_setup_shell::files::write_private(&settings.directory.join(file), b"kept-token")
            .expect("private token");
    }
    std::thread::scope(|scope| {
        let server = scope.spawn(|| peer.serve_after(0, &[(200, ADMIN), (200, BOT), (200, BOT), (200, BOT)]));
        reconcile(settings, &Clock::new()).expect("Done");
        assert_eq!(server.join().expect("peer thread").requests.len(), 4);
    });
}

#[test]
fn done_with_a_listener_draws_a_private_hex_secret() {
    use std::os::unix::fs::PermissionsExt;
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.given.orgs = Box::new([]);
    settings.given.listener = Some(peer.address());
    let path = settings.directory.join("webhook_secret");
    done(settings, peer);
    let bytes = std::fs::read(&path).expect("secret kept");
    assert_eq!(bytes.len(), 64);
    assert!(bytes.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)));
    assert_eq!(std::fs::metadata(path).expect("private file").permissions().mode() & 0o777, 0o600);
}

#[test]
fn done_keeps_the_secret_byte_for_byte_without_rewriting() {
    use std::os::unix::fs::MetadataExt;
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.given.orgs = Box::new([]);
    settings.given.listener = Some(peer.address());
    let path = settings.directory.join("webhook_secret");
    let kept = [b'a'; 64];
    temper_forge_setup_shell::files::write_private(&path, &kept).expect("kept secret");
    let inode = std::fs::metadata(&path).expect("kept file").ino();
    done(settings, peer);
    assert!(std::fs::read(&path).expect("secret stays") == kept);
    assert_eq!(std::fs::metadata(path).expect("same file").ino(), inode);
}

#[test]
fn done_without_a_listener_makes_no_secret() {
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.given.orgs = Box::new([]);
    let path = settings.directory.join("webhook_secret");
    done(settings, peer);
    assert!(!path.exists());
}
#[test]
fn a_first_request_is_made_again_until_the_api_answers() {
    let peer = Peer::reserve();
    let settings = support::settings(peer.address());
    let heard = std::thread::scope(|scope| {
        let server = scope.spawn(|| peer.serve_after(120, &[(403, br#"{"message":"no"}"#)]));
        let refused = reconcile(settings, &Clock::new()).expect_err("403 stops run");
        assert_eq!(refused.line(), "temper forge reconcile: forge 1: admin temper-admin: HTTP 403\n");
        server.join().expect("peer thread")
    });
    assert_eq!(heard.requests.len(), 1);
    assert_eq!(heard.requests[0].line, "GET /api/v1/user HTTP/1.1");
}
#[test]
fn the_admins_token_is_sent_as_a_token_header() {
    let heard = support::refused(403, b"", "temper forge reconcile: forge 1: admin temper-admin: HTTP 403\n");
    assert!(heard.requests[0].headers.contains(&("authorization".into(), "token admin-token-value".into())));
}
#[test]
fn a_run_ends_at_its_deadline_when_nothing_answers() {
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.given.limits.deadline = Duration::from_millis(300); // A short wait keeps the host test focused.
    let refused = reconcile(settings, &Clock::new()).expect_err("no peer");
    assert_eq!(refused.line(), "temper forge reconcile: forge 1: Forgejo did not answer within 300 ms\n");
}
#[test]
fn a_forge_that_goes_silent_after_its_answer_ends_the_run_at_the_deadline() {
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.given.limits.deadline = Duration::from_millis(200); // The whole run, including its second request, is bounded.
    let clock = Clock::new();
    let began = clock.now().now;
    let heard = std::thread::scope(|scope| {
        let server = scope.spawn(|| peer.serve_after(0, &[(200, ADMIN)]));
        let refused = reconcile(settings, &clock).expect_err("silent bot request");
        assert_eq!(refused.line(), "temper forge reconcile: forge 1: bot temper-bot: unavailable\n");
        server.join().expect("peer thread")
    });
    assert!(clock.now().now.saturating_since(began) < Duration::from_millis(400));
    assert_eq!(heard.requests.last().expect("bot request").line, "GET /api/v1/users/temper-bot HTTP/1.1");
}
#[test]
fn a_later_request_that_fails_is_not_made_again() {
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.given.listener = Some(peer.address());
    let directory = settings.directory.clone();
    let heard = std::thread::scope(|scope| {
        let server = scope.spawn(|| peer.serve_after(0, &[(200, ADMIN), (503, b"")]));
        let refused = reconcile(settings, &Clock::new()).expect_err("later 503");
        assert_eq!(refused.line(), "temper forge reconcile: forge 1: bot temper-bot: HTTP 503\n");
        server.join().expect("peer thread")
    });
    assert_eq!(heard.requests.len(), 2);
    assert_eq!(heard.requests[1].line, "GET /api/v1/users/temper-bot HTTP/1.1");
    assert!(!directory.join("state").exists());
    assert!(!directory.join("webhook_secret").exists());
}
#[test]
fn a_forbidden_request_names_its_step_and_holds_no_secret() {
    let heard =
        support::refused(403, b"admin-token-value", "temper forge reconcile: forge 1: admin temper-admin: HTTP 403\n");
    assert_eq!(heard.requests.len(), 1);
}
#[test]
fn a_deleted_admin_is_refused_naming_its_step() {
    let heard = support::refused(401, b"deleted", "temper forge reconcile: forge 1: admin temper-admin: HTTP 401\n");
    assert_eq!(heard.requests.len(), 1);
}
#[test]
fn an_admin_token_readable_by_others_is_refused_naming_it() {
    use std::os::unix::fs::PermissionsExt;
    let peer = Peer::reserve();
    let settings = support::settings(peer.address());
    std::fs::set_permissions(&settings.admin_token, std::fs::Permissions::from_mode(0o644)).expect("unsafe token");
    let refused = reconcile(settings, &Clock::new()).expect_err("unsafe token");
    assert_eq!(
        refused.line(),
        "temper forge reconcile: forge 1: admin_token: readable by others or reached through a link\n"
    );
}
#[test]
fn each_file_is_read_within_its_own_bound() {
    use temper_forge_setup::place::File;
    let peer = Peer::reserve();
    let settings = support::settings(peer.address());
    for (file, expected) in [
        (File::AdminToken, settings.token_bytes),
        (File::ApiToken, settings.token_bytes),
        (File::GitToken, settings.token_bytes),
        (File::User, 21),
        (File::WebhookSecret, settings.hook_secret_bytes),
    ] {
        assert_eq!(bound(&settings, file), expected);
    }
}
#[test]
fn io_short_of_the_http_intake_is_refused_before_any_request() {
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.io.intake = skein_http::client::largest_read(&settings.http) - 1;
    let refused = reconcile(settings, &Clock::new()).expect_err("short intake");
    assert_eq!(refused.line(), "temper forge reconcile: forge 1: io.intake: below the http client's largest read\n");
}
#[test]
fn io_short_of_the_http_output_is_refused_before_any_request() {
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.io.output = skein_http::client::largest_room(&settings.http) - 1;
    let refused = reconcile(settings, &Clock::new()).expect_err("short output");
    assert_eq!(refused.line(), "temper forge reconcile: forge 1: io.output: below the http client's largest read\n");
}

#[test]
fn invalid_given_values_are_refused_before_io() {
    let peer = Peer::reserve();
    let mut settings = support::settings(peer.address());
    settings.given.bot_login = Box::new([]);
    settings.io.output = 0;
    let refused = reconcile(settings, &Clock::new()).expect_err("invalid given");
    assert_eq!(refused.line(), "temper forge reconcile: forge 1: application.setup[0].bot.login: invalid form\n");
    let temper_forge_setup_shell::reconcile::Refused::Setup { forge, .. } = refused else {
        panic!("setup refused before io");
    };
    assert_eq!(forge, 1);
}
