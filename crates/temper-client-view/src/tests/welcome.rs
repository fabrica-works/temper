#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test assertions inspect semantic trees and their bound controls"
)]
use super::fixtures as f;
use skein_ui::{Act, Role, State};
use temper_client_domain::{self as d, Address, Step, pages};

fn node(tree: &skein_ui::Tree, role: Role, name: &str) -> skein_ui::NodeId {
    f::find(tree, role, name.as_bytes()).unwrap_or_else(|| panic!("missing {role:?}: {name}"))
}
fn press(root: &d::Domain, role: Role, name: &str) -> d::Action {
    let mut view = f::page_view(root);
    let id = node(view.tree(), role, name);
    crate::act(&mut view, id, Act::Press).expect("bound control")
}
#[test]
fn the_welcome_has_a_bare_frame_and_no_banner() {
    let root = f::welcome_root(Step::Welcome, true);
    let view = f::page_view(&root);
    let tree = view.tree();
    node(tree, Role::Button, "Sign out");
    assert_eq!(f::banner_statuses(tree).as_ref(), &[Box::from(b"Pat".as_slice())]);
    for role in [Role::Disclosure, Role::Navigation] {
        assert!(!tree.nodes().iter().any(|node| node.role == role));
    }
    assert!(f::find(tree, Role::Region, b"Playground").is_none());
    assert!(!String::from_utf8_lossy(&f::text(tree)).contains("No model yet"));
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Region).count(), 2);
}
#[test]
fn the_welcome_step_says_its_words_and_offers_both_cards() {
    let root = f::welcome_root(Step::Welcome, true);
    assert!(root.pages().welcome().unwrap().decided);
    let view = f::page_view(&root);
    let mut bytes = Vec::new();
    for node in view.tree().nodes() {
        if node.role == Role::Button && node.text.is_empty() {
            bytes.extend_from_slice(node.name.as_deref().unwrap());
        }
        for run in &node.text {
            bytes.extend_from_slice(&run.text);
        }
    }
    let words = String::from_utf8(bytes).unwrap();
    let mut remaining = words.as_str();
    for phrase in [
        "Welcome",
        "Forge",
        "Model",
        "Ready",
        "Welcome to temper",
        "Ask for a change in a chat. temper’s agents make it on a branch, open a pull request, and land it once CI passes.",
        "Try the playground",
        "A built-in scripted model and forge. Ask one of three requests and follow its change from a branch to main.",
        "Nothing leaves this machine, and nothing costs. · Ready at once",
        "Try the playground",
        "Set up temper",
        "Your forge: GitHub, or a Forgejo temper runs on this machine. Your model: an OpenAI API key.",
        "About five minutes",
        "Set up temper",
        "Started in the playground? Connect your own forge and model whenever you like; the playground then steps aside.",
    ] {
        let at = remaining.find(phrase).unwrap_or_else(|| panic!("missing or out of order: {phrase}"));
        remaining = &remaining[at + phrase.len()..];
    }
    assert_eq!(press(&root, Role::Button, "Try the playground"), d::Action::Pages(pages::Act::Playground));
    assert_eq!(
        press(&root, Role::Link, "Set up temper"),
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge } })
    );
}
#[test]
fn a_document_with_no_demo_has_neither_the_playground_card_nor_the_closing_line() {
    let root = f::welcome_root(Step::Welcome, false);
    assert!(root.pages().welcome().unwrap().decided);
    let view = f::page_view(&root);
    node(view.tree(), Role::Region, "Set up temper");
    let words = String::from_utf8_lossy(&f::text(view.tree())).into_owned();
    assert!(!words.contains("playground"));
}
#[test]
fn the_welcome_shows_its_heading_alone_until_it_decides() {
    let (mut root, _, _) = f::root_at(Address::Chats, &f::limits());
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Welcome }) });
    assert!(!root.pages().welcome().unwrap().decided);
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Welcome to temper");
    assert!(!view.tree().nodes().iter().any(|node| node.role == Role::Region));
    assert!(
        !view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Paragraph && node.styles.as_slice().contains(&crate::look::WELCOME_LEAD))
    );
}
#[test]
fn the_steps_mark_the_current_one() {
    for (step, name) in
        [(Step::Welcome, "Welcome"), (Step::Forge, "Forge"), (Step::Model, "Model"), (Step::Ready, "Ready")]
    {
        let view = f::page_view(&f::welcome_root(step, true));
        node(view.tree(), Role::List, "Steps");
        let current: Vec<_> =
            view.tree().nodes().iter().filter(|node| node.states.as_slice().contains(&State::Current)).collect();
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].role, Role::Item);
        assert_eq!(
            f::nodes_text(
                &view.tree().nodes()[view.tree().nodes().iter().position(|node| node.id == current[0].id).unwrap()..]
                    [..usize::try_from(current[0].size).unwrap()]
            )
            .as_ref(),
            name.as_bytes()
        );
    }
}
#[test]
fn forge_and_ready_say_they_are_not_available_yet() {
    for (step, heading, phrase) in [
        (Step::Forge, "Connect a forge", "Connecting a forge here is not available yet."),
        (Step::Ready, "Almost ready", "This step is not available yet."),
    ] {
        let root = f::welcome_root(step, true);
        let view = f::page_view(&root);
        node(view.tree(), Role::Heading { level: 1 }, heading);
        node(view.tree(), Role::Paragraph, phrase);
        if step == Step::Forge {
            node(
                view.tree(),
                Role::Paragraph,
                "Where your repositories live: temper’s agents push branches and open pull requests there.",
            );
            assert_eq!(
                press(&root, Role::Link, "Connect a model"),
                d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Model } })
            );
        }
    }
}
#[test]
fn the_model_step_offers_the_accounts_ways() {
    let mut root = super::accounts::offered_root();
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model }) });
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Connect a model");
    node(view.tree(), Role::Paragraph, "temper’s agents run on it, and what they use is billed to its account.");
    node(view.tree(), Role::Button, "Continue with ChatGPT");
    let action = press(&root, Role::Button, "Use an API key");
    let d::Action::Pages(pages::Act::Accounts(_)) = &action else { panic!("shared account binding") };
    f::root_event(&mut root, d::Event::Act { action });
    let view = f::page_view(&root);
    assert!(view.tree().nodes().iter().any(|node| node.role == Role::TextField));
}
#[test]
fn each_later_step_offers_back_later_and_the_playground_while_the_demo_is_answered() {
    for (step, back) in [(Step::Forge, Step::Welcome), (Step::Model, Step::Forge), (Step::Ready, Step::Model)] {
        for demo in [false, true] {
            let root = f::welcome_root(step, demo);
            assert_eq!(
                press(&root, Role::Link, "‹ Back"),
                d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: back } })
            );
            assert_eq!(press(&root, Role::Link, "Later"), d::Action::Pages(pages::Act::Go { address: Address::Chats }));
            let view = f::page_view(&root);
            assert_eq!(f::find(view.tree(), Role::Button, b"Try the playground instead").is_some(), demo);
            if demo {
                assert_eq!(
                    press(&root, Role::Button, "Try the playground instead"),
                    d::Action::Pages(pages::Act::Playground)
                );
            }
        }
    }
}
#[test]
fn an_unavailable_welcome_keeps_the_signed_out_and_out_of_date_frame() {
    for event in [
        d::Event::SignedOut,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } },
        },
    ] {
        let mut root = f::welcome_root(Step::Welcome, true);
        f::root_event(&mut root, event);
        let view = f::page_view(&root);
        node(view.tree(), Role::Disclosure, "Menu");
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Welcome to temper").is_none());
    }
}
