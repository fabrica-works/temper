# client-api: jig's client API, and temper on it

2026-10-15. Start commits: jig `a3b53546` (on `28901d98`, "Design jig's
client API: plumbing, sugar, and the application's server"), temper
`c7095516` ("Stand temper's client on jig's client API"), skein
`ecccbac`; each session starts from its repository's `origin/main`.
smith is read only: jig keeps its pin of smith, `29fa97c9`, and temper
moves to it (decisions.md, 4).

jig's client becomes presentation-agnostic: jig's primitives as
plumbing operations, shapes of work as sugar the client fills in, the
server and the URL space the application's. The plan builds the six
steps of jig's `docs/design/STATUS.md`, section 3, "jig's client API":
(1) the client crates renamed by the naming rule, (2) the engine's
plumbing, (3) the wire collapsed, (4) the client, `jig-client`, with
the reference UI on it, (5) the server, (6) the browser. It adds a check
that keeps every client crate under its project's `-client` prefix,
in jig and in temper. temper follows each step that breaks it, builds
what its gate needs of "temper on jig's client API" (temper's
`STATUS.md`), and ends pinned to jig's last landing. One skein increment
gives skein's browser harness what jig's attack stories need.

## Inputs

- **jig's design,** at `a3b53546`: `docs/design/STATUS.md`, "jig's
  client API" (the steps, the renames, "What the code does against the
  design"), and each row naming "(jig's client API)"; the sections each
  session's "Read first" names, in `README.md` (1, 10, 11, 15),
  `client/domain/*.md`, `domain/*.md`, `protocol/*.md` and
  `examples.md`.
- **temper's design,** at `c7095516`: `docs/design/STATUS.md`, "temper
  on jig's client API" and its rows; `client.md`, `protocol.md`,
  `configuration.md`, `credentials.md`, `testing.md`, by the sections
  each temper session names.
- **skein's design:** `ui-web.md`, 14.2 (skein 01).
- **The workflows:** jig's and temper's `docs/development/workflow.md`,
  1 and 2; skein's `AGENTS.md` and `testing.md`, 7.
- **`decisions.md`:** the plan's decisions, and what the user may veto.
- **`findings/`:** the code audit at the base, one file per area
  (`renames.md`, `engine-parties-and-tasks.md`,
  `engine-agents-store-config.md`, `wire-and-client.md`,
  `server-and-browser.md`, `temper.md`), and `measurements.md` (the
  suites at the base).
- **`later.md`:** what the designs name that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"jig 03" means that session's last increment has landed on jig's
`origin/main`.

### jig

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01-renames.md` | jig 01: the client crates renamed, and the naming rule's check | nothing | |
| `jig/02-task-fields.md` | jig 02: a task's label, flags and last activity; `AsAllowed`; the escalation holder; a task's goal | nothing | |
| `jig/03-parties.md` | jig 03: parties' `Create`, `Propose` and `Withdraw`; proposing made explicit | jig 02 | |
| `jig/04-grace-and-compound.md` | jig 04: cancel with a grace, `Move`'s check, tracked and priority amended, goals per project, the compound request | jig 02.1 | 04.3 to 04.7: jig 02.3 |
| `jig/05-me-and-queries.md` | jig 05: waiting on, the task line, task queries, `me`, conversations by their flag | nothing | 05.2, 05.5: jig 02.1 |
| `jig/06-agents-tools.md` | jig 06: the agents' tools name revisions | nothing | 06.3: jig 02.1; 06.4: jig 03; 06.5: jig 03, jig 04.5 |
| `jig/07-connectors.md` | jig 07: connectors' topics and procedures, the party's own scope and windows, untracking | nothing | 07.5: jig 04.5; 07.6: jig 02.3 |
| `jig/08-store-indexes.md` | jig 08: the ended indexes, ended read positions, task queries' ended reads | jig 02.1 | 08.3: jig 03.1; 08.4: jig 02.3; 08.5: jig 05 |
| `jig/09-templates.md` | jig 09: templates and policies' overrides, operations per request | jig 02.1, jig 04.1 | 09.2: jig 02.3; 09.3: jig 05; 09.4: jig 04.6 |
| `jig/10-transport.md` | jig 10: `jig-client-protocol-http`, the native shell on it, the link's vocabulary | jig 01 | |
| `jig/11-client-domain.md` | jig 11: `jig-client-domain`: the contract, handles and askers, the request table, kept work | jig 01 | |
| `jig/12-client.md` | jig 12: `jig-client`: the link's states, `me`, `SignIn { with }`, families as bytes, the thin story against the real engine | jig 10, jig 11 | |
| `jig/13-reference-ui.md` | jig 13: the reference UI on the public client; `ops`'s and the test application's roots by asker; `jig-domain-client` gone | jig 12 | |
| `jig/14-wire.md` | jig 14: the wire collapsed, its requests: the version refusal, scopes and windows, the operations, the compound request | jig 03 to 09, jig 21 (the fence below) | |
| `jig/22-documents.md` | jig 22: the wire's documents: `me`, the task line and queries, offers and a decision's effect | jig 14 | |
| `jig/15-helpers.md` | jig 15: `jig-client`'s helpers: `shapes`, `facts`, `thread`, `words` | jig 22 | |
| `jig/21-ops-and-ui.md` | jig 21: `ops` and the reference UI's state on the client alone; `ops`'s wire crate | jig 13 | |
| `jig/16-shed.md` | jig 16: the shapes of work leave the engine and the wire; the reference UI's lists on task queries | jig 15, jig 21 | |
| `jig/17-engine-http.md` | jig 17: `Mint`, the link's secret, `jig-protocol-engine-clients-http` and its guards | jig 16 | |
| `jig/18-server.md` | jig 18: the test application's server owns the listener, mounts `/jig`, serves its bundle | jig 17 | |
| `jig/19-browser.md` | jig 19: the browser parts, the test shell, the bundle and the browser tier | jig 18 | 19.4, 19.5: skein 01 |
| `jig/20-attack-stories.md` | jig 20: jig's stories for a browser shell, and its attack stories | jig 19 | |
| `jig/23-referee.md` | jig 23: the conformance referee's rules for proposals and revisions | jig 03 | 23.2: jig 06.1, jig 06.2, jig 07.3 |

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-harness.md` | skein 01: the harness opens the printed URL, serves other origins, awaits trouble | nothing | |

### temper

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-skein-ui-repin.md` | temper 01: temper on jig's base with skein-ui; its client crates renamed; the terminal on `skein-ui-text` | nothing | |
| `temper/02-names.md` | temper 02: jig's renames followed, and the naming check | jig 01, temper 01 | |
| `temper/03-client.md` | temper 03: temper's client on `jig-client` | temper 02 | 03.2: jig 10; 03.3: jig 11; 03.4: jig 13.2; 03.5: jig 21.2; 03.6: jig 21.4 |
| `temper/04-engine.md` | temper 04: the engine and the forge's family on templates and the collapsed wire | jig 09, temper 03 | 04.2: jig 22; 04.3: jig 16 |
| `temper/05-server.md` | temper 05: `temper-service` owns the clients' listener and mounts jig's component | jig 17, temper 04 | 05.2: jig 18.1 |
| `temper/06-pages.md` | temper 06: the frame, the inbox and the chats, temper's own | temper 04 (its repin past jig 16 brings jig 15) | |
| `temper/07-task-pages.md` | temper 07: the task's page, adoption, the confirmations; `jig-client-view` dropped | temper 06 | |

The closing repin, `temper-repin` (development.md, 5.7), has no file:
it starts once every jig and skein step has landed and temper 07 has,
and moves temper's pins of jig, skein and smith to jig's (Rules).

31 sessions and the repin: jig 23, skein 1, temper 7. jig 21, 22 and 23
are numbered last because they were added after the others: jig 21 and
22 split from jig 13 and jig 14 (jig 21 runs after jig 13, jig 22 after
jig 14), jig 23 from the design trace's referee rules. The increments
per session are in each file.

### How many at once, and in what order

- **As many sessions at once as dependencies allow, up to eight** (the
  coordinator's Codex profile refuses a ninth thread, development.md,
  7.2), held only by development.md 11.4's memory and disk bounds, which
  the coordinator checks before each start (decisions.md, 16). The build
  lane's width stays as the kit sets it (2 tasks; the gate lane 1;
  development.md, 11.1).
- **At the start,** the seven that need nothing: jig 01, jig 02, jig 05,
  jig 06, jig 07, skein 01, temper 01; the eighth slot goes to the first
  session whose Needs land.
- **Then,** as each lands: jig 10 and jig 11 once jig 01 has; temper 02
  once jig 01 and temper 01 have;
  jig 04 and jig 08 once jig 02.1 has; jig 03 once jig 02 has; jig 09
  once jig 02.1 and 04.1 have; jig 12 once jig 10 and 11 have; jig 13
  once jig 12 has; jig 21 once jig 13 has; jig 14 once jig 03 to 09 and
  jig 21 have; jig 22 once jig 14 has; jig 15, 16, 17, 18, 19, 20 in
  that order, jig 16 once jig 21 has too; jig 23 once jig 03 has; the
  temper sessions as their rows say.
- **The fence:** jig 14 starts only after jig 21's last landing, so that
  no repin of temper 03 (onto jig 10.3, 11.5, 13.2, 21.2, 21.4) takes
  the collapsed wire (decisions.md, 12).
- **When slots are short** (eight running, or 11.4's bounds holding
  starts), **start first** the sessions on the longest chain, then those
  the chain's next session waits on: jig 01, jig 02, jig 11, jig 10, jig
  12, jig 13, jig 21, jig 03, jig 04, jig 06, jig 09, jig 08, jig 05, jig
  07, jig 14, jig 22, jig 15, jig 16, jig 17, jig 18, skein 01 (jig 19
  waits on it), jig 19, jig 20; then jig 23. The temper sessions take a
  slot as soon as their Needs land, before any jig session not on the
  longest chain.
- **The longest chain:** the client track, jig 01 → jig 11 (jig 10
  beside it) → jig 12 → jig 13 → jig 21, then jig 14 → jig 22 → jig 15 →
  jig 16 → jig 17 → jig 18 → jig 19 → jig 20 → `temper-repin`; step 2's
  sessions (jig 02 to 09, the longest jig 02 → jig 03 → jig 06.5, 11
  increments against the client track's 22) run beside the client track
  and land before jig 14.

## Rules of this plan

- **Every session reads,** besides its "Read first": its repository's
  `AGENTS.md` and `docs/development/workflow.md`, this README,
  `code-organization.md`, and the rulings of `decisions.md` that name its
  session. Nothing else is read whole.
- **The code graph:** the codebase-memory projects
  `home-free-src-rust-jig`, `-temper`, `-skein`, as implementer.md, 8.1,
  says: never call `list_projects`, `index_status`,
  `check_index_coverage` or `index_repository`. The graph indexes each
  repository's `main`, never the session's worktree: check each name it
  gives with `rg` in the worktree before relying on it.
- **jig never names temper:** jig's session files say "the repository
  downstream of jig", and no file of jig's code names it: `git -C <the
  session's jig worktree> grep -n -i -w temper -- crates testing tests
  examples scripts` prints nothing at each jig increment's end
  (code-organization.md, 6). jig's documents are not this plan's to
  check.
- **Pins.**
  - **A step's landing** is the commit `git -C ~/src/rust/<repository>
    fetch origin && git -C ~/src/rust/<repository> log origin/main
    --all-match --grep='^Plan: client-api$' --grep='^Step:
    <repository>-NN.k$' --format=%H -1` prints (older plans reuse step
    names such as `jig-02.3`); for a whole session, its last step's.
  - jig moves skein once, never smith: jig 19's first increment that
    needs skein 01 runs `heavy cargo update -p skein-lib --precise <skein
    01.3's landing>`, in a commit of its own; after it, `grep -o
    'skein.git#[0-9a-f]*' Cargo.lock | sort -u` prints that one commit,
    and each later `cargo update` of a skein package holds it with
    `--precise`. A failure in a smith crate is a question.
  - Each temper increment whose Needs name a jig capability starts with
    a repin commit of its own onto the landing of the newest jig step its
    Needs name, never jig's `origin/main`, so that one repin takes as few
    other jig landings as it can. When `git -C ~/src/rust/jig merge-base
    --is-ancestor <that landing> <temper's present jig pin>` succeeds,
    no repin commit is made. Otherwise: `heavy cargo update -p
    jig-domain-core --precise <that landing>`, then `heavy cargo update
    -p skein-lib --precise <jig's pin of skein>` and `heavy cargo update
    -p smith-host-protocol --precise <jig's pin of smith>` (each read with
    `git -C ~/src/rust/jig show <that landing>:Cargo.lock | grep -o
    'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u`).
    Then `grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*'
    Cargo.lock | sort -u` prints three lines: `jig.git#<that landing>`,
    and skein's and smith's lines as jig's lock at that landing prints
    them. A `cargo update` that leaves a second revision of a source is
    run once more; a second revision still there after it is a question.
  - **What a temper repin repairs:** the capability its increment's Needs
    name, by its Shape; and what else jig landed since temper's last
    repin, adapted the way jig adapted its own reference code
    (`jig-client-test-domain`, `jig-test-service`, `ops`), as the commit
    of that change shows, in at most 200 added lines outside the Shape's
    Files: `git diff --numstat origin/main...HEAD -- <the changed paths
    outside the increment's Files> ':!Cargo.lock'`, the first column
    summed once before landing and given in the report. More is a
    question (development.md, 5.7), never a reason to pin a jig older
    than temper's present pin.
  - **jig 08.4's census** (the design answer of 2026-10-16): the first
    temper increment whose repin includes jig 08.4's landing (`git -C
    ~/src/rust/jig merge-base --is-ancestor <jig 08.4's landing> <the
    pinned commit>` succeeds) repairs temper's census walk, which jig's
    reference code does not show (temper's `store/README.md`, section 5;
    jig's `protocol/store.md`, 7.2); at the latest temper 04.2's, since
    jig 14 starts after jig 08. Counted within the repin's 200 lines:
    - `crates/temper-engine-domain/src/store.rs`: `census_interval`
      goes, and its callers (`loads.rs`, `census_position`;
      `crates/temper-service/src/store/ranges.rs`, `cursor`) call jig's
      `Census::interval`; `Range::contains` for `Range::Census` admits
      an ended task, not task 0, and an `EndedRead` key the census
      admits;
    - `restart.rs`, `census_page`: each core row goes to
      `restore_inbox_row` as it comes, a forge row refuses;
    - the root's limits check (`limits.rs`, beside `loads.loads < 2`)
      refuses `loads.rows` below 2, and the small profile
      (`crates/temper-service/src/limits.rs`, `root.loads.rows`) and the
      walking world (`tests/engine/domain/src/walking.rs`) ask 2;
    - tests: `tests/engine/domain/tests/loads.rs`,
      `a_census_page_takes_an_ended_read_position_then_its_task` (the
      real store seeded with an `EndedRead` row, its ended task and a
      `Result` row, paged two rows at a time: the position and its task,
      the cursor the `EndedRead` key; then the result's task, the cursor
      its `Result` key), and `a_census_cursor_behind_its_start_is_out_of_order`
      with an `EndedRead` cursor before a `Result` one; and, beside the
      root world's restart stories (`tests/engine/domain/tests/root_routes.rs`),
      `an_ended_chats_read_position_survives_a_restart` (a chat of two
      turns, the first marked, ends; after the restart its line reads
      one unread turn).
  - **The closing repin, `temper-repin`:** temper's README-declared
    repin. The coordinator adds it at setup, `plan set temper-repin
    repo=temper state=waiting` (development.md 7.1 adds none, since the
    plan lives in temper), and starts it once every other step of the
    plan has landed. Its step 1 moves jig to `origin/main`, and skein and
    smith to jig's pins of them (each read as above), in place of 5.7's
    "each to its `origin/main`".
- **Format versions** (development.md, 5.1): each changes once in this
  plan, at its owner's landing, and is edited in place after:
  - jig's store: tasks v3, owner jig 02.1; people v3, owner jig 02.3;
    core v3, owner jig 06.1. An increment that changes one of these
    families' records needs its owner's landing, and its Needs say so.
    The owner moves the family's schema and goldens to the new version
    (`schema/<family>-v3.schema`, `golden/<family>/v3/`), and the old
    version's go: nothing reads them.
  - jig's client wire: v8, owner jig 11's first increment that changes a
    record (its file names it); jig 10 changes no record.
  - temper's store (`STORE_VERSION`): it moves once, in the first temper
    increment whose repin includes jig 02.3's landing (people v3, whose
    refusal bytes temper's store keeps); that increment's report says so.
    temper's client store and wire: the temper session that first changes
    one names it and bumps it; no other temper session changes it before.
- **The meeting point:** a client increment proves against the real
  engine, in `tests/client-protocol`, the joint world `tests/client`, or
  the process tier, from jig 12 on; never only against
  `jig-client-scripted-engine`. jig 12's thin story (the printed link,
  `me` live, `Stop` answered once durable, kept across a reload) grows
  with each client increment after it.
- **Logs cited as `logs/wN/…`** in the session files are the planner's
  evidence, under `~/src/rust/tmp/plan/client-api/logs/`, outside the
  plan; nothing asks to read them.
- **Edits outside an increment's Files** that are no question (temper's
  protocol plan learned each; they replace implementer.md, 8.2's "it is
  in your session's own files" for these cases only):
  - an arm each exhaustive match needs for a variant the increment adds,
    in any crate, at most 5 lines per match;
  - the users of a type the Shape changes, edited to its new form with no
    other change, every assertion, input, expected value and name kept,
    in any number of lines (code-organization.md, 6);
  - a fix the change, its Proof or the gate needs, at most 20 lines, in
    any session's files, every test's name and expected value kept;
  - an expected literal that prints a vocabulary the Shape renames, at
    most 10 lines;
  - a removed mechanism's tests, moved to its replacement or removed;
  - code that moved since the plan was written, followed to its file.

  These limits are guides: each landing's report lists its lines outside
  its Files (`git diff --numstat origin/main...HEAD -- <those paths>`),
  and a limit passed is a question before landing.
- **The browser suite is a suite of its own** in jig, from jig 19:
  `--profile browser`, outside the focused and fuzzy suites, Chromium
  only there; its budget is 60 seconds (`global-timeout`).
- **A browser story's Proof** is `land`'s gate: the implementer runs no
  gate of its own for it, and runs a failing story alone with the
  commands of "One test of the browser suite", below.
- **A Proof's "Bound: N s"** is an aim for the test's time as nextest
  prints it, building aside; a bound missed is an aim missed: the report
  says so, and the landing stands.
- **Test time** (findings/measurements.md):
  - jig: focused median 10.33 s, fuzzy 14.91 s. The run may add, as
    aims, at most 3.0 s to the focused suite (to about 13.3 s of its
    15) and 10 s to the fuzzy suite.
  - temper: focused median 4.00 s, fuzzy 14.36 s. The run may add at
    most 2.0 s to the focused suite and 5 s to the fuzzy suite.
  - Each landing's report names its suites' times from its
    `Gate-Result` line, and nothing else is measured. An aim exceeded is
    a guide missed: the landing stands, and the session's next increment
    first makes the slowest tests cheaper, found once by the measure run
    of the suite that missed (`.config/nextest.toml`'s two commands):
    the focused suite's `heavy --gate -- cargo nextest run --workspace
    --profile measure -j 1`, the fuzzy suite's `heavy --gate -- cargo
    nextest run --workspace --profile measure -j 1 --ignore-default-filter
    -E 'binary(/^fuzzy_/)'` (ruling 109), until its own gate is back
    under the aim; after a session's last increment the
    supervisor decides. Never: raising a `global-timeout`, a slow period
    or a per-test ceiling, or moving a focused test to the fuzzy suite.
  - A test's own time in a session file ("under 20 ms", "under 0.5 s")
    is an aim, development.md 5.6's ceiling: nothing measures one test,
    and a test the gate's log flags `SLOW` is made cheaper before its
    increment lands.
- **Shared files,** which parallel sessions may both change: jig's
  workspace `Cargo.toml` and `Cargo.lock` (jig 01, 10, 11, 12, 16, 17,
  19); `tests/boundary/src/tests/graph.rs` (jig 01, 10 to 13, 16, 17,
  19); `crates/jig-domain-core/src/{lib.rs, documents.rs,
  routing/mod.rs}` (jig 02 to 09, 14, 16);
  `crates/jig-domain-people/src/boundary.rs` and
  `crates/jig-domain-tasks/src/boundary.rs` (jig 02 to 09, 16);
  `crates/jig-protocol-store/src/{format.rs, keys/, codec/}` (jig 02,
  03, 06, 08, 09, 16); `crates/jig-protocol-config/src/` (jig 03, 04,
  09, 17, 18), and in it `tests/domain.rs`'s
  `memory_below_the_components_sum_is_refused` (`over`, the small
  profile's memory sum) by each session that changes a stored record's or
  a component's worst case (jig 02 to 09, 16; ruling 110); `crates/jig-protocol-engine-clients/src/translate/` (jig 14, 16,
  17); `testing/jig-test-domain/` (jig 06, 07, 09);
  `testing/jig-conformance/` (jig 04.7, 06.1, 08.3 to 08.6, 23);
  `crates/jig-client-protocol-link/src/{boundary,component,store,lib}.rs`,
  `crates/jig-client-shell-native/src/` and
  `tests/client-protocol/src/{front,front/persistence,native,fixtures}.rs`
  (jig 10, 11, 12); `tests/boundary/src/tests.rs` (jig 01, 10, 13);
  `crates/jig-client-view/src/` and `examples/ops/client-*` (jig 13, 14,
  15, 16, 21, 22); `crates/jig-protocol-store/src/historical.rs` and the
  roots' `Now` routes (`examples/ops/domain/src/route.rs`,
  `testing/jig-test-service/src/{route.rs, clients/translation.rs}`,
  `tests/client-protocol/src/engine.rs`) (jig 06.5, 07.2); temper's
  `crates/temper-engine-domain/src/store.rs` (temper 02 to 04), temper's
  `Cargo.toml` and `Cargo.lock` (temper 05.1, 06.1, 07.4) and
  `tests/one-process/src/` (temper 05.2, 06.1, 07.4). The session that
  lands second rebases onto the first and follows its change.
- **Files over 800 lines at the base** that an increment changes grow
  by at most 40 lines each, a rule with its count in code-organization.md
  7; their splits are `later.md`'s.

## Prerequisites the run checks at its start

**The run's machine is carbon-forge** (decisions.md, 13): it has the
`wasm32-unknown-unknown` target, which skein's gate and jig 19 onward
need. x1, where the plan was written, has none, so the planner could not
run these at the base; the supervisor's ready check runs them on
carbon-forge when their session becomes ready (development.md, 5.4,
last paragraph): skein 01's and jig 19's and 20's "One test" commands
below, jig 19.4's wasm32 builds and clippy, and the browser suite's
`cargo nextest list` with `--profile browser`.

The coordinator runs each check once, before the first session, and
writes `blocked.md` (kind: process) on a failure:

1. `cargo nextest --version` prints a version.
2. `ls "$(rustc --print sysroot)/lib/rustlib"` lists
   `wasm32-unknown-unknown` (skein's gate; jig 19's wasm32 builds).
3. `/usr/bin/chromium --version` prints `Chromium 1`, or
   `SKEIN_TEST_CHROMIUM` names a file (jig's and skein's browser suites).

## One test of the browser suite

A browser story that fails is run alone, in the session's worktree,
through the build lane: after each fix aimed at a cause its log names,
the two commands below run once each; what counts is the second's
`Summary` line, 1 test run, 1 passed. The same symptom a third time is a
question (development.md, 5.5). Nothing else is run or kept. `target/`
is the worktree's own. The session files point here; this is the one
copy of the commands.

**skein's browser suite** (skein 01), as skein's `scripts/check.sh`, its
last block, builds it:

```sh
heavy sh -c 'cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web && cargo run -p skein-ui-bundle -- --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle --noscript "Notes needs WebAssembly and scripts." && cargo build -p skein-ui-example-server'
heavy sh -c 'SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle" cargo nextest run -p skein-ui-web-tests --profile browser -E "test(<name>)"'
```

**jig's browser suite** (from jig 19.4 on; the bundler's arguments are
those jig 19.4 writes into jig's `scripts/check.sh`; where the two
differ once landed, `scripts/check.sh` wins):

```sh
heavy sh -c 'cargo build --target wasm32-unknown-unknown -p jig-client-test-shell-web && cargo build -p skein-ui-bundle && target/debug/skein-ui-bundle --module target/wasm32-unknown-unknown/debug/jig_client_test_shell_web.wasm --name jig_client_test_shell_web --prefix /bundle/ --stylesheet crates/jig-client-shell-web/jig.css --stylesheet testing/jig-client-test-shell-web/jig_client_test_shell_web.css --pictures jig=crates/jig-client-shell-web/pictures --pictures jig-test=testing/jig-client-test-shell-web/pictures --files testing/jig-client-test-shell-web/files --noscript "jig needs WebAssembly and scripts." --out target/jig-test-bundle && cargo build -p jig-test'
heavy sh -c 'JIG_TEST_BUNDLE="$(pwd)/target/jig-test-bundle" cargo nextest run -p jig-client-browser-world --profile browser -E "test(<name>)"'
```

`cargo run -p skein-ui-bundle` does not resolve in jig's workspace; the
bundle's path is absolute, since nextest runs each test from its
package's directory.

## Live services

None. The browser tier runs a local Chromium against jig's test
application on loopback; no agent signs in anywhere.

## Repin

temper is repinned by its sessions as they follow jig, and by the
closing `temper-repin` (Rules, Pins). smith is read only, and nothing
downstream of temper exists.

## Pulse

```
words: legacy|compat|hack|workaround|actual|genuine
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

Each command runs once, after `done.md`, reading `origin/main`.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. jig's old client crates are gone: `git -C ~/src/rust/jig ls-tree -d
   --name-only origin/main crates/jig-protocol-client-wire
   crates/jig-protocol-client-link crates/jig-shell-native crates/jig-view
   crates/jig-domain-client testing/jig-scripted-engine
   testing/jig-test-domain-client testing/jig-test-view
   testing/jig-test-client-kit testing/jig-test-client-records
   testing/jig-test-protocol-client-wire examples/ops/domain-client
   examples/ops/view` prints nothing.
3. jig's new client crates exist: `git -C ~/src/rust/jig ls-tree -d
   --name-only origin/main crates/jig-client crates/jig-client-domain
   crates/jig-client-protocol-http crates/jig-client-view
   crates/jig-client-shell-web crates/jig-protocol-engine-clients-http
   testing/jig-client-test-shell-web
   testing/jig-client-fake-person-browser tests/client-browser
   examples/ops/client-protocol-wire | wc -l` prints 10.
4. The naming checks hold and wait for nothing: `git -C ~/src/rust/jig
   grep -n 'PENDING' origin/main -- tests/boundary/src` prints a line
   with `[]`, and in a worktree of each at `origin/main`, `heavy cargo
   nextest run -p jig-boundary -E 'test(client_crates_carry_the_client_prefix)'`
   (jig) and `heavy cargo nextest run -p temper-client-world -E
   'test(client_crates_carry_the_client_prefix)'` (temper) each pass one
   test.
5. No shape of work on the wire: `git -C ~/src/rust/jig grep -n -E
   '^(record|enum) (StartChat|SetGoal|Prioritise|Conversations|ConversationLine|GoalLine|Goals|Place|Draft)\b'
   origin/main -- crates/jig-client-protocol-wire/schema` prints nothing.
6. jig's component owns no listener: `git -C ~/src/rust/jig grep -n -w
   -e Listen -e skein_http origin/main --
   crates/jig-protocol-engine-clients/src` prints nothing.
7. jig's gate has the browser suite and the wasm32 builds: `git -C
   ~/src/rust/jig grep -n -e wasm32-unknown-unknown -e 'profile browser'
   origin/main -- scripts/check.sh` prints a line for each.
8. temper's client is its own: `git -C ~/src/rust/temper grep -n -e
   jig-client-view -e jig-domain-client -e jig-view origin/main --
   '*Cargo.toml'` prints nothing.
9. temper pins jig's last landing of this plan: `git -C ~/src/rust/temper
   show origin/main:Cargo.lock | grep -o 'jig.git#[0-9a-f]*' | sort -u`
   prints one commit, which `git -C ~/src/rust/jig log origin/main
   --grep='^Plan: client-api$' --format=%H -1` prints, or a later one.
10. The plan's commits: `git -C ~/src/rust/<repository> log origin/main
    --grep='^Plan: client-api$' --format='%(trailers:key=Step,valueonly,separator=)' | sort -u`
    lists each planned step: in jig the 107 steps `jig-01.1` to
    `jig-23.2`, in skein `skein-01.1` to `skein-01.3`, in temper the 26
    steps `temper-01.1` to `temper-07.4` and `temper-repin`. A supplementary
    step (`.s<k>`) adds a line; a step dropped by a ruling is named in
    decisions.md's rulings instead.
11. The test-time aims, measured once in a worktree of jig and of temper
    at `origin/main`. jig: `heavy --gate -- sh -c 'export
    NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run
    --workspace 2>&1 | grep Summary; cargo nextest run --workspace
    --profile fuzzy 2>&1 | grep Summary; done'`; temper: the same
    without the `export`. The medians of the three `Summary` times per
    suite, against the aims above. An aim missed is recorded with the three slowest tests by name
    in decisions.md's rulings, for the user; the run is done either way.
12. No budget was raised: `git -C ~/src/rust/jig diff a3b53546 origin/main
    -- .config/nextest.toml | grep -E '^[-+].*(global-timeout|period)'`
    prints only the browser profile's added lines, and `git -C
    ~/src/rust/temper diff c7095516 origin/main -- .config/nextest.toml`
    prints nothing.
13. Each store family moved once: `git -C ~/src/rust/jig show
    origin/main:crates/jig-protocol-store/src/format.rs | grep -o -E
    'core: 3|tasks: 3|people: 3' | wc -l` prints 3 (at the base, the same
    command with `2` prints 3).
14. One definition of the client wire's version:
    `git -C ~/src/rust/jig grep -n -E '(CLIENT_VERSION|VERSION): u32 = [0-9]'
    origin/main -- crates` prints one line, in
    `crates/jig-client-protocol-wire/src/lib.rs` (code-organization.md,
    4; at the base it prints two).
15. The client's transitional pieces are gone (code-organization.md,
    6): these two commands print nothing.

    ```sh
    git -C ~/src/rust/jig ls-tree --name-only origin/main -- crates/jig-domain-client crates/jig-client-view/src/bridge.rs crates/jig-client-view/src/watches.rs crates/jig-client-view/src/reads.rs crates/jig-client-view/src/link.rs crates/jig-client-view/src/versions.rs
    git -C ~/src/rust/jig grep -n -E -e 'FrameIs|ByValue|RequestTable' -e 'Kept \{ scope' -e 'fn watch_event' -e 'Event::Frame' origin/main -- crates/jig-client-view/src testing/jig-client-test-domain/src testing/jig-client-test-view/src examples/ops/client-domain/src examples/ops/client-view/src
    ```
16. The aims stated above are these items; none other is checked.
