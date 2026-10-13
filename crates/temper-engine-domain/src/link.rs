//! Worker channels in the hub's link vocabulary (jig's `domain/hosts.md`, 2
//! and 6.4; `testing/jig-test-domain/src/link.rs`). Payloads stay owned here
//! until the fleet asks for or drops them. The protocol layer alone reads an
//! ended answer's outcome and encodes typed assignments.
use crate::boundary::{Link, Parts, Payload, Request, Work, WorkerRequest};
use crate::domain::Domain;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, List, Token, Writer};

/// Translate one channel input into the core's fleet vocabulary.
pub(crate) fn receive(domain: &mut Domain, channel: Token, link: Link) -> core::Event {
    core::Event::Fleet(match link {
        Link::Hello(hello) => fleet::Event::Hello { channel, hello: self::hello(hello) },
        Link::Up { up, parts } => self::up(domain, channel, up, parts),
        Link::Lost => fleet::Event::Lost { channel },
    })
}

fn hello(hello: host::Hello) -> fleet::Hello {
    let mut models = List::with_capacity(u32::try_from(hello.bounds.models.len()).expect("bounded models"));
    for model in hello.bounds.models {
        models
            .push(fleet::ModelBound { name: model.name, window: model.window, output: model.output })
            .expect("one slot per model");
    }
    let mut hosting = List::with_capacity(u32::try_from(hello.hosting.len()).expect("bounded hosted runs"));
    for run in hello.hosting {
        hosting
            .push(fleet::Hosted { run: run.run, attempt: run.attempt, phase: phase(run.phase) })
            .expect("one slot per hosted run");
    }
    fleet::Hello {
        slots: hello.slots,
        stop_bound: hello.stop_bound,
        workstreams: hello.workstreams,
        hosting: hosting.into_boxed(),
        bounds: fleet::Bounds {
            models: models.into_boxed(),
            turns: hello.bounds.turns,
            spend: hello.bounds.spend,
            time: hello.bounds.time,
            waiting: hello.bounds.waiting,
            checks: hello.bounds.checks,
            message_bytes: hello.bounds.message_bytes,
            inbox: hello.bounds.inbox,
            unacknowledged: hello.bounds.unacknowledged,
        },
    }
}

fn phase(phase: host::Phase) -> fleet::Phase {
    match phase {
        host::Phase::Preparing => fleet::Phase::Preparing,
        host::Phase::Starting => fleet::Phase::Starting,
        host::Phase::Active => fleet::Phase::Active,
        host::Phase::Waiting => fleet::Phase::Waiting,
        host::Phase::Ending => fleet::Phase::Ending,
        host::Phase::Answered => fleet::Phase::Answered,
    }
}

/// Retain a worker's bytes and temper's parts; translate only their metadata.
fn up(domain: &mut Domain, channel: Token, up: host::LinkUp, parts: Parts) -> fleet::Event {
    match up {
        host::LinkUp::Turn { run, attempt, turn } => {
            let number = turn.turn;
            let body = payload(
                domain,
                Payload::Turn {
                    task: run.raw(),
                    attempt: attempt.raw(),
                    turn: number,
                    cumulative: turn.spent,
                    read: read_name(turn.read),
                    transcript: turn.body,
                    opens_window: turn.opens_window,
                    refused: refusals(&turn.refused),
                },
            );
            fleet::Event::Turn { channel, run, attempt, turn: number, body }
        }
        host::LinkUp::Answer { run, attempt, answer } => {
            let kind = match answer.ending {
                host::Ending::Ended { .. } => fleet::Answer::Ended,
                host::Ending::Parked { .. } => fleet::Answer::Parked,
                host::Ending::Failed { .. } => fleet::Answer::Failed,
                host::Ending::Refused(host::Refusal::Busy) => fleet::Answer::Busy,
                host::Ending::Refused(host::Refusal::Invalid(_)) => fleet::Answer::Invalid,
            };
            let payload = payload(
                domain,
                Payload::Answer {
                    task: run.raw(),
                    attempt: attempt.raw(),
                    cumulative: answer.spent,
                    read: read_name(answer.read),
                    refused: refusals(&answer.refused),
                    ending: answer.ending,
                    parts,
                },
            );
            fleet::Event::Answer { channel, run, attempt, answer: kind, payload }
        }
        host::LinkUp::Call { run, attempt, call, tool, writes, input, deadline } => {
            fleet::Event::Relay { channel, run, attempt, call: fleet::Call { name: call, tool, writes, input, deadline } }
        }
        host::LinkUp::Rejected { run, attempt, account, generation } => {
            fleet::Event::Rejected { channel, run, attempt, account, generation }
        }
        host::LinkUp::Exhausted { run, attempt, account, retry_after } => {
            fleet::Event::Exhausted { channel, run, attempt, account, retry_after }
        }
        host::LinkUp::Activity { run, attempt, activity } => {
            fleet::Event::Observe { channel, run, attempt, report: report(activity) }
        }
    }
}

fn payload(domain: &mut Domain, value: Payload) -> Token {
    domain.payloads.insert(Some(value)).expect("the fleet's turns and attempts bound the root's payloads").token()
}

fn read_name(name: Option<Token>) -> Option<u64> {
    Some(name?.raw())
}

fn refusals(values: &[host::Refused]) -> Box<[u64]> {
    let mut names = List::with_capacity(u32::try_from(values.len()).expect("bounded refusals"));
    for value in values {
        names.push(value.name.raw()).expect("one slot per refusal");
    }
    names.into_boxed()
}

/// The fleet fenced an answer: an ended one's outcome goes to the protocol
/// layer to read; any other ending is translated here.
pub(crate) fn answer(domain: &mut Domain, run: Token, attempt: Token, payload: Token) {
    let end = match domain.payloads.get(skein_lib::Id::from_token(payload)) {
        Some(Some(Payload::Answer { task, attempt: retained, ending, .. })) => {
            assert!(*task == run.raw() && *retained == attempt.raw(), "fenced answer identity");
            match ending {
                host::Ending::Ended { outcome, .. } => {
                    let request = Request::Worker(WorkerRequest::ReadResult { payload, outcome: outcome.clone() });
                    assert!(crate::domain::now(domain, request), "result read door reserved");
                    return;
                }
                other @ (host::Ending::Refused(_) | host::Ending::Parked { .. } | host::Ending::Failed { .. }) => {
                    end(other)
                }
            }
        }
        Some(
            Some(
                Payload::Busy
                | Payload::Call { .. }
                | Payload::SettledCall(_)
                | Payload::InboxWord(_)
                | Payload::Turn { .. },
            )
            | None,
        )
        | None => unreachable!("the fleet retained the answer body"),
    };
    let event = result_event(domain, payload, end).expect("retained answer");
    domain.work.push(Work::Core(event));
}

/// The answer's task terminal, with temper's saved work translated and what
/// it left on the forge kept for a lost run (domain/forge.md).
pub(crate) fn result_event(domain: &mut Domain, payload: Token, end: tasks::End) -> Option<core::Event> {
    let (task, attempt, cumulative, read, refused, parts) = match domain.payloads.get(skein_lib::Id::from_token(payload))
    {
        Some(Some(Payload::Answer { task, attempt, cumulative, read, refused, parts, .. })) => {
            (*task, *attempt, *cumulative, *read, refused.clone(), parts.clone())
        }
        Some(
            Some(
                Payload::Busy
                | Payload::Call { .. }
                | Payload::SettledCall(_)
                | Payload::InboxWord(_)
                | Payload::Turn { .. },
            )
            | None,
        )
        | None => return None,
    };
    let Parts { saved, pushed } = parts;
    drop(domain.forge_left.insert((task, attempt), pushed));
    let (saved, invalid_saved) = match saved {
        Some(tags) => match crate::route::forge_route::saved_resources(domain.config.forge_connector, &tags) {
            Some(resources) => (Some(resources), false),
            None => (None, true),
        },
        None => (None, false),
    };
    Some(core::Event::AnswerPayload {
        run: Token::new(task),
        attempt: Token::new(attempt),
        payload,
        cumulative,
        end,
        saved,
        invalid_saved,
        read,
        refused,
    })
}

fn end(ending: &host::Ending) -> tasks::End {
    match ending {
        host::Ending::Refused(host::Refusal::Busy) => tasks::End::Refused,
        host::Ending::Refused(host::Refusal::Invalid(_)) => tasks::End::Failed(tasks::Class::Invalid),
        host::Ending::Parked { .. } => tasks::End::Parked,
        host::Ending::Failed { failure, .. } => tasks::End::Failed(match failure {
            host::Failure::Unprepared(host::Preparation::Transient) => tasks::Class::Transient,
            host::Failure::Unprepared(host::Preparation::Permanent { .. }) => tasks::Class::Permanent,
            host::Failure::Run(_) => tasks::Class::Run,
            host::Failure::Agent(_) | host::Failure::Cancelled(_) => tasks::Class::Agent,
        }),
        host::Ending::Ended { .. } => unreachable!("ended payloads leave for the protocol routing"),
    }
}

fn report(activity: host::Activity) -> fleet::Report {
    match activity {
        host::Activity::Phase(phase) => fleet::Report::Activity(match phase {
            host::Phase::Preparing => fleet::Activity::Preparing,
            host::Phase::Starting => fleet::Activity::Starting,
            host::Phase::Active => fleet::Activity::Active,
            host::Phase::Waiting => fleet::Activity::Waiting,
            host::Phase::Ending | host::Phase::Answered => fleet::Activity::Ending,
        }),
        host::Activity::Long { operation, span } => fleet::Report::Activity(fleet::Activity::Long {
            operation: match operation {
                host::Operation::Checks => fleet::Operation::Checks,
                host::Operation::Completion => fleet::Operation::Completion,
                host::Operation::Command => fleet::Operation::Command,
            },
            span,
        }),
        host::Activity::LongDone => fleet::Report::Activity(fleet::Activity::Active),
        host::Activity::Count { kind, elapsed, count } => fleet::Report::Count {
            kind: match kind {
                host::CountKind::Admitted => fleet::CountKind::Admitted,
                host::CountKind::Ended => fleet::CountKind::Ended,
                host::CountKind::LlmStarted => fleet::CountKind::LlmStarted,
                host::CountKind::LlmRetried => fleet::CountKind::LlmRetried,
                host::CountKind::LlmFinished => fleet::CountKind::LlmFinished,
                host::CountKind::ToolStarted => fleet::CountKind::ToolStarted,
                host::CountKind::ToolFinished => fleet::CountKind::ToolFinished,
                host::CountKind::CheckStarted => fleet::CountKind::CheckStarted,
                host::CountKind::CheckFinished => fleet::CountKind::CheckFinished,
                host::CountKind::TextArrived => fleet::CountKind::TextArrived,
            },
            elapsed,
            count,
        },
    }
}

/// One committed inbox word for its live host, its kind kept in its words.
pub(crate) fn message(run: Token, attempt: Token, word: &tasks::Word) -> host::LinkDown {
    host::LinkDown::Message {
        run,
        attempt,
        name: Token::new(word.number),
        sender: sender(word.from),
        words: inbox_words(word),
    }
}

/// Preserve the task-owned message kind when translating its opaque words.
fn inbox_words(word: &tasks::Word) -> Box<[u8]> {
    let (label, number) = match word.kind {
        tasks::MessageKind::Result(_) => (
            b"Result from task ".as_slice(),
            match word.from {
                tasks::Party::Task(task) => task,
                tasks::Party::Person(person) => person,
                tasks::Party::Deployment { .. } => 0,
            },
        ),
        tasks::MessageKind::ProposalDecision { proposal, accepted } => {
            (if accepted { b"Proposal accepted ".as_slice() } else { b"Proposal rejected ".as_slice() }, proposal)
        }
        tasks::MessageKind::PersonStalled { task } => (b"Person task past its waiting deadline ".as_slice(), task),
        tasks::MessageKind::EffectNotMade { entry } => (b"Effect not made ".as_slice(), entry),
        tasks::MessageKind::Words
        | tasks::MessageKind::Question
        | tasks::MessageKind::Answer { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::News { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::Escalation { .. } => return word.words.clone(),
    };
    let number = Decimal::of(number);
    let bytes = label.len().checked_add(number.as_bytes().len()).expect("bounded message label");
    let bytes = bytes.checked_add(2).expect("bounded message separator");
    let bytes = bytes.checked_add(word.words.len()).expect("bounded message envelope");
    let mut writer = Writer::new(bytes);
    writer.put(label).expect("measured label");
    writer.put(number.as_bytes()).expect("measured source");
    writer.put(b":\n").expect("measured separator");
    writer.put(&word.words).expect("measured words");
    writer.finish()
}

fn sender(value: tasks::Party) -> Box<[u8]> {
    match value {
        tasks::Party::Task(number) | tasks::Party::Person(number) => Box::from(Decimal::of(number).as_bytes()),
        tasks::Party::Deployment { .. } => Box::from(&b"deployment"[..]),
    }
}
