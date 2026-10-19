//! Agent tools as bounded values (tools.md, 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod jig;
pub mod key;
pub mod limits;
#[cfg(test)]
mod tests;
