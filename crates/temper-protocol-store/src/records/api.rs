use super::{FITS, w};
use skein_lib::List;
use temper_engine_domain_forge_client::api as d;

fn encode_verdict(value: d::Verdict) -> w::Verdict {
    match value {
        d::Verdict::Approve => w::Verdict::Approve,
        d::Verdict::RequestChanges => w::Verdict::RequestChanges,
        d::Verdict::Comment => w::Verdict::Comment,
    }
}
fn decode_verdict(value: w::Verdict) -> d::Verdict {
    match value {
        w::Verdict::Approve => d::Verdict::Approve,
        w::Verdict::RequestChanges => d::Verdict::RequestChanges,
        w::Verdict::Comment => d::Verdict::Comment,
    }
}
fn encode_check(value: d::Check) -> w::Check {
    match value {
        d::Check::Pending => w::Check::Pending,
        d::Check::Passed => w::Check::Passed,
        d::Check::Failed => w::Check::Failed,
    }
}
fn decode_check(value: w::Check) -> d::Check {
    match value {
        w::Check::Pending => d::Check::Pending,
        w::Check::Passed => d::Check::Passed,
        w::Check::Failed => d::Check::Failed,
    }
}
pub(super) fn encode_write(value: &d::Write) -> w::Write {
    match value {
        d::Write::CreateIssue { key, title, body } => w::Write::CreateIssue(
            w::CreateIssue::new(
                &w::CEILINGS,
                w::CreateIssueParts { key: key.clone(), title: title.clone(), body: body.clone() },
            )
            .expect(FITS),
        ),
        d::Write::OpenPull { title, body, head, base } => w::Write::OpenPull(
            w::OpenPull::new(
                &w::CEILINGS,
                w::OpenPullParts { title: title.clone(), body: body.clone(), head: head.clone(), base: base.clone() },
            )
            .expect(FITS),
        ),
        d::Write::Post { number, key, body } => w::Write::Post(
            w::Post::new(&w::CEILINGS, w::PostParts { number: *number, key: key.clone(), body: body.clone() })
                .expect(FITS),
        ),
        d::Write::Review { number, key, verdict, body } => w::Write::Review(
            w::PostReview::new(
                &w::CEILINGS,
                w::PostReviewParts {
                    number: *number,
                    key: key.clone(),
                    verdict: encode_verdict(*verdict),
                    body: body.clone(),
                },
            )
            .expect(FITS),
        ),
        d::Write::Edit { number, title, body } => w::Write::Edit(
            w::Edit::new(&w::CEILINGS, w::EditParts { number: *number, title: title.clone(), body: body.clone() })
                .expect(FITS),
        ),
        d::Write::SetReviewers { number, reviewers } => w::Write::SetReviewers(
            w::SetReviewers::new(
                &w::CEILINGS,
                w::SetReviewersParts { number: *number, reviewers: encode_reviewers(reviewers) },
            )
            .expect(FITS),
        ),
        d::Write::Close { number } => {
            w::Write::Close(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::Write::Reopen { number } => {
            w::Write::Reopen(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::Write::Merge { number, head } => w::Write::Merge(
            w::MergeWrite::new(&w::CEILINGS, w::MergeWriteParts { number: *number, head: *head }).expect(FITS),
        ),
        d::Write::Update { number } => {
            w::Write::Update(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::Write::Status { commit, context, check } => w::Write::Status(
            w::SetStatus::new(
                &w::CEILINGS,
                w::SetStatusParts { commit: *commit, context: context.clone(), status: encode_check(*check) },
            )
            .expect(FITS),
        ),
        d::Write::CreateBranch { branch, commit } => w::Write::CreateBranch(
            w::CreateBranch::new(&w::CEILINGS, w::CreateBranchParts { branch: branch.clone(), commit: *commit })
                .expect(FITS),
        ),
        d::Write::DeleteBranch { branch } => w::Write::DeleteBranch(
            w::DeleteBranch::new(&w::CEILINGS, w::DeleteBranchParts { branch: branch.clone() }).expect(FITS),
        ),
    }
}
pub(super) fn decode_write(value: w::Write) -> d::Write {
    match value {
        w::Write::CreateIssue(value) => {
            let w::CreateIssueParts { key, title, body } = value.into_parts();
            d::Write::CreateIssue { key, title, body }
        }
        w::Write::OpenPull(value) => {
            let w::OpenPullParts { title, body, head, base } = value.into_parts();
            d::Write::OpenPull { title, body, head, base }
        }
        w::Write::Post(value) => {
            let w::PostParts { number, key, body } = value.into_parts();
            d::Write::Post { number, key, body }
        }
        w::Write::Review(value) => {
            let w::PostReviewParts { number, key, verdict, body } = value.into_parts();
            d::Write::Review { number, key, verdict: decode_verdict(verdict), body }
        }
        w::Write::Edit(value) => {
            let w::EditParts { number, title, body } = value.into_parts();
            d::Write::Edit { number, title, body }
        }
        w::Write::SetReviewers(value) => {
            let w::SetReviewersParts { number, reviewers } = value.into_parts();
            d::Write::SetReviewers { number, reviewers: reviewers.into_boxed() }
        }
        w::Write::Close(value) => {
            let w::NumberParts { number } = value.into_parts();
            d::Write::Close { number }
        }
        w::Write::Reopen(value) => {
            let w::NumberParts { number } = value.into_parts();
            d::Write::Reopen { number }
        }
        w::Write::Merge(value) => {
            let w::MergeWriteParts { number, head } = value.into_parts();
            d::Write::Merge { number, head }
        }
        w::Write::Update(value) => {
            let w::NumberParts { number } = value.into_parts();
            d::Write::Update { number }
        }
        w::Write::Status(value) => {
            let w::SetStatusParts { commit, context, status } = value.into_parts();
            d::Write::Status { commit, context, check: decode_check(status) }
        }
        w::Write::CreateBranch(value) => {
            let w::CreateBranchParts { branch, commit } = value.into_parts();
            d::Write::CreateBranch { branch, commit }
        }
        w::Write::DeleteBranch(value) => {
            let w::DeleteBranchParts { branch } = value.into_parts();
            d::Write::DeleteBranch { branch }
        }
    }
}
fn encode_reviewers(values: &[u64]) -> List<u64> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        result.push(*value).expect("same count");
    }
    result
}
