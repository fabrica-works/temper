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
fn asked(part: c::Part) -> Tree {
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
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        fixtures::domain_limits().client.domain.rows,
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
        &mut crate::frame::Bindings::new(limits.bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    let text = fixtures::text(&builder.finish());
    assert!(skein_lib::bytes::find(&text, b"Look again before deciding.").is_some());
}
const FIRST_CARD: u32 = 0;
const NEW_HOLD_REVISION: c::Revision = c::Revision { value: 8 };

#[test]
fn a_held_plans_release_has_its_public_escalation_card_identity() {
    let root = f::held_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(has_release_card(tree));
}
fn has_release_card(tree: &Tree) -> bool {
    for node in tree.nodes() {
        if node.role == Role::Item
            && node.key == Some(skein_ui::Key { owner: crate::frame::owner(), kind: 2, a: TASK, b: 0 })
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
        &mut crate::frame::Bindings::new(limits.bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    assert!(has_release_card(&builder.finish()));
}

use super::inbox::{styled, subtree};
use crate::look;
use skein_ui::{Node, Tree};
fn children(tree: &Tree, parent: usize) -> List<usize> {
    let nodes = tree.nodes();
    let end = parent.checked_add(usize::try_from(nodes[parent].size).expect("subtree")).expect("subtree end");
    let mut result = List::with_capacity(nodes[parent].size);
    let mut index = parent.checked_add(1).expect("first child");
    while index < end {
        result.push(index).expect("children bounded by nodes");
        index = index.checked_add(usize::try_from(nodes[index].size).expect("child size")).expect("next child");
    }
    result
}
fn named(nodes: &[Node], role: Role, name: &[u8]) -> bool {
    for node in nodes {
        if node.role == role && node.name.as_deref() == Some(name) {
            return true;
        }
    }
    false
}
fn header(task: &c::Task, me: Option<&c::Me>) -> Tree {
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::header(&mut builder, task, me, &fixtures::words());
    builder.finish()
}
fn titled() -> c::Task {
    let mut task = f::document();
    task.chip.task = TASK;
    task.chip.title =
        c::DocumentText::complete(c::Text::new(Box::from(*b"Refresh token handling"), 32).expect("title"));
    task.project = fixtures::PROJECT;
    task
}
#[test]
fn the_title_sits_between_the_headers_two_meta_rows_without_a_title_paragraph() {
    let mut task = titled();
    task.priority = Some(PRIORITY);
    task.requester = c::OwnerRequester::Person(PARTY);
    task.budget.deadline = Some(skein_lib::Wall::EPOCH);
    let root = f::held_root();
    let tree = header(&task, root.pages().me(root.client()));
    let heading = styled(&tree, look::TASK_HEADING);
    let parts = children(&tree, heading);
    assert_eq!(parts.len(), 4);
    let nodes = tree.nodes();
    let parts = parts.as_slice();
    assert_eq!(nodes[parts[0]].role, Role::Heading { level: 2 });
    assert_eq!(nodes[parts[1]].styles.as_slice(), [look::META]);
    assert_eq!(nodes[parts[2]].role, Role::Heading { level: 1 });
    assert_eq!(nodes[parts[2]].name.as_deref(), Some(task.chip.title.bytes()));
    assert_eq!(nodes[parts[3]].styles.as_slice(), [look::META]);
    assert!(!named(nodes, Role::Paragraph, task.chip.title.bytes()));
    let first = subtree(&tree, parts[1]);
    assert!(named(first, Role::Paragraph, b"Priority 1"));
    assert!(named(first, Role::Paragraph, b"Shop"));
    let meta = children(&tree, parts[1]);
    let chip = subtree(&tree, meta.as_slice()[1]);
    assert_eq!(chip[0].role, Role::Slot);
    assert!(named(chip, Role::Paragraph, b"Task 9"));
    assert!(named(chip, Role::Paragraph, b"Agent"));
    let second = subtree(&tree, parts[3]);
    assert!(named(second, Role::Paragraph, b"Asked by Pat"));
    assert_eq!(second[1].role, Role::Status);
    assert!(named(second, Role::Paragraph, b"Due Jan 1"));
}
#[test]
fn a_deadline_names_the_utc_date_and_keeps_the_time_unless_it_is_midnight() {
    let rows: [(u64, &[u8]); 4] = [
        (0, b"Due Jan 1"),
        (86_399, b"Due Jan 1, 23:59 UTC"),
        (86_400, b"Due Jan 2"),
        (86_401, b"Due Jan 2, 00:00 UTC"),
    ];
    for (seconds, expected) in rows {
        let mut task = titled();
        task.budget.deadline = Some(skein_lib::Wall::from_nanos(seconds * 1_000_000_000));
        assert!(named(header(&task, None).nodes(), Role::Paragraph, expected), "deadline {seconds}");
    }
}
#[test]
fn the_first_meta_omits_absent_priority_and_uses_the_project_fallback_and_procedure_part() {
    let mut task = titled();
    task.priority = None;
    let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner") };
    details.procedure = Some(c::OwnerPart {
        connector: fixtures::FORGE,
        kind: c::OwnerPartKind::Procedure,
        key: c::OwnerPartKey::Number(1),
    });
    let tree = header(&task, None);
    let first = subtree(&tree, styled(&tree, look::META));
    assert!(!named(first, Role::Paragraph, b"Priority 1"));
    assert!(named(
        first,
        Role::Paragraph,
        &fixtures::words()
            .say(crate::words::Word::Project, &[skein_ui::phrases::Hole::Number(u64::from(task.project))])
    ));
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Shown on the forge.").is_some());
    let root = f::held_root();
    let mut me = root.pages().me(root.client()).expect("me").clone();
    for index in 0..me.projects.len() {
        me.projects.get_mut(index).expect("project").name = None;
    }
    let tree = header(&task, Some(&me));
    assert!(named(
        tree.nodes(),
        Role::Paragraph,
        &fixtures::words()
            .say(crate::words::Word::Project, &[skein_ui::phrases::Hole::Number(u64::from(task.project))])
    ));
}
#[test]
fn the_bar_links_only_a_requesting_task_before_the_current_task_paragraph() {
    use temper_client_domain::{Action, Address, Event, pages::Act};
    for requester in [
        c::OwnerRequester::Task(7),
        c::OwnerRequester::Person(PARTY),
        c::OwnerRequester::Deployment { project: fixtures::PROJECT },
    ] {
        let limits = fixtures::domain_limits();
        let (mut root, _, watch, _) = fixtures::root_at_with_tree(Address::Task { task: TASK }, &limits);
        let mut task = titled();
        task.requester = requester;
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: watch.expect("task watch"),
                        event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                    },
                },
            },
        );
        let mut view = fixtures::page_view(&root);
        let tree = view.tree();
        let mut main = None;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::Main {
                main = Some(index);
            }
        }
        let main = main.expect("main");
        let bar = children(tree, main).as_slice()[0];
        assert_eq!(tree.nodes()[bar].styles.as_slice(), [look::PAGEBAR]);
        let parts = children(tree, bar);
        let expected = if requester == c::OwnerRequester::Task(7) { 2 } else { 1 };
        assert_eq!(parts.len(), expected);
        let current = &tree.nodes()[*parts.as_slice().last().expect("current")];
        assert_eq!(current.role, Role::Paragraph);
        assert_eq!(current.name.as_deref(), Some(b"Task 9".as_slice()));
        if expected == 2 {
            let link = &tree.nodes()[parts.as_slice()[0]];
            assert_eq!(link.role, Role::Link);
            assert_eq!(link.name.as_deref(), Some(b"Task 7".as_slice()));
            let id = link.id;
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Go { address: Address::Task { task: 7 } }))
            );
        }
    }
}
const PHASES: [(c::Phase, skein_ui::Style); 14] = [
    (c::Phase::Due, look::STATUS_ACTIVE),
    (c::Phase::Preparing, look::STATUS_ACTIVE),
    (c::Phase::Claimed, look::STATUS_ACTIVE),
    (c::Phase::Running, look::STATUS_ACTIVE),
    (c::Phase::Active, look::STATUS_ACTIVE),
    (c::Phase::Done, look::STATUS_DONE),
    (c::Phase::Held, look::STATUS_HELD),
    (c::Phase::Failed, look::STATUS_HELD),
    (c::Phase::BackingOff, look::STATUS_RETRYING),
    (c::Phase::Waiting, look::STATUS_QUIET),
    (c::Phase::Idle, look::STATUS_QUIET),
    (c::Phase::Closing, look::STATUS_QUIET),
    (c::Phase::Ended, look::STATUS_QUIET),
    (c::Phase::Cancelled, look::STATUS_QUIET),
];
#[test]
fn every_phase_has_the_same_status_token_in_the_header_and_its_plan_row() {
    let root = f::held_root();
    for (phase, token) in PHASES {
        let mut task = titled();
        task.chip.phase = phase;
        let header = header(&task, None);
        assert_eq!(fixtures::styles(&header, Role::Status).as_ref(), [Box::from([token])]);
        let mut delivery = f::tree();
        let c::Delivery::OwnerTree(tree) = &mut delivery else { panic!("owner") };
        tree.root = task.chip;
        let mut builder = Builder::new(&fixtures::tree_limits());
        crate::pages::task::plan(
            &mut builder,
            &delivery,
            &root,
            &fixtures::words(),
            &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
            fixtures::domain_limits().client.domain.rows,
        );
        let tree = builder.finish();
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::TreeItem {
                let row = children(&tree, index).as_slice()[0];
                assert_eq!(tree.nodes()[row].styles.as_slice(), [look::PLAN_ROW]);
                let parts = subtree(&tree, row);
                assert!(named(parts, Role::Link, b"Task 9"));
                let mut count = 0_u32;
                for status in parts {
                    if status.role == Role::Status {
                        count = count.checked_add(1).expect("bounded statuses");
                        assert_eq!(status.styles.as_slice(), [token]);
                    }
                }
                assert_eq!(count, 1);
            }
        }
    }
}
#[test]
fn the_held_card_keeps_its_key_in_a_list_after_its_plan_row() {
    let view = fixtures::page_view(&f::held_root());
    let tree = view.tree();
    let hold = styled(tree, look::PLAN_HOLD);
    assert_eq!(tree.nodes()[hold].role, Role::List);
    let items = children(tree, hold);
    assert_eq!(items.len(), 1);
    let item = &tree.nodes()[items.as_slice()[0]];
    assert_eq!(item.role, Role::Item);
    assert_eq!(item.key, Some(skein_ui::Key { owner: crate::frame::owner(), kind: 2, a: TASK, b: 0 }));
    let mut parent = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::TreeItem {
            parent = Some(index);
            break;
        }
    }
    let parent = parent.expect("parent");
    let parts = children(tree, parent);
    assert_eq!(parts.as_slice(), [styled(tree, look::PLAN_ROW), hold]);
}
pub(super) fn activity_tree(rows: bool) -> Tree {
    let mut page = c::ActivityPage { rows: List::with_capacity(1), after: None, before: None, cut: false };
    if rows {
        page.rows
            .push(c::ActivityLine {
                position: 1,
                at: skein_lib::Wall::EPOCH,
                event: c::Activity::News {
                    topic: 1,
                    summary: c::Text::new(Box::from(*b"Ready for review"), 32).expect("news"),
                    part: None,
                },
            })
            .expect("row");
    }
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::latest(&mut builder, Some(&page), &fixtures::words());
    builder.finish()
}
#[test]
fn activity_rows_have_the_time_first_and_words_in_a_slot_and_no_empty_list() {
    let tree = activity_tree(true);
    let events = styled(&tree, look::EVENTS);
    assert_eq!(tree.nodes()[events].role, Role::List);
    for item in children(&tree, events).as_slice() {
        assert_eq!(tree.nodes()[*item].role, Role::Item);
        let parts = children(&tree, *item);
        assert_eq!(parts.len(), 2);
        assert_eq!(tree.nodes()[parts.as_slice()[0]].role, Role::Paragraph);
        assert_eq!(tree.nodes()[parts.as_slice()[0]].name.as_deref(), Some(b"Jan 1, 00:00".as_slice()));
        assert_eq!(tree.nodes()[parts.as_slice()[1]].role, Role::Slot);
        assert!(
            named(subtree(&tree, parts.as_slice()[1]), Role::Text, b"Ready for review")
                || skein_lib::bytes::find(&fixtures::text(&tree), b"Ready for review").is_some()
        );
    }
    for node in activity_tree(false).nodes() {
        assert!(!node.styles.as_slice().contains(&look::EVENTS));
    }
}
#[test]
fn asked_resources_have_a_meta_row_only_when_there_are_parts() {
    let tree = asked(c::Part::Panel { connector: fixtures::FORGE, task: TASK });
    let meta = styled(&tree, look::META);
    assert_eq!(tree.nodes()[meta].role, Role::Slot);
    let mut found = false;
    for node in subtree(&tree, meta) {
        for run in &node.text {
            found |= run.text.as_ref() == b"Shown on the forge.";
        }
    }
    assert!(found);
    let mut builder = Builder::new(&fixtures::tree_limits());
    let mut task = titled();
    task.resources = List::with_capacity(0);
    task.context = None;
    crate::pages::task::asked(&mut builder, &task, &fixtures::words());
    for node in builder.finish().nodes() {
        assert!(!node.styles.as_slice().contains(&look::META));
    }
}
