//! Temper's engine root composes jig's core and the forge connector.
//! Routing, translation, assembly and the core-selected restart adapters follow
//! jig's domain/root.md; skein-lib's journal is the only output path.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod admission;
mod archive;
mod assemble;
mod boundary;
mod decision;
mod domain;
mod limits;
mod link;
pub mod loads;
mod restart;
mod route;
mod stopping;
mod store;
#[cfg(test)]
mod tests;
mod translate;
pub use decision::{
    Counters, Decision, Delivery, Journal, JournalOutput, Limits as JournalLimits, accept, accept_pending, commit,
    committed, fresh, journal_limits, resume, room, takes, uncommitted, worst_case as journal_worst_case,
};
pub use store::{
    CallAnswer, CallKey, CallRecord, CoreRange, Deployment, EscalationDecisionRecord, Family, Key,
    ProposalDecisionRecord, Range, Record, RunProof, TerminalRecord, TurnProof, TurnRecord, Write, record_bytes,
};

pub use admission::try_step;
pub use boundary::Request as Output;
pub use boundary::*;
pub use domain::{Domain, fire, max_out, release, step};
pub use jig_domain_core::{Charter, Delegate, Dependency, Model, ProcedureAction, RunCharter, RunPolicy};
pub use limits::{BriefBudgets, BriefLimits, Limits, worst_case};
pub use route::{Approval, Freshness, Gate, HostDelivery, HostRequest, LandingRule, adopt_repository_ask};
pub use translate::{Config, LandingPolicy};
