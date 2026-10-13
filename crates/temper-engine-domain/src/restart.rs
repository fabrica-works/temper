//! Store adapters for the core-selected restart script (jig's domain/root.md, 8).
use crate::boundary::{DependencyRead, EndedRead, Read, Request, Work};
use crate::domain::{Domain, close, emit};
use crate::limits::{Limits, core_limits, environment_core, route_decision};
use crate::route::{escalation, route_core_requests, route_into};
use crate::{Decision, Delivery, Key, Range, Record, loads};
use alloc::boxed::Box;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Id, List, Queue, ReplyTo, Token};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_client as forge_client;

/// Store transport terminals are read-only; cold rows reserve their full restore route.
pub(crate) fn load_decision(domain: &mut Domain, limits: &Limits) -> Option<Decision> {
    if domain.startup_range.is_some() {
        route_decision(domain, limits)
    } else {
        Decision::reserve_room(&mut domain.journal, &limits.journal, skein_lib::JournalRoom { writes: 0, held: 0 })
    }
}

pub(crate) fn accept_load(domain: &mut Domain, env: &Env<Limits>, decision: Option<Decision>) {
    if let Some(decision) = decision {
        crate::decision::accept_routed(
            &mut domain.journal,
            &mut domain.core.counters,
            &env.limits.journal,
            decision,
            false,
        )
        .expect("read-only store terminal was admitted before changing its owner");
    }
}

pub(crate) fn request_load(
    domain: &mut Domain,
    waiter: Token,
    range: Range,
    after: Option<Key>,
    out: &mut Queue<Request>,
) {
    let mut load_out = Queue::with_capacity(1);
    let most = match &range {
        Range::Core(
            crate::CoreRange::Deployment
            | crate::CoreRange::TaskResult { .. }
            | crate::CoreRange::EscalationDecision { .. }
            | crate::CoreRange::ProposalDecision { .. }
            | crate::CoreRange::Notes(jig_domain_notes::Range::Entry { .. }),
        ) => 1,
        Range::Core(
            crate::CoreRange::Calls
            | crate::CoreRange::Tasks
            | crate::CoreRange::People
            | crate::CoreRange::RunProofs
            | crate::CoreRange::Turns { .. }
            | crate::CoreRange::TaskTranscript { .. },
        )
        | Range::Census(_)
        | Range::Forge => domain.limits.loads.rows,
        Range::Core(crate::CoreRange::Notes(jig_domain_notes::Range::Lines { .. })) => {
            domain.limits.loads.rows.min(domain.limits.notes.load_rows)
        }
    };
    if loads::begin(&mut domain.loads, waiter, range.clone(), after, most, &mut load_out).is_none() {
        match range {
            Range::Core(crate::CoreRange::TaskTranscript { .. }) => {
                transcript_failed(domain, waiter);
                return;
            }
            Range::Core(
                crate::CoreRange::ProposalDecision { .. }
                | crate::CoreRange::EscalationDecision { .. }
                | crate::CoreRange::Calls
                | crate::CoreRange::Deployment
                | crate::CoreRange::Tasks
                | crate::CoreRange::RunProofs
                | crate::CoreRange::People
                | crate::CoreRange::TaskResult { .. }
                | crate::CoreRange::Turns { .. },
            )
            | Range::Census(_)
            | Range::Forge => {}
            Range::Core(crate::CoreRange::Notes(_)) => {
                note_failed(domain, waiter);
                return;
            }
        }
        match range {
            Range::Core(crate::CoreRange::ProposalDecision { .. }) => {
                domain.work.push(Work::ProposalFailed { waiter });
                return;
            }
            Range::Core(crate::CoreRange::EscalationDecision { .. }) => {
                domain.work.push(Work::EscalationFailed { waiter });
                return;
            }
            Range::Core(
                crate::CoreRange::Calls
                | crate::CoreRange::Deployment
                | crate::CoreRange::Tasks
                | crate::CoreRange::People
                | crate::CoreRange::RunProofs
                | crate::CoreRange::Turns { .. }
                | crate::CoreRange::TaskTranscript { .. }
                | crate::CoreRange::TaskResult { .. },
            )
            | Range::Census(_)
            | Range::Forge => {
                unreachable!("startup/result load room reserved")
            }
            Range::Core(crate::CoreRange::Notes(_)) => unreachable!("notes load failure handled above"),
        }
    }
    match load_out.pop().expect("load issued") {
        loads::Request::Load { owner, range, after, most, bytes } => {
            out.push(Request::Store(crate::StoreRequest::Load { owner, range, after, most, bytes }));
        }
        loads::Request::Loaded { .. } | loads::Request::Unloaded { .. } => unreachable!("begin only issues IO"),
    }
}

/// What a live load waiter reads, if it still reads anything.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Waiting {
    Escalation,
    Proposal,
    Ended,
    Transcript,
    Dependency,
    Input,
    Notes,
}

fn waiting(domain: &Domain, waiter: Token) -> Option<Waiting> {
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

pub(crate) fn load_outputs(
    domain: &mut Domain,
    env: &Env<Limits>,
    load_out: &mut Queue<loads::Request>,
    decision: &mut Option<Decision>,
    out: &mut Queue<Request>,
) {
    for _ in 0..load_out.len() {
        match load_out.pop().expect("load terminal count") {
            loads::Request::Loaded { waiter, rows, next, cut } => {
                if waiter == Token::new(u64::MAX) {
                    if cut.is_some() {
                        let _refused = domain.core.restart_refuse();
                        out.push(Request::Stop);
                    } else {
                        startup_page(domain, env, rows, next, decision.take().expect("admitted restore page"), out);
                    }
                    return;
                }
                match waiting(domain, waiter) {
                    Some(Waiting::Notes) => note_loaded(domain, waiter, rows, next, cut),
                    Some(Waiting::Input) if cut.is_none() => input_loaded(domain, waiter, rows, next, out),
                    Some(Waiting::Input) => input_failed(domain, waiter),
                    Some(Waiting::Dependency) if cut.is_none() => dependency_loaded(domain, waiter, rows, next, out),
                    Some(Waiting::Dependency) => dependency_failed(domain, waiter),
                    Some(Waiting::Transcript) => transcript_loaded(domain, waiter, rows, next, cut, out),
                    Some(Waiting::Ended) if cut.is_none() && next.is_none() => {
                        domain.work.push(Work::EndedLoaded { waiter, rows });
                    }
                    Some(Waiting::Ended) => domain.work.push(Work::EndedFailed { waiter }),
                    Some(Waiting::Proposal) if cut.is_none() && next.is_none() => {
                        domain.work.push(Work::ProposalLoaded { waiter, rows });
                    }
                    Some(Waiting::Proposal) => domain.work.push(Work::ProposalFailed { waiter }),
                    Some(Waiting::Escalation) if cut.is_none() => {
                        domain.work.push(Work::EscalationLoaded { waiter, rows });
                    }
                    Some(Waiting::Escalation) => domain.work.push(Work::EscalationFailed { waiter }),
                    None => {}
                }
            }
            loads::Request::Unloaded { waiter, .. } => {
                if waiter == Token::new(u64::MAX) {
                    let _refused = domain.core.restart_refuse();
                    out.push(Request::Stop);
                    return;
                }
                match waiting(domain, waiter) {
                    Some(Waiting::Notes) => note_failed(domain, waiter),
                    Some(Waiting::Input) => input_failed(domain, waiter),
                    Some(Waiting::Dependency) => dependency_failed(domain, waiter),
                    Some(Waiting::Transcript) => transcript_failed(domain, waiter),
                    Some(Waiting::Ended) => domain.work.push(Work::EndedFailed { waiter }),
                    Some(Waiting::Proposal) => domain.work.push(Work::ProposalFailed { waiter }),
                    Some(Waiting::Escalation) => domain.work.push(Work::EscalationFailed { waiter }),
                    None => {}
                }
            }
            loads::Request::Load { .. } => unreachable!("terminal methods never issue IO"),
        }
    }
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
    domain.work.push(Work::Core(jig_domain_core::Event::PreparationFailed { task }));
}

pub(crate) fn begin_dependency_read(domain: &mut Domain, task: u64) -> Option<(Token, u64)> {
    let ids = domain.core.preparation_ids(task).expect("preparing task context");
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
    domain.work.push(Work::Core(jig_domain_core::Event::PreparationFailed { task: read.task }));
}

pub(crate) fn dependency_loaded(
    domain: &mut Domain,
    waiter: Token,
    rows: Box<[Record]>,
    next: Option<Key>,
    out: &mut Queue<Request>,
) {
    let Some(Some(Read::Dependency(read))) = domain.archive_reads.get(Id::from_token(waiter)) else { return };
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

pub(crate) fn startup_page(
    domain: &mut Domain,
    env: &Env<Limits>,
    rows: Box<[Record]>,
    next: Option<Key>,
    mut decision: Decision,
    out: &mut Queue<Request>,
) {
    let Some(range) = domain.startup_range.clone() else { return };
    if let Range::Census(_) = range {
        census_page(domain, env, rows, next, decision, out);
        return;
    }
    for row in rows {
        restore_page_row(domain, env, row);
    }
    route_into(domain, env, &mut decision);
    if domain.core.restart_failure().is_some() {
        out.push(Request::Stop);
        return;
    }
    if range == Range::Core(crate::CoreRange::Deployment) {
        domain.core_header_loaded = true;
        close(domain, env, decision, out);
        let following = match next {
            Some(after) => (range, Some(after)),
            None => (Range::Core(crate::CoreRange::People), None),
        };
        domain.startup_range = Some(following.0.clone());
        request_load(domain, Token::new(u64::MAX), following.0, following.1, out);
        return;
    }
    if let Some(after) = next {
        emit(&mut decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: Some(after) });
        close(domain, env, decision, out);
        return;
    }
    let following = match range {
        Range::Core(crate::CoreRange::Deployment) => Some(Range::Core(crate::CoreRange::People)),
        Range::Core(crate::CoreRange::People) => Some(Range::Core(crate::CoreRange::Tasks)),
        Range::Core(crate::CoreRange::Tasks) => Some(Range::Core(crate::CoreRange::RunProofs)),
        Range::Forge | Range::Census(_) | Range::Core(crate::CoreRange::Calls) => None,
        Range::Core(crate::CoreRange::RunProofs) => Some(Range::Core(crate::CoreRange::Calls)),
        Range::Core(
            crate::CoreRange::EscalationDecision { .. }
            | crate::CoreRange::ProposalDecision { .. }
            | crate::CoreRange::Turns { .. }
            | crate::CoreRange::TaskTranscript { .. }
            | crate::CoreRange::TaskResult { .. },
        ) => {
            unreachable!("startup range")
        }
        Range::Core(crate::CoreRange::Notes(_)) => unreachable!("notes load is outside startup"),
    };
    if range == Range::Core(crate::CoreRange::Tasks) {
        // People's restoration terminal; at a first start the core seeds the
        // configured projects and owners (jig's `protocol/configuration.md`, 3).
        domain.work.push(Work::People(people::Event::Restored));
        route_into(domain, env, &mut decision);
        if domain.core.restart_failure().is_some() {
            out.push(Request::Stop);
            return;
        }
    }
    if let Some(range) = following {
        domain.startup_range = Some(range.clone());
        emit(&mut decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: None });
        close(domain, env, decision, out);
        return;
    }
    domain.startup_range = None;
    let request = if range == Range::Forge {
        // The connector confirms that its restored rows have been reconciled.
        domain.work.push(Work::Forge(forge::Event::Restored { clock: forge_client::RecoveryClock::Wall }));
        jig_domain_core::RestartRequest::Idle
    } else {
        domain.core.restart_done(jig_domain_core::RestartStep::LoadCore)
    };
    domain.work.push(Work::Restart(request));
    route_into(domain, env, &mut decision);
    close(domain, env, decision, out);
}

/// Perform only the step requested by the core; page cursors are transport
/// state, while restart order and admission belong to jig-core.
pub(crate) fn restart_request(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    request: jig_domain_core::RestartRequest,
) {
    match request {
        jig_domain_core::RestartRequest::Idle => {}
        jig_domain_core::RestartRequest::Refused { .. } => domain.stop_pending = true,
        jig_domain_core::RestartRequest::Step(step) => match step {
            jig_domain_core::RestartStep::LoadCore => {
                // Every configured account is wanted from the restart's first
                // step (jig's `domain/engine.md`, 12).
                let routed = jig_domain_core::want_accounts(&mut domain.core, &environment_core(env));
                route_core_requests(domain, env, decision, routed);
                domain.startup_range = Some(Range::Core(crate::CoreRange::Deployment));
                emit(
                    decision,
                    &env.limits,
                    Delivery::Load {
                        waiter: Token::new(u64::MAX),
                        range: Range::Core(crate::CoreRange::Deployment),
                        after: None,
                    },
                );
            }
            jig_domain_core::RestartStep::LoadInbox => {
                let census = domain.core.census(&core_limits(&env.limits));
                let range = Range::Census(census);
                domain.startup_range = Some(range.clone());
                emit(decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: None });
            }
            jig_domain_core::RestartStep::RestoreConnector { connector } => {
                assert_eq!(connector, domain.config.forge_connector, "configured forge connector");
                domain.startup_range = Some(Range::Forge);
                emit(
                    decision,
                    &env.limits,
                    Delivery::Load { waiter: Token::new(u64::MAX), range: Range::Forge, after: None },
                );
            }
            jig_domain_core::RestartStep::AdoptRuns => {
                domain.work.push(Work::Tasks(tasks::Event::Restored));
                // Finish task restoration before draining every emitted claim.
                domain.work.push(Work::AdoptRestored);
            }
            jig_domain_core::RestartStep::ReadAfresh { connector } => {
                assert_eq!(connector, domain.config.forge_connector, "configured forge connector");
                domain.work.push(Work::Forge(forge::Event::ReadAfresh));
            }
            jig_domain_core::RestartStep::SettleOutbox { connector } => {
                assert_eq!(connector, domain.config.forge_connector, "configured forge connector");
                domain.work.push(Work::Forge(forge::Event::SettleOutbox));
            }
            jig_domain_core::RestartStep::Open => {
                let request = domain.core.restart_done(step);
                assert_eq!(request, jig_domain_core::RestartRequest::Idle, "decisions open once");
                for _ in 0..domain.limits.tasks.tasks {
                    let Some(context) = domain.core.take_due() else { break };
                    domain.work.push(Work::Activate(context));
                }
            }
        },
    }
}

pub(crate) fn connector_restart_done(domain: &mut Domain, stage: jig_domain_core::connector::RestartStage) {
    let connector = domain.config.forge_connector;
    let step = match stage {
        jig_domain_core::connector::RestartStage::Restored => {
            jig_domain_core::RestartStep::RestoreConnector { connector }
        }
        jig_domain_core::connector::RestartStage::ReadAfresh => jig_domain_core::RestartStep::ReadAfresh { connector },
        jig_domain_core::connector::RestartStage::Settled => jig_domain_core::RestartStep::SettleOutbox { connector },
    };
    let request = domain.core.restart_done(step);
    domain.work.push(Work::Restart(request));
}

pub(crate) fn connector_restarting(domain: &Domain) -> bool {
    match domain.core.restart_step() {
        Some(
            jig_domain_core::RestartStep::RestoreConnector { .. }
            | jig_domain_core::RestartStep::ReadAfresh { .. }
            | jig_domain_core::RestartStep::SettleOutbox { .. },
        ) => true,
        Some(
            jig_domain_core::RestartStep::LoadCore
            | jig_domain_core::RestartStep::LoadInbox
            | jig_domain_core::RestartStep::AdoptRuns
            | jig_domain_core::RestartStep::Open,
        )
        | None => false,
    }
}

pub(crate) fn take_read(domain: &mut Domain, waiter: Token) -> Option<Read> {
    let entry = domain.archive_reads.get_mut(Id::from_token(waiter))?;
    entry.take()
}

/// Reject unsupported root shapes and identities above durable high-water marks before child
/// restoration. Proof rows consume exact transient live-row correlations and never load archive
/// history into the live map.
pub(crate) fn restore_page_row(domain: &mut Domain, env: &Env<Limits>, row: Record) {
    match row {
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(record))) => {
            if domain.core.restore_core(jig_domain_core::CoreRecord::Call(record), &core_limits(&env.limits))
                != jig_domain_core::Restored::Live
            {
                let _refused = domain.core.restart_refuse();
            }
        }
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(deployment))) => {
            if !domain.forge.bind_deployment(deployment.id, &env.limits.forge) {
                let _refused = domain.core.restart_refuse();
                return;
            }
            match domain
                .core
                .restore_core(jig_domain_core::CoreRecord::Deployment(deployment), &core_limits(&env.limits))
            {
                jig_domain_core::Restored::Deployment { commits } => {
                    domain.core_header_loaded = true;
                    domain.restored_commits = Some(commits);
                }
                jig_domain_core::Restored::Live
                | jig_domain_core::Restored::Archive
                | jig_domain_core::Restored::Rejected => {
                    unreachable!("deployment restore result")
                }
            }
        }
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Projection(row))) => {
            if domain.core.restore_core(jig_domain_core::CoreRecord::Projection(row), &core_limits(&env.limits))
                == jig_domain_core::Restored::Rejected
            {
                let _refused = domain.core.restart_refuse();
            }
        }
        Record::Core(jig_domain_core::Record::People(people::Stored::Policy { project, value })) => {
            if !domain.core.restore_policy(&core_limits(&domain.limits), project, value) {
                let _refused = domain.core.restart_refuse();
            }
        }
        Record::Core(jig_domain_core::Record::People(record)) => {
            domain.work.push(Work::People(people::Event::Restore { record }));
        }
        Record::Forge { row } => {
            domain.work.push(Work::Forge(forge::Event::Restore { record: *row }));
        }
        Record::Core(jig_domain_core::Record::Tasks(record)) => {
            let supported = match &record {
                tasks::Stored::Live(task) => escalation::supported(domain, task),
                tasks::Stored::Update(_)
                | tasks::Stored::Result(_)
                | tasks::Stored::ConversationNewest(_)
                | tasks::Stored::ConversationOldest(_)
                | tasks::Stored::Goal(_)
                | tasks::Stored::EndedDelegate(_)
                | tasks::Stored::Delegate(_)
                | tasks::Stored::Activity(_)
                | tasks::Stored::Milestone(_)
                | tasks::Stored::PersonProposal(_)
                | tasks::Stored::DecidedProposal(_)
                | tasks::Stored::ClosedLedger(_)
                | tasks::Stored::History(_)
                | tasks::Stored::Ended(_)
                | tasks::Stored::Stub(_)
                | tasks::Stored::Ledger(_)
                | tasks::Stored::Writer(_)
                | tasks::Stored::Pool(_) => true,
            };
            if !supported || !domain.core.restore_task_row(&record) {
                let _refused = domain.core.restart_refuse();
                return;
            }
            domain.work.push(Work::Tasks(tasks::Event::Restore { record }));
        }
        Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof))) => {
            if domain.core.restore_core(jig_domain_core::CoreRecord::RunProof(proof), &core_limits(&env.limits))
                != jig_domain_core::Restored::Live
            {
                let _refused = domain.core.restart_refuse();
            }
        }
        Record::Core(jig_domain_core::Record::Notes(_)) => unreachable!("notes are loaded on demand, outside startup"),
        Record::Core(jig_domain_core::Record::Core(
            jig_domain_core::CoreRecord::Turn(_)
            | jig_domain_core::CoreRecord::Terminal(_)
            | jig_domain_core::CoreRecord::EscalationDecision(_)
            | jig_domain_core::CoreRecord::ProposalDecision(_),
        )) => {
            unreachable!("startup excludes archive families")
        }
    }
}

/// Rebuild the derived inbox from the census's ended owners, after people's
/// read positions restored (jig's `protocol/store.md`, 7.2).
fn census_page(
    domain: &mut Domain,
    env: &Env<Limits>,
    rows: Box<[Record]>,
    next: Option<Key>,
    mut decision: Decision,
    out: &mut Queue<Request>,
) {
    for row in rows {
        let restored = match row {
            Record::Core(jig_domain_core::Record::Tasks(row)) => {
                domain.core.restore_inbox_row(&core_limits(&env.limits), &row)
            }
            Record::Core(
                jig_domain_core::Record::Core(_)
                | jig_domain_core::Record::People(_)
                | jig_domain_core::Record::Notes(_),
            )
            | Record::Forge { .. } => false,
        };
        if !restored {
            let _refused = domain.core.restart_refuse();
            out.push(Request::Stop);
            return;
        }
    }
    let range = domain.startup_range.clone().expect("census range in flight");
    if let Some(after) = next {
        emit(&mut decision, &env.limits, Delivery::Load { waiter: Token::new(u64::MAX), range, after: Some(after) });
        close(domain, env, decision, out);
        return;
    }
    domain.startup_range = None;
    let request = domain.core.restart_done(jig_domain_core::RestartStep::LoadInbox);
    domain.work.push(Work::Restart(request));
    route_into(domain, env, &mut decision);
    close(domain, env, decision, out);
}
