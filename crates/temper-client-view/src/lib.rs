//! temper's composed client tree, with jig's pages and the forge's fragments.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod forge;
pub mod frame;
pub mod limits;
mod root;
pub mod words;
pub use limits::{Limits, worst_case};
pub use root::{View, act, build};
#[cfg(test)]
mod tests;
