//! The person's chats: a composer, live lines, then one page of closed lines.
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::{EndedGroup, TaskLine};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{Address, Domain, pages::Act};
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn control(b: &mut Builder, bindings: &mut Bindings, role: Role, name: &[u8], act: Act) {
    if role == Role::Link {
        crate::link(b, name, bindings.insert(act));
        b.text(Run::Plain, name);
    } else {
        b.open(role);
        b.name(name);
        b.bind(bindings.insert(act));
    }
    b.close();
}
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, text_limit: u32) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    let demo = demo_start(root);
    if let Some(demo) = demo {
        start(b, root, demo, words, bindings);
    } else {
        text(b, Role::Heading { level: 1 }, &words.say(Word::Chats, &[]));
    }
    if let Some(chats) = root.pages().chats() {
        if root.pages().me(root.client()).is_some() && root.pages().project().is_some() {
            crate::choice::composer(b, root, None, words, bindings);
            b.open(Role::Slot);
            b.style(crate::look::ENTRY);
            b.open(Role::TextArea);
            b.name(&words.say(Word::StartChat, &[]));
            b.state(State::SubmitsOnEnter);
            b.style(crate::look::COMPOSER);
            b.value(&chats.words, temper_client_domain::pages::shown(root.pages()));
            b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
            b.close();
            b.open(Role::Paragraph);
            b.style(crate::look::HINT);
            words.text(b, Word::ChatHint, &[]);
            b.close();
            if let Some(me) = root.pages().me(root.client())
                && root.pages().choice().valid(me)
            {
                b.open(Role::Button);
                b.name(&words.say(Word::Start, &[]));
                if chats.start.is_some() {
                    b.state(State::Disabled);
                }
                b.bind(bindings.insert(Act::StartChat));
                b.close();
            }
            b.close();
        }
        if chats.refusal.is_some() {
            words.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit);
        }
        if let Some(outcome) = &chats.outcome {
            outcome_text(b, words, outcome, text_limit);
        }
        let live = chats.live(root.client());
        let has_live = match live {
            Some(live) => !live.rows.is_empty(),
            None => false,
        };
        if demo.is_none() || has_live {
            text(b, Role::Heading { level: 2 }, &words.say(Word::LiveChats, &[]));
        }
        if let Some(live) = live
            && !live.rows.is_empty()
        {
            b.open(Role::List);
            b.style(crate::look::ROWS);
            for line in &live.rows {
                row(b, root, line, words, bindings);
            }
            b.close();
        }
        let closed = if let Some(page) = &chats.page
            && let EndedGroup::Rows { rows, after, .. } = &page.ended
        {
            Some((rows, after))
        } else {
            None
        };
        let has_closed = match closed {
            Some((rows, after)) => !rows.is_empty() || after.is_some(),
            None => false,
        };
        if demo.is_none() || has_closed {
            text(b, Role::Heading { level: 2 }, &words.say(Word::ClosedChats, &[]));
        }
        if let Some((rows, after)) = closed {
            if !rows.is_empty() {
                b.open(Role::List);
                b.style(crate::look::ROWS);
                for line in rows {
                    row(b, root, line, words, bindings);
                }
                b.close();
            }
            if after.is_some() {
                control(b, bindings, Role::Button, &words.say(Word::MoreClosedChats, &[]), Act::ClosedChats);
            }
        }
    }
    b.close();
}
fn start(
    b: &mut Builder,
    root: &Domain,
    demo: &temper_client_domain::forge::Demo,
    words: &Words,
    bindings: &mut Bindings,
) {
    b.open(Role::Slot);
    b.style(crate::look::CHATBAR);
    b.open(Role::Slot);
    text(b, Role::Paragraph, &words.say(Word::NewChat, &[]));
    b.open(Role::Status);
    b.style(crate::look::STATUS_QUIET);
    let ready = words.say(Word::Ready, &[]);
    b.name(&ready);
    b.text(Run::Plain, &ready);
    b.close();
    b.close();
    b.open(Role::Slot);
    text(b, Role::Paragraph, &words.say(Word::DemoModelName, &[]));
    b.close();
    b.close();
    crate::frame::demo_banner(b, root, words, bindings);
    if let Some(project) = root.pages().project() {
        super::chat::context(b, root, project, words);
    }
    suggestions(b, root, demo, words, bindings);
}

pub(crate) fn row(b: &mut Builder, root: &Domain, line: &TaskLine, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Item);
    b.style(crate::look::ROW);
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
        status(b, &line.chip, &line.waits_on, (line.project, line.requester), None, me, words);
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
    b.open(Role::Paragraph);
    b.style(crate::look::TIME);
    b.text(Run::Plain, &words.chat_time(line.ended_at.unwrap_or(line.last_activity)));
    b.close();
    b.close();
}

pub(crate) fn outcome_text(b: &mut Builder, words: &Words, outcome: &jig_client::Outcome, text_limit: u32) {
    use crate::cards::P;
    use jig_client::{Answered, Outcome};
    match outcome {
        Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Done(value))) => words.outcome(b, value),
        Outcome::Answered(Answered::Jig(jig_client::contract::Answer::Refused { refusal, .. })) => {
            words.refusal(b, refusal, text_limit);
        }
        Outcome::NotKept | Outcome::Held => words.status(b, P::RequestNotKept, &[]),
        Outcome::Unknown => words.status(b, P::Unknown, &[]),
        Outcome::Limit => words.status(b, P::RequestsInFlight, &[]),
        Outcome::Unconfirmed => words.status(b, P::RequestUnconfirmed, &[]),
        Outcome::Answered(Answered::Family(_)) => {}
        Outcome::Unsent | Outcome::Lost => unreachable!("temper's requests carry no value"),
    }
}

pub(crate) fn status(
    b: &mut Builder,
    chip: &jig_client::contract::Chip,
    waits_on: &jig_client::contract::WaitsOn,
    (project, requester): (u32, jig_client::contract::OwnerRequester),
    wake: Option<&jig_client::contract::OwnerWakePolicy>,
    me: &jig_client::contract::Me,
    words: &Words,
) {
    use jig_client::contract::{ChipPhase, OwnerRequester, WaitsOn, WakeRule};
    let narrowed = if *waits_on == WaitsOn::Delegates
        && let ChipPhase::Owner { details, .. } = chip.details.as_ref()
        && details.live_delegates > 0
    {
        Some(words.say(Word::WaitingForDelegates, &[Hole::Number(u64::from(details.live_delegates))]))
    } else if *waits_on == WaitsOn::News
        && requester == OwnerRequester::Person(me.party.number)
        && let Some(wake) = wake
        && match wake.words {
            WakeRule::Never => false,
            WakeRule::Immediate | WakeRule::Batch { .. } => true,
        }
    {
        Some(words.say(Word::WaitingForYourWords, &[]))
    } else {
        None
    };
    if let Some(name) = narrowed {
        b.open(Role::Status);
        b.style(crate::look::STATUS_QUIET);
        b.name(&name);
        b.text(Run::Plain, &name);
        b.close();
        return;
    }
    let status = jig_client::facts::chat_status(chip, waits_on, project, me);
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
    let token = match status {
        jig_client::facts::ChatStatus::WaitingForYou { .. } => crate::look::STATUS_YOURS,
        jig_client::facts::ChatStatus::Held(_) => crate::look::STATUS_HELD,
        jig_client::facts::ChatStatus::Retrying { .. } => crate::look::STATUS_RETRYING,
        jig_client::facts::ChatStatus::Starting
        | jig_client::facts::ChatStatus::Writing
        | jig_client::facts::ChatStatus::Tool { .. }
        | jig_client::facts::ChatStatus::Working { .. } => crate::look::STATUS_ACTIVE,
        jig_client::facts::ChatStatus::WaitingFor(_)
        | jig_client::facts::ChatStatus::Parked
        | jig_client::facts::ChatStatus::Closing
        | jig_client::facts::ChatStatus::Ended { .. } => crate::look::STATUS_QUIET,
    };
    b.open(Role::Status);
    b.style(token);
    let name = words.say_word(jig_client::words::Word::status(&status), holes.as_slice());
    b.name(&name);
    b.text(Run::Plain, &name);
    b.close();
}

/// The demo whose start the Chats page shows, when it shows one.
pub(crate) fn demo_start(root: &Domain) -> Option<&temper_client_domain::forge::Demo> {
    if *root.pages().address() != Address::Chats {
        return None;
    }
    match root.demo() {
        Some((demo, halves)) if halves.model && root.pages().project() == Some(demo.project) => Some(demo),
        _ => None,
    }
}

fn suggestions(
    b: &mut Builder,
    root: &Domain,
    demo: &temper_client_domain::forge::Demo,
    words: &Words,
    bindings: &mut Bindings,
) {
    let Some(chats) = root.pages().chats() else { return };
    b.open(Role::Slot);
    b.style(crate::look::INVITATION);
    text(b, Role::Heading { level: 1 }, &words.say(Word::DemoHeading, &[]));
    text(
        b,
        Role::Paragraph,
        &words.say(Word::DemoRequest, &[Hole::Name(demo.owner.clone()), Hole::Name(demo.name.clone())]),
    );
    b.open(Role::List);
    b.name(&words.say(Word::SuggestedRequests, &[]));
    for (cue, value) in demo.cues.iter().enumerate() {
        b.open(Role::Item);
        b.open(Role::Button);
        b.name(value);
        if chats.start.is_some() {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert(Act::Suggest { cue: u8::try_from(cue).expect("eight cue indices fit") }));
        b.close();
        b.close();
    }
    b.close();
    text(b, Role::Paragraph, &words.say(Word::DemoScript, &[]));
    b.close();
}
