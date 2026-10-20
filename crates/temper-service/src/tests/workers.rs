use super::fixtures;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_protocol_engine_workers::GrantValue;
use skein_lib::Time;
use skein_lib::Token;
#[test]
fn each_repositorys_grant_carries_its_identitys_value() {
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let filled = values.grants(&[fixtures::application_grant(1, 1)], Time::ZERO);
    let [Some(GrantValue::Application { secret })] = filled.as_ref() else { panic!("one application value") };
    let identity = fixtures::decoded_identity(secret);
    assert_eq!(identity.token().expose(), b"forge-token-1234");
    assert_eq!(identity.login(), b"temper");
    assert_eq!(identity.name(), b"temper");
    assert_eq!(identity.email(), b"temper@forge.example");
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
    let _listen = service.engine.workers.io.pop().unwrap();
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
                scheme: jig_protocol_accounts::table::Scheme::Withdrawn {
                    id: skein_lib::Secret::new(b"account-id".as_slice().into()),
                },
            },
            Time::from_nanos(9),
        )
        .unwrap();
    let mut grant = fixtures::application_grant(1, 2);
    grant.form = jig_domain_host::Form::Withdrawn;
    for (now, generation, expected) in [(8, 2, true), (9, 2, false), (8, 1, false)] {
        grant.generation = generation;
        let filled = values.grants(&[grant], Time::from_nanos(now));
        assert_eq!(filled[0].is_some(), expected, "{now} {generation}");
        if let Some(value) = &filled[0] {
            let GrantValue::Withdrawn { id, bearer } = value else { panic!("a withdrawn grant") };
            assert_eq!(bearer.expose(), b"account-token", "{now} {generation}");
            assert_eq!(id.expose(), b"account-id", "{now} {generation}");
        }
    }
}

fn assignment_with_identity() -> temper_engine_domain::Assignment {
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.grants = Box::new([jig_domain_accounts::Grant {
        form: jig_domain_accounts::Form::Application,
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
    let Some(GrantValue::Application { secret }) = &values[0] else { panic!("an application value") };
    let value = fixtures::decoded_identity(secret);
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

fn decoded_charter(bytes: &[u8]) -> smith_charter::Charter {
    smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut skein_lib::Reader::new(bytes)).unwrap()
}

#[test]
fn a_part_that_does_not_decode_gives_no_parts_and_is_counted() {
    let mut workers = fixtures::workers();
    let mut events = skein_lib::Queue::with_capacity(32);
    for (bytes, expected) in [(b"".as_slice(), 1), (&[0, 0, 0, 1], 2)] {
        let up = fixtures::answer(host::Ending::Ended {
            outcome: Box::new([]),
            work: host::Work { left: Some(Token::new(1)), saved: None },
        });
        workers.received(
            Token::new(8),
            up,
            &[jig_protocol_worker_wire::Part { token: Token::new(1), bytes: bytes.into() }],
            &mut events,
        );
        let Some(temper_engine_domain::Event::Worker(temper_engine_domain::WorkerInput::Link {
            channel,
            link: temper_engine_domain::Link::Up { parts, up },
        })) = events.pop()
        else {
            panic!("answer reaches root")
        };
        assert_eq!(channel, Token::new(8), "{bytes:?}");
        assert_eq!(parts, temper_engine_domain::Parts::none(), "{bytes:?}");
        assert_eq!(
            up,
            fixtures::answer(host::Ending::Ended {
                outcome: Box::new([]),
                work: host::Work { left: Some(Token::new(1)), saved: None },
            }),
            "{bytes:?}"
        );
        assert_eq!(workers.bad_parts, expected, "{bytes:?}");
    }
}
#[test]
fn the_workers_listener_and_accepted_socket_have_their_own_prefixes() {
    let mut workers = fixtures::workers();
    let owner = Token::new(crate::tokens::WORKER_LISTENERS);
    let mut events = skein_lib::Queue::with_capacity(32);
    let Some(skein_io::Request::Listen { owner: requested, addr: addr @ skein_io::Addr::InProcess(_) }) =
        workers.io.pop()
    else {
        panic!("configured in-process listener")
    };
    assert_eq!(requested, owner);
    assert!(workers.up(Time::ZERO, skein_io::Event::Listening { owner, listener: Token::new(3), addr }, &mut events));
    assert!(workers.opened());
    assert!(workers.up(
        Time::ZERO,
        skein_io::Event::Accepted { owner, socket: Token::new(4), peer: addr },
        &mut events
    ));
    let Some(skein_io::Request::Bind { socket, owner: connection }) = workers.io.pop() else {
        panic!("accepted socket bound")
    };
    assert_eq!(socket, Token::new(4));
    assert_eq!(connection, Token::new(crate::tokens::WORKER_CONNECTIONS + 1));
    assert_eq!(workers.sockets.get(&connection), Some(&socket));
}
#[test]
fn a_listener_that_opens_during_closing_is_closed_immediately() {
    let mut workers = fixtures::workers();
    let _listen = workers.io.pop().unwrap();
    let mut events = skein_lib::Queue::with_capacity(32);
    workers.close(Time::ZERO, &mut events);
    assert!(workers.up(
        Time::ZERO,
        skein_io::Event::Listening {
            owner: Token::new(crate::tokens::WORKER_LISTENERS),
            listener: Token::new(3),
            addr: skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap())
        },
        &mut events
    ));
    let Some(skein_io::Request::Close { entity }) = workers.io.pop() else { panic!("late listener close") };
    assert_eq!(entity, Token::new(3));
    assert!(!workers.closed());
    assert!(workers.up(
        Time::ZERO,
        skein_io::Event::Closed { owner: Token::new(crate::tokens::WORKER_LISTENERS) },
        &mut events
    ));
    assert!(workers.closed());
}
#[test]
fn an_assignment_that_cannot_encode_is_answered_invalid_through_the_link() {
    let mut workers = fixtures::workers();
    let mut events = skein_lib::Queue::with_capacity(32);
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.workspace.repositories[0].provider.forge = 99;
    workers.assignment(
        Time::ZERO,
        Token::new(8),
        assignment,
        &fixtures::values_with_identity(1, b"forge-token-1234"),
        &mut events,
    );
    let Some(temper_engine_domain::Event::Worker(temper_engine_domain::WorkerInput::Link {
        channel,
        link: temper_engine_domain::Link::Up { up: host::LinkUp::Answer { run, attempt, answer }, parts },
    })) = events.pop()
    else {
        panic!("invalid assignment answer")
    };
    assert_eq!(channel, Token::new(8));
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert_eq!(parts, temper_engine_domain::Parts::none());
    assert_eq!(
        answer.ending,
        host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
            why: b"assignment cannot encode".as_slice().into(),
        }))
    );
}

#[test]
fn the_roots_facts_are_drained_and_counted_after_each_step() {
    use temper_engine_domain as root;
    let (mut service, run) = fixtures::claimed_chat();
    let preparation = root::Preparation::Missing { tag: 3, missing: root::Missing::Branch };
    let up = host::LinkUp::Answer {
        run: Token::new(run.task),
        attempt: Token::new(run.attempt),
        answer: host::Answer {
            read: None,
            refused: Box::new([]),
            turns: 0,
            spent: 1,
            ending: host::Ending::Failed {
                failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: None }),
                detail: Box::new([]),
                work: host::Work { left: None, saved: None },
            },
        },
    };
    service.engine.step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Up {
            up,
            parts: root::Parts { saved: None, pushed: Box::new([]), preparation: Some(preparation) },
        },
    }));
    fixtures::settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32));
    assert_eq!(service.engine.facts, 1);
    assert_eq!(service.engine.domain.pop_fact(), None);
    assert_eq!(service.engine.domain.facts_lost(), 0);
    service.engine.step(root::Event::Resume);
    assert_eq!(service.engine.facts, 1);
}

#[test]
fn a_worker_assignment_uses_the_services_configured_forges() {
    use temper_engine_domain as root;
    let mut service = fixtures::started();
    let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    assignment.workspace.repositories[0].provider.forge = 99;
    crate::route::request(
        &mut service.engine,
        root::Request::Worker(root::WorkerRequest::Deliver(root::Delivery::Assigned {
            channel: Token::new(8),
            assignment,
        })),
    );
    let Some(root::Event::Worker(root::WorkerInput::Link {
        channel,
        link: root::Link::Up { up: host::LinkUp::Answer { run, attempt, answer }, parts },
    })) = service.engine.events.pop()
    else {
        panic!("worker assignment refused through its root input")
    };
    assert_eq!(channel, Token::new(8));
    assert_eq!((run, attempt), (Token::new(7), Token::new(2)));
    assert_eq!(
        answer.ending,
        host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
            why: b"assignment cannot encode".as_slice().into(),
        }))
    );
    assert_eq!(parts, root::Parts::none());
    assert_eq!(service.end(), None);
}

#[test]
fn engine_values_preserve_slots_and_refuse_a_different_declared_form() {
    use jig_protocol_accounts::table::{Scheme, Value};
    let forms = [
        host::Form::Credential(host::Scheme::Bearer),
        host::Form::Credential(host::Scheme::Key),
        host::Form::Withdrawn,
    ];
    for (scheme, actual) in [
        (Scheme::Bearer, forms[0]),
        (Scheme::Key, forms[1]),
        (Scheme::Withdrawn { id: skein_lib::Secret::new(b"provider-account".as_slice().into()) }, forms[2]),
    ] {
        let mut values = fixtures::values_with_identity(1, b"forge-token");
        values
            .accounts
            .granted(
                2,
                3,
                Value { scheme, token: skein_lib::Secret::new(b"llm-token".as_slice().into()) },
                Time::from_nanos(9),
            )
            .unwrap();
        let grants =
            forms.map(|form| host::Grant { form, account: 2, generation: 3, valid: skein_lib::Duration::from_secs(5) });
        let filled = values.grants(&grants, Time::from_nanos(8));
        for (grant, value) in grants.iter().zip(filled.iter()) {
            assert_eq!(value.is_some(), grant.form == actual);
        }
        assert!(values.grants(&grants, Time::from_nanos(9)).iter().all(Option::is_none));
    }
}
