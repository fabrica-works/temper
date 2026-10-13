//! The worker's limits and its worst case (programming-model.md, section 6).

use alloc::boxed::Box;
use core::mem::size_of;

use crate::wire;
use jig_domain_host as host;
use skein_lib::{Duration, Id, Map, Queue, Slab, Token};
use smith_host_domain as agent;
use temper_worker_domain_checkout as checkout;

use crate::agents::{Bound, Call};
use crate::facts::Fact;
use crate::hand_out::Pending;
use crate::translate::SAVED;
use crate::workspace::{Items, Workspace};

/// The worker domain's limits (section 7), handed to every step read-only: its
/// child domains', each handed down to the one it bounds, and what the hub's
/// hello says (jig's `domain/hosts.md`, section 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub host: host::Limits,
    pub checkout: checkout::Limits,
    pub agent: agent::Limits,
    /// The bounds of what a run on this worker may be given: its agent's
    /// models and its runs' turns, spend, time and inbox, for the engine to
    /// place only what fits.
    pub bounds: host::Bounds,
    /// The longest the worker may take to stop a run it lost contact for
    /// (`domain/worker.md`, section 2; jig's `domain/hosts.md`, 6.6), which
    /// its configuration derives and the hello states. The engine's grace is
    /// strictly longer.
    pub stop_bound: Duration,
}

/// The bytes a message's sender and words take beyond themselves once smith
/// renders them, its separator (smith's `smith-domain-run`, `rendered_bytes`).
const SEPARATOR: u64 = 2;

/// The bytes of a workstream key: the holding task's number, big-endian.
const KEY: u32 = 8;

/// The most memory the domain holds under `limits`, in bytes (6.3), or `None`
/// if it does not fit a `u64` or the limits cannot be honoured: the child
/// domains' own, or limits under which one child domain would refuse what
/// another passes it within its own. The checkout's repositories, names and
/// conflicts must fit smith's directories, and a workstream's key its names;
/// the host's runs must each find a workspace and an agent (no more slots
/// than workspaces or agent processes); what goes down to a run must be what
/// an agent takes (an inbound event, rendered, no larger than its messages,
/// and no more held for a run until it is live, nor carried in its start,
/// than may wait for it in its agent), and smith's turns unacknowledged must
/// fit the host's reserve for a stopping run; a run's calls must fit the
/// host's; and a run's push message, as smith bounds a call, and the save's
/// must fit a commit message.
///
/// It is the child domains', plus what the root keeps: the agents it bound
/// and their calls in flight with their names and what they ask, the
/// hand-outs waiting and the ends waiting behind them, a record of each
/// workspace and the map that finds those being prepared, each run's
/// workspace items until the engine has its answer, the queues that hold what
/// each child domain emits in an entry point until it is routed, and the
/// facts. What the queued requests own is counted where they end up.
#[must_use]
pub fn worst_case(limits: &Limits) -> Option<u64> {
    let Limits { host: host_limits, checkout: checkout_limits, agent: agent_limits, .. } = limits;
    let fits = agent::check(agent_limits).is_ok()
        && checkout_limits.repositories <= agent::MAX_DIRECTORIES
        && checkout_limits.repositories <= agent_limits.directories
        && checkout_limits.conflicts <= agent_limits.conflicts
        && checkout_limits.path_bytes <= agent_limits.path_bytes
        && checkout_limits.name_bytes <= agent_limits.name_bytes
        && checkout_limits.name_bytes <= agent_limits.path_bytes
        && KEY <= checkout_limits.name_bytes
        && host_limits.accounts <= agent_limits.accounts
        && host_limits.slots <= checkout_limits.workspaces
        && host_limits.slots <= agent_limits.agents
        && agent_limits.turns <= host_limits.turns
        && agent_limits.unacknowledged_bytes <= host_limits.turn_queue_bytes
        && agent_limits.calls <= host_limits.run_calls
        && host_limits.event_bytes.checked_add(SEPARATOR)? <= agent_limits.message_bytes
        && host_limits.held <= agent_limits.messages
        && host_limits.inbox <= agent_limits.messages
        && agent_limits.call_bytes <= u64::from(checkout_limits.message_bytes)
        && len(SAVED) <= u64::from(checkout_limits.message_bytes);
    if !fits {
        return None;
    }
    let counts = counts(limits)?;
    let children = host::worst_case(host_limits)?
        .checked_add(checkout::worst_case(checkout_limits)?)?
        .checked_add(agent::worst_case(agent_limits)?)?;
    let calls = agent_limits.agents.checked_mul(agent_limits.calls)?;
    let call = host_limits
        .event_bytes
        .checked_add(agent_limits.call_bytes)?
        .checked_add(u64::from(agent_limits.name_bytes))?;
    let bindings = Map::<Token, Bound>::worst_case(agent_limits.agents)?
        .checked_add(Map::<(Token, Token), Call>::worst_case(calls)?)?
        .checked_add(u64::from(calls).checked_mul(call)?)?
        .checked_add(Queue::<Pending>::worst_case(pending(limits)?)?)?
        .checked_add(Map::<Token, Box<[u8]>>::worst_case(agent_limits.agents)?)?
        .checked_add(u64::from(agent_limits.agents).checked_mul(u64::from(agent_limits.detail_bytes))?)?;
    let host_out = Queue::<host::Request>::worst_case(counts.host_out)?;
    let checkout_out = Queue::<checkout::Request>::worst_case(counts.checkout_out)?;
    let agent_out = Queue::<agent::Output>::worst_case(counts.agent_out)?;
    let facts = Queue::<Fact>::worst_case(facts(limits)?)?;
    children
        .checked_add(bindings)?
        .checked_add(workspace_memory(limits)?)?
        .checked_add(host_out)?
        .checked_add(checkout_out)?
        .checked_add(agent_out)?
        .checked_add(facts)
}

/// The root's records of workspaces, and each run's workspace items until
/// the engine has its answer: a slot's, and one more for an assignment the
/// hub refuses in the step it comes in.
fn workspace_memory(limits: &Limits) -> Option<u64> {
    let host_limits = &limits.host;
    let checkout_limits = &limits.checkout;
    let slots = host_limits.slots;
    let name = u64::from(checkout_limits.name_bytes);
    let conflicts = u64::from(checkout_limits.conflicts)
        .checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?.checked_add(u64::from(checkout_limits.path_bytes))?)?;
    let root = u64::try_from(size_of::<Box<[u8]>>().checked_add(size_of::<bool>())?)
        .ok()?
        .checked_add(name)?
        .checked_add(u64::try_from(size_of::<checkout::Conflicts>()).ok()?)?
        .checked_add(conflicts)?;
    let workspaces = Slab::<Workspace>::worst_case(slots)?
        .checked_add(Map::<Token, Id<Workspace>>::worst_case(slots)?)?
        .checked_add(u64::from(slots).checked_mul(u64::from(checkout_limits.repositories).checked_mul(root)?)?)?;
    let staged = slots.checked_add(1)?;
    let item = u64::try_from(size_of::<wire::Repository>()).ok()?.checked_add(name.checked_mul(4)?)?;
    let feedback = u64::try_from(size_of::<wire::Landing>()).ok()?.checked_add(conflicts)?;
    let per = u64::from(checkout_limits.repositories)
        .checked_mul(item.checked_add(feedback)?.checked_add(4)?)?
        .checked_add(Map::<u32, [u8; 32]>::worst_case(checkout_limits.repositories)?)?
        .checked_add(name)?
        .checked_add(agent::Delivered::worst_case())?;
    let items = Slab::<Items>::worst_case(staged)?
        .checked_add(Map::<(Token, Token), Id<Items>>::worst_case(staged)?)?
        .checked_add(u64::from(staged).checked_mul(per)?)?;
    workspaces.checked_add(items)
}

fn len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("a length fits in a u64")
}

/// Facts kept until the loop drains them: as many as the child domains keep.
pub(crate) fn facts(limits: &Limits) -> Option<u32> {
    limits.host.facts.checked_add(limits.checkout.facts)?.checked_add(limits.agent.facts)
}

/// Hand-outs waiting at once: for each agent, its turns not yet taken, its
/// calls, and its last word.
pub(crate) fn pending(limits: &Limits) -> Option<u32> {
    limits.agent.agents.checked_mul(limits.agent.turns.checked_add(limits.agent.calls)?.checked_add(1)?)
}

/// The hub steps in one entry point that emit requests, at most. An entry
/// point concerns one hosted run, its agent and its workspace (or, the
/// hello, none). It steps the hub itself at most three times (an encoded call
/// name: called, then withdrawn; an encoded last word: finished; and an end
/// that waited behind them). The run's agent tells the hub at most three
/// things that lead to requests, its steps together: what it took (a turn
/// or a call goes out by hand-out first), a fault and that it has gone; the
/// names it left unread and the messages it refused lead to none. Its
/// workspace tells the hub at most two: a prepare, push or save ended, and
/// one refused at once.
const EMITTING: u32 = 8;

/// What an entry point emits and routes, at most, under `limits`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Counts {
    /// Room for what the hub emits in an entry point: the emitting steps'
    /// requests, and room for one more step.
    pub(crate) host_out: u32,
    /// Room for what smith emits: one step's, and one more taken as one of
    /// its notices is routed (a delivery refused at once).
    pub(crate) agent_out: u32,
    /// Room for what the checkout emits: one step's, and the release of a
    /// workspace whose prepare failed.
    pub(crate) checkout_out: u32,
    /// The child domains' requests routed in an entry point.
    pub(crate) routed: u32,
    /// The requests an entry point emits.
    pub(crate) out: u32,
}

/// The counts of an entry point, or `None` if they do not fit a `u32`. Each
/// of the hub's requests leads to at most one capability step, and so does
/// the entry; a failed prepare's release is one more.
pub(crate) fn counts(limits: &Limits) -> Option<Counts> {
    let hub = host::max_out(&limits.host);
    let smith = agent::max_out(&limits.agent);
    let git = checkout::MAX_OUT;
    let requests = EMITTING.checked_mul(hub)?;
    let capabilities = requests.checked_add(2)?;
    let routed = capabilities.checked_mul(smith.checked_add(git)?)?.checked_add(requests)?;
    let replays = limits.host.slots.checked_mul(limits.host.turns.checked_mul(2)?.checked_add(1)?)?;
    Some(Counts {
        host_out: requests.checked_add(hub)?,
        agent_out: smith.checked_mul(2)?,
        checkout_out: git.checked_mul(2)?,
        routed,
        // What is routed out, the hello's replays, the activity drained, the
        // root's own refusal and its stop.
        out: routed.checked_add(replays)?.checked_add(limits.host.told)?.checked_add(2)?,
    })
}

/// The most requests an entry point emits per call under `limits`, which
/// [`worst_case`] accepts. The loop reserves this much room in `out` before
/// calling it.
#[must_use]
pub fn max_out(limits: &Limits) -> u32 {
    counts(limits).expect("worst_case accepted the limits").out
}
