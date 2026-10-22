use super::fixtures as f;
use jig_client::contract as c;
use skein_ui::Role;
use temper_client_domain::Address;

#[test]
fn the_start_draws_its_chatbar_banner_context_and_invitation_before_the_composer() {
    let root = f::demo_root(Address::Chats, f::PROJECT, true);
    let view = f::page_view(&root);
    let nodes = view.tree().nodes();
    let mut main = None;
    let mut chatbar = None;
    let mut context = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Main {
            main = Some(index);
        }
        if node.styles.as_slice().contains(&crate::look::CHATBAR) {
            chatbar = Some(index);
        }
        if node.styles.as_slice().contains(&crate::look::CONTEXT) {
            context = Some(index);
        }
    }
    assert!(main.unwrap() < chatbar.unwrap());
    assert_eq!(nodes[context.unwrap()].role, Role::Slot);
    let context = context.unwrap();
    let end = context + usize::try_from(nodes[context].size).unwrap();
    let mut paragraphs = 0_u32;
    for node in &nodes[context..end] {
        if node.role == Role::Paragraph {
            paragraphs += 1;
        }
    }
    assert_eq!(paragraphs, 2, "project and sign-in");
    let mut after = 0;
    for (role, name) in [
        (Role::Paragraph, b"New chat".as_slice()),
        (Role::Status, b"Ready"),
        (Role::Paragraph, b"Demo model"),
        (Role::Region, b"Demo"),
        (Role::Paragraph, "Shop · acme/shop on Demo forge".as_bytes()),
        (Role::Paragraph, b"Signed in as Pat"),
        (Role::Heading { level: 1 }, b"What shall we change?"),
        (Role::Paragraph, b"Try a request in acme/shop. Follow the task from its branch to a landed change."),
        (Role::List, b"Suggested requests"),
        (Role::Paragraph, b"The demo model plays these requests from a script."),
        (Role::Paragraph, "Using Demo model · Default".as_bytes()),
        (Role::TextArea, b"Start a chat"),
        (Role::Heading { level: 2 }, b"Your chats"),
        (Role::Paragraph, b"No chats yet."),
    ] {
        let mut found = None;
        for (index, node) in nodes.iter().enumerate() {
            let matches = if role == Role::Paragraph {
                f::node_text(view.tree(), index).as_ref() == name
            } else {
                node.name.as_deref() == Some(name)
            };
            if node.role == role && matches {
                assert!(found.is_none(), "only one {name:?}");
                found = Some(index);
            }
        }
        let Some(at) = found else { panic!("missing {name:?}") };
        assert!(at > after, "{name:?} follows the previous part");
        after = at;
    }
    assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Chats").is_none());
    let ready = f::find(view.tree(), Role::Status, b"Ready").unwrap();
    for node in nodes {
        if node.id == ready {
            assert_eq!(node.styles.as_slice(), [crate::look::STATUS_QUIET]);
        }
    }
}

#[test]
fn other_chats_keep_their_heading_and_no_start() {
    for (row, project, model, demo) in [
        ("other project", f::SECOND_PROJECT, true, true),
        ("forge alone", f::PROJECT, false, true),
        ("no answer", f::PROJECT, true, false),
    ] {
        let root = if demo { f::demo_root(Address::Chats, project, model) } else { f::chats_root() };
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Chats").is_some(), "{row}");
        assert!(f::find(view.tree(), Role::Paragraph, b"New chat").is_none(), "{row}");
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"What shall we change?").is_none(), "{row}");
        for node in view.tree().nodes() {
            assert!(!node.styles.as_slice().contains(&crate::look::CONTEXT), "{row}");
        }
    }
}

#[test]
fn the_context_names_the_deployment_owner_then_the_project_owner_then_the_person() {
    let template = f::demo_root(Address::Chats, f::PROJECT, true);
    for (row, deployment, role, named, expected, project) in [
        (
            "deployment owner",
            Some(c::HeldRole::Owner),
            c::HeldRole::Owner,
            true,
            "Signed in as the deployment’s owner".as_bytes(),
            "Shop · acme/shop on Demo forge",
        ),
        (
            "project owner",
            None,
            c::HeldRole::Owner,
            true,
            "Signed in as the project’s owner".as_bytes(),
            "Shop · acme/shop on Demo forge",
        ),
        (
            "another role",
            Some(c::HeldRole::Policy { role: 1 }),
            c::HeldRole::Policy { role: 1 },
            true,
            b"Signed in as Pat",
            "Shop · acme/shop on Demo forge",
        ),
        (
            "project fallback",
            None,
            c::HeldRole::Policy { role: 1 },
            false,
            b"Signed in as Pat",
            "Project 1 · acme/shop on Demo forge",
        ),
    ] {
        let mut me = template.pages().me(template.client()).unwrap().clone();
        me.deployment_role = deployment;
        me.projects.get_mut(0).unwrap().role = role;
        if !named {
            me.projects.get_mut(0).unwrap().name = None;
        }
        let root = f::demo_frame_root(Address::Chats, f::PROJECT, true, None, Some(me));
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Paragraph, expected).is_some(), "{row}");
        let mut found = false;
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.role == Role::Paragraph && f::node_text(view.tree(), index).as_ref() == project.as_bytes() {
                found = true;
            }
        }
        assert!(found, "{row}");
    }
}

#[test]
fn a_chats_context_follows_the_banner_only_in_the_demo_project_even_without_its_model() {
    for (row, project, model, demo, shown) in [
        ("demo", f::PROJECT, true, true, true),
        ("forge alone", f::PROJECT, false, true, true),
        ("other project", f::SECOND_PROJECT, true, true, false),
        ("no answer", f::PROJECT, true, false, false),
    ] {
        let root = if demo { f::demo_root(Address::Chat { task: 1 }, project, model) } else { f::chat::chat_root() };
        let view = f::page_view(&root);
        let mut banner = None;
        let mut context = None;
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.role == Role::Region && node.name.as_deref() == Some(b"Demo") {
                banner = Some(index);
            }
            if node.styles.as_slice().contains(&crate::look::CONTEXT) {
                context = Some(index);
            }
        }
        assert_eq!(context.is_some(), shown, "{row}");
        if shown {
            let banner = banner.unwrap();
            assert_eq!(context.unwrap(), banner + usize::try_from(view.tree().nodes()[banner].size).unwrap(), "{row}");
        }
    }
}

#[test]
fn the_context_repository_has_its_own_node_and_style() {
    // Each page has exactly one demo repository in its context.
    const REPOSITORIES: u32 = 1;
    for address in [Address::Chats, Address::Chat { task: f::chat::TASK }] {
        let root = f::demo_root(address, f::PROJECT, true);
        let view = f::page_view(&root);
        let mut repositories = skein_lib::List::with_capacity(REPOSITORIES);
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::REPOSITORY) {
                repositories.push(f::node_text(view.tree(), index)).expect("one repository");
                assert_eq!(node.role, Role::Text);
            }
        }
        assert_eq!(repositories.as_slice(), [Box::from(b"acme/shop".as_slice())]);
        assert_eq!(crate::look::NAMES.styles[usize::from(crate::look::REPOSITORY.token)], b"temper-repository");
    }
}

#[test]
fn the_invitation_is_a_named_region_with_its_parts_in_order() {
    let root = f::demo_root(Address::Chats, f::PROJECT, true);
    let view = f::page_view(&root);
    let tree = view.tree();
    let region = f::find(tree, Role::Region, b"What shall we change?").expect("named invitation region");
    let nodes = tree.nodes();
    let mut at = 0;
    for (index, node) in nodes.iter().enumerate() {
        if node.id == region {
            at = index;
        }
    }
    assert_eq!(nodes[at].styles.as_slice(), [crate::look::INVITATION]);
    let end = at + usize::try_from(nodes[at].size).unwrap();
    let mut child = at + 1;
    for (role, name) in [
        (Role::Heading { level: 1 }, b"What shall we change?".as_slice()),
        (Role::Paragraph, b"Try a request in acme/shop. Follow the task from its branch to a landed change."),
        (Role::List, b"Suggested requests"),
        (Role::Paragraph, b"The demo model plays these requests from a script."),
    ] {
        assert!(child < end, "part belongs to invitation");
        assert_eq!(nodes[child].role, role);
        assert_eq!(nodes[child].name.as_deref(), Some(name));
        child += usize::try_from(nodes[child].size).unwrap();
    }
    assert_eq!(child, end, "all four parts stay inside the region");
}
