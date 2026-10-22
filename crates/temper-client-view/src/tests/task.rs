use super::fixtures::{self, task as f};
use jig_client::contract as c;
use skein_lib::{Decimal, List, Writer};
use skein_ui::{Builder, Role};
const TASK: u64 = 9;
const CHILD: u64 = 12;
const PRIORITY: u32 = 1;
const CHARTER: u32 = 1;
const PROCEDURE: u32 = 2;
const PARTY: u64 = 73;
const ONE: u32 = 1;
const NO_ROWS: u32 = 0;
const HEADINGS: u32 = 5;
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
    assert_eq!(fixtures::find(tree, Role::Region, b"Result"), None);
    assert_eq!(fixtures::find(tree, Role::Region, b"Budget and authority"), None);
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
fn a_tracked_task_has_its_regions_in_order() {
    let view = page(c::Executor::Agent { charter: CHARTER }, Some(PRIORITY));
    let tree = view.tree();
    let mut headings = List::with_capacity(HEADINGS);
    for node in tree.nodes() {
        if node.role == (Role::Heading { level: 2 }) {
            headings.push(node.name.as_deref().expect("heading named")).expect("page headings");
        }
    }
    assert_eq!(
        headings.as_slice(),
        [b"Task details".as_slice(), b"What was asked", b"Plan", b"Latest activity", b"Budget and authority"]
    );
}
#[test]
fn a_plans_chip_links_to_its_tasks_page_by_number() {
    let mut delivery = f::tree();
    let c::Delivery::OwnerTree(owner) = &mut delivery else { panic!("owner tree") };
    owner.rows = List::with_capacity(ONE);
    owner.rows.push(row()).expect("row");
    let mut builder = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::task::plan(
        &mut builder,
        &delivery,
        &f::held_root(),
        &fixtures::words(),
        &mut bindings,
        fixtures::domain_limits().client.domain.rows,
    );
    let tree = builder.finish();
    let link = &tree.nodes()[index_of(&tree, Role::Link, b"Task 12")];
    assert_eq!(
        bindings.decode(link.binding.expect("bound").token, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: temper_client_domain::Address::Task { task: CHILD }
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
fn a_delegate_branch_resource_on_the_page_is_in_words() {
    let tree = asked(c::Part::Resource {
        connector: fixtures::FORGE,
        path: fixtures::path(&[&1_u16.to_be_bytes(), &1_u32.to_be_bytes(), b"temper/1/c2"]),
    });
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"branch temper/1/c2 of repository 1 on forge 1").is_some());
}
#[test]
fn a_forge_panel_on_the_page_says_shown_on_the_forge() {
    let tree = asked(c::Part::Panel { connector: fixtures::FORGE, task: TASK });
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Shown on the forge.").is_some());
}
#[test]
fn a_plans_items_are_nested_under_their_supplied_parent() {
    let mut delivery = f::tree();
    let c::Delivery::OwnerTree(owner) = &mut delivery else { panic!("owner tree") };
    owner.rows = List::with_capacity(2);
    owner.rows.push(row()).expect("row");
    let mut child = row();
    child.chip.task = GRANDCHILD;
    child.parent = CHILD;
    child.depth = 2;
    owner.rows.push(child).expect("child");
    let tree = plan(&delivery);
    let parent_index = index_of(&tree, Role::TreeItem, b"Task 12");
    let child_index = index_of(&tree, Role::TreeItem, b"Task 13");
    assert!(child_index < parent_index + usize::try_from(tree.nodes()[parent_index].size).expect("parent size"));
    assert!(fixtures::find(&tree, Role::TreeItem, b"Task 9").is_none());
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
    assert!(named(first, Role::Paragraph, "Goal · priority 1".as_bytes()));
    assert!(named(first, Role::Paragraph, b"Shop"));
    let meta = children(&tree, parts[1]);
    let chip = subtree(&tree, meta.as_slice()[1]);
    assert_eq!(chip[0].role, Role::Slot);
    assert!(named(chip, Role::Paragraph, b"Task 9"));
    assert!(named(chip, Role::Paragraph, b"Agent"));
    let second = subtree(&tree, parts[3]);
    assert!(named(second, Role::Paragraph, b"Asked by Pat"));
    assert_eq!(second[1].role, Role::Status);
    assert!(!named(second, Role::Paragraph, b"Due Jan 1"));
    assert_eq!(children(&tree, parts[3]).len(), 2);
}
#[test]
#[expect(clippy::manual_map, reason = "the client matches without a closure")]
fn the_budget_deadline_names_the_utc_date_or_moment_and_none() {
    for (seconds, expected) in [
        (Some(0), b"Jan 1".as_slice()),
        (Some(86_399), b"Jan 1, 23:59 UTC"),
        (Some(86_400), b"Jan 2"),
        (Some(86_401), b"Jan 2, 00:00 UTC"),
        (None, b"None"),
    ] {
        let mut task = titled();
        task.budget.deadline = match seconds {
            Some(seconds) => Some(skein_lib::Wall::from_nanos(seconds * 1_000_000_000)),
            None => None,
        };
        let view = goal(task.clone());
        let tree = view.tree();
        let budget = region_index(tree, b"Budget and authority");
        let parts = children(tree, budget);
        let parts = parts.as_slice();
        assert_eq!(tree.nodes()[parts[8]].name.as_deref(), Some(b"Deadline".as_slice()));
        let deadline = &tree.nodes()[parts[9]];
        assert_eq!(deadline.role, Role::Description);
        assert_eq!(deadline.name.as_deref(), Some(expected));
        assert_eq!(fixtures::node_text(tree, parts[9]).as_ref(), expected);
        let header = header(&task, None);
        let rows = children(&header, styled(&header, look::TASK_HEADING));
        assert_eq!(children(&header, rows.as_slice()[3]).len(), 2, "header has status and requester only");
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
    assert!(!named(first, Role::Paragraph, "Goal · priority 1".as_bytes()));
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
        task.waits_on = c::WaitsOn::Nothing;
        let header = header(&task, None);
        assert_eq!(fixtures::styles(&header, Role::Status).as_ref(), [Box::from([token])]);
        let mut delivery = f::tree();
        let c::Delivery::OwnerTree(tree) = &mut delivery else { panic!("owner") };
        let mut row = row();
        row.chip = task.chip;
        row.chip.task = CHILD;
        tree.rows = List::with_capacity(ONE);
        tree.rows.push(row).expect("row");
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
                assert!(named(parts, Role::Link, b"Task 12"));
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
fn the_goals_held_card_keeps_its_key_without_a_goal_row() {
    let view = fixtures::page_view(&f::held_root());
    let tree = view.tree();
    let hold = styled(tree, look::PLAN_HOLD);
    assert_eq!(tree.nodes()[hold].role, Role::List);
    let items = children(tree, hold);
    assert_eq!(items.len(), 1);
    let item = &tree.nodes()[items.as_slice()[0]];
    assert_eq!(item.role, Role::Item);
    assert_eq!(item.key, Some(skein_ui::Key { owner: crate::frame::owner(), kind: 2, a: TASK, b: 0 }));
    let plan = styled(tree, look::PLAN);
    assert_eq!(children(tree, plan).as_slice(), [hold]);
    assert!(fixtures::find(tree, Role::TreeItem, b"Task 9").is_none());
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

#[test]
fn a_kept_activity_is_parked() {
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::owner_activity(
        &mut builder,
        &c::OwnerActivity::Kept { attempt: 1, pending: List::with_capacity(0) },
        &fixtures::words(),
    );
    assert_eq!(fixtures::text(&builder.finish()).as_ref(), b"parked");
}

fn goal(task: c::Task) -> crate::View {
    fixtures::page_view(&f::root_with(task))
}
fn region_index(tree: &Tree, name: &[u8]) -> usize {
    let id = fixtures::find(tree, Role::Region, name).expect("page region");
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.id == id {
            return index;
        }
    }
    panic!("region index");
}
fn result_words(bytes: &[u8]) -> c::Text {
    c::Text::new(Box::from(bytes), 256).expect("result words")
}
fn document_words(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(result_words(bytes))
}
#[test]
fn a_goal_names_the_result_it_gives() {
    let part =
        c::OwnerPart { connector: fixtures::FORGE, kind: c::OwnerPartKind::Result, key: c::OwnerPartKey::Number(TASK) };
    for (contract, name) in [
        (c::TaskContractDocument::Report { words: 256 }, "Result · report".as_bytes()),
        (
            c::TaskContractDocument::Change { connector: fixtures::FORGE, kind: 1, words: 256 },
            "Result · change".as_bytes(),
        ),
        (
            c::TaskContractDocument::Verdict { choices: List::with_capacity(0), cut: false },
            "Result · verdict".as_bytes(),
        ),
        (
            c::TaskContractDocument::Approval { state: Box::from(b"ready".as_slice()), part },
            "Result · approval".as_bytes(),
        ),
    ] {
        let mut task = f::document();
        task.result_contract = contract;
        let view = goal(task);
        let tree = view.tree();
        let asked = region_index(tree, b"What was asked");
        let parts = children(tree, asked);
        assert_eq!(tree.nodes()[parts.as_slice()[0]].role, Role::Heading { level: 2 });
        let contract = &tree.nodes()[parts.as_slice()[1]];
        assert_eq!(contract.role, Role::Paragraph);
        assert_eq!(contract.name.as_deref(), Some(name));
        assert_eq!(contract.styles.as_slice(), [look::TASK_CONTRACT]);
        assert_eq!(fixtures::node_text(tree, parts.as_slice()[1]).as_ref(), name);
    }
}
#[test]
#[expect(clippy::manual_map, reason = "the client matches without a closure")]
fn an_ended_goal_shows_its_result() {
    for (ending, name, token, said) in [
        (c::Ending::Done, b"Completed".as_slice(), look::STATUS_DONE, Some(b"A **useful** report".as_slice())),
        (c::Ending::Failed { reason: Some(result_words(b"Checks failed")) }, b"Failed", look::STATUS_HELD, None),
        (
            c::Ending::Cancelled {
                reason: Some(result_words(b"No longer needed")),
                by: c::Actor::Party(PARTY),
                at: None,
            },
            b"Cancelled",
            look::STATUS_QUIET,
            None,
        ),
        (c::Ending::Deleted, b"Ended", look::STATUS_QUIET, None),
    ] {
        let mut task = f::document();
        task.result_contract = c::TaskContractDocument::Report { words: 256 };
        task.result = Some(c::TaskResult::Document(Box::new(c::Result {
            task: task.chip.clone(),
            ending,
            value: match said {
                Some(bytes) => Some(c::ResultValue::Report(document_words(bytes))),
                None => None,
            },
            at: None,
            details: None,
            acts: List::with_capacity(0),
        })));
        let view = goal(task);
        let tree = view.tree();
        let index = region_index(tree, b"Result");
        assert!(region_index(tree, b"What was asked") < index);
        assert!(index < region_index(tree, b"Plan"));
        assert_eq!(tree.nodes()[index].styles.as_slice(), [look::TASK_CARD]);
        let nodes = subtree(tree, index);
        assert!(named(nodes, Role::Status, name));
        for node in nodes {
            if node.role == Role::Status {
                assert_eq!(node.styles.as_slice(), [token]);
            }
        }
        let expected = match name {
            b"Completed" => b"A useful report".as_slice(),
            b"Failed" => b"Checks failed",
            b"Cancelled" => b"No longer needed",
            _ => b"Ended",
        };
        assert!(skein_lib::bytes::find(&fixtures::text(tree), expected).is_some());
    }
    let mut task = f::document();
    task.result_contract = c::TaskContractDocument::Change { connector: fixtures::FORGE, kind: 1, words: 256 };
    task.result = Some(c::TaskResult::Ending(Box::new(c::OwnerChipEnding::Done(c::ResultValue::Change {
        kind: 1,
        part: c::OwnerPart {
            connector: fixtures::FORGE,
            kind: c::OwnerPartKind::Result,
            key: c::OwnerPartKey::Number(TASK),
        },
        words: document_words(b"1a2b3c4abcdef0123456789abcdef0123456789abcd"),
    }))));
    let view = goal(task);
    assert!(named(subtree(view.tree(), region_index(view.tree(), b"Result")), Role::Status, b"Landed"));
    assert!(
        skein_lib::bytes::find(&fixtures::text(view.tree()), "CI passed. Landed · merge commit 1a2b3c4".as_bytes())
            .is_some()
    );
}
#[test]
fn a_live_goal_has_no_result_region() {
    for view in [goal(f::document()), fixtures::page_view(&f::held_root())] {
        assert!(fixtures::find(view.tree(), Role::Region, b"Result").is_none());
    }
}
#[test]
fn a_goals_budget_card_shows_its_budget_as_supplied() {
    let mut task = f::document();
    task.budget.numbers = c::OwnerNumbers { budget: 913, spent: 37, spent_below: 83, reserved: 29 };
    task.budget.left = 641;
    task.authority.authority.spend = 877;
    task.authority.authority.delegation.tasks = 11;
    task.authority.authority.delegation.depth = 3;
    let view = goal(task.clone());
    let tree = view.tree();
    let index = region_index(tree, b"Budget and authority");
    assert!(region_index(tree, b"Latest activity") < index);
    assert_eq!(tree.nodes()[index].styles.as_slice(), [look::TASK_CARD, look::TASK_BUDGET]);
    for node in &tree.nodes()[index.checked_add(1).expect("next")..] {
        assert_ne!(node.role, Role::Region, "budget is the last region");
    }
    let parts = children(tree, index);
    let parts = parts.as_slice();
    let balance = joined(&[
        Decimal::of(task.budget.numbers.spent).as_bytes(),
        " spent · ".as_bytes(),
        Decimal::of(task.budget.left).as_bytes(),
        b" left of ",
        Decimal::of(task.budget.numbers.budget).as_bytes(),
    ]);
    assert_eq!(tree.nodes()[parts[1]].name.as_deref(), Some(balance.as_ref()));
    assert_eq!(fixtures::node_text(tree, parts[1]).as_ref(), balance.as_ref());
    assert!(!named(subtree(tree, index), Role::Term, b"Spent"));
    let bounds = joined(&[
        b"Up to ",
        Decimal::of(u64::from(task.authority.authority.delegation.tasks)).as_bytes(),
        b" tasks, ",
        Decimal::of(u64::from(task.authority.authority.delegation.depth)).as_bytes(),
        b" deep",
    ]);
    for (offset, term, value) in [
        (2, b"Delegates spent".as_slice(), Box::from(Decimal::of(task.budget.numbers.spent_below).as_bytes())),
        (4, b"Reserved for delegates", Box::from(Decimal::of(task.budget.numbers.reserved).as_bytes())),
        (10, b"May spend", Box::from(Decimal::of(task.authority.authority.spend).as_bytes())),
        (12, b"May delegate", bounds),
    ] {
        let label = &tree.nodes()[parts[offset]];
        let description = &tree.nodes()[parts[offset + 1]];
        assert_eq!(label.role, Role::Term);
        assert_eq!(label.name.as_deref(), Some(term));
        assert_eq!(fixtures::node_text(tree, parts[offset]).as_ref(), term);
        assert_eq!(description.role, Role::Description);
        assert_eq!(description.name.as_deref(), Some(value.as_ref()));
        assert_eq!(fixtures::node_text(tree, parts[offset + 1]).as_ref(), value.as_ref());
    }
    assert_eq!(tree.nodes()[parts[6]].name.as_deref(), Some(b"Funded by".as_slice()));
}
#[test]
fn the_budget_card_names_one_task_and_other_delegation_bounds() {
    for tasks in [0, 1, 2] {
        let mut task = f::document();
        task.authority.authority.delegation.tasks = tasks;
        task.authority.authority.delegation.depth = 7;
        let delegation = &task.authority.authority.delegation;
        let noun = if tasks == 1 { b" task, ".as_slice() } else { b" tasks, ".as_slice() };
        let expected = joined(&[
            b"Up to ",
            Decimal::of(u64::from(delegation.tasks)).as_bytes(),
            noun,
            Decimal::of(u64::from(delegation.depth)).as_bytes(),
            b" deep",
        ]);
        let view = goal(task);
        let tree = view.tree();
        let parts = children(tree, region_index(tree, b"Budget and authority"));
        let parts = parts.as_slice();
        assert_eq!(tree.nodes()[parts[12]].name.as_deref(), Some(b"May delegate".as_slice()));
        assert_eq!(tree.nodes()[parts[13]].role, Role::Description);
        assert_eq!(tree.nodes()[parts[13]].name.as_deref(), Some(expected.as_ref()));
        assert_eq!(fixtures::node_text(tree, parts[13]).as_ref(), expected.as_ref());
    }
}
#[test]
fn the_budget_card_names_each_funder() {
    let root = f::held_root();
    let person = root.pages().me(root.client()).expect("signed in").party.number;
    for funder in [
        c::Funder::Task(CHILD),
        c::Funder::Pool { project: fixtures::PROJECT, person, period: 1 },
        c::Funder::Pool { project: fixtures::PROJECT, person: person + 1, period: 1 },
        c::Funder::Recurring { project: fixtures::PROJECT, task: CHILD, period: 1 },
        c::Funder::Period { project: fixtures::PROJECT, period: 1 },
    ] {
        let mut task = f::document();
        task.budget.funder = funder;
        let mut view = goal(task);
        let expected = match funder {
            c::Funder::Task(task) => joined(&[b"Task ", Decimal::of(task).as_bytes()]),
            c::Funder::Pool { person: owner, .. } if owner == person => Box::from(b"Your pool".as_slice()),
            c::Funder::Pool { person, .. } => joined(&[b"Person ", Decimal::of(person).as_bytes(), b"'s pool"]),
            c::Funder::Recurring { task, .. } => {
                joined(&[b"Task ", Decimal::of(task).as_bytes(), b"'s recurring budget"])
            }
            c::Funder::Period { project, .. } => {
                joined(&[b"Project ", Decimal::of(u64::from(project)).as_bytes(), b"'s budget"])
            }
        };
        let tree = view.tree();
        let nodes = subtree(tree, region_index(tree, b"Budget and authority"));
        let role = if let c::Funder::Task(_) = funder { Role::Link } else { Role::Description };
        assert!(named(nodes, role, expected.as_ref()));
        if let c::Funder::Task(task) = funder {
            let mut link = None;
            for node in nodes {
                if node.role == Role::Link {
                    link = Some(node.id);
                }
            }
            let id = link.expect("funder link");
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                    address: temper_client_domain::Address::Task { task }
                }))
            );
        }
    }
}
#[test]
fn the_budget_card_names_its_models() {
    let mut names = List::with_capacity(2);
    for name in [b"first".as_slice(), b"second"] {
        names.push(Box::from(name)).expect("two model names");
    }
    for (models, expected) in [
        (Some(c::AuthorityModels::All), [b"All".as_slice()].as_slice()),
        (Some(c::AuthorityModels::Named(names)), [b"first".as_slice(), b"second"].as_slice()),
        (Some(c::AuthorityModels::Named(List::with_capacity(0))), [b"None".as_slice()].as_slice()),
        (None, [].as_slice()),
    ] {
        let mut task = f::document();
        task.authority.authority.models = models;
        let view = goal(task);
        let tree = view.tree();
        let index = region_index(tree, b"Budget and authority");
        let nodes = subtree(tree, index);
        assert_eq!(named(nodes, Role::Term, b"Models"), !expected.is_empty());
        let parts = children(tree, index);
        let mut actual = List::with_capacity(2);
        let mut after_models = false;
        for part in parts.as_slice() {
            let node = &tree.nodes()[*part];
            if after_models {
                assert_eq!(node.role, Role::Description);
                let name = node.name.as_deref().expect("model description");
                assert_eq!(fixtures::node_text(tree, *part).as_ref(), name);
                actual.push(name).expect("model descriptions");
            }
            after_models |= node.role == Role::Term && node.name.as_deref() == Some(b"Models");
        }
        assert_eq!(actual.as_slice(), expected);
    }
}
#[test]
fn a_cut_authority_ends_the_card_with_its_line() {
    for cut in [false, true] {
        let mut task = f::document();
        task.authority.cut = cut;
        let view = goal(task);
        let tree = view.tree();
        let name = b"Some of its authority is not shown.";
        assert_eq!(fixtures::find(tree, Role::Paragraph, name).is_some(), cut);
        if cut {
            let parts = children(tree, region_index(tree, b"Budget and authority"));
            let last = &tree.nodes()[*parts.as_slice().last().expect("last child")];
            assert_eq!(last.name.as_deref(), Some(name.as_slice()));
            assert_eq!(fixtures::node_text(tree, *parts.as_slice().last().expect("last child")).as_ref(), name);
        }
    }
}

fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut len = 0_usize;
    for part in parts {
        len = len.checked_add(part.len()).expect("expected words fit");
    }
    let mut writer = Writer::new(len);
    for part in parts {
        writer.put(part).expect("measured expected words");
    }
    writer.finish()
}

const DEPENDENCY: u64 = 17;
const OTHER_DEPENDENCY: u64 = 23;
const GRANDCHILD: u64 = 13;
const ROW_SPENT: u64 = 5;
fn owner_phase(chip: &mut c::Chip, phase: c::OwnerChipPhase) {
    chip.phase = phase.compact();
    let c::ChipPhase::Owner { phase: target, .. } = chip.details.as_mut() else { panic!("owner") };
    *target = phase;
}
fn row() -> c::OwnerTreeRow {
    let mut chip = f::document().chip;
    chip.task = CHILD;
    let c::ChipPhase::Owner { details, .. } = chip.details.as_mut() else { panic!("owner") };
    details.budget.numbers.spent = ROW_SPENT;
    c::OwnerTreeRow {
        chip,
        parent: TASK,
        depth: ONE,
        proposals: List::with_capacity(NO_ROWS),
        spent: ROW_SPENT + 7,
        dependencies: List::with_capacity(NO_ROWS),
        cut: false,
    }
}
fn plan(delivery: &c::Delivery) -> Tree {
    let mut builder = Builder::new(&fixtures::tree_limits());
    crate::pages::task::plan(
        &mut builder,
        delivery,
        &f::root(c::Executor::Agent { charter: CHARTER }, Some(PRIORITY)),
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        fixtures::domain_limits().client.domain.rows,
    );
    builder.finish()
}
fn plan_with_row(row: c::OwnerTreeRow) -> Tree {
    let mut delivery = f::tree();
    let c::Delivery::OwnerTree(tree) = &mut delivery else { panic!("owner tree") };
    tree.rows = List::with_capacity(ONE);
    tree.rows.push(row).expect("row");
    plan(&delivery)
}
fn index_of(tree: &Tree, role: Role, name: &[u8]) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == role && node.name.as_deref() == Some(name) {
            return index;
        }
    }
    panic!("named node");
}
fn assert_status(tree: &Tree, name: &[u8], token: skein_ui::Style) {
    let index = index_of(tree, Role::Status, name);
    assert_eq!(tree.nodes()[index].styles.as_slice(), [token]);
    assert_eq!(fixtures::node_text(tree, index).as_ref(), name);
}

#[test]
fn a_plan_row_says_what_its_task_waits_for() {
    let cases: &[(&[u64], bool, bool, &[u8])] = &[
        (&[DEPENDENCY], false, true, b"waiting on task 17"),
        (&[DEPENDENCY, OTHER_DEPENDENCY], false, false, b"waiting on 2 tasks"),
        (&[DEPENDENCY], true, true, b"waiting on more than 1 tasks"),
        (&[], true, true, b"waiting on more than 0 tasks"),
        (&[], false, true, b"waiting on its resources"),
        (&[], false, false, b"Waiting"),
    ];
    for (numbers, more, holds, expected) in cases {
        let mut row = row();
        let mut dependencies = List::with_capacity(2);
        for number in *numbers {
            dependencies.push(*number).expect("dependency");
        }
        owner_phase(&mut row.chip, c::OwnerChipPhase::Waiting { dependencies, more: *more, holds: *holds });
        let tree = plan_with_row(row);
        assert_status(&tree, expected, look::STATUS_QUIET);
        assert!(skein_lib::bytes::find(&fixtures::text(&tree), expected).is_some());
    }
}
fn document_text(words: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(skein_lib::bytes::copy_of(words), 32).expect("document text"))
}
#[test]
fn an_ended_row_says_its_ending() {
    let cases: [(c::OwnerChipEnding, &[u8], skein_ui::Style); 3] = [
        (c::OwnerChipEnding::Done(c::ResultValue::Report(document_text(b"Done"))), b"Completed", look::STATUS_DONE),
        (c::OwnerChipEnding::Failed(document_text(b"Failed")), b"Failed", look::STATUS_HELD),
        (
            c::OwnerChipEnding::Cancelled {
                reason: document_text(b"Cancelled"),
                result: None,
                by: c::OwnerRequester::Person(PARTY),
                at: skein_lib::Wall::EPOCH,
            },
            b"Cancelled",
            look::STATUS_QUIET,
        ),
    ];
    for (ending, expected, token) in cases {
        let mut row = row();
        owner_phase(&mut row.chip, c::OwnerChipPhase::Ended);
        let c::ChipPhase::Owner { details, .. } = row.chip.details.as_mut() else { panic!("owner") };
        details.ending = Some(ending);
        assert_status(&plan_with_row(row), expected, token);
    }
}
#[test]
fn a_rows_spend_is_its_supplied_spent() {
    let tree = plan_with_row(row());
    let spend = styled(&tree, look::PLAN_SPEND);
    let node = &tree.nodes()[spend];
    assert_eq!(node.role, Role::Paragraph);
    assert_eq!(node.name.as_deref(), Some(b"5 spent".as_slice()));
    assert_eq!(fixtures::node_text(&tree, spend).as_ref(), b"5 spent");
    let slot = styled(&tree, look::PLAN_ROW);
    let parts = children(&tree, slot);
    assert_eq!(parts.as_slice().last(), Some(&spend));
    assert_eq!(tree.nodes()[parts.as_slice()[2]].role, Role::Status);
    let mut row = row();
    *row.chip.details = c::ChipPhase::Starting;
    for node in plan_with_row(row).nodes() {
        assert!(!node.styles.as_slice().contains(&look::PLAN_SPEND));
    }
}
fn dependency_row() -> c::OwnerTreeRow {
    let mut row = row();
    let mut live = f::document().chip;
    live.task = DEPENDENCY;
    owner_phase(&mut live, c::OwnerChipPhase::Active(c::RunStage::Running { attempt: 1 }));
    row.dependencies = List::with_capacity(3);
    for link in [
        c::OwnerTaskLink::Live(live),
        c::OwnerTaskLink::Stored { number: OTHER_DEPENDENCY, status: Some(c::OwnerTaskStatus::Done) },
        c::OwnerTaskLink::Stored { number: TASK, status: None },
    ] {
        row.dependencies.push(link).expect("dependency");
    }
    row
}
#[test]
fn a_rows_dependency_line_lists_its_dependencies() {
    let mut delivery = f::tree();
    let c::Delivery::OwnerTree(owner) = &mut delivery else { panic!("owner tree") };
    owner.rows = List::with_capacity(2);
    owner.rows.push(dependency_row()).expect("row");
    let mut child = row();
    child.chip.task = GRANDCHILD;
    child.parent = CHILD;
    child.depth = 2;
    owner.rows.push(child).expect("child");
    let tree = plan(&delivery);
    let list = styled(&tree, look::DEPENDENCIES);
    assert_eq!(tree.nodes()[list].role, Role::List);
    assert_eq!(tree.nodes()[list].name.as_deref(), Some(b"Dependencies of task 12".as_slice()));
    let items = children(&tree, list);
    assert_eq!(items.len(), 3);
    for (index, number, name, status) in [
        (0, DEPENDENCY, b"Task 17".as_slice(), Some(b"running".as_slice())),
        (1, OTHER_DEPENDENCY, b"Task 23", Some(b"Completed".as_slice())),
        (2, TASK, b"Task 9", None),
    ] {
        let item = items.as_slice()[index];
        assert_eq!(tree.nodes()[item].role, Role::Item);
        assert_eq!(tree.nodes()[item].key, Some(crate::pages::inbox::card_key(c::ObjectKey::Task(number))));
        let parts = children(&tree, item);
        let link = &tree.nodes()[parts.as_slice()[0]];
        assert_eq!(link.role, Role::Link);
        assert_eq!(link.name.as_deref(), Some(name));
        assert!(link.binding.is_some());
        if let Some(status) = status {
            assert_eq!(parts.len(), 2);
            assert_eq!(tree.nodes()[parts.as_slice()[1]].name.as_deref(), Some(status));
        } else {
            assert_eq!(parts.len(), 1);
        }
    }
    assert_status(&tree, b"running", look::STATUS_ACTIVE);
    assert_status(&tree, b"Completed", look::STATUS_DONE);
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::TreeItem && node.name.as_deref() == Some(b"Task 12") {
            let parts = children(&tree, index);
            assert_eq!(parts.len(), 3);
            assert_eq!(tree.nodes()[parts.as_slice()[0]].styles.as_slice(), [look::PLAN_ROW]);
            assert_eq!(parts.as_slice()[1], list);
            assert_eq!(tree.nodes()[parts.as_slice()[2]].role, Role::TreeItem);
        }
    }
}
#[test]
fn a_cut_dependency_line_ends_with_its_last_line() {
    for populated in [false, true] {
        let mut row = if populated { dependency_row() } else { row() };
        row.cut = true;
        let tree = plan_with_row(row);
        let list = styled(&tree, look::DEPENDENCIES);
        let items = children(&tree, list);
        assert_eq!(items.len(), if populated { 4 } else { 1 });
        let item = *items.as_slice().last().expect("cut item");
        assert_eq!(tree.nodes()[item].role, Role::Item);
        assert_eq!(tree.nodes()[item].key, None);
        let parts = children(&tree, item);
        assert_eq!(parts.len(), 1);
        let line = &tree.nodes()[parts.as_slice()[0]];
        assert_eq!(line.role, Role::Paragraph);
        assert_eq!(line.name.as_deref(), Some(b"More dependencies not shown.".as_slice()));
    }
}
#[test]
fn the_goal_has_no_plan_row() {
    let mut delivery = f::tree();
    let c::Delivery::OwnerTree(owner) = &mut delivery else { panic!("owner tree") };
    let mut dependencies = List::with_capacity(ONE);
    dependencies.push(DEPENDENCY).expect("dependency");
    owner_phase(&mut owner.root, c::OwnerChipPhase::Waiting { dependencies, more: false, holds: false });
    let c::ChipPhase::Owner { details, .. } = owner.root.details.as_mut() else { panic!("owner") };
    details.dependencies = List::with_capacity(ONE);
    details
        .dependencies
        .push(c::Dependency { number: DEPENDENCY, pending: true, added: None, keeps: true, status: None })
        .expect("dependency fact");
    let tree = plan(&delivery);
    assert!(fixtures::find(&tree, Role::TreeItem, b"Task 9").is_none());
    for node in tree.nodes() {
        assert!(!node.styles.as_slice().contains(&look::DEPENDENCIES));
        assert!(!node.styles.as_slice().contains(&look::PLAN_ROW));
    }
}
#[test]
fn the_header_says_what_the_task_waits_for() {
    let mut task = titled();
    let mut dependencies = List::with_capacity(ONE);
    dependencies.push(DEPENDENCY).expect("dependency");
    task.waits_on = c::WaitsOn::Dependencies { pending: dependencies, count: 1 };
    assert_status(&header(&task, None), b"waiting on task 17", look::STATUS_QUIET);
}

pub(super) fn dependency_plan() -> Tree {
    plan_with_row(dependency_row())
}

fn observed(chip: &mut c::Chip, work: Option<c::OwnerChipWork>) {
    owner_phase(chip, c::OwnerChipPhase::Active(c::RunStage::Running { attempt: ONE.into() }));
    let c::ChipPhase::Owner { details, .. } = chip.details.as_mut() else { panic!("owner") };
    if let Some(work) = work {
        details.observation = Some(c::OwnerChipObservation {
            channel: skein_lib::Token::new(1),
            work,
            since: skein_lib::Wall::EPOCH,
            reported_at: skein_lib::Wall::EPOCH,
            started: None,
        });
    } else {
        details.observation = None;
    }
}
#[test]
fn a_running_row_and_header_say_the_observed_work() {
    let cases: [(Option<c::OwnerChipWork>, &[u8]); 4] = [
        (None, b"running"),
        (Some(c::OwnerChipWork::Writing), "running · writing".as_bytes()),
        (
            Some(c::OwnerChipWork::Tool { call: Box::from(*b"call"), tool: Box::from(*b"read_file") }),
            "running · a tool · read_file".as_bytes(),
        ),
        (Some(c::OwnerChipWork::Waiting), b"running"),
    ];
    for (work, expected) in cases {
        let mut row = row();
        observed(&mut row.chip, work);
        let mut task = titled();
        task.chip = row.chip.clone();
        task.waits_on = c::WaitsOn::Nothing;
        assert_status(&header(&task, None), expected, look::STATUS_ACTIVE);
        assert_status(&plan_with_row(row), expected, look::STATUS_ACTIVE);
    }
}
fn party_wait(whom: c::Whom) -> c::WaitsOn {
    c::WaitsOn::Party { whom, entries: ONE, task: CHILD, kind: c::WaitedFor::Question }
}
#[test]
fn the_header_says_its_exact_wait_and_goal_priority_in_both_shells() {
    let mut task = titled();
    task.priority = Some(PRIORITY);
    let root = f::held_root();
    let mut me = root.pages().me(root.client()).expect("me").clone();
    me.projects.get_mut(0).expect("project").role = c::HeldRole::Owner;
    let mut pending = List::with_capacity(ONE);
    pending.push(DEPENDENCY).expect("pending");
    let cases: [(c::WaitsOn, &[u8]); 6] = [
        (c::WaitsOn::Dependencies { pending, count: 3 }, b"waiting on 3 tasks"),
        (party_wait(c::Whom::Person(me.party.number)), b"waiting on an answer from you"),
        (party_wait(c::Whom::Role { project: task.project, role: 0 }), b"waiting on an answer from you"),
        (party_wait(c::Whom::Person(DEPENDENCY)), b"waiting on an answer from person 17"),
        (party_wait(c::Whom::Role { project: task.project, role: 7 }), b"waiting on an answer from role 7"),
        (c::WaitsOn::News, b"waiting on news"),
    ];
    for words in [fixtures::words(), fixtures::browser_words()] {
        for (waits_on, expected) in &cases {
            task.waits_on = waits_on.clone();
            let mut builder = Builder::new(&fixtures::tree_limits());
            crate::pages::task::header(&mut builder, &task, Some(&me), &words);
            let tree = builder.finish();
            assert_status(&tree, expected, look::STATUS_QUIET);
            assert!(fixtures::find(&tree, Role::Paragraph, "Goal · priority 1".as_bytes()).is_some());
        }
        task.waits_on = c::WaitsOn::Delegates;
        for (count, expected) in
            [(0, b"waiting on 0 delegates".as_slice()), (1, b"waiting on 1 delegate"), (2, b"waiting on 2 delegates")]
        {
            let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner") };
            details.live_delegates = count;
            let mut builder = Builder::new(&fixtures::tree_limits());
            crate::pages::task::header(&mut builder, &task, Some(&me), &words);
            assert_status(&builder.finish(), expected, look::STATUS_QUIET);
        }
    }
}
#[test]
fn both_plan_shapes_omit_the_goal_and_keep_its_hold_and_folds() {
    let root = f::held_root();
    let mut owner = f::tree();
    let c::Delivery::OwnerTree(tree) = &mut owner else { panic!("owner tree") };
    tree.root.phase = c::Phase::Held;
    tree.rows = List::with_capacity(ONE);
    tree.rows.push(row()).expect("row");
    tree.folded = List::with_capacity(ONE);
    tree.folded
        .push(c::OwnerTreeFold {
            parent: TASK,
            phases: c::OwnerPhaseCounts { running: 2, ..c::OwnerPhaseCounts::ZERO },
            held_below: 0,
        })
        .expect("root fold");
    let counts =
        c::PhaseCounts { ended: 0, waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 };
    let mut legacy = c::Tree { task: TASK, nodes: List::with_capacity(2), folded: counts, held_below: 0 };
    for (chip, parent, folded) in
        [(tree.root.clone(), None, c::PhaseCounts { running: 2, ..counts }), (row().chip, Some(TASK), counts)]
    {
        legacy
            .nodes
            .push(c::TreeNode {
                chip,
                parent,
                dependencies: List::with_capacity(NO_ROWS),
                proposals: List::with_capacity(NO_ROWS),
                spend: ROW_SPENT + 7,
                folded,
                held_below: 0,
            })
            .expect("node");
    }
    for delivery in [owner, c::Delivery::Tree(Box::new(legacy))] {
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
        assert!(fixtures::find(&tree, Role::TreeItem, b"Task 9").is_none());
        let top = children(&tree, styled(&tree, look::PLAN));
        let nodes = tree.nodes();
        assert_eq!(nodes[top.as_slice()[0]].styles.as_slice(), [look::PLAN_HOLD]);
        assert_eq!(nodes[top.as_slice()[1]].role, Role::Paragraph);
        assert_eq!(nodes[top.as_slice()[1]].name.as_deref(), Some(b"Folded: 2 Running".as_slice()));
        assert_eq!(nodes[top.as_slice()[2]].role, Role::TreeItem);
        assert_eq!(nodes[top.as_slice()[2]].name.as_deref(), Some(b"Task 12".as_slice()));
    }
}
