//! Notices scripted at the existing fake process channel, with the real roots.
use super::*;
use crate::composed::{CHANNEL, World, report_script};
use jig_domain_accounts as accounts;
use jig_domain_host as host;
use skein_lib::{Duration, Wall};

fn saves(world: &World) -> Vec<accounts::Stored> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Accounts(record)) => Some(record.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn composed_notices_reach_the_accounts_child_before_the_attempt_ends() {
    for notice in [
        agent::Up::Unauthorized { account: 0, generation: 1 },
        agent::Up::Limited { account: 0, reset: None },
        agent::Up::Limited { account: 0, reset: Some(Duration::ZERO) },
        agent::Up::Limited { account: 0, reset: Some(Duration::from_secs(9)) },
        agent::Up::Refused {
            account: 0,
            code: b"quota".as_slice().into(),
            reason: b"provider refused".as_slice().into(),
        },
    ] {
        let mut world = World::chat(930, report_script(true));
        world.until_call();
        assert_eq!(world.observed.answers, 0);
        let message = match &notice {
            agent::Up::Unauthorized { account, generation } => {
                agent::Up::Unauthorized { account: *account, generation: *generation }
            }
            agent::Up::Limited { account, reset } => agent::Up::Limited { account: *account, reset: *reset },
            agent::Up::Refused { account, code, reason } => {
                agent::Up::Refused { account: *account, code: code.clone(), reason: reason.clone() }
            }
            _ => unreachable!(),
        };
        world.process.up.push_front(message);
        world.run();
        match notice {
            agent::Up::Unauthorized { account, generation } => assert!(
                world
                    .protocol
                    .accounts
                    .contains(&accounts::Request::Component(accounts::ToComponent::Rejected { account, generation }))
            ),
            agent::Up::Limited { reset, .. } => {
                if reset == Some(Duration::ZERO) {
                    assert!(
                        !saves(&world).iter().any(|record| matches!(
                            record,
                            accounts::Stored::Source { held: Some(accounts::Held::Limited { reset: None }), .. }
                        )),
                        "supplied zero is never an unknown hold"
                    );
                } else {
                    let held = saves(&world)
                        .into_iter()
                        .find_map(|record| match record {
                            accounts::Stored::Source {
                                account: 0,
                                number: 0,
                                held: Some(accounts::Held::Limited { reset }),
                                ..
                            } => Some(reset),
                            _ => None,
                        })
                        .expect("the accounts child saves the source's hold");
                    assert_eq!(held.is_some(), reset.is_some(), "unknown stays unknown; zero stays supplied");
                    if let Some(reset) = reset {
                        assert!(held.unwrap() >= Wall::from_nanos(reset.as_nanos()));
                    }
                }
            }
            agent::Up::Refused { .. } => assert!(saves(&world).iter().any(|record| matches!(
                record,
                accounts::Stored::Source { account: 0, number: 0, held: Some(accounts::Held::Refused), .. }
            ))),
            _ => unreachable!(),
        }
        assert_eq!(world.observed.answers, 1, "notice does not replace the terminal");
        let before = world.protocol.accounts.clone();
        let assignment = world.assignment();
        let up = host::LinkUp::Unauthorized {
            run: Token::new(assignment.task),
            attempt: Token::new(assignment.attempt),
            account: 0,
            generation: 2,
        };
        world.root_step(temper_engine_domain_world::protocol::up(CHANNEL, up, engine::Parts::none()));
        assert_eq!(world.protocol.accounts, before, "ended attempt cannot report again");
    }
}

#[test]
fn composed_notice_fences_drop_old_channels_attempts_sources_and_generations() {
    for (channel, old_attempt, source, generation, renewed) in [
        (Token::new(99), false, 0, 1, false),
        (CHANNEL, true, 0, 1, false),
        (CHANNEL, false, 17, 1, false),
        (CHANNEL, false, 0, 1, true),
    ] {
        let mut world = World::chat(931, report_script(true));
        world.until_call();
        if renewed {
            world.root_step(engine::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
                account: 0,
                generation: 2,
                valid: Duration::from_secs(3600),
                form: accounts::Form::Credential(accounts::Scheme::Bearer),
            })));
        }
        let assignment = world.assignment();
        let up = host::LinkUp::Unauthorized {
            run: Token::new(assignment.task),
            attempt: Token::new(if old_attempt { assignment.attempt + 1 } else { assignment.attempt }),
            account: source,
            generation,
        };
        world.root_step(temper_engine_domain_world::protocol::up(channel, up, engine::Parts::none()));
        world.run();
        assert!(
            !world
                .protocol
                .accounts
                .iter()
                .any(|request| matches!(request, accounts::Request::Component(accounts::ToComponent::Rejected { .. })))
        );
        assert!(!saves(&world).iter().any(|record| matches!(record, accounts::Stored::Source { held: Some(_), .. })));
    }
}
