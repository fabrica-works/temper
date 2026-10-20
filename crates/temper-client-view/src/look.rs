//! temper's style tokens and their names (client/browser.md, 2.5), as jig's `jig-client-shell-web` `look.rs`.
use crate::frame::owner;
use skein_ui::{Style, look::Names};
/// The frame's banner.
pub const BANNER: Style = Style { owner: owner(), token: 0 };
/// The frame's navigation.
pub const NAVIGATION: Style = Style { owner: owner(), token: 1 };
/// Every page's main.
pub const MAIN: Style = Style { owner: owner(), token: 2 };
/// A chat's cards and departures.
pub const CARD: Style = Style { owner: owner(), token: 3 };
/// Each line of a chat's conversation.
pub const TURN: Style = Style { owner: owner(), token: 4 };
/// The chats' and a chat's composers.
pub const COMPOSER: Style = Style { owner: owner(), token: 5 };
/// The banner's wordmark.
pub const BRAND: Style = Style { owner: owner(), token: 6 };
/// The live link status.
pub const LIVE: Style = Style { owner: owner(), token: 7 };
/// The connecting link status.
pub const CONNECTING: Style = Style { owner: owner(), token: 8 };
/// The offline link status.
pub const OFFLINE: Style = Style { owner: owner(), token: 9 };
/// The link region's behind status.
pub const BEHIND: Style = Style { owner: owner(), token: 10 };
/// A chats section's list.
pub const ROWS: Style = Style { owner: owner(), token: 11 };
/// A chats row.
pub const ROW: Style = Style { owner: owner(), token: 12 };
/// The chats' composer box.
pub const ENTRY: Style = Style { owner: owner(), token: 13 };
/// A chat waiting for the reader.
pub const STATUS_YOURS: Style = Style { owner: owner(), token: 14 };
/// A held chat.
pub const STATUS_HELD: Style = Style { owner: owner(), token: 15 };
/// A retrying chat.
pub const STATUS_RETRYING: Style = Style { owner: owner(), token: 16 };
/// A chat starting or doing work.
pub const STATUS_ACTIVE: Style = Style { owner: owner(), token: 17 };
/// A chat waiting, parked, closing or ended.
pub const STATUS_QUIET: Style = Style { owner: owner(), token: 18 };
/// A chats row's absolute UTC time.
pub const TIME: Style = Style { owner: owner(), token: 19 };
/// The start composer's keyboard hint.
pub const HINT: Style = Style { owner: owner(), token: 20 };
/// The banner's corner: link state, person and sign out.
pub const CORNER: Style = Style { owner: owner(), token: 21 };
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
        b"temper-brand",
        b"temper-live",
        b"temper-connecting",
        b"temper-offline",
        b"temper-behind",
        b"temper-rows",
        b"temper-row",
        b"temper-entry",
        b"temper-status-yours",
        b"temper-status-held",
        b"temper-status-retrying",
        b"temper-status-active",
        b"temper-status-quiet",
        b"temper-time",
        b"temper-hint",
        b"temper-corner",
    ],
    regions: &[b"temper-link"],
};
