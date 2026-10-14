//! Forgejo v16.0.5's documents (forge.md, sections 2, 3 and 9).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod client;
pub mod decode;
pub mod document;
pub mod id;
pub mod limits;
pub mod path;
pub mod request;
pub mod server;
#[cfg(test)]
mod tests;
pub mod time;
