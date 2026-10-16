use jig_client_world::harness::{Front, Until};
use skein_fake_person::{Find, Step};
use skein_ui::Role;
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

#[test]
fn a_double_press_of_start_makes_one_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::conversations());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1, "one chat task for the two presses");
}
#[test]
fn a_chat_started_on_jigs_page_is_answered_by_the_run() {
    let mut world = fixtures::world(fixtures::SEED);
    world.front().engine.hello(skein_lib::Token::new(7));
    world.front().answer_runs = true;
    let person = world.client(fixtures::PARTY, fixtures::conversations());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Answered by the run"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}
#[test]
fn a_held_chats_escalation_released_from_its_task_page() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::held_chat(world.front());
    let address = temper_client_domain::Address::Jig(jig_domain_client::address::Address::task(task));
    let person = world.client(fixtures::PARTY, address);
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Release"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release") },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::Press { find: Find::exactly_named(Role::Tab, b"Activity") },
            Step::See { find: Find::text(b"Released"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().engine.task(task).expect("released chat").tries, jig_domain_tasks::Tries::NONE);
}
#[test]
fn signed_out_mid_request_is_made_once_after_signing_in() {
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::conversations());
    world.person(
        person,
        vec![Step::See { find: Find::named(Role::TextArea, b"First words"), within: fixtures::WITHIN }],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    world.fault(jig_client_world::harness::FrontFault::SignedOut);
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"First words"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"First words"), words: Box::from(*b"Hello") },
            Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
            Step::See { find: Find::text(b"Signed out"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 0);
    world.reload(person);
    let now = world.now();
    world.front().sign_in(person, now, skein_lib::Wall::from_nanos(now.as_nanos()));
    world.person(person, vec![Step::See { find: Find::text(b"Hello"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(fixtures::chats(&world.front().engine), 1);
}
