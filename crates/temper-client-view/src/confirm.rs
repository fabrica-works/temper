//! The page's deciding controls, their supplied facts, and its one confirmation.
use crate::{
    frame::Bindings,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{
    Answered, Outcome,
    contract::{ActKind, Admission, Answer, Effect, ReadValue, ReleaseEffect},
};
use jig_client_view::{JigBindings, phrases::jig::JigPhrase as P};
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
fn facts(
    b: &mut Builder,
    root: &Domain,
    page: &ReadValue,
    words: &jig_client_view::words::Words,
    jig: &mut JigBindings,
) {
    match page {
        ReadValue::OwnerProposal(value) => {
            jig_client_view::owner_proposal::render(b, &root.jig().borrow(root.client()), value, words, jig);
        }
        ReadValue::Proposal(value) => {
            jig_client_view::cards::proposal::action(
                b,
                &value.action,
                words,
                jig,
                root.jig().object_rows().rows.capacity(),
            );
            jig_client_view::cards::common::authority(b, &value.needs, words, jig);
            jig_client_view::cards::common::funding(b, &value.funding, value.needs.budget, words);
        }
        ReadValue::Tasks(_)
        | ReadValue::OwnerProject(_)
        | ReadValue::Historical(_)
        | ReadValue::Me(_)
        | ReadValue::Task(_)
        | ReadValue::Escalation(_)
        | ReadValue::OwnerEscalation(_)
        | ReadValue::Question(_)
        | ReadValue::PersonTask(_)
        | ReadValue::OwnerPersonTask(_)
        | ReadValue::Result(_)
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
pub(crate) fn card(
    b: &mut Builder,
    root: &Domain,
    card: &Card,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
    jig_bindings: &mut JigBindings,
) {
    let Some(page) = &card.page else { return };
    facts(b, root, page, jig, jig_bindings);
    if let Some(outcome) = &card.outcome {
        match outcome {
            Outcome::Answered(Answered::Jig(Answer::Done(outcome))) => jig.outcome(b, outcome),
            Outcome::Answered(Answered::Jig(Answer::Refused { refusal, .. })) => {
                jig.refusal(b, refusal, jig_bindings.text_limit());
            }
            Outcome::NotKept => jig.status(b, P::RequestNotKept, &[]),
            Outcome::Unknown => jig.status(b, P::Unknown, &[]),
            Outcome::Limit => jig.status(b, P::RequestsInFlight, &[]),
            Outcome::Unconfirmed => jig.status(b, P::RequestUnconfirmed, &[]),
            Outcome::Answered(Answered::Family(_)) => {}
        }
        return;
    }
    if let Some(progress) = acts::pending(card, root.client()) {
        progress_text(b, jig, progress);
        return;
    }
    if card.key.is_some() {
        return;
    }
    for (act, word) in ACTIONS {
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
                jig.text(b, P::ProposeInstead, &[]);
                jig_client_view::cards::common::recipient(b, to, jig);
            }
            Admission::Refused(refusal) => jig.refusal(b, refusal, jig_bindings.text_limit()),
        }
    }
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    jig: &jig_client_view::words::Words,
    bindings: &mut Bindings,
    jig_bindings: &mut JigBindings,
) {
    let Some(open) = root.pages().confirmation() else { return };
    if open.act == ActKind::Cancel {
        close(b, root, words, jig, bindings);
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
        facts(b, root, page, jig, jig_bindings);
        if jig_client::facts::dangerous(open.act) {
            jig.status(b, P::Dangerous, &[]);
        }
        if let Some(reason) = &card.words {
            b.open(Role::Paragraph);
            b.text(Run::Plain, reason.bytes());
            b.close();
        }
        if let Some(Effect::Release(effect)) = jig_client::facts::effect(acts::offers(page), open.act) {
            jig.text(
                b,
                match effect {
                    ReleaseEffect::Judged => P::ReleaseFresh,
                    ReleaseEffect::RunAgain => P::ResumeRelease,
                    ReleaseEffect::Uncertain => P::EngineFollowsChoice,
                },
                &[],
            );
            if *effect == ReleaseEffect::Uncertain {
                button(b, bindings, words, Word::Again, Act::Again { again: true });
                button(b, bindings, words, Word::DoNotRepeat, Act::Again { again: false });
            }
        }
        button(b, bindings, words, Word::Confirm, Act::Confirm);
        button(b, bindings, words, Word::Cancel, Act::Cancel);
        b.close();
        return;
    }
}

pub(crate) fn progress_text(b: &mut Builder, jig: &jig_client_view::words::Words, progress: jig_client::Progress) {
    jig.progress(
        b,
        match progress {
            jig_client::Progress::InFlight => jig_client_view::intents::Progress::InFlight,
            jig_client::Progress::Retrying { .. } => jig_client_view::intents::Progress::Retrying,
            jig_client::Progress::Parked | jig_client::Progress::NotServed => {
                jig_client_view::intents::Progress::Parked
            }
            jig_client::Progress::Unknown => jig_client_view::intents::Progress::Unknown,
        },
    );
}

fn close(b: &mut Builder, root: &Domain, words: &Words, jig: &jig_client_view::words::Words, bindings: &mut Bindings) {
    use jig_client::contract::Ask;
    let Some(chat) = root.pages().chat() else { return };
    let Some(Ask::Compound { operations, .. }) = &chat.closing else { return };
    b.open(Role::Dialog);
    b.name(&words.say(Word::Close, &[]));
    jig.status(b, P::Dangerous, &[]);
    if let Some(task) = chat.task(root.client())
        && let Some(Effect::Cancel(impact)) = jig_client::facts::effect(task.acts.as_slice(), ActKind::Cancel)
    {
        jig_client_view::confirmation::impact_words(b, impact, jig);
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
