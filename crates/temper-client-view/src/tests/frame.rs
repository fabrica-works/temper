use super::fixtures;
use skein_ui::{Act, Role};
use temper_client_domain::{Action, pages};

#[test]
fn the_navigation_ends_with_accounts() {
    let view = fixtures::signed_in_with_project();
    assert_eq!(
        fixtures::link_names(&view).as_ref(),
        [Box::from(*b"Inbox 0"), Box::from(*b"Chats"), Box::from(*b"Adopt a repository"), Box::from(*b"Accounts")]
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
fn signed_out_the_browser_main_says_how_to_sign_in() {
    let view = fixtures::browser_page_view(&fixtures::signed_out());
    assert_eq!(
        &*fixtures::main_text(view.tree()),
        b"Signed out. To sign in, open a sign-in link temper serve printed.",
    );
}
#[test]
fn out_of_date_the_browser_main_keeps_its_words() {
    let view = fixtures::browser_page_view(&fixtures::out_of_date());
    assert_eq!(&*fixtures::main_text(view.tree()), b"This temper is out of date: run the temper the engine runs.");
}
#[test]
fn the_browser_words_fit_the_views_limits() {
    assert!(crate::Limits::of(&fixtures::domain_limits(), &fixtures::browser_words()).is_some());
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
#[test]
fn each_link_shows_its_name_as_its_text() {
    let view = fixtures::signed_in_with_project();
    let mut links = 0_u32;
    let nodes = view.tree().nodes();
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Link {
            assert_eq!(node.size, 2, "a link holds one text");
            let [run] = nodes[index + 1].text.as_slice() else { panic!("one run per link") };
            assert_eq!(Some(run.text.as_ref()), node.name.as_deref());
            links += 1;
        }
    }
    assert_eq!(links, 4);
}

#[test]
fn the_navigation_is_inside_the_banner_after_projects_before_the_link() {
    let view = fixtures::signed_in_with_project();
    let nodes = view.tree().nodes();
    let mut banner = None;
    let mut nav = None;
    let mut region = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Banner {
            banner = Some(index);
        }
        if node.role == Role::Navigation {
            nav = Some(index);
        }
        if node.region == Some(crate::frame::link_region()) {
            region = Some(index);
        }
    }
    let banner = banner.expect("banner");
    let nav = nav.expect("navigation");
    let region = region.expect("link");
    assert!(nav > banner && nav < banner + usize::try_from(nodes[banner].size).expect("size"));
    assert!(nav < region);
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Button && node.states.as_slice().contains(&skein_ui::State::Selected) {
            assert!(index < nav, "projects precede navigation");
        }
    }
}
#[test]
fn the_navigation_marks_only_the_current_page() {
    use temper_client_domain::{Address, address::Filter};
    for (address, current) in [
        (Address::Inbox { filter: Filter::ALL }, Some(b"Inbox 0".as_slice())),
        (Address::Inbox { filter: Filter { goal: Some(9), ..Filter::ALL } }, Some(b"Inbox 0".as_slice())),
        (Address::Chats, Some(b"Chats".as_slice())),
        (Address::Adopt { project: fixtures::PROJECT }, Some(b"Adopt a repository".as_slice())),
        (Address::Accounts, Some(b"Accounts".as_slice())),
        (Address::Chat { task: 9 }, None),
        (Address::Task { task: 9 }, None),
    ] {
        let root = fixtures::root_at(address, &fixtures::domain_limits()).0;
        let view = fixtures::page_view(&root);
        let mut found = None;
        for node in view.tree().nodes() {
            if node.states.as_slice().contains(&skein_ui::State::Current) {
                assert_eq!(node.role, Role::Link);
                assert!(found.is_none(), "one current page");
                found = node.name.as_deref();
            }
        }
        assert_eq!(found, current);
    }
}

#[test]
fn the_signed_in_corner_groups_link_person_and_sign_out_in_order() {
    let mut root = fixtures::root_in_project(None);
    let view = fixtures::page_view(&root);
    assert_corner(&view, &[(Role::Region, None), (Role::Status, Some(b"Pat")), (Role::Button, Some(b"Sign out"))]);
    fixtures::root_event(&mut root, temper_client_domain::Event::Act { action: Action::Pages(pages::Act::SignOut) });
    fixtures::root_event(&mut root, temper_client_domain::Event::Client { event: jig_client::Event::SignOutLost });
    let view = fixtures::page_view(&root);
    assert_corner(
        &view,
        &[
            (Role::Region, None),
            (Role::Status, Some(b"Pat")),
            (Role::Alert, Some(b"Sign out did not reach the engine.")),
            (Role::Button, Some(b"Sign out")),
        ],
    );
}
#[test]
fn the_unavailable_corner_holds_only_the_link_region() {
    for root in [fixtures::signed_out(), fixtures::out_of_date()] {
        let view = fixtures::page_view(&root);
        assert_corner(&view, &[(Role::Region, None)]);
    }
}

fn assert_corner(view: &crate::View, children: &[(Role, Option<&[u8]>)]) {
    let nodes = view.tree().nodes();
    let mut corner = None;
    let mut banner_end = 0;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Banner {
            banner_end = index.checked_add(usize::try_from(node.size).expect("banner size")).expect("banner end");
        }
        if node.styles.as_slice().contains(&crate::look::CORNER) {
            assert!(corner.is_none(), "one corner");
            assert_eq!(node.role, Role::Slot);
            assert!(index < banner_end, "corner is inside banner");
            corner = Some(index);
        }
    }
    let corner = corner.expect("corner");
    let end = corner.checked_add(usize::try_from(nodes[corner].size).expect("corner size")).expect("corner end");
    assert_eq!(end, banner_end, "corner finishes the banner");
    let mut index = corner.checked_add(1).expect("first child");
    for (role, name) in children {
        let node = &nodes[index];
        assert_eq!(node.role, *role);
        assert_eq!(node.name.as_deref(), *name);
        if *role == Role::Region {
            assert_eq!(node.region, Some(crate::frame::link_region()));
        }
        index = index.checked_add(usize::try_from(node.size).expect("child size")).expect("next child");
    }
    assert_eq!(index, end, "no loose corner contents");
}
