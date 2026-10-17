use crate::{limits, settings};
use jig_domain_host as host;
use skein_lib::{Secret, Token};

pub const ENGINE: &[u8] = br#"{
"limits": {"profile": "small", "overrides": {"tasks.result_bytes":32,"people.amendment_bytes":64,"core.policy_bytes":256,"tasks.spec_bytes":128,"tasks.message_bytes":512,"tasks.hold_bytes":64,"tasks.authority_bytes":64,"people.words":512,"core.call_answer_bytes":2048,"notes.recalled":1}},
"llm": {
"endpoints": [{
"name": "fake",
"number": 1,
"dialect": 0,
"account": 1,
"provider": "fake-provider",
"address": {"socket": "127.0.0.1:7101"},
"transport": "plaintext",
"headers": [],
"connect_ms": 10000,
"handshake_ms": 10000
}],
"models": [{
"name": "small",
"endpoint": "fake",
"window": 8192,
"output": 1024,
"prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000}
}],
"accounts": [{
"number": 1,
"source": {"handed_in": {"file": "accounts/chat.json"}}
}],
"records": "accounts/records"
},
"hosts": {
"workers": [
{"name": "worker-a", "digest": "secrets/worker-a.digest"},
{"name": "worker-b", "digest": "secrets/worker-b.digest"}
],
"listener": [{"in_process": "workers"}]
},
"charters": [{
"number": 1, "instructions": "charters/chat.txt", "model": "small", "alternatives": [],
"hosts": "workers", "waiting_ms": 10000, "resume": true,
"turns": 4, "time_ms": 30000, "reserve_turns": 1, "reserve_time_ms": 5000, "spend": 20,
"inspect": true, "modify": false, "shell": false, "agents": false, "call_timeout_ms": 10000
}],
"templates": [
{"number": 1, "name": "chat", "executor": {"charter": 1}, "contract": {"report": {"words": 32}}, "shallower": 1, "budget": {"as_allowed": null}, "deadline": "none", "tracked": null, "conversation": true, "listed": false, "retention": "kept", "label": 1, "grace_ms": 5000},
{"number": 2, "name": "goal", "executor": "chosen", "contract": {"report": {"words": 32}}, "shallower": 1, "budget": "given", "deadline": "given", "tracked": {"priority": 1}, "conversation": false, "listed": true, "retention": "kept", "label": 2, "grace_ms": null}
],
"clients": {
"listener": {"socket": "127.0.0.1:7100"},
"heartbeat_ms": 10000,
"link": true
},
"deployment": {"directory": "state", "seed": 7},
"owners": [{"provider": 1, "subject": "owner-a"}],
"providers": [{"number": 1, "kind": "link"}, {"number": 2, "kind": "deployment"}],
"connectors": [1],
"memory_bytes": 1152921504606846976,
"rules": {
"ceiling": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []},
"period_spend": 100, "minimum_run_spend": 1, "maximum_run_spend": 100,
"implies": [], "requirements": []
},
"projects": [{"number": 1, "roles": [{"connector": 1, "permission": 1, "role": 0}],
"policy": {"templates": [], "escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
"ceiling": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []},
"roles": [{"number": 0, "period_spend": 100, "requests": 511, "decides": 31,
"authority": {"tools": 1023, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 7, "note_resources": []}}]
}}]
}"#;

pub fn engine() -> settings::Engine {
    let document = jig_protocol_config::document(
        ENGINE,
        &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 32 },
    )
    .expect("engine document");
    settings::Engine {
        jig: jig_protocol_config::engine(&document, 0).expect("typed jig engine"),
        files: settings::Files { entries: skein_lib::List::with_capacity(0) },
        forges: Box::new([settings::Forge {
            number: 1,
            kind: settings::Kind::Forgejo,
            api: jig_protocol_config::Address::Host { name: Box::from(&b"forge.example"[..]), port: 443 },
            transport: settings::Transport::Plaintext,
            git: Box::from(&b"https://forge.example"[..]),
            api_token: Secret::new(Box::from(&b"api-token"[..])),
            user: 42,
            identity: 1,
        }]),
        identities: Box::new([settings::Identity {
            number: 1,
            token: Secret::new(Box::from(&b"forge-token-1234"[..])),
            login: Box::from(&b"temper"[..]),
            author: settings::Author {
                name: Box::from(&b"temper"[..]),
                email: Box::from(&b"temper@forge.example"[..]),
            },
        }]),
        landing: temper_engine_domain::LandingPolicy {
            deployment: Box::new([]),
            projects: skein_lib::Map::with_capacity(0),
        },
        charter: 1,
        limits: limits::profile(limits::Profile::Small),
    }
}

pub const OWNER: Token = Token::new(91);
pub fn routes() -> crate::store::route::Routes {
    let own = limits::profile(limits::Profile::Small);
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let root = limits::engine(&jig, &own).expect("root limits");
    let bounds = temper_protocol_store::bounds(&root.forge).expect("family bounds");
    crate::store::route::Routes::new(2, &root, bounds)
}

pub fn task_record(number: u64) -> jig_domain_tasks::TaskRecord {
    use jig_domain_tasks as tasks;
    use skein_lib::Wall;
    tasks::TaskRecord {
        label: 0,
        flags: tasks::Flags::PLAIN,
        last_activity: Wall::EPOCH,
        ended_below: tasks::Endings::ZERO,
        question_lines: Box::new([]),
        hold: 0,
        proposal_rows: Box::new([]),
        instructions: None,
        refused: Box::new([]),
        grace: None,
        created_at: Wall::EPOCH,
        changed: Wall::EPOCH,
        phase_since: Wall::EPOCH,
        run_since: None,
        retry_notice: None,
        ended_at: None,
        revision: 0,
        revisions: tasks::Revisions { hold: 0, spec: 0, person: 0 },
        milestone: 0,
        activity: 0,
        due_reason: tasks::Why::Made,
        narrowing: false,
        result_position: 0,
        last_reply: 0,
        replies: Box::new([]),
        escalation: tasks::Escalation::Unheld { revision: 0 },
        proposal: None,
        result_proposal: false,
        number,
        project: 1,
        requester: tasks::Party::Deployment { project: 1 },
        run_spent: 0,
        run_reserved: 0,
        root: number,
        depth: 0,
        executor: tasks::Executor::Agent { charter: 1 },
        taken_by: None,
        person_answer: None,
        person_wait: None,
        result_followups: Box::new([]),
        tracked: None,
        recurring: None,
        spec: tasks::Spec { words: Box::new([]), parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: 32 },
        authority: tasks::Authority {
            tools: tasks::Tools(0),
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 1, depth: 0 },
            budget: tasks::Budget { spend: 1, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: 1, period: 1 },
        waiting_on: Box::new([]),
        holdings: Box::new([]),
        holds_taken: false,
        hold_wait_since: None,
        dependencies: Box::new([]),
        delegates: Box::new([]),
        references: Box::new([]),
        questions: Box::new([]),
        subscriptions: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        turn: 0,
        last_message: 0,
        inbox: Box::new([]),
        saved: Box::new([]),
        ever_turned: false,
        made: 1,
        attempt: 0,
        last_answer: None,
        tries: tasks::Tries::NONE,
        invalid_result: None,
        refusals: 0,
        phase: tasks::Phase::Waiting,
    }
}

pub fn store_limits() -> (temper_engine_domain::Limits, jig_domain_core::Limits, jig_protocol_store::Limits) {
    let own = limits::profile(limits::Profile::Small);
    let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
    let root = limits::engine(&jig, &own).expect("root limits");
    (root, jig.core, jig.store)
}

pub fn memory_store() -> crate::store::Store {
    let (root, core, profile) = store_limits();
    memory_store_for(&root, &core, &profile)
}
fn memory_store_for(
    root: &temper_engine_domain::Limits,
    core: &jig_domain_core::Limits,
    profile: &jig_protocol_store::Limits,
) -> crate::store::Store {
    let limits = crate::store::limits(root, core, &root.forge, profile).expect("checked store limits");
    let mut store = crate::store::Store::new(root, &limits);
    store.component = jig_protocol_store::Store::over(
        &limits,
        temper_protocol_store::application(&root.forge).expect("application sizes"),
        jig_protocol_store::Stores {
            store: skein_kv::Store::memory(&limits.store),
            secrets: skein_kv::Store::memory(&limits.secrets),
        },
    );
    store.ask(jig_protocol_store::Request::Open {
        owner: OWNER,
        root: jig_protocol_store::Root { token: OWNER, fd: skein_io::kernel::Fd::new(1) },
    });
    let Some(jig_protocol_store::Event::Opened { .. }) = store.above.pop() else { panic!("store opened") };
    store.opened = true;
    store
}

pub fn landed_row() -> temper_engine_domain_forge::Stored {
    temper_engine_domain_forge::Stored::Landed { commit: [9; 32], task: 3 }
}

pub fn commit(store: &mut crate::store::Store, number: u64, writes: Box<[temper_engine_domain::Write]>) {
    let mut all = skein_lib::List::with_capacity(u32::try_from(writes.len().checked_add(1).unwrap()).unwrap());
    all.push(temper_engine_domain::Write::Save(temper_engine_domain::Record::Core(jig_domain_core::Record::Core(
        jig_domain_core::CoreRecord::Deployment(jig_domain_core::Deployment {
            id: [19; 16],
            tasks: 0,
            people: 0,
            sign_ins: 0,
            messages: 0,
            runs: 0,
            calls: 0,
            connector_rows: 0,
            applications: 0,
            commits: number,
        }),
    ))))
    .unwrap();
    for write in writes {
        all.push(write).unwrap();
    }
    assert!(store.down(temper_engine_domain::StoreRequest::Commit { number, writes: all.into_boxed() }));
}

pub fn load(
    store: &mut crate::store::Store,
    range: temper_engine_domain::Range,
) -> Box<[temper_engine_domain::Record]> {
    assert!(store.down(temper_engine_domain::StoreRequest::Load {
        owner: OWNER,
        range,
        after: None,
        most: 4,
        bytes: 65_536,
    }));
    for _ in 0_u32..store.above.len() {
        crate::store::route::up(&mut store.routes, store.above.pop().expect("store event"), &mut store.routed);
    }
    let Some(crate::store::route::Up::Root(event)) = store.routed.pop() else { panic!("loaded root event") };
    let temper_engine_domain::Event::Store(temper_engine_domain::StoreInput::Loaded { rows, next: None, .. }) = *event
    else {
        panic!("one whole loaded page")
    };
    assert!(store.routed.is_empty());
    rows
}

pub const ACCOUNT: &[u8] = br#"{"access_token":"test-token","expires_at":120,"account_id":"acc"}"#;
pub fn seeds() -> settings::Seeds {
    settings::Seeds { deployment: [7; 16], seed: 7, crypto: Secret::new(Box::new([7; 32])) }
}

pub struct AccountStand {
    pub accounts: crate::accounts::Accounts,
    pub values: crate::values::Values,
}

pub fn accounts_with(record: &[u8]) -> AccountStand {
    let config = engine().jig;
    let mut entries = skein_lib::List::with_capacity(1);
    entries.push((Box::from(&b"accounts/chat.json"[..]), Secret::new(record.into()))).unwrap();
    account_stand(&config, &settings::Files { entries }, None)
}

pub fn missing_account() -> AccountStand {
    let mut config = engine().jig;
    config.llm.accounts[0].source = jig_protocol_config::Source::SignIn(jig_protocol_config::SignIn {
        client: Box::from(&b"client"[..]),
        secret: None,
        authorization: Box::from(&b"http://127.0.0.1:7102/authorize"[..]),
        token: Box::from(&b"http://127.0.0.1:7102/token"[..]),
        endpoint: jig_protocol_config::Address::Addr(skein_io::Addr::Ip(core::net::SocketAddr::from((
            [127, 0, 0, 1],
            7102,
        )))),
        transport: jig_protocol_config::Transport::Plaintext,
        redirect: Box::from(&b"http://127.0.0.1:7103/callback"[..]),
        scope: Box::from(&b"read"[..]),
    });
    let mut issuers = skein_lib::List::with_capacity(1);
    issuers
        .push((
            1,
            skein_oauth_accounts::Endpoint {
                address: core::net::SocketAddr::from(([127, 0, 0, 1], 7102)),
                transport: skein_oauth_accounts::Transport::Plaintext,
            },
        ))
        .unwrap();
    account_stand(
        &config,
        &settings::Files { entries: skein_lib::List::with_capacity(0) },
        Some(crate::AccountsRuntime {
            root: skein_io::kernel::Fd::new(10),
            effective_user: 1000,
            issuers,
            trust_roots: 0,
            trust_root_bytes: 0,
        }),
    )
}
fn account_stand(
    config: &jig_protocol_config::Engine,
    files: &settings::Files,
    runtime: Option<crate::AccountsRuntime>,
) -> AccountStand {
    AccountStand {
        accounts: crate::accounts::Accounts::new(config, files, &seeds(), runtime, 64).expect("configured account"),
        values: crate::values::Values::new(
            config.limits.core.accounts.accounts.checked_add(1).unwrap(),
            config.limits.driver.client.document.token_bytes,
        ),
    }
}

pub fn want(stand: &mut AccountStand, account: u32) -> Box<[temper_engine_domain::Event]> {
    use skein_lib::{Queue, Time, Wall};
    let mut events = Queue::with_capacity(8);
    let wall = Wall::from_nanos(1_000_000_000);
    stand.accounts.iterate(Time::ZERO, wall, &mut stand.values, &mut events);
    stand.accounts.request(jig_domain_accounts::Request::Component(jig_domain_accounts::ToComponent::Want { account }));
    stand.accounts.iterate(Time::ZERO, wall, &mut stand.values, &mut events);
    let mut out = skein_lib::List::with_capacity(events.len());
    for _ in 0_u32..events.len() {
        out.push(events.pop().unwrap()).unwrap();
    }
    out.into_boxed()
}

pub fn lent(stand: &AccountStand, account: u32, generation: u64) -> &Secret {
    &stand.values.accounts.value(account, generation, skein_lib::Time::ZERO).expect("lent account").token
}

pub fn root() -> (temper_engine_domain::Domain, skein_lib::Env<temper_engine_domain::Limits>) {
    use temper_engine_domain as root;
    let settings = engine();
    let limits = limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    let config = root::Config {
        core: jig_protocol_config::core(&settings.jig, [7; 16], 7),
        landing: settings.landing,
        forge: temper_engine_domain_forge_client::Config {
            namespace: Box::new([7; 16]),
            writers: Box::new([temper_engine_domain_forge_client::Writer { forge: 1, author: 42 }]),
        },
        identities: Box::new([temper_engine_domain_forge::GitIdentity { forge: 1, credential: 1 }]),
        forge_charter: settings.charter,
    };
    (
        root::Domain::new(config, &limits),
        skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits },
    )
}

pub fn started() -> crate::Service {
    started_with_projects(&[(1, 100)])
}
pub fn started_with_projects(projects: &[(u32, u64)]) -> crate::Service {
    start_with_store(projects, None)
}
pub fn start_with_store(projects: &[(u32, u64)], stored: Option<crate::store::Store>) -> crate::Service {
    start_with_worker(projects, stored, None, 0)
}
fn start_with_worker(
    projects: &[(u32, u64)],
    stored: Option<crate::store::Store>,
    worker: Option<crate::Worker>,
    worker_bytes: u64,
) -> crate::Service {
    let mut settings = engine();
    settings.jig.limits.core.people.operations = 2;
    settings.jig.limits.core.tasks.tasks = 2;
    settings.jig.limits.core.tasks.project_tasks = 2;
    settings.jig.limits.core.tasks.tree_tasks = 2;
    settings.jig.limits.core.tasks.batch = 1;
    settings.jig.limits.core.tasks.holdings = 2;
    settings.jig.limits.core.tasks.run_inbox = 2;
    settings.jig.limits.core.people.people = 2;
    settings.jig.limits.core.people.sign_ins = 2;
    settings.jig.limits.core.people.pending = 2;
    settings.jig.limits.core.people.waiters = 2;
    settings.jig.limits.core.people.inbox_tasks = 4;
    settings.jig.limits.core.fleet.workers = 2;
    settings.jig.limits.core.fleet.attempts = 2;
    settings.jig.limits.core.fleet.calls = 2;
    settings.jig.limits.core.fleet.slots = 1;
    settings.limits.root.journal.deliveries = 128;
    settings.limits.root.journal.held = 384;
    settings.limits.root.journal.writes = 8192;
    settings.jig.limits.clients.connections = 2;
    settings.jig.limits.clients.streams = 2;
    settings.jig.limits.clients.watches_per_stream = 4;
    settings.jig.limits.clients.exchanges = 4;
    settings.jig.limits.clients.exchanges_per_client = 2;
    let first = settings.jig.projects.into_iter().next().unwrap();
    let mut configured = skein_lib::List::with_capacity(u32::try_from(projects.len()).unwrap());
    for (number, budget) in projects {
        let mut project = jig_protocol_config::Project {
            number: *number,
            roles: first.roles.clone(),
            templates: first.templates.clone(),
            policy: first.policy.clone(),
        };
        project.policy.period_spend = *budget;
        for role in &mut project.policy.roles {
            role.period_spend = *budget;
        }
        configured.push(project).unwrap();
    }
    settings.jig.projects = configured.into_boxed();
    settings.files.entries = skein_lib::List::with_capacity(4);
    settings
        .files
        .entries
        .push((b"charters/chat.txt".as_slice().into(), Secret::new(b"Answer briefly".as_slice().into())))
        .unwrap();
    settings.files.entries.push((b"accounts/chat.json".as_slice().into(), Secret::new(ACCOUNT.into()))).unwrap();
    for worker in &settings.jig.hosts.workers {
        settings.files.entries.push((worker.digest.clone(), Secret::new(WORKER_DIGEST.into()))).unwrap();
    }
    if worker.is_some() {
        settings.jig.limits.hub = worker_settings().jig.limits.hub;
        settings.limits.checkout = worker_limits().checkout;
        settings.files.entries.get_mut(2).unwrap().1 =
            Secret::new(b"8f7be71db3941de11249a5e3fba7750a41c7d9958083ef522cb79e9fd7822878".as_slice().into());
    }
    let root = limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    let store = match stored {
        Some(store) => store,
        None => memory_store_for(&root, &settings.jig.limits.core, &settings.jig.limits.store),
    };
    let worker_io = if worker.is_some() { Some(worker_settings().jig.limits.io) } else { None };
    let io = crate::startup::checks::process_io(&settings.jig.limits.io, worker_io.as_ref()).unwrap();
    let mut service = crate::Service::new(settings, &seeds(), None, forge_runtime(), worker, worker_bytes, io).unwrap();

    service.engine.store = store;
    service
}
mod agent;
mod clients;
pub use agent::agent;
pub(super) mod system;
pub use system::{Kernel, hello, one_process, until_hosting};
pub fn turn(service: &mut crate::Service) {
    crate::iterate(service, skein_lib::Time::from_nanos(1), skein_lib::Wall::from_nanos(1));
    clients::kernel(service);
}
pub fn restart(service: &mut crate::Service) {
    for _ in 0_u32..256 {
        turn(service);
        if !service.work_pending(skein_lib::Time::from_nanos(1)) {
            break;
        }
    }
    assert!(service.engine.domain.ready(), "restart ready");
    assert_eq!(service.end(), None);
}
pub fn ready() -> crate::Service {
    let mut service = started();
    restart(&mut service);
    service
}
pub fn period(number: u64) -> temper_engine_domain::Event {
    temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Period {
        project: 1,
        period: number,
        budget: 100,
        next: None,
    })
}
pub fn get(service: &mut crate::Service, key: jig_domain_core::Key) -> Option<jig_domain_core::Record> {
    let engine = &mut service.engine;
    let owner = Token::new(991);
    engine.store.ask(jig_protocol_store::Request::Get { owner, key: jig_protocol_store::Get::Jig(key) });
    for _ in 0_u32..engine.store.above.len() {
        let event = engine.store.above.pop().unwrap();
        if let jig_protocol_store::Event::Got { row, .. } = event {
            return match row {
                Some(jig_protocol_store::Row::Jig(record)) => Some(record),
                None => None,
                Some(jig_protocol_store::Row::Application { .. }) => panic!("jig record"),
            };
        }
        engine.store.above.push(event);
    }
    panic!("store get answered")
}

pub fn wire_limits() -> temper_protocol_worker_wire::v1::Limits {
    crate::translate::limits(&limits::profile(limits::Profile::Small)).unwrap()
}
pub fn repository(tag: u32, owner: &[u8], name: &[u8]) -> temper_engine_domain::ForgeRepository {
    temper_engine_domain::ForgeRepository {
        tag,
        provider: temper_engine_domain_forge_client::api::Repository { forge: 1, repository: tag },
        name: name.into(),
        host: b"forge.example".as_slice().into(),
        owner: owner.into(),
        start: temper_engine_domain::ForgeStart::Base(b"main".as_slice().into()),
        push: Some(b"temper/task-7".as_slice().into()),
        identity: 1,
    }
}
pub fn assignment_with(repository: temper_engine_domain::ForgeRepository) -> temper_engine_domain::Assignment {
    let settings = engine();
    assignment_with_settings(repository, &settings)
}
pub fn assignment_with_settings(
    repository: temper_engine_domain::ForgeRepository,
    settings: &settings::Engine,
) -> temper_engine_domain::Assignment {
    let task = task_record(7);
    temper_engine_domain::Assignment {
        task: 7,
        attempt: 2,
        conversation: 2,
        charter: 1,
        run: Box::new(temper_engine_domain::RunCharter {
            policy: settings.jig.charters[0].policy.clone(),
            contract: task.contract,
            authority: task.authority,
            budget: 20,
        }),
        sections: Box::new([]),
        messages: Box::new([]),
        saved: Box::new([]),
        workstream: 7,
        workspace: temper_engine_domain::ForgeWorkspace { repositories: Box::new([repository]) },
        transcript: Box::new([]),
        answered: Box::new([]),
        settled: Box::new([]),
        grants: Box::new([]),
    }
}
pub fn values_with_identity(number: u32, token: &[u8]) -> crate::values::Values {
    let mut values = crate::values::Values::new(2, 4096);
    values.identities = skein_lib::Map::with_capacity(1);
    let mut identity = engine().identities.into_iter().next().unwrap();
    identity.token = Secret::new(token.into());
    let value = crate::translate::identity(&identity, &wire_limits()).unwrap();
    values.identities.insert(number, value).unwrap();
    values
}
pub fn application_grant(account: u32, generation: u64) -> jig_domain_host::Grant {
    jig_domain_host::Grant {
        table: jig_domain_host::GrantTable::Application,
        account,
        generation,
        valid: skein_lib::Duration::from_secs(30),
    }
}
pub fn decoded_identity(value: &Secret) -> temper_protocol_worker_wire::v1::Identity {
    temper_protocol_worker_wire::v1::Identity::decode(&wire_limits(), &mut skein_lib::Reader::new(value.expose()))
        .unwrap()
}

pub fn encoding(settings: &settings::Engine) -> crate::assignment::Encoding {
    let mut entries = skein_lib::List::with_capacity(u32::try_from(settings.jig.llm.endpoints.len()).unwrap());
    for endpoint in &settings.jig.llm.endpoints {
        entries
            .push(smith_protocol_channel::Endpoint {
                name: endpoint.name.clone(),
                number: endpoint.number,
                account: endpoint.account,
                dialect: endpoint.dialect,
            })
            .unwrap();
    }
    crate::assignment::Encoding {
        limits: wire_limits(),
        endpoints: smith_protocol_channel::Endpoints::new(entries),
        deployment: [7; 16],
    }
}
pub fn answer(ending: jig_domain_host::Ending) -> jig_domain_host::LinkUp {
    jig_domain_host::LinkUp::Answer {
        run: Token::new(7),
        attempt: Token::new(2),
        answer: jig_domain_host::Answer { read: None, refused: Box::new([]), turns: 0, spent: 0, ending },
    }
}

pub const WORKER_DIGEST: &[u8] = b"0000000000000000000000000000000000000000000000000000000000000000";
pub fn workers() -> crate::workers::Workers {
    let settings = engine();
    let mut entries = skein_lib::List::with_capacity(2);
    for worker in &settings.jig.hosts.workers {
        entries.push((worker.digest.clone(), Secret::new(WORKER_DIGEST.into()))).unwrap();
    }
    let encoding = encoding(&settings);
    let limits = crate::workers::limits(&settings).unwrap();
    crate::workers::Workers::new(&settings.jig, &settings::Files { entries }, encoding, settings.forges, &limits, 256)
        .unwrap()
}

pub fn settle_root(
    service: &mut crate::Service,
    delivered: &mut skein_lib::Queue<temper_engine_domain::Delivery>,
) -> Option<temper_engine_domain::Assignment> {
    use temper_engine_domain as root;
    let engine = &mut service.engine;
    let mut assignment = None;
    for _ in 0_u32..256 {
        if let Some(event) = engine.events.pop() {
            engine.step(event);
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        if engine.requests.is_empty() {
            engine.step(root::Event::Resume);
            root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        }
        for _ in 0..engine.requests.len() {
            let request = engine.requests.pop().unwrap();
            match request {
                root::Request::Worker(root::WorkerRequest::Deliver(root::Delivery::Assigned {
                    assignment: run,
                    ..
                })) => assignment = Some(run),
                root::Request::Worker(root::WorkerRequest::Deliver(delivery)) => delivered.push(delivery),
                root::Request::Party(_) => {}
                request @ root::Request::Worker(_) => crate::route::request(engine, request),
                request @ (root::Request::Store(_) | root::Request::Account(_) | root::Request::ToChild(_)) => {
                    crate::route::request(engine, request);
                }
                root::Request::Forge { .. } | root::Request::Busy(_) | root::Request::Drained | root::Request::Stop => {
                    panic!("unexpected chat request")
                }
            }
        }
        engine.store.iterate();
        for _ in 0..engine.store.above.len() {
            let event = engine.store.above.pop().unwrap();
            crate::store::route::up(&mut engine.store.routes, event, &mut engine.store.routed);
        }
        for _ in 0..engine.store.routed.len() {
            match engine.store.routed.pop().unwrap() {
                crate::store::route::Up::Root(event) => engine.events.push(*event),
                crate::store::route::Up::Ask(request) => engine.store.ask(request),
            }
        }
        if let Some(accounts) = &mut engine.accounts {
            accounts.iterate(engine.env.now, engine.env.wall, &mut engine.values, &mut engine.events);
        }
        engine.domain.reclaim();
        if engine.domain.quiescent() && engine.events.is_empty() && engine.store.idle() {
            return assignment;
        }
    }
    panic!("chat settled");
}
pub fn claimed_chat() -> (crate::Service, temper_engine_domain::Assignment) {
    use temper_engine_domain as root;
    let mut service = ready();
    service.engine.step(root::Event::Worker(root::WorkerInput::Link {
        channel: Token::new(7),
        link: root::Link::Hello(host::Hello {
            slots: 1,
            stop_bound: skein_lib::Duration::from_millis(100),
            workstreams: Box::new([]),
            hosting: Box::new([]),
            bounds: host::Bounds {
                models: Box::new([host::ModelBound { name: b"small".as_slice().into(), window: 8192, output: 1024 }]),
                turns: 8,
                spend: 1000,
                time: skein_lib::Duration::from_secs(60),
                waiting: skein_lib::Duration::from_secs(60),
                checks: skein_lib::Duration::from_secs(1),
                message_bytes: 512,
                inbox: 8,
                unacknowledged: 32,
            },
        }),
    }));
    settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32));
    service.engine.step(root::Event::Party(root::PartyInput::SignedIn {
        reply_to: skein_lib::ReplyTo::new(Token::new(100)),
        identity: jig_domain_people::Identity {
            key: jig_domain_people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32));
    service.engine.step(root::Event::Party(root::PartyInput::Ask {
        reply_to: skein_lib::ReplyTo::new(Token::new(101)),
        sign_in: 1,
        key: [31; 16],
        ask: jig_domain_people::Ask::StartChat { project: 1, words: b"question".as_slice().into() },
    }));
    let run = settle_root(&mut service, &mut skein_lib::Queue::with_capacity(32)).expect("chat assigned");
    (service, run)
}

mod worker;
mod worker_git;
pub use worker::{
    AGENT_JSON, NOW, TEMPER, application_value, assignment as worker_assignment, chat_assignment, identity,
    one_repository, repository_assignment, spawn, worker, worker_agent, worker_files, worker_limits, worker_settings,
};
pub use worker_git::{IDENTITY, WorkerStand, started_worker};
pub fn clients() -> crate::clients::Clients {
    crate::clients::Clients::new(&engine().jig, &seeds(), 64, &engine().limits.root.journal).unwrap()
}
mod startup;
pub use startup::{APPLICATION, document, files, parsed, startup_agent, with, without, worker_runtime};
pub fn forge_runtime() -> skein_lib::List<crate::ForgeRuntime> {
    let mut runtime = skein_lib::List::with_capacity(1);
    runtime
        .push(crate::ForgeRuntime {
            number: 1,
            address: core::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
            transport: temper_engine_protocol_forge::settings::Transport::Plaintext,
            trust_roots: 0,
            trust_root_bytes: 0,
        })
        .unwrap();
    runtime
}
mod forge;
pub use forge::{ForgeFace, hook};
