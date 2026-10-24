//! Parts shared by `ConnectForge`, `ConnectModel` and the welcome (client/README.md, 5.2).
use crate::{
    look,
    words::{Word, Words},
};
use skein_ui::{Binding, Builder, Role, Run, State};

pub(crate) enum Control<'a> {
    Link { name: &'a [u8], to: Binding },
    Button { name: &'a [u8], to: Binding },
    PrimaryLink { name: &'a [u8], to: Binding },
    PrimaryButton { name: &'a [u8], to: Binding },
    PrimaryDisabled { name: &'a [u8] },
    Disabled { name: &'a [u8] },
}
pub(crate) enum Mark {
    Done,
    UnderWay,
    Failed,
    ToCome,
}
/// The list along the top, named Steps, with earlier items styled done.
pub(crate) fn steps(b: &mut Builder, w: &Words, names: &[&[u8]], current: usize) {
    b.open(Role::List);
    b.name(&w.say(Word::WelcomeSteps, &[]));
    b.style(look::FLOW_STEPS);
    for (index, name) in names.iter().enumerate() {
        b.open(Role::Item);
        b.style(look::FLOW_STEP);
        if index == current {
            b.state(State::Current);
        }
        if index < current {
            b.style(look::FLOW_DONE);
        }
        b.text(Run::Plain, name);
        b.close();
    }
    b.close();
}
/// One choice's region, named by its level-2 heading.
pub(crate) fn choice(b: &mut Builder, heading: &[u8], words: &[u8], note: &[u8], action: Control<'_>) {
    card(b, heading);
    crate::cards::text(b, Role::Paragraph, words);
    b.open(Role::Paragraph);
    b.style(look::FLOW_MUTED);
    b.name(note);
    b.text(Run::Plain, note);
    b.close();
    control(b, action);
    b.close();
}
/// A list whose lines say their state in words.
pub(crate) fn progress(b: &mut Builder, w: &Words, name: &[u8], lines: &[(&[u8], Mark)]) {
    b.open(Role::List);
    b.name(name);
    b.style(look::FLOW_PROGRESS);
    for (line, mark) in lines {
        b.open(Role::Item);
        match mark {
            Mark::Done => b.style(look::FLOW_DONE),
            Mark::UnderWay => b.state(State::Current),
            Mark::Failed => b.style(look::FLOW_FAILED),
            Mark::ToCome => {}
        }
        b.text(Run::Plain, line);
        let word = match mark {
            Mark::Done => Some(Word::FlowDone),
            Mark::UnderWay => Some(Word::FlowUnderWay),
            Mark::Failed => Some(Word::FlowFailed),
            Mark::ToCome => None,
        };
        if let Some(word) = word {
            b.open(Role::Status);
            b.style(look::FLOW_MUTED);
            b.name(&w.say(word, &[]));
            w.text(b, word, &[]);
            b.close();
        }
        b.close();
    }
    b.close();
}
/// Open the step's card; the caller closes it.
pub(crate) fn card(b: &mut Builder, heading: &[u8]) {
    b.open(Role::Region);
    b.style(look::FLOW_CARD);
    b.name(heading);
    crate::cards::text(b, Role::Heading { level: 2 }, heading);
}
/// Draw the control in its given kind.
pub(crate) fn control(b: &mut Builder, action: Control<'_>) {
    let primary = match action {
        Control::PrimaryLink { .. } | Control::PrimaryButton { .. } | Control::PrimaryDisabled { .. } => true,
        Control::Link { .. } | Control::Button { .. } | Control::Disabled { .. } => false,
    };
    match action {
        Control::PrimaryLink { name, to } | Control::Link { name, to } => {
            crate::link(b, name, to);
            b.text(Run::Plain, name);
        }
        Control::PrimaryButton { name, to } | Control::Button { name, to } => {
            b.open(Role::Button);
            b.name(name);
            b.bind(to);
        }
        Control::PrimaryDisabled { name } | Control::Disabled { name } => {
            b.open(Role::Button);
            b.name(name);
            b.state(State::Disabled);
        }
    }
    if primary {
        b.style(look::FLOW_PRIMARY);
    }
    b.close();
}
