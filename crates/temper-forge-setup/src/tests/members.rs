// Fixture parsing without a deployment keeps written paths.
use super::fixtures;
use crate::{
    check::{self, Members, User},
    problem,
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, SocketAddr};
fn refused(bytes: &[u8], members: &Members, expected: &[u8]) {
    let problems =
        check::checked(&fixtures::document(bytes), 0, &jig_protocol_config::Directories::working(), members, None, &[])
            .unwrap_err();
    assert_eq!(&*problem::lines(&problems), expected);
}
#[test]
fn a_forge_whose_api_token_names_another_file_is_refused_naming_its_path() {
    let mut members = fixtures::members();
    members.forges[0].api_token = b"elsewhere".as_slice().into();
    refused(fixtures::TWO, &members, b"application.forges[0].api_token: invalid form\n");
}
#[test]
fn a_set_up_forge_whose_user_is_a_number_is_refused() {
    let mut members = fixtures::members();
    members.forges[0].user = User::Id(42);
    refused(fixtures::TWO, &members, b"application.forges[0].user: invalid form\n");
}
#[test]
fn an_owned_forge_whose_api_is_not_its_listen_is_refused() {
    let mut members = fixtures::members();
    members.forges[1].api = Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 4000)));
    refused(fixtures::TWO, &members, b"application.forges[1].api: invalid form\n");
}
#[test]
fn a_listen_taken_by_another_listener_is_refused() {
    let mut members = fixtures::members();
    members.listeners = Box::new([SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))]);
    refused(fixtures::TWO, &members, b"application.setup[1].mode.owned.listen: invalid form\n");
}
#[test]
fn an_entry_at_odds_with_itself_or_its_forge_is_refused_naming_its_path() {
    for (from, to, expected) in [
        (
            br#""forge":2"#.as_slice(),
            br#""forge":1"#.as_slice(),
            b"application.setup[1].forge: invalid form\n".as_slice(),
        ),
        (br#""directory":"forge-2""#, br#""directory":"forge-1""#, b"application.setup[1].directory: invalid form\n"),
        (br#""login":"ada""#, br#""login":"temper""#, b"application.setup[1].mode.owned.person.login: invalid form\n"),
        (
            br#""login":"ada""#,
            br#""login":"temper-admin""#,
            b"application.setup[1].mode.owned.person.login: invalid form\n",
        ),
    ] {
        let mut problems = problem::Problems::empty();
        let setup = crate::Setup::read(
            &fixtures::document(&fixtures::changed(from, to)),
            0,
            &crate::limits::small(),
            &jig_protocol_config::Directories::working(),
            &mut problems,
        )
        .unwrap();
        check::members(&setup, &fixtures::members(), false, &mut problems);
        assert!(skein_lib::bytes::find(&problem::lines(&problems), expected).is_some(), "case {from:?}");
    }
    for field in [b"token".as_slice(), b"login", b"git", b"transport"] {
        let mut members = fixtures::members();
        let expected = match field {
            b"token" => {
                members.forges[0].identity_token = b"elsewhere".as_slice().into();
                b"application.setup[0].forge: invalid form\n".as_slice()
            }
            b"login" => {
                members.forges[0].identity_login = b"other".as_slice().into();
                b"application.setup[0].bot.login: invalid form\n"
            }
            b"git" => {
                members.forges[1].git = b"http://127.0.0.1:9".as_slice().into();
                b"application.forges[1].git: invalid form\n"
            }
            b"transport" => {
                members.forges[1].plaintext = false;
                b"application.forges[1].transport: invalid form\n"
            }
            _ => unreachable!(),
        };
        refused(fixtures::TWO, &members, expected);
    }
}

#[test]
fn a_webhook_secret_off_the_forge_directory_is_refused() {
    let mut members = fixtures::hooked_members();
    members.forges[0].webhook_secret = Some(b"elsewhere".as_slice().into());
    refused_webhooks(&members, b"application.forges[0].webhook_secret: invalid form\n");
}

fn refused_webhooks(members: &Members, expected: &[u8]) {
    let problems = check::checked(
        &fixtures::document(fixtures::TWO),
        0,
        &jig_protocol_config::Directories::working(),
        members,
        Some(fixtures::listener()),
        &[],
    )
    .unwrap_err();
    assert_eq!(&*problem::lines(&problems), expected);
}
#[test]
fn a_set_up_forge_without_a_webhook_secret_is_refused() {
    let mut members = fixtures::hooked_members();
    members.forges[0].webhook_secret = None;
    refused_webhooks(&members, b"application.forges[0].webhook_secret: invalid form\n");
}
