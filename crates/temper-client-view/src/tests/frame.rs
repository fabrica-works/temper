use super::fixtures;
use skein_ui::{Act, Role};
use temper_client_domain::{Action, pages};

#[test]
fn the_navigation_offers_inbox_chats_and_adoption_in_order() {
    let view = fixtures::signed_in_with_project();
    assert_eq!(
        fixtures::link_names(&view).as_ref(),
        [Box::from(*b"Inbox 0"), Box::from(*b"Chats"), Box::from(*b"Adopt a repository")]
    );
}
#[test]
fn the_banner_says_the_links_state_and_the_persons_name() {
    let view = fixtures::signed_in_with_project();
    assert!(fixtures::find(view.tree(), Role::Status, b"Live").is_some());
    assert!(fixtures::find(view.tree(), Role::Status, b"Pat").is_some());
}
#[test]
fn the_project_chosen_is_the_first_of_me_at_start() {
    let root = fixtures::root_in_project(None);
    assert_eq!(root.pages().project(), Some(fixtures::PROJECT));
}
#[test]
fn no_project_chosen_offers_no_adoption_link() {
    let view = fixtures::built();
    assert!(fixtures::find(view.tree(), Role::Link, b"Adopt a repository").is_none());
}
#[test]
fn adopt_a_repository_is_offered_whatever_the_persons_role() {
    let mut view = fixtures::signed_in_with_project();
    let node = fixtures::find(view.tree(), Role::Link, b"Adopt a repository").expect("adoption link");
    assert_eq!(
        crate::act(&mut view, node, Act::Press),
        Some(Action::Pages(pages::Act::Go {
            address: temper_client_domain::Address::Adopt { project: fixtures::PROJECT },
        }))
    );
}
#[test]
fn the_banner_says_connecting_until_me_then_live_then_offline() {
    let view = fixtures::built();
    assert!(fixtures::find(view.tree(), Role::Status, b"Connecting").is_some());
    let (mut root, watch) = fixtures::root_with_watch(None);
    let mut view = fixtures::view();
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Status, b"Live").is_some());
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Status, b"Offline").is_some());
}
#[test]
fn choosing_another_project_changes_the_navigations_project() {
    let mut root = fixtures::root_in_project(None);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: Action::Pages(pages::Act::Project { project: fixtures::SECOND_PROJECT }),
        },
    );
    let mut view = fixtures::view();
    fixtures::build(&mut view, &root);
    let link = fixtures::find(view.tree(), Role::Link, b"Adopt a repository").expect("selected project's adoption");
    assert_eq!(
        crate::act(&mut view, link, Act::Press),
        Some(Action::Pages(pages::Act::Go {
            address: temper_client_domain::Address::Adopt { project: fixtures::SECOND_PROJECT }
        }))
    );
}
#[test]
fn a_sign_out_that_did_not_reach_the_engine_is_offered_again() {
    let mut root = fixtures::root_in_project(None);
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action: Action::Pages(pages::Act::SignOut) });
    fixtures::root_event(&mut root, temper_client_domain::Event::Client { event: jig_client::Event::SignOutLost });
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Alert, b"Sign out did not reach the engine.").is_some());
    assert!(fixtures::find(tree, Role::Button, b"Sign out").is_some());
}
#[test]
fn a_page_behind_says_behind_after_the_links_state() {
    let (mut root, watch) = fixtures::root_with_watch(None);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert_eq!(
        fixtures::banner_statuses(tree).as_ref(),
        [Box::from(*b"Offline"), Box::from(*b"Behind"), Box::from(*b"Pat")]
    );
}

#[test]
fn signed_out_the_main_says_so_and_nothing_else() {
    let view = fixtures::page_view(&fixtures::signed_out());
    assert_eq!(&*fixtures::main_text(view.tree()), b"Signed out.");
}
#[test]
fn signed_out_the_banner_says_the_links_state_alone() {
    let view = fixtures::page_view(&fixtures::signed_out());
    assert_eq!(fixtures::banner_statuses(view.tree()).as_ref(), [Box::from(*b"Signed out")]);
}
#[test]
fn signed_out_the_banner_offers_no_sign_out_button() {
    let view = fixtures::page_view(&fixtures::signed_out());
    assert!(fixtures::find(view.tree(), Role::Button, b"Sign out").is_none());
}
#[test]
fn signed_out_the_page_offers_no_navigation() {
    let view = fixtures::page_view(&fixtures::signed_out());
    for node in view.tree().nodes() {
        assert_ne!(node.role, Role::Navigation);
    }
}
#[test]
fn an_engine_not_serving_jigs_version_says_temper_is_out_of_date() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    assert_eq!(&*fixtures::main_text(view.tree()), b"This temper is out of date: run the temper the engine runs.");
}
#[test]
fn out_of_date_the_banner_says_the_links_state_alone() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    assert_eq!(fixtures::banner_statuses(view.tree()).as_ref(), [Box::from(*b"Out of date")]);
}
#[test]
fn out_of_date_the_banner_offers_no_sign_out_button() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    assert!(fixtures::find(view.tree(), Role::Button, b"Sign out").is_none());
}
#[test]
fn out_of_date_the_page_offers_no_navigation() {
    let view = fixtures::page_view(&fixtures::out_of_date());
    for node in view.tree().nodes() {
        assert_ne!(node.role, Role::Navigation);
    }
}
