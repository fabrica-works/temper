//! Secret-safe identity inspection (protocol/channel.md, section 6).
use temper_protocol_worker_wire::v1;

#[test]
fn an_identitys_debug_holds_no_byte_of_its_token() {
    let parts = v1::IdentityParts {
        token: skein_lib::Secret::new(Box::from(&b"forge-token-1234"[..])),
        login: Box::from(&b"temper"[..]),
        name: Box::from(&b"temper"[..]),
        email: Box::from(&b"temper@forge.example"[..]),
    };
    let identity = v1::Identity::new(&v1::CEILINGS, parts).expect("within the ceilings");
    let printed = format!("{identity:?}");
    assert!(!printed.contains("forge-token-1234"), "{printed}");
    assert!(printed.contains("16"), "the token's length is shown: {printed}");
}
