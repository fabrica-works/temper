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
fn a_persons_change_lands_on_the_forge_and_is_found_after_a_restart() {
    let scratch = support::Scratch::new("landed");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let peer = support::fake_llm(support::landing_scripts(support::ANSWER, &forgejo));
    let first = support::run_until_landed(scratch.path(), support::person::adopt_and_change(&forgejo), peer, &forgejo);
    assert_eq!((first.signed_in, first.finished), (1, true), "{}", String::from_utf8_lossy(&first.stderr));
    let merged = forgejo.merged_pull().expect("the change's pull request is merged");
    assert!(forgejo.main_readme().ends_with(support::EDIT), "main holds the agent's edit");
    let second = support::run(scratch.path(), support::person::found(), None, Duration::from_secs(60));
    assert!(second.finished, "the work is found after the restart");
    assert_eq!(forgejo.merged_pull(), Some(merged), "the forge's state is unchanged by the restart");
    forgejo.stop();
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
