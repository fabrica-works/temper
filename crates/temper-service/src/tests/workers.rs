use super::fixtures;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::Time;
use skein_lib::Token;
#[test]
fn each_repositorys_grant_carries_its_identitys_value() {
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let filled = values.grants(&[fixtures::application_grant(1, 1)], Time::ZERO);
    let [Some(value)] = filled.as_ref() else { panic!("one value for one grant") };
    let identity = fixtures::decoded_identity(&value.value);
    assert_eq!(identity.token().expose(), b"forge-token-1234");
    assert_eq!(identity.login(), b"temper");
    assert_eq!(identity.name(), b"temper");
    assert_eq!(identity.email(), b"temper@forge.example");
    assert_eq!(value.account_id.expose(), b"");
}
#[test]
fn a_missing_identity_or_generation_keeps_its_grant_slot_empty() {
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let grants =
        [fixtures::application_grant(1, 2), fixtures::application_grant(2, 1), fixtures::application_grant(1, 1)];
    let filled = values.grants(&grants, Time::ZERO);
    assert_eq!(filled.len(), 3);
    for (index, present) in [(0, false), (1, false), (2, true)] {
        assert_eq!(filled[index].is_some(), present, "grant {index}");
    }
}
#[test]
fn identities_are_announced_before_the_restart() {
    let mut service = fixtures::started();
    service.engine.store.opened = true;
    assert_eq!(service.engine.domain.next_deadline(&service.engine.env), None);
    crate::engine::iterate(&mut service.engine, Time::ZERO, skein_lib::Wall::EPOCH);
    assert!(service.engine.credentials.is_empty());
    assert_eq!(service.engine.domain.next_deadline(&service.engine.env), Some(Time::from_nanos(u64::MAX)));
    assert!(service.engine.store.routed.is_empty(), "the restart has not asked for a load");
    fixtures::restart(&mut service);
}

#[test]
fn an_accounts_grant_uses_the_lent_generation_until_it_expires() {
    let mut values = fixtures::values_with_identity(1, b"forge-token-1234");
    values
        .accounts
        .granted(
            1,
            2,
            jig_protocol_accounts::table::Value {
                token: skein_lib::Secret::new(b"account-token".as_slice().into()),
                account_id: skein_lib::Secret::new(b"account-id".as_slice().into()),
            },
            Time::from_nanos(9),
        )
        .unwrap();
    let mut grant = fixtures::application_grant(1, 2);
    grant.table = jig_domain_host::GrantTable::Accounts;
    for (now, generation, expected) in [(8, 2, true), (9, 2, false), (8, 1, false)] {
        grant.generation = generation;
        let filled = values.grants(&[grant], Time::from_nanos(now));
        assert_eq!(filled[0].is_some(), expected, "{now} {generation}");
        if let Some(value) = &filled[0] {
            assert_eq!(value.value.expose(), b"account-token", "{now} {generation}");
            assert_eq!(value.account_id.expose(), b"account-id", "{now} {generation}");
        }
    }
}

fn assignment_with_identity() -> temper_engine_domain::Assignment {
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.grants = Box::new([jig_domain_accounts::Grant {
        table: jig_domain_accounts::GrantTable::Application,
        account: 1,
        generation: 1,
        valid: skein_lib::Duration::from_secs(30),
    }]);
    assignment.transcript = Box::new([b"turn".as_slice().into()]);
    assignment
}
#[test]
fn an_assignment_carries_its_items_encoded_charter_and_identity_value() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let assignment = assignment_with_identity();
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let event =
        crate::assignment::down(assignment, &settings.forges, &encoding, &values, Token::new(8), Time::ZERO).unwrap();
    let jig_protocol_engine_workers::Event::Down {
        channel,
        down: jig_domain_host::LinkDown::Assign { assignment },
        parts,
        values,
    } = event
    else {
        panic!("assignment event")
    };
    assert_eq!(channel, Token::new(8));
    assert_eq!(assignment.assignment.run, Token::new(7));
    assert_eq!(assignment.assignment.attempt, Token::new(2));
    assert_eq!(
        assignment.assignment.identity,
        [118, 213, 170, 119, 186, 221, 240, 223, 121, 163, 222, 185, 198, 157, 28, 125]
    );
    assert!(!assignment.assignment.save);
    let workspace = assignment.assignment.workspace.unwrap();
    assert_eq!(workspace.workstream, 7);
    assert_eq!(workspace.items, parts[0].token);
    let items =
        temper_protocol_worker_wire::v1::Items::decode(&encoding.limits, &mut skein_lib::Reader::new(&parts[0].bytes))
            .unwrap();
    assert_eq!(items.repositories().get(0).unwrap().remote(), b"https://forge.example/acme/shop.git");
    let charter = decoded_charter(&assignment.assignment.charter);
    assert!(charter.resume());
    let value = fixtures::decoded_identity(&values[0].as_ref().unwrap().value);
    assert_eq!(value.token().expose(), b"forge-token-1234");
}
#[test]
fn an_assignment_that_cannot_encode_its_items_is_refused() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    assert!(crate::assignment::down(assignment, &[], &encoding, &values, Token::new(8), Time::ZERO).is_none());
}
#[test]
fn a_bad_or_missing_answer_part_discards_all_of_its_parts() {
    use jig_domain_host as host;
    use skein_lib::Token;
    let limits = fixtures::wire_limits();
    let resource = temper_protocol_worker_wire::v1::Preparation::Refused(
        temper_protocol_worker_wire::v1::Tag::new(&limits, temper_protocol_worker_wire::v1::TagParts { tag: 7 })
            .unwrap(),
    );
    let up = fixtures::answer(host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: Some(Token::new(2)) }),
        detail: Box::new([]),
        work: host::Work { left: Some(Token::new(1)), saved: None },
    });
    for bytes in [b"".as_slice(), &[0, 0, 0, 0, 3], &[0, 0, 0, 1]] {
        let parts = [
            jig_protocol_worker_wire::Part { token: Token::new(1), bytes: bytes.into() },
            jig_protocol_worker_wire::Part {
                token: Token::new(2),
                bytes: temper_protocol_worker_wire::preparation(&resource),
            },
        ];
        assert_eq!(crate::workers::parts(&up, &parts, &limits), None, "{bytes:?}");
    }
    assert_eq!(crate::workers::parts(&up, &[], &limits), None, "missing part");
}
#[test]
fn a_named_preparation_part_reaches_the_roots_parts() {
    use jig_domain_host as host;
    use skein_lib::Token;
    let limits = fixtures::wire_limits();
    let resource = temper_protocol_worker_wire::v1::Preparation::Refused(
        temper_protocol_worker_wire::v1::Tag::new(&limits, temper_protocol_worker_wire::v1::TagParts { tag: 7 })
            .unwrap(),
    );
    let up = fixtures::answer(host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: Some(Token::new(2)) }),
        detail: Box::new([]),
        work: host::Work { left: None, saved: None },
    });
    let parts = [jig_protocol_worker_wire::Part {
        token: Token::new(2),
        bytes: temper_protocol_worker_wire::preparation(&resource),
    }];
    assert_eq!(
        crate::workers::parts(&up, &parts, &limits).unwrap().preparation,
        Some(temper_engine_domain::Preparation::Refused { tag: 7 })
    );
}

#[test]
fn an_assignment_preserves_its_ordered_messages_and_answered_calls() {
    let settings = fixtures::engine();
    let encoding = fixtures::encoding(&settings);
    let mut assignment = assignment_with_identity();
    assignment.messages = Box::new([
        word(3, tasks::Party::Person(8), b"first"),
        word(5, tasks::Party::Deployment { project: 1 }, b"second"),
    ]);
    assignment.settled = Box::new([core::SettledCall {
        serial: 6,
        name: b"call".as_slice().into(),
        tool: b"note".as_slice().into(),
        answer: core::SettledAnswer::Host { error: false, body: b"done".as_slice().into() },
    }]);
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let event =
        crate::assignment::down(assignment, &settings.forges, &encoding, &values, Token::new(8), Time::ZERO).unwrap();
    let jig_protocol_engine_workers::Event::Down { down: host::LinkDown::Assign { assignment }, .. } = event else {
        panic!("assignment")
    };
    assert_eq!(assignment.messages.len(), 2);
    for (index, number, sender, words) in
        [(0, 3, b"8".as_slice(), b"first".as_slice()), (1, 5, b"deployment".as_slice(), b"second".as_slice())]
    {
        assert_eq!(assignment.messages[index].name, Token::new(number), "message {index}");
        assert_eq!(assignment.messages[index].sender.as_ref(), sender, "message {index}");
        assert_eq!(assignment.messages[index].words.as_ref(), words, "message {index}");
    }
    assert_eq!(
        assignment.answered.as_ref(),
        &[host::AnsweredCall {
            name: b"call".as_slice().into(),
            tool: b"note".as_slice().into(),
            answer: host::SettledAnswer::Host { error: false, body: b"done".as_slice().into() }
        }]
    );
    assert_eq!(assignment.turns.as_ref(), &[Box::from(b"turn".as_slice())]);
}

fn word(number: u64, from: jig_domain_tasks::Party, words: &[u8]) -> jig_domain_tasks::Word {
    jig_domain_tasks::Word {
        number,
        from,
        words: words.into(),
        kind: jig_domain_tasks::MessageKind::Words,
        at: skein_lib::Wall::EPOCH,
        hits: 0,
        eligible: true,
    }
}

fn decoded_charter(bytes: &[u8]) -> smith_charter::v2::Charter {
    smith_charter::v2::Charter::decode(&smith_charter::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap()
}
