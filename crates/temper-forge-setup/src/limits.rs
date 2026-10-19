//! Reconcile's bounded orgs, deadline and wait (protocol/configuration.md, 7).
use skein_forge_setup::limits::Limits;
use skein_lib::Duration;
/// Why an override cannot be applied.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Unknown,
    Range,
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
