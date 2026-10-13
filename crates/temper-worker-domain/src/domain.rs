//! The domain's state and its entry points (section 3). Each hands what it is
//! given to the child domain it is for, then completes the hand-offs between
//! the child domains (4.5) before it returns, routing what is for the
//! protocol layer out.
//!
//! The host is the hub, and its capabilities answer it: a hand-off goes from a
//! capability to the host, from the host to a capability, and at most once
//! more back and forth, when a capability answers a request of the host at
//! once (smith refuses a message, or its agent has gone once the hub answers
//! its last call; the checkout refuses a prepare) and the host answers the
//! engine, replies to the run or releases the workspace, which leads back to
//! it no more. smith's payloads go out by hand-out and come back as events of
//! their own, so no hand-off waits on them. [`crate::max_out`] follows from the
//! child domains' along that chain (the `limits` module).

use alloc::boxed::Box;

use jig_domain_host as host;
use skein_lib::{Env, Id, Map, Queue, Slab, Time, Token};
use smith_host_domain as agent;
use temper_worker_domain_checkout as checkout;

use crate::agents::{Bound, Call};
use crate::boundary::{Event, Request};
use crate::facts::Fact;
use crate::hand_out::Pending;
use crate::limits::{self, Limits};
use crate::route;
use crate::workspace::{Items, Workspace};

/// The worker domain's state: its child domains', what it keeps of each
/// agent, call, hand-out and workspace between them, and room for what the
/// child domains emit within an entry point.
#[derive(Debug)]
pub struct Domain {
    pub(crate) host: host::Domain,
    pub(crate) checkout: checkout::Domain,
    pub(crate) agent: agent::Domain,
    /// The agents smith started, by their runs' tokens.
    pub(crate) agents: Map<Token, Bound>,
    /// The runs' calls in flight, by their runs' tokens and smith's callbacks.
    pub(crate) calls: Map<(Token, Token), Call>,
    /// The hand-outs out, oldest first.
    pub(crate) pending: Queue<Pending>,
    /// The ends of agents that went with hand-outs still out, with their
    /// detail.
    pub(crate) gone: Map<Token, Box<[u8]>>,
    /// The workspaces the host asked for, until the checkout has released
    /// them.
    pub(crate) workspaces: Slab<Workspace>,
    /// Each run's workspace items, until the engine has its answer.
    pub(crate) items: Slab<Items>,
    pub(crate) items_by_run: Map<(Token, Token), Id<Items>>,
    /// Workspaces being prepared, by the host's token for their run.
    pub(crate) preparing: Map<Token, Id<Workspace>>,
    /// What each child domain emits in an entry point, until it is routed.
    /// Empty between entry points.
    pub(crate) host_out: Queue<host::Request>,
    pub(crate) checkout_out: Queue<checkout::Request>,
    pub(crate) agent_out: Queue<agent::Output>,
    /// Told to shut down, and not stopped yet.
    pub(crate) stopping: bool,
    facts: Queue<Fact>,
    lost: u64,
}

impl Domain {
    /// A domain with room for `limits`, which [`crate::worst_case`] accepts.
    #[must_use]
    pub fn new(limits: &Limits) -> Domain {
        let counts = limits::counts(limits).expect("worst_case accepted the limits");
        let slots = limits.host.slots;
        let agents = limits.agent.agents;
        Domain {
            host: host::Domain::new(&limits.host),
            checkout: checkout::Domain::new(&limits.checkout),
            agent: agent::Domain::new(&limits.agent),
            agents: Map::with_capacity(agents),
            calls: Map::with_capacity(agents.checked_mul(limits.agent.calls).expect("worst_case accepted the calls")),
            pending: Queue::with_capacity(limits::pending(limits).expect("worst_case accepted the hand-outs")),
            gone: Map::with_capacity(agents),
            workspaces: Slab::with_capacity(slots),
            items: Slab::with_capacity(slots.saturating_add(1)),
            items_by_run: Map::with_capacity(slots.saturating_add(1)),
            preparing: Map::with_capacity(slots),
            host_out: Queue::with_capacity(counts.host_out),
            checkout_out: Queue::with_capacity(counts.checkout_out),
            agent_out: Queue::with_capacity(counts.agent_out),
            stopping: false,
            facts: Queue::with_capacity(limits::facts(limits).expect("worst_case accepted the limits")),
            lost: 0,
        }
    }

    /// The host child domain, for a world to look at.
    #[must_use]
    pub const fn host(&self) -> &host::Domain {
        &self.host
    }

    /// The checkout child domain, for a world to look at.
    #[must_use]
    pub const fn checkout(&self) -> &checkout::Domain {
        &self.checkout
    }

    /// The agent child domain, for a world to look at.
    #[must_use]
    pub const fn agent(&self) -> &agent::Domain {
        &self.agent
    }

    /// Workspaces the host asked for that the checkout has not released,
    /// released ones included until they are reclaimed.
    #[must_use]
    pub const fn workspaces(&self) -> u32 {
        self.workspaces.len()
    }

    /// Turns the hub keeps until the engine acknowledges them.
    #[must_use]
    pub fn retained_turns(&self) -> u32 {
        self.host.retained_turns()
    }

    /// Answers the hub keeps until the engine acknowledges them.
    #[must_use]
    pub fn kept_answers(&self) -> u32 {
        self.host.kept_answers()
    }

    /// Turns given up by a worker shutting down with the engine out of reach
    /// past the grace, since the domain was made.
    #[must_use]
    pub const fn abandoned(&self) -> u64 {
        self.host.turns_abandoned()
    }

    /// When the earliest alarm falls due: an agent's.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.agent.next_deadline()
    }

    /// Whether an alarm is due at `now`. The loop calls [`fire`] while one is.
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        self.agent.is_due(now)
    }

    /// Whether the domain has work of its own: the host has runs to cancel,
    /// one at a time, or the worker, shutting down, has nothing left and is
    /// to say so. While it has, the loop calls [`resume`] at the start of the
    /// domain's stage, before its input events, and after the reclaim point.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.host.is_ready() || self.stop_ready()
    }

    /// Shutting down, with no run, agent, hand-out or workspace left.
    fn stop_ready(&self) -> bool {
        self.stopping
            && self.host.hosted() == 0
            && self.agent.agents() == 0
            && self.pending.is_empty()
            && self.workspaces.is_empty()
    }

    /// The oldest fact not drained yet, a child domain's.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }

    /// How many facts were dropped for want of room, the child domains'
    /// included.
    #[must_use]
    pub fn facts_lost(&self) -> u64 {
        self.lost
            .saturating_add(self.host.facts_lost())
            .saturating_add(self.checkout.facts_lost())
            .saturating_add(self.agent.facts_lost())
    }

    /// How much of the runs' activity was dropped for want of room.
    #[must_use]
    pub const fn activity_lost(&self) -> u64 {
        self.host.activity_lost()
    }

    /// The reclaim point: frees what closed in this iteration.
    pub fn reclaim(&mut self) {
        self.host.reclaim();
        self.checkout.reclaim();
        self.agent.reclaim();
        self.workspaces.reclaim();
        self.items.reclaim();
    }
}

/// Keeps `fact` if there is room for it, and counts it otherwise.
fn keep(domain: &mut Domain, fact: Fact) {
    if domain.facts.try_push(fact).is_err() {
        domain.lost = domain.lost.saturating_add(1);
    }
}

/// Handles one event, emitting at most [`crate::max_out`] requests.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    route::event(domain, env, event, out);
    settle(domain, env, out);
}

/// Fires an agent's alarm due at `env.now`, if there is one, emitting at most
/// [`crate::max_out`] requests. A stage fires its alarms after its input events, so
/// progress that arrived in the same iteration wins over a deadline that
/// passed while the loop waited.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.agent.is_due(env.now) {
        assert!(
            domain.agent_out.room() >= agent::max_out(&env.limits.agent),
            "an entry point steps the agents no more than its bound"
        );
        agent::fire(&mut domain.agent, &route::agent_env(env), &mut domain.agent_out);
    }
    settle(domain, env, out);
}

/// Cancels one run the host has on its ready list, if it has one, emitting at
/// most [`crate::max_out`] requests; or, shutting down with nothing left, stops.
pub fn resume(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    assert!(
        domain.host_out.room() >= host::max_out(&env.limits.host),
        "an entry point steps the host within its bound"
    );
    host::resume(&mut domain.host, &route::host_env(env), &mut domain.host_out);
    settle(domain, env, out);
}

/// Completes the hand-offs, sends the runs' activity on, says the worker has
/// stopped once it has, then gathers the facts.
fn settle(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    route::hand_off(domain, env, out);
    for _ in 0..env.limits.host.told {
        let Some(activity) = domain.host.pop_activity() else {
            break;
        };
        out.push(Request::Link(activity));
    }
    if domain.stop_ready() {
        out.push(Request::Stop);
        domain.stopping = false;
    }
    gather(domain, &env.limits);
}

/// Drains the child domains' facts into the domain's own queue, counting what
/// does not fit.
fn gather(domain: &mut Domain, limits: &Limits) {
    for _ in 0..limits.host.facts {
        let Some(fact) = domain.host.pop_fact() else {
            break;
        };
        keep(domain, Fact::Host { fact });
    }
    for _ in 0..limits.checkout.facts {
        let Some(fact) = domain.checkout.pop_fact() else {
            break;
        };
        keep(domain, Fact::Checkout { fact });
    }
    for _ in 0..limits.agent.facts {
        let Some(fact) = domain.agent.pop_fact() else {
            break;
        };
        keep(domain, Fact::Agent { fact });
    }
}
