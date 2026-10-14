//! The shipped binary, on a scratch deployment, drained on a signal.
pub mod support;
use skein_io::kernel::{Exit, Signal};
use skein_lib::Duration;

#[test]
fn temper_serve_prints_its_link_and_drains_on_a_signal() {
    let scratch = support::Scratch::new("drain");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let ended = support::until_link_then(scratch.path(), Signal::Terminate, Duration::from_secs(60));
    let link = support::printed_link(&ended.stdout).expect("one printed line");
    assert!(link.starts_with(b"http://127.0.0.1:"), "a loopback link: {}", String::from_utf8_lossy(link));
    assert!(skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some(), "the deployment's sign-in link");
    assert_eq!(ended.exit, Some(Exit::Code(0)), "drained: {}", String::from_utf8_lossy(&ended.stderr));
    scratch.remove();
}

#[test]
fn temper_open_signs_in_prints_the_inbox_and_exits_on_q() {
    let scratch = support::Scratch::new("temper_open_inbox");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let mut serve = support::serve(scratch.path());
    let link = support::terminal::link(&mut serve, support::WITHIN);
    let open = support::terminal::open(scratch.path(), &scratch.path().join("client"), link.as_ref());
    let mut typist = support::terminal::Typist::new(open, serve, link);
    typist.wait_for(b"== Inbox == Live", support::WITHIN);
    typist.type_line(b"q");
    assert_eq!(typist.exit(support::WITHIN), 0, "stderr: {}", typist.stderr());
    assert!(support::private(&scratch.path().join("client/token")), "the bearer token is kept 0o600");
    scratch.remove();
}

#[test]
fn the_binary_answers_a_chat_from_a_person_signed_in_by_its_link() {
    let scratch = support::Scratch::new("chat");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let peer = support::fake_llm(Box::new([support::chat_script(support::ANSWER)]));
    let seen = support::run(
        scratch.path(),
        support::person::answered_chat(b"Hello", support::ANSWER),
        Some(peer),
        Duration::from_secs(60),
    );
    assert_eq!(seen.signed_in, 1, "signed in once, by the printed link");
    assert!(seen.finished, "the person saw the answer: {}", String::from_utf8_lossy(&seen.stderr));
    scratch.remove();
}

#[test]
fn temper_sign_in_prints_the_visit_and_keeps_the_record() {
    let scratch = support::Scratch::new("temper_sign_in_record");
    support::sign_in::configuration(scratch.path());
    let issuer = support::sign_in::issuer();
    let mut child = support::sign_in::start(scratch.path(), support::sign_in::ACCOUNT);
    let visit = child.line(support::WITHIN);
    assert!(visit.starts_with(support::sign_in::AUTHORIZATION), "the visit is the issuer's: {visit:?}");
    let browser = support::sign_in::browse(&issuer, &visit);
    assert_eq!(child.exit(issuer, browser, None, support::WITHIN), 0, "stderr: {}", child.stderr());
    assert!(support::private(&scratch.path().join("accounts/records/1")), "the record is kept 0o600");
    scratch.remove();
}
#[test]
fn a_running_engine_lends_the_record_temper_sign_in_kept() {
    let scratch = support::Scratch::new("temper_sign_in_running");
    support::sign_in::configuration(scratch.path());
    let engine = support::sign_in::running(scratch.path());
    let issuer = support::sign_in::issuer();
    let mut child = support::sign_in::start(scratch.path(), support::sign_in::ACCOUNT);
    let visit = child.line(support::WITHIN);
    let browser = support::sign_in::browse(&issuer, &visit);
    assert_eq!(child.exit(issuer, browser, Some(engine), support::WITHIN), 0, "stderr: {}", child.stderr());
    scratch.remove();
}
