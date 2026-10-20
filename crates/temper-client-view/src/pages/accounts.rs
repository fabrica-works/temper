//! Whole supplied account lists, with no local account or source policy (client/README.md, 5.5).
use crate::{
    accounts::{
        self as bindings, Bindings, Field, Meaning,
        words::{self, Word as W},
    },
    words::Words,
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::accounts::{self, Act, Broken, Operation},
};
fn run(b: &mut Builder, w: &Words, word: W) {
    b.run(Run::Plain, &words::say(w, word, &[]));
}
fn number(b: &mut Builder, w: &Words, value: u64) {
    b.run(Run::Plain, &words::say(w, W::Number, &[Hole::Number(value)]));
}
fn text(b: &mut Builder, w: &Words, role: Role, word: W) {
    crate::cards::text(b, role, &words::say(w, word, &[]));
}
fn fact(b: &mut Builder, w: &Words, word: W, value: u64) {
    b.open(Role::Paragraph);
    b.open(Role::Text);
    run(b, w, word);
    number(b, w, value);
    b.close();
    b.close();
}
fn control(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: W, meaning: Meaning, disabled: bool) {
    b.open(Role::Button);
    b.name(&words::say(w, word, &[]));
    b.bind(bindings.insert(meaning));
    if disabled {
        b.state(State::Disabled);
    }
    b.close();
}
fn holder(b: &mut Builder, w: &Words, value: c::OwnerAccountHolder) {
    match value {
        c::OwnerAccountHolder::Deployment => run(b, w, W::Deployment),
        c::OwnerAccountHolder::Party(party) => {
            run(b, w, W::Party);
            number(b, w, party);
        }
    }
}
fn kind(b: &mut Builder, w: &Words, value: c::OwnerAccountKind) {
    run(
        b,
        w,
        match value {
            c::OwnerAccountKind::SignIn => W::SignIn,
            c::OwnerAccountKind::Key => W::Key,
            c::OwnerAccountKind::Program => W::Program,
            c::OwnerAccountKind::None => W::None,
        },
    );
}
fn unloaded(b: &mut Builder, w: &Words, at: c::OwnerAccountPlace, why: c::OwnerAccountUnread) {
    run(b, w, W::Unloaded);
    run(
        b,
        w,
        match at {
            c::OwnerAccountPlace::Directory => W::Directory,
            c::OwnerAccountPlace::File => W::File,
        },
    );
    b.run(Run::Plain, b": ");
    run(
        b,
        w,
        match why {
            c::OwnerAccountUnread::Unreadable => W::Unreadable,
            c::OwnerAccountUnread::Refused => W::Refused,
            c::OwnerAccountUnread::Failed => W::Failed,
            c::OwnerAccountUnread::Stalled => W::Stalled,
            c::OwnerAccountUnread::TooLarge => W::TooLarge,
        },
    );
}
fn state(b: &mut Builder, w: &Words, source: &c::OwnerAccountSource) {
    match source.state {
        c::OwnerAccountState::Later { until } => {
            run(b, w, W::Limited);
            if let Some(at) = until {
                b.run(Run::Plain, &words::say(w, W::Until, &[Hole::Wall(at)]));
            }
        }
        c::OwnerAccountState::Unloaded { at, why } => unloaded(b, w, at, why),
        c::OwnerAccountState::Asking => run(b, w, W::Asking),
        c::OwnerAccountState::Granted => run(b, w, if source.usable { W::Ready } else { W::Granted }),
        c::OwnerAccountState::SignInAgain => run(b, w, W::NeedsSignIn),
        c::OwnerAccountState::Misconfigured => run(b, w, W::Misconfigured),
        c::OwnerAccountState::Cancelled => run(b, w, W::Cancelled),
        c::OwnerAccountState::Closing => run(b, w, W::Closing),
    }
    if source.expiring {
        run(b, w, W::Expiring);
    }
    if source.attention {
        run(b, w, W::Attention);
    }
    if let Some(at) = source.spent_until {
        run(b, w, W::Spent);
        b.run(Run::Plain, &words::say(w, W::Until, &[Hole::Wall(at)]));
    }
}
#[expect(
    clippy::too_many_arguments,
    reason = "borrow the common fields of party and deployment accounts without copying documents"
)]
fn account(
    b: &mut Builder,
    w: &Words,
    number_value: u32,
    place: u32,
    provider: Option<u32>,
    display: &c::DocumentText,
    held: c::OwnerAccountHolder,
    plan: bool,
    uses: Option<&skein_lib::List<c::OwnerAccountUse>>,
    sources: &skein_lib::List<c::OwnerAccountSource>,
    cut: bool,
) {
    b.open(Role::Region);
    b.open(Role::Heading { level: 3 });
    b.open(Role::Text);
    run(b, w, W::Account);
    number(b, w, u64::from(number_value));
    b.run(Run::Plain, b" \xc2\xb7 ");
    if let Some(provider) = provider {
        run(b, w, W::Provider);
        number(b, w, u64::from(provider));
    }
    b.run(Run::Plain, b" \xc2\xb7 ");
    b.run(Run::Plain, display.bytes());
    b.close();
    b.close();
    b.open(Role::Paragraph);
    b.open(Role::Text);
    run(b, w, W::Place);
    number(b, w, u64::from(place));
    b.run(Run::Plain, b" \xc2\xb7 ");
    holder(b, w, held);
    if plan {
        b.run(Run::Plain, b" \xc2\xb7 ");
        run(b, w, W::Plan);
    }
    b.run(Run::Plain, b" \xc2\xb7 ");
    match uses {
        None => run(b, w, W::DefaultUses),
        Some(uses) => {
            run(b, w, W::Uses);
            for value in uses {
                match value {
                    c::OwnerAccountUse::Party(p) => {
                        run(b, w, W::Party);
                        number(b, w, *p);
                    }
                    c::OwnerAccountUse::Project(p) => {
                        run(b, w, W::Project);
                        number(b, w, u64::from(*p));
                    }
                }
                b.run(Run::Plain, b"; ");
            }
        }
    }
    if cut || display.cut {
        run(b, w, W::Cut);
    }
    b.close();
    b.close();
    b.open(Role::List);
    for source in sources {
        b.open(Role::Item);
        b.open(Role::Text);
        run(b, w, W::Source);
        number(b, w, u64::from(source.number));
        b.run(Run::Plain, b" \xc2\xb7 ");
        run(b, w, W::Place);
        number(b, w, u64::from(source.place));
        b.run(Run::Plain, b" \xc2\xb7 ");
        kind(b, w, source.kind);
        b.run(Run::Plain, b" \xc2\xb7 ");
        state(b, w, source);
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
fn end(b: &mut Builder, w: &Words, end: &c::OwnerAccountEnd) {
    b.open(Role::Status);
    b.open(Role::Text);
    match end {
        c::OwnerAccountEnd::Made { account, declined } | c::OwnerAccountEnd::Renewed { account, declined } => {
            run(b, w, if let c::OwnerAccountEnd::Made { .. } = end { W::Made } else { W::Renewed });
            number(b, w, u64::from(*account));
            if *declined {
                run(b, w, W::Declined);
            }
        }
        c::OwnerAccountEnd::Refused { refusal } => refused(b, w, *refusal),
        c::OwnerAccountEnd::Failed { failure } => match failure {
            c::OwnerAccountFailure::Later { after } => {
                run(b, w, W::Later);
                if let Some(after) = after {
                    b.run(Run::Plain, &words::say(w, W::After, &[Hole::Span(*after)]));
                }
            }
            c::OwnerAccountFailure::SignInAgain => run(b, w, W::NeedsSignIn),
            c::OwnerAccountFailure::MissingScope { scope } => {
                run(b, w, W::Scope);
                number(b, w, u64::from(*scope));
            }
            c::OwnerAccountFailure::Misconfigured => run(b, w, W::Misconfigured),
            c::OwnerAccountFailure::Unloaded { at, why } => unloaded(b, w, *at, *why),
            c::OwnerAccountFailure::Cancelled => run(b, w, W::Cancelled),
        },
    }
    b.close();
    b.close();
}
fn refused(b: &mut Builder, w: &Words, refusal: c::OwnerAccountRefusal) {
    use c::OwnerAccountRefusal as R;
    let word = match refusal {
        R::Configured => W::Configured,
        R::NotLimited => W::NotLimited,
        R::Uses => W::UsesRefused,
        R::NotKept { why } => match why {
            c::OwnerAccountKeyFailure::TooLong => W::KeyLong,
            c::OwnerAccountKeyFailure::NotStored => W::NotStored,
        },
        R::Limit { bound } => match bound {
            c::OwnerAccountBound::Accounts => W::LimitAccounts,
            c::OwnerAccountBound::HolderAccounts => W::LimitHolder,
            c::OwnerAccountBound::Sources => W::LimitSources,
            c::OwnerAccountBound::SignIns => W::LimitSignIns,
        },
        R::Held { provider } | R::Provider { provider } => {
            run(b, w, if let R::Held { .. } = refusal { W::Held } else { W::ProviderRefused });
            number(b, w, u64::from(provider));
            return;
        }
        R::Subject => W::Subject,
        R::Unknown => W::UnknownNumber,
        R::Numbers => W::Numbers,
        R::Kind { kind: value } => {
            run(b, w, W::Kind);
            kind(b, w, value);
            return;
        }
        R::Closing => W::Closing,
        R::NotStarted => W::NotStarted,
    };
    run(b, w, word);
}
pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    b.open(Role::Main);
    b.style(crate::look::MAIN);
    text(b, w, Role::Heading { level: 1 }, W::Accounts);
    text(b, w, Role::Heading { level: 2 }, W::Own);
    if let Some(doc) = accounts::document(root.client()) {
        if doc.accounts.is_empty() {
            text(b, w, Role::Paragraph, W::Empty);
        }
        for row in &doc.accounts {
            account(
                b,
                w,
                row.number,
                row.place,
                row.provider,
                &row.display,
                row.holder,
                row.plan,
                row.uses.as_ref(),
                &row.sources,
                row.cut,
            );
        }
        for signing in &doc.signing {
            b.open(Role::Status);
            b.open(Role::Text);
            run(b, w, W::Signing);
            number(b, w, u64::from(signing.source));
            b.run(Run::Plain, b" \xc2\xb7 ");
            run(b, w, W::Provider);
            number(b, w, u64::from(signing.provider));
            b.run(Run::Plain, b" \xc2\xb7 ");
            holder(b, w, signing.holder);
            b.close();
            b.close();
        }
        if doc.settled {
            text(b, w, Role::Status, W::Settled);
        }
        if let Some(last) = &doc.last_end {
            fact(b, w, W::LastEnd, u64::from(last.source));
            end(b, w, &last.end);
        }
        if doc.cut {
            text(b, w, Role::Status, W::Cut);
        }
    } else {
        text(b, w, Role::Status, W::Unknown);
    }
    if let Some(form) = root.pages().accounts() {
        if is_owner(root) {
            text(b, w, Role::Heading { level: 2 }, W::Deployment);
            if let Some(doc) = form.deployment(root.client()) {
                for row in &doc.accounts {
                    account(
                        b,
                        w,
                        row.number,
                        row.place,
                        row.provider,
                        &row.display,
                        row.holder,
                        row.plan,
                        row.uses.as_ref(),
                        &row.sources,
                        row.cut,
                    );
                }
                if doc.cut {
                    text(b, w, Role::Status, W::Cut);
                }
            } else {
                text(b, w, Role::Status, W::Unknown);
            }
        }
        form_page(b, root, w, bindings, form);
    }
    b.close();
}
fn form_page(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, form: &accounts::Accounts) {
    let busy = match &form.pending {
        Some(p) => p.outcome.is_none(),
        None => false,
    };
    b.open(Role::Form);
    b.name(&words::say(w, W::Accounts, &[]));
    for (holder, word) in [(c::AccountHolder::Own, W::OwnChoice), (c::AccountHolder::Deployment, W::DeploymentChoice)] {
        if holder == c::AccountHolder::Deployment && !is_owner(root) {
            continue;
        }
        control(
            b,
            w,
            bindings,
            word,
            Meaning::Action(Act::Select { selection: accounts::Selection { holder, ..Default::default() } }),
            busy,
        );
    }
    for (field, word, value) in [
        (Field::Provider, W::ProviderField, form.selection.provider),
        (Field::Account, W::AccountField, form.selection.account),
        (Field::Source, W::SourceField, form.selection.source),
    ] {
        b.open(Role::TextField);
        b.name(&words::say(w, word, &[]));
        let text: Box<[u8]> = match value {
            Some(n) => words::say(w, W::Number, &[Hole::Number(u64::from(n))]),
            None => Box::new([]),
        };
        b.value(&text, u64::from(value.unwrap_or(0)));
        b.bind(bindings.insert(Meaning::Field(field, form.selection)));
        if busy {
            b.state(State::Disabled);
        }
        b.close();
    }
    b.open(Role::TextField);
    b.name(&words::say(w, W::KeyField, &[]));
    b.entry(skein_ui::EntryKind::Secret);
    b.value(if form.has_key() { b"*" } else { b"" }, form.key_written());
    b.bind(bindings.insert(Meaning::Key));
    if busy {
        b.state(State::Disabled);
    }
    b.close();
    let name = root.pages().account_name();
    for (operation, word) in [
        (Operation::SignIn, W::SignInAction),
        (Operation::AddKey, W::AddKey),
        (Operation::ReplaceKey, W::ReplaceKey),
        (Operation::SignOut, W::SignOut),
        (Operation::RemoveKey, W::RemoveKey),
        (Operation::RemoveAccount, W::RemoveAccount),
        (Operation::Resume, W::Resume),
    ] {
        control(b, w, bindings, word, bindings::submit(name, operation), busy);
    }
    // Rank controls use the engine's one-based positions, never provider or holder preference.
    if let Some(doc) = accounts::document(root.client()) {
        for row in &doc.accounts {
            if form.selection.account == Some(row.number) && form.selection.holder == c::AccountHolder::Own {
                rank(b, w, bindings, name, form, row.place, &row.sources, busy);
            }
        }
    }
    if let Some(doc) = form.deployment(root.client()) {
        for row in &doc.accounts {
            if form.selection.account == Some(row.number) && form.selection.holder == c::AccountHolder::Deployment {
                rank(b, w, bindings, name, form, row.place, &row.sources, busy);
            }
        }
    }
    if let Some((_, ask)) = &form.confirm {
        let word = if let c::Ask::SignOut { .. } = ask {
            W::ConfirmSignOut
        } else if let c::Ask::AccountChange { change: c::AccountChange::RemoveKey { .. }, .. } = ask {
            W::ConfirmKey
        } else {
            W::ConfirmAccount
        };
        text(b, w, Role::Alert, word);
        control(b, w, bindings, W::Confirm, Meaning::Action(Act::Confirm), busy);
    }
    control(b, w, bindings, W::Cancel, Meaning::Action(Act::Cancel), false);
    outcome(b, root, w, form);
    b.close();
}
fn outcome(b: &mut Builder, root: &Domain, w: &Words, form: &accounts::Accounts) {
    if let Some(broken) = form.broken {
        text(
            b,
            w,
            Role::Alert,
            match broken {
                Broken::Empty => W::KeyEmpty,
                Broken::TooLong => W::KeyLong,
                Broken::Selection => W::Selection,
                Broken::Rank => W::Rank,
                Broken::ValueAgain => W::ValueAgain,
            },
        );
    }
    if let Some(pending) = &form.pending {
        match &pending.outcome {
            None => text(b, w, Role::Status, W::InFlight),
            Some(jig_client::Outcome::Lost) => text(b, w, Role::Alert, W::Lost),
            Some(jig_client::Outcome::Unsent) => text(b, w, Role::Alert, W::Unsent),
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                c::OwnerOutcome::AccountSignedOut { confirmed },
            ))))) => text(b, w, Role::Status, if *confirmed { W::Confirmed } else { W::Unconfirmed }),
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                c::OwnerOutcome::AccountRemoved { confirmed },
            ))))) => text(b, w, Role::Status, if *confirmed { W::Removed } else { W::RemovedUnconfirmed }),
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Owner(
                c::OwnerOutcome::AccountKeyKept { source },
            ))))) => fact(b, w, W::Saved, u64::from(*source)),
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal, .. }))) => {
                crate::cards::refusal(b, w, refusal);
            }
            Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(outcome)))) => {
                crate::cards::outcome(b, w, outcome);
            }
            Some(
                jig_client::Outcome::Answered(jig_client::Answered::Family(_))
                | jig_client::Outcome::NotKept
                | jig_client::Outcome::Held
                | jig_client::Outcome::Unknown
                | jig_client::Outcome::Limit
                | jig_client::Outcome::Unconfirmed,
            ) => text(b, w, Role::Status, W::Unknown),
        }
    }
    if let Some(address) = form.sign_in_address(root.client()) {
        text(b, w, Role::Paragraph, W::Address);
        b.open(Role::Link);
        b.text(Run::Plain, address);
        b.external(address);
        b.close();
    }
}
#[expect(clippy::too_many_arguments, reason = "rank controls share the current form and supplied source positions")]
fn rank(
    b: &mut Builder,
    w: &Words,
    bindings: &mut Bindings,
    name: skein_lib::RequestKey,
    form: &accounts::Accounts,
    place: u32,
    sources: &skein_lib::List<c::OwnerAccountSource>,
    busy: bool,
) {
    if place > 1
        && let Some(up) = place.checked_sub(1)
    {
        control(b, w, bindings, W::UpAccount, bindings::submit(name, Operation::MoveAccount { place: up }), busy);
    }
    if let Some(down) = place.checked_add(1) {
        control(b, w, bindings, W::DownAccount, bindings::submit(name, Operation::MoveAccount { place: down }), busy);
    }
    for source in sources {
        if form.selection.source != Some(source.number) {
            continue;
        }
        if source.place > 1
            && let Some(up) = source.place.checked_sub(1)
        {
            control(b, w, bindings, W::UpSource, bindings::submit(name, Operation::MoveSource { place: up }), busy);
        }
        if let Some(down) = source.place.checked_add(1) {
            control(b, w, bindings, W::DownSource, bindings::submit(name, Operation::MoveSource { place: down }), busy);
        }
    }
}

fn is_owner(root: &Domain) -> bool {
    match accounts::current_me(root.client()) {
        Some(me) => me.deployment_role == Some(c::HeldRole::Owner),
        None => false,
    }
}
