use super::fixtures;
use crate::look::{BANNER, COMPOSER, MAIN, NAMES, NAVIGATION};
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
    let named: [(skein_ui::Style, &[u8]); 6] = [
        (BANNER, b"temper-banner"),
        (NAVIGATION, b"temper-navigation"),
        (MAIN, b"temper-main"),
        (crate::look::CARD, b"temper-card"),
        (crate::look::TURN, b"temper-turn"),
        (COMPOSER, b"temper-composer"),
    ];
    for (style, name) in named {
        assert_eq!(NAMES.style(style), Some(name));
    }
    assert_eq!(NAMES.region(crate::frame::link_region()), Some(&b"temper-link"[..]));
}
#[test]
fn each_pages_main_carries_the_main_token() {
    let builds: [fn() -> temper_client_domain::Domain; 5] = [
        fixtures::signed_out,
        fixtures::adoption_root,
        fixtures::task::held_root,
        fixtures::chats_root,
        fixtures::chat::chat_root,
    ];
    for build in builds {
        let view = fixtures::page_view(&build());
        assert_eq!(fixtures::styles(view.tree(), Role::Main).as_ref(), [Box::from([MAIN])]);
    }
}
#[test]
fn the_composers_carry_the_composer_token() {
    for root in [fixtures::chats_root(), fixtures::chat::messaging_root()] {
        let view = fixtures::page_view(&root);
        assert_eq!(fixtures::styles(view.tree(), Role::TextArea).as_ref(), [Box::from([COMPOSER])]);
    }
}
