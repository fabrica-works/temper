use super::fixtures;
use crate::{records, v2};
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
    larger.name_bytes = v2::CEILINGS.adopted_host.checked_add(1).expect("one past ceiling");
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

fn application(outcome: forge::ApplicationOutcome) -> forge::Stored {
    forge::Stored::Application { token: u64::MAX, until: skein_lib::Wall::from_nanos(u64::MAX), outcome }
}
fn application_refusals() -> Box<[jig_domain_people::Refusal]> {
    use jig_domain_people as p;
    let mut rows = skein_lib::List::with_capacity(192);
    for row in [
        p::Refusal::Role { held: None },
        p::Refusal::Taken { by: u64::MAX },
        p::Refusal::NotWaiting,
        p::Refusal::NoFurther,
        p::Refusal::Unknown,
        p::Refusal::NotOffered,
        p::Refusal::KeyConflict,
    ] {
        rows.push(row).expect("refusal rows fit");
    }
    for held in
        [p::Role::Owner, p::Role::Maintainer, p::Role::Member, p::Role::Observer, p::Role::Policy { role: u32::MAX }]
    {
        rows.push(p::Refusal::Role { held: Some(held) }).expect("bounded application cases");
    }
    for lacking in [
        p::Lack::Approver,
        p::Lack::Budget { needs: u64::MAX, left: 42 },
        p::Lack::Grant { kind: u16::MAX },
        p::Lack::Funding,
        p::Lack::TaskKind { executor: None },
        p::Lack::Depth { most: u32::MAX },
        p::Lack::Deadline,
        p::Lack::Tasks { left: u32::MAX },
        p::Lack::Tool,
        p::Lack::Scope,
        p::Lack::Reference,
        p::Lack::ResourceAccess,
        p::Lack::ResourceUnavailable,
        p::Lack::Shape,
        p::Lack::Again { required: true },
        p::Lack::Account,
        p::Lack::Writer,
        p::Lack::Required { connector: u16::MAX, kind: 9 },
        p::Lack::Failed { connector: 8, kind: 7 },
        p::Lack::Unguarded { connector: 6, kind: 5 },
    ] {
        for proposable in [None, Some(p::Accepter::Nobody), Some(p::Accepter::Role(u32::MAX))] {
            rows.push(p::Refusal::Authority { lacking, proposable }).expect("bounded application cases");
        }
    }
    for which in [
        p::LimitKind::Words,
        p::LimitKind::Identity,
        p::LimitKind::Roles,
        p::LimitKind::Arithmetic,
        p::LimitKind::Duplicate,
        p::LimitKind::Empty,
        p::LimitKind::Batch,
        p::LimitKind::ProjectLiveTasks,
        p::LimitKind::LiveTasks,
        p::LimitKind::Tree,
        p::LimitKind::Depth,
        p::LimitKind::Delegates,
        p::LimitKind::Subscription,
        p::LimitKind::Dependencies,
        p::LimitKind::Cycle,
        p::LimitKind::Spec,
        p::LimitKind::Contract,
        p::LimitKind::Authority,
        p::LimitKind::Inputs,
        p::LimitKind::Attempts,
        p::LimitKind::LiveDelegates,
        p::LimitKind::Restore,
        p::LimitKind::Read,
        p::LimitKind::Turn,
        p::LimitKind::HoldKind,
        p::LimitKind::HoldTaken,
        p::LimitKind::Holds,
        p::LimitKind::InboxFull,
        p::LimitKind::Notes,
        p::LimitKind::Revision,
        p::LimitKind::Question,
        p::LimitKind::Priority,
        p::LimitKind::Policy,
        p::LimitKind::Resource,
        p::LimitKind::Service,
    ] {
        rows.push(p::Refusal::Limit { which }).expect("bounded application cases");
    }
    application_moved_and_ended(&mut rows);
    rows.into_boxed()
}
fn application_moved_and_ended(rows: &mut skein_lib::List<jig_domain_people::Refusal>) {
    use jig_domain_people as p;
    for reason in [
        p::HoldReason::Failures { class: p::FailureClass::Invalid },
        p::HoldReason::Stalled,
        p::HoldReason::Dependency { task: u64::MAX },
        p::HoldReason::Stopped,
        p::HoldReason::StoppedBy { party: u64::MAX },
        p::HoldReason::Drift,
        p::HoldReason::Effects,
        p::HoldReason::EffectFailed,
        p::HoldReason::Uncertain { entry: u64::MAX },
        p::HoldReason::Budget,
        p::HoldReason::Deadline,
        p::HoldReason::HoldsWaited,
        p::HoldReason::ResourceUnavailable,
        p::HoldReason::ResourceAccess,
        p::HoldReason::PoolLost { pool: u64::MAX },
        p::HoldReason::Procedure,
    ] {
        rows.push(p::Refusal::Moved { now: p::Moved::Hold { revision: u64::MAX, reason } })
            .expect("bounded application cases");
    }
    for class in [
        p::FailureClass::Transient,
        p::FailureClass::Permanent,
        p::FailureClass::Run,
        p::FailureClass::Agent,
        p::FailureClass::Lost,
    ] {
        rows.push(p::Refusal::Moved { now: p::Moved::Hold { revision: 4, reason: p::HoldReason::Failures { class } } })
            .expect("bounded application cases");
    }
    for now in [
        p::Moved::Spec { revision: u64::MAX },
        p::Moved::Person { revision: u64::MAX },
        p::Moved::Note { revision: u32::MAX },
    ] {
        rows.push(p::Refusal::Moved { now }).expect("bounded application cases");
    }
    for how in [
        p::Ending::Done,
        p::Ending::Failed,
        p::Ending::Deleted,
        p::Ending::Decided { by: 7, at: skein_lib::Wall::from_nanos(8) },
    ] {
        rows.push(p::Refusal::Ended { how }).expect("bounded application cases");
    }
    for by in [p::Party::Person(u64::MAX), p::Party::Task(u64::MAX), p::Party::Deployment { project: u32::MAX }] {
        rows.push(p::Refusal::Ended { how: p::Ending::Cancelled { by, at: skein_lib::Wall::from_nanos(u64::MAX) } })
            .expect("bounded application cases");
    }
}
fn application_outcomes() -> Box<[forge::ApplicationOutcome]> {
    let mut outcomes = skein_lib::List::with_capacity(224);
    for refusal in application_refusals() {
        outcomes.push(forge::ApplicationOutcome::CoreRefused(refusal)).expect("bounded application cases");
    }
    for reason in [
        forge::AdoptionRefusal::NoSuchForge,
        forge::AdoptionRefusal::NotFound,
        forge::AdoptionRefusal::NoAccess,
        forge::AdoptionRefusal::Refused,
        forge::AdoptionRefusal::InFlight,
        forge::AdoptionRefusal::TooLarge,
        forge::AdoptionRefusal::Unreachable,
        forge::AdoptionRefusal::InvalidAnswer,
    ] {
        outcomes.push(forge::ApplicationOutcome::ForgeRefused(reason)).expect("bounded application cases");
    }
    for protection in [
        forge::Protection::Unknown,
        forge::Protection::Absent,
        forge::Protection::Rule(temper_engine_domain_forge_client::api::Protection {
            branch: Box::from(b"main".as_slice()),
            contexts: Box::new([Box::from(b"check".as_slice())]),
            approvals: 2,
            dismiss_stale: true,
        }),
    ] {
        let forge::Stored::Repository(row) = fixtures::adopted(protection) else { panic!("repository fixture") };
        outcomes
            .push(forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
                repository: row.provider,
                owner: row.owner,
                name: row.name,
                role: row.role,
                home: row.home,
                kinds: row.kinds,
                protection: row.protection,
            }))
            .expect("bounded application cases");
    }
    outcomes.into_boxed()
}
#[test]
fn every_application_outcome_preserves_its_complete_record() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture bounds");
    for outcome in application_outcomes() {
        let row = application(outcome);
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).expect("key");
        let bytes = records::encode(&row);
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row.clone()), "{row:?}");
        assert_eq!(bytes[2], 15, "the Application family tag");
    }
    assert!(
        jig_protocol_store::people_v4::Refusal::worst_case_bytes(&jig_protocol_store::people_v4::CEILINGS)
            .expect("bounded facts")
            <= 24
    );
}
#[test]
fn application_decoding_refuses_malformed_refusal_facts() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture bounds");
    for refusal in [Box::from([255]), Box::from([2]), Box::from([9, 0])] {
        let value = v2::CoreRefused::new(&v2::CEILINGS, v2::CoreRefusedParts { refusal }).expect("small piece");
        let application = v2::Application::new(
            &v2::CEILINGS,
            v2::ApplicationParts { token: 9, until: 10, outcome: v2::ApplicationOutcome::CoreRefused(value) },
        )
        .expect("application");
        let row =
            v2::Row::new(&v2::CEILINGS, v2::RowParts { value: v2::Value::Application(application) }).expect("row");
        assert_eq!(records::decode(&fixtures::encoded_row(&row), &bounds), Err(crate::Undecodable));
    }
}

#[test]
fn an_application_adopted_name_past_its_bound_fails_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS).expect("fixture bounds");
    let forge::Stored::Repository(row) = fixtures::adopted(forge::Protection::Absent) else {
        panic!("repository fixture")
    };
    let outcome = forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
        repository: row.provider,
        owner: Box::new([0; 129]),
        name: row.name,
        role: row.role,
        home: row.home,
        kinds: row.kinds,
        protection: row.protection,
    });
    assert_eq!(records::decode(&records::encode(&application(outcome)), &bounds), Err(crate::Undecodable));
}

#[test]
fn a_job_steps_name_past_its_limit_fails_to_decode() {
    use temper_engine_domain_forge_client::api::{Answer, JobAttempt, Step};
    let answer = Answer::Job {
        attempt: JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 },
        step: Some(Step { number: 1, name: Box::new([b'x'; 4097]) }),
        log: Box::new([]),
        truncated: false,
    };
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS).expect("Fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable));
}
