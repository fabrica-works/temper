# real-loop: code organization

The crates, modules and names this plan settles, the owner of each shared
interface, the checks it adds, the fixtures, the "Don't" list and the size
limits. Where the design names a crate or a module, the design wins.

## 1. Crates

skein, smith and jig change no file, and no pin moves (decisions.md, 3).

### temper: two crates added

Each new crate's `[package]` inherits the workspace's fields, as
`tests/client`'s does (`version`, `edition`, `rust-version`, `publish`),
and `license.workspace = true` with them once temper's workspace names a
license (the accounts run licenses jig and temper, as skein 1222e7b did
skein); no gate checks it, so the step that makes the crate adds it.

`temper-client-shell-cli` (`crates/temper-client-shell-cli`) is `temper ctl`'s
one-shot host (tools.md, 4): a library, `forbid(unsafe_code)`, no `main`, its
`clippy.toml` the terminal's. Dependencies (`.workspace = true`):
`temper-client-tools`, `temper-client-domain`, `jig-client`,
`jig-client-shell-native`, `jig-client-protocol-{http,link,wire}`,
`skein-{io,lib,json,shell}` (05.1), `jig-client-domain` (05.5); dev-dependency
`jig-client-test-records` (05.2). No terminal dependency: the terminal's numbers
are copied into `limits`. Made by temper 05.1, added to by 05.2 to 05.6.

`temper-real-loop-world` (`tests/real-loop`) is the real loop's world and
fixtures (testing.md, 2.4). Dependencies, by the sessions' Files and sketches:
08.1 `skein-fake-peers`, `skein-fake-llm-domain`, `skein-fake-llm-protocol`,
`skein-llm`, `skein-llm-world`, `skein-world`, `smith-agent-process-world`,
`jig-channel-world`; 08.2 `temper`, `temper-service`, `skein-scratch`,
`skein-{io,lib,shell}`, `smith-agent-shell`, `serde_json`; 08.3
`temper-client-shell-cli`; 09.2 `temper-client-tools`; 09.4 `jig-client`,
`jig-client-shell-native`, `jig-client-protocol-wire`; 13.1 `temper-fake-git`,
`temper-worker-protocol-git`; 13.2 `skein-fake-forge-{domain,protocol}`. Never
`tests/one-process` (08). Made by temper 08.1, added to by 08.2 to 08.4, 09, 13.

| Existing crate or directory | Change | Session |
|---|---|---|
| root `Cargo.toml`, `Cargo.lock` | members `crates/temper-client-shell-cli`, `tests/real-loop`; workspace lines `jig-client-domain`, `temper`, `skein-scratch`; `Cargo.lock` alone in 06.1, 06.3, 09.2, 09.4, 13.1, 13.2, 13.3 | 05.1, 05.5; 08.1, 08.2 |
| `crates/temper` | new `src/ctl.rs`; `lib.rs`, `tests.rs`, `Cargo.toml`; 13.3 a dev-dependency on the world, `Cargo.toml` alone | 05.6; 13.3 |
| `crates/temper-client-tools` | new `src/jig/render/pending.rs`; `render.rs`, `src/guide.rs`, its goldens and help test; the nine comments citing `tools.md` or `testing.md` bare | 05.5 |
| `tests/client` (`temper-client-world`) | the agent beside the person: new `src/agent.rs`; `app.rs`, `front.rs`, `fixtures.rs`, `translate.rs`; `Cargo.toml` (06.1, 06.3) | 06 |
| `crates/temper/tests` (process tier) | new `support/ctl.rs`; `support/{mod,owned,forgejo}.rs`, `process.rs` | 10.1, 10.2; 13.3 |
| `tests/engine/domain` (`temper-engine-domain-world`) | `tests/fuzzy_walking.rs`: the walk in eight shards; `.config/nextest.toml` (no other step edits it); not `src/walking.rs` | 14.1 |

## 2. Layering

Upstream only: skein, smith, jig, then temper. In temper, `temper-client-tools`
(step code under the subset) is the lowest; `temper-client-shell-cli` (ordinary
Rust) stands on it, and `crates/temper` (`ctl.rs`) on both. The host has no
`main`, no terminal, and builds no `Startup`: `ctl::run` and the world's
factory (08.3) do.

`tests/client` hosts the tools' `Root` on jig's client world. `tests/real-loop`
hosts `temper::prepare_in`, the host and skein's fake peers on skein-world's real
kernel. From 13.3 the process tier's `support` calls that world, so
`crates/temper` dev-depends on a crate that depends on `temper`. If cargo or a
lint refuses it, that is a question (13), never a copy.

## 3. Modules and names

- **The host, `src/`:** `arguments`, `directory`, `host` (`Startup`, `Invocation`,
  `OneShot`), `output`, `limits` (`native`, `INTERRUPTED`, `TERMINATED`, `ADDRESS`,
  `LOCK`, `LOCK_TRY`) (05.1); `transport` (05.2); `signing` (05.3, 05.5)
  with `limits::LINK`; `recovery` (05.4); `pending` (05.5). Units in
  `src/tests.rs`, fixtures in `src/tests/fixtures.rs`. The executable is
  `crates/temper/src/ctl.rs` (`run`, 05.6), with `Command::Ctl` in `lib.rs`.
- **Tools:** `src/jig/render/pending.rs` lies under `src/jig/`, so it names no
  temper and no `crate::` path but `crate::jig`; its tests are
  `src/tests/jig/pending.rs` (05.5).
- **`tests/real-loop/src/`:** `lib` (`run`, `run_on`, `Ran`, `Called`), `llm`,
  `scripts` (08.1); `fixtures`, `world`, `service` (`ServiceHost`), `agent`,
  `calls` (08.2); `unread` (09.4); `git` (13.1); `forge` (13.2); `landing`
  (13.3, 13.5). Tests in `tests/`, one binary per subject, each with skein-world's
  `domain::heap::Counting` as its allocator: `peers`, `lifecycle`, `tools`,
  `restart`, `lost`, `lock`, `unread`, `git`, `forge`; `landing` is a unit. A story
  past 1 s moves whole to `fuzzy_restart.rs` or `fuzzy_landing.rs` (09.1, 13.5).
- **`tests/client`:** `agent.rs` (`Agent`, `new`, `limits`, `step`), `Root::Agent`
  and `Event::Call` in `app.rs` (06.1); `front.rs` `Joint::{next_client_limits,
  with_large_turns, answers_to_lose}` (06.1, 06.3, 06.5); tests `agent.rs`,
  `agent_turn.rs`, `agent_recovery.rs`.
- **Process tier:** `support/ctl.rs` (`Call`, `Story`, `Step`, `run`, `alone`,
  `json`; 10.1, extended by 10.2), `Process::Calls`, `Running::id`,
  `Throwaway::pid`. **Domain world:** `drawn`, `shard`, `fuzzy_walking_shard_0..7`.

## 4. Shared interfaces and their owners

| Interface | Owner | Users |
|---|---|---|
| `host::{OneShot, Startup, Invocation}`, `arguments::parse`, `limits::*`, the host's `fixtures::*`, `Command::Ctl`, `ctl::run` | temper 05.1 (`ctl::run`, 05.6) | 05.2 to 05.6, each adding its own fixture; 08.3; 09.3 (`LOCK_TRY`); 10.1 |
| `Agent`, `fixtures::{agent, call, give, answered, done_task}`, `Event::Call` | temper 06.1 | 06.2 to 06.5 |
| `run`, `run_on`, `Ran`, `Called`, `llm::*`, `scripts::*`, `fixtures::{documents, write, AGENT, TOKEN, SEED}` (`CTL` 08.3's), `ServiceHost`, `calls::{Dir, D1, Arg, Argv, Step, task, occurrences}` | temper 08.1, 08.2 | 08.3, 08.4; 09 (`D2`, `Number`, `Kept`, lock and kill steps); 13 (`Advance`, `Delegate`) |
| `Ran::number`, `Called.{locked, busy}`, `calls::Recorded`, `service::Cue`, `unread::*` | temper 09 | 13 |
| `git::*`, `forge::*`, `landing::*`, `run_forge`, `Ran::forge` | temper 13 | 13.4, 13.5; the process tier (`landing::{scripts, configure, EDIT}`, 13.3) |
| `support::configuration::documents` | the process tier (exists) | copied by 08.2, named in its commit body |
| `fixtures::{peer_limits, forge_limits}`, `hooks_address`, fake git's `Transport` | forge-kit's temper-05.2 | 13 copies the first three, imports the last |
| `key::engine`, `limits::{client, tools}`, `root::Root`, `jig::Printed` | the tools plan | 05, 06, 09.2 |

A later session extends a fixture and never redefines it. A name taken from
draft temper 05 or forge-kit is checked against the landed code first.

## 5. Checks, and fixtures

- **The host:** `test -z "$(git grep -n -E 'fn main\(|process::exit' origin/main
  -- crates/temper-client-shell-cli)"` (05). No global stdio, exit or signal
  handler; a signal ends the call with `limits::INTERRUPTED` or `TERMINATED`,
  `Shutdown` with 128 + N (design question 3). The tools' boundary test
  `no_file_under_jig_names_temper_or_an_item_outside_jig` scans `pending.rs`.
- **Checked or not:** `run_on` uses `World::new_controlled_checked` (08.2); only
  13's `git` tests and `run_forge` use `new_controlled`, said in the commit body
  (design question 4). Every listener binds port 0.
- **Names:** no test is named with `_and_` (decisions.md, 13). Each of 06, 08,
  09, 10, 13 and 14 ends with a `git grep` over its test names.
- **Times:** a host unit under 20 ms; a client story and a world test (08.1,
  13.1) under 0.5 s; a real-loop story aims at 1 s, run alone, unoptimized, three
  times, the median in the commit body; a shard (14.1) about 5 s.
- **Fixtures:** one module per crate, made by the first session, extended, never
  redefined: the host's `src/tests/fixtures.rs` (05.1), `tests/client/src/
  fixtures.rs` (exists), `tests/real-loop/src/fixtures.rs` (08.2), the process
  tier's `support/` (10.1). A helper two stories need is the world's, said in the
  commit body. Limits come from the builders, never typed.

## 6. The plan's "Don't" list

- No optimized test build and no `[profile.*]` change. No budget raised to pass
  a slow test: it is profiled and fixed in the code or the test.
- No new fake forge in temper: the forge stories host skein's forge peer
  (decisions.md, 1). No browser shell code.
- No jig, smith or skein change and no repin (decisions.md, 3). No edit of
  `tests/one-process`, `testing/temper-fake-git` or skein's fake peers.
- No `main`, `process::exit` or second parser in the host. No lock on `store`;
  no lock or dial for `pending`. No sleep for order: a cue or an `Until`. No new
  `#[expect]` or `#[allow]` the report does not name.
- **Transitional code and its end:** 08.2's end check "nothing was killed" is
  replaced by 09.2's "as many killed as `kill` steps"; 08.2's held forge listener
  ends in forge stories at 13.2; `owned::attach`'s "read-only proof handle" and
  `forgejo.rs`'s "isolated Forgejo" end at 10.2; `Parsed::File` is usage until 05.6.

## 7. Size limits and the count

The aims, the +40 rule over 800 lines, the 2x and 2.5x bounds and the moves rule
are development.md's, section 5. Part 1's count: `git diff -M --numstat
origin/main...HEAD -- crates testing tools`, less `':!*/tests.rs' ':!*/tests_*.rs'
':!*/tests/*' ':!*/generated/*' ':!*/golden/*' ':!*/schema/*'`, summed by `awk`.
It leaves out top-level `tests/` and `crates/temper/tests/`; `tests/client` and
`tests/real-loop` are counted the same way over `-- <dir>/src`.

| Session | Size of each increment, in order | Counted by |
|---|---|---|
| 05 | at most 290, 280, 200, 280, 300 (the renderer included), 260 production lines | part 1's (05 gives no command) |
| 06 | at most ~260, ~20, ~150, ~20, ~40 lines | `tests/client/src` |
| 08 | at most ~120, ~300, ~200, ~150 lines | `tests/real-loop/src` |
| 09 | at most ~90, ~220, ~160, ~130 lines | `tests/real-loop/src` |
| 13 | at most ~150, ~280, ~150 (moved), 0, ~120 lines | `tests/real-loop/src` |
| 10 | 0 by the plan's count; about 190 and 60 by hand | none (`crates/temper/tests/`) |
| 14 | no production line; about 85 net test lines, 3 of configuration | by hand |

The world grows by about 2070 lines of `src` in 13 increments, and 08.2 to 13.5
all add to `lib.rs`, `calls.rs`, `world.rs` and `service.rs`. An increment that
would take a file past 800 lines splits it in the same commit, by concern; the
host's `src/tests.rs`, which takes the units of six increments, splits by path
first. The files the plan edits are under 800 lines at `efe462a25`; the longest
are `support/owned.rs` (593), `tests/client/src/front.rs` (588), `support/mod.rs`
(572).
