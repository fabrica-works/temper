//! An agent's task's lineage, brief, run and conversation (client/README.md, 5.9).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use jig_client::{Latest, contract as c};
use skein_lib::bytes::copy_of;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{Domain, pages::task};

fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, rows: u32) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::CHAT);
    if let Some(page) = root.pages().task()
        && let Some(task) = page.document(root.client())
    {
        let me = root.pages().me(root.client());
        lineage(b, task, words, bindings);
        b.open(Role::Slot);
        b.style(look::CHATBAR);
        b.open(Role::Slot);
        text(
            b,
            Role::Heading { level: 1 },
            &words.say(Word::ChatTitle, &[Hole::Number(task.chip.task), Hole::Name(copy_of(task.chip.title.bytes()))]),
        );
        super::task::state(b, task::header_state(task, me), words);
        b.close();
        b.open(Role::Slot);
        super::task::executor(b, task.chip.executor, words);
        text(b, Role::Paragraph, &words.say(Word::RowSpent, &[Hole::Number(task.budget.numbers.spent)]));
        b.close();
        b.close();
        b.open(Role::Slot);
        b.style(look::THREAD);
        b.open(Role::Region);
        b.style(look::AGENT_BRIEF);
        b.name(&words.say(Word::TaskBrief, &[]));
        text(b, Role::Heading { level: 2 }, &words.say(Word::TaskBrief, &[]));
        skein_ui::markdown::read(task.spec.specification.words.bytes(), 3, b);
        super::task::requester(b, task.requester, me, words);
        super::task::dependencies(b, task.chip.task, &task.dependencies, false, words, bindings);
        b.close();
        if let Some(run) = task::run(task) {
            run_heading(b, run, words);
        }
        super::chat::conversation(
            b,
            &page.thread,
            words,
            (rows, page.thread.limits.text),
            super::chat::speakers(task, me),
            None,
            bindings,
        );
        super::task::result(b, task, words, look::AGENT_RESULT);
        if let Some(document) = root.client().document(page.tree)
            && let Some(Latest::Jig(tree)) = document.latest
        {
            let delegates = match tree {
                c::Delivery::OwnerTree(tree) => !tree.rows.is_empty(),
                c::Delivery::Tree(tree) => tree.nodes.len() > 1,
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
                | c::Delivery::OwnerAccounts(_) => false,
            };
            if delegates || !page.cards.is_empty() {
                super::task::plan(b, tree, root, words, bindings, rows);
            }
        }
        b.close();
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}
fn lineage(b: &mut Builder, task: &c::Task, words: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::AGENT_LINEAGE);
    if let Some(parent) = task.lineage_chips.as_slice().last() {
        b.open(Role::Slot);
        super::task::task_link(b, parent.task, words, bindings);
        text(b, Role::Paragraph, parent.title.bytes());
        b.close();
    } else if let c::OwnerRequester::Task(parent) = task.requester {
        b.open(Role::Slot);
        super::task::task_link(b, parent, words, bindings);
        b.close();
    }
    b.open(Role::Slot);
    text(b, Role::Paragraph, &words.say(Word::Task, &[Hole::Number(task.chip.task)]));
    text(b, Role::Paragraph, task.chip.title.bytes());
    b.close();
    b.close();
}
fn run_heading(b: &mut Builder, run: task::Run, words: &Words) {
    let (word, attempt, since) = match run {
        task::Run::Live { attempt, since } => (Word::RunLive, Some(attempt), since),
        task::Run::Waiting { attempt, since } => (Word::RunWaiting, Some(attempt), since),
        task::Run::Ended { attempt } => (Word::RunOver, Some(attempt), None),
        task::Run::NoRun => (Word::NoRun, None, None),
    };
    let heading = match attempt {
        Some(attempt) => words.say(word, &[Hole::Number(attempt)]),
        None => words.say(word, &[]),
    };
    b.open(Role::Slot);
    b.style(look::AGENT_RUN);
    text(b, Role::Heading { level: 2 }, &heading);
    if let Some(at) = since {
        text(b, Role::Paragraph, &words.run_started(at));
    }
    b.close();
}
