//! Loads of durable rows outside the working set: what a run's preparation
//! reads (its transcript, its dependencies' results, its inputs), notes, and
//! the core's historical queries (jig's `domain/engine.md`, 7.2).

use crate::boundary::{DependencyRead, EndedRead, Read, Request, Work};
use crate::domain::Domain;
use crate::restart::request_load;
use crate::{Key, Range, Record, loads};
use alloc::boxed::Box;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Id, List, Queue, ReplyTo, Token};

/// What a live load waiter reads, if it still reads anything.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Waiting {
    Escalation,
    Proposal,
    Ended,
    Transcript,
    Dependency,
    Input,
    Notes,
}

pub(crate) fn waiting(domain: &Domain, waiter: Token) -> Option<Waiting> {
    match domain.archive_reads.get(Id::from_token(waiter))? {
        Some(Read::Escalation(_)) => Some(Waiting::Escalation),
        Some(Read::Proposal(_)) => Some(Waiting::Proposal),
        Some(Read::Ended(_)) => Some(Waiting::Ended),
        Some(Read::Transcript { .. }) => Some(Waiting::Transcript),
        Some(Read::Dependency(_)) => Some(Waiting::Dependency),
        Some(Read::InputCheck(_)) => Some(Waiting::Input),
        Some(Read::Notes { .. }) => Some(Waiting::Notes),
        None => None,
    }
}

/// Free a reserved read slot the core no longer needs.
pub(crate) fn retire_read(domain: &mut Domain, waiter: Token) {
    if take_read(domain, waiter).is_some() {
        domain.archive_reads.retire(Id::from_token(waiter));
    }
}

pub(crate) fn input_failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::InputCheck(read)) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain
        .work
        .push(Work::Core(jig_domain_core::Event::DelegateInputFailed { to: ReplyTo::new(read.to), key: read.key }));
}

pub(crate) fn input_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    out: &mut Queue<Request>,
) {
    let Some(Some(Read::InputCheck(read))) = domain.archive_reads.get(Id::from_token(waiter)) else { return };
    let wanted = *read.ids.get(usize::try_from(read.at).expect("bounded input index")).expect("requested input");
    let project = read.project;
    let creator = read.key.task;
    if next.is_some() || rows.len() != 1 {
        input_failed(domain, waiter);
        return;
    }
    let row = match rows.into_iter().next().expect("one input result row") {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) => row,
        Record::Core(
            jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Call(_)
                | jig_domain_core::CoreRecord::Deployment(_)
                | jig_domain_core::CoreRecord::EscalationDecision(_)
                | jig_domain_core::CoreRecord::ProposalDecision(_)
                | jig_domain_core::CoreRecord::Turn(_)
                | jig_domain_core::CoreRecord::RunProof(_)
                | jig_domain_core::CoreRecord::Terminal(_)
                | jig_domain_core::CoreRecord::Projection(_),
            )
            | jig_domain_core::Record::Tasks(_)
            | jig_domain_core::Record::People(_)
            | jig_domain_core::Record::Notes(_),
        )
        | Record::Forge { .. } => {
            input_failed(domain, waiter);
            return;
        }
    };
    let Some(stub) = domain.core.input_stub(creator, project, wanted, row) else {
        input_failed(domain, waiter);
        return;
    };
    let Some(Some(Read::InputCheck(read))) = domain.archive_reads.get_mut(Id::from_token(waiter)) else {
        unreachable!("input read survives validation")
    };
    read.stubs.push(stub).expect("bounded historical input count");
    read.at = read.at.checked_add(1).expect("bounded input index");
    if let Some(&next) = read.ids.get(usize::try_from(read.at).expect("bounded input index")) {
        request_load(domain, waiter, Range::Core(crate::CoreRange::TaskResult { task: next }), None, out);
        return;
    }
    let Some(Read::InputCheck(read)) = take_read(domain, waiter) else { unreachable!("completed input read") };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Core(jig_domain_core::Event::DelegateValidated {
        to: ReplyTo::new(read.to),
        key: read.key,
        batch: read.batch,
        stubs: read.stubs.into_boxed(),
    }));
}

pub(crate) fn take_note_owner(domain: &mut Domain, waiter: Token) -> Option<Token> {
    match take_read(domain, waiter) {
        Some(Read::Notes { owner }) => Some(owner),
        Some(
            Read::Ended(_)
            | Read::Escalation(_)
            | Read::Proposal(_)
            | Read::Transcript { .. }
            | Read::Dependency(_)
            | Read::InputCheck(_),
        )
        | None => None,
    }
}

pub(crate) fn note_failed(domain: &mut Domain, waiter: Token) {
    let Some(owner) = take_note_owner(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Core(jig_domain_core::Event::Notes(jig_domain_notes::Event::LoadFailed { owner })));
}

pub(crate) fn note_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
) {
    if cut.is_some() {
        note_failed(domain, waiter);
        return;
    }
    let mut kept = List::with_capacity(u32::try_from(rows.len()).expect("bounded note page"));
    for row in rows {
        match row {
            Record::Core(jig_domain_core::Record::Notes(record)) => {
                let valid = match &record {
                    notes::Record::Entry(entry) => notes::valid_entry(entry, &domain.limits.notes),
                    notes::Record::Line(line) => notes::valid_line(line, &domain.limits.notes),
                };
                if !valid {
                    note_failed(domain, waiter);
                    return;
                }
                kept.push(record).expect("note page room");
            }
            Record::Core(
                jig_domain_core::Record::Core(
                    jig_domain_core::CoreRecord::Call(_)
                    | jig_domain_core::CoreRecord::Deployment(_)
                    | jig_domain_core::CoreRecord::EscalationDecision(_)
                    | jig_domain_core::CoreRecord::ProposalDecision(_)
                    | jig_domain_core::CoreRecord::Turn(_)
                    | jig_domain_core::CoreRecord::RunProof(_)
                    | jig_domain_core::CoreRecord::Terminal(_)
                    | jig_domain_core::CoreRecord::Projection(_),
                )
                | jig_domain_core::Record::Tasks(_)
                | jig_domain_core::Record::People(_),
            )
            | Record::Forge { .. } => {
                note_failed(domain, waiter);
                return;
            }
        }
    }
    let Some(owner) = take_note_owner(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    domain.work.push(Work::Core(jig_domain_core::Event::Notes(jig_domain_notes::Event::Loaded {
        owner,
        rows: jig_domain_notes::Rows { records: kept },
        more: next.is_some(),
    })));
}

/// One ended owner row the core asked for, or none when the store has none.
pub(crate) fn ended_loaded(domain: &mut Domain, waiter: Token, rows: Box<[Record]>) {
    let Some(Read::Ended(read)) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    let mut row = None;
    if rows.len() == 1
        && let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) = rows.into_iter().next()
    {
        row = Some(ended);
    }
    ended_answer(domain, read, row, false);
}

pub(crate) fn ended_failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::Ended(read)) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    ended_answer(domain, read, None, true);
}

/// Answer the core's ended-row question in its own vocabulary.
pub(crate) fn ended_answer(domain: &mut Domain, read: EndedRead, row: Option<Box<tasks::TaskRecord>>, busy: bool) {
    let event = match read {
        EndedRead::SeenChat { request, .. } => jig_domain_core::Event::SeenChatLoaded { request, row, busy },
        EndedRead::Chat { .. } => match row {
            Some(row) => jig_domain_core::Event::ForgetChat { row },
            None => return,
        },
        EndedRead::PersonTask { request, person, project, task } => {
            jig_domain_core::Event::HistoricalPersonTask { request, person, project, task, row, busy }
        }
    };
    domain.work.push(Work::Core(event));
}

pub(crate) fn transcript_failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::Transcript { task }) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    // A stale read fails no current preparation.
    if domain.core.transcripts.contains_key(&task) {
        domain.work.push(Work::Core(jig_domain_core::Event::PreparationFailed { task }));
    }
}

pub(crate) fn begin_dependency_read(domain: &mut Domain, task: u64) -> Option<(Token, u64)> {
    // A message can end the preparation before its transcript is read; then
    // nothing is read for it (jig's `examples/ops/domain/src/restart.rs`).
    let ids = domain.core.preparation_ids(task)?;
    if ids.is_empty() {
        domain.work.push(Work::StartBrief { task });
        return None;
    }
    let first = *ids.first().expect("nonempty preparation IDs");
    let count = ids.len();
    let read = DependencyRead {
        task,
        ids,
        at: 0,
        results: List::with_capacity(u32::try_from(count).expect("bounded result count")),
    };
    let Ok(waiter) = domain.archive_reads.insert(Some(Read::Dependency(read))) else {
        domain.work.push(Work::Core(jig_domain_core::Event::PreparationFailed { task }));
        return None;
    };
    Some((waiter.token(), first))
}

pub(crate) fn dependency_failed(domain: &mut Domain, waiter: Token) {
    let Some(Read::Dependency(read)) = take_read(domain, waiter) else { return };
    domain.archive_reads.retire(Id::from_token(waiter));
    // A stale read fails no current preparation.
    if domain.core.contexts.contains_key(&read.task) {
        domain.work.push(Work::Core(jig_domain_core::Event::PreparationFailed { task: read.task }));
    }
}

pub(crate) fn dependency_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    out: &mut Queue<Request>,
) {
    let Some(Some(Read::Dependency(read))) = domain.archive_reads.get(Id::from_token(waiter)) else { return };
    // A message can end the preparation while this read is in flight: its
    // answer belongs to no current preparation.
    if !domain.core.contexts.contains_key(&read.task) {
        retire_read(domain, waiter);
        return;
    }
    let wanted = *read.ids.get(usize::try_from(read.at).expect("bounded index")).expect("one requested result");
    if next.is_some() || rows.len() != 1 {
        dependency_failed(domain, waiter);
        return;
    }
    let row = match rows.into_iter().next().expect("one dependency result row") {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) => row,
        Record::Core(
            jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Call(_)
                | jig_domain_core::CoreRecord::Deployment(_)
                | jig_domain_core::CoreRecord::EscalationDecision(_)
                | jig_domain_core::CoreRecord::ProposalDecision(_)
                | jig_domain_core::CoreRecord::Turn(_)
                | jig_domain_core::CoreRecord::RunProof(_)
                | jig_domain_core::CoreRecord::Terminal(_)
                | jig_domain_core::CoreRecord::Projection(_),
            )
            | jig_domain_core::Record::Tasks(_)
            | jig_domain_core::Record::People(_)
            | jig_domain_core::Record::Notes(_),
        )
        | Record::Forge { .. } => {
            dependency_failed(domain, waiter);
            return;
        }
    };
    let Some(result) = domain.core.dependency_result(read.task, wanted, row) else {
        dependency_failed(domain, waiter);
        return;
    };
    let Some(Some(Read::Dependency(read))) = domain.archive_reads.get_mut(Id::from_token(waiter)) else {
        unreachable!("read survives validation")
    };
    read.results.push(result).expect("one result per bounded ID");
    read.at = read.at.checked_add(1).expect("bounded result index");
    if let Some(&next) = read.ids.get(usize::try_from(read.at).expect("bounded index")) {
        request_load(domain, waiter, Range::Core(crate::CoreRange::TaskResult { task: next }), None, out);
        return;
    }
    let Some(Read::Dependency(read)) = take_read(domain, waiter) else { unreachable!("complete dependency read") };
    domain.archive_reads.retire(Id::from_token(waiter));
    assert!(
        domain.core.dependency_results.insert(read.task, read.results.into_boxed()).is_ok(),
        "one preparation result set"
    );
    domain.work.push(Work::StartBrief { task: read.task });
}

pub(crate) fn transcript_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    cut: Option<loads::Cut>,
    out: &mut Queue<Request>,
) {
    if cut.is_some() {
        transcript_failed(domain, waiter);
        return;
    }
    let Some(Some(Read::Transcript { task })) = domain.archive_reads.get(Id::from_token(waiter)) else { return };
    let task = *task;
    // A message can invalidate preparation while this store read is in
    // flight; its old answer belongs to no current preparation (jig's
    // `examples/ops/domain/src/restart.rs`).
    if !domain.core.transcripts.contains_key(&task) {
        retire_read(domain, waiter);
        return;
    }
    for row in rows {
        let turn = match row {
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) => turn,
            Record::Core(
                jig_domain_core::Record::Core(
                    jig_domain_core::CoreRecord::ProposalDecision(_)
                    | jig_domain_core::CoreRecord::Call(_)
                    | jig_domain_core::CoreRecord::EscalationDecision(_)
                    | jig_domain_core::CoreRecord::Deployment(_)
                    | jig_domain_core::CoreRecord::RunProof(_)
                    | jig_domain_core::CoreRecord::Terminal(_)
                    | jig_domain_core::CoreRecord::Projection(_),
                )
                | jig_domain_core::Record::Tasks(_)
                | jig_domain_core::Record::People(_)
                | jig_domain_core::Record::Notes(_),
            )
            | Record::Forge { .. } => {
                transcript_failed(domain, waiter);
                return;
            }
        };
        if !domain.core.append_transcript(task, turn) {
            transcript_failed(domain, waiter);
            return;
        }
    }
    if let Some(after) = next {
        let Some(Some(Read::Transcript { .. })) = domain.archive_reads.get(Id::from_token(waiter)) else {
            unreachable!("transcript read survives its page")
        };
        let from = match &after {
            Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Turn { attempt, turn, .. })) => {
                (*attempt, *turn)
            }
            Key::Core(_) | Key::Forge(_) => unreachable!("transcript pages end at a turn"),
        };
        request_load(domain, waiter, Range::Core(crate::CoreRange::TaskTranscript { task, from }), Some(after), out);
        return;
    }
    let Some(Read::Transcript { .. }) = take_read(domain, waiter) else { unreachable!("finished transcript read") };
    domain.archive_reads.retire(Id::from_token(waiter));
    // A window the store could not read whole is the core's to settle
    // (jig's `domain/engine.md`, 7.2).
    if !domain.core.transcript_readable(task) {
        domain.work.push(Work::Core(jig_domain_core::Event::TranscriptUnread { task }));
        return;
    }
    if let Some((waiter, first)) = begin_dependency_read(domain, task) {
        request_load(domain, waiter, Range::Core(crate::CoreRange::TaskResult { task: first }), None, out);
    }
}

pub(crate) fn take_read(domain: &mut Domain, waiter: Token) -> Option<Read> {
    let entry = domain.archive_reads.get_mut(Id::from_token(waiter))?;
    entry.take()
}
