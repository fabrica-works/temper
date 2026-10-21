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
fn chip(b: &mut skein_ui::Builder, name: &[u8], title: &[u8], phase: &[u8]) {
    b.open(Role::TreeItem);
    b.name(name);
    b.open(Role::Link);
    b.name(name);
    b.bind(binding());
    b.close();
    text(b, Role::Paragraph, title);
    text(b, Role::Paragraph, b"Agent");
    text(b, Role::Status, phase);
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
    text(&mut b, Role::Status, b"Running");
    for words in [b"Shop".as_slice(), b"Asked by Pat", b"Priority 1"] {
        text(&mut b, Role::Paragraph, words);
    }
    b.close();
    region(&mut b, b"What was asked");
    text(&mut b, Role::Paragraph, b"Rotate tokens safely.");
    text(&mut b, Role::Paragraph, b"Shown on the forge.");
    b.close();
    region(&mut b, b"Plan");
    b.open(Role::Tree);
    b.name(b"Plan");
    chip(&mut b, b"Task 9", b"Refresh token handling", b"Running");
    chip(&mut b, b"Task 12", b"Review the change", b"Held");
    golden_cards::held(&mut b);
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
    b.close();
    b.close();
    b.finish()
}
