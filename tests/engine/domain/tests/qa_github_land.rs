//! Limits of the change procedure through the engine root.
use skein_lib::{Duration, ReplyTo, Token, Wall};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_world::{direct::Driver, fixtures};
use temper_engine_forge_world::changes::change_row;

fn config() -> engine::Config {
    use jig_domain_authority as a;
    let mut config = temper_engine_domain_world::walking::config(71);
    config.core.connectors = Box::new([1]);
    for role in &mut config.core.seed_projects[0].policy.permissions {
        role.connector = 1;
    }
    let mut rules = config.core.authority.rules().clone();
    for grant in &mut rules.ceiling.grants {
        grant.connector = 1;
    }
    let mut policy = config.core.seed_projects[0].policy.policy.clone();
    for grant in &mut policy.ceiling.grants {
        grant.connector = 1;
    }
    for grant in &mut policy.projections {
        grant.connector = 1;
    }
    for role in &mut policy.roles {
        for grant in &mut role.authority.grants {
            grant.connector = 1;
        }
    }
    let mut authority = a::Domain::new(rules, *config.core.authority.limits()).expect("forge-number policy");
    let mut out = skein_lib::Queue::with_capacity(a::POLICY_MAX_OUT);
    a::step(&mut authority, a::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(a::PolicyFact::Added { project: 1 }));
    config.core.seed_projects[0].policy.policy = authority.policy(1).expect("configured seed policy").clone();
    config.core.authority = jig_domain_authority::Domain::new(authority.rules().clone(), *authority.limits())
        .expect("configured authority");
    config
}

fn driver() -> (Driver, u64) {
    driver_with_limits(&fixtures::forge_limits())
}

fn driver_with_limits(limits: &engine::Limits) -> (Driver, u64) {
    use temper_engine_domain_world::{forge_stand::ForgeStand, store::Store};
    let mut driver = Driver::configured(Store::new(), [config(), config()], limits);
    driver.forge = Some(ForgeStand::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let sign_in = driver.session();
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        scope: jig_domain_people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(900)),
        sign_in,
        key: [71; 16],
        digest: [71; 32],
        carried: None,
        request: temper_engine_domain::Application::Forge(forge::Ask::Adopt(fixtures::adoption(1))),
    }));
    driver.settle();
    (driver, sign_in)
}

fn step(driver: &mut Driver, task: u64) {
    driver.send(engine::Event::Forge(Box::new(forge::Event::StepChange {
        task,
        entry: 990,
        heard: change::Heard {
            delegate: change::Status::Unknown,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
    })));
    driver.pump();
}

#[test]
#[ignore = "needs: requester-backed change setup with silent CI and clock controls; deferred to round 3"]
fn silent_ci_at_shipped_stall_boundary() {
    // QA: domain/forge.md 8.4: a forge wait is held past the default 30 min.
    let mut limits = fixtures::forge_limits();
    // The world's tiny fixture uses 30 s; exercise the shipped value from the brief.
    limits.forge.change_policy.stall = Duration::from_secs(30 * 60);
    let (mut driver, _) = driver_with_limits(&limits);
    driver.forge.as_mut().expect("forge").push_branch(2, b"temper/51");
    let skein_forge::api::Answer::Commit(head) = driver.forge.as_mut().expect("forge").read(
        temper_engine_forge_world::REPO,
        skein_forge::api::Read::Branch { branch: b"temper/51".as_slice().into() },
    ) else {
        panic!("pushed head")
    };
    let mut row = change_row();
    row.change.since = Wall::EPOCH;
    row.change.last_head = Some(head);
    row.change.state = change::State::Checking { head };
    driver.send(engine::Event::Forge(Box::new(forge::Event::Change { row })));
    driver.pump();
    for millis in [1_799_999, 1_800_000, 1_800_001] {
        driver.fire_at(Duration::from_millis(millis));
        step(&mut driver, 51);
        let row = driver
            .store
            .rows
            .values()
            .find_map(|record| {
                if let engine::Record::Forge { row } = record
                    && let forge::Stored::Change(row) = row.as_ref()
                {
                    Some(row)
                } else {
                    None
                }
            })
            .expect("durable change");
        if millis < 1_800_000 {
            assert!(
                matches!(row.change.state, change::State::Checking { .. }),
                "QA: no early stall at {millis} ms: {:?}",
                row.change.state
            );
        } else {
            let change::State::Held { why, .. } = &row.change.state else {
                panic!("QA: 30 min must hold silent CI at {millis} ms: {:?}", row.change.state);
            };
            assert_eq!(format!("{why:?}"), "Stalled", "QA: held for the stalled check");
        }
    }
}

#[test]
#[ignore = "needs: requester-backed change procedure fixture with silent CI on the engine-domain public surface"]
fn stalled_change_notifies_its_requester() {
    // QA: domain/forge.md 8.4; Q2's requester half. The public direct fixture
    // can inject change_row(51), but that row has no core task or requester.
    let (mut driver, _) = driver();
    // Needed: create a real procedure task requested by the signed-in person,
    // attach a never-reporting required CI context, then drive its stall timer.
    driver.fire_at(Duration::from_secs(1800));
    driver.pump();
    let requester_change = driver.store.rows.values().any(|record| {
        matches!(record,
            engine::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)))
                if task.number == 51
        )
    });
    assert!(requester_change, "needs a real task/requester before asserting a delivered hold notice");
    assert!(
        driver.task(51).is_some_and(|task| matches!(task.phase, jig_domain_tasks::Phase::Held { .. })),
        "QA: the real procedure task must be held too"
    );
    assert!(
        driver.delivered.iter().any(|delivery| format!("{delivery:?}").contains("Stalled")),
        "QA: the requester must receive a notice naming the Stalled reason"
    );
}

#[test]
#[ignore = "needs: delayed required-brief reads and many-file paging controls; deferred to round 3"]
fn slow_many_file_brief_reaches_shipped_gather_limit() {
    // QA: domain/forge.md 8.4 requires failed reads to be represented in briefs;
    // domain/testing.md 4.2 requires fault/timing controls. Q6 also needs the
    // gather-expiry contract missing from the named design sections.
    let mut limits = fixtures::forge_limits();
    limits.brief.gather = Duration::from_secs(30);
    let (mut driver, _) = driver_with_limits(&limits);
    // Needed: a real repair brief over many files and several pages, each
    // forge read delayed at GitHub-like latency, with reads pending at 30 s.
    driver.fire_at(Duration::from_millis(29_999));
    driver.pump();
    let before = driver.forge.as_ref().expect("forge").calls;
    driver.fire_at(Duration::from_millis(30_001));
    driver.pump();
    assert!(
        driver.forge.as_ref().expect("forge").calls > before,
        "needs slow paged brief reads in flight across the gather deadline"
    );
    // Assert the documented timeout outcome here once the contract is named;
    // do not guess partial-success versus failure/hold from implementation.
}
