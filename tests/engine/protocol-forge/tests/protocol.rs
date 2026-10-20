use skein_fake_forge_protocol::Route;
use skein_forge_machine_world::joint::REPO;
use skein_lib::Wall;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_issues as issues;
use temper_engine_protocol_forge_world::{fixtures, protocol::Protocol};
fn change_row() -> top::ChangeRow {
    top::ChangeRow {
        task: 51,
        repository: REPO,
        branch: Box::from(&b"temper/51"[..]),
        base: Box::from(&b"main"[..]),
        title: Box::from(&b"Small fix"[..]),
        body: Box::from(&b"Fix the file"[..]),
        priority: 1,
        change: change::Change {
            task: 51,
            state: change::State::Producing { requested: false },
            gates: Box::new([]),
            clean: Box::new([]),
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: Wall::from_nanos(100_000_000_000),
            ready_since: None,
            owns_turn: false,
            last_head: None,
        },
        pull: None,
        pending: None,
        effect: change::EffectResult::None,
        delegate: None,
        delegate_status: change::Status::Unknown,
        verdicts: Box::new([]),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
    }
}

fn change_step(
    world: &mut Protocol,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
) -> change::Decision {
    change_step_with_release(world, entry, heard, gates, false)
}

fn change_step_with_release(
    world: &mut Protocol,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
    released: bool,
) -> change::Decision {
    change_step_task(world, 51, entry, heard, gates, released)
}

fn change_step_task(
    world: &mut Protocol,
    task: u64,
    entry: u64,
    heard: change::Status,
    gates: Box<[change::GateReport]>,
    released: bool,
) -> change::Decision {
    world.take_seen();
    world.event(top::Event::StepChange {
        task,
        entry,
        heard: change::Heard { delegate: heard, effect: change::EffectResult::None, released, cancelled: false },
        gates,
        queue_repair_active: false,
    });
    world.run_for(1);
    let mut decision = None;
    let mut committed = None;
    for event in world.take_seen() {
        if let top::Request::ChangeDecision { task: decided, decision: current, entry, .. } = event
            && decided == task
        {
            decision = Some(current);
            committed = entry;
        }
    }
    if let Some(entry) = committed {
        world.event(top::Event::Committed { entry });
        world.run_for(1);
    }
    decision.expect("change step made a decision")
}
#[test]
fn a_change_produced_opened_checked_queued_and_landed() {
    let mut world = opened_change();
    let finished = land_change(&mut world);
    assert!(finished, "change should land");
    assert_eq!(world.writes(), 2);
}

#[test]
fn a_created_issue_with_its_answer_lost_is_found_after_restart() {
    let mut world = Protocol::new(23);
    world.adopt();
    world.slow_calls(skein_lib::Duration::from_secs(1));
    let view = issues::GoalView {
        phase: issues::Phase::Waiting,
        goal: 77,
        repository: u64::from(REPO.repository),
        title: Box::from("Goal"),
        goal_text: Box::from("Plan"),
        plan: Box::new([]),
        milestones: Box::new([]),
        finished: None,
    };
    world.event(top::Event::Project { entry: 1, repository: REPO, view });
    world.event(top::Event::Committed { entry: 1 });
    world.run_for(1);
    assert_eq!(world.writes(), 1);
    world.restart();
    world.run_for(20);
    assert_eq!(world.writes(), 1);
    assert!(world.stored().get(&top::Key::Entry(1)).is_none());
    match world.stored().get(&top::Key::Issue(77)) {
        Some(top::Stored::Issue(row)) => assert!(row.number.is_some()),
        Some(_) | None => panic!("recovered issue"),
    }
}

#[test]
fn a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice() {
    let mut world = Protocol::new(89);
    world.adopt();
    world.produce(b"temper/51");
    let pull = world.open_pull(b"temper/51", b"main");
    let head = temper_engine_forge_world::translate::commit(world.branch(b"temper/51"));
    world.slow_calls(skein_lib::Duration::from_secs(3));
    enqueue_merge(&mut world, pull, head);
    world.event(top::Event::Committed { entry: 1 });
    world.run_for(11);
    assert_eq!(world.writes(), 1);
    assert!(world.stored().get(&top::Key::Entry(1)).is_some());
    let landed = world.branch(b"main");
    world.restart();
    world.run_for(30);
    assert_eq!(world.writes(), 1);
    assert_eq!(world.branch(b"main"), landed);
    assert!(world.stored().get(&top::Key::Entry(1)).is_none());
}

#[test]
fn a_failed_ci_head_is_repaired_then_checked_again() {
    let mut world = Protocol::new(13);
    world.adopt();
    world.event(top::Event::Change { row: change_row() });
    assert_eq!(
        change_step(&mut world, 1, change::Status::Unknown, Box::new([])),
        change::Decision::Delegate(change::Delegate::Produce)
    );
    world.produce(b"temper/51");
    assert_eq!(
        change_step(&mut world, 2, change::Status::Passed, Box::new([])),
        change::Decision::Effect(change::Effect::Open)
    );
    world.status(b"temper/51", skein_fake_forge_domain::api::Check::Failed);
    assert_eq!(
        change_step(&mut world, 3, change::Status::Passed, Box::new([])),
        change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci))
    );
    let before = world.writes();
    world.push(b"temper/51", b"repaired");
    world.status(b"temper/51", skein_fake_forge_domain::api::Check::Passed);
    let mut finished = false;
    for entry in 4..20 {
        if let change::Decision::Finish { .. } = change_step(&mut world, entry, change::Status::Passed, Box::new([])) {
            finished = true;
            break;
        }
    }
    assert!(finished);
    assert_eq!(world.writes(), before + 1); // only the merge is a connector write
}

#[test]
fn an_external_move_of_a_held_branch_is_reported_as_drift() {
    let mut world = Protocol::new(31);
    world.adopt();
    let (name, pushed) = held_branch(&mut world);
    world.push(b"temper/51", b"outside");
    world.event(top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: REPO,
            change: skein_forge::api::Change::Branch(Box::from(&b"temper/51"[..])),
            key: None,
        },
    });
    world.run_for(5);
    assert!(world.seen().iter().any(|event| matches!(event,
        top::Request::Drift { task: 51, resource } if resource == &name)));
    let Some(top::Stored::Hold(hold)) = world.stored().get(&top::Key::Hold(name.clone())) else {
        panic!("affected hold");
    };
    assert!(
        matches!(hold.drift.as_ref(), Some(top::Drift { at, change: top::DriftChange::Branch { expected, observed: Some(_) } }) if *expected == pushed && *at > Wall::EPOCH)
    );
    world.take_seen();
    step_change(&mut world, 10, change::Status::Passed, false);
    world.run_for(1);
    assert!(world.seen().iter().any(|event| matches!(
        event,
        top::Request::ChangeDecision { task: 51, decision: change::Decision::Hold(change::Hold::BranchMoved), .. }
    )));
    world.event(top::Event::ReleaseChange { task: 51 });
    world.take_seen();
    step_change(&mut world, 11, change::Status::Passed, true);
    world.run_for(1);
    assert!(!world.seen().iter().any(|event| matches!(
        event,
        top::Request::ChangeDecision { task: 51, decision: change::Decision::Hold(change::Hold::BranchMoved), .. }
    )));
}

#[test]
fn idle_forge_call_cost_does_not_follow_old_issue_history() {
    let mut small = Protocol::new(47);
    small.preload_history(2);
    small.adopt();
    small.event(top::Event::Names { task: 1, resources: Box::new([main_name()]) });
    small.run_for(60);
    let before = small.tally().total;
    small.run_for(60);
    let small_cost = small.tally().total - before;

    let mut large = Protocol::new(47);
    large.preload_history(20);
    large.adopt();
    large.event(top::Event::Names { task: 1, resources: Box::new([main_name()]) });
    large.run_for(60);
    let before = large.tally().total;
    large.run_for(60);
    assert_eq!(large.tally().total - before, small_cost);
}

#[test]
fn honest_costs_equal_the_requests_the_fake_served() {
    let mut world = Protocol::new(11);
    world.adopt();
    world.push(b"work", b"one");
    world.open_pull(b"work", b"main");
    world.run_for(60);
    let tally = world.tally();
    let starts = tally.routes.get(&Route::User).copied().unwrap_or(0)
        + tally.routes.get(&Route::ApiSettings).copied().unwrap_or(0);
    assert!(world.costs() > 0, "the story made calls");
    assert_eq!(world.costs(), tally.total - starts);
}

#[test]
fn a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one() {
    let mut world = Protocol::new(12);
    world.adopt();
    world.event(top::Event::Names { task: 1, resources: Box::new([main_name()]) });
    world.run_for(61);
    for _ in 0..6 {
        world.issue(b"Changed", b"body");
    }
    let before = world.tally().routes.get(&Route::Issues).copied().unwrap_or(0);
    let first = world.listing_passes().len();
    world.event(top::Event::Hint {
        hint: skein_forge::api::Hint { repository: REPO, change: skein_forge::api::Change::Item(1), key: None },
    });
    world.run_for(2);
    let pages = world.tally().routes.get(&Route::Issues).copied().unwrap_or(0) - before;
    assert!(pages > 0, "the pass listed the changed items");
    let passes = &world.listing_passes()[first..];
    assert_eq!(passes.iter().sum::<u64>(), pages, "every HTTP listing is in a pass");
    for (pass, pages) in passes.iter().enumerate() {
        assert!(*pages <= 7, "pass {pass}: six changed items, page size one, plus one: {pages}");
    }
}

#[test]
fn the_api_token_reaches_no_record_request_or_fact() {
    let mut world = Protocol::new(11);
    world.adopt();
    world.event(top::Event::Project { entry: 1, repository: REPO, view: fixtures::goal_view() });
    world.event(top::Event::Committed { entry: 1 });
    world.run_for(60);
    assert_eq!(world.issues_made(), 1);
    assert!(!fixtures::token_absent(&fixtures::TOKEN), "the check recognizes byte secrets");
    assert!(
        !fixtures::token_absent(&core::str::from_utf8(fixtures::TOKEN).expect("ASCII token")),
        "the check recognizes text secrets"
    );
    assert!(world.token_kept());
}

fn main_name() -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([Box::from(b"main".as_slice())])),
    }
}

fn step_change(world: &mut Protocol, entry: u64, delegate: change::Status, released: bool) {
    world.event(top::Event::StepChange {
        task: 51,
        entry,
        heard: change::Heard { delegate, effect: change::EffectResult::None, released, cancelled: false },
        gates: Box::new([]),
        queue_repair_active: false,
    });
}

fn land_change(world: &mut Protocol) -> bool {
    let mut finished = false;
    for entry in 4..24 {
        world.take_seen();
        step_change(world, entry, change::Status::Passed, false);
        world.run_for(1);
        let replies = world.take_seen();
        for reply in replies {
            match reply {
                top::Request::ChangeDecision { decision: change::Decision::Effect(_), entry: Some(number), .. } => {
                    world.event(top::Event::Committed { entry: number });
                    world.run_for(1);
                }
                top::Request::ChangeDecision { decision: change::Decision::Finish { .. }, .. } => finished = true,
                top::Request::ChangeDecision { decision: change::Decision::Delegate(_), .. } => {
                    panic!("unexpected delegate")
                }
                top::Request::ChangeDecision { .. }
                | top::Request::Resource { .. }
                | top::Request::Adopted { .. }
                | top::Request::Save { .. }
                | top::Request::Erase { .. }
                | top::Request::Taken { .. }
                | top::Request::Refused { .. }
                | top::Request::Outcome { .. }
                | top::Request::NotMade { .. }
                | top::Request::ContinueRelease { .. }
                | top::Request::EffectsSettled { .. }
                | top::Request::Released { .. }
                | top::Request::Retained { .. }
                | top::Request::ReleaseFailed { .. }
                | top::Request::ProjectionEffect { .. }
                | top::Request::WakeProcedure { .. }
                | top::Request::ProjectAfter { .. }
                | top::Request::ProjectionSettled { .. }
                | top::Request::GoalTopic { .. }
                | top::Request::GoalUntopic { .. }
                | top::Request::ProjectionFailed { .. }
                | top::Request::News { .. }
                | top::Request::Drift { .. }
                | top::Request::Read { .. }
                | top::Request::BriefClient { .. }
                | top::Request::BriefReady { .. }
                | top::Request::BriefSized { .. }
                | top::Request::BriefTaken { .. }
                | top::Request::ApplicationPart { .. }
                | top::Request::ApplicationDone { .. }
                | top::Request::ApplicationAgain { .. }
                | top::Request::RestartDone { .. }
                | top::Request::Call { .. } => {}
            }
        }
        if finished {
            break;
        }
    }
    finished
}

fn opened_change() -> Protocol {
    let mut world = Protocol::new(11);
    world.adopt();
    world.take_seen();
    world.event(top::Event::Change { row: change_row() });
    step_change(&mut world, 2, change::Status::Unknown, false);
    world.run_for(1);
    assert!(world.seen().iter().any(|request| matches!(
        request,
        top::Request::ChangeDecision { task: 51, decision: change::Decision::Delegate(change::Delegate::Produce), .. }
    )));
    world.produce(b"temper/51");
    world.take_seen();
    step_change(&mut world, 3, change::Status::Passed, false);
    world.run_for(1);
    assert!(world.seen().iter().any(|request| matches!(
        request,
        top::Request::ChangeDecision {
            task: 51,
            decision: change::Decision::Effect(change::Effect::Open),
            entry: Some(3),
            ..
        }
    )));
    assert_eq!(world.writes(), 0);
    world.event(top::Event::Committed { entry: 3 });
    world.run_for(2);
    assert_eq!(world.writes(), 1);

    world
}

fn enqueue_merge(world: &mut Protocol, pull: u64, head: skein_forge::api::Commit) {
    world.event(top::Event::Enqueue {
        entry: skein_forge_client::Entry {
            number: 1,
            task: 51,
            repository: REPO,
            effect: skein_forge_client::Effect {
                write: skein_forge::api::Write::Merge { number: pull, head },
                condition: skein_forge_client::Condition::Merge { base: Box::from(&b"main"[..]) },
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    });
}

fn held_branch(world: &mut Protocol) -> (top::Name, skein_forge::api::Commit) {
    let name = top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([Box::from(&b"temper"[..]), Box::from(&b"51"[..])])),
    };
    world.event(top::Event::Change { row: change_row() });
    world.event(top::Event::Names { task: 51, resources: Box::new([name.clone()]) });
    world.event(top::Event::Hold { task: 51, resource: name.clone(), from: None });
    world.run_for(1);
    world.event(top::Event::Claim { task: 51, attempt: 1, writes: Box::new([name.clone()]), holders: Box::new([]) });
    world.produce(b"temper/51");
    let pushed = temper_engine_forge_world::translate::commit(world.branch(b"temper/51"));
    world.event(top::Event::Answered { task: 51, attempt: 1, pushed: Box::new([(name.clone(), pushed)]) });
    world.run_for(1);
    world.take_seen();

    (name, pushed)
}
