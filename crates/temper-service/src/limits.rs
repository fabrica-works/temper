//! The small profile and checked field overrides (configuration.md, section 7).
mod names;
use skein_lib::Duration;
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_client as client;
use temper_worker_domain_checkout as checkout;
/// Temper's profile for one person's machine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Profile {
    Small,
}
/// Root's own limits, beyond jig's child limits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Root {
    pub journal: engine::JournalLimits,
    pub loads: engine::loads::Limits,
    pub brief: engine::BriefLimits,
}
/// Temper's domain limits, each under its owner's prefix.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub forge_component: temper_engine_protocol_forge::Limits,
    pub root: Root,
    pub forge: forge::Limits,
    pub checkout: checkout::Limits,
    pub git_component: temper_worker_protocol_git::Limits,
}
/// Refusal of an override; the caller retains its name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    /// Unknown field path.
    Unknown,
    /// Value outside the field's representation.
    Range,
}
const CLIENT: client::Limits = client::Limits {
    // Logical calls retained, queued and in flight, from the engine forge world fixture.
    pending: 4,
    // Calls in flight, from the engine forge world fixture.
    calls: 2,
    // Requests allowed per rate window, from the engine forge world fixture.
    rate: 100,
    // Requests reserved for writes, from the engine forge world fixture.
    reserve: 0,
    // Rate window duration, from the engine forge world fixture.
    window: Duration::from_secs(10),
    // Owned bytes of one operation, from the engine forge world fixture.
    op_bytes: 512,
    // Decoded bytes of one answer: the small profile's 65,536-byte ceiling.
    answer_bytes: 65_536,
    // Rows in one answer page, from the engine forge world fixture.
    rows: 8,
    // Comment and review delivery stamps per resource, from the engine forge world fixture.
    inbox: 16,
    // Attempts of a read, from the engine forge world fixture.
    read_attempts: 3,
    // First read retry delay, from the engine forge world fixture.
    backoff: Duration::from_secs(1),
    // Maximum read retry delay, from the engine forge world fixture.
    backoff_max: Duration::from_secs(4),
    // Durable outbox entries, from the engine forge world fixture.
    entries: 8,
    // Attempts of a write, from the engine forge world fixture.
    write_attempts: 3,
    // Longest time a sent write may arrive, from the engine forge world fixture.
    lifetime: Duration::from_secs(10),
    // Allowance for clock movement, from the engine forge world fixture.
    clock_margin: Duration::from_secs(1),
    // Live forge resources, from the engine forge world fixture.
    resources: 8,
    // Repositories retained, from the engine forge world fixture.
    repositories: 2,
    // Normal resource polling interval, from the engine forge world fixture.
    poll: Duration::from_secs(5),
    // Maximum resource polling interval, from the engine forge world fixture.
    poll_max: Duration::from_secs(30),
    // Delay before a hinted read, from the engine forge world fixture.
    hinted: Duration::from_secs(1),
    // Delay before a slow read, from the engine forge world fixture.
    slow: Duration::from_secs(40),
    // Bound Forgejo's settling of mergeability and hinted CI after a move.
    settle: Duration::from_secs(30),
    // Facts awaiting drain, from the engine forge world fixture.
    facts: 16,
};
const FORGE: forge::Limits = forge::Limits {
    // Repositories retained, from the engine forge world fixture.
    repositories: 2,
    // Tasks participating in the forge, from the engine forge world fixture.
    tasks: 8,
    // Held task resources, from the engine forge world fixture.
    holds: 8,
    // Subscriptions to forge topics, from the engine forge world fixture.
    subscriptions: 8,
    // Durable outbox entries, from the engine forge world fixture.
    entries: 8,
    // At least jig small core.call_records: the root's worst case.
    named_calls: 32,
    // Live changes, from the engine forge world fixture.
    changes: 8,
    // Live issue projections, from the engine forge world fixture.
    issues: 8,
    // Held to the agent's two checkout repositories.
    resources_per_task: 2,
    // Paths retained by one subscription, from the engine forge world fixture.
    paths_per_subscription: 4,
    // Leaves 122 bytes for a git base within the checkout's 256 bytes.
    name_bytes: 64,
    // Output records reserved per forge step, from the engine forge world fixture.
    output: 80,
    // Facts awaiting drain, from the engine forge world fixture.
    facts: 16,
    // Adoptions in flight, from the engine forge world fixture.
    adoptions: 2,
    // Keyed application requests retained, from the engine forge world fixture.
    applications: 4,
    // Collaborators kept by the forge, from the engine forge world fixture.
    collaborators: 8,
    // Landing operations retained, from the engine forge world fixture.
    landings: 8,
    // Project judge tables, from the engine forge world fixture.
    judge_projects: 2,
    // Criteria in each judge table, from the engine forge world fixture.
    judge_criteria: 8,
    // Forge brief sections gathering at once, from the engine forge world fixture.
    brief_sections: 32,
    // Bytes of one gathered forge section, from the engine forge world fixture.
    brief_bytes: 4096,
    // Forge client calls, retries and outbox room.
    client: CLIENT,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        // Items in an issue plan, from the engine forge world fixture.
        plan_items: 8,
        // Milestones in an issue projection, from the engine forge world fixture.
        milestones: 8,
        // Bytes in an issue title, from the engine forge world fixture.
        title_bytes: 128,
        // Bytes in an issue body, from the engine forge world fixture.
        body_bytes: 512,
        // Bytes in an issue comment, from the engine forge world fixture.
        comment_bytes: 256,
        // Interval between issue projection checks, from the engine forge world fixture.
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        // Repair attempts, from the engine forge world fixture.
        repairs: 3,
        // Resolution attempts, from the engine forge world fixture.
        resolutions: 3,
        // Update attempts, from the engine forge world fixture.
        updates: 3,
        // Time before a change stalls, from the engine forge world fixture.
        stall: Duration::from_secs(30),
        // Gates retained by a change, from the engine forge world fixture.
        gates: 8,
        // Clean branch heads retained, from the engine forge world fixture.
        clean_heads: 8,
    },
    // Time to gather a landing queue, from the engine forge world fixture.
    queue_window: Duration::from_secs(30),
};
const CHECKOUT: checkout::Limits = checkout::Limits {
    // Cached or held checkout workspaces, from the checkout world fixture.
    workspaces: 3,
    // Held to the agent's two repository directories.
    repositories: 2,
    // Bytes in a checkout key, name, remote or branch: the 256-byte ceiling.
    name_bytes: 256,
    // Holds the agent's largest commit-message argument.
    message_bytes: 4096,
    // Conflicted paths per repository, from the checkout merges world.
    conflicts: 4,
    // Bytes per conflicted path: four NUL-terminated paths fit stdout_bytes 1,024.
    path_bytes: 255,
    // Time for an operation reaching a forge, from the checkout world fixture.
    remote_timeout: Duration::from_secs(60),
    // Time for a local checkout operation, from the checkout world fixture.
    local_timeout: Duration::from_secs(10),
    // Facts awaiting drain, from the checkout world fixture.
    facts: 256,
};
/// Root room follows the domain world; byte carriers take jig's small profile
/// bounds where its children require more than the world's reduced fixtures.
const ROOT: Root = Root {
    journal: engine::JournalLimits {
        // Commits awaiting durability, from the root world.
        commits: 3,
        // At least deliveries * 3: the root's worst case.
        held: 1536,
        // At least `route_bound` plus `call_records`: the root's worst case.
        writes: 12288,
        // At least jig small fleet.workers * fleet.max_out + 1: the root's worst case.
        deliveries: 512,
        // At least row_bound, including forge.client.answer_bytes: the root's worst case.
        transcript_bytes: 65_536,
        // At least jig small people.words: the root's worst case.
        result_bytes: 2048,
        // Run charter carrier in the root's worst case; jig small core.run_bytes.
        run_bytes: 16_384,
    },
    loads: engine::loads::Limits {
        // Concurrent loads, from the root world.
        loads: 2,
        // Rows in one page, from the root world.
        rows: 2,
        // Kept page bytes in loads' worst case; jig small core.load_bytes.
        bytes: 65_536,
        // At least loads.bytes: loads' worst case; jig small core.load_bytes.
        reply_bytes: 65_536,
        // Deep row admission bound for the root's largest forge.client.answer_bytes.
        transcript_bytes: 65_536,
    },
    brief: engine::BriefLimits {
        // Concurrent gathers, from the root world.
        briefs: 2,
        // Sections per gather, from the root world.
        sections: 3,
        // Parts per section, from the root world.
        parts: 2,
        // One section's read bytes, from the root world.
        read_bytes: 256,
        budgets: engine::BriefBudgets {
            // Task section's bytes, from the root world.
            task: 128,
            // Dependencies section's bytes, from the root world.
            dependencies: 128,
            // CI section's bytes, from the root world.
            ci: 128,
            // Reviews section's bytes, from the root world.
            reviews: 128,
            // Pull section's bytes, from the root world.
            pull: 128,
            // Attempts section's bytes, from the root world.
            attempts: 128,
            // Plan section's bytes, from the root world.
            plan: 128,
            // Notes section's bytes, from the root world.
            notes: 64,
        },
        // Assembled brief bytes, from the root world.
        brief_bytes: 384,
        // Time for a gather, from the root world.
        gather: Duration::from_secs(1),
    },
};
/// The standalone process's socket admission.
pub const SOCKETS: u32 = 48;

/// Constant small-profile values in each owner's limits.
#[must_use]
pub fn profile(profile: Profile) -> Limits {
    match profile {
        Profile::Small => Limits {
            forge_component: crate::forge::small(&FORGE),
            root: ROOT,
            forge: FORGE,
            checkout: CHECKOUT,
            git_component: GIT,
        },
    }
}
/// Set one dotted field name byte for byte.
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    names::apply(limits, name, value)
}
/// Build the root's limits from jig's children and temper's own room.
#[must_use]
pub fn engine(jig: &jig_protocol_config::Limits, limits: &Limits) -> Option<engine::Limits> {
    let root = engine::Limits {
        journal: limits.root.journal,
        loads: limits.root.loads,
        brief: limits.root.brief,
        tasks: jig.core.tasks,
        authority: jig.core.authority,
        people: jig.core.people,
        fleet: jig.core.fleet,
        accounts: jig.core.accounts,
        notes: jig.core.notes,
        documents: jig.core.documents,
        templates: jig.core.templates,
        call_records: jig.core.call_records,
        forge: limits.forge,
    };
    engine::worst_case(&root)?;
    Some(root)
}

const GIT: temper_worker_protocol_git::Limits = temper_worker_protocol_git::Limits {
    // One operation per cached workspace, plus the component's startup and sweep.
    operations: 3,
    // At least every conflict path including its terminating NUL.
    stdout_bytes: 1024,
    // The tail retained for classification and the checkout's diagnostic.
    stderr_bytes: 512,
    // Each stdout/stderr demand, bounded further by io's read size.
    read_bytes: 4096,
    // One page of trash entries held while sweeping.
    sweep_entries: 16,
    // Every name in that page at the kernel's longest name.
    sweep_bytes: 4080,
};

/// Limits decoded by the setup and box owners, kept with their values.
#[must_use]
pub fn handed_on(name: &[u8]) -> bool {
    name.starts_with(b"setup.") || name.starts_with(b"box.")
}

/// Server bounds follow jig's shipped small profile, with its HTTP intake bounds.
#[must_use]
pub fn server(jig: &jig_protocol_config::Limits) -> crate::server::Limits {
    crate::server::Limits {
        connections: 4,
        http: skein_http::server::Limits {
            head: 4096,
            headers: 32,
            body: u64::from(jig.clients_http.body),
            read: jig.clients_http.read,
            response: 1024,
            send: jig.clients_http.send,
        },
        stall: Duration::from_secs(30),
    }
}
