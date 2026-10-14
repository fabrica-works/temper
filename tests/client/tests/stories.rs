use jig_client_world::harness::Until;
use jig_fake_person::{Find, Step};
use jig_view::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn signing_in_shows_the_inbox_from_the_engine_root() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Status, b"Live"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Status, b"Signed in as Person"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Inbox"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().sign_ins, 1, "one sign-in reached the engine root");
}
