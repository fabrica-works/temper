//! Temper's engine and worker protocol services.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub mod forge;
pub use forge::ForgeRuntime;
pub mod limits;
pub mod settings;
pub mod startup;
pub use startup::{Named, StoreLimits, named, startup, store_limits};
pub use temper_protocol_records::NotConnected;
pub mod system;
pub use system::{Service, iterate};
pub mod store;
#[cfg(test)]
mod tests;
pub mod tokens;

pub mod accounts;
mod d2;
pub use d2::{D2, D2Preseed, d2_preseed};
pub mod assignment;
pub mod clients;
pub mod family;
pub mod identities;
pub mod program_sources;
pub mod server;
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
mod setup;
mod stop;
mod tools;
/// How the service ended (protocol/README.md, 6).
#[derive(PartialEq, Eq, Debug)]
pub enum End {
    Drained,
    Aborted,
    Stopped(Stop),
}
/// Why the service stopped; failed startup names its configuration paths.
#[derive(PartialEq, Eq, Debug)]
pub enum Stop {
    CommitFailed {
        number: u64,
    },
    Start(Problems),
    /// The store uses another format version (jig's protocol/store.md, 5.3).
    StoreVersion,
    Listener,
    /// The webhooks listener failed.
    Hooks,
}
use jig_protocol_config::Problems;

/// Operator diagnostics taken by the shell (protocol/forge.md, 2.3).
#[derive(PartialEq, Eq, Debug)]
pub enum Note {
    ForgeLacking { number: u16, names: Box<[u8]> },
    ForgeStopped { number: u16, why: skein_forge_connection::Stopped },
    ForgeNotConnected { number: u16, why: NotConnected },
    ForgeUnconfigured { number: u16 },
    ForgeUnkept { number: u16, installation: u64 },
}
