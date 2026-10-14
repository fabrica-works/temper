use super::fixtures;
use crate::{limits, tokens};
use skein_lib::{Duration, Token};

#[test]
fn no_two_component_prefixes_meet() {
    let owners = [
        tokens::WORKER_LISTENERS,
        tokens::WORKER_CONNECTIONS,
        tokens::ACCOUNTS,
        tokens::CLIENTS,
        tokens::FORGE,
        tokens::GIT,
        tokens::STORE[0],
        tokens::STORE[1],
        tokens::STORE[2],
    ];
    for (index, owner) in owners.iter().enumerate() {
        for other in owners.iter().skip(index + 1) {
            assert_ne!(tokens::prefix(Token::new(*owner)), tokens::prefix(Token::new(*other)), "{owner:x}, {other:x}");
        }
    }
    assert_eq!(tokens::prefix(Token::new(tokens::SIGNALS)), 0);
    for index in [0, 1, 4094] {
        assert_eq!(tokens::worker(index), Some((3_u64 << 60_u32) + (u64::from(index + 1) << 48_u32)));
        assert_eq!(tokens::git(index), Some((6_u64 << 60_u32) + (u64::from(index + 1) << 48_u32)));
    }
    assert_ne!(tokens::worker(0), Some(tokens::ACCOUNTS));
    for index in [4095, 4096, u32::MAX] {
        assert_eq!(tokens::worker(index), None, "{index}");
        assert_eq!(tokens::git(index), None, "{index}");
    }
}

#[test]
fn an_override_sets_the_field_its_dotted_name_names() {
    let mut limits = limits::profile(limits::Profile::Small);
    let rows: [(&[u8], u64); 4] = [
        (b"forge.repositories", 3),
        (b"forge.client.calls", 5),
        (b"checkout.remote_timeout_ms", 7_000),
        (b"root.loads.rows", 9),
    ];
    for (name, value) in rows {
        assert_eq!(limits::apply(&mut limits, name, value), Ok(()), "{name:?}");
    }
    assert_eq!(limits.forge.repositories, 3);
    assert_eq!(limits.forge.client.calls, 5);
    assert_eq!(limits.checkout.remote_timeout, Duration::from_millis(7_000));
    assert_eq!(limits.root.loads.rows, 9);
}

#[test]
fn an_unknown_name_or_a_value_past_its_type_is_refused() {
    let mut limits = limits::profile(limits::Profile::Small);
    let before = limits;
    let rows: [(&[u8], u64, limits::Refused); 5] = [
        (b"forge.unknown", 1, limits::Refused::Unknown),
        (b"Forge.repositories", 1, limits::Refused::Unknown),
        (b"forge.repositories", u64::from(u32::MAX) + 1, limits::Refused::Range),
        (b"root.loads.rows", u64::MAX, limits::Refused::Range),
        (b"checkout.remote_timeout_ms", u64::MAX, limits::Refused::Range),
    ];
    for (name, value, refused) in rows {
        assert_eq!(limits::apply(&mut limits, name, value), Err(refused), "{name:?}");
        assert_eq!(limits, before, "{name:?}");
    }
}

#[test]
fn the_small_profile_passes_each_domains_worst_case() {
    let settings = fixtures::engine();
    assert!(temper_engine_domain_forge::worst_case(&settings.limits.forge).is_some());
    assert!(temper_worker_domain_checkout::worst_case(&settings.limits.checkout).is_some());
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let root = limits::engine(&jig, &settings.limits).expect("coherent root profile");
    assert!(temper_engine_domain::worst_case(&root).is_some());
}

#[test]
fn the_root_limits_take_jigs_children_and_temper_own() {
    let settings = fixtures::engine();
    let root = limits::engine(&settings.jig.limits, &settings.limits).expect("coherent root limits");
    let jig = settings.jig.limits.core;
    assert_eq!(root.journal, settings.limits.root.journal);
    assert_eq!(root.loads, settings.limits.root.loads);
    assert_eq!(root.brief, settings.limits.root.brief);
    assert_eq!(root.forge, settings.limits.forge);
    assert_eq!(root.tasks, jig.tasks);
    assert_eq!(root.authority, jig.authority);
    assert_eq!(root.people, jig.people);
    assert_eq!(root.fleet, jig.fleet);
    assert_eq!(root.accounts, jig.accounts);
    assert_eq!(root.notes, jig.notes);
    assert_eq!(root.call_records, jig.call_records);
    let mut refused = settings.limits;
    refused.root.journal.transcript_bytes = 1;
    assert_eq!(limits::engine(&settings.jig.limits, &refused), None);
}
