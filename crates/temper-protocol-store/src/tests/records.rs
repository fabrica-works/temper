use super::fixtures;
use crate::{records, v1};
use temper_engine_domain_forge as forge;

#[test]
fn each_family_of_names_holds_and_topics_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics() {
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{name}");
    }
}
#[test]
fn a_truncated_row_fails_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics() {
        let bytes = records::encode(&row);
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
    let bytes = records::encode(&row);
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

#[test]
fn each_write_of_an_entry_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for (name, write) in fixtures::writes() {
        for condition in fixtures::conditions() {
            for held in [false, true] {
                let row = forge::Stored::Entry { entry: fixtures::entry(write.clone(), condition.clone()), held };
                let bytes = records::encode(&row);
                assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{name}, {condition:?}, {held}");
            }
        }
    }
}
#[test]
fn a_proposed_effect_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    let row = forge::Stored::ProposedEffect(forge::ProposedEffect {
        number: 1,
        resource: forge::What::Branch(Box::new([Box::new([0_u8, 255])])),
        entry: fixtures::entry(
            temper_engine_domain_forge_client::api::Write::Merge { number: 2, head: [3; 32] },
            temper_engine_domain_forge_client::Condition::Merge { base: Box::new([4]) },
        ),
    });
    let bytes = records::encode(&row);
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}
#[test]
fn outbox_decoding_refuses_fields_past_their_bounds() {
    use temper_engine_domain_forge_client::{Condition, api::Write};
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    let rows = [
        ("reviewer count", Write::SetReviewers { number: 1, reviewers: Box::new([1; 9]) }),
        ("operation bytes", Write::Post { number: 1, key: Box::new([]), body: Box::new([1; 513]) }),
    ];
    for (name, write) in rows {
        let row = forge::Stored::Entry { entry: fixtures::entry(write, Condition::None), held: false };
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable), "{name}");
    }
}

#[test]
fn each_named_call_payload_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for payload in &fixtures::payloads() {
        let row = fixtures::named_call(payload.clone());
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
    for answer in fixtures::answers_without_item_pieces() {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}
#[test]
fn error_tags_are_numbered_as_the_schema_lists_them() {
    for (error, tag) in fixtures::errors() {
        let bytes = fixtures::encoded(&records::encode_error(error));
        assert_eq!(bytes[0], tag, "{error:?}");
    }
}
#[test]
fn an_answer_piece_past_its_limit_fails_to_decode() {
    let answer = temper_engine_domain_forge_client::api::Answer::Branches(Box::new([Box::new([0_u8; 4125])]));
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable));
}
#[test]
fn a_full_answer_at_the_fixture_limits_fits_its_pieces() {
    let mut branches = skein_lib::List::with_capacity(8);
    for _ in 0_u32..8 {
        let branch: Box<[u8]> = Box::new([0_u8; 496]);
        branches.push(branch).expect("fixture rows");
    }
    let answer = temper_engine_domain_forge_client::api::Answer::Branches(branches.into_boxed());
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}

#[test]
fn each_item_answer_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for answer in fixtures::item_answers() {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}
#[test]
fn malformed_answer_item_pieces_fail_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    let cases: [(&str, Box<[u8]>); 3] = [
        ("unfinished length", Box::new([0, 0, 0])),
        ("item shorter than length", Box::new([0, 0, 0, 1])),
        ("empty summary", Box::new([0, 0, 0, 0])),
    ];
    for (name, piece) in cases {
        let row = fixtures::item_piece_row(piece);
        assert_eq!(records::decode(&fixtures::encoded_row(&row), &bounds), Err(crate::Undecodable), "{name}");
    }
}
#[test]
fn answer_item_counts_past_rows_fail_to_decode() {
    let mut items = skein_lib::List::with_capacity(9);
    for _ in 0_u32..9 {
        items.push(fixtures::summary()).expect("fixture room");
    }
    let answer = temper_engine_domain_forge_client::api::Answer::Items {
        items: items.into_boxed(),
        more: false,
        now: skein_lib::Time::from_nanos(1),
    };
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable));
}

#[test]
fn each_remaining_answer_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for answer in fixtures::remaining_answers() {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}
#[test]
fn fixed_answer_lists_past_their_counts_fail_to_decode() {
    use temper_engine_domain_forge_client::api::{Answer, Collaborator, Permission};
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    let mut collaborators = skein_lib::List::with_capacity(9);
    for user in 0_u64..9 {
        collaborators.push(Collaborator { user, permission: Permission::Write }).expect("fixture room");
    }
    let cases = [
        ("collaborators", Answer::Collaborators { collaborators: collaborators.into_boxed(), more: true }),
        (
            "commits",
            Answer::Compare {
                before: [1; 32],
                after: [2; 32],
                contains_before: false,
                files: Box::new([]),
                commits: Box::new([[3; 32]; 9]),
            },
        ),
    ];
    for (name, answer) in cases {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable), "{name}");
    }
}

#[test]
fn each_client_row_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for row in &fixtures::client_rows() {
        let bytes = records::encode(row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row.clone()));
    }
}
#[test]
fn client_list_counts_take_their_domain_bounds() {
    use temper_engine_domain_forge_client::{Delivery, Echo, Stored, What};
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    let mut echoes = fixtures::live(What::Issue(1));
    echoes.echoes = Box::new([Echo::Comment(1); 9]);
    let mut comments = fixtures::live(What::Issue(1));
    comments.comments = Box::new([Delivery { id: 1, revision: 1 }; 17]);
    let mut reviews = fixtures::live(What::Issue(1));
    reviews.reviews = Box::new([Delivery { id: 1, revision: 1 }; 17]);
    for (name, value) in [("echoes use rows", echoes), ("comments use inbox", comments), ("reviews use inbox", reviews)]
    {
        let row = forge::Stored::Client(Stored::Live(value));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable), "{name}");
    }
}
#[test]
fn a_cached_summary_key_uses_the_answer_byte_bound() {
    use temper_engine_domain_forge_client::{Stored, What};
    let mut live = fixtures::live(What::Issue(1));
    let mut summary = fixtures::summary();
    summary.key = Some(Box::new([1_u8; 600]));
    live.cached.item = Some(summary);
    let row = forge::Stored::Client(Stored::Live(live));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}

#[test]
fn a_change_in_each_state_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for state in &fixtures::change_states() {
        let row = forge::Stored::Change(fixtures::change_row(state.clone()));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{state:?}");
    }
}
#[test]
fn a_negative_priority_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for priority in [-1_i32, i32::MIN, 0_i32, i32::MAX] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Landed { merge: [1; 32] });
        row.priority = priority;
        let row = forge::Stored::Change(row);
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{priority}");
    }
}
#[test]
fn eight_wrapping_holds_keep_their_order_at_the_schema_ceiling() {
    use temper_engine_domain_forge_change::{Hold, State};
    let mut state = State::Checking { head: [1; 32] };
    for why in [
        Hold::Stalled,
        Hold::Repairs,
        Hold::Resolutions,
        Hold::Updates,
        Hold::BranchMoved,
        Hold::BranchMissing,
        Hold::PullClosed,
        Hold::Retargeted,
    ] {
        state = State::Held { was: Box::new(state), why };
    }
    let row = forge::Stored::Change(fixtures::change_row(state));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}
#[test]
fn each_change_delegate_round_trips_unchanged() {
    use temper_engine_domain_forge_change::{Delegate, Repair, State};
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for delegate in [
        Delegate::Produce,
        Delegate::Repair(Repair::Ci),
        Delegate::Repair(Repair::Gate(1)),
        Delegate::Repair(Repair::Semantic),
        Delegate::Resolve { base: [2; 32] },
        Delegate::Gate { number: 3, head: [4; 32] },
    ] {
        let mut row = fixtures::change_row(State::Checking { head: [5; 32] });
        row.delegate = Some((6, delegate));
        let row = forge::Stored::Change(row);
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{delegate:?}");
    }
}

#[test]
fn an_issue_row_with_its_desired_view_round_trips_unchanged() {
    use temper_engine_domain_forge_issues::{Phase, Projected};
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for phase in
        [Phase::Waiting, Phase::Active, Phase::Settling, Phase::Held, Phase::Done, Phase::Failed, Phase::Cancelled]
    {
        let mut row = fixtures::issue_row();
        row.desired.as_mut().expect("full view").phase = phase;
        let row = forge::Stored::Issue(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "{phase:?}");
    }
    let mut without = fixtures::issue_row();
    without.before = None;
    without.desired = None;
    without.state = Projected::default();
    without.number = None;
    without.pending = None;
    without.failed = false;
    let row = forge::Stored::Issue(without);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "absent fields");
    let mut unfinished = fixtures::issue_row();
    unfinished.desired.as_mut().expect("full view").finished = None;
    let row = forge::Stored::Issue(unfinished);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "no final words");
}
#[test]
fn every_family_encodes_and_decodes_to_itself() {
    use temper_engine_domain_forge_client::{Condition, Stored, What, api::Write};
    let entry = fixtures::entry(Write::Post { number: 1, key: Box::new([2]), body: Box::new([3]) }, Condition::None);
    let remaining = [
        (
            "call",
            fixtures::named_call(forge::CallPayload::Refused(temper_engine_domain_forge_client::api::Error::Busy)),
        ),
        (
            "proposal",
            forge::Stored::ProposedEffect(forge::ProposedEffect {
                number: 4,
                resource: forge::What::Issue(5),
                entry: entry.clone(),
            }),
        ),
        ("entry", forge::Stored::Entry { entry, held: true }),
        ("client", forge::Stored::Client(Stored::Live(fixtures::live(What::Issue(6))))),
        (
            "change",
            forge::Stored::Change(fixtures::change_row(temper_engine_domain_forge_change::State::Landed {
                merge: [7; 32],
            })),
        ),
        ("issue", forge::Stored::Issue(fixtures::issue_row())),
    ];
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics().into_iter().chain(remaining) {
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{name}");
    }
}
#[test]
fn issue_fields_use_their_domain_limits_at_and_past_each_bound() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture limits fit");
    for (name, field, limit) in [
        ("title", 0_u8, 128_usize),
        ("goal", 1, 512),
        ("plan item", 2, 256),
        ("milestone", 3, 256),
        ("finished", 4, 256),
    ] {
        for extra in [0_usize, 1] {
            let mut row = fixtures::issue_row();
            let view = row.desired.as_mut().expect("full view");
            let text = "x".repeat(limit + extra).into_boxed_str();
            match field {
                0 => view.title = text,
                1 => view.goal_text = text,
                2 => view.plan[0].text = text,
                3 => view.milestones[0].text = text,
                4 => view.finished = Some(text),
                _ => panic!("table field"),
            }
            let row = forge::Stored::Issue(row);
            let decoded = records::decode(&records::encode(&row), &bounds);
            if extra == 0 {
                assert_eq!(decoded, Ok(row), "{name} at limit");
            } else {
                assert_eq!(decoded, Err(crate::Undecodable), "{name} past limit");
            }
        }
    }
}
