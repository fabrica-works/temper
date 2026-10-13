//! The unit tests' worker: limits, an assignment and a harness that steps the
//! root and collects what it emits.

use crate::{Domain, Event, Limits, Request, agent, checkout, host, max_out, step, wire, worst_case};
use alloc::boxed::Box;
use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall};

pub const RUN: Token = Token::new(31);
pub const ATTEMPT: Token = Token::new(932);
pub const WORKSTREAM: u64 = 31;

/// Two runs of up to two repositories each, two turns and two calls in flight
/// at a time, with smith's limits around the hub's.
pub fn limits() -> Limits {
    Limits {
        host: host::Limits {
            slots: 2,
            accounts: 4,
            charter_bytes: 8_192,
            transcript_bytes: 1_024,
            delivery_evidence_bytes: 256,
            turn_bytes: 64,
            outcome_bytes: 4_096,
            detail_bytes: 32,
            held: 2,
            inbox: 2,
            event_bytes: 96,
            run_calls: 2,
            facts: 64,
            told: 16,
            turns: 2,
            turn_queue_bytes: 128,
        },
        checkout: checkout::Limits {
            workspaces: 4,
            repositories: 2,
            name_bytes: 32,
            message_bytes: 512,
            conflicts: 0,
            path_bytes: 0,
            remote_timeout: Duration::from_secs(60),
            local_timeout: Duration::from_secs(10),
            facts: 64,
        },
        agent: agent::Limits {
            agents: 2,
            directories: 8,
            conflicts: 0,
            path_bytes: 64,
            name_bytes: 64,
            accounts: 4,
            charter_bytes: 8_192,
            transcript_bytes: 1_024,
            answered_bytes: 4_096,
            message_bytes: 128,
            messages: 2,
            calls: 2,
            call_bytes: 512,
            answer_bytes: agent::Delivered::worst_case(),
            turns: 2,
            turn_bytes: 64,
            unacknowledged_bytes: 128,
            fact_bytes: 64,
            outcome_bytes: 4_096,
            detail_bytes: 32,
            spawn_timeout: Duration::from_secs(1),
            no_progress: Duration::from_secs(120),
            silence: Duration::from_secs(60),
            long_span: Duration::from_secs(60),
            long_open: 8,
            wall_time: Duration::from_secs(300),
            cancel_grace: Duration::from_secs(5),
            exit_grace: Duration::from_secs(5),
            teardown: Duration::from_secs(1),
            kill_after: Duration::from_secs(2),
            facts: 64,
        },
        bounds: host::Bounds {
            models: Box::new([]),
            turns: 8,
            spend: 100,
            time: Duration::from_secs(60),
            waiting: Duration::from_secs(60),
            checks: Duration::from_secs(60),
            message_bytes: 96,
            inbox: 2,
            unacknowledged: 2,
        },
        stop_bound: Duration::from_secs(20),
    }
}

/// jig's assignment of the run, in the workstream [`WORKSTREAM`] if it has
/// a workspace, with the grants `grants`.
pub fn assignment(workspace: bool, grants: Box<[host::Grant]>) -> host::Assignment {
    let workspace =
        if workspace { Some(host::Workspace { workstream: WORKSTREAM, items: Token::new(0) }) } else { None };
    host::Assignment {
        messages: Box::new([]),
        turns: Box::new([]),
        answered: Box::new([]),
        assignment: host::RunAssignment {
            identity: [3; 16],
            run: RUN,
            attempt: ATTEMPT,
            workspace,
            save: false,
            charter: Box::from(&b"charter"[..]),
            grants,
        },
    }
}

/// Read-only repositories named `names`, each starting from `main`.
pub fn items(names: &[&[u8]]) -> wire::Workspace {
    let mut repositories = List::with_capacity(u32::try_from(names.len()).expect("a few names"));
    for (tag, name) in names.iter().enumerate() {
        let repository = wire::Repository {
            tag: u32::try_from(tag).expect("a few names"),
            name: Box::from(*name),
            remote: Box::from(&b"org/app"[..]),
            start: wire::Start::Base { branch: Box::from(&b"main"[..]) },
            access: wire::Access::ReadOnly,
            identity: 7,
        };
        repositories.push(repository).expect("room for each name");
    }
    wire::Workspace { repositories: repositories.into_boxed() }
}

/// The root's refusal of the run as an invalid start, for `why`.
pub fn refused(why: &[u8]) -> Request {
    let refusal = host::Refusal::Invalid(host::Invalid::Start { why: Box::from(why) });
    Request::Answer {
        run: RUN,
        attempt: ATTEMPT,
        answer: host::Answer {
            read: None,
            refused: Box::new([]),
            turns: 0,
            spent: 0,
            ending: host::Ending::Refused(refusal),
        },
        work: wire::Work { landed: Box::new([]), saved: None },
        preparation: None,
    }
}

pub struct Harness {
    pub domain: Domain,
    pub env: Env<Limits>,
    pub out: Queue<Request>,
}

impl Harness {
    pub fn new() -> Harness {
        let limits = limits();
        assert!(worst_case(&limits).is_some());
        Harness {
            domain: Domain::new(&limits),
            out: Queue::with_capacity(max_out(&limits)),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        }
    }

    /// Steps the root with `event`, then reclaims, as the loop would.
    pub fn step(&mut self, event: Event) -> Box<[Request]> {
        step(&mut self.domain, &self.env, event, &mut self.out);
        let mut rows = List::with_capacity(max_out(&self.env.limits));
        while let Some(row) = self.out.pop() {
            rows.push(row).expect("reserved outputs");
        }
        self.domain.reclaim();
        rows.into_boxed()
    }

    /// The hub's start for the run, handed out to be decoded, after an
    /// assignment with no repository.
    pub fn decode(&mut self, grants: Box<[host::Grant]>) -> (Token, host::Start) {
        let rows = self.step(Event::Link(host::LinkDown::Assign { assignment: assignment(false, grants) }));
        for row in rows {
            if let Request::Decode { agent, run, start } = row {
                assert_eq!(run, RUN);
                return (agent, *start);
            }
        }
        panic!("the start is handed out")
    }
}

/// A grant of `account` in `table`.
pub const fn grant(table: host::GrantTable, account: u32) -> host::Grant {
    host::Grant { table, account, generation: 1, valid: Duration::from_secs(60) }
}

/// smith's typed delivery fields, each a name and its value.
pub fn fields(named: &[(&[u8], &[u8])]) -> agent::Fields {
    let mut fields = List::with_capacity(u32::try_from(named.len()).expect("a few fields"));
    for (name, value) in named {
        let field = smith_domain_run::outcome::Field { name: Box::from(*name), value: Box::from(*value) };
        fields.push(field).expect("room for each field");
    }
    let change = smith_domain_run::outcome::Change { fields: fields.into_boxed() };
    agent::Fields::new(change, 512).expect("bounded fields")
}
