use super::fixtures;
use skein_ui::{Act, Role};
const GOAL: u64 = 7;
#[test]
fn entries_show_proposals_questions_held_and_person_tasks_then_updates() {
    let root = fixtures::inbox_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let mut headings = skein_lib::List::with_capacity(8);
    for node in tree.nodes() {
        if node.role == (Role::Heading { level: 2 }) || node.role == (Role::Heading { level: 3 }) {
            headings.push(node.name.as_deref().expect("named heading")).expect("eight headings");
        }
    }
    assert_eq!(
        headings.as_slice(),
        [b"Needs you".as_slice(), b"Proposal", b"Question", b"Held", b"Person task", b"Updates", b"Updates", b"Reply"]
    );
}
#[test]
fn an_entry_naming_a_task_links_to_its_page_by_number() {
    let root = fixtures::inbox_root();
    let mut view = fixtures::view();
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::Link, b"Task 9").expect("numbered task link");
    assert_eq!(
        crate::act(&mut view, node, Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: temper_client_domain::Address::Task { task: fixtures::INBOX_TASK }
        }))
    );
}
#[test]
fn the_goal_filter_keeps_its_project_and_kind() {
    let filter = temper_client_domain::address::Filter {
        project: Some(fixtures::PROJECT),
        kind: Some(temper_client_domain::address::Kind::Question),
        goal: None,
    };
    let root = fixtures::inbox_root_filtered(filter);
    let mut view = fixtures::view();
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::TextField, b"Goal").expect("goal filter");
    assert_eq!(
        crate::act(&mut view, node, Act::Input { value: Box::from(*b"7") }),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Filter {
            filter: temper_client_domain::address::Filter { goal: Some(GOAL), ..filter }
        }))
    );
}
