//! Supplied next-run choices and account waits (client/README.md, 3.5 and 5.5).
use crate::{
    frame::Bindings,
    words::{Word as W, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        choice::{self, Act as C},
    },
};

fn line(b: &mut Builder, w: &Words, word: W, holes: &[Hole]) {
    let text = w.say(word, holes);
    b.open(Role::Paragraph);
    if word != W::AccountBanner {
        b.style(crate::look::ACCOUNT_LINE);
    }
    b.name(&text);
    b.text(skein_ui::Run::Plain, &text);
    b.close();
}
fn control(b: &mut Builder, bindings: &mut Bindings, role: Role, name: &[u8], act: Act, pressed: bool) {
    b.open(role);
    b.name(name);
    if role == Role::Link {
        b.text(skein_ui::Run::Plain, name);
    }
    if pressed {
        b.state(State::Pressed);
    }
    b.bind(bindings.insert(act));
    b.close();
}
fn account_link(b: &mut Builder, bindings: &mut Bindings, name: &[u8]) {
    control(b, bindings, Role::Link, name, Act::Go { address: Address::Accounts }, false);
}
pub(crate) fn banner(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    if crate::frame::unavailable(root).is_some()
        || *root.pages().address() == Address::Accounts
        || root.pages().availability(root.client()) != Some(false)
    {
        return;
    }
    b.open(Role::Slot);
    b.style(crate::look::ACCOUNTS);
    line(b, w, W::AccountBanner, &[]);
    for word in [W::AccountSignIn, W::AccountProgram, W::AccountKey] {
        account_link(b, bindings, &w.say(word, &[]));
    }
    b.close();
}
pub(crate) fn composer(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings) {
    if let Some(task) = task {
        match task.chip.executor {
            c::Executor::Agent { .. } => {}
            c::Executor::Procedure { .. } | c::Executor::Party { .. } | c::Executor::Role { .. } => return,
        }
    }
    let draft = root.pages().choice();
    b.open(Role::Slot);
    b.style(crate::look::CHOICE);
    match draft.context {
        Some(choice::Context::Task { .. }) if draft.requested != draft.original => {
            requested(b, root, draft.original.as_ref(), W::ChoiceRequested, w);
            if draft.requested.is_some() {
                requested(b, root, draft.requested.as_ref(), W::ChoiceDraft, w);
            } else {
                line(b, w, W::ChoiceDraftDefault, &[]);
            }
        }
        Some(choice::Context::Template { .. }) if draft.requested == draft.original => {
            requested(b, root, draft.requested.as_ref(), W::ChoiceTemplate, w);
        }
        Some(choice::Context::Template { .. } | choice::Context::Task { .. }) | None => {
            requested(b, root, draft.requested.as_ref(), W::ChoiceRequested, w);
        }
    }
    if let Some(task) = task {
        committed(b, root, task, w, bindings);
    }
    if let Some(me) = temper_client_domain::pages::accounts::current_me(root.client())
        && let Some(project) = draft.project()
        && let Some(models) = choice::models(me, project)
        && !models.is_empty()
        && draft.pending.is_none()
        && match task {
            Some(task) => editable(task),
            None => true,
        }
    {
        b.open(Role::List);
        b.style(crate::look::ROWS);
        let selected_model = draft.selected(me);
        for (model, index) in models.iter().zip(0u32..) {
            b.open(Role::Item);
            b.style(crate::look::ROW);
            provider(b, root, task, model.provider, w);
            let selected = selected_model == Some(index);
            control(
                b,
                bindings,
                Role::Button,
                &shown(model.name.bytes(), w),
                Act::Choice(C::Model { index }),
                selected,
            );
            if selected {
                for (effort, index) in model.efforts.iter().zip(0u32..) {
                    let selected = match &draft.requested {
                        Some(choice) => choice.main.effort.as_ref() == Some(effort),
                        None => false,
                    };
                    control(
                        b,
                        bindings,
                        Role::Button,
                        &shown(effort.bytes(), w),
                        Act::Choice(C::Effort { index }),
                        selected,
                    );
                }
            }
            b.close();
        }
        b.close();
        control(b, bindings, Role::Button, &w.say(W::ChoiceReset, &[]), Act::Choice(C::Default), false);
        if task.is_some() && draft.requested != draft.original && draft.valid(me) {
            control(b, bindings, Role::Button, &w.say(W::ChoiceApply, &[]), Act::Choice(C::Apply), false);
        }
    }
    if draft.pending.is_some() {
        line(b, w, W::ChoicePending, &[]);
    }
    if let Some(outcome) = &draft.outcome {
        crate::pages::chats::outcome_text(b, w, outcome, 0);
    }
    b.close();
}

/// Includes the UTF-8 ellipsis; smaller phrasebooks retain their joined Name bound.
pub(crate) const MODEL_SHOWN: u32 = 256;
pub(crate) fn shown(name: &[u8], w: &Words) -> Box<[u8]> {
    let Ok(bound) = usize::try_from(MODEL_SHOWN.min(w.name_bytes)) else { return Box::default() };
    if name.len() <= bound {
        return skein_lib::bytes::copy_of(name);
    }
    let mark = "…".as_bytes();
    let Some(prefix_bound) = bound.checked_sub(mark.len()) else { return Box::default() };
    let Some(scan) = prefix_bound.checked_add(1) else { return Box::default() };
    // Text arrived validated; a continuation byte cannot begin the cut suffix.
    let mut end = 0;
    for (index, byte) in name.iter().enumerate().take(scan) {
        if byte & 0xc0 != 0x80 {
            end = index;
        }
    }
    let Some(prefix) = name.get(..end) else { return Box::default() };
    let Some(bytes) = end.checked_add(mark.len()) else { return Box::default() };
    let mut out = skein_lib::Writer::new(bytes);
    if out.put(prefix).is_err() || out.put(mark).is_err() {
        return Box::default();
    }
    out.finish()
}

fn requested(b: &mut Builder, root: &Domain, choice: Option<&c::LlmChoice>, label: W, w: &Words) {
    let Some(choice) = choice else {
        line(b, w, W::ChoiceDefault, &[]);
        return;
    };
    line(b, w, label, &[Hole::Name(shown(choice.main.model.bytes(), w))]);
    if let Some(effort) = &choice.main.effort {
        line(b, w, W::ChoiceEffort, &[Hole::Name(shown(effort.bytes(), w))]);
    }
    if let Some(account) = choice.account {
        account_line(b, root, w, account, (W::ChoicePreference, W::ChoicePreferenceName));
    }
}
#[expect(clippy::manual_find, reason = "the programming model uses bounded loops, not closures")]
fn account_row(root: &Domain, account: u32) -> Option<&c::OwnerAccountLine> {
    let doc = temper_client_domain::pages::accounts::document(root.client())?;
    for row in &doc.accounts {
        if row.number == account {
            return Some(row);
        }
    }
    None
}
fn account_line(b: &mut Builder, root: &Domain, w: &Words, account: u32, labels: (W, W)) {
    match account_row(root, account) {
        Some(row) => line(b, w, labels.1, &[Hole::Name(skein_lib::bytes::copy_of(row.display.bytes()))]),
        None => line(b, w, labels.0, &[Hole::Number(u64::from(account))]),
    }
}

fn provider(b: &mut Builder, root: &Domain, task: Option<&c::Task>, number: Option<u32>, w: &Words) {
    let Some(number) = number else { return };
    let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) else { return };
    for offer in &me.providers {
        if offer.provider == number {
            line(b, w, W::ChoiceProvider, &[Hole::Name(skein_lib::bytes::copy_of(offer.name.bytes()))]);
            break;
        }
    }
    let payer = match task {
        None => Some(c::OwnerAccountUse::Party(me.party.number)),
        Some(task) => match task.budget.funder {
            c::Funder::Pool { person, .. } => Some(c::OwnerAccountUse::Party(person)),
            c::Funder::Period { project, .. } | c::Funder::Recurring { project, .. } => {
                Some(c::OwnerAccountUse::Project(project))
            }
            c::Funder::Task(_) => None,
        },
    };
    let mut readiness = c::Readiness::Unknown;
    if let Some(doc) = temper_client_domain::pages::accounts::document(root.client()) {
        for row in &doc.readiness {
            if row.provider == number && Some(row.payer) == payer {
                readiness = row.readiness;
                break;
            }
        }
    }
    match readiness {
        c::Readiness::Unavailable => line(b, w, W::NeedsAccount, &[]),
        c::Readiness::Unknown => line(b, w, W::AccountReadinessUnknown, &[]),
        c::Readiness::Usable => {}
    }
}
fn editable(task: &c::Task) -> bool {
    match jig_client::facts::offer(task.acts.as_slice(), c::ActKind::Amend) {
        Some(c::Admission::Allowed | c::Admission::Bounded { .. } | c::Admission::Proposed { .. }) => true,
        Some(c::Admission::Refused(_)) | None => false,
    }
}
fn committed(b: &mut Builder, root: &Domain, task: &c::Task, w: &Words, bindings: &mut Bindings) {
    if let Some(account) = task.kept_account {
        line(b, w, W::ChoiceKept, &[Hole::Number(u64::from(account))]);
        if let Some(row) = account_row(root, account) {
            b.open(Role::Paragraph);
            b.style(crate::look::ACCOUNT_LINE);
            if let Some(name) = &row.provider_name {
                b.text(skein_ui::Run::Plain, name.bytes());
                b.text(skein_ui::Run::Plain, b" \xc2\xb7 ");
            }
            b.text(skein_ui::Run::Plain, row.display.bytes());
            if row.plan {
                b.text(skein_ui::Run::Plain, b" \xc2\xb7 ");
                b.text(skein_ui::Run::Plain, &crate::accounts::words::say(w, crate::accounts::words::Word::Plan, &[]));
            }
            if let Some(address) = &row.manage_usage
                && skein_ui::markdown::safe_url(address)
                && c::valid_manage_usage(address)
            {
                b.open(Role::Link);
                let text = w.say(W::Usage, &[]);
                b.name(&text);
                b.text(skein_ui::Run::Plain, &text);
                b.external(address);
                b.close();
            }
            b.close();
        }
    }
    waiting(b, root, task.run_wait, w, bindings);
}
pub(crate) fn waiting(
    b: &mut Builder,
    root: &Domain,
    wait: Option<c::TaskRunWait>,
    w: &Words,
    bindings: &mut Bindings,
) {
    match wait {
        None => {}
        Some(c::TaskRunWait::Credential) => line(b, w, W::WaitCredential, &[]),
        Some(c::TaskRunWait::Host) => line(b, w, W::WaitCapacity, &[]),
        Some(c::TaskRunWait::Account { account, source, why }) => {
            account_wait(b, root, account, source, why, w, bindings);
        }
    }
}
fn account_wait(
    b: &mut Builder,
    root: &Domain,
    account: Option<u32>,
    source: Option<u32>,
    why: c::TaskWaitWhy,
    w: &Words,
    bindings: &mut Bindings,
) {
    use crate::accounts::words::{self, Word as A};
    use c::{TaskSourceState as S, TaskWaitWhy as R};
    b.open(Role::Status);
    b.style(crate::look::STATUS_ACTIVE);
    let name = w.say(W::AccountWaiting, &[]);
    b.name(&name);
    b.text(skein_ui::Run::Plain, &name);
    b.close();
    if let Some(account) = account {
        account_line(b, root, w, account, (W::WaitingAccount, W::WaitingAccountName));
    }
    if let Some(source) = source {
        b.open(Role::Paragraph);
        b.text(skein_ui::Run::Plain, &words::say(w, A::Source, &[]));
        b.text(skein_ui::Run::Plain, &words::say(w, A::Number, &[Hole::Number(u64::from(source))]));
        b.close();
    }
    match why {
        R::Refreshing => line(b, w, W::WaitRefreshing, &[]),
        R::Host => line(b, w, W::WaitHost, &[]),
        R::Empty => {
            line(b, w, W::WaitEmpty, &[]);
            account_link(b, bindings, &w.say(W::AccountSignIn, &[]));
            account_link(b, bindings, &w.say(W::AccountKey, &[]));
        }
        R::Invalid(why) => {
            line(b, w, W::WaitInvalid, &[]);
            if let Some(account) = account {
                w.refusal(b, &c::Refusal::Account { account, why }, 0);
            }
        }
        R::Source(S::Limited { until }) => {
            source_wait(b, w, A::Limited, until);
            account_link(b, bindings, &words::say(w, A::Resume, &[]));
        }
        R::Source(S::Later { until }) => source_wait(b, w, A::Later, until),
        R::Source(S::Unloaded { at, why }) => {
            b.open(Role::Paragraph);
            crate::pages::accounts::unloaded(b, w, at, why);
            b.close();
        }
        R::Source(S::Asking) => source_wait(b, w, A::Asking, None),
        R::Source(S::Granted) => source_wait(b, w, A::Granted, None),
        R::Source(S::Refused) => {
            source_wait(b, w, A::Refused, None);
            account_link(b, bindings, &w.say(W::AccountKey, &[]));
        }
        R::Source(S::NeedsSignIn) => {
            source_wait(b, w, A::NeedsSignIn, None);
            account_link(b, bindings, &w.say(W::AccountSignIn, &[]));
            account_link(b, bindings, &w.say(W::AccountKey, &[]));
        }
        R::Source(S::Misconfigured) => source_wait(b, w, A::Misconfigured, None),
        R::Source(S::Cancelled) => source_wait(b, w, A::Cancelled, None),
        R::Source(S::Closing) => source_wait(b, w, A::Closing, None),
    }
}
fn source_wait(b: &mut Builder, w: &Words, word: crate::accounts::words::Word, until: Option<skein_lib::Wall>) {
    use crate::accounts::words::{self, Word as A};
    b.open(Role::Paragraph);
    b.text(skein_ui::Run::Plain, &words::say(w, word, &[]));
    if let Some(until) = until {
        b.text(skein_ui::Run::Plain, &words::say(w, A::Until, &[Hole::Wall(until)]));
    }
    b.close();
}
