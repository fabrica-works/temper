//! The pages a browser moves between (client/browser.md, 4.1, stories 2 to 4): back and forward between the chats
//! and a chat, an address that names no page opens the inbox at `/`, Enter starts one chat where Shift+Enter breaks
//! its line, and a panic shows the failure page.
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{
    Harness, Outcome, Script, Step,
    stories::{self, Shown},
};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, FIRST_CHAT, FIRST_CHAT_ADDRESS, Tier, WITHIN};

#[test]
fn back_then_forward_returns_to_the_chat() {
    let tier = Tier::new("back-then-forward");
    let mut started = Script::new(temper_client_browser_world::start_chat(b"Hello").into_boxed_slice());
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut started]), Outcome::Done, "the chat to return to");
    let chats = Shown { address: b"/chats", sees: Find::exactly_named(Role::Heading { level: 1 }, b"Chats") };
    let chat = Shown { address: FIRST_CHAT_ADDRESS, sees: Find::exactly_named(Role::Heading { level: 1 }, FIRST_CHAT) };
    assert_eq!(stories::back_and_forward(&tier.setup(), chats, chat, tier.states().live), Outcome::Done);
}

#[test]
fn a_non_page_address_opens_the_inbox_at_slash() {
    let tier = Tier::new("non-page-address");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let mut script = Script::new(Box::new([
        see(tier.states().live),
        Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") }),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Chats")),
        Step::Person(PersonStep::Go { address: Box::from(b"/nowhere".as_slice()) }),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Inbox")),
        Step::Address(Box::from(b"/".as_slice())),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn enter_starts_one_chat_keeping_a_shift_enter_break() {
    let tier = Tier::new("composer");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let composer = || Find::exactly_named(Role::TextArea, b"Start a chat");
    let chats = || Step::Person(PersonStep::Press { find: Find::exactly_named(Role::Link, b"Chats") });
    let mut script = Script::new(Box::new([
        see(tier.states().live),
        chats(),
        see(composer()),
        Step::Type { find: composer(), words: Box::from(b"Hello".as_slice()) },
        Step::KeyWithShift(Key::Enter),
        Step::Type { find: composer(), words: Box::from(b"there".as_slice()) },
        Step::Key(Key::Enter),
        see(Find::exactly_named(Role::Heading { level: 1 }, FIRST_CHAT)),
        Step::Address(Box::from(FIRST_CHAT_ADDRESS)),
        see(Find::exactly_named(Role::Paragraph, b"Hello")),
        chats(),
        see(Find::exactly_named(Role::Link, FIRST_CHAT)),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Link, b"Chat 2"), within: ABSENT }),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn a_panic_shows_the_failure_page() {
    let tier = Tier::new("panic");
    assert_eq!(stories::panic(&tier.setup(), b"/test/panic", Find::text(b"The application failed.")), Outcome::Done);
}
