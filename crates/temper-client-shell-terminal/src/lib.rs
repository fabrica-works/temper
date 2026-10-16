//! temper's native client and terminal shell (client.md, section 2).
#![forbid(unsafe_code)]
pub mod client;
pub mod limits;
pub mod lines;
pub mod route;
pub mod terminal;
#[cfg(test)]
mod tests;
