# Carriers of milestone 1's pages

Read: temper `docs/design/client/browser.md` (including the designer's amendments, a959f6e10) and `client/README.md`
(3.5, 5.1, 5.3, 5.4, 5.5, 6.6, 7), and the mockups `docs/design/client/mockups/{index,chats}.html`, all at temper
`origin/main` `116d0a645`; the code of temper at `116d0a645`, jig at `8cc7e79f` and skein at `8644b70` (each
`rev-parse --short origin/main`). The checkouts' own HEADs differ (jig `39c1e4db`, skein `9fbda6b`), so every file was
read from `git show origin/main:<path>`, never from a working tree. Each name was checked with `rg` in those copies; the
graph (projects `home-free-src-rust-{temper,jig,skein}`) was used for a few name and absence lookups (that no `parse`,
`print` or `Elsewhere` exists in `temper-client-domain`, for one), and the rest is `rg` and reading. No build was run.
The mockups are "reference material, not contract" (README 3.5), so a mockup-only feature is marked "none" and noted as
mockup-only.

Paths are under `crates/` of the named repository. Short forms: `tcd` = `temper-client-domain/src`, `tcv` =
`temper-client-view/src`, `tsv` = `temper-service/src`, `jcd` = `jig-client-domain/src`, `jc` = `jig-client/src`,
`jsw` = `jig-client-shell-web/src`, `jpl` = `jig-client-protocol-link/src`, `jpe` =
`jig-protocol-engine-clients-http/src`, `su` = `skein-ui/src`, `suw` = `skein-ui-web/src`. A "Code" cell with a step
(temper 02.1, skein 01.3 and so on) names a carrier the plan adds, in `docs/development/plans/browser-1/`.

`temper-client-protocol-wire` carries none of milestone 1's pages: it holds the forge-family records alone (`FORGE`,
`CLIENT_VERSION`, `generated/v1.rs`, in `temper-client-protocol-wire/src/lib.rs`). The wire for chats, the inbox and the
sign-in is jig's (`jig-client-protocol-wire/src/generated/v9.rs`, `TaskLine`, `Message` and the rest), reached through
`jig_client`.

## Sign-in: the printed link

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| The line `temper serve` prints: `http://` + loopback address + `/jig/v1/sign-in/link?secret=<base64url>` | browser.md 2.4; 4.1 story 1 | `printed::link(address, secret)` and `address_text` (IPv4 or `[::1]`); `jig_protocol_engine_clients_http::link_path(&Settings, &Secret)` | `tsv/clients/printed.rs` 3-40; `tsv/clients.rs` `print_link` 153, `http_settings` 639; jig `jpe/lib.rs` `link_path` 165 |
| A GET of that link answers with the page (the bundle's shell), not a 404 | browser.md 2.2, 2.4 | today: `tsv/clients.rs` 254-255 leaves the GET of `/jig/v1/sign-in/link` unclaimed and answers 404 with `Box::new([http::link_page()])`; jig `claims` is false for a GET of `/v1/sign-in/link` | temper 01.2 (replaces the 404 by the page; keeps jig's `link_page`, `referrer-policy: no-referrer`, `jpe/lib.rs` 156-164, 224-226) |
| The page's headers (policy on every answer, `frame-ancestors`, no referrer) | browser.md 2.2, 4.1 story 5 | jig's page policy (`jig_protocol_engine_clients_http`), not carried by temper today | temper 01.3 |
| The query `?secret=`: base64url that decodes to exactly 32 bytes, at the path `/jig/v1/sign-in/link` | browser.md 2.4 | `jig_client_shell_web::sign_in::Route { path, home }`, `Taken { secret, address }`, `take(&Route, path_and_query) -> Option<Taken>` | jig `jsw/sign_in.rs` 4-27; the shell's `Route { path: b"/jig/v1/sign-in/link", home: b"/" }` is temper 02.2 |
| The secret leaves the address bar: the history entry is replaced with `/` before anything else | browser.md 2.4; 4.1 story 1 ("the address reads `/`") | `jig_client_shell_web::web::take_from_page(&Route) -> Option<Box<[u8]>>` (wasm32); `Taken::address` | jig `jsw/web.rs` 10; called first by temper 02.2 (secret taken first) |
| Signing in with the secret | browser.md 2.4 | `jig_client::Event::SignIn { with }`, `jig_client_protocol_link::boundary::With::Link(Secret)` | jig `jc/client.rs` 14; `jpl/boundary.rs` 142-147; `jpl/sign_in.rs` 13; sent by temper 02.2 |
| Signing in by the cookie on a reload (no secret in the address) | browser.md 2.4; 4.1 story 1 | `With::Transport`; the browser sends the cookie jig's mount set; request header `Jig-Client: browser` | jig `jpl/boundary.rs` 144; `jsw/adapter.rs` 55; chosen by temper 02.2, proved by 04.2 |
| The link's state on the page: Connecting, Live, Offline, Signed out, Out of date | browser.md 2.4, 4.3 (states are `Role::Status`) | `jig_client::LinkView { state, party, not_served, sign_out_lost, credential }`; `jig_client_domain::Link::{SignedOut, Starting, Live, Offline, OutOfDate}` | jig `jc/client.rs` 60-65, `link()` 229; shown by `tcv/frame.rs` (see the frame table) |
| The same address on a second load: each load is a client of its own with its own stream | browser.md 2.1 (small store `temper.`) | `jig_client` records in the small store, prefix `temper.`; the store is the platform's | temper 02.3 (cookie and small store); skein `suw/store.rs` carries it |

## The signed-out page

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| Main paragraph "Signed out. To sign in, open a sign-in link temper serve printed." | browser.md 2.4; 4.1 story 1 (a spent link) | `Word::SignedOutMain` (today the terminal's words: "Signed out.", `tcv/words.rs` 474); `Words::english(connector, text)` at 291 takes no shell | temper 03.1 (`words::Shell { Terminal, Browser }` as a parameter of `Words::english`) |
| The out-of-date paragraph (the other page `unavailable` can pick) | browser.md 2.4 | `Word::OutOfDateMain` (`tcv/words.rs` 475); picked by `Link::OutOfDate` in `unavailable` | `tcv/frame.rs` 69-75; `tcv/root.rs` 65-74 (exists; not changed by a step) |
| One paragraph in `Main`, no heading | browser.md 2.4 | `View::build`: Main > Paragraph | `tcv/root.rs` 65-74 |
| A banner with only "temper" and the status "Signed out", no navigation, no buttons | browser.md 2.4 | `Word::Temper` (450), `Word::SignedOut` (472) and `Word::OutOfDate` (473) in a `Role::Status`; no navigation when the link is not live | `tcv/frame.rs` 76-88, 145- ; `tcv/words.rs` 450, 472-473 |
| What triggers it: the link is `Link::SignedOut` or `Link::OutOfDate` | browser.md 2.4 | `pages::Event::Link { link }`; `boundary::Event::SignedOut`; `Request::Signing { signing: Signing::SignedOut }` from the client | `tcd/pages.rs` (event handler at `Event::Link`, ~230-237 in the copy); `tcd/boundary.rs` 12-19; jig `jc/client.rs` |
| A spent or absent secret leaves the shell signed out | browser.md 2.4; 4.1 story 1 | `jig_client::Event::SignIn { with: With::Link(secret) }` refused by jig's mount; the link goes to `Link::SignedOut` | jig `jpl/sign_in.rs`; the story is 04.2 |
| The page title "temper" | decisions.md 7 (browser.md names none); README 3.5 (a page's title is its level-1 heading) | none in the view (a page title is the shell's) | temper 02.4 |
| "The application failed." (the restart's limit passed) | browser.md 2.1; 4.1 story 4 | `skein_ui_web::restart::Restart::new(restarts, window, name, words)`; the words are temper's | skein `suw/restart.rs` 13; temper 02.2 (`Restart::new(3, 60 s, "temper-restarts", "The application failed.")`) |

## The frame (banner and navigation) on a signed-in page

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| Banner paragraph "temper" | README 5.1; browser.md 2.4 | `Word::Temper` | `tcv/frame.rs` `banner` 76-144; `tcv/words.rs` 450 |
| One button per project, selected one marked | README 5.1 | `jig_client_domain::contract::Me::projects` (`MeProject::name: Option<DocumentText>`, fallback "Project N" from `MeProject::project: u32`); `State::Selected`; `Act::Project { project }` | jig `jcd/contract/me.rs` 10, 33; `tcv/frame.rs`; `tcd/pages.rs` 217 |
| The link status: "Connecting", "Live", "Offline" | README 5.1; browser.md 4.3 | `LinkView::state` (`Link::Live` with `me` present is "Live", else "Connecting"; `Link::Offline` is "Offline") | `tcv/frame.rs` 79-130; jig `jc/client.rs` 60-65 |
| "Behind" status when the person's record or the page's read is stale | README 5.1 | `jig_client::State::Behind` on `Document::state`, for `pages::me_handle()` and the inbox | `tcv/frame.rs` 119-131; `tcv/words.rs` 96 |
| The party's name | README 5.1 | `Me::party.name` | `tcv/frame.rs` 135; jig `jcd/contract/me.rs` |
| The alert "Sign out did not reach the engine." | README 5.1 | `LinkView::sign_out_lost` | `tcv/frame.rs` 137-139; jig `jc/client.rs` 60-65 |
| Act: the "Sign out" button | README 5.1; browser.md 2.4 | `Act::SignOut`, which pushes `jig_client::Event::SignOut`; shown when `me.is_some() \|\| sign_out_lost` | `tcd/pages.rs` 217-219; `tcv/frame.rs` 140-141; jig `jc/client.rs` 15 |
| Navigation links: "Inbox N" (N = `me.needs_total`), "Chats", and "Adopt a repository" when a project exists | README 5.3 | `Role::Link` bound to `Act::Go { address }`; `Me::needs_total` | `tcv/frame.rs` 145-181; `tcd/pages.rs` 217-219 |
| Each link has an `href` (open in a new tab, copy the address) | README 5.3; browser.md 2.3 | `Page::links` carrying an `Address`; the shell prints it with `address::print` | temper 02.4 (links), temper 02.1 (`print`) |
| Navigation entries Board, Changes and Accounts | README 5.3 | none: the view builds no such links and the domain has no such `Address` | none (milestone 2 and later pages; see the closing list) |
| The accounts banner (the person's accounts and the model shown) | README 5.1, 5.5 | none: no `account` word in `tcv/words.rs`, `tcv/frame.rs`, `tcv/pages/chat.rs` or `tcv/pages/chats.rs` (`rg -c` finds none) | none |
| Look: tokens `temper-*`, `temper.css` after `skein-ui.css`, light only | browser.md 2.5 | the six `look` tokens in the view, plus the stylesheet | temper 03.3; `owner()` (`tcv/frame.rs` 6) made `const fn` there |

## Addresses

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| `/` is the inbox | browser.md 2.3 | `Address::Inbox { filter: Filter }`; `Filter::ALL` | `tcd/address.rs` 1-30; the codec is temper 02.1 |
| `/chats` is the list of chats | browser.md 2.3 | `Address::Chats` | `tcd/address.rs`; codec temper 02.1 |
| `/chats/<task>` is one chat, `<task>` a canonical decimal `u64` (no leading zero, at most 20 digits) | browser.md 2.3 | `Address::Chat { task: u64 }` | `tcd/address.rs`; codec temper 02.1 (model: jig `jig-client-view/src/address.rs` `decimal` 285-297) |
| `/tasks/<task>` is milestone 2's task page | browser.md 2.3 | `Address::Task { task: u64 }` | `tcd/address.rs`; codec temper 02.1 |
| `/projects/<project>/adopt` is milestone 2's adoption form, `<project>` a canonical decimal that fits a `u32` | browser.md 2.3 | `Address::Adopt { project: u32 }` | `tcd/address.rs`; codec temper 02.1 (jig `project_decimal` 311) |
| The inbox's filter (`project`, `kind`, `goal`) is not in the address; the inbox prints `/` whatever its filter | browser.md 2.3, 5 | `Filter { project, kind, goal }`, `Kind::{Proposal, Question, Held, PersonTask}` | `tcd/address.rs`; `print` drops it, temper 02.1 |
| Parse: bytes to `Place::Address(Address)` or `Place::Missing`; cut at `?`; one trailing `/` stripped; a path not starting with `/` or still ending in `/` is `Missing`; no percent-decoding; no prefix, no `Elsewhere` | browser.md 2.3 | `Place::{Address, Missing}`; `pub fn parse(path: &[u8]) -> Place` | `Place` in `tcd/address.rs`; `address::parse` is temper 02.1, copied from jig `jig-client-view/src/address.rs` `parse` 219-276 |
| Print: `Address` to the path bytes, fixed paths | browser.md 2.3 | `pub fn print(address: &Address) -> Box<[u8]>` | temper 02.1 (jig `print` 152-215) |
| Never printed: `/jig`, `/bundle/`, `/test/`, `/robots.txt` (they are not pages; parse gives `Missing`) | browser.md 2.3 | none beyond the codec's fixed list | temper 02.1 (`parse`), temper 01.1 (the bundle key) |
| A non-page address opens the inbox and the address bar is replaced with `/` | browser.md 2.3; 4.1 story 2 | `Place::Missing` opens no page in the domain (`went`); the shell turns it into `Inbox { Filter::ALL }` and a `replace` | `tcd/route.rs` `Start` 11-18, `went` 29-43; `Request::Show { address, replace }` `tcd/boundary.rs` 27-33; the domain always emits `replace: false` (`route.rs` 50, 66); temper 02.2 and 02.4 |
| Back and forward between chats and a chat step `Went { place }` | browser.md 2.3; 4.1 story 2 | `Event::Went { place }`; history entries pushed by `Show` | `tcd/boundary.rs` 12-19; temper 02.4 (history); skein 01.2 (`back_and_forward` awaits a loaded page) |
| A page opens at its address on start (reload) | browser.md 2.3 | `Event::Start { address, kept }` | `tcd/boundary.rs` 12-19; `tcd/route.rs` 11-18; temper 02.2 |
| The fake person reads and goes to an address | browser.md 4.1 | `Step::Address`, `Step::Person` of skein's browser fake | skein 01.1 |
| The page title is "temper" on every page | decisions.md 7 (browser.md names none) | none in the view | temper 02.4 |

## The inbox at `/` (the landing page; its story only opens it and reads the heading)

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| Heading 1 "Inbox" | browser.md 4.2; README 3.5 | `Word::InboxTitle` | `tcv/pages/inbox.rs` `main` 26-60 |
| Filters: "All projects", one button per project, the kinds, "All goals", and the "Goal" text field | README 3.5 | `Filter { project, kind, goal }` set by `Act::Filter`; `Me::projects` | `tcv/pages/inbox.rs` `filters` 62-117; `tcd/pages/inbox.rs` `Inbox { filter, cards }` 27-30; `Bindings::decode` `Filter` + `Input` (`tcv/frame.rs` 31-52) |
| Headings "Needs you" and "Updates" and a card per entry | README 3.5 | `Inbox::cards: Vec<Card>`; `Card { object, group, project, page, key, outcome }` | `tcd/pages/inbox.rs` 14-23; `tcv/pages/inbox.rs` `entry` 118- |
| A link from an entry to the task | README 3.5 | `Act::Go { address: Address::Task { task } }` | `tcv/pages/inbox.rs` `entry` |
| Act: choosing a filter shows the inbox again at `/` (no history entry added) | browser.md 2.3 | `Act::Filter { filter }` -> `Request::Show { address: Address::Inbox { filter } }`, which prints `/` | `tcd/pages.rs` 194; temper 02.4 (a filter's `Show` of the inbox adds no entry, `02-shell.md` 275) |
| Act: "Mark seen" when results exist | README 3.5 | `Inbox::results_through()` (211); `Ask::Seen` | `tcd/pages/inbox.rs` 211; `tcv/pages/inbox.rs` |
| The inbox is watched | README 5.4 | `Subject::Inbox { window }`; the proposal read `Read::Proposal` | `tcd/pages/inbox.rs` 57, 153 |
| Bounds: `rows: 2` and `text: 128` per the plan | README 5.4; plan 02-shell.md | `jig_client::DomainLimits::{rows, text}` | see the bounds table |

## Chats (`/chats`)

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| Heading 1 "Chats" | README 3.5; mockup chats.html | `Word` for the title in `chats::main` | `tcv/pages/chats.rs` |
| Act: the "Start a chat" text area, shown when `me` and a project exist | README 3.5 | `Chats::words: Box<[u8]>`; bound `Act::ChatWords { words }` | `tcv/pages/chats.rs`; `tcd/pages/chats.rs` `write` 94-98; `Bindings::decode` `ChatWords` + `Input` |
| Act: Enter in that text area starts the chat; Shift+Enter makes a newline | browser.md 2.5, 4.1 story 3 | `State::SubmitsOnEnter` on the text area; `Act::Submit`; `Event::Enter { node, shift, composing }`; `Bindings::decode` maps `Submit` to `Act::StartChat` | skein `su/tree.rs` 146 (checked 444), `su/binding.rs` 21, 38; skein `suw/page.rs` 537-551, `suw/mapping.rs` 310 (`enterkeyhint=send`); temper 03.2 (the state and the `decode` arm); skein 01.3 (the fake person's Shift) |
| Act: the "Start" button | README 3.5 | `Act::StartChat` -> `Chats::start` -> `jig_client::shapes::chat(me, project, Text)`; double-start guard `Chats::start: Option<Key>`; empty words ignored | `tcv/pages/chats.rs`; `tcd/pages/chats.rs` 99-127; jig `jc/shapes.rs` 71 |
| After "Start", the page of the new chat opens at `/chats/<task>` | browser.md 4.1 story 3 | `Chats::told` shows `Address::Chat { task: first }` on `OwnerOutcome::Created { first, .. }` | `tcd/pages/chats.rs` 128-162 |
| A refusal when starting is not offered | README 3.5 | `Chats::refusal: Option<shapes::NotOffered>` | `tcd/pages/chats.rs`; `tcv/pages/chats.rs` |
| The outcome of a start | README 3.5 | `Chats::outcome: Option<jig_client::Outcome>` shown by `outcome_text` | `tcd/pages/chats.rs`; jig `jcd/boundary.rs` `Outcome` 149 |
| Heading 2 "Live chats" and a row per chat | README 3.5 | `Chats::live: Handle`; `TaskPage`; `TaskLine` (query `TaskQuery { project, requester: OwnerRequester::Person(me.party.number), label, order: QueryOrder::Activity }`) | `tcd/pages/chats.rs` `configure` 41-86; jig `jcd/contract/queries.rs` 72 |
| Heading 2 "Closed" and its rows | README 3.5 | `Chats::closed: Handle`; `Read::Tasks { phase: Ended, order: Ended }`; `EndedGroup::Rows { rows, after }` | `tcd/pages/chats.rs`; `tcv/pages/chats.rs` |
| Act: "More closed chats" when there are more | README 3.5 | `EndedGroup::Rows::after`; `Act::ClosedChats`; `Chats::older` | `tcd/pages/chats.rs` 87; `tcv/pages/chats.rs` |
| A row: the link "Chat N" to `/chats/N` | README 3.5; browser.md 2.3 | `Act::Go { address: Address::Chat { task: chip.task } }`, in an Item keyed `Key { owner: frame::owner(), kind: 10, a: chip.task, b: 0 }` | `tcv/pages/chats.rs` 63-69; `tcv/frame.rs` 6 |
| A row: the title | README 3.5; mockup chats.html | `TaskLine::chip` -> `Chip::title: DocumentText` | jig `jcd/contract/documents.rs` 34, `document_text.rs` 6 |
| A row: the status ("Waiting for you", "Parked", ...) | README 3.5 | `jig_client::facts::chat_status(chip, waits_on, project, me)` and `jig_client::words::Word::status`; the view overrides two words | jig `jc/facts.rs` 28; `tcv/words.rs` 502-504 |
| A row: "Spent N; delegates spent M" | README 3.5 | `TaskLine::budget: OwnerBudget` -> `OwnerNumbers::{spent, spent_below}` | jig `jcd/contract/owner_budget.rs` 9-18 |
| A row: "Live delegates N" | README 3.5 | `ChipPhase::Owner { details }` -> `OwnerChipDetails::live_delegates: u32` | jig `jcd/contract/owner_chip.rs` 28; `jcd/contract/documents.rs` 47 |
| A row: the project name and the task number tag ("temper · T0") | mockup chats.html only | none shown: `TaskLine::project: u32` and `MeProject::name` exist, the row does not use them | none |
| A row: a snippet of the last turn | mockup chats.html only | none: no field of `TaskLine` carries it | none |
| A row: the age ("4 min ago") | mockup chats.html only | `TaskLine::last_activity: Wall` exists and is not shown | none |
| The Live / Closed / All-projects filter row | mockup chats.html only | none: the view uses two headings, and `Filter` belongs to the inbox | none |
| The composer hint text, the placeholder "Ask temper anything...", the send arrow | mockup chats.html only | none: `Role::TextArea` has a name and no placeholder or hint in `skein-ui` (`rg` finds no `placeholder` in `su/builder.rs`, `su/tree.rs`, `su/binding.rs`) | none |
| The composer's over-bound feedback | README 5.4 (only the adoption form gets "At most N bytes.") | `Chats::write` ignores words over `limits.client.domain.text`; nothing tells the person | none (README 5.4 asks for none here) |

## A chat (`/chats/<task>`)

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| Heading 1 "Chat N" and a paragraph with the title | README 3.5; browser.md 4.1 (the story reads the title) | `Chat::task: u64`; `Chat::document` (`Task`); `Chip::title` | `tcv/pages/chat.rs` `main` 29-65; `tcd/pages/chat.rs` |
| The header: status | README 3.5 | `chats::status` over `jig_client::facts::chat_status` | `tcv/pages/chat.rs` `header`; jig `jc/facts.rs` 28 |
| The header: "Spent N; delegates spent M" | README 3.5 | `task.budget.numbers: OwnerNumbers::{spent, spent_below}` | `tcv/pages/chat.rs`; jig `jcd/contract/owner_budget.rs` 9 |
| The header: "Live delegates N" (the Chats row has it) | README 3.5 | `OwnerChipDetails::live_delegates` exists; the chat's header does not show it | none (the view would read `live_delegates` as `tcv/pages/chats.rs` does) |
| The header: "Agent" label and "spent of budget" | mockup chats.html only | `OwnerNumbers::budget` exists and is not shown; no agent label | none |
| The conversation: a log of lines | README 3.5 | `Chat::thread: jig_client::thread::Thread`; `Line::{Words, Turn, Provisional, Undecoded, RunBegan, RunEnded, Left, Activity}`; `Role::Log` named `P::ConversationLog`, each line a `Region` | `tcv/pages/chat.rs` `conversation`; jig `jc/thread/model.rs` 15-24, `jc/thread.rs` 74 |
| A person's words, with their state (being sent, sent, read) | README 3.5 | `Line::Words { by, words, state: BeingSent \| Sent \| Read }` | jig `jc/thread/model.rs`; `jc/thread/words.rs` |
| A committed turn: speaker label (Person, Agent, Tool result, Instructions) and blocks `Text` (markdown), `Call` (tool name and outcome), `Result`, `Opaque` | README 3.5 | `Turn`, `Message`, `Block`, `Call`, `Speaker` | jig `jcd/contract/turn.rs`; `tcv/pages/chat.rs`; `tcv/limits.rs` (`markdown_depth: 3`) |
| A turn being written: "Writing..." and then markdown cut at the bound | README 3.5, browser.md 2.5 (live text patch) | `Line::Provisional`; `TurnCut` | jig `jc/thread/model.rs`; `tcv/pages/chat.rs` `P::TurnCut` |
| Proposals and questions as cards after the conversation (Accept, Reject with a reason, Answer with words, Release, Leave held, Take, Hand back) | README 3.5 | `Chat::cards`; `confirm::card`; `Act::Decide`, `Act::CardWords`, `Act::Confirm`, `Act::Cancel`, `Act::Again`; `Task::proposals`, `Task::acts: List<Offered>` | `tcv/pages/chat.rs`; `tcv/confirm.rs`; jig `jcd/contract/documents.rs` 101 |
| A proposal placed where the call that made it stands | mockup chats.html only | `Call::objects` exists; the view puts every card after the conversation | none |
| A proposal's "waiting N min" and "Open in inbox" | mockup chats.html only | no field or link in the card view | none |
| "Accept goal" in place of "Accept" | mockup chats.html only (a wording) | none | none |
| Per-turn time and a day divider | mockup chats.html only | `ActivityLine::at: Wall` exists and `thread::Line` drops it | none |
| A work disclosure with tool-call counts | mockup chats.html only | `Call` exists per block; no count or disclosure | none |
| Act: the "Message" text area, shown when `allowed(Message)` | README 3.5; browser.md 2.5 | `Chat::words`; `Act::MessageWords { words }`; `allowed` over `jig_client::facts::offer(offers, ActKind)` returning `Admission::Allowed \| Bounded` | `tcv/pages/chat.rs` `composer` 96-115; `tcd/pages/chat.rs` 443-448; jig `jc/facts.rs` 128 |
| Act: Enter sends the message, Shift+Enter a newline | browser.md 2.5 | `State::SubmitsOnEnter`; `Act::Submit`; `Bindings::decode` maps `Submit` to `Act::Send` | temper 03.2; skein `su/tree.rs` 146, `suw/page.rs` 537-551 |
| Act: the "Send" button | README 3.5 | `Act::Send` -> `Ask::Message { project, task, words: Text::new(.., domain.text) }`; one pending request per kind | `tcd/pages/chat.rs` `send` 364-404 |
| Act: "Stop" when allowed | README 3.5 | `Act::Stop` -> `Ask::Stop`; `allowed(Stop)` | `tcv/pages/chat.rs` 96-115; `tcd/pages/chat.rs` |
| Act: "Close" when allowed, then the close dialog | README 3.5 | `Act::Close`; `shapes::close_chat(me, project, chat, delegates, reason)`; `Chat::closing`; `confirm::main` | `tcd/pages/chat.rs` `prepare_close` 241-256, `close` 286; jig `jc/shapes.rs` 77 |
| Act: "Mark seen" | README 3.5 | `Act::Seen` -> `Ask::Seen { through: SeenThrough::Conversation { task, through: thread.loaded } }` | `tcd/pages/chat.rs` `send` |
| Act: "Older" when more lines exist | README 3.5 | `thread.lines.edges.after` is `window::Edge::More`; `Act::Older` | `tcv/pages/chat.rs`; `tcd/pages/chat.rs` |
| The pending progress and the outcome of an act | README 3.5 | `Chat::pending`, `Chat::outcome: Option<jig_client::Outcome>` | `tcv/pages/chat.rs` |
| Left cards ("departures") | README 3.5 | `Chat::departures`; `P::CardLeft` | `tcv/pages/chat.rs`; `tcd/pages/chat.rs` |
| The composer's account and model line, and "needs an account" | README 3.5, 5.5 | none: no `account` in the chat's view, domain or words | none |
| "Starting: waiting for an account" | README 5.5 | none | none |
| The thread's bounds: lines `domain.rows`, bytes `domain.document_bytes`, text `domain.text` | README 5.4 | `jig_client::thread::Limits { lines, bytes, text }`, set by `thread_limits` | jig `jc/thread.rs` 31-34; `tcd/pages/chat.rs` 407-413 |

## Bounds (shared by the pages above)

| What the page shows or takes | Design | Carrier (type, field) | Code |
| --- | --- | --- | --- |
| The composers' length (Start a chat, Message): at most `text` bytes; longer input is not taken | README 5.4 | `jig_client_domain::Limits::text` (re-exported `jig_client::DomainLimits`); `Chats::write` keeps the old text when the new is longer; `Text::new(bytes, limit)` checks length and UTF-8 | jig `jcd/limits.rs` 10-31, `jcd/contract/common.rs` 14, `jc/lib.rs` 13; `tcd/pages/chats.rs` 94-98; `tcd/pages/chat.rs` 364-404 |
| The lists' length (live, closed, lines, cards): `rows` | README 5.4 | `Limits::rows` | jig `jcd/limits.rs`; `tcd/pages/chats.rs` 41-86 |
| The browser's values for those limits: `text` 128, `rows` 2, `projects` 2, `handles` 17, `watches` 16, `reads` 16, `delegates` 8, `segments` 9, `authority` 12, `fields` 16, `parts` 16, `kept` 16, `records` 16, `document_bytes` and `turn_bytes` 1_048_576; `forge::Limits { field_bytes: 256, checks: 8 }`; `EXCHANGES` 8; `BODY_BYTES` 32 MiB; stream piece 4096 | README 5.4; browser.md 2.1 ("derived as the terminal's are") | `temper_client_domain::Limits { forge, client: jig_client::Limits { domain, exchanges } }` | `tcd/limits.rs`; jig `jc/limits.rs` 5; the terminal's `native()` in `temper-client-shell-terminal/src/limits.rs` 24-65; copied for the browser by temper 02.2 and 02.3 (`limits.rs`) |
| Every page's worst case fits the view's node and style counts | browser.md 2.1 | `Limits::of` (`text_per_node`, `styles_per_node: 2`, `markdown_depth: 3`); `View::new` asserts `worst_case` | `tcv/limits.rs` 9-51; `tcv/root.rs` 22-34; `tcd/limits.rs` `worst_case` |
| The skein tree's own limits (nodes, text, styles) | browser.md 2.1 | `skein_ui::Limits` | skein `su/limits.rs` 13-28 |
| Words longer than the bound do not grow the composer past it; the composer grows by CSS (`field-sizing: content`) | browser.md 2.5 | none in the view (a stylesheet rule) | temper 03.3 |
| Over-bound words are dropped without a word to the person | README 5.4 | none | none |

## The rows marked "none"

By page, with the design's section and what each would take:

- **Frame:** the Board, Changes and Accounts entries (README 5.3) need an `Address` (or none, as links to pages not
  yet built) and a view entry; the accounts banner (README 5.1, 5.5) needs jig's accounts record on `Me` and a view
  node.
- **Chats row:** project name and number tag, snippet, age, the filter row, composer hint/placeholder/send arrow are
  all from the mockup only. Name and age have upstream fields (`MeProject::name`, `TaskLine::last_activity`) and
  need only view code and words; a snippet has no field upstream (jig would have to carry the last turn's first
  line in `TaskLine`); a placeholder needs a `skein-ui` node attribute.
- **Chat:** the account and model line, "needs an account" and "Starting: waiting for an account" (README 3.5, 5.5)
  need an accounts record in `jig_client` and a view row; the day divider and per-turn time need `thread::Line` to
  keep `ActivityLine::at`; the work disclosure with counts needs a view count over `Block::Call`; "waiting N min",
  "Open in inbox" and the proposals' place at their call need view code (`Call::objects` exists); "Agent" and
  "spent of budget" need view code over `OwnerNumbers::budget`; "Live delegates" on the chat needs view code over
  `OwnerChipDetails::live_delegates`.
- **Bounds:** the composers' 128-byte and the lists' 2-row limits are the terminal's `native().client` copied, so a
  long message is ignored silently; README 5.4 names no feedback for it.

## Design that no milestone-1 step covers

- Everything on the "none" rows above (all are mockup details or README 5.1/5.5/3.5 items beyond the stories).
- The Board, Changes and Accounts navigation (README 5.3), and the accounts banner (README 5.1, 5.5).
- Over-bound feedback for the two composers.
- The document title per page: the plan sets "temper" for every page (decision 7, 02.4); README 3.5 says a page's title is its level-1 heading, which the view has, and browser.md names no document title.
- The browser's out-of-date words: 03.1 changes only `SignedOutMain`; `OutOfDateMain` stays "This temper is out of
  date: run the temper the engine runs." (`tcv/words.rs` 475), a sentence written for the terminal.

Covered by a step: the printed link's page (temper 01.2), the page policy (01.3), the codec and the fixed paths (02.1),
the shell's start order, `Missing` -> inbox with `replace` and the failure words (02.2), the bundle, cookie, small store
and limits (02.3), history, links and the title (02.4), the signed-out words (03.1), Enter and Shift+Enter (03.2), the
look and CSS growth (03.3), the stories (04.1 to 04.5), and the fake person's address, `back_and_forward`, Shift and
button-click (skein 01.1 to 01.4).
