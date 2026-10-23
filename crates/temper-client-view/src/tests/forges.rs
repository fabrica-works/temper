#![allow(clippy::disallowed_types, clippy::disallowed_methods, reason = "tests inspect semantic trees")]
use super::{connect_forge as cf, fixtures as f};
use skein_lib::List;
use skein_ui::{Act, Role, State};
use temper_client_domain::ChatsFilter;
use temper_client_domain::{self as d, Address, Choice, Step};
use temper_client_protocol_wire::{self as wire, v1 as w};
const HERE: u16 = 3;
fn root(address: Address, connection: w::Connection) -> d::Domain {
    f::forge_root(address, cf::rows(connection, w::Offer::Offered))
}
fn action(root: &d::Domain, role: Role, name: &[u8]) -> d::Action {
    let mut view = f::page_view(root);
    let node = f::find(view.tree(), role, name).expect("named action");
    crate::act(&mut view, node, Act::Press).expect("bound action")
}
fn go(address: Address) -> d::Action {
    d::Action::Pages(d::pages::Act::Go { address })
}
#[test]
fn the_forges_page_says_its_words_and_folds_each_row() {
    let view = f::page_view(&root(Address::Forges, cf::connected()));
    cf::words_in_order(
        view.tree(),
        &[
            "Forges",
            "Your forges 1",
            "Each forge temper is connected to, or setting up. Add one to bring its repositories to temper.",
            "+ Add a forge",
        ],
    );
    let row = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Forgejo on this machine"))
        .unwrap();
    assert!(!row.states.as_slice().contains(&State::Expanded));
    assert_eq!(row.description.as_deref(), Some("forge 3 · Forgejo · Connected as temper".as_bytes()));
    assert!(f::find(view.tree(), Role::Term, b"Account").is_some());
    assert!(f::find(view.tree(), Role::Description, "temper · user 42".as_bytes()).is_some());
}
#[test]
fn a_forge_row_opened_offers_its_action() {
    let failed = w::Connection::SetUpFailed(
        w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into() }).unwrap(),
    );
    let root = root(Address::Forges, failed);
    assert_eq!(action(&root, Role::Button, b"Set up again"), d::Action::Forge(d::forge::Act::SetUp { forge: HERE }));
    let view = f::page_view(&root);
    let row = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Forgejo on this machine"))
        .unwrap();
    assert_eq!(row.description.as_deref(), Some("forge 3 · Forgejo · Set up failed: fetch failed".as_bytes()));
    let root = self::root(Address::Forges, w::Connection::Connecting);
    assert_eq!(
        action(&root, Role::Link, "Open its set-up ›".as_bytes()),
        go(Address::ConnectForge { forge: Some(Choice::Forge(HERE)) })
    );
    let root = self::root(Address::Forges, cf::connected());
    assert_eq!(action(&root, Role::Link, b"Adopt a repository"), go(Address::Adopt { project: f::PROJECT }));
    let root = self::root(
        Address::Forges,
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"token refused".as_slice().into() }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Paragraph, b"Restart temper box: its preparation mends it").is_some());
}
#[test]
fn no_forge_of_yours_yet_offers_connect_a_forge() {
    let root = root(Address::Forges, w::Connection::NotSetUp);
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Paragraph, b"No forge of yours yet.").is_some());
    for name in [b"+ Add a forge".as_slice(), b"Connect a forge"] {
        assert_eq!(action(&root, Role::Link, name), go(Address::ConnectForge { forge: None }));
    }
}
#[test]
fn the_navigation_has_forges_with_its_dot_and_words() {
    for (connection, style, text) in [
        (w::Connection::Connecting, crate::look::NAV_LIMITED, b"a forge is setting up".as_slice()),
        (
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            crate::look::NAV_NEEDS,
            b"a forge needs you",
        ),
    ] {
        for address in [Address::Forges, Address::ConnectForge { forge: None }] {
            let root = root(address, connection.clone());
            let view = f::page_view(&root);
            let nav = view
                .tree()
                .nodes()
                .iter()
                .find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Forges"))
                .unwrap();
            assert!(nav.states.as_slice().contains(&State::Current));
            let dot = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&style)).unwrap();
            assert_eq!(dot.role, Role::Text);
            assert_eq!(f::nodes_text(core::slice::from_ref(dot)).as_ref(), text);
        }
    }
}
#[test]
fn the_setup_strip_replaces_the_demo_banner_and_says_whats_missing() {
    let root = root(Address::Chats { filter: ChatsFilter::All, start: false }, w::Connection::NotSetUp);
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Region, b"Setup").is_some());
    assert!(f::find(view.tree(), Role::Region, b"Playground").is_none());
    assert!(f::find(view.tree(), Role::Paragraph, b"temper has no forge and no model yet.").is_some());
    assert_eq!(
        action(&root, Role::Link, b"Continue setting up"),
        go(Address::Welcome { step: Step::Forge { forge: None } })
    );
    for address in [
        Address::Accounts,
        Address::Welcome { step: Step::Model { way: None } },
        Address::Forges,
        Address::ConnectForge { forge: None },
    ] {
        let view = f::page_view(&self::root(address, w::Connection::NotSetUp));
        assert!(f::find(view.tree(), Role::Region, b"Setup").is_none());
    }
    let (mut root, accounts) = f::forge_root_accounts(
        Address::Chats { filter: ChatsFilter::All, start: false },
        cf::rows(cf::connected(), w::Offer::Offered),
    );
    let view = f::page_view(&root);
    assert!(f::find(view.tree(), Role::Paragraph, b"temper has no model yet.").is_some());
    f::welcome_account(&mut root, accounts);
    assert!(f::find(f::page_view(&root).tree(), Role::Region, b"Setup").is_none());
}
#[test]
fn the_demo_banners_connect_a_forge_is_a_link() {
    let root =
        f::demo_frame_root(Address::Chats { filter: ChatsFilter::All, start: false }, f::PROJECT, true, None, None);
    assert_eq!(action(&root, Role::Link, b"Connect a forge"), go(Address::ConnectForge { forge: None }));
    assert!(f::find(f::page_view(&root).tree(), Role::Button, b"Connect a forge").is_none());
}
#[test]
fn a_set_up_under_way_shows_in_either_banner_with_view() {
    for demo in [false, true] {
        let rows = cf::rows(w::Connection::Connecting, w::Offer::Offered);
        let root = if demo {
            f::demo_forges_root(Address::Chats { filter: ChatsFilter::All, start: false }, rows)
        } else {
            f::forge_root(Address::Chats { filter: ChatsFilter::All, start: false }, rows)
        };
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Paragraph, "Forgejo on this machine · Connecting".as_bytes()).is_some());
        assert_eq!(action(&root, Role::Link, b"View"), go(Address::ConnectForge { forge: Some(Choice::Forge(HERE)) }));
        assert!(f::find(view.tree(), Role::Link, b"Connect a forge").is_none());
    }
}
#[test]
fn the_playground_forge_shows_while_the_forge_half_holds() {
    let mut rows = List::with_capacity(2);
    rows.push(cf::row(wire::FORGE, w::Kind::Forgejo, w::Mode::Fake, cf::connected(), w::Offer::NotOffered)).unwrap();
    rows.push(cf::row(HERE, w::Kind::Forgejo, w::Mode::Owned, w::Connection::Connecting, w::Offer::Offered)).unwrap();
    let mut root = f::demo_forges_root(Address::Forges, rows);
    assert!(d::forge::listed::playground(&root).is_some());
    let view = f::page_view(&root);
    let row = f::find(
        view.tree(),
        Role::Disclosure,
        "Playground forge · the playground’s, until a forge of yours connects · acme/shop".as_bytes(),
    )
    .unwrap();
    assert!(!view.tree().nodes().iter().find(|n| n.id == row).unwrap().states.as_slice().contains(&State::Expanded));
    f::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) },
    );
    assert!(d::forge::listed::playground(&root).is_some());
    let root = f::demo_forges_root(Address::Forges, cf::rows(cf::connected(), w::Offer::Offered));
    assert!(d::forge::listed::playground(&root).is_none());
    assert!(f::find(f::page_view(&root).tree(), Role::Heading { level: 2 }, b"The playground").is_none());
}

#[test]
fn set_up_again_waits_for_an_offer() {
    let failed = w::Connection::SetUpFailed(
        w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into() }).unwrap(),
    );
    let root = f::forge_root(Address::Forges, cf::rows(failed, w::Offer::NotOffered));
    assert!(f::find(f::page_view(&root).tree(), Role::Button, b"Set up again").is_none());
}

#[test]
fn a_full_forge_reason_and_login_fit_after_utf8_replacement() {
    for connection in [
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::from([255_u8; 256]) }).unwrap(),
        ),
        w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: u64::MAX, login: Box::from([255_u8; 256]) })
                .unwrap(),
        ),
    ] {
        let root = root(Address::Forges, connection);
        let domain = f::domain_limits();
        let words = f::words();
        let limits = crate::Limits::of(&domain, &words).unwrap();
        let mut view = crate::View::new(&limits, &domain, words, wire::FORGE);
        crate::build(&mut view, &root, f::NOW, &mut skein_lib::Queue::with_capacity(limits.tree.patches));
        let row = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Forgejo on this machine"))
            .unwrap();
        let summary = core::str::from_utf8(row.description.as_deref().unwrap()).unwrap();
        assert_eq!(summary.matches('�').count(), 256_usize);
    }
}

#[test]
fn the_github_row_draws_its_app_installations_and_last_connect() {
    let root = f::forge_root(Address::Forges, cf::github_rows(true));
    let view = f::browser_page_view(&root);
    cf::words_in_order(
        view.tree(),
        &[
            "Account",
            "my-app[bot] · user 9",
            "temper’s App",
            "my-app · ",
            "acme",
            ", installation 2 · suspended",
            ", installation 3 · uninstalled at the next reconcile",
            "The last connect stopped: fixture stopped",
            "Install on another account",
        ],
    );
}
#[test]
fn install_on_another_account_follows_an_accepted_installation() {
    for (accepted, name) in [(true, b"Install on another account".as_slice()), (false, b"Install on GitHub")] {
        let root = f::forge_root(Address::Forges, cf::github_rows(accepted));
        assert_eq!(
            action(&root, Role::Button, name),
            d::Action::Forge(d::forge::Act::Install { forge: 2, back: b"/forges".as_slice().into() })
        );
    }
}
#[test]
fn all_sixty_four_installations_fit_without_shortening_their_logins() {
    let mut rows = cf::github_rows(true);
    let mut row = rows.as_slice()[0].clone().into_parts();
    rows.clear();
    row.installations = List::with_capacity(d::forge::forges::INSTALLATIONS);
    for id in 1..=u64::from(d::forge::forges::INSTALLATIONS) {
        row.installations
            .push(
                w::Installation::new(
                    &w::CEILINGS,
                    w::InstallationParts {
                        id,
                        account: Some(
                            w::Account::new(&w::CEILINGS, w::AccountParts { id, login: Box::from([255_u8; 256]) })
                                .unwrap(),
                        ),
                        suspended: true,
                        stray: true,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    rows.push(w::ForgeRow::new(&w::CEILINGS, row).unwrap()).unwrap();
    let root = f::forge_root(Address::Forges, rows);
    let domain = f::limits();
    let words = f::browser_words();
    let limits = crate::Limits::of(&domain, &words).unwrap();
    let mut view = crate::View::new(&limits, &domain, words, wire::FORGE);
    crate::build(&mut view, &root, f::NOW, &mut skein_lib::Queue::with_capacity(limits.tree.patches));
    let text = f::text(view.tree());
    let text = core::str::from_utf8(&text).unwrap();
    assert_eq!(text.matches('�').count(), 64 * 256);
}
