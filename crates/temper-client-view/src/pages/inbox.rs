//! Home: needs first, updates afterwards, without deciding controls (06.2).
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use jig_client::contract::{InboxGroup, ObjectKey, Proposer, ReadValue, ResultValue};
use skein_lib::Decimal;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    address::{Filter, Kind},
    pages::{Act, inbox::Card},
};
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn control(b: &mut Builder, bindings: &mut Bindings, role: Role, name: &[u8], act: Act) {
    b.open(role);
    b.name(name);
    b.bind(bindings.insert(act));
    b.close();
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
) {
    b.open(Role::Main);
    text(b, Role::Heading { level: 1 }, &words.say(Word::InboxTitle, &[]));
    let Some(inbox) = root.pages().inbox() else {
        b.close();
        return;
    };
    filters(b, root, inbox.filter, words, bindings);
    for (heading, groups) in [
        (
            Word::NeedsYou,
            [
                Some(InboxGroup::Proposals),
                Some(InboxGroup::Questions),
                Some(InboxGroup::Escalations),
                Some(InboxGroup::PersonTasks),
            ],
        ),
        (Word::InboxUpdates, [Some(InboxGroup::Results), Some(InboxGroup::Replies), None, None]),
    ] {
        text(b, Role::Heading { level: 2 }, &words.say(heading, &[]));
        for group in groups.into_iter().flatten() {
            for card in &inbox.cards {
                if card.group == group {
                    entry(b, card, words, jig, bindings);
                }
            }
        }
    }
    if inbox.results_through().is_some() {
        control(b, bindings, Role::Button, &words.say(Word::MarkSeen, &[]), Act::Seen);
    }
    b.close();
}
#[expect(clippy::manual_map, reason = "view code spells out the optional goal")]
fn filters(b: &mut Builder, root: &Domain, filter: Filter, words: &Words, bindings: &mut Bindings) {
    control(
        b,
        bindings,
        Role::Button,
        &words.say(Word::AllProjects, &[]),
        Act::Filter { filter: Filter { project: None, ..filter } },
    );
    if let Some(me) = root.pages().me(root.client()) {
        for project in &me.projects {
            let fallback = words.say(Word::Project, &[Hole::Number(u64::from(project.project))]);
            let name = match &project.name {
                Some(name) => name.bytes(),
                None => &fallback,
            };
            control(
                b,
                bindings,
                Role::Button,
                name,
                Act::Filter { filter: Filter { project: Some(project.project), ..filter } },
            );
        }
    }
    for (kind, word) in [
        (None, Word::AllKinds),
        (Some(Kind::Proposal), Word::Proposal),
        (Some(Kind::Question), Word::Question),
        (Some(Kind::Held), Word::Held),
        (Some(Kind::PersonTask), Word::PersonTask),
    ] {
        control(b, bindings, Role::Button, &words.say(word, &[]), Act::Filter { filter: Filter { kind, ..filter } });
    }
    control(
        b,
        bindings,
        Role::Button,
        &words.say(Word::AllGoals, &[]),
        Act::Filter { filter: Filter { goal: None, ..filter } },
    );
    b.open(Role::TextField);
    b.name(&words.say(Word::Goal, &[]));
    let value = match filter.goal {
        Some(goal) => Some(Decimal::of(goal)),
        None => None,
    };
    b.value(
        match &value {
            Some(value) => value.as_bytes(),
            None => b"",
        },
        temper_client_domain::pages::shown(root.pages()),
    );
    b.bind(bindings.insert(Act::Filter { filter }));
    b.close();
}
fn entry(b: &mut Builder, card: &Card, words: &Words, jig: &jig_client_view::words::Words, bindings: &mut Bindings) {
    b.open(Role::Item);
    let kind = match card.group {
        InboxGroup::Proposals => Word::Proposal,
        InboxGroup::Questions => Word::Question,
        InboxGroup::Escalations => Word::Held,
        InboxGroup::PersonTasks => Word::PersonTask,
        InboxGroup::Results => Word::InboxUpdates,
        InboxGroup::Replies => Word::Reply,
    };
    text(b, Role::Heading { level: 3 }, &words.say(kind, &[]));
    if let Some(project) = card.project {
        text(b, Role::Paragraph, &words.say(Word::Project, &[Hole::Number(u64::from(project))]));
    }
    let task = match card.object {
        ObjectKey::Task(task)
        | ObjectKey::Escalation(task)
        | ObjectKey::PersonTask(task)
        | ObjectKey::Result(task)
        | ObjectKey::Question { task, .. } => Some(task),
        ObjectKey::Proposal(proposal) => match proposal.proposer {
            Proposer::Task(task) => Some(task),
            Proposer::Party(_) => None,
        },
        ObjectKey::Note { .. } => None,
    };
    if let Some(task) = task {
        control(
            b,
            bindings,
            Role::Link,
            &words.say(Word::Task, &[Hole::Number(task)]),
            Act::Go { address: Address::Jig(jig_client_view::address::Address::task(task)) },
        );
    }
    if let Some(page) = &card.page {
        body(b, page, jig);
    }
    b.close();
}
fn body(b: &mut Builder, page: &ReadValue, jig: &jig_client_view::words::Words) {
    use jig_client_view::{document_text, specification};
    match page {
        ReadValue::Question(question) => {
            text(b, Role::Paragraph, question.task.title.bytes());
            text(b, Role::Paragraph, question.words.bytes());
        }
        ReadValue::OwnerProposal(proposal) => document_text::markdown(b, &proposal.reason, jig),
        ReadValue::Proposal(proposal) => text(b, Role::Paragraph, proposal.why.bytes()),
        ReadValue::Escalation(held) => document_text::plain(b, &held.task.title, jig),
        ReadValue::OwnerEscalation(held) => {
            document_text::plain(b, &held.task.chip.title, jig);
            specification::document(b, &held.task.spec, jig);
        }
        ReadValue::PersonTask(task) => specification::document(b, &task.spec, jig),
        ReadValue::OwnerPersonTask(task) => specification::document(b, &task.spec, jig),
        ReadValue::Task(task) => {
            document_text::plain(b, &task.chip.title, jig);
            specification::document(b, &task.spec, jig);
        }
        ReadValue::Result(result) => {
            document_text::plain(b, &result.task.title, jig);
            jig.ending(b, &result.ending);
            let value = match &result.value {
                Some(
                    ResultValue::Report(text)
                    | ResultValue::Failure(text)
                    | ResultValue::Words(text)
                    | ResultValue::OwnerVerdict { words: text, .. }
                    | ResultValue::Verdict { words: text, .. }
                    | ResultValue::Connector { summary: text, .. }
                    | ResultValue::Change { words: text, .. }
                    | ResultValue::Choice { reason: text, .. },
                ) => Some(text),
                Some(ResultValue::Approved { words, .. }) => words.as_ref(),
                None => None,
            };
            if let Some(value) = value {
                document_text::markdown(b, value, jig);
            }
        }
        ReadValue::Tasks(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Historical(_)
        | ReadValue::Me(_)
        | ReadValue::Ended(_)
        | ReadValue::OwnerEnded(_)
        | ReadValue::History(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::OwnerInbox(_)
        | ReadValue::Spend(_)
        | ReadValue::OwnerSpend(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_)
        | ReadValue::OwnerDeployment(_) => {}
    }
}
