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
