use super::fixtures;
use crate::{End, Stop, forge, limits};
use skein_forge_connection::{StartRead, Stopped};
use skein_lib::{Duration, Time, Token};
use temper_engine_domain as root;
#[test]
fn a_refused_start_check_stops_the_engine_naming_the_forge() {
    let mut service = fixtures::started();
    let mut forge = fixtures::ForgeFace::new();
    forge.connected(&mut service);
    forge.answer(&mut service, b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n");
    for _ in 0_u32..64 {
        forge.turn(&mut service);
    }
    assert_eq!(
        service.end(),
        Some(&End::Stopped(Stop::Forge { number: 1, why: Stopped::Refused { read: StartRead::User, status: 401 } }))
    );
}
#[test]
fn a_forge_call_waiting_past_its_deadline_reaches_the_root_unavailable() {
    let mut service = fixtures::started();
    let call = Token::new(7);
    let mut face = fixtures::ForgeFace::new();
    face.at(&mut service, Time::from_nanos(1));
    face.connected(&mut service);
    face.refuse(&mut service);
    crate::route::request(
        &mut service.engine,
        root::Request::Forge {
            call,
            repository: skein_forge::api::Repository { forge: 1, repository: 2 },
            op: skein_forge::api::Op::Read(skein_forge::api::Read::Settings),
        },
    );
    service.engine.forge.env.now = Time::from_nanos(11_000_000_000);
    service.engine.forge.component.fire(
        &service.engine.forge.env,
        &mut service.engine.forge.above,
        &mut service.engine.forge.io,
    );
    let Some(skein_forge_connection::Event::Answered(answer)) = service.engine.forge.above.pop() else {
        panic!("call terminal")
    };
    assert_eq!(
        answer,
        skein_forge_connection::Answered { call, cost: 0, result: Err(skein_forge::api::Error::Unavailable) }
    );
    let root::Event::Forge(event) = forge::event(answer) else {
        panic!("forge event");
    };
    assert_eq!(forge::back(event).unwrap().call, call);
    assert_eq!(service.engine.forge.above.len(), 0);
}
#[test]
fn an_answer_the_root_cannot_take_is_offered_again_next_pass() {
    let mut service = fixtures::ready();
    for number in 2..16 {
        if root::try_step(&mut service.engine.domain, &service.engine.env, fixtures::period(number)).is_err() {
            break;
        }
    }
    let answer = skein_forge_connection::Answered {
        call: Token::new(7),
        cost: 0,
        result: Err(skein_forge::api::Error::Unavailable),
    };
    assert!(root::try_step(&mut service.engine.domain, &service.engine.env, forge::event(answer.clone())).is_err());
    service.engine.forge.above.push(skein_forge_connection::Event::Answered(answer.clone()));
    crate::engine::forge_pass(&mut service.engine);
    assert_eq!(service.engine.forge.above.len(), 0);
    service.engine.forge.down(skein_forge_connection::Request::Offer);
    let Some(skein_forge_connection::Event::Answered(offered)) = service.engine.forge.above.pop() else {
        panic!("offered answer")
    };
    assert_eq!(offered, answer);
}
#[test]
fn an_answer_returned_in_the_drain_is_never_offered_again() {
    let mut service = fixtures::ready();
    service.shutdown();
    fixtures::ForgeFace::returned(
        &mut service,
        skein_forge_connection::Answered {
            call: Token::new(7),
            cost: 0,
            result: Err(skein_forge::api::Error::Unavailable),
        },
    );
    for _ in 0_u32..4 {
        service.engine.forge.down(skein_forge_connection::Request::Offer);
        assert_eq!(fixtures::ForgeFace::pending(&service), 0);
    }
    for _ in 0_u32..64 {
        fixtures::turn(&mut service);
    }
    assert_eq!(service.end(), Some(&End::Drained));
    assert!(service.engine.forge.closed);
    assert_eq!(service.engine.forge.above.len(), 0);
}
#[test]
fn forge_component_limits_that_disagree_are_problems_naming_both() {
    let rows: [(skein_forge_connection::Unusable, &[u8]); 3] = [
        (skein_forge_connection::Unusable::Connections { connections: 3, calls: 2 }, b"application.limits.overrides.forge_component.connections: past its bound\napplication.limits.overrides.forge.client.calls: past its bound\n"),
        (skein_forge_connection::Unusable::Deadline { deadline: Duration::from_secs(11), lifetime: Duration::from_secs(10) }, b"application.limits.overrides.forge_component.deadline_ms: past its bound\napplication.limits.overrides.forge.client.lifetime_ms: past its bound\n"),
        (skein_forge_connection::Unusable::BodyBytes { body_bytes: 1, needed: 1045 }, b"application.limits.overrides.forge_component.body_bytes: past its bound\napplication.limits.overrides.forge.client.op_bytes: past its bound\n"),
    ];
    for (refusal, expected) in rows {
        let mut found = jig_protocol_config::Problems::empty();
        forge::problems(refusal, &mut found);
        assert_eq!(
            crate::startup::lines(&crate::startup::Problems::jig(found, &jig_protocol_config::Path::root())).as_ref(),
            expected,
            "{refusal:?}"
        );
    }
}
#[test]
fn forge_component_overrides_take_dotted_names_and_ms() {
    type Row = (&'static [u8], u64, u64, fn(&limits::Limits) -> u64);
    let mut own = limits::profile(limits::Profile::Small);
    let rows: [Row; 2] = [
        (b"forge_component.idle_ms", 7000, 7_000_000_000, |limits| limits.forge_component.idle.as_nanos()),
        (b"forge_component.tls.read", 8192, 8192, |limits| u64::from(limits.forge_component.tls.read)),
    ];
    for (name, input, expected, read) in rows {
        assert_eq!(limits::apply(&mut own, name, input), Ok(()), "{name:?}");
        assert_eq!(read(&own), expected, "{name:?}");
    }
    assert_eq!(limits::apply(&mut own, b"forge_component.deadline_ms", u64::MAX), Err(limits::Refused::Range));
}
#[test]
fn a_hint_the_root_cannot_take_is_offered_again_next_pass() {
    let mut service = fixtures::ready();
    let hinted = fixtures::hook(&mut service);
    for number in 2..16 {
        if root::try_step(&mut service.engine.domain, &service.engine.env, fixtures::period(number)).is_err() {
            break;
        }
    }
    assert!(root::try_step(&mut service.engine.domain, &service.engine.env, forge::hinted(hinted.clone())).is_err());
    service.engine.forge.above.push(skein_forge_connection::Event::Hint(hinted.clone()));
    crate::engine::forge_pass(&mut service.engine);
    assert_eq!(service.engine.forge.above.len(), 0);
    service.engine.forge.down(skein_forge_connection::Request::Offer);
    let Some(skein_forge_connection::Event::Hint(offered)) = service.engine.forge.above.pop() else {
        panic!("offered hint")
    };
    assert_eq!(offered, hinted);
}
#[test]
fn a_lost_webhooks_listener_drains_the_service_and_stops_it() {
    let mut service = fixtures::started();
    service.engine.forge.above.push(skein_forge_connection::Event::ListenerLost);
    crate::engine::forge_pass(&mut service.engine);
    let mut forge = fixtures::ForgeFace::new();
    for _ in 0_u32..64 {
        forge.turn(&mut service);
    }
    assert_eq!(service.end(), Some(&End::Stopped(Stop::Hooks)));
}

#[test]
fn a_hook_secret_bound_names_its_limit() {
    let mut found = jig_protocol_config::Problems::empty();
    forge::problems(skein_forge_connection::Unusable::SecretBytes { bytes: 16 }, &mut found);
    assert_eq!(
        crate::startup::lines(&crate::startup::Problems::jig(found, &jig_protocol_config::Path::root())).as_ref(),
        b"application.limits.overrides.forge_component.hook_secret_bytes: past its bound\n"
    );
}
#[test]
fn a_hook_server_over_io_names_both_limits() {
    let mut found = jig_protocol_config::Problems::empty();
    forge::problems(
        skein_forge_connection::Unusable::Stack {
            stacked: skein_forge_connection::limits::Stacked::ServerOverIo,
            demand: 8192,
            cap: 4096,
        },
        &mut found,
    );
    assert_eq!(crate::startup::lines(&crate::startup::Problems::jig(found, &jig_protocol_config::Path::root())).as_ref(),
        b"application.limits.overrides.forge_component.hook_head_bytes: past its bound\napplication.limits.overrides.io.intake: past its bound\napplication.limits.overrides.io.output: past its bound\n");
}

#[test]
fn temper_word_keeps_the_marker_bytes_and_length() {
    let config = fixtures::engine();
    let mut found = jig_protocol_config::Problems::empty();
    let settings =
        forge::settings(&config.forges, fixtures::forge_runtime(), None, &mut found).expect("fixture settings");
    assert_eq!(settings.word.as_ref(), b"temper");
    assert_eq!(skein_forge::marker::length(&settings.word, 2), Some(25));
    let encoded = skein_forge::marker::encode(&settings.word, &[0xab, 1], b"", 25);
    assert_eq!(encoded.as_deref(), Some(b"<!-- temper:key ab01 -->\n".as_slice()));
}

fn problem_lines(refusal: skein_forge_connection::Unusable) -> Box<[u8]> {
    let mut found = jig_protocol_config::Problems::empty();
    forge::problems(refusal, &mut found);
    crate::startup::lines(&crate::startup::Problems::jig(found, &jig_protocol_config::Path::root()))
}

#[test]
fn the_component_s_new_refusals_are_problems_naming_their_members() {
    let rows: [(skein_forge_connection::Unusable, &[u8]); 2] = [
        (skein_forge_connection::Unusable::Word, b"application.limits.overrides.forge_component: past its bound\n"),
        (skein_forge_connection::Unusable::PlaintextName { forge: 1 }, b"application.forges[1].api: past its bound\n"),
    ];
    for (refusal, expected) in rows {
        assert_eq!(problem_lines(refusal).as_ref(), expected, "{refusal:?}");
    }
}

#[test]
fn the_forge_fixture_number_differs_from_its_index() {
    let config = fixtures::engine();
    assert_ne!(config.forges[0].number, 0);
}

#[test]
fn a_refusal_naming_a_forge_is_indexed_to_the_configured_forge() {
    let config = fixtures::engine();
    let refusal = skein_forge_connection::Unusable::PlaintextName { forge: config.forges[0].number };
    assert_eq!(
        problem_lines(forge::indexed(refusal, &config.forges)).as_ref(),
        b"application.forges[0].api: past its bound\n"
    );
}

#[test]
fn the_start_request_carries_the_user_agent_and_the_host() {
    let mut service = fixtures::started();
    let mut face = fixtures::ForgeFace::new();
    face.connected(&mut service);
    assert_eq!(
        face.sent(),
        b"GET /api/v1/user HTTP/1.1\r\nHost: forge.example:3000\r\n\
        Authorization: token api-token\r\nUser-Agent: temper/0.0.0\r\n\r\n"
    );
}

// Small operations fit the profile's default room. A larger key needs
// twice its bytes in hex, the six-byte word and the marker's 15-byte framing.
const SMALL_OP_BYTES: u32 = 100;
const LARGE_OP_BYTES: u32 = 10_000;
const PROFILE_BODY_BYTES: u32 = 16_384;
const LARGE_BODY_BYTES: u32 = 2 * LARGE_OP_BYTES + 6 + 15;

#[test]
fn the_small_profile_keeps_the_body_room_the_word_derives() {
    for (op_bytes, expected) in [(SMALL_OP_BYTES, PROFILE_BODY_BYTES), (LARGE_OP_BYTES, LARGE_BODY_BYTES)] {
        let mut limits = limits::profile(limits::Profile::Small).forge;
        limits.client.op_bytes = op_bytes;
        assert_eq!(forge::small(&limits).body_bytes, expected, "op_bytes={op_bytes}");
    }
}

#[test]
fn the_shared_numbers_are_the_clients_plain_numbers() {
    let config = fixtures::engine();
    let limits = &config.limits;
    let io = &config.jig.limits.io;
    assert_eq!(
        forge::shared(limits, io),
        skein_forge_connection::Shared {
            calls: limits.forge.client.calls,
            rows: limits.forge.client.rows,
            answer_bytes: limits.forge.client.answer_bytes,
            op_bytes: limits.forge.client.op_bytes,
            lifetime: limits.forge.client.lifetime,
            name_bytes: limits.forge.name_bytes,
            repositories: limits.forge.repositories,
            io: *io,
        }
    );
}

const HEAD_PROBLEMS: &[u8] = b"application.forges[0].api_token: past its bound\n\
    application.limits.overrides.forge_component.target_bytes: past its bound\n\
    application.limits.overrides.forge_component.token_bytes: past its bound\n\
    application.limits.overrides.forge_component.tls.send: past its bound\n\
    application.limits.overrides.io.output: past its bound\n";

#[test]
fn a_head_past_the_request_room_is_a_problem_naming_its_members_and_the_forge() {
    let forges = fixtures::forges();
    let refusal = skein_forge_connection::Unusable::Request { forge: forges[0].number };
    assert_eq!(problem_lines(forge::indexed(refusal, &forges)).as_ref(), HEAD_PROBLEMS);
}

// The output room is 1,048,576 bytes, less the fixed 192, Host's 18,
// token's 9 and user agent's 12 bytes: exactly 1,048,345 target bytes.
const FITTING_TARGET_BYTES: u32 = 1_048_345;
const REFUSED_TARGET_BYTES: u32 = 1_048_346;

#[test]
fn the_longest_head_of_the_shipped_forge_fits_its_room_to_the_byte() {
    let mut config = fixtures::engine();
    for (target_bytes, expected) in [(FITTING_TARGET_BYTES, None), (REFUSED_TARGET_BYTES, Some(HEAD_PROBLEMS))] {
        config.limits.forge_component.target_bytes = target_bytes;
        let problems = match forge::prepare(&config, fixtures::forge_runtime()) {
            Ok(_) => None,
            Err(found) => {
                Some(crate::startup::lines(&crate::startup::Problems::jig(found, &jig_protocol_config::Path::root())))
            }
        };
        assert_eq!(problems.as_deref(), expected, "target_bytes={target_bytes}");
    }
}
