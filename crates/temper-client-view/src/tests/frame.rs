use super::fixtures;
use skein_ui::{Act, Role};
use temper_client_domain::{Action, pages};

#[test]
fn the_navigation_is_named_main_navigation() {
    let view = fixtures::signed_in_with_project();
    assert!(fixtures::person_find(view.tree(), Role::Navigation, b"Main navigation").is_some());
}

#[test]
fn the_navigation_ends_with_accounts() {
    let view = fixtures::signed_in_with_project();
    assert_eq!(
        fixtures::link_names(&view).as_ref(),
        [Box::from(*b"Inbox"), Box::from(*b"Chats"), Box::from(*b"Adopt a repository"), Box::from(*b"Accounts")]
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
    assert_menu(
        &view,
        &[
            (Role::Button, Some(b"Second")),
            (Role::Button, Some(b"Shop")),
            (Role::Navigation, Some(b"Main navigation")),
            (Role::Slot, None),
        ],
    );
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
    assert_menu(
        &view,
        &[
            (Role::Button, Some(b"Shop")),
            (Role::Button, Some(b"Second")),
            (Role::Navigation, Some(b"Main navigation")),
            (Role::Slot, None),
        ],
    );
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
    for (index, node) in nodes.iter().enumerate().take(banner + usize::try_from(nodes[banner].size).expect("size")) {
        if node.role == Role::Button && node.states.as_slice().contains(&skein_ui::State::Pressed) {
            assert!(index < nav, "projects precede navigation");
        }
    }
}
#[test]
fn the_navigation_marks_only_the_current_page() {
    use temper_client_domain::{Address, address::Filter};
    for (address, current) in [
        (Address::Inbox { filter: Filter::ALL }, Some(b"Inbox".as_slice())),
        (Address::Inbox { filter: Filter { goal: Some(9), ..Filter::ALL } }, Some(b"Inbox".as_slice())),
        (Address::Chats, Some(b"Chats".as_slice())),
        (Address::Adopt { project: fixtures::PROJECT }, Some(b"Adopt a repository".as_slice())),
        (Address::Accounts, Some(b"Accounts".as_slice())),
        (Address::Chat { task: 9 }, Some(b"Chats".as_slice())),
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

#[test]
fn navigation_shows_the_inbox_count_only_when_nonzero() {
    const NEEDS: u32 = 3;
    for (count, expected) in [(0, b"Inbox".as_slice()), (NEEDS, b"Inbox 3")] {
        let (mut root, watch) = fixtures::root_with_watch(None);
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.needs_total = count;
        fixtures::root_event(
            &mut root,
            temper_client_domain::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(
                            Box::new(me),
                        )),
                    },
                },
            },
        );
        let view = fixtures::page_view(&root);
        let id = fixtures::find(view.tree(), Role::Link, expected).expect("inbox link");
        let nodes = view.tree().nodes();
        let mut at = None;
        for (index, node) in nodes.iter().enumerate() {
            if node.id == id {
                at = Some(index);
                break;
            }
        }
        let at = at.expect("inbox link position");
        assert_eq!(nodes[at].id, id);
        let end = at + usize::try_from(nodes[at].size).unwrap();
        let mut counts = skein_lib::List::with_capacity(1);
        for node in &nodes[at + 1..end] {
            if node.styles.as_slice().contains(&skein_ui::Style { owner: crate::frame::owner(), token: 92 }) {
                counts.push(node).expect("only one count node");
            }
        }
        if count == 0 {
            assert!(counts.is_empty());
        } else {
            let [node] = counts.as_slice() else { panic!("one count node") };
            assert_eq!(node.role, Role::Text);
            assert_eq!(node.text.get(0).unwrap().text.as_ref(), b"3");
        }
    }
}

#[test]
fn the_banner_menu_holds_projects_navigation_and_corner_in_order() {
    let view = fixtures::signed_in_with_project();
    assert_menu(
        &view,
        &[
            (Role::Button, Some(b"Shop")),
            (Role::Button, Some(b"Second")),
            (Role::Navigation, Some(b"Main navigation")),
            (Role::Slot, None),
        ],
    );
}

#[test]
fn projects_me_leaves_unnamed_take_their_numbers_in_the_banner() {
    let (mut root, watch) = fixtures::root_with_watch(Some(fixtures::PROJECT));
    let mut me = root.pages().me(root.client()).expect("me").clone();
    for index in 0..me.projects.len() {
        me.projects.get_mut(index).expect("project").name = None;
    }
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
    );
    let view = fixtures::page_view(&root);
    assert_menu(
        &view,
        &[
            (Role::Button, Some(b"Project 1")),
            (Role::Button, Some(b"Project 2")),
            (Role::Navigation, Some(b"Main navigation")),
            (Role::Slot, None),
        ],
    );
    for name in [b"Project 1", b"Project 2"] {
        assert!(fixtures::person_find(view.tree(), Role::Button, name).is_some(), "a person finds {name:?}");
    }
}

#[test]
fn the_banner_menu_keeps_navigation_and_corner_while_connecting() {
    assert_menu(&fixtures::built(), &[(Role::Navigation, Some(b"Main navigation")), (Role::Slot, None)]);
}

#[test]
fn the_unavailable_banner_menu_holds_its_corner_alone() {
    for root in [fixtures::signed_out(), fixtures::out_of_date()] {
        assert_menu(&fixtures::page_view(&root), &[(Role::Slot, None)]);
    }
}

#[test]
fn the_offline_banner_menu_keeps_its_choices() {
    let (mut root, watch) = fixtures::root_with_watch(None);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    assert_menu(
        &fixtures::page_view(&root),
        &[
            (Role::Button, Some(b"Shop")),
            (Role::Button, Some(b"Second")),
            (Role::Navigation, Some(b"Main navigation")),
            (Role::Slot, None),
        ],
    );
}

#[test]
fn the_qa_controls_keep_their_accessible_names_with_the_menu_closed() {
    for (root, controls) in [
        (
            fixtures::chats_root(),
            &[
                (Role::Button, b"Sign out".as_slice()),
                (Role::Link, b"Chats"),
                (Role::TextArea, b"Start a chat"),
                (Role::Button, b"Start"),
            ][..],
        ),
        (
            fixtures::chat::messaging_root(),
            &[
                (Role::Button, b"Sign out".as_slice()),
                (Role::Link, b"Chats"),
                (Role::TextArea, b"Message"),
                (Role::Button, b"Send"),
                (Role::Log, b"Conversation"),
            ][..],
        ),
    ] {
        let view = fixtures::page_view(&root);
        for (role, name) in controls {
            assert!(fixtures::person_find(view.tree(), *role, name).is_some(), "{role:?}: {name:?}");
        }
    }
}

fn assert_menu(view: &crate::View, children: &[(Role, Option<&[u8]>)]) {
    let nodes = view.tree().nodes();
    let mut banner = None;
    for (index, node) in nodes.iter().enumerate() {
        if node.role == Role::Banner {
            assert!(banner.is_none(), "one banner");
            banner = Some(index);
        }
    }
    let banner = banner.expect("banner");
    let end = banner.checked_add(usize::try_from(nodes[banner].size).expect("banner size")).expect("banner end");
    let brand = banner.checked_add(1).expect("brand index");
    assert_eq!(nodes[brand].role, Role::Paragraph);
    assert_eq!(nodes[brand].name.as_deref(), Some(b"temper".as_slice()));
    let menu = brand.checked_add(usize::try_from(nodes[brand].size).expect("brand size")).expect("menu index");
    assert_eq!(nodes[menu].role, Role::Disclosure);
    assert_eq!(nodes[menu].name.as_deref(), Some(b"Menu".as_slice()));
    assert_eq!(nodes[menu].styles.as_slice(), [crate::look::MENU]);
    assert!(!nodes[menu].states.as_slice().contains(&skein_ui::State::Expanded));
    let menu_end = menu.checked_add(usize::try_from(nodes[menu].size).expect("menu size")).expect("menu end");
    assert_eq!(menu_end, end, "menu finishes banner");
    assert_eq!(fixtures::person_find(view.tree(), Role::Disclosure, b"Menu"), Some(nodes[menu].id));
    let mut index = menu.checked_add(1).expect("first child");
    for (role, name) in children {
        assert_eq!(nodes[index].role, *role);
        assert_eq!(nodes[index].name.as_deref(), *name);
        index = index.checked_add(usize::try_from(nodes[index].size).expect("child size")).expect("next child");
    }
    assert_eq!(index, end, "all banner contents follow the brand inside Menu");
}
