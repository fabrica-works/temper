//! The page's deciding controls, their supplied facts, and its one confirmation.
use crate::cards::P;
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{
    Answered, Outcome,
    contract::{ActKind, Admission, Answer, Effect, ReadValue, ReleaseEffect},
};
use skein_ui::{Builder, Role, Run};
use temper_client_domain::{
    Domain, acts,
    pages::{Act, inbox::Card},
};
const ACTIONS: [(ActKind, Word); 7] = [
    (ActKind::Accept, Word::Accept),
    (ActKind::Reject, Word::Reject),
    (ActKind::Answer, Word::Answer),
    (ActKind::Release, Word::Release),
    (ActKind::LeaveHeld, Word::LeaveHeld),
    (ActKind::Take, Word::Take),
    (ActKind::HandBack, Word::HandBack),
];
fn button(b: &mut Builder, bindings: &mut Bindings, words: &Words, word: Word, act: Act) {
    b.open(Role::Button);
    b.name(&words.say(word, &[]));
    b.bind(bindings.insert(act));
    b.close();
}
pub(crate) fn card(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings, rows: u32) {
    let Some(page) = &card.page else { return };
    crate::cards::proposal_reason(b, page);
    facts(b, page, words, rows);
    controls(b, root, card, words, bindings);
}
pub(crate) fn facts(b: &mut Builder, page: &ReadValue, words: &Words, rows: u32) {
    crate::cards::proposal_facts(b, page, words, rows);
    crate::cards::decision(b, words, Some(page), ActKind::Accept);
}
pub(crate) fn controls(b: &mut Builder, root: &Domain, card: &Card, words: &Words, bindings: &mut Bindings) {
    let Some(page) = &card.page else { return };
    if let Some(outcome) = &card.outcome {
        match outcome {
            Outcome::Answered(Answered::Jig(Answer::Done(outcome))) => words.outcome(b, outcome),
            Outcome::Answered(Answered::Jig(Answer::Refused { refusal, .. })) => {
                words.refusal(b, refusal, 0);
                if let jig_client::contract::Refusal::Moved { .. } = refusal {
                    words.card_text(b, P::LookAgain, &[]);
                }
            }
            Outcome::NotKept | Outcome::Held => words.status(b, P::RequestNotKept, &[]),
            Outcome::Unknown => words.status(b, P::Unknown, &[]),
            Outcome::Limit => words.status(b, P::RequestsInFlight, &[]),
            Outcome::Unconfirmed => words.status(b, P::RequestUnconfirmed, &[]),
            Outcome::Answered(Answered::Family(_)) => {}
            Outcome::Unsent | Outcome::Lost => unreachable!("temper's requests carry no value"),
        }
        return;
    }
    if let Some(progress) = acts::pending(card, root.client()) {
        progress_text(b, words, progress);
        return;
    }
    if card.key.is_some() {
        return;
    }
    for (act, word) in ACTIONS {
        if root.pages().task().is_some() && act != ActKind::Release {
            continue;
        }
        let Some(admission) = jig_client::facts::offer(acts::offers(page), act) else { continue };
        match admission {
            Admission::Allowed | Admission::Bounded { .. } => {
                if act == ActKind::Answer || act == ActKind::Reject || act == ActKind::LeaveHeld {
                    b.open(Role::TextArea);
                    b.name(&words.say(if act == ActKind::Answer { Word::AnswerWords } else { Word::Reason }, &[]));
                    b.value(
                        match &card.words {
                            Some(words) => words.bytes(),
                            None => b"",
                        },
                        temper_client_domain::pages::shown(root.pages()),
                    );
                    b.bind(bindings.insert(Act::CardWords { object: card.object, words: Box::default() }));
                    b.close();
                }
                button(
                    b,
                    bindings,
                    words,
                    word,
                    Act::Decide { object: card.object, revision: acts::revision(page), act },
                );
            }
            Admission::Proposed { to } => {
                words.card_text(b, P::ProposeInstead, &[]);
                crate::cards::recipient(b, words, to);
            }
            Admission::Refused(refusal) => words.refusal(b, refusal, 0),
        }
    }
}
pub(crate) fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, rows: u32) {
    let Some(open) = root.pages().confirmation() else { return };
    if open.act == ActKind::Cancel {
        close(b, root, words, bindings);
        return;
    }
    let Some(cards) = root.pages().cards() else { return };
    for card in cards {
        if card.object != open.object {
            continue;
        }
        let Some(page) = &card.page else { return };
        let mut word = None;
        for (act, name) in ACTIONS {
            if act == open.act {
                word = Some(name);
            }
        }
        let Some(word) = word else { return };
        b.open(Role::Dialog);
        b.name(&words.say(word, &[]));
        b.text(Run::Plain, &words.say(word, &[]));
        crate::cards::proposal(b, page, words, rows);
        crate::cards::decision(b, words, Some(page), open.act);
        if let Some(reason) = &card.words {
            b.open(Role::Paragraph);
            b.text(Run::Plain, reason.bytes());
            b.close();
        }
        if let Some(Effect::Release(effect)) = jig_client::facts::effect(acts::offers(page), open.act)
            && *effect == ReleaseEffect::Uncertain
        {
            button(b, bindings, words, Word::Again, Act::Again { again: true });
            button(b, bindings, words, Word::DoNotRepeat, Act::Again { again: false });
        }
        button(b, bindings, words, Word::Confirm, Act::Confirm);
        button(b, bindings, words, Word::Cancel, Act::Cancel);
        b.close();
        return;
    }
}

pub(crate) fn progress_text(b: &mut Builder, words: &Words, progress: jig_client::Progress) {
    crate::cards::progress(b, words, progress);
}

fn close(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings) {
    use jig_client::contract::Ask;
    let Some(chat) = root.pages().chat() else { return };
    let Some(Ask::Compound { operations, .. }) = &chat.closing else { return };
    b.open(Role::Dialog);
    b.name(&words.say(Word::Close, &[]));
    if let Some(task) = chat.task(root.client())
        && let Some(Effect::Cancel(_)) = jig_client::facts::effect(task.acts.as_slice(), ActKind::Cancel)
    {
        crate::cards::decision(b, words, Some(&ReadValue::Task(Box::new(task.clone()))), ActKind::Cancel);
    }
    close_choices(b, chat.task, operations, words, bindings);
    b.close();
}
pub(crate) fn close_choices(
    b: &mut Builder,
    chat: u64,
    operations: &skein_lib::List<jig_client::contract::Operation>,
    words: &Words,
    bindings: &mut Bindings,
) {
    use jig_client::contract::Operation;
    use skein_ui::phrases::Hole;
    for operation in operations {
        match operation {
            Operation::Cancel { task, grace, .. } if *task == chat => match grace {
                Some(grace) if grace.as_nanos() % 1_000_000_000 == 0 => {
                    words.text(b, Word::CloseGrace, &[Hole::Span(*grace)]);
                }
                Some(grace) => words.text(b, Word::CloseGraceNanos, &[Hole::Number(grace.as_nanos())]),
                None => words.text(b, Word::CloseNow, &[]),
            },
            Operation::Cancel { task, .. } => delegate(b, *task, Word::DelegateCancel, words, bindings),
            Operation::Move { task, .. } => delegate(b, *task, Word::DelegateMove, words, bindings),
            Operation::Message { .. }
            | Operation::Amend { .. }
            | Operation::Release { .. }
            | Operation::Stop { .. } => {}
        }
    }
    button(b, bindings, words, Word::Confirm, Act::Confirm);
    button(b, bindings, words, Word::Cancel, Act::Cancel);
}

fn delegate(b: &mut Builder, task: u64, word: Word, words: &Words, bindings: &mut Bindings) {
    use jig_client::shapes::Fate;
    use skein_ui::phrases::Hole;
    words.text(b, word, &[Hole::Number(task)]);
    for (word, fate) in [(Word::CancelDelegate, Fate::Cancel), (Word::TakeDelegate, Fate::Move)] {
        b.open(Role::Button);
        b.name(&words.say(word, &[Hole::Number(task)]));
        b.bind(bindings.insert(Act::DelegateFate { task, fate }));
        b.close();
    }
}
