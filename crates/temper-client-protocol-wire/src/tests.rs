//! Version, bounds and total outcome round trips (client/README.md, 4.6).
mod fixtures;
use crate::v1;
use alloc::boxed::Box;
use skein_lib::{List, Reader};

#[test]
fn an_adopt_of_another_version_is_refused() {
    let mut bytes = crate::requesting(
        &v1::Requesting::new(
            &fixtures::limits(),
            v1::RequestingParts { request: v1::Request::Adopt(fixtures::adopt()) },
        )
        .expect("request"),
    );
    *bytes.get_mut(1).expect("the record starts with a u16 version") =
        u8::try_from(crate::CLIENT_VERSION + 1).expect("test version fits");
    let problem =
        v1::Requesting::decode(&fixtures::limits(), &mut Reader::new(&bytes)).expect_err("another version is refused");
    assert_eq!(problem, v1::Problem { path: v1::Path::RequestingVersion, reason: skein_codec::Reason::Version });
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
        let answer = v1::Answer::new(&limits, v1::AnswerParts { outcome }).expect("an outcome");
        let bytes = crate::answer(&answer);
        let back = v1::Answer::decode(&limits, &mut Reader::new(&bytes)).expect("decodes");
        assert_eq!(back, answer, "outcome row {row}");
    }
}

#[test]
fn limits_take_each_bound_from_its_source_field() {
    assert_eq!(
        fixtures::limits(),
        v1::Limits {
            forges_forges: crate::FORGES,
            set_up_failed_why: crate::SET_UP_WHY_BYTES,
            connected_login: 16,
            stopped_reason: 16,
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

#[test]
fn requesting_round_trips_and_refuses_trailing_and_version_three_adopt_bytes() {
    let limits = fixtures::limits();
    for request in [
        v1::Request::Adopt(fixtures::adopt()),
        v1::Request::SetUp(v1::SetUp::new(&limits, v1::SetUpParts { forge: u16::MAX }).expect("forge number")),
    ] {
        let record = v1::Requesting::new(&limits, v1::RequestingParts { request }).expect("request");
        let bytes = crate::requesting(&record);
        assert_eq!(crate::decode_requesting(&bytes, &limits), Some(record));
        assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &limits), None);
    }
    // Version 3's Adopt was versioned directly, without a request discriminator.
    let adopt = fixtures::adopt();
    let mut writer = skein_lib::Writer::new(usize::try_from(adopt.measure()).expect("length") + 2);
    writer.put(&[0, 3]).expect("old version");
    adopt.encode(&mut writer).expect("old adoption body");
    assert_eq!(crate::decode_requesting(&writer.finish(), &limits), None);
}

#[test]
fn forges_watch_round_trips_and_refuses_trailing_bytes() {
    let limits = fixtures::limits();
    let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Forges }).expect("forges watch");
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
    assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &limits), None);
}

fn forges_rows(numbers: &[u16], connection: &v1::Connection) -> v1::Forges {
    let limits = fixtures::limits();
    let mut forges = List::with_capacity(u32::try_from(numbers.len()).expect("small list"));
    for number in numbers {
        forges
            .push(
                v1::ForgeRow::new(
                    &limits,
                    v1::ForgeRowParts {
                        number: *number,
                        kind: v1::Kind::Forgejo,
                        mode: v1::Mode::Owned,
                        connection: connection.clone(),
                        offer: v1::Offer::Offered,
                    },
                )
                .expect("row"),
            )
            .expect("row capacity");
    }
    v1::Forges::new(&limits, v1::ForgesParts { forges }).expect("forges")
}

#[test]
fn each_forge_offer_round_trips() {
    let limits = fixtures::limits();
    let mut forges = List::with_capacity(3);
    for (number, offer) in [(1, v1::Offer::NotOffered), (2, v1::Offer::NotUnderTheBox), (3, v1::Offer::Offered)] {
        let row = v1::ForgeRow::new(
            &limits,
            v1::ForgeRowParts {
                number,
                kind: v1::Kind::Forgejo,
                mode: v1::Mode::Owned,
                connection: v1::Connection::NotSetUp,
                offer,
            },
        )
        .expect("forge row");
        forges.push(row).expect("three rows");
    }
    let record = v1::Forges::new(&limits, v1::ForgesParts { forges }).expect("forges");
    let bytes = crate::forges(&record);
    assert_eq!(crate::decode_forges(&bytes, &limits), Some(record));
    assert_eq!(crate::decode_forges(&fixtures::trailing(&bytes), &limits), None);
}

#[test]
fn each_forge_connection_round_trips_at_its_bounds() {
    let limits = fixtures::limits();
    let failed =
        v1::SetUpFailed::new(&limits, v1::SetUpFailedParts { why: fixtures::field_bytes(256) }).expect("why at bound");
    let connected = v1::Connected::new(&limits, v1::ConnectedParts { id: u64::MAX, login: fixtures::field_bytes(16) })
        .expect("login at bound");
    let stopped =
        v1::Stopped::new(&limits, v1::StoppedParts { reason: fixtures::field_bytes(16) }).expect("reason at bound");
    for connection in [
        v1::Connection::NotConnected,
        v1::Connection::NotSetUp,
        v1::Connection::SetUpFailed(failed),
        v1::Connection::Connecting,
        v1::Connection::Connected(connected),
        v1::Connection::Stopped(stopped),
    ] {
        let record = forges_rows(&[u16::MAX], &connection);
        let bytes = crate::forges(&record);
        assert_eq!(crate::decode_forges(&bytes, &limits), Some(record));
        assert_eq!(crate::decode_forges(&fixtures::trailing(&bytes), &limits), None);
    }
    for step in [None, Some(v1::SetUpStep::Fetch), Some(v1::SetUpStep::Prepare), Some(v1::SetUpStep::Run)] {
        let connection =
            v1::Connection::SettingUp(v1::SettingUp::new(&limits, v1::SettingUpParts { step }).expect("setup step"));
        let record = forges_rows(&[1], &connection);
        assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), Some(record));
    }
}

#[test]
fn forge_fields_refuse_one_past_their_bounds_on_construction_and_decode() {
    let limits = fixtures::limits();
    let mut writer = skein_lib::Writer::new(4 + 257);
    writer.put(&257_u32.to_be_bytes()).expect("why length");
    writer.put(&[b'w'; 257]).expect("why bytes");
    assert_eq!(
        v1::SetUpFailed::decode(&limits, &mut Reader::new(&writer.finish())).expect_err("why past ceiling"),
        v1::Problem { path: v1::Path::SetUpFailedWhy, reason: skein_codec::Reason::Bound }
    );
    assert_eq!(
        v1::SetUpFailed::new(&limits, v1::SetUpFailedParts { why: fixtures::field_bytes(257) })
            .expect_err("past bound"),
        v1::Problem { path: v1::Path::SetUpFailedWhy, reason: skein_codec::Reason::Bound }
    );
    assert_eq!(
        v1::Connected::new(&limits, v1::ConnectedParts { id: 1, login: fixtures::field_bytes(17) })
            .expect_err("past bound"),
        v1::Problem { path: v1::Path::ConnectedLogin, reason: skein_codec::Reason::Bound }
    );
    assert_eq!(
        v1::Stopped::new(&limits, v1::StoppedParts { reason: fixtures::field_bytes(17) }).expect_err("past bound"),
        v1::Problem { path: v1::Path::StoppedReason, reason: skein_codec::Reason::Bound }
    );
    // A record legal at schema ceilings must still respect the receiving side's bounds.
    for connection in [
        v1::Connection::Connected(
            v1::Connected::new(&v1::CEILINGS, v1::ConnectedParts { id: 1, login: fixtures::field_bytes(17) })
                .expect("schema login"),
        ),
        v1::Connection::Stopped(
            v1::Stopped::new(&v1::CEILINGS, v1::StoppedParts { reason: fixtures::field_bytes(17) })
                .expect("schema reason"),
        ),
    ] {
        let row = v1::ForgeRow::new(
            &v1::CEILINGS,
            v1::ForgeRowParts {
                number: 1,
                kind: v1::Kind::Github,
                mode: v1::Mode::Connected,
                connection,
                offer: v1::Offer::NotOffered,
            },
        )
        .expect("schema row");
        let mut forges = List::with_capacity(1);
        forges.push(row).expect("one row");
        let record = v1::Forges::new(&v1::CEILINGS, v1::ForgesParts { forges }).expect("schema forges");
        assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), None);
    }
}

#[test]
fn forges_refuse_seventeen_rows_unsorted_rows_and_duplicate_numbers() {
    let limits = fixtures::limits();
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let at = forges_rows(&numbers, &v1::Connection::NotConnected);
    assert_eq!(crate::decode_forges(&crate::forges(&at), &limits), Some(at.clone()));
    let mut parts = at.into_parts();
    let mut rows = List::with_capacity(17);
    for row in parts.forges.into_boxed() {
        rows.push(row).expect("room");
    }
    rows.push(
        v1::ForgeRow::new(
            &limits,
            v1::ForgeRowParts {
                number: 17,
                kind: v1::Kind::Github,
                mode: v1::Mode::Fake,
                connection: v1::Connection::NotSetUp,
                offer: v1::Offer::NotOffered,
            },
        )
        .expect("row"),
    )
    .expect("extra room");
    parts.forges = rows;
    assert_eq!(
        v1::Forges::new(&limits, parts).expect_err("past bound"),
        v1::Problem { path: v1::Path::ForgesForges, reason: skein_codec::Reason::Bound }
    );
    // A peer can send seventeen rows even though the constructor refuses them.
    let at = forges_rows(&numbers, &v1::Connection::NotConnected);
    let row = at.forges().get(0).expect("row");
    let length = at.measure().checked_add(row.measure()).expect("seventeen rows fit");
    let mut writer = skein_lib::Writer::new(usize::try_from(length).expect("length"));
    writer.put(&4_u16.to_be_bytes()).expect("version");
    writer.put(&17_u32.to_be_bytes()).expect("row count");
    for row in at.forges() {
        row.encode(&mut writer).expect("row bytes");
    }
    row.encode(&mut writer).expect("extra row bytes");
    assert_eq!(crate::decode_forges(&writer.finish(), &limits), None);
    for numbers in [&[2, 1][..], &[1, 1][..]] {
        let record = forges_rows(numbers, &v1::Connection::NotSetUp);
        assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), None);
    }
}

#[test]
fn forge_login_and_stop_reason_round_trip_at_schema_ceilings() {
    let limits = v1::CEILINGS;
    for count in [1024, 1025] {
        let connected =
            v1::Connected::new(&limits, v1::ConnectedParts { id: u64::MAX, login: fixtures::field_bytes(count) });
        let stopped = v1::Stopped::new(&limits, v1::StoppedParts { reason: fixtures::field_bytes(count) });
        if count == 1025 {
            connected.expect_err("login past schema ceiling");
            stopped.expect_err("reason past schema ceiling");
            continue;
        }
        let connected = connected.expect("login ceiling");
        let stopped = stopped.expect("reason ceiling");
        for kind in [v1::Kind::Forgejo, v1::Kind::Github] {
            for mode in [v1::Mode::Connected, v1::Mode::Managed, v1::Mode::Owned, v1::Mode::Fake] {
                for connection in
                    [v1::Connection::Connected(connected.clone()), v1::Connection::Stopped(stopped.clone())]
                {
                    let row = v1::ForgeRow::new(
                        &limits,
                        v1::ForgeRowParts {
                            number: u16::MAX,
                            kind: kind.clone(),
                            mode: mode.clone(),
                            connection,
                            offer: v1::Offer::NotOffered,
                        },
                    )
                    .expect("forge row");
                    let mut forges = List::with_capacity(1);
                    forges.push(row).expect("one forge");
                    let record = v1::Forges::new(&limits, v1::ForgesParts { forges }).expect("forges");
                    assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), Some(record));
                }
            }
        }
    }
}
