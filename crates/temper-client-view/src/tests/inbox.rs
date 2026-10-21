use super::fixtures;
use crate::look;
use jig_client::contract as c;
use skein_ui::{Node, State, Tree};
use temper_client_domain::{
    Address, Domain, Event,
    address::{Filter, Kind},
};

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
            address: Address::Task { task: fixtures::INBOX_TASK }
        }))
    );
}
#[test]
fn the_goal_filter_keeps_its_project_and_kind() {
    let filter = Filter { project: Some(fixtures::PROJECT), kind: Some(Kind::Question), goal: None };
    let root = fixtures::inbox_root_filtered(filter);
    let mut view = fixtures::view();
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::TextField, b"Goal").expect("goal filter");
    assert_eq!(
        crate::act(&mut view, node, Act::Input { value: Box::from(*b"7") }),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Filter {
            filter: Filter { goal: Some(GOAL), ..filter }
        }))
    );
}

pub(super) fn subtree(tree: &Tree, index: usize) -> &[Node] {
    let node = &tree.nodes()[index];
    &tree.nodes()[index..index.checked_add(usize::try_from(node.size).expect("subtree size")).expect("subtree end")]
}
pub(super) fn styled(tree: &Tree, token: skein_ui::Style) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&token) {
            return index;
        }
    }
    panic!("styled node {token:?}");
}
fn heading(tree: &Tree, name: &[u8]) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::HEADING) {
            for child in subtree(tree, index) {
                if child.role == (Role::Heading { level: 2 }) && child.name.as_deref() == Some(name) {
                    return index;
                }
            }
        }
    }
    panic!("section heading");
}
fn style_count(nodes: &[Node], token: skein_ui::Style) -> usize {
    let mut count = 0_usize;
    for node in nodes {
        if node.styles.as_slice().contains(&token) {
            count = count.checked_add(1).expect("tree bounds count");
        }
    }
    count
}
fn role_node(nodes: &[Node], role: Role) -> &Node {
    for node in nodes {
        if node.role == role {
            return node;
        }
    }
    panic!("role {role:?}");
}
#[test]
fn headings_hold_the_page_title_filters_and_mark_seen() {
    let view = fixtures::page_view(&fixtures::inbox_root());
    let tree = view.tree();
    let bar = subtree(tree, styled(tree, look::PAGEBAR));
    assert_eq!(bar.len(), 3);
    assert_eq!(bar[1].role, Role::Heading { level: 1 });
    assert_eq!(bar[1].name.as_deref(), Some(b"Inbox".as_slice()));
    let needs = subtree(tree, heading(tree, b"Needs you"));
    assert_eq!(style_count(needs, look::FILTERS), 1);
    for node in needs {
        assert_ne!(node.name.as_deref(), Some(b"Mark seen".as_slice()));
    }
    let updates = subtree(tree, heading(tree, b"Updates"));
    assert_eq!(updates.len(), 4);
    assert_eq!(updates[3].role, Role::Button);
    assert_eq!(updates[3].name.as_deref(), Some(b"Mark seen".as_slice()));
}
#[test]
fn filters_press_exactly_the_current_project_kind_and_all_goals() {
    for (filter, expected) in [
        (Filter::ALL, [b"All projects".as_slice(), b"All kinds", b"All goals"]),
        (
            Filter { project: Some(fixtures::PROJECT), kind: Some(Kind::Question), goal: None },
            [b"Shop".as_slice(), b"Question", b"All goals"],
        ),
    ] {
        let root = fixtures::inbox_root_filtered(filter);
        let mut view = fixtures::page_view(&root);
        let tree = view.tree();
        let mut pressed = skein_lib::List::with_capacity(3);
        let mut ids = skein_lib::List::with_capacity(3);
        let filters = subtree(tree, styled(tree, look::FILTERS));
        for node in filters {
            if node.states.as_slice().contains(&State::Pressed) {
                assert_eq!(node.role, Role::Button);
                pressed.push(node.name.as_deref().expect("filter name")).expect("one pressed per group");
                ids.push(node.id).expect("three controls");
            }
        }
        assert_eq!(pressed.as_slice(), expected);
        // Each pressed button remains an action for the same filter.
        for id in ids.as_slice() {
            assert_eq!(
                crate::act(&mut view, *id, Act::Press),
                Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Filter { filter }))
            );
        }
    }
}
#[test]
fn cards_are_grouped_in_lists_with_kinds_and_named_project_paths() {
    let view = fixtures::page_view(&fixtures::inbox_root());
    let tree = view.tree();
    let mut lists = skein_lib::List::with_capacity(2);
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::CARDS) {
            assert_eq!(node.role, Role::List);
            lists.push(index).expect("two lists");
        }
    }
    assert_eq!(lists.len(), 2);
    assert_eq!(tree.nodes()[lists.as_slice()[0]].styles.as_slice(), [look::CARDS]);
    assert_eq!(tree.nodes()[lists.as_slice()[1]].styles.as_slice(), [look::CARDS, look::UPDATES]);
    let kinds = [
        look::KIND_PROPOSAL,
        look::KIND_QUESTION,
        look::KIND_HELD,
        look::KIND_PERSON_TASK,
        look::KIND_RESULT,
        look::KIND_REPLY,
    ];
    for (i, token) in kinds.into_iter().enumerate() {
        let card = styled(tree, token);
        let node = &tree.nodes()[card];
        assert_eq!(node.role, Role::Item);
        assert_eq!(node.styles.as_slice(), [look::CARD, token]);
        let list = lists.as_slice()[usize::from(i >= 4)];
        assert!(card > list && card < list.checked_add(subtree(tree, list).len()).expect("list end"));
        let meta = subtree(tree, card.checked_add(1).expect("meta index"));
        assert_eq!(meta[0].styles.as_slice(), [look::META]);
        assert_eq!(meta[1].role, Role::Heading { level: 3 });
        let project = role_node(meta, Role::Paragraph);
        assert_eq!(project.name.as_deref(), Some(b"Shop".as_slice()));
        let link = role_node(meta, Role::Link);
        assert_eq!(link.name.as_deref(), Some(if i == 5 { b"Task 10".as_slice() } else { b"Task 9".as_slice() }));
    }
}
#[test]
fn absent_groups_have_no_empty_lists() {
    let limits = fixtures::domain_limits();
    let (mut root, _, watch) = fixtures::root_at(Address::Inbox { filter: Filter::ALL }, &limits);
    let mut out = skein_lib::Queue::with_capacity(temper_client_domain::max_out(&limits));
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits },
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("inbox watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(c::Inbox {
                        entries: skein_lib::List::with_capacity(0),
                        needs: c::Needs { proposals: 0, questions: 0, escalations: 0, person_tasks: 0 },
                        updates: c::Count { value: 0, limited: false },
                        results_seen: 0,
                        replies_seen: skein_lib::List::with_capacity(0),
                        more: false,
                    }))),
                },
            },
        },
        &mut out,
    );
    let view = fixtures::page_view(&root);
    assert_eq!(style_count(view.tree().nodes(), look::CARDS), 0);
    let filtered =
        fixtures::page_view(&fixtures::inbox_root_filtered(Filter { kind: Some(Kind::Question), ..Filter::ALL }));
    assert_eq!(style_count(filtered.tree().nodes(), look::CARDS), 1);
    assert_eq!(style_count(filtered.tree().nodes(), look::UPDATES), 0);
}
pub(super) fn question_root() -> Domain {
    let task = fixtures::task::document();
    let page = c::Question {
        task: task.chip,
        from: None,
        message: fixtures::INBOX_QUESTION,
        words: c::Text::new(Box::from(*b"Which path?"), 32).expect("question"),
        since: skein_lib::Wall::EPOCH,
        acts: skein_lib::List::with_capacity(0),
        ending: None,
    };
    fixtures::inbox_with_document(
        Filter::ALL,
        Some((
            c::Read::Question { task: fixtures::INBOX_TASK, message: fixtures::INBOX_QUESTION },
            c::ReadValue::Question(Box::new(page)),
        )),
    )
}
#[test]
fn a_title_stays_a_paragraph_and_empty_facts_are_absent() {
    let root = question_root();
    let view = fixtures::page_view(&root);
    let tree = view.tree();
    let title = styled(tree, look::TITLE);
    assert_eq!(tree.nodes()[title].role, Role::Paragraph);
    assert_eq!(tree.nodes()[title].name, Some(fixtures::task::document().chip.title.bytes().into()));
    assert_eq!(style_count(tree.nodes(), look::DETAILS), 0);
}
#[test]
fn an_unknown_project_name_keeps_the_numbered_fallback() {
    let limits = fixtures::domain_limits();
    let (mut root, me_watch, inbox_watch) = fixtures::root_at(Address::Inbox { filter: Filter::ALL }, &limits);
    let mut me = root.pages().me(root.client()).expect("me").clone();
    me.projects.get_mut(0).expect("project").name = None;
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    let mut entries = skein_lib::List::with_capacity(1);
    entries
        .push(c::InboxEntry {
            reply: None,
            lineage: skein_lib::List::with_capacity(0),
            goal: None,
            recipient: None,
            stall: None,
            object: c::ObjectKey::Task(10),
            project: fixtures::PROJECT,
            group: c::InboxGroup::Replies,
            position: 1,
            at: skein_lib::Wall::EPOCH,
        })
        .expect("entry");
    let watch = inbox_watch.expect("inbox watch");
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(c::Inbox {
                        entries,
                        needs: c::Needs { proposals: 0, questions: 0, escalations: 0, person_tasks: 0 },
                        updates: c::Count { value: 1, limited: false },
                        results_seen: 0,
                        replies_seen: skein_lib::List::with_capacity(0),
                        more: false,
                    }))),
                },
            },
        },
    );
    let view = fixtures::page_view(&root);
    let meta = subtree(view.tree(), styled(view.tree(), look::META));
    assert_eq!(role_node(meta, Role::Paragraph).name.as_deref(), Some(b"Project 1".as_slice()));
}

#[test]
fn a_reply_card_names_its_task_after_the_task_ends() {
    use skein_fake_person::{Face, Find, Found, Target, TreeFace};

    let mut task = fixtures::task::document();
    task.chip.task = fixtures::INBOX_REPLY_TASK;
    task.chip.title = c::DocumentText {
        bytes: c::Text::new(Box::from(*b"Land a small change"), 32).expect("task title"),
        cut: false,
    };
    task.chip.phase = c::Phase::Ended;
    let name = Box::from(task.chip.title.bytes());
    let result = c::Result {
        task: task.chip.clone(),
        ending: c::Ending::Done,
        value: None,
        at: None,
        details: None,
        acts: skein_lib::List::with_capacity(0),
    };
    let root = fixtures::inbox_with_document(
        Filter::ALL,
        Some((
            c::Read::Task { task: fixtures::INBOX_REPLY_TASK },
            c::ReadValue::Historical(Box::new(c::OwnerHistorical {
                key: c::ObjectKey::Task(fixtures::INBOX_REPLY_TASK),
                value: c::OwnerHistoricalValue::Task { document: Box::new(task), result: Box::new(result) },
                at: None,
                by: None,
            })),
        )),
    );
    let view = fixtures::page_view(&root);
    let mut face = TreeFace::new(view.tree());
    let find =
        Find::named(Role::Heading { level: 3 }, b"Reply").within(Box::new([Target::Role { role: Role::Item, name }]));
    let found = match face.find(&find) {
        Found::Node(_) => true,
        Found::Absent | Found::Pending => false,
    };
    assert!(found, "ended task's named Item holds H3 Reply");
}

#[test]
fn loaded_task_held_and_result_titles_keep_the_title_token() {
    let mut task = fixtures::task::document();
    task.chip.task = fixtures::INBOX_TASK;
    let mut reply = task.clone();
    reply.chip.task = fixtures::INBOX_REPLY_TASK;
    for (read, value) in [
        (c::Read::Task { task: fixtures::INBOX_REPLY_TASK }, c::ReadValue::Task(Box::new(reply))),
        (
            c::Read::Escalation { task: fixtures::INBOX_TASK },
            c::ReadValue::OwnerEscalation(Box::new(c::OwnerEscalation {
                task: Box::new(task.clone()),
                acts: skein_lib::List::with_capacity(0),
            })),
        ),
        (
            c::Read::Result { task: fixtures::INBOX_TASK },
            c::ReadValue::Result(Box::new(c::Result {
                task: task.chip.clone(),
                ending: c::Ending::Done,
                value: None,
                at: None,
                details: None,
                acts: skein_lib::List::with_capacity(0),
            })),
        ),
    ] {
        let root = fixtures::inbox_with_document(Filter::ALL, Some((read.clone(), value)));
        let mut loaded = false;
        for card in &root.pages().inbox().expect("inbox").cards {
            loaded |= card.page.is_some();
        }
        assert!(loaded, "{read:?}");
        let view = fixtures::page_view(&root);
        let title = &view.tree().nodes()[styled(view.tree(), look::TITLE)];
        assert_eq!(title.role, Role::Paragraph);
        assert_eq!(title.name.as_deref(), Some(task.chip.title.bytes()));
    }
}
