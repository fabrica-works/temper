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
    let accounts = || Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model");
    let chats = || Find::exactly_named(Role::Heading { level: 1 }, b"Chats");
    let address = || Find::text(b"Open the sign-in page");
    let mut script = Script::new(Box::new([
        see(Find::exactly_named(Role::Link, b"Set up temper")),
        Step::Person(PersonStep::Go { address: Box::from(b"/accounts/connect".as_slice()) }),
        see(accounts()),
        Step::Address(Box::from(b"/accounts/connect".as_slice())),
        see(Find::exactly_named(Role::Link, b"Use an API key")),
        press(b"Use an API key", Role::Link),
        input(b"Key (pasted once)", b"private-browser-key"),
        press(b"Add key", Role::Button),
        see(Find::text(b"Added your Test provider API key")),
        press(b"Chats", Role::Link),
        see(chats()),
        Step::Back,
        see(accounts()),
        // The browser kit refuses typing into a nonempty field: this proves re-entry after clearing.
        input(b"Key (pasted once)", b"another-private-key"),
        press("‹ All ways".as_bytes(), Role::Link),
        press(b"Sign in to Test provider", Role::Button),
        see(address()),
        press(b"Chats", Role::Link),
        see(chats()),
        Step::Back,
        see(accounts()),
        see(Find::exactly_named(Role::Button, b"Sign in to Test provider")),
        Step::Address(Box::from(b"/accounts/connect?provider=17&way=sign-in".as_slice())),
        Step::Forward,
        see(chats()),
        Step::Address(Box::from(b"/chats".as_slice())),
        Step::Back,
        see(accounts()),
        // Leaving the flow drops its sign-in state; the accounts watch retains the live source.
        press(b"Accounts", Role::Link),
        see(Find::exactly_named(Role::Heading { level: 2 }, b"Your accounts")),
        see(Find::named(Role::Disclosure, b"Test provider API key")),
        Step::Person(PersonStep::Press { find: Find::named(Role::Disclosure, b"Test provider API key") }),
        see(Find::exactly_named(Role::Button, b"Replace")),
        see(Find::exactly_named(Role::Button, b"Remove account")),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
