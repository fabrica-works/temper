use super::*;

fn configured(under_box: bool) -> Domain {
    let mut d = domain();
    d.under_box = under_box;
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
            assert!(matches!(
                sent.first(),
                Some(Request::Save { record: Stored::Demo(Demo { forge: 1, state: DemoState::Running }), .. })
            ));
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
            let mut d = configured(under_box);
            outputs(&mut d, Event::Restore { record: Stored::Demo(Demo { forge: 1, state }) });
            let sent = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
            assert_eq!(group_asks(&sent), usize::from(under_box && state == DemoState::Running));
            assert!(!makes_act(&sent));
            assert_eq!(d.demo_state(), Some(state));
            assert_eq!(d.application_answer(Token::new(101)), None);
        }
    }
}

fn group_asks(sent: &[Request]) -> usize {
    let mut count = 0_usize;
    for request in sent {
        if matches!(request, Request::SetUp { .. }) {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
fn saved(sent: &[Request]) -> bool {
    for request in sent {
        if matches!(request, Request::Save { .. }) {
            return true;
        }
    }
    false
}
fn makes_act(sent: &[Request]) -> bool {
    for request in sent {
        if matches!(request, Request::DemoStart { .. }) {
            return true;
        }
    }
    false
}
