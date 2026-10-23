//! The first run's welcome and shared account ways (client/README.md, 3.5, 5.6).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run};
use temper_client_domain::{Address, Domain, Step, forge::forges::Connection, pages::Act};

fn text(b: &mut Builder, w: &Words, role: Role, word: Word) {
    crate::cards::text(b, role, &w.say(word, &[]));
}
fn paragraph(b: &mut Builder, w: &Words, word: Word, style: skein_ui::Style) {
    b.open(Role::Paragraph);
    b.style(style);
    b.name(&w.say(word, &[]));
    w.text(b, word, &[]);
    b.close();
}
fn control(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, act: Act) {
    let name = w.say(word, &[]);
    let binding = bindings.insert(act.clone());
    if let Act::Go { .. } = act {
        crate::link(b, &name, binding);
        b.text(Run::Plain, &name);
    } else {
        b.open(Role::Button);
        b.name(&name);
        b.bind(binding);
    }
    b.close();
}
fn go(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, step: Step) {
    control(b, w, bindings, word, Act::Go { address: Address::Welcome { step } });
}
fn steps(b: &mut Builder, root: &Domain, w: &Words, current: Step) {
    let welcome = w.say(Word::WelcomeStep, &[]);
    let forge = w.say(Word::WelcomeForgeStep, &[]);
    let model = w.say(Word::WelcomeModelStep, &[]);
    let ready = w.say(Word::WelcomeReadyStep, &[]);
    let current = match current {
        Step::Welcome => 0,
        Step::Forge { .. } => 1,
        Step::Model { .. } => 2,
        Step::Ready => {
            let connected = match crate::forge::connect::personal(root) {
                Some(row) => match temper_client_domain::forge::connect::step(
                    Some(temper_client_domain::Choice::Forge(row.number)),
                    root.forges(),
                    root.forge().demo(),
                ) {
                    temper_client_domain::forge::connect::Shown::Ready(_) => true,
                    temper_client_domain::forge::connect::Shown::Choose
                    | temper_client_domain::forge::connect::Shown::Details(_)
                    | temper_client_domain::forge::connect::Shown::SettingUp(_) => false,
                },
                None => false,
            };
            if connected && super::accounts::first_usable_heading(root, w).is_some() { 4 } else { 3 }
        }
    };
    crate::flow::steps(b, w, &[&welcome, &forge, &model, &ready], current);
}
fn card(b: &mut Builder, w: &Words, title: Word, description: Word, time: Word) {
    b.open(Role::Region);
    b.style(look::WELCOME_CARD);
    b.name(&w.say(title, &[]));
    text(b, w, Role::Heading { level: 2 }, title);
    text(b, w, Role::Paragraph, description);
    paragraph(b, w, time, look::WELCOME_MUTED);
}
#[expect(clippy::too_many_arguments, reason = "welcome composes the forge and accounts flows with their bindings")]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    forge: &mut crate::forge::bindings::Bindings,
    step: Step,
    text_per_node: u32,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::WELCOME);
    steps(b, root, w, step);
    let playground = root.demo_answered() && root.forge().demo().is_some();
    match step {
        Step::Welcome => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeTitle);
            if let Some(page) = root.pages().welcome()
                && page.decided
            {
                paragraph(b, w, Word::WelcomeLead, look::WELCOME_LEAD);
                b.open(Role::Slot);
                b.style(look::WELCOME_CHOICES);
                if playground {
                    card(
                        b,
                        w,
                        Word::WelcomePlayground,
                        Word::WelcomePlaygroundDescription,
                        Word::WelcomePlaygroundCost,
                    );
                    control(b, w, bindings, Word::WelcomePlayground, Act::Playground);
                    b.close();
                }
                card(b, w, Word::WelcomeSetup, Word::WelcomeSetupDescription, Word::WelcomeSetupTime);
                go(b, w, bindings, Word::WelcomeSetup, Step::Forge { forge: None });
                b.close();
                b.close();
                if playground {
                    paragraph(b, w, Word::WelcomeAfter, look::WELCOME_AFTER);
                }
            }
        }
        Step::Forge { forge: forge_choice } => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeForgeTitle);
            paragraph(b, w, Word::WelcomeForgeLead, look::WELCOME_LEAD);
            if let Some(row) = crate::forge::connect::personal(root) {
                crate::forge::connect::status(b, root, w, row, false);
                crate::forge::connect::first_line(b, root, w, row);
                crate::forge::connect::link(
                    b,
                    w,
                    bindings,
                    Word::ForgeView,
                    Address::ConnectForge { forge: Some(temper_client_domain::Choice::Forge(row.number)) },
                );
            } else {
                crate::forge::connect::body(
                    b,
                    root,
                    w,
                    bindings,
                    forge,
                    forge_choice,
                    crate::forge::connect::Place::Welcome,
                );
            }
        }
        Step::Model { .. } => {
            if let Some(row) = temper_client_domain::forge::connect::underway_forge(root.forges(), root.forge().demo())
            {
                crate::forge::connect::status(b, root, w, row, false);
                crate::forge::connect::first_line(b, root, w, row);
                crate::forge::connect::each_step(b, w, row);
            }
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeModelTitle);
            paragraph(b, w, Word::WelcomeModelLead, look::WELCOME_LEAD);
            super::connect_model::body(b, root, w, accounts, text_per_node);
        }
        Step::Ready => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeReadyTitle);
            ready(b, root, w, bindings, forge);
        }
    }
    if step != Step::Welcome {
        b.open(Role::Slot);
        b.style(look::WELCOME_FOOT);
        let back = match step {
            Step::Welcome | Step::Forge { .. } => Step::Welcome,
            Step::Model { .. } => Step::Forge { forge: None },
            Step::Ready => Step::Model { way: None },
        };
        go(b, w, bindings, Word::WelcomeBack, back);
        control(b, w, bindings, Word::WelcomeLater, Act::Go { address: Address::Chats });
        if playground {
            control(b, w, bindings, Word::WelcomePlaygroundInstead, Act::Playground);
        }
        b.close();
    }
    b.close();
}

fn ready(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    forge: &mut crate::forge::bindings::Bindings,
) {
    use crate::forge::connect;
    use skein_ui::phrases::Hole;
    b.open(Role::List);
    b.open(Role::Item);
    let row = connect::personal(root);
    if let Some(row) = row {
        let ready = temper_client_domain::forge::connect::step(
            Some(temper_client_domain::Choice::Forge(row.number)),
            root.forges(),
            root.forge().demo(),
        );
        if row.first.is_some() && ready == temper_client_domain::forge::connect::Shown::Ready(row.number) {
            connect::ready_card(b, root, w, bindings, row);
        } else if row.first.is_some() && ready == temper_client_domain::forge::connect::Shown::SettingUp(row.number) {
            connect::setting_up(b, root, w, forge, row);
        } else if row.first.is_none()
            && let Connection::Connected { login, .. } = &row.connection
        {
            crate::cards::text(
                b,
                Role::Paragraph,
                &w.say(
                    Word::WelcomeForgeConnected,
                    &[Hole::Name(connect::name(w, row, root)), Hole::Name(skein_lib::bytes::copy_of(login))],
                ),
            );
        } else {
            connect::status(b, root, w, row, true);
            if connect::underway(row) {
                connect::each_step(b, w, row);
            }
        }
    } else {
        text(b, w, Role::Paragraph, Word::WelcomeNoForge);
        go(b, w, bindings, Word::WelcomeForgeTitle, Step::Forge { forge: None });
    }
    b.close();
    b.open(Role::Item);
    if let Some(account) = super::accounts::first_usable_heading(root, w) {
        crate::cards::text(b, Role::Paragraph, &w.say(Word::WelcomeModelAccount, &[Hole::Name(account)]));
    } else {
        text(b, w, Role::Paragraph, Word::WelcomeNoModel);
        go(b, w, bindings, Word::WelcomeModelTitle, Step::Model { way: None });
    }
    b.close();
    b.close();
    if let Some(row) = row
        && row.first.is_none()
        && let Connection::Connected { .. } = row.connection
    {
        connect::adopt(b, root, w, bindings);
    }
}
