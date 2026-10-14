use super::fixtures;
use crate::{records, v1};
use temper_engine_domain_forge as forge;

#[test]
fn each_family_of_names_holds_and_topics_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics() {
        let bytes = records::encode(&row).expect("written by 04.2");
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{name}");
    }
}
#[test]
fn a_truncated_row_fails_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics() {
        let bytes = records::encode(&row).expect("written by 04.2");
        for length in 0..bytes.len() {
            let prefix = bytes.get(..length).expect("prefix within row");
            assert_eq!(records::decode(prefix, &bounds), Err(crate::Undecodable), "{name}, {length}");
        }
    }
}
#[test]
fn a_branch_past_name_bytes_fails_to_decode() {
    let mut segments = skein_lib::List::with_capacity(129);
    for _ in 0_u32..129 {
        segments.push(1_u8).expect("bounded bytes");
    }
    let row = forge::Stored::Hold(forge::Hold {
        name: forge::Name { forge: 1, repository: 2, what: forge::What::Branch(Box::new([segments.into_boxed()])) },
        task: 1,
        writer: None,
        drift: None,
    });
    let bytes = records::encode(&row).expect("schema ceiling admits the branch");
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable));
}
#[test]
fn bounds_take_each_limit_from_its_source() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    let limits = bounds.limits();
    for (name, value, expected) in [
        ("segments", limits.segments_segments, 512),
        ("host", limits.adopted_host, 128),
        ("resources", limits.names_resources, 4),
        ("goal tasks", limits.subscriber_goal_tasks_some, 8),
    ] {
        assert_eq!(value, expected, "{name}");
    }
    assert_eq!(bounds.forge(), &fixtures::LIMITS);
}
#[test]
fn bounds_refuse_sources_past_schema_ceilings() {
    let mut larger = fixtures::LIMITS;
    larger.name_bytes = v1::CEILINGS.adopted_host.checked_add(1).expect("one past ceiling");
    assert_eq!(crate::bounds(&larger), None);
}
