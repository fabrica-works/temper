//! A chat's delegation: the task a delegate call made, watched as a card (client/README.md, 5.7).
use crate::Limits;
use alloc::boxed::Box;
use jig_client::{Handle, contract as c};
use skein_lib::List;

/// A task a chat's delegate call made, and the watch on its document.
#[derive(Debug)]
pub struct Delegation {
    pub task: u64,
    pub handle: Handle,
    pub final_task: Option<Box<c::Task>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Change,
    Report,
    Verdict,
    Approval,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Held,
    Landed,
    Done,
    Failed,
    Cancelled,
    Ended,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Said<'a> {
    Merge(&'a [u8]),
    Words(&'a [u8]),
}
/// What a delegation's card shows, from its task's document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts<'a> {
    pub task: u64,
    pub kind: Kind,
    pub state: State,
    pub title: &'a [u8],
    pub said: Option<Said<'a>>,
    pub steps: Option<u8>,
}
#[must_use]
#[expect(clippy::manual_map, reason = "the client's domain matches without a closure")]
pub fn facts(task: &c::Task) -> Facts<'_> {
    let kind = match task.result_contract {
        c::TaskContractDocument::Change { .. } => Kind::Change,
        c::TaskContractDocument::Report { .. } => Kind::Report,
        c::TaskContractDocument::Verdict { .. } => Kind::Verdict,
        c::TaskContractDocument::Approval { .. } => Kind::Approval,
    };
    let ended = match &task.result {
        Some(c::TaskResult::Document(result)) => {
            let (state, value, reason) = ending(&result.ending);
            Some((state, result.value.as_ref().or(value), reason))
        }
        Some(c::TaskResult::Ending(how)) => Some(owner_ending(how)),
        None => match task.chip.details.as_ref() {
            c::ChipPhase::Owner { details, .. } => match &details.ending {
                Some(how) => Some(owner_ending(how)),
                None => None,
            },
            c::ChipPhase::Ended { how, .. } => Some(ending(how)),
            c::ChipPhase::Active(_)
            | c::ChipPhase::Waiting { .. }
            | c::ChipPhase::Idle { .. }
            | c::ChipPhase::Starting
            | c::ChipPhase::Running { .. }
            | c::ChipPhase::WaitingForWords { .. }
            | c::ChipPhase::BackingOff { .. }
            | c::ChipPhase::Procedure { .. }
            | c::ChipPhase::Person { .. }
            | c::ChipPhase::Closing(_)
            | c::ChipPhase::Held { .. } => None,
        },
    };
    let (mut state, value, reason) = ended.unwrap_or((
        if task.hold.is_some() || task.chip.phase == c::Phase::Held { State::Held } else { State::Running },
        None,
        None,
    ));
    if state == State::Done
        && kind == Kind::Change
        && let Some(c::ResultValue::Change { .. }) = value
    {
        state = State::Landed;
    }
    let words = match value {
        Some(
            c::ResultValue::Report(words)
            | c::ResultValue::OwnerVerdict { words, .. }
            | c::ResultValue::Verdict { words, .. }
            | c::ResultValue::Connector { summary: words, .. }
            | c::ResultValue::Change { words, .. }
            | c::ResultValue::Choice { reason: words, .. }
            | c::ResultValue::Failure(words)
            | c::ResultValue::Words(words)
            | c::ResultValue::Approved { words: Some(words), .. },
        ) => Some(words.bytes()),
        Some(c::ResultValue::Approved { words: None, .. }) | None => None,
    }
    .or(reason);
    let said = match words {
        Some(words) => {
            let mut commit = state == State::Landed && words.len() >= 7;
            for byte in words {
                commit &= byte.is_ascii_digit() || (b'a'..=b'f').contains(byte);
            }
            Some(if commit {
                Said::Merge(words.get(..7).expect("commit has seven hex bytes"))
            } else {
                Said::Words(words)
            })
        }
        None => None,
    };
    Facts {
        task: task.chip.task,
        kind,
        state,
        title: task.chip.title.bytes(),
        said,
        steps: if kind == Kind::Change { Some(if state == State::Landed { 4 } else { 0 }) } else { None },
    }
}
fn ending(how: &c::Ending) -> (State, Option<&c::ResultValue>, Option<&[u8]>) {
    match how {
        c::Ending::Task(how) => owner_ending(how),
        c::Ending::Done => (State::Done, None, None),
        c::Ending::Failed { reason: Some(reason) } => (State::Failed, None, Some(reason.bytes())),
        c::Ending::Failed { reason: None } => (State::Failed, None, None),
        c::Ending::Cancelled { reason: Some(reason), .. } | c::Ending::Withdrawn { reason } => {
            (State::Cancelled, None, Some(reason.bytes()))
        }
        c::Ending::Cancelled { reason: None, .. } => (State::Cancelled, None, None),
        c::Ending::Decided { .. } | c::Ending::Deleted => (State::Ended, None, None),
    }
}
fn owner_ending(how: &c::OwnerChipEnding) -> (State, Option<&c::ResultValue>, Option<&[u8]>) {
    match how {
        c::OwnerChipEnding::Done(value) => (State::Done, Some(value), None),
        c::OwnerChipEnding::Failed(reason) => (State::Failed, None, Some(reason.bytes())),
        c::OwnerChipEnding::Cancelled { reason, result, .. } => {
            (State::Cancelled, result.as_ref(), Some(reason.bytes()))
        }
    }
}
/// The tasks a delegate call made: none for a failed call or another tool.
pub fn delegated(call: &c::Call, into: &mut List<u64>) {
    if call.failed || call.tool.bytes() != b"delegate" {
        return;
    }
    let mut named = false;
    for object in &call.objects {
        if let c::ObjectKey::Task(task) = object {
            named = true;
            if into.push(*task).is_err() {
                return;
            }
        }
    }
    if named {
        return;
    }
    let Some(words) = &call.outcome else { return };
    let Some(numbers) = words.bytes().strip_prefix(b"Delegated tasks: ") else { return };
    let mut from = 0;
    for (at, byte) in numbers.iter().chain(core::iter::once(&b',')).enumerate() {
        if *byte == b',' {
            if let Some(task) = number(numbers.get(from..at).unwrap_or_default())
                && into.push(task).is_err()
            {
                return;
            }
            from = at.saturating_add(1);
        }
    }
}
fn number(bytes: &[u8]) -> Option<u64> {
    let bytes = bytes.trim_ascii();
    if bytes.is_empty() {
        return None;
    }
    let mut number = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
#[must_use]
pub fn bound(limits: &Limits) -> u32 {
    // Leave room for me, accounts, the chat's task and its run; other pages close first.
    limits.client.domain.reads.min(limits.client.domain.watches.saturating_sub(4))
}
