//! Volatile evidence of Forgejo settling, beside the change's persisted push.
use skein_lib::{Duration, Env, Wall};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_client as client;

use crate::{ChangeRow, Domain, Limits};
type Commit = client::api::Commit;

/// Hints and fresh reads, retained only for this incarnation of a task.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Settle {
    since: Option<Wall>,
    job: bool,
    base: Option<Commit>,
    open: Option<(Wall, Wall)>,
}

impl Settle {
    pub(crate) const NEW: Settle = Settle { since: None, job: false, base: None, open: None };

    /// First reads open no window: only a prior base read detects a move.
    pub(crate) fn fold(
        mut self,
        now: Wall,
        own: Wall,
        settle: Duration,
        moving: bool,
        ci_pending: bool,
        base: Option<Commit>,
    ) -> Settle {
        if self.base.is_some() && self.base != base {
            self.since = Some(now);
        }
        self.base = base;
        let unsettled = moving || (self.job && ci_pending);
        if !unsettled {
            self.since = None;
            self.job = false;
        }
        let start = own.max(self.since.unwrap_or(own));
        let end = Wall::from_nanos(start.as_nanos().saturating_add(settle.as_nanos()));
        self.open = if unsettled && now < end { Some((start, end)) } else { None };
        self
    }
}

/// Only a named branch move or this head's job end advances the start.
pub(crate) fn hinted(d: &mut Domain, env: &Env<Limits>, hint: &client::api::Hint, tasks: &[u64]) {
    for task in tasks {
        let row = d.changes.get(task).expect("a change the hint wakes");
        let job = match &hint.change {
            client::api::Change::Commit(head) => {
                if row.change.last_head != Some(*head) {
                    continue;
                }
                true
            }
            client::api::Change::Branch(branch) | client::api::Change::BranchMoved { branch, .. } => {
                if *row.branch != **branch && *row.base != **branch {
                    continue;
                }
                false
            }
            client::api::Change::Item(_) => continue,
        };
        let mut settle = d.settling.get(task).copied().unwrap_or(Settle::NEW);
        settle.since = Some(env.wall);
        settle.job |= job;
        d.settling.insert(*task, settle).expect("one settle row per change");
    }
}

/// Fold the fresh pull and CI before constructing this step's facts.
pub(crate) fn observe(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &ChangeRow,
    pull: Option<&client::api::Pull>,
    ci: change::Status,
) {
    let (moving, base) = match pull {
        Some(pull) => {
            (pull.state == client::api::State::Open && pull.merged.is_none() && !pull.mergeable, pull.base_commit)
        }
        None => (false, None),
    };
    let pending = match ci {
        change::Status::Pending | change::Status::Unknown => true,
        change::Status::Passed | change::Status::Failed => false,
    };
    let settle = d.settling.get(&row.task).copied().unwrap_or(Settle::NEW).fold(
        env.wall,
        row.change.since,
        env.limits.client.settle,
        moving,
        pending,
        base,
    );
    d.settling.insert(row.task, settle).expect("one settle row per change");
}

pub(crate) fn window(d: &Domain, task: u64) -> Option<(Wall, Wall)> {
    d.settling.get(&task)?.open
}

/// Bring a timed wait forward to the next doubling read, bounded by its end.
pub(crate) fn cap(d: &Domain, env: &Env<Limits>, task: u64, decision: change::Decision) -> change::Decision {
    let Some((start, end)) = window(d, task) else { return decision };
    match decision {
        change::Decision::Wait { until: Some(until) } => {
            let interval =
                env.wall.as_nanos().saturating_sub(start.as_nanos()).max(env.limits.client.hinted.as_nanos());
            let next = Wall::from_nanos(env.wall.as_nanos().saturating_add(interval));
            change::Decision::Wait { until: Some(next.min(end).min(until)) }
        }
        change::Decision::Wait { until: None }
        | change::Decision::None
        | change::Decision::Delegate(_)
        | change::Decision::Effect(_)
        | change::Decision::Ready
        | change::Decision::QueueRepair
        | change::Decision::Finish { .. }
        | change::Decision::Cancel
        | change::Decision::Hold(_) => decision,
    }
}
