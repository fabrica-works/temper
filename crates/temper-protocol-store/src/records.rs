//! Total translations between durable domain rows and generated records.
use crate::{Bounds, Undecodable, v1 as w};
use alloc::boxed::Box;
use skein_lib::{Reader, Writer};
use temper_engine_domain_forge as d;
mod answers;
mod api;
mod client;
#[cfg(test)]
pub(crate) use api::encode_error;
mod entries;
mod names;
mod topics;
const FITS: &str = "admitted fields fit the schema";

/// Encode a supported family admitted by the domain; later families return none.
#[must_use]
pub fn encode(value: &d::Stored) -> Option<Box<[u8]>> {
    let value = match value {
        d::Stored::Repository(value) => w::Value::Repository(names::encode_adopted(value)),
        d::Stored::Hold(value) => w::Value::Hold(names::encode_hold(value)),
        d::Stored::Names { task, resources } => w::Value::Names(topics::encode_names(*task, resources)),
        d::Stored::Subscription(value) => w::Value::Subscription(topics::encode_subscriber(value)),
        d::Stored::BranchHead(value) => w::Value::BranchHead(topics::encode_branch_head(value)),
        d::Stored::PullState(value) => w::Value::PullState(topics::encode_pull_state(value)),
        d::Stored::Ci(value) => w::Value::Ci(topics::encode_ci_state(value)),
        d::Stored::Landed { commit, task } => w::Value::Landed(topics::encode_landed(*commit, *task)),
        d::Stored::Release(value) => w::Value::Release(topics::encode_release(value)),
        d::Stored::ProposedEffect(value) => w::Value::ProposedEffect(entries::encode_proposed(value)),
        d::Stored::Entry { entry, held } => w::Value::Entry(entries::encode_entry_row(entry, *held)),
        d::Stored::Call { key, answer } => w::Value::Call(entries::encode_call(key, answer)),
        d::Stored::Client(value) => w::Value::Client(client::encode(value)),
        d::Stored::Change(_) | d::Stored::Issue(_) => return None,
    };
    let row = w::Row::new(&w::CEILINGS, w::RowParts { value }).expect("admitted row fits the schema");
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("row length fits"));
    row.encode(&mut writer).expect("measured room");
    Some(writer.finish())
}

/// Decode one supported row with the checked startup bounds.
pub fn decode(bytes: &[u8], bounds: &Bounds) -> Result<d::Stored, Undecodable> {
    let row = w::Row::decode(bounds.limits(), &mut Reader::new(bytes)).or(Err(Undecodable))?;
    Ok(match row.into_parts().value {
        w::Value::Client(value) => d::Stored::Client(client::decode(value, bounds)?),
        w::Value::Call(value) => entries::decode_call(value, bounds)?,
        w::Value::ProposedEffect(value) => d::Stored::ProposedEffect(entries::decode_proposed(value, bounds)?),
        w::Value::Entry(value) => entries::decode_entry_row(value),
        w::Value::Repository(value) => d::Stored::Repository(names::decode_adopted(value, bounds)?),
        w::Value::Hold(value) => d::Stored::Hold(names::decode_hold(value, bounds)?),
        w::Value::Names(value) => {
            let w::NamesParts { task, resources } = value.into_parts();
            d::Stored::Names { task, resources: topics::decode_names(resources, bounds)? }
        }
        w::Value::Subscription(value) => d::Stored::Subscription(topics::decode_subscriber(value)),
        w::Value::BranchHead(value) => d::Stored::BranchHead(topics::decode_branch_head(value, bounds)?),
        w::Value::PullState(value) => d::Stored::PullState(topics::decode_pull_state(value, bounds)?),
        w::Value::Ci(value) => d::Stored::Ci(topics::decode_ci_state(value)),
        w::Value::Landed(value) => {
            let w::LandedParts { commit, task } = value.into_parts();
            d::Stored::Landed { commit, task }
        }
        w::Value::Release(value) => d::Stored::Release(topics::decode_release(value, bounds)?),
    })
}

pub(crate) fn limits(forge: &d::Limits) -> Option<w::Limits> {
    Some(w::Limits {
        live_branch_branch: forge.name_bytes,
        live_record_comments: forge.client.inbox,
        live_record_reviews: forge.client.inbox,
        live_record_echoes: forge.client.rows,
        status_context: forge.client.answer_bytes,
        status_description: forge.client.answer_bytes,
        status_url: forge.client.answer_bytes,
        remark_path: forge.client.answer_bytes,
        remark_body: forge.client.answer_bytes,
        file_path: forge.client.answer_bytes,
        file_before_some: forge.client.answer_bytes,
        file_after_some: forge.client.answer_bytes,
        statuses_answer_statuses: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        remarks_answer_remarks: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        pull_files_answer_files: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        compare_answer_files: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        compare_answer_commits: forge.client.rows,
        checks_answer_statuses: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        collaborators_answer_collaborators: forge.client.rows,
        summary_labels: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        summary_key_some: forge.client.answer_bytes,
        summary_title: forge.client.answer_bytes,
        summary_body: forge.client.answer_bytes,
        comment_key_some: forge.client.answer_bytes,
        comment_body: forge.client.answer_bytes,
        review_key_some: forge.client.answer_bytes,
        review_body: forge.client.answer_bytes,
        items_answer_items: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        item_answer_comments: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        reviews_answer_reviews: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        pull_reviewers: forge.client.rows,
        pull_head: forge.client.answer_bytes,
        pull_base: forge.client.answer_bytes,
        job_answer_log: forge.client.answer_bytes,
        file_answer_path: forge.client.answer_bytes,
        file_answer_bytes: forge.client.answer_bytes,
        branches_answer_branches: forge.client.answer_bytes.checked_add(forge.client.rows.checked_mul(4)?)?,
        ..w::CEILINGS
    })
}
