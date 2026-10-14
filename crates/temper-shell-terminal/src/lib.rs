//! temper's native client and terminal shell (client.md, section 2).
#![forbid(unsafe_code)]
pub mod client;
pub mod command;
pub mod limits;
pub mod render;
pub mod route;
#[cfg(test)]
mod tests;
