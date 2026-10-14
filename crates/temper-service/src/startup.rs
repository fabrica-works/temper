//! Strict startup diagnostics and the application's configuration.
#[expect(dead_code, reason = "10.1b connects the application reader to startup")]
pub(crate) mod application;
mod problems;
pub use problems::{Problem, Problems, What, lines};
