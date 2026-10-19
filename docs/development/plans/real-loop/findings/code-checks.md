# Code checks: the session files against temper at e139ccaf4

Checked in `tmp/planning/real-loop` (temper `origin/main`) and the pinned kits (skein 92e177c3, jig
eceadcaf, smith 7b4dfc07). `T` = `crates/temper-client-tools/src`; `JC`, `SK` = the jig and skein
`crates/` of `tmp/planning/view`. Every name below was found with `rg`, not taken from the graph.

## 1. Decisions M to Q against the sessions

| Decision | Code | Where the sessions touch it | Verdict |
|---|---|---|---|
| M, act with no tool | `T/jig/render/acts.rs` `text`, `has_tool` (ten acts): `NAME (no tool)`, no call | only 06.4's `printed_act` reads an `acts:` line | no mismatch |
| N, inbox cursor | `T/jig/render/lists.rs` `inboxes`, `inbox_cursor`, `group_cursor`; `T/jig/argv.rs:523` `check_inbox`; `T/jig/runner/reads.rs:152` | no session reads an inbox on (06.1's `inbox()` is jig's fixture) | nothing to check |
| O, inbox first page | `lists.rs` `updates_fit`; `render/document/read_sets.rs:84`; goldens `inbox_*_next.txt` | as N | nothing to check |
| P, limited activity | `lists.rs` `activity_limit`; `T/jig/render/activity.rs:9-41` (`read with: activity N` after `task`) | none passes `--limit` or `--activity K` | nothing to check |
| Q, restored proposal | `T/jig/render/outcome.rs:213` `proposed` prints `, to ` only if `operation.to`; test `tests/jig/acts.rs:234` | 05.4 and 06.5 restore a chat and a stop only | nothing to check |
| `kept:` line | `outcome.rs:142` `text`: `kept: WORD, OUTCOME`; `T/jig/runner/recovery.rs` `kept` | 05.4 `kept: chat, `; 06.5 lines 289, 313 | match, but 09.2 (1) |
| parked, dropped | `recovery.rs` `text`: `kept: N for party P, unsent while it is not signed in here`; `dropped: N kept records these tools cannot read` | 09.4 line 303 | match |
| pending row | to build in 05.5; `Operation::of` words `chat`, `stop` | 05.5 `chat, party 73, not sent`, `stop, party 73, not sent` | match |
| tasks page | `lists.rs` `tasks`; golden `tasks_next.txt` | 09.2 `listed` (rows `N · ...`) | match |
| acts as calls | `acts.rs:169` `call` ends every call with ` --key NAME` | 06.4, 08, 09, 13.4 | 06.4 (2, 3) |

### Mismatches

| # | Session, increment | The plan says | The code does |
|---|---|---|---|
| 1 | 09, 09.2 (line 195) | the cut call's stderr begins `kept: create done: task ` | `Operation::of` (`T/jig/render/outcome.rs:16`) names a chat `chat`: `kept: chat, done: task N, chat` |
| 2 | 06, 06.4 (line 259) | the agent gives `release D --hold R` | the printed act is `release D --hold R --key NAME` (`acts.rs:169`); wording gap, the call carries a key |
| 3 | 06, 06.4 (lines 262, 297) | `adopt acme/shop --key shop` | `adopt::ROW` (`T/adopt.rs:7`) has no positional; it needs `--project --owner --name --landing`, and `--key` (Keyed::Required): usage, exit 1. 13.4 has the right form |
| 4 | 06, 06.3 (line 220), outside M to Q | `read_whole` follows each `read on with:` line | a cut part ends its line `N bytes left; read on with: turn T NAME --part P --from F` (`T/jig/render/turn.rs:109`), and the size line says `read with: turn T NAME` (`turn.rs:20`): match mid-line, both phrases |

### The two gaps

| Gap | Answer |
|---|---|
| (a) adopt's `kept:` line | Two forms. Answered, unknown, decode failure: `kept: adopt ` + the forge's words, no comma (`T/forge.rs:327` `answer_text`, `:542` `decode_failure`; `tests/adopt.rs:299`, golden `adopt_refused.txt`). Held, Limit, Unconfirmed, NotKept: `outcome::kept` (`forge.rs:244-257`), so `kept: adopt, REASON`. Every jig act is `kept: OP, OUTCOME`. tools.md 6.1 (line 504) gives no punctuation. No session asserts an adopt kept line (06.4, 13.4 assert exit and stdout), so none is wrong; one that does must pin the no-comma form. The mixed forms are a tools finding. |
| (b) pending's word for a chat | `chat`: `Operation::of` gives it to pending rows and kept lines alike (no `create` word in code). `guide::PENDING` and the help text say `create` (`T/guide.rs:36,40,41,61,62`; `tests/golden/help.txt:35,39,40`; `golden/help_pending.txt:6,7`; `tests/help.rs:140,147,148`): stale. 05.4, 05.5, 06.5 use `chat`, correctly. 05.5's Files omit `guide.rs` and those goldens; add them, or `help pending` teaches `create`. |

## 2. Tool positions as bare numbers

`table::TOOLS` (`T/table.rs:8`) has 22 rows: 0 me, 1 inbox, 2 tasks, 3 task, 4 chat, 5 activity, 6 turn,
7 run, 8 chat (create), 9 message, 10 close, 11 answer, 12 decide, 13 release, 14 stop, 15 cancel,
16 wait, 17 adopt, 18 sign-in, 19 sign-out, 20 pending, 21 help.

| Kind of site | Count and where |
|---|---|
| `tool == N` or `!= N` | 23 in 7 files (`render/document/read_sets.rs` 8, `render/activity.rs` 4, `render/turn.rs` 4, `render.rs` 2, `render/document/turn_window.rs` 2, `runner/acts.rs` 2, `runner/reads.rs` 1): the retrospective's "about 23" is exact |
| `match call.tool` | 7 more: `runner.rs:115` (`0`, `1..=7`, `8..=15`, `16`, `_`), `runner/acts.rs:36,235,297`, `runner/reads.rs:45`, `render/outcome.rs:292`, `render/lists.rs:106`; plus `check_selected`'s `(tool, local, value)` table, `render.rs:316` |
| local slots (1 first read or needs, 2 activity, 3 tree watch, 4 run watch, 5 updates) | 25 in 5 files: `runner/reads.rs`, `render.rs`, `render/lists.rs`, `render/document/{subjects,read_sets}.rs` |
| by name | `root.rs:70`: `TOOLS[call.tool].name == adopt::ROW.name`; 17 to 21 reach `runner.rs:115`'s `_` ("call not supported") |

**Verdict.** No increment of 05, 06, 08, 09 or 13 adds, removes or reorders a row. 05.5 renders row 20
(`rows::PENDING`, present) and 13.4 calls row 17 (`adopt`, present); the positions hold through this
plan. The first new reader of positions is 05.1's `arguments::class` (line 97: "reads `TOOLS[call.tool]`",
tests it against `rows::{SIGN_IN, SIGN_OUT, PENDING, HELP, WAIT}`). `Tool` derives only `Debug`
(`T/jig/table.rs:14`), so it cannot use `==`: 05.1 must compare `.name`, as `root.rs:70` does, or an index.
Say so in 05.1. Naming the positions (`pub const ME: u8 = 0` to `HELP: u8 = 21` beside `TOOLS` in
`T/table.rs`, a test that `TOOLS[X].name` is the row's; the slots as consts in `T/jig/runner/reads.rs`)
belongs to the first increment that adds a row: none of these five is it.

## 3. Carriers for 05 (the retrospective's proposal 2)

| Item | Carrier | Status |
|---|---|---|
| pending: operation | `Operation::of(&jig_client::Ask)` and `Operation { word, task, to, party, retry }` (`T/jig/render/outcome.rs:8,16`, `pub(crate)`, in reach of `render/pending.rs`); input `jig_client::Held { key, request }` (`JC/jig-client-domain/src/boundary.rs:144`, re-exported `jig-client/src/lib.rs:15`) | carried |
| pending: party | `skein_lib::RequestRecord<R>.scope: u64` (`SK/skein-lib/src/requests.rs:106-110`); `jig_client::Pending.party` needs a live client, and `pending` starts none | carried by `scope` |
| pending: first sent | `RequestRecord.first_sent: Option<skein_lib::Wall>`; the JSON integer is `Wall::as_nanos` (`T/jig/render/document/read_turns.rs:93`) | JSON carried |
| first-sent words | `render/acts.rs:386` `wall` prints nanoseconds or `none`; `document.rs:277` `utc` is private clock time; `temper_client_view::words` (`WallFormat::Seconds`) is no tools dependency; jig's `words::english` is `WallFormat::Clock`, no wall hole | none |
| `party N`, `not sent` | literals of 05.5's new `render/pending.rs` | none |
| unread count | records `Shell::loaded()` returns (`JC/jig-client-shell-native/src/shell.rs:81`) that `translate::store::restore_table` or `v9::Kept::decode` refuses; a bare `u32` | none |
| unread line | 5.6: `N kept records these tools cannot read; a call that sends drops them`; `recovery.rs` `text` prints `dropped: N kept records ...`, another line | none (new text) |
| `kept:` line | `outcome::kept`, `text` (`outcome.rs:142`); `runner::Out::Kept { line }` (`runner.rs:45`) to `root::Request::Kept { line }` (`root.rs:29`) | carried |
| `dropped:`, parked | `T/jig/runner/recovery.rs` `dropped`, `parked`, `line`, `text`; party from `jig_client::Pending.party` | carried |
| exit 0, 1, 2, 3 | `jig::Printed.status: u8` (`T/jig/runner.rs:13`); literals at `render.rs:202,250-252,341`, `wait.rs:291-292`; the host's `skein_io::kernel::Exit::Code(u8)` (`SK/skein-io/src/kernel.rs:611`), `skein_shell::Host::exit` (`drive.rs:12`) | typed `u8`; no named const |
| exit 130, 143 | which signal: `skein_io::Event::Shutdown { signal: kernel::ServiceSignal }` (`records.rs:128`, `kernel.rs:601`); the numbers: 05.1's new `limits::{INTERRUPTED, TERMINATED}`; `temper/src/open.rs:119` maps `Exit::Signal` to 1, another rule | numbers: none |
| link deadline, lock | `limits::{DEADLINE 30 s, LOCK 60 s, LOCK_TRY 100 ms}` (`T/limits.rs:8-10`, `skein_lib::Duration`) | carried |
| `address`, link-file bounds | `limits::ADDRESS`, `limits::LINK`: new in 05.1, 05.3 | none |
| page and read bounds | `limits::{ROWS 32, TEXT, PART, DOCUMENT}`, `limits::tools() -> jig::Limits`, `limits::client()` (`reads` 16, `parts` 16); io: `skein_io::Limits` (`SK/skein-io/src/limits.rs:13`), `jig_client_shell_native::connections::Limits` | carried |
| `sign-in FILE`, `sign-in -` | `rows::SIGN_IN` (`T/jig/rows.rs:101`, `Kind::File`), `call::Parsed::File { file: Source::{Path, Stdin} }`; effect `Shell::sign_in(&[u8]) -> Result<Secret, Unlinked>`, `jig_client::Event::SignIn { with: With::{Link, Held} }` (`JC/jig-client/src/client.rs:14`, `jig-client-protocol-link/src/boundary.rs:142`); the read of stdin to EOF | carried, but the stdin read: none |
| `sign-out` | `rows::SIGN_OUT` (`rows.rs:109`), `jig_client::Event::SignOut`, `Shell::keep(None)` (`shell.rs:89`) | carried |
