//! Named story values and one client profile, shared by every story.
use crate::{app::Temper, front::Joint};
use jig_client_world::harness::FrontFault;
use jig_client_world::harness::{Settings, World};
use jig_domain_tasks as tasks;
use skein_lib::{Duration, RequestLimits, Time, Token};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
use temper_client_domain::{Limits, forge};
use temper_client_view as view;
use temper_engine_domain as engine;
use temper_engine_domain_world::direct::Driver;
pub const FORGE: u16 = temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge);
pub const FORGE_HOST: &[u8] = b"forge.example";
pub const REPOSITORIES: &[(&[u8], &[u8], u32)] = &[(b"acme", b"shop", 7), (b"acme", b"second", 8)];
pub const PARTY: u64 = 1;
pub const SEED: u64 = 1101;
pub const WITHIN: Duration = Duration::from_secs(1);
#[must_use]
pub fn domain_limits() -> Limits {
    {
        let requests = RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        };
        let client = jig_client::Limits {
            domain: jig_client::DomainLimits {
                accounts: jig_client::AccountCounts { accounts: 16, sources: 8, sign_ins: 4 },
                owners: &[jig_client::Spoken {
                    family: temper_client_protocol_wire::FORGE,
                    version: temper_client_protocol_wire::CLIENT_VERSION,
                }],
                requests,
                ask_bytes: requests.bytes,
                handles: 9,
                watches: 9,
                turn_bytes: 65536,
                backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                authority: 12,
                heartbeat: Duration::from_millis(100),
                reads: 8,
                parts: 8,
                delegates: 2,
                document_bytes: 65536,
                records: 2,
                kept: 4,
                projects: 4,
                text: 64,
                rows: 4,
                fields: 4,
                segments: 9,
            },
            exchanges: 8,
        };

        Limits {
            forge: forge::Limits { field_bytes: 256, checks: 8 },
            client,
            accounts: temper_client_domain::limits::AccountLimits { accounts: 16, sources: 8, uses: 8, sign_ins: 4 },
        }
    }
}

type Books = view::words::Words;
static BOOKS: OnceLock<Mutex<BTreeMap<u32, Books>>> = OnceLock::new();
pub fn phrasebooks(limits: &Limits) -> Books {
    let mut books = BOOKS.get_or_init(|| Mutex::new(BTreeMap::new())).lock().expect("fixture books lock");
    books
        .entry(limits.client.domain.text)
        .or_insert_with(|| {
            view::words::Words::english(FORGE, limits.client.domain.text, view::words::Shell::Terminal)
                .expect("temper words fit")
        })
        .clone()
}
#[must_use]
pub fn view_bounds(limits: &Limits, words: &view::words::Words) -> view::Limits {
    view::Limits::of(limits, words).expect("world view bound fits")
}
#[must_use]
pub fn inbox() -> temper_client_domain::Address {
    temper_client_domain::Address::Inbox { filter: temper_client_domain::address::Filter::ALL }
}
#[must_use]
pub fn world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::new(seed, domain_limits()))
}
#[must_use]
pub fn world_with_large_turns(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_large_turns(seed, &domain_limits()))
}
fn world_with(seed: u64, front: Joint) -> World<Temper, Joint> {
    let mut world = World::new(
        Settings {
            seed,
            words: view::words::jig_language(),
            heap_bytes_per_client: 64 * 1024 * 1024,
            iterations: 4096,
            ..Settings::default()
        },
        front,
    );
    world.randomness(2, 0, 0);
    world
}
#[must_use]
pub fn codec() -> jig_protocol_transcript::Limits {
    jig_protocol_transcript::Limits::new(
        32768,
        jig_client::contract::TurnLimits { messages: 32, blocks: 32, text: 8192 },
    )
}

#[must_use]
pub fn chats_page() -> temper_client_domain::Address {
    temper_client_domain::Address::Chats
}
#[must_use]
pub fn chats(driver: &Driver) -> u32 {
    u32::try_from(
        driver
            .store
            .rows
            .values()
            .filter(|record| {
                matches!(
                    record,
                    engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                    if matches!(row.requester, tasks::Party::Person(_)) && row.tracked.is_none()
                )
            })
            .count(),
    )
    .expect("bounded chats")
}
#[must_use]
pub fn faults(seed: u64) -> Vec<(Time, FrontFault)> {
    let choices = [
        FrontFault::Restart { offline: Duration::from_millis(30) },
        FrontFault::NoAnswer,
        FrontFault::DropWatch,
        FrontFault::Busy,
        FrontFault::SignedOut,
    ];
    (0..4)
        .map(|index| {
            (
                Time::from_nanos(Duration::from_millis(20 * (index + 1)).as_nanos()),
                choices[usize::try_from((seed + index) % 5).expect("five faults")].clone(),
            )
        })
        .collect()
}
/// A chat made and held by two failed real worker attempts.
pub fn held_chat(front: &mut Joint) -> u64 {
    let driver = &mut front.engine;
    driver.hello(Token::new(7));
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(Token::new(1001)),
        sign_in: driver.session(),
        key: [31; 16],
        ask: temper_engine_domain_world::walking::chat(1, b"Held chat"),
    }));
    driver.settle();
    let first = driver.last_assignment();
    driver.answer(
        Token::new(7),
        first.task,
        first.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
    driver.env.now = Time::from_nanos(Duration::from_secs(2).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let second = driver.last_assignment();
    driver.answer(
        Token::new(7),
        second.task,
        second.attempt,
        0,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
    assert!(matches!(driver.task(first.task).expect("held chat").phase, tasks::Phase::Held { .. }));
    first.task
}

/// The stored ended task a client's one-shot task/result read asks for.
#[must_use]
pub fn read(
    store: &temper_engine_domain_world::store::Store,
    range: &jig_domain_core::reads::Range,
) -> jig_domain_core::reads::Loaded {
    use jig_domain_core::{Key, Record, reads};
    let (reads::Scope::Task { task } | reads::Scope::Result { task } | reads::Scope::PersonTask { task }) = range.scope
    else {
        return reads::Loaded::Gone;
    };
    let Some(engine::Record::Core(Record::Tasks(tasks::Stored::Ended(row)))) =
        store.rows.get(&engine::Key::Core(Key::Tasks(tasks::Key::Ended(task))))
    else {
        return reads::Loaded::Gone;
    };
    let value = reads::Row {
        value: Record::Tasks(tasks::Stored::Ended(row.clone())),
        ended_read: None,
        owner: Some(row.clone()),
        ancestors: Box::new([]),
        related: None,
        unjoined: None,
        linked: Box::new([]),
        inbox: None,
    };
    if range.after.is_some() {
        return reads::Loaded::Page { rows: Box::new([]), next: None };
    }
    if range.rows == 0 || value.owned_bytes().expect("finite ended row") > range.bytes {
        return reads::Loaded::Busy;
    }
    reads::Loaded::Page { rows: Box::new([value]), next: None }
}
#[must_use]
pub fn project() -> temper_client_domain::Address {
    temper_client_domain::Address::Inbox { filter: temper_client_domain::address::Filter::ALL }
}
#[must_use]
pub fn adopt(owner: &[u8], name: &[u8]) -> Vec<skein_fake_person::Step<temper_client_domain::Address>> {
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    vec![
        Step::See { find: Find::named(Role::Link, b"Adopt a repository"), within: WITHIN },
        Step::Press { find: Find::named(Role::Link, b"Adopt a repository") },
        Step::See { find: Find::named(Role::TextField, b"Owner"), within: WITHIN },
        Step::Type { find: Find::named(Role::TextField, b"Owner"), words: owner.into() },
        Step::Type { find: Find::named(Role::TextField, b"Name"), words: name.into() },
        Step::Type { find: Find::named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::Press { find: Find::named(Role::Button, b"Adopt") },
    ]
}
#[must_use]
pub fn adopt_and_see_outcome(owner: &[u8], name: &[u8]) -> Vec<skein_fake_person::Step<temper_client_domain::Address>> {
    use skein_fake_person::{Find, Step};
    let mut steps = adopt(owner, name);
    steps.extend([Step::See { find: Find::text(b"Adopted acme/shop on forge 1 as owned."), within: WITHIN }]);
    steps
}

const QUESTION_CHAT_REPLY: Token = Token::new(1002);
const QUESTION_CHAT_KEY: [u8; 16] = [32; 16];
pub(crate) const QUESTION_WORKER: Token = Token::new(7);
pub(crate) const QUESTION_PROJECT: u32 = 1;
const QUESTION_CALL: u64 = 1003;
const QUESTION_DELEGATE: u64 = 1004;
const QUESTION_COMPLETION: u32 = 1;
const QUESTION_POSITION: u32 = 1;
const QUESTION_ASSIGN_AT: Time = Time::from_nanos(2_000_000_000);
/// A question committed by the running chat for its person.
pub fn question(front: &mut Joint, words: &[u8]) -> u64 {
    *front = Joint::with_questions(SEED, &domain_limits());
    let driver = &mut front.engine;
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(QUESTION_CHAT_REPLY),
        sign_in: driver.session(),
        key: QUESTION_CHAT_KEY,
        ask: driver.chat_ask(QUESTION_PROJECT, b"Question chat"),
    }));
    driver.settle();
    let parent = driver.last_assignment();
    driver.call(
        QUESTION_WORKER,
        parent.task,
        parent.attempt,
        QUESTION_DELEGATE,
        engine::Call {
            completion: QUESTION_COMPLETION,
            position: QUESTION_POSITION,
            tool: engine::Tool::Delegate {
                batch: Box::new([temper_engine_domain_world::fixtures::report_delegate(
                    b"Ask the person",
                    Box::new([]),
                )]),
            },
        },
    );
    driver.settle();
    driver.answer(QUESTION_WORKER, parent.task, parent.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.env.now = QUESTION_ASSIGN_AT;
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    let run = driver.last_assignment();
    assert_ne!(run.task, parent.task, "delegate assigned: {:?}", driver.answered(QUESTION_DELEGATE));
    driver.call(
        QUESTION_WORKER,
        run.task,
        run.attempt,
        QUESTION_CALL,
        engine::Call {
            completion: QUESTION_COMPLETION,
            position: QUESTION_POSITION,
            tool: engine::Tool::Message {
                target: parent.task,
                form: engine::MessageForm::Question,
                words: words.into(),
            },
        },
    );
    driver.settle();
    assert!(
        matches!(driver.answered(QUESTION_CALL), Some(engine::CallAnswer::Sent { .. })),
        "question committed: {:?}",
        driver.answered(QUESTION_CALL)
    );
    parent.task
}
#[test]
fn the_question_fixture_commits_its_numbered_question_before_the_client_starts() {
    let mut front = Joint::new(SEED, domain_limits());
    let task = question(&mut front, b"Which provider first?");
    assert!(
        front
            .engine
            .task(task)
            .expect("chat")
            .inbox
            .iter()
            .any(|message| message.words.as_ref() == b"Which provider first?")
    );
}

const CHAT_REPLY: Token = Token::new(1005);
const CHAT_KEY: [u8; 16] = [35; 16];
const CHAT_PROJECT: u32 = 1;
/// A real running chat, ready for the person's next words.
pub fn chat(front: &mut Joint, words: &[u8]) -> u64 {
    let driver = &mut front.engine;
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(CHAT_REPLY),
        sign_in: driver.session(),
        key: CHAT_KEY,
        ask: driver.chat_ask(CHAT_PROJECT, words),
    }));
    driver.settle();
    driver.last_assignment().task
}
const FIRST_REPLY_TURN: u32 = 1;
const REPLY_SPEND: u64 = 0;
/// A real committed worker turn puts this chat in the inbox's replies group.
pub fn replied_chat(front: &mut Joint, words: &[u8]) -> u64 {
    let task = chat(front, words);
    let assignment = front.engine.last_assignment();
    let event = temper_engine_domain_world::protocol::turn(
        QUESTION_WORKER,
        task,
        assignment.attempt,
        FIRST_REPLY_TURN,
        REPLY_SPEND,
        None,
        b"reply",
    );
    front.engine.send(event);
    front.engine.settle();
    task
}
#[must_use]
pub fn chat_page(task: u64) -> temper_client_domain::Address {
    temper_client_domain::Address::Chat { task }
}

const GOAL_REPLY: Token = Token::new(1201);
const GOAL_KEY: [u8; 16] = [41; 16];
const GOAL_PROJECT: u32 = 1;
const GOAL_PRIORITY: u32 = 1;
const GOAL_LABEL: u32 = 2;
const DELEGATE_CALL: u64 = 1202;
const DELEGATE_COMPLETION: u32 = 1;
const DELEGATE_POSITION: u32 = 1;
const RETRY_AT: Time = Time::from_nanos(2_000_000_000);
const NO_SPEND: u64 = 0;
/// A tracked goal with a failed delegate, held by the real engine.
pub fn goal_with_held_delegate(front: &mut Joint) -> (u64, u64) {
    *front = Joint::with_questions(SEED, &domain_limits());
    let goal = make_goal(front, false);
    let driver = &mut front.engine;
    let parent = driver.last_assignment();
    assert_eq!(parent.task, goal);
    driver.call(
        QUESTION_WORKER,
        parent.task,
        parent.attempt,
        DELEGATE_CALL,
        engine::Call {
            completion: DELEGATE_COMPLETION,
            position: DELEGATE_POSITION,
            tool: engine::Tool::Delegate {
                batch: Box::new([temper_engine_domain_world::fixtures::report_delegate(
                    b"Review the change",
                    Box::new([]),
                )]),
            },
        },
    );
    driver.settle();
    let Some(engine::CallAnswer::Delegated(children)) = driver.answered(DELEGATE_CALL) else {
        panic!("delegate admitted: {:?}", driver.answered(DELEGATE_CALL));
    };
    let held = children[0].task;
    driver.answer(QUESTION_WORKER, parent.task, parent.attempt, NO_SPEND, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    fail_delegate(driver, held);
    driver.env.now = RETRY_AT;
    engine::fire(&mut driver.root, &driver.env);
    engine::release(&mut driver.root, &driver.env, &mut driver.out);
    driver.collect();
    driver.settle();
    fail_delegate(driver, held);
    assert!(matches!(driver.task(held).expect("delegate").phase, tasks::Phase::Held { .. }));
    (goal, held)
}
fn fail_delegate(driver: &mut Driver, held: u64) {
    let run = driver.last_assignment();
    assert_eq!(run.task, held, "delegate has the next worker attempt");
    driver.answer(
        QUESTION_WORKER,
        held,
        run.attempt,
        NO_SPEND,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts::none(),
    );
    driver.settle();
}
fn make_goal(front: &mut Joint, procedure: bool) -> u64 {
    let driver = &mut front.engine;
    driver.sign_in();
    driver.settle();
    if procedure {
        adopt_goal_repository(driver);
    }
    let jig_domain_people::Ask::Create { mut batch, .. } = driver.chat_ask(GOAL_PROJECT, b"Fix the adopted repository")
    else {
        panic!("create fixture");
    };
    batch[0].tracked = Some(GOAL_PRIORITY);
    batch[0].label = GOAL_LABEL;
    batch[0].flags.conversation = false;
    batch[0].flags.listed = true;
    if procedure {
        batch[0].authority.tools = NO_TOOLS;
        batch[0].parameters = Box::new([jig_domain_people::Parameter::Resource {
            name: REPOSITORY_PARAMETER,
            connector: FORGE_CONNECTOR,
            resource: GOAL_REPOSITORY,
        }]);
        batch[0].executor =
            jig_domain_people::TaskExecutor::Procedure { connector: FORGE_CONNECTOR, code: FORGE_PROCEDURE };
        batch[0].contract =
            jig_domain_people::Contract::Change { connector: FORGE_CONNECTOR, kind: FORGE_CHANGE, words: REPORT_BYTES };
    }
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(GOAL_REPLY),
        sign_in: driver.session(),
        key: GOAL_KEY,
        ask: jig_domain_people::Ask::Create { project: GOAL_PROJECT, batch },
    }));
    driver.settle();
    let mut reply = None;
    for delivery in driver.delivered.iter().rev() {
        if let engine::Delivery::WebReply { reply: value, .. } = delivery {
            reply = Some(value);
            break;
        }
    }
    let Some(jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Created { first, .. })) = reply else {
        panic!("goal creation expected: {reply:?}");
    };
    *first
}
#[must_use]
pub fn task_page(task: u64) -> temper_client_domain::Address {
    temper_client_domain::Address::Task { task }
}
#[must_use]
pub fn plan_item(task: u64, status: &[u8]) -> skein_fake_person::Find {
    use skein_fake_person::{Find, Target};
    Find::named(skein_ui::Role::Status, status).within(Box::new([Target::ExactRole {
        role: skein_ui::Role::TreeItem,
        name: format!("Task {task}").into_bytes().into_boxed_slice(),
    }]))
}
const FORGE_CONNECTOR: u16 = 0;
const NO_TOOLS: u64 = 0;
const FORGE_PROCEDURE: u32 = 2;
const FORGE_CHANGE: u16 = 1;
const REPORT_BYTES: u32 = 32;
/// A tracked procedure root whose chip names its forge-owned panel.
pub fn goal_with_forge_panel(front: &mut Joint) -> u64 {
    *front = Joint::with_procedures(SEED, &domain_limits());
    make_goal(front, true)
}

const ADOPT_FORGE: u16 = 1;
const ADOPT_REPLY: Token = Token::new(1203);
const ADOPT_KEY: [u8; 16] = [43; 16];
const ADOPT_DIGEST: [u8; 32] = [43; 32];
const REPOSITORY_PARAMETER: u32 = 1;
const GOAL_REPOSITORY: u64 = 7;
fn adopt_goal_repository(driver: &mut Driver) {
    let mut request = temper_engine_domain_world::fixtures::adoption(ADOPT_FORGE);
    request.owner = b"acme".as_slice().into();
    request.name = b"shop".as_slice().into();
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        scope: jig_domain_people::ApplicationScope::Project(GOAL_PROJECT),
        reply_to: skein_lib::ReplyTo::new(ADOPT_REPLY),
        sign_in: driver.session(),
        key: ADOPT_KEY,
        digest: ADOPT_DIGEST,
        carried: None,
        request,
    }));
    driver.settle();
}

/// Guard on one-step answer waits, never a client limit.
const ANSWER_STEPS: u32 = 4096;
#[must_use]
pub fn agent(world: &mut World<Temper, Joint>) -> jig_client_world::harness::ClientId {
    let limits = temper_client_tools::limits::client();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, inbox(), limits, crate::agent::new)
}
#[must_use]
pub fn call(words: &[&[u8]]) -> crate::app::Event {
    match temper_client_tools::table::parse(words).unwrap_or_else(|error| panic!("agent call usage: {error:?}")) {
        temper_client_tools::jig::call::Parsed::Call { call, json } => {
            crate::app::Event::Call { number: 0, call, json, value: None }
        }
        temper_client_tools::jig::call::Parsed::File { .. } => panic!("agent call is a file"),
    }
}
pub fn give(world: &mut World<Temper, Joint>, agent: jig_client_world::harness::ClientId, words: &[&[u8]]) -> u32 {
    use jig_client_world::harness::Until;
    let start = world.now();
    for _ in 0..ANSWER_STEPS {
        if world.root(agent).agent().ready() && world.root(agent).agent().kept() == 0 {
            break;
        }
        world.run(Until::Steps(2));
    }
    assert!(world.now() <= start.saturating_add(WITHIN), "agent ready within world time");
    assert!(world.root(agent).agent().ready() && world.root(agent).agent().kept() == 0, "agent ready before call");
    let number = world.root(agent).agent().calls() + 1;
    let crate::app::Event::Call { call, json, .. } = call(words) else { unreachable!() };
    world.event(agent, crate::app::Event::Call { number, call, json, value: None });
    number
}
pub fn answered(
    world: &mut World<Temper, Joint>,
    agent: jig_client_world::harness::ClientId,
    number: u32,
) -> temper_client_tools::jig::Printed {
    use jig_client_world::harness::Until;
    let start = world.now();
    for _ in 0..ANSWER_STEPS {
        let outcome = world.run(Until::Steps(2));
        if let Some(printed) = world.root(agent).agent().printed(number) {
            assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}");
            assert!(world.now() <= start.saturating_add(WITHIN), "agent answered within world time");
            return temper_client_tools::jig::Printed {
                status: printed.status,
                stdout: printed.stdout.clone(),
                stderr: printed.stderr.clone(),
            };
        }
    }
    panic!("agent call {number} was not answered");
}
#[must_use]
pub fn done_task(printed: &temper_client_tools::jig::Printed) -> u64 {
    let stdout = std::str::from_utf8(&printed.stdout).expect("printed UTF-8");
    stdout
        .strip_prefix("done: task ")
        .and_then(|value| value.split_once(", "))
        .expect("done task outcome")
        .0
        .parse()
        .expect("task number")
}

/// tools.md, 12: two 16 KiB parts within the tools' 32 KiB text bound.
pub const LARGE_TURN_TEXT: usize = 24_576;
/// The encoded turn's single row fits `LOAD_BYTES`; its activity row crosses it.
// Jig 2a62db26's task-record fields add 14 B to the owner row; trim padding only.
pub const LARGE_TURN_OPAQUE: usize = 38_558;
#[must_use]
pub fn codec_for(limits: &jig_client::DomainLimits) -> jig_protocol_transcript::Limits {
    let bytes = limits.document_bytes.min(u64::from(jig_client_protocol_wire::CEILINGS.turn_carried_bytes));
    jig_protocol_transcript::Limits::new(
        u32::try_from(bytes).expect("wire bound fits"),
        jig_client::contract::TurnLimits { messages: limits.rows, blocks: limits.rows, text: limits.text },
    )
}
/// Commit smith's session turn through the worker's real turn boundary.
pub fn large_turn(front: &mut Joint, task: u64) -> (u64, u32) {
    use smith_domain_session::llm;
    let assignment = front.engine.last_assignment();
    let mut turn =
        temper_worker_domain_world::protocol::session_turn(FIRST_REPLY_TURN, REPLY_SPEND, LARGE_TURN_TEXT, true);
    for message in &mut turn.messages {
        let mut blocks = message.content.to_vec();
        blocks.push(llm::Block::Opaque { bytes: vec![0; LARGE_TURN_OPAQUE].into_boxed_slice() });
        message.content = blocks.into_boxed_slice();
    }
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("large turn encodes");
    front.engine.send(temper_engine_domain_world::protocol::turn(
        QUESTION_WORKER,
        task,
        assignment.attempt,
        FIRST_REPLY_TURN,
        REPLY_SPEND,
        None,
        &body,
    ));
    front.engine.settle();
    (assignment.attempt, FIRST_REPLY_TURN)
}
/// Complete ownership of a turn alone and joined to its activity row.
#[must_use]
pub fn turn_row_costs(front: &Joint, task: u64) -> (u64, u64) {
    use jig_domain_core::{self as core, reads};
    let owner = front.engine.task(task).expect("live turn owner").clone();
    let turn = front
        .engine
        .store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Core(value @ core::Record::Core(core::CoreRecord::Turn(turn))) = record
                && turn.task == task
            {
                Some(value.clone())
            } else {
                None
            }
        })
        .expect("durable turn");
    let mut row = reads::Row {
        value: turn,
        owner: Some(Box::new(owner)),
        ancestors: Box::new([]),
        related: None,
        unjoined: None,
        ended_read: None,
        linked: Box::new([]),
        inbox: None,
    };
    let single = row.owned_bytes().expect("single row cost");
    let line = front
        .engine
        .store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) = record
                && line.task == task
                && matches!(line.line, tasks::Line::Turn { .. })
            {
                Some(line.clone())
            } else {
                None
            }
        })
        .expect("durable turn activity");
    row.related = Some(row.value);
    row.value = core::Record::Tasks(tasks::Stored::Activity(line));
    (single, row.owned_bytes().expect("activity row cost"))
}
/// Follow the printed continuation calls, retaining the turn's text alone.
pub fn read_whole(
    world: &mut World<Temper, Joint>,
    agent: jig_client_world::harness::ClientId,
    words: &[&[u8]],
) -> (Vec<u8>, u32) {
    let mut words: Vec<Box<[u8]>> = words.iter().map(|word| Box::from(*word)).collect();
    let mut text = Vec::new();
    let mut reads = 0;
    loop {
        let borrowed: Vec<&[u8]> = words.iter().map(AsRef::as_ref).collect();
        let number = give(world, agent, &borrowed);
        let printed = answered(world, agent, number);
        assert_eq!(printed.status, 0, "{printed:?}");
        reads += 1;
        let stdout = std::str::from_utf8(&printed.stdout).expect("turn text UTF-8");
        let mut next = None;
        for line in stdout.lines() {
            if let Some((_, call)) = line.split_once("read on with: ").or_else(|| line.split_once("read with: ")) {
                next = Some(call.split_ascii_whitespace().map(|word| Box::from(word.as_bytes())).collect());
            } else if !line.starts_with("turn ") && !line.starts_with("opaque ") {
                text.extend_from_slice(line.as_bytes());
            }
        }
        let Some(next) = next else { return (text, reads) };
        words = next;
    }
}

/// Copy an executable act from the first acts section of a printed document.
#[must_use]
pub fn printed_act(stdout: &[u8], act: &[u8]) -> Vec<Box<[u8]>> {
    let stdout = std::str::from_utf8(stdout).expect("printed document UTF-8");
    let act = std::str::from_utf8(act).expect("act UTF-8");
    let line = stdout
        .lines()
        .skip_while(|line| *line != "acts:")
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .find(|line| line.split_ascii_whitespace().next() == Some(act))
        .expect("printed act");
    line.split_ascii_whitespace().map(|word| Box::from(word.as_bytes())).collect()
}

/// Cut the answer after application, before the client's first retry.
pub fn cut(world: &mut World<Temper, Joint>, agent: jig_client_world::harness::ClientId, words: &[&[u8]]) {
    use jig_client_world::harness::Until;
    world.fault(FrontFault::NoAnswer);
    give(world, agent, words);
    let start = world.now();
    let first_retry = start.saturating_add(temper_client_tools::limits::client().client.domain.requests.first);
    for _ in 0..ANSWER_STEPS {
        world.run(Until::Steps(2));
        if world.front().answers_to_lose() == 0 {
            assert!(start <= world.now() && world.now() < first_retry, "answer cut before first retry");
            return;
        }
    }
    panic!("answer was not cut");
}

#[must_use]
pub fn accounts_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_accounts(seed, domain_limits()))
}
