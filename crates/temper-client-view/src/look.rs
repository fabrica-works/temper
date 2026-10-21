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
/// The accounts' banner: no usable account, its words and acts.
pub const ACCOUNTS: Style = Style { owner: owner(), token: 22 };
/// The composer's account and model line.
pub const ACCOUNT_LINE: Style = Style { owner: owner(), token: 23 };
/// The conversation page main.
pub const CHAT: Style = Style { owner: owner(), token: 24 };
/// The chat identity and controls.
pub const CHATBAR: Style = Style { owner: owner(), token: 25 };
/// The conversation and its tail.
pub const THREAD: Style = Style { owner: owner(), token: 26 };
/// One message in a conversation turn.
pub const MESSAGE: Style = Style { owner: owner(), token: 27 };
/// A message spoken by a person.
pub const BY_PERSON: Style = Style { owner: owner(), token: 28 };
/// A message spoken by the agent.
pub const BY_AGENT: Style = Style { owner: owner(), token: 29 };
/// A message spoken by a tool.
pub const BY_TOOL: Style = Style { owner: owner(), token: 30 };
/// A message containing instructions.
pub const BY_SYSTEM: Style = Style { owner: owner(), token: 31 };
/// The speaker and message state.
pub const WHO: Style = Style { owner: owner(), token: 32 };
/// The contents of a message.
pub const SAYS: Style = Style { owner: owner(), token: 33 };
/// A tool call or result line.
pub const WORK: Style = Style { owner: owner(), token: 34 };
/// A run or omission marker.
pub const MARKER: Style = Style { owner: owner(), token: 35 };
/// The chat proposal and departure list.
pub const CARDS: Style = Style { owner: owner(), token: 36 };
/// The chat composer and account area.
pub const COMPOSER_AREA: Style = Style { owner: owner(), token: 37 };
/// A list page main.
pub const PAGE: Style = Style { owner: owner(), token: 38 };
/// The page breadcrumb bar.
pub const PAGEBAR: Style = Style { owner: owner(), token: 39 };
/// A section heading row.
pub const HEADING: Style = Style { owner: owner(), token: 40 };
/// The heading row filters.
pub const FILTERS: Style = Style { owner: owner(), token: 41 };
/// The inbox updates list.
pub const UPDATES: Style = Style { owner: owner(), token: 42 };
/// A card kind and path.
pub const META: Style = Style { owner: owner(), token: 43 };
/// A card title.
pub const TITLE: Style = Style { owner: owner(), token: 44 };
/// A card supplied facts.
pub const DETAILS: Style = Style { owner: owner(), token: 45 };
/// An inbox proposal.
pub const KIND_PROPOSAL: Style = Style { owner: owner(), token: 46 };
/// An inbox question.
pub const KIND_QUESTION: Style = Style { owner: owner(), token: 47 };
/// An inbox held task.
pub const KIND_HELD: Style = Style { owner: owner(), token: 48 };
/// An inbox person task.
pub const KIND_PERSON_TASK: Style = Style { owner: owner(), token: 49 };
/// An inbox result.
pub const KIND_RESULT: Style = Style { owner: owner(), token: 50 };
/// An inbox reply.
pub const KIND_REPLY: Style = Style { owner: owner(), token: 51 };
/// The task page header region.
pub const TASK_HEADING: Style = Style { owner: owner(), token: 52 };
/// A task page card region.
pub const TASK_CARD: Style = Style { owner: owner(), token: 53 };
/// The plan tree.
pub const PLAN: Style = Style { owner: owner(), token: 54 };
/// One plan row's own parts.
pub const PLAN_ROW: Style = Style { owner: owner(), token: 55 };
/// The list around a held row card.
pub const PLAN_HOLD: Style = Style { owner: owner(), token: 56 };
/// The task activity list.
pub const EVENTS: Style = Style { owner: owner(), token: 57 };
/// A done task phase.
pub const STATUS_DONE: Style = Style { owner: owner(), token: 58 };
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
        b"temper-accounts",
        b"temper-account-line",
        b"temper-chat",
        b"temper-chatbar",
        b"temper-thread",
        b"temper-message",
        b"temper-by-person",
        b"temper-by-agent",
        b"temper-by-tool",
        b"temper-by-system",
        b"temper-who",
        b"temper-says",
        b"temper-work",
        b"temper-marker",
        b"temper-cards",
        b"temper-composer-area",
        b"temper-page",
        b"temper-pagebar",
        b"temper-heading",
        b"temper-filters",
        b"temper-updates",
        b"temper-meta",
        b"temper-title",
        b"temper-details",
        b"temper-kind-proposal",
        b"temper-kind-question",
        b"temper-kind-held",
        b"temper-kind-person-task",
        b"temper-kind-result",
        b"temper-kind-reply",
        b"temper-task-heading",
        b"temper-task-card",
        b"temper-plan",
        b"temper-plan-row",
        b"temper-plan-hold",
        b"temper-events",
        b"temper-status-done",
    ],
    regions: &[b"temper-link"],
};
