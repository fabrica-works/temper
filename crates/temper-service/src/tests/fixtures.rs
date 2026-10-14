use crate::{limits, settings};
use skein_lib::Secret;

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
"chats": {"charter": 1, "close_grace_ms": 5000},
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
"policy": {"escalation_role": 0, "period_spend": 100, "projections": [], "requirements": [],
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

pub const OWNER: skein_lib::Token = skein_lib::Token::new(91);
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
        ended_below: tasks::Endings::ZERO,
        question_lines: Box::new([]),
        hold: 0,
        proposal_rows: Box::new([]),
        instructions: None,
        refused: Box::new([]),
        chat_close: None,
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
