//! The chat's chronological lines come from jig's thread, without reordering.
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{
    contract as c,
    thread::{Line, Thread, Words as WordState},
};
use jig_client_view::{phrases::jig::JigPhrase as P, words::Words as JigWords};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{Act, chat::Chat},
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
    jig: &JigWords,
    bindings: &mut Bindings,
    jig_bindings: &mut jig_client_view::JigBindings,
) {
    b.open(Role::Main);
    if let Some(chat) = root.pages().chat()
        && let Some(task) = chat.task(root.client())
    {
        text(b, Role::Heading { level: 1 }, &words.say(Word::Chat, &[Hole::Number(chat.task)]));
        text(b, Role::Paragraph, task.chip.title.bytes());
        header(b, task, root.pages().me(root.client()), words);
        conversation(b, &chat.thread, root, words, jig, jig_bindings);
        if let Some(result) = &task.result {
            result_text(b, result, root, jig, jig_bindings);
        }
        composer(b, chat, task, words, bindings, temper_client_domain::pages::shown(root.pages()));
        for (key, _) in &chat.pending {
            if let Some(pending) = root.client().pending(*key) {
                crate::confirm::progress_text(b, jig, pending.progress);
            }
        }
        if let Some(outcome) = &chat.outcome {
            super::chats::outcome_text(b, jig, outcome, jig_bindings.text_limit());
        }
    }
    b.close();
}
fn result_text(
    b: &mut Builder,
    result: &c::TaskResult,
    root: &Domain,
    words: &JigWords,
    bindings: &mut jig_client_view::JigBindings,
) {
    let client = root.jig().borrow(root.client());
    match result {
        c::TaskResult::Document(result) => jig_client_view::cards::result::facts(b, &client, result, words, bindings),
        c::TaskResult::Ending(ending) => match ending.as_ref() {
            c::OwnerChipEnding::Done(value) => {
                jig_client_view::cards::result::value_nodes(b, &client, value, words, bindings);
            }
            c::OwnerChipEnding::Failed(reason) => skein_ui::markdown::read(reason.bytes(), 2, b),
            c::OwnerChipEnding::Cancelled { reason, result, .. } => {
                skein_ui::markdown::read(reason.bytes(), 2, b);
                if let Some(value) = result {
                    jig_client_view::cards::result::value_nodes(b, &client, value, words, bindings);
                }
            }
        },
    }
}
pub(crate) fn header(b: &mut Builder, task: &c::Task, me: Option<&c::Me>, words: &Words) {
    if let Some(me) = me {
        super::chats::status(b, &task.chip, &task.waits_on, task.project, me, words);
    }
    text(
        b,
        Role::Paragraph,
        &words.say(
            Word::ChatSpend,
            &[Hole::Number(task.budget.numbers.spent), Hole::Number(task.budget.numbers.spent_below)],
        ),
    );
}
fn composer(b: &mut Builder, chat: &Chat, task: &c::Task, words: &Words, bindings: &mut Bindings, shown: u64) {
    if temper_client_domain::pages::chat::allowed(task, c::ActKind::Message) {
        b.open(Role::TextArea);
        b.name(&words.say(Word::Message, &[]));
        b.value(&chat.words, shown);
        b.bind(bindings.insert(Act::MessageWords { words: Box::default() }));
        b.close();
        control(b, bindings, Role::Button, &words.say(Word::Send, &[]), Act::Send);
    }
    if temper_client_domain::pages::chat::allowed(task, c::ActKind::Stop) {
        control(b, bindings, Role::Button, &words.say(Word::Stop, &[]), Act::Stop);
    }
    control(b, bindings, Role::Button, &words.say(Word::MarkSeen, &[]), Act::Seen);
    if let jig_client::thread::window::Edge::More(_) = chat.thread.lines.edges.after {
        control(b, bindings, Role::Button, &words.say(Word::Older, &[]), Act::Older);
    }
}
pub(crate) fn conversation(
    b: &mut Builder,
    thread: &Thread,
    root: &Domain,
    words: &Words,
    jig: &JigWords,
    bindings: &mut jig_client_view::JigBindings,
) {
    b.open(Role::Log);
    b.name(&jig.say(P::ConversationLog, &[]));
    for line in thread.lines().into_boxed() {
        b.open(Role::Region);
        match line {
            Line::Words { by, words, state } => {
                if let Some(by) = by {
                    text(b, Role::Paragraph, by.name.bytes());
                }
                text(b, Role::Paragraph, words);
                jig.status(
                    b,
                    match state {
                        WordState::BeingSent => P::WordsBeingSent,
                        WordState::Sent { .. } => P::WordsSent,
                        WordState::Read { .. } => P::WordsRead,
                    },
                    &[],
                );
            }
            Line::Turn { turn, .. } => committed(b, turn, jig),
            Line::Provisional { text, cut, .. } => {
                jig.status(b, P::Provisional, &[]);
                skein_ui::markdown::read(text, 2, b);
                if cut {
                    jig.status(b, P::TurnCut, &[]);
                }
            }
            Line::Undecoded { why, .. } => match why {
                c::TurnProblem::Version { found } => {
                    jig.status(b, P::TurnUnreadVersion, &[Hole::Number(u64::from(found))]);
                }
                c::TurnProblem::Malformed => jig.status(b, P::TurnUnreadMalformed, &[]),
                c::TurnProblem::TooLarge => jig.status(b, P::TurnUnreadTooLarge, &[]),
            },
            Line::RunBegan { attempt } => {
                text(b, Role::Paragraph, &words.say(Word::RunBegan, &[Hole::Number(attempt)]));
            }
            Line::RunEnded { attempt } => {
                text(b, Role::Paragraph, &words.say(Word::RunEnded, &[Hole::Number(attempt)]));
            }
            Line::Left { lines } => {
                text(b, Role::Status, &words.say(Word::LinesLeft, &[Hole::Number(u64::from(lines))]));
            }
            Line::Activity { line } => match &line.event {
                c::Activity::Owner { source, .. } => jig_client_view::owner_activity::render(b, source, jig),
                c::Activity::Result { result } => {
                    jig_client_view::cards::result::facts(b, &root.jig().borrow(root.client()), result, jig, bindings);
                }
                c::Activity::News { summary, .. } => text(b, Role::Paragraph, summary.bytes()),
                c::Activity::Decision { by, how, .. } => {
                    jig.actor(b, *by);
                    jig.decision(b, how);
                }
                c::Activity::Words { .. }
                | c::Activity::RunBegan { .. }
                | c::Activity::RunEnded { .. }
                | c::Activity::Turn { .. } => {}
            },
        }
        b.close();
    }
    b.close();
}
fn committed(b: &mut Builder, turn: &c::Turn, jig: &JigWords) {
    for message in &turn.messages {
        jig.text(
            b,
            match message.speaker {
                c::Speaker::Person => P::Person,
                c::Speaker::Assistant => P::Agent,
                c::Speaker::Tool => P::CallCompleted,
                c::Speaker::System => P::TurnInstructions,
            },
            &[],
        );
        for block in &message.blocks {
            match block {
                c::Block::Text(words) => skein_ui::markdown::read(words.bytes(), 2, b),
                c::Block::Call(call) => {
                    text(b, Role::Paragraph, call.tool.bytes());
                    if let Some(summary) = &call.outcome {
                        text(b, Role::Paragraph, summary.bytes());
                    }
                }
                c::Block::Result { summary, failed, .. } => {
                    jig.text(b, if *failed { P::CallFailed } else { P::CallCompleted }, &[]);
                    text(b, Role::Paragraph, summary.bytes());
                }
                c::Block::Opaque { dialect, size } => {
                    jig.text(b, P::OpaqueDialect, &[jig_client_view::words::name(dialect.bytes())]);
                    jig.text(b, P::OpaqueBytes, &[Hole::Number(*size)]);
                }
            }
        }
    }
    if turn.cut {
        jig.status(b, P::TurnCut, &[]);
    }
}
