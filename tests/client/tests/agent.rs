use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step};
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn a_chat_the_agent_starts_is_seen_by_the_person() {
    let mut world = fixtures::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let number = fixtures::give(&mut world, agent, &[b"chat", b"--project", b"1", b"--key", b"hello", b"Hello"]);
    let printed = fixtures::answered(&mut world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    let task = fixtures::done_task(&printed);
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(person, vec![Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}

fn call(
    world: &mut jig_client_world::harness::World<temper_client_world::app::Temper, temper_client_world::front::Joint>,
    agent: jig_client_world::harness::ClientId,
    words: &[&[u8]],
) -> temper_client_tools::jig::Printed {
    let number = fixtures::give(world, agent, words);
    fixtures::answered(world, agent, number)
}

// The fixture forge has repository acme/shop as 7 on forge 1.
const FORGE: u16 = 1;
const REPOSITORY: u32 = 7;
// A successful adoption is one admitted application, per protocol/forge.md.
const ADOPTIONS: usize = 1;
#[test]
fn a_repository_the_agent_adopts_is_held_by_the_engine() {
    let mut world = fixtures::world(fixtures::SEED);
    let agent = fixtures::agent(&mut world);
    let printed = call(
        &mut world,
        agent,
        &[
            b"adopt",
            b"--project",
            b"1",
            b"--owner",
            b"acme",
            b"--name",
            b"shop",
            b"--landing",
            b"main",
            b"--key",
            b"shop",
        ],
    );
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(world.front().engine_adopted(FORGE, REPOSITORY));
    assert_eq!(world.front().applications_admitted(), ADOPTIONS);
}

#[test]
fn a_held_task_the_agent_releases_from_the_printed_act_runs_again() {
    let mut world = fixtures::world(fixtures::SEED);
    let (_goal, held) = fixtures::goal_with_held_delegate(world.front());
    let agent = fixtures::agent(&mut world);
    let printed = call(&mut world, agent, &[b"task", held.to_string().as_bytes()]);
    let mut words = fixtures::printed_act(&printed.stdout, b"release");
    for word in &mut words {
        if word.as_ref() == b"NAME" {
            *word = Box::from(*b"release");
        }
    }
    let words: Vec<&[u8]> = words.iter().map(AsRef::as_ref).collect();
    let printed = call(&mut world, agent, &words);
    assert_eq!((printed.status, printed.stdout.as_ref()), (0, format!("done: task {held}, release\n").as_bytes()));
    assert_eq!(world.front().engine.task(held).expect("released delegate").tries, jig_domain_tasks::Tries::NONE);
}
