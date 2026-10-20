//! Owner's 16:52Z departure: host tests own masking, clearing patches and targets;
//! one Chromium story owns visible address text, field re-entry and accounts history.
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Tier, WITHIN};
#[test]
fn the_accounts_page_returns_after_a_key_send() {
    let tier = Tier::accounts("accounts-history");
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |name: &[u8], role| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let input = |name: &[u8], value: &[u8]| {
        Step::Person(PersonStep::Type { find: Find::exactly_named(Role::TextField, name), words: Box::from(value) })
    };
    let accounts = || Find::exactly_named(Role::Heading { level: 1 }, b"Accounts");
    let chats = || Find::exactly_named(Role::Heading { level: 1 }, b"Chats");
    let address = || Find::text(b"Open the sign-in page");
    let mut script = Script::new(Box::new([
        see(tier.states().live),
        press(b"Accounts", Role::Link),
        see(accounts()),
        Step::Address(Box::from(b"/accounts".as_slice())),
        input(b"Provider number", b"17"),
        input(b"Key (pasted once)", b"private-browser-key"),
        press(b"Add key", Role::Button),
        see(Find::text(b"Key kept as source")),
        // The browser kit refuses typing into a nonempty field: this proves re-entry after clearing.
        input(b"Key (pasted once)", b"another-private-key"),
        press(b"Cancel", Role::Button),
        press(b"Sign in with provider", Role::Button),
        see(address()),
        press(b"Chats", Role::Link),
        see(chats()),
        Step::Back,
        see(accounts()),
        see(address()),
        Step::Address(Box::from(b"/accounts".as_slice())),
        Step::Forward,
        see(chats()),
        Step::Address(Box::from(b"/chats".as_slice())),
        Step::Back,
        see(accounts()),
        // Drain the live sign-in within this fixture's short public deadline before closing its server.
        see(Find::text(b"No sign-ins in progress.")),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
