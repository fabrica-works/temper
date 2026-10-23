//! Public forge admission, without protocol or credential bytes.
#![expect(clippy::disallowed_methods, reason = "ordinary test assertions inspect captured requests")]
#![expect(clippy::match_like_matches_macro, reason = "explicit test patterns; matches is banned by the subset")]
use super::*;
use core::num::NonZeroU64;
use jig_domain_core::Carried;
use jig_domain_people::Role as PartyRole;

const VALUE: Carried = Carried { number: 1_u64 << 63_u32, digest: [17; 32] };
fn github(kind: Kind, mode: Mode) -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config { namespace: Box::from(b"test".as_slice()), writers: Box::new([]) },
        &[],
        &[Configured { number: 1, kind, mode, asked: false }],
        false,
    )
    .unwrap()
}
fn connect(forge: u16) -> Ask {
    Ask::Connect { forge, app: NonZeroU64::new(7).unwrap() }
}
fn application(
    d: &mut Domain,
    number: u64,
    ask: Ask,
    role: Option<PartyRole>,
    carried: Option<Carried>,
) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number),
            retention: Duration::from_secs(60),
            ask,
            role,
            carried,
        },
    )
}
fn refused(d: &mut Domain, ask: Ask, role: Option<PartyRole>, carried: Option<Carried>, why: ForgeRefusal) {
    let install = if let Ask::Install { .. } = ask { true } else { false };
    let out = application(d, 1, ask, role, carried);
    let expected = if install {
        Answered::Install(InstallAnswer::Refused(why))
    } else {
        Answered::Connect(ConnectAnswer::Refused(why))
    };
    assert_eq!(d.application_answer(Token::new(1)), Some(expected));
    assert_eq!(
        out.iter()
            .filter(|r| if let Request::CarriedUnused { number } = r { *number == VALUE.number } else { false })
            .count(),
        usize::from(carried.is_some())
    );
    assert!(!out.iter().any(|r| if let Request::Connect { .. } | Request::Install { .. } = r { true } else { false }));
}
#[test]
fn connect_requires_the_deployment_owner_and_a_configured_real_github_and_key() {
    for role in
        [None, Some(PartyRole::Member), Some(PartyRole::Maintainer), Some(PartyRole::Observer), Some(PartyRole::Owner)]
    {
        let mut d = github(Kind::GitHub, Mode::Connected);
        if role == Some(PartyRole::Owner) {
            assert!(
                application(&mut d, 1, connect(1), role, Some(VALUE))
                    .iter()
                    .any(|r| if let Request::Connect { .. } = r { true } else { false })
            );
        } else {
            refused(&mut d, connect(1), role, Some(VALUE), ForgeRefusal::Role);
        }
    }
    refused(
        &mut github(Kind::GitHub, Mode::Connected),
        connect(2),
        Some(PartyRole::Owner),
        Some(VALUE),
        ForgeRefusal::NoSuchForge,
    );
    for (kind, mode) in [(Kind::Forgejo, Mode::Connected), (Kind::GitHub, Mode::Fake)] {
        refused(&mut github(kind, mode), connect(1), Some(PartyRole::Owner), Some(VALUE), ForgeRefusal::NotTaken);
    }
    refused(&mut github(Kind::GitHub, Mode::Connected), connect(1), Some(PartyRole::Owner), None, ForgeRefusal::NoKey);
    for state in [Connection::Connecting, Connection::Connected { id: 9, login: Box::from(b"app[bot]".as_slice()) }] {
        let mut d = github(Kind::GitHub, Mode::Connected);
        outputs(&mut d, Event::Connection { forge: 1, state });
        refused(&mut d, connect(1), Some(PartyRole::Owner), Some(VALUE), ForgeRefusal::Connected);
    }
}
#[test]
fn admitted_connect_marks_connecting_answers_then_erases_once_at_its_end() {
    for success in [false, true] {
        let mut d = github(Kind::GitHub, Mode::Connected);
        let out = application(&mut d, 1, connect(1), Some(PartyRole::Owner), Some(VALUE));
        assert_eq!(d.connection(1), Connection::Connecting);
        assert!(out.iter().any(|r| if let Request::Connect { forge: 1, request, app, carried } = r {
            *request == Token::new(1) && app.get() == 7 && *carried == VALUE
        } else {
            false
        }));
        assert_eq!(d.application_answer(Token::new(1)), None);
        outputs(&mut d, Event::ConnectAnswered { forge: 1, request: Token::new(1), answer: ConnectAnswer::Connecting });
        assert_eq!(d.application_answer(Token::new(1)), Some(Answered::Connect(ConnectAnswer::Connecting)));
        let out = application(&mut d, 2, connect(1), Some(PartyRole::Owner), Some(VALUE));
        assert_eq!(
            d.application_answer(Token::new(2)),
            Some(Answered::Connect(ConnectAnswer::Refused(ForgeRefusal::Busy)))
        );
        assert_eq!(out.iter().filter(|r| if let Request::CarriedUnused { .. } = r { true } else { false }).count(), 1);
        let out = outputs(&mut d, Event::ConnectEnded { forge: 1, request: Token::new(1), connected: success });
        assert_eq!(
            out.iter()
                .filter(|r| if let Request::EraseCarried { number } = r { *number == VALUE.number } else { false })
                .count(),
            1
        );
        assert!(
            outputs(&mut d, Event::ConnectEnded { forge: 1, request: Token::new(1), connected: success }).is_empty()
        );
        if !success {
            assert_eq!(d.connection(1), Connection::NotConnected);
        }
    }
}
#[test]
fn install_requires_connection_and_defers_its_answer_until_the_run_finishes() {
    let ask = Ask::Install { forge: 1, back: Some(Box::from(b"/forges".as_slice())) };
    refused(
        &mut github(Kind::GitHub, Mode::Connected),
        ask.clone(),
        Some(PartyRole::Owner),
        Some(VALUE),
        ForgeRefusal::NotConnected,
    );
    refused(&mut github(Kind::GitHub, Mode::Connected), ask.clone(), None, Some(VALUE), ForgeRefusal::Role);
    refused(
        &mut github(Kind::GitHub, Mode::Connected),
        Ask::Install { forge: 2, back: None },
        Some(PartyRole::Owner),
        Some(VALUE),
        ForgeRefusal::NoSuchForge,
    );
    refused(
        &mut github(Kind::Forgejo, Mode::Connected),
        ask.clone(),
        Some(PartyRole::Owner),
        Some(VALUE),
        ForgeRefusal::NotTaken,
    );
    for answer in [
        InstallAnswer::Connecting,
        InstallAnswer::Link(Box::from(b"http://fake/apps/app/installations/new".as_slice())),
    ] {
        let mut d = github(Kind::GitHub, Mode::Connected);
        outputs(
            &mut d,
            Event::Connection {
                forge: 1,
                state: Connection::Connected { id: 9, login: Box::from(b"app[bot]".as_slice()) },
            },
        );
        let out = application(&mut d, 1, ask.clone(), Some(PartyRole::Owner), Some(VALUE));
        assert!(out.iter().any(|r| if let Request::Install { forge: 1, back: Some(back), .. } = r {
            back.as_ref() == b"/forges"
        } else {
            false
        }));
        assert_eq!(out.iter().filter(|r| if let Request::CarriedUnused { .. } = r { true } else { false }).count(), 1);
        assert_eq!(d.application_answer(Token::new(1)), None);
        application(&mut d, 2, ask.clone(), Some(PartyRole::Owner), None);
        assert_eq!(
            d.application_answer(Token::new(2)),
            Some(Answered::Install(InstallAnswer::Refused(ForgeRefusal::Busy)))
        );
        outputs(&mut d, Event::InstallAnswered { forge: 1, request: Token::new(1), answer: answer.clone() });
        assert_eq!(d.application_answer(Token::new(1)), Some(Answered::Install(answer)));
    }
}

#[test]
fn a_busy_service_refusal_names_the_carried_value_unused_once() {
    let mut d = github(Kind::GitHub, Mode::Connected);
    application(&mut d, 1, connect(1), Some(PartyRole::Owner), Some(VALUE));
    let out = outputs(
        &mut d,
        Event::ConnectAnswered { forge: 1, request: Token::new(1), answer: ConnectAnswer::Refused(ForgeRefusal::Busy) },
    );
    assert_eq!(out.iter().filter(|r| if let Request::CarriedUnused { .. } = r { true } else { false }).count(), 1);
    assert!(!out.iter().any(|r| if let Request::EraseCarried { .. } = r { true } else { false }));
    assert_eq!(d.connection(1), Connection::NotConnected);
}
