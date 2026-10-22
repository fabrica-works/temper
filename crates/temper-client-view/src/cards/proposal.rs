//! The proposal's task specifications, executors and authority are delivered facts.
use super::{P, phrase, specification, status, text};
use crate::words::{Word, Words};
use jig_client::contract as c;
use skein_ui::{Builder, Role, phrases::Hole};
pub(super) fn draw(b: &mut Builder, page: &c::ReadValue, words: &Words, rows: u32) {
    reason(b, page);
    facts(b, page, words, rows);
}
fn batch(page: &c::ReadValue) -> Option<&skein_lib::List<c::ProposedTask>> {
    match page {
        c::ReadValue::OwnerProposal(proposal) => match &proposal.action {
            c::OwnerProposalAction::Batch(tasks) => Some(tasks),
            c::OwnerProposalAction::Effect { .. }
            | c::OwnerProposalAction::ProcedureEffect { .. }
            | c::OwnerProposalAction::Amend { .. }
            | c::OwnerProposalAction::Widen { .. }
            | c::OwnerProposalAction::Release { .. } => None,
        },
        c::ReadValue::Proposal(proposal) => match &proposal.action {
            c::ProposalAction::Batch { tasks, .. } => Some(tasks),
            c::ProposalAction::Effect { .. } | c::ProposalAction::Amend { .. } | c::ProposalAction::Widen { .. } => {
                None
            }
        },
        c::ReadValue::Task(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::OwnerProject(_)
        | c::ReadValue::Historical(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::Escalation(_)
        | c::ReadValue::OwnerEscalation(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::OwnerPersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::OwnerEnded(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::OwnerInbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::OwnerSpend(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_)
        | c::ReadValue::OwnerDeployment(_) => None,
    }
}
pub(super) fn reason(b: &mut Builder, page: &c::ReadValue) {
    if let Some(tasks) = batch(page) {
        for task in tasks {
            let written = task.spec.specification.words.bytes();
            let end = skein_lib::bytes::find(written, b"\n").unwrap_or(written.len());
            text(b, Role::Heading { level: 2 }, written.get(..end).expect("title prefix"));
            if end < written.len() {
                skein_ui::markdown::read(written.get(end..).expect("spec body"), 3, b);
            }
        }
    } else {
        why(b, page);
    }
}
fn why(b: &mut Builder, page: &c::ReadValue) {
    match page {
        c::ReadValue::OwnerProposal(proposal) => super::document(b, &proposal.reason),
        c::ReadValue::Proposal(proposal) => text(b, Role::Paragraph, proposal.why.bytes()),
        c::ReadValue::Task(_)
        | c::ReadValue::Tasks(_)
        | c::ReadValue::OwnerProject(_)
        | c::ReadValue::Historical(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::Escalation(_)
        | c::ReadValue::OwnerEscalation(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::OwnerPersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::OwnerEnded(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::OwnerInbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::OwnerSpend(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_)
        | c::ReadValue::OwnerDeployment(_) => {}
    }
}
pub(super) fn facts(b: &mut Builder, page: &c::ReadValue, words: &Words, rows: u32) {
    if let Some(tasks) = batch(page) {
        batch_facts(b, page, tasks, words, rows);
        return;
    }
    match page {
        c::ReadValue::OwnerProposal(proposal) => match &proposal.action {
            c::OwnerProposalAction::Batch(tasks) => proposed(b, tasks, words, rows),
            c::OwnerProposalAction::Effect { part, authority: value, .. }
            | c::OwnerProposalAction::ProcedureEffect { part, authority: value, .. } => {
                crate::forge::owner_part(b, part, words);
                authority(b, value, words, rows);
            }
            c::OwnerProposalAction::Amend { task, before, after, .. } => {
                phrase(b, words, P::ProposalAmend, &[Hole::Number(task.number())]);
                specification(b, &after.spec, words);
                authority_change(b, &before.authority, &after.authority, words, rows);
            }
            c::OwnerProposalAction::Widen { task, before, after } => {
                phrase(b, words, P::ProposalTask, &[Hole::Number(task.number())]);
                authority_change(b, before, after, words, rows);
            }
            c::OwnerProposalAction::Release { task, hold } => {
                phrase(b, words, P::ProposalTask, &[Hole::Number(task.number())]);
                if let Some(hold) = hold {
                    held(b, hold, words, rows);
                }
            }
        },
        c::ReadValue::Proposal(proposal) => {
            match &proposal.action {
                c::ProposalAction::Batch { tasks, .. } => proposed(b, tasks, words, rows),
                c::ProposalAction::Effect { part } => crate::forge::drawn(b, part, words),
                c::ProposalAction::Amend { task, .. } => phrase(b, words, P::ProposalAmend, &[Hole::Number(*task)]),
                c::ProposalAction::Widen { .. } => phrase(b, words, P::Authority, &[]),
            }
            phrase(b, words, P::Spend, &[Hole::Number(proposal.needs.budget)]);
            phrase(
                b,
                words,
                P::Delegation,
                &[Hole::Number(u64::from(proposal.needs.tasks)), Hole::Number(u64::from(proposal.needs.depth))],
            );
            for grant in window(b, words, proposal.needs.grants.as_slice(), rows) {
                crate::forge::drawn(b, &c::Part::Kind { connector: grant.connector, kind: grant.kind }, words);
                pattern(b, &grant.pattern, words, rows);
            }
        }
        c::ReadValue::Task(task) => task_hold(b, task, words, rows),
        c::ReadValue::OwnerEscalation(escalation) => task_hold(b, &escalation.task, words, rows),
        c::ReadValue::Escalation(escalation) => {
            phrase(b, words, P::HoldRevision, &[Hole::Number(escalation.hold.revision.value)]);
        }
        c::ReadValue::Tasks(_)
        | c::ReadValue::OwnerProject(_)
        | c::ReadValue::Historical(_)
        | c::ReadValue::Me(_)
        | c::ReadValue::Question(_)
        | c::ReadValue::PersonTask(_)
        | c::ReadValue::OwnerPersonTask(_)
        | c::ReadValue::Result(_)
        | c::ReadValue::Ended(_)
        | c::ReadValue::OwnerEnded(_)
        | c::ReadValue::History(_)
        | c::ReadValue::Turn(_)
        | c::ReadValue::Activity(_)
        | c::ReadValue::Inbox(_)
        | c::ReadValue::OwnerInbox(_)
        | c::ReadValue::Spend(_)
        | c::ReadValue::OwnerSpend(_)
        | c::ReadValue::Project(_)
        | c::ReadValue::Notes(_)
        | c::ReadValue::Deployment(_)
        | c::ReadValue::OwnerDeployment(_) => {}
    }
}
fn batch_facts(
    b: &mut Builder,
    page: &c::ReadValue,
    tasks: &skein_lib::List<c::ProposedTask>,
    words: &Words,
    rows: u32,
) {
    b.open(Role::Slot);
    b.style(crate::look::PROPOSAL_FACTS);
    text(b, Role::Term, &words.say(Word::ProposalWhy, &[]));
    b.open(Role::Description);
    why(b, page);
    b.close();
    proposed(b, tasks, words, rows);
    text(b, Role::Term, &words.say(Word::ProposalBudget, &[]));
    b.open(Role::Description);
    super::decision(b, words, Some(page), c::ActKind::Accept);
    b.close();
    b.close();
    for task in window(b, words, tasks.as_slice(), rows) {
        phrase(b, words, P::ProposalBudget, &[Hole::Number(task.budget)]);
    }
    if let c::ReadValue::Proposal(proposal) = page {
        phrase(b, words, P::Spend, &[Hole::Number(proposal.needs.budget)]);
        phrase(
            b,
            words,
            P::Delegation,
            &[Hole::Number(u64::from(proposal.needs.tasks)), Hole::Number(u64::from(proposal.needs.depth))],
        );
        for grant in window(b, words, proposal.needs.grants.as_slice(), rows) {
            crate::forge::drawn(b, &c::Part::Kind { connector: grant.connector, kind: grant.kind }, words);
            pattern(b, &grant.pattern, words, rows);
        }
    }
}

fn task_hold(b: &mut Builder, task: &c::Task, words: &Words, rows: u32) {
    match &task.hold {
        Some(c::TaskHold::Owner(hold)) => held(b, hold, words, rows),
        Some(c::TaskHold::Document(hold)) => phrase(b, words, P::HoldRevision, &[Hole::Number(hold.revision.value)]),
        None => {}
    }
}
fn held(b: &mut Builder, hold: &c::OwnerChipHold, words: &Words, _rows: u32) {
    super::hold(b, &hold.reason, words);
    phrase(b, words, P::HoldRevision, &[Hole::Number(hold.revision)]);
}
fn proposed(b: &mut Builder, tasks: &skein_lib::List<c::ProposedTask>, words: &Words, rows: u32) {
    for task in window(b, words, tasks.as_slice(), rows) {
        text(b, Role::Term, &words.say(Word::Goal, &[]));
        b.open(Role::Description);
        phrase(b, words, P::ProposalTask, &[Hole::Number(task.number)]);
        super::specification_facts(b, &task.spec, words);
        match task.executor {
            c::OwnerExecutor::Agent { charter } => phrase(b, words, P::Charter, &[Hole::Number(u64::from(charter))]),
            c::OwnerExecutor::Procedure { code, .. } => {
                phrase(b, words, P::ProcedureCode, &[Hole::Number(u64::from(code))]);
            }
            c::OwnerExecutor::Person(_) => words.text(b, Word::PersonTask, &[]),
        }
        authority(b, &task.authority, words, rows);
        for dependency in &task.dependencies {
            phrase(b, words, P::Input, &[Hole::Number(*dependency)]);
        }
        if task.cut {
            status(b, words, P::Cut, &[]);
        }
        b.close();
        text(b, Role::Term, &words.say(Word::ProposalLanding, &[]));
        b.open(Role::Description);
        for resource in window(b, words, task.resources.as_slice(), rows) {
            b.open(Role::Paragraph);
            crate::forge::owner_part(b, resource, words);
            b.close();
        }
        if let Some(priority) = task.priority {
            text(b, Role::Paragraph, &words.say(Word::Priority, &[Hole::Number(u64::from(priority))]));
        }
        b.close();
    }
}
pub(super) fn has_rows(page: &c::ReadValue) -> bool {
    batch(page).is_some()
}
fn authority_change(
    b: &mut Builder,
    before: &c::OwnerAuthorityDocument,
    after: &c::OwnerAuthorityDocument,
    words: &Words,
    rows: u32,
) {
    phrase(b, words, P::AuthorityBefore, &[]);
    authority(b, before, words, rows);
    phrase(b, words, P::AuthorityAfter, &[]);
    authority(b, after, words, rows);
}
fn authority(b: &mut Builder, document: &c::OwnerAuthorityDocument, words: &Words, rows: u32) {
    let value = &document.authority;
    phrase(b, words, P::Spend, &[Hole::Number(value.spend)]);
    phrase(b, words, P::Tools, &[Hole::Number(value.tools)]);
    phrase(b, words, P::NoteBits, &[Hole::Number(u64::from(value.notes))]);
    phrase(
        b,
        words,
        P::Delegation,
        &[Hole::Number(u64::from(value.delegation.tasks)), Hole::Number(u64::from(value.delegation.depth))],
    );
    if let Some(deadline) = value.deadline {
        b.text(skein_ui::Run::Plain, &words.deadline(deadline));
    }
    for kind in window(b, words, value.delegation.kinds.as_slice(), rows) {
        let (word, number) = match kind {
            c::OwnerAuthorityExecutor::Charter(number) => (P::Charter, *number),
            c::OwnerAuthorityExecutor::Procedure(number) => (P::ProcedureCode, *number),
            c::OwnerAuthorityExecutor::Role(number) => (P::PersonRole, *number),
        };
        phrase(b, words, word, &[Hole::Number(u64::from(number))]);
    }
    for grant in window(b, words, value.grants.as_slice(), rows) {
        crate::forge::drawn(b, &c::Part::Kind { connector: grant.connector, kind: grant.kind }, words);
        pattern(b, &grant.pattern, words, rows);
    }
    for scope in window(b, words, value.note_resources.as_slice(), rows) {
        pattern(b, &scope.pattern, words, rows);
    }
    if document.cut {
        status(b, words, P::Cut, &[]);
    }
}
fn pattern(b: &mut Builder, pattern: &c::OwnerPattern, words: &Words, _rows: u32) {
    for segment in &pattern.segments {
        phrase(b, words, P::PatternSegment, &[super::name(segment)]);
    }
    let (word, bytes) = match &pattern.last {
        c::OwnerPatternEnd::Exact(bytes) => (P::PatternExact, bytes),
        c::OwnerPatternEnd::Open(bytes) => (P::PatternOpen, bytes),
    };
    phrase(b, words, word, &[super::name(bytes)]);
}

fn window<'a, T>(b: &mut Builder, words: &Words, values: &'a [T], rows: u32) -> &'a [T] {
    let count = values.len().min(usize::try_from(rows).expect("row bound fits"));
    if count < values.len() {
        status(b, words, P::Cut, &[]);
    }
    values.get(..count).expect("bounded prefix")
}
