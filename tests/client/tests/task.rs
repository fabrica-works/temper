use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;
#[test]
fn a_goals_page_shows_its_plan_with_the_held_task_marked() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, held) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Plan"), within: fixtures::WITHIN },
            Step::See { find: fixtures::plan_item(held, b"Held"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
#[test]
fn a_task_with_the_chat_label_opens_as_its_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::task_page(task));
    world.person(person, vec![Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
#[test]
fn a_forge_panel_on_a_goals_page_says_shown_on_the_forge() {
    let mut world = fixtures::world(fixtures::SEED);
    let goal = fixtures::goal_with_forge_panel(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(person, vec![Step::See { find: Find::text(b"Shown on the forge."), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
