//! Version, bounds and total outcome round trips (client/README.md, 4.6).
mod fixtures;
use crate::v1;
use alloc::boxed::Box;
use skein_lib::{List, Reader};

#[test]
fn an_adopt_of_another_version_is_refused() {
    let mut bytes = crate::adopt(&fixtures::adopt());
    *bytes.get_mut(1).expect("the record starts with a u16 version") =
        u8::try_from(crate::CLIENT_VERSION + 1).expect("test version fits");
    let problem =
        v1::Adopt::decode(&fixtures::limits(), &mut Reader::new(&bytes)).expect_err("another version is refused");
    assert_eq!(problem, v1::Problem { path: v1::Path::AdoptVersion, reason: skein_codec::Reason::Version });
}

#[test]
fn an_owner_one_byte_past_its_bound_is_refused() {
    let parts = fixtures::adopt().into_parts();
    v1::Adopt::new(&fixtures::limits(), v1::AdoptParts { owner: Box::new([b'a'; 16]), ..parts.clone() })
        .expect("at the bound");
    let made = v1::Adopt::new(&fixtures::limits(), v1::AdoptParts { owner: Box::new([b'a'; 17]), ..parts });
    assert_eq!(
        made.expect_err("past the owner bound"),
        v1::Problem { path: v1::Path::AdoptOwner, reason: skein_codec::Reason::Bound }
    );
}

#[test]
fn checks_one_past_their_count_are_refused() {
    let mut checks = List::with_capacity(3);
    for number in 1..=3 {
        checks.push(number).expect("counted room");
    }
    let parts = v1::AdoptParts { checks, ..fixtures::adopt().into_parts() };
    let problem = v1::Adopt::new(&fixtures::limits(), parts).expect_err("past the checks count");
    assert_eq!(problem, v1::Problem { path: v1::Path::AdoptChecks, reason: skein_codec::Reason::Bound });
}

#[test]
fn each_outcome_round_trips_unchanged() {
    let limits = fixtures::limits();
    for (row, outcome) in fixtures::outcome_rows().into_iter().enumerate() {
        let answer = v1::AdoptAnswer::new(&limits, v1::AdoptAnswerParts { outcome }).expect("an outcome");
        let bytes = crate::answer(&answer);
        let back = v1::AdoptAnswer::decode(&limits, &mut Reader::new(&bytes)).expect("decodes");
        assert_eq!(back, answer, "outcome row {row}");
    }
}

#[test]
fn limits_take_each_bound_from_its_source_field() {
    assert_eq!(
        fixtures::limits(),
        v1::Limits {
            demo_answer_owner: 16,
            demo_answer_name: 16,
            demo_answer_cues: crate::CUES,
            demo_answer_cues_item: crate::CUE_BYTES,
            adopt_owner: 16,
            adopt_name: 16,
            adopt_prefix: 16,
            adopt_landing: 16,
            adopt_checks: 2,
            rule_branch: 16,
            rule_contexts: 2,
            rule_contexts_item: 16,
            adopted_owner: 16,
            adopted_name: 16,
            core_refused_refusal: 64,
        }
    );
    let bounds = crate::Bounds { checks: 65, ..fixtures::BOUNDS };
    assert_eq!(crate::limits(&bounds), None, "past the checks ceiling");
}

#[test]
fn each_owner_round_trips_its_number() {
    for (owner, number) in [(crate::Owner::Forge, 1), (crate::Owner::Pages, 2), (crate::Owner::Runner, 3)] {
        assert_eq!(crate::number(owner), number);
        assert_eq!(crate::owner(number), Some(owner));
    }
}

#[test]
fn an_unknown_owner_number_has_no_owner() {
    assert_eq!(crate::owner(65_535), None);
}

#[test]
fn the_demo_read_and_answer_round_trip_with_nonempty_cues() {
    let limits = fixtures::limits();
    let reading = v1::Reading::new(&limits, v1::ReadingParts { read: v1::Read::Demo }).expect("demo read");
    assert_eq!(crate::decode_reading(&crate::reading(&reading), &limits), Some(reading));
    for (row, cue, accepted) in [("cue", "Change it", true), ("empty cue", "", false)] {
        let mut cues = List::with_capacity(crate::CUES);
        cues.push(cue.as_bytes().into()).expect("one cue");
        let answer = v1::DemoAnswer::new(
            &limits,
            v1::DemoAnswerParts {
                project: 7,
                forge: crate::FORGE,
                provider: 3,
                owner: b"acme".as_slice().into(),
                name: b"shop".as_slice().into(),
                cues,
            },
        )
        .expect("schema answer");
        let back = crate::decode_demo_answer(&crate::demo_answer(&answer), &limits);
        assert_eq!(back.is_some(), accepted, "{row}");
        if accepted {
            assert_eq!(back, Some(answer), "{row}");
        }
    }
}
