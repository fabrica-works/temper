# client-api: decisions

The decisions this plan rests on, numbered; the supervisor's rulings
during the run follow them, from 101; the choices left to the user close
the file. The user delegated every decision of this plan (2026-10-15):
"there are no important open questions, so there should be no need for
my input, take reasonable decisions when in doubt." The planner took
each below under that delegation, the most reversible option where the
choice would have been the user's (development.md, 10.4).

## The plan's own

1. **Scope.**
   - **(a) jig's six steps in full** (jig's `STATUS.md`, section 3,
     "jig's client API"), the design's "Left" for each row naming "(jig's
     client API)", but what those rows themselves leave "not planned"
     (`later.md`).
   - **(b) The client naming rule as a check** that holds after the run:
     the user's rule ("Client crates (wherever they live, in jig or temper
     or possibly even smith) should be named
     `<project>-client-{domain,protocol,<whatever>}-<...>`"), stated in
     jig's `README.md`, 15 and temper's `client.md`, 2;
     code-organization.md, 4 gives the check, landed by jig 01.4 and
     temper 02.1.
   - **(c) temper, as far as its gate needs:** each jig change that
     breaks temper is followed in temper, and temper builds the rows of
     "temper on jig's client API" without which it does not compile or
     pass its gate on the new jig (findings/temper.md, 5): its crates
     renamed; its client root on `jig-client` with two owners, the
     forge's request as bytes and `Owner` in its wire crate; its terminal
     on `skein-ui-text` over `jig-client-shell-native`, the credential
     kept by its host and given back as `Held`; `templates` in its
     configuration; the forge's family on the collapsed wire;
     `temper-service` owning the clients' listener and mounting jig's
     component under `/jig`; `application.clients`; the pages its
     stories use (Inbox, Chats, Chat, the task's page, the adoption form)
     after the mockups, with their confirmations; `tests/client` on
     `jig-client-world` with jig's check; and the three rows skein-ui's
     `later.md` left for temper's next repin (temper 01).
   - **Left for temper's next plan** (`later.md`): the board and
     reordering goals, the changes page, an agent's task page, the
     confirmations no present story makes, the rebound-Host attack story,
     temper's browser, the forge as a sign-in provider through `Mint`.
   - **Why:** the user's default for this plan ("jig's six steps in full,
     and the repin of the repositories downstream of what the plan
     changes, with the repairs it breaks"), and a temper on jig's last
     landing with its gate green, rather than a temper left 192 commits
     behind (development.md, 5.7). temper's present pages are jig's
     reference UI, which temper's design excludes (temper's `client.md`,
     1 and 9), so the pages its stories use are built here; the others
     add nothing its gate checks.
2. **The plan lives in temper** (development.md, 3.4): it changes jig,
   skein and temper, and temper's changes are sessions of their own, not
   a repin. jig's `STATUS.md` gets a row for it that names no temper
   (jig's `AGENTS.md`; development.md, 3.1): "client-api, of the
   repository downstream of jig". jig's session files, though they live
   in temper's repository, never name temper either (README, Rules), so
   that nothing of temper's reaches jig's files.
3. **The order of jig's steps.** The design gives six steps "in order";
   the plan keeps that order where a step needs the one before, and runs
   in parallel what does not (development.md, 5.1, "Parallel"):
   - step 1 first, alone among the client crates (it moves them);
   - step 2's sessions (jig 02 to 09) beside the client's restructuring
     (jig 10 to 13: the HTTP half, `jig-client-domain`, `jig-client`, the
     reference UI on it), which the code audit found independent of
     steps 2 and 3 (findings/wire-and-client.md, 6);
   - step 3, the wire collapsed (jig 14), after both, so that it edits
     the new client crates once rather than `jig-domain-client`'s pages
     before they move; the shapes of work leave the engine, the wire and
     the reference UI together in jig 16, after the helpers that replace
     them (jig 15), so that no temporary translation is written
     (findings/engine-parties-and-tasks.md, 4);
   - step 5 after step 3 (both rewrite the clients' component), step 6
     last.
4. **smith is read only.** jig keeps its pin of smith, `29fa97c9`; no
   step of this plan needs smith, and smith's parity run is active on
   another machine. temper moves its smith pin to jig's at its first
   repin (temper 01), so that one revision of smith serves temper and
   jig's crates.
5. **skein changes once:** skein 01, the harness's printed URL, other
   origins and awaited trouble, which jig's attack stories need and
   skein's harness cannot do at the base
   (findings/server-and-browser.md, 4). It is generic, names no jig, and
   lands before jig 19 repins skein onto it.
6. **temper follows jig step by step.** Each temper increment whose
   Needs name a jig capability repins, at its start, onto the landing of
   the newest jig step its Needs name (never jig's `origin/main`), and
   repairs what jig's landings since its last repin broke,
   adapted the way jig adapted its own reference code (README, Rules).
   temper's client root and native client are copies of jig's reference
   ones (findings/temper.md, 3), so following jig's own diffs is the
   cheapest correct port. A repin that needs more than 200 lines of
   repairs outside its Shape is a question.
7. **temper's view on jig's reference UI, until temper 07, is
   transitional.** temper's view arranges jig's reference pages today
   (`jig-view`, which temper 02 follows to `jig-client-view`); when those
   pages move into `jig-client-view` (jig 13), temper's view follows them
   there, and temper 06 and 07 replace them with
   temper's own pages; temper 07 drops the dependency
   (code-organization.md, 6). This keeps each temper increment within
   its Size; the alternative, temper's pages before jig's reference UI
   moves, would rebuild them on `jig-domain-client`, which goes.
8. **One check for the naming rule, in jig, used by temper:**
   `jig_boundary::client_names::breaches` (jig 01.4), called by jig's
   boundary test and by temper's (temper 02.1). The rule is mechanical
   (code-organization.md, 4) and was tried at the base: after step 1 it
   flags only `jig-domain-client` in jig, which jig 13 removes; in temper
   it flags `temper-shell-terminal` and `temper-protocol-client-wire`
   until temper 01 renames them (findings/renames.md, 5). smith has no
   client crate: its check waits for one (`later.md`).
9. **Format versions, once each** (development.md, 5.1): jig's store
   families tasks, people and core move to v3 at their owners' landings
   (README, Rules); the client wire moves to v8 once, in jig 11; nothing
   is translated, and readers refuse other versions.
10. **No live service.** The browser tier runs a local Chromium on
    loopback, as skein's does.
11. **The goal of a task.** STATUS's paragraph says the goal is computed
    "as the nearest tracked ancestor in another" document; the audit
    found that computation keeps the topmost tracked task
    (`crates/jig-domain-core/src/documents/task_tree.rs:653-667`). Either
    way the design is one rule, "the nearest tracked task at or above
    it" (`domain/core.md`, 6.2), which jig 02 builds once for every
    document.

12. **The fence: jig 14 starts after jig 21** (taken 2026-10-16 by the
    planner's successor, with 13 and 14 below). temper 03's last
    increments repin onto jig 21.2's and 21.4's landings; a jig 14 landed
    before them would bring the collapsed wire into those repins, past
    their 200 lines of repairs (the consistency review's item 10). With
    the fence, the client track (jig 01 → 11 → 12 → 13 → 21, 22
    increments) becomes the longest chain, about 11 increments longer than
    step 2's (jig 02 → 03 → 06.5, 11). Since jig 14 waits for jig 13
    either way (it edits the new client crates), the fence costs jig 21's
    four increments on the longest chain, against a temper 03 that would
    otherwise take on temper 04's repairs. jig 14's Needs at start are
    "jig 03 to 09, jig 21", whole sessions, since each of its increments
    needs "the session's start"; the review's per-increment Needs for
    jig 14 were not taken.
13. **The run's machine is carbon-forge.** The user said (2026-10-16):
    "the run will happen on carbon-forge, it has wasm32." x1, where the
    plan was written, has no `wasm32-unknown-unknown` target and no
    rustup; nothing was installed on it. The commands that need the
    target (skein's gate and browser suite, jig 19 onward) were not run
    at the base; the README lists them for the supervisor's ready check
    on carbon-forge, and its Prerequisites check the target there.
14. **jig 04 starts after jig 02.1, and 04.6 needs no jig 03.** 04.1
    and 04.2 need only the tasks family at version 3 (jig 02.1); 04.3 to
    04.7 need jig 02.3 (the escalation holder, the people family at
    version 3). A compound holds no `Create`, `Propose`, `Withdraw` or
    `Decide` (04.6's Don't), so the README's former "04.6: jig 03" is
    dropped, and jig 08 starts after jig 02.1 likewise.

15. **Two items of step 5 wait** (`later.md`; jig's `STATUS.md` marks
    them not planned): a client in the engine's process, by an
    in-process transport (`protocol/service.md`, 10), since no
    application of this plan runs a client in its engine's process and
    nothing would prove it beyond a test-only client; and the clients'
    listener off loopback with its certificate (service.md, 7;
    browser.md, 12.2), which needs a TLS server side skein does not
    have. Both add to what this plan builds without changing it.

16. **Sessions at once: up to eight, by the machine's own capacity.**
    The user set it (2026-10-16): "The number of concurrent sessions on
    carbon-forge should be capped only by the machine's own capacity (I
    think 7/8 is sustainable, or at least iirc it was in the past)." So
    the run starts as many sessions as dependencies allow, up to eight,
    the coordinator's Codex profile refusing a ninth thread
    (development.md, 7.2), and held only by development.md 11.4's memory
    and disk bounds, which the coordinator checks before each start. The
    build lane's width stays as the kit sets it (2 tasks, the gate lane
    1). Nothing else in the plan rests on the number: the test-time aims
    are measured by the gate lane, which runs alone. This replaces the
    planner's earlier four.
17. **Sessions past five increments.** jig 04 holds seven increments,
    and jig 07, 08, 15 and temper 03 and 06 six each, past development.md 5.3's four or
    five, after the reviews' items were added (jig 04.7, 07.6, 08.6,
    15.6). They stay one session each: a split would renumber other
    sessions' Needs at the end of planning, and their implementers hand
    over within the session as any session's do (development.md, 7.3).
    The referee's two rules the trace found unbuilt went to a session of
    their own, jig 23, off the longest chain.

## Rulings

101. **temper 02's Shapes against main** (ready check of temper-02.1).
     temper 02's session file, checked against `main` at skein `15ddb754`, jig `375545d7`, temper `385219e9`: two references corrected, no code.
     - "Read first" and "What the design asks" quoted a "temper on jig's client API" row in temper's `STATUS.md`, section 3, which the plan's landing removed. They now quote section 2's `client.md` row, `docs/design/STATUS.md:40`: "the crates `temper-protocol-client-wire` renamed `temper-client-protocol-wire` and `temper-shell-terminal` `temper-client-shell-terminal`, by the client naming rule".
     - Files named `tests/support/mod.rs`; temper has no root `tests/support/`. It is `crates/temper-client-shell-terminal/tests/support/mod.rs` (today `crates/temper-shell-terminal/tests/support/mod.rs`).
     Unchanged: the rest of the file. The eight further files naming `jig_view` (`crates/temper-client-view/src/tests.rs`, the terminal's `src/command.rs`, `src/render.rs`, `src/tests/command.rs`, `src/tests/render.rs`, `tests/client/tests/{adoption,stories}.rs`, `tests/one-process/src/stories.rs`) are temper 01.1's and 01.3's to move; a further copy temper 02.1 finds naming a renamed jig crate, it refreshes and lists in its commit body.

102. **temper 01.1's dropped State::Provisional golden** (plan defect, temper 01's block of 2026-10-16T08:09:41Z).
     Ruling 102: temper 01.1 deletes `State::Provisional` from the `every_role` fixture (`crates/temper-shell-terminal/src/tests/fixtures.rs:504`) and the final ` (provisional)` from `src/tests/golden/every_role.txt:48`, with the printer's line (`render.rs:486`). The session's step 7 says "`(State::Provisional, b" (provisional)")` goes", its Removes names "`State::Provisional`'s line", and temper's `STATUS.md` names "`State::Provisional` dropped": skein-ui at `f12f29d4` has no such state, so the fixture cannot keep it and the golden line follows. This is a plan defect: the golden rule named jig's pages alone. The rule now reads: a golden line that changes for a reason other than jig's pages or the dropped `State::Provisional` is a bug, and a question with the line. Name the changed line in the landing's report with the others. Unchanged: every other golden changes for jig's pages alone; the printer's other repairs stay the role arms. A further copy of the state found while repairing (none on `main` but these three), delete it and list it in the commit body.

103. **jig 04's Shapes against main** (ready check of jig-04.1).
     jig 04's session file, checked against `main` at jig `375545d78` and re-read at `3f3fd7ae0`, temper `385219e9`; corrected in `jig/04-grace-and-compound.md`:
     - 04.1's Files gain the store codec's grace (`codec/tasks/phase.rs:406-414`, its test `:229-232`), `charters::chats`' caller (`jig-protocol-config/src/lib.rs:96-97`), `closing_chat.rs:173,217` (`Grace` no longer `Copy`), the `SMALL` table's rows (`tests/overrides.rs:131`: `tasks.grace_ms` 600000 at 04.1, `tasks.project_goals` 8 at 04.2, `people.operations` 8 at 04.6); the override key is `b"grace_ms"`.
     - `task_problem` (`jig-protocol-tools/src/render.rs:212`) and `task_answer` (`jig-domain-core/src/routing/refusals.rs:23`) match every `tasks::Refusal`: 04.1 and 04.2 give each a `Grace` and a `Goals` arm. Until 04.5 adds people v3's `LimitKind::{Grace, Goals}`, `task_answer` maps `Grace` to `LimitKind::Spec` (no person's ask carries a grace before 04.5) and `Goals` to `LimitKind::ProjectLiveTasks`; a 04.2 test of a person's refused goal expects `ProjectLiveTasks`, and 04.5 changes it to `Goals`.
     - 04.2's four `people::Amendment` files move to 04.5; 04.5 lists `people::Ask::Cancel`'s real users (12 literals and `asks.rs:402`, in 10 files); 04.6 gains `run.rs:382` (`PersonTaskRoute`); 04.7 gains jig's `tests/conformance/src/lib.rs:215` and temper's `scenarios.rs:512`.
     - Downstream counts (temper): 2 tasks `Amendment` literals, 3 people ones, 2 `LimitKind` matches and `refusal.rs:251`'s decode; `answers.rs:351` added to 04.1's and 04.2's downstream lines.
     - References: `#[path = "lifecycle/grace.rs"] mod grace;` (and `compound.rs` likewise, as `lifecycle.rs:774` does); `Harness` at `tests/revisions.rs:94`; `cohort` at `chat.rs:6`; the sketches' `expect` gains `reason`, and the no_std crates `use alloc::boxed::Box;`.
     A further user of a changed type or match that the Files miss, the implementer edits and lists in the commit body.

104. **jig 10's Shapes against main** (ready check of jig-10.1).
     jig 10's session file, checked against `main` at skein `15ddb754`, jig `375545d78`, re-read at `3f3fd7ae0` (jig 01.1's renames); corrected in `jig/10-transport.md`:
     - 10.1's Files gain `front/persistence.rs:100`, jig-test's sign-in at `tests/support/mod.rs:169`, the link's `src/tests.rs` (`mod sign_in;`) and the fixtures whose matches `Request::Signing` breaks; downstream, temper's `tests/support/mod.rs:128` and `src/tests.rs:146`.
     - 10.1's Shape: `Shell::loaded()` returns `Option<(Option<Secret>, Vec<StoreRead>)>` (`shell.rs:85`; its user `native.rs:301`). `Shell::sign_in` keeps today's address check and `Host` (`connections.rs:82-85`, `:158`): headers are the transport's (protocol/client.md, 4.3: "There is no method, path or header here: those are the transport's").
     - 10.2's Files gain the users of `Exchange { path, body }`, `Abandon`, `Answered`/`Failed` and the shell's `Limits.deadline` the report listed, in jig and in temper (`limits.rs:71,100`, `client.rs:263`); `Harness::answer` (`fixtures.rs:120`) loses `status`, with its 35 callers; `Shell::next_deadline` (`shell.rs:327`) includes each open exchange's deadline.
     - The sketch derives `Debug, PartialEq, Eq` (`Stream` `Debug` alone) and defines `Heard`; the 10.2 reference test uses a slice `contains` of the whole `skein_http::Header` (`clippy.toml:145`); `client_step` at `graph.rs:138`; Done when greps only `crates/jig-client-protocol-link` (jig 11 adds `jig-client-domain`, jig 12 `jig-client`).
     A further user of a changed type or match that the Files miss, the implementer edits and lists in the commit body.

105. **jig 08's Shapes against main** (ready check of jig-08.1).
     jig 08's session file, checked against `main` at jig `375545d78`, re-read at `3f3fd7ae0`, temper `8d6b6f68`; corrected in `jig/08-store-indexes.md`:
     - 8.1, 8.2: `output_bound`'s count of an ending's saves (`jig-domain-tasks/src/domain.rs:451`, 26 today) becomes 27 at 8.1 and 28 at 8.2; `domain.rs` in both Files. 8.2's read story ends the first chat before asking `goal(100)`, as `restart.rs:483` does.
     - 8.3: `World.initial_ask` and `World::configured_with(.., initial)`, read at `tests/core/src/effects/deliveries.rs:153-158`, with the three effects files in Files; people `domain.rs:429` (`Event::ForgetChat`); the forget's queue capacity 8 (`inbox.rs:415`); `chosen_cut` at `scenarios.rs:18`; an arm per new scenario in `tests/conformance/src/lib.rs:216-252`; downstream, temper's `tests/conformance/src/config.rs:132-138` at 8.3 and 8.5.
     - 8.4: `Census::admits` at `restart/census.rs:16`; `census_owner` (`tests/ops/src/conformance/mod.rs:428, 1069-1076`) hands up the `EndedRead` row, then its ended record; `census_ended` (`tests/core/src/effects/store.rs:50-52`) counts only `Stored::Ended` rows, so `restart.rs:497` keeps 3. Downstream: temper's census walk (`temper-engine-domain/src/store.rs:281-296`, `loads.rs:366`, `temper-service/src/store/ranges.rs:14`) fails silently through the `Core::People(_)` wildcard; what temper's root does with an `EndedRead` row is a question for the designer (temper's `store/README.md`, 5), and that downstream line waits on its answer. jig 08's own increments do not.
     - 8.6, downstream: temper's `tests/conformance/src/observations.rs` and `tests/one-process/src/referee.rs` fill no `read_positions`, so the rule checks nothing there; nothing breaks.
     - Sketches list the phases (`Waiting | Active(_) | Held { .. } | Closing(_)`, no wildcard, no `matches!`) and carry `Update`'s derives (`index.rs:34`); `tasks_key` at `fixtures/keys.rs:100`, its `below(19)` (:102) 20 then 21; `people_key`'s `below(7)` (:60) 8.
     A further user of a changed type, match or count that the Files miss, the implementer edits and lists in the commit body.

106. **temper's census and jig's EndedRead rows** (design answer, ready check of jig-08.1).
     The designer's answer to the census question of jig-08.1's ready check (temper's census and jig's `EndedRead` rows), landed as jig `9a24ded8a` and temper `97315175`:

     1. **temper's root takes the `EndedRead` row as a row of the page, checks it and keeps nothing.** temper's `store/README.md`, 5, now: "An ended conversation comes as two rows, its ended read position (`EndedRead`) and then its ended task; an unread result as one". The root checks "each row is an ended task or an ended read position, and the cursor a key the census admits, past the page's start in the walk's order". It "neither pairs, orders nor dedupes the rows: it hands each to jig's core as it comes", and "the core holds an ended read position until its task's record". Pairing is jig's core's check (08.4), not the root's.
     2. **The cursor** is "the key of the last index row walked: an `EndedRead` key, which is a row of the page, or a `Result` key, which is not".
     3. **The interval:** `EndedRead` (people tag 7, `[JIG, 2, 7]`) is interval 0, where `ConversationOldest` was; then 1 + k for the census's k-th party's results. temper no longer computes it. jig's 7.2: "The census places each key it admits in the walk's order (`Census::interval`), so that a root that keeps its own cursors finds the interval to go on from". `census_interval` and its `Core::People(_)` arm go. 08.4 builds `Census::interval`, and `admits` is its `is_some`.
     4. **Pages hold whole pairs, at least two rows.** jig's 7.2: the census "never parts a position from its record: a page counts both against the asker's rows and bytes, and ends before a pair that does not fit, so a root asks the census for at least two rows". temper asks restart pages one row at a time today, so every census page that met an ended conversation would be cut, and the restart stopped:
        `crates/temper-service/src/limits.rs:204: rows: 1,`
        `tests/engine/domain/src/walking.rs:134: loads: loads::Limits { loads: 2, rows: 1, ...`
        (`git -C temper grep -n 'rows: 1,' origin/main -- crates/temper-service/src/limits.rs tests/engine/domain/src/walking.rs`). jig's test store asks 2 (`testing/jig-test-store/src/route.rs:125`). The repair raises both to 2, and temper's limits check refuses less.
     5. **Each ended task once per party** (jig's 7.2: "neither part's progress skips a row of the other"). At the base, `crates/jig-domain-core/src/inbox/census.rs:116` (`let mark = if row.result_position > after { &mut progress.unread_result ...`) has one unread mark per party. An ended chat's unread result, met in the first part, skips every lower unread result of the same party in the second. 08.4 fixes it, with `a_restart_keeps_an_unread_goal_older_than_an_unread_chat`. 08.4 also refuses a record after a pending position that is not its task, and a census that ends with a position pending. Size goes from 300 to 340 lines.
     6. **Who repairs temper:** "the first temper increment whose repin includes jig 08.4's landing (`git -C ~/src/rust/jig merge-base --is-ancestor <jig 08.4's landing> <the pinned commit>` succeeds) repairs temper's census walk ... at the latest temper 04.2's, since jig 14 starts after jig 08". It counts within the repin's 200 lines. The plan's README, "jig 08.4's census", lists:
        - the files: `store.rs` (`census_interval` goes; `Range::contains`), `loads.rs`, `ranges.rs`, `restart.rs` (`census_page`), `limits.rs`, the small profile, the walking world;
        - the tests: `a_census_page_takes_an_ended_read_position_then_its_task`, the out-of-order test with an `EndedRead` cursor, and `an_ended_chats_read_position_survives_a_restart`.
        temper 03 and 04 each point to it.
     7. Nothing is deferred: no `later.md` row.

107. **jig 11's Shapes against main** (ready check of jig-11.1).
     jig 11's session file, checked against `main` at jig `375545d78`, re-read at `ea4eb2db6` (jig 01.2's renames), temper `e3008e4f`; corrected in `jig/11-client-domain.md`:
     - 11.1's users of `Hello { versions }` gain `crates/jig-client-view/src/tests/pages.rs:68`, `examples/ops/view/src/tests.rs:102`, `tests/client/src/joint/engine.rs:170`; `tests/client-protocol` names `front.rs:126` alone; downstream, temper's terminal `tests/fixtures.rs:226` and its client world's `front.rs:81-82` (`CLIENT_VERSION`, which 11.1 removes); `observe.rs:203`; `client_step` at `graph.rs:138`.
     - 11.2's downstream users of the link's payload types gain the terminal's `tests.rs:20,122,129`, `tests/family.rs:55,57,121,125,150,154,182,183`, `tests/fixtures.rs:76`, and the one-process world's `referee/client.rs:148`.
     - 11.3: the version sweep's per-crate counts are 20, 21, 1, 4, 1, 4, 3, 1, 10 (65 in all; 8 downstream); served ranges at `tests/client-protocol/src/fixtures.rs:60`, `tests/engine-clients/src/fixtures.rs:20`, `tests/engine-clients/tests/stream.rs:143`; the six `Transport::new(VersionRange { first: 7, last: 7 })` of `tests/client/src/joint/transport.rs` and the two `Script { versions }` stay, since no hello reads them; the table limits at `tests/client/src/test.rs:70-77`.
     - The sketch derives `Clone, Debug, PartialEq, Eq` on each type (`Copy` on the eleven the check named, `Debug` alone on `Domain`) and marks seven functions `#[must_use]`, as jig's workspace lints ask.
     A further user of a changed type, version or count that the Files miss, the implementer edits and lists in the commit body.

108. **temper 01.2 makes the slowest tests cheaper first** (process, temper 01's block of 2026-10-16T09:07:52Z).
     Ruling 108: option A, as the README's Rules, "Test time", already say: "An aim exceeded is a guide missed: the landing stands, and the session's next increment first makes the slowest tests cheaper, found by `heavy --gate -- cargo nextest run --workspace --profile measure -j 1` once, until its own gate is back under the aim". temper's fuzzy aim is 19.36 s (14.36 s and the run's 5 s); 01.1's gate gave 20.452 s.
     - temper 01.2, before its renames: run that command once, and make cheaper the slowest tests it names, slowest first, until the gate's fuzzy suite is under 19.36 s. The repairs may change those tests' files and their fixtures modules, in a commit of their own within 01.2, and are not counted in 01.2's size.
     - A repair keeps each test's behaviour: what it checks, its cases and its names. A repair that would drop a case or change what a test checks is a question, with the test and its time.
     - Never, as the rule says: a raised `global-timeout`, slow period or per-test ceiling, or a focused test moved to the fuzzy suite.
     - 01.2's report names the measure run's three slowest tests with their times, each repair, and its gate's suite times from `Gate-Result`; nothing else is measured.
     Unchanged: 01.2's Scope, Files and Test shape for the renames.

109. **The fuzzy suite's measure run** (plan defect, temper 01's block of 2026-10-16T09:24:40Z).
     Ruling 109: a plan defect. The README's measure command runs the default profile's filter, which leaves out every `fuzzy_` binary; the suite that missed its aim is the fuzzy one. temper 01.2 runs, once, `heavy --gate -- cargo nextest run --workspace --profile measure -j 1 --ignore-default-filter -E 'binary(/^fuzzy_/)'`, the command `.config/nextest.toml` documents for the fuzzy suite, and makes cheaper the slowest fuzzy tests it names, as ruling 108 says, until its gate's fuzzy suite is under 19.36 s. The focused run already made counts as 01.2's focused measurement; it is not run again, and no focused test is repaired for it (the focused suite, 5.016 s, is under its 6.0 s aim). The README's "Test time" now names both commands: the measure run of the suite that missed. Unchanged: the rest of ruling 108.

110. **The small profile's memory sum is a shared literal** (steer of the check of 2026-10-16T08:53:33Z).
     From the check of 2026-10-16T08:53:33Z, for the steer that follows: `crates/jig-protocol-config/src/tests/domain.rs:169`, `memory_below_the_components_sum_is_refused`, pins the small profile's memory sum (`What::Memory { over: 11_900_420_359 }` on `main`), which moves with each stored record's or component's worst case: it failed jig-02.1's gate 178 and jig-05.1's gate 191. The README's "Shared files" now name it, for jig 02 to 09 and 16. An increment whose Shape changes such a worst case runs that test alone in the build lane before `land` and sets `over` to the value it prints; the session that lands second rebases and sets it again from its own run.

111. **The process tier on carbon-forge** (process, the check of 2026-10-16T08:53:33Z).
     The process tier on carbon-forge, from the check of 2026-10-16T08:53:33Z: skein's tree keeper (`testing/skein-world/src/tree.rs:150-161`) returns `Error::Other(16)` when a second observer starts in one process without a cgroup it may manage; `Cgroup::prepare` (`:524-590`) needs a cgroup above the observer's own that the runner user owns, and here it owns none (pueued runs in root's `user-0.slice/session-c5.scope`, every level root:root 755; `loginctl show-user runner`: "not logged in or lingering"). temper's process stories start `temper serve` and then `temper open`, so they fail (`logs/20261016T083347-158.log:176`: "temper open starts: Tree(Other(16))"); jig's start one binary and pass (jig-protocol's `heavy.tsv`, 2026-10-13T16:52:29Z, exit 0).
     - Until the machine gives the runner a cgroup, a temper Proof on the process tier runs one layer down, as implementer.md 8.8 says ("If the Proof line cannot be followed, say why in the report, run the same check one layer down … Do not stop for this"), and its report names this ruling. jig's Proofs on the process tier (jig 10, 18) run at their layer.
     - The fix needs root and goes to the user (decisions.md, "For the user", E). Once it is in, the supervisor probes it and rules again; the process Proofs left run at their layer from then, and temper 01.1's process Proof runs once in the first temper increment that follows.
     Unchanged: the gates, which run no process tier.

112. **The profiles' store bounds are a shared literal** (plan defect, jig 02's block of 2026-10-16T09:42:12Z).
     Ruling 112: option A, a plan defect, and now a standing rule beside ruling 110's: the profiles' store bounds are derived from the stored records' worst cases, as the memory sum is. jig 02.1 raises the small and standard profiles' store bounds (`value`, commit, snapshot) in `crates/jig-protocol-config/src/profile/{small,standard}.rs` to what the build requires, and sets both expected tuples of `both_jig_profiles_have_a_bounded_service` (`testing/jig-test-service/src/tests.rs:22-24`) to match, keeping the test's name and every other assertion; its report and commit body name each old and new value. These files are outside 02.1's Files by this ruling, and not counted in its Size. The README's "Shared files" now say the same for every session that grows a stored record's worst case (jig 02 to 09, 16): the session that lands second rebases and sets them again from its own build. A bound that would have to fall, or a value no profile can hold, is a question.

113. **The ready checks' corrections, landed** (process, the re-check of jig-10.1 and jig-11.1).
     The session files' corrections of rulings 103 (jig 04), 104 (jig 10), 105 (jig 08) and 107 (jig 11) did not land with them: each was committed in the supervisor's worktree before `plan ruling`, whose `docs_worktree` switches a clean worktree to `origin/main` and so dropped the commits (`git log origin/main -- docs/development/plans/client-api/jig/10-transport.md` showed only `d9b68d48`). They land now, unchanged from those rulings' texts, with:
     - jig 08.4: the designer's text of ruling 106 (`97315175`) kept where the two met (08.4's core Files and temper's census walk).
     - The re-check of jig 10 and jig 11 after jig 01.4 (jig `2900e9b3b`, temper `ed566100`): jig 10.2's native shell dependency list at `tests/boundary/src/tests.rs:244-252` (manifest parsing moved to `tests/boundary/src/manifest.rs`); jig 11's `examples/ops/client-view/src/tests.rs:102` (renamed by jig 01.3); temper's contract path in 21 files, 40 lines, not 24 and 55. Nothing else in the files those landings moved changed a name, line or count the two sessions give.
     The supervisor applies plan edits uncommitted from now on.

114. **jig 03's Shapes against main** (ready check of jig-03.1).
     jig 03's session file, checked against `main` at jig `375545d785`, and re-read on `main` after jig 01's renames; corrected in `jig/03-parties.md`:
     - 03.1's Shape: the people child refuses no role (`admission`, `domain.rs:1838`, routes the ask): `create_request` gains a step 0, `Role { held }`, as `goal_start` refuses at `acts/bounds.rs:233-238`, and `an_observers_create_is_refused_by_role` replays a root's `Decided { Refused(Role) }`, as `tests.rs:478` does. Refusals answer through `person_task_refusal` (`routing/refusals.rs:11`); the codec's people limit is at `src/codec/people.rs:25`.
     - 03.1's world: `offers::world` builds its own limits, so `tests/core/src/stories/offers.rs` gains `world_with(seed, role, edit)`, which raises the journal's `writes` and `held` (`tests/core/src/world.rs:176-185`) to the edited core's `room_max` and two connectors' `MAX_OUT` before `World::configured`. With the plan's limits the journal held 2308 writes and 6924 held against the 7554 each that `room_max` needs, and `root::worst_case` refused ("testing root bound fits"); with the change the check's test passed, clippy clean.
     - 03.1, 03.3: the test mirrors of `Refusal` (`jig-test-connector/src/application_core.rs:6`, `jig-fake-parties/src/facts.rs:6`) stay; the mappers translate `NotWellFormed { Cycle }` to the mirror's `Limit { Cycle }` and a later `Malformed` to its nearest refusal, named in the landing's report; `Accepter::Role` is `Some(true)`, `Nobody` `Some(false)`, `None` stays `None`; `jig-fake-parties/src/facts.rs` leaves 03.3's Files.
     - Files: 03.2 gains `Lack::TaskKind`'s other users (`authority_facts.rs:42,69`, `routing/refusals.rs:47`, people `tests.rs:1178`, engine-clients `fixtures/answers/lacks.rs:28`, `tests/people/src/referee.rs:121`); 03.4 `run.rs:397`, `goals.rs:104,117`, `inbox/census.rs:51`, `stored.rs:327`, `outcomes.rs:37`; 03.5 `run.rs:392`, `outcomes.rs:143`, `tests/people/src/world.rs:713` (moved from 03.4), `sign_in.rs:15,25`, `component.rs:711`; `tests/core/tests/stories.rs` at `:1308`, `:976`, `:1219`.
     - The sketches carry derives (`Copy` on `Given`, `Accepter`, `Malformed`, `TaskExecutor`, `Flags`, `Retention`); the Rules' 800-line list gains `testing/jig-test-domain/src/routes.rs`, `tests/people/src/world.rs`, `tests/tasks/src/world.rs`.
     A further user of a changed type or match that the Files miss, the implementer edits and lists in the commit body.

115. **jig 07.2's procedure decision carries targets and admissions** (plan defect, jig 07's block of 2026-10-16T09:58:55Z).
     Ruling 115: option A, a plan defect, not a design gap. The Shape's own route asks for what its type leaves out: it checks "each message's target referenced (tasks.md, 8.5)" and each amendment "admitted as a run's is", yet `messages: Box<[Word]>` carries no target and `amendments: Box<[(u64, Amendment)]>` neither the message number nor the stop that `amend` admits a run's amendment with. 07.2's Shape now reads `amendments: Box<[ProcedureAmendment]>, messages: Box<[ProcedureMessage]>`, with `tasks::ProcedureMessage { task: u64, word: Word }` and `tasks::ProcedureAmendment { task: u64, message: u64, stop_run: bool, amendment: Amendment }`, both filled by the core from its `StepMessage` and `StepAmendment`, which keep their fields. The tasks child applies them through the entrances a run's word and amendment take today. Unchanged: `batch`, `end`, the order of checks and parts, and the refusals.

116. **Named numbers in tests** (steer of the check of 2026-10-16T09:53:25Z).
     From the check of 2026-10-16T09:53:25Z, for the steer that follows: bare numbers naming identities in new tests, a second sighting (skein's harness tests, `Token::new(7)`, at the check of 08:53; jig-05.1's `crates/jig-domain-core/src/tests/waits.rs`, tasks 1, 2 and 99, persons 1 and 2, message 3), become a rule of the README, "Named numbers in tests": a new test names each number that identifies a party, a task, a message, an entry or a token, when an assertion names it, as a `const` at the file's top or in the crate's fixtures module, as `tests/core/src/fixtures.rs:2-3` does; counts and expected values stay literals; base tests stay as they are.

117. **jig 12's Shapes against main** (ready check of jig-12.1).
     jig 12's session file, checked against `main` at skein `b23380a6a`, jig `817f15608`, temper `c707fd7b6` (names that jig 10 and 11 add are theirs, not mismatches); corrected in `jig/12-client.md`:
     - Choices, the link's limits: `jig_client_domain::Limits` also gains the bounds the link reads today from `Limits.client`: `delegates` (`crates/jig-client-protocol-link/src/limits.rs:100`), `records`, `kept` and `parts` (`src/store.rs:36`).
     - 12.1's Files: `Limits.client`'s user `crates/jig-protocol-engine-clients/src/tests/fixtures.rs:740-770` (`client_limits()`); the small store's names' users the native shell's `shell.rs:85` and `connections.rs:122-123`, the link's `component.rs`, `src/tests/fixtures.rs:105, 139, 171`, `src/tests/fixtures/link_{deliveries,reads,store,stream}.rs`, `src/tests/store.rs:61`, and `front/persistence.rs:32, 79` beside `:38`.
     - 12.1's unit sketch: `signed_in` has opened `me`, and with it the stream (the link's `component.rs:349-357`), so a later watch leaves as `[Request::Exchange { credential: Some(_), message: Message::Watch(_), .. }]`, not `OpenStream`.
     - 12.1's fixtures copy the client world's limits as literals, as jig 11's do: a dev-dependency on `jig-client-world` would be a cycle once 12.5 gives it `jig-client`; 12.5's Files gain `tests/client/Cargo.toml`.
     - Re-exports: `jig_client` also re-exports `Retry` and `Backoff` (12.2 reads `Retrying { why: Undecodable }`), and the domain's `Limits` as `DomainLimits`, the type of `jig_client::Limits.domain`, so an application depending on `jig-client` alone names them. The sketch's `Client::new` and `Client::typed` carry `/// # Errors` (clippy pedantic's `missing_errors_doc`).
     - 12.2's Files: the users of `Request::NotKept` and `Answer.why`: the link's `component.rs:241, 299` and `src/tests/fixtures.rs:126`; the native shell's exhaustive matches at `connections.rs:120-133` and `shell.rs:117-130`; downstream, its terminal's `route.rs:50-134` (`Answer` at :84), `src/tests/family.rs:174`, `src/tests/fixtures.rs:77`.
     - Lines: `pressure()` at `stories.rs:67`, not 59; `FrontFault::Versions` at `front/faults.rs:43-48`, not 44. The two referee tests live in `tests/client-protocol/tests/client.rs`, so that the Done when's fourteen count them.
     - Checked: the boundary test sketch in `graph.rs` passes clippy and fails only on "one client", as 12.1's to add (`jig-boundary`, 69 of 70 passing); the interface sketch passes clippy with jig 10's and 11's types stubbed, once it has the `# Errors` docs.
     A further user of a changed type or name that the Files miss, the implementer edits and lists in the commit body.

118. **jig 11.1's move size: re-wraps are rewrite lines** (plan defect, jig 11's block of 2026-10-16T10:40:26Z).
     Ruling 118: option A, a plan defect. 11.1's Size already counts the `use` lines a nested block splits into as rewrite lines ("the rest the `use` lines split out of nested `use` blocks, which are rewrite lines too"), but code-organization.md, 7's count ("the added lines that name a moved path or a new name") cannot see them, nor the lines rustfmt re-wraps around a longer path. jig 11.1 lands as committed, at its remainder of 198, with no reduction and no split.
     For the moves left (jig 13.2, temper 02.1's path rewrite, temper 03.3, 03.4, jig 15.4, jig 17.3), code-organization.md, 7 now says: "Lines that rustfmt re-wraps because a path grew (a `use` list's continuation lines, a signature or a call re-wrapped) are rewrite lines too, though they name no new path. When the difference passes twice the Size, the report lists, by file, the difference's lines that are no such re-wrap; within twice the Size, the increment lands without asking." The count's command and the Size of each increment are unchanged.

119. **One land at a time in each repository** (steer of the check of 2026-10-16T10:44:32Z).
     From the check of 2026-10-16T10:44:32Z, for the steer that follows: passed gates thrown away because `main` moved while they waited in the queue, a second sighting (the check of 09:53Z: tasks 214, 235, 259; this check: tasks 274, 275, 300, 317, 320, 333, 338, 1,145 s of the gate lane and 2,446 s of the build lane for their prebuilds), become a rule of the README, "One land at a time in each repository": a session starts `land` only when no other land of its repository (pueue labels `land <repository>` and `land <repository> prebuild`) is queued, running or paused, and waits for that in one command, then lands; until `land` rebases after its queue wait. Unchanged: land's steps and gate; the steers of 09:29Z (point 3) and 10:08:29Z (point 2).

120. **jig 04's Shapes against main, after jig 02.1** (ready check of jig-04.1).
     jig 04's session file, re-checked after jig 02.1 on the files moved since jig `3f3fd7ae0` and temper `8d6b6f68`, at jig `d7eca9abd`, temper `747d8b10`, skein `b23380a`; corrected in `jig/04-grace-and-compound.md`:
     - Lines moved by jig 02.1: the tasks world's `World::new` :191, `make` :824, `claim` :861, `elapse` :964, `restart` :994, `settle` :942, `record` :798; `render.rs`'s `task_problem` :222; tests/revisions.rs's `Harness` :96, `to` :102, `amendment` :208; the tracked `Make` refusal at core `routing/tasks.rs:1361`; the base sizes of the 800-line files (boundary.rs 1804, domain.rs 1278, stored.rs 1072, core routing/tasks.rs 1459, core tests.rs 2993).
     - Files gained: the tasks `Amendment` literal at `tests/conformance/src/actions.rs:102` (04.2) and the people one at `tests/conformance/src/scenarios.rs:227` (04.5), both jig 06.1's `RacingDecisions`; for 04.1, and for 04.2, 04.5 and 04.6 where their records grow, the values derived from the task record's worst case (rulings 110 and 112): `STORE_KV`'s `value`, `chunk`, `page.bytes` in `profile/small.rs:305, 313-314` and `profile/standard.rs:144, 150-151`, `What::Memory { over }` at config `tests/domain.rs:169`, and their copies in `testing/jig-test-service/src/tests.rs:24-25`.
     - 04.3: `apply_message` keeps jig 02.1's `last_activity` write (`inbox.rs:326`), as `closing::end_task`'s split keeps its own (`closing.rs:389`); `tests/tasks/tests/activity.rs:139, 159` join the tests that pass unchanged.
     - A design gap, to the designer at once (question.md): `Core::chat_requester` (`jig-domain-core/src/inbox.rs:108`) returns `None` for any tracked task, so a chat tracked by 04.2's or 04.5's amendment stops being its party's chat. Interim, in the session file's Rules: 04.2 and 04.5 build and test tracking on tasks that are no conversation, and nothing of the session admits, refuses or tests tracking a conversation until the answer amends the file; 04.1, 04.3, 04.4 and 04.6 do not rest on it.
     - Checked: in a scratch worktree at jig `d7eca9abd`, clippy `-D warnings` on `jig-tasks-world` and `jig-core-world` with the session's sketches passed; nextest's default profile on `jig-domain-tasks`, `jig-tasks-world` and `jig-core-world` passed 420. The fuzzy profile was not run (the queue).
     A further user of a changed type or value that the Files miss, the implementer edits and lists in the commit body.

121. **jig 08's Shapes against main, after jig 02.1** (ready check of jig-08.1).
     jig 08's session file, re-checked after jig 02.1 on the files moved since jig `3f3fd7ae0` and temper `8d6b6f68`, at jig `d7eca9abd`, temper `747d8b10`, skein `b23380a6a`; corrected in `jig/08-store-indexes.md`:
     - The sketches: `#[must_use]` on `EndedByParty::key`, `EndedByProject::key`, `forgets` and `ended_part`, as the base's `Update::key` (`crates/jig-domain-tasks/src/index.rs:42`); clippy `-D warnings` failed on the six without it. `forgets(row, reading)` reads `row.flags`, which jig 02.1 put on `TaskRecord` (`boundary.rs:765`), in place of a `flags` parameter; `tasks::Flags` and `tasks::Retention` leave the stub list (landed in `flags.rs`).
     - Lines moved: `stored.rs`'s `Stored::Result` :792; core `routing/tasks.rs`'s `ChatState` :784; `testing/jig-test-domain/src/lib.rs` :982; `census_owner` :1071-1078; `tests/core/src/observations.rs` :593; conformance `peers`' arms :216-253; the 800-line files' sizes (boundary.rs 1804, routing/tasks.rs 1459, core tests.rs 2993, tests/tasks world.rs 1206, observations.rs 1068, jig-test-domain lib.rs 1125, referee.rs 1116). The fuzzy sweep has nine scenarios (jig 06.1's `RacingDecisions`).
     - 8.1's Files gain the values derived from `tasks::max_out`, which `output_bound`'s multiplier raises (27, then 28 in 8.2): `route_room_checked` (`routing/mod.rs:277`) and `room_max`, and under rulings 110 and 112 the profiles' bounds, `What::Memory { over }` at config `tests/domain.rs:169`, and their copies in `testing/jig-test-service/src/tests.rs:24-25`, where they move.
     - Not checked, for the implementer's first build: the adapted 8.1 and 8.3 reference tests did not compile in the check's 25 minutes (the queue), and no suite ran.
     A further user of a changed type or value that the Files miss, the implementer edits and lists in the commit body.

122. **temper's walking replay keeps its Debug comparison** (process, temper 01's block of 2026-10-16T11:24:09Z).
     Ruling 122: option B, process. temper 01.3 lands with `run_replayed`'s complete-state `Debug` comparison as it stands, every seed and every iteration kept; the trace's typed comparison stays. Option A is refused: `PartialEq` on jig's `Core` and temper's `engine::Domain` is production code of two repositories, outside temper 01's Scope. Option C is refused: ruling 108 keeps "what it checks", and the steer of 2026-10-16T11:19:17Z kept "its complete-state check".
     The fuzzy suite's aim stays missed while this test is its time; the README's "Test time" says "after a session's last increment the supervisor decides", and now says: "temper's `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure` is no test to make cheaper in this run (ruling 122, `later.md`): the slowest tests a repair finds are the others." The typed comparison is a `later.md` row, for a jig and temper plan after this one.
     Point 1 of the steer of 2026-10-16T11:19:17Z ends with this answer. Unchanged: rulings 108 and 109 for every other test; the gate.

123. **A cancel with a grace during a grace: interim** (design gap, interim, jig 04's block of 2026-10-16T11:24:44Z).
     Ruling 123, interim: a design gap, gone to the designer (question.md, "jig 04.1: a cancel with a grace made during a grace"). tasks.md, 7 settles the deadline ("A cancel made during a grace ends the task no later than that grace would: one without a grace cancels it at once") and 8.2 the room ("a cancel, its grace's ask to finish taking the same room"); neither says whether a later cancel's ask to finish is heard, nor whose `by` and reason end the task.
     Until the answer lands in `jig/04-grace-and-compound.md`, jig 04.1 goes on with everything else of its Shape: the types, codec and configuration; a first cancel with a grace (one ask to finish, one slot, `grace::schedule`, `grace::fire`); a cancel without a grace during a grace cancelling at once; and a cancel with a grace during a grace keeping the earlier `until`, as the Shape says, with nothing else of it: it adds no word and changes no `by` or reason, and no test asserts either. 04.1 does not land before the answer; when it comes, the designer's text replaces this.

124. **The grace keeps its ask's number** (plan defect, jig 04's block of 2026-10-16T11:29:21Z).
     Ruling 124: option "add `message: u64`", a plan defect, not a design gap. The design keeps the ask apart in its own words (tasks.md, 8.2): room kept, "a cancel, its grace's ask to finish taking the same room, so none of them can be refused", against merged, "a new amendment with one still waiting"; and the plan's Choices already give the number ("The cancel carries its finish message's number … `Control::Cancel` is `{ reason, grace, message }`"). What the Shape missed is that the durable grace keeps it.
     04.1's Shape now reads: `Grace { by: Party, until: Wall, reason: Box<[u8]>, message: u64 }`, `message` the ask to finish's number, the cancel's `message`; `inbox::prepare`'s merge (its one key for `MessageKind::Amendment`, `inbox.rs:31`) passes over the word of that number, which keeps its slot; `stored.rs`'s count of amendment words (`:395`) and the restore admit it beside one ordinary amendment. Ruling 120's derived values (the task record's worst case) count the field. No new `MessageKind`; no reading of text or sender.
     Unchanged: ruling 123, interim: a cancel with a grace during a grace keeps the earlier `until`, adds no word and changes no `by` or reason, and 04.1 does not land before the designer's answer to that question.

125. **A tracked conversation stays one; a task is asked to finish once** (design gap, jig 04's block of 2026-10-16T11:24:44Z and the ready check of jig-04.1).
     The designer's answers, landed as jig a70de2906 and temper ed6593899:

     # Answer

     Both answers have a `## Decision` of their own, first; their reasoning
     follows under one `## Reasoning`, so that `sed '/^## Reasoning/,$d'`
     keeps both decisions.


     1. **(A): the conversation flag decides, and a tracked conversation stays
        its party's.** Tracking never ends a conversation, and no tracking is
        refused for one. jig's tasks.md, 7, "Tracking and untracking", now
        reads: "Neither touches its flags, so a conversation stays one:
        tracked, its turns are still entries for its requester, with their
        read position and unread count (section 3; people.md, section 6). No
        tracking is refused for a task's being a conversation." core.md, 6.2:
        "A chat, a triage or a repair is made untracked ... A conversation
        tracked later stays a conversation". people.md, 6: "A conversation is
        a task its creator flagged so ..., whatever its charter, tracked or
        not".
     2. **04.2 and 04.5 admit tracking any task, a conversation included,**
        and add no refusal, branch or test for its being one. The rule "A
        tracked conversation stays one" replaces ruling 120's interim rule in
        jig 04's Rules. No test of jig 04 asserts what a tracked
        conversation's inbox lines, offers or read position show.
     3. **The code changes at jig 05.5, as planned.** 05.5 already makes
        `chat_requester` into `conversation_requester`, "the requester party
        of a task flagged conversation, whatever its charter, tracking or
        recurrence", tested by `a_tracked_conversation_keeps_its_read_position`.
        It now also names the two other tests of a chat that read `tracked`
        and `recurring`: `documents/inbox_deployment.rs:340-345` and
        `routing/watch_marks.rs:202-207`. Both go, as `chat_requester`'s do.
        Its Don't now reads: "read `settings.chat`, `tracked` or `recurring`
        to know a conversation".
     4. **Between 04.5 and 05.5,** a chat tracked on `main` leaves its party's
        inbox. No client can track one before jig 14 carries the tracked
        amendment on the wire (04.5's Files: "the wire's cancel and amendment
        give `grace: None`, `tracked: None`: jig 14 carries them").
     5. **Unchanged:** 04.7 (`MoveChat`, `Scenario::Compound`) and jig 08 use
        the core's conversation test, whichever name it has. Nothing is
        deferred: no `later.md` row.


     1. **(A): a task is asked to finish once.** tasks.md, 7 now reads: "One
        with a grace asks nothing more, since a task is asked to finish once,
        by the first. If its grace ends sooner, the task's grace now ends with
        it, and the task is then cancelled with this later cancel's maker and
        reason, since that cancel is what ends it. If its grace ends no
        sooner, nothing of the task changes." 8.2's room kept: "a cancel, its
        grace's ask to finish taking the same room, one ask in a task's life
        (section 7)".
     2. **04.1's Shape:** `ask_to_finish` on an active task whose `grace` is
        `Some` adds no word and takes no room. If its `until` is earlier, the
        `Grace` takes this cancel's `by`, `until` and `reason`, keeps its
        `message` (ruling 124's field: the first ask, the one word there is),
        and `grace::schedule` re-arms the alarm. If it is equal or later,
        nothing changes. The cancel is answered as the first was, and its
        `message` is left unused. A held or waiting task is still cancelled at
        once.
     3. **04.1's tests:** `a_second_grace_never_ends_the_task_later_than_the_first`
        (a 5 s grace, then a 10 s one: one ask, ends at 5 s with `b"wrap up"`);
        new, `a_sooner_second_grace_ends_the_task_with_its_own_reason` (5 s,
        then 2 s with `b"now"`: one ask, its words `b"wrap up"`, ends at 2 s
        with `b"now"`); the full-inbox test also admits a second cancel
        with a grace, which adds no word. Size goes from 235 to 255.
     4. **04.5:** a party's cancel with a grace during a grace takes 4.1's
        path. It is answered `Outcome::Cancelled { task }`, and the message
        number the core took for it stays unused. 04.6's compound applies
        cancels in order, so a second one sees the first's grace. 04.7's
        scenarios cancel once: unchanged.
     5. A rule in jig 04's Rules, "A task is asked to finish once", binds
        04.1, 04.5 and 04.6. Nothing is deferred: no `later.md` row.

126. **jig 09's Shapes against main** (ready check of jig-09.1).
     jig 09's session file, checked against `main` at skein `de0f5ba42`, jig `805f1a5b7`, temper `9d7fcc789` (names jig 02.3, 04.1, 04.2 and 04.6 add are theirs); corrected in `jig/09-templates.md`:
     - The sketches derive what jig's lints ask: `Clone, Copy, PartialEq, Eq, Hash, Debug` on `TemplateExecutor`, `TemplateBudget`, `TemplateDeadline`; `Clone, PartialEq, Eq, Hash, Debug` on `TemplateOverride`, `Template` and the base's `PolicyChange` (clippy failed on six types without `Debug`).
     - The verdict contract's JSON form carries `tasks::Verdict`'s `followups` and `fields` (`jig-domain-tasks/src/boundary.rs`).
     - Lines moved: `front.rs`'s `chats_made` :263; the store fixtures' core `Limits` :503 and `people_policy` :369.
     - 09.2's users: `tests/client/src/joint/wire/asks.rs:714`'s match over `PolicyChange` (`Templates { .. }` joins the panic arm at :718); the `documents::Policy` literal at `crates/jig-protocol-engine-clients/src/tests/fixtures/objects.rs:76`.
     - 09.3's bound: `me`'s frame projects are bounded by `documents.lines` (`documents/small.rs:91`), not `people.projects`; the growth is added in `document_with_context_worst_case(&Limits)` (`documents.rs:459`), which sees `templates`, and `src/tests/frame_deployment_role.rs:68-74`'s `held` counts it; the memory value at config `tests/domain.rs:169` and the profiles' bounds move with it (rulings 110, 112).
     - 09.4's channel fixture: `"tasks.tasks":2` needs `"people.inbox_tasks":4` (core `memory.rs:99`: `inbox_tasks` at least twice `tasks`; the check saw 2 refused and 4 accepted), and `"tasks.project_goals":1` suffices.
     - Downstream: the service's two matches over every `config::What` (`startup/problems.rs:81-103`, `:128-170`) gain arms for 9.1's `Grace` and 9.4's `Operations`, at the repin.
     - Checked: clippy on the sketches with the derives and on `jig-protocol-config` with the 09.1 reference test (a stub `What::Grace`) passed; the reference test compiles and panics at the base in `fixtures::with`, as it should before 09.1. Not captured: the channel worlds' runs with the corrected limits.
     A further user of a changed type or value that the Files miss, the implementer edits and lists in the commit body.

127. **One land at a time, by a lock** (steer of the check of 2026-10-16T11:44:40Z).
     From the check of 2026-10-16T11:44:40Z, for the steer that follows: ruling 119's poll of the queue lets two lands through, since a land has no task in the queue between its prebuild and its gate, nor between its tries (jig 07's gate 370 ended 11:42:19Z, its retry 374 was queued 11:42:26Z, jig 02's 375 at 11:42:29Z; jig 06's prebuild 356 at about 11:30:07Z, between jig 07's 351 and 355); 7 passed gates voided in the hour (317, 320, 333, 338, 348, 355, 370). The README's "One land at a time in each repository" now holds a lock for the whole of `land`, retries included: `flock /srv/data/git/runner/src/tmp/temper/client-api/land-<repository>.lock land …`, in place of the poll. Unchanged: the rule's scope (each repository) and its end (a `land` that holds its repository's lock from its rebase to its push); land's steps and gate.

128. **jig 02.2's peers failure is its own** (process, jig 02's block of 2026-10-16T11:59:17Z).
     Ruling 128: process. The failure is jig 02.2's own, not another session's: `main` passes `peers_scenarios`, and 02.2 on it fails twice. jig 07.4's landing, `99ead87df`, on `4b2ef1e69`, passed the gate with `PASS [ 0.275s] … jig-core-world::peers peers_scenarios` (task 384); 02.2, rebased onto `4b2ef1e69`, failed it in its gate (task 386) and alone (0.083 s), at `tests/core/tests/peers.rs:105`, "all four configured hosts ran work".
     jig 02 diagnoses it within 02.2, in at most 30 minutes from this answer: run the test alone on 02.2 and on `origin/main`, and find which of 02.2's changes moves the scenario's hosts. Then:
     - a defect in 02.2's code: fix it there; the test passes unchanged;
     - the Shape's own behaviour changing the scenario's outcome (as a task with no goal carrying no goal scope): a question, with the cause by file:line and the two runs' outputs;
     - no cause found in 30 minutes: a question, with what was run.
     The test's assertions and expected values stay as they are; a fixture input changes only under an answer to such a question. Lines of the fix outside 02.2's Files are reported as the README's "Edits outside an increment's Files" says. 02.3 goes on meanwhile, as proposed.

129. **jig 02.2's peers chats admitted one by one** (plan defect, jig 02's block of 2026-10-16T12:15:49Z).
     Ruling 129: option A, a plan defect. The diagnosis under ruling 128 shows the cause is 02.2's Shape ("a task with no goal has no goal scope: its brief carries none"): without each chat's own goal load, the first two chats finish preparation in one commit and both pin to the engine. The story checks that the four hosts run the same scripts, not concurrent preparation. jig 02.2 makes `tests/core/tests/peers.rs`'s four `Chat` requests admitted one by one, each drained before the next (`acts`/`drain`, as the report proposes); every assertion and expected value stays as it is, and the change is reported as an edit outside 02.2's Files.
     Option B is not taken: host selection is placement policy, outside 02.2's Shape. The race the diagnosis found (two claims prepared together both choose the engine from `fleet.free_kind` before `ClaimDone`, `run_preparation.rs:185-192`, `charters.rs:74-76`) is a `later.md` row.

130. **temper 02's Shapes against main, after temper 01** (ready check of temper-02.1).
     temper 02's session file, re-checked after temper 01 on the files moved since skein `15ddb754`, jig `375545d7`, temper `385219e9`, at skein `de0f5ba4`, jig `99ead87d`, temper `f6f01026`; corrected in `temper/02-names.md`:
     - Files: `crates/temper/src/open.rs` leaves the list (temper 01.3 removed its only old name, `jig_view::phrases`); the Rules' grep finds 8 manifests and 34 other files beside `Cargo.lock`, otherwise the listed ones. `crates/temper/Cargo.toml:16`'s `jig-view` key is still renamed.
     - `kit_root` at jig's `tests/boundary/src/tests.rs:8-12`, not 14-18.
     - Shape 2: the renames are followed by `cargo fmt`, which reorders the `use` lines they move (4 files, 17 lines; the gate's `cargo fmt --check` fails without it).
     - Checked, in a scratch worktree with Shapes 1, 2 and 4 applied and the naming test: the pins come back to the rule's three lines (jig `2900e9b3`, skein `f12f29d4`, smith `29fa97c9`) with `--precise`, skein on its second run as the Pins rule foresees; clippy on the workspace passed, so Shape 3 needs no repair; nextest's default profile passed 276 of 276 on the 8 crates reached, `client_crates_carry_the_client_prefix` among them, and the fuzzy profile 11 of 11. The process Proof failed to spawn `temper open` twice (`crates/temper/tests/support/terminal.rs:43`), the machine's limit of ruling 111, not confirmed on an unchanged tree: 02.1's process Proof runs one layer down under ruling 111.
     A further user of a renamed crate that the Files miss, the implementer edits and lists in the commit body.

131. **Generated code only by the drift tests' write mode** (steer of the check of 2026-10-16T12:34:18Z).
     From the check of 2026-10-16T12:34:18Z, for the steer that follows: jig 07 (07.2) built and ran a scratch crate, `logs/jig-07/codegen` (heavy task 418, a 9.7 MB `target/`), to regenerate the test connector's code and goldens, compiling `skein-codegen` from skein's `main` (`7e8fb700`), not jig's pin (`f12f29d4`); the second sighting after jig 06's `logs/jig-06/codegen` (the steer of 2026-10-16T09:29Z, point 4, which applied only to the store-family sessions). The README's new rule "Generated code" holds for every session: a schema's code and goldens are written only by its drift test's write mode, in the build lane (`SKEIN_CODEGEN_WRITE=1`, `tests/store-codecs/tests/drift.rs` for the store families and the test connector, `tests/client-wire/tests/{drift.rs, test_client_wire.rs}` for the client wires); no session builds a crate of its own to generate them. jig 07 runs the test connector's write mode once, keeps what it writes, and removes `logs/jig-07/codegen/`. Unchanged: the steer of 09:29Z, point 4; the README's "Format versions".

132. **temper 03.1's Shapes against main** (ready check of temper-03.1).
     temper 03.1's session file, checked at skein `7e8fb700`, jig `5077e22c`, temper `5b79cd74` (temper's pins jig `a3b53546`, skein `f12f29d4`, whose `skein-ui-text` is main's), corrected in `temper/03-client.md` (report: `reports/check-temper-03-1.md`):
     - Scope: `temper: closed` is the terminal's own line (`Terminal::close` says it), never the host's; the host closes the client once `Terminal::closed()` is true, after the drain (`ui-text.md`, 7: "says it is closed (`NAME: closed`); the host then closes its application").
     - Files: `terminal.rs` is 318 lines; `lines.rs` (59) and `tests/lines.rs` (39) go with `Lines`, whose only user is `terminal.rs`; the crate's `Cargo.toml` (`skein-sim` as a dev-dependency, by `tests/one-process/Cargo.toml:44`'s git URL) and `Cargo.lock`; the users grep's lines named (`open.rs:101`, `tests/support/mod.rs:167`, `tests/terminal.rs:182`).
     - Shape: `Settings.name` is a `Box<[u8]>` (`Box::from(*b"temper")`, `skein-ui-text/src/terminal.rs:17`) and `Terminal::new` returns `Option`, expected as `open.rs` expects its other startup parts; the host's own io takes `1 << 61`, since skein's terminal tags with `1 << 62` (`terminal.rs:12`), the bit `terminal/run.rs:13` uses today, and a completion goes to the terminal when `Terminal::owns` is true.
     - Test shape: the world's limits are its own `tree_limits()` (`support/mod.rs:10`); it drives the terminal's pipes on `skein_sim::Sim`, as skein's `tests/ui-text/tests/terminal.rs:20-42` does; in `the_end_of_stdin_closes_and_exits_zero_after_the_drain`, `assert!(world.client.closed, …)` (`tests/terminal.rs:59`) moves after `world.stdout_takes()`, name, assertions and expected values unchanged; the new test takes `first = world.all_output().len()` before the notice and finds both strings from it with `skein_lib::bytes::find_from` (`World::signed_in()` prints at t=0).
     - Checked by the sub-agent, through heavy: the Shape's sketch failed E0308 as written and passed clippy corrected (task 458); the new test failed as written (task 464, `tests/terminal.rs:254`) and passed corrected, with the crate's default profile 34 of 34 and fuzzy 1 of 1, clippy clean (task 473), on the present `Loop`. Not run: the process Proof (ruling 111), a world rebuilt on skein's terminal.
     - Open: how the host's lines are bounded (`Terminal::say` asserts room, and no public method gives it) goes to the designer (`question.md`); temper-03.1 waits for the answer and for temper-02.1's landing.
     A further user of a changed name or value that the Files miss, the implementer edits and lists in the commit body.

133. **jig 05's order directions are the design's** (plan defect, jig 05's block of 2026-10-16T13:13:44Z).
     Ruling 133: a plan defect, no design gap. The design gives the directions in its own words, the same as the plan's Choices and 05.2's code. jig's `docs/design/client/domain/contract.md`, 6.1, since jig `0ec80c047` (2026-10-16T06:47Z, before the run's claim): "It orders by number, ascending; last change or last activity, latest first; priority, ascending, tracked tasks before untracked; or ended at, latest first, its live group, which has no end, by number ascending. Ties go by number, ascending." The session file's question ("contract.md, 6.1 names the orders, not their directions") was written before that landing; it leaves the file. 05.2's directions stand as landed (`7dc234d1b`), and 05.3 and 05.5 follow them; nothing to change.

134. **The terminal's room: skein-01.s1, then temper 03.1** (design gap, the ready check of temper-03.1).
     Ruling 134: the designer's answer, landed as skein `d047b45a` and temper `3d695094`; the supplement step skein-01.s1 is `skein/01-s1-terminal-room.md`, from the answer's item 4.
     # Answer


     1. **A setting bounds the application's lines, and the terminal gives the room.** `ui-text.md`, 3: the settings gain "*its lines*: the most of the application's own lines the terminal holds that the output has not taken (section 4); 0 for an application that prints none". 4: it "gives the host the room left (`Terminal::room`): the settings' lines less those waiting, and none once it is closing (section 7). The host says a line only within that room. A line said beyond it is the host's defect: the terminal stops, as a full queue does (a panic, fail-stop). What has no room waits in the host". The terminal keeps room for "a whole read's refusals and its `closed`. It asks to read only while that room holds a whole read's refusals, whatever the application has said".
     2. **The host's pace** (7, as the code does at `terminal.rs:251`): "The terminal asks to read only once the host has taken every command it gave (`Terminal::commands`) ... A host that takes none holds the person's lines unread in the input".
     3. **temper** (`client.md`, 6.2 and 6.3): temper's lines, 32. "A line with none waits in the host, in the order they came; a line equal to one already waiting adds none". "The host takes no command from the terminal while one of temper's lines waits in it ... What waits is then at most one of each line but a service's credential, and one credential for each request already sent". The client's notices are taken at every iteration, so the sign-in's retries keep their backoff.
     4. **skein-01.s1, the supervisor's to add** (development.md, 10.4; the plan's files name it so):
        - Files: `crates/skein-ui-text/src/terminal.rs`: `Settings.lines: u32`; the line queue `lines + BATCH + 1`, each entry marked the application's or the terminal's own; `pub fn room(&self) -> u32`; `say` asserts room; the read's condition (`:252`) counts the terminal's own lines alone; `close` queues `closed` as its own. `tests/ui-text/tests/terminal.rs`: `settings()` gains `lines: 2`.
        - Test `the_applications_lines_keep_their_room_while_the_output_is_held`: `start(32, 1)`; `room()` 2; say `one`, `two`; `room()` 0; feed `x\ny\n` one byte per `feed`; drain and drive as `a_print_waits_for_the_last_to_be_written` does; the bytes are `== test == \n[1] Accept\ntest: one\ntest: two\n`, then `test: commands: n, n words, p, q\n` twice; `room()` 2; after `close()`, 0. On the base's one queue, the second line's feed would fail: two application lines stop the read (63 places free, `:252` wants 64).
        - Test `untaken_commands_hold_the_input`: `start(32, 1)`; feed `1`, then `\n`, leaving the command untaken; `sim.peer_feed` of `p` returns 1, then of `\n` returns 0; once the command is popped and the terminal driven, the `\n` fits.
        - Size at most 80 lines. Proof: skein's gate, by `land`. Don't: change `BATCH`, a refusal's text or `say`'s prefix; give `lines` a default; name a downstream repository. skein 01's "nothing under `crates/` changes" does not bind it.
     5. **temper takes it alone, ahead of jig, until jig 19.4.** README, "Pins", "The terminal's room": "03.1's first commit moves temper's skein to skein-01.s1's landing", Cargo.lock alone; "while `git -C ~/src/rust/skein merge-base --is-ancestor <skein-01.s1's landing> <jig's pin of skein>` fails, each temper repin takes skein-01.s1's landing in place of jig's pin of skein". jig moves skein once, at 19.4, now "to skein-01.s1's landing (which holds skein 01.3's)", with "`lines: 0` in jig's `skein_ui_text::Settings` literal" (`tests/client-protocol/src/native/world.rs:232`, jig `7dc234d1b`), outside temper's build.
     6. **temper 03.1** Needs skein-01.s1's landing; its Shape gains the move, `lines: limits.lines` (`limits.rs:181`), the waiting lines (`table.requests`, 16, plus 3) with no command popped while one waits, and the unit test `an_equal_line_waits_once_in_the_order_it_came`. Its Size stays 200.
     7. Nothing is deferred: no `later.md` row.

135. **temper 03.1's waiting lines reach Loop::new** (ready check of temper-03.1, after the answer of 13:28Z).
     temper 03.1's changed lines (the answer of 13:28Z, ruling 134), re-checked at skein `d047b45`, jig `7dc234d1b`, temper `f37cea0d`: every name holds; skein-01.s1's `Settings.lines` and `Terminal::room` are not landed. One gap, corrected in `temper/03-client.md`, 03.1's Shape ("temper's lines"): the waiting queue's capacity, `table.requests + 3`, reaches `Loop::new` from its callers, since `limits::Terminal` has no `requests` (`terminal.rs:62-90`): `open.rs:101` passes `bounds.client.table.requests` (`bounds` is `limits::native()`, `open.rs:91`; `table.requests` 16, `limits.rs:63`), and `tests/support/mod.rs:172` and `tests/terminal.rs:182` pass `limits::native()`'s; `Loop::worst_case` (`terminal.rs:300-308`, added into the host's memory at `run.rs:68-69`) charges that queue in place of `limits.lines`. Checked by the sub-agent: `heavy cargo update -p skein-lib --precise d047b45a` in a scratch worktree moved skein alone (the pins grep printed jig `2900e9b3`, skein `d047b45a`, smith `29fa97c9`, `Cargo.lock` the only change); no crate in temper's build names `skein_ui_text::Settings` (jig's `tests/client-protocol/src/native/world.rs:232` is outside temper's `Cargo.lock`). Not run: suites (no code corrected). temper-03.1 waits for skein-01.s1's landing.
     A further user of a changed name or value that the Files miss, the implementer edits and lists in the commit body.

136. **jig 07.2's Size is 600** (plan defect, jig 07's block of 2026-10-16T13:43:16Z).
     Ruling 136: option A, a plan defect. 07.2's Size was set below what its Scope asks: "a step's decision holds a batch, amendments …, messages, and its end …, and commits as one decision, or nothing of it does" (`jig/07-connectors.md`, 07.2) needs the staged admission, its two roots and the callers' sweep the report counts, and the report names no capability beyond it. Size is now at most 600 lines, so the 1,121 counted (`logs/jig-07/size-atomic.txt`) land within twice it, as code-organization.md, 7 says: "An increment lands within twice its Size without asking". Option B is not taken: it would split work already written and validated, for no change in what lands. Unchanged: the rule that a file over 800 lines at the base grows by at most 40 lines (07.2's three such files are within it); the 800-line aim, under which `testing/jig-test-connector/src/boundary.rs` (757 at the base, +122) is a miss the report notes, the landing standing; the Don't list; the gate. A count that passes 1,200 is a question again, with the count by file.

137. **skein-01.s1's untaken-commands test fills the intake first** (plan defect, skein 01's block of 2026-10-16T13:49:12Z).
     Ruling 137: a plan defect in the test's shape, as the report proposes; skein-io is unchanged. `ui-text.md`, 7: "The terminal asks to read only once the host has taken every command it gave (`Terminal::commands`) ... A host that takes none holds the person's lines unread in the input". The terminal's demand stops, as the code does; skein-io's pipe still reads ahead into its bounded intake whatever the terminal demands (`crates/skein-io/src/pipe.rs:352-361`, at most `intake: BATCH`, 64, `terminal.rs:77`), and those bytes stay unread by the terminal. So the test fills the intake first: after the command is left untaken, feed and drive 64 `p` bytes; then `sim.peer_feed` of `p` returns 1 (the pipe's one byte) and of `\n` returns 0; once the command is popped and the terminal driven, the `\n` fits (returns 1). The test's name, `start(32, 1)` and its other test, `the_applications_lines_keep_their_room_while_the_output_is_held`, are unchanged. If the corrected test fails for another cause, that is a question, with the run's output.

138. **jig 09.1's Shapes against main, after jig 04.1** (ready check of jig-09.1).
     jig 09.1, re-checked after jig 04.1's landing on the files moved since ruling 126 (jig `805f1a5b7`, temper `9d7fcc789`), at jig `5568ba3f4`, temper `f37cea0d9`; corrected in `jig/09-templates.md`:
     - The `chats` member's users are 10 files in jig, 6 in `jig-protocol-config` (`git grep -l -E '<the plan's pattern>' origin/main -- '*.rs'`): jig 04.1's `a_chat_close_grace_past_the_tasks_limit_refuses_the_start` (`tests/domain.rs:248`) writes `close_grace_ms`; the Files already name it.
     - `jig-protocol-store/src/tests/fixtures.rs:504`, not `:503` (04.1's `grace` in `TASKS_LIMITS` above it).
     - The standard profile writes `templates: 16`, as Choices gives it: its `CORE` spreads `..small::CORE` (`profile/standard.rs:98-115`) and would inherit 8.
     - Checked by the sub-agent: the 09.1 sketches (`TemplateExecutor`, `TemplateBudget`, `TemplateDeadline`, `TemplateOverride`, `Template`, `merged`, `templates()`, `chat()`) pass clippy `-D warnings` in a scratch crate unchanged; `tasks::Limits::grace` is at `limits.rs:50`, the small profile's 600 s; `charters::chats` takes `limits.core.tasks.grace`; the six core `Limits` literals in jig are those listed, and temper's one at `temper-engine-domain/src/limits.rs:111`; nothing in jig matches `config::What` exhaustively; temper's `startup/problems.rs` is unchanged since `9d7fcc789`; `jig-protocol-config`'s default profile 79 of 79 and the proof test (0.33 s) pass at the base.
     A further user of a changed name or value that the Files miss, the implementer edits and lists in the commit body.

139. **skein's other origins answer their favicon** (process, skein 01's block of 2026-10-16T14:12:02Z).
     Ruling 139: process, a defect of skein 01.2's own harness, which this session wrote; skein 01 repairs it. The failing test's other origin serves one page, `/` with `<!doctype html><h1>Elsewhere</h1>` (`tests/ui-web/tests/browser_harness.rs:51`), and answers every other path 404 (`testing/skein-fake-person-browser/src/origins.rs:182-183`, tested by `an_unknown_origin_path_is_404`). The page loads no resource of its own, so the 404 the browser logged is its own request for `/favicon.ico`, which races the `See` step: the snapshot had already reached "heading Elsewhere". skein's other page server answers that path 204 (`tests/browser/src/pages.rs:26-31`). The repair:
     - The origin server answers `/favicon.ico` with 204 and no body when no page of the origin names that path, beside the 404 at `origins.rs:182-183`; a unit test beside `an_unknown_origin_path_is_404`, `an_origins_favicon_is_204` (`origins::response("/favicon.ico")` starts with `HTTP/1.1 204 No Content\r\n`).
     - A commit of its own, `Step: skein-01.s1`, before or after the terminal's commit; its lines are reported as outside skein-01.s1's Files and not counted in its Size.
     - `a_page_of_another_origin_is_opened_by_its_step` keeps its steps, assertions and expected values. Run it alone once with the README's browser command, then land both commits under `land-skein.lock`.
     - If the test still fails alone with the repair, or the logged 404 names another path: a question, with the run's output; no gate retried blind.
     This also serves jig 20's attack stories, whose pages are other origins.

140. **The count leaves out tests_*.rs modules** (plan defect, jig 02's block of 2026-10-16T14:38:10Z).
     Ruling 140: a plan defect in the count, neither option. code-organization.md, 7: "An increment's size is its hand-written production lines", and its jig command leaves out `tests.rs` and `tests/`, but not test modules named `tests_*.rs`: `jig-domain-authority`'s `tests_fixtures.rs`, `tests_numbers.rs`, `tests_party.rs` and `tests_policy.rs`, each `#[cfg(test)]` (`src/lib.rs:43-50`). The command now also leaves out `':!*/tests_*.rs'`. 02.4's count is then its 24 production lines (`allowed.rs` 14, `lib.rs` 2, `funding.rs` 8), within its Size of 40; Size stays 40, and the report gives the corrected command's number. Unchanged: every Size; the tests' and fixture's lines, which stay as written; the rule that a file over 800 lines grows by at most 40.

141. **temper 04.1's Shapes against main** (ready check of temper-04.1).
     temper 04.1's session file, checked at skein `054c06e`, jig `bc30abd58`, temper `187064e2` (temper's jig pin `2900e9b3b`, before jig 02.3, so `STORE_VERSION` is 1 and 04.1 bumps it); jig 09's names against `jig/09-templates.md`'s Shapes, not landed. Corrected in `temper/04-engine.md` (04.1, and 04.3's line numbers):
     - walking.rs's core `Settings` literal is at :256-263 (`close_grace` :257, `chat: 1` :260); `core_limits` ends at :143.
     - One field of temper's: `temper_engine_domain::Limits.templates` (`limits.rs:29-54`), filled at `temper-service/src/limits.rs:264-278` (`jig.core.templates`) and in the walking world's limits (:119, from the small profile); without it jig 09.1's core `worst_case` is `None` (`limits.rs:216`).
     - jig 09's refusals at the repin: `startup/problems.rs`'s two `config::What` matches (:81-102, :139-163) gain `What::Grace { template }` and `What::Operations { .. }`; `tests/startup.rs:117-140`'s `jig_refusals_print_words` a row each; walking.rs's `Config` (:245) `template_overrides`, `Settings` (:256) `templates`; `tests/fixtures.rs`'s `Project` (:436) `templates`, `ENGINE`'s policy (:65) `"templates": []`, its templates' reports' `words` 32 (`"tasks.result_bytes":32`).
     - The process tier: `crates/temper/tests/support/forgejo.rs:148` sets `tasks.delegates` 4 over jig's channel document, whose `people.operations` 2 after jig 09.4 is below 4 + 1: its overrides (:135-168) gain `("people.operations", 5)`.
     - The store bump's users: `src/tests/sizes.rs:48` (`assert_eq!(crate::STORE_VERSION, 1)`), `tests/codecs/tests/drift.rs:56-58`; the generated module `v1` moves to `v2` as jig's people family did, with its users (`lib.rs:8-10`, `records.rs`, `records/application.rs`, `tests/fixtures.rs`, `tests/records.rs`, `tests/codecs/tests/fuzzy_store.rs`); regenerated by the drift test's write mode (ruling 131).
     - The reference test: `Vec` is disallowed in temper (`clippy.toml:24`): `let labels: Box<[u32]> = …; assert_eq!(*labels, [1, 2], …)`.
     - Checked by the sub-agent, through heavy: the sketch, with stand-ins for jig 09.1's `Template` and `Engine.templates`, failed clippy on `Vec` and passed corrected (fmt, clippy `-D warnings`); `temper-service`'s default profile 154 passed; its fuzzy profile selects none. Not checked: corrections 2 to 5 against jig 09's code (not landed); store v2's `CoreRefused` bound.
     A further user of a changed name or value that the Files miss, the implementer edits and lists in the commit body.

142. **The process tier at its layer again** (the probe of ruling 111, 2026-10-16T14:58Z).
     Ruling 111's machine limit is gone ("For the user", E, done by the director): `/sys/fs/cgroup/runner/pueued` exists, owned by runner, with `cpu memory` delegated, and pueued (pid 851526) is in it; `heavy sh -c 'cat /proc/self/cgroup'` prints `0::/runner/pueued` (task 596). The probe, once, at temper `e73d3d5f` (task 598, `logs/20261016T145811-598.log`): `temper_open_signs_in_prints_the_inbox_and_exits_on_q` no longer fails on `Error::Other(16)`: `temper serve` starts and prints its link; `temper open` then panics at `crates/temper-client-shell-terminal/src/terminal.rs:72` (`Print::new(view_limits).expect("print bounds")`, in `Loop::new`), and the test fails at `crates/temper/tests/support/terminal.rs:285` ("terminal exited before the script"), 0.247 s. A defect on temper's `main`, which ruling 111's layer-down Proofs (temper 01.3, 02.1) did not reach.
     From now, temper's Proofs on the process tier run at their layer, as their Proof lines say; ruling 111's "one layer down" ends. temper 03.1, which replaces `Loop::new` with `skein_ui_text::Terminal::new` (whose `Print::new(view)?` takes the same view limits), finds why `Print::new` refuses temper's native view limits (`limits::view(&limits::native())`, `crates/temper/src/open.rs:92`), repairs it within 03.1, and runs its process Proof at its layer once, with temper 01.1's (`temper_open_signs_in_prints_the_inbox_and_exits_on_q`), naming the cause by file:line in its report. A cause outside 03.1's Files, in skein's `Print` or in a limit whose value the design gives, is a question, with the failing call's limits.

143. **The terminal's price, a path's segments, the templates' JSON: skein-01.s2 and jig-13.s1** (design gap, temper 03's block of 2026-10-16T14:53:32Z, jig 09's of 2026-10-16T14:57:59Z and the director's request of 2026-10-16T14:55:38Z).
     The designer's answers of 2026-10-16T15:20Z, landed as skein 8907ea3a, jig 7efaeef4d and 413726589, and temper 0625a5fa; the supplement steps are skein-01.s2 (`skein/01-s2-terminal-price.md`, the first answer's item 2) and jig-13.s1 (`jig/13-s1-path-segments.md`, the second answer's item 4), added by ruling 143:


     ### temper 03.1: skein's terminal's worst case and operations, 2026-10-16T15:20Z

     Landed: skein 8907ea3a (`docs/design/ui-text.md`, 1, 3, 4 and 8; `STATUS.md`); temper 0625a5fa (`docs/design/client.md`, 6.2 and 6.8; the plan's `README.md`, "Pins" and the sessions table; `temper/03-client.md`, 03.1; `jig/19-browser.md`, 19.4 and Done when; `skein/01-s1-terminal-room.md`).

     ### Decision

     1. **skein's terminal prices itself; an application's line has a byte bound of its own.** `ui-text.md`, 3: the settings gain "*its line bytes*: the most bytes of each of the application's own lines, without the name, `: ` and the newline the terminal adds (section 4); 0 for an application that prints none", and the terminal exports "`Terminal::worst_case`, the most heap the terminal holds, and `Terminal::operations`, the most operations it has in flight at once, which the host's ring holds beside its own. Each is none for settings or limits the terminal refuses, and none past its integer". 4: "The host says a line only within that room, and of at most the settings' line bytes. A line said beyond the room, or longer, is the host's defect". Not skein-ui's `text_per_node`: the line is the application's, not a node's text.
     2. **skein-01.s2, the supervisor's to add.** Needs: nothing (forgejo-17's skein step landed, `267730eb`). Files: `crates/skein-ui-text/src/terminal.rs`: `Settings.line_bytes: u32`; one private builder of `new`'s checks and io limits (`:72-91` at `267730eb`), used by `new` and by `pub fn worst_case(settings: &Settings, view: &skein_ui::Limits) -> Option<u64>` and `pub fn operations(settings: &Settings, view: &skein_ui::Limits) -> Option<u32>` (`skein_io::operations` of those limits); `worst_case` sums, checked: `skein_io::worst_case` of them; `Queue::worst_case` of completions 32, submissions 256, raw 256, events 32, commands `BATCH`, lines `lines + BATCH + 1`; `lines × (name + 3 + line_bytes)`; `(BATCH + 1) × (name + 40)` (the longest refusal, `: no [`, 20 digits, `] on the page\n`); `BATCH × longest` (commands' words); `BATCH` (a read); `List::<u8>::worst_case(longest + 1)`; a line's unwritten rest, the longest line said; `List::<NodeId>::worst_case(view.nodes)`; `render::worst_case(view)?.1`. `say` asserts `line.len() <= line_bytes`. `tests/ui-text/tests/terminal.rs`: `settings()` gains `line_bytes: 64`.
        - Tests there: `the_worst_case_counts_each_application_line_at_its_bytes` (at `fixtures::LIMITS`, `lines: 2`: `line_bytes` 65 gives 3 more than 64, two lines and a line's rest; `operations` is `skein_io::operations` of 2 sockets, 10; both `None` for a 129-byte name); `a_line_past_its_bytes_stops_the_terminal` (`#[should_panic]`, a 65-byte line at 64).
        - Proof: skein's gate, by `land`. Size: at most 80 lines. Don't: change `BATCH`, a refusal, `say`'s prefix or `Terminal::new`'s signature; give `line_bytes` a default; name a downstream repository.
     3. **temper takes it as it took skein-01.s1** (README, "Pins", "The terminal's room"): 03.1's first commit moves temper's skein to skein-01.s2's landing, which holds skein-01.s1's and the floor; jig 19.4 moves jig's skein there too, its repair `lines: 0, line_bytes: 0`, since the closing repin moves temper to jig's pin.
     4. **temper 03.1** (`client.md`, 6.2: "their line bytes, DIRECTORY's bytes and 160"; 6.8: "The host adds the terminal's worst case to its memory, and the terminal's operations ... to its ring's ... it copies none of the terminal's count"): Needs skein-01.s2; `Settings { .., lines: limits.lines, line_bytes }`; `run.rs` adds `Terminal::worst_case` and `Terminal::operations` of the same settings; `Loop::worst_case` keeps what `Loop` holds; `the_native_limits_make_the_terminal` checks both are `Some`. Size stays 200.
     5. Nothing is deferred.

     ### jig, before temper 04: path segments bounded by `client.rows`, 2026-10-16T15:20Z

     Landed: jig 7efaeef4d (`client/domain/contract.md`, 5.3; `client/domain/client.md`, 14 and 16; `client/domain/view.md`, 7 and 10; `protocol/client.md`, 3; `STATUS.md`); temper 0625a5fa (`docs/design/configuration.md`, 7 and its example; `client.md`, 5.4; `STATUS.md`; the plan's `temper/04-engine.md`, 04.4; `later.md`).

     ### Decision

     1. **A path is held whole; only the view cuts it.** `contract.md`, 5.3: "A document carries each whole, with as many segments as the engine admits in a name (the most of its authority's and its tasks' bounds on a name's segments): the engine never cuts one, nor refuses a document for one." `client.md`, 14: "a path's segments: ... a bound of its own, never a window's rows; the application sets it at least the most its engine admits in a name". `view.md`, 7: "When the phrase before them and their segments would pass a node's text, the view leaves out the first segments, shows in their place the mark `…` ... then the last segments that fit, the last one always. ... what the view acts on (a filter by a resource, a draft's pattern) takes the whole path".
     2. **Its limit:** `segments: u32` in the client's limits, beside `rows` and `text` (`jig_client_domain::Limits`, after jig 12.1), set by the application in code: the client has no profile. jig's fixtures give 4, the small profile's `authority.segments`, `tasks.hold_segments` and `tasks.authority_segments`. The engine gains no configured limit: its documents bound a path and a pattern by the most of those three, derived where its limits are made; `documents.lines` bounds short lists alone. temper's client gives 9.
     3. **jig-view's compose is not raised.** A path no longer needs rows, and its node holds "the phrase before a path, the mark and one segment at its most bytes" (`view.md`, 10). A longer text is deferred (`later.md`).
     4. **The step, the supervisor's to add:** a supplement step after jig 13.2 (`jig-13.s1`), whose landing temper 04.4 Needs; temper 04 starts after temper 03.4's repin past jig 13.2, so it waits on nothing more. Files, at jig `7efaeef4d`'s code (the step follows them through 12.1's and 13.2's moves):
        - link: `limits.rs:31, :41, :79`, `translate/documents.rs:312, :330`, `translate/pages.rs:84, :108`, `translate/pages_part7.rs:107, :117` take `segments`;
        - view: `owner_chip.rs:39-45, :95-97`, `resource_names.rs:9-11`, `owner_amendment.rs:27-29`, `owner_historical.rs:571-573`, `sections/project/owner.rs:203-205` take `segments`; the cut where paths are written (`words/held.rs:118-122`, `owner_authority.rs:117-125`, `policy_edit.rs:107-108`); `limits.rs:77-80`'s node text holds a phrase, the mark and `literal_segment::max_bytes(text)`;
        - core: `documents/task_tree.rs` `path` (`:328-340`) and the approval part (`:404-411`), and `project_note.rs`' `path` calls, by the names' bound;
        - tests: a task holding a name of `authority.segments` segments, at `documents.lines` 2, has its task line and `task` document served; the link decodes a path of `segments` segments at `rows: 2`, and refuses one more as too large; a path past a node's text shows `…` and its last segments, and the notes filter made from it carries the whole path. Size: aim for at most 160 production lines; past it, report why. Don't: cut a path in the client's data or the engine's documents; change what `rows` bounds besides paths; raise `compose`.
     5. **temper** (`configuration.md`, 7: "`documents.lines` bounds a document's short lists alone, at most the client's rows ... temper's client takes a path's segments, 9, apart from its rows"; the example names the three name bounds 8): 04.4 Needs the step, sets `segments: 9`, and holds the engine's name bounds at most it; no `documents.lines` for a path.
     6. Deferred: the reference UI's text past 128 bytes in temper's terminal, `later.md`'s new row (jig's `STATUS.md` too); the `documents.lines` row goes.

     ### jig 09.1: the templates' JSON in configuration.md, 10, 2026-10-16T15:20Z

     Landed: jig 413726589 (`protocol/configuration.md`, 3.1 and 10); temper 0625a5fa (the plan's `jig/09-templates.md`, Choices and its question).

     ### Decision

     1. **Section 10 takes 09.1's form.** It had the template's form since jig `0ec80c047`, but not a verdict choice's `followups` and `fields`, which `tasks::Verdict` holds and 09.1 reads; it now reads `{"verdict": {"choices": [{"code": c, "words": n, "followups": n, "fields": [{"name": s, "max": n}]}]}}`, writes jig's goal beside its chat, and says "jig's defaults are written, never implied: the `templates` member holds each template the engine has, jig's among them, and an application drops one of jig's by leaving it out (3.1)".
     2. Nothing in 09.1 changes; nothing is deferred.

144. **The land lock as one command; a test's helpers once; bit sets named** (steer of 2026-10-16T15:30Z, from the check of 2026-10-16T15:08:55Z).
     The check of 2026-10-16T15:08:55Z's three findings become README rules, "Rules of this plan": "One land at a time in each repository" gains that the lock is taken only as the one `flock … land …` command, with no shell or other command under it, and validation before it; "Named numbers in tests" gains bit sets and masks as `const`s built from named bits; and a new rule, "A test's helpers once", has a session search the fixtures, `tests.rs` and the world's `src/` for what a helper sends or builds before writing one, with the first jig increment that adds a test under `tests/core/tests/` moving `ask_holder` (`tests/core/tests/fields.rs:63-70`) into `tests/core/src/fixtures.rs` as `pub fn ask_as`. Checked: `git -C jig grep -n 'fn ask_as\|fn ask_holder' origin/main -- tests/core/tests` printed `fields.rs:63`, `refusals.rs:12`, `stories.rs:19`; `pub fn decide_proposal` is `tests/core/src/effects.rs:472`; `rules.rs:165` documents bits 1, 2, 4, 8 and 16 as Batch, Effect, Widen, Amend and Escalation.

145. **jig 07.2's land waits its turn, outside a shell** (infrastructure, jig 07's block of 2026-10-16T15:24:13Z).
     The holder's wait was not stale, and it has ended: jig 08's locked shell (PID 4041993) waited for its own land's gate, landed jig-08.2 (`e1b483760`), and released `land-jig.lock` at about 15:26Z. The lock is now held as one command, `flock … land` (jig 05, PID 4050192), with jig 11's `flock … land` (PID 4104402) waiting behind it; jig 07's locked shell is gone. Nothing is killed and no infrastructure is recovered. jig 07 follows the steer of 2026-10-16T15:30:25Z, point 1, and the README's "One land at a time in each repository" as ruling 144 amends it ("The lock is taken only as that one command ... Validation, the memory literal's run (ruling 110) and the focused tests run before it, on the session's own rebase"): it refreshes `main`, rebases and validates outside the lock, then runs `flock /srv/data/git/runner/src/tmp/temper/client-api/land-jig.lock land …`, which waits its turn after jig 05's and jig 11's. Ruling 136 stands for 07.2's Size.

146. **jig 03's Shapes against main, after jig 02** (ready check of jig-03.1).
     jig 03's session file, re-checked against jig `45b1479f3` (jig 02's last landing, 02.4) and temper `df37f032` by a read-only sub-agent, then against jig `e1b483760` (jig 08.2) for the files jig 08.2 moved, is corrected in `jig/03-parties.md`: the wire is v8 since jig 11.3 (`cec2a5385`), so each "v7" reads v8, the tests `a_v8_amend_*` included, the Choices holding under v8 (`RefusalAuthority.proposable` still `option bool`, `AmendProposed` and `LackingTaskKind` unchanged, no Create, Propose or Withdraw ask); line numbers moved by jig 02.3 and 08.2 (`acts/bounds.rs` +3 after :232-234, `routing/mod.rs`, `routing/parties.rs`, tasks' `boundary.rs:1278`, `overrides.rs` :174 and :195, `tests/core/src/world.rs:178-187`, `stories.rs` :981, :1224, :1313, `tests/tasks/src/world.rs` :192, :73); 03.1's Files gain `src/tests/overrides.rs`'s `SMALL` row `(b"people.batch", 2)` and the values derived from people's worst case (`JOURNAL`, `STORE_KV`, `src/tests/domain.rs:169`); 03.4's gain `translate/answers.rs:81-84`'s `GoalProposed` arm; the joint wire world's `outcome` (`tests/client/src/joint/wire/outcomes.rs:17`) answers `unreachable!` for `Created`, `Proposed` and `Withdrawn`, which no v8 ask leads to; 03.1's `live` is the landed `jig_core_world::fixtures::live_row`; the no-holder tests stand on jig 02.3's `no_holder_fixture`; sketch 1's people `Flags` gains `PLAIN`, as the tasks child's. Checked: the sketches with `PLAIN`, `heavy cargo clippy -p jig-domain-tasks -p jig-domain-authority -p jig-domain-people -p jig-domain-core --all-targets -- -D warnings`, passed at `45b1479f3`; `nextest -p jig-domain-people -p jig-domain-tasks`, 105 of 105; temper's search S18 still prints 28 files (`git grep -l 'people::Ask::' origin/main -- '*.rs' | wc -l` at temper `47e0e184`), unchanged. Refresh any further copy the gate names, and list it in the commit body.

147. **jig 09.2's Files take the store format test's bounds** (plan defect, jig 09's block of 2026-10-16T15:34:54Z).
     Option A, as a plan defect: 09.2's Files miss `crates/jig-protocol-store/src/tests/format.rs`, whose five expected values are the fixture store's value and page bounds that 09.2's `templates: list 16 TemplateOverride` in `PolicyValue` raises. They are now in its Files: value `80_949` at `:76`, `:81`, `:90` and `:201`, page `81_061` at `:86` (jig `e1b483760`; `:203` on 09.2's branch), each set to what the new code computes (value 81_241, page 81_353 by its report). They are derived values of the Shape, not implementer.md 8.2's "The case the Shape changes" ("at most 4 assertions in all"), so that allowance stays whole for 09.2's other tests. The memory sum (`What::Memory { over: … }`, `crates/jig-protocol-config/src/tests/domain.rs:169`) is ruling 110's read-and-set, as before. Refresh any further literal derived from the policy record's bound that the gate names, and list each in the commit body.

148. **skein-01.s2's Shapes against main** (ready check of skein-01.s2).
     skein-01.s2's session file, checked against skein `8907ea3`, jig `e1b483760` and temper `973c21a6` by a read-only sub-agent, is corrected in `skein/01-s2-terminal-price.md`: `say`'s assert has the message "application line within its bytes", and `a_line_past_its_bytes_stops_the_terminal` is `#[should_panic(expected = "application line within its bytes")]`, since a bare `#[should_panic]` fails `cargo clippy --workspace --all-targets -- -D warnings` (`clippy::should_panic_without_expect`); `worst_case` and `operations` are `Terminal`'s associated functions, `skein_ui_text::worst_case` being render's (`lib.rs:8`); jig's `tests/client-protocol/src/native.rs:443` copy of the operations, `10`, is named for jig 19.4. Checked in a scratch worktree with the whole sketch applied: the expected values hold (3 more for `line_bytes` 65 than 64, `operations` 10, both `None` for a 129-byte name); clippy with `-D warnings` on every package that depends on skein-ui-text passes after the correction; nextest, 105 of 105; the sketch is 84 lines against "at most 80", within twice Size. Refresh any further copy, and list it in the commit body.

149. **jig 23's Shapes against main, while jig 03 runs** (ready check of jig-23.1).
     jig 23's session file, checked against jig `e1b483760` and temper `24226aaa` by a read-only sub-agent, is corrected in `jig/23-referee.md`: `referee.rs` is 1,116 lines; with `Snapshot.party_proposals`, `Observed` takes `#[expect(clippy::large_enum_variant, reason = …)]` (`referee.rs:276`), as `crates/jig-protocol-tools/src/lib.rs:42` does, and `Source` derives `PartialOrd, Ord` for the credits; `party_proposals` takes the `Pending` rows alone (`proposals.rs:150`); 23.1's Files gain the three tests of `tests/core/tests/referee.rs` that commit an unasked task proposal (`:462-463`, `:594`, `:605` through `proposed_batch` `:575`), each sending `Observed::ProposeAsked { proposer: Source::Task(1) }` first, and the rule runs after `durable`'s Lost check (`:550-558`) so that `:417` passes; the procedure exemption is `task.executor.0 == 1` (`tests/core/src/observations.rs:146`); 23.2's literals are `tests/core/tests/referee.rs:51` and `tests/core/src/observations.rs:138`, which the other adapters reuse; the one-process adapter (`tests/one-process/src/referee.rs:144-151`) is unchanged; both Proofs add `heavy cargo nextest run -p jig-core-world -p jig-one-process-world` once under each profile, since every core-world story runs the referee. Checked by the sub-agent in a scratch worktree: clippy `-D warnings` on the conformance, core, ops, one-process, client and client-protocol worlds failed without the `#[expect]` and passed with it; `jig-core-world --test referee` passed 3 of 4 before the three tests' asks (`:601`, "proposal 9 of Task(1) waits unasked") and 4 of 4 after. Not checked: the other eight violation tests and the adapters' code, which jig 03.5's landing changes. Refresh any further copy, and list it in the commit body.

150. **A run watch holds the newest turn alone** (design gap, jig 11's block of 2026-10-16T15:58:06Z).
     The designer's answer of 2026-10-16T16:30Z, landed as jig bab2e89cb and temper 9ae5fca0:


     ### jig 11.5: the provisional text a run watch holds, 2026-10-16T16:30Z

     Landed: jig bab2e89cb (`docs/design/client/domain/client.md`, 8.3 and 10; `contract.md`, 7.3); temper 9ae5fca0 (the plan's `jig/11-client-domain.md`, What the design asks and 11.5's Scope, Files, Shape, Test shape and Don't; `jig/15-helpers.md`, 15.4's Scope).

     ### Decision

     1. **One turn, the newest.** The supervisor's reading holds. `client.md`, 8.3, now: "It holds that one turn alone, the newest delivered: a delivery naming a later turn, or a later retry of the same turn, takes the place of what it holds, from its own offset; one naming an earlier turn, or an earlier retry, changes nothing, and nothing is told. Names are ordered by attempt, then number, then retry." That order is `ProvisionalTurnName`'s derived `Ord` (`crates/jig-client-domain/src/contract/watches.rs:81-86`: `attempt`, `number`, `retry`).
     2. **11.5's Shape stands, with one `RunText` per engine watch of a run:** `Domain::text(handle) -> Option<&RunText>`, `Limits.turn_bytes` the one bound, no list of turns and no bound on their number. The subject already asks from one turn and one offset (`Subject::Run { task, turn: Option<ProvisionalTurnName>, offset }`, `watches.rs:27`), and `client.md`, 14, bounds "a run's provisional turn".
     3. **Earlier turns are the thread's.** `client.md`, 8.3: "Keeping each turn's text until its committed turn takes its place is the thread's (section 10), fed the run's text at each `Text`"; 10: "the thread keeps each turn's line, since the client holds only the turn being written (8.3)"; `contract.md`, 7.3: "`jig_client::thread` keeps a turn's provisional text until then". `Conversation::stream`'s list of live turns (`self.live`) and its fence on the committed turn (`conversation.rs:310-314`) move with `Conversation` into the thread (jig 15.4, whose Scope now says the list stays). 11.5 follows only its offset rules for one turn: text past the offset taken once, a gap only with a cut (`:357`), a later retry replacing (`:340`). The client reads no activity, so it has no committed turn to fence on.
     4. **What "never resets an earlier one's" asserts.** `client.md`, 8.3: "A later turn's text starts under its own name: it is never appended to the earlier turn's, nor taken as that text cut back to its start, and what the application read under the earlier turn's name stays that turn's." `a_later_turn_s_text_never_resets_an_earlier_one`: turn 1 `ab` from 0, turn 2 `c` from 0, then turn 1 `d` from 2, reading `text(RUN)` after each: turn 1's `ab`; turn 2's `c` from 0, neither `abc` nor under turn 1; turn 2's `c` still; `Text { handle: RUN }` told twice in all.
     5. **The later-retry test also delivers the earlier retry after it:** `text(RUN)` unchanged, nothing told. No test is added; 11.5's Size stays 150.
     6. **11.5's Don't** gains: "no list of turns and no fence on a committed turn in the client, which reads no activity (the thread's, jig 15.4)".
     7. Nothing is deferred: no `later.md` row.

151. **Every crate's lints in one round** (steer of 2026-10-16T16:45Z, from the check of 2026-10-16T16:24:41Z).
     The check of 2026-10-16T16:24:41Z's third sighting of one-crate-a-round lint failures (jig 03.1's seven rounds since 15:55Z, each showing only the crates the last fix let build) becomes a README rule, "Every crate's lints in one round": after the workspace check, the first scoped clippy round runs with `-- --cap-lints warn`, every warning is fixed, then `-- -D warnings` once confirms; land's prebuild keeps `-D warnings`. Checked: in a scratch two-crate workspace whose second crate depends on the first, both with a denied clippy lint, `cargo clippy --keep-going -p a -p b -- -D warnings` printed `error: could not compile 'a'` and linted no `b` (exit 101); `cargo clippy --keep-going -p a -p b -- --cap-lints warn` printed the warning at `a/src/lib.rs:1:21` and at `b/src/lib.rs:1:21` (exit 0).

152. **jig 07.3's proposals ride tasks::ProcedureDecision** (plan defect, jig 07's block of 2026-10-16T16:51:54Z).
     Yes, as a plan defect: 07.3's Shape asks that a proposal "refused as 'no proposal could be accepted' refuses the step (`StepRefusal::Refused`)", and names no carrier for the proposals into the tasks child. The default is jig 07's proposal: `tasks::ProcedureDecision` (`crates/jig-domain-tasks/src/boundary.rs:277`) gains `proposals: Box<[tasks::Proposal]>` beside `amendments` and `messages`, and `tasks::procedure` checks and installs them with the step's other parts, so a refusal, an existing proposal or an inbox reservation refuses the whole step. `ProcedureDecision::new` (`:305`) gives `Box::new([])`, so its `..ProcedureDecision::new(..)` users change nothing; the whole literals gain `proposals: Box::new([])` or the step's: `crates/jig-domain-core/src/routing/procedure_steps.rs:87`, `crates/jig-domain-core/src/tests/acts/proposals.rs:80`, `tests/tasks/tests/inbox.rs:517`, `tests/tasks/tests/procedures.rs:126`, `:147`, `:176` (jig `055b8a547`). These are in 07.3's Files now; a further one gets the same, listed in the commit body. Not the core-only carrier: it would need a second admission API across the boundary. Size and the rest of 07.3 stand.

153. **temper's engine takes jig 06's revisions in temper-03.s1** (plan defect, temper 03's block of 2026-10-16T17:02:26Z).
     Ruling 153, a plan defect: the implementer's option B. The repin 03.2's Needs name (jig 10.3, `94eaa0340`) brings jig 06.1 (`350da6c54`) and 06.2 (`4b2ef1e69`), whose tools and answers no temper step carries; the README's 200 lines ("What a temper repin repairs") were not sized for them. jig's design answers what they are in its own words (`domain/engine.md`, 7.3): "`amend`, `release` | a task below it: an amendment naming its spec's revision as the run last read it, but for a change of priority alone; a release naming its hold's, and, for an effect held uncertain, whether to make it again | done, with the task's spec revision after the commit, the one its next amendment names (a release moves none); refused, and why; moved, with what it is now"; "`decide` | ... | done; beyond its authority, and passed up; moved, with what it is now".

     1. **A supplement step, `temper-03.s1`** (`temper/03-s1-engine-revisions.md`), in session 03, before 03.2. It makes the repin commit onto `94eaa0340` (the one already made on `client-api/temper-03-2` moves to it), with the repin's other repairs within the README's 200 lines (the `EndedByParty` arm among them), and carries jig 06 through temper's engine tools as those two commits adapted jig's reference code: `amend` and `release` name a `revision` (`release` an optional `again` too) through `calls.rs`, `Tool`, and `route/tools.rs` to `NamedAction::Amend` and `NamedAction::Release`; `CallAnswer` gains `Again`, `PassedUp`, `Revised`, `Moved`, mapped both ways, taken as core-owned, and rendered to the run. Its file gives the Files, tests, proof and size (aim: 220 production lines besides the repin's repairs).
     2. **Unchanged:** temper's store format (`store-v1.schema`'s `CallPayload` holds the connector's answers alone; jig's core v3 keeps the four), `STORE_VERSION`, temper's tool names (`decide` and `decide_escalation` stay two), `ProposedAction`.
     3. **temper 03.2** needs temper-03.s1's landing: its pin then holds jig 10.3, it makes no repin commit, and its 150 lines are its Shape's.
     4. **A further user** of a changed tool or answer that the compiler names gets the same change, listed in the commit body.
     5. **Deferred:** temper's `domain/engine.md`, 7.3's tools table, written before jig 06, names neither the revisions nor "moved": a `later.md` row for the designer's next landing in temper.

154. **temper's skein may be ahead of this plan's target** (the director's request of 2026-10-16T17:16Z).
     The director's request, for the user, before forge-setup's first skein step lands: temper's skein may be ahead of this plan's target for it. Amends the plan's `README.md`, "Pins" (the repin's skein update and check, "temper's skein ahead", the closing repin), and the repin checks of `temper/03-client.md`, `04-engine.md` and `05-server.md`.

     1. **temper's skein may be ahead** of the floor, skein-01.s2's landing (`8a10437f`) or jig's pin: another temper plan moves it forward, forge-setup's first (skein lib's `sha256::Hmac` and `sha256::equal`, additive).
     2. **No skein move of this plan takes it back.** Each repin's `heavy cargo update -p skein-lib --precise <target>` is not run when `git -C ~/src/rust/skein merge-base --is-ancestor <target> <temper's skein>` succeeds; the check of skein's line reads that command, not an equality. The closing repin moves skein to jig's pin only where that is forward.
     3. **Unchanged:** jig's and smith's lines, read as before; jig's own pin of skein and jig 19's move; a build failure in a jig or smith crate on temper's skein is a question.

     Checked: `git -C temper show origin/main:Cargo.lock | grep -o 'forgetic/skein.git#[0-9a-f]*' | sort -u` → `skein.git#8a10437fa55735f83502cd189b0314fbcf3af5fc` (skein-01.s2's landing, `8a10437`); jig's at its 10.3 landing → `skein.git#f12f29d4…`; jig's and temper's `Cargo.toml` name skein as `{ git = "https://github.com/forgetic/skein.git" }`, no `rev`: one source, which temper's lock pins.

155. **temper-03.s1 carries the client's repin repairs** (plan defect, temper 03's block of 2026-10-16T17:18:34Z).
     Ruling 155, a plan defect, the implementer's proposal: yes. Ruling 153's Don't ("touch temper's client crates (03.2's)") missed that the 10.3 pin holds jig 11.1 and 11.2, which landed before jig 10.3: their client changes reach temper with this repin, not 03.3's.

     1. **temper-03.s1 carries the client crates' repairs that the repin needs to compile and pass the gate,** taken from 03.2's draft on `client-api/temper-03-2`, within the README's 200 lines of a repin's repairs: the contract's path, `jig_domain_client::contract` to `jig_client_domain::contract` (jig 11.1's `sed`, as 03.3's Files count it: 21 files, 40 lines), `Event::Hello`'s users (it has `served` alone now), and the shell's repairs the compiler names next.
     2. **Unchanged:** 03.2's behaviour beyond that (its tests and its process Proof) stays 03.2's; s1's engine Shape, its tests and its proof; the 200 lines.
     3. **If the client's repairs pass 200 lines, or the gate needs more of 03.2's Shape:** land temper-03.s1 and 03.2 in one `land`, s1's commits first, each step's last commit with its own `Step:` trailer, rather than asking again.
     4. **03.2 and 03.3** do not do again what s1 carried; 03.3's Files are what remains of them.

156. **A party's Create: no references; its inputs' load keyed by its token** (plan defect, jig 03's block of 2026-10-16T17:27:01Z).
     Ruling 156, a plan defect on both headings: the design answers them, and the kit has the shape.

     1. **References: a `Create` names none.** people.md, 5.1, landed before the run (jig `0ec80c047`): "A party's `Create` may name, as a dependency, a live task, or, as an input, an ended task's result: a task in their tree, or any task of the project for a maintainer, as a task's batch reaches its delegates and the tasks it was introduced to (tasks.md, section 4 and 8.5); a batch gives its tasks no references." The session's default holds; nothing goes to the designer.
     2. **Inputs: the implementer's proposal.** A core `Request` keyed by the party request's `Token`, with its person, project and its bounded unique input task numbers, as `Request::HistoricalPersonTask { request: Token, person, project, task }` is keyed; a matching `Event` with the stubs `DelegateValidated` gives, or the failure `DelegateInputFailed` gives, and `busy`, as the `Historical*` events carry it. The core keeps the checked creation until it returns. tasks.md, section 3: "While a task that ended is named by a live one (… an input …), the core keeps a stub of it".
     3. **The roots:** each root that answers `HistoricalPersonTask` answers the new request the same way (`git grep -l 'HistoricalPersonTask' -- '*.rs'` names them; 03.2's Files list them). Their routing counts beside the 290 lines, aim for at most 80; past it, the report says why.
     4. **Downstream:** temper's root (`crates/temper-engine-domain/src/{archive.rs, route/now.rs}`, `crates/temper-service/src/clients/translation.rs`, `tests/client/src/translate.rs`, which answer `HistoricalPersonTask` today) follows at temper's first repin past jig 03.2's landing, as the README's "What a temper repin repairs" says. jig 03.2 does not name temper.
     5. **Unchanged:** `DelegateValidated` and `DelegateInputFailed` for an agent's delegates; the wire (v8); `routing/create.rs` past 80 lines of inputs is still raised in the report.

157. **The inputs' load answers with rows; the core judges reach** (plan defect, jig 03's block of 2026-10-16T17:44:41Z).
     Ruling 157, a plan defect: the implementer's preferred option. Ruling 156 gave the answer event the stubs; a stub (task, phase, result) cannot show reach, and a root must not judge it.

     1. **The answer event carries the rows the root loaded,** as `Event::HistoricalPersonTask` carries its `row: Option<Box<tasks::TaskRecord>>`: each input's ended `TaskRecord`, its tree's root's where that root has left live memory, `None` for a row not found, and `busy`.
     2. **The core judges reach from them and builds the stubs,** or refuses as `DelegateInputFailed` does. No validator of the core's runs in a root, and no root holds a pending loader state of its own beyond what it holds for `HistoricalPersonTask`. root.md, 11, "What a root never does": "decide anything of the core's or a connector's".
     3. **The shared historical transport** (`crates/jig-protocol-store/src/historical.rs`) loads the rows one `Get` each, as it loads one today.
     4. **Unchanged:** ruling 156's request (keyed by the party request's `Token`, with person, project and the bounded unique input task numbers), its roots, its 80-line aim and its downstream rule; the agent's delegate path.

158. **jig 12's Shapes against main** (ready check of jig-12.1).
     jig 12's Shapes against main, the re-check of jig-12.1 at jig `755e02e53`, temper `dceef129b` (a sub-agent's clippy and nextest runs in a scratch worktree, through heavy). Amends `jig/12-client.md`:

     1. **Sketches:** `assert!(matches!(..))` (12.1's unit sketch, 12.2's story) becomes a `let [..] = .. else { panic!(..) };`: the root `clippy.toml:84` bans `core::matches` in step crates, tests included. 12.5's `debug_assert!` becomes `assert!` (`clippy.toml:87`).
     2. **12.5's name:** `typed` becomes `unlinked` (`Client::unlinked`, `src/tests/unlinked.rs`, `an_unlinked_client_*`, the Proof's filter): `tests/boundary/src/names.rs:36` forbids "Typed" in any identifier; with the rename `kit_sources_keep_the_kit_vocabulary` passes.
     3. **Files added:** 12.5, `tests/boundary/src/tests/graph.rs:297-341`'s allow list gains `"jig-client"`; 12.1, temper's terminal's `client.rs:88-100` (it copies the link's `store.rs:36` budget); 12.3, the link's `src/store.rs:170-172` and `jig-client-protocol-http/src/tests/fixtures.rs:15-21`; 12.2, the link's `src/tests/fixtures.rs:96-107, 130-141, 161-173`.
     4. **Moved lines** in 12.1 and 12.2, refreshed. A further line that moved is followed by its name, and listed in the commit body.

     Checked: the sub-agent's runs: clippy `-D warnings` on a scratch `jig-client` with 12.1's sketch failed on `matches!`, passed with the `let`-`else`; `debug_assert!` failed, `assert!` passed; nextest on `jig-boundary` and `jig-client` with 12.5's dependency failed `client_world_joins_only_the_client_and_test_roles`, then `kit_sources_keep_the_kit_vocabulary` on `typed`; after both corrections, 73 passed. Not checked: 12.3 to 12.5's code compiled; the workspace gate.

159. **jig 04.4's Size is 500** (plan defect, jig 04's block of 2026-10-16T17:55:19Z).
     Ruling 159: option A, a plan defect, as ruling 136 was for 07.2. 04.4's Size was set below what its Shape asks: "`Staged { closing, moved, reserved, revisions }` ... 4.3's checks take `staged: &Staged`", with tests that make an amendment then a release of one held task, and words then a stop release, in one request, under the Don't "clone the domain or a task record to check". The staged view those checks read, without a copy of a task record, is the work the report counts (`compound.rs` 303, `compound/staged.rs` 338 of 886), and the report names no capability beyond 04.4's Scope. Size is now at most 500 lines, so the 886 counted land within twice it, as code-organization.md, 7 says: "An increment lands within twice its Size without asking". Option B is not taken: it would split work already written, for no change in what lands.

     Unchanged: the rule that a file over 800 lines at the base grows by at most 40 lines (`control.rs` 897 at the base, +50 −38; `domain.rs` 1283, +9; `boundary.rs` 1839, +14 −2: each within it); the 800-line aim; the Don't list; the Proof; the gate; implementer.md, 8.6: "Never trim a doc comment, a test or the formatting to meet a Size". A count that passes 1,000 is a question again, with the count by file.

160. **temper's terminal rows is 3 until temper 04.4** (plan defect, temper 03's block of 2026-10-16T18:03:17Z).
     Ruling 160, a plan defect in the order of steps. temper's design says what its client does: `client.md`, 5.4: "A path's segments have a bound of their own, 9, apart from the windows' rows ... so a page of the forge's work is never refused for a path". Ruling 143 put that bound in jig-13.s1, after jig 13.2, on the premise that temper 04 "waits on nothing more"; but temper 03's repin onto jig `94eaa034` already delivers a delegate chip holding a forge path. `hub_name` makes each forge hold three segments, forge, repository and branch (`forge_route.rs:609-611`), and jig at the pin bounds a receiving path by the client's rows (`limits.rs:31`, `translate/documents.rs:330`), against the terminal's `rows: 2`. jig-13.s1 cannot be pinned: it waits on jig 12 and 13.2, neither landed. An upstream amendment to the pin is not taken: temper pins jig's `main`.

     Interim, until temper 04.4:
     1. The combined s1/03.2 landing sets the terminal's `rows: 3` (`crates/temper-client-shell-terminal/src/limits.rs:51`), its comment "interim 3 for a forge hold's three segments, ruling 160; temper 04.4 returns it to 2 with `segments: 9`". It refreshes the values derived from it (the worst-case sums of `the_native_limits_pass_each_worst_case` and `the_native_limits_make_the_terminal`, the render envelope check) and lists each in the commit body; those lines are outside the repin's 200.
     2. The check, once, before anything else: with `rows: 3`, `heavy cargo nextest run -p temper-client-shell-terminal`, then `heavy cargo nextest run -p temper-one-process-world -E 'test(an_adopted_repository_gets_its_change_pushed_through_git)'`. If `Limits::compose` refuses 3 (the native limits' test fails), or the story still fails on a path, stop and report it as a question with the failing value; try nothing else.
     3. temper 04.4 sets `segments: 9` and returns `rows` to 2 (its Files say so now).

     Unchanged: every other native bound; the engine's `documents.rows` and `documents.lines`, 2, which `crates/temper/tests/configuration.rs:17-18` keep at most the native rows; every story assertion; ruling 155's combined landing; jig-13.s1 and its place. A further value derived from `rows` gets the same refresh, listed in the commit body.

161. **temper 03 names the native operation that refuses rows 3, then reports** (plan defect, temper 03's block of 2026-10-16T18:16:16Z).
     Ruling 161, a plan defect: ruling 160's step 2 met its stop. With `rows: 3`, `limits::worst_case(&bounds)` is `None` (`crates/temper-client-shell-terminal/src/client.rs:85`). Which operation refuses decides the repair: a native bound of temper's own (`limits.rs:28-60`) is a plan value the supervisor may change; a product inside jig at the pin (`94eaa034`) is not reachable from temper, which pins jig's `main`. So the repair waits on one diagnosis, at most 20 minutes:

     1. In the temper-03-s1 worktree, `rows: 3` as it is, add one throwaway unit test to `crates/temper-client-shell-terminal` that evaluates from `limits::native()` each operand of `limits::worst_case` (`limits.rs:134-144`) in its order: `view`'s three (`english`, `Words::english`, `temper_client_view::Limits::of`), then `client::worst_case`, `temper_client_view::worst_case`, `link`, `link::worst_case`, `shell`, `Set::worst_case`; it prints each as `Some(n)` or `None`. Run it once: `heavy cargo nextest run -p temper-client-shell-terminal -E 'test(<its name>)' --no-capture`.
     2. For the first `None`, follow its function into the code at the pin (jig's `Cargo.lock` pin, or temper's own) to the one checked operation that refuses: its file:line, the operation, its operands' values at `rows: 2` and at `rows: 3`, and the native bounds of `limits.rs:28-60` that enter each operand.
     3. Report that as a question, with the test's output; delete the test; commit nothing new. Try no repair: no other bound changed, no expected value refreshed.

     The supervisor then rules the repair from the report. Unchanged: ruling 160's `rows: 3` and its comment, uncommitted; ruling 155's combined landing; the private commits 6b487696, e6b3c96f, 5fbd0459.

162. **jig 13's Shapes against main** (ready check of jig-13.1).
     Ruling 162, the ready check of jig-13.1: jig 13's Shapes against main (skein `04c1562ed`, jig `23e3fbbdb`, temper `0d67b373`; the check's 17 items are in supervision.md, "Ready check of jig-13.1, returned 2026-10-16T18:16:04Z"). Landed in `jig/13-reference-ui.md` and `jig/13-s1-path-segments.md`:
     - 13.1: Removes six records and five enums, by name; the sketch's `Record` as the page's `store::Record` has it (`jig-domain-client/src/store.rs:16-24`), `version` included and `words` a `contract::Text`, with `#[derive(Clone, Debug, PartialEq, Eq)]`, and `Copy` on `RecordKey`; the view's `src/tests/fixtures.rs` is new; the missed users in Files (`fuzzy_client_wire.rs:25,27-35`, the wire's `tests.rs:229,244-255`, the link's `fixtures/link_store.rs` and `tests/store.rs`, `boundary.rs:43,101`); temper's `route.rs` 21 lines, and `tests/family.rs:147-157`.
     - 13.2: a rename rule: where both crates have a test module of one name, the moved crate's go under `src/tests/domain/` with their names, its `src/tests.rs` becomes `src/tests/domain.rs`, and the commit body lists each; `tests/boundary/src/tests/graph.rs` and `tests.rs:142` in Files; the counts at `23e3fbbdb`; "prints nothing first" needs 12.1 and 13.1; temper downstream 191 lines in 38 files.
     - 13.3: `limits`, `document_value`, `read_answer`, `read_answer_value` move with `request`; `to_root` stays in `jig-test-service`; the missed users in Files.
     - 13.4: the missed users in Files, the root's 25 `jig_client_domain` uses in 12 files to `jig_client::`; `Client::unlinked` (12.5's name) for `Client::typed`; the bridge's `Told::Text` and `Told::Dropped` are named, their present events settled by the re-check after jig 12.5, as a steer before 13.4 starts.
     - s1: `profile/small.rs:47, :67, :119`; the `resource_path_fits` callers and `owner_historical.rs:487`, `owner_authority.rs:95-103` in Files.
     Not checked: 13.2's and 13.4's compiles and path-rewrite counts; a fresh check takes them after jig 12.5 lands. A further copy or user found when reached is refreshed and listed in the commit body.

163. **temper's terminal rows stays 2; the forge-path story waits for temper 04.4** (plan defect, temper 03's block of 2026-10-16T18:34:11Z).
     Ruling 163, a plan defect in the order of steps. Ruling 160's interim cannot hold: at the pin (`94eaa034`), `crates/jig-client-view/src/limits.rs:48`, `limits.nodes.checked_mul(limits.text_per_node)`, a u32, refuses `rows: 3` (nodes 19,123,621 × 1,024 = 19,582,587,904, report of 18:33:20Z). No native bound of temper's own repairs it within an interim: `u32::MAX / 19,123,621` is 224, below the 554 that the phrase alone gives `text_per_node` (2 × 277), so `text` cannot; nodes would have to fall below 4,194,303, a fifth, through objects, parts, notices and the other bounds of the target. jig's repair is upstream of the pin, and the designed fix is jig-13.s1's `segments` (`client.md`, 5.4), which temper 04.4 takes.

     So, until temper 04.4:
     1. The terminal's `rows` stays 2: drop ruling 160's uncommitted `rows: 3` and its comment.
     2. The combined s1/03.2 landing marks the one-process story `an_adopted_repository_gets_its_change_pushed_through_git` (`tests/one-process/tests/one_process.rs:43`) `#[ignore = "a forge hold's three segments need segments: 9, temper 04.4 (ruling 163)"]`, as `testing/temper-throwaway-forgejo/tests/throwaway.rs:34` marks its own, and lands under the gate as ruling 155 says. A further test that fails only on a forge path's segments gets the same mark, listed in the commit body and the report; past three such tests, stop and report.
     3. temper 04.4 removes the mark, and its Proof runs the story once (its Files and Proof say so now).

     Unchanged: every native bound; every story assertion; ruling 155's combined landing; jig-13.s1 and its place; the private commits 6b487696, e6b3c96f, 5fbd0459.

164. **temper 03.3's repin takes jig 07.2's procedure step and jig 09's templates limit** (plan defect, temper 03's block of 2026-10-16T19:14:48Z).
     Ruling 164, a plan defect on both headings, the implementer's proposals: yes. The README answers both, "What a temper repin repairs": "what else jig landed since temper's last repin, adapted the way jig adapted its own reference code (`jig-client-test-domain`, `jig-test-service`, `ops`), as the commit of that change shows, in at most 200 added lines outside the Shape's Files". 03.3's pin, jig `755e02e53` (jig 11.5's landing), holds jig 07.2 (`055b8a547`), 09.1 (`b850f2edc`) and 09.2 (`8912bca2a`). No temper step names 07.2's move, and 04.1, which would have taken jig 09 at its own repin, comes after 03.3.

     1. **The procedure step:** temper's `PartyInput::ProcedureStep` takes jig 07.2's: `action: ProcedureAction` becomes `decision: ProcedureStep` (jig `crates/jig-domain-core/src/delegation.rs:39`); temper's `lib.rs:39` re-exports `ProcedureStep` and `StepEnd` in place of `ProcedureAction`; the forge's route (`route/forge_route.rs:1835-1906`) makes each step as 07.2's commit makes its reference roots' (one delegate is a batch of one), and routes `Ask::StepMade` and `Ask::StepRefused` through the forge's existing association, as jig's reference roots route them. Users: `crates/temper-engine-domain/src/{boundary.rs:8,364-369, lib.rs:39, route.rs:34,100-101, route/forge_route.rs}`, `tests/engine/domain/tests/root_routes.rs:1654-1659`. A further user the compiler names gets the same change, listed in the commit body.
     2. **The templates limit:** 03.3 carries of 04.1 what the repin needs to compile: `temper_engine_domain::Limits.templates` (`limits.rs:29-54`), `core_limits` taking it as `documents` is (`:109-143`, never a literal), and the service's `templates: jig.core.templates` (`crates/temper-service/src/limits.rs:264-278`), with any limits literal of a world the compiler names, listed in the commit body. 04.1 does what remains: the configurations' `templates` in place of `chats`, the startup's reader, its tests.
     3. **Counted:** both within the README's 200 added lines outside 03.3's Files, summed once as the README says, each item's lines given in the report. Past 200, land and give the count; a question only past 300.
     4. **Unchanged:** 03.3's Shape, Size (120, the path rewrite apart) and Proof; the forge's behaviour (each step makes what it made); temper's store format beyond what 03.3's Files already name; temper's tool names; 04.1's Needs and Shape.

165. **jig 08.3's results mark forgets in its own decision** (design gap, jig 08's block of 2026-10-16T18:47:43Z).
     Ruling 165: the designer's answer of 2026-10-16T19:14Z, landed as jig 134cbc189 and temper 4f86717c:


     ### jig 08.3: a forget lost to a crash between the results mark and its loads, 2026-10-16T19:14Z

     Landed: jig 134cbc189 (`docs/design/domain/people.md`, 6; `protocol/store.md`, 7.4, 11.1, 15 item 6; `STATUS.md`); temper 4f86717c (the plan's `jig/08-store-indexes.md`: Choices, 08.3, 08.4, the sketches, "Questions to raise when reached").

     ### Decision

     1. **The forget survives the crash: the mark and the erase are one decision.** people.md 6 now says: "The `Seen` after which it is read forgets it, while the party's inbox holds it (the bound below), in the `Seen`'s own decision: the mark and the erase commit together, so no restart sees the task read and not forgotten." This is the supervisor's reading. Ways 2 and 3 are not taken. testing.md 5 is unchanged, and the scenario's `check` stays: no row of the forgettable conversation.
     2. **The way: neither loads nor a pending mark.** people.md 6: "A `Seen` over results loads nothing for it: the inbox's entry for an unread result carries what its forget needs (the task's class, its label and listing, and, for a conversation, its last turn and whether its turns are read)." 08.3's Shape:
        - `people::ResultRef` gains `forgettable: Option<people::Forgettable { label, listed, last_reply: Option<u64> }>`, `None` for a kept task. It is set where a ref is made from the ended record: `routing/tasks.rs:730` and `inbox/census.rs:131` on jig's main at db13a6f18 (the plan's `:716` is at its base, 5fc5d31f9).
        - `forget::Owner` (task, party, project, label, ended_at, result_position, listed, last_reply) holds what a forget erases. `Owner::of_row` makes it from a loaded record and `Owner::of_result` from a cached ref. `forgets(owner: &Owner, reading: Reading)`.
        - On `Seen::Results { through }`: in the mark's own step, after its `Decided { MarkedSeen }`, the core pushes `Event::Forget { owner }` for each cached ref, read before the mark applies, with `position <= through`, `forgettable` set and a task not live. Nothing is loaded. A mark over kept results only, jig's own templates among them, is decided as at the base.
        - `Now::LoadEndedChat` goes, with nothing in its place (no `LoadForgettable`). `Event::Forget` carries an `Owner`, and only the core's own routes push it.
        - On `Seen::Chat` of an ended conversation: as at the base, `seen_chat_loaded` pushes the mark and `Event::Forget { owner }` (from `Owner::of_row`) into one decision.
     3. **08.4** keeps this so:
        - a `Seen::Chat`'s forget takes `Reading.turns` from the position its mark decides, with no load after the mark;
        - `Forgettable.turns_read` takes `chat_seen`'s place for a cached ref. It is set at the end, at the census from the joined `EndedRead`, and by a `Seen::Chat` in its decision;
        - `AfterRead` goes, and `Now::LoadEndedRead { request, task, party }` replaces it.
     4. **Tests** (in the plan):
        - 08.3: `a_results_mark_forgets_in_the_commit_that_moves_its_position` (the one commit of `world.commits` that erases `tasks::Key::Ended(task)` also saves `people::Stored::ReadResults`);
        - 08.3: `conversation_ends_while_read_chosen_cut_ends_the_story` cuts after the commit of the results mark;
        - 08.3: `an_owner_of_a_cached_result_is_the_owner_of_its_record`;
        - 08.4: `a_conversation_read_before_a_restart_is_forgotten_by_its_results_mark`.
     5. **Size:** 08.3 goes from 175 to 215 lines of production code (40 for `Owner` and the forget from a cached result); 08.4 from 340 to 355 (15 for `turns_read`).
     6. **The unlanded increment** (jig `6cf299600`) stands, but for its `Seen::Results` route. `Now::LoadForgettable` and its root arms go, `Event::Forget { row }` becomes `{ owner }`, and `ResultRef` gains its field: `forgettable: None` in the six test files that make one (`git grep -l 'ResultRef {' origin/main -- ':!docs' | wc -l` prints 9 at db13a6f18: all but `boundary.rs` and the two production sites).
     7. **Nothing new is deferred.** A forgettable task the cache does not name, or one a live task names when it is read, is retention's, already not planned (jig's `STATUS.md` people.md row, client-api's `later.md` row of what that file leaves not planned). No `later.md` row is added. The plan's open question on the uncached task is answered and removed.

166. **jig 08.4's mark loads held, a second mark busy** (design gap, the supervisor's question of 2026-10-16T19:18:33Z).
     Ruling 166: the designer's answer of 2026-10-16T19:32Z, landed as jig 09a600c43 and temper f92fddd0:


     ### jig 08.4: an ended conversation's mark decided on a position not yet durable, 2026-10-16T19:32Z

     Landed: jig 09a600c43 (`docs/design/domain/engine.md`, 5.3; `domain/people.md`, 6; `protocol/store.md`, 3; `STATUS.md`); temper f92fddd0 (the plan's `jig/08-store-indexes.md`, 08.4 and "Who owns what"; `README.md`, "jig 08.4's census"; `later.md`).

     ### Decision

     1. **The design does not accept it.** `max(loaded, marked)` does not hold. When both marks' loads go out before either mark is decided, each takes the max over the same old position, so the second can save a position below the first's. The stale save is durable, not just a count, against people.md 6: "nothing read comes back unread". And after the first mark's forget, the second can save an `EndedRead` for a task that has been erased.
     2. **What the design asks.** engine.md 5.3, new: "What a decision loads stays as loaded until it is decided. Its loads are held (4.2): each is sent once every commit made before it is durable, so it sees them. While they are in flight, a request whose decision would change what they load is answered busy". engine.md 4.2 already said: "held: ... a load (so that it sees every commit before it)". people.md 6, new: "One such mark loads at a time: while it does, the party's next `Seen` over that conversation's turns, or over results that would forget it, is answered busy". Busy is an answer the contract already has: "nothing was made and nothing kept; send it again, under its key, after a backoff" (contract.md 4).
     3. **What changes in 08.4** (its Shape):
        - `core::Held::Load(Box<Now>)`. `Seen::Chat` on an ended conversation sends `Now::SeenChat`, then `Now::LoadEndedRead`, each inside a `Held::Load`, never as `Request::Now`. A root holds it in the decision. Once the journal releases it, the root routes the wrapped `Now` the way the base routes that load (`examples/ops/domain/src/route.rs:1368-1374`).
        - While a `Seen::Chat` of a task waits on its loads, the core answers `Again(Busy)`, before any load, to another `Seen::Chat` of that task and to a `Seen::Results` that would forget it. The check is `people::Domain::chat_mark_loading(task, except)`, in `ended.rs`. 8.3's `MarkTurns` marks two conversations, one mark each, so neither is busy.
        - Files: `routing/people.rs` (:279-289, :310), `routing/vocabulary.rs`, the held arms of the three roots, the 19 exhaustive lists of `core::Held` (`git grep -l 'Held::NotesLoad' -- ':!docs'`), and temper's `route/held.rs` at its repin (README, "jig 08.4's census").
        - Don't: send a mark's load as `Request::Now`.
        - Size goes from 355 to 395 lines (40 for this).
     4. **The check goes in the core world, with no new store.** jig-core-world already holds each commit back from the component for `world.store_delay` ticks (`testing/jig-test-store-files/src/store.rs:117-138`). A `Request::Now` load reaches the component at once (`:198-204`). store.md 14: "The world holds commits between the root and the component". The tests, each with `world.store_delay = 3`:
        - `activity.rs`: `a_mark_sent_as_its_chat_ends_waits_for_the_end` (sent `Now`, the load finds no ended record and the mark is refused unknown); `a_second_mark_on_an_ended_chat_while_the_first_loads_is_busy` (a mark through 2, then one through 1: the store keeps `EndedRead { through: 2 }`);
        - `forgetting.rs`: `a_results_mark_while_a_chat_mark_loads_is_busy` (no row of the task is left, `EndedRead` included).
        testing.md is unchanged.
     5. **Deferred:** jig's other point loads (`HistoricalProposal`, `HistoricalEscalation`, `HistoricalPersonTask`, `PartyInputs`) still go out `Now`, against engine.md 4.2. They have a `later.md` row, for a jig plan after this one, and jig's `STATUS.md` engine.md row lists them as not planned. jig is upstream of the plan, so no downstream `STATUS.md` needs a row.

167. **The run's steers' Pass on texts in the README, out of context.md** (process, the coordinator's block of 2026-10-16T19:39:21Z).
     Every steer's Pass on text leaves context.md. In their place context.md keeps one line: "Steers' Pass on texts: in the plan's README, its rules and 'The run's steers'; each session reads them there." development.md 7.5 has context.md hold "decisions in force from answers and steers that the plan does not yet say"; after this ruling the plan says each text.

     1. Their Until has come, and nothing takes their place:
        - 15:06Z (ruling 142, temper 03.1's process test): temper-03.1 landed `72e17bdab`.
        - 17:52Z point 3 (jig 04.4's clippy rounds): jig-04.4 landed `5fc5d31f9`.
        - 14:09Z point 1's sentences for jig 05 (the five renames): jig 05's five steps have landed.
     2. The README says them already, under these rules: 10:08Z point 1 and 15:30Z point 3, "Named numbers in tests"; 12:48Z point 1, "Generated code"; 15:30Z point 1, "One land at a time in each repository"; 15:30Z point 2, "A test's helpers once"; 16:41Z point 2, "Every crate's lints in one round"; 17:17Z, "Pins", "temper's skein ahead".
     3. The rest land now in the README's new rule "The run's steers", each as its steer gave it, with the sessions it applies to: 09:29Z points 1 to 3, 10:08Z point 2, 12:48Z point 2, 14:09Z point 1 without its jig 05 sentences, 16:41Z point 1, 17:52Z point 2, 18:42Z points 1 and 2, and 19:38Z points 1 and 2. Left out of them: the sentences of past counts ("In the first hour, …", "Since 15:55Z, …"), and 12:48Z point 2's "A commit you have already made lands as it is", which named the commits made by 12:48Z. 09:29Z point 2 now cites `domain.rs:166` (the test's line on main, not :169); 18:42Z point 2 says "Three were missing at jig `bb977c909`"; 19:38Z point 2 adds that `person_unchanged` and `fixtures::unchanged` come from jig 04.5.
     4. The coordinator gives none of these texts again in a session's prompt: the README's first rule has every session read "this README". A running session has them from the coordinator's messages already.
     5. The 19:38:33Z steer's points 1 and 2 are done: their texts reached the five jig sessions, and they are in the README now.
     6. From now until the run's end, a steer's Pass on text that holds until the run's end lands in "The run's steers" before the steer is sent, and context.md names only its steer's time and point. A Pass on text with an earlier Until stays in context.md, as before.
     7. Unchanged: each steer's Do, Stop and Until; the README's other rules; context.md's limit of 60 lines of at most 200 characters.

168. **jig 12.2 goes on without the parked request's NotStored, until the designer answers** (interim, jig 12's block of 2026-10-16T19:42:06Z).
     Interim, until the designer's answer: a design gap, not a plan defect. client.md 11 says "`NotStored` (a quota full, a disk failing) retires the requests the write carried, unsent, each answered *not kept*"; skein's lib.md 12 gives the request table no retirement but "*Final* retires the request" and the retention cutoff, and no text of either design says how a request with no attempt is retired. The question went to the designer at 19:46:42Z (`question.md`).

     1. jig 12.2 goes on with the work that does not rest on the answer: the barrier (a step's records in one `Save`, a keyed request's first `Exchange` only after `Stored`), the link's `Request::NotKept { attempt }` for each held keyed attempt and the client's `answered(now, attempt, Final)` for it, `Start { kept }`, the reload and two-client stories, and the `why: Option<Retry>` and `Dropped` parts of its Shape.
     2. It does not write the parked request's `NotStored` path or its offline regression, and changes no file of skein: the answer decides both (option A, a skein entrance; or option B, another behaviour in client.md 11).
     3. 12.2 does not land before the answer's unblock. When the rest is done first, commit it on the branch, report, and wait for the unblock.
     4. Unchanged: 12.2's Scope, Files, Shape and Tests; jig 12.3 to 12.5 wait for 12.2 as the plan says.

169. **temper 03.3 converts ENGINE's chats to templates, 04.1's conversion** (plan defect, temper 03's block of 2026-10-16T19:54:38Z).
     Ruling 169, a plan defect, the implementer's option A: yes. 03.3's pin carries jig 09.1's reader, which refuses `ENGINE`'s `chats` and wants `templates`, so 04.1's `ENGINE` conversion is a repair of 03.3's repin, as the README's "What a temper repin repairs" has it ("what else jig landed since temper's last repin"), and as ruling 164 carried 04.1's limits.

     1. 03.3 converts `ENGINE` (`crates/temper-service/src/tests/fixtures.rs`) as 04.1's Files give it, word for word: `:46`'s `"chats": {"charter": 1, "close_grace_ms": 5000}` becomes `"templates"`, "jig's two defaults written out in jig 09.1's format, as jig 09's fixtures write them: 'jig's defaults are written, not implied', their reports' `words` 32, within `ENGINE`'s `"tasks.result_bytes":32`"; its policy at `:65` gains `"templates": []`; and the `Project` literal at `:436` gains `templates` where the pinned types ask it.
     2. 04.1 keeps what remains: its reader work, its tests (`a_chats_member_is_refused_as_unknown_naming_its_path` among them), `forgejo.rs`'s `people.operations` after jig 09.4, and the rest of its Files. Its "Removes: the `chats` member in `ENGINE`" is 03.3's now.
     3. The lines count among 03.3's repin repairs (ruling 164: past 200, it lands with the count in its report; past 300, a question). Existing assertions keep their names and values.
     4. Any further copy of the `chats` member the repin's suites find: convert it the same way, and list it in the commit body. On temper's main only `fixtures.rs:46` holds one.
     5. Unchanged: ruling 164; 03.3's Shape, Test shape and Proof; 04.1's Don't ("set a template's field by hand where jig's fixture gives it").

170. **jig 12.2's request not kept: skein-01.s3's `not_kept`** (design gap, jig 12's block of 2026-10-16T19:42:06Z).
     Ruling 170: the designer's answer of 2026-10-16T20:03Z, landed as skein 1f71424d, jig 3b8bf1263 and temper b694e960; the supplement step is skein-01.s3 (`skein/01-s3-not-kept.md`, the answer's item 2), added by this ruling:



     1. **Option A. The table gains `not_kept`, and client.md 11 stands, parked requests included.** lib.md 12, new: "a request whose record the asker's store did not write is retired by its key (`not_kept`), whatever its state: parked, ready, in flight or retrying ... It is never sent; an answer for its attempt changes nothing. Its erase is emitted, as any retired request's, since an earlier write may hold its record. Its owner value is handed back, as a final answer's, with no progress ... A key that names no request, or a retired one, hands back nothing." client.md 11, now: "each answered *not kept*, by its key ...: parked, or with its `Exchange` waiting for the write ... A record is whole in each write, so a request whose record a later write carries again is decided by that later write".
     2. **skein-01.s3, the supervisor's to add.** Needs: nothing.
        - Files: `crates/skein-lib/src/requests.rs`: `pub fn not_kept(&mut self, key: RequestKey) -> Option<O>`, beside `answered` (:268-:307 at `4e293b7`). It finds the entry by `self.index`. A key not held, or `State::Retired`, gives `None`. Otherwise it sets `state = State::Retired { erase: true }`, `save = false`, `changed = false` and returns `Some(owner)`. The module doc's table gains `| parked, ready, in flight, retrying | not kept | retired | owner value, erase |`.
        - Tests in `crates/skein-lib/src/tests/requests.rs`:
          - `not_kept_retires_a_parked_request_and_erases_it`: link down, `ask`, take its `Save` (`first_sent` `None`). `not_kept(key)` is `Some(owner(1))`, then `None`, and `progress()` is `None`. The next take gives `Erase(key)` alone. After `link(true)` and `confirmed(Some(7))`, a take gives nothing. `reclaim()` leaves `is_empty()` and `bytes()` 0.
          - `not_kept_makes_a_held_attempt_stale`: `live()`, `ask`, `sent` gives the attempt. `not_kept(key)` is `Some`, then `answered(.., attempt, Final)` is `Stale`, and a take gives `Erase(key)`. A second request answered `Final` gives `None` from `not_kept`.
        - Size: at most 30 lines. Proof: skein's gate, by `land`.
        - Don't: change `answered`, `RequestOut`, `RequestAnswered`, `RequestProgress` or `tests/lib/src/request_table.rs`; name a downstream repository.
     3. **jig 12.2's Shape (landed):**
        - **The skein move.** 12.2 needs skein-01.s3. Its first commit moves jig's skein to s3's landing, with the one repair `lines: 0, line_bytes: 0` (`tests/client-protocol/src/native/world.rs:232`). This is the plan's one jig move of skein, moved forward from jig 19.4; jig 19 makes none (README, "Pins"; `jig/19-browser.md`, "No repin").
        - **The client's bookkeeping.** `jig-client` numbers its writes; each is answered before the next (client.md, 16). Per table key, it keeps the number of the last `Save` that carried its record: at most `requests.requests` keys.
        - **On `NotStored` of write `w`.** Before the link takes it, the domain gets `Up::NotKept { key }` for each key whose last write is `w`. It calls `table.not_kept(key)`, and for `Some(owner)` tells `Answered { key: owner, outcome: NotKept }`.
        - **The link's `NotKept { attempt }`** then reaches the domain as `Lost`. That answer is stale for a request just retired, and a retry for a request whose record a later write carries.
        - **Tests and Size.** New tests: `a_parked_request_whose_record_is_not_stored_is_answered_not_kept` and `a_failed_write_answers_only_the_requests_it_carried`. Size goes from 215 to 250.
     4. **Nothing is deferred:** no `later.md` row.

171. **skein-01.s3's Shapes against main** (ready check of skein-01.s3).
     skein-01.s3's session file, checked against skein `1f71424` and temper `4c522a82` by a read-only sub-agent, is corrected in `skein/01-s3-not-kept.md`: `answered` spans `requests.rs:268-:305`; skein has no `docs/development/workflow.md`, so the file names its gate, `scripts/check.sh`; `not_kept`'s guard and the tests' `Save` and `Erase` checks are exhaustive `match`es, since skein's `clippy.toml:86` disallows `core::matches` in tests too (a `matches!` failed `cargo clippy --workspace --all-targets -- -D warnings`) and `wildcard_enum_match_arm` is denied; the first test gains a second parked request whose `Save` is not taken, whose `not_kept` gives `Erase` and no `Save`, which pins `save = false` (no test of the answer's two failed without it); Size goes from 30 to 70, the sketch with two tests being 57 lines. Checked in a scratch worktree with the sketch applied: clippy with `-D warnings` and `cargo fmt --all --check` clean; `cargo nextest run -p skein-lib -p skein-lib-tests`, 161 passed, both `not_kept_*` included; the fuzzy profile, 6 passed, `fuzzy_request_table` included; no `not_kept` in skein, jig or temper, and no downstream match on the table's enums changes. Refresh any further copy, and list it in the commit body.

172. **jig 08.4's ended read position in its own field of reads::Row** (plan defect, jig 08's block of 2026-10-16T20:28:01Z).
     Ruling 172: yes, the implementer's proposal. A plan defect: the plan put the ended read position in `reads::Row.related`, which an inbox reply's activity line already holds. The design names the position as a join of its own, not as "the record it relates to": people.md 6, "the child holds none of them in memory, a read of the conversation or a task query's row joins it by a point read"; store.md 7.3, "joins each row in the same step by point reads: its owner task, live or ended; its requester ancestry ...; its links; the record it relates to", and "for a task flagged conversation, its requester's ended read position, by a point read (7.4)".

     1. **`reads::Row.ended_read: Option<u64>`** (`crates/jig-domain-core/src/reads/vocabulary.rs:194`), documented as the ended read position joined for an ended conversation's requester party. The store's indexed path (`crates/jig-protocol-store/src/reads/indexed.rs`, `joined`) fills it by a point read of `people::Key::EndedRead { task, party }`, the row's owner and the range's party, priced within the read's bounds as the other joins are (store.md 7.3, "Joins are point reads"); `None` where no such row is stored, and on every other row. `related` keeps its immutable evidence, the inbox reply's `ActivityLine` included.
     2. **The core reads it** where 08.4's Shape read `related`: `conversation_from_row`'s `seen` and the reply filter of an ended inbox conversation (`reads/rows.rs`) take `row.ended_read`. Count it in `Row::owned_bytes` only if `row_bytes` (`vocabulary.rs:455`) counts inline fields; it adds no heap.
     3. **jig 08.5's task-query row** uses the same field, not `related`: one carrier for the position across 08.4 and 08.5.
     4. **Every `reads::Row` literal gains `ended_read: None`** but the indexed join: ten in jig (`git grep -n -E '^\s*unjoined[:,]' -- '*.rs' ':!**/generated/**'` at jig `05e9d00e7`); refresh any further copy, and list it in the commit body. temper's one, `tests/client/src/fixtures.rs:238`, is the repair of the first temper increment whose repin includes jig 08.4's landing (README, "jig 08.4's census").
     5. **Size** goes from 395 to 410 lines.
     6. **Don't:** put the position in `related` or `linked`, or a second `Record` in `Row`.

173. **skein 01's browser test's Close after Closed: skein-01.s4** (plan defect, skein 01's block of 2026-10-16T20:38:21Z).
     Ruling 173: a plan defect of this plan's skein 01, fixed in a supplement step in the same session. The failing test is skein-01.3's (`b23380a`, "Await each person's expected trouble and chosen HTTP status"), and it panicked once, at `crates/skein-http/src/client.rs:899`, `State::Closed => unreachable!("a Close after Closed")`, in a gate whose other 37 browser tests passed. skein-01.s3's Shape touches neither.

     1. **skein-01.s3 runs `land` once more, as is,** from its commit `083881ec`. If the gate passes, it lands, and its report names this ruling and the failed gate's log. Nothing else is wanted: no further retries.
     2. **skein-01.s4** (`skein/01-s4-close-after-closed.md`) is skein 01's next step, after s3: it finds why `close` was reached in `Closed`, fixes the cause, and adds a test that drives that order without a browser, failing before the fix. Its proof adds the browser test 20 times, once, before landing.
     3. **If s3's second gate fails too,** s3 waits: the session does s4 first, on a fresh branch from `origin/main`, lands it, then rebases s3 onto it and lands s3.
     4. jig 12.2's move of jig's skein still waits for s3's landing (ruling 170); s4 does not hold it.

174. **jig 04.6's compound admission across core and tasks** (plan defect, jig 04's block of 2026-10-16T20:39:52Z).
     Plan defect: 04.6's Shape had the core admit each operation "as it would alone" but named no way to judge it against what earlier operations leave in the tasks child, which the design asks in its own words: "They are admitted in order, each checked as it would be alone against the state those before it leave ... the refusal names the first that failed, and why" (jig's tasks.md, 7, "Several operations, one decision"); the route is "people (one project, within operations per request) → for each operation in order: authority, one pool counted across them → tasks" (engine.md, 4.4); and "a parent owns its children's state and routes between them" (engine.md, 4).

     The proposal is approved: `jig-domain-tasks` gains a public, read-only `CompoundAdmission`, which holds the private `Staged` and borrows the `Domain`. For one operation it runs the checks `compound()` runs (`compound.rs:190`, `check`) and stages it. Its borrowed accessors give the core the projected fields its admits read: revisions, tracking, requester ancestry, funding, phase, address. `compound()` runs its checks through the same code, not a copy. The core admits each operation in order as it would alone, its own admit (`control_admit`, `amend_admit`, `move_admit`, `person_message`, 4.5's maintainer check) and then the admission's, against the projection; it sends one `tasks::Event::Compound` once all pass. The first failure, the core's or the tasks child's, is `Outcome::CompoundRefused { at }`, with `at` its place.

     Don't: clone a `Domain` or a task record for the admission; mutate the `Domain` through it; widen `Staged`'s fields past `pub(crate)`; write a second copy of the tasks child's checks in the core. The admission's signatures are yours, within these bounds. Add one test to `tests/core/tests/operations.rs` for the case your report names: operation 0 refused by the tasks child and operation 1 by the core answers `CompoundRefused { at: 0 }`; under 0.5 s.

     04.6's Size becomes at most 420 lines (was 300). Unchanged: the rest of 04.6's Shape, Test shape and Proof, and 04.7.

175. **jig 03.4's decided party proposal in its proposer's updates** (plan defect, jig 03's block of 2026-10-16T20:57:06Z).
     Plan defect: 03.4's Files gave no way for a decided party proposal to reach its proposer's inbox, which the design asks and places. people.md, 5.1: "`Propose` asks, with a reason, and the party is its proposer: its decision, a rejection included, reaches them." store.md's reads: "a proposal | a party's: its record, by key, with its decision"; "an inbox group | the index of the party's inbox, by time: what needs them oldest first, updates newest first"; and the tasks family keeps "decided proposals" on demand, by "proposal", the core family "decisions on proposals" by "proposer, proposal".

     Your proposal is approved, as the default and only path: `tasks::UpdateObject` gains a variant for a decided party proposal, indexed for its proposer in the updates group, in the tasks family's v3 schema and codec; the store's and the core's update join loads `Stored::DecidedProposal` with its `CoreRecord::ProposalDecision` for it, in place of a `TaskRecord`. A withdrawn proposal's entry is made the same way. Pending proposals stay out of `Group::Updates` (`Key::PersonProposal`, as now). Retention (`forget.rs`) removes the entry with the decided proposal it names, as it does a result's.

     Don't: serve the entry by scanning decided proposals; add a second index for it; change the existing `Result` and `Reply` rows or their order ("A result sorts before a reply of the same task and time"). Add every exhaustive match the compiler names over `UpdateObject` an arm (jig only; temper names `UpdateObject` nowhere), and refresh any golden or fixture key the variant changes, listing each in the commit body.

     03.4's Size becomes at most 420 lines (was 330), a codec increment. Unchanged: 03.4's Shape, Test shape and Proof (the acceptance in the proposer's inbox), and 03.5.

176. **The check of 20:41Z's lasting steers in the README** (steer of 2026-10-16T21:01Z).
     The check of 2026-10-16T20:41:43Z's steer of 21:01Z: its points 1 (a .rs file of at most 800 lines at `origin/main` stays at most 800, with the three that crossed it brought back when next edited) and 3 (`person_unchanged` taken from `jig_core_world::fixtures` where a crate can reach it) hold to the run's end, so their Pass on texts join the README's rule "The run's steers" (ruling 167), and the coordinator gives them in no prompt. Point 2 (a session past 150K hands over at its landing, development.md, section 7.3, step 4) is the coordinator's alone. Unchanged: the 18:42Z and 19:38Z points.

177. **jig 03.4: no retention for a decided proposal's update** (plan defect, jig 03's block of 2026-10-16T21:03:50Z).
     Plan defect in ruling 175: its retention sentence rested on a premise nobody checked. Nothing forgets a decided proposal on `main`, so 03.4 adds no retention, no read mark for proposals, and no erase helper without a caller: the proposer's update lives as long as its decided proposal does, which is the decided proposal's lifetime today, unchanged. `Seen` and read marks keep covering results and chats only.

     The removal waits for the plan that forgets decided proposals: later.md's new row names it (the decided proposal, its core decision and its update's index key removed as one deletion), and jig's retention plan takes it. The rest of ruling 175 stands: the `UpdateObject` variant, its v3 schema and codec, the join to `Stored::DecidedProposal` with `CoreRecord::ProposalDecision`, and 03.4's Size of at most 420 lines. Ruling 175's Files name `crates/jig-domain-core/src/forget.rs`: touch it only for an arm an exhaustive match over `UpdateObject` there needs, and no more.

178. **The 22:44Z steer's point 2 in the README's steers** (process, the coordinator's block of 2026-10-16T22:46:03Z).
     Ruling 178: the 22:44:00Z steer's point 2 holds until the run's end, so its Pass on text goes into the README's "The run's steers", as ruling 167 says ("A later steer's text that holds until the run's end lands here, by the supervisor, before the steer is sent"). It is there now, under "22:44Z, point 2 (jig 12)", word for word as the steer gave it. Points 1 and 3 end at the landings of jig 03.4 and 04.6 and stay out of the README. The coordinator gives point 2's text in no later prompt: jig 12's sessions read it in the README.

179. **jig 12.4 goes on without the owner versions' declaration, until the designer answers** (design gap, jig 12's block of 2026-10-16T22:51:29Z).
     Ruling 179, interim, until the designer's answer to `question.md` (asked 2026-10-16T22:53:02Z): a design gap, not a plan defect. client.md 8.2 says "The client speaks the one version of each owner its build holds", and client.md 6 that "a family's request" carries "its owner's number, its scope, its version and its record's bytes", but no design text says where the client learns its build's versions (declared at construction, or read from each family message) nor how its memory of owners and ranges is bounded. That a read of an owner not served is answered so is the design's own text ("its reads are answered and its watches end as version not served"), but the answer's shape goes to the designer with the rest.

     Until the answer, jig 12.4 goes on only with work that does not rest on it:
     - families as bytes under their owner, on the same table, watches and reads (client.md, 4 and 6), and the story `a_family_request_is_answered_as_its_owner_s_bytes` with its proof;
     - the link's `Hello { family: Some(_), .. }` to `Up::FamilyServed { family, versions }`, carried to the domain, with no judgement of it yet;
     - the fuzzy suite's `random_scripted_applications_keep_the_client_s_rules`;
     - `a_family_document_past_the_bytes_limit_ends_its_handle`.

     It adds none of these until the answer: a declaration of the build's owner/version pairs, in `Limits` or elsewhere; a memory of owners' ranges; `Readout::NotServed` or another read's answer; `Why::NotServed` and `Progress::NotServed`; `LinkView.not_served` filled; the three unit tests on an owner not served. 12.4 lands only after the answer, as one increment, unless the answer splits it.

180. **jig 12.4: the family owners' versions declared at construction** (design answer, jig 12's block of 2026-10-16T22:51:29Z).
     Ruling 180, the designer's answer, landed as jig b09e2239e and temper 21c0ec9a; it ends ruling 179's interim:


     1. **The implementer's proposal: a declaration, at construction.** client.md, 8.2, now: "each family owner's, which the application declares as it makes the client, with its limits (section 14): each owner whose families it sends, by its number, with its wire's version ... Each family's request, read and subject names a declared owner, in its declared version; any other is the application's mistake, and loud, as a key conflict is". The client never learns a version from a message.
     2. **Each hello judged whole.** "A declared owner is not served when the hello names no range of it, or one that leaves out its version. The judgement holds until the next hello, so `client.link()` names an owner not served from the hello on, before the application has asked anything of it. The client keeps, for each declared owner, its judgement and the range the hello named of it; of an owner it does not declare, nothing."
     3. **The bound.** client.md, 14: "the family owners the application declares, each once, with its version (8.2): no more than a hello's ranges hold beside jig's". The wire's `Hello.served` holds 16 (`CEILINGS.hello_served`), so at most 15.
     4. **A read of an owner not served.** client.md, 7: "a family's outside its owner's range or any read while the link is out of date, is answered at once and never sent: version not served, naming the owner and the range the last hello named of it, or none where it named none."
     5. **A kept request of another version.** client.md, 11: "one whose owner the build does not declare, or whose version is not the one declared (8.2), is such a record", that is, dropped at start and the application told (protocol/client.md, 10).
     6. **12.4's Shape (landed in the plan):**
        - `jig_client_domain::Limits.owners: &'static [Spoken]`, with `Spoken { family: u16, version: u32 }`. The link's `Limits::new` refuses more than 15 owners, or one family twice.
        - The link's `Request::Hello { family, versions: Option<VersionRange> }` comes once for jig and once per declared owner. At a hello, the link reopens only the watches in range.
        - `Up::Served` gains jig's `versions`. `Up::FamilyServed { family, versions: Option<VersionRange> }`.
        - `Readout::NotServed { family: Option<u16>, served: Option<VersionRange> }`. `LinkView.not_served: &[u16]` is unchanged.
        - An undeclared owner or version on `Send`, `Read` or `Watch` `assert!`s. A kept one is dropped and told `Told::Dropped`.
        - Five tests are added and two link tests change. Size goes from 150 to 210.
        - jig 13.4, jig 21.2 and temper 03.5 declare their families' owners.
     7. **Nothing is deferred:** no `later.md` row.

181. **jig 04.6's compound answer stands, 04.7 goes on, until the designer answers** (design gap, jig 04's block of 2026-10-16T23:16:03Z).
     Ruling 181, interim, until the designer's answer to `question.md` (asked 2026-10-16T23:17:18Z): a design gap, not a plan defect, as the plan's own "Questions to raise when reached" (jig 04, "A compound's answer") says. contract.md 3.4 says "done, with each operation's outcome in order", but names no value an operation's outcome holds beyond "made" for any kind but a message, so whether the landed `Outcome::Compound { project, operations, first_message }` carries it is the designer's.

     Until the answer:
     - 04.6's landing (jig 983862753) stands as it is: no change to `people::Outcome`, its `Copy`, or its codec.
     - jig 04.7 goes on, with its Shape as written. Its conformance scenarios and stories judge a compound by "done" or "refused, at <place>", and by what the store holds after it; no assertion of 04.7's adds `first_message` or `operations` beyond what 04.6's tests already check, so a changed answer moves no test of 04.7's.
     - jig 09.4 starts as the plan says; nothing in it rests on the answer.

     If the answer changes the compound's answer, its plan change comes as a further unblock with its own step.

182. **jig 03.4 carries 23.1's party_proposals evidence** (plan defect, jig 03's block of 2026-10-16T23:18:01Z).
     Ruling 182, a plan defect in the order of steps: 03.4's Shape says "creator: Party::Person(proposer), each member funded by the accepter's pool", and its proof reaches the referee's accepted-task check, which has no evidence of a pending party proposal until 23.1's `Snapshot.party_proposals`. The implementer's proposal is taken: 03.4 carries the minimum of it, and 23.1 keeps the rest.

     jig 03.4 adds, beside its Shape:
     - `Snapshot.party_proposals: BTreeMap<u64, u64>` in `testing/jig-conformance/src/referee.rs` (each pending party proposal's number and its proposing party), as 23.1's Shape gives it;
     - its filling in the three adapters that build `Snapshot` (`tests/core/src/observations.rs`, `tests/conformance/src/lib.rs`, `tests/ops/src/conformance/observations.rs`), from the `Pending` `tasks::Stored::PersonProposal` rows alone (that type also holds accepted and rejected ones); `tests/one-process/src/referee.rs` unchanged;
     - the party `ProposalDecisionRecord`'s `created` range for an accepted batch (`crates/jig-domain-core/src/proposals.rs:354`, `created: None` on main), so the core world's observations fill `Snapshot.accepted_tasks`;
     - the referee's accepted-task check takes an accepted party batch's tasks by the pending party proposal's identity; its source-ceiling check against `accepted.by` (`referee.rs:437`) stays as it is, and the result-delivery check's `source` is not rewritten;
     - one referee case in `tests/core/tests/referee.rs` that an accepted party batch funded by its accepter passes and one past the accepter's ceiling fails, then the joint proof `a_partys_batch_beyond_their_role_is_proposed_and_accepted` once.

     23.1 keeps `Observed::{ProposeAsked, ProposeRefused}`, the ask credits and their rule, and uses `party_proposals` as 03.4 lands it. The referee and adapter lines are test support, outside 03.4's production count; the steer of 22:44Z, point 1 (1050 lines of production code at most, past it a question) stands.

183. **jig 04.7's compound referee takes the ask as its baseline** (plan defect, jig 04's block of 2026-10-16T23:19:06Z).
     Plan defect: 04.7's Shape gives `Observed::Compound { tasks, made }` the rule "refused, none changed since the request" with no observation of the request, and the referee's only atomicity rule, `batch` (`referee.rs:779`), reads when each task first appeared (`self.first`), not when a row changed. Option A of the report, as the plan already does for 23.1's `Observed::ProposeAsked`:

     - `Observed::CompoundAsked { tasks: Vec<u64> }`, declared by the scenario's action before each send of the compound (`Action::Compound` and `Action::CompoundAgain`). The referee keeps, for the first ask of a `tasks` list, each named task's row (`referee::Task`) from its last durable snapshot (`self.last`); a later ask of the same list keeps that baseline, so a request sent again after the compound's commit is judged against the first ask.
     - On each `Observed::Durable`, the first commit after the baseline that changes any named row is recorded, with the rows it changed. Later commits do not move it (the chat's grace ending, a moved delegate's turn).
     - `Observed::Compound { tasks, made }`, with the `tasks` of its ask, under `Promise::Once`: made, the recorded commit changed every named row; refused, no named row changed since the baseline. An answer whose `tasks` were never asked breaks the rule too.
     - A task whose operation changes no field of `referee::Task` (a graced cancel keeps the task's phase until the grace ends, `crates/jig-domain-tasks/src/grace.rs:170`) is left out of both `tasks` lists, and the report names it. Nothing else changes in the referee's row.
     - Option B (a commit number on `Observed::Compound` and a row history) is not taken.
     - The referee's two tests stay as the Test shape names them: `a_compound_made_across_two_commits_breaks_once` (an ask, two commits each changing one named row, a made answer) and `a_refused_compound_with_a_changed_task_breaks_once` (an ask, a commit changing a named row, a refused answer).
     - Size: at most 150 lines (about 110 in `testing/jig-conformance`), in place of 130 and 90. Past 150, the session reports it with its count by file. No production code changes for this ruling; `people::Outcome` and the store stay as they are.
     - Unchanged: ruling 181 (04.6's answer stands until the designer answers question.md); 04.7's scenarios, cuts, proof and "Don't". If the designer's answer changes the compound's answer, it comes as a ruling of its own.
     - Any further copy of `Observed`'s variants that 04.7 finds matched exhaustively takes the new variant; list it in the commit body.

184. **jig 04.6: a compound's operations numbered by place, and jig-04.s1** (design answer, jig 04's question of 2026-10-16T23:16:03Z).
     Ruling 184, the designer's answer, landed as jig a20cf96c4 and temper 90ed12e0; it ends ruling 181's interim. Item 3 is the supplement step jig-04.s1 (`jig/04-s1-numbered-by-place.md`), in session jig 04 after 04.7 lands; jig 14 and jig 15 start after it:


     1. **Each operation's outcome is made, with a message number by place, and nothing more.** contract.md, 3.4, now: "Its operations take consecutive message numbers, one each, in the request's order, and the answer names the first: the operation at place *i* took the first plus *i*. A `Message`'s words have that number in its task's inbox, as a lone `Message`'s answer names theirs. What another operation adds to its task's inbox, if anything, has its number too; one that adds nothing leaves its number unused." It also says: "Nothing more of an operation is answered, as nothing more is when it is alone. What it changed (a grace's end, a spec's revision, a requester and the reservation moved, a hold lifted) is in its task's documents". people.md, 5.1 says the same ("*Numbered by place*").
     2. **The landed answer keeps its shape and `Copy`:** `Outcome::Compound { project, operations, first_message }`. What changes is its numbering. On main, only the operations that add a message take a number (`crates/jig-domain-core/src/compound.rs:60`, `issue` at :224). Now every operation takes one.
     3. **A step after 04.6 (the supervisor's), about 20 lines:**
        - `issue` takes a fresh `Family::Message` number for every operation, and `first_message` is the number of the operation at place 0. Stop, Move, a Cancel without a grace and a Release whose `again` is not `Some(false)` take one too, and leave it unused.
        - `valid_answer_shape` (`crates/jig-domain-people/src/domain.rs:1345`) admits a made compound only when `first_message != 0`, as `Created` checks `first != 0` (:1359).
        - The fakes' `first_message: 0` become 1: `crates/jig-domain-people/src/tests/compound.rs:33`, `tests/people/tests/compound.rs:27`, `tests/people/src/world.rs:563`.
        - In `tests/core/tests/operations.rs` (:134-135 and :239-240), the chat's cancel with a grace is at place 2 of `closing`. So its grace's `message` is asserted as `first_message + 2`, and the answer's `first_message` stays 1.
        - Proof: `heavy cargo nextest run -p jig-core-world -E 'test(a_compound_closes_a_conversation_moving_one_delegate_and_cancelling_another)'`, once. Nothing else is wanted.
     4. **jig 14.3 (in the plan):** the wire's answer is `OwnerOutcomeCompound { project: u32, operations: u32, first_message: u64 }`, field for field. There is no `OperationOutcome` and no per-operation list. The link's test is now `a_compound_answer_names_its_first_message_and_operations`.
     5. **jig 15.4 (in the plan):** `Thread::sending(key, by, words, place)`, with `place` 0 for a lone `Message`. `answered` marks the words sent with `first_message + place`, tested by `words_sent_in_a_compound_are_numbered_by_their_place`.
     6. **Unchanged:** 04.7, whose tests judge only made or refused at a place (ruling 181). Nothing is deferred: no `later.md` row.

185. **The 23:45Z steer's two rules in the README** (steer of 2026-10-16T23:45:53Z).
     The steer of 2026-10-16T23:45Z, from the check of 23:45:25Z: its points 1 (a function too long is split, not excused) and 2 (request ids, durations and times named in tests) hold until the run's end, so their Pass on texts are in the README's "The run's steers" (ruling 167), under "23:45Z, point 1" and "23:45Z, point 2". Unchanged: the 12:48Z `too_many_arguments` rule, the base's exceptions, 04.6's landed code (no rework step), ruling 149's `large_enum_variant`; seeds and expected values stay literals.

186. **jig 04.7: eight commits from Ready, and a base exception kept** (plan defect, jig 04's block of 2026-10-16T23:49:15Z).
     Plan defects, both proposals A:

     1. **The eight commits count from `Action::Ready`.** Every program starts with the harness's `vec![Step::Drain, Step::Action(Action::Ready), Step::Drain]` (`testing/jig-conformance/src/scenarios.rs:139`), shared by every scenario; 04.7's "Each program has at most 8 commits" counts the scenario's own steps after it. 04.7's Shape now says: "Each program has at most 8 commits after its `Action::Ready`; the harness's start before it, shared by every program, is not counted (ruling 186)." Record the commit at `Ready` and assert at most 8 further commits, as the report proposes. The whole world's 15 is not bounded. Unchanged: the fuzzy sweep's guide ("Aim for at most 1 s added to the fuzzy suite by the two; past 2 s, the session reports it, with the sweep's time by scenario from the gate's log"), which bounds what the 15 cuts cost; no sweep of their own.
     2. **`tag_tasks` keeps its exception.** It is excused at the base (`crates/jig-domain-core/src/routing/tasks.rs:19`, `#[expect(clippy::too_many_lines, reason = "one exhaustive task vocabulary routes its bounded child decision")]`, the function running to :1358), and 04.7's Files prescribe its `MoveChat` arm ("at most 15 lines"). The 23:45Z steer's sentence "A landed function keeps its exception until a session widens it." said the opposite of what it meant; the README's text is now: "A function already excused, at the base or since, keeps its exception when a Shape widens it (as 04.7 widens `tag_tasks`, ruling 186)." The rule stands for what clippy finds: no new `#[expect(clippy::too_many_lines)]` on code a session adds; new functions split as the steer says. No dispatch split, here or as a step.

187. **jig 12.5: the unlinked client's start** (plan defect, jig 12's block of 2026-10-16T23:50:01Z).
     A plan defect, not a design gap: the design answers the block's rule in its own words, and the type is the plan's. client.md, 4.1: "`Start { kept }`: the small store's records, as the host loaded them, before any other event (section 11)"; 11: "Restored at `Start { kept }`, before any other event: each record admitted to the table under its key, parked until the link is live". The unlinked client is 12.5's construct (the design names none: `git grep -n -i unlinked origin/main -- docs/design` prints nothing), and its Shape gave `Event::Up` and `Request::Down` but no start for the domain's records. The report's proposal stands:

     - `Event::StartUnlinked { kept: List<RequestRecord<Held>> }`: the unlinked host's start, before any other event, as `Start` is the linked host's. The client gives the domain `domain::Event::Start { kept }` as the host gave it (`crates/jig-client-domain/src/boundary.rs:90`); the records leave in `Down::{Save, Erase}`, as on main.
     - Under `unlinked`, `Event::Start` joins `Event::{Reply, Lost, Streamed, StreamGone}` as dropped, and under `new`, `Event::StartUnlinked` joins `Event::Up`: each `assert!`s, as 12.5's Shape says.
     - `an_unlinked_client_s_records_are_the_table_s` covers the restore: a record given at `StartUnlinked` is told `Kept` and parked. No further test.
     - Size: at most 110 lines, in place of 100.
     - Not taken: a sum type in `Start` (every linked host would wrap its list); a restore method outside `step` (it would relax the start rule).
     - Unchanged: `Event::Start { kept: List<Record> }` and the linked host's path; 12.5's routing, proof and "Don't".

188. **temper-04.1's Shapes against main** (ready check of temper-04.1).
     temper-04.1's Shapes against main: the re-check of what moved since jig `bc30abd58` and temper `187064e2`, at jig `3f2ae98ad` (jig 09.4's landing) and temper `edc7ad57`, by a read-only sub-agent. temper 03.4's repin onto jig 13.2 is the first temper repin past jig 03.1 to 09.4 (temper's pin is jig `755e02e5`), so the repairs it meets are 03.4's: its session file gains a "Repin" paragraph naming the errors the re-check saw at a trial repin to jig 09.4 (seven in `temper-engine-domain`; people v3's `Refusal` in `temper-protocol-store`'s `refusal.rs:290-370`; `CoreRefused`'s bound of 23, which the drift test settles; `What::Operations` in `startup/problems.rs` and in `temper-forge-setup/src/problem.rs`, a user 04.1's Files missed; `("people.operations", 5)` in `forgejo.rs`), within the README's 200 lines and jig 08.4's census. 04.1 makes no repin commit when temper's pin holds jig 09.4's landing, and adds what 03.4 left; its Files, Shape (startup hands the whole `engine` member to `config::engine`, `startup.rs:60`), Test shape (`replacen(…, 1)` in the chats test) and Removes (nothing: 03.3 removed `chats`) are amended, with the moved lines. Unchanged: 04.1's tests, proof and Size; the README's repin rules. The re-check's suites: the sketch and a Grace row under `clippy -D warnings` on `temper-service --all-targets`, clean; the three named tests pass at temper's present pin; the workspace at the trial repin did not compile (the seven errors), so no workspace, fuzzy or one-process suite ran.

189. **jig 03.5 goes on without the amendment's acceptance, until the designer answers** (design gap, jig 03's block of 2026-10-17T00:08:23Z).
     A design gap: authority.md, 9 says "An acceptance funds the action from the accepter: ... a widening moves budget or a grant from the accepter into the proposer", and a party's amendment widens a task that is not its proposer ("never the task an amendment would change"). Whether the task's funding moves to the accepter, or only the widening's budget, and what is recorded as its funder, is the designer's; the question went at once. Until the answer, ruling 189 lets jig 03.5 go on only with work that does not rest on it:

     - goes on: `people::Action::Amend` and `tasks::PersonAction::Amend` (sketches 1 and 2) with their codecs; `propose_request` for an amendment (reach, revision, needs, `Approver`/`Nobody` refusals, `ProposePerson`, `Proposed`); the refusal of an `Amend` beyond the role naming who may accept; `AmendProposed` gone with its arms; the withdrawal when the task ends; the rejection reaching the party; the transitional piece in `jig-protocol-engine-clients`; and their tests (`a_partys_amendment_proposal_names_the_party_as_proposer`, `an_amendment_proposal_whose_task_ends_is_withdrawn`, `a_widening_amendment_beyond_the_role_is_refused_naming_who_may_accept`, `a_rejected_amendment_proposal_reaches_the_party_not_the_task`, the three `tests/engine-clients` tests);
     - waits: the acceptance of a party's amendment (`routing/party_decisions.rs`'s arm for it), its funding and any change to `funders.rs` or `control.rs`'s `can_resize`, the Proof's test, and the moved `tests/core/tests/stories.rs` and `tests/people/tests/simulation.rs` tests that accept one. No commit lands an acceptance that charges the task's present funder in the meantime.
     - Unchanged: 03.4's party batch acceptance; `person_proposal_accept`'s task-proposed `Amend` and `Widen` arms on main, until the answer says otherwise; the steer of 22:44Z, point 1 (1050 production lines at most for jig 03's codec-heavy increments, past it a question).

     If the work that may go on is done before the answer, jig 03 commits it as WIP on its branch, reports it, and waits; it lands nothing of 03.5 before the answer's ruling.

190. **jig 03.5: the accepter becomes the amended task's funder** (design answer, jig 03's block of 2026-10-17T00:08:23Z).
     Ruling 190, the designer's answer, landed as jig 1ee6c02ac and temper cf9493c5; it ends ruling 189's interim, and 03.5 goes on whole under its amended Shape (Size 320):


     1. **The accepter becomes the task's funder.** authority.md, 7, "Whoever
        widens a task funds it": "From that commit, the giver is the task's
        funder: *when it already was,* its reservation is resized to the
        task's new budget; *otherwise,* the task is funded anew from it, as a
        moved task is: its old funder is given back what the task had left and
        keeps what it spent, and the giver reserves the task's new budget less
        what the task spent. Its requester, its tree and its delegates'
        funding stay as they were." authority.md, 9: "an amendment or a
        widening makes the accepter the funder of the task it changes, whoever
        proposed it". Nothing is split: "a task has one funder".
     2. **The same rule binds every acceptance of an amendment or a widening**:
        a party's `Amend`, a task's `Widen` and a task's `Amend`, whether a
        party accepts it (`person_proposal_accept`, `party_decisions.rs:242`,
        `:267`) or a task does (`named_proposal_accept`, `runs.rs:123`, `:143`).
     3. **03.5** (temper cf9493c5): `tasks::Event::Amend` gains `funder:
        Option<Funder>`. `None` is a direct amendment, which resizes its
        present funder, as on main. `Some(f)` is the accepter's:
        `Funder::Pool { project, person, period: core.period(project) }`,
        opened first as the batch arms open it (`party_decisions.rs:50`,
        `:220`); or `Funder::Task(key.task)`.
        - `check_amend`: when `f` is the task's funder, `can_resize`
          (`control.rs:340`). Otherwise, the move's preflight
          (`moving.rs:198-229`) at the new budget, refused `Funding`.
        - `apply_amend`: `resize` (:412), or the move's funding
          (`moving.rs:347-368`, `:403-408`), shared through `funders.rs`.
        - The authority check is unchanged (`check.rs:653`).
        - New test: `tests/tasks/tests/funding.rs`,
          `an_amendment_accepted_by_another_funder_funds_its_task_anew`.
        - The Proof asserts the owner's pool as the task's funder, holding
          the widened budget, and the old funder given its reservation back.
        - Size goes from 260 to 320. `control.rs` and `boundary.rs` take at
          most 40 more lines each.
     4. **Deferred: a direct widening by an amender that does not fund the
        task.** The design now says the amender funds it; main resizes the
        present funder, and 03.5 passes `None` there. Its row is in `later.md`
        ("a jig plan after this one") and in jig's `STATUS.md`.
     5. **Unchanged:** 03.4's batch acceptance; who is recorded as `by`; jig
        22's "the pool that funds it, after", which follows authority.md, 7.

191. **jig 13's Shapes against main** (ready check of jig-13.1).
     jig 13's Shapes against main: the re-check of what moved since jig `23e3fbbdb` and temper `0d67b373` (jig 12.2 to 12.5), at jig `73734dd6f` and temper `9abb2f98`, by a read-only sub-agent. 13.1: `Request::Dropped` stays and loses only its `keeper` (`crates/jig-client/src/client.rs:400` tells `Told::Dropped { count: 1 }` from it; `a_kept_record_of_another_version_is_dropped_and_told` keeps passing); its Files gain the users `jig-client`'s `client.rs:118-120, :395-412` and `tests/{store.rs, fixtures.rs:211}`, the link's `component.rs:66`, `tests/client-protocol/src/client.rs:35-36`; the application gives the link only the client's records at `Start` (root.md, 8); `#[must_use]` on `records.rs`'s value-returning functions; temper's `src/tests/family.rs:153-163`. 13.2 and 13.3: moved lines, counts, three comment-only copies, the codec's `family/` files. 13.4: thirteen users its Files missed, `tests/ops/src/joint/front.rs:129-144` among them; the reference test's accessors; the sketches' `#[must_use]` and `to_le_bytes`; the root-map grep as `'\bMap\b|BTreeMap|HashMap'` (the old pattern missed `Map::<u64, u32>` at `route.rs:28`). The bridge's tells and values with no reference UI counterpart (`Told::{Text, Dropped}`, the UI's `Request::Continue`, `Link::{SignedOut, OutOfDate}`, `Why::{NotServed, Limit}`, `Readout::{Limit, NotServed}`, `Progress::NotServed`), and the native host on `Client::new`, go to the designer (question.md) before 13.4 starts. Suites: `jig-boundary` 72/72 with 13.1's and 13.3's manifest edits and `skein-codec` allowed; the `records.rs` and `Owner` sketches pass clippy with the two fixes. Not run: the gate's full suites (no code correction reaches them). Refresh any further copy, and list it in the commit body.

192. **jig 21's Shapes against main** (ready check of jig-21.1).
     jig 21's Shapes against main: the ready check of jig-21.1, at skein `f981746a7`, jig `73734dd6f` and temper `9abb2f98`, by a read-only sub-agent. 21.1: the codec takes and returns the wire crate's own `v1` records, never `jig-ops-client-domain`'s types, since 21.2 makes that crate depend on the wire crate (`cargo metadata`: a cyclic package dependency; `client_wire_links_only_its_codec_and_foundation` forbids it too); 21.2's root maps its values to them, as jig 13.3 does for the test family. `Client::typed` is `Client::unlinked` (jig 12.5); `jig_domain_client::contract` is `jig_client_domain::contract`. Files gain the users the check found in 21.2 (three manifests, `lib.rs:11`, `tests/ops/tests/fuzzy_joint.rs:561`), 21.3 (`crates/jig-client/src/client.rs`, for `Event`'s derives) and 21.4 (both `limits.rs`, the tests building `Event::{Link, Confirmed}`, the sides' `Request::Frame` and `FrameShown`); moved lines are noted per increment. 21.3's `step`'s `wall` and the reference UI's `Request::{SignIn, Fact}` go to the designer (question.md); jig 21 starts meanwhile, and 21.3 waits for the answer. Suites: clippy `-D warnings` on the eight crates the session reaches and the sketch with `Debug`, `PartialEq`, `Eq`; default profile 480/480, fuzzy 60/60; the five proof commands of 21.2 to 21.4 select their tests and pass at the base. Refresh any further copy, and list it in the commit body.

193. **The 00:40Z steer's two rules in the README** (steer of 2026-10-17T00:41:21Z).
     The 00:40Z steer, from the check of 00:40Z: its points 1 (a test's helpers once: `inbox_objects` and `second_party` in `tests/core/src/fixtures.rs`) and 3 (a failure's message built only when the check fails) hold until the run's end, so their Pass on texts are in the README's "The run's steers" (ruling 167). Points 2 (04.7's `[false, true]` loop split, by jig 04.s1) and 4 (`actual` in 12.5's `unlinked.rs`, by jig 13.1) end with their sessions' landings and rest on the README's 18:42Z and 14:09Z rules.

194. **jig 13.4's bridge, and jig 21.3's wall, SignIn and Fact** (design answer, the questions of 2026-10-17T00:33:54Z (re-check of ruling 191) and 00:34:56Z (ready check of jig-21.1)).
     Ruling 194, the designer's answers, landed as jig 64e1ab93b and temper 8a14b8ed: jig 13.4 (Size 370), jig 21.3, jig 21.4 (Size 300) and temper 03.6 go under their amended Shapes, and jig 21.3 waits for no answer now; nothing is deferred.


     ### jig 13.4: what the reference UI's bridge does with the client's tells and values it has no counterpart for, 2026-10-17T00:52Z



     1. **`Dropped { count }` was a gap in the design, and now has a rule.** client.md, 11 now says: "The client tells only how many, `Dropped { count }`, once at start, under no asker: nothing of a record it does not read is read, its key's asker included." root.md, 8 now says: "The root gives the count to one child, which says so: the reference UI, where the application has it". view.md, 18.4 now says: "a notice says that many requests kept from before were not sent". The bridge maps it to nothing until jig 21.4, which shows the notice.
     2. **`Text`** maps to nothing. For a run's handle, the client tells `Changed` and then `Text` in one step (`jig-client-domain/src/watches.rs:110, :112`). `Changed` already delivers the run's document, which holds the client's text.
     3. **`Continue`** reaches no client event: `asked` gives `None`. client.md, 8.3 says: "Opened again, the watch starts from the beginning of the turn being written: the client keeps what it holds". contract.md, 7.3 says the client "is sent the rest when there is more". view.md, 18.1 now says: "It never continues a run's watch".
     4. **The link.** `Starting` and `Offline` map to the UI's link. `Live` maps to `Hello { served: true }`, then `Live`. `SignedOut` maps to `SignedOut`, and `OutOfDate` to `Hello { served: false }`. view.md, 18.2: "signed out and out of date are pages not about the work".
     5. **A sign-out that did not reach the engine.** client.md, 4.2 now says `Link` is told "whenever it changes, and when a sign-out does not reach the engine". The bridge then gives the UI `SignOutLost`. `Client::unlinked` answers `SignOut` at once (`client.rs:236`). In 13.4 it leaves the sign-out to its world instead, so that `a_lost_sign_out_keeps_the_person_signed_in_and_says_so` (`tests/client/tests/shell.rs:81`) runs.
     6. **The client's limit.** `Why::Limit` maps to `Closed`, and `Readout::Limit` to `None(Busy)`. view.md, 18.2 now says: "A handle the client ends, or a read it answers, refused by the client's own limit ..., is one the page left out."
     7. **`NotServed`.** Under jig's asker, only a read while the link is out of date is answered so (`reads.rs:18`). The bridge maps it, and `Family { .. }`, to `None(Refused)`. `Why::NotServed` (mapped to `Unknown`) and `Progress::NotServed` (mapped to `Parked`) come only for a family's handle or request (`domain.rs:140, :175`; `requests.rs:21, :161`).
     8. **Keys (found by the check; not asked).** 13.4's "its tokens as numbers" would repeat a kept key after a reload, because a token is a slot and a generation, both fresh at a restart (skein `id.rs:41-45`). That breaks view.md, 18.1: "draws its keys' numbers from its seed, fresh for each request it means". A key's number is now its token xored with a salt that the root draws from its seed.
     9. **`KeptGone`.** The bridge gives none until 21.4: no owner names a kept request's party once it is answered (root.md, 5).
     10. **The native host.** In 13.4, `jig-client-shell-native` moves onto `jig_client::{Request, Event}`. protocol/client.md, 12 says the shell is the application's "loop around `jig-client`". 13.4's Size goes from 300 to 370, and 21.4's from 290 to 300.
     11. **Nothing is deferred:** no `later.md` row.

     ### jig 21.3: `step`'s `wall`, and the reference UI's `Request::{SignIn, Fact}`, 2026-10-17T00:52Z



     1. **`Ui::step` and `fire` take `wall`.** view.md, 18.1 is the text that was wrong, and the plan's sketch stands. view.md, 18.1 now reads `step(now, wall, &Client, Event, ..)` and says: "the wall time it shows the engine's wall times against ('waiting 4 min')". client.md, 4 now reads `step(now, wall, ..)` and says: "`now` and `wall` are the monotonic instant and the wall time ... (programming-model.md, section 9: `env.now` and `env.wall`)".
     2. **`SignIn` goes, in 21.3.** view.md, 18.6: "Signing in is the application's". pages.md, 8: the sign-in page leaves a slot "which the application's view fills with its own nodes". view.md, 18.1 now says: "It never asks to sign in". `Act::SignIn`, `signin::Provider` and the page's provider list go too. The UI comes back to the place it kept with no `back` ("Once `me` names the party, the reference UI returns to the place it kept", 18.6). The test application's "Sign in with Test" button moves into its slot.
     3. **`Fact` goes, in 21.3.** view.md, 18.1 now says: "A limit that refuses is said where it refuses, on the page or in place (18.2, 18.4), never as a report beside it." No root, world or shell reads a fact.
     4. **temper 03.6** drops the arms its root and its terminal shell have for the UI's `Continue`, `SignIn` and `Fact`. temper signs in by the printed link (temper's `client.md`, 7).
     5. **Nothing is deferred:** no `later.md` row.

195. **jig 03.5: the referee's evidence for an accepted amendment is its durable acceptance** (plan defect, jig 03's block of 2026-10-17T01:00:47Z).
     Ruling 195, a plan defect: 03.5's Files missed the referee's evidence for an accepted amendment or widening. The design says what counts, in its own words. jig's `domain/testing.md`, 5, "Authority holds": "every effect a fake system saw is covered by the scenario's policy as it stood when the effect was decided, for the task that asked, or by a proposal whose acceptance the store made durable first". `domain/authority.md`, 9: "an amendment or a widening makes the accepter the funder of the task it changes, whoever proposed it, ..., its grants covered by the accepter's". So the referee judges an amended or widened task against its accepter, and its evidence is the durable acceptance, as for a batch's tasks: not the funder.

     1. **One map, as on main.** `Snapshot::accepted_tasks` (`testing/jig-conformance/src/referee.rs:232`) holds, for each task, the last durable acceptance that set its authority: a batch member's, as on main, or an accepted amendment's or widening's, `Acceptance { proposal, by }` with `by` the accepter (`Source::Person(person)` for a party, `Source::Task(holder)` for a task). Its doc comment says both. `source()` (:449) and the authority chain (:520, :540) stay as they are. `Task::source` stays the requester, for result delivery and every other reader.
     2. **Nothing else is added:** no funder in `Task` or `Snapshot`, no new map, and no rule for `Funder::{Period, Recurring}`: an acceptance's accepter is a party or a task (ruling 190, item 3).
     3. **`durable()` (:462-501).** An entry still never disappears. It may change only to the acceptance of another proposal that passes the check a new entry passes: the proposal pending in the last snapshot from its proposer (`party_proposals` for a party's; the proposing task's `proposal` and holder for a task's), and the accepter covering it as on main. The three mutations of `tests/core/tests/referee.rs:609-615` stay violations.
     4. **The adapters.** The two that fill `accepted_tasks` from accepted decisions, `tests/core/src/observations.rs:781-795` and `tests/ops/src/conformance/observations.rs:230-236`, add an entry for an accepted amendment or widening, keyed by the task it changes. A further adapter that fills it does the same, and the commit body lists it.
     5. **Tests.** One referee case in `tests/core/tests/referee.rs`: a later amendment's acceptance by another party replaces the entry, and the task is judged against that party. The joint core proof stays as 03.5's Proof says.
     6. **Size.** Within the ceiling 03.5 already has (ruling 190 and the 22:44Z steer): the referee takes at most 40 lines, and each adapter at most 25. Past them, say so in the report.

196. **jig 13.2: a move onto the orphan's path counts blob to blob** (plan defect, jig 13's block of 2026-10-17T01:02:50Z).
     Ruling 196, a plan defect: the count's command misses a move onto a path main already holds. `code-organization.md`, 7 names jig 13.2 among the moves, which "counts only what it adds beyond the move". So `domain.rs` onto the orphan `ui.rs` counts as a move, as you propose.

     1. **The moved file's added lines** are those of `git diff --numstat origin/main:crates/jig-domain-client/src/domain.rs HEAD:crates/jig-client-view/src/ui.rs` (84 at your draft `1f1922722`), in place of `-M`'s 1,280 for `ui.rs`. The removal of `domain.rs` and of the orphan add nothing.
     2. **The rest is counted as section 7 says:** the remaining files' added lines, less those that name a moved path or a new name, and less rustfmt's re-wraps (ruling 118). Within twice the Size (120), 13.2 lands without asking; past it, the report lists the lines by file, as section 7 says.
     3. **The report gives both numbers:** the command's 2,437, and the count with item 1 in place of `ui.rs`'s line.
     4. The plan's `code-organization.md`, 7 now says this for any such move (temper, this ruling's commit).

197. **jig 13.4: the unlinked client asks its world to sign out** (plan defect, jig 13's block of 2026-10-17T01:28:40Z).
     Plan defect. The design does not name `Client::unlinked`: its vocabulary is the plan's (`jig/12-client.md`: "The typed transport (`Client::unlinked`) carries the domain's `Down` and `Up` instead of the link's bytes, for the client's world ... no shell uses it"). Ruling 194 put in 13.4's Shape: "`Client::unlinked` answers `SignOut` at once (`client.rs:236`); it leaves the sign-out to its world instead, which answers it signed out or not reached, as the link does from the wire".

     1. **Yes, the composition-only variants, as proposed:** the unit `jig_client::Request::SignOut`, to the world, and the unit `jig_client::Event::SignOutLost`, from it. Both are unlinked only, as `Request::Down` and `Event::Up` are. `jig-client-domain`'s `Down` and `Up` do not change.
     2. **Under `unlinked`:** `Event::SignOut` sets `signing_out` and pushes `Request::SignOut`, in place of the answer at once (`client.rs:232`). The world answers `Event::Up { up: Up::SignedOut }` (signed out) or `Event::SignOutLost` (not reached). `SignOutLost` keeps the party and does what `signing()`'s `Again | Lost | Refused` arm does (`client.rs:272-277`: `sign_out_lost = signing_out`, `signing_out` cleared), then tells `Link`, as 13.4 has the linked arm tell it. An unlinked `Up::SignedIn` or `Up::SignedOut` clears `signing_out` and `sign_out_lost`, as `signing()`'s `SignedIn` and `SignedOut` arms do (`client.rs:248-262`).
     3. **Under `new`:** `Event::SignOutLost` is a host's mistake and `assert!`s, as `Event::{StartUnlinked, Up}` do (12.5's Shape). `Request::SignOut` is never pushed. Linked clients keep the link's exchange and `signing()`.
     4. **The matches it reaches:** each exhaustive match on `jig_client::Request` gains the arm: `tests/client-protocol/src/client.rs:148` (beside `Down`'s "linked clients emit wire work" panic), `tests/client/tests/unlinked.rs:49-55` (with the arms that panic), and the native shell's arms that 13.4 adds (a linked host: it panics with the others). The client's world takes `Request::SignOut` where it took the UI's sign-out (`tests/client/src/scripted.rs:587`'s `sign_out`): its `lose_sign_out` and `recovery` case answers `Event::SignOutLost`, and otherwise it acts the engine's `SignedOut` and answers `Event::Up { up: Up::SignedOut }`. Refresh any further match on `jig_client::{Request, Event}` the compiler finds, and list it in the commit body. temper names neither.
     5. **Size:** 13.4 goes from at most 370 to at most 380 lines; `jig-client`'s sign-out about 25.
     6. **Tests:** 13.4's `an_unlinked_sign_out_its_world_loses_keeps_the_party_signed_in` (`crates/jig-client/src/tests/`) covers it, with `a_lost_sign_out_keeps_the_person_signed_in_and_says_so` in the gate's client world. Nothing else is wanted.

     Unchanged: ruling 187's drops (`Event::Lost` stays a wire event, never an unlinked one); 13.4's Don't.

198. **jig-13.s1's Shapes against main** (ready check of jig-13.s1).
     jig-13.s1's Shapes against main: the ready check of jig-13.s1, at skein `f981746a7`, jig `c92d4a09d` and temper `4702bd017`, by a read-only sub-agent. 13.2 has landed. The view's own `Limits` (`limits/ui.rs:18`) gains `segments` beside `jig_client_domain::Limits`; the literals are the ones the compiler names, never the scripted engine's `Bounds`, whose `payload.rs` stays. The engine's names' bound is `DocumentLimits`' new `segments`, which no operator sets (`set_documents` destructures it as `segments: _`), set to the most of `authority.segments`, `tasks.hold_segments` and `tasks.authority_segments` once the overrides are applied; the core's worst-case sizers (`task_tree.rs:968-1018`, `offers.rs:221-223`, `project_note.rs:569-577`) size a path by it. This reads the Don't "give the engine a configured limit for a path" as a limit an operator sets, and keeps "derived where the engine's limits are made". The view's node counts from `rows` (`owner_authority::document_nodes`, `proposed_task::part_nodes`) take `segments` for a path's part; two more path writers, the notes filter, and the view's tests that pin a capacity or scale `rows` (regenerated) are in the Files. Test shape 1 holds a name of `tasks.hold_segments` segments, which bounds a held name, in a world at `documents.lines` 2. Lines refreshed (`small.rs:120`, link `limits.rs:41, :51, :89`, view `limits.rs:74-78`, `owner_historical.rs:572-574`). Downstream, the temper step that first pins a jig holding s1 sets `segments` at temper's four literals named in the session file. The check saw 11 view tests fail only under its own partial fixture change (pins and `rows`-scaled tests the step regenerates); no suite failed on main; the workspace gate was not run. Refresh any further copy, and list it in the commit body.

199. **temper 03.4: jig 13.2's repin repairs as temper-03.s2** (plan defect, temper 03's block of 2026-10-17T01:32:47Z).
     Plan defect, the implementer's option A, as ruling 153 added temper-03.s1. The README's Pins say "in at most 200 added lines outside the Shape's Files"; ruling 188 repeats "within the README's 200 lines and jig 08.4's census". Those 200 lines were not sized for jig 13.2's repin, whose required repairs count 340 outside 03.4's Files (415 with path rewrites) before the four new conformance scenarios compile.

     1. **A supplement step, `temper-03.s2`** (`temper/03-s2-repin-sweep.md`), in session 03, before 03.4. It makes the repin 03.4's Needs name (the local repin `eab4babab` and the saved WIP `1f60a44a9` move to it), with every repair it needs in temper's engine, store and conformance adapters and their tests, and the four new scenarios (`Compound`, `Grace`, `ConversationEndsWhileRead`, `QueryWatchedWhileTasksEnd`) with their 12 actions, each run against temper's adapter and passing. A scenario that needs behaviour temper's engine lacks is reported as a question, never skipped.
     2. **Size:** aim for at most 550 added lines; past it, report why. The README's 200 lines do not bind s2; they bind 03.4 again from its next commit.
     3. **It lands alone,** by temper's gate at the jig 13.2 pin; then 03.4 goes on from its Shape on that landing.

     Unchanged: 03.4's Shape and Size; ruling 188's census for 03.4's own work; no lint exception.

200. **jig 13.s1: the cut's bound is the view's own node text; no skein change** (plan defect, jig 13.s1's block of 2026-10-17T01:37:08Z).
     **jig 13.s1: the cut's bound is the view's own node text; no skein change** (plan defect, jig 13.s1's block of 2026-10-17T01:37:08Z). The session file names where paths are written and says "`limits.rs:74-78`'s node text holds a phrase, the mark and `literal_segment::max_bytes(text)`", but not how a writer learns that node text.

     1. **No skein increment and no repin.** skein-ui's `Builder` keeps its limits `pub(crate)` (`crates/skein-ui/src/builder.rs:42`), and ruling 170 made jig 12.2's move "the plan's one jig move of skein".
     2. **The cut's bound is jig-client-view's own node text:** the `text_per_node` that `limits::envelope` derives from the client's `Limits.text` and the phrase (`crates/jig-client-view/src/limits.rs:74-77`). Each root keeps it or widens it with `.max(..)` (`testing/jig-client-test-view/src/limits.rs:57`, `:101`; `examples/ops/client-view/src/limits.rs:61`; temper's `crates/temper-client-view/src/limits.rs:24`), so a path cut by it fits the tree's node in every root. Write the derivation once, in a function that `envelope` and the cut both call (default: `limits::node_text(text: u32, phrase: u32) -> Option<u32>`).
     3. **How it reaches the five writers** (`words/held.rs`'s `part_fact`, `owner_authority::pattern`, `policy_edit::pattern`, `proposed_task::part`, `owner_historical.rs`'s `goal`), the default: a parameter, which each caller takes from what it already holds: the client's `Limits`, or `JigBindings`, which holds the client's text as `text_limit` (`binding.rs:52`) and may gain an accessor for it; a caller that holds neither takes it from its own caller. Another route inside jig-client-view is fine if it is smaller: name it in the report.
     4. **Size:** the threading counts apart from the 160 production lines: aim for at most 80 lines of it; past it, report the count and go on. No temper code calls the five writers; a `pub` signature that jig's ops or test application calls is changed there too, and listed in the commit body.
     5. **Unchanged:** the Don't (no cut in the client's data or the engine's documents; `compose` not raised); what the view acts on takes the whole path (`view.md`, 7: "what the view acts on (a filter by a resource, a draft's pattern) takes the whole path").

201. **temper 03.s2 carries the client sweep jig 13.2's pin forces** (plan defect, temper 03.s2's block of 2026-10-17T01:37:57Z).
     temper-03.s2 carries the client sweep the jig 13.2 pin forces, as the saved WIP `1f60a44a9` holds it. Ruling 199 kept s2 to "every repair it needs in temper's engine, store and conformance adapters" with "Don't: 03.4's Shape work". At jig 13.2, `jig-domain-client` is gone and `link::Saved` is a struct with no `Jig` variant. temper's client crates do not compile without 03.4's Scope, and keeping `Record::Jig` would need a mapping 03.4 deletes.

     1. s2 also carries 03.4's Scope and Files as the WIP holds them:
        - the client crates' and tests' path rewrites: `jig_domain_client::` and `jig_client_domain::`, `pages::` to `page::`, and `limits::Backoff` to `limits::ui::Backoff`;
        - the link surface, in the shell-terminal's `client.rs`, `route.rs`, `limits.rs` and tests, and the domain's `forge.rs`, `tests.rs` and `tests/form.rs`;
        - 03.4's Shape and Removes: `store::Record::Jig`, `Key::Jig`, and the root's `Save` and `Erase` of the reference UI's records.

        03.4's "Don't: keep `jig-domain-client` in a manifest; alias its paths" now binds s2.
     2. s2's Size aim stays 550, counted as added production lines. That is 415 at 01:32Z, which includes the client's about 179. s2 reports its test lines apart, and past 550 it reports why.
     3. s2's Don't becomes: "03.4's new unit test and its Proof (they follow on this landing); keep `jig-domain-client` in a manifest or alias its paths; a lint exception; a skipped or ignored scenario."
     4. 03.4 keeps its Test shape (`the_reference_uis_records_never_reach_the_small_store`), its Proof and its review, on s2's landing, in at most 30 lines.

     If s2 finds a further client file the pin forces, it carries that file too and lists it in the commit body.

202. **temper 03.s2 repairs Grace's forge release in temper's engine** (plan defect, temper 03.s2's block of 2026-10-17T01:52:07Z).
     Option A. The repair is already in s2's scope. s2's Shape (`temper/03-s2-repin-sweep.md`, ruling 199) reads: "with every repair it needs in temper's engine, store and conformance adapters and their tests, the four new conformance scenarios (`Compound`, `Grace`, ...) ... each runs against temper's adapter and passes." The forge release and its `EffectFailed` hold live in temper's engine crates (`crates/temper-engine-domain/src/route/forge_route.rs`, and the `temper-engine-domain-forge*` crates).

     1. The diagnosis and the repair are in s2: aim for at most 30 minutes of diagnosis. Past that, report what was found, with the file:line where the release fails, and go on with the other scenarios.
     2. The regression test goes at the lowest layer that shows the failure: a unit or engine test of the release after a cold start, at most 40 lines, under 20 ms. It is counted with the test lines, apart. The repair's production lines count in s2's 550.
     3. If the diagnosis puts the cause outside temper, report it as a block naming the file:line and the repository, and never skip `Grace`. Outside temper means jig's `Grace` scenario or engine side, or a rule of the design that temper's release breaks.

     This changes nothing else in ruling 201's split.

203. **jig 06.4: a party's amendment keeps its revision; one given amount; stale ends withdrawn** (plan defect, jig 06.4's blocks of 2026-10-17T01:37:39Z, 01:39:18Z and 01:43:34Z).
     These are plan defects. 06.4's Files name `routing/party_controls.rs:80`, which 03.5 removed, and its Shape's "ends the proposal `ProposalOutcome::Stale`" names a state the design does not have.

     1. **A party's amendment keeps its revision (option A).** The design says, in tasks.md: "An amendment names its task's spec's revision ... whoever makes it: a party ... One whose revision has moved is refused as moved". hosts.md says: "`propose`: ... an amendment, naming the revision as `amend` does".
        - `ProposedAmendment` gains `revision: u64`, so `tasks::PersonAction::Amend` carries it as well as `ProposalAction::Amend`. Both are one record (`tasks-v3.schema:134`).
        - The proposal is built at `routing/propose.rs:217` with the revision that `party_amend_revision` (`:193`) checked, priority-only exemption kept.
        - It is checked again at acceptance, in `goal_decide`'s Amend arm (`routing/party_decisions.rs:68`).
        - One test: a party twin of `an_amendment_proposal_accepted_after_its_spec_moved_is_stale`. List the about 11 sites that name both fields in the commit body.
     2. **The delegate reply's given amount: carry one `Option<people::Given>`, not `Box<[Made]>`.** The design gives at most one: "At most one task of a batch gives `AsAllowed` ... The answer says what was given" (authority.md).
        - `BatchKind::Delegation { stubs, given }`, and the `delegating` tuple (`lib.rs:260`) gains `Option<people::Given>`.
        - `MadeRoute::Delegated` gains `given`, and `tasks: Box<[u64]>` is unchanged.
        - The one consuming site (`routing/tasks.rs:428`) builds `CallPart::Delegated`'s `Made { task, revision, given }`: revision 0 for a new task, as 06.4's Shape says.
        - No new `memory.rs` term: the map's slot is priced at `memory.rs:191`.
        - If the run path does not yet refuse a batch with two `AsAllowed` as not well formed, as the party path does (`routing/create.rs:274`, `AsAllowedTwice`), add that refusal and one test.
     3. **A stale proposal ends withdrawn, with no new tasks input.** tasks.md's proposal states are "pending, accepted, rejected or withdrawn", and "Withdrawn ... when the state it was made against has changed so that it no longer applies".
        - When the acceptance's revision check fails, the core sends the existing `WithdrawProposal` or `WithdrawPersonProposal`, naming the proposal's proposer, and keeps a route flag.
        - Once the child's `Withdrawn` reply returns, it answers `CallPart::Moved` or `Refusal::Moved`. Item 1's party twin uses the same path.
        - `StaleProposal` is refused: it would add a pub `tasks::Event` and arms in three matches to buy only a reply tag that the core turns into Moved.
     4. **Size.** 06.4 aims for at most 480 lines. That covers the WIP's +358 and these three answers, about 75 production and 60 test lines. Past 480, report why.

     A further user of a type these items change, which the Files miss, is edited and listed in the commit body.

204. **The 02:00Z steer's run-end texts in the README** (process, jig 13.4's block of 2026-10-17T02:01:28Z).
     The block is right. Ruling 167, instruction 6, says "a steer's Pass on text that holds until the run's end lands in 'The run's steers' before the steer is sent", and the 02:00Z steer's points 1 to 3 hold until the run's end. They are in the README now, word for word as the steer gave them, under "02:00Z, point 1 (jig)", "02:00Z, point 2 (jig)" and "02:00Z, point 3 (jig, tests under `tests/engine-clients/tests/`)". Point 2's Do is done too: 21:01Z, point 1 lists the four files over 800 at jig `795ca6bc3` (`tests/ops/tests/fuzzy_joint.rs` 805, `jig-protocol-engine-clients/src/tests/fixtures.rs` 814, `jig-protocol-engine-clients/src/component.rs` 808, `tests/core/tests/parties.rs` 816), and drops `jig-domain-people/src/boundary.rs`, back to 796. Point 4 was sent (the coordinator's 02:00Z line) and ended with jig 23.1's landing at 02:03:13Z.

205. **jig-13.s2: the test client wire carries the scripted family's records** (plan defect, jig 13.4's block of 2026-10-17T01:44:01Z).
     A plan defect, not a design gap: jig's `client/domain/root.md`, 234-237, answers where the records go, in its own words: the family's records are "in the application's wire crate, `<application>-client-protocol-wire`, which encodes what the owner asks and decodes what the client hands back". 13.4's Shape says the family sides' values cross "encoded by 13.3's codec", and that codec carries only `jig_test_connector`'s `ClientDocument` and `ApplicationOutcome`; the scripted family's `Item`, `PartDocument` (with `Parameters`) and the answers `Touched`, `Moved`, `Gone` and `Refused` have no carrier. Converting them to `Object` or `Panel` would drop their fields and change the stories, which 13.4 keeps ("The worlds' stories keep their names").

     A supplement step, jig-13.s2 (`jig/13-s2-scripted-wire.md`), gives them one, lossless: records in `test-client-v1.schema` (v1 edited in place, as jig `23e3fbbdb` did in this run), code and goldens by the README's "Generated code" command, the codec, and `family/wire.rs` mapping `family::Document` and `family::Answer` whole, every variant. It runs in a slot of its own beside 13.4, as 13.s1 does. 13.4 needs its landing before the family sides' values cross the client; until then 13.4 goes on with the root, the bridge, sign-out and the native shell, as its report says. 13.4's Size (380) is unchanged; s2's aim is 220 hand-written lines, generated code and goldens apart.

     13.4's other block, of 01:40:04Z (the view's `Limits.backoff`/`heartbeat` and timer-only states), is not answered here.

206. **jig-13.s1: the scripted engine bounds a path by segments** (plan defect, jig 13.s1's block of 2026-10-17T02:12:28Z).
     A plan defect, answered by the design in its own words. jig's `client/domain/client.md`, 727-729: "a path's segments: the most segments of a resource's path or a pattern a document may carry, a bound of its own, never a window's rows". s1's Files kept the scripted engine's `payload.rs:80-91` on `bounds.rows` on the premise "s1's tests do not run on it"; the joint role stories admit through it (`tests/client/src/joint/wire/documents.rs:19-26`), so the premise is wrong.

     1. The scripted engine's `Bounds` (`testing/jig-client-scripted-engine/src/lib.rs:38`) gains `segments: u32`. Its `Default` (`lib.rs:51-59`) sets it to its `rows`, 32.
     2. `payload.rs:80-91` bounds a path's or a pattern's list by `bounds.segments`, and every other list by `bounds.rows`, as today. Name in the report where the payload tells a path's list from another.
     3. The joint admission's `bounds` (`tests/client/src/joint/wire/documents.rs:19-26`) passes the client's `segments` beside `rows`. The full literals set it: `tests/client/src/stories.rs:81`, `tests/client/src/stories/client_construction.rs:43`, `tests/ops/src/client/stories.rs:328`. An engine test that pins a path refused by `rows` sets `segments` beside it. Refresh any further literal or such test the compiler or the suite names, and list each in the commit body.
     4. Size: the scripted engine's `segments` counts apart from the 160 and the 80: aim for at most 40 lines; past it, report the count and go on.
     5. Unchanged: s1's Don't ("change what `rows` bounds besides paths" among them); the 13 role stories keep `rows = 2` (`tests/client/src/joint/role_stories.rs:10`) and pass with the gate.

207. **jig 13.4: the view's timers go; behind is the client's** (plan defect, jig 13.4's block of 2026-10-17T01:40:04Z).
     Yes: a plan defect, answered by the design in its own words. jig's `client/domain/view.md`, 1099-1100: "Backoffs and heartbeats are the client's (client.md, section 8)". Where the view's "behind" comes from, `view.md`, 906-907: "a handle of the page whose state the client gives as behind (client.md, 8.1) makes it behind"; `client.md`, 413-416: "*behind,* its watch dropped and opening again, or changed, so the document held may be old; *ended.* A handle that falls behind, or is live again, is told `Changed { handle }`". 13.4's Files omitted the view's `Limits` fields and the states only the timers used.

     1. In `crates/jig-client-view/src/`, `Limits.backoff` and `Limits.heartbeat` go (`limits/ui.rs`), with their readers (`ui.rs:489, :927, :981, :983, :1239`), the silence timer (`Timer::Silent`), `retry` (`ui.rs:986`) and `Following::Reopening` (`watches.rs`, `page/routing.rs:64`, `versions.rs`), and what then has no reader (the re-check saw `rng` and `Domain::new`'s seed, `ui.rs:91, :147`; `Domain::arm`, `:472`; `open`'s `env`, `:477`). A seed the bridge's salt draws from (ruling 194) stays.
     2. The view's `Watch.behind` is set from the client's handle state alone. The bridge gives a `Changed` whose `client.document(..)` is `State::Behind` as a new `WatchEvent::Behind`, whose arm sets `behind` and keeps the document; `State::Live` as `Delivered(..)`, which clears it as `ui.rs:884` does; `State::Opening` as nothing. Its readers (`ui.rs:553`, `page/shown.rs:324`) are unchanged.
     3. `WatchEnd::{Dropped, Busy, Stopping, Closed}` each retire the watch, as `Closed` does today outside recovery: an end the bridge gives is the client's `Ended`, and "Dropped is not ended": the client opens the watch again itself (`client.md`, 418-421). The `SignedOut` path is unchanged.
     4. `tests/client/src/joint/session.rs:118`, `tests/client-protocol/src/fixtures.rs:143, :153` and `tests/client/tests/unlinked.rs:35` take the heartbeat and backoff from the client's `Limits` (`jig-client-domain`'s `limits.rs:19, :29`). Refresh each further site the compiler names (29 at jig `b1213c1e2`), and list it in the commit body.
     5. No new test. The proof of item 2 is `a_dropped_watch_marks_the_page_behind_then_reopens` (`tests/client/tests/faults.rs:103`) and `a_dropped_watch_and_slow_delivery_show_the_page_behind_before_the_held_chat_is_released` (`tests/client/src/joint/stories/fault_stories.rs:248`), passing under the gate, now through the client.
     6. The re-check's corrections are in 13.4's Files: 26 `jig_client_domain` uses in 13 files; `testing/jig-client-test-domain/Cargo.toml`, `Cargo.lock` and `tests/boundary/src/tests/graph.rs:244`, together; `tests/client-protocol/src/client.rs`; corrected paths for `referee_stories.rs:40` and `unlinked.rs:24`; `client.rs:232`; `root.first()` and `root.jig()` exist (`domain.rs:30-36`); the reference test imports `crate::route` and `jig_client::Told`, calls `crate::max_out(..)` and names `Link::Live`.
     7. Size: the removed lines do not count. The added lines of items 2 to 4 count apart from the 380: aim for at most 40; past it, report the count and go on.
     8. Downstream: the temper step that first pins a jig holding 13.4 drops the two fields at `temper-client-domain/src/tests/fixtures.rs:40, :44`, `temper-client-shell-terminal/src/limits.rs:55, :59` and `temper-client-view/src/tests/fixtures.rs:37, :41`, and lists them in its commit body.
     9. Unchanged: ruling 205 (the scripted family's values cross once jig-13.s2 has landed); 13.4's Don't.

208. **jig 13.s2: entrance refusals stay on jig's wire, no test-wire record** (plan defect, jig 13.s2's block of 2026-10-17T02:15:13Z).
     Ruling 208 (plan defect, jig 13.s2): neither a local refusal codec nor a shared one. The test client wire carries no record for `family::Answer::Refused(EntranceRefusal)`. The design puts an entrance refusal on jig's wire, not in the application's bytes: jig's `client/domain/contract.md`, 234-240: "**refused at admission with jig's refusal:** a request the people child refuses there ..., or one whose sign-in has ended, is answered as jig's are (section 4) and never reaches its connector. The client hands that answer to the application under the request's key and owner"; and `protocol/client.md`, 150-152: "**Each application's wire crate** holds its own: its families' asks and their outcomes, its subjects, reads and documents. It is generated the same way, and depends on no crate of jig's but `skein-codec`." 13.s2's Files named `Refused` because `family.rs` holds it, not because anything sends it as the family's bytes: no connector ever answers it.

     - The schema's `FamilyAnswer` is `{ touched, moved, gone, connector: ApplicationOutcome }`; `FamilyDocument` is unchanged from the session file's default.
     - `family/wire.rs` maps every `family::Document` variant, and every `family::Answer` variant but `Refused`, to the wire and back. Encoding `Refused` returns an `Invalid` of its own (default name `Invalid::EntranceRefusal`, beside `Invalid::MissingValue`, `family/wire.rs:10-13`); no `FamilyAnswer` decodes to `Refused`.
     - Test shape: the round trips for item, part with and without parameters, touched, moved and gone, and one test that encoding `Refused` returns that `Invalid`. Size aim 220 stands.
     - jig 13.4: an entrance refusal reaches its root as the client's `jig_client_domain::contract::Answer::Refused(why)`, and the root gives the family's asker `family::Answer::Refused(EntranceRefusal::new(why)...)`, as `tests/ops/src/joint/front.rs:526-530` does for temper's. If 13.4's draft already sends `Refused` through 13.s2's codec, it builds it so instead; nothing else in 13.4 changes.
     - Unchanged: `EntranceRefusal`, jig's client wire (`v8::Refused`) and its link translation; the Don't list of 13.s2 (no change to `jig_test_connector` or its schema; no field dropped, defaulted or inferred). A further place that would put a jig refusal in the test client wire gets the same answer: no record; name it in the commit body.

209. **jig 23.2: own-stop release waits for the designer; Task literal users** (interim, jig 23.2's block of 2026-10-17T02:15:23Z).
     Ruling 209 (interim, jig 23.2; the question went to the designer at 2026-10-17T02:27Z): neither option A nor option B yet. Whether words that release a task the party stopped must name the hold's revision is a design question: jig's `domain/tasks.md`, 522-526, says "a release its hold's (5.5), whoever makes it", `domain/people.md`, 276-277, lists "words that release a task the party stopped" among what a compound request holds, and `Ask::Say { release }` (`crates/jig-domain-people/src/boundary.rs:272`) carries no revision. The designer's answer decides between an exemption in the referee and a revision carried by the words; it comes as a ruling.

     - Until it comes, 23.2 goes on only with what does not rest on it: the rule for spec revisions, for holds released by a named `Release`, and for amendment proposals; the adapters' `Named` from inbound asks and named actions; the Task-literal users; its tests but the story `tests/core/tests/peers.rs:197` (`a_person_stops_a_scripted_run_and_words_release_it_for_a_new_attempt`).
     - Add no exemption, field or variant for a release by words (no `Task.stopper`, no `Observed::OwnStopRelease`, no flag on `Hold::StoppedBy`), and change nothing in `jig-domain-people` or `jig-domain-tasks`. 23.2 does not run `land` until the answer's ruling; when nothing else is left, it reports so and waits.
     - The users of `referee::Task`'s literal, at jig `795ca6bc3`, are four, not two: `tests/core/tests/referee.rs:56`, `tests/core/src/observations.rs:140`, `testing/jig-conformance/src/referee/compound/tests.rs:6` and `tests/conformance/tests/referee.rs:219` (`position_snapshot`). Refresh any further literal the compiler names, and list it in the commit body.
     - Unchanged: the rest of 23.2's session file; its second question (a procedure's named revision) stands as written.

210. **jig 23.2: words release nothing; jig-14.s1 removes the words' release** (design answer, jig 23.2's block of 2026-10-17T02:15:23Z).
     Ruling 210 (design answer, jig 23.2): the designer's answer, landed as jig 5a9a97485 and temper 3cd97ab2; the step its point 4 asks for is jig-14.s1 (`jig/14-s1-words-release.md`, needs jig 14.3). Its decision:


     1. **The first reading: every release names its hold's revision, a release with words too, and the referee exempts none.** No words release a task. people.md, 5.1 now says: "Words release nothing: a `Message` adds words to the inbox, and words to a task the party stopped come with a `Release` in one compound request (below), naming the hold's revision as it would alone." It rests on text the design already had:
        - contract.md, 3.2: `Message` names "a task; the words", and `Release` names "a held task; its hold's revision".
        - contract.md, 3.4: "each names its revision where it would alone".
        - pages.md, 4: "Words to a chat the person stopped release it too: one compound request, the words and the release".
        - people.md, 11: "a person stops a run, then writes to its task and releases it in one compound request".
        - tasks.md, 7: "a release its hold's (5.5), whoever makes it".

        `Ask::Say { release }` and `Operation::Message { release }` (`crates/jig-domain-people/src/boundary.rs:272`, `compound.rs:13`) diverge from it.
     2. **jig 23.2 (in the plan).** The rule stays as its Shape had it: no `Task.stopper`, no `Observed::OwnStopRelease`, no flag on a hold.
        - The adapters read a compound's `Release` and `Amend` as they read the lone asks.
        - The story `tests/core/tests/peers.rs:197` becomes the compound. The fake parties' `Ask::Say.release` becomes `Option<u64>`, the hold's revision. The core world's peers kit sends `Some(hold)` as `Compound [Message { release: false }, Release { revision: hold, again: None }]`. The story reads `hold` from `jig_core_world::fixtures::live_row(&world, task).revisions.hold`, and checks `Reply::Done` where it checked `Reply::Said`.
        - Nothing under `crates/` changes, and the Size stays 60 (`testing/jig-conformance` only). This ends ruling 209's interim.
     3. **jig 14.3 (in the plan).**
        - `OwnerOutcomeSaid` loses `released`.
        - A `Message` translates to `people::Ask::Say { .., release: false }`.
        - The reference UI's words to a stopped chat name the hold's revision their page showed.
        - Size 310 (was 280).
     4. **A step after jig 14.3 (the supervisor's to add) removes the words' release from the engine.**
        - Files, at jig `5a9a97485`:
          - people: `boundary.rs:272, 443`; `compound.rs:13, 27`; `domain.rs:1332, 1385-1386`.
          - core: `sibling_routes.rs:59-114, 443-476`; `routing/party_controls.rs:273-306`; `routing/people.rs:410-411`; `routing/tasks.rs:153-155`; `compound.rs:204-217`.
          - tasks: `inbox.rs:213-262, 396-506, 668`; `boundary.rs:1378-1380, 1651`; `domain.rs:619-623`; `compound.rs:15, 297-305`.
          - store: `people-v3.schema:113, 140, 223`, edited in place as jig 03.5 and 04.6 did, and `codec/people/{asks,compound,outcomes}.rs`.
          - `jig-protocol-engine-clients`: `translate/{asks,answers}.rs`.
        - Shape:
          - people: `Ask::Say { project, task, words }`, `Operation::Message { task, words }`, `Outcome::Said { task, message }`.
          - core and tasks: `PersonMessage`, `inbox::Message`, `Event::Message` and `Event::AuthorizedMessage` lose `release`. `Request::Sent` loses `released`, and `MessagePlan` loses its release plan.
        - Tests:
          - `crates/jig-domain-tasks/src/tests/revisions/compound.rs:125` holds the task `StoppedBy` first, as `:105` does, then sends `[Message, Release]`.
          - The tests of words that release go or send a `Release`: `crates/jig-domain-people/src/tests.rs:1334-1336`, `crates/jig-domain-tasks/src/tests/revisions.rs:593-673`, `tests/people/tests/simulation.rs:512`.
          - The other literals lose the field.
        - Proof: `heavy cargo nextest run -p jig-core-world -E 'test(=peers_scenarios)'`, once, under 1 s; nothing else is wanted.
        - Size: at most 150 lines of production code. Its 17 engine files hold 122 lines naming `release` or `released`, other releases among them.
        - Don't: touch `Amend.release` or a lone `Release`.
     5. **temper absorbs both at its repins.**
        - Past 23.2, the fake parties' `Say`: `tests/system/src/composed.rs:391, 441-442` and `tests/system/tests/composed.rs:42`.
        - Past the step, people's `Say`: `tests/conformance/src/stories.rs:220, 241`, `tests/engine/domain/src/direct.rs:347`, `tests/engine/domain/tests/forge_routes.rs:2068` and `tests/system/src/world.rs:244`.
     6. **Unchanged:** people's `Amend { release: Option<u64> }` names its revision, and stays. Nothing is deferred: no `later.md` row.

211. **jig 13.s2 carries the family's subjects and touch whole** (plan defect, jig 13.4's block of 2026-10-17T02:37:03Z).
     Ruling 211 (plan defect, jig 13.4 and 13.s2): both carriers whole, as 13.4 proposes, and 13.s2 carries them. jig's `client/domain/root.md`, 42-45: "**Families as bytes.** An application's requests, reads and subjects ride the client as bytes under their owner's number and version, on the same connection and through the same request table; its wire crate encodes and decodes them (section 7)." 13.s2's Files named the family's documents and answers only, and its "`Invalid::MissingValue` ... stays" assumed no story sends a touch without parameters; the first side sends one for an untouched item (`first.rs:741-751`).

     - 13.s2: the test client wire carries `family::Subject` whole (both scopes, every `Kind`, `Effect(ProposalKey)` and `OwnerEffect` apart), default its draft's `FamilySubject` in `Subject` and `Read`; and `Ask::Touch` with `parameters: Option<u64>` as it is, default `value: option`. `subject` and `request` return the family's value whole; `Invalid::MissingValue`, `Invalid::Scope` and `Invalid::CoreEffectReference` go, with their tests, where nothing else returns them. Size aim 540 (was 220). Each match the compiler names (`testing/jig-test-service`, `tests/client-protocol`, `tests/client/src/joint/requests.rs:86, :101, :150`) is in its Files; list each in the commit body.
     - What the test service answers for a subject it holds no row for stays as it is; a story that needs another answer is a block.
     - 13.4: the root tests keep their requests and subjects (`root.rs:489`'s `parameters: None`; the deployment page's six reads and the scoped outboxes); nothing infers a value. Its reads and touches cross once 13.s2 lands; until then it goes on with the hosts, the view and the rest.
     - Unchanged: ruling 208 (no `Refused` record); 13.s2's Don't list; the test connector's types and schema.

212. **jig 03.5's dropped `run_spent <= spent` checks stand** (scope, the check of 2026-10-17T01:59:16Z, point 4).
     The drop stands, and nothing replaces either check. jig 03.5 (`b1213c1e2`) removed, without a ruling, `&& task.run_spent <= task.spent` from the referee's Spend promise (`testing/jig-conformance/src/referee.rs`, :554-556 before it) and `|| task.run_spent > task.numbers.spent` from `crates/jig-domain-tasks/src/stored.rs`'s `valid_record` (:340 before it). Both were wrong once a task is funded anew: `funders/refunding.rs:90` sets `numbers.spent = 0` on an accepted Amend or Widen (`control.rs:427`) and on a Move (`moving.rs:326`), and `run_spent` goes back to 0 only at the next claim (`run.rs:104`). The design names no per-run counter: `git grep run_spent origin/main -- docs` in jig prints nothing. Its promise is authority.md 31-32, "Recorded spend never exceeds a budget. A run reserves each completion's maximum before making it", which the referee keeps (`referee.rs:559-563`, `total <= task.budget && task.run_reserved <= task.reserved`), and authority.md 290-293 says a task funded anew's "old funder is given back what the task had left and keeps what it spent".

     Checked: with both checks restored at jig `40d1bec49`, `heavy cargo nextest run --no-fail-fast -p jig-conformance -p jig-domain-tasks -p jig-core-world -p jig-tasks-world` ran 579 tests, 1 failed: `an_amendment_accepted_by_another_funder_funds_its_task_anew` (`tests/tasks/tests/funding.rs:399`), "world store corrupted: Problem { task: Some(1), why: Restore, .. }" at the restart after `(spent, run_spent) == (1, 4)` (the run's `logs/20261017T030505-679.log`).

     No session changes anything for this ruling.

213. **temper-03.s2: 30 minutes for the merge story's derived bound** (plan defect, temper 03.s2's block of 2026-10-17T03:23:15Z).
     Ruling 213, plan defect: temper-03.s2 gets 30 minutes more for the merge story, aimed by a cause already known. The first refusal is the pinned core's: at jig `c92d4a09d` (s2's pin, jig 13.2), `task_tree.rs:328-329` bounds an adopted resource path by `limits.documents.lines`. jig main bounds it by `limits.documents.segments` since 13.s1 (`40d1bec49`), as `client/domain/client.md` 727-729 says: "a path's segments: the most segments of a resource's path or a pattern a document may carry, a bound of its own, never a window's rows". Raising the fixture's `documents.lines` stands in for the missing `segments`, and the stream that then drops at Time(0) right after Hello is a second effect of that raise, not of the merge.

     1. s2 keeps its pin at jig `c92d4a09d` (ruling 199): no repin in this ruling.
     2. First look, at most 30 minutes from receiving this: run the merge story with the fixture's `documents.lines` at 4 and at 8, and compare the first refusal or close that each run logs before `Notice(Fact(StreamDropped))` (server side and client side). The question to answer: which bound derived from `documents.lines` (an envelope, a window, a frame or a body size) passes a bound of the transport or of the HTTP server at 8.
     3. If it is a derived bound of a fixture: raise the fixture's paired bound as well, in the fixture only, with one line of comment naming this ruling and jig 13.s1's `segments`; then land s2.
     4. If the 30 minutes end with no cause, or the cause is outside temper's fixtures: stop, put the fixture's `documents.lines` back to 4, and block again with the two runs' first lines and logs. The next ruling then weighs moving s2's pin to jig `40d1bec49`.
     5. Unchanged: rulings 199, 201 and 202, the fuzzy profile in s2's gate, and 03.4's turn after s2.

214. **jig 06.5: a task's decided proposal from its proposer's history row** (plan defect, jig 06.5's block of 2026-10-17T03:20:41Z).
     Ruling 214, plan defect, answered by the design in its own words. jig `docs/design/protocol/store.md` 383 (7.3, Parties' reads): "| a proposal | a party's: its record, by key, with its decision; a task's: the row of its proposer's history that keeps it |". 06.5's Needs put both under `tasks::Key::DecidedProposal(number)`; main keeps only a party's there (jig 11.2, `c231752c5`), and the design keeps a task's in its history.

     1. A task proposer's `Event::DecidedProposal` row is the `proposal` of its `Key::History { task, revision }` record (`Stored::History { proposal: Option<Box<Proposal>> }`, `control.rs:104-113`), the revision found in the owner's `proposal_rows` by `kept.proposal == proposal`, as the store's `reads/point.rs:79-88` already finds it. Share that lookup with `point.rs`; do not copy it.
     2. A proposal no longer in `proposal_rows` (it keeps the newest `inbox_messages`) is the row `None`, which the Shape answers `ProposalRefused(Unknown)`.
     3. Not `CoreRecord::ProposalDecision`: it holds the outcome only (`jig-domain-core/src/stored.rs:650-661`), and `decision_record` writes none for a withdrawn proposal or one decided by a task or a deployment (`proposals.rs:383`).
     4. No change to `Stored`, `Key`, the tasks-v3 schema, the codec, generated code or goldens. 06.5's Files gain `crates/jig-protocol-store/src/reads/point.rs:61-90`. Size stays at most 150 lines.
     5. Unchanged: the party arm (`DecidedProposal(number)`), the Shape's answers, its Test shape and its Don't ("keep decided proposals in memory to answer this").

215. **jig 13.4: Size re-estimated to 850, stop at 950; no split** (plan defect, jig 13.4's block of 2026-10-17T03:42:54Z).
     Ruling 215, plan defect: jig 13.4's Size is re-estimated, and 13.4 goes on as one increment. Its Size, "at most 380 lines (the root about 120, the bridge about 170, the family sides about 40, the native shell about 25, `jig-client`'s sign-out about 25, ruling 197)", was set before ruling 211 gave the family adapter "the family's subjects and touch whole" and before ruling 207 removed the view's timers. The draft counts 703 at `92274c78e` (`logs/jig-13/size-stop-20261017.txt`): the family side 176, the bridge 133, the test root about 200.

     1. **No split.** No part lands by itself: the root's `RequestTable` goes with its replacement (13.4's Removes), and 10 of the 23 root tests failed at the last run.
     2. **Size:** the 703 stand. Aim to land at most 850 production lines, counted by the command of the 03:42Z report, ruling 207's items 2 to 4 included (its 40 apart no longer apply); past 850, the report gives the count by file and why, and the session goes on. At 950, stop and block with the count by file. Counting at each commit stays. This replaces the 02:57:44Z steer's stop at 700 for 13.4, until 13.4 lands.
     3. **The handle offset stands.** The test root's `reads: Slab<Reading>` and `watches: Slab<Watching>` (`testing/jig-client-test-domain/src/first.rs:178-179`) issue the same token values, so an `unwatch(handle)` removed a read. Read slots numbered above watch slots, in the root and in the view's bridge, as the report gives them, is a fix the Shape's "a handle's number its token's" needs; the design leaves an application's tokens its own (`client/domain/view.md`, 647: "an application's are its own, as its tokens are"). The commit body names the collision with its file:line and the offset. No new test: the root tests that failed on it are its proof.
     4. **The fixes outside Files** (`cards::contains` widened to `pub(crate)`; the test view's dev dependency on the wire crate) stand, within the README's 20 lines each; list each in the commit body, and each further one.
     5. **Unchanged:** 13.4's Shape, Removes, Test shape and Proof; the 02:57:44Z steer's points 2 and 3; ruling 211.

216. **temper 03.s2: the pin stays; the merge story takes ruling 163's mark until temper 04.4** (plan defect, temper 03.s2's block of 2026-10-17T03:28:34Z).
     Ruling 216, plan defect in the order of steps, as ruling 163 (`decisions.md`, 163, point 2): "A further test that fails only on a forge path's segments gets the same mark, listed in the commit body and the report; past three such tests, stop and report." At s2's pin, jig `c92d4a09d`, the core bounds an adopted path by `documents.lines` (`task_tree.rs:329`); the merge story's branch resource has 8 segments by construction (`forge_route.rs:271-300`: 4, `branch`, and `temper/1/c2`'s 3), so no path of 4 segments carries it, and `documents.lines` 8 passes the v8 wire's ceiling of 4. The designed fix is jig 13.s1's `segments`, which temper 04.4 takes.

     1. **The pin stays** at jig `c92d4a09d` (ruling 199). A repin to `40d1bec49` was tried in a scratch worktree: 19 production and 8 test lines of repairs, and the story still failed, at step 18 (`world.rs:442`, `person step 18: Missing { find: Text "@plan Fix the adopted repository" }`), with jig 23.1's and 03.5's proposal changes in the pin. The README's Pins take "as few other jig landings as it can".
     2. **s2 marks** `a_merge_in_progress_is_pushed_with_both_parents` (`tests/one-process/tests/fuzzy_one_process.rs:39`) `#[ignore = "a forge branch's eight segments need jig 13.s1's segments, temper 04.4 (ruling 216)"]`, with `documents.lines` at 4 as now, and names it in the commit body and the report. It is the second test under ruling 163's point 2; a third that fails only on a forge path's segments takes the same mark; past three, stop and report.
     3. **Then s2 lands** by temper's gate, the fuzzy profile included, with the cache observer's fix in `tests/one-process/src/referee/client/cache.rs`.
     4. **temper 04.4** removes this mark too, and its Proof runs the story once (its Files and Proof say so now, with the step-18 failure seen at `40d1bec49`).
     5. **Unchanged:** rulings 199, 201, 202 and 213's points 1 and 5; s2's Don't, whose "a skipped or ignored scenario" means the conformance scenarios; every assertion of the story; 03.4's turn after s2.

## For the user

The choices below have long-term consequences; each is the planner's,
under the user's delegation, and the user may veto it before the run:

- **A. temper's scope** (1c): temper builds the pages its stories use
  now, and its board, changes page, agent's task page and browser wait
  for temper's next plan. The alternative, temper's whole "temper on
  jig's client API" here, adds about 900 lines and the rebound-Host
  story.
- **B. The naming check lives in `jig-boundary`** (8), which temper then
  takes as a dev-dependency. The alternative is a copy of the check in
  temper.
- **C. The plan lives in temper** (2), so jig's `STATUS.md` row names
  the plan without its repository.
- **D. Two items of jig's step 5 wait** (15): a client in the engine's
  process, and the clients' listener off loopback with TLS. The
  alternative for the first is an increment in jig 18 (about 150 lines)
  with a test-only client.
- **E. The process tier on carbon-forge** (ruling 111, 2026-10-16): the
  runner user owns no cgroup here, so skein's tree keeper allows one
  observer per process, and temper's process stories, which start two
  binaries, fail with `Tree(Other(16))`. Until the machine gives it one,
  temper's Proofs on the process tier run one layer down (implementer.md
  8.8). The fix needs root, takes effect at once, needs no restart of
  pueued, and lasts until the next reboot:
  `sudo sh -c 'mkdir -p /sys/fs/cgroup/runner/pueued && echo "+cpu +memory" > /sys/fs/cgroup/runner/cgroup.subtree_control && chown -R runner:runner /sys/fs/cgroup/runner && pgrep -x pueued > /sys/fs/cgroup/runner/pueued/cgroup.procs'`.
  Default: one layer down until the user runs it.
