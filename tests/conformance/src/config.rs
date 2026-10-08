//! Tiny forge policies and limits reconstructed for each cold start.
use jig_conformance::scenarios::Scenario;
use jig_core_authority as authority;
use skein_lib::{Duration, Queue};
use temper_engine_domain::engine;
use temper_engine_domain_world::walking;

#[expect(clippy::too_many_lines, reason = "the fixture specifies its policy and child bounds together")]
pub fn root(seed: u64, scenario: Option<jig_conformance::scenarios::Scenario>) -> (engine::Config, engine::Limits) {
    let mut limits = walking::limits();
    limits.people.people = 4;
    limits.people.amendment_bytes = 2048;
    limits.people.holdings = 4;
    limits.authority.requirements = 8;
    limits.authority.facts = 8;
    limits.brief.gather = Duration::from_secs(5);
    limits.brief.read_bytes = 512;
    limits.brief.budgets.pull = 384;
    limits.brief.budgets.ci = 384;
    limits.brief.brief_bytes = 768;
    limits.tasks.tasks = 16;
    limits.tasks.project_tasks = 16;
    limits.tasks.delegates = 4;
    limits.tasks.tree_tasks = 16;
    limits.tasks.depth = 3;
    limits.tasks.parameters = 8;
    limits.tasks.spec_bytes = 512;
    limits.tasks.executor_kinds = 2;
    limits.tasks.authority_grants = 8;
    limits.tasks.authority_segments = 9;
    limits.tasks.authority_bytes = 512;
    limits.tasks.contract_choices = 2;
    limits.tasks.inbox_messages = 8;
    limits.tasks.inbox_bytes = 1024;
    limits.authority.executors = 2;
    limits.authority.batch = 2;
    limits.authority.grants = 8;
    limits.authority.segments = 9;
    limits.authority.segment_bytes = 64;
    limits.authority.writes = 2;

    limits.journal.result_bytes = 4096;
    limits.journal.transcript_bytes = 32_768;
    limits.fleet.slots = 2;
    limits.fleet.workstreams = 2;
    limits.notes.scopes = 3 + limits.authority.grants;
    limits.tasks.funders = 16;
    limits.tasks.batch = 4;
    limits.call_records = 16;
    limits.forge.named_calls = 16;
    limits.fleet.calls = 8;
    limits.fleet.attempts = 16;
    limits.fleet.turns = 16;
    limits.people.requests = 32;
    limits.loads.rows = 32;
    limits.loads.reply_bytes = 65_536;
    limits.loads.bytes = 65_536;
    limits.forge.client.lifetime = Duration::from_secs(1);
    limits.forge.queue_window = Duration::from_secs(1);
    limits.journal.writes = 8192;
    limits.journal.deliveries = 512;
    limits.journal.held = 4096;

    let mut config = walking::config(seed);
    let pattern = authority::Pattern {
        segments: Box::new([b"forge".as_slice().into(), b"forge.example".as_slice().into(), b"org".as_slice().into()]),
        last: authority::Last::Open(b"repo".as_slice().into()),
    };
    // Every chat uses its real owned push workspace. Only the producer stories
    // need pull/merge grants; shared stories do not project issues.
    let kinds: &[u16] = match scenario {
        Some(Scenario::LateCopy | Scenario::UncertainRestarts | Scenario::NarrowingPolicy) => &[1, 2, 6],
        Some(Scenario::OtherHand | Scenario::ChangedJudge) => &[1, 2, 3, 4],
        Some(Scenario::StandingTask | Scenario::RunBudget) => &[1, 2],
        Some(Scenario::ShrinkingPool) => panic!("forge has no pooled holds"),
        None => &[1, 8, 2, 3, 4, 6],
    };
    let grants: Box<[authority::Grant]> =
        kinds.iter().map(|&kind| authority::Grant { connector: 0, kind, pattern: pattern.clone() }).collect();
    let mut rules = config.authority.rules().clone();
    // Reserve room in the chat's 100-unit allotment for its 20-unit delegate.
    rules.maximum_run_spend = 60;
    rules.ceiling.tools = authority::Tools(1023);
    let mut ceiling = grants.to_vec();
    if scenario.is_none() {
        ceiling.push(authority::Grant { connector: 0, kind: 7, pattern });
    }
    rules.ceiling.grants = ceiling.into_boxed_slice();
    rules.ceiling.delegation.kinds = Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(2)]);
    rules.ceiling.delegation.tasks = 12;
    rules.ceiling.delegation.depth = 3;
    let mut policy = config.authority.policy(1).expect("walking project").clone();
    policy.ceiling.tools = rules.ceiling.tools;
    policy.ceiling.grants.clone_from(&rules.ceiling.grants);
    policy.ceiling.delegation = rules.ceiling.delegation.clone();
    policy.projections = if scenario.is_none() { rules.ceiling.grants.clone() } else { Box::new([]) };
    policy.roles[0].authority.tools = rules.ceiling.tools;
    policy.roles[0].authority.grants.clone_from(&grants);
    policy.roles[0].authority.delegation = rules.ceiling.delegation.clone();
    let mut domain = authority::Domain::new(rules, limits.authority).expect("policy limits");
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    config.authority = domain;
    config.chat_authority.tools = authority::Tools(1023);
    config.chat_authority.grants = grants;
    config.chat_authority.delegation.kinds =
        Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(2)]);
    config.chat_authority.delegation.tasks = 8;
    config.chat_authority.delegation.depth = 2;
    let landing: Box<[engine::LandingRule]> = Box::new([engine::LandingRule {
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: Box::new([
                b"forge".as_slice().into(),
                b"forge.example".as_slice().into(),
                b"org".as_slice().into(),
                b"repo".as_slice().into(),
                b"branch".as_slice().into(),
            ]),
            last: authority::Last::Exact(b"main".as_slice().into()),
        },
        ci: true,
        up_to_date: true,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]);
    config.landing.deployment.clone_from(&landing);
    assert!(config.landing.projects.insert(1, landing).is_ok());
    (config, limits)
}
