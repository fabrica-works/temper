//! The first run's welcome and shared account ways (client/README.md, 3.5, 5.6).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run};
use temper_client_domain::{Address, Domain, Step, pages::Act};

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
fn steps(b: &mut Builder, w: &Words, current: Step) {
    let welcome = w.say(Word::WelcomeStep, &[]);
    let forge = w.say(Word::WelcomeForgeStep, &[]);
    let model = w.say(Word::WelcomeModelStep, &[]);
    let ready = w.say(Word::WelcomeReadyStep, &[]);
    let current = match current {
        Step::Welcome => 0,
        Step::Forge => 1,
        Step::Model => 2,
        Step::Ready => 3,
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
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    step: Step,
    text_per_node: u32,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::WELCOME);
    steps(b, w, step);
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
                go(b, w, bindings, Word::WelcomeSetup, Step::Forge);
                b.close();
                b.close();
                if playground {
                    paragraph(b, w, Word::WelcomeAfter, look::WELCOME_AFTER);
                }
            }
        }
        Step::Forge => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeForgeTitle);
            paragraph(b, w, Word::WelcomeForgeLead, look::WELCOME_LEAD);
            text(b, w, Role::Paragraph, Word::WelcomeForgeUnavailable);
            go(b, w, bindings, Word::WelcomeModelTitle, Step::Model);
        }
        Step::Model => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeModelTitle);
            paragraph(b, w, Word::WelcomeModelLead, look::WELCOME_LEAD);
            if let Some(form) = root.pages().accounts() {
                super::accounts::add_account(b, root, w, accounts, form, text_per_node);
            }
        }
        Step::Ready => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeReadyTitle);
            text(b, w, Role::Paragraph, Word::WelcomeReadyUnavailable);
        }
    }
    if step != Step::Welcome {
        b.open(Role::Slot);
        b.style(look::WELCOME_FOOT);
        let back = match step {
            Step::Welcome | Step::Forge => Step::Welcome,
            Step::Model => Step::Forge,
            Step::Ready => Step::Model,
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
