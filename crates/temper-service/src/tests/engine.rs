use super::fixtures;
use skein_lib::{Duration, Time};
use temper_engine_domain as root;

#[test]
fn the_root_reports_the_earliest_credentials_expiry() {
    let (mut domain, mut env) = fixtures::root();
    env.now = Time::from_nanos(5_000_000_000);
    assert_eq!(domain.next_deadline(&env), None);
    for (number, valid, expected) in [(1, 7, 12_u64), (2, 2, 7)] {
        root::try_step(
            &mut domain,
            &env,
            root::Event::Credential { number, generation: 1, valid: Duration::from_secs(valid) },
        )
        .expect("credential admitted");
        assert_eq!(domain.next_deadline(&env), Some(Time::from_nanos(expected.checked_mul(1_000_000_000).unwrap())));
    }
}

#[test]
fn the_forge_reports_the_earliest_procedure_or_projection_wait() {
    let own = crate::limits::profile(crate::limits::Profile::Small);
    let mut domain = temper_engine_domain_forge::Domain::new(
        &own.forge,
        7,
        temper_engine_domain_forge_client::Config {
            namespace: Box::new([7; 16]),
            writers: Box::new([temper_engine_domain_forge_client::Writer { forge: 1, author: 42 }]),
        },
        &[temper_engine_domain_forge::GitIdentity { forge: 1, credential: 1 }],
    )
    .unwrap();
    let env = skein_lib::Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: own.forge };
    assert_eq!(domain.next_wall_deadline(), None);
    domain.wait_change(&env, 1, Some(skein_lib::Wall::from_nanos(7)));
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(7)));
    domain.projection_after(2, skein_lib::Wall::from_nanos(3));
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(3)));
    domain.projection_settled(2);
    assert_eq!(domain.next_wall_deadline(), Some(skein_lib::Wall::from_nanos(7)));
    domain.offered_change(1);
    assert_eq!(domain.next_wall_deadline(), None);
}
