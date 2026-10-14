use super::{Bounds, FITS, Undecodable, names, topics, w};
use skein_lib::List;
use temper_engine_domain_forge_client::api as d;
fn encode_permission(value: d::Permission) -> w::Permission {
    match value {
        d::Permission::None => w::Permission::None,
        d::Permission::Read => w::Permission::Read,
        d::Permission::Write => w::Permission::Write,
        d::Permission::Admin => w::Permission::Admin,
    }
}
fn decode_permission(value: w::Permission) -> d::Permission {
    match value {
        w::Permission::None => d::Permission::None,
        w::Permission::Read => d::Permission::Read,
        w::Permission::Write => d::Permission::Write,
        w::Permission::Admin => d::Permission::Admin,
    }
}
fn encode_branch_creation(value: d::BranchCreation) -> w::BranchCreation {
    match value {
        d::BranchCreation::Created => w::BranchCreation::Created,
        d::BranchCreation::Exists => w::BranchCreation::Exists,
    }
}
fn decode_branch_creation(value: w::BranchCreation) -> d::BranchCreation {
    match value {
        w::BranchCreation::Created => d::BranchCreation::Created,
        w::BranchCreation::Exists => d::BranchCreation::Exists,
    }
}
fn encode_reviewers(values: &[u64]) -> List<u64> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        result.push(*value).expect("same count");
    }
    result
}
fn encode_pull(value: &d::Pull) -> w::Pull {
    let d::Pull { reviewers, number, state, head, base, commit, base_commit, merged, mergeable, ci } = value;
    w::Pull::new(
        &w::CEILINGS,
        w::PullParts {
            reviewers: encode_reviewers(reviewers),
            number: *number,
            state: topics::encode_state(*state),
            head: head.clone(),
            base: base.clone(),
            commit: *commit,
            base_commit: *base_commit,
            merged: *merged,
            mergeable: *mergeable,
            ci: topics::encode_ci(*ci),
        },
    )
    .expect(FITS)
}
fn decode_pull(value: w::Pull) -> d::Pull {
    let w::PullParts { reviewers, number, state, head, base, commit, base_commit, merged, mergeable, ci } =
        value.into_parts();
    d::Pull {
        reviewers: reviewers.into_boxed(),
        number,
        state: topics::decode_state(state),
        head,
        base,
        commit,
        base_commit,
        merged,
        mergeable,
        ci: topics::decode_ci(ci),
    }
}
fn encode_job_attempt(value: &d::JobAttempt) -> w::JobAttempt {
    let d::JobAttempt { head, run, job, attempt } = value;
    w::JobAttempt::new(&w::CEILINGS, w::JobAttemptParts { head: *head, run: *run, job: *job, attempt: *attempt })
        .expect(FITS)
}
fn decode_job_attempt(value: w::JobAttempt) -> d::JobAttempt {
    let w::JobAttemptParts { head, run, job, attempt } = value.into_parts();
    d::JobAttempt { head, run, job, attempt }
}
#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn encode(value: &d::Answer) -> Option<w::Answer> {
    Some(match value {
        d::Answer::Pull(value) => w::Answer::Pull(encode_pull(value)),
        d::Answer::Commit(commit) => {
            w::Answer::Commit(w::Commit::new(&w::CEILINGS, w::CommitParts { commit: *commit }).expect(FITS))
        }
        d::Answer::Branches(values) => w::Answer::Branches(
            w::BranchesAnswer::new(&w::CEILINGS, w::BranchesAnswerParts { branches: names::encode_piece(values) })
                .expect(FITS),
        ),
        d::Answer::Job { attempt, log, truncated } => w::Answer::Job(
            w::JobAnswer::new(
                &w::CEILINGS,
                w::JobAnswerParts { attempt: encode_job_attempt(attempt), log: log.clone(), truncated: *truncated },
            )
            .expect(FITS),
        ),
        d::Answer::File { head, path, bytes, truncated } => w::Answer::File(
            w::FileAnswer::new(
                &w::CEILINGS,
                w::FileAnswerParts { head: *head, path: path.clone(), bytes: bytes.clone(), truncated: *truncated },
            )
            .expect(FITS),
        ),
        d::Answer::Protection(value) => w::Answer::Protection(
            w::ProtectionAnswer::new(
                &w::CEILINGS,
                w::ProtectionAnswerParts {
                    protection: match value {
                        Some(value) => Some(encode_protection(value)),
                        None => None,
                    },
                },
            )
            .expect(FITS),
        ),
        d::Answer::Settings(value) => w::Answer::Settings(names::encode_settings(value)),
        d::Answer::Permission(value) => w::Answer::Permission(
            w::PermissionValue::new(&w::CEILINGS, w::PermissionValueParts { permission: encode_permission(*value) })
                .expect(FITS),
        ),
        d::Answer::Created(number) => {
            w::Answer::Created(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::Answer::Commented(number) => {
            w::Answer::Commented(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::Answer::Reviewed(number) => {
            w::Answer::Reviewed(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::Answer::Merged(commit) => {
            w::Answer::Merged(w::Commit::new(&w::CEILINGS, w::CommitParts { commit: *commit }).expect(FITS))
        }
        d::Answer::Branch(value) => w::Answer::Branch(
            w::BranchCreationValue::new(
                &w::CEILINGS,
                w::BranchCreationValueParts { creation: encode_branch_creation(*value) },
            )
            .expect(FITS),
        ),
        d::Answer::Done => w::Answer::Done,
        d::Answer::Items { .. }
        | d::Answer::Item { .. }
        | d::Answer::Reviews { .. }
        | d::Answer::Statuses { .. }
        | d::Answer::Remarks { .. }
        | d::Answer::PullFiles { .. }
        | d::Answer::Compare { .. }
        | d::Answer::Checks(_)
        | d::Answer::Collaborators { .. } => return None,
    })
}
pub(super) fn decode(value: w::Answer, bounds: &Bounds) -> Result<d::Answer, Undecodable> {
    Ok(match value {
        w::Answer::Pull(value) => d::Answer::Pull(decode_pull(value)),
        w::Answer::Commit(value) => d::Answer::Commit(value.into_parts().commit),
        w::Answer::Branches(value) => {
            d::Answer::Branches(names::decode_contexts(&value.into_parts().branches, bounds)?)
        }
        w::Answer::Job(value) => {
            let w::JobAnswerParts { attempt, log, truncated } = value.into_parts();
            d::Answer::Job { attempt: decode_job_attempt(attempt), log, truncated }
        }
        w::Answer::File(value) => {
            let w::FileAnswerParts { head, path, bytes, truncated } = value.into_parts();
            d::Answer::File { head, path, bytes, truncated }
        }
        w::Answer::Protection(value) => d::Answer::Protection(match value.into_parts().protection {
            Some(value) => Some(decode_protection(value, bounds)?),
            None => None,
        }),
        w::Answer::Settings(value) => d::Answer::Settings(names::decode_settings(value)),
        w::Answer::Permission(value) => d::Answer::Permission(decode_permission(value.into_parts().permission)),
        w::Answer::Created(value) => d::Answer::Created(value.number()),
        w::Answer::Commented(value) => d::Answer::Commented(value.number()),
        w::Answer::Reviewed(value) => d::Answer::Reviewed(value.number()),
        w::Answer::Merged(value) => d::Answer::Merged(value.into_parts().commit),
        w::Answer::Branch(value) => d::Answer::Branch(decode_branch_creation(value.into_parts().creation)),
        w::Answer::Done => d::Answer::Done,
    })
}
fn encode_protection(value: &d::Protection) -> w::Protection {
    let d::Protection { branch, contexts, approvals, dismiss_stale } = value;
    w::Protection::new(
        &w::CEILINGS,
        w::ProtectionParts {
            branch: branch.clone(),
            contexts: names::encode_piece(contexts),
            approvals: *approvals,
            dismiss_stale: *dismiss_stale,
        },
    )
    .expect(FITS)
}
fn decode_protection(value: w::Protection, bounds: &Bounds) -> Result<d::Protection, Undecodable> {
    let w::ProtectionParts { branch, contexts, approvals, dismiss_stale } = value.into_parts();
    Ok(d::Protection { branch, contexts: names::decode_contexts(&contexts, bounds)?, approvals, dismiss_stale })
}
