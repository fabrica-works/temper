//! Locate's owned names and the forge's nonzero repository id (protocol/forge.md, 12).
use crate::api::Answer;

pub(crate) fn bytes(owner: &[u8], name: &[u8]) -> Option<u64> {
    u64::try_from(owner.len()).ok()?.checked_add(u64::try_from(name.len()).ok()?)
}

pub(crate) fn accepts(answer: &Answer) -> bool {
    match answer {
        Answer::Located { repository } => *repository != 0,
        Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Reviews { .. }
        | Answer::Statuses { .. }
        | Answer::Remarks { .. }
        | Answer::Commit(_)
        | Answer::Branches(_)
        | Answer::PullFiles { .. }
        | Answer::Compare { .. }
        | Answer::Checks(_)
        | Answer::Job { .. }
        | Answer::File { .. }
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators { .. }
        | Answer::Permission(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Branch(_)
        | Answer::Done => false,
    }
}
