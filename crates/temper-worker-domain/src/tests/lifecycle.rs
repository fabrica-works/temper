//! A hosted run's way through the root: its start, its agent's end, its
//! answer's items, and the worker's stop.

use super::fixtures::{ATTEMPT, RUN, assignment, assignment_of, grant, items, parked, turn};
use super::harness::{Harness, answered, operated};
use crate::{Event, Request, agent, host};
use alloc::boxed::Box;

/// The start smith's agent is sent, among `rows`.
fn sent_start(rows: &[Request]) -> &agent::Start {
    for row in rows {
        if let Request::Process(process) = row
            && let agent::process::Request::Send { message: agent::Down::Start { start, .. }, .. } = process.as_ref()
        {
            return start;
        }
    }
    panic!("smith's start is sent: {rows:?}")
}

#[test]
fn a_run_with_no_repository_starts_smith_with_no_workspace() {
    let mut h = Harness::new();
    let assign = Event::Assign { assignment: assignment(true, Box::new([])), items: Ok(items(&[])), save: None };
    let rows = h.run(assign);
    let start = sent_start(&rows);
    assert_eq!((start.workspace, start.directories.len()), (None, 0));
}

#[test]
fn only_the_llm_accounts_grants_reach_smith() {
    let mut h = Harness::new();
    let grants = Box::new([grant(host::GrantTable::Accounts, 1), grant(host::GrantTable::Application, 7)]);
    let assign = Event::Assign { assignment: assignment(true, grants), items: Ok(items(&[])), save: None };
    let rows = h.run(assign);
    let llm = grant(host::GrantTable::Accounts, 1);
    let expected = agent::Grant { account: llm.account, generation: llm.generation, valid: llm.valid };
    assert_eq!(&*sent_start(&rows).grants, [expected]);
}

#[test]
fn an_agent_that_fails_before_its_run_is_admitted_ends_a_cancelled_start() {
    let mut h = Harness::new();
    h.run(Event::Link(host::LinkDown::Assign { assignment: assignment(false, Box::new([])) }));
    h.run(Event::Link(host::LinkDown::Cancel { run: RUN, attempt: ATTEMPT }));
    let rows = h.gone();
    let cancelled = host::Failure::Cancelled(host::Reason::Engine);
    let ending =
        host::Ending::Failed { failure: cancelled, detail: Box::new([]), work: host::Work { left: None, saved: None } };
    assert!(answered(&rows, &ending), "{rows:?}");
}

#[test]
fn an_agents_end_waits_behind_its_hand_outs() {
    let mut h = Harness::new();
    h.started();
    h.hold = true;
    h.say(parked(0));
    let rows = h.gone();
    let owner = h.owner.expect("a hosted run");
    let hosting = h.domain.host().hosting(owner).expect("a hosted run");
    assert_eq!(hosting.phase, host::Phase::Active, "{rows:?}");
    let ending = host::Ending::Parked { work: host::Work { left: None, saved: None } };
    assert!(answered(&h.release(), &ending));
}

#[test]
fn a_worker_shutting_down_waits_for_the_engine_to_have_its_turns() {
    let mut h = Harness::new();
    h.started();
    h.say(agent::Up::Turn { turn: turn(1) });
    h.say(parked(1));
    h.gone();
    h.run(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }));
    assert!(!h.run(Event::Shutdown).contains(&Request::Stop));
    let rows = h.run(Event::Link(host::LinkDown::TurnKept { run: RUN, attempt: ATTEMPT, turn: 1 }));
    assert!(rows.contains(&Request::Stop));
}

#[test]
fn assignments_the_hub_refuses_leave_room_for_one_in_the_same_iteration() {
    let mut h = Harness::new();
    for run in 1..=4 {
        let rows = h.unreclaimed(assignment_of(run, &[b"app"], 3));
        let [Request::Answer { answer, .. }] = &*rows else { panic!("one answer: {rows:?}") };
        assert_eq!(answer.ending, host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Messages)));
    }
    assert!(operated(&h.unreclaimed(assignment_of(5, &[b"app"], 0))), "the fifth is prepared");
}

#[test]
fn acknowledged_answers_free_their_workspace_items() {
    let mut h = Harness::new();
    let unprepared = host::Failure::Unprepared(host::Preparation::Transient);
    let ending = host::Ending::Failed {
        failure: unprepared,
        detail: Box::new([]),
        work: host::Work { left: None, saved: None },
    };
    for run in 1..=4 {
        assert!(answered(&h.run(assignment_of(run, &[b"app"], 0)), &ending), "run {run} is hosted, not refused");
        h.run(Event::Link(host::LinkDown::Acknowledge { run: skein_lib::Token::new(run), attempt: ATTEMPT }));
    }
}
