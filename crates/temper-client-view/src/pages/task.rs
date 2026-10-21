//! A task's header, specification, supplied subtree and latest committed activity.
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use jig_client::{Latest, contract as c};
use skein_lib::{List, bytes::copy_of};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, task::Shape},
};
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn region(b: &mut Builder, word: Word, words: &Words) {
    b.open(Role::Region);
    b.name(&words.say(word, &[]));
    text(b, Role::Heading { level: 2 }, &words.say(word, &[]));
}
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, rows: u32) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::PAGE);
    if let Some(page) = root.pages().task()
        && let Some(task) = page.document(root.client())
    {
        b.open(Role::Slot);
        b.style(look::PAGEBAR);
        if let c::OwnerRequester::Task(number) = task.requester {
            task_link(b, number, words, bindings);
        }
        text(b, Role::Paragraph, &words.say(Word::Task, &[Hole::Number(task.chip.task)]));
        b.close();
        header(b, task, root.pages().me(root.client()), words);
        crate::choice::composer(b, root, Some(task), words, bindings);
        if page.shape == Shape::Tracked {
            asked(b, task, words);
        }
        if let Some(document) = root.client().document(page.tree)
            && let Some(Latest::Jig(tree)) = document.latest
        {
            plan(b, tree, root, words, bindings, rows);
        }
        if page.shape == Shape::Tracked {
            latest(b, page.latest.as_deref(), words);
        }
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}
pub(crate) fn header(b: &mut Builder, task: &c::Task, me: Option<&c::Me>, words: &Words) {
    region(b, Word::TaskHeader, words);
    b.style(look::TASK_HEADING);
    b.open(Role::Slot);
    b.style(look::META);
    if let Some(priority) = task.priority {
        text(b, Role::Paragraph, &words.say(Word::Priority, &[Hole::Number(u64::from(priority))]));
    }
    b.open(Role::Slot);
    text(b, Role::Paragraph, &words.say(Word::Task, &[Hole::Number(task.chip.task)]));
    executor(b, task.chip.executor, words);
    b.close();
    let mut named = false;
    if let Some(me) = me {
        for project in &me.projects {
            if project.project == task.project
                && let Some(name) = &project.name
            {
                text(b, Role::Paragraph, name.bytes());
                named = true;
            }
        }
    }
    if !named {
        text(b, Role::Paragraph, &words.say(Word::Project, &[Hole::Number(u64::from(task.project))]));
    }
    if let c::ChipPhase::Owner { details, .. } = task.chip.details.as_ref()
        && let Some(part) = &details.procedure
    {
        crate::forge::owner_part(b, part, words);
    }
    b.close();
    text(b, Role::Heading { level: 1 }, task.chip.title.bytes());
    b.open(Role::Slot);
    b.style(look::META);
    phase(b, task.chip.phase, words);
    requester(b, task.requester, me, words);
    if let Some(deadline) = task.budget.deadline {
        text(b, Role::Paragraph, &words.deadline(deadline));
    }
    b.close();
    b.close();
}

fn executor(b: &mut Builder, executor: c::Executor, words: &Words) {
    let word = match executor {
        c::Executor::Agent { .. } => Word::Agent,
        c::Executor::Procedure { .. } => Word::Procedure,
        c::Executor::Party { .. } | c::Executor::Role { .. } => Word::PersonTask,
    };
    text(b, Role::Paragraph, &words.say(word, &[]));
}
fn requester(b: &mut Builder, by: c::OwnerRequester, me: Option<&c::Me>, words: &Words) {
    let (word, number) = match by {
        c::OwnerRequester::Person(number) => {
            if let Some(me) = me
                && me.party.number == number
            {
                text(b, Role::Paragraph, &words.say(Word::AskedBy, &[Hole::Name(copy_of(me.party.name.bytes()))]));
                return;
            }
            (Word::AskedByPerson, number)
        }
        c::OwnerRequester::Task(number) => (Word::AskedByTask, number),
        c::OwnerRequester::Deployment { project } => (Word::AskedByDeployment, u64::from(project)),
    };
    text(b, Role::Paragraph, &words.say(word, &[Hole::Number(number)]));
}
fn phase(b: &mut Builder, phase: c::Phase, words: &Words) {
    b.open(Role::Status);
    b.style(phase_token(phase));
    let name = words.say_word(jig_client::words::Word::Phase(phase), &[]);
    b.name(&name);
    b.text(Run::Plain, &name);
    b.close();
}
fn phase_token(phase: c::Phase) -> skein_ui::Style {
    match phase {
        c::Phase::Due | c::Phase::Preparing | c::Phase::Claimed | c::Phase::Running | c::Phase::Active => {
            look::STATUS_ACTIVE
        }
        c::Phase::Done => look::STATUS_DONE,
        c::Phase::Held | c::Phase::Failed => look::STATUS_HELD,
        c::Phase::BackingOff => look::STATUS_RETRYING,
        c::Phase::Waiting | c::Phase::Idle | c::Phase::Closing | c::Phase::Ended | c::Phase::Cancelled => {
            look::STATUS_QUIET
        }
    }
}
pub(crate) fn asked(b: &mut Builder, task: &c::Task, words: &Words) {
    region(b, Word::Asked, words);
    b.style(look::TASK_CARD);
    skein_ui::markdown::read(task.spec.specification.words.bytes(), 3, b);
    let parts = !task.resources.is_empty()
        || match &task.context {
            Some(context) => !context.parts.is_empty(),
            None => false,
        };
    if parts {
        b.open(Role::Slot);
        b.style(look::META);
    }
    for resource in &task.resources {
        match resource {
            c::TaskResource::Part(part) => crate::forge::drawn(b, part, words),
            c::TaskResource::Owner(resource) => crate::forge::owner_part(b, &resource.part, words),
        }
    }
    if let Some(context) = &task.context {
        for part in &context.parts {
            crate::forge::drawn(b, part, words);
        }
    }
    if parts {
        b.close();
    }
    b.close();
}
fn chip(b: &mut Builder, chip: &c::Chip, words: &Words, bindings: &mut Bindings) {
    b.open(Role::TreeItem);
    b.name(&words.say(Word::Task, &[Hole::Number(chip.task)]));
    b.open(Role::Slot);
    b.style(look::PLAN_ROW);
    b.open(Role::Slot);
    task_link(b, chip.task, words, bindings);
    text(b, Role::Paragraph, chip.title.bytes());
    b.close();
    executor(b, chip.executor, words);
    phase(b, chip.phase, words);
    if let c::ChipPhase::Owner { details, .. } = chip.details.as_ref()
        && let Some(part) = &details.procedure
    {
        crate::forge::owner_part(b, part, words);
    }
    b.close();
}
fn task_link(b: &mut Builder, task: u64, words: &Words, bindings: &mut Bindings) {
    let name = words.say(Word::Task, &[Hole::Number(task)]);
    b.open(Role::Link);
    b.name(&name);
    b.text(Run::Plain, &name);
    b.bind(bindings.insert(Act::Go { address: Address::Task { task } }));
    b.close();
}
fn parent(b: &mut Builder, stack: &mut List<u64>, parent: Option<u64>) {
    let mut keep = 0;
    for (index, number) in stack.as_slice().iter().enumerate() {
        if Some(*number) == parent {
            keep = index.checked_add(1).expect("tree stack fits");
        }
    }
    for _ in keep..stack.as_slice().len() {
        b.close();
    }
    stack.truncate(u32::try_from(keep).expect("tree stack fits"));
}
fn folded(b: &mut Builder, counts: &[(c::Phase, u32)], held_below: u32, words: &Words) {
    for (phase, count) in counts {
        if *count != 0 {
            text(
                b,
                Role::Paragraph,
                &words.say(
                    Word::Folded,
                    &[
                        Hole::Number(u64::from(*count)),
                        Hole::Name(words.say_word(jig_client::words::Word::Phase(*phase), &[])),
                    ],
                ),
            );
        }
    }
    if held_below != 0 {
        text(b, Role::Status, &words.say(Word::HeldBelow, &[Hole::Number(u64::from(held_below))]));
    }
}
pub(crate) fn plan(
    b: &mut Builder,
    tree: &c::Delivery,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
) {
    region(b, Word::Plan, words);
    b.style(look::TASK_CARD);
    b.open(Role::Tree);
    b.style(look::PLAN);
    b.name(&words.say(Word::Plan, &[]));
    match tree {
        c::Delivery::OwnerTree(tree) => {
            let mut stack = List::with_capacity(tree.rows.len().checked_add(1).expect("tree stack fits"));
            chip(b, &tree.root, words, bindings);
            controls(b, &tree.root, root, words, bindings, rows);
            stack.push(tree.root.task).expect("root fits");
            owner_folds(b, tree, tree.root.task, words);
            for row in &tree.rows {
                parent(b, &mut stack, Some(row.parent));
                chip(b, &row.chip, words, bindings);
                controls(b, &row.chip, root, words, bindings, rows);
                stack.push(row.chip.task).expect("tree stack fits");
                owner_folds(b, tree, row.chip.task, words);
            }
            parent(b, &mut stack, None);
            folded(b, &[], tree.held_below, words);
        }
        c::Delivery::Tree(tree) => {
            let mut stack = List::with_capacity(tree.nodes.len());
            for node in &tree.nodes {
                parent(b, &mut stack, node.parent);
                chip(b, &node.chip, words, bindings);
                controls(b, &node.chip, root, words, bindings, rows);
                stack.push(node.chip.task).expect("tree stack fits");
                counts(b, node.folded, node.held_below, words);
            }
            parent(b, &mut stack, None);
            counts(b, tree.folded, tree.held_below, words);
        }
        c::Delivery::OwnerProject(_)
        | c::Delivery::Me(_)
        | c::Delivery::Task(_)
        | c::Delivery::OwnerProposal(_)
        | c::Delivery::Chips(_)
        | c::Delivery::Inbox(_)
        | c::Delivery::OwnerInbox(_)
        | c::Delivery::Tasks(_)
        | c::Delivery::Run(_)
        | c::Delivery::RunState(_)
        | c::Delivery::Deployment(_)
        | c::Delivery::OwnerDeployment(_)
        | c::Delivery::OwnerAccounts(_) => {}
    }
    b.close();
    b.close();
}
fn controls(b: &mut Builder, chip: &c::Chip, root: &Domain, words: &Words, bindings: &mut Bindings, rows: u32) {
    if let Some(page) = root.pages().task() {
        for card in &page.cards {
            if card.object == c::ObjectKey::Task(chip.task) && (chip.phase == c::Phase::Held || card.key.is_some()) {
                b.open(Role::List);
                b.style(look::PLAN_HOLD);
                b.open(Role::Item);
                b.key(super::inbox::card_key(c::ObjectKey::Escalation(chip.task)));
                crate::confirm::card(b, root, card, words, bindings, rows);
                b.close();
                b.close();
            }
        }
    }
}
fn owner_folds(b: &mut Builder, tree: &c::OwnerTree, parent: u64, words: &Words) {
    use c::Phase as P;
    for fold in &tree.folded {
        if fold.parent == parent {
            let f = fold.phases;
            folded(
                b,
                &[
                    (P::Waiting, f.waiting),
                    (P::Idle, f.idle),
                    (P::Due, f.starting),
                    (P::Running, f.running),
                    (P::BackingOff, f.backing_off),
                    (P::Closing, f.closing),
                    (P::Held, f.held),
                    (P::Done, f.done),
                    (P::Failed, f.failed),
                    (P::Cancelled, f.cancelled),
                ],
                fold.held_below,
                words,
            );
        }
    }
}
fn counts(b: &mut Builder, f: c::PhaseCounts, held: u32, words: &Words) {
    use c::Phase as P;
    folded(
        b,
        &[
            (P::Ended, f.ended),
            (P::Waiting, f.waiting),
            (P::Running, f.running),
            (P::Held, f.held),
            (P::Closing, f.closing),
            (P::Done, f.done),
            (P::Failed, f.failed),
            (P::Cancelled, f.cancelled),
        ],
        held,
        words,
    );
}
pub(crate) fn latest(b: &mut Builder, page: Option<&c::ActivityPage>, words: &Words) {
    region(b, Word::Latest, words);
    b.style(look::TASK_CARD);
    if let Some(page) = page
        && !page.rows.is_empty()
    {
        b.open(Role::List);
        b.style(look::EVENTS);
        for row in &page.rows {
            b.open(Role::Item);
            text(b, Role::Paragraph, &words.say(Word::ActivityAt, &[Hole::Wall(row.at)]));
            b.open(Role::Slot);
            activity(b, &row.event, words);
            b.close();
            b.close();
        }
        b.close();
    }
    b.close();
}
fn activity(b: &mut Builder, event: &c::Activity, words: &Words) {
    if let Some((_, text)) = event.word_message() {
        text_node(b, text.bytes());
        return;
    }
    if let Some((_, turn)) = event.decoded_turn() {
        for message in &turn.messages {
            for block in &message.blocks {
                match block {
                    c::Block::Text(text) => text_node(b, text.bytes()),
                    c::Block::Call(call) => {
                        text_node(b, call.tool.bytes());
                        if let Some(text) = &call.outcome {
                            text_node(b, text.bytes());
                        }
                    }
                    c::Block::Result { summary, .. } => text_node(b, summary.bytes()),
                    c::Block::Opaque { size, .. } => {
                        text(b, Role::Paragraph, &words.say(Word::Opaque, &[Hole::Number(*size)]));
                    }
                }
            }
        }
        return;
    }
    match event {
        c::Activity::RunBegan { attempt, .. } => {
            text(b, Role::Paragraph, &words.say(Word::RunBegan, &[Hole::Number(*attempt)]));
        }
        c::Activity::RunEnded { attempt, .. } => {
            text(b, Role::Paragraph, &words.say(Word::RunEnded, &[Hole::Number(*attempt)]));
        }
        c::Activity::Owner { source, .. } => owner_activity(b, &source.line, words),
        c::Activity::News { summary, part, .. } => {
            text_node(b, summary.bytes());
            if let Some(part) = part {
                crate::forge::drawn(b, part, words);
            }
        }
        c::Activity::Result { result } => text_node(b, result.task.title.bytes()),
        c::Activity::Decision { .. } => words.text(b, Word::DecisionMade, &[]),
        c::Activity::Turn { .. } => words.text(b, Word::UnreadTurn, &[]),
        c::Activity::Words { .. } => {}
    }
}
fn text_node(b: &mut Builder, value: &[u8]) {
    skein_ui::markdown::read(value, 3, b);
}
pub(crate) fn owner_activity(b: &mut Builder, event: &c::OwnerActivity, words: &Words) {
    match event {
        c::OwnerActivity::Words { words, .. } => text_node(b, words.bytes()),
        c::OwnerActivity::Heard(word) | c::OwnerActivity::Notified { word, .. } => text_node(b, word.words.bytes()),
        c::OwnerActivity::RunBegan { attempt, .. } => words.text(b, Word::RunBegan, &[Hole::Number(*attempt)]),
        c::OwnerActivity::RunEnded { attempt, .. } => words.text(b, Word::RunEnded, &[Hole::Number(*attempt)]),
        c::OwnerActivity::Decided { .. } => words.text(b, Word::DecisionMade, &[]),
        c::OwnerActivity::Turn { .. } => words.text(b, Word::UnreadTurn, &[]),
        c::OwnerActivity::Milestone { position } => words.text(b, Word::Milestone, &[Hole::Number(*position)]),
    }
}
