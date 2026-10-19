//! Bounds shared by parsing, running and rendering.
use skein_lib::Duration;
/// The generic tools' bounds, from the host's preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub rows: u32,
    pub ask_bytes: u64,
    pub text: u32,
    pub part: u32,
    pub wait: Duration,
    pub wait_most: Duration,
}
