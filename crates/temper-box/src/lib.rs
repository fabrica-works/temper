//! The box's steps, restarts and ordered shutdown (protocol/setup.md, 7).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod host;
pub mod limits;
pub mod lines;
pub mod machine;
pub mod pipe;
pub mod sink;
#[cfg(test)]
mod tests;
