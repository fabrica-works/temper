use super::*;
use skein_ui::{Role, Run};
fn text(b: &mut skein_ui::Builder, role: Role, words: &[u8]) {
    b.open(role);
    b.text(Run::Plain, words);
    b.close();
}
fn region(b: &mut skein_ui::Builder, name: &[u8]) {
    b.open(Role::Region);
    b.name(name);
    text(b, Role::Heading { level: 2 }, name);
}
fn chip(b: &mut skein_ui::Builder, name: &[u8], title: &[u8], phase: &[u8], spent: &[u8]) {
    b.open(Role::TreeItem);
    b.name(name);
    b.open(Role::Slot);
    b.style(temper_client_view::look::PLAN_ROW);
    b.open(Role::Slot);
    b.open(Role::Link);
    b.name(name);
    b.bind(binding());
    b.close();
    text(b, Role::Paragraph, title);
    b.close();
    text(b, Role::Paragraph, b"Agent");
    text(b, Role::Status, phase);
    b.open(Role::Paragraph);
    b.style(temper_client_view::look::PLAN_SPEND);
    b.name(spent);
    b.text(Run::Plain, spent);
    b.close();
    b.close();
}
pub fn page() -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    text(&mut b, Role::Heading { level: 1 }, b"Refresh token handling");
    region(&mut b, b"Task details");
    for words in [b"Task 9".as_slice(), b"Refresh token handling", b"Agent"] {
        text(&mut b, Role::Paragraph, words);
    }
    text(&mut b, Role::Status, b"running");
    for words in [b"Shop".as_slice(), b"Asked by Pat", "Goal · priority 1".as_bytes()] {
        text(&mut b, Role::Paragraph, words);
    }
    b.close();
    region(&mut b, b"What was asked");
    text(&mut b, Role::Paragraph, "Result · report".as_bytes());
    text(&mut b, Role::Paragraph, b"Rotate tokens safely.");
    text(&mut b, Role::Paragraph, b"Shown on the forge.");
    b.close();
    region(&mut b, b"Plan");
    b.open(Role::Tree);
    b.name(b"Plan");
    chip(&mut b, b"Task 12", b"Review the change", b"Held", b"4 spent");
    golden_cards::held(&mut b);
    b.close();
    chip(&mut b, b"Task 13", b"Write the release notes", b"waiting on task 12", b"0 spent");
    b.open(Role::List);
    b.name(b"Dependencies of task 13");
    b.style(temper_client_view::look::DEPENDENCIES);
    b.open(Role::Item);
    b.key(skein_ui::Key { owner: temper_client_view::frame::owner(), kind: 0, a: 12, b: 0 });
    b.open(Role::Link);
    b.name(b"Task 12");
    b.text(Run::Plain, b"Task 12");
    b.bind(binding());
    b.close();
    text(&mut b, Role::Status, b"Held");
    b.close();
    b.close();
    b.close();
    text(&mut b, Role::Status, b"Held below: 1");
    b.close();
    b.close();
    region(&mut b, b"Latest activity");
    b.open(Role::Item);
    text(&mut b, Role::Paragraph, b"Jan 1, 00:00");
    text(&mut b, Role::Paragraph, b"The review needs attention.");
    b.close();
    b.close();
    region(&mut b, b"Budget and authority");
    text(&mut b, Role::Paragraph, "7 spent · 493 left of 500".as_bytes());
    for (term, description) in [
        (b"Delegates spent".as_slice(), b"0".as_slice()),
        (b"Reserved for delegates", b"0"),
        (b"Funded by", b"Your pool"),
        (b"Deadline", b"None"),
        (b"May spend", b"500"),
        (b"May delegate", b"Up to 1 task, 1 deep"),
        (b"Models", b"All"),
    ] {
        text(&mut b, Role::Term, term);
        text(&mut b, Role::Description, description);
    }
    b.close();
    b.close();
    b.close();
    b.finish()
}
