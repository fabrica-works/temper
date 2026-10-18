use super::*;
use skein_ui::{Role, Run};
fn text(b: &mut skein_ui::Builder, role: Role, words: &[u8]) {
    b.open(role);
    b.text(Run::Plain, words);
    b.close();
}
pub fn page() -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&tree_limits());
    synthetic_frame(&mut b, false);
    b.open(Role::Main);
    text(&mut b, Role::Heading { level: 1 }, b"Chat 9");
    text(&mut b, Role::Paragraph, b"Hello");
    text(&mut b, Role::Status, b"Writing");
    text(&mut b, Role::Paragraph, b"Spent 7; delegates spent 2");
    b.open(Role::Log);
    b.name(b"Conversation");
    text(&mut b, Role::Paragraph, b"Pat");
    text(&mut b, Role::Paragraph, b"Hello");
    text(&mut b, Role::Status, b"Read");
    text(&mut b, Role::Paragraph, b"Agent");
    golden_cards::report(&mut b, b"I can help.");
    text(&mut b, Role::Status, b"Writing...");
    text(&mut b, Role::Paragraph, b"Writing an answer");
    b.close();
    for (role, name) in [
        (Role::TextArea, b"Message".as_slice()),
        (Role::Button, b"Send"),
        (Role::Button, b"Stop"),
        (Role::Button, b"Close"),
        (Role::Button, b"Mark seen"),
        (Role::Button, b"Older"),
    ] {
        b.open(role);
        b.name(name);
        b.bind(binding());
        b.close();
    }
    b.close();
    b.close();
    b.finish()
}
