//! client/README.md, 8 and 10; domain/forge.md, 8.2–8.8.
//! The landing half of the panel story, with each step's external facts at hand.
use jig_client::contract as c;
use jig_client_world::harness::{Until, World};
use jig_domain_tasks as tasks;
use skein_fake_person::{Find, Step};
use skein_forge::api;
use skein_lib::Token;
use skein_world::domain::Verdict;
use std::fmt::Write as _;
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_world::forge_stand::ForgeStand;

const REPOSITORY: api::Repository = api::Repository { forge: 1, repository: 7 };
const BRANCH: &[u8] = b"temper/handed-change";
const BASE: &[u8] = b"main";
const BUILD: &[u8] = b"build";
const STEPS: u32 = 200;
const WORKER: Token = Token::new(7);

struct Facts {
    pull: api::Pull,
    checks: Box<[api::Status]>,
    main: api::Commit,
}

// Observe only between engine steps, without charging the engine's calls.
fn read(front: &mut Joint, what: api::Read) -> api::Answer {
    front.engine.forge.as_mut().expect("story's forge").read(REPOSITORY, what)
}

fn facts(front: &mut Joint) -> Facts {
    let api::Answer::Pull(pull) = read(front, api::Read::PullFor { head: BRANCH.into(), base: BASE.into() }) else {
        panic!("opened pull request")
    };
    let api::Answer::Checks(checks) = read(front, api::Read::Checks { commit: pull.commit }) else {
        panic!("CI contexts at the pull's head")
    };
    let api::Answer::Commit(main) = read(front, api::Read::Branch { branch: BASE.into() }) else { panic!("main head") };
    Facts { pull, checks, main }
}

// Joint polling fires deadlines and settles immediate work at simulated time.
fn step(world: &mut World<Temper, Joint>) {
    let outcome = world.run(Until::Steps(2));
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn pull_opened(front: &Joint, goal: u64) -> bool {
    front.engine.store.rows.values().any(|record| {
        matches!(record, engine::Record::Forge { row } if matches!(row.as_ref(),
            temper_engine_domain_forge::Stored::Change(change) if change.task == goal && change.pull.is_some()))
    })
}

fn gate_task(front: &Joint, goal: u64) -> Option<tasks::TaskRecord> {
    front.engine.store.rows.values().find_map(|record| {
        if let engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = record
            && row.root == goal
            && row.number != goal
            && front.hosted_attempt(row.number) == Some(row.attempt)
        {
            Some(*row.clone())
        } else {
            None
        }
    })
}

fn finished(document: &c::Task, goal: u64, pull: u64) {
    assert_eq!(document.chip.task, goal);
    assert_eq!(document.chip.phase, c::Phase::Ended);
    let Some(c::TaskResult::Document(result)) = document.result.as_ref() else { panic!("historical task result") };
    assert_eq!(result.task.task, goal);
    let c::Ending::Task(ending) = &result.ending else { panic!("task's finished result") };
    let c::OwnerChipEnding::Done(c::ResultValue::Change { part, .. }) = ending.as_ref() else {
        panic!("successfully landed change result");
    };
    assert_eq!(part.key, c::OwnerPartKey::Number(pull));
}

#[test]
fn a_change_handed_its_pushed_branch_lands_after_ci_and_its_gate() {
    let mut world = fixtures::world(fixtures::SEED);
    *world.front() = Joint::with_change_landing(fixtures::SEED, &fixtures::domain_limits());
    world.front().engine.forge = Some(ForgeStand::with_checks(fixtures::REPOSITORIES, &[BUILD]));
    fixtures::adopt_change_repository(world.front());
    world.front().engine.forge.as_mut().expect("story's forge").push_branch(REPOSITORY.repository, BRANCH);
    let api::Answer::Commit(head) = read(world.front(), api::Read::Branch { branch: BRANCH.into() }) else {
        panic!("pushed branch head")
    };
    let goal = fixtures::goal_with_pushed_change(world.front(), BRANCH);

    for _ in 0..STEPS {
        if pull_opened(world.front(), goal) {
            break;
        }
        step(&mut world);
    }
    assert!(pull_opened(world.front(), goal), "pull opened within the step budget");
    let opened = facts(world.front());
    assert_ne!(opened.pull.number, 0);
    assert_eq!(opened.pull.state, api::State::Open);
    assert_eq!(opened.pull.head.as_ref(), BRANCH);
    assert_eq!(opened.pull.base.as_ref(), BASE);
    assert_eq!(opened.pull.commit, head);
    assert_eq!(opened.pull.merged, None);

    for _ in 0..STEPS {
        if gate_task(world.front(), goal).is_some() {
            break;
        }
        step(&mut world);
    }
    let gate = gate_task(world.front(), goal).expect("gate assigned within the step budget");
    let checked = facts(world.front());
    assert_eq!(checked.pull.number, opened.pull.number);
    assert_eq!(checked.pull.commit, head);
    assert_eq!(checked.pull.ci, api::Ci::Passed);
    assert_eq!(checked.checks.len(), 1);
    assert_eq!(checked.checks[0].context.as_ref(), BUILD);
    assert_eq!(checked.checks[0].check, api::Check::Passed);
    assert_eq!(checked.pull.merged, None, "blocking gate has no verdict yet");
    assert_eq!(
        world.front().engine.task(goal).expect("change task").made,
        2,
        "a handed branch needs only its gate, no producer answer"
    );
    assert_eq!(gate.requester, tasks::Party::Task(goal));
    assert!(matches!(gate.contract, tasks::Contract::Verdict { .. }));
    assert!(gate.spec.parameters.contains(&tasks::Parameter::Bytes { name: 3, value: BRANCH.into() }));
    let mut hexadecimal = String::with_capacity(40);
    for byte in &head[..20] {
        write!(&mut hexadecimal, "{byte:02x}").expect("hexadecimal head");
    }
    assert!(gate.spec.parameters.contains(&tasks::Parameter::Bytes { name: 8, value: hexadecimal.as_bytes().into() }));

    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(person, vec![Step::See { find: Find::text(b"Shown on the forge."), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    let verdict = tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() };
    world.front().engine.answer(
        WORKER,
        gate.number,
        gate.attempt,
        0,
        tasks::End::Finished { result: verdict.clone(), cancel_delegates: false },
        engine::Parts::none(),
    );
    world.front().engine.settle();
    for _ in 0..STEPS {
        if world.front().engine.task(goal).is_none() {
            break;
        }
        step(&mut world);
    }
    assert!(world.front().engine.task(goal).is_none(), "change finished within the step budget");
    let gate_key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(gate.number)));
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(answered)))) =
        world.front().engine.store.rows.get(&gate_key)
    else {
        panic!("gate's durable answer");
    };
    assert_eq!(answered.phase, tasks::Phase::Ended(tasks::Ending::Done(verdict)));
    let landed = facts(world.front());
    assert_eq!(landed.pull.number, opened.pull.number);
    assert_eq!(landed.pull.commit, head);
    assert_eq!(landed.pull.state, api::State::Closed);
    let merge = landed.pull.merged.expect("pull request's merge commit");
    assert_eq!(landed.main, merge);
    assert_ne!(merge, checked.main);

    world.person(person, vec![Step::See { find: Find::text(b"Shown on the forge."), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let root = world.root(person).person();
    let document = root.pages().task().expect("task page").document(root.client()).expect("finished task document");
    finished(document, goal, landed.pull.number);
}
