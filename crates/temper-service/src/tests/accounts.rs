use super::fixtures;
use jig_protocol_store::{Event as StoreEvent, SecretOwner, Secrets};
use skein_lib::{Time, Token};

const TOKEN: &[u8] = b"test-token";

#[test]
fn a_handed_in_record_is_kept_before_its_value_is_lent() {
    let mut service = fixtures::started();
    let get = || Secrets::Get { owner: SecretOwner::Source, number: 1 };
    // The accounts are wanted once the core's ranges are restored (as
    // jig-test-domain at the restart's LoadInbox), past temper's restart.
    for _ in 0_u32..24 {
        fixtures::turn(&mut service);
        let lent = service.engine.values.accounts.value(1, 1, Time::from_nanos(1)).is_some();
        let Some(StoreEvent::Secret { value, .. }) = service.engine.store.secrets(Token::new(99), get()) else {
            panic!("a get is answered at once");
        };
        assert!(!lent || value.is_some(), "no value is lent before its record is kept");
    }
    let Some(StoreEvent::Secret { value: Some(kept), .. }) = service.engine.store.secrets(Token::new(99), get()) else {
        panic!("the handed-in record is kept");
    };
    let Ok(record) = skein_oauth::decode_record(kept.expose(), &fixtures::engine().jig.limits.driver.client.document)
    else {
        panic!("a kept record decodes");
    };
    assert_eq!((record.key, &*record.access_token), (1, TOKEN));
    let value = service.engine.values.accounts.value(1, 1, Time::from_nanos(1)).expect("lent value");
    assert_eq!(value.token.expose(), TOKEN);
    let id = skein_lib::Secret::new(b"acc".as_slice().into());
    assert_eq!(value.scheme, jig_protocol_accounts::table::Scheme::Withdrawn { id });
}

#[test]
fn a_signed_in_source_without_runtime_is_refused_at_its_path() {
    let mut settings = fixtures::engine();
    settings.jig.llm.accounts[0].source = fixtures::signed_in_source();
    fixtures::fill_files(&mut settings);
    let Err(found) = crate::Service::engine(settings, &fixtures::seeds(), None, fixtures::forge_runtime()) else {
        panic!("runtime absent");
    };
    assert_eq!(
        found.problems.get(0),
        Some(&jig_protocol_config::Problem {
            path: jig_protocol_config::Path::root()
                .member(b"engine")
                .member(b"llm")
                .member(b"accounts")
                .child(jig_protocol_config::Segment::Index(0))
                .member(b"source"),
            what: jig_protocol_config::What::Refers,
        })
    );
}
