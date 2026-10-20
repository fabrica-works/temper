//! Signing in by the printed link (client/browser.md, 4.1, story 1): the page takes the secret from its own
//! address, puts `/` in its place, and the browser keeps jig's cookie.
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Tier, WITHIN};

#[test]
fn the_printed_link_signs_in() {
    let tier = Tier::new("printed-link");
    let inbox = Find::exactly_named(Role::Heading { level: 1 }, b"Inbox");
    let mut script = Script::new(Box::new([
        Step::Person(PersonStep::See { find: inbox, within: WITHIN }),
        Step::Person(PersonStep::See { find: tier.states().live, within: WITHIN }),
        Step::Address(Box::from(b"/".as_slice())),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
