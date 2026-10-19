//! Live facts shared by conditions, digests and wait stories.
use super::*;
/// One counted need distinguishes an empty inbox from a waiting one.
pub(crate) const ONE_NEED: u32 = 1;
/// A policy role must differ from the four built-in roles.
pub(crate) const POLICY_ROLE: u32 = 7;
/// A role in another project must not match the task's project.
pub(crate) const OTHER_PROJECT: u32 = PROJECT + 1;
/// The settled chat need not share the task's opaque label.
pub(crate) const CHAT_LABEL: u32 = 97;
pub(crate) const LIVE_PHASES: &[c::Phase] = &[
    c::Phase::Waiting,
    c::Phase::Idle,
    c::Phase::Due,
    c::Phase::Preparing,
    c::Phase::Claimed,
    c::Phase::Running,
    c::Phase::BackingOff,
    c::Phase::Active,
    c::Phase::Closing,
    c::Phase::Held,
];
pub(crate) const ENDED_PHASES: &[c::Phase] = &[c::Phase::Ended, c::Phase::Done, c::Phase::Failed, c::Phase::Cancelled];
pub(crate) fn chat_at(task: u64, stage: c::RunStage, work: Option<c::OwnerChipWork>) -> client::Latest {
    let mut task = goal(task, c::WaitsOn::Nothing);
    task.label = CHAT_LABEL;
    task.flags.conversation = true;
    task.chip.phase = c::OwnerChipPhase::Active(stage).compact();
    task.chip.details = Box::new(c::ChipPhase::Owner {
        phase: c::OwnerChipPhase::Active(stage),
        details: Box::new(c::OwnerChipDetails {
            project: task.project,
            executor: c::OwnerExecutor::Agent { charter: PROJECT },
            phase_since: Wall::EPOCH,
            run_since: None,
            attempt: Some(ATTEMPT),
            retry: Some(c::RetryNotice::Refused { count: ONE_NEED }),
            wake: task.wake,
            person_wait: None,
            hold: None,
            budget: task.budget.clone(),
            activity: 0,
            resources: List::with_capacity(EMPTY),
            resources_cut: false,
            resources_observed: true,
            procedure: None,
            live_delegates: 0,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        }),
    });
    let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner fixture") };
    details.observation = None;
    if let Some(work) = work {
        details.observation = Some(c::OwnerChipObservation {
            channel: Token::new(1),
            work,
            since: Wall::EPOCH,
            reported_at: Wall::EPOCH,
            started: None,
        });
    }
    client::Latest::Jig(c::Delivery::Task(Box::new(task)))
}
pub(crate) fn task_in(latest: &mut client::Latest) -> &mut c::Task {
    let client::Latest::Jig(c::Delivery::Task(task)) = latest else { panic!("task fixture") };
    task
}
pub(crate) fn ended_chat(phase: c::Phase) -> client::Latest {
    let mut latest = chat_at(TASK, c::RunStage::Idle, None);
    let task = task_in(&mut latest);
    task.chip.phase = phase;
    let c::ChipPhase::Owner { phase, details } = task.chip.details.as_mut() else { panic!("owner fixture") };
    *phase = c::OwnerChipPhase::Ended;
    details.ended_at = Some(DECIDED_AT);
    latest
}
pub(crate) fn party_wait(whom: c::Whom, kind: c::WaitedFor) -> c::WaitsOn {
    c::WaitsOn::Party { whom, task: OTHER_TASK, kind, entries: ONE_NEED }
}
pub(crate) fn inbox_counts(needs: u32, updates: u32) -> client::Latest {
    client::Latest::Jig(c::Delivery::Inbox(Box::new(c::Inbox {
        entries: List::with_capacity(EMPTY),
        needs: c::Needs { questions: needs, proposals: 0, escalations: 0, person_tasks: 0 },
        updates: c::Count { value: updates, limited: false },
        results_seen: 0,
        replies_seen: List::with_capacity(EMPTY),
        more: false,
    })))
}
pub(crate) fn owner_inbox_counts(needs: u32, updates: u32) -> client::Latest {
    client::Latest::Jig(c::Delivery::OwnerInbox(Box::new(c::OwnerInbox {
        party: PARTY,
        needs: List::with_capacity(EMPTY),
        updates: List::with_capacity(EMPTY),
        next: None,
        cached_needs: needs,
        cached_updates: c::Count { value: updates, limited: false },
        results_seen: 0,
        updates_time_complete: true,
        selection: None,
    })))
}
pub(crate) fn holds(
    condition: crate::jig::subject::Condition,
    latest: &client::Latest,
    me: Option<&c::Me>,
) -> Option<bool> {
    crate::jig::conditions::holds(&condition, &crate::jig::conditions::Facts { subject: latest, me })
}

pub(crate) fn tree_at(chip: c::Chip) -> client::Latest {
    let mut nodes = List::with_capacity(PAGE_ROWS);
    nodes
        .push(c::TreeNode {
            chip,
            parent: None,
            dependencies: List::with_capacity(EMPTY),
            proposals: List::with_capacity(EMPTY),
            spend: 0,
            folded: phase_counts(),
            held_below: 0,
        })
        .expect("root row");
    client::Latest::Jig(c::Delivery::Tree(Box::new(c::Tree {
        task: TASK,
        nodes,
        folded: phase_counts(),
        held_below: 0,
    })))
}
pub(crate) fn phase_counts() -> c::PhaseCounts {
    c::PhaseCounts { ended: 0, running: 0, waiting: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}
pub(crate) fn owner_tree_at(chip: c::Chip) -> client::Latest {
    client::Latest::Jig(c::Delivery::OwnerTree(Box::new(c::OwnerTree {
        root: chip,
        rows: List::with_capacity(EMPTY),
        folded: List::with_capacity(EMPTY),
        held_below: 0,
        next: None,
    })))
}
pub(crate) fn chip_phase(latest: &mut client::Latest, phase: c::OwnerChipPhase) {
    let task = task_in(latest);
    task.chip.phase = phase.compact();
    let c::ChipPhase::Owner { phase: current, .. } = task.chip.details.as_mut() else { panic!("owner chip") };
    *current = phase;
}
