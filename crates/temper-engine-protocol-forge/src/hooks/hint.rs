//! Verified delivery fields mapped to the forge client's hints (forge.md, 5).
use crate::marker;
use skein_lib::bytes;
use temper_engine_domain_forge_client::api;
use temper_forge_forgejo::{delivery::Delivery, document::Body};

pub(crate) fn hint(forge: u16, delivery: &Delivery) -> Option<api::Hint> {
    let (repository, change, body) = match delivery {
        Delivery::Issue { repository, number, body }
        | Delivery::Comment { repository, number, body }
        | Delivery::Review { repository, number, body, .. } => (*repository, api::Change::Item(*number), Some(body)),
        Delivery::Pull { repository, number } => (*repository, api::Change::Item(*number), None),
        Delivery::Push { repository, after, pusher, .. } => {
            let branch = bytes::copy_of(delivery.branch()?);
            let change = if delivery.deleted() {
                api::Change::Branch(branch)
            } else {
                api::Change::BranchMoved { branch, head: after.bytes, actor: *pusher }
            };
            (*repository, change, None)
        }
        Delivery::Ref { repository, .. } => {
            (*repository, api::Change::Branch(bytes::copy_of(delivery.branch()?)), None)
        }
        Delivery::Job { repository, commit, .. } => (*repository, api::Change::Commit(commit.bytes), None),
    };
    let key = match body {
        Some(Body::Kept(text)) => marker::decode(text),
        Some(Body::Long(_)) | None => None,
    };
    Some(api::Hint { repository: api::Repository { forge, repository: u32::try_from(repository).ok()? }, change, key })
}
