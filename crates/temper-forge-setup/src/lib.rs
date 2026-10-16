//! Forge setup, checked before any command acts (setup.md, 8 and 11).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod check;
pub mod ini;
pub mod limits;
pub mod place;
pub mod prepare;
pub mod problem;
pub mod reconcile;
pub mod runner;
pub mod setup;
pub mod spec;
pub mod state;
pub mod step;
pub use setup::Setup;
pub use step::{Step, steps};
#[cfg(test)]
mod tests;
