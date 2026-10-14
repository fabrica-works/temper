//! Temper's engine and worker protocol services.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod forge;
pub use forge::ForgeRuntime;
pub mod limits;
pub mod settings;
pub mod startup;
pub use startup::{Named, named, startup};
pub mod system;
pub use system::{Service, iterate};
pub mod store;
#[cfg(test)]
mod tests;
pub mod tokens;

pub mod accounts;
pub mod assignment;
pub mod clients;
pub mod family;
pub mod identities;
pub mod translate;
pub mod values;
mod worker;
mod worker_git;
pub mod worker_route;
pub mod worker_settings;
pub use worker::{Worker, WorkerRuntime};
pub use worker_settings::worker_worst_case;
pub mod workers;
pub use accounts::AccountsRuntime;
mod engine;
mod route;
mod stop;
mod tools;
/// How the service ended (protocol.md, 6).
#[derive(PartialEq, Eq, Debug)]
pub enum End {
    Drained,
    Aborted,
    Stopped(Stop),
}
/// Why the service stopped; failed startup names its configuration paths.
#[derive(PartialEq, Eq, Debug)]
pub enum Stop {
    CommitFailed { number: u64 },
    Start(Problems),
    Listener,
    Forge { number: u16, why: temper_engine_protocol_forge::Stopped },
}
use jig_protocol_config::Problems;
