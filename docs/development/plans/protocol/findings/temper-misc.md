# Audit: temper-misc

2026-10-14, read only, by a sub-agent of the planner, at temper `cc2697cf` with jig `d91a89c4`, smith `dfc0fe57` and skein `b5d3a2c3`. Evidence the plan cites: the retiring crates' dependents and stories, the Python conformance tool, the suites, crate kinds and lints, adding a crate at the pins, and STATUS's rows. Where it and landed code differ, the landed code wins.

I've checked all seven items. The main surprise is item 6: adding a jig crate that isn't already in `Cargo.lock` does need a repin step. In a scratch test it moved skein and smith off their pins. Paths are relative to `~/src/rust/temper` unless absolute.

**1. Dependents of the four retiring crates**

Sizes, every file counted:

| Crate | Lines | Package name |
|---|---|---|
| `crates/temper-web-domain` | 3710 (`domain.rs` 1523, `tests.rs` 701, `tests_w4.rs` 601) | |
| `crates/temper-web-view` | 2310 | |
| `testing/temper-fake-person` | 410 | |
| `tests/web/domain` | 1995 | `temper-web-domain-world` |
| **Total** | **8425** | |

- **Cargo.toml:** workspace members at lines 6, 15, 17 and 27; `[workspace.dependencies]` entries at 37, 54 and 55. `temper-web-domain-world` has no workspace dependency entry.
- **Cargo.lock:** entries at 993–998 (fake-person), 1041 (web-domain), 1048–1055 (web-domain-world) and 1059–1063 (web-view). Cargo drops them by itself once the members are removed; removing members does not change any other pin.
- **Dependencies between them:** only among themselves.
  - `crates/temper-web-view/Cargo.toml:11` uses `temper-web-domain = { path = "../temper-web-domain" }`, a path, not `.workspace`.
  - `testing/temper-fake-person/Cargo.toml:11-12` depends on the other two.
  - `tests/web/domain/Cargo.toml:12-14` depends on all three.
- **`use` sites:** none outside the four directories.
- **No references** in `scripts/check.sh`, `.config/nextest.toml`, `AGENTS.md`, `clippy.toml` or `tests/clippy.toml`.
- **Docs that cite them by path:**
  - `docs/design/client.md:44-45` and `:828-833`
  - `docs/design/protocol.md:261-263` and `:555-557`
  - `docs/design/testing.md:532-534` and `:758-761`
  - `docs/design/STATUS.md:11` (the web-implementation plan row) and `:57`
  - The superseded plan, `docs/plans/web-implementation/`:
    - `README.md:24,27,94-105,182-186`
    - `01-domain.md:3,41`
    - `02-view.md:3,38,272`
    - `03-testing.md:41,103,116-121,242,280-281`
    - `04-increments.md:12`
  - Not a citation: `docs/design/web/mockups/{app,agent-task,terminal-chat}.js` only use the localStorage key `'temper-web-mockup-v1'`.

**2. Stories in `tests/web/domain`**

These run against a scripted engine (`tests/web/domain/src/engine.rs`, 181 lines; `scenario.rs`), not temper's engine root. There are 29 world tests. There are also unit tests: 32 in `temper-web-domain`, 13 in `temper-web-view` and 2 in `temper-fake-person`.

`tests/slice.rs`:
- `signs_in_then_starts_one_chat`: signed out, press Sign in, type and Start chat, the task page opens. **Sign-in plus a new chat.**
- `composer_survives_reload_without_submission`: client-only (draft kept).
- `seed_replays_the_same_boundary_trace`: replay.
- `held_chat_can_be_released_from_its_page`: task 27 held, Release, Confirm, decision `Released`. **Escalation.**
- `held_chat_can_be_left_with_a_reason`: Leave held plus a reason, which is `Reject{reason}`. **Escalation.**
- `completed_chat_shows_report_after_reconnect_with_markdown_subset`: rendering of a task's result.

`tests/faults.rs`:
- `restart_before_durable_resends_same_key`, `restart_after_durable_answers_by_key`, `reload_in_flight_and_double_press_make_one_chat`, `busy_answer_retries_with_the_same_key`: keyed idempotency of starting a chat.
- `offline_press_is_parked_until_the_watch_returns`.
- `dropped_and_missed_watch_recover`.
- `signed_out_mid_request_resumes_after_sign_in`.
- `refused_chat_shows_a_reason`: no project role, so the chat is refused.
- `revision_change_while_confirmation_is_open_prevents_decision`, `stale_button_press_after_task_changes_does_nothing`: escalation against a stale revision.
- `unsent_reason_survives_immediate_reload`: client-only.

Other files:
- `tests/two_tabs.rs`: `two_tabs_answer_before_watch_change` and `two_tabs_watch_change_before_answer`. Two tabs decide the same escalation; the second sees "Ada released at", which is `DecidedBefore`.
- `tests/referee.rs`, 7 tests checking the referee itself: `rejects_visible_before_durable`, `rejects_two_keys_making_one_task`, `rejects_lost_unsent_words`, `rejects_silent_refusal`, `rejects_false_live_link`, `unanswered_press_fails_its_deadline`, `unrecovered_watch_fails_its_deadline`.
- `tests/memory.rs`: `full_chat_window_stays_within_declared_heap_plus_fixed_render_words`.
- `tests/fuzzy_web.rs`:
  - `seeded_people_and_faults_keep_the_world_consistent`: 32 seeds, pinned 5, 13 and 16; every `Fault` must be reached.
  - `held_task_random_people_keep_one_decision_across_faults`: seeds 100–131, at most one decision each.

What the scripted engine models:
- A `signed_in` flag, with `expire_signin` to sign out.
- Chat reads.
- `Ask::StartChat{project,words}`, answered `Started{task}`, `Refused(Role)` or `Refused(Unknown)`.
- `Ask::Decide{Waiting::Escalation{task},revision,Release|Reject{reason}|Pass}`, answered `Decided`, `DecidedBefore`, `Refused::Moved{revision}` or `Refused::Ended`.
- Durable keys with `KeyConflict`, task-snapshot watches, and moving a task's revision or ending it with a report.
- Faults: Restart, Busy, Dropped, Missed, Reload, DoublePress, SignedOut, Latency (`world.rs:88`).

The engine-root equivalents for the rewrite in `tests/client`:
- **Sign-in:** `PartyInput::SignedIn{reply_to,identity}` (`crates/temper-engine-domain/src/boundary.rs:333`) goes to `core::Event::SignIn` (`route.rs:101`). Refusals arrive as `SignInRefused` (`boundary.rs:451`).
- **Asks:** `PartyInput::Ask{reply_to,sign_in,key:[u8;16],ask: people::Ask}` (`boundary.rs:340`, `route.rs:104`).
  - A new chat is jig's `people::Ask::StartChat` (jig-domain-people `boundary.rs:331`).
  - An escalation decision is `people::Ask::DecideEscalation`.
  - temper's `EscalationChoice{Release,Reject{reason},Pass}` is at `boundary.rs:189`.
- **Reads and watches:** `Read`, `Watch` and `WatchRun` (`boundary.rs:352,358,365`).
- **Historical held-chat decisions:** `route/escalation.rs` (`Query`), with `EscalationLoaded` and `EscalationFailed` (`boundary.rs:488-489`).
- **Existing root-level coverage:** `tests/engine/domain/tests/escalation.rs` has 10 escalation tests at the root, including key conflict and owner races.

Under client.md §9, these carry over: sign-in plus a new chat (slice 1, the faults idempotency tests), and escalation decided (slice 4–5, faults revision/stale, two_tabs, fuzzy held). The rest (drafts, markdown rendering, the referee self-tests, memory) belong to jig's client domain and world kit.

**3. `tools/forgejo-conformance`**

Files: `run.py` (212 lines), `next_domain.py` (200), `README.md` (60).

What `run.py` does, in order:
1. **Checks the binary.** It runs the hardcoded `/tmp/temper-forgejo-15/forgejo-15.0.0-linux-amd64 --version` with `GIT_CONFIG_GLOBAL=<scratch>/gitconfig` and `GIT_CONFIG_NOSYSTEM=1`. The output must start with "forgejo version 15.0.0" and the SHA-256 must match the pinned digest (`run.py:35-41`).
2. **Starts a webhook receiver** on 127.0.0.1:0. It answers 204 and keeps three headers (x-forgejo-event, x-forgejo-signature, content-type) plus the body (`:44-55`).
3. **Picks a port** by binding 127.0.0.1:0 and closing (`:23-26`), then makes a tempfile scratch directory and a copy of the environment with the git config isolated (`:56-59`).
4. **Writes `app.ini`** (`:61-94`):
   - top level: `APP_NAME`, `RUN_USER=$USER`, `WORK_PATH=S`
   - `[server]`: `PROTOCOL=http`, `HTTP_ADDR=127.0.0.1`, `HTTP_PORT=<port>`, `ROOT_URL=http://127.0.0.1:<port>/`, `SSH_DOMAIN=127.0.0.1`, `APP_DATA_PATH=S/data`, `DISABLE_SSH=true`, `OFFLINE_MODE=true`
   - `[database]`: `DB_TYPE=sqlite3`, `PATH=S/forgejo.db`
   - `[repository]`: `ROOT=S/repositories`, `DEFAULT_BRANCH=main`
   - `[security]`: `INSTALL_LOCK=true`, `SECRET_KEY` and `INTERNAL_TOKEN` each 32 random bytes in hex
   - `[service]`: `DISABLE_REGISTRATION=true`
   - `[webhook]`: `ALLOWED_HOST_LIST=127.0.0.1`
   - `[actions]`: `ENABLED=true`
   - `[api]`: `MAX_RESPONSE_ITEMS=2`
   - `[log]`: `MODE=console`, `LEVEL=Error`
5. **Migrates:** `forgejo --work-path S --config S/app.ini migrate` (`:95-96`).
6. **Creates users before the server runs** (`:97-100`):
   `forgejo --work-path S --config S/app.ini admin user create --username fixture --password P --email fixture@example.invalid --admin --must-change-password=false`
   and the same for `reviewer` without `--admin`. `P = secrets.token_urlsafe(32)`, shared by both.
7. **Starts the server:** `forgejo ... web` as a child, stdout and stderr to `S/server.log` (`:101-102`).
8. **Waits for readiness:** up to 200 polls of `GET /api/v1/version` (0.1 s timeout, 0.05 s sleep), failing if the process exits (`:124-133`).
9. **Makes tokens:** `POST /api/v1/users/{name}/tokens` with Basic auth `name:P` and body `{"name":"isolated","scopes":["all"]}`, expecting 201, and keeps the `sha1` field. Later calls send `Authorization: token <sha1>` (`:105-111`, `:135-136`).
10. **Sets up the fixture** (`:141-182`): `POST /user/repos {"name":"specimen","auto_init":true,"default_branch":"main"}`, a hook to the receiver with secret `isolated-fixture-hook-secret`, then `PUT /repos/fixture/specimen/collaborators/reviewer {"permission":"write"}`, then the probes.
11. **Tears down** (`:188-208`): server terminate, wait 10 s, then kill; receiver shutdown. It scans for secrets and refuses output if any leaked, writes `partial-observations.json`, then `observations.json` with provenance (binary, version, sha256, time, isolation).

What `next_domain.py` adds (runner registration):
- `PATCH` the repo with `{"has_actions":true}` using the admin token (`:97`).
- `GET .../actions/runners/registration-token` returns `.token` (`:98-99`).
- Writes `runner/config.yaml` (`:103-117`):
  - `log.level: info`
  - `runner`: `file: <runner>/registration.json`, `capacity: 1`, `timeout: 1m`, `shutdown_timeout: 1s`, `fetch_interval: 100ms`, `report_interval: 100ms`, `labels: [temper-host:host]`
  - `cache.enabled: false`
  - `host.workdir_parent: <runner>/jobs`
- Gives the runner a minimal environment: PATH, USER, LOGNAME, LANG and the two GIT_CONFIG variables (`:118`).
- Registers: `forgejo-runner --config C register --no-interactive --instance http://127.0.0.1:PORT --token T --name temper-00a --labels temper-host:host`, run in the runner directory (`:121-125`). Then starts `forgejo-runner --config C daemon` (`:127`).
- The workflow is `.forgejo/workflows/conformance.yaml` with `runs-on: temper-host` and an inline step that does `exit 1` (`:85-96`). The runner stops with terminate, wait 5 s, then kill.
- The runner path is hardcoded to `/home/free/.local/bin/forgejo-runner` (v3.5.1).

**Where the tool and the design disagree** (the Rust replacement should follow the design):
- The tool pins v15 by a `/tmp` path plus digest. `docs/design/forge.md:733-745` and `testing.md:195-203` require v16.0.5 named by `TEMPER_FORGEJO`, refusing without it or on another version, and recording the digest.
- The runner should come from `TEMPER_FORGEJO_RUNNER`; without it the Actions facts are reported as unchecked.
- The process tier runs no runner (`testing.md:204`).

Docs that cite the tool or the throwaway Forgejo:
- `forge.md:692,733-745`
- `testing.md:195-203,741,771`
- `git.md:578-579`
- `protocol.md:112,236-239,505-506,536-537,575`
- `domain/forge.md:856,866`
- `draft/qa.md:54,100-101,280`

**4. Test suites**

Each world's binaries; focused unless marked fuzzy:

| World | Package | Focused | Fuzzy |
|---|---|---|---|
| `tests/engine/domain` | `temper-engine-domain-world` | escalation, forge_routes, loads, memory, preparation, referee, roles, root_routes, walking | endings, escalation, loads, roles, walking |
| `tests/system` | | composed, simulation | composed |
| `tests/conformance` | | simulation, stories | shared, stories |
| `tests/engine/forge` | | memory, replay, simulation | simulation |
| `tests/engine/forge/client` | | memory, simulation | simulation |
| `tests/worker/checkout` | | git, memory, merges, simulation | merges, simulation |
| `tests/worker/domain` | | runtime, runtime_memory, whole | runtime, whole |
| `tests/fake-forge` | | memory | — |
| `tests/web/domain` | | faults, memory, referee, slice, two_tabs | web |
| `tests/fake-checkout` | | no tests (lib only) | — |

Unit tests live in `crates/*/src/tests.rs` (plus `temper-web-domain/src/tests_w4.rs`), `testing/*/src/tests.rs`, and the world libraries of system, worker-domain and engine-domain.

Measurements:
- **Commit 796d5027 trailer:** `Gate-Result: 682 tests / 1.176 s; 31 tests / 6.257 s`.
- **Per-test log:** `~/src/rust/tmp/temper/jig-reconcile/logs/gate-final.log`, outside the repo; nothing in `docs/` records the slowest tests.
- **Slowest focused:**
  - engine-domain-world escalation `held_and_decision_commit_cuts_restore…`, 0.707 s
  - escalation `a_historical_query_retries_busy…`, 0.312 s
  - worker-checkout `facts_change_nothing…`, 0.293 s
  - worker-checkout memory, 0.293 s
  - conformance simulation `a_changed_judge…`, 0.249 s
- **Slowest fuzzy:**
  - `fuzzy_walking …commit_cuts_latency…`, 4.346 s
  - conformance `fuzzy_shared a_changed_judge…`, 3.448 s
  - `fuzzy_stories a_ci_repair…`, 2.708 s
  - `fuzzy_escalation fifteen_small…`, 2.651 s
  - `fuzzy_walking terminal…`, 2.156 s
- **The web crates' share:** 74 focused tests (32 + 13 + 2 + 27), each ≤ 0.005 s, and 2 fuzzy (0.777 s and 0.776 s). Retiring them leaves 608 focused and 29 fuzzy.
- **Budgets:** `.config/nextest.toml` sets default `not binary(/^fuzzy_/)` at 15 s with a 3 s slow flag, fuzzy at 60 s with a 20 s slow flag, and a `measure` profile with no limit. `testing.md:658-663` adds guides: worlds below the domain (protocol, machine, client, one-process) ≤ 4 s focused and ≤ 25 s fuzzy, a focused protocol-world test ≤ 0.5 s, a one-process story ≤ 1 s. `scripts/check.sh` runs fmt, clippy `-D warnings`, nextest, then nextest with the fuzzy profile, with `NEXTEST_DOUBLE_SPAWN=0`.

**5. Lints and crate kinds**

- **Step crates** (all `crates/*`, `testing/temper-fake-forge-domain`, `testing/temper-fake-person`): `[lints] workspace = true`, which is the full subset table at `Cargo.toml:93-176`. The root `clippy.toml` applies to them: banned types, macros and closure methods, with unwrap, expect, panic and indexing allowed in tests.
- **Ordinary-Rust test worlds** (all `tests/*`): their own `[lints]` table. `tests/clippy.toml` applies to them and bans only HashMap, HashSet, Instant, SystemTime and `thread::spawn`.
- **Lookup rule:** clippy uses the clippy.toml nearest above the crate's manifest. So an ordinary-Rust crate outside `tests/` (`crates/temper`, the terminal shell's loop, an ordinary fake under `testing/`) needs its own `clippy.toml`, or it gets the step-code bans.
  - Precedents: jig d91a89c `crates/jig-shell-native/clippy.toml` and `testing/jig-test/clippy.toml` (the tests-style list); skein b5d3a2c `examples/echo/shell/{Cargo.toml,clippy.toml}`, a binary that also bans File, TcpStream, OwnedFd, `process::Command`/`Child`, Instant and SystemTime, and adds `indexing_slicing`, `arithmetic_side_effects` and `as_conversions`.
  - jig's `testing/jig-fake-person` instead sets `disallowed_types/methods/macros = "allow"` in its `[lints.clippy]`.
- **One lints table per crate:** a mixed crate (`protocol.md:225`: the terminal's loop is ordinary Rust, its renderer and reader are step code) can't have both. It is either two crates or the subset enforced by review.
- **`unsafe`:** step crates add `#![forbid(unsafe_code)]` at the root (the workspace table only denies it); world crates forbid it in Cargo and put no `#!` attributes in `lib.rs`.
- **Layout:** unit tests go in `src/tests.rs` (`tests_outside_test_module` is denied); world tests in `tests/*.rs`, fuzzy ones in `tests/fuzzy_*.rs`; world packages are named `temper-<component>[-<child>]-world` (`testing.md:723-743`).

Step crate example (`crates/temper-engine-domain-forge-change`):
```toml
[package]
name = "temper-…"
description = "…"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true
[dependencies]
skein-lib.workspace = true
[lints]
workspace = true
```
```rust
//! What it is (design doc, section).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod boundary;
mod domain;
#[cfg(test)]
mod tests;
```
Ordinary-Rust world example (`tests/engine/domain/Cargo.toml`, same `[package]` block):
```toml
[dependencies]
skein-world.workspace = true
# …
[lints.rust]
unsafe_code = "forbid"
elided_lifetimes_in_paths = "deny"
unused_must_use = "deny"
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
missing_panics_doc = "allow"          # the world asserts its invariants by panicking
wildcard_enum_match_arm = "deny"
match_wildcard_for_single_variants = "deny"
unwrap_used = "deny"
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
disallowed_types = "deny"
disallowed_methods = "deny"
```
Its `lib.rs` is a `//!` doc, then `mod …; pub use …;` with no `#!` attributes (as in `tests/engine/forge/client/src/lib.rs:1-6`). `tests/fake-checkout` also allows `missing_errors_doc`.

**6. Adding a jig crate at the existing pin: not free**

`Cargo.lock` holds jig `d91a89c4…`, skein `b5d3a2c3…` and smith `dfc0fe57…`. I tested on a `git archive` copy in `~/src/rust/tmp/plan/temper-protocol-notes/lockprobe/` with `cargo metadata --offline`:
- **Already in the lock** (`jig-protocol-store`, along with the 26 jig packages listed there): adding it as a direct dependency changes nothing.
- **Not yet in the lock** (any of `jig-domain-client`, `jig-view`, `jig-fake-person`, `jig-shell-native`, `jig-protocol-client-link`, `jig-protocol-client-wire`, `jig-protocol-engine-clients`, `jig-test-client-kit`, `jig-scripted-engine`, `jig-test-view`): the package is added at d91a89c4, but Cargo also re-resolves the packages around it.
  - It moved 11 skein packages to `20682578` and 3 smith packages to `f2c37fe8`, leaving the lock split across two skein revisions.
  - Cargo does this when a new dependency matches nothing in the lock: it unlocks that git source and everything it depends on.
  - Offline, jig stayed at d91a89c4 only because the local git cache has nothing newer. Online it would likely move too: the local `~/src/rust/jig` shows origin/main 8 commits past d91a89c4, and skein 4 past b5d3a2c3.
- **Fix, verified:** after editing the manifests, run `cargo update -p <a skein package> --precise b5d3a2c3363dbbd1db8dae933cd87a643a18b3f2` and `cargo update -p <a smith package> --precise dfc0fe5700c628d7bce7da3d2ee918e9c1fbe46d`, plus the jig one if it moved. Repeat until each git source has exactly one revision. The resulting lock was the original plus the new packages, with nothing removed. The plan should include this step and a check that `grep -o 'forgetic/(jig|skein|smith).git#…' Cargo.lock | sort -u` shows exactly three revisions.
- Removing the four web crates triggers none of this.

**7. STATUS.md rows that name the plan `protocol`**

- `docs/design/STATUS.md:22-24` (prose: "the plan that builds it, `protocol`, is to be written").
- Section 2 table rows:
  - 34 `channel.md`
  - 35 `client.md`
  - 36 `configuration.md`
  - 37 `credentials.md`
  - 39 `forge.md`
  - 40 `git.md`
  - 41 `llm.md`
  - 42 `performance.md`
  - 43 `protocol.md`
  - 44 `testing.md`
  - 51 `domain/engine.md`
  - 52 `domain/forge.md`
  - 53 `domain/people.md`
  - 54 `domain/agent.md`
  - 55 `domain/worker.md`
  - 56 `store/README.md`
  - 57 `web/` ("which `client.md` retires (protocol)")
- Section 3 prose: 61-67 (a failed preparation's repository and reason are "the protocol plan's") and 72-73 (the forge's client documents wait "after the protocol plan").
- The §1 Plans table has only the web-implementation row (line 11); there is no `protocol` row yet.
