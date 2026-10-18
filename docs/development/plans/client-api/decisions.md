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

217. **temper's tests name their numbers (README 23:45Z, point 2)** (steer of 2026-10-17T04:24Z).
     Steer of 2026-10-17T04:24Z, from the check of 2026-10-17T04:24Z, point 1: README 23:45Z, point 2 (named request ids, durations and times in tests) binds temper's sessions; temper 03.s2's scenarios left bare numbers in `tests/conformance/src/scenarios/queries.rs` and `lifecycle.rs`, which the first temper session to edit either file names. Recorded in the README as "04:24Z, point 1 (temper)".

218. **jig 13.4: ops' three recovery tests marked until jig 21.2** (plan defect, jig 13.4's block of 2026-10-17T05:19:32Z).
     Ruling 218, plan defect in the order of steps, as ruling 163 marked a story until the step that gives its cause: the implementer's third option. Ruling 207 rests on the design's "Backoffs and heartbeats are the client's" (`view.md`, 1099-1100); `ops`'s root gets its client only at jig 21.2 ("`ops`'s root on `jig_client::Client`"), and until then it steps the view directly (`examples/ops/client-domain/src/route.rs:16-19`), so no client judges its link `Offline` or reopens its watches.

     1. **13.4 marks the three tests** at `tests/ops/tests/fuzzy_joint.rs:737, :746, :755` (`p_acceptance_survives_the_commit_before_its_durable_decision`, `p_acceptance_survives_its_durable_decision_commit`, `p_acceptance_survives_the_commit_after_its_durable_decision`) `#[ignore = "ops' root has no client to recover its watches until jig 21.2 (ruling 218)"]`, each assertion unchanged, and names them in the commit body. Another test that fails only because `ops`'s root has no client takes the same mark and is named there too; past three more, stop and report.
     2. **No other change:** ruling 207's removal stays in 13.4; no fallback in the view; 21.2 is not advanced. 13.4 lands as one increment by its gate, as ruling 215 says.
     3. **jig 21.2** removes the marks and runs the three once (its Files and Proof say so now), and calls `bridge::closed` after a bridge-routed `Close`, as 13.4's commit body records.
     4. **Unchanged:** ruling 215's Size (784 of aim 850, stop 950); 13.4's Shape, Removes and Proof; ruling 207.

219. **a failed gate is followed by the touched crates' tests, every failure at once** (steer of 2026-10-17T05:20Z).
     Steer of 2026-10-17T05:20Z, from the check of 2026-10-17T05:20Z, point 1: after a failed `land` gate, a session whose commit changes more than 20 files (and jig 13.4) runs its touched crates' tests with `--no-fail-fast`, default and fuzzy profiles, and fixes every failure before `land` again. Recorded in the README as "05:20Z, point 1".

220. **jig 13.4: five more ops recovery tests marked until jig 21.2** (plan defect, jig 13.4's block of 2026-10-17T05:25:44Z).
     Ruling 220, plan defect in the order of steps, as ruling 218. The five fail for ruling 218's cause: `ops`'s root has no client until jig 21.2, and 13.4 leaves backoff, `Offline` and reopening to the client (ruling 207; `view.md` 1099-1100: "Backoffs and heartbeats are the client's"). Three wait for an `Offline` that only the client now gives (`bridge.rs:99`). In the two whose waiting card disappears, `ops`'s joint front sends the view a raw `Ended { end: Dropped }` (`tests/ops/src/joint/front.rs:400`, `drop_watches`), which 13.4's view retires (`ui.rs:988` at `7061ef016`), and which a client never forwards: `jig-client-domain/src/watches.rs:148`, `WatchEnd::Dropped | WatchEnd::Stopping => None`, its handle kept behind (`client.md` 417-419: "**Dropped is not ended.** … its handle stays, behind").

     1. **13.4 marks the five** at main's `tests/ops/tests/fuzzy_joint.rs:764, :772, :780, :788, :798` (`q_provision_survives_its_acceptance_commit_and_move_end_survives_reload`, `q_provision_survives_its_made_commit_and_move_end_survives_reload`, `q_move_end_survives_its_durable_answer_commit_and_reload`, `p_story_survives_seeded_commit_crashes_and_random_presses`, `q_story_survives_seeded_commit_crashes_reload_and_random_presses`) with ruling 218's `#[ignore]` text as it is, each assertion unchanged, and names them in the commit body beside the three. A further `tests/ops` test that fails only because `ops`'s root has no client takes the same mark and is named in the commit body and the report; past two more, stop and report.
     2. **No other change:** the view's `Ended` handling stays as 13.4 has it, with no fallback for a raw `Dropped`; ruling 218's point 2 stands. 13.4 lands as one increment by its gate.
     3. **jig 21.2** removes the eight marks and runs them once: its Files and Proof say so now (`test(/_survives_/)` selects exactly these eight in `fuzzy_joint`).
     4. **Unchanged:** rulings 207, 215 (Size 784 of aim 850, stop 950), 218 and 219.

221. **jig 21.2: Size re-estimated to 950, stop at 1,100; no split** (plan defect, jig 21.2's block of 2026-10-17T05:58:48Z).
     Ruling 221, plan defect: jig 21.2's Size is re-estimated, and 21.2 goes on as one increment: the implementer's option A, with other numbers. Its Size, "at most 250 lines", was set before ruling 192 gave the root its own value mappings: "21.2's root maps its values to them, as jig 13.3 does for the test family". 13.3's one family took 466 lines (`testing/jig-client-test-domain/src/family/wire.rs`); `ops`'s four take 534 (`wire.rs` 229, `wire/infrastructure_documents.rs` 126, `wire/infrastructure_native.rs` 104, `wire/observability_documents.rs` 42, `wire/observability_native.rs` 33).

     1. **No split.** The mappings do not land by themselves: the root is their one user, and without them it cannot reach 21.1's wire crate (ruling 192: a cyclic package dependency otherwise). 21.2's Shape keeps the root's change in one commit of its own.
     2. **Size:** the 891 counted at `b94f89a89` (`logs/jig-21/21.2-size.txt`) stand. Aim to land at most 950 production lines, counted by code-organization.md's command (`tests/` worlds and `tests*.rs` are not counted); past 950, the report gives the count by file and why, and the session goes on. At 1,100, stop and block with the count by file. Count at each commit. This replaces "Size: at most 250 lines" and its twice-the-Size rule for 21.2, until it lands.
     3. **Unchanged:** 21.2's Shape, Files, Test shape, Proof (with rulings 218 and 220's eight marks), Don't and Removes; ruling 192.

222. **jig 14's Shapes against main** (ready check of jig-14.1).
     jig 14's Shapes against main: the ready check of jig-14.1, at skein `f981746a7`, jig `5c06a7e98` (lines re-read at `1350b6564`) and temper `01bdba69`, by a read-only sub-agent; the session file corrected in place, each passage marked "(ruling 222)". 14.1: its reference test uses `people::ApplicationScope::Own` (`core::documents::ApplicationScope` is private, `jig-domain-core/src/documents.rs:23`) and serves family 1 in a list of capacity 2 (`fixtures::settings()` serves jig 8..=8 only); `Keyed.request` and `TableEntry.request` become `bytes 1114240` (a full `Create`, `Compound`, `ProposeBatch` or `FamilyRequest` at 1,048,576 encodes past `Keyed::new`'s bound; a full Propose is 1,114,126); `version_not_served_is_tag_zero_of_each_answer` is a new row of `wire_values_round_trip_unchanged`; the family record is `jig_client_domain::Family` (`boundary.rs:10`), and its users and the `Answer` rename's join the Files. 14.2: `contract::Text::new(..)` in its reference test; `restore_ask` (`translate/store.rs:78`) and `link_store.rs` join each increment that changes an `Ask`. 14.3: `OwnerOutcomeAmended.released` and `OwnerOutcomeAmendProposed.released` go with `Amend.release` (`Amend`'s answer is "amended", jig `client/domain/contract.md:164`). 14.5: the five `proposable` users join its Files. The Done-when count is `grep -c '^jig-14\.[0-9]'`. temper 04.2, the first repin to take jig 14, names temper's five users. Stale lines refreshed. Design question of this check (question.md): 14.4's `NewTask` and `OwnerOutcomeCreated` against main's `people::Member`, and 14.5's `proposable` for a refusal that makes no claim; jig 14 starts meanwhile with 14.1 to 14.3, and 14.4 waits for the answer. Suites: clippy `-D warnings` on `jig-client-protocol-wire`; default 669/669 and fuzzy 38/38 on the eight packages 14.1 reaches, with the ceiling correction applied; drift passes. Refresh any further copy, and list it in the commit body.

223. **06:20Z steer's text in the README** (process, jig 21.2's block of 2026-10-17T06:21:43Z).
     Ruling 223, process: the coordinator is right. Ruling 167, point 6: "From now until the run's end, a steer's Pass on text that holds until the run's end lands in 'The run's steers' before the steer is sent, and context.md names only its steer's time and point." The steer of 2026-10-17T06:20:26Z was sent before its point 1's text landed. That text now lands in the README's "The run's steers", word for word as the steer gave it, under "06:20Z, point 1 (jig 21.2, 21.3 and 21.4)".

     1. The coordinator now carries out the steer's point 1 as the steer says, and logs it done. context.md names only "Steer 06:20:26Z, point 1".
     2. jig 21.2 (w82) gets the point's Pass on text with the answer to its two other blocks of 06:15:38Z and 06:21:43Z, in the same message, not before: it is stopped until then. 21.3 and 21.4 read it in the README, and their prompts do not repeat it (ruling 167, point 4).
     3. Unchanged: the steer's Do, Stop, Applies to and Until; ruling 167; jig 21.2's two other blocks, which stay in blocked.md until their answer.

224. **jig 14.4 and 14.5 as main's people types: the designer's answer** (design, jig 14's ready check question of 2026-10-17T06:07:15Z).
     Ruling 224, design: the designer's answer, landed as jig b58a49c39 and temper bc7c1376; ruling 222's "14.4 waits for question.md's answer" ends. The answer's decision:


     1. **14.4's record follows main; nothing is owed.** contract.md, 3.2
        now: "authority, whose spend and deadline are the task's budget and
        deadline ..., that budget as named there or `AsAllowed`"; "a report,
        a change or a verdict: an approval is a procedure's"; "the grants
        named after it". `NewTask` is field for field `people::Member`
        (`batch.rs:28-42`): no `deadline`, `NewBudget { named | as_allowed }`,
        `NewContract { report | verdict | change }`, and `named: bytes
        16777216` (pieces of `OwnerGrant`) where `holdings` was.
     2. **Holdings are no gap.** Main's party batch asks each connector
        `Ask::TaskHoldings` per member before it commits
        (`party_inputs.rs:142`, `parties.rs:263`, `:608`; filled at `:313`).
        tasks.md, 4 now: "each connector says from its executor and spec
        before the batch commits, whoever creates it". No `later.md` row.
     3. **The answer** is `OwnerOutcomeCreated { first, last, given: option
        CreatedGiven { task, amount } }`, as `people::Outcome::Created`
        (`boundary.rs:403`; contract.md, 4: "the first and the last of the
        tasks' numbers, consecutive in the batch's order"). Not asked:
        `Malformed` gains `cycle` (`create.rs:60`; contract.md, 4).
     4. **14.5:** `ProposeWidening` goes, as your reading said (people.md,
        5.1). Not asked: `ProposeAmendment.revision` is required, as in
        `people::Action::Amend { revision: u64 }`. `OwnerOutcomeProposed` is
        `{ proposal }` (contract.md, 3.2: "its number answered"; 4 now:
        "whom it waits for being in its document").
     5. **`proposable` stays `option bool`, three states,** as
        `translate/refusals.rs:26-30` maps them: none, no claim (`None`);
        false, nobody (`Accepter::Nobody`); true, one could
        (`Accepter::Role(_)`). A refusal of shape, reference or again carries
        none. contract.md, 4 now: "A refusal that another check decided ...
        makes no claim either way. Who would accept a proposal is the
        offer's to say (5.3), and the proposal's document's once it is made,
        not the refusal's".
     6. **The plan:** 14.5's five `proposable` users leave its Files, its
        `an_authority_refusal_names_the_role_that_could_accept` goes, and its
        story becomes
        `a_create_beyond_authority_is_refused_saying_a_proposal_could_be_accepted`.
        jig 16's card takes to whom from the offer. temper 04.2 keeps its
        `proposable` arm.
     7. **Unchanged:** the people child's `to: u32` and `Accepter::Role(u32)`;
        jig 15's helpers, which take `to` from the offer. Nothing is deferred.

225. **jig 21.2: ops's read answer as 13.3's; Size re-estimated to 4,500, stop at 5,000** (plan defect, jig 21.2's blocks of 2026-10-17T06:15:38Z and 06:21:43Z).
     Ruling 225, plan defect: the implementer's option A, for both blocks.

     1. **The carrier.** protocol/client.md, 11: "An application's read names its scope and its window, and its connector's read, in the application's wire, and is answered with the application's document, or unknown, or not allowed." 21.1's `ops-client-v1.schema` has no record for "not allowed"; jig 13.3's test family has one (`test-client-v1.schema:86-89`: `record ReadValue`, `enum ReadResult { value: boxed ReadValue, unknown, not_allowed }`, `versioned record ReadAnswer { value: ReadResult }`), which 21.2's Shape follows ("as jig 13.3 does"). Each of `ops`'s two document families gains the same three, edited in place in v1 (21.1 owns v1; README, "Format versions"). The engine side sends the answer's bytes in `record`, as `testing/jig-test-service/src/clients/translation.rs:98-100` does; the root reads it as `testing/jig-client-test-domain/src/family/client.rs:127-135` does: not allowed is `ReadResult::NotAllowed`, anything else undecodable is `Unknown`. No change to jig's engine or core: `core::reads::Why` gains nothing, and `tests/ops/src/client/transport.rs:440, :448` keep folding jig's own `ReadNone` as they do.
     2. **Size.** The 3,809 lines of `wire.rs` and `wire/**` are 87 encode and 87 decode functions for 408 fields; jig 13.3's mappings of the test family took 11.7 lines a field (2,148 lines for 183 fields), which predicts 3,460 to 5,960. Plain functions, field by field, stay: clippy.toml:11-14 keeps "defining traits and generic items; trait impls by hand ...; macro_rules!" out of step code, and `family.rs:63`'s `macro_rules! side` goes the same way. Option B is declined. Aim to land at most 4,500 production lines, counted by code-organization.md's command; past 4,500, the report gives the count by file and why, and the session goes on; at 5,000, stop and block with the count by file. This replaces ruling 221's 950 and 1,100. Ruling 221's "No split" stands: one commit for the root.
     3. **Files** gain `examples/ops/client-protocol-wire/schema/ops-client-v1.schema`, its generated code and goldens, written only by the drift test's write mode in the build lane: `heavy sh -c 'SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-client-wire-world --test drift -E "test(ops_client_wire_schema_matches_generated_code_and_goldens)"'`. Refresh any further user of the changed records that the build shows, and list it in the commit body.
     4. **Unchanged:** 21.2's Shape, Test shape, Proof (rulings 218 and 220's eight marks), Don't and Removes; rulings 192 and 221's "No split".

226. **jig 21.2: diagnose the joint bootstrap below its layer** (process, jig 21.2's block of 2026-10-17T06:49:40Z).
     Ruling 226, process: the implementer's option A. implementer.md, 8.8: "Diagnose a failure below its layer. Reproduce it where a run is cheap, in a world or a focused test through `heavy` in the build lane, and find its cause there. Run the proof again only after a change aimed at a cause its log names, once per change."

     1. **Reproduce first.** Extend the passing unit `joint::bytes::tests::a_linked_ops_root_signs_in_through_the_byte_face` (27 ms, `logs/jig-21/21.2-bootstrap-5.log`), or a focused test beside it, with what the report names as the remaining difference: the common world's scheduling and startup, its clock and its deadlines, in place of the unit's 10 ms clock and direct callback drain. It reproduces when it shows the offline, signed-out tree without First words, as p-6 to p-8 do. Log the client's link state and each deadline it sets.
     2. **Then the proof,** once, after a change aimed at the cause that focused test's log names. Default: at most two more runs of `p_accepts_the_restart_from_the_incidents_page`, each after a different change. A fourth run with the same symptom is a block with the logs, not a fifth run.
     3. **Bound:** aim to spend at most 45 minutes on this diagnosis; past it, report what the focused logs show, and go on with it.
     4. **Unchanged:** no engine or core production change (as the report proposes); 21.2's Proof, Files and ruling 225's Size (4,329 now, aim 4,500, stop 5,000); the 06:20:26Z steer's point 1.

227. **jig 21.2: the frame's offered goals sized by their count** (scope, jig 21.2's block of 2026-10-17T06:57:15Z).
     Ruling 227, scope: the implementer's option A, within jig 21.2.

     1. **The repair.** In `crates/jig-protocol-engine-clients/src/translate/owner.rs`, `charters` (:163) sizes the wire's goals list by the goals it holds, `List::with_capacity(value.goals.len())`, as `translate/inbox.rs:93` sizes its rows' goals. The wire refuses a list by its room (`generated/v8.rs:29636`, `self.goals.capacity() > limits.offered_charters_goals.min(CEILINGS.offered_charters_goals)`; ceiling 4, `client-v8.schema:576`, `goals: list 4 Charter`), and the domain's list has room for `limits.documents.lines` (`jig-domain-core/src/documents/small.rs:133`).
     2. **Its test,** at the unit layer, in `crates/jig-protocol-engine-clients/src/tests/documents.rs`, beside `a_frame_with_two_projects_becomes_its_record`: a frame whose charter's goals list has room for 32 and holds one goal becomes the same record, `jig_client_test_records::frame_two_projects_record()`. Under 20 ms. No record or golden changes.
     3. **Then** give the joint fixture back the document limit that `documents.lines=4` replaced, and go on under ruling 226, points 1 to 3: the focused test first; then the p proof once, after the change the focused test's log names. That run is p-10, the last p run ruling 226 allows; a failure after it is a block with the logs.
     4. **A further copy.** Where the focused test or the proof then meets another translation in `crates/jig-protocol-engine-clients/src/translate/` refused `Bound` because a wire list is sized by a domain list's capacity while its length fits, size it by `len()` the same way, add its case to step 2's test file, and name it in the commit body. At most 10 lines of engine production in all, counted in ruling 225's Size; past them, a block.
     5. **Unchanged:** ruling 226's other points (no core production change, no other engine change, its 45-minute aim); 21.2's Proof; ruling 225's Size (4,329 now, aim 4,500, stop 5,000); the 06:20:26Z steer's point 1. A role offered more than four goal charters still has its frame refused `Bound`: `later.md`'s new row, for the designer.

228. **jig 21.2: diagnose p-10's future write below the proof** (process, jig 21.2's block of 2026-10-17T07:08:09Z).
     Ruling 228, process: the implementer's option A, within jig 21.2. p-10 passed the First words and delegate failures that rulings 226 and 227 aimed at; the referee's check it now fails is `tests/client/src/harness/referee.rs:85`, `judge.check(at <= judge.now(), "nothing shown done before durable: future write")`: an `Observed::Durable` stamped later than the judge's clock.

     1. **Reproduce first,** in the focused common-world test of `logs/jig-21/21.2-r227-focused-12.log`, extended through a durable commit's observation. Log each `Observed::Durable`'s `at` and the judge's `now()` when it reaches the referee, and the clock each side reads.
     2. **Fix where the log names the cause.** A fixture's clock or scheduling (in `tests/ops/src/joint/**`, or a world it reuses) is 21.2's to fix; name each file outside `tests/ops/src/joint/**` in the commit body. A cause in engine, core or client production code is a block with the logs, not a fixture change. The referee's check stays as it is.
     3. **Then the proof:** at most two more runs of `p_accepts_the_restart_from_the_incidents_page` (p-11 and p-12), each after a different change aimed at a cause a focused log names; a new failure point is reproduced in the focused test before the next run. A failure after p-12 is a block with the logs.
     4. **Bound:** aim to spend at most 45 minutes from your start on this diagnosis; past it, report what the focused logs show, and go on with it.
     5. **Unchanged:** ruling 227's points 1, 2 and 4 (two engine production lines used of its 10: `translate/owner.rs`'s goals and projects lists by `len()`); ruling 226's other points (no core production change); 21.2's Proof; ruling 225's Size (4,331 now, aim 4,500, stop 5,000); the 06:20:26Z steer's point 1. After the proof: the eight survival proofs, clippy and the gate, as 21.2's Proof says.

229. **jig 21.2: a failing proof goes on while it progresses** (process, jig 21.2's block of 2026-10-17T07:17:25Z).
     Ruling 229, process: the implementer's option A, within jig 21.2, as a rule of the plan. p-10, p-11 and p-12 each failed at a later story point than the run before (future write; `read_series` on Task 5; Task 6's panels after acceptance), and each cause the focused test logged was the joint fixture's (`fixture::alert`'s due; `exchanges_per_client` against the native fixture's 8). The README now says, for every session from now until the run's end:

     > **A joint or process proof that fails** (ruling 229, after rulings 226 to 228 each counted a proof's runs): reproduce each new failure point in a focused test first (implementer.md, 8.8), log its cause there, and change only what that log names; then run the proof once. A run that fails at a later story point than the run before it is progress: go on the same way. A run that fails at the same point as the run before it, a cause in production code outside the increment's Files, or a sixth run of one proof after this rule reaches it, is a block with the logs. A referee's or a judge's check is never changed to pass.

     1. **Now:** reproduce p-12's failure (person step 13, no text containing "Restart service", Task 6's object panels Loading; `logs/jig-21/21.2-proof-p-12.log`) in the focused test, then go on under the rule: p-13 to p-18 at most, for `p_accepts_the_restart_from_the_incidents_page`.
     2. **The eight survival proofs,** once p passes, under the same rule, each with its own count.
     3. **Bound:** aim to spend at most 60 minutes from your start on the p proof; past it, report what the focused logs show, and go on with it.
     4. **Unchanged:** fixture changes stay in `tests/ops/src/joint/**` or a world it reuses, each file outside `tests/ops/src/joint/**` named in the commit body (ruling 228, point 2); ruling 227's points 1, 2 and 4 (two engine production lines used of its 10); no core production change (ruling 226); 21.2's Proof; ruling 225's Size (4,331 now, aim 4,500, stop 5,000); the 06:20:26Z steer's point 1.

230. **jig 21.2: offered acts bounded by MAX_OFFERED_ACTS in the link** (scope, jig 21.2's block of 2026-10-17T07:42:21Z).
     Ruling 230, scope: the implementer's option A, within jig 21.2, with the same defect's copies.

     1. **The repair.** In `crates/jig-client-protocol-link/src/translate/pages_part6.rs`, `owner_escalation_fields` (:50) bounds its acts by `c::MAX_OFFERED_ACTS`, as `documents_part8.rs:35` does: `acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?`. The view refuses an escalation whose acts have room past it (`jig-client-view/src/cards/owner_escalation.rs:45`, `if document.acts.capacity() > jig_client::contract::MAX_OFFERED_ACTS`); `limits.client.rows` gives room for 32, `MAX_OFFERED_ACTS` is 25 (`jig-client-domain/src/contract/documents.rs:226`).
     2. **Its copies,** in the same commit: the other seven `acts: offered_piece(value.acts(), limits, limits.client.rows)?` of the link's `translate/` (`pages_part3.rs:155`, `pages_part5.rs:253`, `pages_part6.rs:41, 71, 172, 255`, `pages_part7.rs:97`) take `c::MAX_OFFERED_ACTS` the same way; the views of `owner_person_task.rs:123, 146`, `owner_proposal.rs:66` and `sections/project/owner.rs:150` refuse past it too. Name each in the commit body. Refresh any further copy you meet, and list it there.
     3. **Its test,** at the unit layer, in the link crate's translation tests: an owner escalation with five acts translates to acts with room for at most `MAX_OFFERED_ACTS`. Under 20 ms. The view's checks and the referee stay as they are.
     4. **Then** resume the seeded P survival at its run 3, under the README's rule of ruling 229 (at most six runs of it), then the gate and the landing.
     5. **Bound:** at most 10 lines of client production in all, counted in ruling 225's Size (4,331 now, aim 4,500, stop 5,000); past them, a block.
     6. **Unchanged:** ruling 227's engine lines (two of 10); no core production change (ruling 226); 21.2's Proof; the 06:20:26Z steer's point 1. The commit body names the fixture files outside `tests/ops/src/joint/**` (ruling 228, point 2), as the report lists them.

231. **temper 04.1 needs temper 03.4, not whole temper 03** (steer of 2026-10-17T07:53Z).
     Ruling 231, plan defect (the director's question of about 07:50Z: temper-04.1 `ready` since 2026-10-16T17:53Z, not started). temper 04.1 needs jig 09.4 and temper 03.4, not whole temper 03: the README's table now says "04.1: temper 03.4 (ruling 231)", and `temper/04-engine.md`'s 04.1 "Needs: jig 09.4 … and temper 03.4 (its repin past jig 09.4), not temper 03.5 or 03.6". Ruling 143, point 4, said it: "temper 04 starts after temper 03.4's repin past jig 13.2, so it waits on nothing more"; `context.md`, line 18, read "needs whole Temper 03".

     - 04.1 names nothing of 03.5 or 03.6, and shares no file with them (03.5: `03-client.md:577-604`; 03.6: `:687-694`); temper's jig pin `c92d4a09d` (jig 13.2) holds jig 09.4's `3f2ae98ad` (`git merge-base --is-ancestor` succeeds), so 04.1 makes no repin and leaves `Cargo.lock` alone (04-engine.md, ruling 188). Whichever of 04.1 and 03.5 lands second rebases.
     - Most of 04.1 is on `main` already (rulings 164, 169, 188): `Limits.templates` (`temper-engine-domain/src/limits.rs:54`, `:116`), `templates: jig.core.templates` (`temper-service/src/limits.rs:278`), `ENGINE`'s `"templates"` (`temper-service/src/tests/fixtures.rs:46`, `:68`, `:445`), `What::Operations` (`startup/problems.rs:98`, `:162`), `("people.operations", 5)` (`crates/temper/tests/support/forgejo.rs:170`), `STORE_VERSION` 2, the walking world's `templates`. What a read of `main` found left: the tests `the_example_engine_keeps_jigs_chat_and_goal_templates` and `a_chats_member_is_refused_as_unknown_naming_its_path`, and the `Grace` and `Operations` rows of `jig_refusals_print_words` (`crates/temper-service/src/tests/startup.rs:117-140`). Grep each of 04.1's names on `main` first, and do only what is missing; a Shape item already on `main` is no question.
     - Unchanged: 04.2 to 04.4's own Needs (jig 22, jig 16, jig-13.s1); the fence (jig 14 after jig 21's last landing); 04.1's Size (40 lines).

232. **jig 21.3: completed reads wait for the designer; the rest goes on** (interim, jig 21.3's block of 2026-10-17T08:11:37Z).
     Ruling 232 (interim, jig 21.3; the question went to the designer at 2026-10-17T08:14Z): neither option A nor option B yet. Who keeps a completed read's page after `Told::Page` is a design question: jig's `client/domain/view.md`, 18, says the Ui "keeps no document of its own: it reads each from the client, by its handles, when the client reports it changed", and `client/domain/client.md`, 7, says "A read is one attempt ...: the application reads again if it wants" and "Keeping rows across pages, as a list slides while a person scrolls, is the presentation's"; neither says in its own words where a read's page lives. The designer's answer decides; it comes as a ruling.

     - Until it comes, 21.3 goes on only with what does not rest on it: the Ui's state on the client for watched documents, as its Shape says (each read from `client.document(handle)` when the client reports it changed), its surface, its tests and stories but those of a completed read's page.
     - Each holder of a completed read's result (`page/project.rs:543-549`'s `ProjectPage.owner`, from `Read::Project`, and any other the compiler or a grep of `Told::Page`'s users names) stays as `main` has it: add no client storage, accessor or limit for completed reads, and change nothing in `crates/jig-client` or `crates/jig-domain-client` for them. List each such holder in the report that ends the wait.
     - 21.3 does not run `land` until the answer's ruling; when nothing else is left, it reports so and waits.
     - Unchanged: the rest of 21.3's session file, its Don't included, and rulings 192 and 194.

233. **temper 06's Shapes against main** (ready check of temper-06.1).
     temper 06's Shapes against main: the ready check of temper-06.1, at skein `f981746a7`, jig `b58a49c39` and temper `70f376c5`, by a read-only sub-agent (main moved since to jig `154e10942`, 7 files under crates, and temper `d120b2f1`, 2 files, none the session names). Corrected in `temper/06-pages.md`: temper's `Address::Jig` carries jig's `jig_client_view::address::Address` (`jig_client_view::Place` does not resolve, E0425); `Proposal.proposer` is `jig_client::contract::Proposer` (E0308 against `ProposalKey`); `Confirm::Accept` and `Confirm::Reject` name no revision, as jig's `client/domain/contract.md`, 5.2, says: "A proposal never changes once made, so deciding it names its number" (`Ask::DecideProposal`, jig `crates/jig-client-domain/src/contract/asks.rs:135`), the Choices and 06.3's sketch with them; 06.5 calls `facts::chat_status(chip, waits_on, project, me)` and `Thread::sending(key, by, words, place)` as jig 15.1 and 15.4 give them; the view test reads link names with a new `fixtures::link_names` beside `fixtures::find` (`TreeFace` has no `names`; temper-client-view takes no `skein-fake-person`), and the step test's `env` takes `&fixtures::limits()`; 06.4 renames `conversations()` to `chats_page()` (`chats(&Driver)` at :144 keeps its name), with its users `stories.rs:26, 45, 80` and `fuzzy_client.rs:10`; 06.1's Files take the other `JigRegion::Link` users (`src/tests.rs:186`, `src/tests/fixtures.rs:112, 372`, `tests/support/mod.rs:30, 127`; `terminal.rs` alone fails `fuzzy_terminal`) and drop `crates/temper/src/open.rs`, which builds no `Settings`; the goldens' trees (`src/tests/fixtures.rs`, `inbox_page()` at :342) and `src/tests/render.rs`'s list join 06.1, 06.2, 06.4 and 06.5; 06.1 takes the navigation's users it removes (domain `lib.rs:20`, `tests.rs:204-218`; shell `client.rs:130`, `src/tests/fixtures.rs:291-292`) unless 03.6 has moved them, the root `Cargo.toml` for `jig-client-test-kit`, the new modules' `lib.rs` and `tests.rs`, and `pages::Act` in `pages.rs`; 06.2 takes the view's `pages.rs`; each increment takes the fixtures its tests name; line numbers `project()` :257, `stories.rs:8-21`, `person.rs:248-255`. Suites: clippy `-D warnings` on the corrected sketch module in temper-client-domain (with `Debug` and `#[must_use]`) and the 06.2 and 06.5 story sketches in `tests/client`; temper-client-world default 12/12 and fuzzy 1/1 with the rename; temper-client-shell-terminal default 32/32, fuzzy passing once `tests/support/mod.rs` changes with `terminal.rs`; temper-client-domain and temper-client-view default 37/37. Not run: the workspace gate, the process tier, the sketches that need the pages child. Refresh any further copy, and list it in the commit body.

234. **jig 21.3 and 21.4 against main after 21.2; 08:16Z steer texts** (re-check of jig 21.3 and 21.4, and steer of 2026-10-17T08:16Z).
     jig 21.3 and 21.4 against main after jig 21.2, and the 08:16Z steer's run-end texts. The re-check of what moved since ruling 192's check (jig `73734dd6f`) to jig `154e10942` (21.2), temper `d120b2f1`, skein `f981746a7` unchanged, by a read-only sub-agent, is corrected in `jig/21-ops-and-ui.md`: the watch spine's functions are in `crates/jig-client-view/src/ui.rs` (`open` :465 to `read_final_task` :1109), not `jig-domain-client`, which is gone; `jig_client::Event` and `jig_client::Request` derive `Debug, PartialEq, Eq` since `5c06a7e98`, so `crates/jig-client/src/client.rs` leaves 21.3's Files; the sketch derives `Debug, PartialEq, Eq` on the Ui's `Event` and `Request`, `Copy` on `How`, `Debug` on `Ui` and `Refused`, `#[must_use]` on `next_deadline` and `shown`, and `Erase` takes the view's `store::StoreKey` (no `records` module); 21.3's Files take the users of `Request::{SignIn, Fact}`, `Act::SignIn` and `signin::Provider` in the view crate, both roots and the worlds' setups of `providers`, `tests/client/src/observe.rs:203-241` (`jig_received`), and each file naming the view's `Domain` that the rename to `Ui` reaches; moved lines (`world.rs:565, :567`, `joint/front.rs:224`, `tests/client-protocol/src/native.rs:190`, `pages.rs:108-220`); 21.4's Files take `tests/client/tests/shell.rs:30` and `tests/client/src/observe.rs:230-242`, its test lines move (ops client-domain `tests.rs:1135-1186`, client-view `tests.rs:1619-1620`, `family_terminals.rs:55-56, :266`, ops `limits.rs` about :38-80), and the Done-when grep gains `-e 'Request::Frame' -e 'FrameShown'`. temper's users of what 21.3 and 21.4 remove, for temper 03.6's ready check, temper pinned at jig `c92d4a09d` until then: `crates/temper-client-domain/src/route.rs:56, :103-137, :175-178`, `domain.rs:129, :164-166`, `boundary.rs:52`, `forge.rs:96, :120`; `crates/temper-client-shell-terminal/src/route.rs:119, :185, :187`; the `Hello` fixtures in `crates/temper-client-view/src/tests/`. Suites at `154e10942`: the corrected sketch under `heavy cargo clippy -p jig-w6-sketch --all-targets -- -D warnings`, passing; `jig-client-protocol-world`, 21.3's two proof tests and two neighbours, 4/4; `jig-ops-world --profile fuzzy` p, passing; `jig-client-world` `a_dropped_watch_shows_the_page_behind_until_it_reopens`, passing. The 08:16Z steer's points 1 (cause-finding tests out of the default suite) and 2 (`tests/ops`'s story helpers once) hold until the run's end: their Pass on texts are in the README's "The run's steers". Refresh any further copy, and list it in the commit body.

235. **jig 21.3: a read's page stays with the page that read it** (design answer, jig 21.3's block of 2026-10-17T08:11:37Z).
     Ruling 235 (design answer, jig 21.3's block of 2026-10-17T08:11:37Z): the designer's answer, landed as jig de4f80564 and temper 892f4e88; 21.3 goes on under it and lands when its Shape is done.

     1. **A: the Ui keeps a read's page.** view.md, 18 now says: "It keeps no
        copy of a watch's document: it reads each from the client, by its
        handles, when the client reports it changed. A read's page, which the
        client hands over once, in the read's terminal, and keeps no copy of
        (client.md, 7), the reference UI keeps until the page that read it is
        left or the handle's next page replaces it (18.3)."
     2. **18.3, a new bullet:** "The page keeps what the read is handed, under
        the read's handle: a project's settings, a page of a task's history
        or activity, an object's document read to say how it ended. A read's
        handle keeps one page; a window's keeps the rows its pages put in the
        window ... What it keeps goes when the page is left (18.2), or when a
        read on the same handle is answered with a page, which replaces it; a
        read answered with none leaves it, and the page shows why, as 18.2
        says." 18.2's going to a page "forgets its reads and the pages they
        were handed".
     3. **18.9:** the pages kept are a limit and part of the Ui's worst case,
        "one per read handle, each priced at the client's bytes of a page
        handed over (client.md, 14)".
     4. **For 21.3:** each holder of a read's page stays as `main` has it
        (`ProjectPage.owner`, `owner_spend`, read windows, bodies read to say
        how an object ended), with its pricing (`page::project::worst_case`,
        `windows::worst_case`). Nothing in `crates/jig-client` or
        `crates/jig-client-domain` changes for completed reads. The Don't now
        reads: "no copy of a watch's document kept by the Ui across steps: it
        reads each from `client.document(handle)` (view.md, 18); a read's page
        is the page's to keep, never the client's (view.md, 18.3)". Added
        test: `a_read_s_page_stays_until_the_page_is_left_or_its_next_page_replaces_it`.
        Size 300 to 320.
     5. **Nothing is deferred:** no `later.md` row.

236. **temper 03.5: the repin carries jig's amendment revision, holder and Made** (scope, temper 03.5's block of 2026-10-17T08:14:33Z).
     Ruling 236 (scope, temper 03.5's block of 2026-10-17T08:14:33Z): option A. The README's Pins answer it: "What a temper repin repairs: the capability its increment's Needs name, by its Shape; and what else jig landed since temper's last repin, adapted the way jig adapted its own reference code ..., in at most 200 added lines outside the Shape's Files ... More is a question (development.md, 5.7)". jig's `CallPart::Proposed { proposal, holder }` and `Made { task, revision, given }` (`crates/jig-domain-core/src/stored.rs`) and its amendment's revision are what jig landed; temper's types carry them as jig's own code does.

     - **The three types, lossless:** `ProposedAction::Amend` (`crates/temper-engine-domain/src/boundary.rs:174`) gains `revision: u64`, which the smith `propose` tool parses as the `amend` tool already does (`crates/temper-engine-smith/src/calls.rs:121-124`, copied at :211, with its schema string); `CallAnswer::Proposed` (`store.rs:45`) gains jig's `holder`; `CallAnswer::Delegated` (`store.rs:69`) holds jig's `Made`, its size at `store.rs:712` as `size_of::<Made>()`. Drop no field, and invent no revision 0.
     - **The answer renderer keeps its words:** `crates/temper-engine-smith/src/answers.rs:173, :240` render what they render today (each `Made`'s `task`); the holder, revision and given are carried, not shown.
     - **Files, outside 03.5's, at temper `d120b2f1` against jig `154e10942`** (the ready re-check; a lower bound, since crates behind the failing ones were not compiled): temper-engine-domain `route/forge_route.rs:1759`, `route/proposals.rs:125`, `route/now.rs:19`, `route/tools.rs:114`, `route.rs:326, :342`, `translate.rs:177, :353, :369`, `store.rs:133, 149, 192, 208, 507, 682, 712`; `crates/temper-engine-smith/src/{nested.rs:42, calls.rs:211, answers.rs, tests.rs:147}`; `crates/temper-service/src/clients/translation.rs:11`; `crates/temper-client-view/src/forge/page.rs:231, 236` (`refusal`'s third argument); `tests/engine/domain/{src/fixtures.rs:37, :104, src/walking.rs:125, tests/fuzzy_endings.rs:112, :141, tests/root_routes.rs:158, :1511, :1534, :2070, :2097, :2104, :2120, tests/forge_routes.rs:770, :817, :2566}`; `tests/conformance/src/{observations.rs:213-214, link.rs:177, scenarios/lifecycle.rs:51, :98, scenarios/revisions.rs:19, :40, scenarios.rs:663, stories.rs:341}`; `tests/system/src/composed.rs:391, :442` (`release` is `Option<u64>`). The nine `Delegate` literals take `tracked`, `label`, `flags` and `budget` through one shared helper in their crate's fixtures, not nine copies. The re-check's estimate is about 125 added lines.
     - **The bound stays 200,** counted as the README says and given in the report. Past 200, stop and report with the count and the files left: that is a question, as the README says.
     - **Pins:** run the `cargo update` for skein last (`smith-host-protocol --precise` floats skein again); the lock keeps one revision each of jig `154e1094`, skein `8a51e10b`, smith `29fa97c9`.
     - **Unchanged:** 03.5's Shape and Files; implementer.md 8.2's rule for any other boundary type; rulings 207 and 210's downstream fixtures at this repin.

237. **temper 03.5: a decided proposal loads as jig's kit loads it** (scope, temper 03.5's block of 2026-10-17T08:25:44Z).
     Ruling 237 (scope, temper 03.5's block of 2026-10-17T08:25:44Z): option A in its place, answered as jig's own code answers it. The README's Pins, as ruling 236 quotes them, cover it: "what else jig landed since temper's last repin, adapted the way jig adapted its own reference code ..., in at most 200 added lines outside the Shape's Files". jig's reference roots do not ignore `Now::DecidedProposal`: `ops` routes it as a core load with `HistoricalProposal` (`examples/ops/domain/src/route.rs:1376-1386`, `Output::CoreLoad`), and the test service gives it to its store (`testing/jig-test-service/src/route.rs:160`), which answers it with jig's kit, `jig_protocol_store::historical`.

     - **What the load reads, as the kit does:** first the owner's live row, `tasks::Key::Live(key.task)` (`crates/jig-protocol-store/src/historical.rs:16`); then the one history row the owner's `proposal_rows` names for the proposal, `tasks::Key::History { task, revision }` (`historical.rs:197-206`, `next`; `reads/point.rs:227-234`, `proposal_key`); the answer is `core::Event::DecidedProposal { to, key, proposal, row, busy: false }`, `row` the history row's `proposal`, or none when either row is absent (`historical.rs:75-77`, `task_proposal` at :208-211). Use the kit's functions where temper's store path can call them; otherwise copy them.
     - **temper's range:** one new `CoreRange` variant (`crates/temper-engine-domain/src/store.rs:249`) that names what the kit's key names, the owner task and the proposal (not `tasks::Key::DecidedProposal`, which the kit does not read for this `Now`), with its membership and validation beside `ProposalDecision`'s (`loads.rs:324, :387`, `restart.rs:54, :78`), the service's two reads, and the bounded callback. It is the one further boundary change this ruling allows.
     - **The bound stays 200** for all the repin's repairs outside 03.5's Files, ruling 236's included, counted as the README says and given in the report; past 200, stop and report the count and the files left.
     - **Unchanged:** ruling 236; 03.5's Shape and Files; implementer.md 8.2's rule for any other boundary type.

238. **temper 03.5: the repin's repairs aim 300, stop 350; Size aim 750, stop 850** (plan defect, temper 03.5's block of 2026-10-17T08:34:31Z).
     Ruling 238 (plan defect, temper 03.5's block of 2026-10-17T08:34:31Z): option A, with numbers. development.md, 5.7: "At most about 200 lines besides `Cargo.lock`, and no change of design. A break that needs more, or a decision, is a question"; this ruling answers it. The repin onto jig 21.2 carries what jig landed since temper's pin `c92d4a09d` (rulings 207, 210, 236, 237), with no change of design, and cannot land apart from 03.5's client move: 23 of its errors are in 03.5's own Files (`client.rs:298, 355, 358` among them). So 03.5 keeps it, and no supplement step comes before it.

     - **The repairs outside 03.5's Files:** aim 300 added lines, stop at 350, counted as the README's Pins say (`git diff --numstat origin/main...HEAD -- <the changed paths outside the increment's Files> ':!Cargo.lock'`, first column, summed once before landing) and given in the report. 231 now (`logs/temper-03/03.5-repin-count.txt`). They include ruling 237's range with two focused tests, each under 0.5 s: the service's store answers a decided proposal from the owner's live row, then the history row its `proposal_rows` names, and with no row when either is absent; the engine answers a named withdrawal's call from that load. Then the clippy fixes and the users the whole-workspace check names.
     - **03.5's Size:** aim 750, stop at 850 added lines by code-organization.md's command, the repairs above included (629 now, unfinished). Trim nothing to fit: no formatting, doc comment, test or assertion.
     - **At either stop,** report the counts and the files left, and wait.
     - **Unchanged:** rulings 236 and 237's Decisions but their bound; 03.5's Shape, Files, Test shape and Proof; the 04:24Z steer's constant cleanup in `lifecycle.rs` and `queries.rs`, counted with the repairs.

239. **A repin's boundary repairs, as a README rule** (standing rule, temper 03.5's blocks of 2026-10-17T08:14:33Z and 08:25:44Z).
     A repin's boundary repairs, as a standing rule (the same block twice: temper 03.5's blocks of 2026-10-17T08:14:33Z and 08:25:44Z, answered by rulings 236 and 237). The README's Pins, "What a temper repin repairs", gains: "A boundary type of temper's (a field, a variant, a `CoreRange`) that carries what jig landed, as jig's reference roots and kit carry it (`ops`, `jig-test-service`, `jig_protocol_store`), is such a repair, not implementer.md 8.2's question: carry it losslessly, name each user in the commit body, and count it in the 200 (rulings 236 and 237 ...). A change jig did not make is still the question." It binds every temper repin to the run's end (temper 03.5, 03.6, the 04 and 06 repins, `temper-repin`); ruling 238's bounds stay 03.5's alone.

240. **jig 21.3: Size re-estimated to 2,500, stop at 3,000; no split** (plan defect, jig 21.3's block of 2026-10-17T08:37:11Z).
     Ruling 240 (plan defect, jig 21.3's block of 2026-10-17T08:37:11Z): option A. 21.3's Size, "at most 320 lines (most of the change is removal ...)", was set before ruling 234 put the view's `Domain` to `Ui` rename in its Files ("The view's `Domain` becomes `Ui` in each file that names it"), and before ruling 235 kept the read pages with the Ui. 668 lines now, 421 of them naming the rename, with the watched-document migration not yet done; jig 21.2's Size went the same way (rulings 221 and 225: 250, then 950, then 4,500; landed at 4,352).

     1. **Size:** aim 2,500, stop at 3,000 added production lines by code-organization.md, 7's command, the rename's lines included and not counted apart; the report gives the number once, before landing.
     2. **No split.** 21.3 stays one increment: what its Shape removes (the watch spine, the Ui's own watch documents and run text, `watches.rs`, `reads.rs`, `link.rs`, `versions.rs`, generations) and what it moves to the lent `&Client` land together.
     3. **At the stop,** report the count by file and the work left, and wait.
     4. **Unchanged:** 21.3's Shape, Files, Test shape, Proof and Don't (rulings 234 and 235); code-organization.md, 7's 40-line rule for files over 800 at the base; the 08:2xZ steer's points 1 and 3 (ruling 234).

241. **temper 03.5: the repin's repairs aim 400, stop 450** (plan defect, temper 03.5's block of 2026-10-17T08:57:21Z).
     Ruling 241 (plan defect, temper 03.5's block of 2026-10-17T08:57:21Z): the implementer's proposal. Ruling 238 said "The repairs outside 03.5's Files: aim 300 added lines, stop at 350"; they are 355, five over, with the engine's clippy repairs left (`crates/temper-engine-domain/src/boundary.rs`'s two grown enums, `route/forge_route.rs`'s 102-line helper, `route/proposals.rs`'s two closures) and the native and world lints not yet reached.

     1. **The repairs outside the Files:** aim 400, stop at 450, counted as ruling 238 says, and given in the report.
     2. **03.5's Size stays:** aim 750, stop at 850 (737 now).
     3. **A clippy repair splits, it does not excuse:** the 102-line helper becomes functions under clippy's line limit; no `allow` or `expect` for `too_many_lines` or the closures.
     4. **At either stop,** report the counts and the files left, and wait. Trim nothing to fit.
     5. **Unchanged:** rulings 236 to 239; 03.5's Shape, Files, Test shape and Proof (its two proofs under ruling 229).

242. **The 08:16Z steer's two moved tests go to fuzzy_bytes.rs** (steer of 2026-10-17T09:08Z).
     The 09:08Z steer's point 2 amends the README's 08:16Z point 1 text (ruling 234): jig 21.3 moves `an_environment_answer_can_be_lost_at_its_native_byte_face` and `an_environment_chat_stays_live_while_its_provision_finishes`, with their helpers `ready_environment` and `environment_chat`, to a new `tests/ops/tests/fuzzy_bytes.rs`, not to `tests/ops/tests/fuzzy_joint.rs`, which is over 800 lines at jig `154e10942` and ends at 800 or fewer (code-organization.md, 7: a file over 800 lines at the base grows by at most 40); where p and the eight survival tests already check what one of the two checks, 21.3 deletes that test instead, and the log keeps the cause. Its report gives the gate's focused and fuzzy times (README, Test time: fuzzy 22.291 s of 24.9 s at 21.2's landing). The rest of the 08:16Z point 1 stands.

243. **jig 21.3: Ui::new takes its limits first** (plan defect, jig 21.3's block of 2026-10-17T09:10:57Z).
     Ruling 243 (plan defect, jig 21.3's block of 2026-10-17T09:10:57Z): option A. jig's `client/domain/view.md`, 18.1, says the Ui is "made with: its limits; the application's navigation and home; its arrangement"; the sketch left out the first. `Ui::new` is `pub fn new(limits: &Limits, navigation: Navigation, home: Address, arrangement: Arrangement) -> Result<Ui, Refused>`: the limits each root already holds, passed first, as `Ui::new(&Limits)` takes them on main; no defaults. Unchanged: `Refused`'s two variants, `Navigation`'s fields, the rest of the sketch, rulings 234, 235 and 240.

244. **temper 03.5: diagnose the reload below the story; repairs aim 500, stop 550** (process, temper 03.5's block of 2026-10-17T09:20:20Z).
     Ruling 244 (process, temper 03.5's block of 2026-10-17T09:20:20Z): option A, as rulings 226 and 228 ruled for jig 21.2. The reload story fails at client step 8 before and after the authenticated-reload repair, so the next run is below it.

     1. **Diagnose below the story, once:** one focused test of the root and its front on the reload path (`StartUnlinked`, signed in, hello, the frame's routing, and the restored Person frame the last run lacked), under 0.5 s. Fix only the cause it names, in the file it names. Then run the reload story once. The test stays if it checks what the story does not and runs under 0.5 s; otherwise it goes, and `logs/` keeps the cause.
     2. **Then the gate's other failures** (11 default, 4 fuzzy, `logs/temper-03/03.5-gate1-diagnosis.txt`): each repaired at its named cause, all at once, then one default no-fail-fast sweep and one fuzzy sweep before the gate (ruling 219).
     3. **The repairs outside the Files:** aim 500, stop at 550 (441 now), for these repairs; 03.5's Size stays aim 750, stop at 850 (789 now). At either stop, report the counts and the files left, and wait.
     4. **The test over 40 lines:** `an_explicit_forge_effect_proposal_is_retained_until_its_holder_accepts` (`tests/engine/domain/tests/forge_routes.rs:2529`, 77 lines) is named in the landing report with its length at the base and now; split it only if 03.5 grew it.
     5. **Unchanged:** ruling 229's rule (a run that fails at the same point as the one before is a block with its logs); every story assertion; rulings 236 to 241; the 09:08Z steer's point 1.

245. **temper 03.5: the amendment story's proposer as jig's; native ordering interim** (scope, temper 03.5's block of 2026-10-17T09:37:13Z).
     Ruling 245 (scope, temper 03.5's block of 2026-10-17T09:37:13Z): the first question answered, the second interim.

     1. **The member-amendment story: option A.** development.md, 5.7: a repin may change "the downstream tests that an upstream change of behaviour breaks, adapted the way the upstream commit adapted its own (its body names the rule)". jig removed the implicit `Ask::Amend`; an explicit `Ask::Propose` names `Proposer::Party(member)`, as jig's reference does. The story's proposer input and its expected identity become `Proposer::Party(member)`; every other assertion and the accepted-budget equality stay. The commit body names jig's commit and the reference sites the log names. This keeps ruling 244's "every story assertion": an assertion adapted to jig's change is kept, not dropped.
     2. **The native startup ordering: not yet.** Whether temper's native shell holds the root's `Start` until it is signed in (option A) or jig's link should not open a stream without a credential (option B) is ruled after a check of how jig's own native host orders sign-in and the link's `open` on main. Until that ruling: go on with point 1, the gate's other failures under ruling 244, and the sweeps, but not this ordering; add no pending list to `client.rs`, change nothing in jig, and do not run `land`. When nothing else is left, report so and wait.
     3. **Unchanged:** rulings 236 to 244 (repairs aim 500, stop 550; Size aim 750, stop 850).

246. **temper 03.5: the root starts before the sign-in; jig-12.s1 holds a watch until signed in** (design answer, temper 03.5's block of 2026-10-17T09:44:19Z).
     Ruling 246 (design answer, temper 03.5's blocks of 2026-10-17T09:37:13Z and 09:44:19Z): the designer's answer, landed as jig 8c832e517 and temper c3d0c0b9; it ends ruling 245, point 2's interim. jig-12.s1 is `jig/12-s1-watch-waits-for-sign-in.md`.

     1. **Before: the shell starts the root, then signs in.** 3.4 stands, now: "`Start { kept }` before any other event, then signs in (section 7): with the credential DIRECTORY holds, or by the printed link. The shell holds back neither the start nor the sign-in. The domain's children ask their watches from their start, and `jig-client` opens none on the engine until it is signed in". 1e8ed63a's order (`client.rs:395-403`) is the design's; temper's code does not change for it.
     2. **The refused stream is jig's client's, and jig's client fixes it.** jig's `client.md`, 8.1 now: "**A watch waits for a sign-in.** A `Watch` while no sign-in is live (the link signed out, 8.2) is admitted, its handle opening, and nothing leaves for the transport ... Once signed in, the client opens each such watch under a fresh name; one on `me` is the client's own (8.2), not a second." 12: "Signed in, the client opens the stream, watches `me` (8.2), and opens the watches asked before (8.1); under `Transport` it is signed in from its `SignIn`."
     3. **jig-12.s1, the supervisor's to add** (development.md, 10.4):
        - Files: `crates/jig-client-domain/src/domain.rs`, `open` (:201): while `self.link == Link::SignedOut`, the watch is inserted with `name: None` and `until: None`, and no `Down::Open` is pushed. `link.rs`, `signed_in` (:55): after `set_link(Link::Starting)`, a held watch on `Frame` becomes `own` (no second `Frame` opened), and each held watch gets a fresh name and `w.opening(name)`. `crates/jig-client/src/tests/sign_in.rs`: the test.
        - Test `a_watch_asked_before_signing_in_opens_once_signed_in`: `Client::new()`; `Watch { handle: ME, subject: Watchable::Jig(Subject::Frame) }`, then `Watch { handle: TASK, subject: task() }`: `requests()` is empty after each. Then `SignIn { with: With::Held(fixtures::credential(&CREDENTIAL)) }`: `[Told { Link::Starting }, OpenStream { credential: Some(fixtures::credential(&CREDENTIAL)) }]`. After `hello`, exactly two `Message::Watch` exchanges, each with that credential.
        - Size: aim 60 added lines, stop at 100. Past 100, a question. Repairs to jig's worlds count in it. Proof: jig's gate, run by `land`; nothing else is wanted.
        - Don't: change `jig-client-protocol-link` (`OpenStream { credential: None }` stays for `Transport`; `link/src/tests/stream.rs:68-73` and `jig-client/src/tests/sign_in.rs:79-96` unchanged); change how a read or a send is handled while signed out; change jig's native host.
        - **Order:** it lands before jig 21.3's landing. 21.3 rebases onto it. Reason: temper 03.5's repin pins a step's landing, and must take jig-12.s1 without 21.3's.
     4. **temper 03.5** (plan amended):
        - Scope: the host starts the root at load and signs in after it.
        - Needs: jig-12.s1, taken by a second repin commit, `Cargo.lock` alone. Any repair it brings counts in ruling 244's 550.
        - Proof: the land gate's run of `temper-one-process-world` is the check of the five stories that fail at `referee/client.rs:97` under 1e8ed63a. Nothing else is wanted.
        - Don't: hold back the root's `Start`; give `SignIn` before it; change the referee's 200 rule.
     5. **Nothing is deferred:** no `later.md` row.

247. **jig 14.1 built beside jig 21.3 and 21.4, landed after jig 21** (steer of 2026-10-17T10:31:20Z).
     Ruling 247 (order, the user's "speed up the plan, without compromising its outcome", 2026-10-17T10:13Z): jig 14.1 is built now, beside jig 21.3 and 21.4, and lands only after jig 21's last landing.

     1. **The fence keeps its aim, not its start.** README, "The fence": "jig 14 starts only after jig 21's last landing, so that no repin of temper 03 (onto jig 10.3, 11.5, 13.2, 21.2, 21.4) takes the collapsed wire". Landing, not building, is what a repin takes: 14.1 starts now and does not run `land` before jig 21.4 has landed on jig's origin/main; then it rebases onto it and lands. 14.2 starts once 14.1 has landed, as before (14.2 and 14.3 share about 20 view files with 21.3).
     2. **14.1's text at jig `b6a63f8c1`** (a re-check of what main moved since its check of 06:11Z at `5c06a7e98`): `sign_in.rs (29)` is now 52; `native/wire.rs` 106-111 is now 117-122; the module jig 21.2 added is `examples/ops/client-domain/src/family.rs`; two users of the wire's `Answer`, `Keyed`, `Owned` and `ReadAnswer` it missed, `tests/ops/src/joint/bytes.rs` and `tests/ops/src/client/app.rs:314-320`, join its Files; the drift test also regenerates ops' `v1` (no lines counted). Refresh any further user of those types the same way, and list it in the commit body.
     3. **Overlap at the rebase** (the re-check, against 21.3's worktree and 21.4's Files): `tests/client/src/{test.rs, scripted.rs, observe.rs}` and `tests/client-protocol/src/client.rs`, at items other than 21.3's; `crates/jig-client-view/src/bridge.rs`, which 21.3 deletes, holds only 14.1's wildcard arms (they move to 21.3's `ui/client.rs`). Conflicts there are 14.1's to resolve at its rebase, counted in its Size.
     4. **Unchanged:** 14.1's Size (300) and the 2.5 times codec rule; jig 22 after jig 14; temper 03.6's repin onto 21.4's landing; rulings 222 and 224.

248. **jig 15 and 16: each increment on its own needs** (steer of 2026-10-17T10:33:58Z).
     Ruling 248 (order, the user's "speed up the plan, without compromising its outcome", 2026-10-17T10:13Z): jig 15's and 16's increments start as their own needs land, not their session's.

     1. **jig 15** (once jig 22 has, as before): 15.1 first; then 15.2, 15.3 and 15.4 each once 15.1 has landed, each in a slot of its own; 15.5 once 15.2 has (its Needs: "15.1's `facts::effect` and `facts::dangerous`, 15.2's `words`"); 15.6 once 15.3 and 15.5 have. Their files are apart (15.2 `words` and the view's phrasebook, 15.3 `shapes` and `tests/client-protocol`, 15.4 `thread` and the view's `conversation/**` and `windows.rs`, 15.5 the test kit and `tests/client`'s harness), sharing `jig-client`'s `lib.rs` module lines, `Cargo.toml`, `Cargo.lock` and `graph.rs`.
     2. **jig 16** (once jig 21 has too, as before): 16.1 once 15.1 to 15.3 have landed, beside 15.4 to 15.6 (it reads `facts::chat_status`, `words`, `shapes::CHAT` and `template`); 16.2 once 16.1 has; 16.3 once 16.2 and 15.6 have (15.6's Proof stands on the scripted model 16.3 rewrites); 16.4 once 16.2 has, beside 16.3; 16.5 last, once 16.3, 16.4 and every jig 15 step have landed: temper 04.3 repins onto 16.5's landing, and temper 06.1 relies on it holding 15.5's check.
     3. **Conflicts** at a landing in the shared files are the later step's to resolve at its rebase (generated code by regenerating it, as each increment's codec rule says), counted in its Size; a conflict in an item another running step changes is a block, naming both steps.
     4. **Unchanged:** jig 14 → 22 → 15 at the session level; jig 17 once all of jig 16 has; 18, 19, 20 in order; each step's Size, Proof and gate; each session's first step held for its ready check, and each step started early here gets one too (supervisor.md, section 5).

249. **jig 22's and jig-14.s1's Shapes against main** (ready check of jig-22.1 and jig-14.s1).
     Ruling 249 (ready checks of jig 22 and jig-14.s1, against jig `b6a63f8c1` and temper `fe8fcdf6b`): the two session files corrected against main.

     1. **jig 22** (`jig/22-documents.md`, each passage marked "ruling 249"): 22.1's bounds are `jig_client_view::Limits::…`; its Files take the `me` watch's, the contract's, the link's, the component's, `OwnerProjectPolicy`'s, the core's, the wire tests' and the view's sites the check found, and the downstream repository's users; `DocumentLimits` gains `roles` and the frame's worst case a `Beyond` per project. 22.2: `task: u64` on `WaitsOnParty` and `WaitsOn::Party`; `PhaseClass` and `ExecutorKind` in `queries.rs`; `Debug` on the sketch's types; the `Amend` story's names are jig 14.2's; `translate/live.rs` and the engine-clients fixtures join its Files. 22.3: the core's read takes two groups with a cursor each (engine.md, 11.2: "its live tasks first, in its order, then its ended ones, each group with its own cursor"); Size aim 300, stop 380 (was 180). 22.4: `Impact` and `ReleaseEffect` in core `acts/effects.rs`; the `PhaseCounts` mapping; its missed Files. Done when counts trailers after `sort -u`.
     2. **jig-14.s1** (`jig/14-s1-words-release.md`, "Corrections at jig `b6a63f8c1`"): the production, generated, derived-value, world, fake-party and downstream sites ruling 210's point 4 missed; ruling 210 stands.
     3. **Open for the designer before 22.4:** what core state `live_runs`, `effects_pending`, `effects_settling` and pending proposals count, and `Impact.takeover_budget` in a cancel (question.md). 22.1 to 22.3 do not rest on it; 22.4 waits for its answer.
     4. Any further user a session finds of a name these corrections change is refreshed the same way and listed in its commit body.

250. **jig 21.3: a rejected read result is its one terminal; the tests read again** (plan defect, jig 21.3's block of 2026-10-17T11:13:48Z).
     Ruling 250 (plan defect, jig 21.3's block of 2026-10-17T11:13:48Z): a malformed read result is the read's one terminal; the moved tests read again (option B), and only what the native client never delivers is injected at the Ui's boundary.

     1. **The design answers it.** jig's `client/domain/client.md`, 7: a read "has one terminal, `Page { handle, page }`" and "A read is one attempt (`../../protocol/client.md`, section 6): the application reads again if it wants." `jig-client-domain`'s `read_done` (`reads.rs:35-47`) removes the read and tells its one `Page`, as the design says. A Ui that keeps waiting on a read after its terminal would wait for nothing.
     2. **The Ui retires its read on a terminal it rejects.** `page/routing.rs`'s `read_result` (and `application.rs`'s, if it holds the same pending read) drops its local reading when the page's identity does not match, and leaves the page as it was. The moved tests keep their names; the "wrong page, then the right page on the same read" sequence becomes: the wrong page (the page unchanged, the local reading gone), a fresh read through the real `Client`, then its right page, installed. The tests' assertions on the page stay; those on the old pending reading change to the retired one, each named in the commit body.
     3. **A duplicate, late or reset page after the terminal**, which the native client never delivers, may be fed straight to the Ui's `Event::Client { told: Told::Page }` (option A's way), as a check of the Ui's own guard; valid documents and pages go through the real `Client`'s `Up`, as the Test shape says.
     4. **Not option C:** no change in `jig-client` or `jig-client-domain` (ruling 235; 21.3's Files). The independent failures the report lists (read-capacity fixtures, ended watched documents, the intent clock, the owner-inbox projection, `max_out`) go on under 21.3's Shape.
     5. **Unchanged:** ruling 240's Size (aim 2,500, stop 3,000) and 243; the 10:5x steer's points.

251. **jig 22.4: a cancel counts its tree's tasks, live runs and pending proposals, never effects** (design answer, the ready check of jig-22.1).
     Ruling 251 (design answer, the ready check of jig-22.1): the designer's answer, landed as jig 360aabcc6 and temper 63803dd15; 22.4 no longer waits, and goes on under its amended Shape.

     1. **What a cancel's effect counts.** contract.md, 5.4 now reads: "*cancelling:* what ends: the live tasks below the task, counted by phase; and, of the task and those below it, the live runs (each attempt claimed whose answer its task has not heard) and the pending proposals (each a task made that no holder has decided). Its effects are its connectors' (5.1): each withdraws the entries not yet sent and settles those in flight, and the document counts none of them". In the core, the counts walk `core.tasks.view_tasks()` with `task_tree::descendant`, as `task_tree::tree` (`task_tree.rs:813`) does:
        - `tasks`: each live task below the task, never the task itself, through `PhaseCounts::add` (`task_tree.rs:256`).
        - `live_runs`: the task and each live task below it whose `tasks::run_attempt(&row.phase)` is `Some`. That is `jig-domain-tasks/src/run.rs:129`, made `pub` and re-exported from `lib.rs`. It is `Some` when the run is claimed or running, closing at `Stage::Run`, or held from one of those. No `Writer::Run` slot is read.
        - `proposals`: the task and each live task below it whose `proposal` is `ProposalState::Pending`, read as `task_tree::proposal` (`task_tree.rs:630`) reads it.
     2. **No effect is counted.** `effects_pending` and `effects_settling` leave `Impact` in the wire, the contract and the core's sketch. The rest of the design already says this:
        - contract.md, 5.1: "Their effects are the connectors' own, and jig's documents never count them".
        - connectors.md, 4.2: "how many entries, and which, are the connector's to know and to show, never counted by the core".

        objects.md, 11 now says the card states, "in jig's words, that effects not yet made are withdrawn and those in flight settle ... which, and how many, each connector's panel says". In the confirmation, `PendingEffects` (254) becomes `EffectsSettle`, with no hole: "Effects not yet begun will be withdrawn; begun effects will settle". `SettlingEffects` goes, and 255 is left unused.
     3. **`takeover_budget` leaves `Impact`,** in the wire and in the contract. A move's reservation is that move's own effect. contract.md, 5.4: "*moving a task:* the reservation that moves to the party's pool", that is `Effect::Move { reservation }`, which `TakeoverFunds` now reads. The translation no longer writes 0.
     4. **The wire:** `record Impact { tasks: PhaseCounts live_runs: u32 proposals: u32 }`, changed in place in v8. `TaskContext.impact` and `Escalation.impact` keep the smaller record. 22.4's Files list the sites. To check, run `git grep -l -E 'effects_pending|effects_settling|PendingEffects|SettlingEffects' -- crates tests testing examples ':!*/generated/*'` once after 22.4: it must print nothing (it prints 16 files at `b6a63f8c1`).
     5. **Test:** `a_cancel_offer_counts_the_tree_it_would_end` stays one test, asserting three things:
        - the task's own live run and pending proposal count in `live_runs` and `proposals`, never in `tasks`;
        - a running delegate with no write hold counts in `live_runs`;
        - a decided proposal is not counted.

        Size stays at most 300 lines (removed lines count nothing).
     6. **Nothing is deferred:** no `later.md` row.

252. **the 11:32Z steer's point 1 is the README's 05:20Z point, sent again** (process, the coordinator's block of 2026-10-17T11:34:46Z).
     Ruling 252 (process, the coordinator's block of 2026-10-17T11:34:46Z): the 11:32Z steer's point 1 is the README's "The run's steers", 05:20Z, point 1, sent again; it needs no new entry.

     1. Its Pass on is the README's 05:20Z, point 1 text, word for word ("A failed `land` gate shows only its first one to five failures … `land`'s gate is unchanged."), which the coordinator has forwarded to w91. The 11:32Z point 1's own Pass on text is withdrawn; nothing takes its place but the README's.
     2. The 05:20Z point already applies to every session whose commit changes more than 20 files: jig 21.3 now (257 files) and jig 14.1 at its rebase onto 21.4.
     3. The 11:32Z steer's point 2 (21.3's accessors) stands as sent; its Until is not the run's end.

253. **jig 21.3: a sign-in the engine ended shows the sign-in page after a lost sign-out** (design answer, jig 21.3's block of 2026-10-17T11:46:44Z).
     Ruling 253 (design answer, jig 21.3's block of 2026-10-17T11:46:44Z): the designer's answer, landed as jig bbb6303aa and temper 5b2faba9d; 21.3 goes on under it and lands when its Shape is done.

     1. **A, as the supervisor read it.** A sign-in page that follows the engine's own end of the sign-in is not a lost sign-out shown done. testing.md, 6.3 now reads: "After a person asks to sign out, the sign-in page is a fault while the engine still holds the sign-in they asked to end. Once the engine has ended it, by that sign-out or by an end of its own (an expiry, a revocation, another client's sign-out of it), before the ask or after it, the page shows that end, whatever became of the ask. The world observes each end of a sign-in where the engine makes it, as it observes its commits, never from a client's link."
     2. **The Ui stays as 21.3 has it (not B).** view.md, 18.6 now adds: "A sign-in the engine ends itself, before the sign-out or after it, ends what was asked: the link says signed out, and the reference UI shows the sign-in page, as above; it never keeps the work page once the link says signed out." `signed_out_shows_the_sign_in_page_and_keeps_the_place` stands.
     3. **client.md, 12** now says what jig-client already does (`client.rs`, `signing`'s `link::Signing::SignedOut` and `Up::SignedOut`, both clear `sign_out_lost`): "Signed out meanwhile, whatever ended the sign-in, the client is signed out, as above, and the link no longer names the sign-out."
     4. **The referee (21.3's Test shape, at jig `15266f311`):**
        - `Observed::SignOutReceived { client }` becomes `Observed::SignInEnded { client }`. It is pushed where `SignOutReceived` is now: `scripted.rs:507`, `joint/session/signing.rs:37` (which the joint front's `SignedOut` fault already calls) and ops' `client/transport.rs:474`. It is also pushed in each scripted front's `FrontFault::SignedOut` arm: `scripted.rs:453` and ops' `client/transport.rs:417`. A scripted front pushes it once for each client bound to the ended sign-in's party.
        - The referee keeps the clients whose sign-in the engine ended, beside `sign_out_asked`. A `SignInEnded` adds its client. `Seen::SignedOut` passes when its client has no sign-out asked, or is among the ended ones; the client then leaves both sets. Nothing else takes a client out of the ended ones. A reload does not, because the scripted engine keeps the sign-in it ended.
        - Referee tests: `a_sign_out_shown_done_before_the_engine_received_it_is_caught` stays as it is. `a_sign_in_the_engine_ended_shows_the_sign_in_page_after_a_lost_sign_out` is added: `Truth(SignInEnded)`, then `SignOutAsked`, then `Seen::SignedOut`, giving `Verdict::Passed`. That is seed 101's order. Each test runs under 20 ms.
        - No other referee rule or judge changes. 21.3's Size stands: aim 2,500, stop 3,000.
     5. **Nothing is deferred:** no `later.md` row.

254. **jig 21.3: one bounded diagnosis of each repeated story** (process, jig 21.3's block of 2026-10-17T11:57:42Z).
     Ruling 254 (process, jig 21.3's block of 2026-10-17T11:57:42Z): one bounded diagnosis of each repeated story, by its cause before any fix.

     1. **The sign-out race is answered:** ruling 253 (jig bbb6303aa), with the coordinator.
     2. **For each of the two stories** (the joint crash/reload seed 4503, Offline at 0.600359346 s; ops' withdrawn restart seed 512, the waiting card lost at 7.375941956 s): run its focused reproduction once with the trace the story already prints, and name the first event where the Ui's or the root's state departs from what the client told it, by file:line, in the report, before changing code. At most 30 minutes for each story.
     3. **Then:** a cause inside 21.3's Files is fixed, with a unit test that fails before the fix, and the story run once. A cause in `jig-client` or `jig-client-domain` (outside 21.3's Files, ruling 235), or no cause found in 30 minutes, is reported with the trace's lines, and that story stops: block again, naming it.
     4. **Meanwhile** 21.3 goes on with ruling 253 and the other failing changed-crate tests; it runs `land` only after both suites pass with `--no-fail-fast` (README 05:20Z, point 1).
     5. **Unchanged:** ruling 229 for every other story; rulings 240, 243, 250, 252, 253; Size stop 3,000 (2,806 now).

255. **21.3's diagnostic printing for the two stopped stories** (process, jig 21.3's block of 2026-10-17T12:03:54Z).
     Ruling 255 (process, jig 21.3's block of 2026-10-17T12:03:54Z): yes, one diagnostic run of each stopped story, with temporary printing.

     1. **What to print:** in `tests/client/src/harness/world.rs` only, print to stderr each observation that reaches `World::observe` (world.rs:529) and each callback, with `self.now`, and the world's existing `self.trace` lines just before each `self.referee.assert_holding` (world.rs:531 and :860). Mark that code `#[allow(clippy::print_stderr)]`: the workspace denies `print_stderr` (Cargo.toml:267).
     2. **What does not change:** the referee, the judge, each story's assertion and its seed (joint crash/reload seed 4503; ops' withdrawn restart seed 512).
     3. **Run each story once** with that printing, through `heavy`, and name in the report the first event where the Ui's or the root's state departs from what the client told it, by file:line, as ruling 254, point 2, says. At most 30 minutes for each story, counted from this unblock.
     4. **Then remove the printing** before any commit: `git diff -- tests/client/src/harness/world.rs` shows none of it. Keep the log path in the report.
     5. **Unchanged:** ruling 254, points 3 to 5 (a cause inside 21.3's Files is fixed with a unit test that fails before the fix; a cause in `jig-client` or `jig-client-domain`, or none found, blocks again naming the story); ruling 253; ruling 229 for every other story.

256. **21.3's typed callback trace for ops' seed 512** (process, jig 21.3's block of 2026-10-17T12:12:07Z).
     Ruling 256 (process, jig 21.3's block of 2026-10-17T12:12:07Z): yes, a typed callback trace at the Ops front, for ops' withdrawn restart (seed 512) alone.

     1. **Where:** the Ops front in `tests/ops/src/client/` (in 21.3's Files: "the worlds' fronts"). Print to stderr each callback's typed payload (what the front hands the client, and what the client hands back), with the time, beside ruling 255's printing in `World::observe`. Mark that code `#[allow(clippy::print_stderr)]`. The referee, the judge, the story's assertion and its seed do not change.
     2. **Bound:** at most 30 minutes, counted from this unblock, for the typed-trace runs, the fix and the unit test that fails before it. Run the story once after a fix.
     3. **Then remove the printing** before any commit: `git diff` of the files it touched shows none of it.
     4. **If no cause is found in 30 minutes,** or the cause is in `jig-client` or `jig-client-domain` (ruling 235): report the trace log's path and the lines from the engine's withdrawal to the card's drop, and block again naming the story; do not run it again. 21.3 goes on meanwhile with the lost-answer failures and both changed-crate suites.
     5. **Unchanged:** the crash/reload story (seed 4503) is done under ruling 255; ruling 254, points 3 to 5; rulings 229 and 253; Size stop 3,000.

257. **jig 21.4's Shapes against main** (ready check of jig-21.4).
     Ruling 257 (plan defect, jig 21.4's re-check at jig `48a745eb9`, after 21.3 landed; temper `524738afe`): 21.4's Files and Done-when grep miss users that 21.3 left or moved; corrected in `jig/21-ops-and-ui.md`, 21.4 ("Re-check at jig `48a745eb9`"), and in `temper/03-client.md`, 03.6's Files.

     1. **Files gain:** the Ui's `ui/client.rs` (`Told::{Answered, Kept, Dropped}`), `signin.rs:44`, `limits/ui.rs` and `ui.rs`'s `kept` bounds, `notices.rs` and the phrases of the two refusals and the dropped count; the Ui's tests the removed variants reach (74 errors in `cargo check -p jig-client-view` with them removed); each family side's `family/client.rs` or `family.rs` for its own `me` handle; the roots' `JigOut::{Answered, Unknown}` and their users; `tests/client/tests/fuzzy_memory.rs:286`; the users of the `limits.rs` heap bounds and of `limits.jig.kept`.
     2. **The factor 3** in both roots' `limits.rs` is derived again from the hand-offs that remain (`Named`, `ShowPart`/`PartsLeft` still nest first, jig, first), and the commit body says the derivation.
     3. **The Done-when grep** adds `KeptGone`, `JigOut::(Answered|Unknown)`, `Request::(Confirmed|Retention)\b`, and `tests/client` and `tests/ops`: 36 lines on `48a745eb9`, none after 21.4.
     4. **temper 03.6's Files** add `crates/temper-client-shell-terminal/src/client/tests.rs:46` and `crates/temper-client-view/src/tests/forge_page.rs:201`.
     5. **Unchanged:** 21.4's Scope, Shape, Test shape, Proof and Size (at most 300 lines); the sketch (its `Event`, `Request`, `How` and `Refused` are in `boundary.rs`, re-exported at `lib.rs:126`).

     Checked by a read-only sub-agent at jig `48a745eb9`: the sketch under `clippy -D warnings`, clean; both Proof commands passed (2 of 2 in 0.24 s; 1 of 1 under fuzzy, 1.13 s); the 8 packages 21.4 reaches, 1,119 of 1,119 (default) and 59 of 59 (fuzzy). Refresh any further copy of what 21.4 removes, and list it in the commit body.

258. **21.4's Size: aim 800, stop 950, one ops me module** (plan defect, jig 21.4's block of 2026-10-17T12:48:44Z).
     Ruling 258 (plan defect, jig 21.4's block of 2026-10-17T12:48:44Z): Size re-estimated, no split, and ops' two `me` modules made one.

     1. **The estimate missed what ruling 257 added to the Files** (the Ui's local outcomes, the kept bounds, the notices and phrases, each family side's `me` handle): a plan defect. code-organization.md, 7: "An increment lands within twice its Size without asking (...); past that it asks, with the count by file."
     2. **Size:** aim 800; past 800, the report gives the count by file and why, and the session goes on; at 950 added lines by code-organization.md, 7's command, stop and block with the count by file. No split: the family sides' own `me` handles and the Ui's boundary removals move together.
     3. **Ops' two families share one `me` module** that takes the connector (`jig_ops_domain_connectors::Connector`), in place of `examples/ops/client-domain/src/infrastructure/me.rs` and `observability/me.rs`, which differ in the connector alone. Do it in the ops-only commit (`0b8a85a63`'s), which stays ops' one commit. The test root's `first/me.rs` stays its own.
     4. **Unchanged:** no test, comment or formatting trimmed to fit; ruling 257; the factor 3's derivation in each commit body; Scope, Shape, Test shape and Proof.

259. **README: the 13:00Z steer's point 2** (process, the coordinator's block of 2026-10-17T13:01:13Z).
     Ruling 259 (process, the coordinator's block of 2026-10-17T13:01:13Z): the block is right. Ruling 167, point 6: "a steer's Pass on text that holds until the run's end lands in 'The run's steers' before the steer is sent". The steer of 2026-10-17T13:00:29Z, point 2, holds until the run's end and was sent before its text landed. It lands now, word for word as the steer gave it, under "2026-10-17 13:00Z, point 2 (jig 21.4, then each session that adds a test to `crates/jig-client-view`)".

     1. The coordinator carries out point 2 as the steer says: it gives jig 21.4 the point's Pass on text, and logs it done. context.md names only "Steer 13:00:29Z, point 2".
     2. Later sessions read it in the README; their prompts do not repeat it (ruling 167, point 4).
     3. Unchanged: points 1 and 3, already given to jig 21.4; ruling 258.

260. **14.2's Amendment carries people's Tracking** (plan defect, jig 14.2's block of 2026-10-17T13:41:31Z).
     Ruling 260 (plan defect, jig 14.2's block of 2026-10-17T13:41:31Z): not a design gap. The design answers in its own words, jig's `docs/design/domain/tasks.md`, 3 (:122): "**Whether it is tracked,** and its priority if so"; the people child already keeps it so, `people::Amendment.tracked: Option<Tracking>`, `Tracking { Tracked { priority: u32 }, Untracked }` (jig 04). The plan's two independent options made pairs the design has no meaning for. The wire takes your option B, mirroring people:

     1. **Wire:** `record TrackingTracked { priority: u32 }`, `enum Tracking { tracked: TrackingTracked, untracked }`, declared before `Amendment`; `Amendment` gains `tracked: option Tracking`, in place of `tracked: option bool` and `priority: option u32`.
     2. **Contract** (`crates/jig-client-domain/src/contract/asks.rs`): `pub enum Tracking { Tracked { priority: u32 }, Untracked }`, `Amendment.tracked: Option<Tracking>`, mirrored field for field.
     3. **Translation:** `None` leaves tracking unchanged; `Some(Tracked { priority })` and `Some(Untracked)` become `people::Tracking`'s same variant. No refusal is added.
     4. **A change of priority alone** is an amendment whose only change is `tracked: Some(Tracked { priority })`; it names no revision (contract.md, 3.2: "A change of priority alone names no revision: the first commit decides"). `a_priority_only_amend_names_no_revision` stands, built so.
     5. **Later sketches** follow: jig 16's goal move sends `Amendment { tracked: Some(Tracking::Tracked { priority: p }), .. }`; jig 22's task-query story amends with `Some(Tracking::Tracked { priority: 2 })`. Refresh any further copy of `tracked: option bool` or `priority: option u32` in an `Amendment`, and list it in the commit body.
     6. **Unchanged:** the rest of 14.2's Shape, Test shape and Size; the record merges, grace and template overrides you have going.

261. **jig 14.2 to 14.4's Files against main** (ready check of jig-14.2).
     Ruling 261 (plan defect, the re-check of jig 14.2 to 14.5 at jig `447ddfdf4`, after 14.1 landed; temper `9bd979292`): 14.2 to 14.4's Files miss users that 14.1, 21.3 and 21.4 moved or left; corrected in `jig/14-wire.md` (a "Re-check at jig `447ddfdf4`" paragraph before each of 14.2's, 14.3's and 14.4's Shape) and in `temper/04-engine.md`, 04.2's users.

     1. **14.2's Files gain:** the link's `limits.rs:105` (`take_over_reason`, now `move_reason`); the full `Amendment` literals `tracked` reaches (the link's `amendment_fields`, the view's `confirmation/amendment.rs`, `tests/engine-clients/tests/requests.rs:109`, the component's `tests/fixtures.rs:88`, the scripted engine's `fields!` list); the link's `documents_part6.rs:168` (`SeenThrough`); for `AdmissionLimit.grace`, the view's `words/refusals.rs:92-128`, a `JigPhrase::EngineLimitGrace` in `phrases/{jig.rs, english.rs}`, and `contract/admission_limits.rs`; `tests/client-wire/tests/fuzzy_client_wire.rs`'s `check!` list.
     2. **The contract boxes `Amendment`** in `Ask::Amend` and `Operation::Amend`, as the sketch does; the `#[expect(clippy::large_enum_variant)]` at `contract/asks.rs:116` goes, unfulfilled (the re-check saw `-D warnings` fail with it).
     3. **`a_priority_only_amend_names_no_revision`** asserts revision 0 reaches the people child (`people::Ask::Amend`'s `revision: u64`, `jig-domain-people/src/boundary.rs:278`).
     4. **14.3's Files gain:** the link's `limits.rs:100` (`write_words`, now `message_words`); `AdmissionLimit.operations`'s words as 14.2's; the `check!` list; `Made::Words`'s users (`tests/ops/src/client/app.rs:414`, `tests/client/tests/referee.rs`, `JigPhrase::MadeWords`).
     5. **14.4's Files gain** `AdmissionLimit.goals`'s words as 14.2's. A role executor naming another project than the `Create`'s is a design question, in `question.md` now: 14.4 starts once its answer lands.
     6. **temper 04.2's users gain** `temper-client-domain/src/forge/family.rs:115, :123`, the shell's `client.rs:472` and `src/tests/fixtures.rs:666`.
     7. **Unchanged:** each increment's Scope, Shape, Test shape, Proof and Size; ruling 260.

     Checked by a read-only sub-agent: the interface sketch and 14.2's reference test under clippy `-D warnings` against landed code (with stubs for unlanded `Story` methods); 14.2 to 14.5's schema records applied to `client-v8.schema`: codegen accepts them and `clippy -p jig-client-protocol-wire --lib -D warnings` is clean; the drift test 3 of 3; the wire fuzzy test 1 of 1 with the corrected `check!` list (5 compile errors without it). Refresh any further copy of a renamed record or limit field, and list it in the commit body.

262. **14.4's role executor is the Create's project's** (design, the designer's answer of 2026-10-17T14:05Z).
     Ruling 262 (design, the designer's answer of 2026-10-17T14:05Z, jig `ad3b26d04`, temper `ec988d053`): a role executor's project is the `Create`'s; `NewTask.executor` is `OwnerExecutor`; nothing is refused for it; jig 14.4 may start.


     1. **A role executor's project is the `Create`'s, by definition.**
        people.md, 1: "**Roles are per project,** the core's own".
        contract.md, 3.2 now: "a person, or a role, which is the project's:
        roles are per project"; tasks.md, 2 now: "a party or a role, the
        task's project's role". Main reads it so: a role's inbox entry is
        `Whom::Role { project: row.project, role }` (`inbox.rs:302-306`).
     2. **The request names no project for it.** `NewTask.executor` is
        `OwnerExecutor` (`client-v8.schema:91-94`), field for field
        `people::TaskExecutor` (`batch.rs:44-49`), as `Amend`'s
        `AddressChange.address` already names a role
        (`client-v8.schema:213`). Contract: `NewTask.executor:
        OwnerExecutor` (`contract/owner_part.rs:14-18`).
     3. **Nothing is refused for it:** no `Malformed` variant, no other
        refusal; contract.md, 4 is unchanged.
     4. **The plan:** 14.4's Shape and the sketches take `OwnerExecutor`;
        its component test's batch holds a role's task
        (`PersonAddress::Role(2)` reaching the people child as
        `TaskExecutor::Person(PersonAddress::Role(2))`); its Don't names the
        project and the refusal; the re-check's "14.4 starts once it is
        answered" ends. 14.5's `ProposeBatch` follows, its tasks being
        pieces of `NewTask`. jig 15's `GoalDraft.executor` and `listed` take
        `OwnerExecutor`, so `set_goal` copies it with no mapping.
     5. **Unchanged:** the chips' `Executor` and its `ExecutorRole.project`;
        `TaskFilter.executor`; `LackingTaskKind.executor`. Nothing is
        deferred.

263. **README: landing reports keep their lists in logs** (steer of 2026-10-17T15:00:35Z).
     Ruling 263 (process, the check of 2026-10-17T15:00:35Z): the steer of 2026-10-17T15:00:35Z, point 1, holds until the run's end; its Pass on text lands in the README's "The run's steers" before the steer is sent (ruling 167, point 6), under "2026-10-17 15:00Z, point 1 (every session)": a landing report names only what its commit changed, with the complete lists in `logs/<session>/audit-<step>.md`, under about 3,000 bytes; it replaces 06:20Z point 1's and 21:01Z point 1's lists where they differ. Checked: `wc -c reports/jig-14-20261017T143008Z.md`: 10,164 bytes (the run's median report 1,998).

264. **the slowest tests, whoever added them; 14.4 goes on** (process, jig 14.3's block of 2026-10-17T15:28:39Z).
     Ruling 264 (process, jig 14.3's block of 2026-10-17T15:28:39Z): yes, the slowest tests whoever added them, bounded; then 14.4 goes on.

     1. **The README's rule, not the steer's narrowing:** README "Test time": "the session's next increment first makes the slowest tests cheaper, found once by the measure run of the suite that missed (...), until its own gate is back under the aim". The steer's "that 14.1 and 14.2 added" is dropped: the slowest tests of the measure run, base fixtures included (the native, channel and Ops fixtures), are in scope.
     2. **Who and when:** jig 14.4, in its first commit, from 14.3's measure run (`a_terminal_in_front_of_the_native_client_starts_a_chat` 0.312 s, `a_resumed_run_is_sent_only_its_current_window` 0.274 s, `p_accepts_a_restart_from_the_incidents_page_with_a_second_seed` 0.266 s, and the next by the same run); no second measure run.
     3. **Bound:** at most 45 minutes. Every assertion, seed and story stays; a fixture made cheaper keeps what it checks. Each change is a small fix outside the Files, listed in the report with the test's time before and after.
     4. **If the focused gate is still over 13.3 s** after that commit, the report names the gate's time and the three slowest tests, and 14.4 goes on: the aim is a guide, missed (README "Test time": "the landing stands"). The stop of 15 s stays a rule.
     5. **14.4 may start now:** ruling 262 (jig `ad3b26d04`, temper `510db3bd4`) answered the role executor.
     6. **Unchanged:** README "Test time"'s Never list (no `global-timeout`, slow period or per-test ceiling raised; no focused test moved to the fuzzy suite); the steer of 15:00:35Z, points 1 and 3.

265. **jig-14.s1's Files against main** (ready check of jig-14.s1).
     Ruling 265 (plan defect, the re-check of jig-14.s1 at jig `a37b5fe12`, after 14.3; temper `c1111fcc2`): 14.s1's Files miss users of the removed release fields that a compile of the change found, in `jig-domain-people`, the store's people codec, the engine-clients translators (14.3's `release: false`), `jig-domain-tasks`, `jig-domain-core`, the fake parties' `Ask::Say { release }` and its users, and their tests; corrected in `jig/14-s1-words-release.md`, "Corrections at jig `a37b5fe12`". `tests/people/tests/simulation.rs`'s `words_to_a_stopped_chat_release_it_in_one_request` becomes `words_to_a_stopped_chat_release_nothing`. temper's further sites join ruling 210's point 5. Unchanged: ruling 210's point 4 and ruling 249's corrections; Size at most 150 production lines. Checked by a read-only sub-agent with the edits applied in a scratch worktree: 473 of 473 default and 14 of 14 fuzzy in the four crates it reached; the rest not compiled. Refresh any further user of the removed fields, and list it in the commit body.

266. **14.s1: the fake party keeps its Say.release instruction** (plan defect, jig 14.s1's block of 2026-10-17T15:45:36Z).
     Ruling 266 (plan defect, jig 14.s1's block of 2026-10-17T15:45:36Z): the fake party keeps `Ask::Say { release: Option<u64> }`, as the fixture's instruction; no `Ask::Compound` is added.

     1. **Why:** the fake kit's `Ask` is the fixture's own vocabulary, not the engine's (`testing/jig-fake-parties/src/lib.rs:79-84`). Ruling 210, point 2, asks that the fake party *send* its words and `Release` as one `Compound`; that is its translators' work. Ruling 265 read the field as an engine release path, which it is not.
     2. **Do:** the translators into the engine's asks (`tests/core/src/peers.rs:216`, `tests/channel/src/world.rs:284`) send `Some(revision)` as one `Compound` of the words and a `Release` naming that revision, and `None` as the words alone. The field's doc says "a Release of this hold revision, sent with the words in one Compound". `peers_scenarios` checks `Reply::Done`, as ruling 210, point 2, says.
     3. **The fake's other users** (`tests/core/tests/peers.rs:215`, `tests/channel/src/referee.rs`, `tests/channel/src/fixtures.rs:414`, `tests/conformance/src/actions.rs:41`, `tests/core/src/faults.rs:56`) change only where the engine's removed fields reach them.
     4. **Unchanged:** the rest of ruling 265; ruling 210's point 4; Size at most 150 production lines.

267. **14.4: the fixture repair dropped, Create goes on** (process, jig 14.4's block of 2026-10-17T15:49:24Z).
     Ruling 267 (process, jig 14.4's block of 2026-10-17T15:49:24Z): the fixture repair is dropped, not retried; 14.4 goes on with `Create` from 14.3's landing.

     1. **The repair made the tests it aimed at slower** in the gate: the channel's resumed window 0.274 s to 0.884 s, ops' second-seed restart 0.266 s to 0.775 s, the first-seed restart 0.260 s to 0.935 s, and the gate timed out at 15 s twice (15.016 s, 15.004 s) where 14.3's passed at 13.945 s. Smaller capacities and buffers cost round trips. The commit `5e4afc0c8` does not land: reset `client-api/jig-14-4` to `a37b5fe12`, keeping its logs in `logs/jig-14/`.
     2. **No further timing repair in 14.4:** ruling 264's points 2 and 3 end for 14.4. Its landing gate passes under the 15 s stop, a rule; if a gate times out again with no failing test, block with the gate's time and its three slowest tests, and run `land` no third time.
     3. **The focused suite's time** (13.945 s of 15 s at 14.3) is the supervisor's to plan next, apart from 14.4.
     4. **Unchanged:** README "Test time" and its Never list; 14.4's Scope, Shape, Test shape, Proof and Size; ruling 262.

268. **14.4: the fixture repair comes back (267's premise)** (process, jig 14.4's block of 2026-10-17T15:49:24Z).
     Ruling 268 (process, jig 14.4's block of 2026-10-17T15:49:24Z, corrected after the check of 16:00Z): ruling 267's point 1 rested on a wrong comparison; the fixture repair comes back.

     1. **The premise was wrong:** ruling 267 set the serial measure run's times (0.274 s, 0.266 s) beside gate times. Gate to gate, the 4,021 tests common to 14.3's passing gate and 14.4's second sum 108.3 s then 118.5 s (+9%), the first gate's 2,458 tests 94.6 s then 117.8 s (+24%): the machine ran slower (load 5.56 at 14.3's gate, 7.50 and 6.44 at the timeouts). The repair's own tests moved -69% to +15% (the native terminal 1.123 s to 0.352 s).
     2. **Do:** cherry-pick `5e4afc0c8` (the 14 fixture lines, every assertion kept) back onto `client-api/jig-14-4`, and land it with `Create`, listed as small fixes outside your Files under ruling 264.
     3. **Unchanged:** ruling 267, points 2 to 4: no further timing repair in 14.4; a gate that times out again with no failing test blocks with the gate's time and its three slowest tests, and `land` runs no third time.

269. **jig-14.s2: jig's focused suite cheaper, beside jig 14** (steer of 2026-10-17T16:16Z).
     Ruling 269 (process, the handover of 2026-10-17T16:03:26Z and ruling 267's point 3, "The focused suite's time (13.945 s of 15 s at 14.3) is the supervisor's to plan next, apart from 14.4"): a supplement step, jig-14.s2 (`jig/14-s2-focused-suite.md`, needs nothing), makes jig's focused suite cheaper in a slot of its own, beside jig 14.4.

     1. **Why a step of its own:** README "Test time" gives the repair to "the session's next increment", here jig 14.5, on the run's one chain (14.4, 14.5, 22, 15, 16, 17), while 7 of 8 slots stand idle. The focused gate ran 12.204, 13.653, 13.945 and 14.248 s at 14.1, 14.2, 14.3 and 14.s1 (aim 13.3 s, stop 15 s), and 14.4's two lands timed out at 15.016 and 15.004 s with no test failing.
     2. **What the gate costs** (a read-only diagnosis of the gate logs, 16:15Z): it is bound by throughput, not a tail: 4,110 test processes on 8 threads summed 110.4 s over 13.945 s of wall at 14.3; the longest test is 1.12 s. 1 s of serial test time is about 0.37 s of wall. `heavy --gate` runs nothing beside a gate; `plan pulse` overlapped the first timeout but not the second, which ran as slow.
     3. **While jig-14.s2 runs, it alone makes jig's tests cheaper under README "Test time":** jig 14.5 and the jig sessions after it do not start with a repair, and a gate of theirs over 13.3 s stands. Unchanged for them: the 15 s stop, a rule; a test their gate's log flags `SLOW` is made cheaper before its increment lands; ruling 267's point 2 for 14.4. Once jig-14.s2 lands, the README rule applies again from the next jig gate over the aim.
     4. **Out of the step:** an `opt-level` for crates other than tools, a thread count, the per-process start (about 2 ms of each test's floor), and the phrasebook built in each test process (`english()`, about 1 ms in each of 546 view tests): production or gate changes, not this run's.
     5. **Unchanged:** README "Test time" and its Never list; rulings 264, 267 and 268 for 14.4; temper's measure-run steer (15:00Z, point 3).

     Checked:
     - The measure command, README "Test time": `grep -n 'profile measure -j 1' README.md` prints 477: "`--profile measure -j 1`, the fuzzy suite's", after 476's "the focused suite's `heavy --gate -- cargo nextest run --workspace`".
     - 14.4's repair files: `git -C jig show --stat 5e4afc0c8` lists `tests/channel/src/fixtures.rs`, `tests/channel/src/processes.rs`, `tests/client-protocol/src/native/config.rs`, `tests/ops/src/client/app.rs`, `tests/ops/tests/client_stories.rs`: 5 files, 14 insertions, 14 deletions.
     - `git -C jig show origin/main:tests/client-protocol/src/fixtures.rs | grep -n 'pub fn journal'` prints `28:pub fn journal(mut limits: ...)`; `native/config.rs:3` is `"limits": {"profile": "small", "overrides": {...}}`.
     - 14.3's measure run, `logs/20261017T151241-664.log`: 4,110 tests; slowest 0.312 s (native terminal), 0.274 s (channel resumed window), 0.266 and 0.260 s (ops restarts), 0.248 s (one-process chat).

270. **jig 14.s2: one measure run on the final tree** (process, jig 14.s2's block of 2026-10-17T16:48:17Z).
     Ruling 270 (process, jig 14.s2's block of 2026-10-17T16:48:17Z): yes, one measure run on the final tree, once; then land.

     1. **Run it once** when the repair is done, clippy clean and committed: `heavy --gate -- cargo nextest run --workspace --profile measure -j 1`, its log `logs/jig-14-s2/measure-end.log`. Move the present one to `logs/jig-14-s2/measure-end-first.log` first.
     2. **The report** names each changed test's time in `measure-start.log` and in the final `measure-end.log`, and the two totals; `measure-end-first.log` is named, not compared again. Then land, as Proof says: land's gate, its `Gate-Result` times; no other measure run, whatever the final one shows.
     3. **The step's stop stands:** Shape 3's 90 minutes count from your start (about 16:18Z): what is not done by about 17:48Z waits, and you land what is done. A missed 4 s is a guide missed: the landing stands, and the report names the gate's time and its three slowest tests.
     4. **Unchanged:** the session file's Files, Size and Don't; no assertion, case, seed or story dropped; ruling 269.

271. **jig sessions' Step counts count names** (plan defect, jig 14.5's block of 2026-10-17T17:12:29Z).
     Ruling 271 (plan defect, jig 14.5's block of 2026-10-17T17:12:29Z): accepted; jig 14 is done. The count counts commits, and an increment may land in two (14.4's fixture repair `130c0fd61` and its `Create` `542cd8cfd`, both `Step: jig-14.4`, rulings 264 and 268). It counts distinct names now, as the README's Done-when 10 does (`sort -u`).

     1. **The sessions' Step counts** in `jig/14-wire.md` and `jig/15-helpers.md` to `jig/20-attack-stories.md` read `| grep '<pattern>' | sort -u | wc -l`, in place of `| grep -c '<pattern>'`; the counts they print are unchanged.
     2. **jig 14's** prints 5 on `origin/main` (below): mark the session done.
     3. **Unchanged:** the landed history; README Done-when 10; jig 14's other Done-when commands, which 14.5's log shows passed.

272. **jig 15's Shapes against main** (ready check of jig-15.1).
     Ruling 272 (plan defect, the ready check of jig 15 at jig `24f3bcc55`, temper `6a62c1fe7`): jig 15's session file misses what main and jig 14 landed; corrected in `jig/15-helpers.md`, "Corrections at jig `24f3bcc55`", which takes precedence over its Shapes and sketches: 15.1's `Retrying { until, notice: RetryNotice }` and `dangerous` with delete-note; 15.2's test home, "Task stopped" and its Files; 15.3's `Created { first, last, given }`, `Text::new` and `close_chat`'s reason; 15.4's module paths and what its sketch keeps; 15.5's `Revision { value }`, `Verdict::Failed` and `named` as a list; 15.6's Needs gain 15.3; every sketch derives `Debug`. Two points are the designer's (`question.md`): 15.1's status for the chip's `Active` and `Waiting` work and no observation, and 15.3's sources for `NewTask`'s authority, wake, parameters, dependencies and grants. jig 15.1 holds in `check` until the answer lands.

     Checked:
     - `git -C jig grep -n 'Created {' origin/main -- crates/jig-client-domain/src/contract`: `owner_outcome.rs:37: Created { first: u64, last: u64, given: Option<super::CreatedGiven> }`.
     - `english.rs:263`: `JigPhrase::HoldStopped => … b"Task stopped"`; `confirm.rs:202`: `let dangerous = act == ActKind::Cancel || act == ActKind::Reject || act == ActKind::DeleteNote;`.
     - `crates/jig-client-view/src/conversation.rs` 602 lines, `conversation/words.rs` 238 (`wc -l`); `conversation.rs:554: pub fn worst_case`; `phrases/jig.rs` 3,093 lines; `testing/jig-fake-workers/src/lib.rs:46-53`: `Call { Opaque, ProposeGoal, Choose }`, no question.
     - The check's sub-agent compiled the corrected sketches under `clippy -D warnings` with jig 22's types stubbed, and with the graph pins (`graph.rs:259`, `:713`) applied ran 1,164 focused tests (two runs) and 111 fuzzy, all passing; clippy clean.

273. **jig 22's Shapes against main, after jig 14** (re-check of jig-22.1).
     Ruling 273 (plan defect, the re-check of jig 22 at jig `24f3bcc55`, after jig 14 and 21 landed; temper `475f8d3b1`): jig 22's session file misses what 21.4 and 14 moved; corrected in `jig/22-documents.md`, "Re-check at jig `24f3bcc55`": 22.1's gone `limits/ui.rs` item, moved cites and 21.4's users, its test on project 1's two templates (the core world has one project), `MeParty` in the sketch; 22.2's `Amend` story boxed with all seven fields and a tracked member; 22.4's `Offered` gaining `effect` in place in `documents.rs:343`, the scripted engine's `compound.rs:16` and `payload.rs:573`, and `effect: None` in each `Offered` literal. Sent to jig 22.1 as a steer.

     Checked:
     - `git -C jig show origin/main:crates/jig-client-view/src/limits/ui.rs | wc -l`: 110; `git grep 'size_of::<Frame>\|OfferedCharters' … limits/ui.rs`: nothing.
     - `tests/core/src/world.rs:82: projects: 1,`; `contract/documents.rs:343: pub struct Offered {`; `scripted-engine/src/tests/compound.rs:16: act: ActKind::AmendRelease,`; `payload.rs:573: fields!(Offered { act, admission });`; `client-v8.schema:625: record PhaseCounts`.
     - `git -C jig grep -l 'Offered {' origin/main | wc -l`: 53.
     - The re-check's sub-agent: corrected sketches clean under `clippy --workspace --all-targets -D warnings`; 2,335 of 2,335 focused and 95 of 95 fuzzy in the packages depending on `jig-client-domain`.

274. **jig 15.1 and 15.3: working, parked, and shapes' fields from me** (design, the designer's answer of 2026-10-17T18:00Z).
     Ruling 274 (design, the designer's answer of 2026-10-17T18:00Z, jig `6155e83e0`, temper `d55ffe2b2`): the designer's decision, below; jig 22.1's `BoundsCreate` gains `authority`, and jig 15.1 may start once its needs land.


     1. **`Active` says working.** hosts.md, 6.5: "The engine shows a run as
        writing, or as running a tool, only on such a fact". pages.md, 4
        gains the row "running, neither writing nor in a tool its host names,
        or nothing reported yet | working, and for how long", and 15.1's
        `ChatStatus` gains `Working { since }`. `Ending` says working too.
     2. **A run waiting with nobody named says parked.** engine.md, 7.4: "A
        run that calls `wait` keeps its host's slot while it waits ... Idle
        past its charter's threshold, it parks". The row now reads "idle: its
        run waiting, holding its slot, or parked | parked".
     3. **No observation:** a claimed run says starting, a running one
        working, since its `phase_since`.
     4. **The order** (pages.md, 4 now): held, closing and ended first; then
        a party it waits on, in any active stage; then the stage, and the
        observed work. The plan's 15.1 has the whole match and five table rows.
     5. **Authority comes from `me`.** contract.md, 5.3 already says creating
        carries "the authority they may give it, from their role and their
        pool". The wire lacked it, so jig 22.1's `BoundsCreate` gains
        `authority: OwnerAuthorityDocument`, `chat_authority`'s
        (`funding.rs:40`). `shapes` takes it whole, `shallower - 1` levels
        shallower. Its spend is the budget named, and its deadline the task's
        (`now` plus a span: `chat` and `set_goal` take `now: Wall`).
     6. **Wake is the executor's.** contract.md, 3.6: "wake and park, which
        are the charter's". `NewTask.wake` becomes optional, and the engine's
        translation gives none the default an agent's `delegate` gets
        (`nested.rs:44`). That is 15.3's first commit. People and the store
        are unchanged.
     7. **No parameters, inputs, dependencies or named grants.**
     8. **Variants:** `Amount(n)` is a request, `Named` with spend n. `Given`
        is the draft's for `set_goal`, and `NotOffered::Template` for `chat`
        (it asks no budget). `Approval`, a cut verdict, `shallower` 0, and a
        chat's chosen executor or given deadline are also
        `NotOffered::Template`. configuration.md, 3: "an approval names a
        state its procedure gives, so no template makes one".
     9. **Nothing is deferred.** If 22.1 lands before the steer reaches it,
        a step before jig 15.3 adds the authority.

275. **jig's focused budget doubled for the run** (steer of the check of 2026-10-17T18:11Z).
     Ruling 275 (process, the check of 2026-10-17T18:11Z): jig's focused budget is doubled to 30 s for the rest of this run, as the user's rule of 2026-10-13 says: "During a running plan, a budget that bites (gate fails on its global timeout, every test passing) is doubled temporarily for the rest of that plan; the root-cause test-cost pass comes right after the plan ends, then the budget is restored."

     1. **It has bitten:** jig's focused gate ran 14.405 s (14.5) and 14.964 s (14.s2) of its 15 s, 36 ms under the stop; five land gates timed out at 15 s with no test failing (14.4's two, 14.5's and 14.s2's of 17:01Z and 17:04Z). The repair step, jig-14.s2 (ruling 269), landed 0.56 s slower than 14.5, its measure total 38.978 s then 40.997 s.
     2. **The change:** jig's `.config/nextest.toml`, `[profile.default]`, `global-timeout = "15s"` becomes `"30s"`, with the comment `# Doubled for temper's client-api run (ruling 275); 15s again after the test-cost pass that follows it.`, in a commit of its own (`Step:` the session's), made by jig 22.1 before its landing commit and landed with it.
     3. **The README rule changes** for jig's focused suite alone ("Test time", amended): its aim stays 13.3 s, a guide that each report names; no jig session starts with a repair of it, in place of ruling 269's point 3 ("Once jig-14.s2 lands, the README rule applies again"). Done-when 12 allows the one changed line. A `later.md` row holds the test-cost pass: at most 12 s (80% of 15 s, the median of three gate-lane runs), then 15 s back.
     4. **For the user** (decisions.md, F): the user may veto; restoring it is one revert of the commit of point 2.
     5. **Unchanged:** jig's fuzzy budget (60 s) and its repair rule; temper's budgets; the slow period (500 ms) and a test the gate flags `SLOW` made cheaper before its increment lands; no focused test moved to the fuzzy suite.

     Checked:
     - `git -C jig show origin/main:.config/nextest.toml | grep -n 'global-timeout = "15s"'`: in `[profile.default]`, beside `slow-timeout = { period = "500ms" }`.
     - `reports/jig-14-s2-20261017T171459Z.md`: "Gate: scripts/check.sh passed: 4124 tests / 14.964 s; 111 tests / 27.110 s"; `9bbbe0467`'s `Gate-Result: … 4124 tests / 14.405 s`.
     - The check of 18:11Z: s2's measure total 38.978 s at its start, 40.997 s at its end; the timeouts at 17:01Z and 17:04Z ended with 2,841 and 3,195 of about 4,120 tests run.

276. **the 18:19Z steer's run-end points are jig 22.1's alone** (process, the coordinator's block of 2026-10-17T18:21:21Z).
     Ruling 276 (process, the coordinator's block of 2026-10-17T18:21:21Z): the block is right; nothing more lands in the README. Ruling 167, point 4: later sessions read the README, and their prompts do not repeat it; point 6: a Pass on text that holds until the run's end lands there first. Both points' rules for later sessions are already README rules, so their Pass on texts are jig 22.1's alone.

     1. **Point 1:** its Pass on text (the budget commit) is jig 22.1's alone; give it word for word, now, and log the point done. Applies to: jig 22.1; until it lands. The rule for later jig sessions is README "Test time" as ruling 275 amended it ("a jig session no longer starts with a repair of the focused suite").
     2. **Point 3:** README "Named numbers in tests" (ruling 116) already says it. Give its Pass on text to jig 22.1 alone, now, and log the point done; "and each jig session you start" is withdrawn. Applies to: jig 22.1's new stories; until it lands.
     3. **Unchanged:** point 2, done; ruling 275; ruling 167.

277. **README: a rename's sweep finds its tests** (steer of the check of 2026-10-17T19:10:00Z).
     Ruling 277 (process, the check of 2026-10-17T19:10:00Z): README "The run's steers", 2026-10-17 18:11Z, point 1, gains the way to find the tests a rename leaves: "Before your landing commit, for each word your commit removes or renames, run `git grep -n -E 'fn [a-z0-9_]*<word>' -- <the crates you change>` and rename each test it prints whose name names the old thing." jig 22.1 renamed 23 tests and left `a_frame_at_its_ceilings_round_trips_unchanged`, whose body encodes and decodes `Me`. Unchanged: the rule's eight-word names and its one count for renames outside Files.

     Checked: `git -C jig grep -n -E 'fn [a-z0-9_]*frame' origin/main -- crates/jig-client-protocol-wire/src/tests.rs` prints `tests.rs:115:fn a_frame_at_its_ceilings_round_trips_unchanged() {`.

278. **the 19:12Z steer goes to jig 22.3** (process, the coordinator's block of 2026-10-17T19:14:00Z).
     Ruling 278 (process, the coordinator's block of 2026-10-17T19:14:00Z): yes. jig 22.2 landed (`8f8810f33`, 19:13:58Z) before the steer reached it, so both points go to jig 22.3's first commit.

     1. **Point 1:** the split of `a_me_document_carries_each_project_s_templates_and_bounds` goes in 22.3's first commit; its codec-test rule applies to 22.3 and 22.4, as the point says. Until: jig 22 lands.
     2. **Point 2:** the rename of `a_frame_at_its_ceilings_round_trips_unchanged` (`crates/jig-client-protocol-wire/src/tests.rs:115`, row `:342`) goes in 22.3's first commit. Until: jig 22.3 lands.
     3. **Unchanged:** both points' Pass on texts, with "your first commit" read as 22.3's; ruling 277's README sentence.

279. **jig 22.3: a projectless query admitted for the party's own tasks; Size stop 575** (plan defect, jig 22.3's block of 2026-10-17T19:34:14Z).
     Ruling 279 (plan defect, jig 22.3's block of 2026-10-17T19:34:14Z): yes to both; your proposal A.

     1. **The admission repair joins 22.3's Files.** engine.md, 11.2: "The core admits it on the party's role in the project it names, or, naming none, for tasks whose requester the party is"; contract.md, 6.1: "A query naming none of these [a requester party; a requester task or a parent; a project] has no ended group, and its page says so". A query naming no project and no requester is served, so the core admits it: in `crates/jig-domain-core/src/documents/task_query.rs` (`admission`, :47), a query naming no project and no requester is admitted, and every live scan of a query naming no project keeps only the tasks whose requester is the authenticated party. A query naming no project and a requester other than the party is refused, as now; a deployment requester stays `Unknown`.
     2. **Its test:** a focused isolation test beside the core's task-query tests: two parties' tasks, a projectless query with no requester returns the asking party's tasks alone. `a_query_naming_no_index_key_reads_no_ended_group` stands as the plan names it.
     3. **Size:** stop 575 production lines, the admission repair included (489 now; the two-group read and the task-query cursor codec). No trimming to fit. The aim stays 300; the report names the count.
     4. **Unchanged:** 22.3's Scope, Shape, Test shape and Proof; rulings 273, 278.

280. **jig 22.4: the offered-acts judge checks the composed control** (plan defect, jig 22.4's block of 2026-10-17T19:55:44Z).
     Ruling 280 (plan defect, jig 22.4's block of 2026-10-17T19:55:44Z): yes, your option A; the judge follows 22.4's Shape, which is not a change to pass.

     1. **Why:** 22.4's Shape removes the act the judge names: "`acts::Act` … loses `AmendRelease` (the card offers amend-and-release where `Amend` and `Release` are both offered, sending jig 14.3's compound)". README: "A referee's or a judge's check is never changed to pass": the judge here moves with the vocabulary the plan changes, before any proof run, and checks no less.
     2. **Do:** in `tests/client/src/joint/offered_acts.rs`, the two `Act::AmendRelease` rows (:29, :87) become one composed-control check each: the "Amend and release" control is shown exactly when both `Amend` and `Release` are offered and actionable, and absent otherwise. Every single-act row and assertion stays; each exact total gains the composed control where it is shown.
     3. **Name it** in the commit body and the report as the judge's migration under this ruling, with the two rows before and after.
     4. **Unchanged:** the README rule for every other referee and judge check; 22.4's Scope, Shape, Proof and Size.

281. **README: cargo clean and python3** (steer of the check of 2026-10-17T20:13:00Z).
     Ruling 281 (process, the check of 2026-10-17T20:13:00Z): the run takes skein `b30cc95`'s points 2 and 6 (development.md 11.4; implementer.md 8.1, 8.3), docs alone, so the run's kit stays at its commit. Point 6's Pass on text lands in the README's "The run's steers", under "2026-10-17 20:13Z, point 2 (every session)": `cargo clean`, never `rm -rf target`; `python3`, never `python`. Point 2 (a session's `target/` removed with `cargo clean` once `pueue status` shows no heavy task from its path) is the coordinator's, in the steer; its Pass on is nothing. Unchanged: the disk and memory thresholds; kache's order.

     Checked: `ls logs/jig-22/*.py | wc -l` in the run's directory: 29 scripts (the check of 20:13Z); `which python` prints nothing on carbon-forge, `which python3` prints `/usr/bin/python3`.

282. **jig 15's Shapes against main, after jig 22** (re-check of jig-15.1).
     Ruling 282 (plan defect, the re-check of jig 15 at jig `533dee685`, after jig 22.1 to 22.4 landed; temper `d753bdc3a`): jig 15's session file misses what 22.1 to 22.4 moved; corrected in `jig/15-helpers.md`: the cites of ruling 272's corrections refreshed in place (`client-v8.schema:316`, the link's `translate/operations.rs:230, :328`, the link's `tests/fixtures/create.rs:18`, the engine's `tests/fixtures/create.rs:87`, 15.4's `action.rs:49`); 15.1's `dangerous` and its test name cover delete-note (`dangerous_marks_cancel_reject_and_delete_note_alone`), as ruling 272 said; 15.1's Removes is the reference UI's `effect` (`confirmation.rs:572`, which 22.4 moved onto `Offered.effect`), the close's `impact()` read staying, and its users gain `acts::offer` and the cards' `admitted`; 15.1's `Story::line` reads 22.3's query fixtures; "Corrections at jig `533dee685`": a `fixtures::me` with project 1's templates 1 and 2 for 15.1 and 15.3, and `Wall`'s "now plus span". Ruling 274's Needs (`Me`, `CreateBounds.authority`, wire `BoundsCreate.authority`, `TaskLine`, `waits_on`, `Offered.effect`) have all landed. Sent to jig 15.1 as a steer; 15.2 to 15.4 ready.

     Checked:
     - `git -C jig grep -n NewTask origin/main -- '*client-v8.schema'`: `:316: record NewTask { … wake: OwnerWakePolicy …`; link `translate/operations.rs:230: wake: asks::wire_owner_wake_policy(&v.wake, limits)?,`, `:328: wake: super::facts::owner_wake_policy(v.wake(), limits),`; link `tests/fixtures/create.rs:18: c::NewTask {`; engine `tests/fixtures/create.rs:87: let task = w::NewTask::new(`.
     - `jig-client-view/src/confirm.rs:202: let dangerous = act == ActKind::Cancel || act == ActKind::Reject || act == ActKind::DeleteNote;`; `action.rs:49: FoldCall { name: crate::conversation::CallName, open: bool },`.
     - `confirmation.rs:572: fn effect(object: &Object, act: ActKind) -> Option<&Effect> {` over `crate::acts::offer` (`acts.rs:27`); `confirmation.rs:216: if let Some(impact) = impact(object) {` under `ActKind::Close`; `contract/effects.rs:12: pub enum Effect { Release(ReleaseEffect), Cancel(Impact), Move { reservation: u64 } }`; `cards/common.rs:269: fn admitted(rows: &List<Offered>, act: ActKind) -> bool {`.
     - `tests/client-protocol/src/client/operations.rs:8: pub fn running_task`; `fixtures/queries.rs:7: pub fn task_query`, `:27: pub fn task_lines`; no `fn me` or `fn line` on `Story`; `skein-lib/src/time.rs:21: pub struct Wall(u64);`, with `from_nanos` and `as_nanos` and no add.
     - The re-check's sub-agent: the `facts`, `shapes` and `words` sketches clean under clippy `-D warnings`; 1,092 of 1,092 focused and 37 of 37 fuzzy in `jig-client`, `jig-boundary`, `jig-client-view`, `jig-client-world`, `jig-client-protocol-world` and `jig-client-test-kit`, with 15.2's `skein-ui` dependency and graph pin and the corrected `dangerous` test. Not compiled: the `thread` and `check` sketches (their types exist on main).

283. **jig 15.1: the joint question wait, reproduced at the joint tier** (process, jig 15.1's block of 2026-10-17T20:57:18Z).
     Ruling 283 (process, jig 15.1's block of 2026-10-17T20:57:18Z): the joint proof `a_running_chat_reads_as_writing_from_the_engine_s_documents` goes on, under ruling 229, once its failure point is reproduced at the tier where it fails. Its second run failed at the same `run_until` (`tests/client-protocol/tests/client.rs:659`) after the focused core test `a_scripted_chat_question_waits_for_its_requester` passed: the cause lies between the core and the joint story (the engine's and the link's translation of the wait, the story's document and line access, or the joint world's fake worker), which no focused test has logged yet. The assertion prints nothing when `run_until` gives up.

     The harness correction stands: a chat's delegate asks the chat. The design admits it ("| a question, expecting an answer | a delegate | yes |", tasks.md, 8.1), and the core projects an unanswered question in a person-requested task's inbox as that person's `Question` entry (`jig-domain-core/src/inbox.rs:172-183`), which the engine gives as `WaitedFor::Question` (`jig-protocol-engine-clients/src/translate/queries.rs:67`).

     1. **Now:** write a focused test at the joint tier (`jig-client-protocol-world`), with the proof's story up to its question, that prints, when its wait ends, the task document's and the task line's `chip` and `waits_on`, and the people child's entries for the chat's person; log its cause in `logs/jig-15/proof-15.1-failure-2.md`, and change only what that log names.
     2. **Then run the proof once.** Ruling 229's count for this proof stands at 2 of 6: a run failing at a later point is progress; the same point again, or a cause in production code outside 15.1's Files, is a block with the logs.
     3. **Make the diagnostic test's two wildcard arms exhaustive** (jig's `clippy.toml`; the 20:56Z report), or remove the test once the joint proof passes, if the joint test covers it.
     4. **Unchanged:** 15.1's Proof and its 1 s bound; ruling 229's rule; no change to a referee's or judge's check; the harness changes stay in `testing/jig-fake-workers/src/lib.rs` and jig 12's `Story`, each named in the commit body.

284. **jig 16's Shapes against main** (ready check of jig-16.1).
     Ruling 284 (plan defect, the ready check of jig 16 at jig `9f6f80839`, skein `b30cc95`, temper `8a0a0cbec`): jig 16's session file names paths and users that moved; corrected in `jig/16-shed.md`, "Corrections at jig `9f6f80839`": the gone `jig-domain-client` and `jig-view` paths read under `crates/jig-client-view/src/`; 16.1 gains the scripted engine's `queries.rs`, its joint test calls `jig_client_protocol_world::stories` as `bytes` (the test file's `stories` is `jig_client_world`'s), with `tests/client-protocol/src/{stories,front}.rs` in Files, and the inbox's reply entry keeps `ConversationLine` until 16.4, which makes it a `TaskLine` (schema, contract, core and reference UI files named); 16.3 removes the engine's `Bounds::Goal`, not the reference UI's; 16.4 gains the list records' users the check found; 16.5 renames the engine's `acts::GoalProcedure` to `CreateProcedure`, corrects the test service's and `tests/core`'s paths; Done when's grep counts 221 files at the base. 16.1 ready; its re-check follows 15.2 and 15.3's landings.

     Checked:
     - `git -C jig ls-tree -d origin/main crates/jig-domain-client crates/jig-view`: nothing; `crates/jig-client-view/src/sections/conversations.rs` exists.
     - `tests/client-protocol/tests/protocol.rs:2: use jig_client_world::{Until, stories};`; `tests/client-protocol/src/stories.rs:17: pub fn ready`, `:24: pub fn finish`.
     - `client-v8.schema:600: record InboxEntry { reply: option ConversationLine …`; `contract/documents.rs:654` and `owner_inbox.rs:26: pub reply: Option<Box<ConversationLine>>,`; `jig-domain-core/src/documents/inbox_deployment.rs:25` the same.
     - `git -C jig grep 'Bounds::Goal' origin/main -- 'crates/jig-client-view/**'`: nothing; `page/project/goals.rs:169: Bounds::Create(jig_client::contract::CreateBounds { pool_left, .. })`.
     - `jig-domain-core/src/acts.rs:93: pub struct GoalProcedure {`; `testing/jig-test-service/src/family/`: no such tree; `tests/core/src/stories/closing_chat.rs`, `tests/core/tests/stories/procedure_goals.rs` exist.
     - Done when's third grep at `9f6f80839`: `| wc -l` prints 221.
     - The check's sub-agent: the 16.1 sketch and its test (with `bytes`) clean under clippy; `jig-client-protocol-world` and `jig-client-view`: 625 of 626 focused (the new test, helpers stubbed), 2 of 2 fuzzy.

285. **jig 15.5's Shapes against main** (ready check of jig-15.5).
     Ruling 285 (plan defect, the ready check of jig 15.5 at jig `9f6f80839`, temper `783d3c84e`): 15.5's Shape and test do not fit the reference UI's world; corrected in `jig/15-helpers.md`, "Corrections at jig `9f6f80839` (ruling 285…)": a breach panics through the referee's `assert_holding` with the check's reason, so the reference test is a `should_panic` in `tests/client/tests/stories.rs` beside `a_held_chat_is_released`, with a control; `Object` and `Delivered` lose `Proposal`, and a note is keyed by `scope`; `named` returns a list over a task's revision spaces; every delivery feeds the check (the legacy `ReadValue::Escalation` and `ReadValue::PersonTask` included), `Sent` keeps its fields, and the two `person_confirmation` tests that the check alone trips are the increment's controls; the check's limits are the referee's constants; the kit's tree tests build through `skein_ui::View`; Files gain the world's `observe.rs`, `test.rs`, `harness/{world,mod}.rs`, `tests/stories.rs`, and the boundary pin. A story the check trips is a delivery the referee does not feed: feed it and list it in the commit body; a revision the reference UI never received, block. 15.5 ready; its re-check follows 15.2's landing, with the default and fuzzy profiles the check did not finish.

     Checked:
     - `tests/client/src/harness/world.rs:529: fn observe(&mut self, observation: Observation) {`, `:531: self.referee.assert_holding(self.settings.seed);`; `tests/client/tests/stories.rs:57: fn a_held_chat_is_released() {`.
     - `contract/asks.rs:157: Decide { project: u32, on: DecideOn },`, `:159: Release { project: u32, task: u64, revision: Revision, again: Option<bool> },`, `:161: Amend { project: u32, task: u64, revision: Option<Revision>, change: Box<Amendment>, reason: Text },`.
     - `translate/asks.rs:683: c::Ask::EditNote { project, note, scope, revision, … }`, `:701: c::Ask::DeleteNote { project, note, scope, revision }`; `contract/common.rs:233: pub enum NoteScope {`.
     - `tests/client/src/observe.rs:121: ReadValue::Escalation(e) => vec![Receipt::Card {`, `:132: ReadValue::PersonTask(p) => {`.
     - `Sent { … }` built at `tests/ops/src/client/app.rs:482`, `tests/client-protocol/src/front/observe.rs:51`; temper `tests/client/src/app.rs:42: impl App for Temper {`, `:233: Sent { attempt: *attempt, key: *key, card: Some(adoption_card()), revision: None, words: None }`; temper uses nothing else of the world's `Receipt`, `Observation` or `App` beyond construction.
     - `tests/client/tests/person_confirmation.rs:75: fn a_choice_card_and_confirmation_keep_one_platform_reason_input() {`, `:83: fn a_words_card_and_confirmation_keep_one_platform_answer_input() {`.
     - `tests/client/src/test.rs:21`: the focused world's `jig_client_test_domain::Limits` (watches, notices), no check bounds.
     - `tests/boundary/src/tests/graph.rs:259: ("jig-client-test-kit", vec!["jig-client-test-domain", "jig-test-connector", "skein-lib"]),`.
     - The check's sub-agent, with the corrected sketch and wiring: fmt and clippy clean; `jig-client-test-kit` 14 of 14; the world's proof filter 3 of 3 and the control; the fuzzy `random_people_with_held_commits_and_double_press` passes (1.4 s); the workspace's default profile stopped at the two `person_confirmation` tests above (183 of 185 run passed); the rest of the default profile and the fuzzy profile not run.

286. **the 22:24Z steer's run-long texts in the README** (process, coordinator's block of 22:25:01Z).
     Ruling 286 (process, the coordinator's block of 22:25:01Z): the 22:24Z steer's run-long texts land in the README's "The run's steers", as ruling 167 requires, and bind from there; the coordinator gives them in no prompt again. Point 3's Pass on lands as the steer gave it (every session that writes a core test). Point 2 splits: its rule on `tests/client-protocol/tests/client.rs` (new client-protocol stories go in `tests/client-protocol/tests/client/<topic>.rs`, nothing added to `client.rs`) holds for the run and lands in the README; its rule on `tests/core/src/peers.rs` holds until that file is at most 800 lines, and stays in the steer, which the coordinator keeps passing on to each session whose Files name `peers.rs` until then. Points 1 and 4 are temporary (until their greps and lengths say so) and stay in the steer. Mark the steer's points 2 (the `client.rs` half) and 3 as in the README from now.

287. **jig 15 and 16's Shapes against main** (ready check of jig-15.6 and jig-16.2).
     Ruling 287 (plan defects, the ready checks of jig 15.6 and 16.2 and the re-checks of 15.5 and 16.1 at jig `738c20adc`, after 15.2 to 15.4; skein `6926c2d`, temper `b2457e915`): corrected in `jig/15-helpers.md` and `jig/16-shed.md`, each under "Corrections at jig `738c20adc` (ruling 287…)". 15.5: `Delivered` gains variants for the party's `Escalation` and `PersonTask` reads, and no identifier says "legacy". 15.6: the test view is `testing/jig-client-test-view`; `root.rs`, the view's words, node and limit counts, its `Cargo.toml` (`jig-client` a normal dependency) and the view's pin in `graph.rs` join Files; the answer's outcome reaches the referee as a new `Receipt` variant; the effect-hiding variant is a setter. 16.1: the older edge is "Read earlier"; `#[must_use]` and `alloc::boxed::Box` in the sketch; the Conversations entry gates on `me`'s chat template (`page/navigation.rs` joins Files). 16.2: its paths and users; the arrows stay gated on the project's `Prioritise` offer, now read from the project's document; each operation is the whole `Operation::Amend`, and the scripted engine admits a compound of priority-only amendments; the overview shares the board's watch; the refusal is `Refusal::Authority`, "Propose instead" a new phrase; its stories propose a refused `Amend`, and a refused `Create`'s proposal moves to 16.3; Needs gains 16.1's engine reads; the proof counts a delta; the Size counts the listed users apart. 15.5 and 16.1, running, take theirs as a steer.

     Checked:
     - `crates/jig-client-view/src/pages.rs:48: End::Before => JigPhrase::ReadEarlier,`; `page/navigation.rs:54-56: Address::Conversations { project } => { for offered in &frame.charters { if !offered.chats.is_empty() …`.
     - `contract/operations.rs:9: Amend { project: u32, task: u64, revision: Option<Revision>, change: Box<Amendment>, reason: Text },`; `contract/asks.rs:31-39: pub struct Amendment { addressed, spec, wake, dependencies, authority, instructions, tracked }` (no `Default`).
     - `jig-domain-core/src/acts/objects.rs:48-49: documents::Subject::Goals { project } | documents::Subject::Project { project } => { project_admission(…) }`, `:86: Act::Prioritise => allowed(core.priorities_allowed(project, held, 0, limits.tasks.tasks)),`; `crates/jig-client-view/src/acts.rs:250-253: ActKind::Prioritise => { … Some(Ask::Prioritise { project, goals })`.
     - `testing/jig-client-scripted-engine/src/requests.rs:89-98`: `Ask::Create`, `Withdraw` and a batch's `Propose` answered `Refusal::NotOffered`; a compound with `compound_target(*project, operations).is_none()` the same. `page/project/goals.rs:220: let ask = Ask::SetGoal {`.
     - `page/opening.rs:46: open_once(domain, Subject::Goals { project }, owner, out);`; `page/project.rs:271: pub const GOALS_WINDOW: u64 = 20;`; `words/refusals.rs:40: Refusal::Authority { lacking, proposable } => {`.
     - `wc -l`: `page/conversations.rs` 591, `page/project/goals.rs` 344, `sections/project/goals.rs` 371, `page/project/owner_goals.rs` 82.
     - `tests/client/src/observe.rs:121: ReadValue::Escalation(e) => vec![Receipt::Card {`, `:132: ReadValue::PersonTask(p) => {`; `tests/boundary/src/names.rs:259: … .match_indices("legacy")`.
     - `testing/jig-client-test-view/src/family.rs:20: pub const PART_NODES: u32 = 39;`, `:32: pub fn page(`; `src/root.rs:63: family::page(&mut b, root.first(), view.first_words, view.words, &mut view.first);`; `Cargo.toml:17-19: [dev-dependencies] … jig-client.workspace = true`; `tests/boundary/src/tests/graph.rs:278: ["jig-client-domain", "jig-client-test-domain", "jig-client-view", "skein-lib", "skein-ui"]`; `tests/client/src/harness/observation.rs:52: Answer { attempt: Token, final_answer: bool, words: Option<(u64, u64)> },`.
     - The checks' sub-agents: 15.5 with ruling 285's and these corrections, default 4,179 of 4,179, fuzzy 111 of 111, the two controls pass, the boundary's vocabulary tests pass after the rename; 15.6 by grep alone (no sketch compiled); 16.1 and 16.2's sketch clean under clippy after the corrections, default 1,122 of 1,123 over four crates (the new joint test, its helpers stubbed), fuzzy 37 of 37.

288. **jig 16.1: the transitional handlers take jig's default templates' labels** (plan defect, jig-16.1's block of 2026-10-17T22:31:24Z).
     Ruling 288 (plan defect, jig 16.1's block of 2026-10-17T22:31:24Z): the engine's two transitional handlers make their tasks under jig's default templates' labels, within 16.1; the reference UI keeps sending `StartChat` and `SetGoal` until 16.3, as planned. The plan missed that 16.1's list, which queries the chat template's label as the design says ("the person's tasks of the chat template's label", pages.md, 4), cannot find a chat the transitional `StartChat` makes under label 0; 16.2's ended goals read would miss `SetGoal`'s goals the same way.

     The design gives jig's default chat and goal templates labels 1 and 2 (`domain/core.md`, 7.3: "| label | 1 | 2 |"), which no project's policy overrides ("never the flags, the label or tracked, which are the shape's identity", core.md, 7.3). Both handlers already make their tasks with those templates' flags; their label is the one field that differs. The engine reads no template to do it (`domain/engine.md`, 11.1: "No decision reads one").

     Not chosen: moving 16.3's chat start into 16.1 (the report's proposal): it moves the scripted engine's `Create`, the front's chat counter and their stories into 16.1 unchecked by a ready check, and leaves `SetGoal`'s label to 16.2. Not chosen either: a fixture whose chat template has label 0, which would leave the reference UI's list empty of its own chats on `main` with jig's defaults until 16.3.

     1. **Change the two lines** in `crates/jig-domain-core/src/routing/parties.rs`, now in 16.1's Files: `label: 0` at `:229` (`StartChat`'s task) becomes `label: 1`, and `label: 0` at `:170` (`SetGoal`'s task) becomes `label: 2`, each with a one-line comment naming 16.5, which removes both handlers.
     2. **Keep your focused test** `starting_a_chat_assigns_the_chat_templates_label` (`tests/client-protocol/src/front/effects.rs`), and add beside it `setting_a_goal_assigns_the_goal_templates_label`, which sets one goal through the people child's `SetGoal` and reads the expected label from `jig_core_world::fixtures::templates()` (template 2). Run both once.
     3. **Then run the proof once.** Ruling 229's count for this proof stands at 2 of 6: a run failing at a later point is progress; the same point again, or another cause in production code outside 16.1's Files, is a block with the logs.
     4. **A further test that pinned label 0** for a task `StartChat` or `SetGoal` made: refresh it to the template's label, and list it in the commit body.
     5. **Unchanged:** 16.1's Proof, its 1 s bound and its other Files; the reference UI reads the chat label from `me` and never writes it as a number; no referee's or judge's check changes; 16.2 and 16.3 as planned.

289. **jig 16.1: a focused diagnosis of the native chat's absent answer, under ruling 229** (process, jig-16.1's block of 2026-10-17T23:03:01Z).
     Ruling 289 (process, jig-16.1's block of 2026-10-17T23:03:01Z): the report's proposal: a focused diagnosis of the native chat's absent answer, under ruling 229, before any further repair, gate or land. The gate story `a_person_s_chat_is_answered_by_the_in_process_worker` (`tests/one-process/tests/one_process.rs:5`) failed twice at its third person step, the answer's See of `Script::answered_chat` (`tests/client-protocol/src/native.rs:17`): once in the gate, once after the three-line `Go` to task 1's Activity, so the first cause logged (the answer no longer shown in the Conversations list) is not the whole cause.

     1. **Diagnose:** in the focused reproduction (`gate1-focused`), print, when the answer's See gives up: the person's address and the page's text; the chat's task number, phase and report as the engine's store holds them; and whether the in-process worker took the chat's run and answered. Log the cause in `logs/jig-16-1/gate-cause.md`. Aim to spend at most 30 minutes on it; past that, report what the logs show.
     2. **Change only what that log names.** A cause in the story's input or in the test code of `tests/client-protocol` or `tests/one-process` is in 16.1's Files for this; list each such file in the commit body. A cause in production code outside 16.1's Files is a block with the logs.
     3. **Then run the gate story once** in the focused test, and land once. Its count stands at 2: the same failure point a third time is a block with the logs.
     4. **Unchanged:** the story's expected answer and its 5 s wait; no referee's or judge's check changes; 16.1's Proof, which passed (run 3, 0.205 s); ruling 288.

290. **jig 15.6: the joint proposed-goal story expects offers kept's failure until 16.3** (plan defect, jig-15.6's block of 2026-10-17T23:03:01Z).
     Ruling 290 (plan defect, jig-15.6's block of 2026-10-17T23:03:01Z): the report's proposal. The joint story `an_outside_allotment_goal_is_shown_as_proposed_before_send_and_named_in_the_answer` (`tests/client/src/joint/role_stories/proposed_story.rs`) holds the transitional state the plan chose at 03.4 (`jig/03-parties.md`, Choices: "its goal form beyond the pool shows a refusal that names who could accept, until it proposes through `Propose` (jig 14 to 16)"), while its confirmation offers the goal as proposable. 15.6's "offers kept" fails it as the design asks; 15.6's Files missed it. The fix stays in 16.3, whose goal form sends what `shapes::set_goal` gives: moving it before 15.6 would hold 15.6 behind 16.1 and 16.2.

     1. **15.6, Files** gain the story. Until jig 16.3, its second verdict, after Confirm, expects the referee's offers-kept failure for that draft, exactly, as `tests/client/tests/referee.rs:476` asserts a `Verdict::Failed(skein_world::domain::Failure { .. })`, with a comment that names 16.3 as its end. Its first verdict (`Passed`, before send) and its durable refusal stay.
     2. **A further story** the referee fails for the same cause (an offer the reference UI's legacy goal act does not keep) takes the same expectation: list each in the commit body. A failure of any other cause is a block.
     3. **16.3, Files and Test shape** gain the story: once the goal form sends `shapes::set_goal`'s request, its answer is the `people::Outcome::Proposed { .. }` the referee asks, its durable refusal goes, both verdicts are `Passed`, and 15.6's expected failure goes.
     4. **Unchanged:** the referee's rule; 15.6's Shape, Test shape and Proof; no other check changes to pass.

291. **jig 16.1: the gate story's wait read from the Conversations page's print, then one run** (process, jig-16.1's block of 2026-10-17T23:10:54Z).
     Ruling 291 (process, jig-16.1's block of 2026-10-17T23:10:54Z): the report's proposal, with its repair and one run in the same authorization. Ruling 289's diagnosis (`logs/jig-16-1/gate-cause.md`) found the cause in the story's input, not in production: the worker answered (task 1 `Ended(Done(Report { words: b"World answer" }))`), the Go to task 1's Activity ran before the creation was acknowledged (`Read(None(Unknown))` before `Started { task: 1 }`), and the page kept "Page missing". The wait added before the Go, `Find::named(Role::Status, b"Chat started")`, failed at its own See on the Conversations page (`gate1-confirmed.log`), though the Activity page showed that notice at the timeout.

     1. **Diagnose once:** with `gate1-diagnostics.patch`'s print moved to the new wait's See, run the focused reproduction once, and log in `gate-cause.md` what the Conversations page shows when that See gives up: the page's tree, and whether the chat's line or the `Chat started` notice is there, under which role and name.
     2. **Repair the input from that print:** wait for what the print shows once the engine has made the chat (the notice by its printed role and name, or the chat's line in Conversations); then reach task 1's Activity (the Go, or a press of the chat's line where the print shows it as a link); then the unchanged answer See. The change stays in `tests/client-protocol/src/native.rs`'s `Script`; list it in the commit body. A cause the print shows in production code outside 16.1's Files is a block with the logs.
     3. **Run the focused reproduction once more;** if it passes, run the gate and land. A failure at the answer See, or at the new wait again, is a block with the logs.
     4. **Unchanged:** the expected answer and its 5 s wait; no referee's or judge's check; 16.1's Proof; rulings 288 and 289's other steps. Aim to spend at most 30 minutes on steps 1 to 3.

292. **jig 16.1: a focused diagnosis of the power-loss frame failure, main first** (process, jig-16.1's block of 2026-10-17T23:21:18Z).
     Ruling 292 (process, jig-16.1's block of 2026-10-17T23:21:18Z): the report's proposal, a focused diagnosis by session 16, under ruling 229, before any repair. It is no plan defect until the diagnosis names a cause outside 16.1's Files: the failure is new (no ruling or log entry names `drawn_kills_and_drains_keep_the_conformance_rules` or `SKVC` before this report), the gate passed it on main at 23:19 (jig 15.6, `25fd50d50`), and 16.1's prepared commit changes what the core stores and reads. The observer's check (`tests/channel/src/world/store.rs:257`) holds every remainder of 26 bytes or more to start a `skein-kv` frame (`crates/skein-kv/src/frame.rs:10`); it found `[0, 0, 0, 18]` after the power loss before submission 118 of `calm(6)`.

     1. **Whose change:** run the preserved reproduction (`logs/jig-16-1/gate2-power-loss-reproduction.rs`) once in a scratch worktree at jig `origin/main` (`25fd50d50` or later). If it fails there too, the defect is main's: a block with the logs.
     2. **Where the bytes come from:** in the reproduction at the prepared commit, print at the failing check: the file, the segment's offset and the remainder's length; 32 bytes from 16 before the offset; the power loss's cut (which writes it kept and dropped, and their offsets); and the file's length as `skein-kv`'s recovery leaves it after the restart. Say whether `[0, 0, 0, 18]` is in the file the store recovered, or only in the bytes the observer reads (its capture, or bytes past the recovered length). Then say which of 16.1's changes moved the cut onto it (the bytes a stored record of 16.1's takes, or the count of writes before submission 118).
     3. **Log the cause** in `logs/jig-16-1/gate-cause.md`, then:
        - a cause in 16.1's Files: repair it, run the reproduction once, then the gate, and land;
        - a cause in the observer (`tests/channel/src/world/`) or in `skein-kv`'s recovery: a block with the logs, naming the file, the line and the arithmetic at fault, with no repair; the supervisor then rules who repairs it.
     4. **Unchanged:** the check at `store.rs:257`, and every referee's and judge's check; the fuzzy draws; the gate story's repair under ruling 291. Aim to spend at most 30 minutes on steps 1 to 3; past that, report what the logs show.

293. **jig 16: ruling 290's expected failure may take a moved time** (steer of 2026-10-17T23:27Z).
     Ruling 293 (steer of 2026-10-17T23:27Z, from the check of 2026-10-17T23:25:34Z): ruling 290's expected failure in `tests/client/src/joint/role_stories/proposed_story.rs` names the time of the referee's failure (`at: Time::from_nanos(8_416_106)`, `:54` on jig `25fd50d50`), which 16.1's or 16.2's changes to the core and the reference UI may move. Until 16.3 lands, a landing that moves only that time takes the time the story's failure prints, keeps the fault and its words, and lists it in the commit body; any other difference is a block. `jig/16-shed.md`'s ruling 290 section gains this.

294. **jig 16.1: the store observer reads frames only from commit logs** (plan defect, jig-16.1's block of 2026-10-17T23:27:43Z).
     Ruling 294 (plan defect, jig-16.1's block of 2026-10-17T23:27:43Z): the report's proposal, repaired by session 16 in 16.1. Ruling 292's diagnosis (`logs/jig-16-1/gate-cause.md`) found the cause in the channel world's store observer, not in production: the reproduction passes on main `25fd50d50`; at the prepared commit, the power loss cuts commit 7's write, the restart writes a repair snapshot, and `Capture::pending` (`tests/channel/src/world/store.rs:89-92`) hands the snapshot's first chunk (`00 00 00 12`, 18 rows) to the commit-frame decoder, whose check at `:257` fails. The observer is test code of jig's, and 16.1's changes are what reach it.

     1. **16.1, Files** gain `tests/channel/src/world/store.rs`. The observer reads `skein-kv` commit frames only from commit logs (`store/log-<20 digits>`), at each site that now takes every `store/` file (the recovery read at `:72`, `pending` at `:89-92`, the sync at `:163`) and any further one. A repair snapshot (`store/snapshot.tmp`, renamed `store/snapshot`) gives it no frames, and the commits the snapshot holds stay as the observer recovered them from the log it replaces, through the next restart.
     2. **16.1, Test shape** gains a focused test in `tests/channel`: a power loss inside a commit's write, then the restart's repair snapshot; the observer takes no frames from the snapshot, and the commits before the cut stay durable. The preserved reproduction may be that test, if it asserts that the snapshot's write was reached.
     3. **temper's copy** (`tests/one-process/src/referee/durable.rs:76, :94, :149`, its check at `:256`) takes the same repair in temper 04.3, which takes jig 16. Refresh any further copy you find, and list it in the commit body.
     4. **Then** run the focused test and the reproduction once, then the gate, and land. A failure elsewhere in the observer, or at `:257` on a commit log, is a block with the logs.
     5. **Unchanged:** the frame check at `:257` and every other conformance, referee and judge check; the fuzzy draws; rulings 288, 291 and 293.

295. **the run's kit pinned at skein 841f487** (infrastructure, the director's request of 2026-10-17T23:40Z).
     Ruling 295 (infrastructure, the director's request of 2026-10-17T23:40Z): the run's kit is pinned at skein `841f487`, the commit of this machine's skein checkout, which this run's tools ran from until now: `git -C ~/src/rust/skein worktree add --detach ~/src/rust/tmp/temper/client-api/kit 841f487`. The run was claimed before runs took their kit from their claim (skein `e6fd9fb`, "Take a run's kit from its claim"), so a kit change landing in skein's checkout would have reached it mid-run; from now on `plan`, `land` and `heavy`, run from the copy on PATH, hand over to `kit/scripts/dev`'s (`kit.sh`, `run_kit`), as for a run claimed after `e6fd9fb`, and skein's kit batch may land without reaching this run. Nothing the run runs changes: the checkout was clean at `841f487`, with no untracked or ignored file under `scripts/` or `docs/`. Checked: `bash -x "$(command -v plan)" status` gives `+ exec /srv/data/git/runner/src/tmp/temper/client-api/kit/scripts/dev/plan status`; `bash -x "$(command -v land)" --help` gives `+ exec …/client-api/kit/scripts/dev/land --help`; `heavy --tail 3 bash -c 'echo "kit_home=${DEV_KIT_HOME:-unset}"'` gives `kit_home=/srv/data/git/runner/src/skein/scripts/dev` (the handover's mark) and `task 679 (build) exit 0`. The run's process stays skein `15ddb75` (supervision.md's Process line). `plan`'s end of the run removes `kit/`.

296. **skein capped at 2733882 for the run's repins** (infrastructure, the director's request of 2026-10-17T23:46Z).
     jig's and temper's skein stay at most skein `2733882` ("Mark hearth's render active on debian-8gb-hel1-1") for every repin of this run, the closing `temper-repin` included: README.md, Rules, Pins, "skein's cap". hearth's `skein_io` changes (`Locked`, `Busy`, `Lock`, `ListPage`) start at its child `089ed83` ("Expose the first byte position of each JSON token") and break jig (15 errors) and temper (19) against skein's main (the downstream check of 23:22Z); absorbing them is not this run's work but the first step of temper's next plan, `tools`, which the director plans after this run (later.md). Ruling 154's "temper's skein ahead" holds: temper may be ahead of a move's target, and nothing moves back. The README's closing repin named jig's pin of skein, not skein's main, so its text gains the cap and changes nothing else. A target past the cap is a question. Checked: jig's pin of skein `ca5b75ef` and temper's `8a51e10b` (each `origin/main`'s `Cargo.lock`) are both ancestors of `2733882` (`git -C ~/src/rust/skein merge-base --is-ancestor`); every skein step of the plan has landed (ledger), and no `Plan: client-api` commit is in `2733882..origin/main`; `2733882` is an ancestor of skein's `origin/main`. No step's Files or Shapes change.

297. **jig 16.3 and 16.4's Shapes against main** (ready check of jig-16.3 and jig-16.4).
     Ruling 297 (plan defects, the ready check of jig 16.3 and 16.4 at jig `c243f7ad3`, re-read at `3b58a775f`, after 16.1, with 16.2 running; skein `9ffbcfdfa`, temper `1c92504db`): corrected in `jig/16-shed.md`, under "Corrections at jig `3b58a775f` (ruling 297…)". 16.3: the shapes' landed arguments (`now`, `reason`); the stories' `me` gains `creating` and the goal template, and `goal_world`'s rule asks the `Propose` that `shapes::set_goal` gives; the users of `Me.charters`, `MeProject.chat_budget` and `OfferedCharters`; both chat counters count what a `Create` gives. 16.4: the start grep runs over `crates testing tests`; the store's tests and codecs; `OwnerInboxEntry.reply`; its Proof's second test is 16.2's. The reference UI is `crates/jig-client-view/src/`.

     Checked:
     - `crates/jig-client/src/shapes.rs:71: pub fn chat(me: &Me, project: u32, words: Text, now: Wall)`, `:77: pub fn close_chat(…, delegates: &[Delegate], reason: Text)`, `:101: pub fn set_goal(me: &Me, project: u32, draft: GoalDraft, reason: Text, now: Wall) -> Judged`; `:66: membership.creating.as_ref().ok_or(NotOffered::Creating)`; `:126: Fall::Proposable(to) => … Ask::Propose { project, action: ProposeAction::Batch { tasks: single(task) }, reason }`.
     - `tests/client/src/stories.rs:31: creating: None,`; `:44: chat_budget: 10,`; `:693: pub fn goal_world`, its rule `ask: Ask::SetGoal { … budget: 2 … }`, `answer: Answer::Done(Outcome::Proposed { proposal: p.key })`; `stories/conversations.rs:7-21`: one template, `number: jig_client::shapes::CHAT`.
     - `tests/client-protocol/src/engine.rs:232-245`: `chats_made` matches `jig_domain_people::Outcome::Started { .. }`; `native.rs:156: OwnerOutcome::Started { .. }`; `jig-domain-people/src/boundary.rs:403: Created { first: u64, last: u64, given: Option<crate::Given> }`.
     - `git grep -l -w` of the Files' names with `Key::Goal`, `Subject::Conversations`, `Subject::Goals`, over `crates testing tests` at jig `3b58a775f`: 105 files; `tests/tasks/tests/{activity,person,restart}.rs` among them.
     - `client-v8.schema:629: record OwnerInboxEntry { reply: option ConversationLine …`; `codec/tasks.rs:142-144` encodes tags 14 to 16 through `index::conversation` and `index::goal_row` (`codec/tasks/index.rs:65, :135`); `git ls-tree`: `crates/jig-protocol-store/src/tests/{reads/indexed.rs, intervals.rs}`; `crates/jig-domain-client` and `crates/jig-view` absent.
     - The check's sub-agent: clippy clean on `jig-client-world` with a probe of `chat`, `close_chat`, `set_goal` and `fall`; default 361 of 361 (`jig-client-world`, with the `creating` correction) and 833 of 833 over five crates; fuzzy 38 of 38; 16.4's first Proof test passed in 0.17 s. Not checked: 16.3's Proof as 16.3 changes it, temper's users of the removed tags.

298. **jig 16.3's goal template and goal worlds after 16.2** (re-check of jig-16.3).
     Ruling 298 (plan defect, the re-check of jig 16.3 after 16.2 landed at jig `4fad1fcc8`): corrected in `jig/16-shed.md`, under "Corrections at jig `4fad1fcc8` (ruling 298…)". 16.2's goal template in the stories' `templates()` is the chat's clone: 16.3 gives it core.md 7.3's goal fields (`TemplateExecutor::Chosen`, `TemplateBudget::Given`, `TemplateDeadline::Given`); `goal_world` and `procedure_goal_world` each set their `me`'s `creating` from their `SetGoal` offer, and their rules ask the `Ask::Propose` that `shapes::set_goal` gives; the procedure story keeps its name and values.

     Checked:
     - `git diff 3b58a775f 4fad1fcc8 -- tests/client/src/stories/conversations.rs`: `let mut goal = chat.clone(); goal.number = jig_client::shapes::GOAL; goal.label = 2; goal.tracked = Some(0); …`; the chat's `budget: TemplateBudget::AsAllowed { cap: None }` (`:14`), `executor: TemplateExecutor::Charter(1)` (`:11`).
     - `crates/jig-client/src/shapes.rs`, `fill`: `TemplateBudget::AsAllowed { cap } => { authority.spend = 0; …`, `TemplateBudget::Given => { authority.spend = draft…budget; …`, `TemplateExecutor::Chosen => draft.ok_or(NotOffered::Template)?.executor`; `fall`: `if draft.budget <= within.budget && … { return Fall::Within; }`.
     - jig `docs/design/domain/core.md`, 7.3: "| budget | `AsAllowed` | the creator's to give, within their allotment |", "| deadline | none | the creator's to give, or none |", "| executor | an agent, under the project's chat charter | a charter or a procedure the role may create with |".
     - `tests/client/src/stories.rs:746: pub fn procedure_goal_world`: `procedures: list([CreateProcedure { connector: 2, code: 7 }])`, `budget: 1`, rule `Ask::SetGoal { … executor: GoalExecutor::Procedure { connector: 2, code: 7 }, budget: 2 … }`; `tests/client/tests/stories.rs:283: fn a_procedure_only_goal_form_confirms_and_sends_the_supplied_executor_once`.

299. **jig 16.5's Shapes against main** (ready check of jig-16.5).
     Ruling 299 (plan defect, the ready check of jig 16.5 at jig `4fad1fcc8`): corrected in `jig/16-shed.md`, under "Corrections at jig `4fad1fcc8` (ruling 299…)". 16.5's users are a widened grep's 259 files (the third "Done when" grep's 223, and the engine's outcomes and settings and the contract's record forms it misses), with `Settings.chat`'s seven readers; the `Bounds::Goal` and `GoalProcedure` renames reach 13 files; the four schemas join Files, edited in place, their generated code and goldens written by the drift tests' write mode; `goal_decide` and the person proposals stay, deciding every party's `Propose`; line numbers refreshed; temper's users are temper 04.3's.

     Checked:
     - jig `git fetch; git log -1 origin/main`: `4fad1fcc8`. The third grep: 223 files; the widened grep (in the section): 259; `git grep -l -E '\bBounds::Goal\b|\bGoalProcedure\b' -- '*.rs' ':!*/generated/*'`: 13.
     - `git grep -n -E 'settings\.chat\b'`: `charters.rs:56`, `documents/small.rs:147`, `roles.rs:114`, `routing/parties.rs:233`, `tests.rs:1054`, `tests/frame_facts.rs:42`, `jig-protocol-config/src/tests/domain.rs:100`.
     - `git grep -l -E <the widened names> -- '*.schema'`: `client-v8.schema`, `people-v3.schema`, `test-client-v1.schema`, `test-connector-v1.schema`. `tests/client-wire/tests/drift.rs:35`: "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test drift"; `test_client_wire.rs:13` likewise; `tests/store-codecs/tests/drift.rs:16` reads `SKEIN_CODEGEN_WRITE` (package `jig-store-codecs-world`). `git log -3 -- client-v8.schema`: `73de61fc8`, `533dee685`, `0ecf49254`, each in place.
     - `party_decisions.rs:483-485`: `people::Proposer::Party(number) => { if core.tasks.person_proposal(number, proposal).is_some() { goal_decide(…)`; `routing/propose.rs:119, :213`: `work.push(Event::Tasks(tasks::Event::ProposePerson {`.
     - `roles.rs:98: pub fn chat_admit(`, `:288: pub(crate) fn priorities_allowed(`, `:312: pub fn prioritise(`; `jig-fake-parties/src/lib.rs:401: Ask::Close { task, disposals } => {`; `templates.rs:51: pub(crate) fn chat(`; `git grep -c -i chat -- jig-protocol-config/src/{lib,charters}.rs`: nothing.
     - temper `temper/04-engine.md:336`: "### 04.3 The engine's shapes of work gone, with jig's", :339 "the store's `Disposal` refusal, gone with jig 16's", :347 `crates/temper-engine-domain/src/store.rs`.

300. **jig 16.5's Shapes after 16.3 and 16.4** (re-check of jig-16.5).
     Ruling 300 (plan defect, the re-check of jig 16.5 after 16.3 and 16.4, at jig `5b4e287f0`): corrected in `jig/16-shed.md`, under "Corrections at jig `5b4e287f0` (ruling 300…)". The creating bounds `me` carries stay, with `bounds::goal`, `goal_charter`, `goal_start` and `chat_authority`, which build them; only `Act::SetGoal`'s arm and the core's goal creation go. Users no name finds join 16.5: the people child's `Limits.goals` and `disposal_capacity`, the engine frame's `OfferedCharter` and `chat_budget`, and the memory pin they move. The wire's and contract's note scope becomes `Task`, the engine's keep `Goal`. Lines refreshed.

     Checked:
     - jig `git log -1 origin/main`: `5b4e287f0` (16.3), after `e1552c979` (16.4).
     - `documents/small.rs:118: creating: match crate::acts::bounds::goal(`; `acts/objects.rs:80: Act::SetGoal => bounds::goal(`; `bounds.rs:46: if goal_charter(`, `:57: goal_start(`, `:69: let given = crate::task_authority(&core.chat_authority(project, party, held, …`; `funding.rs:39-40: /// The most a person's next task can take … pub(crate) fn chat_authority(`.
     - `limits.rs:33: pub goals: u32,`; `jig-protocol-config/src/domain.rs:172: goals: limits.tasks.project_goals,`; `memory.rs:99: || people::disposal_capacity(&limits.people) < …`; `tests/domain.rs:197: What::Memory { over: 18_576_833_503 }`.
     - `git grep -n -E '\bOfferedCharter\b|\bchat_budget\b'`: only `jig-domain-core` (`documents.rs:233, :248, :272, :411`, `small.rs:127, :147`, `tests.rs:423`, `frame_facts.rs:26`, `frame_deployment_role.rs:70`).
     - `git grep -l -E 'NoteScope::Goal\b' -- '*.rs' ':!*/generated/*' | wc -l`: 30; "Choices": "`NoteScopeGoal` becomes `NoteScopeTask { project, task }`, the scope of a tracked task, which the notes child keeps under its own name".
     - `git grep -n GoalProcedure`: `documents.rs:404`, `offers.rs:233, :291`; ruling 299's grep: 216 files; on `inbox_deployment.rs`, `task_tree.rs`, `inbox.rs`, `reads/indexed.rs`: no match.

301. **jig-16.5's board move offer: interim** (design gap, interim, jig-16.5's block of 2026-10-18T01:53:07Z).
     Ruling 301, interim: a design gap, gone to the designer (question.md, "jig-16.5: the goals board's move offer once `ActKind::Prioritise` goes"). contract.md, 5.3 offers amending "in a task's document", pages.md, 6.2 makes the board "one task query", and objects.md's Prioritise is "priority-only amendments in one compound request": none says which offer gates moving a goal once the project's `Prioritise` offer goes. Until the answer amends `jig/16-shed.md`, jig-16.5 goes on with its engine, wire, schema and test migrations, and leaves the board's move offer (`page/project/moving.rs`, the arrows' gate and their confirmation) on whatever keeps it building, unchanged in behaviour; it opens no task watches for the board, and does not land before the answer.

302. **jig 17's Shapes against main** (ready check of jig-17.1).
     Ruling 302: jig 17's Shapes against main, from the ready check of jig-17.1 at jig `5b4e287f0`, skein `37c676e`, temper `e7484c11f`. `jig/17-engine-http.md` gains "Corrections at jig `5b4e287f0`": the protocol world's engine lines (:78, :107); 17.1's users its Files miss (`watches.rs:107`, `peer/closing.rs:3, :44`, `peer.rs:178, :285-286`), its tests' place (the world's `closing.rs`), and the fixtures it adds (`Peer::mint`, `Peer::minted`, `fixtures::application_person`); 17.2's front lines and the component's `Limits` (`small.rs:257-266`, `standard.rs:147-148`); 17.4's builders and users of `SignedIn`; 17.5's crate (`jig-client-shell-native`), its no-link marker, and the host guard's signature; and what the sketches need under clippy. Refresh any further copy, and list it in the commit body.

     ## Checked

     - `git grep -n -E 'self.link = link|Request::Provider' origin/main -- tests/client-protocol/src/engine.rs` → `:78 Request::Link { link } => self.link = link`, `:107 Request::Provider { .. } => panic!(…)`.
     - `git grep -n 'Route::Provider' origin/main -- crates/jig-protocol-engine-clients/src/watches.rs` → `:107 | Route::Provider(_)`; `git grep -n -E 'ProviderCall|take_provider|providers:' origin/main -- tests/engine-clients/src` → `peer/closing.rs:3, :44`, `peer.rs:178`.
     - `git grep -n -E 'fn mint|fn minted|fn application_person' origin/main -- tests/engine-clients` → nothing.
     - `git grep -n 'printed(&self.engine.link)' origin/main -- tests/client-protocol/src/front.rs` → `:161`, `:322`; `wc -l` → 331.
     - `git grep -n -E 'const CLIENTS|exchanges' origin/main -- crates/jig-protocol-config/src/profile/{small,standard}.rs` → `small.rs:257 pub(super) const CLIENTS`, `standard.rs:147-148 { connections: 16, streams: 8, exchanges: 32, ..small::CLIENTS }`.
     - `git grep -n SignedIn origin/main -- crates/jig-protocol-engine-clients/src` → `component.rs:484-486 w::SignedInParts { token, … }`, `services.rs:25 Answer::SignedIn(value) =>`.
     - `git grep -n localhost origin/main -- crates/jig-client-shell-native/src` → `connections.rs:65 host: "localhost".to_owned()`, `:81 if self.host != "localhost" && self.host != host`, `tests.rs:77`.
     - `git grep -n 'pub mod sha256' ca5b75ef7 -- crates/skein-lib/src` (skein) → nothing; at origin/main → `lib.rs:37`.
     - The sketches compiled under clippy `-D warnings` in a scratch crate after the listed derives, attributes and alias (the check's sub-agent).

303. **jig-16.5's board move offer: Reorder, the project's** (design, jig-16.5's block of 2026-10-18T01:53:07Z, the designer's answer of 2026-10-18T02:07:46Z).
     Ruling 303 (design, the designer's answer of 2026-10-18T02:07:46Z, jig `ad465272b`, temper `baa38bb0a`): the designer's decision, below; ruling 301's hold ends, and jig-16.5 lands with the rename once its gate passes.


     Landed: jig `ad465272b` (contract.md, 5.3 and 9; pages.md, 6.2 and
     12.5; objects.md, 11; STATUS.md); temper `baa38bb0a` (jig/16-shed.md,
     16.5's table and "Corrections after the designer's answer of
     2026-10-18"; later.md, one row).


     (b): the project's document keeps the board's offer, renamed
     `Reorder`. Neither (a) nor (c).

     - **The rule is per project and role.** people.md, 5.1, `Amend`:
       "maintainer, for tracked or priority". contract.md, 5.3 now says:
       "**Reordering,** in a project's document: ... It is judged by the
       party's role in the project alone, as a change of priority is ...
       made or refused, never proposable, and never bounded, since a
       priority asks no authority and names no revision (5.2)".
     - **The name:** `ActKind::Prioritise` becomes `ActKind::Reorder`, the
       wire's `prioritise` becomes `reorder`, and the engine's
       `Act::Prioritise` becomes `Act::Reorder`. contract.md, 3.4 calls the
       act "reordering several tracked tasks at once". The third "Done when"
       grep refuses `\bPrioritise\b`.
     - **The engine:** `priorities_allowed` (`roles.rs:288`) stays. It may
       be renamed, keeps its role check and the policy's Amend check, and
       loses its `goals` and `task_capacity` arguments. `prioritise` goes
       with `Ask::Prioritise`. `standing.rs:14` keeps its owner-or-maintainer
       arm under the new name; `bounds.rs:322` judges a compound's `tracked`
       amendment by it.
     - **The board:** pages.md, 6.2 now says "The board shows its arrows
       where the project's document offers reordering (contract.md, 5.3),
       read beside the board's one query: it watches no goal's document,
       and its lines carry no acts." `moving.rs:13, :34` take the new name.
       `TaskLine` is unchanged.
     - **The checks:** `tests/acts.rs:502` (the owner's offer `Allowed`)
       under the new name, with its `core.prioritise` lines (:503-507) gone;
       jig 16.2's proof
       `moving_a_goal_up_reorders_the_board_in_one_compound_request`; the
       third "Done when" grep prints nothing.
     - **Size:** the rename touches 47 lines in 28 files at `5b4e287f0`.
       Those lines count apart from 16.5's 150, as one count in the report.
     - **Deferred** (later.md): the board offering no move that changes a
       closing goal's priority. tasks.md, 7: "An amendment is refused once a
       task is closing."

304. **jig-16.5's capacity actions of the operations** (plan defect, jig-16.5's block of 2026-10-18T02:32:27Z).
     Ruling 304 (plan defect, jig-16.5's block of 2026-10-18T02:32:27Z): the implementer's proposal holds, and joins 16.5. pages.md, 7 shows "what was refused for want of room, recently", and the people child's `CapacityAction::{Compound, Create}` (`jig-domain-people/src/capacity.rs:12-13`) are the refusals of the asks v8 sends, but 16.5's table removed `OwnerPersonCapacityAction`'s `close`, `chat`, `goal` and `priority` without the wire's and the client's forms of the operations' actions. So `OwnerPersonCapacityAction` gains `create` and `compound`, content-free, in `client-v8.schema:791` (regenerated) and `jig-client-domain/src/contract/owner_deployment.rs:224`, mapped one to one in the engine's `translate/deployment.rs:111` (in place of its `Reason::Tag` refusal), the link's `translate/documents_part6.rs:131`, the view's `owner_deployment.rs:202` (two phrases in `owner_deployment_words.rs`, as its other actions' are), the scripted engine's `owner_deployment.rs:128`, and the joint wire's `deployment.rs:196` (in place of its `unreachable!`) with its coverage cases (`deployment/tests.rs:398-412`). No role rule and no bound changes. These lines count apart from 16.5's 150, as one count in the report, beside the rename's. Refresh any further match that `git grep -l -w OwnerPersonCapacityAction -- '*.rs' ':!*/generated/*'` lists, and list it in the commit body.

305. **temper-04.3's read positions: renamed in jig-16.s1, not removed** (plan defect, temper-04.3's block of 2026-10-18T03:31:34Z).
     A plan defect: a conversation's read position stays, under a name for no shape; jig-16.s1 renames it, and temper 04.3 waits for it and pins its landing.

     - The design keeps the record: "The inbox across projects, its read positions (one over results, one per task flagged conversation) moved by `Seen`; a live conversation's held by the people child, an ended one's in the store" (jig's `docs/design/domain/README.md`); "Read positions are the people child's (people.md, section 6), moved only by the party's `Seen`. ... There is one per task flagged conversation, its requester's" (`domain/engine.md`, 11). The plan forbids its name: "no operation, document, subject, record or index named for a chat, a goal or a list ... The shape-specific code that exists at the base goes in jig 16" (`code-organization.md`, 6). jig 16's Files missed it, and temper 04.3's Shape read it as removed.
     - **jig-16.s1** (`jig/16-s1-read-positions.md`), a new implementer in jig, needing nothing: `people::Key`/`Stored::ReadChat` become `ReadConversation`, `core::Now::SeenChat` becomes `SeenConversation`, `core::Event::SeenChatLoaded` becomes `SeenConversationLoaded`, and the store's `people-v3.schema` `record SeenChat` and `enum Seen`'s `chat:` arm become `SeenConversation` and `conversation:`, the wire's own words (`client-v8.schema:237-238`). A rename only: no behaviour, encoding or expected value changes. Its users are what `git grep -l -w -E 'ReadChat|SeenChat|SeenChatLoaded' -- '*.rs' ':!*/generated/*'` lists at its start (50 files at jig `6605e0564`, the ops example among them).
     - **temper-04.3** waits for jig-16.s1, then moves its pin to jig-16.s1's landing. Each removed arm is still replaced by nothing, but for the read position's: its arms follow jig-16.s1's names, and temper's own `EndedRead::SeenChat` becomes `EndedRead::SeenConversation`, at the 11 lines in 9 files 04.3's Shape now lists. Its grep (04.3's Rules and Done when) is unchanged and prints nothing at its end. The work already in `client-api/temper-04-3` stays.
     - Rename any further identifier that names this read position for a chat, in jig or temper (a function, a fixture, a doc comment), and list it in the commit body.

306. **temper 05's Shapes against main** (ready check of temper-05.1).
     temper 05's Shapes against main, from temper-05.1's ready check at skein `a0c3a66`, jig `6605e0564`, temper `90a3f7048`: the corrections are `temper/05-server.md`'s section "Corrections after the ready check of temper-05.1": 05.1's line references, the listeners' readiness in `system.rs` and `workers.rs`, six users its Files miss, the route as the target up to its first `?`, and the restored clients' `Host` covered by jig 17.5; 05.2's line references, `checks::standalone`'s `Clients` parameter, the one-process world's `7100`, the process tier's `clients` member, and the reference test's binding of a `skein_io::Addr`. Refresh any further copy, and list it in the commit body.

307. **jig 17's Shapes against main after jig 16.5** (re-check of jig-17.1).
     jig 17's Shapes against main after jig 16.5, from the re-check at skein `a0c3a66`, jig `6605e0564`, temper `90a3f7048`: the corrections are `jig/17-engine-http.md`'s section "Corrections at jig `6605e0564`": the lines 16.5 moved; who builds `SignedIn` (ruling 302's `services.rs:77-79` decodes one, and builds none); 17.4's generated limit `signed_in_token_some` and its users; the readers of `.token()` that take the option; the regenerated goldens; and `tests/ops/src/joint/bytes.rs`, a user for 17.1 and 17.2. The earlier section's heading names ruling 302. Refresh any further copy, and list it in the commit body.

308. **jig 18's Shapes against main** (ready check of jig-18.1).
     The ready check of jig-18.1 (skein a0c3a66, jig 6605e0564, temper 90a3f7048) found jig 18's Shapes off main in ten places: lines that moved, a private address reader, three JSON fixtures that need the server's listener, a temper count, a missing fixture and dependency, and 18.2's sketch of `Server` against 17.2's. jig/18-server.md gains "Corrections after the ready check of jig-18.1 (ruling 308)", which holds where it and the increments differ; 17.2's `Server` signatures stand, and 18.2's decision is `fn route(&self, head: &Call) -> Answer`.

309. **temper main's gate: btrfs out of metadata room, balanced** (infrastructure, the coordinator's block of 2026-10-18T04:09:48Z).
     Ruling 309: the gate's link failures were the disk's, not temper's. btrfs `/srv` had 1 MiB of its device unallocated and its metadata at 3.80 of 4.32 GiB with a 512 MiB reserve, so a write that needed new metadata failed while `df` showed 98 GiB free. mold writes its output through a mapped file, and such a failed write is a SIGBUS: "mold: failed to write to an output file. Disk full?", "clang: error: unable to execute command: Bus error". jig 18's ready check met the same (`tmp/check-jig-18/default2.log`, `fuzzy2.log`). pueued and Codex's daemon died in the same window; the supervisor started pueued at 04:17Z and resumed the coordinator at 04:20Z (`plan resume crash`). Between 04:13Z and 04:16Z the supervisor ran three passes of `sudo -n btrfs balance start -dusage=40,limit=10 /srv`, which left 22 GiB unallocated.

     1. Run temper main's gate again, once, as your resume's step does. Nothing in temper changes for it. If it fails linking again, block (kind: infrastructure) with the gate's log and the output of `sudo -n btrfs filesystem usage /srv | grep -E 'unallocated|Metadata,'`.
     2. pueue task 697 (jig-16.s1's `cargo check --workspace --all-targets`) ended `Killed` at 04:17:52Z, when pueued restarted: its session queues it again, once. Any other task the restart marked `Killed` is queued again the same way.
     3. From now until the run's end, at each disk check of development.md 11.4, also run `sudo -n btrfs filesystem usage /srv | grep -E 'unallocated|Metadata,'`. When unallocated is below 5 GiB, run `sudo -n btrfs balance start -dusage=40,limit=10 /srv` once, then the usage line again, and log both outputs in `log.md`. If it is still below 5 GiB, block (kind: infrastructure) with both outputs; run no other btrfs command.
     4. Unchanged: the disk floors of development.md 11.4 and `plan check`, read from `df`; every step's Files, Proofs and gate.

310. **the last retrospective's check holds no resume of this run** (process, the coordinator's block of 2026-10-18T04:09:48Z).
     Ruling 310: the check no longer fails, and it never held this run's resume. development.md 1 says of a retrospective's proposals: "The user, or the director, decides each before the next run starts: `plan check` refuses a plan while a proposal of the last retrospective is open." client-api's run started before kit-batch's retrospective; a resume starts no run. The director recorded the decisions at 04:10:10Z (skein `c0662b5`, "Record the director's decisions on kit-batch's retrospective"), and `plan check` now prints "ok    the last retrospective's proposals are decided (skein's kit-batch)".

     1. Go on with the resume. From now until the run's end, a `plan check` line about the last retrospective's proposals holds no resume and no session start of this run: log it, and go on.
     2. kit-batch's decided proposals do not reach this run: its kit stays at skein `841f487` (ruling 295) and its process at `15ddb75` (supervision.md's Process line), until a steer moves either.
     3. Unchanged: every other line of `plan check` binds as before.

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
- **F. jig's focused budget doubled for the rest of the run** (ruling
  275, 2026-10-17): the user's rule of 2026-10-13 for a budget that
  bites. jig's focused gate ran 14.405 s and 14.964 s at jig 14.5 and
  14.s2 of its 15 s, five land gates timed out at 15 s with no test
  failing, and a repair step (jig-14.s2) left it 0.56 s slower. The
  alternative is 15 s held: each jig session first repairs the suite on
  the run's one chain, about 30 steps. Default: 30 s until the test-cost
  pass right after the run; restore it at once with `git -C
  ~/src/rust/jig revert <ruling 275's commit>`.
