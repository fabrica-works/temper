//! Reconcile's bounded orgs, deadline and wait (protocol/configuration.md, 7).
use skein_lib::Duration;
/// Limits checked before a setup is read.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Limits {
    pub orgs: u32,             // Orgs per forge.
    pub deadline: Duration,    // Whole reconcile run.
    pub backoff: Duration,     // First retry delay.
    pub backoff_max: Duration, // Largest retry delay.
}
/// Why an override cannot be applied.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Unknown,
    Range,
}
/// Two limits whose values disagree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unfit {
    pub name: &'static [u8],
    pub other: &'static [u8],
}
/// The shipped profile (forge-setup decision 5).
#[must_use]
pub fn small() -> Limits {
    Limits {
        orgs: 8,
        deadline: Duration::from_secs(120),
        backoff: Duration::from_millis(100),
        backoff_max: Duration::from_secs(5),
    }
}
/// Applies a dotted setup override, refusing zero and overflow.
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    if ![b"setup.orgs".as_slice(), b"setup.deadline_ms", b"setup.backoff_ms", b"setup.backoff_max_ms"].contains(&name) {
        return Err(Refused::Unknown);
    }
    if value == 0 {
        return Err(Refused::Range);
    }
    match name {
        b"setup.orgs" => limits.orgs = u32::try_from(value).or(Err(Refused::Range))?,
        b"setup.deadline_ms" => limits.deadline = duration(value)?,
        b"setup.backoff_ms" => limits.backoff = duration(value)?,
        b"setup.backoff_max_ms" => limits.backoff_max = duration(value)?,
        _ => return Err(Refused::Unknown),
    }
    Ok(())
}
fn duration(value: u64) -> Result<Duration, Refused> {
    match value.checked_mul(1_000_000) {
        Some(n) => Ok(Duration::from_nanos(n)),
        None => Err(Refused::Range),
    }
}
/// Checks both relationships, naming the offending pair.
pub fn check(limits: &Limits) -> Result<(), Unfit> {
    if limits.backoff > limits.backoff_max {
        return Err(Unfit { name: b"setup.backoff_ms", other: b"setup.backoff_max_ms" });
    }
    if limits.backoff_max >= limits.deadline {
        return Err(Unfit { name: b"setup.backoff_max_ms", other: b"setup.deadline_ms" });
    }
    Ok(())
}
