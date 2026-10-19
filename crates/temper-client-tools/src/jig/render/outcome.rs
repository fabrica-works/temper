//! Durable answers use the same operation words before and after a restart.
use super::{Text, acts, document};
use crate::jig::{Limits, runner::Printed};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Outcome, contract as c};
use skein_json::writer::Encoder;
#[derive(Debug)]
pub(crate) struct Operation {
    pub word: &'static [u8],
    pub task: Option<u64>,
    pub to: Option<c::Recipient>,
    pub party: Option<u64>,
    pub retry: Option<Box<[u8]>>,
}
impl Operation {
    pub(crate) fn of(ask: &Ask) -> Self {
        let (word, task): (&[u8], Option<u64>) = match ask {
            Ask::Family { .. } => (b"adopt", None),
            Ask::Jig(ask) => match ask.as_ref() {
                c::Ask::Create { .. } | c::Ask::Propose { action: c::ProposeAction::Batch { .. }, .. } => {
                    (b"chat", None)
                }
                c::Ask::Message { task, .. } => (b"message", Some(*task)),
                c::Ask::Compound { operations, .. } => {
                    let mut task = None;
                    for operation in operations {
                        if let c::Operation::Cancel { task: number, .. } = operation {
                            task = Some(*number);
                        }
                    }
                    (b"close", task)
                }
                c::Ask::Answer { on, .. } => (
                    b"answer",
                    Some(match on {
                        c::AnswerOn::Question { task, .. } | c::AnswerOn::Person { task, .. } => *task,
                    }),
                ),
                c::Ask::Decide { on, .. } => (
                    b"decide",
                    match on {
                        c::DecideOn::Escalation { task, .. } => Some(*task),
                        c::DecideOn::Proposal { proposal, .. } => match proposal.proposer {
                            c::Proposer::Task(task) => Some(task),
                            c::Proposer::Party(_) => None,
                        },
                    },
                ),
                c::Ask::Release { task, .. } => (b"release", Some(*task)),
                c::Ask::Stop { task, .. } => (b"stop", Some(*task)),
                c::Ask::Cancel { task, .. } => (b"cancel", Some(*task)),
                c::Ask::Take { task, .. } => (b"take", Some(*task)),
                c::Ask::HandBack { task, .. } => (b"hand-back", Some(*task)),
                c::Ask::Amend { task, .. }
                | c::Ask::Propose { action: c::ProposeAction::Amendment { task, .. }, .. } => (b"amend", Some(*task)),
                c::Ask::Move { task, .. } => (b"move", Some(*task)),
                c::Ask::Seen { .. } => (b"seen", None),
                c::Ask::EditNote { .. } => (b"edit-note", None),
                c::Ask::DeleteNote { .. } => (b"delete-note", None),
                c::Ask::ChangePolicy { .. } => (b"change-policy", None),
                c::Ask::SetRoles { .. } => (b"set-roles", None),
                c::Ask::MakeService { .. } => (b"make-service", None),
                c::Ask::SetPool { .. } => (b"set-pool", None),
                c::Ask::Withdraw { .. } => (b"withdraw", None),
            },
        };
        Self { word, task, to: None, party: None, retry: None }
    }
}
pub(crate) fn print(
    operation: &Operation,
    outcome: &Outcome,
    json: bool,
    me: Option<&c::Me>,
    limits: &Limits,
) -> Printed {
    let status = match outcome {
        Outcome::Answered(Answered::Jig(c::Answer::Done(_))) => 0,
        Outcome::Answered(Answered::Jig(c::Answer::Refused { .. })) => 2,
        Outcome::NotKept
        | Outcome::Answered(Answered::Family(_))
        | Outcome::Unknown
        | Outcome::Limit
        | Outcome::Unconfirmed
        | Outcome::Held => 1,
    };
    let bytes = bytes(operation, outcome, json, me, limits, false);
    if status == 0 {
        Printed { status, stdout: bytes, stderr: Box::default() }
    } else {
        Printed { status, stdout: Box::default(), stderr: bytes }
    }
}
pub(crate) fn kept(
    operation: &Operation,
    outcome: &Outcome,
    json: bool,
    me: Option<&c::Me>,
    limits: &Limits,
) -> Box<[u8]> {
    bytes(operation, outcome, json, me, limits, true)
}
fn bytes(
    operation: &Operation,
    outcome: &Outcome,
    json: bool,
    me: Option<&c::Me>,
    limits: &Limits,
    kept: bool,
) -> Box<[u8]> {
    if json {
        let bounds = skein_json::writer::Limits {
            depth: 32,
            length: u32::try_from(limits.ask_bytes).expect("output bytes fit"),
        };
        let mut measure = Encoder::measure(&bounds);
        encode(operation, outcome, &mut measure, kept);
        let Ok(size) = measure.measured() else { return super::failure(b"result limit", true, 1).stderr };
        let mut write = Encoder::write(size, &bounds);
        encode(operation, outcome, &mut write, kept);
        let bytes = write.finish();
        if kept { newline(&bytes) } else { bytes }
    } else {
        let mut measure = Text::measure();
        text(operation, outcome, me, limits, kept, &mut measure);
        let mut write = Text::writing_exact(measure);
        text(operation, outcome, me, limits, kept, &mut write);
        write.finish()
    }
}
fn client_reason(outcome: &Outcome) -> Option<&'static [u8]> {
    match outcome {
        Outcome::NotKept => Some(b"request not kept"),
        Outcome::Unknown => Some(b"unknown past retention"),
        Outcome::Limit => Some(b"request limit"),
        Outcome::Held => Some(b"name held by another request"),
        Outcome::Unconfirmed => Some(b"tools' bug; unconfirmed request; report it as a finding"),
        Outcome::Answered(Answered::Family(_)) => Some(b"tools' bug; unexpected family answer; report it as a finding"),
        Outcome::Answered(Answered::Jig(_)) => None,
    }
}
fn text(operation: &Operation, outcome: &Outcome, me: Option<&c::Me>, limits: &Limits, kept: bool, out: &mut Text) {
    if kept {
        out.put(b"kept: ");
        out.put(operation.word);
        out.put(b", ");
    }
    if let Some(reason) = client_reason(outcome) {
        out.put(reason);
        out.put(b"\n");
        return;
    }
    let Outcome::Answered(Answered::Jig(answer)) = outcome else { unreachable!("client answers handled") };
    match answer {
        c::Answer::Done(c::Outcome::Proposed { proposal }) => proposed(*proposal, operation, me, out),
        c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::Proposed { proposal })) => proposed(
            c::ProposalKey {
                proposer: c::Proposer::Party(operation.party.expect("request party retained")),
                number: *proposal,
            },
            operation,
            me,
            out,
        ),
        c::Answer::Done(c::Outcome::DecidedFirst { by, how, at }) => {
            out.put(b"decided first: by ");
            out.put(by.name.bytes());
            out.put(b", ");
            decision(how, out);
            out.put(b", ");
            document::utc(*at, out);
        }
        c::Answer::Done(done) => {
            out.put(b"done: ");
            let task = match done {
                c::Outcome::Made { what: c::Made::Tasks { tasks } } => tasks.get(0).copied(),
                c::Outcome::Owner(c::OwnerOutcome::Created { first, .. }) => Some(*first),
                c::Outcome::Owner(_)
                | c::Outcome::Made { .. }
                | c::Outcome::Proposed { .. }
                | c::Outcome::Decided { .. }
                | c::Outcome::DecidedFirst { .. }
                | c::Outcome::Changed => operation.task,
            };
            if let Some(task) = task {
                out.put(b"task ");
                out.number(task);
                out.put(b", ");
            }
            out.put(operation.word);
        }
        c::Answer::Refused { refusal, .. } => {
            out.put(operation.word);
            out.put(b": ");
            acts::refusal(refusal, &document::phrasebook(limits.text), out);
            if let c::Refusal::Moved { .. } = refusal
                && let Some(task) = operation.task
            {
                out.put(b"\nread again: task ");
                out.number(task);
            }
            if let c::Refusal::Authority { proposable: Some(true), .. } = refusal {
                out.put(b"\npropose instead: ");
                match &operation.retry {
                    Some(call) => out.put(call),
                    None => out.put(operation.word),
                }
            }
        }
    }
    out.put(b"\n");
}
fn proposed(proposal: c::ProposalKey, operation: &Operation, me: Option<&c::Me>, out: &mut Text) {
    out.put(b"proposed: ");
    let name = crate::jig::argv::value::encode(
        &[],
        crate::jig::table::Kind::Proposal,
        &crate::jig::call::Value::Proposal(proposal),
    )
    .expect("proposal identity is printable");
    out.put(&name);
    if let Some(to) = &operation.to {
        out.put(b", to ");
        acts::recipient(to, me, out);
    }
}
fn decision(how: &c::Decision, out: &mut Text) {
    out.put(match how {
        c::Decision::Accepted => b"accepted",
        c::Decision::Rejected { .. } => b"rejected",
        c::Decision::PassedUp => b"passed up",
        c::Decision::Released { .. } => b"released",
        c::Decision::LeftHeld { .. } => b"left held",
    });
}
fn encode(operation: &Operation, outcome: &Outcome, out: &mut Encoder, kept: bool) {
    out.object_start();
    out.key(b"operation");
    out.string(operation.word);
    out.key(b"outcome");
    if let Some(reason) = client_reason(outcome) {
        out.object_start();
        out.key(b"error");
        out.string(reason);
        out.object_end();
    } else {
        let Outcome::Answered(Answered::Jig(answer)) = outcome else { unreachable!("client answers handled") };
        document::answer_json(answer, out);
    }
    if !kept {
        out.key(b"task");
        match operation.task {
            Some(task) => out.unsigned(task),
            None => out.null(),
        }
        out.key(b"to");
        match &operation.to {
            Some(to) => document::recipient_json(to, out),
            None => out.null(),
        }
    }
    if let Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal, .. })) = outcome {
        if let c::Refusal::Moved { .. } = refusal
            && let Some(task) = operation.task
        {
            let mut measured = Text::measure();
            read_again(task, &mut measured);
            let mut write = Text::writing_exact(measured);
            read_again(task, &mut write);
            out.key(b"read_again");
            out.string(&write.finish());
        }
        if let c::Refusal::Authority { proposable: Some(true), .. } = refusal {
            out.key(b"propose_instead");
            out.string(operation.retry.as_deref().unwrap_or(operation.word));
        }
    }
    out.object_end();
}
fn read_again(task: u64, out: &mut Text) {
    out.put(b"task ");
    out.number(task);
}
pub(crate) fn newline(bytes: &[u8]) -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(bytes.len().checked_add(1).expect("line size fits"));
    out.put(bytes).expect("measured line");
    out.put(b"\n").expect("measured newline");
    out.finish()
}
pub(crate) fn retry(call: &crate::jig::call::Call) -> Box<[u8]> {
    use crate::jig::{argv, rows};
    let tool = match call.tool {
        8 => &rows::CHAT_CREATE,
        9 => &rows::MESSAGE,
        10 => &rows::CLOSE,
        11 => &rows::ANSWER,
        12 => &rows::DECIDE,
        13 => &rows::RELEASE,
        14 => &rows::STOP,
        15 => &rows::CANCEL,
        _ => unreachable!("act tool"),
    };
    let mut call = call.clone();
    call.tool = 0;
    let mut words = skein_lib::List::with_capacity(128);
    argv::back(&[tool], &call, &mut words).expect("parsed act");
    let mut measure = Text::measure();
    retry_words(words.as_slice(), &mut measure);
    let mut out = Text::writing_exact(measure);
    retry_words(words.as_slice(), &mut out);
    out.finish()
}
fn retry_words(words: &[Box<[u8]>], out: &mut Text) {
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            out.put(b" ");
        }
        super::lists::argument(word, out);
    }
}
