//! Keep: the child/table routing shape and exhaustive owner variants.
//! Replace: Ops connector sides with the forge's side and temper's navigation.
//! The children keep documents, visibility and pending acts. The root retains
//! only those children and the shared client (client/domain/root.md, 3–10).
//! Each step routes one event; take reclaims after routing every tell.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod address;
pub mod boundary;
pub mod domain;
pub mod forge;
pub mod limits;
pub mod owner;
mod parts;
mod route;
pub use address::{Address, Place};
pub use boundary::{Action, Event, Fact, JigOut, Request};
pub use domain::{
    Domain, configure_navigation, fire, navigation, new, next_deadline, provider, shown, step, take, unlinked,
};
pub use limits::{Limits, max_out, worst_case};
#[cfg(test)]
mod tests;
