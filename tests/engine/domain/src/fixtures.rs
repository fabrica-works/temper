//! The forge-backed direct driver's limits and adoption form values.
use crate::direct::Driver;
use jig_domain_tasks as tasks;
use skein_lib::Token;
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

#[must_use]
pub fn forge_limits() -> engine::Limits {
    let mut limits = crate::walking::limits();
    limits.people.people = 4;
    limits.people.holdings = 4;
    limits.journal.writes +=
        (jig_domain_people::max_out(&limits.people) - jig_domain_people::max_out(&crate::walking::limits().people)) * 4;
    limits
}

#[must_use]
pub fn adoption(forge: u16) -> forge::Adoption {
    forge::Adoption {
        project: 1,
        home: true,
        forge,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: forge::Role::Owned,
        landing: Box::from(&b"main"[..]),
        ci: true,
        checks: Box::new([]),
    }
}

#[must_use]
pub fn report_delegate(words: &[u8], dependencies: Box<[engine::Dependency]>) -> engine::Delegate {
    engine::Delegate {
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: 128 },
        authority: tasks::Authority {
            tools: tasks::Tools(1023),
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 10, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        symbolic_grants: Box::new([]),
        dependencies,
        wake: tasks::WakePolicy::DEFAULT,
    }
}

pub fn tool_call(driver: &mut Driver, source: &engine::Assignment, call: u64, tool: engine::Tool) {
    let body = engine::Call { completion: 1, position: u32::try_from(call).expect("small call"), tool };
    driver.call(Token::new(7), source.task, source.attempt, call, body);
    for _ in 0..60 {
        driver.advance(true);
    }
}

#[must_use]
pub fn task_amendment(words: Option<&[u8]>, authority: Option<tasks::Authority>, reason: &[u8]) -> tasks::Amendment {
    tasks::Amendment {
        addressed: None,
        instructions: None,
        release: None,
        spec: words.map(|words| tasks::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) }),
        wake: None,
        dependencies: None,
        authority,
        reason: reason.into(),
    }
}

/// An authenticated chat with authority left for one direct delegate.
#[must_use]
pub fn revision_delegate() -> (Driver, engine::Assignment, u64) {
    use crate::walking::{config, limits};
    let mut bounds = limits();
    bounds.call_records = 8;
    bounds.journal.writes = 3000;
    bounds.journal.held = 300;
    bounds.journal.deliveries = 100;
    bounds.tasks.depth = 2;
    bounds.brief.sections = 5;
    let mut configuration = config(92);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.delegation.depth = 2;
    let mut policy = configuration.core.authority.policy(1).expect("fixture project").clone();
    policy.ceiling.delegation.depth = 2;
    policy.roles[0].authority.delegation.depth = 2;
    let mut authority = jig_domain_authority::Domain::new(rules, bounds.authority).expect("fixture authority");
    let mut out = skein_lib::Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
    jig_domain_authority::step(&mut authority, jig_domain_authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = authority;
    let (mut driver, parent) = Driver::configured_chat(configuration, &bounds);
    let mut member = report_delegate(b"first", Box::new([]));
    member.authority.budget.spend = 100;
    tool_call(&mut driver, &parent, 60, engine::Tool::Delegate { batch: Box::new([member]) });
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(60) else {
        panic!("one delegate admitted: {:?}", driver.answered(60));
    };
    let child = children[0];
    (driver, parent, child)
}
