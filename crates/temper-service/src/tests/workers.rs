use super::fixtures;
use skein_lib::Time;
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
