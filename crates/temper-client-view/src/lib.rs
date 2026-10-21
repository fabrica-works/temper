//! Temper's own pages on skein-ui and jig-client's documents.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod accounts;
pub mod cards;
mod choice;
mod confirm;
pub mod forge;
pub mod frame;
pub mod limits;
pub mod look;
pub mod pages;
mod root;
pub mod words;
pub use limits::{Limits, worst_case};
pub use root::{View, act, build, event};
#[cfg(test)]
mod tests;
