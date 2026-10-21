//! Home: needs first, updates afterwards, and deciding controls (06.3).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use jig_client::contract::{InboxGroup, ObjectKey, OwnerHistoricalValue, Proposer, ReadValue, ResultValue};
use skein_lib::Decimal;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
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
    if role == Role::Link {
        b.text(Run::Plain, name);
    }
    b.bind(bindings.insert(act));
    b.close();
}
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, rows: u32) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::PAGE);
    b.open(Role::Slot);
    b.style(look::PAGEBAR);
    text(b, Role::Heading { level: 1 }, &words.say(Word::InboxTitle, &[]));
    b.close();
    let Some(inbox) = root.pages().inbox() else {
        b.close();
        return;
    };
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
        b.open(Role::Slot);
        b.style(look::HEADING);
        text(b, Role::Heading { level: 2 }, &words.say(heading, &[]));
        if heading == Word::NeedsYou {
            filters(b, root, inbox.filter, words, bindings);
        } else if inbox.results_through().is_some() {
            control(b, bindings, Role::Button, &words.say(Word::MarkSeen, &[]), Act::Seen);
        }
        b.close();
        let mut opened = false;
        for group in groups.into_iter().flatten() {
            for card in &inbox.cards {
                if card.group == group {
                    if !opened {
                        b.open(Role::List);
                        b.style(look::CARDS);
                        if heading == Word::InboxUpdates {
                            b.style(look::UPDATES);
                        }
                        opened = true;
                    }
                    entry(b, root, card, words, bindings, rows);
                }
            }
        }
        if opened {
            b.close();
        }
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}
fn filter_control(b: &mut Builder, bindings: &mut Bindings, name: &[u8], filter: Filter, pressed: bool) {
    b.open(Role::Button);
    b.name(name);
    if pressed {
        b.state(State::Pressed);
    }
    b.bind(bindings.insert(Act::Filter { filter }));
    b.close();
}
#[expect(clippy::manual_map, reason = "view code spells out the optional goal")]
fn filters(b: &mut Builder, root: &Domain, filter: Filter, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::FILTERS);
    b.open(Role::Slot);
    filter_control(
        b,
        bindings,
        &words.say(Word::AllProjects, &[]),
        Filter { project: None, ..filter },
        filter.project.is_none(),
    );
    if let Some(me) = root.pages().me(root.client()) {
        for project in &me.projects {
            let fallback = words.say(Word::Project, &[Hole::Number(u64::from(project.project))]);
            let name = match &project.name {
                Some(name) => name.bytes(),
                None => &fallback,
            };
            filter_control(
                b,
                bindings,
                name,
                Filter { project: Some(project.project), ..filter },
                filter.project == Some(project.project),
            );
        }
    }
    b.close();
    b.open(Role::Slot);
    for (kind, word) in [
        (None, Word::AllKinds),
        (Some(Kind::Proposal), Word::Proposal),
        (Some(Kind::Question), Word::Question),
        (Some(Kind::Held), Word::Held),
        (Some(Kind::PersonTask), Word::PersonTask),
    ] {
        filter_control(b, bindings, &words.say(word, &[]), Filter { kind, ..filter }, filter.kind == kind);
    }
    b.close();
    b.open(Role::Slot);
    filter_control(
        b,
        bindings,
        &words.say(Word::AllGoals, &[]),
        Filter { goal: None, ..filter },
        filter.goal.is_none(),
    );
    b.open(Role::Slot);
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
    b.close();
    b.close();
    b.close();
}
fn entry(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings, rows: u32) {
    b.open(Role::Item);
    b.key(card_key(card.object));
    b.style(look::CARD);
    let (kind, style) = match card.group {
        InboxGroup::Proposals => (Word::Proposal, look::KIND_PROPOSAL),
        InboxGroup::Questions => (Word::Question, look::KIND_QUESTION),
        InboxGroup::Escalations => (Word::Held, look::KIND_HELD),
        InboxGroup::PersonTasks => (Word::PersonTask, look::KIND_PERSON_TASK),
        InboxGroup::Results => (Word::InboxUpdates, look::KIND_RESULT),
        InboxGroup::Replies => (Word::Reply, look::KIND_REPLY),
    };
    b.style(style);
    b.open(Role::Slot);
    b.style(look::META);
    text(b, Role::Heading { level: 3 }, &words.say(kind, &[]));
    if let Some(project) = card.project {
        let fallback = words.say(Word::Project, &[Hole::Number(u64::from(project))]);
        let mut name = fallback.as_ref();
        if let Some(me) = root.pages().me(root.client()) {
            for entry in &me.projects {
                if entry.project == project
                    && let Some(value) = &entry.name
                {
                    name = value.bytes();
                }
            }
        }
        text(b, Role::Paragraph, name);
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
            Act::Go { address: Address::Task { task } },
        );
    }
    b.close();
    if let Some(page) = &card.page {
        body(b, page, words);
        crate::cards::proposal_reason(b, page);
        if has_facts(page) {
            b.open(Role::Slot);
            b.style(look::DETAILS);
            crate::confirm::facts(b, page, words, rows);
            b.close();
        }
        crate::confirm::controls(b, root, card, words, bindings);
    }
    b.close();
}
fn has_facts(page: &ReadValue) -> bool {
    let proposal = match page {
        ReadValue::Proposal(_) | ReadValue::OwnerProposal(_) | ReadValue::Escalation(_) => true,
        ReadValue::Task(task) => task.hold.is_some(),
        ReadValue::OwnerEscalation(held) => held.task.hold.is_some(),
        ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::OwnerPersonTask(_)
        | ReadValue::Result(_)
        | ReadValue::Tasks(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Historical(_)
        | ReadValue::Me(_)
        | ReadValue::Ended(_)
        | ReadValue::OwnerEnded(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
        | ReadValue::Activity(_)
        | ReadValue::Inbox(_)
        | ReadValue::OwnerInbox(_)
        | ReadValue::Spend(_)
        | ReadValue::OwnerSpend(_)
        | ReadValue::Project(_)
        | ReadValue::Notes(_)
        | ReadValue::Deployment(_)
        | ReadValue::OwnerDeployment(_) => false,
    };
    let decision = jig_client::facts::decision(Some(page), jig_client::contract::ActKind::Accept);
    proposal
        || decision.dangerous
        || decision.effect.is_some()
        || !decision.funding.is_empty()
        || decision.funds.is_some()
}
fn title(b: &mut Builder, value: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::TITLE);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn body(b: &mut Builder, page: &ReadValue, words: &Words) {
    match page {
        ReadValue::Question(question) => {
            title(b, question.task.title.bytes());
            text(b, Role::Paragraph, question.words.bytes());
        }
        ReadValue::Escalation(held) => title(b, held.task.title.bytes()),
        ReadValue::OwnerEscalation(held) => {
            title(b, held.task.chip.title.bytes());
            crate::cards::specification(b, &held.task.spec, words);
        }
        ReadValue::PersonTask(task) => crate::cards::specification(b, &task.spec, words),
        ReadValue::OwnerPersonTask(task) => crate::cards::specification(b, &task.spec, words),
        ReadValue::Task(task) => {
            title(b, task.chip.title.bytes());
            crate::cards::specification(b, &task.spec, words);
        }
        ReadValue::Historical(page) => match &page.value {
            OwnerHistoricalValue::Task { document, .. } => title(b, document.chip.title.bytes()),
            OwnerHistoricalValue::Proposal(_)
            | OwnerHistoricalValue::PersonProposal { .. }
            | OwnerHistoricalValue::Escalation { .. }
            | OwnerHistoricalValue::Question { .. }
            | OwnerHistoricalValue::PersonTask { .. } => {}
        },
        ReadValue::Result(result) => {
            title(b, result.task.title.bytes());
            words.ending(b, &result.ending);
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
                crate::cards::document(b, value);
            }
        }
        ReadValue::OwnerProposal(_)
        | ReadValue::Proposal(_)
        | ReadValue::Tasks(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Me(_)
        | ReadValue::Ended(_)
        | ReadValue::OwnerEnded(_)
        | ReadValue::History(_)
        | ReadValue::Turn(_)
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

pub(crate) fn card_key(object: ObjectKey) -> skein_ui::Key {
    let (kind, a, b) = match object {
        ObjectKey::Task(task) => (0, task, 0),
        ObjectKey::Proposal(key) => match key.proposer {
            Proposer::Task(task) => (1, task, key.number),
            Proposer::Party(party) => (29, party, key.number),
        },
        ObjectKey::Escalation(task) => (2, task, 0),
        ObjectKey::Question { task, message } => (3, task, message),
        ObjectKey::PersonTask(task) => (4, task, 0),
        ObjectKey::Result(task) => (5, task, 0),
        ObjectKey::Note { project, note } => (6, u64::from(project), note),
    };
    skein_ui::Key { owner: crate::frame::owner(), kind, a, b }
}
