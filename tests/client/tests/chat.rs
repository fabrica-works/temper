use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;
#[test]
fn a_message_in_a_chat_is_answered_in_its_conversation() {
    let mut world = fixtures::world(fixtures::SEED);
    world.front().answer_runs = true;
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::chat_page(task));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Message"), words: Box::from(*b"And then?") },
            Step::Press { find: Find::named(Role::Button, b"Send") },
            Step::See { find: Find::text(b"And then?"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Answered by the run"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
#[test]
fn a_task_with_the_chat_label_opens_as_a_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world
        .client(fixtures::PARTY, temper_client_domain::Address::Jig(jig_client_view::address::Address::task(task)));
    world.person(person, vec![Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}

const DIAGNOSTIC_NOW: skein_lib::Time = skein_lib::Time::ZERO;
const DIAGNOSTIC_WALL: skein_lib::Wall = skein_lib::Wall::EPOCH;
const POLL_ROUNDS: usize = 12;
const START_OFFSET: i32 = 0;
const DIAGNOSTIC_CLIENT: jig_client_world::harness::ClientId = jig_client_world::harness::ClientId(0);
#[test]
fn prepared_chat_answer_reaches_the_pages_task_watch() {
    use jig_client_world::harness::{App, Duty, Front};
    use temper_client_world::{app::Temper, front::Joint};
    let limits = fixtures::domain_limits();
    let mut front = Joint::new(fixtures::SEED, limits);
    front.answer_runs = true;
    let task = fixtures::chat(&mut front, b"Hello");
    front.bind(DIAGNOSTIC_CLIENT, fixtures::PARTY);
    let (mut root, _) = Temper::new(fixtures::SEED);
    let env = skein_lib::Env { now: DIAGNOSTIC_NOW, wall: DIAGNOSTIC_WALL, limits };
    let mut out = skein_lib::Queue::with_capacity(Temper::max_out(&limits));
    Temper::step(&mut root, &env, Temper::start(fixtures::chat_page(task), Vec::new(), START_OFFSET), &mut out);
    for _ in 0..POLL_ROUNDS {
        Temper::take(&mut root, &env, &mut out);
        while let Some(request) = out.pop() {
            if let Duty::Wire(request) = Temper::duty(request) {
                front.wire(DIAGNOSTIC_CLIENT, env.now, env.wall, request);
            }
        }
        for (_, event) in front.poll(env.now, env.wall) {
            front.received(DIAGNOSTIC_CLIENT, &event);
            Temper::step(&mut root, &env, event, &mut out);
        }
    }
    let chat = root.pages().chat().expect("chat page");
    assert!(front.engine.task(task).is_none(), "the answer ends the real run");
    assert_eq!(chat.task(root.client()).expect("task document").chip.phase, jig_client::contract::Phase::Ended);
}
