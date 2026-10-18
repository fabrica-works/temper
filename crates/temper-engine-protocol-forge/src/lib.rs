//! Forge protocol component (protocol/forge.md, sections 2, 3, 4, 7 and 8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod actions;
pub mod boundary;
pub mod component;
mod connection;
mod contents;
pub mod conventions;
mod failure;
pub mod grow;
pub mod hooks;
pub mod limits;
pub mod marker;
mod plan;
pub mod reading;
mod reads;
pub mod settings;
pub mod start;
pub mod tables;
mod writes;
pub use boundary::{Answered, Event, Request, StartRead, Stopped};
pub use component::{Component, Diagnostics, MAX_OUT};
pub use limits::{Limits, Shared, Unusable};
pub use settings::Settings;
#[cfg(test)]
mod tests;
