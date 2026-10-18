use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn a_held_chats_escalation_released_from_the_inbox() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::held_chat(world.front());
    let address = fixtures::inbox();
    let person = world.client(fixtures::PARTY, address);
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Release task"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release task") },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Released"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().engine.task(task).expect("released chat").tries, jig_domain_tasks::Tries::NONE);
}

#[test]
fn releasing_says_what_releasing_does_before_it_is_sent() {
    let mut world = fixtures::world(fixtures::SEED);
    fixtures::held_chat(world.front());
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Release task"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release task") },
            Step::See {
                find: Find::text(b"The next run resumes its conversation; tries count again from zero"),
                within: fixtures::WITHIN,
            },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
