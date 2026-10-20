use super::fixtures;
use crate::look::*;
use skein_ui::Role;

#[test]
fn the_frame_carries_its_tokens() {
    let view = fixtures::page_view(&fixtures::inbox_root());
    for (role, token) in [(Role::Banner, BANNER), (Role::Navigation, NAVIGATION), (Role::Main, MAIN)] {
        assert_eq!(fixtures::styles(view.tree(), role).as_ref(), [Box::from([token])], "{role:?}");
    }
}
#[test]
fn the_looks_names_pass_skeins_check() {
    assert!(skein_ui::look::check(&[NAMES]).is_none());
}
#[test]
fn each_style_or_region_is_named_under_temper() {
    let named: [(skein_ui::Style, &[u8]); 38] = [
        (BANNER, b"temper-banner"),
        (NAVIGATION, b"temper-navigation"),
        (MAIN, b"temper-main"),
        (CARD, b"temper-card"),
        (TURN, b"temper-turn"),
        (COMPOSER, b"temper-composer"),
        (BRAND, b"temper-brand"),
        (LIVE, b"temper-live"),
        (CONNECTING, b"temper-connecting"),
        (OFFLINE, b"temper-offline"),
        (BEHIND, b"temper-behind"),
        (ROWS, b"temper-rows"),
        (ROW, b"temper-row"),
        (ENTRY, b"temper-entry"),
        (STATUS_YOURS, b"temper-status-yours"),
        (STATUS_HELD, b"temper-status-held"),
        (STATUS_RETRYING, b"temper-status-retrying"),
        (STATUS_ACTIVE, b"temper-status-active"),
        (STATUS_QUIET, b"temper-status-quiet"),
        (TIME, b"temper-time"),
        (HINT, b"temper-hint"),
        (CORNER, b"temper-corner"),
        (ACCOUNTS, b"temper-accounts"),
        (ACCOUNT_LINE, b"temper-account-line"),
        (CHAT, b"temper-chat"),
        (CHATBAR, b"temper-chatbar"),
        (THREAD, b"temper-thread"),
        (MESSAGE, b"temper-message"),
        (BY_PERSON, b"temper-by-person"),
        (BY_AGENT, b"temper-by-agent"),
        (BY_TOOL, b"temper-by-tool"),
        (BY_SYSTEM, b"temper-by-system"),
        (WHO, b"temper-who"),
        (SAYS, b"temper-says"),
        (WORK, b"temper-work"),
        (MARKER, b"temper-marker"),
        (CARDS, b"temper-cards"),
        (COMPOSER_AREA, b"temper-composer-area"),
    ];
    for (style, name) in named {
        assert_eq!(NAMES.style(style), Some(name));
    }
    assert_eq!(NAMES.region(crate::frame::link_region()), Some(&b"temper-link"[..]));
}
#[test]
fn each_pages_main_carries_the_main_token() {
    let builds: [fn() -> temper_client_domain::Domain; 6] = [
        fixtures::signed_out,
        fixtures::out_of_date,
        fixtures::inbox_root,
        fixtures::adoption_root,
        fixtures::task::held_root,
        fixtures::chats_root,
    ];
    for build in builds {
        let view = fixtures::page_view(&build());
        assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN])]);
    }
    let view = fixtures::page_view(&fixtures::account_root().0);
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN])]);
    let view = fixtures::page_view(&fixtures::chat::chat_root());
    assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN, CHAT])]);
}
#[test]
fn the_composers_carry_the_composer_token() {
    for root in [fixtures::chats_root(), fixtures::chat::messaging_root()] {
        let view = fixtures::page_view(&root);
        assert_eq!(fixtures::styles(view.tree(), Role::TextArea).as_ref(), [Box::from([COMPOSER])]);
    }
}

#[test]
fn the_chat_structure_tokens_stay_on_their_roles() {
    let view = fixtures::page_view(&super::chats::listed_root());
    for (token, role) in [
        (BRAND, Role::Paragraph),
        (LIVE, Role::Status),
        (ROWS, Role::List),
        (ROW, Role::Item),
        (ENTRY, Role::Slot),
        (CORNER, Role::Slot),
        (TIME, Role::Paragraph),
        (HINT, Role::Paragraph),
    ] {
        let mut found = false;
        for node in view.tree().nodes() {
            if node.styles.as_slice().contains(&token) {
                assert_eq!(node.role, role);
                found = true;
            }
        }
        assert!(found, "{token:?}");
    }
    for (line, token) in super::chats::status_rows() {
        let (tree, _) = super::chats::tree(&line);
        assert_eq!(fixtures::styles(&tree, Role::Status).as_ref(), [Box::from([token])]);
    }
}
#[test]
fn the_connection_tokens_stay_on_statuses() {
    let (mut root, watch) = fixtures::root_with_watch(None);
    let connecting = fixtures::built();
    assert_status_token(&connecting, CONNECTING);
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
            },
        },
    );
    let view = fixtures::page_view(&root);
    for token in [OFFLINE, BEHIND] {
        assert_status_token(&view, token);
    }
}

fn assert_status_token(view: &crate::View, token: skein_ui::Style) {
    for node in view.tree().nodes() {
        if node.role == Role::Status && node.styles.as_slice().contains(&token) {
            return;
        }
    }
    panic!("status token {token:?} absent");
}

#[test]
fn the_conversation_tokens_stay_on_their_roles() {
    let message_tree = fixtures::chat::tree(&super::chat::structured_thread());
    let proposal = fixtures::page_view(&fixtures::chat::proposal_root());
    let composer = fixtures::page_view(&fixtures::chat::messaging_root());
    for token in [
        CHAT,
        CHATBAR,
        THREAD,
        MESSAGE,
        BY_PERSON,
        BY_AGENT,
        BY_TOOL,
        BY_SYSTEM,
        WHO,
        SAYS,
        WORK,
        MARKER,
        CARDS,
        COMPOSER_AREA,
    ] {
        let role = match token {
            CHAT => Role::Main,
            MARKER => Role::Region,
            CARDS => Role::List,
            _ => Role::Slot,
        };
        let mut found = false;
        for tree in [&message_tree, proposal.tree(), composer.tree()] {
            for node in tree.nodes() {
                if node.styles.as_slice().contains(&token) {
                    assert_eq!(node.role, role);
                    found = true;
                }
            }
        }
        assert!(found, "{token:?}");
    }
}
