//! The standalone stack on one checked real loop (domain/testing.md, 2.4).
#![forbid(unsafe_code)]

pub mod fixtures;
pub mod llm;
pub mod scripts;

/// A real deadline bounds every story, including startup and drain.
pub const PATIENCE: skein_lib::Duration = skein_lib::Duration::from_secs(10);
