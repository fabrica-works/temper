//! Git's fixed invocations, checked before a hosted program touches files.
#![forbid(unsafe_code)]
mod check;
pub mod fixtures;
pub mod id;
pub use check::{Checked, Refusal, check};
