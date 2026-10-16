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
