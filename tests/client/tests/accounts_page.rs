use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;
#[test]
fn an_account_act_is_answered_through_the_page() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::project());
    let heading = Find::exactly_named(Role::Heading { level: 1 }, b"Accounts");
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Link, b"Accounts"), within: fixtures::WITHIN },
            Step::See { find: Find::exactly_named(Role::Status, b"Live"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"Accounts") },
            Step::See { find: heading.clone(), within: fixtures::WITHIN },
            Step::Type {
                find: Find::exactly_named(Role::TextField, b"Provider number"),
                words: Box::from(b"17".as_slice()),
            },
            Step::Type {
                find: Find::exactly_named(Role::TextField, b"Key (pasted once)"),
                words: Box::from(b"private-page-key".as_slice()),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Add key") },
            Step::Pause { span: skein_lib::Duration::from_millis(50) },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let root = world.root(person).person();
    let form = root.pages().accounts().expect("accounts page");
    assert!(matches!(
        form.pending.as_ref().expect("answered account operation").outcome,
        Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Refused { .. })))
    ));
    assert!(!form.has_key());
    assert!(world.kept(person).is_empty(), "a one-shot value has no small-store record");
}
