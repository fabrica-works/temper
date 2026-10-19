# tools: code organization

The crates, modules and names this plan settles, the owner of each shared
interface, the checks it adds, the fixtures, the "Don't" list and the size
limits. Where the design names a crate or a module, the design wins.

## 1. Crates

### skein (skein 01)

No crate is added. `skein-lib`'s `src/requests.rs` gains `RequestNumber`,
`RequestAdmission { key, number }` and `RequestRefused<T>` (`Full`, `Held`),
returned by `ask` and `restore`, with `not_kept(key, number)` (01.1), then
`ask_given` (01.2); new `src/tests/requests/admissions.rs`.

### jig: crates changed; none added, renamed or removed

| Crate | Change | Session |
|---|---|---|
| all | repin onto skein `b4c6241`, smith `a277e922` (at most 200 lines besides `Cargo.lock`); then onto skein 01 | 01.1; 02.1 |
| `jig-protocol-transcript` | jobs as call outcomes (`summary.rs`); new `tests/client/src/joint/stories/job_stories.rs` | 01.2 |
| `jig-client-domain` | `Event::Send { key, name: Option<RequestKey>, request }`, `Outcome::Held`, `Domain::send`; `contract/`: `Read::Turn`, `ReadValue::Turn(Box<ArchivedTurn>)`, `ReadNone::{TooLarge { bytes }, Malformed}`, `TurnProblem::TooLarge { bytes }`, `NewDeadline` | 02.2; 03.3, 03.4; 05.2 |
| `jig-client` | `client.rs` forwards `Send`, reads `drain`'s bound once; `facts.rs`; `shapes.rs`: `chat`, `set_goal`, `fall` without a clock | 02.2, 04.3; 03.3; 05.2, 05.4 |
| `jig-client-protocol-wire` | version 8 to 9: `client-v9{,-part1}.schema`, `generated/v9.rs`, `golden/v9/` replace v8's; the deadline selector | 03.1; 05.2 |
| `jig-client-protocol-link`, `jig-protocol-engine-clients` | translations; new `jig-client-protocol-link/src/translate/turn.rs` | 03.2 to 03.4; 05.2 |
| `jig-protocol-store` | new `transcripts/read/party.rs`; people and tasks families v3 to v4 (core stays 3, notes and component 1) | 03.2; 05.1, 05.2 |
| `jig-domain-{core,tasks,people,authority}` | new `reads/turn.rs`, `routing/deadlines.rs`; `Deadline { Named, Span }`; `deadline_after` in the authority's `value.rs` | 03.2; 05.2 (Named), 05.3 (Span) |

### temper: one crate added

`temper-client-tools` (`crates/temper-client-tools`) is step code under the
subset (`no_std` with `alloc`): the table, parser, runner, renderers, waits,
JSON and help; no I/O, no clock. Dependencies (`.workspace = true`):
`jig-client`, `temper-client-domain`, `temper-client-protocol-wire`,
`skein-lib`, `skein-json`, and `skein-ui` (03.2, its first use); dev-dependency
(the preset test) `temper-client-shell-terminal`; never `temper-client-view`. Nothing depends
on it in this plan; the next plan's host will. Made by temper 02.1; added to
by 02.2, 02.3, 03.2 to 03.4, 04.1 to 04.4, 11.2, 12.1 and 12.2.

| Existing crate | Change | Session |
|---|---|---|
| all | repin; the arms jig 02 to 05 force (`people_member`, `tasks::New`, `Outcome::Held`) | 01; 03.1 |
| `temper-client-protocol-wire` | `Owner::Runner` = 3 | 03.2 |
| `temper-client-domain` | `owner.rs`, `route.rs` arms; new `forge/words.rs` (the adoption words moved from the view; `skein-ui` added); `forge.rs` `ask`, `answer`; `forge/form.rs` `write` | 03.2; 11.1; 11.2 |
| `temper-client-view`, `temper-service` | `forge/*`, `words.rs`, `root.rs` use the domain's words; `family.rs` arm for `Runner`, wire v9 | 11.1; 03.1, 03.2 |

## 2. Layering

skein, then jig (`jig-client-domain`, `jig-client`), then
`temper-client-domain` (owners, routing, forge words), then
`temper-client-tools`: its `src/jig/` stands on `jig-client` and skein alone,
the rest on the domain. This plan builds every layer's part; the next plan
stacks a host and its stories on `Root`.

## 3. Modules and names

- **`src/jig.rs`, `src/jig/`** name no temper and no `crate::` path but
  `crate::jig`: `key`, `table` (`Tool`, `Argument`, `Kind`, `Keyed`, `Limits`:
  rows, ask bytes, text, part, wait and its most; `jig::Limits`), `call`, `argv`
  (`parse`, `check`, `back`), `rows` (02.1, 02.2); `subject` (02.3); `runner`
  (03.2; 04.3, 04.4; `Printed`, as `jig::Printed`); `render` with `document`,
  `acts` (03.2), `lists`, `activity`, `turn` (03.3), `outcome` (03.4), and
  `Part` (04.1); `digest` (04.1); `conditions` (04.2); `wait` (04.3, 04.4);
  `json` (12.1); `help` (12.2).
- **Beside it:** `limits`, `key` (`TOOLS`, `engine`) (02.1); `table` (02.2);
  `adopt` (`ROW`, 02.3); `root` (03.2; 04.3, 11.2); `forge` (adopter and forge
  part under `Owner::Forge.asker()`, 11.2); `guide` (`GUIDE`), `help` (12.2).
- **Tests:** none under `src/jig/`. `src/tests/{fixtures,boundary,limits,key}.rs`
  (02.1), then `table` (02.2), `subject` (syntax, 02.3), `root` (03.2), `adopt`
  (11.2), `json` (12.1), `help` (12.2); `src/tests/jig/`, the tests of
  `src/jig/`'s printing and waiting: `reads` (03.2), `lists`, `turn` (03.3),
  `acts` (03.4), `digest`, `conditions`, `wait` (04); goldens in
  `src/tests/golden/` (03.2 to 03.4, 04, 11.2, 12.2).

## 4. Shared interfaces and their owners

| Interface | Owner | Users, by name |
|---|---|---|
| `RequestAdmission`, `RequestRefused`, `ask_given`, `not_kept(key, number)` | skein 01 | jig 02.1, 02.2 |
| `Event::Send { name }`, `Outcome::Held` | jig 02.2 | jig 05.5; temper 03.1, 03.4, 11 |
| `Read::Turn`, `ReadNone`, wire v9 | jig 03 | jig 05; temper 03.1, 03.3 |
| `NewDeadline`, `Deadline`, `deadline_after`, store v4 | jig 05 | temper 03.1 |
| `jig::table`, `argv::{parse, check, back}`, `jig::Limits`, `adopt::ROW`, `jig::subject` | temper 02 | temper 03, 04, 11.2, 12 |
| `Root`, `root::Limits`, `Request`, `Runner`, `jig::Printed`, `render::Parts` | temper 03.2 | temper 04.3, 11.2; the next plan |
| `digest`, `conditions`, `wait`, `render::Part`; `jig::json`, `help::text`, `GUIDE` | temper 04; temper 12 | temper 12.2; the next plan |
| `forge/words.rs`, `forge::{ask, answer, write}` | temper 11 | `src/forge.rs`; the view |

## 5. Checks, and fixtures

- **The boundary:** `no_file_under_jig_names_temper_or_an_item_outside_jig`
  (`src/tests/boundary.rs`, 02.1) scans `src/jig.rs` and `src/jig/**`, so no
  test lies there; `client_crates_carry_the_client_prefix` covers the crate unchanged.
- **One wire version:** `git -C ~/src/rust/jig grep -n -E
  '(CLIENT_VERSION|VERSION): u32 = [0-9]' origin/main -- crates` prints one
  line, `= 8` today, `= 9` from jig 03.1. `format::current` is `people: 4,
  tasks: 4` (05.1).
- **The preset** equals the terminal's `native().client`, `rows` aside (02.1);
  `EVERY_TOOL` feeds the table (02.2) and help (12.2) tests.
- **Fixtures:** one module per crate, made by the first session, extended,
  never redefined: `src/tests/fixtures.rs` (02.1; 03, 04, 11, 12; 02's `call`
  is the parser's, 03's harness `submit`s), `tests/lib/src/request_table.rs`
  (skein 01), the `Story` helpers of `tests/client-protocol` (02.3; 03.5, 05.5).

## 6. The plan's "Don't" list

- No name of temper in jig: `git -C <jig worktree> grep -n -i -w temper --
  crates testing tests examples scripts` prints nothing at each jig
  increment's end. skein names no jig, smith or temper.
- No dependency on `temper-client-view`; the terminal only as a dev-dependency.
  No second parser of argv or conditions, no copy of form checks or family
  codecs, no name or key in a kept or pending line, no alias, no old reader.
- **Transitional code and its end:** jig 01.1's refusal of job outcomes and
  `unreachable!` arms, to jig 01.2; jig 02.1's `unreachable!("a drawn key is
  never held")`, to 02.2; ruling 275's doubled timeout (`.config/nextest.toml`,
  30 s to 15 s), to jig 04.4; the root's path-join fallback for `render::Parts`,
  temper 03.2 to 11.2; each repin's pins, to the next repin (temper 01, 03.1).

## 7. Size limits and the count

The aims, the +40 rule over 800 lines, the 2x and 2.5x bounds and the moves
rule are development.md's, section 5. The count: `git diff -M --numstat
origin/main...HEAD -- crates examples testing` for jig (`crates testing
tools` for temper, `crates testing` for skein), less `':!*/tests.rs'
':!*/tests_*.rs' ':!*/tests/*' ':!*/generated/*' ':!*/golden/*'
':!*/schema/*'`, summed by `awk`. Lines at origin/main today (jig `6ad8bb40`,
skein `064c55f`, temper `1c918a77c`):

| File | Lines | Sessions | Rule |
|---|---|---|---|
| jig `jig-domain-tasks/src/{boundary,stored}.rs` | 1857, 1082 | 05.2; 05.3 | +40 each |
| jig `jig-domain-core/src/routing/tasks.rs`, `effects.rs`, `documents/objects.rs` | 1586, 1454, 930 | 05.2, 05.3 | +40 each |
| jig `jig-client-domain` `contract/documents.rs`; scripted engine `payload.rs` | 1028; 1329 | 03.3, 05.2 | +40 each |
| jig `jig-client-view` `sections/project/owner.rs`, `objects.rs`, `page/routing.rs` | 1209, 1133, 1103 | 04.1, 04.2; 03.3 | +40; arms only |
| jig `examples/ops/client-view/src/tests.rs`; `tests/core/tests/parties.rs` | 1760; 817 | 04.2; 05 | +40; none added |
| jig `tests/client-protocol/tests/client.rs`; `jig-protocol-store` `transcripts/read.rs`, `codec/tasks/phase.rs` | 797; 745; 768 | 02.2; 03.2; 05.2 | under 800 (`mod named;`: 799) |
| skein `skein-lib/src/requests.rs`, `tests/lib/tests/memory.rs`, `tests/lib/src/request_table.rs` | 669, 761, 635 | 01 | `admissions.rs` takes the new tests |
| temper `temper-service/src/tests/fixtures.rs`, `tests/engine/domain/tests/root_routes.rs`; `tests/conformance/src/scenarios.rs` | 840, 4097; 753 | 03.1 | +40 each; under 800 |

The next plan's crates stack on this one and are not built here: the host
`temper-client-shell-cli`, `temper ctl` in `crates/temper`, `tests/real-loop`
and `testing/temper-fake-forge-host`.
