use super::*;

fn configured(under_box: bool) -> Domain {
    let mut d = domain();
    d.under_box = under_box;
    for number in [1, 2] {
        d.configured.insert(number, Configured { number, kind: Kind::Forgejo, mode: Mode::Fake, asked: true }).unwrap();
    }
    d.connection_limits(3, 1);
    d.configure_demo(Some(DemoConfig {
        provider: 1,
        adoption: fixtures::adoption(),
        deadline: Duration::from_secs(10),
    }));
    d
}
fn start(d: &mut Domain, number: u64) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            request: Token::new(number),
            token: Token::new(number.checked_add(100).unwrap()),
            retention: Duration::from_secs(60),
            ask: Ask::StartDemo,
        },
    )
}

#[test]
fn demo_start_is_not_offered_without_a_demo_or_after_its_end() {
    for ended in [false, true] {
        let mut d = if ended { configured(true) } else { domain() };
        outputs(&mut d, Event::Restore { record: Stored::Demo(Demo { forge: 1, state: DemoState::Ended }) });
        assert_eq!(
            start(&mut d, 1).as_ref(),
            [Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }]
        );
        assert_eq!(d.application_answer(Token::new(101)), Some(Answered::StartDemo(StartAnswer::NotOffered)));
    }
}

#[test]
fn demo_first_start_saves_running_and_only_asks_a_dormant_group_under_the_box() {
    for under_box in [false, true] {
        for connection in [
            Connection::NotConnected,
            Connection::Connecting,
            Connection::Connected { id: 7, login: b"temper".as_slice().into() },
        ] {
            let mut d = configured(under_box);
            outputs(&mut d, Event::Connection { forge: 1, state: connection.clone() });
            let sent = start(&mut d, 1);
            assert_eq!(
                sent.first(),
                Some(&Request::Save {
                    read_afresh: None,
                    release: None,
                    record: Stored::Demo(Demo { forge: 1, state: DemoState::Running }),
                })
            );
            let asks = group_asks(&sent);
            assert_eq!(asks, usize::from(under_box && connection == Connection::NotConnected));
            assert!(sent.contains(&Request::DemoStart { request: Token::new(1) }));
            outputs(&mut d, Event::DemoAnswered { answer: StartAnswer::Running });
            assert!(!saved(&start(&mut d, 2)));
        }
    }
}

#[test]
fn simultaneous_demo_starts_share_the_answer_and_bound_their_waiters() {
    let mut d = configured(true);
    start(&mut d, 1);
    for number in 2..=u64::from(LIMITS.applications) {
        assert!(start(&mut d, number).is_empty());
    }
    assert_eq!(
        start(&mut d, 20).as_ref(),
        [Request::ApplicationAgain { request: Token::new(20), why: jig_domain_people::Again::Busy }]
    );
    let answer = StartAnswer::Failed(StartFailure::Account);
    let sent = outputs(&mut d, Event::DemoAnswered { answer: answer.clone() });
    assert_eq!(sent.len(), usize::try_from(LIMITS.applications).unwrap());
    for number in 1..=u64::from(LIMITS.applications) {
        assert_eq!(
            d.application_answer(Token::new(number.checked_add(100).unwrap())),
            Some(Answered::StartDemo(answer.clone()))
        );
    }
    assert_eq!(d.application_answer(Token::new(900)), None);
}

#[test]
fn demo_restore_asks_only_for_a_running_dormant_group_and_makes_no_act() {
    for under_box in [false, true] {
        for state in [DemoState::Running, DemoState::Ended] {
            for connection in [
                Connection::NotConnected,
                Connection::Connecting,
                Connection::Connected { id: 7, login: b"temper".as_slice().into() },
            ] {
                let mut d = configured(under_box);
                outputs(&mut d, Event::Connection { forge: 1, state: connection.clone() });
                outputs(&mut d, Event::Restore { record: Stored::Demo(Demo { forge: 1, state }) });
                let sent = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
                assert_eq!(
                    group_asks(&sent),
                    usize::from(under_box && state == DemoState::Running && connection == Connection::NotConnected)
                );
                assert!(!makes_act(&sent));
                assert_eq!(d.demo_state(), Some(state));
                assert_eq!(d.application_answer(Token::new(101)), None);
            }
        }
    }
}

fn group_asks(sent: &[Request]) -> usize {
    let mut count = 0_usize;
    for request in sent {
        if let Request::SetUpGroup { .. } = request {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
fn saved(sent: &[Request]) -> bool {
    for request in sent {
        if let Request::Save { .. } = request {
            return true;
        }
    }
    false
}
fn makes_act(sent: &[Request]) -> bool {
    for request in sent {
        if let Request::DemoStart { .. } = request {
            return true;
        }
    }
    false
}

fn connected(d: &mut Domain, forge: u16) -> Box<[Request]> {
    outputs(d, Event::Connection { forge, state: Connection::Connected { id: 7, login: b"temper".as_slice().into() } })
}

#[test]
fn first_other_connection_ends_once_from_none_or_running_and_answers_held_starts() {
    for under_box in [false, true] {
        for running in [false, true] {
            let mut d = configured(under_box);
            if running {
                start(&mut d, 1);
            }
            let sent = connected(&mut d, 2);
            assert!(sent.contains(&Request::Save {
                read_afresh: None,
                release: None,
                record: Stored::Demo(Demo { forge: 1, state: DemoState::Ended })
            }));
            assert_eq!(sent.contains(&Request::StopGroup { forge: 1 }), under_box && running);
            assert_eq!(d.demo_state(), Some(DemoState::Ended));
            assert!(d.forge_ended(1));
            assert!(!d.forge_ended(2));
            let forges = d.forges();
            let [forge] = forges.rows.as_ref() else { panic!("only the other forge remains listed") };
            assert_eq!(forge.number, 2);
            assert_eq!(d.offer(1), Offer::NotOffered);
            if running {
                assert!(sent.contains(&Request::ApplicationDone { request: Token::new(1), token: Token::new(101) }));
                assert_eq!(d.application_answer(Token::new(101)), Some(Answered::StartDemo(StartAnswer::NotOffered)));
            }
            start(&mut d, 2);
            assert_eq!(d.application_answer(Token::new(102)), Some(Answered::StartDemo(StartAnswer::NotOffered)));
            let sent = connected(&mut d, 2);
            assert!(!saved(&sent));
            assert!(!sent.contains(&Request::StopGroup { forge: 1 }));
            assert!(connected(&mut d, 1).is_empty());
            assert_eq!(d.connection(1), Connection::NotConnected);
        }
    }
}

#[test]
fn own_unconfigured_and_no_demo_connections_do_not_end_the_demo() {
    let mut d = configured(true);
    assert!(!saved(&connected(&mut d, 1)));
    connected(&mut d, 3);
    assert_eq!(d.demo_state(), None);
    let mut d = configured(true);
    d.configure_demo(None);
    connected(&mut d, 2);
    assert_eq!(d.demo_state(), None);
}

#[test]
fn restored_ended_demo_hides_repositories_but_keeps_lookups_and_refuses_setup() {
    let mut d = configured(true);
    let home = adopted();
    let mut context = home.clone();
    context.provider.repository = 3;
    context.home = false;
    context.role = Role::Context;
    context.name = b"context".as_slice().into();
    for row in [home.clone(), context] {
        outputs(&mut d, Event::Restore { record: Stored::Repository(row) });
    }
    outputs(&mut d, Event::Restore { record: Stored::Demo(Demo { forge: 1, state: DemoState::Ended }) });
    let sent = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert_eq!(group_asks(&sent), 0);
    assert!(d.forge_ended(1));
    assert_eq!(d.home(5), None);
    assert!(d.context_repositories(5, 2).unwrap().is_empty());
    assert!(d.adopted_ids(1).is_empty());
    assert_eq!(d.repository(REPO), Some(&home));
    assert_eq!(d.repository_tag(5, tag(REPO.forge, REPO.repository)), Some(&home));
    assert!(connected(&mut d, 1).is_empty());
    outputs(
        &mut d,
        Event::Application {
            request: Token::new(4),
            token: Token::new(104),
            retention: Duration::from_secs(60),
            ask: Ask::SetUp {
                forge: 1,
                org: b"org".as_slice().into(),
                repository: b"repo".as_slice().into(),
                project: SetUpProject::Existing(5),
            },
        },
    );
    assert_eq!(
        d.application_answer(Token::new(104)),
        Some(Answered::SetUp(SetUpAnswer::Refused(SetUpRefusal::NotOffered)))
    );
}

#[test]
fn an_ended_home_does_not_prevent_a_new_home_adoption() {
    let mut d = configured(false);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Demo(Demo { forge: 1, state: DemoState::Ended }) });
    connected(&mut d, 2);
    let mut request = fixtures::adoption();
    request.forge = 2;
    outputs(
        &mut d,
        Event::Application {
            request: Token::new(4),
            token: Token::new(104),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(request),
        },
    );
    let sent = ready(&mut d);
    let [Request::Call { call, .. }] = sent.as_ref() else { panic!("locate on the other forge") };
    outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Answered {
            call: *call,
            cost: 1,
            free: 0,
            rate: None,
            result: Ok(skein_forge::api::Answer::Located { repository: 9 }),
        }),
    );
    let sent = outputs(&mut d, Event::Adopt { request: Token::new(4) });
    assert!(sent.is_empty());
    let mut permission = false;
    for request in ready(&mut d).as_ref() {
        if let Request::Call { op: skein_forge::api::Op::Read(skein_forge::api::Read::Permission { .. }), .. } = request
        {
            permission = true;
        }
    }
    assert!(permission);
    assert_eq!(d.application_outcome(Token::new(104)), None);
}

#[test]
fn closing_the_box_before_the_end_suppresses_stop_and_ended_connections_stay_inert() {
    let mut d = configured(true);
    start(&mut d, 1);
    outputs(&mut d, Event::BoxClosed);
    assert!(!connected(&mut d, 2).contains(&Request::StopGroup { forge: 1 }));
    for state in [
        Connection::NotConnected,
        Connection::NotSetUp,
        Connection::Connecting,
        Connection::SettingUp { step: Some(SetUpStep::Run) },
        Connection::SetUpFailed { why: b"failed".as_slice().into() },
        Connection::Stopped { reason: b"stopped".as_slice().into() },
        Connection::Connected { id: 99, login: b"different".as_slice().into() },
    ] {
        assert!(outputs(&mut d, Event::Connection { forge: 1, state }).is_empty());
        assert_eq!(d.connection(1), Connection::NotConnected);
        assert_eq!(d.demo_state(), Some(DemoState::Ended));
    }
}
