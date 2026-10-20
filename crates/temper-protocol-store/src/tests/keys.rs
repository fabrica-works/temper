use super::fixtures;
use crate::keys;
use skein_lib::Rng;

#[test]
fn encoded_keys_sort_as_the_derived_order() {
    let mut random = Rng::new(0x0401);
    let mut previous = fixtures::key(&mut random);
    for _ in 0_u32..2_000 {
        let next = fixtures::key(&mut random);
        let encoded = keys::encode(&previous, u32::MAX).cmp(&keys::encode(&next, u32::MAX));
        assert_eq!(encoded, previous.cmp(&next), "{previous:?} against {next:?}");
        previous = next;
    }
}

#[test]
fn a_decoded_key_is_the_key_encoded() {
    let mut random = Rng::new(0x0401);
    for _ in 0_u32..2_000 {
        let key = fixtures::key(&mut random);
        let bytes = keys::encode(&key, u32::MAX).expect("a drawn key fits");
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
    }
}

#[test]
fn a_shorter_branch_sorts_before_its_extensions() {
    let shorter = fixtures::branch(Box::new([Box::from([0, 1, 255])]));
    let longer = fixtures::branch(Box::new([Box::from([0, 1, 255]), Box::new([])]));
    assert!(keys::encode(&shorter, u32::MAX) < keys::encode(&longer, u32::MAX));
}

#[test]
fn a_key_decoder_refuses_each_malformed_key() {
    let mut what = skein_kv::KeyWriter::new(u32::MAX);
    what.tag(3).u64(1).u64(2).tag(4);
    let mut forge = skein_kv::KeyWriter::new(u32::MAX);
    forge.tag(2).u64(65_536).u64(2);
    let mut trailing = skein_kv::KeyWriter::new(u32::MAX);
    trailing.tag(4).u64(1).tag(0);
    let mut branch = skein_kv::KeyWriter::new(u32::MAX);
    branch.tag(3).u64(1).u64(2).tag(1);
    for _ in 0_u32..129 {
        branch.tag(1).bytes(&[]);
    }
    branch.tag(0);
    for (name, bytes) in [
        ("family tag", Box::from([17])),
        ("What tag", what.finish().expect("small key")),
        ("forge width", forge.finish().expect("small key")),
        ("trailing byte", trailing.finish().expect("small key")),
        ("branch count", branch.finish().expect("small key")),
        ("cut key", Box::from([4, 0, 0, 0, 0, 0, 0, 0])),
    ] {
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), None, "{name}");
    }
}

#[test]
fn every_forge_key_lies_in_the_interval_and_no_core_key_does() {
    let interval = crate::interval();
    let mut random = Rng::new(0x0402);
    for _ in 0_u32..500 {
        let key = fixtures::key(&mut random);
        let relative = keys::encode(&key, u32::MAX).expect("a drawn key fits");
        let tagged = jig_protocol_store::keys::application(&relative, u32::MAX).expect("tagged");
        assert!(fixtures::within(&interval, &tagged), "{key:?}");
        assert!(temper_engine_domain::Range::Forge.contains(&temper_engine_domain::Key::Forge(key)));
    }
    for key in fixtures::core_keys() {
        let tagged = jig_protocol_store::keys::encode(&key, u32::MAX).expect("a core key fits");
        assert!(!fixtures::within(&interval, &tagged), "{key:?}");
        assert!(!temper_engine_domain::Range::Forge.contains(&temper_engine_domain::Key::Core(key)));
    }
}

#[test]
fn the_largest_key_is_a_branch_of_empty_segments() {
    let mut segments: skein_lib::List<Box<[u8]>> = skein_lib::List::with_capacity(fixtures::LIMITS.name_bytes);
    for _ in 0..fixtures::LIMITS.name_bytes {
        segments.push(Box::new([])).expect("bounded branch");
    }
    let key = fixtures::branch(segments.into_boxed());
    let bytes = keys::encode(&key, u32::MAX).expect("largest key fits");
    assert_eq!(bytes.len(), 403);
    assert_eq!(keys::largest(&fixtures::LIMITS), Some(403));
    assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
}

#[test]
fn application_keys_follow_the_release_family_and_round_trip() {
    use temper_engine_domain_forge::Key;
    let previous = keys::encode(&Key::Release(u64::MAX), u32::MAX).expect("release key");
    for token in [0, 1, u64::MAX] {
        let key = Key::Application(token);
        let bytes = keys::encode(&key, u32::MAX).expect("application key");
        assert_eq!(bytes[0], 15);
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        assert!(bytes > previous);
    }
}

#[test]
fn writer_keys_follow_application_and_take_every_forge_number() {
    use temper_engine_domain_forge::Key;
    let previous = keys::encode(&Key::Application(u64::MAX), u32::MAX).expect("application key");
    for forge in [0, 1, u16::MAX] {
        let key = Key::Writer(forge);
        let bytes = keys::encode(&key, u32::MAX).expect("writer key");
        assert_eq!(bytes[0], 16);
        assert_eq!(bytes.len(), 9);
        assert_eq!(keys::decode(&bytes, &fixtures::LIMITS), Some(key));
        assert!(bytes > previous);
    }
    let mut too_wide = skein_kv::KeyWriter::new(u32::MAX);
    too_wide.tag(16).u64(65_536);
    assert_eq!(keys::decode(&too_wide.finish().expect("small key"), &fixtures::LIMITS), None);
}
