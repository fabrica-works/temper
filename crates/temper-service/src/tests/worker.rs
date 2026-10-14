use super::fixtures;
use crate::worker_settings;

#[test]
fn the_link_s_credential_names_temper_s_payload_version() {
    let settings = fixtures::worker_settings();
    let files = fixtures::worker_files();
    let secret = files.contents(&settings.jig.secret).unwrap();
    let credential = worker_settings::credential(&settings, secret);
    assert_eq!(credential.payload, temper_protocol_worker_wire::PAYLOAD_VERSION);
    assert_eq!(credential.secret.expose(), b"worker-secret-value");
    assert_eq!(credential.name.as_ref(), b"worker-a");
}

#[test]
fn the_worker_sizes_each_part_from_its_payload_codec() {
    let settings = fixtures::worker_settings();
    let agent = fixtures::worker_agent();
    let limits = fixtures::worker_limits();
    let prepared = worker_settings::prepare(&settings, &agent, &limits).unwrap();
    assert_eq!(prepared.root.host.slots, 1);
    assert_eq!(prepared.root.checkout.repositories, 1);
    assert_eq!(prepared.prefix, (3_u64 << 60_u32) + (1_u64 << 48_u32));
    assert_eq!(prepared.link.sizes.items, 420);
    assert!(worker_settings::worker_worst_case(&settings, &agent, &limits).unwrap() > 0);
}
