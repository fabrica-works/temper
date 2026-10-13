//! Worker-root stories at the protocol's boundary (jig's `domain/hosts.md`,
//! sections 4, 6.3 and 6.8; `domain/worker.md`, 4.1 and 4.3). Stories that
//! need smith's typed start run in the worker's world.

mod fixtures;

use crate::{Event, Invalid, Limits, Request, agent, host, translate, worst_case};
use alloc::boxed::Box;
use fixtures::{Harness, assignment, fields, grant, items, limits, refused};
use skein_lib::Duration;

#[test]
fn host_and_smith_turn_windows_must_fit_the_declared_reserve() {
    let fixture = limits();
    let turns = Limits { agent: agent::Limits { turns: 3, ..fixture.agent }, ..limits() };
    assert!(worst_case(&turns).is_none());
    let bytes = Limits { agent: agent::Limits { unacknowledged_bytes: 129, ..fixture.agent }, ..limits() };
    assert!(worst_case(&bytes).is_none());
}

#[test]
fn a_repository_name_longer_than_smith_takes_a_path_does_not_fit() {
    let fixture = limits();
    let limits = Limits { agent: agent::Limits { path_bytes: 31, ..fixture.agent }, ..limits() };
    assert!(worst_case(&limits).is_none());
}

#[test]
fn a_repository_named_twice_is_refused_as_an_invalid_start_naming_it() {
    let mut h = Harness::new();
    let assign =
        Event::Assign { assignment: assignment(true, Box::new([])), items: items(&[b"app", b"app"]), save: None };
    assert_eq!(&*h.step(assign), [refused(b"repository named twice")]);
    assert_eq!(h.domain.host().hosted(), 0);
}

#[test]
fn workspace_items_without_a_workstream_are_refused_as_an_invalid_start() {
    let mut h = Harness::new();
    let assign = Event::Assign { assignment: assignment(false, Box::new([])), items: items(&[b"app"]), save: None };
    assert_eq!(&*h.step(assign), [refused(b"workstream")]);
}

#[test]
fn a_run_with_no_repository_is_decoded_with_no_workspace() {
    let mut h = Harness::new();
    let (_, start) = h.decode(Box::new([]));
    assert_eq!(start.workspace, None);
    assert_eq!(h.domain.workspaces(), 0);
}

#[test]
fn git_identities_stay_with_the_hub_and_only_llm_grants_reach_smith() {
    let mut h = Harness::new();
    let llm = grant(host::GrantTable::Accounts, 1);
    let git = grant(host::GrantTable::Application, 7);
    let (_, start) = h.decode(Box::new([llm, git]));
    assert_eq!(&*start.grants, [llm]);
}

#[test]
fn an_undecodable_call_name_refuses_the_start_as_invalid() {
    let mut h = Harness::new();
    let (agent, _) = h.decode(Box::new([]));
    let rows = h.step(Event::Decoded { agent, start: Err(Invalid::CallName) });
    assert!(rows.contains(&refused(b"call name")));
    assert_eq!(h.domain.host().hosted(), 0, "a refused start frees its slot unacknowledged");
}

#[test]
fn an_assignment_hosted_already_is_a_wire_retry_and_dropped() {
    let mut h = Harness::new();
    h.decode(Box::new([]));
    let again = Event::Link(host::LinkDown::Assign { assignment: assignment(false, Box::new([])) });
    assert!(h.step(again).is_empty());
}

#[test]
fn a_worker_shutting_down_refuses_assignments_as_busy() {
    let mut h = Harness::new();
    h.step(Event::Shutdown);
    let rows = h.step(Event::Link(host::LinkDown::Assign { assignment: assignment(false, Box::new([])) }));
    let [Request::Answer { answer, .. }] = &*rows else { panic!("one answer: {rows:?}") };
    assert_eq!(answer.ending, host::Ending::Refused(host::Refusal::Busy));
}

#[test]
fn a_worker_shutting_down_still_says_its_slots() {
    let mut h = Harness::new();
    h.step(Event::Shutdown);
    let rows = h.step(Event::Ready);
    let [Request::Hello(hello)] = &*rows else { panic!("one hello: {rows:?}") };
    assert_eq!((hello.slots, hello.stop_bound), (2, Duration::from_secs(20)));
}

#[test]
fn a_worker_told_to_shut_down_with_nothing_left_says_stop_once() {
    let mut h = Harness::new();
    assert_eq!(&*h.step(Event::Shutdown), [Request::Stop]);
    assert!(!h.domain.is_ready());
    assert!(h.step(Event::Lost).is_empty());
}

#[test]
fn an_exhausted_account_stays_a_failure_of_its_own() {
    let exhausted = agent::ModelFault::Exhausted { account: 1, retry_after_ms: Some(5) };
    assert_eq!(translate::run_failure(&agent::RunFailure::Model(exhausted)), host::RunFailure::Exhausted);
}

#[test]
fn a_delivery_is_exactly_a_title_and_a_body() {
    let both = fields(&[(b"title", b"a title"), (b"body", b"a body")]);
    let delivered = crate::smith_delivery::fields(both, 512).expect("a commit message");
    assert_eq!((&*delivered.title, &*delivered.body), (&b"a title"[..], &b"a body"[..]));
}

#[test]
fn a_delivery_with_a_field_beyond_its_title_and_body_is_refused() {
    let extra = fields(&[(b"title", b"t"), (b"body", b"b"), (b"tag", b"x")]);
    assert!(crate::smith_delivery::fields(extra, 512).is_none());
}

#[test]
fn a_delivery_with_an_empty_title_is_refused() {
    let empty = fields(&[(b"title", b""), (b"body", b"b")]);
    assert!(crate::smith_delivery::fields(empty, 512).is_none());
}
