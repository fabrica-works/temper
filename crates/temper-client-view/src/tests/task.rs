use super::fixtures::{self, task as f};
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role};
const TASK: u64 = 9;
const CHILD: u64 = 12;
const PRIORITY: u32 = 1;
const CHARTER: u32 = 1;
const PROCEDURE: u32 = 2;
const PARTY: u64 = 73;
const ONE: u32 = 1;
const NO_ROWS: u32 = 0;
const HEADINGS: u32 = 4;
fn page(executor: c::Executor, priority: Option<u32>) -> crate::View {
    let root = f::root(executor, priority);
    let mut view = fixtures::view();
    fixtures::build(&mut view, &root);
    view
}
fn assert_header_and_plan(executor: c::Executor) {
    let view = page(executor, None);
    let tree = view.tree();
    assert!(fixtures::find(tree, Role::Heading { level: 2 }, b"Task details").is_some());
    assert!(fixtures::find(tree, Role::Heading { level: 2 }, b"Plan").is_some());
    assert_eq!(fixtures::find(tree, Role::Heading { level: 2 }, b"What was asked"), None);
    assert_eq!(fixtures::find(tree, Role::Heading { level: 2 }, b"Latest activity"), None);
}
#[test]
fn a_procedures_task_shows_its_header_and_plan_only() {
    assert_header_and_plan(c::Executor::Procedure { connector: fixtures::FORGE, kind: PROCEDURE });
}
#[test]
fn a_person_tasks_page_shows_its_header_and_plan_only() {
    assert_header_and_plan(c::Executor::Party { party: PARTY });
}
#[test]
fn an_agents_task_shows_its_header_and_plan_only() {
    assert_header_and_plan(c::Executor::Agent { charter: CHARTER });
}
#[test]
fn a_tracked_task_has_the_mockups_four_regions_in_order() {
    let view = page(c::Executor::Agent { charter: CHARTER }, Some(PRIORITY));
    let tree = view.tree();
    let mut headings = List::with_capacity(HEADINGS);
    for node in tree.nodes() {
        if node.role == (Role::Heading { level: 2 }) {
            headings.push(node.name.as_deref().expect("heading named")).expect("four headings");
        }
    }
    assert_eq!(headings.as_slice(), [b"Task details".as_slice(), b"What was asked", b"Plan", b"Latest activity"]);
}
#[test]
fn a_plans_chip_links_to_its_tasks_page_by_number() {
    let root = f::root(c::Executor::Agent { charter: CHARTER }, Some(PRIORITY));
    let mut view = fixtures::view();
    let link = fixtures::find(fixtures::build(&mut view, &root), Role::Link, b"Task 9").expect("numbered task");
    assert_eq!(
        crate::act(&mut view, link, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: temper_client_domain::Address::Task { task: TASK }
        }))
    );
}
fn asked(part: c::Part) -> skein_ui::Tree {
    let mut task = f::document();
    task.resources = List::with_capacity(ONE);
    task.resources.push(c::TaskResource::Part(part)).expect("one part");
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::pages::task::asked(&mut b, &task, &fixtures::words());
    b.finish()
}
#[test]
fn a_forge_resource_on_the_page_is_in_the_forges_words() {
    let tree = asked(c::Part::Resource {
        connector: fixtures::FORGE,
        path: fixtures::path(&[b"forge", b"forge.example", b"acme", b"shop"]),
    });
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"acme/shop on forge.example").is_some());
}
#[test]
fn a_forge_panel_on_the_page_says_shown_on_the_forge() {
    let tree = asked(c::Part::Panel { connector: fixtures::FORGE, task: TASK });
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Shown on the forge.").is_some());
}
#[test]
fn a_plans_items_are_nested_under_their_supplied_parent() {
    let mut tree = f::tree();
    let c::Delivery::OwnerTree(value) = &mut tree else { panic!("owner tree") };
    let mut chip = value.root.clone();
    chip.task = CHILD;
    value.rows = List::with_capacity(ONE);
    value
        .rows
        .push(c::OwnerTreeRow {
            chip,
            parent: TASK,
            depth: ONE,
            proposals: List::with_capacity(NO_ROWS),
            spent: 0,
            dependencies: List::with_capacity(NO_ROWS),
            cut: false,
        })
        .expect("child");
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::pages::task::plan(
        &mut b,
        &tree,
        &f::root(c::Executor::Agent { charter: CHARTER }, Some(PRIORITY)),
        &fixtures::words(),
        &fixtures::jig_words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        &mut jig_client_view::JigBindings::new(fixtures::tree_limits().bindings, TEXT_BYTES),
    );
    let tree = b.finish();
    let parent = fixtures::find(&tree, Role::TreeItem, b"Task 9").expect("parent");
    let child = fixtures::find(&tree, Role::TreeItem, b"Task 12").expect("child");
    let nodes = tree.nodes();
    let mut parent_index = None;
    let mut child_index = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.id == parent {
            parent_index = Some(index);
        }
        if node.id == child {
            child_index = Some(index);
        }
    }
    let parent_index = parent_index.expect("parent index");
    let child_index = child_index.expect("child index");
    assert!(child_index < parent_index + usize::try_from(nodes[parent_index].size).expect("parent size"));
}
#[test]
fn the_release_confirmation_says_what_releasing_does() {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = f::held_root();
    fixtures::root_event(
        &mut root,
        Event::Act {
            action: Action::Pages(Act::Decide {
                object: c::ObjectKey::Task(TASK),
                revision: Some(f::HOLD_REVISION),
                act: c::ActKind::Release,
            }),
        },
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Release task").is_some());
    let text = fixtures::text(tree);
    assert!(
        skein_lib::bytes::find(&text, b"The next run resumes its conversation; tries count again from zero").is_some()
    );
    assert!(skein_lib::bytes::find(&text, b"Hold revision 7").is_some());
}

#[test]
fn a_moved_hold_is_refused_and_the_card_asks_to_look_again() {
    let root = f::held_root();
    let mut card = root.pages().cards().expect("cards").get(FIRST_CARD).expect("held").clone();
    card.outcome = Some(jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::Moved { now: NEW_HOLD_REVISION, what: c::Moved::Hold { reason: None } },
        at: None,
    })));
    let limits = fixtures::tree_limits();
    let mut builder = Builder::new(&limits);
    crate::confirm::card(
        &mut builder,
        &root,
        &card,
        &fixtures::words(),
        &fixtures::jig_words(),
        &mut crate::frame::Bindings::new(limits.bindings),
        &mut jig_client_view::JigBindings::new(limits.bindings, TEXT_BYTES),
    );
    let text = fixtures::text(&builder.finish());
    assert!(skein_lib::bytes::find(&text, b"Inspect current object").is_some());
}
const FIRST_CARD: u32 = 0;
const NEW_HOLD_REVISION: c::Revision = c::Revision { value: 8 };
const TEXT_BYTES: u32 = 32;

#[test]
fn a_held_plans_release_has_its_public_escalation_card_identity() {
    let root = f::held_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(has_release_card(tree));
}
fn has_release_card(tree: &skein_ui::Tree) -> bool {
    for node in tree.nodes() {
        if node.role == Role::Item
            && node.key == Some(skein_ui::Key { owner: jig_client_view::Owner::Jig.opaque(), kind: 2, a: TASK, b: 0 })
        {
            return true;
        }
    }
    false
}
#[test]
fn a_released_plans_item_keeps_its_decision_result_while_running() {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = f::held_root();
    for act in [
        Act::Decide { object: c::ObjectKey::Task(TASK), revision: Some(f::HOLD_REVISION), act: c::ActKind::Release },
        Act::Confirm,
    ] {
        fixtures::root_event(&mut root, Event::Act { action: Action::Pages(act) });
    }
    let mut tree = f::tree();
    let c::Delivery::OwnerTree(value) = &mut tree else { panic!("owner tree") };
    value.root.phase = c::Phase::Running;
    let limits = fixtures::tree_limits();
    let mut builder = Builder::new(&limits);
    crate::pages::task::plan(
        &mut builder,
        &tree,
        &root,
        &fixtures::words(),
        &fixtures::jig_words(),
        &mut crate::frame::Bindings::new(limits.bindings),
        &mut jig_client_view::JigBindings::new(limits.bindings, TEXT_BYTES),
    );
    assert!(has_release_card(&builder.finish()));
}
