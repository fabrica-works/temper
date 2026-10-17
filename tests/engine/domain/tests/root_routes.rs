use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use temper_engine_domain as engine;
use temper_engine_domain::{Delivery, Key, Record, Write};
use temper_engine_domain_world::direct::Driver;
use temper_engine_domain_world::fixtures::{report_delegate, task_amendment, tool_call};
use temper_engine_domain_world::protocol;
use temper_engine_domain_world::store::Store;
use temper_engine_domain_world::walking::{Settings, World, charter, config, limits};
use temper_engine_domain_world::walking_referee::{FINAL_SPEND, QUESTION, REPORT};

fn turn(driver: &mut Driver, assignment: &engine::Assignment, number: u32, cumulative: u64) {
    driver.send(protocol::turn(Token::new(7), assignment.task, assignment.attempt, number, cumulative, None, b"step"));
}

/// Step a hub record through the root until its decision's commit is issued:
/// its route crosses the fleet and the world's protocol first.
fn until_pending(driver: &mut Driver) {
    for _ in 0..10 {
        if !driver.store.pending.is_empty() {
            return;
        }
        driver.advance(false);
    }
    panic!("the record's commit is issued within ten steps");
}

/// A worker's hello with its own slots, stop bound and hosted runs.
fn hello_with(channel: Token, slots: u32, stop_bound: Duration, hosting: Box<[host::Hosting]>) -> engine::Event {
    let hello = host::Hello { slots, stop_bound, workstreams: Box::new([]), hosting, bounds: protocol::bounds() };
    engine::Event::Worker(engine::WorkerInput::Link { channel, link: engine::Link::Hello(hello) })
}

/// Every hub record the root sent down a worker channel, in order.
fn downs(driver: &Driver) -> Vec<&host::LinkDown> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| if let Delivery::Link { down, .. } = delivery { Some(down) } else { None })
        .collect()
}

/// How many answers the root acknowledged to their workers.
fn acknowledged(driver: &Driver) -> usize {
    downs(driver).into_iter().filter(|down| matches!(down, host::LinkDown::Acknowledge { .. })).count()
}

/// How many turns the root told their workers it kept.
fn kept_turns(driver: &Driver) -> usize {
    downs(driver).into_iter().filter(|down| matches!(down, host::LinkDown::TurnKept { .. })).count()
}

/// The runs the root cancelled, as (task, attempt), in order.
fn cancelled(driver: &Driver) -> Vec<(u64, u64)> {
    downs(driver)
        .into_iter()
        .filter_map(|down| {
            if let host::LinkDown::Cancel { run, attempt } = down { Some((run.raw(), attempt.raw())) } else { None }
        })
        .collect()
}

/// The committed words the root sent a live run, as (name, words).
fn messages_to(driver: &Driver, run: &engine::Assignment) -> Vec<(u64, Box<[u8]>)> {
    downs(driver)
        .into_iter()
        .filter_map(|down| match down {
            host::LinkDown::Message { run: task, attempt, name, words, .. }
                if task.raw() == run.task && attempt.raw() == run.attempt =>
            {
                Some((name.raw(), words.clone()))
            }
            host::LinkDown::Message { .. }
            | host::LinkDown::Assign { .. }
            | host::LinkDown::Relayed { .. }
            | host::LinkDown::TurnKept { .. }
            | host::LinkDown::Acknowledge { .. }
            | host::LinkDown::Cancel { .. }
            | host::LinkDown::Grant { .. } => None,
        })
        .collect()
}

/// A live task's stored row.
fn live(driver: &Driver, task: u64) -> &tasks::TaskRecord {
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task))))
    else {
        panic!("task {task} is live")
    };
    row
}

/// A person's amendment of a task's words or authority, with its reason.
fn person_amendment(words: Option<&[u8]>, authority: Option<people::Authority>, reason: &[u8]) -> people::Amendment {
    people::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: words.map(|words| people::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) }),
        wake: None,
        dependencies: None,
        authority,
        reason: reason.into(),
    }
}

/// A person's amendment of `task` at its current specification revision.
fn amend(driver: &Driver, task: u64, amendment: people::Amendment) -> people::Ask {
    people::Ask::Amend { project: 1, task, revision: live(driver, task).revisions.spec, amendment, release: None }
}

/// A tracked goal a person asks for under the deployment's charter.
fn goal(spec: &[u8], budget: u64, priority: u32) -> people::Ask {
    people::Ask::SetGoal {
        project: 1,
        spec: spec.into(),
        executor: people::GoalExecutor::Charter(1),
        budget,
        priority,
        deadline: None,
    }
}

/// The goal a person's ask started, from its reply.
fn started_goal(driver: &Driver) -> u64 {
    driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::GoalStarted { task }), .. } =
                delivery
            {
                Some(*task)
            } else {
                None
            }
        })
        .expect("goal started")
}

/// The proposal a person's goal or amendment became, from its reply.
fn proposed(driver: &Driver) -> u64 {
    driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply {
                reply:
                    people::Reply::Outcome(
                        people::Outcome::Proposed { proposal, .. } | people::Outcome::AmendProposed { proposal, .. },
                    ),
                ..
            } = delivery
            {
                Some(*proposal)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            panic!("proposal committed: {:?}", driver.delivered.iter().rev().take(4).collect::<Vec<_>>())
        })
}

/// Whether some web reply carries this outcome.
fn replied(driver: &Driver, outcome: &people::Outcome) -> bool {
    driver.delivered.iter().any(
        |delivery| matches!(delivery, Delivery::WebReply { reply: people::Reply::Outcome(found), .. } if found == outcome),
    )
}

/// The tracked goals the store holds.
/// The tracked goals the store holds.
fn goals(driver: &Driver) -> Vec<u64> {
    driver
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) = row
                && task.tracked.is_some()
            {
                Some(task.number)
            } else {
                None
            }
        })
        .collect()
}

/// Rebuild the configuration's authority with project one's policy changed.
fn with_policy(configuration: &mut engine::Config, change: impl FnOnce(&mut authority::Policy)) {
    let mut policy = configuration.core.authority.policy(1).expect("configured project").clone();
    change(&mut policy);
    let rules = configuration.core.authority.rules().clone();
    let mut domain = authority::Domain::new(rules, *configuration.core.authority.limits()).expect("valid authority");
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = domain;
}

/// The walking configuration with its role's chats at 100 each: a chat takes
/// its role's spend from its person's pool (jig a2616181), so one pool of 500
/// funds more than one.
fn sharing_config(seed: u64) -> engine::Config {
    let mut configuration = config(seed);
    with_policy(&mut configuration, |policy| policy.roles[0].authority.budget.spend = 100);
    configuration
}

/// Run the driver while the world's account component keeps its grants back.
fn advance_withholding(driver: &mut Driver, withheld: &mut Vec<engine::Event>, steps: u32) {
    for _ in 0..steps {
        driver.advance(true);
        for event in std::mem::take(&mut driver.events) {
            if matches!(
                event,
                engine::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted { .. }))
            ) {
                withheld.push(event);
            } else {
                driver.events.push_back(event);
            }
        }
    }
}

fn people_roles_config(seed: u64) -> (engine::Config, engine::Limits) {
    let mut limits = limits();
    limits.fleet.slots = 2;
    limits.authority.roles = 4;
    limits.people.people = 4;
    limits.people.sign_ins = 4;
    limits.people.holdings = 4;
    limits.people.requests = 8;
    limits.people.pending = 4;
    // jig's children name more outputs per route now; the journal takes them.
    limits.journal.writes = 6000;
    limits.journal.deliveries = 40;
    limits.journal.held = 120;
    let mut config = config(seed);
    let mut authority =
        authority::Domain::new(config.core.authority.rules().clone(), limits.authority).expect("larger policy room");
    let mut policy = config.core.authority.policy(1).expect("configured project").clone();
    let owner = policy.roles[0].clone();
    let mut maintainer = owner.clone();
    maintainer.number = 1;
    let mut member = owner.clone();
    member.number = 2;
    member.authority.budget.spend = 100;
    member.period_spend = 500;
    member.decides = authority::Proposals(0);
    let mut observer = owner.clone();
    observer.number = 3;
    observer.requests = authority::Requests(128);
    observer.decides = authority::Proposals(0);
    policy.roles = Box::new([owner, maintainer, member, observer]);
    let mut out = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut authority, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    config.core.authority = authority;
    (config, limits)
}

fn sign_in_person(driver: &mut Driver, user: u64, reply: u64) -> (u64, u64) {
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        reply_to: ReplyTo::new(Token::new(reply)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: (user).to_be_bytes().into() },
            login: b"person".as_slice().into(),
            name: b"Person".as_slice().into(),
        },
    }));
    driver.settle();
    signed_in(driver, reply)
}

/// The person and session a sign-in's reply named. Person numbers come from
/// the replies: a first start seeds the configured owners (jig 0f55ef2d).
fn signed_in(driver: &Driver, reply: u64) -> (u64, u64) {
    driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply { to, reply: people::Reply::SignedIn { person, sign_in, .. } } = delivery
                && *to == ReplyTo::new(Token::new(reply))
            {
                Some((*person, *sign_in))
            } else {
                None
            }
        })
        .expect("signed in")
}

fn people_roles_driver() -> (Driver, u64, u64, u64, u64, u64) {
    let (config, limits) = people_roles_config(219);
    let mut driver = Driver::configured(Store::new(), config, &limits);
    driver.settle();
    let (owner, owner_session) = sign_in_person(&mut driver, 7, 2001);
    let (maintainer, maintainer_session) = sign_in_person(&mut driver, 8, 2002);
    let (member, member_session) = sign_in_person(&mut driver, 9, 2003);
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2004)),
        sign_in: owner_session,
        key: [200; 16],
        ask: people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: owner, role: people::Role::Owner },
                people::Holding { person: maintainer, role: people::Role::Maintainer },
                people::Holding { person: member, role: people::Role::Member },
            ]),
        },
    }));
    driver.settle();
    (driver, owner_session, maintainer, maintainer_session, member, member_session)
}

#[test]
fn a_member_starts_a_chat_and_an_observer_is_refused() {
    let (mut driver, owner_session, maintainer, _, member, member_session) = people_roles_driver();
    let (observer, observer_session) = sign_in_person(&mut driver, 11, 2030);
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2031)),
        sign_in: owner_session,
        key: [207; 16],
        ask: people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person: maintainer, role: people::Role::Maintainer },
                people::Holding { person: member, role: people::Role::Member },
                people::Holding { person: observer, role: people::Role::Observer },
            ]),
        },
    }));
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2033)),
        sign_in: observer_session,
        key: [209; 16],
        ask: people::Ask::StartChat { project: 1, words: b"hello".as_slice().into() },
    }));
    driver.settle();
    // Role refusals carry the role held (jig b7bdd9e8).
    let refused = people::Refusal::Role { held: Some(people::Role::Observer) };
    assert!(driver.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Refused(found)), .. } if *found == refused
    )));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2032)),
        sign_in: member_session,
        key: [208; 16],
        ask: people::Ask::StartChat { project: 1, words: b"hello".as_slice().into() },
    }));
    for _ in 0..20 {
        driver.advance(true);
        if driver.delivered.iter().any(|delivery| {
            matches!(
                delivery,
                Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Started { .. }), .. }
            )
        }) {
            break;
        }
    }
    assert!(driver.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Started { .. }), .. }
    )));
}

#[test]
fn a_person_answers_a_numbered_question_from_their_task() {
    let (mut driver, parent) = batch_fixture_with(2, 1);
    let child = call_batch(&mut driver, &parent, 2034, Box::new([report_delegate(b"ask", Box::new([]))]))[0];
    let asker = assigned_task(&driver, child);
    let words = b"ship?".as_slice().into();
    tool_call(
        &mut driver,
        &asker,
        2035,
        engine::Tool::Message { target: parent.task, form: engine::MessageForm::Question, words },
    );
    let Some(engine::CallAnswer::Sent { message: question }) = driver.answered(2035).cloned() else {
        panic!("numbered question committed")
    };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2036)),
        sign_in: driver.session(),
        key: [210; 16],
        ask: people::Ask::AnswerQuestion { project: 1, task: parent.task, question, words: b"yes".as_slice().into() },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|item| matches!(item,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::QuestionAnswered {
            task, question: answered, ..
        }), .. } if *task == parent.task && *answered == question
    )));
    assert!(
        live(&driver, parent.task)
            .inbox
            .iter()
            .any(|word| word.kind == (tasks::MessageKind::Answer { question }) && word.from == tasks::Party::Person(1))
    );
}

#[test]
fn a_maintainer_prioritises_project_goals_and_a_member_cannot() {
    let (mut driver, _, _, maintainer_session, _, member_session) = people_roles_driver();
    driver.send(hello_with(Token::new(7), 2, Duration::from_secs(1), Box::new([])));
    driver.settle();
    for index in 0..2 {
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: ReplyTo::new(Token::new(2040 + index)),
            sign_in: member_session,
            key: [u8::try_from(211 + index).expect("small key"); 16],
            ask: goal(b"goal", 20, 1),
        }));
        driver.settle();
    }
    let goals = goals(&driver);
    assert_eq!(goals.len(), 2);
    let priorities: Box<[(u64, u32)]> = Box::new([(goals[0], 4), (goals[1], 9)]);
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2042)),
        sign_in: member_session,
        key: [213; 16],
        ask: people::Ask::Prioritise { project: 1, goals: priorities.clone() },
    }));
    driver.settle();
    // Role refusals carry the role held (jig b7bdd9e8).
    let refused = people::Refusal::Role { held: Some(people::Role::Member) };
    assert!(replied(&driver, &people::Outcome::Refused(refused)));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2043)),
        sign_in: maintainer_session,
        key: [214; 16],
        ask: people::Ask::Prioritise { project: 1, goals: priorities.clone() },
    }));
    driver.settle();
    assert!(replied(&driver, &people::Outcome::Prioritised { project: 1 }));
    for (number, priority) in priorities {
        assert_eq!(live(&driver, number).tracked, Some(priority));
    }
}

#[test]
fn a_person_amends_their_live_task_and_the_change_reaches_its_run() {
    let (mut driver, assignment) = batch_fixture();
    let amendment = person_amendment(Some(b"revised goal"), None, b"new context");
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2044)),
        sign_in: driver.session(),
        key: [215; 16],
        ask: amend(&driver, assignment.task, amendment),
    }));
    driver.settle();
    assert!(replied(&driver, &people::Outcome::Amended { task: assignment.task, released: false }));
    assert_eq!(live(&driver, assignment.task).spec.words.as_ref(), b"revised goal");
    assert!(driver.store.rows.values().any(|record| matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::History(history)))
            if history.task == assignment.task && history.change == tasks::Change::Amended
                && history.by == tasks::Party::Person(1)
    )));
}

#[test]
fn a_requester_may_amend_and_cancel_without_role_control_bits() {
    let bounds = limits();
    let mut configuration = config(2045);
    let mut policy = configuration.core.authority.policy(1).expect("project policy").clone();
    policy.roles[0].requests = authority::Requests(1 | 4);
    let mut authority = authority::Domain::new(configuration.core.authority.rules().clone(), bounds.authority)
        .expect("valid authority");
    let mut output = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut authority, authority::Event::Policy { project: 1, policy }, &mut output);
    assert_eq!(output.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = authority;
    let mut driver = Driver::configured(Store::new(), configuration, &bounds);
    driver.settle();
    driver.sign_in();
    driver.settle();
    hello(&mut driver);
    driver.settle();
    chat(&mut driver, 46);
    driver.settle();
    let task = assigned(&driver).task;
    let amendment = person_amendment(Some(b"requester edit"), None, b"requester decides");
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2046)),
        sign_in: driver.session(),
        key: [47; 16],
        ask: amend(&driver, task, amendment),
    }));
    driver.settle();
    assert!(replied(&driver, &people::Outcome::Amended { task, released: false }));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2047)),
        sign_in: driver.session(),
        key: [48; 16],
        ask: people::Ask::Cancel { project: 1, task, reason: b"requester closes".as_slice().into(), grace: None },
    }));
    driver.settle();
    assert!(replied(&driver, &people::Outcome::Cancelled { task }));
}

#[test]
fn a_members_wider_amendment_waits_for_a_maintainer_to_accept() {
    let (mut driver, _, _, maintainer_session, _, member_session) = people_roles_driver();
    driver.send(hello_with(Token::new(7), 2, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2045)),
        sign_in: member_session,
        key: [216; 16],
        ask: goal(b"goal", 50, 1),
    }));
    driver.settle();
    let task = started_goal(&driver);
    let wider = people::Authority {
        tools: 0,
        grants: Box::new([]),
        delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
        spend: 150,
        deadline: None,
        notes: 0,
        note_resources: Box::new([]),
    };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2046)),
        sign_in: member_session,
        key: [217; 16],
        ask: amend(&driver, task, person_amendment(None, Some(wider), b"more scope")),
    }));
    driver.settle();
    let proposal = proposed(&driver);
    assert_eq!(live(&driver, task).authority.budget.spend, 50);
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2047)),
        sign_in: maintainer_session,
        key: [218; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(task),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ProposalDecided {
            proposer: people::Proposer::Task(proposer), proposal: decided, choice: people::ProposalChoice::Accepted, ..
        }), .. } if *proposer == task && *decided == proposal
    )));
    assert_eq!(live(&driver, task).authority.budget.spend, 150);
}

#[test]
fn a_members_goal_past_their_allotment_becomes_a_proposal_a_maintainer_accepts() {
    let (mut driver, _, maintainer, maintainer_session, member, member_session) = people_roles_driver();
    hello(&mut driver);
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2010)),
        sign_in: member_session,
        key: [201; 16],
        ask: jig_core_world::fixtures::propose_goal(goal(b"ship a goal", 150, 7)),
    }));
    driver.settle();
    let proposal = proposed(&driver);
    assert!(
        !driver
            .store
            .rows
            .values()
            .any(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
    );
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2011)),
        sign_in: maintainer_session,
        key: [202; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Party(member),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ProposalDecided {
            proposer: people::Proposer::Party(proposer), proposal: decided, choice: people::ProposalChoice::Accepted, ..
        }), .. } if *proposer == member && *decided == proposal
    )));
    assert!(driver.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
            if task.requester == tasks::Party::Person(member) && task.tracked == Some(7)
    )));
    // A decided proposal moves to its history family (jig c231752c).
    assert!(
        matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::DecidedProposal(proposal)))),
            Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::DecidedProposal(row))))
                if row.state == tasks::PersonProposalState::Accepted { by: tasks::Party::Person(maintainer) }
        )
    );
}

#[test]
fn two_maintainers_decide_one_proposal_and_the_second_is_told_by_whom_and_how() {
    let (mut driver, owner_session, first, first_session, member, member_session) = people_roles_driver();
    hello(&mut driver);
    driver.settle();
    let (second, second_session) = sign_in_person(&mut driver, 10, 2020);
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2021)),
        sign_in: owner_session,
        key: [203; 16],
        ask: people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person: first, role: people::Role::Maintainer },
                people::Holding { person: second, role: people::Role::Maintainer },
                people::Holding { person: member, role: people::Role::Member },
            ]),
        },
    }));
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(2022)),
        sign_in: member_session,
        key: [204; 16],
        ask: jig_core_world::fixtures::propose_goal(goal(b"race for goal", 150, 3)),
    }));
    driver.settle();
    let proposal = proposed(&driver);
    let proposer = people::Proposer::Party(member);
    for (reply, session, key, decision) in [
        (2023, first_session, 205, people::ProposalDecision::Accept),
        (2024, second_session, 206, people::ProposalDecision::Reject { reason: b"too late".as_slice().into() }),
    ] {
        let ask = people::Ask::DecideProposal { project: 1, proposer, proposal, decision };
        let to = ReplyTo::new(Token::new(reply));
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: to,
            sign_in: session,
            key: [key; 16],
            ask,
        }));
        driver.settle();
    }
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { to, reply: people::Reply::Outcome(people::Outcome::ProposalDecided {
            proposer: found, proposal: named, by, choice: people::ProposalChoice::Accepted, ..
        }) } if *to == ReplyTo::new(Token::new(2024)) && *found == proposer && *named == proposal && *by == first
    )));
    let decision = jig_domain_core::CoreKey::ProposalDecision { proposer, proposal };
    assert!(driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Core(decision))));
}

#[test]
fn durable_start_turn_and_answer_callbacks_survive_full_journal_pressure() {
    let mut driver = Driver::new(Store::new());
    driver.hello(Token::new(7));
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(201)),
        sign_in: driver.session(),
        key: [8; 16],
        ask: people::Ask::StartChat { project: 1, words: QUESTION.into() },
    }));
    driver.pressure();
    assert!(!driver.delivered.iter().any(|delivery| matches!(delivery, Delivery::Assigned { .. })));
    driver.settle();
    let assigned = driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Assigned { .. })).count();
    assert_eq!(assigned, 1);
    let assignment = driver.last_assignment();
    turn(&mut driver, &assignment, 1, 3);
    until_pending(&mut driver);
    driver.pressure();
    assert_eq!(kept_turns(&driver), 0);
    driver.settle();
    turn(&mut driver, &assignment, 2, 8);
    driver.settle();
    assert_eq!(kept_turns(&driver), 2);
    let end =
        tasks::End::Finished { result: tasks::TaskResult::Report { words: REPORT.into() }, cancel_delegates: false };
    driver.answer(Token::new(7), assignment.task, assignment.attempt, FINAL_SPEND, end, engine::Parts::none());
    until_pending(&mut driver);
    driver.pressure();
    assert_eq!(acknowledged(&driver), 0);
    driver.settle();
    assert_eq!(acknowledged(&driver), 1);
    assert_eq!(driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Result { .. })).count(), 1);
}

// The named result read that followed the restart is the core's now (jig
// 62410d35); the restart's own promise stays.
#[test]
fn restart_replays_no_raw_notice_of_an_ended_result() {
    let mut world = World::new(Settings { restart: false, ..Settings::calm(92) });
    world.run();
    assert!(
        world
            .store
            .rows
            .values()
            .any(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(_))))),
        "ended task"
    );
    let rows = world.store.rows.clone();
    let mut driver = Driver::new(world.store);
    assert!(!driver.root.quiescent(), "cold startup has outstanding work");
    driver.settle();
    assert!(driver.delivered.is_empty(), "startup does not repeat any person or worker notice");
    assert_eq!(driver.store.rows, rows, "startup commits nothing for an ended result");
}

// The role inbox views between the steps are the core's reads now (jig
// 62410d35); the claim's own route stays.
#[test]
fn a_person_task_addressed_to_a_role_taken_by_one_handed_back_answered_by_another() {
    let (mut driver, root) = batch_fixture_custom(2, 1, 500, true, false, true);
    let first = driver.session();
    sign_in_person(&mut driver, 8, 970);
    let second = driver.store.header().sign_ins;
    let mut delegate = report_delegate(b"Choose between the reports", Box::new([]));
    delegate.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    let choice = |code| tasks::Verdict { code, words: 16, followups: 0, fields: Box::new([]) };
    delegate.contract = tasks::Contract::Verdict { choices: Box::new([choice(1), choice(2)]) };
    let task = call_batch(&mut driver, &root, 971, Box::new([delegate]))[0];
    for (reply, session, key, ask) in [
        (974, first, 74, people::Ask::TakePerson { project: 1, task }),
        (975, first, 75, people::Ask::HandBackPerson { project: 1, task }),
        (976, second, 76, people::Ask::TakePerson { project: 1, task }),
    ] {
        let to = ReplyTo::new(Token::new(reply));
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: to,
            sign_in: session,
            key: [key; 16],
            ask,
        }));
        driver.settle();
        let taken = if reply == 975 { None } else { Some(if session == first { 1 } else { 2 }) };
        assert_eq!(live(&driver, task).taken_by, taken);
    }
    assert!(replied(&driver, &people::Outcome::PersonTaken { task }));
    let result = people::PersonResult::Verdict { code: 2, words: b"second".as_slice().into() };
    let revision = live(&driver, task).revisions.person;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(977)),
        sign_in: second,
        key: [77; 16],
        ask: people::Ask::AnswerPerson { project: 1, task, revision, result },
    }));
    driver.settle();
    assert!(replied(&driver, &people::Outcome::PersonAnswered { task }), "{:?}", driver.delivered);
    assert!(driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))));
}

#[test]
fn a_person_stops_a_run_and_releases_it() {
    let (mut driver, assignment) = running_fixture();
    let session = driver.session();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(980)),
        sign_in: session,
        key: [80; 16],
        ask: people::Ask::Stop { project: 1, task: assignment.task },
    }));
    for _ in 0..40 {
        driver.advance(true);
    }
    assert!(replied(&driver, &people::Outcome::Stopped { task: assignment.task }));
    assert_eq!(cancelled(&driver), [(assignment.task, assignment.attempt)]);
    let row = live(&driver, assignment.task);
    assert!(matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::StoppedBy { party: 1 }, .. }));
    driver.answer(Token::new(7), assignment.task, assignment.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let revision = live(&driver, assignment.task).revisions.hold;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(981)),
        sign_in: session,
        key: [81; 16],
        ask: people::Ask::Release { project: 1, task: assignment.task, revision, again: None },
    }));
    for _ in 0..40 {
        driver.advance(true);
    }
    assert!(replied(&driver, &people::Outcome::Released { task: assignment.task }));
}

fn hello(driver: &mut Driver) {
    driver.hello(Token::new(7));
}

fn chat(driver: &mut Driver, key: u8) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(401)),
        sign_in: driver.session(),
        key: [key; 16],
        ask: people::Ask::StartChat { project: 1, words: QUESTION.into() },
    }));
}

fn assigned(driver: &Driver) -> engine::Assignment {
    driver
        .delivered
        .iter()
        .find_map(
            |delivery| {
                if let Delivery::Assigned { assignment, .. } = delivery { Some(assignment.clone()) } else { None }
            },
        )
        .unwrap_or_else(|| panic!("assigned chat: {:?}", driver.delivered))
}

// Accounts are wanted at the restart's first step and granted by the
// component; a missing grant retains the activation (jig 7dcc65c5).
#[test]
fn an_ungranted_account_retains_the_chat_until_its_grant() {
    let mut driver = Driver::new(Store::new());
    hello(&mut driver);
    let mut withheld = Vec::new();
    advance_withholding(&mut driver, &mut withheld, 50);
    assert!(driver.root.ready());
    assert_eq!(driver.protocol.accounts, [accounts::Request::Component(accounts::ToComponent::Want { account: 1 })]);
    driver.sign_in();
    advance_withholding(&mut driver, &mut withheld, 50);
    chat(&mut driver, 9);
    advance_withholding(&mut driver, &mut withheld, 50);
    assert!(!driver.delivered.iter().any(|delivery| matches!(delivery, Delivery::Assigned { .. })));
    assert_eq!(withheld.len(), 1, "one grant for the one wanted account");
    driver.events.extend(withheld);
    driver.settle();
    let grant = accounts::Grant {
        table: accounts::GrantTable::Accounts,
        account: 1,
        generation: 1,
        valid: Duration::from_secs(3600),
    };
    assert_eq!(assigned(&driver).grants.as_ref(), [grant]);
    assert_eq!(driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Assigned { .. })).count(), 1);
}

// A full journal hands each loss back to the world, which offers it again
// once there is room (D6; jig's `domain/root.md`, 3).
#[test]
fn repeated_unknown_losses_under_commit_pressure_consume_no_callback_room() {
    let mut driver = Driver::new(Store::new());
    driver.settle();
    // The world sends the deployment's first period once the root is ready.
    driver.settle();
    assert!(driver.period_sent);
    driver.sign_in();
    driver.sign_in();
    driver.sign_in();
    assert_eq!(driver.store.pending.len(), 3);
    for channel in 1000..5000 {
        driver.send(protocol::lost(Token::new(channel)));
    }
    assert_eq!(driver.returned, 4000);
    assert_eq!(driver.store.pending.len(), 3);
    let transactions = driver.transactions.len();
    for _ in 0..4100 {
        driver.advance(true);
    }
    assert!(driver.events.is_empty() && driver.store.pending.is_empty());
    assert!(driver.root.quiescent());
    assert_eq!(driver.transactions.len(), transactions, "unknown losses commit nothing");
    assert!(driver.delivered.iter().all(|delivery| matches!(delivery, Delivery::WebReply { .. })));
}

#[test]
fn refused_terminal_clears_fleet_handoff_without_charging_rejected_spend() {
    let mut driver = Driver::new(Store::new());
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 10);
    driver.settle();
    let assignment = assigned(&driver);
    let result = tasks::TaskResult::Report { words: vec![b'x'; 129].into_boxed_slice() };
    let end = tasks::End::Finished { result, cancel_delegates: false };
    driver.answer(Token::new(7), assignment.task, assignment.attempt, FINAL_SPEND, end, engine::Parts::none());
    driver.settle();
    assert_eq!(acknowledged(&driver), 1);
    let task = live(&driver, assignment.task);
    assert_eq!(task.run_spent, 0);
    assert_eq!(task.numbers.spent, 0);
    assert_eq!(task.tries.invalid, 1);
    assert!(matches!(task.phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
}

#[test]
fn a_refused_assignment_spends_no_try() {
    let mut limits = limits();
    limits.fleet.attempts = 1;
    let mut driver = Driver::configured(Store::new(), sharing_config(94), &limits);
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 11);
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    assert_eq!(driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Assigned { .. })).count(), 1);
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) =
        driver.store.rows.get(&temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(2))))
    else {
        panic!("second chat is durable but unplaced");
    };
    assert!(matches!(task.phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
    assert_eq!(task.run_spent, 0);
    assert_eq!(task.numbers.spent, 0);
    assert_eq!(task.tries, tasks::Tries::NONE);
}

// A chat's authority comes from its person's role policy (jig a2616181).
#[test]
fn static_authority_waits_are_durable_holds_and_do_not_spin_the_ready_pass() {
    for deadline in [false, true] {
        let mut configuration = config(95);
        with_policy(&mut configuration, |policy| {
            if deadline {
                policy.roles[0].authority.budget.deadline = Some(Wall::EPOCH);
            } else {
                policy.roles[0].authority.budget.spend = 1;
            }
        });
        let mut driver = Driver::configured(Store::new(), configuration, &limits());
        driver.env.wall = Wall::from_nanos(1);
        hello(&mut driver);
        driver.settle();
        driver.sign_in();
        driver.settle();
        chat(&mut driver, 13);
        driver.settle();
        let expected = if deadline { tasks::Hold::Deadline } else { tasks::Hold::Budget };
        assert!(matches!(live(&driver, 1).phase, tasks::Phase::Held { why, .. } if why == expected));
        let header = driver.store.header();
        let outputs = driver.delivered.len();
        for _ in 0..20 {
            driver.advance(true);
        }
        assert_eq!(driver.store.header(), header);
        assert_eq!(driver.delivered.len(), outputs);
        assert!(driver.root.quiescent());
    }
}

#[test]
fn invalid_nonfinal_people_restore_page_stops_before_issuing_its_continuation() {
    let mut store = Store::new();
    for number in 1..=3 {
        let record = Record::Core(jig_domain_core::Record::People(people::Stored::Person {
            number,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
                login: b"same".as_slice().into(),
                name: b"Same".as_slice().into(),
            },
        }));
        store.seed(Box::new([record]), Box::new([]));
    }
    let mut driver = Driver::new(store);
    for _ in 0..30 {
        driver.advance(true);
    }
    assert!(driver.stopped, "duplicate identity in middle page is a terminal startup failure");
    assert!(!driver.root.ready());
    assert!(!driver.root.quiescent());
    assert!(driver.events.is_empty(), "failed page did not issue another continuation");
}

#[test]
fn invalid_nonfinal_task_restore_page_stops_before_issuing_its_continuation() {
    let mut world = World::new(Settings { restart: false, ..Settings::calm(96) });
    world.run();
    let mut record = world
        .store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(record))) = row {
                Some(record.as_ref().clone())
            } else {
                None
            }
        })
        .expect("ended fixture");
    record.spec.words = vec![b'x'; 65].into_boxed_slice();
    record.phase = tasks::Phase::Waiting;
    let row = Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(record))));
    world.store.seed(Box::new([row]), Box::new([]));
    let mut driver = Driver::new(world.store);
    for _ in 0..60 {
        driver.advance(true);
    }
    assert!(driver.stopped, "oversized child shape ends startup");
    assert!(!driver.root.ready());
    assert!(!driver.root.quiescent());
    assert!(driver.events.is_empty());
}

#[test]
fn root_configuration_refuses_a_decoded_page_larger_than_synchronous_route_room() {
    let mut limits = limits();
    limits.loads.rows = u32::MAX;
    assert!(engine::worst_case(&limits).is_none());
}

#[test]
fn an_early_worker_hello_does_not_number_a_commit_before_the_durable_header() {
    let mut driver = Driver::configured(Store::new(), config(98), &limits());
    driver.hello(Token::new(7));
    assert!(driver.transactions.is_empty(), "the cold load must precede commit numbering");
    assert!(driver.delivered.is_empty(), "the core accepts a valid early worker");
    driver.settle();
    assert!(driver.root.ready());
    assert!(!driver.stopped);
}

#[test]
fn maximum_cold_hello_batch_and_duplicate_losses_fit_one_startup_decision() {
    let mut limits = limits();
    limits.fleet.workers = 4;
    // Each worker's fleet outputs take route and delivery room (jig's fleet `max_out`).
    limits.journal.writes += 4;
    limits.journal.deliveries = 25;
    let mut driver = Driver::configured(Store::new(), config(97), &limits);
    for channel in 1..=4 {
        driver.send(hello_with(Token::new(channel), 1, Duration::from_secs(5), Box::new([])));
        driver.send(protocol::lost(Token::new(channel)));
        driver.send(protocol::lost(Token::new(channel)));
    }
    driver.send(hello_with(Token::new(1), 1, limits.fleet.grace, Box::new([])));
    driver.settle();
    assert_eq!(driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Refuse { .. })).count(), 5);
    assert!(!driver.stopped);
    limits.fleet.workers = 16;
    assert!(engine::worst_case(&limits).is_none(), "cold callback outputs plus continuation must fit");
}

#[test]
fn overflowing_child_limits_refuse_root_startup_without_panicking() {
    let mut configured = limits();
    configured.tasks.tasks = u32::MAX;
    assert!(engine::worst_case(&configured).is_none());
    configured = limits();
    configured.fleet.workers = u32::MAX;
    configured.fleet.slots = u32::MAX;
    assert!(engine::worst_case(&configured).is_none());
}

fn running_fixture() -> (Driver, engine::Assignment) {
    let mut driver = Driver::new(Store::new());
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    assert!(
        driver.delivered.iter().any(|item| matches!(item, Delivery::Assigned { .. })),
        "delegate fixture delivery: {:?}",
        driver.delivered
    );
    let assignment = assigned(&driver);
    turn(&mut driver, &assignment, 1, 3);
    driver.settle();
    (driver, assignment)
}

fn unavailable_call(driver: &mut Driver, assignment: &engine::Assignment, call: u64) {
    let body = engine::Call { completion: 2, position: 0, tool: engine::Tool::Unavailable };
    driver.call(Token::new(7), assignment.task, assignment.attempt, call, body);
}

// A chat's delegation comes from its role's, one level shallower (jig a2616181).
fn delegate_fixture() -> (Driver, engine::Assignment) {
    let mut configuration = config(91);
    let mut rules = configuration.core.authority.rules().clone();
    rules.maximum_run_spend = 60;
    rules.ceiling.delegation.depth = 2;
    let mut policy = configuration.core.authority.policy(1).expect("fixture project").clone();
    policy.ceiling.delegation.depth = 2;
    policy.roles[0].authority.delegation.depth = 2;
    let mut authority = jig_domain_authority::Domain::new(rules, *configuration.core.authority.limits())
        .expect("expanded fixture authority");
    let mut policy_out = Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
    jig_domain_authority::step(
        &mut authority,
        jig_domain_authority::Event::Policy { project: 1, policy },
        &mut policy_out,
    );
    assert_eq!(policy_out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = authority;
    let mut driver = Driver::configured(Store::new(), configuration, &limits());
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    assert!(
        driver.delivered.iter().any(|item| matches!(item, Delivery::Assigned { .. })),
        "delegate fixture delivery: {:?}",
        driver.delivered
    );
    let assignment = assigned(&driver);
    (driver, assignment)
}

fn batch_fixture() -> (Driver, engine::Assignment) {
    batch_fixture_with(1, 1)
}

fn batch_fixture_with(slots: u32, depth: u32) -> (Driver, engine::Assignment) {
    batch_fixture_with_pool(slots, depth, 500)
}

fn batch_fixture_with_pool(slots: u32, depth: u32, pool_budget: u64) -> (Driver, engine::Assignment) {
    batch_fixture_custom(slots, depth, pool_budget, false, false, false)
}

// Chats are funded by their role's policy: its period spend is the person's
// pool, and the chat's authority is the role's, one level shallower, within
// the pool (jig a2616181). The role's spend covers a 150 goal and leaves the
// pool room for one beside its chat.
fn batch_fixture_custom(
    slots: u32,
    depth: u32,
    pool_budget: u64,
    second_owner: bool,
    procedure: bool,
    person: bool,
) -> (Driver, engine::Assignment) {
    let mut bounds = limits();
    bounds.tasks.tasks = 4;
    // jig's core prices each task's inbox projections (`memory.rs`, `worst_case`).
    bounds.people.inbox_tasks = 8;
    bounds.people.inbox_recipients = 17;
    bounds.tasks.project_tasks = 4;
    bounds.tasks.tree_tasks = 4;
    bounds.tasks.depth = depth;
    bounds.tasks.delegates = 3;
    bounds.tasks.batch = 3;
    bounds.tasks.dependencies = 2;
    bounds.tasks.inbox_messages = 6;
    bounds.tasks.inbox_bytes = 384;
    if person {
        bounds.tasks.contract_choices = 2;
        bounds.people.requests = 8;
    }
    if second_owner {
        bounds.people.initial_owners = 2;
    }
    bounds.authority.batch = 3;
    if procedure || person {
        bounds.authority.executors = 2;
        bounds.tasks.executor_kinds = 2;
    }
    bounds.fleet.attempts = 5;
    bounds.fleet.slots = slots;
    bounds.call_records = 4;
    bounds.brief.sections = 5;
    bounds.journal.writes = 6000;
    bounds.journal.deliveries = 100;
    bounds.journal.held = 300;
    let mut configuration = config(92);
    if second_owner {
        configuration.core.owners = Box::new([
            people::InitialOwner {
                project: 1,
                identity: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            },
            people::InitialOwner {
                project: 1,
                identity: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            },
        ]);
    }
    // A run's budget holds its finishing reserve (jig a020bf1c); as jig's
    // fixtures do, the charter's price unit keeps a run budget of 5 fundable.
    let mut cheaper = charter();
    cheaper.policy.model.price_unit = 2000;
    configuration.core.settings.charters = Box::new([cheaper]);
    let mut rules = configuration.core.authority.rules().clone();
    // Each claimed run leaves room in its task allotment for delegates made during the run.
    rules.maximum_run_spend = 5;
    if procedure || person {
        rules.ceiling.delegation.kinds = Box::new([
            jig_domain_authority::Executor::Charter(1),
            if procedure {
                jig_domain_authority::Executor::Procedure(1)
            } else {
                jig_domain_authority::Executor::Role(0)
            },
        ]);
    }
    rules.ceiling.delegation.tasks = 4;
    rules.ceiling.delegation.depth = depth + 1;
    let mut policy = configuration.core.authority.policy(1).expect("fixture project").clone();
    if procedure || person {
        policy.ceiling.delegation.kinds.clone_from(&rules.ceiling.delegation.kinds);
        policy.roles[0].authority.delegation.kinds.clone_from(&rules.ceiling.delegation.kinds);
    }
    policy.ceiling.delegation.tasks = 4;
    policy.ceiling.delegation.depth = depth + 1;
    policy.roles[0].authority.delegation.tasks = 4;
    policy.roles[0].authority.delegation.depth = depth + 1;
    policy.roles[0].authority.budget.spend = 150;
    policy.roles[0].period_spend = pool_budget;
    let mut authority = jig_domain_authority::Domain::new(rules, bounds.authority).expect("larger fixture authority");
    let mut policy_out = Queue::with_capacity(jig_domain_authority::POLICY_MAX_OUT);
    jig_domain_authority::step(
        &mut authority,
        jig_domain_authority::Event::Policy { project: 1, policy },
        &mut policy_out,
    );
    assert_eq!(policy_out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = authority;
    let mut driver = Driver::configured(Store::new(), configuration, &bounds);
    driver.send(hello_with(Token::new(7), slots, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let assignment = assigned(&driver);
    (driver, assignment)
}

// A chat's note scopes are its role's (jig a2616181).
fn note_config() -> engine::Config {
    let mut configuration = config(3100);
    let mut rules = configuration.core.authority.rules().clone();
    rules.ceiling.notes = authority::Scopes(7);
    let mut policy = configuration.core.authority.policy(1).expect("project policy").clone();
    policy.ceiling.notes = authority::Scopes(7);
    policy.roles[0].authority.notes = authority::Scopes(7);
    let mut checked = authority::Domain::new(rules, *configuration.core.authority.limits()).expect("note policy");
    let mut facts = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut checked, authority::Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.authority = checked;
    configuration
}

fn note_fixture() -> (Driver, engine::Assignment) {
    let mut driver = Driver::configured(Store::new(), note_config(), &limits());
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let assignment = assigned(&driver);
    (driver, assignment)
}

/// A run's note tool writing a new project note.
fn new_note(description: &[u8], body: &[u8], author: notes::Author) -> engine::Tool {
    engine::Tool::Note {
        entry: notes::New {
            name: 0,
            scope: notes::Scope::Project { project: 1 },
            description: description.into(),
            body: body.into(),
            references: skein_lib::List::with_capacity(1),
            author,
        },
        recalled: None,
    }
}

#[test]
fn a_new_runs_brief_carries_the_durable_note_index() {
    let mut driver = Driver::configured(Store::new(), note_config(), &limits());
    driver.settle();
    let mut store = driver.store;
    let scope = notes::Scope::Project { project: 1 };
    store.seed(
        Box::new([Record::Core(jig_domain_core::Record::Notes(notes::Record::Entry(notes::Entry {
            name: 1,
            scope: scope.clone(),
            description: b"seeded hint".as_slice().into(),
            body: b"full text".as_slice().into(),
            references: skein_lib::List::with_capacity(1),
            author: notes::Author::Task { task: 1, attempt: 1 },
            revision: 1,
            changed: Wall::EPOCH,
        })))]),
        Box::new([]),
    );
    store.seed(
        Box::new([Record::Core(jig_domain_core::Record::Notes(notes::Record::Line(notes::Line {
            scope,
            name: 1,
            description: b"seeded hint".as_slice().into(),
            revision: 1,
        })))]),
        Box::new([]),
    );
    let mut driver = Driver::configured(store, note_config(), &limits());
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let assignment = assigned(&driver);
    assert!(assignment.sections.iter().any(|section| matches!(section,
        engine::BriefSection {
            kind: engine::BriefKind::Core(brief::Core::NotesIndex),
            body: engine::BriefBody::Text(text),
        } if text.windows(b"seeded hint".len()).any(|window| window == b"seeded hint")
    )));
}

// A chat's tool families are its role's (jig a2616181).
#[test]
fn a_note_tool_without_its_family_is_refused_with_an_authority_finding() {
    let mut configuration = note_config();
    with_policy(&mut configuration, |policy| policy.roles[0].authority.tools = authority::Tools(1023 & !128));
    let mut driver = Driver::configured(Store::new(), configuration, &limits());
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let assignment = assigned(&driver);
    let author = notes::Author::Task { task: assignment.task, attempt: assignment.attempt };
    tool_call(&mut driver, &assignment, 3104, new_note(b"not authorized", b"none", author));
    assert!(matches!(driver.answered(3104),
        Some(engine::CallAnswer::ToolDenied { answer: authority::Answer::Propose, findings })
            if findings.contains(&authority::Finding::Tool)
    ));
    assert!(
        !driver.store.rows.values().any(|record| matches!(record, Record::Core(jig_domain_core::Record::Notes(_))))
    );
}

fn written_note_name(driver: &Driver, expected_call: u64) -> u64 {
    let Some(engine::CallAnswer::NoteWritten { name, revision: 1 }) = driver.answered(expected_call) else {
        panic!("note write answered after commit")
    };
    *name
}

#[test]
fn a_note_written_corrected_by_a_person_and_recalled_is_durable() {
    let (mut driver, assignment) = note_fixture();
    let author = notes::Author::Task { task: assignment.task, attempt: assignment.attempt };
    tool_call(&mut driver, &assignment, 3101, new_note(b"original", b"first body", author));
    let name = written_note_name(&driver, 3101);
    assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Notes(notes::Key::Entry { name }))),
        Some(Record::Core(jig_domain_core::Record::Notes(notes::Record::Entry(entry)))) if entry.body.as_ref() == b"first body"
    ));
    let change = people::NoteChange::Correct {
        description: b"corrected".as_slice().into(),
        body: b"second body".as_slice().into(),
        references: Box::new([]),
        recalled: 1,
    };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(3102)),
        sign_in: driver.session(),
        key: [31; 16],
        ask: people::Ask::EditNote {
            project: 1,
            name,
            scope: Box::new(people::NoteScope::Project),
            change: Box::new(change),
        },
    }));
    driver.settle();
    assert!(replied(&driver, &people::Outcome::NoteEdited { name }));
    tool_call(&mut driver, &assignment, 3103, engine::Tool::Recall { by: notes::Recall::Name { name }, page: 0 });
    assert!(matches!(driver.answered(3103),
        Some(engine::CallAnswer::NoteRecalled { entries, more: false })
            if entries.len() == 1 && entries[0].revision == 2 && entries[0].body.as_ref() == b"second body"
    ));
    let mut restarted = Driver::configured(driver.store, note_config(), &limits());
    let hosting = protocol::hosting(assignment.task, assignment.attempt, host::Phase::Active);
    restarted.send(hello_with(Token::new(7), 1, Duration::from_secs(1), Box::new([hosting])));
    restarted.settle();
    tool_call(&mut restarted, &assignment, 3101, new_note(b"original", b"first body", author));
    // The reasked name is answered from its settled answer, without a render (D15).
    assert_eq!(restarted.answered(3101), None);
    assert!(downs(&restarted).into_iter().any(|down| matches!(down,
        host::LinkDown::Relayed { call, answer: host::Relayed::Answer { error: false, .. }, .. }
            if call.as_ref() == 3101_u64.to_be_bytes()
    )));
    let entries = restarted
        .store
        .rows
        .values()
        .filter(|record| matches!(record, Record::Core(jig_domain_core::Record::Notes(notes::Record::Entry(_)))))
        .count();
    assert_eq!(entries, 1, "reasking a named note did not write a second entry");
}

// A note named in another scope moved from what the person saw (jig b7bdd9e8).
#[test]
fn a_person_deletes_a_note_only_in_its_authorized_scope_and_after_commit() {
    let (mut driver, assignment) = note_fixture();
    tool_call(&mut driver, &assignment, 3110, new_note(b"hint", b"body", notes::Author::Task { task: 0, attempt: 0 }));
    let name = written_note_name(&driver, 3110);
    let moved = people::Refusal::Moved { now: people::Moved::Note { revision: 1 } };
    for (request, scope, expected) in [
        (3111, people::NoteScope::Deployment, people::Outcome::Refused(moved)),
        (3112, people::NoteScope::Project, people::Outcome::NoteEdited { name }),
    ] {
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: ReplyTo::new(Token::new(request)),
            sign_in: driver.session(),
            key: [u8::try_from(request - 3000).expect("small request"); 16],
            ask: people::Ask::EditNote {
                project: 1,
                name,
                scope: Box::new(scope),
                change: Box::new(people::NoteChange::Delete { recalled: 1 }),
            },
        }));
        for _ in 0..10 {
            driver.advance(false);
        }
        assert!(
            !driver.delivered.iter().any(|delivery| matches!(delivery,
                Delivery::WebReply { to, .. } if *to == ReplyTo::new(Token::new(request))
            )),
            "party answer waits for its commit"
        );
        assert!(driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Notes(notes::Key::Entry { name }))));
        driver.settle();
        assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
            Delivery::WebReply { to, reply: people::Reply::Outcome(outcome), .. }
                if *to == ReplyTo::new(Token::new(request)) && *outcome == expected
        )));
    }
    assert!(!driver.store.rows.keys().any(|key| matches!(key, Key::Core(jig_domain_core::Key::Notes(_)))));
}

// A call past the named-answer room is answered busy by the hub's relay, as
// the root's hub calls are (jig's `domain/hosts.md`, 6.4).
#[test]
fn a_pending_note_reserves_its_named_answer_until_the_store_returns() {
    let mut bound = limits();
    bound.call_records = 1;
    bound.fleet.calls = 2;
    let mut driver = Driver::configured(Store::new(), note_config(), &bound);
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let assignment = assigned(&driver);
    let recall = engine::Tool::Recall { by: notes::Recall::Name { name: 99 }, page: 0 };
    driver.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        3120,
        engine::Call { completion: 1, position: 3120, tool: recall },
    );
    let unavailable = engine::Call { completion: 1, position: 3121, tool: engine::Tool::Unavailable };
    driver.call(Token::new(7), assignment.task, assignment.attempt, 3121, unavailable);
    driver.settle();
    // A call waiting for a call record is answered busy, to ask again
    // (jig's `domain/engine.md`, 5.1).
    assert!(downs(&driver).into_iter().any(|down| matches!(down,
        host::LinkDown::Relayed { call, answer: host::Relayed::Busy, .. } if call.as_ref() == 3121_u64.to_be_bytes()
    )));
    assert_eq!(driver.answered(3121), None);
    // The recall's decision and its settled answer each take a call number (D15).
    assert_eq!(driver.store.header().calls, 2);
    assert!(matches!(driver.answered(3120),
        Some(engine::CallAnswer::NoteRecalled { entries, more: false }) if entries.is_empty()
    ));
}

fn call_batch(
    driver: &mut Driver,
    parent: &engine::Assignment,
    call: u64,
    batch: Box<[engine::Delegate]>,
) -> Box<[u64]> {
    tool_call(driver, parent, call, engine::Tool::Delegate { batch });
    match driver.answered(call) {
        Some(engine::CallAnswer::Delegated(numbers)) => numbers.clone(),
        other => panic!("whole batch answered: {other:?}"),
    }
}

fn assigned_task(driver: &Driver, task: u64) -> engine::Assignment {
    driver
        .delivered
        .iter()
        .find_map(|item| {
            if let Delivery::Assigned { assignment, .. } = item
                && assignment.task == task
            {
                Some(assignment.clone())
            } else {
                None
            }
        })
        .expect("task assigned")
}

/// The proposal a run's call made, from its answer.
fn proposal_of(driver: &Driver, call: u64) -> u64 {
    let Some(engine::CallAnswer::Proposed { proposal }) = driver.answered(call) else {
        panic!("proposal routed: {:?}", driver.answered(call))
    };
    *proposal
}

/// A run's proposal of one more delegate, for its covering task to decide.
fn propose(words: &[u8], spend: u64, reason: &[u8], as_holder: bool) -> engine::Tool {
    let mut delegate = report_delegate(words, Box::new([]));
    delegate.authority.budget.spend = spend;
    engine::Tool::Propose {
        action: engine::ProposedAction::Batch(Box::new([delegate])),
        reason: reason.into(),
        as_holder,
    }
}

/// Whether the task's pending proposal waits at `holder`.
fn proposal_waits_at(driver: &Driver, task: u64, holder: tasks::ProposalHolder) -> bool {
    matches!(&live(driver, task).proposal, Some(proposal)
        if matches!(proposal.state, tasks::ProposalState::Pending { holder: found, .. } if found == holder))
}

/// A run's amendment of a delegate's words or authority, with its reason.

// A pending proposal is a virtual inbox entry that reaches the live run as
// the hub's message, named by its proposal (jig's `domain/hosts.md`, 2).
#[test]
fn a_proposal_reaches_its_covering_task_and_is_accepted() {
    let (mut driver, parent) = batch_fixture_with(3, 2);
    let child = call_batch(&mut driver, &parent, 61, Box::new([report_delegate(b"child", Box::new([]))]))[0];
    let child_run = assigned_task(&driver, child);
    tool_call(&mut driver, &child_run, 62, propose(b"grandchild", 10, b"need a helper", false));
    let proposal = proposal_of(&driver, 62);
    assert!(
        messages_to(&driver, &parent)
            .iter()
            .any(|(name, words)| *name == proposal && words.as_ref() == b"need a helper")
    );
    driver.send(protocol::turn(Token::new(7), parent.task, parent.attempt, 1, 1, Some(proposal), b"consider proposal"));
    driver.settle();
    assert!(downs(&driver).into_iter().any(|down| matches!(down,
        host::LinkDown::TurnKept { run, attempt, turn: 1 }
            if run.raw() == parent.task && attempt.raw() == parent.attempt
    )));
    let since = driver.env.wall;
    assert!(matches!(&live(&driver, child).proposal, Some(p)
        if p.state == tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(parent.task), since }));
    let decide = engine::Tool::Decide { proposer: child, proposal, decision: engine::ProposalChoice::Accept };
    tool_call(&mut driver, &parent, 63, decide);
    let accepted = engine::CallAnswer::ProposalDecided { proposal, outcome: tasks::ProposalOutcome::Accepted };
    assert_eq!(driver.answered(63), Some(&accepted));
    let row = live(&driver, child);
    assert!(row.proposal.is_none() && row.delegates.len() == 1);
    assert!(
        row.inbox.iter().any(|word| word.kind == tasks::MessageKind::ProposalDecision { proposal, accepted: true })
    );
}

#[test]
fn a_chats_goal_accepted_as_its_persons_outlives_the_chat() {
    let (mut driver, parent) = batch_fixture_with(3, 2);
    let child = call_batch(&mut driver, &parent, 64, Box::new([report_delegate(b"chat work", Box::new([]))]))[0];
    let child_run = assigned_task(&driver, child);
    tool_call(&mut driver, &child_run, 65, propose(b"long goal", 150, b"goal outlives chat", true));
    let proposal = proposal_of(&driver, 65);
    let person = signed_in(&driver, 101).0;
    assert!(proposal_waits_at(&driver, child, tasks::ProposalHolder::Person(person)));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(406)),
        sign_in: driver.session(),
        key: [66; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(child),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(
        |item| matches!(item, Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ProposalDecided {
        proposer: people::Proposer::Task(proposer), proposal: found, choice: people::ProposalChoice::Accepted, ..
    }), .. } if *proposer == child && *found == proposal)
    ));
    let goal_number = driver
        .store
        .rows
        .values()
        .find_map(|value| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = value
                && row.requester == tasks::Party::Person(person)
                && row.number != parent.task
            {
                Some(row.number)
            } else {
                None
            }
        })
        .expect("goal owned by accepting person");
    assert_ne!(goal_number, child);
    tool_call(&mut driver, &parent, 67, engine::Tool::Cancel { target: child, reason: b"chat done".as_slice().into() });
    assert!(
        driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(goal_number)))),
        "person's goal survives chat subtree cancellation"
    );
}

#[test]
fn a_proposal_routed_past_a_procedure_to_a_person_is_accepted() {
    let (mut driver, root) = batch_fixture_custom(3, 3, 500, false, true, false);
    let mut procedure = report_delegate(b"procedure", Box::new([]));
    procedure.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    procedure.authority.budget.spend = 50;
    procedure.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    procedure.authority.delegation.tasks = 1;
    procedure.authority.delegation.depth = 1;
    let parent = call_batch(&mut driver, &root, 601, Box::new([procedure]))[0];
    assert!(driver.delivered.iter().any(|item| matches!(item,
        Delivery::Procedure { task, step: 1, connector: 1, code: 1 } if *task == parent
    )));
    driver.send(engine::Event::Party(engine::PartyInput::ProcedureStep {
        task: parent,
        step: 1,
        connector: 1,
        code: 1,
        decision: engine::ProcedureStep {
            batch: Box::new([report_delegate(b"child", Box::new([]))]),
            ..engine::ProcedureStep::new(engine::StepEnd::Wait)
        },
    }));
    driver.settle();
    let child = driver
        .store
        .rows
        .values()
        .find_map(|value| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = value
                && row.requester == tasks::Party::Task(parent)
            {
                Some(row.number)
            } else {
                None
            }
        })
        .expect("procedure delegate committed");
    let child_run = assigned_task(&driver, child);
    tool_call(&mut driver, &child_run, 602, propose(b"larger goal", 150, b"needs person funding", false));
    let proposal = proposal_of(&driver, 602);
    let person = signed_in(&driver, 101).0;
    assert!(proposal_waits_at(&driver, child, tasks::ProposalHolder::Person(person)));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(603)),
        sign_in: driver.session(),
        key: [67; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(child),
            proposal,
            decision: people::ProposalDecision::Accept,
        },
    }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|item| matches!(item,
            Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ProposalDecided {
                proposer: people::Proposer::Task(proposer), proposal: found, choice: people::ProposalChoice::Accepted, ..
            }), .. } if *proposer == child && *found == proposal
        )),
        "deliveries: {:?}",
        driver.delivered
    );
}

#[test]
fn a_recurring_procedure_uses_the_root_period_route() {
    let mut driver = Driver::new(Store::new());
    hello(&mut driver);
    driver.settle();
    let mut member = tasks::New {
        label: 0,
        flags: tasks::Flags::PLAIN,
        number: 1,
        project: 1,
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec { words: b"period work".as_slice().into(), parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: 16 },
        authority: report_delegate(b"template", Box::new([])).authority,
        numbers: tasks::Numbers { budget: 10, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: 1, period: 0 },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    };
    member.authority.budget.spend = 10;
    let mut authority = report_delegate(b"template", Box::new([])).authority;
    authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    authority.delegation.tasks = 2;
    authority.delegation.depth = 1;
    authority.budget.spend = 30;
    driver.send(engine::Event::Party(engine::PartyInput::StartRecurring {
        project: 1,
        authority,
        template: tasks::RecurringTemplate {
            key: 1,
            batch: Box::new([member]),
            overlap: tasks::RecurringOverlap::Skip,
        },
    }));
    driver.settle();
    let master = driver
        .store
        .rows
        .iter()
        .find_map(|(key, value)| match (key, value) {
            (
                Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(number))),
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))),
            ) if row.recurring.is_some() => Some(*number),
            _ => None,
        })
        .expect("recurring task committed");
    let child = driver
        .store
        .rows
        .iter()
        .find_map(|(key, value)| match (key, value) {
            (
                Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(number))),
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))),
            ) if row.requester == tasks::Party::Task(master) => Some(*number),
            _ => None,
        })
        .expect("period batch committed");
    assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.funder == tasks::Funder::Recurring {
            project: 1, task: master, period: 1,
        }
    ));
    let run = assigned_task(&driver, child);
    driver.answer(
        Token::new(7),
        child,
        run.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Period { project: 1, period: 2, budget: 1_000, next: None }));
    driver.settle();
    let next = driver
        .store
        .rows
        .iter()
        .find_map(|(key, value)| match (key, value) {
            (
                Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(number))),
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))),
            ) if row.requester == tasks::Party::Task(master) && *number != child => Some(*number),
            _ => None,
        })
        .expect("new period batch committed");
    assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(next)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.funder == tasks::Funder::Recurring {
            project: 1, task: master, period: 2,
        }
    ));
}

#[test]
fn a_stalled_proposal_passes_up() {
    let (mut driver, root) = batch_fixture_with(3, 4);
    let mut holder = report_delegate(b"holder", Box::new([]));
    holder.authority.budget.spend = 50;
    holder.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    holder.authority.delegation.tasks = 2;
    holder.authority.delegation.depth = 3;
    let middle = call_batch(&mut driver, &root, 68, Box::new([holder]))[0];
    let middle_run = assigned_task(&driver, middle);
    let leaf = call_batch(&mut driver, &middle_run, 69, Box::new([report_delegate(b"leaf", Box::new([]))]))[0];
    let leaf_run = assigned_task(&driver, leaf);
    tool_call(&mut driver, &leaf_run, 70, propose(b"needed", 10, b"need helper", false));
    assert!(proposal_waits_at(&driver, leaf, tasks::ProposalHolder::Task(middle)), "{:?}", driver.delivered.last());
    driver.env.wall = Wall::from_nanos(Duration::from_millis(11).as_nanos());
    driver.env.now = Time::from_nanos(Duration::from_millis(11).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    assert!(proposal_waits_at(&driver, leaf, tasks::ProposalHolder::Task(root.task)));
}

#[test]
fn a_proposal_rejected_tells_the_proposer_why() {
    let (mut driver, parent) = batch_fixture_with(2, 2);
    let child = call_batch(&mut driver, &parent, 71, Box::new([report_delegate(b"child", Box::new([]))]))[0];
    let child_run = assigned_task(&driver, child);
    tool_call(&mut driver, &child_run, 72, propose(b"more", 10, b"need more", false));
    let proposal = proposal_of(&driver, 72);
    let reason = b"different plan".as_slice().into();
    let decision = engine::ProposalChoice::Reject { reason };
    tool_call(&mut driver, &parent, 73, engine::Tool::Decide { proposer: child, proposal, decision });
    let row = live(&driver, child);
    assert!(row.proposal.is_none());
    assert!(
        row.inbox.iter().any(|word| word.kind == tasks::MessageKind::ProposalDecision { proposal, accepted: false }
            && word.words.as_ref() == b"different plan")
    );
    let rejected = engine::CallAnswer::ProposalDecided { proposal, outcome: tasks::ProposalOutcome::Rejected };
    assert_eq!(driver.answered(73), Some(&rejected));
}

#[test]
fn an_amendment_reaches_a_live_run() {
    let (mut driver, parent) = batch_fixture_with(2, 1);
    let member = report_delegate(b"original", Box::new([]));
    let mut wider = member.authority.clone();
    wider.budget.spend = 20;
    let child = call_batch(&mut driver, &parent, 90, Box::new([member]))[0];
    let running = assigned_task(&driver, child);
    let amendment = task_amendment(Some(b"revised"), Some(wider), b"the target changed");
    let revision = live(&driver, child).revisions.spec;
    tool_call(&mut driver, &parent, 91, engine::Tool::Amend { target: child, revision, amendment });
    assert_eq!(driver.answered(91), Some(&engine::CallAnswer::Revised { task: child, revision: revision + 1 }));
    assert!(driver.transactions.iter().any(|writes| {
        writes.iter().any(|write| {
            matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record))))
            if record.part == jig_domain_core::CallPart::Revised { task: child, revision: revision + 1 })
        }) && writes.iter().any(|write| {
            matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))))
            if task.number == child && task.spec.words.as_ref() == b"revised" && task.numbers.budget == 20)
        }) && writes.iter().any(|write| {
            matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::History(row))))
            if row.task == child && row.change == tasks::Change::Amended)
        })
    }));
    let amended =
        live(&driver, child).inbox.iter().find(|word| matches!(word.kind, tasks::MessageKind::Amendment { .. }));
    let amended = amended.expect("the amendment's message").number;
    assert!(messages_to(&driver, &running).iter().any(|(name, _)| *name == amended));
    assert_eq!(live(&driver, parent.task).numbers.reserved, 25);
}

#[test]
fn an_ancestor_amends_a_live_grandchild() {
    let (mut driver, root) = batch_fixture_with(3, 2);
    let mut middle = report_delegate(b"middle", Box::new([]));
    middle.authority.budget.spend = 50;
    middle.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    middle.authority.delegation.tasks = 1;
    middle.authority.delegation.depth = 1;
    let child = call_batch(&mut driver, &root, 901, Box::new([middle]))[0];
    let child_run = assigned_task(&driver, child);
    let grandchild =
        call_batch(&mut driver, &child_run, 902, Box::new([report_delegate(b"grandchild", Box::new([]))]))[0];
    let grandchild_run = assigned_task(&driver, grandchild);
    let amendment = task_amendment(Some(b"revised by ancestor"), None, b"tree plan changed");
    let revision = live(&driver, grandchild).revisions.spec;
    tool_call(&mut driver, &root, 903, engine::Tool::Amend { target: grandchild, revision, amendment });
    assert_eq!(driver.answered(903), Some(&engine::CallAnswer::Revised { task: grandchild, revision: revision + 1 }));
    let row = live(&driver, grandchild);
    assert_eq!(row.spec.words.as_ref(), b"revised by ancestor");
    let amended = row.inbox.iter().find(|word| matches!(word.kind, tasks::MessageKind::Amendment { .. }));
    let amended = amended.expect("the amendment's message").number;
    assert!(messages_to(&driver, &grandchild_run).iter().any(|(name, _)| *name == amended));
}

// Waiting words travel in the run's start (jig 0f6041cb).
#[test]
fn a_narrowing_stops_the_old_run_and_offers_the_amendment_to_the_next() {
    let (mut driver, parent) = batch_fixture_with(2, 1);
    let member = report_delegate(b"child", Box::new([]));
    let mut narrower = member.authority.clone();
    narrower.budget.spend = 5;
    let child = call_batch(&mut driver, &parent, 90, Box::new([member]))[0];
    let first = assigned_task(&driver, child);
    let amendment = task_amendment(None, Some(narrower), b"smaller scope");
    let revision = live(&driver, child).revisions.spec;
    tool_call(&mut driver, &parent, 91, engine::Tool::Amend { target: child, revision, amendment });
    assert!(cancelled(&driver).contains(&(child, first.attempt)));
    driver.answer(Token::new(7), child, first.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let second = driver.last_assignment();
    assert_eq!(second.task, child);
    assert!(second.attempt > first.attempt);
    assert!(second.messages.iter().any(
        |word| matches!(word.kind, tasks::MessageKind::Amendment { .. }) && word.words.as_ref() == b"smaller scope"
    ));
    let task = live(&driver, child);
    assert_eq!(task.numbers.budget, 5);
    assert!(!task.narrowing);
}

#[test]
fn a_cancel_closes_three_levels_with_runs_live_deepest_first() {
    let (mut driver, parent) = batch_fixture_with(3, 2);
    let mut child_spec = report_delegate(b"child", Box::new([]));
    child_spec.authority.budget.spend = 20;
    child_spec.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    let child = call_batch(&mut driver, &parent, 90, Box::new([child_spec]))[0];
    let child_run = assigned_task(&driver, child);
    let grandchild =
        call_batch(&mut driver, &child_run, 91, Box::new([report_delegate(b"grandchild", Box::new([]))]))[0];
    let grandchild_run = assigned_task(&driver, grandchild);
    let reason = b"goal withdrawn".as_slice().into();
    tool_call(&mut driver, &parent, 92, engine::Tool::Cancel { target: child, reason });
    assert_eq!(cancelled(&driver), [(grandchild, grandchild_run.attempt), (child, child_run.attempt)]);
    assert_eq!(driver.answered(92), Some(&engine::CallAnswer::Controlled));
    for run in [&grandchild_run, &child_run] {
        driver.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
        driver.settle();
    }
    let endings: Vec<_> = driver
        .transactions
        .iter()
        .flat_map(|writes| writes.iter())
        .filter_map(|write| {
            if let Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task)))) = write {
                Some((task.number, task.phase.clone()))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(endings.len(), 2);
    assert_eq!(endings[0].0, grandchild);
    assert_eq!(endings[1].0, child);
    assert!(endings.iter().all(|(_, phase)| matches!(phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. }))));
}

#[test]
fn release_tool_resets_a_delegates_exhausted_tries() {
    let (mut driver, parent) = batch_fixture_with(2, 1);
    let child = call_batch(&mut driver, &parent, 90, Box::new([report_delegate(b"child", Box::new([]))]))[0];
    let first = assigned_task(&driver, child);
    driver.answer(Token::new(7), child, first.attempt, 0, tasks::End::Failed(tasks::Class::Run), engine::Parts::none());
    driver.settle();
    driver.env.now = Time::from_nanos(Duration::from_secs(2).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(2).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let second = driver.last_assignment();
    assert_eq!(second.task, child);
    driver.answer(
        Token::new(7),
        child,
        second.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
    let revision = live(&driver, child).revisions.hold;
    tool_call(&mut driver, &parent, 91, engine::Tool::Release { target: child, revision, again: None });
    assert_eq!(
        driver.answered(91),
        Some(&engine::CallAnswer::Revised { task: child, revision: live(&driver, child).revisions.spec })
    );
    let task = live(&driver, child);
    assert_eq!(task.tries, tasks::Tries::NONE);
    assert!(matches!(task.phase, tasks::Phase::Active(_)));
    assert!(driver.delivered.iter().any(|item| matches!(item, Delivery::Assigned { assignment, .. }
        if assignment.task == child && assignment.attempt > second.attempt)));
}

#[test]
fn introduced_siblings_can_exchange_named_words_after_the_references_commit() {
    let (mut driver, parent) = batch_fixture();
    let numbers = call_batch(
        &mut driver,
        &parent,
        201,
        Box::new([report_delegate(b"left", Box::new([])), report_delegate(b"right", Box::new([]))]),
    );
    for _ in 0..30 {
        driver.advance(true);
    }
    let words = b"before".as_slice().into();
    tool_call(
        &mut driver,
        &parent,
        202,
        engine::Tool::Message { target: numbers[1], form: engine::MessageForm::Words, words },
    );
    assert!(matches!(driver.answered(202), Some(engine::CallAnswer::Sent { .. })));
    tool_call(&mut driver, &parent, 203, engine::Tool::Introduce { left: numbers[0], right: numbers[1] });
    assert_eq!(driver.answered(203), Some(&engine::CallAnswer::Introduced));
    assert_eq!(live(&driver, numbers[0]).references.as_ref(), [numbers[1]]);
    let right = live(&driver, numbers[1]);
    assert_eq!(right.references.as_ref(), [numbers[0]]);
    assert!(
        right.inbox.iter().any(|word| word.from == tasks::Party::Task(parent.task) && word.words.as_ref() == b"before")
    );
}

#[test]
fn task_subscriptions_are_named_by_the_root_and_unsubscribe_removes_them() {
    let (mut driver, parent) = batch_fixture();
    let numbers = call_batch(&mut driver, &parent, 211, Box::new([report_delegate(b"watched", Box::new([]))]));
    let kind = tasks::SubscriptionKind::Task { target: numbers[0], held: true, result: true };
    tool_call(&mut driver, &parent, 212, engine::Tool::Subscribe { kind });
    let Some(engine::CallAnswer::Subscribed { subscription }) = driver.answered(212).cloned() else {
        panic!("subscription answered after commit")
    };
    assert_eq!(live(&driver, parent.task).subscriptions.as_ref(), [tasks::Subscription { number: subscription, kind }]);
    tool_call(&mut driver, &parent, 213, engine::Tool::Unsubscribe { subscription });
    assert_eq!(driver.answered(213), Some(&engine::CallAnswer::Unsubscribed));
    assert!(live(&driver, parent.task).subscriptions.is_empty());
}

fn park_task(driver: &mut Driver, assignment: &engine::Assignment) {
    driver.answer(Token::new(7), assignment.task, assignment.attempt, 0, tasks::End::Parked, engine::Parts::none());
    for _ in 0..30 {
        driver.advance(true);
    }
}

fn finish_task(driver: &mut Driver, assignment: &engine::Assignment, result: tasks::TaskResult) {
    let end = tasks::End::Finished { result, cancel_delegates: false };
    driver.answer(Token::new(7), assignment.task, assignment.attempt, 0, end, engine::Parts::none());
    for _ in 0..30 {
        driver.advance(true);
    }
}

/// One delegate a chat asks for, with no authority of its own beyond its spend.
fn plain_delegate(words: &[u8]) -> engine::Delegate {
    engine::Delegate {
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec { words: words.into(), parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: 128 },
        authority: tasks::Authority {
            tools: tasks::Tools(0),
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 10, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
    }
}

#[test]
fn a_delegate_batch_and_its_named_answer_commit_together() {
    let (mut driver, parent) = delegate_fixture();
    let tool = engine::Tool::Delegate { batch: Box::new([plain_delegate(b"investigate")]) };
    driver.call(Token::new(7), parent.task, parent.attempt, 99, engine::Call { completion: 1, position: 0, tool });
    assert_eq!(driver.answered(99), None);
    for _ in 0..30 {
        driver.advance(true);
    }
    let Some(engine::CallAnswer::Delegated(numbers)) = driver.answered(99) else {
        panic!("delegation answer after commit")
    };
    let child = numbers[0];
    assert!(driver.transactions.iter().any(|writes| {
        writes.iter().any(|write| {
            matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record))))
            if record.part == jig_domain_core::CallPart::Delegated(Box::new([child])))
        }) && writes.iter().any(|write| {
            matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))))
                if task.number == child && task.requester == tasks::Party::Task(parent.task))
        })
    }));
}

#[test]
fn a_delegate_result_enters_its_requesters_inbox_with_the_end() {
    let (mut driver, parent) = delegate_fixture();
    let tool = engine::Tool::Delegate { batch: Box::new([plain_delegate(b"spike")]) };
    driver.call(Token::new(7), parent.task, parent.attempt, 100, engine::Call { completion: 1, position: 0, tool });
    for _ in 0..30 {
        driver.advance(true);
    }
    let Some(engine::CallAnswer::Delegated(numbers)) = driver.answered(100) else { panic!("one child was made") };
    let child = numbers[0];
    park_task(&mut driver, &parent);
    let child_assignment = assigned_task(&driver, child);
    let result = tasks::TaskResult::Report { words: b"spike complete".as_slice().into() };
    finish_task(&mut driver, &child_assignment, result);
    assert!(live(&driver, parent.task).inbox.iter().any(|message| message.from == tasks::Party::Task(child)
        && message.kind == tasks::MessageKind::Result(tasks::ResultKind::Report)
        && message.words.as_ref() == b"spike complete"));
    assert!(driver.transactions.iter().any(|writes| {
        writes.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task)))) if task.number == child))
            && writes.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))))
                if task.number == parent.task && task.inbox.iter().any(|message| message.from == tasks::Party::Task(child))))
    }));
}

#[test]
fn a_plan_of_spikes_a_choice_and_changes_runs_in_dependency_order() {
    let (mut driver, parent) = batch_fixture();
    let spike = report_delegate(b"spike", Box::new([]));
    let mut choice = report_delegate(b"choose", Box::new([engine::Dependency::Batch(0)]));
    choice.contract = tasks::Contract::Verdict {
        choices: Box::new([tasks::Verdict { code: 7, words: 128, followups: 0, fields: Box::new([]) }]),
    };
    let mut change = report_delegate(b"change", Box::new([engine::Dependency::Batch(1)]));
    change.contract = tasks::Contract::Change { connector: 1, kind: 2, words: 128 };
    let numbers = call_batch(&mut driver, &parent, 110, Box::new([spike, choice, change]));
    assert_eq!(numbers.len(), 3);
    assert!(!driver.delivered.iter().any(|item| matches!(item, Delivery::Assigned { assignment, .. }
        if assignment.task == numbers[1] || assignment.task == numbers[2])));
    park_task(&mut driver, &parent);
    let first = assigned_task(&driver, numbers[0]);
    assert!(!driver.delivered.iter().any(|item| matches!(item, Delivery::Assigned { assignment, .. }
        if assignment.task == numbers[1] || assignment.task == numbers[2])));
    finish_task(&mut driver, &first, tasks::TaskResult::Report { words: b"spike says yes".as_slice().into() });
    let second = assigned_task(&driver, numbers[1]);
    assert!(second.sections.iter().any(|section| section.kind == engine::BriefKind::Core(brief::Core::Results)
        && matches!(&section.body, engine::BriefBody::Text(text) if text.windows(b"spike says yes".len()).any(|part| part == b"spike says yes"))));
    assert!(!driver.delivered.iter().any(|item| matches!(item, Delivery::Assigned { assignment, .. }
        if assignment.task == numbers[2])));
    finish_task(
        &mut driver,
        &second,
        tasks::TaskResult::Verdict { code: 7, fields: Box::new([]), words: b"choose path".as_slice().into() },
    );
    let third = assigned_task(&driver, numbers[2]);
    assert!(third.sections.iter().any(|section| section.kind == engine::BriefKind::Core(brief::Core::Results)
        && matches!(&section.body, engine::BriefBody::Text(text) if text.windows(b"verdict 7".len()).any(|part| part == b"verdict 7"))));
    finish_task(
        &mut driver,
        &third,
        tasks::TaskResult::Change { connector: 1, kind: 2, resource: 17, words: b"changed".as_slice().into() },
    );
    let parent_row = driver
        .store
        .rows
        .get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(parent.task))))
        .expect("parent remains live");
    let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(parent_row))) = parent_row else {
        panic!("parent row")
    };
    assert_eq!(parent_row.inbox.len(), 3, "one durable result per delegate");
}

#[test]
fn a_batch_beyond_authority_is_refused_whole() {
    let (mut driver, parent) = batch_fixture();
    let allowed = report_delegate(b"small", Box::new([]));
    let mut excessive = report_delegate(b"large", Box::new([]));
    excessive.authority.budget.spend = 200;
    tool_call(&mut driver, &parent, 111, engine::Tool::Delegate { batch: Box::new([allowed, excessive]) });
    assert!(matches!(driver.answered(111),
        Some(engine::CallAnswer::DelegationDenied { findings, .. }) if !findings.is_empty()));
    assert_eq!(driver.store.header().tasks, parent.task, "no member received an ID");
    assert!(!driver.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) if task.requester == tasks::Party::Task(parent.task))));
}

#[test]
fn a_negative_verdict_starts_dependents_and_a_failure_holds_them() {
    let (mut driver, parent) = batch_fixture();
    let mut verdict = report_delegate(b"check", Box::new([]));
    verdict.contract = tasks::Contract::Verdict {
        choices: Box::new([tasks::Verdict { code: 0, words: 128, followups: 0, fields: Box::new([]) }]),
    };
    let next = report_delegate(b"respond", Box::new([engine::Dependency::Batch(0)]));
    let held = report_delegate(b"finish", Box::new([engine::Dependency::Batch(1)]));
    let numbers = call_batch(&mut driver, &parent, 112, Box::new([verdict, next, held]));
    park_task(&mut driver, &parent);
    let first = assigned_task(&driver, numbers[0]);
    finish_task(
        &mut driver,
        &first,
        tasks::TaskResult::Verdict { code: 0, fields: Box::new([]), words: b"no".as_slice().into() },
    );
    let second = assigned_task(&driver, numbers[1]);
    assert!(second.sections.iter().any(|section| section.kind == engine::BriefKind::Core(brief::Core::Results)
        && matches!(&section.body, engine::BriefBody::Text(text) if text.windows(b"verdict 0".len()).any(|part| part == b"verdict 0"))));
    finish_task(&mut driver, &second, tasks::TaskResult::Failure { reason: b"blocked".as_slice().into() });
    let row = driver
        .store
        .rows
        .get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(numbers[2]))))
        .expect("dependent held live");
    let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) = row else { panic!("dependent row") };
    assert!(matches!(task.phase, tasks::Phase::Held { why: tasks::Hold::Dependency(id), .. } if id == numbers[1]));
    assert!(!driver.delivered.iter().any(|item| matches!(item, Delivery::Assigned { assignment, .. }
        if assignment.task == numbers[2])));
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the script selects the requester's next assignment")]
fn an_ended_delegate_can_be_named_as_a_later_tasks_input() {
    let (mut driver, parent) = batch_fixture();
    let first = call_batch(&mut driver, &parent, 113, Box::new([report_delegate(b"first", Box::new([]))]));
    park_task(&mut driver, &parent);
    let first_assignment = assigned_task(&driver, first[0]);
    finish_task(&mut driver, &first_assignment, tasks::TaskResult::Report { words: b"found it".as_slice().into() });
    let parent_again = driver
        .delivered
        .iter()
        .rev()
        .find_map(|item| match item {
            Delivery::Assigned { assignment, .. }
                if assignment.task == parent.task && assignment.attempt > parent.attempt =>
            {
                Some(assignment.clone())
            }
            _ => None,
        })
        .expect("result wakes requester");
    let mut second = report_delegate(b"use result", Box::new([]));
    second.spec.inputs = Box::new([first[0]]);
    let second = call_batch(&mut driver, &parent_again, 114, Box::new([second]));
    assert!(matches!(
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Stub(first[0])))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Stub(stub)))) if stub.task == first[0] && stub.result.raw() == first[0]
    ));
    park_task(&mut driver, &parent_again);
    let assigned = assigned_task(&driver, second[0]);
    assert!(assigned.sections.iter().any(|section| section.kind == engine::BriefKind::Core(brief::Core::Results)
        && matches!(&section.body, engine::BriefBody::Text(text) if text.windows(b"found it".len()).any(|part| part == b"found it"))));
}

#[test]
fn a_missing_historical_input_refuses_the_whole_batch() {
    let (mut driver, parent) = batch_fixture();
    let mut member = report_delegate(b"use missing", Box::new([]));
    member.spec.inputs = Box::new([999]);
    tool_call(&mut driver, &parent, 115, engine::Tool::Delegate { batch: Box::new([member]) });
    assert!(matches!(driver.answered(115),
        Some(engine::CallAnswer::DelegationRefused(problem)) if problem.why == tasks::Refusal::Inputs));
    assert_eq!(driver.store.header().tasks, parent.task);
}

#[test]
fn a_call_asked_twice_across_a_restart_is_decided_once() {
    let (mut driver, assignment) = running_fixture();
    unavailable_call(&mut driver, &assignment, 71);
    assert_eq!(driver.answered(71), None, "the answer waits for its named decision to commit");
    driver.settle();
    let key = temper_engine_domain::CallKey {
        task: assignment.task,
        attempt: assignment.attempt,
        completion: 2,
        position: 0,
    };
    assert!(matches!(
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))),
        Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record)))) if record.part == jig_domain_core::CallPart::Unavailable
    ));
    // A call is the hub's relay now: its decision and its settled answer
    // each take a call number (D15; jig's `domain/hosts.md`, 6.4).
    assert_eq!(driver.store.header().calls, 2);
    assert_eq!(driver.answered(71), Some(&engine::CallAnswer::Unavailable));
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::Link { channel, down: host::LinkDown::Relayed { run, attempt, call, .. } }
            if *channel == Token::new(7) && run.raw() == assignment.task && attempt.raw() == assignment.attempt
                && call.as_ref() == 71_u64.to_be_bytes()
    )));
    let mut restarted = Driver::new(driver.store);
    let hosting = protocol::hosting(assignment.task, assignment.attempt, host::Phase::Active);
    restarted.send(hello_with(Token::new(7), 1, Duration::from_secs(1), Box::new([hosting])));
    restarted.settle();
    unavailable_call(&mut restarted, &assignment, 72);
    restarted.settle();
    assert_eq!(restarted.store.header().calls, 2, "the repeated name did not make another decision");
    // The repeat is answered from the first call's settled answer, the
    // world protocol's text for it, without a render (D15).
    assert_eq!(restarted.answered(72), None);
    assert!(downs(&restarted).into_iter().any(|down| matches!(down,
        host::LinkDown::Relayed { call, answer: host::Relayed::Answer { error: false, text }, .. }
            if call.as_ref() == 72_u64.to_be_bytes() && text.as_ref() == 71_u64.to_be_bytes()
    )));
}

#[test]
fn a_lost_attempt_is_told_of_the_calls_committed_after_its_last_turn() {
    let (mut driver, first) = running_fixture();
    unavailable_call(&mut driver, &first, 81);
    driver.settle();
    driver.send(protocol::lost(Token::new(7)));
    driver.env.now = Time::from_nanos(Duration::from_secs(6).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(6).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    driver.env.now = Time::from_nanos(Duration::from_secs(7).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(7).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.hello(Token::new(8));
    driver.settle();
    let second = driver.last_assignment();
    assert!(second.attempt > first.attempt);
    let key = temper_engine_domain::CallKey { task: first.task, attempt: first.attempt, completion: 2, position: 0 };
    assert_eq!(second.answered.len(), 1);
    assert_eq!((second.answered[0].key, &second.answered[0].answer), (key, &engine::CallAnswer::Unavailable));
}

// A full journal hands the call back to the world instead of answering it
// busy; the world offers it again once there is room (D6; jig's
// `domain/root.md`, 3).
#[test]
fn calls_under_backpressure_are_handed_back_and_decided_once_on_retry() {
    let (mut driver, assignment) = running_fixture();
    driver.sign_in();
    driver.sign_in();
    driver.sign_in();
    assert_eq!(driver.store.pending.len(), 3, "three issued commits fill the journal");
    let returned = driver.returned;
    unavailable_call(&mut driver, &assignment, 82);
    assert_eq!(driver.returned, returned + 1);
    assert_eq!(driver.store.header().calls, 0);
    assert!(
        !driver
            .store
            .rows
            .keys()
            .any(|key| matches!(key, Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(_)))))
    );
    assert_eq!(driver.answered(82), None, "the handed-back call is not decided");
    driver.settle();
    // The decision and its settled answer each take a call number (D15).
    assert_eq!(driver.store.header().calls, 2);
    assert_eq!(driver.answered(82), Some(&engine::CallAnswer::Unavailable));
}

#[test]
fn invalid_current_proof_stops_before_any_restored_closing_effect_or_result() {
    let (driver, assignment) = running_fixture();
    for corruption in 0..5 {
        let mut store = driver.store.fork();
        let mut rows = store.rows.clone();
        let key = temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task)));
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) = rows.get_mut(&key) else {
            panic!("live task");
        };
        task.last_answer = Some(assignment.attempt);
        task.phase = tasks::Phase::Closing(tasks::Closing {
            stage: tasks::Stage::Effects,
            ending: tasks::Ending::Done(tasks::TaskResult::Report { words: REPORT.into() }),
        });
        let proof_key = temper_engine_domain::Key::Core(jig_domain_core::Key::Core(
            jig_domain_core::CoreKey::RunProof(assignment.task),
        ));
        let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
            rows.get_mut(&proof_key)
        else {
            panic!("proof");
        };
        proof.terminal = Some(temper_engine_domain::TerminalRecord {
            task: assignment.task,
            attempt: assignment.attempt,
            cumulative: 3,
            end: tasks::End::Finished {
                result: tasks::TaskResult::Report { words: REPORT.into() },
                cancel_delegates: false,
            },
        });
        match corruption {
            0 => {
                rows.remove(&proof_key);
            }
            1 => {
                let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
                    rows.get_mut(&proof_key)
                else {
                    panic!("proof");
                };
                proof.turn.as_mut().expect("kept turn").cumulative = 999;
            }
            2 => {
                let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
                    rows.get_mut(&proof_key)
                else {
                    panic!("proof");
                };
                proof.terminal = Some(temper_engine_domain::TerminalRecord {
                    task: assignment.task,
                    attempt: assignment.attempt,
                    cumulative: 999,
                    end: tasks::End::Parked,
                });
            }
            3 => {
                let header = store.header();
                let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) = rows.get_mut(&key)
                else {
                    panic!("task");
                };
                task.attempt = header.runs + 1;
                task.last_answer = Some(task.attempt);
                let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
                    rows.get_mut(&proof_key)
                else {
                    panic!("proof");
                };
                proof.attempt = header.runs + 1;
            }
            4 => {
                let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
                    rows.get_mut(&proof_key)
                else {
                    panic!("proof");
                };
                proof.terminal = None;
            }
            _ => unreachable!(),
        }
        let erases = store.rows.keys().filter(|key| !rows.contains_key(*key)).cloned().collect();
        store.seed(rows.into_values().collect(), erases);
        let header = store.header();
        let before = store.rows.clone();
        let mut restarted = Driver::new(store);
        for _ in 0..100 {
            restarted.advance(true);
        }
        assert!(restarted.stopped, "corruption {corruption}");
        assert_eq!(restarted.store.rows, before, "no child restoration consequences committed");
        assert_eq!(restarted.store.header(), header);
        assert!(restarted.delivered.is_empty(), "no cancellation/result before root proof validation");
    }
}

#[test]
fn root_restore_refuses_malformed_task_shapes() {
    let (driver, assignment) = running_fixture();
    for malformed in 0..3 {
        let mut store = driver.store.fork();
        let mut rows = store.rows.clone();
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) = rows
            .get_mut(&temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task))))
        else {
            panic!("task");
        };
        match malformed {
            0 => task.requester = tasks::Party::Task(assignment.task),
            1 => task.executor = tasks::Executor::Agent { charter: 99 },
            2 => task.contract = tasks::Contract::Verdict { choices: Box::new([]) },
            _ => unreachable!(),
        }
        let erases = store.rows.keys().filter(|key| !rows.contains_key(*key)).cloned().collect();
        store.seed(rows.into_values().collect(), erases);
        let before = store.rows.clone();
        let mut restarted = Driver::new(store);
        for _ in 0..100 {
            restarted.advance(true);
        }
        assert!(restarted.stopped);
        assert_eq!(restarted.store.rows, before);
        assert!(restarted.delivered.is_empty());
    }
}

#[test]
fn bounded_invalid_typed_terminal_preserves_original_root_evidence_and_charges_once() {
    let (mut driver, assignment) = running_fixture();
    let end = tasks::End::Finished {
        result: tasks::TaskResult::Verdict {
            code: 99,
            fields: Box::new([]),
            words: b"invalid contract".as_slice().into(),
        },
        cancel_delegates: false,
    };
    driver.answer(Token::new(7), assignment.task, assignment.attempt, 8, end.clone(), engine::Parts::none());
    driver.settle();
    let key = temper_engine_domain::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Terminal {
        task: assignment.task,
        attempt: assignment.attempt,
    }));
    assert_eq!(
        driver.store.rows.get(&key),
        Some(&Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Terminal(
            temper_engine_domain::TerminalRecord {
                task: assignment.task,
                attempt: assignment.attempt,
                cumulative: 8,
                end: end.clone()
            }
        ))))
    );
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) = driver
        .store
        .rows
        .get(&temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task))))
    else {
        panic!("retryable invalid task");
    };
    assert_eq!(task.run_spent, 8);
    assert_eq!(task.numbers.spent, 8);
    assert_eq!(task.tries.invalid, 1);
    let rows = driver.store.rows.clone();
    driver.answer(Token::new(7), assignment.task, assignment.attempt, 8, end, engine::Parts::none());
    driver.settle();
    assert_eq!(driver.store.rows, rows, "fenced duplicate never reaches child priced admission");
}

fn escalation_archive_driver() -> (Driver, temper_engine_domain::EscalationDecisionRecord) {
    use temper_engine_domain_world::{escalation, escalation_referee::Story};
    let mut world = escalation::World::new(escalation::Settings::calm(9200, Story::Release));
    world.run();
    let archive = world
        .store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::EscalationDecision(
                archive,
            ))) = row
            {
                Some(archive.clone())
            } else {
                None
            }
        })
        .expect("actual completed escalation archive");
    let configured = escalation::limits();
    let mut driver = Driver::configured(world.store, config(9200), &configured);
    driver.settle();
    (driver, archive)
}

#[test]
fn failed_named_escalation_history_read_closes_once_and_same_key_retries() {
    let (mut driver, archive) = escalation_archive_driver();
    let rows = driver.store.rows.clone();
    let session = driver.store.header().sign_ins;
    driver.delivered.clear();
    driver.fail_archive_once = true;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(900)),
        sign_in: session,
        key: [9; 16],
        ask: people::Ask::DecideEscalation {
            project: archive.project,
            task: archive.task,
            revision: archive.revision,
            decision: people::EscalationDecision::Release,
        },
    }));
    driver.settle();
    assert!(!driver.stopped);
    assert!(driver.root.quiescent());
    assert_eq!(driver.store.rows, rows, "failed IO saves no keyed answer or task/financial change");
    assert_eq!(driver.delivered.len(), 1);
    // A retryable answer is Again, outside the final refusals (jig b7bdd9e8).
    assert!(matches!(
        &driver.delivered[0],
        Delivery::WebReply { reply: people::Reply::Again(people::Again::Busy), .. }
    ));
    driver.delivered.clear();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(901)),
        sign_in: session,
        key: [9; 16],
        ask: people::Ask::DecideEscalation {
            project: archive.project,
            task: archive.task,
            revision: archive.revision,
            decision: people::EscalationDecision::Release,
        },
    }));
    driver.settle();
    assert_eq!(driver.delivered.len(), 1);
    assert!(
        matches!(&driver.delivered[0], Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::EscalationDecided { by, .. }), .. } if *by == archive.by)
    );
    assert!(driver.root.quiescent(), "shared read and people's pending slots retired");
}

fn held_waiting_store() -> Store {
    use temper_engine_domain_world::{escalation, escalation_referee::Story};
    let mut world = escalation::World::new(escalation::Settings::calm(9202, Story::Reject));
    for _ in 0..300 {
        world.iterate();
        if world.store.rows.values().any(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) if matches!(task.escalation, tasks::Escalation::Waiting { .. }))) {
            return world.store.fork();
        }
    }
    panic!("actual failure never committed Waiting");
}

fn held_driver(store: Store) -> Driver {
    let configured = temper_engine_domain_world::escalation::limits();
    Driver::configured(store, config(9202), &configured)
}

#[test]
fn release_rejudges_a_still_expired_deadline_and_escalates_the_new_hold() {
    let mut store = held_waiting_store();
    let mut task_number = 0;
    let mut requester = 0;
    let mut rows = store.rows.clone();
    for row in rows.values_mut() {
        if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) = row {
            task_number = task.number;
            let tasks::Party::Person(person) = task.requester else { unreachable!() };
            requester = person;
            task.phase = tasks::Phase::Held {
                was: tasks::Was::Active(tasks::Active::Due),
                why: tasks::Hold::Deadline,
                source: None,
            };
            task.authority.budget.deadline = Some(Wall::EPOCH);
        }
    }
    let sign_in = store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { number, person, .. })) = row
                && *person == requester
            {
                Some(*number)
            } else {
                None
            }
        })
        .expect("requester session");
    store.seed(rows.into_values().collect(), Box::new([]));
    let mut driver = held_driver(store);
    driver.settle();
    driver.env.wall = Wall::from_nanos(1);
    driver.delivered.clear();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(990)),
        sign_in,
        key: [90; 16],
        ask: people::Ask::DecideEscalation {
            project: 1,
            task: task_number,
            revision: 1,
            decision: people::EscalationDecision::Release,
        },
    }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::EscalationDecided {
                    choice: people::EscalationChoice::Released,
                    ..
                }),
                ..
            }
        )),
        "{:?}",
        driver.delivered
    );
    assert!(driver.store.rows.values().any(
        |row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
        if task.number == task_number && matches!(task.phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. })
            && matches!(task.escalation, tasks::Escalation::Waiting { revision: 2, .. }))
    ));
}

#[test]
fn restored_waiting_rechecks_snapshot_membership_and_same_holder_changes_nothing() {
    let store = held_waiting_store();
    let rows = store.rows.clone();
    let mut same = held_driver(store);
    same.settle();
    assert!(!same.stopped);
    assert!(same.root.quiescent());
    assert_eq!(same.store.rows, rows, "same holder preserves revision without another commit");
    let mut store = held_waiting_store();
    let requester = store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) = row
                && let tasks::Party::Person(person) = task.requester
            {
                return Some(person);
            }
            None
        })
        .expect("person chat");
    let mut rows = store.rows.clone();
    for row in rows.values_mut() {
        if let Record::Core(jig_domain_core::Record::People(people::Stored::Roles { holdings, .. })) = row {
            *holdings = holdings
                .iter()
                .copied()
                .filter(|holding| holding.person != requester)
                .collect::<Vec<_>>()
                .into_boxed_slice();
        }
    }
    store.seed(rows.into_values().collect(), Box::new([]));
    let mut changed = held_driver(store);
    changed.settle();
    assert!(!changed.stopped);
    // Rerouting keeps the hold's revision (jig 00de0b8c).
    assert!(changed.store.rows.values().any(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) if matches!(task.escalation, tasks::Escalation::Waiting { revision: 1, holder: tasks::EscalationHolder::Role { project: 1, role: 0 }, .. }))));
    assert!(
        !changed
            .delivered
            .iter()
            .any(|delivery| matches!(delivery, Delivery::Assigned { .. } | Delivery::Result { .. }))
    );
}

// The escalation reads that followed the restart are the core's now (jig
// 62410d35); the restart's own promise stays.
#[test]
fn a_rejected_hold_stays_rejected_across_a_restart() {
    use temper_engine_domain_world::{escalation, escalation_referee::Story};
    let mut world = escalation::World::new(escalation::Settings::calm(9203, Story::Reject));
    world.run();
    assert!(world.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
            if matches!(task.escalation, tasks::Escalation::Rejected { .. })
    )));
    let before = world.store.rows.clone();
    let mut configured = escalation::limits();
    configured.people.people = 3;
    configured.people.sign_ins = 3;
    let mut driver = Driver::configured(world.store, config(9203), &configured);
    driver.settle();
    assert_eq!(driver.store.rows, before, "rejection never reopens or reroutes at startup");
    assert!(driver.delivered.is_empty());
}

#[test]
#[expect(clippy::too_many_lines, reason = "one history replay story retains its full script")]
fn coalesced_history_waiters_survive_simultaneous_io_completion_under_full_journal() {
    use temper_engine_domain_world::{escalation, escalation_referee::Story};
    let mut world = escalation::World::new(escalation::Settings::calm(9204, Story::Release));
    world.run();
    let archive = world
        .store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::EscalationDecision(
                archive,
            ))) = row
            {
                Some(archive.clone())
            } else {
                None
            }
        })
        .expect("actual immutable decision");
    let mut configured = escalation::limits();
    configured.people.waiters = 8;
    configured.journal.writes = tasks::max_out(&configured.tasks) * 8
        + people::max_out(&configured.people) * 4
        + configured.people.pending * 2
        + fleet::max_out(&configured.fleet) * 4
        + temper_engine_domain_forge::max_out(&configured.forge)
        + configured.tasks.tasks
        + 6;
    configured.journal.deliveries =
        configured.tasks.tasks * 4 + 8 + configured.people.pending * configured.people.waiters;
    configured.journal.held = configured.journal.deliveries * 3 + 16;
    let mut insufficient = configured;
    insufficient.journal.deliveries -= 1;
    assert_eq!(engine::worst_case(&insufficient), None, "startup prices every pending flight's duplicate waiters");
    let session = world.store.header().sign_ins;
    let mut driver = Driver::configured(world.store, config(9204), &configured);
    driver.settle();
    for key in 9_u8..11 {
        for waiter in 0_u64..8 {
            driver.send(engine::Event::Party(engine::PartyInput::Ask {
                reply_to: ReplyTo::new(Token::new(950 + u64::from(key - 9) * 8 + waiter)),
                sign_in: session,
                key: [key; 16],
                ask: people::Ask::DecideEscalation {
                    project: archive.project,
                    task: archive.task,
                    revision: archive.revision,
                    decision: people::EscalationDecision::Release,
                },
            }));
        }
    }
    for _ in 0..40 {
        if driver.events.len() == 2
            && driver
                .events
                .iter()
                .all(|event| matches!(event, engine::Event::Store(engine::StoreInput::Loaded { .. })))
        {
            break;
        }
        if let Some(index) = driver.events.iter().position(|event| matches!(event, engine::Event::Released(_))) {
            let event = driver.events.remove(index).expect("released continuation");
            driver.send(event);
        } else {
            engine::release(&mut driver.root, &driver.env, &mut driver.out);
            driver.collect();
        }
        driver.root.reclaim();
    }
    assert_eq!(driver.events.len(), 2, "one real named IO for each coalesced flight");
    driver.sign_in();
    driver.sign_in();
    driver.sign_in();
    assert_eq!(driver.store.pending.len(), 3, "all store commit slots full before IO completes");
    driver.advance(false);
    driver.advance(false);
    // Store answers are always taken (D6; jig's `domain/root.md`, 3).
    assert!(!driver.events.iter().any(|event| matches!(event, engine::Event::Store(_))), "both loads were taken");
    assert!(!driver.root.quiescent());
    assert!(
        !driver.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::EscalationDecided { .. }), .. }
        )),
        "no waiter reply escapes pressure before its keyed outcome can commit"
    );
    driver.settle();
    let replies: Vec<_> = std::mem::take(&mut driver.delivered)
        .into_iter()
        .filter_map(|delivery| {
            if let Delivery::WebReply {
                to,
                reply: people::Reply::Outcome(people::Outcome::EscalationDecided { by, .. }),
            } = delivery
            {
                Some((to.into_token().raw(), by))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(replies.len(), 16);
    let mut rights = std::collections::BTreeSet::new();
    for (to, by) in replies {
        assert!((950..966).contains(&to));
        assert!(rights.insert(to), "each admitted waiter ends once");
        assert_eq!(by, archive.by);
    }
    assert!(driver.root.quiescent(), "all query/pending/store obligations retired");
}

#[test]
fn restored_unreported_claim_spends_no_try_without_committed_work() {
    for kept_turn in [false, true] {
        let mut original = Driver::new(Store::new());
        original.hello(Token::new(7));
        original.settle();
        original.sign_in();
        original.settle();
        original.send(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: ReplyTo::new(Token::new(501)),
            sign_in: original.session(),
            key: [18; 16],
            ask: people::Ask::StartChat { project: 1, words: QUESTION.into() },
        }));
        original.settle();
        let assignment = original
            .delivered
            .iter()
            .find_map(|delivery| {
                if let Delivery::Assigned { assignment, .. } = delivery { Some(assignment.clone()) } else { None }
            })
            .expect("actual worker assignment before loss");
        if kept_turn {
            turn(&mut original, &assignment, 1, 3);
            original.settle();
            assert!(
                downs(&original)
                    .into_iter()
                    .any(|down| matches!(down, host::LinkDown::TurnKept { run, attempt, turn: 1 }
                if run.raw() == assignment.task && attempt.raw() == assignment.attempt))
            );
        }
        let ledgers: Vec<_> = original
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(ledger))) = row {
                    Some(*ledger)
                } else {
                    None
                }
            })
            .collect();
        let mut restored = Driver::new(original.store);
        for _ in 0..200 {
            restored.advance(true);
            if restored.root.ready() && restored.events.is_empty() && restored.store.pending.is_empty() {
                break;
            }
        }
        assert!(restored.root.ready() && restored.events.is_empty() && restored.store.pending.is_empty());
        // A restored claim waits for fleet grace; it cannot be quiescent yet.
        restored.env.now = Time::from_nanos(6_000_000_000);
        restored.env.wall = Wall::from_nanos(6_000_000_000);
        engine::fire(&mut restored.root, &restored.env);
        engine::release(&mut restored.root, &restored.env, &mut restored.out);
        restored.collect();
        let end = if kept_turn { tasks::End::Failed(tasks::Class::Lost) } else { tasks::End::Refused };
        let cumulative = if kept_turn { 3 } else { 0 };
        assert_eq!(restored.store.pending.len(), 1, "one canonical loss transaction");
        let writes = &restored.store.pending.front().expect("loss transaction").1;
        let canonical = writes
            .iter()
            .find_map(|write| {
                if let Write::Save(Record::Core(jig_domain_core::Record::Core(
                    jig_domain_core::CoreRecord::Terminal(terminal),
                ))) = write
                {
                    Some(terminal)
                } else {
                    None
                }
            })
            .expect("loss commits actual canonical terminal");
        assert_eq!(
            (canonical.task, canonical.attempt, canonical.cumulative),
            (assignment.task, assignment.attempt, cumulative),
        );
        assert_eq!(canonical.end, end);
        assert!(writes.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) if proof.task == assignment.task && proof.attempt == assignment.attempt && proof.terminal.as_ref() == Some(canonical))), "canonical terminal and proof share one commit");
        assert!(writes.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) if task.number == assignment.task && task.numbers.spent == cumulative && task.run_spent == cumulative && task.last_answer == Some(assignment.attempt) && task.tries.lost == u32::from(kept_turn))), "canonical terminal and classified try share one commit");
        restored.settle();
        let after: Vec<_> = restored
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(ledger))) = row {
                    Some(*ledger)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(after, ledgers, "topology loss never charges or posts funding again");
        assert_eq!(acknowledged(&restored), 0, "no worker terminal was offered after restart");
    }
}

fn administration_config(seed: u64) -> engine::Config {
    use jig_domain_authority as authority;
    let mut config = config(seed);
    let mut policy = config.core.authority.policy(1).expect("actual project").clone();
    policy.roles[0].requests = authority::Requests(1 | 4 | 256);
    let mut findings = Queue::with_capacity(authority::POLICY_MAX_OUT);
    authority::step(&mut config.core.authority, authority::Event::Policy { project: 1, policy }, &mut findings);
    assert_eq!(findings.pop(), Some(authority::PolicyFact::Changed { project: 1 }));
    config
}

#[test]
fn an_owner_makes_a_service_once_and_the_service_cannot_start_a_chat() {
    let mut driver = Driver::configured(Store::new(), administration_config(9501), &limits());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let owner_session = driver.session();
    let ask = people::Ask::MakeService { project: 1, name: b"builder".as_slice().into(), role: people::Role::Owner };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9502)),
        sign_in: owner_session,
        key: [95; 16],
        ask: ask.clone(),
    }));
    driver.settle();
    let service = driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::ServiceMade { person }), ..
            } = delivery
            {
                Some(*person)
            } else {
                None
            }
        })
        .expect("service creation answered after commit");
    assert!(driver.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::People(people::Stored::Person { number, identity }))
            if *number == service && identity.key.provider == 1 && identity.key.subject.as_ref() == service.to_be_bytes()
    )));
    let created = driver.store.header().people;
    let mut restarted = Driver::configured(driver.store, administration_config(9501), &limits());
    restarted.settle();
    restarted.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9503)),
        sign_in: owner_session,
        key: [95; 16],
        ask,
    }));
    restarted.settle();
    assert_eq!(restarted.store.header().people, created);
    assert!(restarted.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ServiceMade { person }), .. }
            if *person == service
    )));
    restarted.send(engine::Event::Party(engine::PartyInput::SignedIn {
        reply_to: ReplyTo::new(Token::new(9504)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 1, subject: service.to_be_bytes().into() },
            login: Box::new([]),
            name: b"builder".as_slice().into(),
        },
    }));
    restarted.settle();
    let service_session = restarted
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply { reply: people::Reply::SignedIn { person, sign_in, .. }, .. } = delivery
                && *person == service
            {
                Some(*sign_in)
            } else {
                None
            }
        })
        .expect("service sign-in uses its existing party");
    restarted.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9505)),
        sign_in: service_session,
        key: [96; 16],
        ask: people::Ask::StartChat { project: 1, words: Box::new([]) },
    }));
    restarted.settle();
    // Role refusals carry the role held (jig b7bdd9e8).
    let refused = people::Refusal::Role { held: Some(people::Role::Owner) };
    assert!(replied(&restarted, &people::Outcome::Refused(refused)));
}

// A chat's authority is its role's at the time it starts (jig a2616181): a
// later chat takes the changed role's tools, an earlier one keeps its grant.
#[test]
fn a_policy_change_applies_to_later_decisions_only() {
    let mut driver = Driver::configured(Store::new(), administration_config(9401), &limits());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let session = driver.session();
    let start = |key: u8, reply: u64, words: &[u8]| {
        let ask = people::Ask::StartChat { project: 1, words: words.into() };
        engine::Event::Party(engine::PartyInput::Ask {
            reply_to: ReplyTo::new(Token::new(reply)),
            sign_in: session,
            key: [key; 16],
            ask,
        })
    };
    driver.send(start(41, 9402, b"first"));
    for _ in 0..30 {
        driver.advance(true);
    }
    let first = live(&driver, 1).authority.clone();
    let role = people::PolicyRole {
        number: 0,
        authority: people::Authority {
            tools: 0,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([people::Executor::Charter(1)]), tasks: 2, depth: 1 },
            spend: 500,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        period_spend: 50,
        requests: 1 | 4 | 256,
        decides: authority::Proposals::ALL.0,
    };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9403)),
        sign_in: session,
        key: [42; 16],
        ask: people::Ask::ChangePolicy { project: 1, change: people::PolicyChange::Role(role) },
    }));
    for _ in 0..30 {
        driver.advance(true);
    }
    assert!(matches!(
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::Policy(1)))),
        Some(Record::Core(jig_domain_core::Record::People(people::Stored::Policy { project: 1, value }))) if value.roles[0].period_spend == 50
    ));
    assert_eq!(live(&driver, 1).authority, first, "existing task keeps its grant");
    let mut restarted = Driver::configured(driver.store, administration_config(9401), &limits());
    for _ in 0..30 {
        restarted.advance(true);
    }
    restarted.send(start(43, 9404, b"later"));
    for _ in 0..30 {
        restarted.advance(true);
    }
    assert!(replied(&restarted, &people::Outcome::Started { task: 2 }));
    assert_eq!(live(&restarted, 2).authority.tools, tasks::Tools(0), "restored policy governs later decisions");
    assert_eq!(first.tools, tasks::Tools(1023));
}

// The first start saves the configured policy (jig 0f55ef2d); a refused
// change leaves it as it was.
#[test]
fn a_policy_change_beyond_the_deployments_rules_is_refused() {
    let mut driver = Driver::configured(Store::new(), administration_config(9411), &limits());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let policy = Key::Core(jig_domain_core::Key::People(people::Key::Policy(1)));
    let before = driver.store.rows.get(&policy).cloned();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9412)),
        sign_in: driver.session(),
        key: [44; 16],
        ask: people::Ask::ChangePolicy {
            project: 1,
            change: people::PolicyChange::ProjectSpend { period_spend: 1001 },
        },
    }));
    driver.settle();
    // Authority refusals carry what is lacking (jig b7bdd9e8): the deployment's
    // rules refuse the policy's shape.
    let refused = people::Refusal::Authority { lacking: people::Lack::Shape, proposable: None };
    assert!(replied(&driver, &people::Outcome::Refused(refused)));
    assert!(before.is_some());
    assert_eq!(driver.store.rows.get(&policy), before.as_ref());
}

#[test]
fn a_policy_change_survives_a_restart() {
    let mut driver = Driver::configured(Store::new(), administration_config(9413), &limits());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let session = driver.session();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9414)),
        sign_in: session,
        key: [45; 16],
        ask: people::Ask::ChangePolicy { project: 1, change: people::PolicyChange::ProjectSpend { period_spend: 800 } },
    }));
    driver.settle();
    assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::Policy(1)))),
        Some(Record::Core(jig_domain_core::Record::People(people::Stored::Policy { value, .. }))) if value.period_spend == 800));
    let mut restarted = Driver::configured(driver.store, administration_config(9413), &limits());
    restarted.settle();
    restarted.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9415)),
        sign_in: session,
        key: [46; 16],
        ask: people::Ask::StartChat { project: 1, words: b"later".as_slice().into() },
    }));
    restarted.settle();
    // Authority refusals carry what is lacking (jig b7bdd9e8): the period's
    // budget is past the restored project spend.
    let lacking = people::Lack::Budget { needs: 1000, left: 800 };
    assert!(replied(&restarted, &people::Outcome::Refused(people::Refusal::Authority { lacking, proposable: None })));
}

#[test]
fn an_owner_changes_a_pool_once_with_a_key() {
    let mut driver = Driver::configured(Store::new(), administration_config(9411), &limits());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let session = driver.session();
    let person = signed_in(&driver, 101).0;
    let ask = people::Ask::SetPool { project: 1, person, budget: 300 };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9412)),
        sign_in: session,
        key: [44; 16],
        ask: ask.clone(),
    }));
    driver.settle();
    let pool = tasks::Funder::Pool { project: 1, person, period: 1 };
    assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(pool)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(row)))) if row.numbers.budget == 300));
    let before = driver.transactions.len();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9413)),
        sign_in: session,
        key: [44; 16],
        ask,
    }));
    driver.settle();
    assert_eq!(driver.transactions.len(), before, "saved answer does not carve again");
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(9414)),
        sign_in: session,
        key: [45; 16],
        ask: people::Ask::SetPool { project: 1, person, budget: 250 },
    }));
    driver.settle();
    let period = tasks::Funder::Period { project: 1, period: 1 };
    assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(pool)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(row)))) if row.numbers.budget == 250));
    assert!(matches!(driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(period)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(row)))) if row.numbers.reserved == 250));
}

/// Two chats of one requester, both held and waiting at that requester.
fn held_cohort() -> (Driver, [u64; 2], [u64; 2], Vec<tasks::TaskRecord>) {
    use temper_engine_domain_world::roles;
    let prior = roles::World::new(roles::Settings::calm(9310, roles::Base::Requester));
    let (sessions, persons) = (prior.sessions, prior.people);
    let mut configured = roles::limits();
    configured.people.sign_ins = 8;
    configured.journal.writes = tasks::max_out(&configured.tasks) * 8
        + people::max_out(&configured.people) * 4
        + configured.people.pending * 2
        + fleet::max_out(&configured.fleet) * 4
        + temper_engine_domain_forge::max_out(&configured.forge)
        + configured.tasks.tasks
        + 6;
    let mut driver = Driver::configured(prior.store, administration_config(9310), &configured);
    driver.settle();
    // The held chat took its person's whole pool, so a second chat of theirs
    // is held at its start and waits at them too (jig a2616181).
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(1200)),
        sign_in: sessions[0],
        key: [120; 16],
        ask: people::Ask::StartChat { project: 1, words: QUESTION.into() },
    }));
    driver.settle();
    let original: Vec<_> = driver
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(record))) = row {
                Some(record.as_ref().clone())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(original.len(), 2, "two genuine held task admissions");
    for record in &original {
        assert!(matches!(
            record.escalation,
            tasks::Escalation::Waiting { revision: 1, holder: tasks::EscalationHolder::Person(person), .. }
                if person == persons[0]
        ));
    }
    (driver, sessions, persons, original)
}

// A full journal hands the ask back to the world, saving no keyed answer
// (D6; jig's `domain/root.md`, 3); the world offers it again once there is
// room. Rerouting keeps each hold's revision (jig 00de0b8c), so a roster
// change cannot exhaust one: that overflow case goes with the mechanism.
#[test]
fn multiple_waiting_recipients_preflight_together_and_a_full_journal_hands_the_ask_back() {
    let (mut driver, sessions, persons, original) = held_cohort();
    let owner = [people::Holding { person: persons[1], role: people::Role::Owner }];
    let ask = people::Ask::SetRoles { project: 1, holdings: Box::new(owner) };
    driver.sign_in();
    driver.sign_in();
    driver.sign_in();
    assert_eq!(driver.store.pending.len(), 3, "real issued journal pressure");
    let (returned, transactions) = (driver.returned, driver.transactions.len());
    let to = ReplyTo::new(Token::new(1201));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: to,
        sign_in: sessions[1],
        key: [121; 16],
        ask,
    }));
    assert_eq!(driver.returned, returned + 1);
    let answer_key = people::Key::Answer(people::RequestKey { person: persons[1], key: [121; 16] });
    assert!(!driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::People(answer_key))), "nothing saved");
    driver.settle();
    let writes = driver.transactions[transactions..]
        .iter()
        .find(|writes| {
            writes.iter().any(|write| {
                matches!(
                    write,
                    Write::Save(Record::Core(jig_domain_core::Record::People(people::Stored::Roles { .. })))
                )
            })
        })
        .expect("one serialized role decision");
    assert!(writes.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::People(people::Stored::Roles { holdings, .. }))) if holdings.as_ref() == owner)));
    assert!(writes.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::People(people::Stored::Answer { key, outcome: people::Outcome::RolesSet { project: 1 }, .. }))) if key.person == persons[1] && key.key == [121;16])));
    for record in &original {
        let tasks::Escalation::Waiting { entry: old_entry, .. } = record.escalation else {
            panic!("source is waiting")
        };
        let actual = writes
            .iter()
            .find_map(|write| match write {
                Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(actual))))
                    if actual.number == record.number =>
                {
                    Some(actual)
                }
                Write::Save(_) | Write::Erase(_) => None,
            })
            .expect("each exact affected task belongs to the same cohort");
        // Rerouting keeps the hold's revision (jig 00de0b8c).
        assert!(matches!(actual.escalation, tasks::Escalation::Waiting {
            revision: 1, holder: tasks::EscalationHolder::Role { project: 1, role: 0 }, entry, ..
        } if entry > old_entry));
        let mut expected = record.clone();
        expected.escalation = actual.escalation.clone();
        expected.changed = actual.changed;
        assert_eq!(**actual, expected);
    }
    assert!(
        !writes.iter().any(|write| matches!(
            write,
            Write::Save(
                Record::Core(
                    jig_domain_core::Record::Tasks(tasks::Stored::Ledger(_))
                        | jig_domain_core::Record::Core(
                            jig_domain_core::CoreRecord::RunProof(_)
                                | jig_domain_core::CoreRecord::Terminal(_)
                                | jig_domain_core::CoreRecord::Turn(_)
                                | jig_domain_core::CoreRecord::EscalationDecision(_)
                                | jig_domain_core::CoreRecord::ProposalDecision(_)
                        )
                ) | Record::Forge { .. }
            )
        )),
        "role change never rewrites economic or accepted-work evidence"
    );
}

#[test]
fn read_only_role_stages_have_one_not_ready_terminal_and_empty_projects_need_no_financial_ledger() {
    let mut insufficient = temper_engine_domain_world::roles::limits();
    assert!(engine::worst_case(&insufficient).is_some(), "valid role cascade factory");
    insufficient.journal.writes -= 1;
    assert!(
        engine::worst_case(&insufficient).is_none(),
        "exact cross-route room includes the complete role handoff cohort"
    );
    let mut oversized = temper_engine_domain_world::roles::limits();
    oversized.people.holdings = oversized.journal.transcript_bytes;
    assert!(engine::worst_case(&oversized).is_none(), "one roster-bearing durable row must fit before any clone");
    let configured = limits();
    let mut child = tasks::Domain::new(&configured.tasks, 9312, Box::new([1]));
    let environment = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: configured.tasks };
    let mut out = Queue::with_capacity(tasks::max_out(&configured.tasks));
    let right = |request| ReplyTo::new(Token::new(request));
    tasks::step(
        &mut child,
        &environment,
        tasks::Event::InspectEscalations { reply_to: right(1300), project: 1 },
        &mut out,
    );
    let inspected =
        tasks::Request::EscalationsInspected { reply_to: right(1300), result: Err(tasks::Refusal::NotReady) };
    assert_eq!((out.len(), out.pop()), (1, Some(inspected)), "no mutation or lost startup terminal");
    tasks::step(
        &mut child,
        &environment,
        tasks::Event::RecheckEscalations { reply_to: right(1301), project: 1 },
        &mut out,
    );
    let rechecked =
        tasks::Request::EscalationsRechecked { reply_to: right(1301), result: Err(tasks::Refusal::NotReady) };
    assert_eq!((out.len(), out.pop()), (1, Some(rechecked)), "no mutation or lost startup terminal");
    tasks::step(&mut child, &environment, tasks::Event::Restored, &mut out);
    assert!(out.is_empty());
    tasks::step(
        &mut child,
        &environment,
        tasks::Event::InspectEscalations { reply_to: right(1302), project: 1 },
        &mut out,
    );
    let empty = tasks::Request::EscalationsInspected { reply_to: right(1302), result: Ok(Box::new([])) };
    assert_eq!(out.pop(), Some(empty), "one empty snapshot terminal");
}

/// A resumed run's turn, which continues its transcript's window.
fn resumed_turn(driver: &mut Driver, run: &engine::Assignment, number: u32, cumulative: u64, body: &[u8]) {
    let turn = host::Turn {
        opens_window: false,
        refused: Box::new([]),
        turn: number,
        spent: cumulative,
        read: None,
        body: body.into(),
    };
    let up = host::LinkUp::Turn { run: Token::new(run.task), attempt: Token::new(run.attempt), turn };
    driver.send(protocol::up(Token::new(7), up, engine::Parts::none()));
}

#[test]
fn words_typed_while_a_run_works_reach_it_once_committed() {
    let (mut driver, assignment) = Driver::chat();
    let number = driver.say(assignment.task, 41);
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::Link { channel, down: host::LinkDown::Message { run, attempt, name, words, .. } }
            if *channel == Token::new(7) && run.raw() == assignment.task && attempt.raw() == assignment.attempt
                && name.raw() == number && words.as_ref() == [41])));
    let task = live(&driver, assignment.task);
    assert_eq!(task.inbox.len(), 1);
    assert_eq!(task.inbox[0].number, number);
}

#[test]
fn a_read_fence_takes_only_what_the_run_read() {
    let (mut driver, assignment) = Driver::chat();
    let first = driver.say(assignment.task, 42);
    let second = driver.say(assignment.task, 43);
    driver.send(protocol::turn(Token::new(7), assignment.task, assignment.attempt, 1, 1, Some(first), b"read one"));
    driver.settle();
    let task = live(&driver, assignment.task);
    assert_eq!(task.inbox.len(), 1);
    assert_eq!(task.inbox[0].number, second);
    assert!(driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Turn {
        task: assignment.task,
        attempt: assignment.attempt,
        turn: 1
    }))));
}

// Waiting words travel in the run's start (jig 0f6041cb).
#[test]
fn words_to_a_parked_chat_wake_it() {
    let (mut driver, assignment) = Driver::chat();
    driver.answer(Token::new(7), assignment.task, assignment.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let number = driver.say(assignment.task, 44);
    let next = driver.last_assignment();
    assert!(next.attempt > assignment.attempt);
    assert_eq!(next.messages.len(), 1);
    assert_eq!(next.messages[0].number, number);
    assert_eq!(next.messages[0].words.as_ref(), [44]);
}

// A resumed run continues its transcript's window; a run's first turn opens
// one (jig c83ab353).
#[test]
fn a_chat_parks_and_resumes_from_its_transcript() {
    let (mut driver, first) = Driver::chat();
    driver.send(protocol::turn(Token::new(7), first.task, first.attempt, 1, 1, None, b"first"));
    driver.settle();
    driver.answer(Token::new(7), first.task, first.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(first.task, 45);
    let second = driver.last_assignment();
    assert_eq!(second.transcript.len(), 1);
    assert_eq!(second.transcript[0].as_ref(), b"first");
    resumed_turn(&mut driver, &second, 1, 1, b"second");
    driver.settle();
    driver.answer(Token::new(7), second.task, second.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(second.task, 46);
    let third = driver.last_assignment();
    assert_eq!(third.transcript.len(), 2);
    assert_eq!(third.transcript[0].as_ref(), b"first");
    assert_eq!(third.transcript[1].as_ref(), b"second");
    assert!(third.attempt > second.attempt);
}

#[test]
fn a_chat_past_the_resume_limit_starts_fresh_with_the_tail_in_its_brief() {
    let (mut driver, first) = Driver::chat();
    let mut body = vec![b'x'; 300];
    body[298..].copy_from_slice(b"yz");
    driver.send(protocol::turn(Token::new(7), first.task, first.attempt, 1, 1, None, &body));
    driver.settle();
    driver.answer(Token::new(7), first.task, first.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(first.task, 47);
    let next = driver.last_assignment();
    assert!(next.transcript.is_empty());
    let tail = next
        .sections
        .iter()
        .find_map(|section| match &section.body {
            engine::BriefBody::Text(bytes) if section.kind == engine::BriefKind::Core(brief::Core::TranscriptTail) => {
                Some(bytes)
            }
            engine::BriefBody::Text(_) | engine::BriefBody::Missing(_) => None,
        })
        .expect("bounded transcript tail section");
    assert!(tail.ends_with(b"yz"), "the newest conversation bytes survive the cut");
    assert!(tail.len() <= 128, "tail fits its section budget");
}

#[test]
fn a_run_failing_transiently_is_retried_then_held_past_its_tries() {
    let (mut driver, first) = Driver::chat();
    let transient = tasks::End::Failed(tasks::Class::Transient);
    driver.answer(Token::new(7), first.task, first.attempt, 0, transient.clone(), engine::Parts::none());
    driver.settle();
    let task = live(&driver, first.task);
    assert_eq!(task.tries.transient, 1);
    assert!(matches!(task.phase, tasks::Phase::Active(tasks::Active::BackingOff { .. })));
    driver.env.now = Time::from_nanos(Duration::from_secs(2).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(2).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let second = driver.last_assignment();
    assert!(second.attempt > first.attempt);
    driver.answer(Token::new(7), second.task, second.attempt, 0, transient, engine::Parts::none());
    driver.settle();
    let task = live(&driver, first.task);
    assert_eq!(task.tries.transient, 2);
    assert!(matches!(task.phase, tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Transient), .. }));
}

#[test]
#[expect(
    clippy::too_many_lines,
    clippy::wildcard_enum_match_arm,
    reason = "the one end-to-end story checks each routing stage"
)]
fn a_delegate_held_past_its_tries_is_escalated_two_levels_to_a_person_released_and_finished() {
    let (mut driver, root) = batch_fixture_with(3, 4);
    let mut holder = report_delegate(b"middle", Box::new([]));
    holder.authority.budget.spend = 50;
    holder.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    holder.authority.delegation.tasks = 2;
    holder.authority.delegation.depth = 3;
    let middle = call_batch(&mut driver, &root, 601, Box::new([holder]))[0];
    let middle_run = assigned_task(&driver, middle);
    let leaf = call_batch(&mut driver, &middle_run, 602, Box::new([report_delegate(b"leaf", Box::new([]))]))[0];
    let first = assigned_task(&driver, leaf);
    driver.answer(
        Token::new(7),
        leaf,
        first.attempt,
        0,
        tasks::End::Failed(tasks::Class::Transient),
        engine::Parts::none(),
    );
    driver.settle();
    driver.env.now = Time::from_nanos(Duration::from_secs(2).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(2).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let second = driver.last_assignment();
    assert_eq!(second.task, leaf);
    driver.answer(
        Token::new(7),
        leaf,
        second.attempt,
        0,
        tasks::End::Failed(tasks::Class::Transient),
        engine::Parts::none(),
    );
    driver.settle();
    let held =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(leaf)))).expect("held delegate");
    let revision = match held {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) => match row.escalation {
            tasks::Escalation::Waiting { revision, holder: tasks::EscalationHolder::Task(task), .. }
                if task == middle =>
            {
                revision
            }
            ref other => panic!("first recipient must be middle: {other:?}"),
        },
        other => panic!("held delegate row: {other:?}"),
    };
    let tasks::Escalation::Waiting { entry, .. } = live(&driver, leaf).escalation else {
        panic!("held delegate waits")
    };
    assert!(messages_to(&driver, &middle_run).iter().any(|(name, _)| *name == entry));
    tool_call(
        &mut driver,
        &middle_run,
        603,
        engine::Tool::DecideEscalation { task: leaf, revision, decision: engine::EscalationChoice::Pass },
    );
    assert!(matches!(
        driver.answered(603),
        Some(engine::CallAnswer::EscalationDecided { outcome: tasks::EscalationOutcome::Passed { .. }, .. })
    ));
    let held =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(leaf)))).expect("held delegate");
    let second_revision = match held {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) => match row.escalation {
            tasks::Escalation::Waiting { revision, holder: tasks::EscalationHolder::Task(task), .. }
                if task == root.task =>
            {
                revision
            }
            ref other => panic!("second recipient must be root task: {other:?}"),
        },
        other => panic!("held delegate row: {other:?}"),
    };
    // Passing keeps the hold's revision (jig 00de0b8c).
    assert_eq!(second_revision, revision);
    tool_call(
        &mut driver,
        &root,
        604,
        engine::Tool::DecideEscalation {
            task: leaf,
            revision: second_revision,
            decision: engine::EscalationChoice::Pass,
        },
    );
    let held =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(leaf)))).expect("held delegate");
    let person_revision = match held {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) => match row.escalation {
            tasks::Escalation::Waiting { revision, holder: tasks::EscalationHolder::Person(_), .. } => revision,
            ref other => panic!("third recipient must be person: {other:?}"),
        },
        other => panic!("held delegate row: {other:?}"),
    };
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(605)),
        sign_in: driver.session(),
        key: [65; 16],
        ask: people::Ask::DecideEscalation {
            project: 1,
            task: leaf,
            revision: person_revision,
            decision: people::EscalationDecision::Release,
        },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|item| matches!(item,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::EscalationDecided {
            task, choice: people::EscalationChoice::Released, ..
        }), .. } if *task == leaf)));
    let third = driver.last_assignment();
    assert_eq!(third.task, leaf);
    assert!(third.attempt > second.attempt);
    driver.answer(
        Token::new(7),
        leaf,
        third.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"finished".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    assert!(matches!(
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(leaf)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(_))))
    ));
}

#[test]
fn a_moved_task_is_funded_anew_by_its_new_requester() {
    let (mut driver, root) = batch_fixture_with(3, 2);
    let mut child_spec = report_delegate(b"long goal", Box::new([]));
    child_spec.authority.budget.spend = 50;
    child_spec.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Charter(1)]);
    child_spec.authority.delegation.tasks = 1;
    child_spec.authority.delegation.depth = 1;
    let child = call_batch(&mut driver, &root, 620, Box::new([child_spec]))[0];
    let child_run = assigned_task(&driver, child);
    let grandchild = call_batch(&mut driver, &child_run, 621, Box::new([report_delegate(b"subgoal", Box::new([]))]))[0];
    driver.send(protocol::turn(Token::new(7), child, child_run.attempt, 1, 5, None, b"worked"));
    driver.settle();
    let person = signed_in(&driver, 101).0;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(622)),
        sign_in: driver.session(),
        key: [62; 16],
        ask: people::Ask::Move { project: 1, task: child, reason: b"keep the goal".as_slice().into() },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|item| matches!(item,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Moved { task }), .. } if *task == child)));
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(moved)))) =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child))))
    else {
        panic!("moved task remains live")
    };
    assert_eq!(moved.requester, tasks::Party::Person(person));
    assert_eq!(moved.funder, tasks::Funder::Pool { project: 1, person, period: 1 });
    assert_eq!(moved.numbers, tasks::Numbers { budget: 45, spent: 0, spent_below: 0, reserved: 10 });
    assert_eq!(moved.root, child);
    assert_eq!(moved.depth, 0);
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(beneath)))) =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(grandchild))))
    else {
        panic!("grandchild remains live")
    };
    assert_eq!((beneath.root, beneath.depth), (child, 1));
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(old)))) =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(root.task))))
    else {
        panic!("old requester remains live")
    };
    assert!(!old.delegates.contains(&child));
    assert!(old.references.contains(&child));
    assert_eq!((old.numbers.reserved, old.numbers.spent_below), (5, 5));
    driver.answer(
        Token::new(7),
        root.task,
        root.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"chat done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    assert!(driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child)))));
    assert!(driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(grandchild)))));
}

#[test]
fn a_move_the_new_funder_cannot_cover_is_refused_whole() {
    let (mut driver, root) = batch_fixture_with_pool(2, 1, 100);
    let child = call_batch(&mut driver, &root, 630, Box::new([report_delegate(b"child", Box::new([]))]))[0];
    let before_parent =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(root.task)))).cloned();
    let before_child = driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child)))).cloned();
    let pool = tasks::Funder::Pool { project: 1, person: signed_in(&driver, 101).0, period: 1 };
    let before_pool = driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(pool)))).cloned();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(631)),
        sign_in: driver.session(),
        key: [63; 16],
        ask: people::Ask::Move { project: 1, task: child, reason: b"out of room".as_slice().into() },
    }));
    driver.settle();
    // Authority refusals carry what is lacking (jig b7bdd9e8): the child's
    // budget against the person's spent pool.
    let lacking = people::Lack::Budget { needs: 10, left: 0 };
    assert!(
        replied(
            &driver,
            &people::Outcome::Refused(people::Refusal::Authority {
                lacking,
                proposable: Some(people::Accepter::Role(0))
            })
        ),
        "{:?}",
        driver.delivered
    );
    assert_eq!(
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(root.task)))),
        before_parent.as_ref()
    );
    assert_eq!(
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child)))),
        before_child.as_ref()
    );
    assert_eq!(
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(pool)))),
        before_pool.as_ref()
    );
}

#[test]
fn a_move_carves_the_new_persons_pool_in_the_same_commit() {
    let (mut driver, root) = batch_fixture_custom(2, 1, 500, true, false, false);
    let child = call_batch(&mut driver, &root, 640, Box::new([report_delegate(b"second owner goal", Box::new([]))]))[0];
    let (person, session) = sign_in_person(&mut driver, 8, 641);
    let pool = tasks::Funder::Pool { project: 1, person, period: 1 };
    assert!(!driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(pool)))));
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(642)),
        sign_in: session,
        key: [64; 16],
        ask: people::Ask::Move { project: 1, task: child, reason: b"adopt goal".as_slice().into() },
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|item| matches!(item,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Moved { task }), .. } if *task == child)));
    assert!(driver.transactions.iter().any(|writes| {
        writes.iter().any(|write| matches!(write,
            Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.number == child && row.requester == tasks::Party::Person(person)))
        && writes.iter().any(|write| matches!(write,
            Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(row)))) if row.funder == pool && row.numbers.reserved == 10))
        && writes.iter().any(|write| matches!(write,
            Write::Save(Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome: people::Outcome::Moved { task }, .. }))) if *task == child))
    }));
}

#[test]
fn moving_a_held_delegate_rechecks_its_escalation_recipient() {
    let (mut driver, root) = batch_fixture_with(2, 1);
    let child = call_batch(&mut driver, &root, 650, Box::new([report_delegate(b"held goal", Box::new([]))]))[0];
    let first = assigned_task(&driver, child);
    driver.answer(
        Token::new(7),
        child,
        first.attempt,
        0,
        tasks::End::Failed(tasks::Class::Transient),
        engine::Parts::none(),
    );
    driver.settle();
    driver.env.now = Time::from_nanos(Duration::from_secs(2).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(2).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let second = driver.last_assignment();
    driver.answer(
        Token::new(7),
        child,
        second.attempt,
        0,
        tasks::End::Failed(tasks::Class::Transient),
        engine::Parts::none(),
    );
    driver.settle();
    let before =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child)))).expect("held child");
    assert!(
        matches!(before, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) if matches!(row.escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(parent), .. } if parent == root.task))
    );
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(651)),
        sign_in: driver.session(),
        key: [65; 16],
        ask: people::Ask::Move { project: 1, task: child, reason: b"adopt held work".as_slice().into() },
    }));
    driver.settle();
    let person = signed_in(&driver, 101).0;
    let after = driver
        .store
        .rows
        .get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child))))
        .expect("moved held child");
    assert!(
        matches!(after, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) if matches!(row.escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(found), .. } if found == person))
    );
}

#[test]
fn moving_a_proposer_rechecks_its_decision_holder() {
    let (mut driver, root) = batch_fixture_with(2, 2);
    let child = call_batch(&mut driver, &root, 660, Box::new([report_delegate(b"goal", Box::new([]))]))[0];
    let run = assigned_task(&driver, child);
    tool_call(
        &mut driver,
        &run,
        661,
        engine::Tool::Propose {
            action: engine::ProposedAction::Batch(Box::new([report_delegate(b"helper", Box::new([]))])),
            reason: b"need help".as_slice().into(),
            as_holder: false,
        },
    );
    let before =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child)))).expect("proposer live");
    assert!(
        matches!(before, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) if matches!(&row.proposal,
        Some(proposal) if matches!(proposal.state,
            tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(holder), .. } if holder == root.task)))
    );
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(662)),
        sign_in: driver.session(),
        key: [66; 16],
        ask: people::Ask::Move { project: 1, task: child, reason: b"adopt proposal".as_slice().into() },
    }));
    driver.settle();
    let person = signed_in(&driver, 101).0;
    let after = driver
        .store
        .rows
        .get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(child))))
        .expect("moved proposer");
    assert!(
        matches!(after, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) if matches!(&row.proposal,
        Some(proposal) if matches!(proposal.state,
            tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(holder), .. } if holder == person)))
    );
}

#[test]
fn saved_work_reaches_the_next_attempt() {
    let (mut driver, first) = Driver::chat();
    let parts = engine::Parts { saved: Some(Box::new([3])), pushed: Box::new([]), preparation: None };
    driver.answer(Token::new(7), first.task, first.attempt, 0, tasks::End::Parked, parts);
    driver.settle();
    assert_eq!(
        live(&driver, first.task).saved.as_ref(),
        [tasks::SavedResource { connector: 0, path: Box::new([Box::from(3u32.to_be_bytes())]) }]
    );
    driver.say(first.task, 49);
    let second = driver.last_assignment();
    assert_eq!(second.saved.as_ref(), [3]);
    assert!(second.attempt > first.attempt);
}

#[test]
fn a_worker_lost_mid_run_resumes_at_the_last_committed_turn() {
    let (mut driver, first) = Driver::chat();
    turn(&mut driver, &first, 1, 1);
    driver.settle();
    assert!(driver.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Turn {
        task: first.task,
        attempt: first.attempt,
        turn: 1
    }))));
    driver.send(protocol::lost(Token::new(7)));
    driver.env.now = Time::from_nanos(Duration::from_secs(6).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(6).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    assert_eq!(live(&driver, first.task).tries.lost, 1);
    driver.env.now = Time::from_nanos(Duration::from_secs(7).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(7).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.hello(Token::new(8));
    driver.settle();
    let second = driver.last_assignment();
    assert!(second.attempt > first.attempt);
    assert_eq!(second.transcript.len(), 1);
    assert_eq!(second.transcript[0].as_ref(), b"step");
}

#[test]
fn a_worker_frozen_past_its_grace_gets_no_next_attempt_until_its_sum_has_passed() {
    let (mut driver, first) = Driver::chat();
    driver.send(hello_with(Token::new(8), 1, Duration::from_secs(5), Box::new([])));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(delivery, Delivery::Refuse { channel }
        if *channel == Token::new(8))),
        "a worker whose stop bound reaches engine grace cannot place runs"
    );
    driver.send(protocol::lost(Token::new(7)));
    driver.env.now = Time::from_nanos(Duration::from_secs(4).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(4).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    for _ in 0..10 {
        driver.advance(true);
    }
    assert_eq!(live(&driver, first.task).tries.lost, 0);
    assert_eq!(driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::Assigned { .. })).count(), 1);
    driver.env.now = Time::from_nanos(Duration::from_secs(6).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(6).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    assert_eq!(live(&driver, first.task).tries.lost, 1);
}

/// A hub call the world's protocol cannot decode by itself: the test decodes
/// it, and gets the decode request's reply right back.
fn typed_call(driver: &mut Driver, assignment: &engine::Assignment, name: &[u8]) -> ReplyTo {
    let up = host::LinkUp::Call {
        run: Token::new(assignment.task),
        attempt: Token::new(assignment.attempt),
        call: name.into(),
        tool: b"unknown-tool".as_slice().into(),
        writes: false,
        input: b"opaque-input".as_slice().into(),
        deadline: Duration::from_secs(2),
    };
    driver.send(protocol::up(Token::new(7), up, engine::Parts::none()));
    for _ in 0..20 {
        driver.advance(true);
        if !driver.host_requests.is_empty() {
            break;
        }
    }
    let engine::HostRequest::Decode { to, task, attempt, call } = *driver.host_requests.remove(0) else {
        panic!("typed decode request")
    };
    assert_eq!((task, attempt), (assignment.task, assignment.attempt));
    assert_eq!(
        (call.name.as_ref(), call.tool.as_ref(), call.input.as_ref()),
        (name, b"unknown-tool".as_slice(), b"opaque-input".as_slice())
    );
    assert!(!call.writes);
    assert_eq!(call.deadline, Duration::from_secs(2));
    to
}

/// The world's protocol renders the decided answer as opaque text, which the
/// root keeps settled beside the call's durable decision.
fn typed_unavailable(
    driver: &mut Driver,
    assignment: &engine::Assignment,
    name: &[u8],
) -> jig_domain_core::SettledCall {
    let to = typed_call(driver, assignment, name);
    let body = engine::Call { completion: 2, position: 0, tool: engine::Tool::Unavailable };
    driver.send(engine::Event::Worker(engine::WorkerInput::DecodedHostCall { to, body }));
    driver.settle();
    let rendered = driver.protocol.answers.iter().filter(|(named, _)| named.as_ref() == name).collect::<Vec<_>>();
    assert_eq!(rendered.len(), 1, "one render after the decision");
    assert_eq!(rendered[0].1, engine::CallAnswer::Unavailable);
    let key = temper_engine_domain::CallKey {
        task: assignment.task,
        attempt: assignment.attempt,
        completion: 2,
        position: 0,
    };
    let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record)))) =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key))))
    else {
        panic!("durable decision")
    };
    record.settled.clone().expect("settled typed answer")
}

#[test]
fn typed_calls_keep_their_opaque_answers_for_resumed_assignments() {
    let (mut driver, first) = Driver::chat();
    turn(&mut driver, &first, 1, 1);
    driver.settle();
    let settled = typed_unavailable(&mut driver, &first, b"opaque-call");
    assert_ne!(settled.serial, 0);
    assert_eq!(
        settled.answer,
        jig_domain_core::SettledAnswer::Host { error: false, body: b"opaque-call".as_slice().into() }
    );
    driver.answer(Token::new(7), first.task, first.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(first.task, 93);
    let resumed = driver.last_assignment();
    assert_eq!(resumed.transcript.as_ref(), [Box::<[u8]>::from(&b"step"[..])]);
    assert_eq!(resumed.settled.as_ref(), std::slice::from_ref(&settled));
    assert_eq!(resumed.answered[0].settled.as_ref(), Some(&settled));
}

#[test]
fn a_typed_call_replay_uses_the_settled_record_without_rendering_again() {
    let (mut driver, assignment) = Driver::chat();
    let saved = typed_unavailable(&mut driver, &assignment, b"replay");
    let (before, rows) = (driver.store.applied, driver.store.rows.clone());
    let (delivered, rendered) = (driver.delivered.len(), driver.protocol.answers.len());
    let to = typed_call(&mut driver, &assignment, b"replay");
    let body = engine::Call { completion: 2, position: 0, tool: engine::Tool::Unavailable };
    driver.send(engine::Event::Worker(engine::WorkerInput::DecodedHostCall { to, body }));
    driver.settle();
    assert_eq!(driver.store.applied, before);
    assert_eq!(driver.store.rows, rows);
    assert_eq!(driver.protocol.answers.len(), rendered, "no render again");
    let jig_domain_core::SettledAnswer::Host { error, body } = saved.answer else { panic!("a host answer") };
    assert!(driver.delivered[delivered..].iter().any(|delivery| matches!(delivery,
        Delivery::Link { down: host::LinkDown::Relayed { call, answer: host::Relayed::Answer { error: sent, text } , .. }, .. }
            if call.as_ref() == b"replay" && *sent == error && *text == body)));
}

#[test]
fn restart_refuses_live_tasks_or_claims_past_lowered_limits_without_dropping_them() {
    let mut driver = Driver::configured(Store::new(), sharing_config(91), &limits());
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 98);
    driver.settle();
    chat(&mut driver, 99);
    for _ in 0..100 {
        driver.advance(true);
    }
    assert_eq!(
        driver
            .store
            .rows
            .keys()
            .filter(|key| matches!(key, Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(_)))))
            .count(),
        2
    );
    assert_eq!(
        driver
            .store
            .rows
            .values()
            .filter(|row| matches!(
                row,
                Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(_)))
            ))
            .count(),
        2
    );
    let before = driver.store.rows.clone();
    let mut task_bounds = limits();
    task_bounds.people.operations = 1;
    task_bounds.tasks.tasks = 1;
    task_bounds.tasks.project_tasks = 1;
    task_bounds.tasks.tree_tasks = 1;
    let mut claim_bounds = limits();
    claim_bounds.fleet.attempts = 1;
    for (bounds, step) in
        [(task_bounds, jig_domain_core::RestartStep::LoadCore), (claim_bounds, jig_domain_core::RestartStep::AdoptRuns)]
    {
        let store = driver.store.fork();
        let mut restarted = Driver::configured(store, sharing_config(91), &bounds);
        for _ in 0..100 {
            restarted.advance(true);
        }
        assert!(restarted.stopped);
        assert_eq!(restarted.root.restart_failure(), Some(step));
        assert_eq!(restarted.store.rows, before);
        assert!(restarted.delivered.is_empty());
    }
}

#[test]
fn signing_out_routes_to_people_and_expires_the_session() {
    let mut driver = Driver::new(Store::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let session = driver.session();
    let reply = Token::new(1103);
    driver.send(engine::Event::Party(engine::PartyInput::SignOut { reply_to: ReplyTo::new(reply), sign_in: session }));
    assert!(!driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { to, reply: people::Reply::SignedOut } if to == &ReplyTo::new(reply))));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { to, reply: people::Reply::SignedOut } if to == &ReplyTo::new(reply))));
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(1104)),
        sign_in: session,
        read: jig_domain_core::reads::Read::Frame,
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::Read { answer: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::SignedOut), .. }
    )));
}

#[path = "root_routes/ended_read.rs"]
mod ended_read;
