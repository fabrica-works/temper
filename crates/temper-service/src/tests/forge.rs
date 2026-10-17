use super::fixtures;
use crate::{End, Stop, forge, limits};
use skein_lib::{Duration, Time, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge_client as client;
use temper_engine_protocol_forge::{self as kit, StartRead, Stopped};
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
            repository: client::api::Repository { forge: 1, repository: 2 },
            op: client::api::Op::Read(client::api::Read::Settings),
        },
    );
    service.engine.forge.env.now = Time::from_nanos(11_000_000_000);
    service.engine.forge.component.fire(
        &service.engine.forge.env,
        &mut service.engine.forge.above,
        &mut service.engine.forge.io,
    );
    let Some(kit::Event::Answered(answer)) = service.engine.forge.above.pop() else { panic!("call terminal") };
    assert_eq!(answer, kit::Answered { call, cost: 0, result: Err(client::api::Error::Unavailable) });
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
    let answer = kit::Answered { call: Token::new(7), cost: 0, result: Err(client::api::Error::Unavailable) };
    assert!(root::try_step(&mut service.engine.domain, &service.engine.env, forge::event(answer.clone())).is_err());
    service.engine.forge.above.push(kit::Event::Answered(answer.clone()));
    crate::engine::forge_pass(&mut service.engine);
    assert_eq!(service.engine.forge.above.len(), 0);
    service.engine.forge.down(kit::Request::Offer);
    let Some(kit::Event::Answered(offered)) = service.engine.forge.above.pop() else { panic!("offered answer") };
    assert_eq!(offered, answer);
}
#[test]
fn an_answer_returned_in_the_drain_is_never_offered_again() {
    let mut service = fixtures::ready();
    service.shutdown();
    fixtures::ForgeFace::returned(
        &mut service,
        kit::Answered { call: Token::new(7), cost: 0, result: Err(client::api::Error::Unavailable) },
    );
    for _ in 0_u32..4 {
        service.engine.forge.down(kit::Request::Offer);
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
    let rows: [(kit::Unusable, &[u8]); 3] = [
        (kit::Unusable::Connections { connections: 3, calls: 2 }, b"application.limits.overrides.forge_component.connections: past its bound\napplication.limits.overrides.forge.client.calls: past its bound\n"),
        (kit::Unusable::Deadline { deadline: Duration::from_secs(11), lifetime: Duration::from_secs(10) }, b"application.limits.overrides.forge_component.deadline_ms: past its bound\napplication.limits.overrides.forge.client.lifetime_ms: past its bound\n"),
        (kit::Unusable::BodyBytes { body_bytes: 1, needed: 1045 }, b"application.limits.overrides.forge_component.body_bytes: past its bound\napplication.limits.overrides.forge.client.op_bytes: past its bound\n"),
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
    service.engine.forge.above.push(kit::Event::Hint(hinted.clone()));
    crate::engine::forge_pass(&mut service.engine);
    assert_eq!(service.engine.forge.above.len(), 0);
    service.engine.forge.down(kit::Request::Offer);
    let Some(kit::Event::Hint(offered)) = service.engine.forge.above.pop() else { panic!("offered hint") };
    assert_eq!(offered, hinted);
}
#[test]
fn a_lost_webhooks_listener_drains_the_service_and_stops_it() {
    let mut service = fixtures::started();
    service.engine.forge.above.push(kit::Event::ListenerLost);
    crate::engine::forge_pass(&mut service.engine);
    let mut forge = fixtures::ForgeFace::new();
    for _ in 0_u32..64 {
        forge.turn(&mut service);
    }
    assert_eq!(service.end(), Some(&End::Stopped(Stop::Hooks)));
}
