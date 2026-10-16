//! The box's steps, restarts and ordered shutdown (setup.md, 7).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod limits;
pub mod machine;
#[cfg(test)]
mod tests;
