//! The chat follows jig-client's thread in chronological order.
use crate::{cards::P, look};
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{
    contract as c,
    thread::{Line, Thread, Words as WordState},
};
use skein_lib::List;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{Act, chat::Chat, delegation},
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
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, rows: u32, text_limit: u32) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::CHAT);
    if let Some(chat) = root.pages().chat()
        && let Some(task) = chat.task(root.client())
    {
        b.open(Role::Slot);
        b.style(look::CHATBAR);
        b.open(Role::Slot);
        text(
            b,
            Role::Heading { level: 1 },
            &words.say(
                Word::ChatTitle,
                &[Hole::Number(chat.task), Hole::Name(skein_lib::bytes::copy_of(task.chip.title.bytes()))],
            ),
        );
        if let Some(me) = root.pages().me(root.client()) {
            super::chats::status(b, &task.chip, &task.waits_on, task.project, me, words);
        }
        if let Some((demo, halves)) = root.demo()
            && halves.model
            && task.project == demo.project
        {
            text(b, Role::Paragraph, &words.say(Word::DemoModelName, &[]));
        }
        b.close();
        b.open(Role::Slot);
        spend(b, task, words);
        if temper_client_domain::pages::chat::allowed(task, c::ActKind::Stop) {
            control(b, bindings, Role::Button, &words.say(Word::Stop, &[]), Act::Stop);
        }
        if temper_client_domain::pages::chat::allowed(task, c::ActKind::Cancel) {
            control(b, bindings, Role::Button, &words.say(Word::Close, &[]), Act::Close);
        }
        control(b, bindings, Role::Button, &words.say(Word::MarkSeen, &[]), Act::Seen);
        b.close();
        b.close();
        crate::frame::demo_banner(b, root, words, bindings);
        b.open(Role::Slot);
        b.style(look::THREAD);
        if let jig_client::thread::window::Edge::More(_) = chat.thread.lines.edges.after {
            control(b, bindings, Role::Button, &words.say(Word::Older, &[]), Act::Older);
        }
        let me = root.pages().me(root.client());
        #[expect(clippy::manual_map, reason = "view code spells out the signed-in person")]
        let person = match me {
            Some(me) => Some(me.party.number),
            None => None,
        };
        let requester = match task.requester {
            c::OwnerRequester::Person(number) => Some(number) == person,
            c::OwnerRequester::Task(_) | c::OwnerRequester::Deployment { .. } => false,
        };
        conversation(
            b,
            &chat.thread,
            words,
            (rows, text_limit),
            (person, requester),
            Some((chat, root.client())),
            bindings,
        );
        if !chat.cards.is_empty() || !chat.departures().is_empty() {
            b.open(Role::List);
            b.style(look::CARDS);
        }
        for card in &chat.cards {
            b.open(Role::Item);
            b.style(look::CARD);
            b.key(super::inbox::card_key(card.object));
            crate::confirm::card(b, root, card, words, bindings, rows);
            b.close();
        }
        for object in chat.departures() {
            b.open(Role::Item);
            b.style(look::CARD);
            b.key(super::inbox::card_key(*object));
            words.status(b, P::CardLeft, &[]);
            b.close();
        }
        if !chat.cards.is_empty() || !chat.departures().is_empty() {
            b.close();
        }
        if let Some(result) = &task.result {
            result_text(b, result, words);
        }
        for (key, _) in &chat.pending {
            if let Some(pending) = root.client().pending(*key) {
                crate::confirm::progress_text(b, words, pending.progress);
            }
        }
        if let Some(outcome) = &chat.outcome {
            super::chats::outcome_text(b, words, outcome, 0);
        }
        b.close();
        composer(b, root, chat, task, words, bindings, temper_client_domain::pages::shown(root.pages()));
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}
fn result_text(b: &mut Builder, result: &c::TaskResult, words: &Words) {
    match result {
        c::TaskResult::Document(result) => crate::cards::result(b, result, words),
        c::TaskResult::Ending(ending) => match ending.as_ref() {
            c::OwnerChipEnding::Done(value) => {
                crate::cards::value_nodes(b, value, words);
            }
            c::OwnerChipEnding::Failed(reason) => skein_ui::markdown::read(reason.bytes(), 2, b),
            c::OwnerChipEnding::Cancelled { reason, result, .. } => {
                skein_ui::markdown::read(reason.bytes(), 2, b);
                if let Some(value) = result {
                    crate::cards::value_nodes(b, value, words);
                }
            }
        },
    }
}
fn spend(b: &mut Builder, task: &c::Task, words: &Words) {
    text(
        b,
        Role::Paragraph,
        &words.say(
            Word::ChatSpend,
            &[Hole::Number(task.budget.numbers.spent), Hole::Number(task.budget.numbers.spent_below)],
        ),
    );
}
fn composer(
    b: &mut Builder,
    root: &Domain,
    chat: &Chat,
    task: &c::Task,
    words: &Words,
    bindings: &mut Bindings,
    shown: u64,
) {
    if temper_client_domain::pages::chat::allowed(task, c::ActKind::Message) {
        b.open(Role::Slot);
        b.style(look::COMPOSER_AREA);
        crate::choice::composer(b, root, Some(task), words, bindings);
        b.open(Role::Slot);
        b.style(look::ENTRY);
        b.open(Role::TextArea);
        b.name(&words.say(Word::Message, &[]));
        b.state(State::SubmitsOnEnter);
        b.style(look::COMPOSER);
        b.value(&chat.words, shown);
        b.bind(bindings.insert(Act::MessageWords { words: Box::default() }));
        b.close();
        b.open(Role::Paragraph);
        b.style(look::HINT);
        words.text(b, Word::ChatSendHint, &[]);
        b.close();
        control(b, bindings, Role::Button, &words.say(Word::Send, &[]), Act::Send);
        b.close();
        b.close();
    } else {
        crate::choice::composer(b, root, Some(task), words, bindings);
    }
}
pub(crate) fn conversation(
    b: &mut Builder,
    thread: &Thread,
    words: &Words,
    limits: (u32, u32),
    person: (Option<u64>, bool),
    delegations: Option<(&Chat, &jig_client::Client)>,
    bindings: &mut Bindings,
) {
    let (rows, text_limit) = limits;
    let bound = match delegations {
        Some((chat, _)) => chat.delegations.capacity(),
        None => 0,
    };
    let mut tasks = List::with_capacity(bound);
    let mut shown = List::with_capacity(bound);
    b.open(Role::Log);
    b.name(&words.say_card(P::ConversationLog, &[]));
    for line in thread.lines().into_boxed() {
        if let Line::Activity { line } = &line
            && hidden(&line.event, delegations)
        {
            continue;
        }
        b.open(Role::Region);
        b.style(look::TURN);
        match line {
            Line::Words { by, words: message, state } => {
                person_words(b, by, message, state, person.0, words);
            }
            Line::Turn { turn, .. } => {
                committed(
                    b,
                    turn,
                    words,
                    (rows, text_limit, person.1),
                    delegations,
                    (&mut tasks, &mut shown),
                    bindings,
                );
            }
            Line::Provisional { text, cut, .. } => {
                b.open(Role::Slot);
                b.style(look::MESSAGE);
                b.style(look::BY_AGENT);
                b.open(Role::Slot);
                b.style(look::WHO);
                words.status(b, P::Provisional, &[]);
                b.close();
                b.open(Role::Slot);
                b.style(look::SAYS);
                markdown(b, text, words, text_limit);
                if cut {
                    words.status(b, P::TurnCut, &[]);
                }
                b.close();
                b.close();
            }
            Line::Undecoded { why, .. } => match why {
                c::TurnProblem::Version { found } => {
                    words.status(b, P::TurnUnreadVersion, &[Hole::Number(u64::from(found))]);
                }
                c::TurnProblem::Malformed => words.status(b, P::TurnUnreadMalformed, &[]),
                c::TurnProblem::TooLarge { .. } => words.status(b, P::TurnUnreadTooLarge, &[]),
            },
            Line::RunBegan { attempt } => {
                b.style(look::MARKER);
                text(b, Role::Paragraph, &words.say(Word::RunBegan, &[Hole::Number(attempt)]));
            }
            Line::RunEnded { attempt } => {
                b.style(look::MARKER);
                text(b, Role::Paragraph, &words.say(Word::RunEnded, &[Hole::Number(attempt)]));
            }
            Line::Left { lines } => {
                b.style(look::MARKER);
                text(b, Role::Status, &words.say(Word::LinesLeft, &[Hole::Number(u64::from(lines))]));
            }
            Line::Activity { line } => match &line.event {
                c::Activity::Owner { source, .. } => super::task::owner_activity(b, &source.line, words),
                c::Activity::Result { result } => {
                    crate::cards::result(b, result, words);
                }
                c::Activity::News { summary, .. } => text(b, Role::Paragraph, summary.bytes()),
                c::Activity::Decision { .. } => {
                    words.text(b, Word::DecisionMade, &[]);
                }
                c::Activity::Words { .. }
                | c::Activity::RunBegan { .. }
                | c::Activity::RunEnded { .. }
                | c::Activity::Turn { .. } => {}
            },
        }
        b.close();
    }
    // The live delegates anchor follows the window's last line (client/README.md, 5.7).
    live_delegates(b, &mut tasks, delegations, &mut shown, words, bindings);
    b.close();
}
fn live_delegates(
    b: &mut Builder,
    tasks: &mut List<u64>,
    delegations: Option<(&Chat, &jig_client::Client)>,
    shown: &mut List<u64>,
    words: &Words,
    bindings: &mut Bindings,
) {
    tasks.clear();
    if let Some((chat, client)) = delegations
        && let Some(document) = chat.task(client)
    {
        for delegate in &document.delegates {
            let task = delegate.chip.task;
            if !shown.as_slice().contains(&task) && chat.delegation(client, task).is_some() && tasks.push(task).is_err()
            {
                break;
            }
        }
    }
    if !tasks.is_empty() {
        b.open(Role::Region);
        b.style(look::TURN);
        cards(b, tasks, delegations, shown, words, bindings);
        b.close();
    }
}
fn person_words(
    b: &mut Builder,
    by: Option<&c::Party>,
    message: &[u8],
    state: WordState,
    person: Option<u64>,
    words: &Words,
) {
    b.open(Role::Slot);
    b.style(look::MESSAGE);
    b.style(look::BY_PERSON);
    b.open(Role::Slot);
    b.style(look::WHO);
    match by {
        Some(by) if Some(by.number) != person => text(b, Role::Paragraph, by.name.bytes()),
        Some(_) | None => text(b, Role::Paragraph, &words.say_card(P::You, &[])),
    }
    words.status(
        b,
        match state {
            WordState::BeingSent => P::WordsBeingSent,
            WordState::Sent { .. } => P::WordsSent,
            WordState::Read { .. } => P::WordsRead,
        },
        &[],
    );
    b.close();
    b.open(Role::Slot);
    b.style(look::SAYS);
    text(b, Role::Paragraph, message);
    b.close();
    b.close();
}

fn committed(
    b: &mut Builder,
    turn: &c::Turn,
    words: &Words,
    limits: (u32, u32, bool),
    delegations: Option<(&Chat, &jig_client::Client)>,
    lists: (&mut List<u64>, &mut List<u64>),
    bindings: &mut Bindings,
) {
    let (rows, text_limit, requester) = limits;
    let (tasks, shown) = lists;
    for message in prefix(b, turn.messages.as_slice(), rows, words) {
        b.open(Role::Slot);
        b.style(look::MESSAGE);
        b.style(match message.speaker {
            c::Speaker::Person => look::BY_PERSON,
            c::Speaker::Assistant => look::BY_AGENT,
            c::Speaker::Tool => look::BY_TOOL,
            c::Speaker::System => look::BY_SYSTEM,
        });
        b.open(Role::Slot);
        b.style(look::WHO);
        words.card_text(
            b,
            match message.speaker {
                c::Speaker::Person => {
                    if requester {
                        P::You
                    } else {
                        P::Person
                    }
                }
                c::Speaker::Assistant => P::Temper,
                c::Speaker::Tool => P::CallCompleted,
                c::Speaker::System => P::TurnInstructions,
            },
            &[],
        );
        b.close();
        b.open(Role::Slot);
        b.style(look::SAYS);
        for block in prefix(b, message.blocks.as_slice(), rows, words) {
            match block {
                c::Block::Text(value) => markdown(b, value.bytes(), words, text_limit),
                c::Block::Call(call) => {
                    tasks.clear();
                    delegation::delegated(call, tasks);
                    cards(b, tasks, delegations, shown, words, bindings);
                    if carded(tasks, delegations) {
                        continue;
                    }
                    b.open(Role::Slot);
                    b.style(look::WORK);
                    text(b, Role::Paragraph, call.tool.bytes());
                    if let Some(summary) = &call.outcome {
                        text(b, Role::Paragraph, summary.bytes());
                    }
                    b.close();
                }
                c::Block::Result { call, summary, failed } => {
                    tasks.clear();
                    for message in &turn.messages {
                        for block in &message.blocks {
                            if let c::Block::Call(source) = block
                                && source.number == *call
                            {
                                delegation::delegated(source, tasks);
                            }
                        }
                    }
                    if carded(tasks, delegations) {
                        continue;
                    }
                    tasks.clear();
                    delegation::results(summary.bytes(), tasks);
                    if requested(tasks, delegations) {
                        cards(b, tasks, delegations, shown, words, bindings);
                        continue;
                    }
                    b.open(Role::Slot);
                    b.style(look::WORK);
                    words.card_text(b, if *failed { P::CallFailed } else { P::CallCompleted }, &[]);
                    text(b, Role::Paragraph, summary.bytes());
                    b.close();
                }
                c::Block::Opaque { dialect, size } => {
                    words.card_text(b, P::OpaqueDialect, &[crate::cards::name(dialect.bytes())]);
                    words.card_text(b, P::OpaqueBytes, &[Hole::Number(*size)]);
                }
            }
        }
        b.close();
        b.close();
    }
    if turn.cut {
        words.status(b, P::TurnCut, &[]);
    }
}

fn cards(
    b: &mut Builder,
    tasks: &List<u64>,
    delegations: Option<(&Chat, &jig_client::Client)>,
    shown: &mut List<u64>,
    words: &Words,
    bindings: &mut Bindings,
) {
    let Some((chat, client)) = delegations else { return };
    for task in tasks {
        if !shown.as_slice().contains(task)
            && let Some(document) = chat.delegation(client, *task)
        {
            crate::cards::delegation::card(b, document, words, bindings);
            shown.push(*task).expect("shown delegations fit their bound");
        }
    }
}
fn requested(tasks: &List<u64>, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    let Some((chat, client)) = delegations else { return false };
    if tasks.is_empty() {
        return false;
    }
    for task in tasks {
        if chat.requested(client, *task).is_none() {
            return false;
        }
    }
    true
}
fn carded(tasks: &List<u64>, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    let Some((chat, client)) = delegations else { return false };
    if tasks.is_empty() {
        return false;
    }
    for task in tasks {
        if chat.delegation(client, *task).is_none() {
            return false;
        }
    }
    true
}
fn hidden(event: &c::Activity, delegations: Option<(&Chat, &jig_client::Client)>) -> bool {
    let Some((chat, client)) = delegations else { return false };
    let task = match event {
        c::Activity::Owner { source, .. } => {
            if let c::OwnerActivity::Heard(c::OwnerWord {
                from: c::OwnerRequester::Task(task),
                kind: c::OwnerMessageKind::Result(_),
                ..
            }) = &source.line
            {
                Some(*task)
            } else {
                None
            }
        }
        c::Activity::Result { result } => Some(result.task.task),
        c::Activity::Words { .. }
        | c::Activity::RunBegan { .. }
        | c::Activity::RunEnded { .. }
        | c::Activity::Turn { .. }
        | c::Activity::News { .. }
        | c::Activity::Decision { .. } => None,
    };
    match task {
        Some(task) => chat.delegation(client, task).is_some(),
        None => false,
    }
}
fn prefix<'a, T>(b: &mut Builder, values: &'a [T], most: u32, words: &Words) -> &'a [T] {
    let end = values.len().min(usize::try_from(most).expect("window fits"));
    if end < values.len() {
        words.status(b, P::TurnCut, &[]);
    }
    values.get(..end).expect("bounded prefix")
}
fn markdown(b: &mut Builder, bytes: &[u8], words: &Words, most: u32) {
    let most = bytes.len().min(usize::try_from(most).expect("text bound fits"));
    let mut end = most;
    if end < bytes.len() {
        // The protocol validated UTF-8; align a display window to the preceding character.
        for index in most.saturating_sub(3)..=most {
            if let Some(byte) = bytes.get(index)
                && byte & 0xc0 != 0x80
            {
                end = index;
            }
        }
    }
    skein_ui::markdown::read(bytes.get(..end).expect("UTF-8 prefix"), 2, b);
    if end < bytes.len() {
        words.status(b, P::TurnCut, &[]);
    }
}
