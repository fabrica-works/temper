//! temper's style tokens and their names (client/browser.md, 2.5), as jig's `jig-client-shell-web` `look.rs`.
use crate::frame::owner;
use skein_ui::{Style, look::Names};
/// The frame's banner.
pub const BANNER: Style = Style { owner: owner(), token: 0 };
/// The frame's navigation.
pub const NAVIGATION: Style = Style { owner: owner(), token: 1 };
/// Every page's main.
pub const MAIN: Style = Style { owner: owner(), token: 2 };
/// The chats' rows, and a chat's cards and departures.
pub const CARD: Style = Style { owner: owner(), token: 3 };
/// Each line of a chat's conversation.
pub const TURN: Style = Style { owner: owner(), token: 4 };
/// The chats' and a chat's composers.
pub const COMPOSER: Style = Style { owner: owner(), token: 5 };
/// The look's names (skein's ui.md, 8): styles by token, regions by number.
pub const NAMES: Names<'static> = Names {
    owner: owner(),
    prefix: b"temper",
    styles: &[
        b"temper-banner",
        b"temper-navigation",
        b"temper-main",
        b"temper-card",
        b"temper-turn",
        b"temper-composer",
    ],
    regions: &[b"temper-link"],
};
