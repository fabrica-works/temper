//! Restart and shutdown bounds (protocol/configuration.md, 7).
use skein_lib::{Duration, Queue, Time};
/// The box limits, durations in milliseconds at configuration boundaries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub backoff: Duration,
    pub backoff_max: Duration,
    pub restarts: u32,
    pub window: Duration,
    pub grace: Duration,
    pub line_bytes: u32,
}
/// The most restarts whose exit times can be retained.
pub const MAX_RESTARTS: u32 = 64;
/// The failed cross-check or restart bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unfit {
    Backoff,
    Window,
    Restarts,
    LineBytes,
}
/// An unknown name or a value beyond u32.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Unknown,
    Range,
}
impl Unfit {
    /// The names to report for this failure.
    #[must_use]
    pub fn names(self) -> &'static [&'static [u8]] {
        match self {
            Self::Backoff => &[b"box.backoff_ms", b"box.backoff_max_ms"],
            Self::Window => &[b"box.backoff_max_ms", b"box.window_ms"],
            Self::Restarts => &[b"box.restarts"],
            Self::LineBytes => &[b"box.line_bytes"],
        }
    }
}
/// The shipped profile (forge-setup decision 5).
#[must_use]
pub fn small() -> Limits {
    Limits {
        backoff: Duration::from_millis(500),
        backoff_max: Duration::from_secs(30),
        restarts: 5,
        window: Duration::from_secs(300),
        grace: Duration::from_secs(10),
        line_bytes: 256,
    }
}
/// Applies one whole dotted name.
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    if ![
        b"box.backoff_ms".as_slice(),
        b"box.backoff_max_ms",
        b"box.restarts",
        b"box.window_ms",
        b"box.grace_ms",
        b"box.line_bytes",
    ]
    .contains(&name)
    {
        return Err(Refused::Unknown);
    }
    let value = u32::try_from(value).or(Err(Refused::Range))?;
    let duration = Duration::from_millis(u64::from(value));
    match name {
        b"box.backoff_ms" => limits.backoff = duration,
        b"box.backoff_max_ms" => limits.backoff_max = duration,
        b"box.restarts" => limits.restarts = value,
        b"box.window_ms" => limits.window = duration,
        b"box.grace_ms" => limits.grace = duration,
        b"box.line_bytes" => limits.line_bytes = value,
        _ => return Err(Refused::Unknown),
    }
    Ok(())
}
/// Checks relationships in their documented order.
pub fn check(limits: &Limits) -> Result<(), Unfit> {
    if limits.backoff > limits.backoff_max {
        return Err(Unfit::Backoff);
    }
    if limits.backoff_max >= limits.window {
        return Err(Unfit::Window);
    }
    if limits.restarts > MAX_RESTARTS {
        return Err(Unfit::Restarts);
    }
    if limits.line_bytes < 64 {
        return Err(Unfit::LineBytes);
    }
    Ok(())
}
/// Machine state, exit history and two timer slots per step.
#[must_use]
pub fn worst_case(limits: &Limits, steps: u32) -> Option<u64> {
    let state = u64::try_from(size_of::<crate::machine::State>()).ok()?;
    let history = Queue::<Time>::worst_case(limits.restarts.checked_add(1)?)?;
    let timers = u64::try_from(size_of::<Option<Time>>()).ok()?.checked_mul(2)?;
    state.checked_add(history)?.checked_add(timers)?.checked_mul(u64::from(steps))
}
