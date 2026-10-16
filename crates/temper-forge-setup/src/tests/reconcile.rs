use super::fixtures::{self, Run, secret};
use crate::{
    place::File,
    reconcile::{Event, Failure, Need, Reply, Stage, Why},
};
use temper_forge_forgejo::{document::Document, request::Request};
fn user(user: temper_forge_forgejo::document::User) -> Event {
    Event::Reply(Reply::Document(Document::User(user)))
}
#[test]
fn an_admin_token_of_a_plain_user_stops_at_the_admin() {
    let mut run = Run::managed();
    run.step(Event::Begin);
    run.step(Event::Read(Some(secret(fixtures::ADMIN))));
    let need = run.step(user(fixtures::person(b"ada", b"Ada")));
    assert_eq!(
        need,
        Need::Stop(Failure {
            forge: 1,
            step: Stage::Admin,
            subject: b"temper-admin".as_slice().into(),
            why: Why::NotAdmin,
        })
    );
}
#[test]
fn an_absent_bot_is_made_with_a_drawn_password_and_the_marker() {
    let mut run = Run::managed();
    run.bot_lookup();
    assert_eq!(run.step(Event::Reply(Reply::Status(404))), Need::Draw(32));
    let need = run.step(Event::Drawn(secret(&[0x0a; 32])));
    assert_eq!(
        need,
        Need::Send {
            request: Request::CreateUser {
                login: b"temper".as_slice().into(),
                email: b"temper@example.invalid".as_slice().into(),
                full_name: b"temper:bot".as_slice().into(),
                password: secret(b"0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a"),
                must_change_password: false,
            },
            token: secret(fixtures::ADMIN),
        }
    );
    assert_eq!(run.step(user(fixtures::person(b"temper", b"temper:bot"))), Need::Read(File::User));
    assert_eq!(run.step(Event::Read(None)), Need::Write { file: File::User, bytes: secret(b"42") });
    assert_eq!(run.step(Event::Written), Need::Done);
}
#[test]
fn a_bot_with_the_marker_is_kept_and_nothing_is_written() {
    let mut run = Run::managed();
    run.bot_lookup();
    assert_eq!(run.step(user(fixtures::person(b"temper", b"note temper:bot account"))), Need::Read(File::User));
    assert_eq!(run.step(Event::Read(Some(secret(b"42")))), Need::Done);
}
#[test]
fn a_user_of_the_bots_login_without_the_marker_is_refused_naming_the_login() {
    let mut run = Run::managed();
    run.bot_lookup();
    let need = run.step(user(fixtures::person(&run.bot(), b"a note")));
    assert_eq!(
        need,
        Need::Stop(Failure { forge: run.forge(), step: Stage::Bot, subject: run.bot(), why: Why::Unmarked })
    );
}
#[test]
fn a_bot_that_is_a_site_admin_is_refused() {
    let mut run = Run::managed();
    run.bot_lookup();
    let mut bot = fixtures::person(b"temper", b"temper:bot");
    bot.is_admin = true;
    assert_eq!(
        run.step(user(bot)),
        Need::Stop(Failure { forge: 1, step: Stage::Bot, subject: b"temper".as_slice().into(), why: Why::BotIsAdmin })
    );
}
#[test]
fn a_refused_request_stops_naming_its_step_and_subject() {
    let mut run = Run::managed();
    run.bot_lookup();
    run.step(Event::Reply(Reply::Status(404)));
    run.step(Event::Drawn(secret(&[0x0a; 32])));
    assert_eq!(
        run.step(Event::Reply(Reply::Status(403))),
        Need::Stop(Failure {
            forge: 1,
            step: Stage::Bot,
            subject: b"temper".as_slice().into(),
            why: Why::Refused(403),
        })
    );
}
#[test]
fn a_forge_that_stops_answering_stops_the_run_at_its_step() {
    let mut run = Run::managed();
    run.bot_lookup();
    assert_eq!(
        run.step(Event::Reply(Reply::Down)),
        Need::Stop(Failure { forge: 1, step: Stage::Bot, subject: b"temper".as_slice().into(), why: Why::Down })
    );
}
#[test]
fn the_bots_id_is_written_in_decimal_only_when_the_file_differs() {
    for old in [None, Some(secret(b"041")), Some(secret(b"42\n")), Some(secret(b"43"))] {
        let mut run = Run::owned();
        run.bot_lookup();
        run.step(user(fixtures::person(b"temper", b"temper:bot")));
        assert_eq!(
            run.step(Event::Read(old.clone())),
            Need::Write { file: File::User, bytes: secret(b"42") },
            "{old:?}"
        );
        assert_eq!(run.step(Event::Written), Need::Done);
    }
}
#[test]
fn a_missing_admin_token_stops_before_any_request() {
    let mut run = Run::managed();
    run.step(Event::Begin);
    assert_eq!(
        run.step(Event::Read(None)),
        Need::Stop(Failure {
            forge: 1,
            step: Stage::Admin,
            subject: b"temper-admin".as_slice().into(),
            why: Why::Missing,
        })
    );
}
#[test]
fn an_empty_admin_token_stops_before_any_request() {
    let mut run = Run::managed();
    run.step(Event::Begin);
    assert_eq!(
        run.step(Event::Read(Some(secret(b"")))),
        Need::Stop(Failure {
            forge: 1,
            step: Stage::Admin,
            subject: b"temper-admin".as_slice().into(),
            why: Why::Missing,
        })
    );
}
#[test]
fn a_draw_of_another_length_stops_before_creating_the_bot() {
    let mut run = Run::managed();
    run.bot_lookup();
    run.step(Event::Reply(Reply::Status(404)));
    assert_eq!(
        run.step(Event::Drawn(secret(&[0x0a; 31]))),
        Need::Stop(Failure { forge: 1, step: Stage::Bot, subject: b"temper".as_slice().into(), why: Why::Unexpected })
    );
}
