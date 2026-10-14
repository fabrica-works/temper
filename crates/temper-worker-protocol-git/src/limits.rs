//! Limits and counted payloads (git.md, section 8).
use alloc::boxed::Box;
use skein_io::kernel;
use skein_lib::List;
use temper_worker_domain_checkout::{self as checkout, git};

/// The component's configured bounds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub operations: u32,
    pub stdout_bytes: u32,
    pub stderr_bytes: u32,
    pub read_bytes: u32,
    pub sweep_entries: u32,
    pub sweep_bytes: u32,
}

/// The rule that refuses startup, named under `git_component`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unfit {
    Operations,
    StdoutBytes,
    StderrBytes,
    ReadBytes,
    SweepEntries,
    SweepBytes,
    Memory,
}

/// Validate every rule before allocating the component.
pub fn check(limits: &Limits, checkout: &checkout::Limits) -> Result<(), Unfit> {
    if limits.operations < checkout.workspaces {
        return Err(Unfit::Operations);
    }
    let output = largest_output(checkout).ok_or(Unfit::StdoutBytes)?;
    if limits.stdout_bytes < output {
        return Err(Unfit::StdoutBytes);
    }
    if u64::from(limits.stderr_bytes) < u64::try_from(git::PushDiagnostic::CAPACITY).expect("diagnostic bound fits u64")
    {
        return Err(Unfit::StderrBytes);
    }
    if limits.read_bytes == 0 {
        return Err(Unfit::ReadBytes);
    }
    if limits.sweep_entries == 0 {
        return Err(Unfit::SweepEntries);
    }
    if u64::from(limits.sweep_bytes) < u64::try_from(kernel::LONGEST_NAME).expect("name bound fits u64") {
        return Err(Unfit::SweepBytes);
    }
    worst_case(limits, checkout).ok_or(Unfit::Memory)?;
    Ok(())
}

/// A NUL path list, or push's framing, URL, refspec and status summary.
#[must_use]
pub fn largest_output(checkout: &checkout::Limits) -> Option<u32> {
    let paths = checkout.conflicts.checked_mul(checkout.path_bytes.checked_add(1)?)?;
    // To <URL> LF, flag TAB <64 digit id>:refs/heads/<branch> TAB summary LF,
    // Done LF. The longest accepted summary and fixed framing fit 256 bytes.
    let push = checkout.name_bytes.checked_mul(2)?.checked_add(256)?;
    Some(paths.max(push).max(65))
}

/// Argument/environment/message/output/path payloads and one sweep page.
/// Process and file state containers are added with their implementations.
#[must_use]
pub fn worst_case(limits: &Limits, checkout: &checkout::Limits) -> Option<u64> {
    let name = u64::from(checkout.name_bytes);
    // Forty argument boxes; settings/options/ids have fewer than 1024 bytes,
    // and the longest leased push owns its remote and branch twice.
    let arguments = List::<Box<[u8]>>::worst_case(40)?.checked_add(1024)?.checked_add(name.checked_mul(3)?)?;
    // channel.md's grant ceiling is 8192 bytes (the pinned jig schema).
    // Four identity fields, their base64 header and author copies, fixed
    // variable names, and HOME fit this envelope for a filesystem path.
    let environment = List::<Box<[u8]>>::worst_case(14)?.checked_add(32768)?;
    let paths = List::<Box<[u8]>>::worst_case(checkout.conflicts)?
        .checked_add(u64::from(checkout.conflicts).checked_mul(u64::from(checkout.path_bytes))?)?;
    let mut each = arguments
        .checked_add(environment)?
        .checked_add(u64::from(checkout.message_bytes).checked_add(3)?)?
        .checked_add(paths)?;
    for bytes in [limits.stdout_bytes, limits.stderr_bytes, limits.read_bytes] {
        each = each.checked_add(u64::from(bytes))?;
    }
    let page = List::<kernel::Entry>::worst_case(limits.sweep_entries)?.checked_add(u64::from(limits.sweep_bytes))?;
    u64::from(limits.operations).checked_mul(each)?.checked_add(page)
}

/// The most records one entry point emits, except abort.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub events: u32,
    pub io: u32,
    pub kernel: u32,
}
/// One terminal, a spawn or kill and closes, and one file operation.
pub const MAX_OUT: MaxOut = MaxOut { events: 1, io: 4, kernel: 1 };
