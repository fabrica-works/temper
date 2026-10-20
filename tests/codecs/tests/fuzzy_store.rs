//! Arbitrary and mutated durable rows and keys (protocol/store.md, section 6).
use skein_lib::{Duration, Rng};
use temper_engine_domain_forge as forge;
use temper_protocol_store::{Bounds, keys, records};

const LIMITS: forge::Limits = forge::Limits {
    repositories: 2,
    tasks: 8,
    holds: 8,
    subscriptions: 8,
    entries: 8,
    named_calls: 16,
    changes: 8,
    issues: 8,
    resources_per_task: 4,
    paths_per_subscription: 4,
    name_bytes: 128,
    output: 80,
    facts: 16,
    adoptions: 2,
    applications: 4,
    collaborators: 8,
    landings: 8,
    judge_projects: 2,
    judge_criteria: 8,
    brief_sections: 32,
    brief_bytes: 4096,
    client: skein_forge_client::Limits {
        pending: 4,
        calls: 2,
        rate: 100,
        reserve: 0,
        window: Duration::from_secs(10),
        op_bytes: 512,
        answer_bytes: 4096,
        rows: 8,
        inbox: 16,
        read_attempts: 3,
        backoff: Duration::from_secs(1),
        backoff_max: Duration::from_secs(4),
        entries: 8,
        write_attempts: 3,
        lifetime: Duration::from_secs(10),
        clock_margin: Duration::from_secs(1),
        resources: 8,
        repositories: 2,
        poll: Duration::from_secs(5),
        poll_max: Duration::from_secs(30),
        hinted: Duration::from_secs(1),
        slow: Duration::from_secs(40),
        settle: Duration::from_secs(30),
        facts: 16,
    },
    issue_policy: temper_engine_domain_forge_issues::Limits {
        plan_items: 8,
        milestones: 8,
        title_bytes: 128,
        body_bytes: 512,
        comment_bytes: 256,
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        repairs: 3,
        resolutions: 3,
        updates: 3,
        stall: Duration::from_secs(30),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
};

fn record(bytes: &[u8], bounds: &Bounds) {
    if let Ok(row) = records::decode(bytes, bounds) {
        assert_eq!(records::encode(&row).as_ref(), bytes, "durable row");
        let key = forge::stored_key(&row);
        let encoded = keys::encode(&key, u32::MAX).expect("a decoded row's key fits");
        assert_eq!(keys::decode(&encoded, &LIMITS), Some(key));
        assert_eq!(records::row(&encoded, bytes, bounds), Ok(row));
    }
}
fn key(bytes: &[u8]) {
    if let Some(key) = keys::decode(bytes, &LIMITS) {
        assert_eq!(keys::encode(&key, u32::MAX).expect("decoded key fits").as_ref(), bytes, "durable key");
    }
}
fn golden(bytes: &[u8], bounds: &Bounds) {
    record(bytes, bounds);
    // Value's goldens hold each family; prepend Row's version to reach its decoder.
    let mut versioned = vec![0_u8, 3];
    versioned.extend_from_slice(bytes);
    record(&versioned, bounds);
}
#[test]
fn store_decoders_never_panic_and_reencode_the_same() {
    let bounds = temper_protocol_store::bounds(&LIMITS).expect("fixture limits fit");
    let mut rng = Rng::new(0x7e3d_0405);
    let goldens = temper_codecs_world::goldens("crates/temper-protocol-store/golden/v3");
    for _ in 0..2_000 {
        record(&temper_codecs_world::drawn(&mut rng, 512), &bounds);
        key(&temper_codecs_world::drawn(&mut rng, 512));
        let writer = forge::Stored::Writer(skein_forge_client::Writer {
            forge: u16::try_from(rng.below(65_536)).expect("u16"),
            author: rng.next_u64().max(1),
        });
        let bytes = records::encode(&writer);
        record(&bytes, &bounds);
        record(&temper_codecs_world::mutated(&mut rng, &bytes), &bounds);
        let bytes = keys::encode(&forge::stored_key(&writer), u32::MAX).expect("writer key");
        key(&bytes);
        key(&temper_codecs_world::mutated(&mut rng, &bytes));
    }
    for bytes in &goldens {
        golden(bytes, &bounds);
        for _ in 0..8 {
            golden(&temper_codecs_world::mutated(&mut rng, bytes), &bounds);
        }
    }
}
