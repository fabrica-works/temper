//! One-shot command host over the tools' root (client/tools.md, 4).
#![forbid(unsafe_code)]
pub mod arguments;
mod directory;
pub mod host;
pub mod limits;
mod output;
#[cfg(test)]
mod tests;
