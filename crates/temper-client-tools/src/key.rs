//! The engine key of a named tools request (tools.md, 7).
use crate::jig::key::{Name, named};
pub const TOOLS: &[u8] = b"temper-tools";
#[must_use]
pub fn engine(name: &Name) -> skein_lib::RequestKey {
    named(TOOLS, name)
}
