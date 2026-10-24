//! Forge setup, checked before any command acts (protocol/setup.md, 8 and 11).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod check;
pub mod fetch;
pub mod given;
pub mod ini;
pub mod limits;
pub mod name;
pub mod place;
pub mod prepare;
pub mod problem;
pub mod run;
pub mod runner;
pub mod setup;
pub mod sign_in;
pub mod spec;
pub mod state;
pub mod step;
pub mod upgrade;
pub use setup::Setup;
pub use step::{Group, Kind, Stage, Step, Steps, steps};
#[cfg(test)]
mod tests;

/// The first repository's starter CI (protocol/setup.md, 6.7).
pub const STARTER_PATH: &[u8] = b".forgejo/workflows/ci.yml";
pub const STARTER_WORKFLOW: &[u8] = b"on:\n  push:\n    branches: [main]\n  pull_request:\njobs:\n  check:\n    runs-on: temper-host\n    steps:\n      - run: echo \"temper's starter check passed; replace this step with your own checks\"\n";
