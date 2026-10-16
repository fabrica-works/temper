//! Forge setup, checked before any command acts (setup.md, 8 and 11).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod check;
pub mod limits;
pub mod problem;
pub mod setup;
pub mod step;
pub use setup::Setup;
pub use step::{Step, steps};
#[cfg(test)]
mod tests;
