//! Proposal reads and the settings review supplied by wire 19 (client/tools.md, 5.2).
use super::{Cut, Limits, Text, settings};
use jig_client::contract as c;
use skein_json::writer::Encoder;

#[derive(Clone, Copy)]
enum Proposal<'a> {
    Owner(&'a c::OwnerProposal),
    Legacy(&'a c::Proposal),
}
fn selected(value: &c::ReadValue) -> Proposal<'_> {
    match value {
        c::ReadValue::OwnerProposal(p) => Proposal::Owner(p),
        c::ReadValue::Proposal(p) => Proposal::Legacy(p),
        c::ReadValue::Settings(_)
        | c::ReadValue::Setting(_)
        | c::ReadValue::Revision(_)
        | c::ReadValue::Revisions(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::OwnerProject(_)
        | c::ReadValue::Historical(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Task(_)
        | c::ReadValue::Escalation(_)
        | c::ReadValue::OwnerEscalation(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::OwnerPersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::OwnerEnded(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::OwnerInbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::OwnerSpend(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_)
        | c::ReadValue::OwnerDeployment(_) => unreachable!("selected proposal read"),
    }
}

pub(super) fn text(value: &c::ReadValue, limits: &Limits, out: &mut Text) {
    let (key, reason, action) = match selected(value) {
        Proposal::Owner(p) => (p.key, p.reason.bytes.bytes(), owner_action(&p.action)),
        Proposal::Legacy(p) => (p.key, p.why.bytes(), legacy_action(&p.action)),
    };
    out.put(b"proposal ");
    settings::proposal_text(key, out);
    out.put(b" \xc2\xb7 ");
    out.put(action);
    if let c::ReadValue::OwnerProposal(p) = value
        && let c::OwnerProposalAction::Settings(s) = &p.action
    {
        out.put(b" \xc2\xb7 against revision ");
        out.number(u64::from(s.against));
        if let Some(review) = &s.review {
            out.put(b", newest ");
            out.number(u64::from(review.newest));
        }
    }
    out.put(b"\n");
    out.words(reason);
    out.put(b"\n");
    if let c::ReadValue::OwnerProposal(p) = value
        && let c::OwnerProposalAction::Settings(s) = &p.action
        && let Some(review) = &s.review
    {
        for item in &review.items {
            settings::setting_text(item.setting, out);
            out.put(b" \xc2\xb7 ");
            out.put(settings::edit_word(&item.edit));
            out.put(b" \xc2\xb7 now ");
            review_value_text(&item.now, limits, out);
            out.put(b" (");
            match &item.origin {
                c::SettingOrigin::Change(r) => {
                    out.put(b"revision ");
                    out.number(u64::from(r.revision));
                }
                c::SettingOrigin::Default(r) => {
                    out.put(b"default since revision ");
                    out.number(u64::from(r.revision));
                }
                c::SettingOrigin::NotOffered => out.put(b"not offered"),
            }
            out.put(b") \xc2\xb7 would be ");
            review_value_text(&item.would_be, limits, out);
            out.put(b"\n");
        }
        if review.cut {
            out.put(b"cut: more not shown\n");
        }
    }
}
fn review_value_text(value: &c::ReviewValue, limits: &Limits, out: &mut Text) {
    match value {
        c::ReviewValue::Value(set) => settings::value_inline(&set.value, limits, out),
        c::ReviewValue::TooLarge(size) => {
            out.put(b"too large, ");
            out.number(size.bytes);
            out.put(b" bytes");
        }
        c::ReviewValue::None => out.put(b"none"),
    }
}
fn owner_action(action: &c::OwnerProposalAction) -> &'static [u8] {
    match action {
        c::OwnerProposalAction::Settings(_) => b"settings",
        c::OwnerProposalAction::Batch(_) => b"batch",
        c::OwnerProposalAction::Effect { .. } => b"effect",
        c::OwnerProposalAction::ProcedureEffect { .. } => b"procedure_effect",
        c::OwnerProposalAction::Amend { .. } => b"amend",
        c::OwnerProposalAction::Widen { .. } => b"widen",
        c::OwnerProposalAction::Release { .. } => b"release",
    }
}
fn legacy_action(action: &c::ProposalAction) -> &'static [u8] {
    match action {
        c::ProposalAction::Batch { .. } => b"batch",
        c::ProposalAction::Effect { .. } => b"effect",
        c::ProposalAction::Widen { .. } => b"widen",
        c::ProposalAction::Amend { .. } => b"amend",
    }
}
pub(super) fn encode(value: &c::ReadValue, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    let proposal = selected(value);
    let (key, project, acts) = match proposal {
        Proposal::Owner(p) => (p.key, p.project, &p.acts),
        Proposal::Legacy(p) => (p.key, p.project, &p.acts),
    };
    out.key(b"key");
    super::encode_proposal_key(&key, out, budget);
    out.key(b"project");
    out.unsigned(u64::from(project));
    match proposal {
        Proposal::Owner(p) => {
            out.key(b"reason");
            super::encode_document_text(&p.reason, out, budget);
            out.key(b"action");
            match &p.action {
                c::OwnerProposalAction::Settings(s) => encode_settings(s, out),
                action @ (c::OwnerProposalAction::Batch(_)
                | c::OwnerProposalAction::Effect { .. }
                | c::OwnerProposalAction::ProcedureEffect { .. }
                | c::OwnerProposalAction::Amend { .. }
                | c::OwnerProposalAction::Widen { .. }
                | c::OwnerProposalAction::Release { .. }) => out.string(owner_action(action)),
            }
            out.key(b"as_holder");
            out.boolean(p.as_holder);
            out.key(b"waiting");
            out.object_start();
            out.key(b"holder");
            holder(p.waiting.holder, out, budget);
            out.key(b"since");
            out.unsigned(p.waiting.since.as_nanos());
            out.object_end();
            out.key(b"funding");
            out.array_start();
            for offer in &p.funding {
                out.object_start();
                out.key(b"source");
                super::encode_funder(&offer.source, out, budget);
                out.key(b"needed");
                out.unsigned(offer.needed);
                for (key, value) in [(b"left".as_slice(), offer.left), (b"after".as_slice(), offer.after)] {
                    out.key(key);
                    match value {
                        Some(n) => out.unsigned(n),
                        None => out.null(),
                    }
                }
                out.object_end();
            }
            out.array_end();
        }
        Proposal::Legacy(p) => {
            out.key(b"why");
            budget.words(p.why.bytes(), out);
            out.key(b"action");
            out.string(legacy_action(&p.action));
            out.key(b"waiting");
            super::encode_recipient(&p.waiting, out, budget);
            out.key(b"since");
            out.unsigned(p.since.as_nanos());
        }
    }
    out.key(b"acts");
    out.array_start();
    for act in acts {
        super::encode_offered(act, out, budget);
    }
    out.array_end();
    out.object_end();
}
fn holder(holder: c::OwnerHistoryHolder, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    match holder {
        c::OwnerHistoryHolder::Task(n) | c::OwnerHistoryHolder::Person(n) => {
            out.key(if matches!(holder, c::OwnerHistoryHolder::Task(_)) { b"task" } else { b"person" });
            out.object_start();
            out.key(b"value");
            out.unsigned(n);
            out.object_end();
        }
        c::OwnerHistoryHolder::Policy { project, kind } => {
            out.key(b"policy");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(project));
            out.key(b"kind");
            super::read_owner_words::encode_owner_message_proposal_kind(kind, out, budget);
            out.object_end();
        }
    }
    out.object_end();
}
fn encode_settings(s: &c::OwnerProposalActionSettings, out: &mut Encoder) {
    out.object_start();
    out.key(b"settings");
    out.object_start();
    out.key(b"against");
    out.unsigned(u64::from(s.against));
    out.key(b"items");
    out.array_start();
    for item in &s.items {
        out.object_start();
        out.key(b"setting");
        settings::encode_setting(item.setting, out);
        out.key(b"edit");
        out.string(settings::edit_word(&item.edit));
        out.object_end();
    }
    out.array_end();
    out.key(b"review");
    match &s.review {
        Some(review) => encode_review(review, out),
        None => out.null(),
    }
    out.key(b"made");
    match s.made {
        Some(n) => out.unsigned(u64::from(n)),
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}
fn encode_review(review: &c::SettingsReview, out: &mut Encoder) {
    out.object_start();
    out.key(b"newest");
    out.unsigned(u64::from(review.newest));
    out.key(b"items");
    out.array_start();
    for item in &review.items {
        out.object_start();
        out.key(b"setting");
        settings::encode_setting(item.setting, out);
        out.key(b"edit");
        out.string(settings::edit_word(&item.edit));
        out.key(b"now");
        encode_review_value(&item.now, out);
        out.key(b"origin");
        match &item.origin {
            c::SettingOrigin::Default(r) | c::SettingOrigin::Change(r) => {
                out.object_start();
                out.key(if matches!(item.origin, c::SettingOrigin::Default(_)) { b"default" } else { b"change" });
                out.object_start();
                out.key(b"revision");
                out.unsigned(u64::from(r.revision));
                out.object_end();
                out.object_end();
            }
            c::SettingOrigin::NotOffered => out.string(b"not_offered"),
        }
        out.key(b"would_be");
        encode_review_value(&item.would_be, out);
        out.object_end();
    }
    out.array_end();
    out.key(b"cut");
    out.boolean(review.cut);
    out.object_end();
}
fn encode_review_value(value: &c::ReviewValue, out: &mut Encoder) {
    match value {
        c::ReviewValue::Value(set) => {
            out.object_start();
            out.key(b"value");
            out.object_start();
            out.key(b"value");
            out.object_start();
            out.key(match &set.value {
                c::SettingValue::Charter(_) => b"charter",
                c::SettingValue::Template(_) => b"template",
                c::SettingValue::Policy(_) => b"policy",
                c::SettingValue::Rules(_) => b"rules",
            });
            settings::encode_value(&set.value, out);
            out.object_end();
            out.object_end();
            out.object_end();
        }
        c::ReviewValue::TooLarge(size) => {
            out.object_start();
            out.key(b"too_large");
            out.object_start();
            out.key(b"bytes");
            out.unsigned(size.bytes);
            out.object_end();
            out.object_end();
        }
        c::ReviewValue::None => out.string(b"none"),
    }
}
