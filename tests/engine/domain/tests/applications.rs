//! The forge's keyed adoption through the root (domain/forge.md, 4).
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain::Delivery;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures};

fn driver() -> Driver {
    let mut driver = Driver::forge();
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}
fn adopt(driver: &mut Driver, request: forge::Adoption, reply: u64) {
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(reply)),
        sign_in: driver.session(),
        key: [95; 16],
        digest: [5; 32],
        request,
    }));
    driver.settle();
}
fn outcomes(driver: &Driver) -> Vec<&Option<forge::ApplicationOutcome>> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| if let Delivery::Application { outcome, .. } = delivery { Some(outcome) } else { None })
        .collect()
}

#[test]
fn an_adoption_naming_a_forge_not_configured_is_refused_as_no_such_forge() {
    let mut driver = driver();
    adopt(&mut driver, fixtures::adoption(2), 95);
    assert_eq!(
        outcomes(&driver),
        [&Some(forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::NoSuchForge))]
    );
    assert_eq!(driver.forge.as_ref().expect("a forge stand").calls, 0);
}

#[test]
fn an_adoption_on_a_configured_forge_is_adopted_with_what_temper_may_do() {
    let mut driver = driver();
    adopt(&mut driver, fixtures::adoption(1), 95);
    assert_eq!(
        outcomes(&driver),
        [&Some(forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
            repository: temper_engine_domain_forge_client::api::Repository { forge: 1, repository: 2 },
            owner: Box::from(&b"org"[..]),
            name: Box::from(&b"repo"[..]),
            role: forge::Role::Owned,
            home: true,
            kinds: forge::Kinds {
                read: true,
                push: true,
                open: true,
                land: true,
                review: true,
                status: true,
                comment: true,
                issue: true,
                branch: true
            },
            protection: forge::Protection::Absent,
        }))]
    );
}

#[test]
fn an_adoption_sent_again_under_its_key_is_answered_its_kept_outcome() {
    let mut driver = driver();
    adopt(&mut driver, fixtures::adoption(1), 95);
    let calls = driver.forge.as_ref().expect("forge stand").calls;
    let first = outcomes(&driver)[0].clone();
    adopt(&mut driver, fixtures::adoption(1), 96);
    assert_eq!(outcomes(&driver), [&first, &first]);
    assert_eq!(driver.forge.as_ref().expect("forge stand").calls, calls);
}

#[test]
fn an_adoption_the_forge_user_may_not_read_is_refused_as_no_access() {
    let mut driver = driver();
    let mut request = fixtures::adoption(1);
    request.name = b"private".as_slice().into();
    request.provider.repository = 3;
    adopt(&mut driver, request, 95);
    assert_eq!(outcomes(&driver), [&Some(forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::NoAccess))]);
}

#[test]
fn an_adoption_sent_again_after_a_restart_is_answered_its_kept_outcome() {
    let mut first = driver();
    adopt(&mut first, fixtures::adoption(1), 95);
    let outcome = outcomes(&first)[0].clone();
    let calls = first.forge.as_ref().expect("forge stand").calls;
    let mut restarted =
        Driver::configured(first.store, temper_engine_domain_world::walking::config(71), &fixtures::forge_limits());
    restarted.forge = first.forge;
    restarted.settle();
    restarted.sign_in();
    restarted.settle();
    let calls_after_restart = restarted.forge.as_ref().expect("forge stand").calls;
    assert!(calls_after_restart >= calls);
    adopt(&mut restarted, fixtures::adoption(1), 96);
    assert_eq!(outcomes(&restarted), [&outcome]);
    assert_eq!(restarted.forge.as_ref().expect("forge stand").calls, calls_after_restart);
}
