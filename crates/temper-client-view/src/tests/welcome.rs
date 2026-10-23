#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test assertions inspect semantic trees and their bound controls"
)]
use super::connect_forge as forge;
use super::fixtures as f;
use skein_ui::{Act, Role, State};
use temper_client_domain::ChatsFilter;
use temper_client_domain::{self as d, Address, Step, pages};
use temper_client_protocol_wire::v1 as w;

const HERE: u16 = 3;
fn model_view(rows: skein_lib::List<w::ForgeRow>) -> crate::View {
    f::page_view(&f::forge_root(Address::Welcome { step: Step::Model { way: None } }, rows))
}
fn before_model(tree: &skein_ui::Tree) -> &[skein_ui::Node] {
    let main = tree.nodes().iter().position(|n| n.role == Role::Main).expect("welcome's main");
    let heading = node(tree, Role::Heading { level: 1 }, "Connect a model");
    let heading = tree.nodes().iter().position(|n| n.id == heading).unwrap();
    &tree.nodes()[main..heading]
}
fn model_forge_line(tree: &skein_ui::Tree) -> Option<&skein_ui::Node> {
    before_model(tree).iter().find(|n| n.role == Role::Paragraph)
}
fn line_before_model(tree: &skein_ui::Tree, expected: &str) -> skein_ui::NodeId {
    let line = model_forge_line(tree).expect("forge line before Model's heading");
    assert_eq!(line.name.as_deref(), Some(expected.as_bytes()));
    assert_eq!(before_model(tree).iter().filter(|n| n.role == Role::Paragraph).count(), 1);
    line.id
}
fn each_step(tree: &skein_ui::Tree, shown: bool) {
    assert_eq!(f::find(tree, Role::Disclosure, b"Each step").is_some(), shown);
}

#[test]
fn model_without_a_persons_forge_has_no_line() {
    let view = model_view(skein_lib::List::with_capacity(0));
    assert!(model_forge_line(view.tree()).is_none());
    each_step(view.tree(), false);
}

#[test]
fn model_names_the_new_project_while_forgejo_sets_up() {
    let (mut root, watch) = f::forge_root_watch(
        Address::Welcome { step: Step::Forge { forge: None } },
        forge::rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    for action in [d::forge::Act::SetupNew { new: true }, d::forge::Act::SetUp { forge: HERE }] {
        f::root_event(&mut root, d::Event::Act { action: d::Action::Forge(action) });
    }
    // The setup request supplies the names until the watch's row carries them.
    forge::deliver(
        &mut root,
        watch,
        forge::rows(
            w::Connection::SettingUp(w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: None }).unwrap()),
            w::Offer::Offered,
        ),
    );
    f::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let view = f::page_view(&root);
    line_before_model(view.tree(), "Forgejo on this machine · Setting up · work/hello, into the new project hello");
    each_step(view.tree(), true);
}

#[test]
fn model_ticks_the_first_repository_made_in_its_project() {
    let view = model_view(forge::first_rows(forge::connected(), forge::made()));
    let line = line_before_model(view.tree(), "Forgejo on this machine · work/hello, in Shop");
    let line = view.tree().nodes().iter().find(|n| n.id == line).unwrap();
    assert!(line.styles.as_slice().contains(&crate::look::FLOW_DONE));
    each_step(view.tree(), false);
}

#[test]
fn model_says_github_has_no_repository_chosen_yet() {
    let view = model_view(forge::github_rows(true));
    line_before_model(view.tree(), "GitHub · no repository chosen yet");
    each_step(view.tree(), true);
}

#[test]
fn model_keeps_github_connecting_until_an_installation_is_accepted() {
    let view = model_view(forge::github_rows(false));
    line_before_model(view.tree(), "GitHub · Connecting: installed nowhere yet");
    each_step(view.tree(), true);
}

#[test]
fn model_says_which_setup_failed_and_keeps_its_steps() {
    let view = model_view(forge::rows(
        w::Connection::SetUpFailed(
            w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into() }).unwrap(),
        ),
        w::Offer::Offered,
    ));
    let line = line_before_model(view.tree(), "Forgejo on this machine · Set up failed: fetch failed");
    let line = view.tree().nodes().iter().find(|n| n.id == line).unwrap();
    assert!(!line.styles.as_slice().contains(&crate::look::FLOW_DONE));
    each_step(view.tree(), true);
}

#[test]
fn model_names_the_failed_first_repository_step() {
    for (step, connection, failed) in [
        (
            w::CreateStep::Admin,
            "Forgejo on this machine · Set up failed: checking the admin's token · work/hello, into Shop",
            "Making work/hello failed: checking the admin's token",
        ),
        (
            w::CreateStep::Org,
            "Forgejo on this machine · Set up failed: making the organization · work/hello, into Shop",
            "Making work/hello failed: making the organization",
        ),
        (
            w::CreateStep::Repository,
            "Forgejo on this machine · Set up failed: making the repository · work/hello, into Shop",
            "Making work/hello failed: making the repository",
        ),
        (
            w::CreateStep::File,
            "Forgejo on this machine · Set up failed: adding the starter workflow · work/hello, into Shop",
            "Making work/hello failed: adding the starter workflow",
        ),
    ] {
        let view = model_view(forge::first_rows(
            forge::connected(),
            w::FirstState::Failed(w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step }).unwrap()),
        ));
        line_before_model(view.tree(), connection);
        each_step(view.tree(), true);
        forge::words_in_order(view.tree(), &[failed, "failed"]);
        node(view.tree(), Role::Status, "failed");
    }
}

#[test]
fn model_keeps_steps_while_the_first_repository_is_making() {
    let view = model_view(forge::first_rows(forge::connected(), w::FirstState::Making));
    line_before_model(view.tree(), "Forgejo on this machine · Connected as temper (42) · work/hello, into Shop");
    each_step(view.tree(), true);
    node(view.tree(), Role::Status, "under way");
}

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
        "Your forge: GitHub, or a Forgejo temper runs on this machine. Your model: your ChatGPT plan, or an OpenAI API key.",
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
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } })
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
    let (mut root, _, _) = f::root_at(Address::Chats { filter: ChatsFilter::All, start: false }, &f::limits());
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
    for (step, name) in [
        (Step::Welcome, "Welcome"),
        (Step::Forge { forge: None }, "Forge"),
        (Step::Model { way: None }, "Model"),
        (Step::Ready, "Ready"),
    ] {
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
fn the_model_step_offers_the_accounts_ways() {
    let mut root = super::connect_model::offered_root();
    f::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Connect a model");
    node(view.tree(), Role::Paragraph, "temper’s agents run on it, and what they use is billed to its account.");
    node(view.tree(), Role::Button, "Continue with ChatGPT");
    let action = press(&root, Role::Link, "Use an API key");
    let d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Model { way: Some(_) } } }) = &action
    else {
        panic!("shared account binding")
    };
    let d::Action::Pages(pages::Act::Go { address }) = action else { unreachable!() };
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
    let view = f::page_view(&root);
    assert!(view.tree().nodes().iter().any(|node| node.role == Role::TextField));
}
#[test]
fn each_later_step_offers_back_later_and_the_playground_while_the_demo_is_answered() {
    for (step, back) in [
        (Step::Forge { forge: None }, Step::Welcome),
        (Step::Model { way: None }, Step::Forge { forge: None }),
        (Step::Ready, Step::Model { way: None }),
    ] {
        for demo in [false, true] {
            let root = f::welcome_root(step, demo);
            assert_eq!(
                press(&root, Role::Link, "‹ Back"),
                d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: back } })
            );
            assert_eq!(
                press(&root, Role::Link, "Later"),
                d::Action::Pages(pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } })
            );
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

fn ready_model_fixture() -> (d::Domain, skein_lib::Token) {
    let (mut root, watch) = super::connect_model::flow_fixture(d::WayKind::SignIn);
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) });
    (root, watch)
}
fn signing_document() -> jig_client::contract::OwnerAccounts {
    let mut doc =
        super::accounts::card_document(b"openai", b"alice@example.com", jig_client::contract::OwnerAccountKind::SignIn);
    doc.accounts.clear();
    doc.usable = false;
    doc
}
#[test]
fn ready_waits_for_the_accounts_document_before_drawing_a_model_line() {
    let (root, _) = ready_model_fixture();
    let view = f::page_view(&root);
    let words = String::from_utf8_lossy(&f::text(view.tree())).into_owned();
    assert!(!words.contains("Model:"));
    assert!(f::find(view.tree(), Role::Link, b"Connect a model").is_none());
}
#[test]
fn ready_names_an_own_sign_in_under_way_without_a_connect_act() {
    let (mut root, watch) = ready_model_fixture();
    super::accounts::deliver(&mut root, watch, signing_document());
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Model: ChatGPT · signing in");
    assert!(f::find(view.tree(), Role::Link, b"Connect a model").is_none());
}
#[test]
fn ready_does_not_name_another_holders_sign_in() {
    let (mut root, watch) = ready_model_fixture();
    let mut doc = signing_document();
    doc.signing.get_mut(0).unwrap().holder = jig_client::contract::OwnerAccountHolder::Deployment;
    super::accounts::deliver(&mut root, watch, doc);
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Model: none yet");
    node(view.tree(), Role::Link, "Connect a model");
}
#[test]
fn ready_names_a_usable_account_before_a_sign_in_under_way() {
    let (mut root, watch) = ready_model_fixture();
    super::accounts::deliver(
        &mut root,
        watch,
        super::accounts::card_document(b"openai", b"alice@example.com", jig_client::contract::OwnerAccountKind::SignIn),
    );
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Model: ChatGPT · alice@example.com · OpenAI · held by Pat");
    assert!(f::find(view.tree(), Role::Link, b"Connect a model").is_none());
}

fn configured_model_document() -> jig_client::contract::OwnerAccounts {
    let mut document = signing_document();
    document.signing.clear();
    document.usable = true;
    document
}
fn configured_model_fixture(address: Address) -> d::Domain {
    let (mut root, watch) = f::forge_root_accounts(
        address,
        super::connect_forge::rows(
            temper_client_protocol_wire::v1::Connection::Connecting,
            temper_client_protocol_wire::v1::Offer::Offered,
        ),
    );
    let document = configured_model_document();
    super::accounts::deliver(&mut root, watch, document);
    root
}
#[test]
fn ready_names_the_configured_model_without_an_account_row() {
    let root = configured_model_fixture(Address::Welcome { step: Step::Ready });
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Model: the default model");
    assert!(f::find(view.tree(), Role::Link, b"Connect a model").is_none());
}
#[test]
fn the_setup_strip_continues_to_ready_with_a_configured_model() {
    let root = configured_model_fixture(Address::Chats { filter: ChatsFilter::All, start: false });
    assert_eq!(
        press(&root, Role::Link, "Continue setting up"),
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Ready } })
    );
}

#[test]
fn ready_names_the_new_chat_templates_model_and_effort() {
    use jig_client::contract as c;
    let text_bytes = f::limits().client.domain.text;
    let (mut root, me_watch, _, _, accounts_watch) =
        f::root_at_full(Address::Welcome { step: Step::Ready }, &f::limits(), b"Shop");
    let template_root = f::chat::messaging_root();
    let mut me = pages::accounts::current_me(template_root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    let template = project.templates.get_mut(0).unwrap();
    template.llm = Some(c::LlmChoice {
        account: None,
        main: c::ModelEffort {
            model: c::Text::new(b"configured-model".as_slice().into(), text_bytes).unwrap(),
            effort: Some(c::Text::new(b"high".as_slice().into(), text_bytes).unwrap()),
        },
        subagents: skein_lib::List::with_capacity(0),
    });
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    let document = configured_model_document();
    super::accounts::deliver(&mut root, accounts_watch.unwrap(), document);
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Model: configured-model · high");
}
