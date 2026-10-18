//! The person's chats: a composer, live lines, then one page of closed lines.
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::{EndedGroup, TaskLine};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{Address, Domain, pages::Act};
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
    text_limit: u32,
) {
    b.open(Role::Main);
    text(b, Role::Heading { level: 1 }, &words.say(Word::Chats, &[]));
    if let Some(chats) = root.pages().chats() {
        if root.pages().me(root.client()).is_some() && root.pages().project().is_some() {
            b.open(Role::TextArea);
            b.name(&words.say(Word::StartChat, &[]));
            b.value(&chats.words, temper_client_domain::pages::shown(root.pages()));
            b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
            b.close();
            control(b, bindings, Role::Button, &words.say(Word::Start, &[]), Act::StartChat);
        }
        if chats.refusal.is_some() {
            jig.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit);
        }
        if let Some(outcome) = &chats.outcome {
            outcome_text(b, jig, outcome, text_limit);
        }
        text(b, Role::Heading { level: 2 }, &words.say(Word::LiveChats, &[]));
        if let Some(live) = chats.live(root.client()) {
            for line in &live.rows {
                row(b, root, line, words, bindings);
            }
        }
        text(b, Role::Heading { level: 2 }, &words.say(Word::ClosedChats, &[]));
        if let Some(page) = &chats.page
            && let EndedGroup::Rows { rows, after, .. } = &page.ended
        {
            for line in rows {
                row(b, root, line, words, bindings);
            }
            if after.is_some() {
                control(b, bindings, Role::Button, &words.say(Word::MoreClosedChats, &[]), Act::ClosedChats);
            }
        }
    }
    b.close();
}
pub(crate) fn row(b: &mut Builder, root: &Domain, line: &TaskLine, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Item);
    b.key(skein_ui::Key { owner: crate::frame::owner(), kind: 10, a: line.chip.task, b: 0 });
    control(
        b,
        bindings,
        Role::Link,
        &words.say(Word::Chat, &[Hole::Number(line.chip.task)]),
        Act::Go { address: Address::Chat { task: line.chip.task } },
    );
    text(b, Role::Paragraph, line.chip.title.bytes());
    if let Some(me) = root.pages().me(root.client()) {
        let status = jig_client::facts::chat_status(&line.chip, &line.waits_on, line.project, me);
        let holes = match &status {
            jig_client::facts::ChatStatus::Tool { tool, .. } => Some(Hole::Name(tool.clone())),
            jig_client::facts::ChatStatus::Starting
            | jig_client::facts::ChatStatus::Writing
            | jig_client::facts::ChatStatus::Working { .. }
            | jig_client::facts::ChatStatus::WaitingForYou { .. }
            | jig_client::facts::ChatStatus::WaitingFor(_)
            | jig_client::facts::ChatStatus::Parked
            | jig_client::facts::ChatStatus::Retrying { .. }
            | jig_client::facts::ChatStatus::Held(_)
            | jig_client::facts::ChatStatus::Closing
            | jig_client::facts::ChatStatus::Ended { .. } => None,
        };
        text(b, Role::Status, &words.say_word(jig_client::words::Word::status(&status), holes.as_slice()));
    }
    text(
        b,
        Role::Paragraph,
        &words.say(
            Word::ChatSpend,
            &[Hole::Number(line.budget.numbers.spent), Hole::Number(line.budget.numbers.spent_below)],
        ),
    );
    if let jig_client::contract::ChipPhase::Owner { details, .. } = line.chip.details.as_ref() {
        text(b, Role::Paragraph, &words.say(Word::ChatDelegates, &[Hole::Number(u64::from(details.live_delegates))]));
    }
    b.close();
}

fn outcome_text(b: &mut Builder, jig: &jig_client_view::words::Words, outcome: &jig_client::Outcome, text_limit: u32) {
    use jig_client::{Answered, Outcome};
    use jig_client_view::phrases::jig::JigPhrase as P;
    match outcome {
        Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Done(value))) => jig.outcome(b, value),
        Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Refused { refusal, .. })) => {
            jig.refusal(b, refusal, text_limit);
        }
        Outcome::NotKept => jig.status(b, P::RequestNotKept, &[]),
        Outcome::Unknown => jig.status(b, P::Unknown, &[]),
        Outcome::Limit => jig.status(b, P::RequestsInFlight, &[]),
        Outcome::Unconfirmed => jig.status(b, P::RequestUnconfirmed, &[]),
        Outcome::Answered(Answered::Family(_)) => {}
    }
}
