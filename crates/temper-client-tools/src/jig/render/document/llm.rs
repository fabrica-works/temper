//! Supplied preferences and next-run waits (jig's client/domain/contract.md, 5.1).
use super::{Cut, Text, accounts};
use jig_client::contract as c;
use skein_json::writer::Encoder;

pub(super) const fn account_why(value: c::AccountWhy) -> &'static [u8] {
    match value {
        c::AccountWhy::Unknown => b"unknown",
        c::AccountWhy::Provider => b"provider",
        c::AccountWhy::Uses => b"uses",
        c::AccountWhy::Limited => b"limited",
    }
}
fn number(value: Option<u32>, out: &mut Encoder) {
    match value {
        Some(value) => out.unsigned(u64::from(value)),
        None => out.null(),
    }
}
pub(super) fn encode_task(value: &c::Task, out: &mut Encoder, cut: &mut Cut) {
    out.key(b"llm");
    match &value.llm {
        None => out.null(),
        Some(choice) => {
            out.object_start();
            out.key(b"account");
            number(choice.account, out);
            out.key(b"main");
            model(&choice.main, out, cut);
            out.key(b"subagents");
            out.array_start();
            for entry in &choice.subagents {
                model(entry, out, cut);
            }
            out.array_end();
            out.object_end();
        }
    }
    out.key(b"kept_account");
    number(value.kept_account, out);
    out.key(b"run_wait");
    match value.run_wait {
        None => out.null(),
        Some(c::TaskRunWait::Credential) => out.string(b"credential"),
        Some(c::TaskRunWait::Host) => out.string(b"host"),
        Some(c::TaskRunWait::Account { account, source, why }) => {
            out.object_start();
            out.key(b"account");
            out.object_start();
            out.key(b"account");
            number(account, out);
            out.key(b"source");
            number(source, out);
            out.key(b"why");
            wait(why, out);
            out.object_end();
            out.object_end();
        }
    }
}
fn model(value: &c::ModelEffort, out: &mut Encoder, cut: &mut Cut) {
    out.object_start();
    out.key(b"model");
    cut.words(value.model.bytes(), out);
    out.key(b"effort");
    match &value.effort {
        Some(effort) => cut.words(effort.bytes(), out),
        None => out.null(),
    }
    out.object_end();
}
fn wait(value: c::TaskWaitWhy, out: &mut Encoder) {
    match value {
        c::TaskWaitWhy::Refreshing => out.string(b"refreshing"),
        c::TaskWaitWhy::Host => out.string(b"host"),
        c::TaskWaitWhy::Empty => out.string(b"empty"),
        c::TaskWaitWhy::Invalid(why) => {
            out.object_start();
            out.key(b"invalid");
            out.string(account_why(why));
            out.object_end();
        }
        c::TaskWaitWhy::Source(state) => source(state, out),
    }
}
fn source(value: c::TaskSourceState, out: &mut Encoder) {
    match value {
        c::TaskSourceState::Asking => out.string(b"asking"),
        c::TaskSourceState::Granted => out.string(b"granted"),
        c::TaskSourceState::Refused => out.string(b"refused"),
        c::TaskSourceState::NeedsSignIn => out.string(b"needs_sign_in"),
        c::TaskSourceState::Misconfigured => out.string(b"misconfigured"),
        c::TaskSourceState::Cancelled => out.string(b"cancelled"),
        c::TaskSourceState::Closing => out.string(b"closing"),
        c::TaskSourceState::Unloaded { at, why } => {
            out.object_start();
            out.key(b"unloaded");
            out.object_start();
            accounts::unloaded(at, why, out);
            out.object_end();
            out.object_end();
        }
        c::TaskSourceState::Later { until } => deadline(b"later", until, out),
        c::TaskSourceState::Limited { until } => deadline(b"limited", until, out),
    }
}
fn deadline(label: &[u8], until: Option<skein_lib::Wall>, out: &mut Encoder) {
    out.object_start();
    out.key(label);
    out.object_start();
    out.key(b"until");
    match until {
        Some(wall) => out.unsigned(wall.as_nanos()),
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}
pub(super) fn text_wait(value: Option<c::TaskRunWait>, out: &mut Text) {
    let Some(value) = value else { return };
    out.put(b"\nnext run: ");
    match value {
        c::TaskRunWait::Credential => out.put(b"credential"),
        c::TaskRunWait::Host => out.put(b"host"),
        c::TaskRunWait::Account { account, source, why } => {
            out.put(b"account ");
            optional_number(account, out);
            out.put(b" source ");
            optional_number(source, out);
            out.put(b": ");
            match why {
                c::TaskWaitWhy::Refreshing => out.put(b"refreshing"),
                c::TaskWaitWhy::Host => out.put(b"host"),
                c::TaskWaitWhy::Empty => out.put(b"empty"),
                c::TaskWaitWhy::Invalid(why) => out.put(account_why(why)),
                c::TaskWaitWhy::Source(value) => source_text(value, out),
            }
        }
    }
}
fn optional_number(value: Option<u32>, out: &mut Text) {
    match value {
        Some(value) => out.number(u64::from(value)),
        None => out.put(b"unknown"),
    }
}
fn source_text(value: c::TaskSourceState, out: &mut Text) {
    match value {
        c::TaskSourceState::Later { until } => text_deadline(b"later until ", until, out),
        c::TaskSourceState::Limited { until } => text_deadline(b"limited until ", until, out),
        c::TaskSourceState::Asking => out.put(b"asking"),
        c::TaskSourceState::Granted => out.put(b"granted"),
        c::TaskSourceState::Refused => out.put(b"refused"),
        c::TaskSourceState::NeedsSignIn => out.put(b"needs sign-in"),
        c::TaskSourceState::Misconfigured => out.put(b"misconfigured"),
        c::TaskSourceState::Unloaded { at, why } => {
            out.put(b"unloaded ");
            out.put(match at {
                c::OwnerAccountPlace::Directory => b"directory",
                c::OwnerAccountPlace::File => b"file",
            });
            out.put(b": ");
            out.put(match why {
                c::OwnerAccountUnread::Unreadable => b"unreadable",
                c::OwnerAccountUnread::Refused => b"refused",
                c::OwnerAccountUnread::Failed => b"failed",
                c::OwnerAccountUnread::Stalled => b"stalled",
                c::OwnerAccountUnread::TooLarge => b"too large",
            });
        }
        c::TaskSourceState::Cancelled => out.put(b"cancelled"),
        c::TaskSourceState::Closing => out.put(b"closing"),
    }
}

fn text_deadline(label: &[u8], until: Option<skein_lib::Wall>, out: &mut Text) {
    out.put(label);
    match until {
        Some(wall) => super::utc(wall, out),
        None => out.put(b"unknown"),
    }
}

pub(super) fn refusal(account: u32, why: c::AccountWhy, out: &mut Encoder) {
    out.object_start();
    out.key(b"account");
    out.object_start();
    out.key(b"number");
    out.unsigned(u64::from(account));
    out.key(b"why");
    out.string(account_why(why));
    out.object_end();
    out.object_end();
}
