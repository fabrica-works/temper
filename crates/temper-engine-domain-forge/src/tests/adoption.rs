//! Core admission and the bounded retained application outcome.
use super::*;
use jig_domain_people as people;

#[test]
fn a_core_refusal_is_kept_as_jigs_refusal() {
    let mut d = domain();
    outputs(
        &mut d,
        Event::Application {
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            adoption: fixtures::adoption(),
        },
    );
    let requests = outputs(
        &mut d,
        Event::ApplicationAnswered {
            request: Token::new(4),
            outcome: people::Outcome::Refused(people::Refusal::Unknown),
        },
    );
    assert!(requests.contains(&Request::ApplicationDone { request: Token::new(4), token: Token::new(9) }));
    assert_eq!(d.application_outcome(Token::new(9)), Some(&ApplicationOutcome::CoreRefused(people::Refusal::Unknown)));
}

#[test]
fn a_full_outcome_table_answers_the_application_again_busy() {
    let mut d = domain();
    for index in 1..=LIMITS.applications {
        let request = Token::new(u64::from(index));
        outputs(
            &mut d,
            Event::Application {
                request,
                token: request,
                retention: Duration::from_secs(60),
                adoption: fixtures::adoption(),
            },
        );
        outputs(
            &mut d,
            Event::ApplicationAnswered { request, outcome: people::Outcome::Refused(people::Refusal::Unknown) },
        );
    }
    let requests = outputs(
        &mut d,
        Event::Application {
            request: Token::new(99),
            token: Token::new(99),
            retention: Duration::from_secs(60),
            adoption: fixtures::adoption(),
        },
    );
    assert_eq!(requests.as_ref(), &[Request::ApplicationAgain { request: Token::new(99), why: people::Again::Busy }]);
    assert_eq!(d.application_outcome(Token::new(99)), None);
}

#[test]
fn a_kept_outcome_is_saved_with_the_end_of_its_retention() {
    let mut d = domain();
    let mut adoption = fixtures::adoption();
    adoption.provider.forge = 9;
    let requests = outputs(
        &mut d,
        Event::Application {
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            adoption,
        },
    );
    let row = Stored::Application {
        token: 9,
        until: Wall::from_nanos(160_000_000_000),
        outcome: ApplicationOutcome::ForgeRefused(AdoptionRefusal::NoSuchForge),
    };
    assert!(requests.contains(&Request::Save { record: row, release: None, read_afresh: None }));
    assert!(requests.contains(&Request::ApplicationDone { request: Token::new(4), token: Token::new(9) }));
}

#[test]
fn an_outcome_past_its_retention_is_erased_at_the_next_application() {
    let mut d = domain();
    let outcome = ApplicationOutcome::CoreRefused(people::Refusal::Unknown);
    outputs(&mut d, Event::Restore { record: Stored::Application { token: 9, until: env().wall, outcome } });
    let requests = outputs(
        &mut d,
        Event::Application {
            request: Token::new(4),
            token: Token::new(10),
            retention: Duration::from_secs(60),
            adoption: fixtures::adoption(),
        },
    );
    assert!(requests.contains(&Request::Erase { key: Key::Application(9) }));
    assert_eq!(d.application_outcome(Token::new(9)), None);
}
