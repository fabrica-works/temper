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
        tokens::SERVER,
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
    let rows: [(&[u8], u64); 7] = [
        (b"forge.repositories", 3),
        (b"forge.client.calls", 5),
        (b"forge.client.settle_ms", 2_000),
        (b"checkout.remote_timeout_ms", 7_000),
        (b"root.loads.rows", 9),
        (b"clients.bundle_files", 2),
        (b"clients.bundle_bytes", 61),
    ];
    for (name, value) in rows {
        assert_eq!(limits::apply(&mut limits, name, value), Ok(()), "{name:?}");
    }
    assert_eq!(limits.forge.repositories, 3);
    assert_eq!(limits.forge.client.calls, 5);
    assert_eq!(limits.forge.client.settle, Duration::from_secs(2));
    assert_eq!(limits.checkout.remote_timeout, Duration::from_millis(7_000));
    assert_eq!(limits.root.loads.rows, 9);
    assert_eq!(limits.clients, limits::Clients { bundle_files: 2, bundle_bytes: 61 });
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
    assert_eq!(temper_worker_protocol_git::check(&settings.limits.git_component, &settings.limits.checkout), Ok(()));
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

#[test]
fn each_git_component_limit_has_its_dotted_override() {
    let names: [(&[u8], [u32; 6]); 6] = [
        (b"git_component.operations", [7, 1024, 512, 4096, 16, 4080]),
        (b"git_component.stdout_bytes", [3, 7, 512, 4096, 16, 4080]),
        (b"git_component.stderr_bytes", [3, 1024, 7, 4096, 16, 4080]),
        (b"git_component.read_bytes", [3, 1024, 512, 7, 16, 4080]),
        (b"git_component.sweep_entries", [3, 1024, 512, 4096, 7, 4080]),
        (b"git_component.sweep_bytes", [3, 1024, 512, 4096, 16, 7]),
    ];
    for (name, expected) in names {
        let mut limits = limits::profile(limits::Profile::Small);
        limits::apply(&mut limits, name, 7).unwrap();
        let git = limits.git_component;
        assert_eq!(
            [git.operations, git.stdout_bytes, git.stderr_bytes, git.read_bytes, git.sweep_entries, git.sweep_bytes],
            expected,
            "{name:?}"
        );
        assert!(temper_worker_protocol_git::worst_case(&git, &limits.checkout).is_some(), "{name:?}");
    }
}

#[test]
fn the_small_profile_gives_settle_thirty_seconds() {
    assert_eq!(limits::profile(limits::Profile::Small).forge.client.settle, Duration::from_secs(30));
}

#[test]
fn the_hook_limits_have_dotted_overrides() {
    type Row = (&'static [u8], u64, u64, fn(&limits::Limits) -> u64);
    let mut own = limits::profile(limits::Profile::Small);
    let rows: [Row; 5] = [
        (b"forge_component.hooks", 3, 3, |l| u64::from(l.forge_component.hooks)),
        (b"forge_component.hook_head_bytes", 4096, 4096, |l| u64::from(l.forge_component.hook_head_bytes)),
        (b"forge_component.hook_body_bytes", 8192, 8192, |l| u64::from(l.forge_component.hook_body_bytes)),
        (b"forge_component.hook_secret_bytes", 64, 64, |l| u64::from(l.forge_component.hook_secret_bytes)),
        (b"forge_component.hook_deadline_ms", 7000, 7_000_000_000, |l| l.forge_component.hook_deadline.as_nanos()),
    ];
    for (name, input, expected, read) in rows {
        assert_eq!(limits::apply(&mut own, name, input), Ok(()), "{name:?}");
        assert_eq!(read(&own), expected, "{name:?}");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests inspect diagnostic text")]
fn records_and_installations_override_and_check_both_bounds() {
    let jig = fixtures::engine().jig;
    let defaults = limits::profile(limits::Profile::Small);
    assert_eq!(defaults.records.value_bytes, 4096);
    assert_eq!(defaults.forges.installations, 8);
    assert_eq!(defaults.forge_component.installations, 8);
    assert_eq!(defaults.forge_component.key_bytes, u32::try_from(skein_tls::RS256_KEY_BYTES).unwrap());
    assert_eq!(defaults.forge_component.renew_before, Duration::from_secs(600));
    for (name, value, other) in [
        (b"records.value_bytes".as_slice(), 0, b"forge_component.key_bytes".as_slice()),
        (
            b"records.value_bytes",
            u64::from(crate::startup::checks::secret_value_bound(&jig, &defaults).unwrap()) + 1,
            b"engine.limits.store.value",
        ),
        (b"records.value_bytes", u64::from(defaults.forge_component.key_bytes), b"forge_component.key_bytes"),
        (b"forges.installations", 512, b"records.value_bytes"),
        (b"forge_component.key_bytes", u64::from(u32::MAX), b"AppKey"),
        (b"forges.installations", u64::from(u32::MAX), b"Accepted"),
        (b"forges.installations", 0, b"forge_component.installations"),
        (b"forges.installations", 9, b"forge_component.installations"),
        (b"forge_component.installations", 7, b"forges.installations"),
    ] {
        let mut own = defaults;
        limits::apply(&mut own, name, value).unwrap();
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::records(&jig, &own, &[], &mut problems);
        let lines = crate::startup::lines(&problems);
        assert!(lines.windows(name.len()).any(|part| part == name), "{name:?}");
        assert!(lines.windows(other.len()).any(|part| part == other), "{other:?}");
    }
    let mut own = defaults;
    for (name, value) in [(b"forge_component.key_bytes".as_slice(), 2000), (b"forge_component.renew_before_ms", 60_000)]
    {
        limits::apply(&mut own, name, value).unwrap();
    }
    assert_eq!(own.forge_component.key_bytes, 2000);
    assert_eq!(own.forge_component.renew_before, Duration::from_secs(60));
    let mut problems = crate::startup::Problems::empty();
    crate::startup::checks::records(&jig, &own, &[], &mut problems);
    assert!(problems.is_empty());
}
#[test]
fn encoded_setup_writes_fit_exactly_and_name_the_short_bound() {
    use crate::startup::{Problems, checks};
    use temper_protocol_records as records;
    let settings = fixtures::engine();
    let mut own = settings.limits;
    let key = skein_lib::bytes::zeroed(usize::try_from(own.forge_component.key_bytes).unwrap());
    assert_eq!(records::encode(&key).len(), 1197); // DER bound plus VERSION.
    let mut ids = skein_lib::List::with_capacity(150);
    for id in 1..=150 {
        ids.push(id).unwrap();
    }
    assert_eq!(records::encode(skein_forge_setup::item::accepted_item(ids.as_slice()).expose()).len(), 1201);
    for (installations, value, expected) in [
        (8, 1197, ""),
        (
            8,
            1196,
            "application.limits.overrides.records.value_bytes: must agree with AppKey (forge_component.key_bytes)\n",
        ),
        (150, 1201, ""),
        (
            150,
            1200,
            "application.limits.overrides.records.value_bytes: must agree with Accepted (forges.installations)\n",
        ),
    ] {
        own.forges.installations = installations;
        own.forge_component.installations = installations;
        own.records.value_bytes = value;
        let mut problems = Problems::empty();
        checks::records(&settings.jig, &own, &[], &mut problems);
        assert_eq!(crate::startup::lines(&problems).as_ref(), expected.as_bytes());
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests inspect diagnostic text")]
fn origin_inventory_keeps_configured_text_prefixes_and_forge_indices() {
    use crate::startup::checks;
    use jig_protocol_config::{Address, Directories};
    use temper_protocol_records as records;
    let document = fixtures::document();
    let input = super::startup::github_input(core::str::from_utf8(&document).unwrap());
    let mut forges = skein_lib::List::with_capacity(5);
    forges.push(crate::named(&document, &Directories::working()).unwrap().forges.into_vec().pop().unwrap()).unwrap();
    for (api, length) in [
        (Address::Host { name: Box::from(b"api.github.com".as_slice()), port: 443 }, 20),
        (Address::Host { name: Box::from(b"forge.example".as_slice()), port: 12345 }, 21),
        (Address::Addr(skein_io::Addr::Ip("127.0.0.1:3000".parse().unwrap())), 16),
        (Address::Addr(skein_io::Addr::Ip("[::1]:3000".parse().unwrap())), 12),
    ] {
        let mut forge =
            crate::named(input.as_bytes(), &Directories::working()).unwrap().forges.into_vec().pop().unwrap();
        forge.api = api;
        let encoded =
            records::encode(&records::origin(skein_forge_connection::settings::Kind::GitHub, &forge.api).encode());
        assert_eq!(encoded.len(), length);
        assert_eq!(&encoded[..2], &[records::VERSION, 2]);
        let index = usize::try_from(forges.len()).unwrap();
        forges.push(forge).unwrap();
        let inventory = checks::record_writes(&fixtures::engine().limits, forges.as_slice());
        assert_eq!(inventory.len(), forges.len() + 2);
        assert_eq!(
            *inventory.get(u32::try_from(index + 2).unwrap()).unwrap(),
            (Some(u64::try_from(length).unwrap()), b"Origin".as_slice(), Some(index))
        );
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary tests inspect diagnostic text")]
fn github_component_checks_key_deadline_and_renewal() {
    for (name, value, expected) in [
        (b"forge_component.key_bytes".as_slice(), 1195, "forge_component.key_bytes"),
        (b"forge_component.deadline_ms", 271_000, "forge_component.deadline_ms"),
        (b"forge_component.renew_before_ms", 19_999, "forge_component.renew_before_ms"),
        (b"forge_component.renew_before_ms", 1_800_001, "forge_component.renew_before_ms"),
        (b"forge_component.installations", 0, "forge_component"),
    ] {
        let mut config = fixtures::engine();
        config.forges[0].kind = crate::settings::Kind::GitHub;
        config.forges[0].api_token = None;
        limits::apply(&mut config.limits, name, value).unwrap();
        if name == b"forge_component.deadline_ms" {
            config.limits.forge.client.lifetime = Duration::from_secs(300);
        }
        let Err(found) = crate::forge::prepare(&config, fixtures::forge_runtime()) else { panic!("{expected}") };
        let found = crate::startup::Problems::jig(found, &jig_protocol_config::Path::root());
        let lines = crate::startup::lines(&found);
        assert!(core::str::from_utf8(&lines).unwrap().contains(expected), "{expected}: {lines:?}");
    }
}
