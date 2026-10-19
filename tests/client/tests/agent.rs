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
