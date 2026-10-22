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
            pull_address: crate::ADDRESS_BYTES,
            queued_ahead: crate::TASKS_AHEAD,
            change_panel_owner: 16,
            change_panel_name: 16,
            change_panel_branch: 16,
            change_panel_landing: 16,
            change_panel_gates: crate::GATES,
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

#[test]
fn demo_cues_obey_their_count_and_text_bounds() {
    for (row, count, bytes, accepted) in [
        ("no cues", 0, b"x".as_slice(), true),
        ("one byte", 1, b"x".as_slice(), true),
        ("at count", crate::CUES, b"x".as_slice(), true),
        ("past count", crate::CUES + 1, b"x".as_slice(), false),
        ("at bytes", 1, &[b'a'; 256], true),
        ("past bytes", 1, &[b'a'; 257], false),
        ("not UTF-8", 1, &[255], false),
    ] {
        let limits = fixtures::limits();
        let mut cues = List::with_capacity(count);
        for _ in 0..count {
            cues.push(bytes.into()).expect("row's cue capacity");
        }
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
        );
        assert_eq!(answer.is_ok(), accepted, "{row}");
        if let Ok(answer) = answer {
            assert_eq!(crate::decode_demo_answer(&crate::demo_answer(&answer), &limits), Some(answer), "{row}");
        }
    }
}

#[test]
fn a_change_watch_round_trips_and_refuses_trailing_bytes() {
    let limits = fixtures::limits();
    let subject = v1::ChangeSubject::new(&limits, v1::ChangeSubjectParts { task: 17 }).expect("change task");
    let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Change(subject) }).expect("watch");
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
    let trailing = fixtures::trailing(&bytes);
    assert_eq!(crate::decode_watching(&trailing, &limits), None);
}

#[test]
fn each_change_panel_round_trips_unchanged() {
    let limits = fixtures::limits();
    for (row, panel) in fixtures::panel_rows().as_slice().iter().enumerate() {
        assert_eq!(crate::decode_change_panel(&crate::change_panel(panel), &limits).as_ref(), Some(panel), "row {row}");
    }
}

#[test]
fn a_change_panel_of_another_version_is_refused() {
    let limits = fixtures::limits();
    let panel = v1::ChangePanel::new(&limits, fixtures::panel_parts()).expect("panel");
    let mut bytes = crate::change_panel(&panel);
    *bytes.get_mut(1).expect("u16 version") = u8::try_from(crate::CLIENT_VERSION + 1).expect("test version fits");
    assert_eq!(
        v1::ChangePanel::decode(&limits, &mut Reader::new(&bytes)).expect_err("another version"),
        v1::Problem { path: v1::Path::ChangePanelVersion, reason: skein_codec::Reason::Version }
    );
    assert_eq!(crate::decode_change_panel(&bytes, &limits), None);
}

#[test]
fn a_change_panel_refuses_trailing_bytes_and_a_zero_queue_place() {
    let limits = fixtures::limits();
    for (row, place, trailing) in [("first", 1, false), ("zero", 0, false), ("trailing", 1, true)] {
        let parts = v1::ChangePanelParts {
            step: v1::Step::Queued(fixtures::queued(place, 0, &limits)),
            ..fixtures::panel_parts()
        };
        let panel = v1::ChangePanel::new(&limits, parts).expect("schema panel");
        let bytes = crate::change_panel(&panel);
        let bytes = if trailing { fixtures::trailing(&bytes) } else { bytes };
        assert_eq!(
            crate::decode_change_panel(&bytes, &limits),
            if place == 0 || trailing { None } else { Some(panel) },
            "{row}"
        );
    }
}

#[test]
fn change_panel_bounds_accept_at_and_refuse_one_past() {
    let limits = fixtures::limits();
    for (field, path, bound) in [
        (fixtures::PanelBound::Address, v1::Path::PullAddress, crate::ADDRESS_BYTES),
        (fixtures::PanelBound::Ahead, v1::Path::QueuedAhead, crate::TASKS_AHEAD),
        (fixtures::PanelBound::Gates, v1::Path::ChangePanelGates, crate::GATES),
        (fixtures::PanelBound::Owner, v1::Path::ChangePanelOwner, fixtures::BOUNDS.field_bytes),
        (fixtures::PanelBound::Name, v1::Path::ChangePanelName, fixtures::BOUNDS.field_bytes),
        (fixtures::PanelBound::Branch, v1::Path::ChangePanelBranch, fixtures::BOUNDS.field_bytes),
        (fixtures::PanelBound::Landing, v1::Path::ChangePanelLanding, fixtures::BOUNDS.field_bytes),
    ] {
        for count in [bound, bound + 1] {
            let made = fixtures::panel_at_bound(field, count);
            if count == bound {
                let panel = made.expect("at bound");
                assert_eq!(crate::decode_change_panel(&crate::change_panel(&panel), &limits), Some(panel), "{path:?}");
            } else {
                assert_eq!(made.expect_err("past bound"), v1::Problem { path, reason: skein_codec::Reason::Bound });
            }
        }
    }
}
