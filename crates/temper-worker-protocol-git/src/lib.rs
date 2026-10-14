//! The checkout's fixed git invocations (git.md).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod boundary;
pub mod invocation;
pub mod limits;
pub mod output;
#[cfg(test)]
mod tests;

pub use boundary::{Event, Identity, Outputs, Request, Settings, Unusable, Version};
pub use limits::{Limits, MAX_OUT, MaxOut, Unfit, check, worst_case};
