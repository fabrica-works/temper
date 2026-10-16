use crate::*;
mod adoption;
mod fixtures;
use alloc::boxed::Box;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use temper_engine_domain_forge_client as client;

const REPO: client::api::Repository = client::api::Repository { forge: 1, repository: 2 };
const CLIENT: client::Limits = client::Limits {
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
    facts: 16,
};
const LIMITS: Limits = Limits {
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
    client: CLIENT,
};

fn env() -> Env<Limits> {
    Env { now: Time::from_nanos(100_000_000_000), wall: Wall::from_nanos(100_000_000_000), limits: LIMITS }
}
fn domain() -> Domain {
    Domain::new(
        &LIMITS,
        4,
        client::Config {
            namespace: Box::from(&b"test"[..]),
            writers: Box::new([client::Writer { forge: 1, author: 7 }]),
        },
        &[GitIdentity { forge: 1, credential: 1 }],
    )
    .expect("valid connector")
}
fn outputs(d: &mut Domain, event: Event) -> Box<[Request]> {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    step(d, &env(), event, &mut out);
    let mut values = skein_lib::List::with_capacity(out.len());
    for _ in 0..out.len() {
        values.push(out.pop().expect("counted output")).expect("sized from queue");
    }
    values.into_boxed()
}
fn ready(d: &mut Domain) -> Box<[Request]> {
    let mut out = Queue::with_capacity(max_out(&LIMITS));
    resume(d, &env(), &mut out);
    let mut values = skein_lib::List::with_capacity(out.len());
    for _ in 0..out.len() {
        values.push(out.pop().expect("counted output")).expect("sized from queue");
    }
    values.into_boxed()
}
fn adopted() -> Repository {
    Repository {
        project: 5,
        home: true,
        provider: REPO,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: Protection::Absent,
        settings: client::api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}
/// A commit status write: a set, made without a find.
fn status(number: u64, task: u64) -> client::Entry {
    client::Entry {
        number,
        task,
        repository: REPO,
        effect: client::Effect {
            write: client::api::Write::Status {
                commit: [3; 32],
                context: Box::from(&b"test"[..]),
                check: client::api::Check::Passed,
            },
            condition: client::Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
    }
}
fn branch() -> Name {
    Name { forge: 1, repository: 2, what: What::Branch(Box::new([Box::from(&b"temper"[..]), Box::from(&b"7"[..])])) }
}

#[test]
fn a_hold_and_writer_slot_are_exclusive_until_answered() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let name = branch();
    let first = outputs(&mut d, Event::Hold { task: 7, resource: name.clone(), from: None });
    assert_eq!(first.len(), 1);
    assert_eq!(
        first[0],
        Request::Save {
            read_afresh: Some(name.clone()),
            release: None,
            record: Stored::Hold(Hold { name: name.clone(), task: 7, writer: None, drift: None })
        }
    );
    assert_eq!(
        outputs(&mut d, Event::Hold { task: 8, resource: name.clone(), from: None }).as_ref(),
        &[Request::Taken { task: 8, resource: name.clone(), by: 7 }]
    );
    outputs(&mut d, Event::Claim { task: 9, attempt: 1, writes: Box::new([name.clone()]), holders: Box::new([7]) });
    let reported = [Pushed { tag: name.repository, commit: [2; 32] }];
    assert_eq!(d.reported_heads(&LIMITS, 9, 1, &reported).as_ref(), &[(name.clone(), [2; 32])]);
    assert!(d.reported_heads(&LIMITS, 9, 2, &reported).is_empty(), "another attempt cannot report this writer");
    assert!(d.reported_heads(&LIMITS, 8, 1, &reported).is_empty(), "the holder cannot report its child's writer");
    assert!(d.reported_heads(&LIMITS, 9, 1, &[Pushed { tag: 99, commit: [2; 32] }]).is_empty());
    assert_eq!(
        outputs(
            &mut d,
            Event::Claim { task: 10, attempt: 1, writes: Box::new([name.clone()]), holders: Box::new([7]) }
        )
        .as_ref(),
        &[Request::Taken { task: 10, resource: name.clone(), by: 9 }]
    );
    outputs(&mut d, Event::Answered { task: 9, attempt: 1, pushed: Box::new([(name.clone(), [2; 32])]) });
    let claimed = outputs(
        &mut d,
        Event::Claim { task: 10, attempt: 1, writes: Box::new([name.clone()]), holders: Box::new([7]) },
    );
    assert_eq!(
        claimed.as_ref(),
        &[Request::Save {
            read_afresh: None,
            release: None,
            record: Stored::Hold(Hold {
                name,
                task: 7,
                writer: Some(Writer::Run { task: 10, attempt: 1 }),
                drift: None
            })
        }]
    );
}

#[test]
fn an_outbox_entry_waits_for_its_commit() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    assert_eq!(
        outputs(&mut d, Event::Restored { clock: client::RecoveryClock::Monotonic }).as_ref(),
        &[Request::RestartDone { stage: RestartStage::Restored }],
    );
    assert_eq!(
        outputs(&mut d, Event::ReadAfresh).as_ref(),
        &[Request::RestartDone { stage: RestartStage::ReadAfresh }],
    );
    assert_eq!(outputs(&mut d, Event::SettleOutbox).as_ref(), &[Request::RestartDone { stage: RestartStage::Settled }],);
    let entry = status(11, 7);
    assert_eq!(
        outputs(&mut d, Event::Enqueue { entry: entry.clone() }).as_ref(),
        &[Request::Save { read_afresh: None, release: Some(11), record: Stored::Entry { entry, held: false } }]
    );
    assert!(ready(&mut d).is_empty());
    outputs(&mut d, Event::Committed { entry: 11 });
    assert!(d.is_ready());
}

#[test]
#[expect(clippy::too_many_lines, reason = "one adoption test names every connector output explicitly")]
fn adoption_reads_permission_before_committing_its_role() {
    let mut d = domain();
    let request = Adoption {
        project: 5,
        home: true,
        forge: REPO.forge,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        landing: Box::from(&b"main"[..]),
        ci: true,
        checks: Box::new([]),
    };
    fixtures::begin_adoption(&mut d, Token::new(4), request);
    let sent = ready(&mut d);
    let (call, op) = match &sent[0] {
        Request::Call { call, op, .. } => (*call, op.clone()),
        Request::Resource { .. }
        | Request::Adopted { .. }
        | Request::Save { .. }
        | Request::Erase { .. }
        | Request::Taken { .. }
        | Request::Refused { .. }
        | Request::Outcome { .. }
        | Request::NotMade { .. }
        | Request::ContinueRelease { .. }
        | Request::Released { .. }
        | Request::Retained { .. }
        | Request::ReleaseFailed { .. }
        | Request::News { .. }
        | Request::Drift { .. }
        | Request::ProjectionEffect { .. }
        | Request::WakeProcedure { .. }
        | Request::ProjectAfter { .. }
        | Request::ProjectionFailed { .. }
        | Request::ProjectionSettled { .. }
        | Request::GoalTopic { .. }
        | Request::GoalUntopic { .. }
        | Request::ChangeDecision { .. }
        | Request::Read { .. }
        | Request::BriefClient { .. }
        | Request::BriefReady { .. }
        | Request::BriefSized { .. }
        | Request::EffectsSettled { .. }
        | Request::BriefTaken { .. }
        | Request::ApplicationPart { .. }
        | Request::ApplicationDone { .. }
        | Request::ApplicationAgain { .. }
        | Request::RestartDone { .. } => panic!("permission call"),
    };
    assert_eq!(op, client::api::Op::Read(client::api::Read::Permission { user: 7 }));
    let result = outputs(
        &mut d,
        Event::Client(client::Event::Answered {
            call,
            cost: 1,
            result: Ok(client::api::Answer::Permission(client::api::Permission::Write)),
        }),
    );
    assert!(result.is_empty());
    let next = ready(&mut d);
    assert_eq!(next.len(), 1);
    let branches = match &next[0] {
        Request::Call { call, op: client::api::Op::Read(client::api::Read::Branches), .. } => *call,
        Request::Call { .. }
        | Request::Resource { .. }
        | Request::Adopted { .. }
        | Request::Save { .. }
        | Request::Erase { .. }
        | Request::Taken { .. }
        | Request::Refused { .. }
        | Request::Outcome { .. }
        | Request::NotMade { .. }
        | Request::ContinueRelease { .. }
        | Request::Released { .. }
        | Request::Retained { .. }
        | Request::ReleaseFailed { .. }
        | Request::News { .. }
        | Request::Drift { .. }
        | Request::ProjectionEffect { .. }
        | Request::WakeProcedure { .. }
        | Request::ProjectAfter { .. }
        | Request::ProjectionFailed { .. }
        | Request::ProjectionSettled { .. }
        | Request::GoalTopic { .. }
        | Request::GoalUntopic { .. }
        | Request::ChangeDecision { .. }
        | Request::Read { .. }
        | Request::BriefClient { .. }
        | Request::BriefReady { .. }
        | Request::BriefSized { .. }
        | Request::EffectsSettled { .. }
        | Request::BriefTaken { .. }
        | Request::ApplicationPart { .. }
        | Request::ApplicationDone { .. }
        | Request::ApplicationAgain { .. }
        | Request::RestartDone { .. } => panic!("branches call"),
    };
    assert!(
        outputs(
            &mut d,
            Event::Client(client::Event::Answered {
                call: branches,
                cost: 1,
                result: Ok(client::api::Answer::Branches(Box::new([Box::from(&b"main"[..])]))),
            })
        )
        .is_empty()
    );
    let next = ready(&mut d);
    match &next[0] {
        Request::Call { op: client::api::Op::Read(client::api::Read::Settings), .. } => {}
        Request::Call { .. }
        | Request::Resource { .. }
        | Request::Adopted { .. }
        | Request::Save { .. }
        | Request::Erase { .. }
        | Request::Taken { .. }
        | Request::Refused { .. }
        | Request::Outcome { .. }
        | Request::NotMade { .. }
        | Request::ContinueRelease { .. }
        | Request::Released { .. }
        | Request::Retained { .. }
        | Request::ReleaseFailed { .. }
        | Request::News { .. }
        | Request::Drift { .. }
        | Request::ProjectionEffect { .. }
        | Request::WakeProcedure { .. }
        | Request::ProjectAfter { .. }
        | Request::ProjectionFailed { .. }
        | Request::ProjectionSettled { .. }
        | Request::GoalTopic { .. }
        | Request::GoalUntopic { .. }
        | Request::ChangeDecision { .. }
        | Request::Read { .. }
        | Request::BriefClient { .. }
        | Request::BriefReady { .. }
        | Request::BriefSized { .. }
        | Request::EffectsSettled { .. }
        | Request::BriefTaken { .. }
        | Request::ApplicationPart { .. }
        | Request::ApplicationDone { .. }
        | Request::ApplicationAgain { .. }
        | Request::RestartDone { .. } => panic!("settings follows collision read"),
    }
}

#[test]
fn adoption_without_ci_refuses_an_empty_check_policy() {
    let mut d = domain();
    let request = Adoption {
        project: 5,
        home: true,
        forge: REPO.forge,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        landing: Box::from(&b"main"[..]),
        ci: false,
        checks: Box::new([]),
    };
    assert_eq!(
        fixtures::begin_adoption(&mut d, Token::new(5), request).as_ref(),
        &[Request::Adopted { reply_to: Token::new(5), result: Err(client::api::Error::Refused) }]
    );
    assert!(ready(&mut d).is_empty());
}

#[test]
fn restored_ci_subscriptions_are_read_in_the_fresh_step_and_delay_its_completion() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Subscription(Subscriber {
                task: 7,
                number: 1,
                topic: Topic::Ci { repository: REPO, head: [4; 32] },
                own_change: None,
                goal_tasks: None,
                paths: Box::new([]),
            }),
        },
    );
    assert_eq!(
        outputs(&mut d, Event::Restored { clock: client::RecoveryClock::Monotonic }).as_ref(),
        &[Request::RestartDone { stage: RestartStage::Restored }]
    );
    let fresh = outputs(&mut d, Event::ReadAfresh);
    assert!(fresh.is_empty(), "the subscription read is scheduled before completion");
    let calls = ready(&mut d);
    let Request::Call { call, op: client::api::Op::Read(client::api::Read::Statuses { commit, .. }), .. } = &calls[0]
    else {
        panic!("CI statuses requested in fresh step")
    };
    assert_eq!(*commit, [4; 32]);
    let done = outputs(
        &mut d,
        Event::Client(client::Event::Answered {
            call: *call,
            cost: 1,
            result: Ok(client::api::Answer::Statuses {
                ci: client::api::Ci::Passed,
                statuses: Box::new([]),
                more: false,
            }),
        }),
    );
    assert!(done.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
}

#[test]
fn workspace_items_are_sized_context_is_read_only_and_saved_work_starts_the_next_run() {
    let mut d = domain();
    let mut repository = adopted();
    repository.role = Role::Context;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    let item = d
        .workspace_item(items::Seed {
            task: 7,
            parent: None,
            repository: REPO,
            run_branch: Box::from(&b"temper/7/r7"[..]),
            saved_branch: Some(Box::from(&b"temper/7/s7"[..])),
        })
        .expect("item");
    assert_eq!(item.start, items::Start::Saved(Box::from(&b"temper/7/s7"[..])));
    assert!(item.push.is_none());
    assert!(item.bytes > 0);
    assert_eq!(d.context_repositories(5, 1).expect("bounded context").len(), 1);
    assert!(d.context_repositories(5, 0).is_none());
}

// Each repository's git identity is the configured credential of its forge
// (credentials.md; brief D10).
#[test]
fn a_workspace_item_names_its_forges_git_identity() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let item = d
        .workspace_item(items::Seed {
            task: 7,
            parent: None,
            repository: REPO,
            run_branch: Box::from(&b"temper/7/r7"[..]),
            saved_branch: None,
        })
        .expect("item");
    assert_eq!(item.credential, Some(1));
}

#[test]
fn named_resource_reports_distinguish_private_branches_context_and_shared_landings() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: client::RecoveryClock::Monotonic });
    let private = branch();
    assert_eq!(d.resource_facts(&private), Some((Access::Owned, resources::HoldKind::Exclusive { wait: true })));
    let shared = Name { what: What::Branch(Box::new([Box::from(&b"main"[..])])), ..private.clone() };
    assert_eq!(d.resource_facts(&shared), Some((Access::Participant, resources::HoldKind::Shared)));
    let report = outputs(&mut d, Event::Names { task: 7, resources: Box::new([private.clone()]) });
    assert!(report.contains(&Request::Resource {
        project: 5,
        name: private.clone(),
        role: Access::Owned,
        hold: resources::HoldKind::Exclusive { wait: true }
    }));
    let mut context = adopted();
    context.role = Role::Context;
    outputs(&mut d, Event::Restore { record: Stored::Repository(context) });
    assert_eq!(d.resource_facts(&private), Some((Access::Context, resources::HoldKind::Shared)));
    assert_eq!(
        outputs(&mut d, Event::Hold { task: 7, resource: private, from: None }).as_ref(),
        &[Request::Refused { task: 7 }]
    );
}

#[test]
fn native_effect_rows_keep_the_clients_recovery_promises() {
    use client::api::Write;
    let empty = || Box::<[u8]>::default();
    let writes = [
        (3, EffectForm::Creation, Write::OpenPull { title: empty(), body: empty(), head: empty(), base: empty() }),
        (10, EffectForm::Set, Write::Edit { number: 1, title: None, body: None }),
        (11, EffectForm::Transition, Write::Update { number: 1 }),
        (4, EffectForm::Transition, Write::Merge { number: 1, head: [1; 32] }),
        (12, EffectForm::Set, Write::Close { number: 1 }),
        (12, EffectForm::Set, Write::Reopen { number: 1 }),
        (
            5,
            EffectForm::Creation,
            Write::Review { number: 1, key: empty(), verdict: client::api::Verdict::Approve, body: empty() },
        ),
        (7, EffectForm::Creation, Write::Post { number: 1, key: empty(), body: empty() }),
        (6, EffectForm::Set, Write::Status { commit: [1; 32], context: empty(), check: client::api::Check::Passed }),
        (8, EffectForm::Creation, Write::CreateIssue { key: empty(), title: empty(), body: empty() }),
        (13, EffectForm::Set, Write::Edit { number: 1, title: None, body: None }),
        (13, EffectForm::Set, Write::Close { number: 1 }),
        (13, EffectForm::Set, Write::Reopen { number: 1 }),
        (14, EffectForm::Set, Write::SetReviewers { number: 1, reviewers: Box::new([]) }),
        (9, EffectForm::Creation, Write::CreateBranch { branch: empty(), commit: [1; 32] }),
        (15, EffectForm::Set, Write::DeleteBranch { branch: empty() }),
    ];
    for (kind, expected_form, write) in writes {
        let (form, recovery) = effect_shape(kind).expect("native effect kind");
        assert_eq!(form, expected_form, "kind {kind}");
        let expected = match client::recovery(&write) {
            client::Recovery::Keyed => Recovery::Keyed,
            client::Recovery::Conditional => Recovery::Conditional,
            client::Recovery::Idempotent => Recovery::Idempotent,
            client::Recovery::Unrecoverable => Recovery::Unrecoverable,
        };
        assert_eq!(recovery, expected, "kind {kind}");
    }
}

/// An issue creation whose lifetime ran out unanswered: unrecoverable, it
/// is held for a person (domain/forge.md, 6).
fn held_issue() -> client::Entry {
    client::Entry {
        number: 11,
        task: 7,
        repository: REPO,
        effect: client::Effect {
            write: client::api::Write::CreateIssue {
                key: Box::from(&b"k"[..]),
                title: Box::from(&b"Finding"[..]),
                body: Box::from(&b"Fix"[..]),
            },
            condition: client::Condition::None,
        },
        start: Some(client::Position { at: Time::from_nanos(1), comment: 0, review: 0, review_page: 1 }),
        attempt: Some(client::Attempt {
            sent: Time::from_nanos(1),
            deadline: Time::from_nanos(2),
            wall: Wall::from_nanos(1),
            expires: Wall::from_nanos(2),
        }),
        failures: 1,
    }
}

/// A forge restored with the held issue creation, before its restart runs.
fn held_restored() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry: held_issue(), held: true } });
    d
}

/// A restarted forge whose held issue creation waits for a person.
fn held_creation() -> Domain {
    let mut d = held_restored();
    outputs(&mut d, Event::Restored { clock: client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

#[test]
fn a_held_effect_survives_a_restart_out_of_the_clients_outbox() {
    let mut d = held_restored();
    assert_eq!(
        outputs(&mut d, Event::Restored { clock: client::RecoveryClock::Monotonic }).as_ref(),
        &[Request::RestartDone { stage: RestartStage::Restored }]
    );
    assert_eq!(
        outputs(&mut d, Event::ReadAfresh).as_ref(),
        &[Request::RestartDone { stage: RestartStage::ReadAfresh }]
    );
    // Nothing is looked for: the client never had the held entry.
    assert_eq!(outputs(&mut d, Event::SettleOutbox).as_ref(), &[Request::RestartDone { stage: RestartStage::Settled }]);
    assert!(ready(&mut d).is_empty());
    assert!(!d.effect_room(&LIMITS, 7), "the held entry keeps its room");
}

#[test]
fn a_held_effect_settled_as_not_made_is_erased_and_reported() {
    let mut d = held_creation();
    assert_eq!(
        outputs(&mut d, Event::Resolve { entry: 11, again: false }).as_ref(),
        &[Request::Erase { key: Key::Entry(11) }, Request::NotMade { entry: 11, task: 7 }]
    );
}

#[test]
fn a_held_effect_asked_again_is_looked_for_first_even_with_an_otherwise_full_outbox() {
    let mut d = held_creation();
    for number in 12..19 {
        outputs(&mut d, Event::Enqueue { entry: status(number, 8) });
        outputs(&mut d, Event::Committed { entry: number });
    }
    assert!(!d.effect_room(&LIMITS, 0), "the outbox is full");
    assert_eq!(
        outputs(&mut d, Event::Resolve { entry: 11, again: true }).as_ref(),
        &[Request::Save {
            read_afresh: None,
            release: None,
            record: Stored::Entry { entry: held_issue(), held: false }
        }]
    );
    assert_eq!(
        ready(&mut d).as_ref(),
        &[Request::Call {
            call: Token::new(1),
            repository: REPO,
            op: client::api::Op::Read(client::api::Read::Items {
                since: Time::from_nanos(1),
                page: 1,
                kind: Some(client::api::Kind::Issue),
            }),
        }]
    );
}

#[test]
fn closing_a_task_settles_its_held_effect_not_made_and_frees_its_room() {
    let mut d = held_creation();
    for number in 12..19 {
        outputs(&mut d, Event::Enqueue { entry: status(number, 8) });
    }
    let row = ReleaseRow {
        task: 7,
        root: 7,
        ending: ReleaseEnding::Done,
        close_pull: None,
        pending: None,
        failed: false,
        effects_settled: false,
        releasing: false,
    };
    assert_eq!(
        outputs(&mut d, Event::SettleEffects { task: 7, root: 7, ending: ReleaseEnding::Done }).as_ref(),
        &[
            Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) },
            Request::Erase { key: Key::Entry(11) },
            Request::NotMade { entry: 11, task: 7 },
            Request::ContinueRelease { task: 7 },
        ]
    );
    assert_eq!(
        outputs(&mut d, Event::Enqueue { entry: status(19, 8) }).as_ref(),
        &[Request::Save {
            read_afresh: None,
            release: Some(19),
            record: Stored::Entry { entry: status(19, 8), held: false }
        }]
    );
    let settled = ReleaseRow { effects_settled: true, ..row };
    assert_eq!(
        outputs(&mut d, Event::ContinueRelease { task: 7, entry: 20 }).as_ref(),
        &[
            Request::Save { read_afresh: None, release: None, record: Stored::Release(settled) },
            Request::EffectsSettled { task: 7 },
        ]
    );
}

fn run_branch(task: &[u8]) -> Name {
    Name { forge: 1, repository: 2, what: What::Branch(Box::new([Box::from(&b"temper"[..]), Box::from(task)])) }
}

/// A ready forge whose run 9 writes four branches the forge last saw at
/// commit 2, each held by its own task: their reads fill the client's table.
fn four_writers() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for (holder, task) in [(7, b"7".as_slice()), (8, b"8"), (10, b"10"), (11, b"11")] {
        let name = run_branch(task);
        let head = BranchHead { name: name.clone(), commit: [2; 32], owned: true };
        outputs(&mut d, Event::Restore { record: Stored::BranchHead(head) });
        outputs(&mut d, Event::Hold { task: holder, resource: name.clone(), from: None });
        outputs(&mut d, Event::Claim { task: 9, attempt: 1, writes: Box::new([name]), holders: Box::new([holder]) });
    }
    outputs(&mut d, Event::Restored { clock: client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

/// The forge's answer to client call `call`: the branch is at `commit`.
fn branch_at(d: &mut Domain, call: u64, commit: u8) -> Box<[Request]> {
    let result = Ok(client::api::Answer::Commit([commit; 32]));
    let answered = outputs(d, Event::Client(client::Event::Answered { call: Token::new(call), cost: 1, result }));
    d.reclaim();
    answered
}

// A lost writer's slot is freed only once its branch is read afresh
// (connectors.md, 3.3): a read refused for want of room in the client waits,
// and is asked again once room frees, rather than given up.
#[test]
fn a_lost_writers_read_refused_for_room_is_asked_again_when_room_frees() {
    let mut d = four_writers();
    outputs(&mut d, Event::Lost { task: 9, attempt: 1 });
    ready(&mut d);
    ready(&mut d);
    // The first branch moved; its comparison finds the client's table full.
    assert!(branch_at(&mut d, 1, 4).is_empty(), "nothing refused");
    ready(&mut d);
    ready(&mut d);
    branch_at(&mut d, 2, 2);
    ready(&mut d);
    branch_at(&mut d, 3, 2);
    assert_eq!(
        ready(&mut d).as_ref(),
        &[Request::Call {
            call: Token::new(5),
            repository: REPO,
            op: client::api::Op::Read(client::api::Read::Compare { before: [2; 32], after: [4; 32] }),
        }]
    );
}

#[test]
fn a_ci_sections_words_keep_their_end_and_others_their_start() {
    for (tail, gathered, cut) in [(true, b"cdefghij", b"ghij"), (false, b"abcdefgh", b"abcd")] {
        let mut words = brief::BriefWords::with_capacity(8, tail);
        brief::append(&mut words, b"abcdefghij");
        assert_eq!(
            brief::brief_part(&words, 1, 8),
            BriefRead::Got { bytes: Box::from(&gathered[..]), left: 2 },
            "Tail: {tail}",
        );
        assert_eq!(
            brief::brief_part(&words, 1, 4),
            BriefRead::Got { bytes: Box::from(&cut[..]), left: 6 },
            "Tail: {tail}",
        );
    }
}
