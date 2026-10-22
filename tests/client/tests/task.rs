use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step, Target};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn a_goals_plan_says_which_task_a_delegate_waits_for() {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { goal, first, second } = fixtures::goal_with_dependent_delegates(world.front());
    let driver = &world.front().engine;
    let delegates = driver.task(goal).expect("goal").delegates.len();
    let spent = driver.task(first).expect("first delegate").numbers.spent;
    let second_spent = driver.task(second).expect("dependent delegate").numbers.spent;
    assert!(spent > 0, "the row has its own committed spend");
    let budget = fixtures::task_budget(world.front(), goal);
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See {
                find: Find::named(Role::Status, format!("waiting on {delegates} delegates").as_bytes()).within(
                    Box::new([Target::ExactRole { role: Role::Region, name: b"Task details".as_slice().into() }]),
                ),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: fixtures::plan_item(second, format!("waiting on task {first}").as_bytes()),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::text(format!("{spent} spent").as_bytes()).within(Box::new([Target::ExactRole {
                    role: Role::TreeItem,
                    name: format!("Task {first}").into_bytes().into(),
                }])),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::text(format!("{second_spent} spent").as_bytes()).within(Box::new([Target::ExactRole {
                    role: Role::TreeItem,
                    name: format!("Task {second}").into_bytes().into(),
                }])),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: fixtures::dependency_item(second, Find::named(Role::Link, format!("Task {first}").as_bytes())),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::text(
                    format!("{} spent · {} left of {}", budget.numbers.spent, budget.left, budget.numbers.budget)
                        .as_bytes(),
                )
                .within(Box::new([Target::ExactRole {
                    role: Role::Region,
                    name: b"Budget and authority".as_slice().into(),
                }])),
                within: fixtures::WITHIN,
            },
            Step::Gone {
                find: Find::exactly_named(Role::TreeItem, format!("Task {goal}").as_bytes()),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_finished_dependency_says_completed_in_its_dependants_line() {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { goal, first, second } = fixtures::goal_with_a_finished_dependency(world.front());
    let budget = fixtures::task_budget(world.front(), goal);
    let spent_below = budget.numbers.spent_below;
    assert!(spent_below > 0, "the completed delegate's expense settled to its goal");
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See {
                find: fixtures::dependency_item(second, Find::named(Role::Link, format!("Task {first}").as_bytes())),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: fixtures::dependency_item(second, Find::named(Role::Status, b"Completed")),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::named(Role::Term, b"Delegates spent").within(Box::new([Target::ExactRole {
                    role: Role::Region,
                    name: b"Budget and authority".as_slice().into(),
                }])),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::exactly_named(Role::Description, format!("{spent_below}").as_bytes()).within(Box::new([
                    Target::ExactRole { role: Role::Region, name: b"Budget and authority".as_slice().into() },
                ])),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_delegates_page_shows_its_brief_run_and_report() {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { goal, first, second } = fixtures::goal_with_dependent_delegates(world.front());
    let attempt = world.front().engine.last_assignment().attempt;
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(person, open_delegate_and_return_to_goal(goal, first, attempt));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    fixtures::report_first_delegate(world.front(), first);
    world.person(person, see_reported_delegate_and_dependency(first, second, attempt));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn open_delegate_and_return_to_goal(goal: u64, first: u64, attempt: u64) -> Vec<Step<temper_client_domain::Address>> {
    vec![
        Step::See {
            find: Find::exactly_named(Role::Link, format!("Task {first}").as_bytes()).within(Box::new([
                Target::ExactRole { role: Role::TreeItem, name: format!("Task {first}").into_bytes().into() },
            ])),
            within: fixtures::WITHIN,
        },
        Step::Press {
            find: Find::exactly_named(Role::Link, format!("Task {first}").as_bytes()).within(Box::new([
                Target::ExactRole { role: Role::TreeItem, name: format!("Task {first}").into_bytes().into() },
            ])),
        },
        Step::See {
            find: Find::named(Role::Heading { level: 1 }, format!("T{first} · Investigate the change").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(b"Investigate the change")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(format!("Asked by task {goal}").as_bytes())
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::named(Role::Heading { level: 2 }, format!("Run {attempt} · live now").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(b"Findings in progress")
                .within(Box::new([Target::ExactRole { role: Role::Log, name: b"Conversation".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::Gone { find: Find::named(Role::Region, b"Task details"), within: fixtures::WITHIN },
        Step::Gone { find: Find::named(Role::Region, b"Result"), within: fixtures::WITHIN },
        Step::Press { find: Find::exactly_named(Role::Link, format!("Task {goal}").as_bytes()) },
        Step::See { find: Find::named(Role::Region, b"Task details"), within: fixtures::WITHIN },
    ]
}

fn see_reported_delegate_and_dependency(
    first: u64,
    second: u64,
    attempt: u64,
) -> Vec<Step<temper_client_domain::Address>> {
    vec![
        Step::Go { address: fixtures::task_page(first) },
        Step::See {
            find: Find::named(Role::Heading { level: 2 }, format!("Run {attempt} · ended").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::named(Role::Status, b"Completed")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Result".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(b"The change is feasible")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Result".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::Gone {
            find: Find::named(Role::Heading { level: 2 }, format!("Run {attempt} · live now").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::Go { address: fixtures::task_page(second) },
        Step::See {
            find: Find::exactly_named(Role::Link, format!("Task {first}").as_bytes()).within(Box::new([
                Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() },
                Target::ExactRole {
                    role: Role::List,
                    name: format!("Dependencies of task {second}").into_bytes().into(),
                },
            ])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::named(Role::Status, b"Completed").within(Box::new([
                Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() },
                Target::ExactRole {
                    role: Role::List,
                    name: format!("Dependencies of task {second}").into_bytes().into(),
                },
            ])),
            within: fixtures::WITHIN,
        },
    ]
}

#[test]
fn a_finished_goals_page_shows_its_report() {
    use jig_domain_tasks as tasks;
    let mut world = fixtures::world(fixtures::SEED);
    let goal = fixtures::finished_goal(world.front());
    let key = temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(goal)));
    let Some(temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
        world.front().engine.store.rows.get(&key)
    else {
        panic!("ended goal retained");
    };
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) = &row.phase else {
        panic!("goal finished with a report");
    };
    let words = words.clone();
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See {
                find: Find::named(Role::Status, b"Completed")
                    .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Result".as_slice().into() }])),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::text(&words)
                    .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Result".as_slice().into() }])),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_goals_page_shows_its_plan_with_the_held_task_marked() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, held) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Plan"), within: fixtures::WITHIN },
            Step::See { find: fixtures::plan_item(held, b"held"), within: fixtures::WITHIN },
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
#[test]
fn a_held_task_released_from_a_goals_plan_runs_again() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, held) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Button, b"Release task"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Release task") },
            Step::See { find: Find::named(Role::Dialog, b"Release task"), within: fixtures::WITHIN },
            Step::See {
                find: Find::text(b"The next run resumes its conversation; tries count again from zero"),
                within: fixtures::WITHIN,
            },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: fixtures::plan_item(held, b"running"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert_eq!(world.front().engine.task(held).expect("released delegate").tries, jig_domain_tasks::Tries::NONE);
}

#[test]
fn a_chat_reply_is_identified_by_its_task_in_the_native_inbox() {
    let mut world = fixtures::world(fixtures::SEED);
    fixtures::replied_chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    let reply = Find::named(Role::Heading { level: 3 }, b"Reply")
        .within(Box::new([skein_fake_person::Target::Role { role: Role::Item, name: Box::from(*b"Hello") }]));
    world.person(person, vec![Step::See { find: reply, within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_client_closed_on_a_tasks_page_opens_at_the_inbox_again() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, _) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Plan"), within: fixtures::WITHIN },
            Step::Reload,
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Inbox"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
