use skein_forge_connection::settings::Kind;
use skein_forge_setup::item::Bounds;
use skein_lib::{Rng, Secret};
use temper_protocol_records::{self as records, Item, NotConnected, Origin, Rows};
fn bounds() -> Bounds {
    Bounds { token_bytes: 64, name_bytes: 64, key_bytes: 1196, installations: 8 }
}
fn expected(kind: Kind) -> Origin {
    Origin { kind, api: Box::from("127.0.0.1:3000") }
}
fn put(rows: &mut Rows, item: Item, bytes: &[u8]) {
    rows.push(records::number(1, item), Secret::new(records::encode(bytes).into_boxed_slice()), 4096);
}
fn complete(kind: Kind) -> Rows {
    let mut rows = Rows::new();
    put(&mut rows, Item::Origin, &expected(kind).encode());
    put(&mut rows, Item::User, &9_u64.to_le_bytes());
    match kind {
        Kind::Forgejo => {
            let byte = u8::try_from(Rng::new(83).between(33, 126)).expect("ASCII");
            put(&mut rows, Item::ApiToken, &[byte; 32]);
        }
        Kind::GitHub => {
            put(&mut rows, Item::AppId, &7_u64.to_le_bytes());
            put(&mut rows, Item::AppSlug, b"test-app");
            put(&mut rows, Item::AppKey, skein_fake_forge_github::keys::KEYS[0]);
        }
    }
    rows
}
#[test]
fn record_numbers_versions_and_origins_have_golden_bytes() {
    let goldens = temper_codecs_world::goldens("crates/temper-protocol-records/golden");
    assert_eq!(records::number(0x1234, Item::Origin).to_le_bytes().as_slice(), goldens[0]);
    assert_eq!(records::encode(&[0x21, 0x42]), goldens[1]);
    for (i, address) in ["127.0.0.1:3000", "[::1]:3000", "forge.test:443", "api.github.com:443"].iter().enumerate() {
        let kind = if i == 3 { Kind::GitHub } else { Kind::Forgejo };
        let configured = if let Ok(socket) = address.parse() {
            jig_protocol_config::Address::Addr(skein_io_address(socket))
        } else {
            let (name, port) = address.rsplit_once(':').expect("host port");
            jig_protocol_config::Address::Host { name: name.as_bytes().into(), port: port.parse().expect("port") }
        };
        let origin = records::origin(kind, &configured);
        assert_eq!(origin.encode().as_ref(), goldens[i + 2]);
        assert_eq!(Origin::decode(&origin.encode()), Ok(origin));
    }
    for item in [
        Item::AdminToken,
        Item::ApiToken,
        Item::GitToken,
        Item::User,
        Item::WebhookSecret,
        Item::AppId,
        Item::AppSlug,
        Item::AppKey,
        Item::Accepted,
        Item::Origin,
    ] {
        assert_eq!(records::parse(records::number(65535, item)), Some((1, 65535, item)));
    }
    for number in [0, 1 << 48, (1 << 48) | 0x000b, (1 << 48) | 0xffff, records::CARRIED, u64::MAX] {
        assert_eq!(records::parse(number), None);
    }
    for bytes in [&b""[..], &[2, 1], &[0]] {
        assert_eq!(records::value(bytes), Err(records::Unreadable));
    }
}
fn skein_io_address(socket: core::net::SocketAddr) -> skein_io::Addr {
    skein_io::Addr::Ip(socket)
}
#[test]
fn access_checks_each_record_and_preserves_the_access_and_accepted_ids() {
    for kind in [Kind::Forgejo, Kind::GitHub] {
        let mut rows = complete(kind);
        put(&mut rows, Item::Accepted, &1_u64.to_le_bytes());
        let byte = u8::try_from(Rng::new(84).between(33, 126)).expect("ASCII");
        put(&mut rows, Item::WebhookSecret, &[byte; 32]);
        for hooks in [false, true] {
            let start = records::access(kind, &expected(kind), rows.forge(1).expect("rows"), &bounds(), hooks)
                .expect("complete");
            assert_eq!(start.access.user, 9);
            assert_eq!(start.access.webhook_secret.is_some(), hooks && kind == Kind::Forgejo);
            match start.access.credential {
                skein_forge::credential::Credential::Token(token) => assert_eq!(token.expose().len(), 32),
                skein_forge::credential::Credential::App { id, key } => {
                    assert_eq!(id, 7);
                    assert!(key.expose() == skein_fake_forge_github::keys::KEYS[0]);
                    assert_eq!(start.accepted.as_ref(), &[1]);
                    assert_eq!(start.slug.as_deref(), Some(b"test-app".as_slice()));
                }
            }
        }
    }
    assert!(Rows::new().forge(1).is_none());
    let mut incomplete = Rows::new();
    put(&mut incomplete, Item::Origin, &expected(Kind::GitHub).encode());
    assert_eq!(
        records::access(Kind::GitHub, &expected(Kind::GitHub), incomplete.forge(1).expect("rows"), &bounds(), false)
            .err(),
        Some(NotConnected::Incomplete)
    );
    for (item, bytes) in [
        (Item::AppId, vec![0; 8]),
        (Item::AppSlug, b"Bad".to_vec()),
        (Item::AppKey, vec![0; 1197]),
        (Item::Accepted, vec![1]),
        (Item::User, vec![0; 8]),
    ] {
        let mut rows = complete(Kind::GitHub);
        put(&mut rows, item, &bytes);
        assert_eq!(
            records::access(Kind::GitHub, &expected(Kind::GitHub), rows.forge(1).expect("rows"), &bounds(), false)
                .err(),
            Some(NotConnected::Unreadable)
        );
    }
    for code in [4, 11] {
        let mut rows = complete(Kind::GitHub);
        rows.push((1 << 48) | (1 << 16) | code, Secret::new(Box::from([2, 1])), 4096);
        assert_eq!(
            records::access(Kind::GitHub, &expected(Kind::GitHub), rows.forge(1).expect("rows"), &bounds(), false)
                .err(),
            Some(NotConnected::Unreadable)
        );
    }
    for kind in [Kind::Forgejo, Kind::GitHub] {
        let mut rows = complete(kind);
        let other = if kind == Kind::Forgejo { Kind::GitHub } else { Kind::Forgejo };
        put(&mut rows, Item::Origin, &expected(other).encode());
        assert_eq!(
            records::access(kind, &expected(kind), rows.forge(1).expect("rows"), &bounds(), false).err(),
            Some(NotConnected::OtherForge { kind: other, api: expected(other).api })
        );
    }
}

#[test]
fn foreign_credentials_and_values_past_the_bound_are_refused() {
    let token = [u8::try_from(Rng::new(85).between(33, 126)).expect("ASCII"); 32];
    for (kind, item, bytes, other) in [
        (Kind::Forgejo, Item::AppKey, skein_fake_forge_github::keys::KEYS[0], Kind::GitHub),
        (Kind::GitHub, Item::ApiToken, token.as_slice(), Kind::Forgejo),
    ] {
        let mut rows = complete(kind);
        put(&mut rows, item, bytes);
        assert_eq!(
            records::access(kind, &expected(kind), rows.forge(1).expect("rows"), &bounds(), false).err(),
            Some(NotConnected::OtherForge { kind: other, api: expected(kind).api })
        );
    }
    let mut rows = complete(Kind::GitHub);
    let start = records::access(Kind::GitHub, &expected(Kind::GitHub), rows.forge(1).expect("rows"), &bounds(), false)
        .expect("complete without Accepted");
    assert!(start.accepted.is_empty());
    rows.push(records::number(1, Item::AppKey), Secret::new(records::encode(&[0x30, 0]).into_boxed_slice()), 1);
    assert_eq!(
        records::access(Kind::GitHub, &expected(Kind::GitHub), rows.forge(1).expect("rows"), &bounds(), false).err(),
        Some(NotConnected::Unreadable)
    );
}
