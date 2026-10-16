# forge-setup: decisions

The planner's decisions, each with its reason. Rulings and answers of the
run are appended after "For the user".

1. **The box first, on polling; webhooks join it.** Sessions are numbered
   in setup.md 14's order and start by availability. The box needs
   prepare, run and reconcile (temper 07 to 10, 13), and none of them
   needs webhooks, while the webhooks' listener waits for client-api's
   temper 05.1 (temper-service's `server.rs`, about 10-18 13Z) and
   temper's skein move. So `temper box` and the process tier on an owned
   Forgejo with CI (temper 10.2) land on polling, and webhooks reach them
   in temper 11. The throwaway Forgejo's hook (setup.md 14, item 3)
   serves the conformance's delivery facts (temper 04.1) alone: by the
   time the engine has a listener, the process tier runs on owned.
2. **temper's skein moves once, in temper 05.1,** after skein 01.2 and
   after the director's client-api ruling lands (temper's skein may be
   ahead of client-api's floor; its repins check `git merge-base
   --is-ancestor`; none moves skein back). The director chose this on
   2026-10-16; the ruling is client-api's ruling 154 (`c36858db7`), on
   `main` at the plan's base, so 05.1 waits for skein 01.2 and temper
   04.2 alone.
3. **No repin but temper's.** skein 01 adds `skein_lib::sha256` and moves
   three private copies onto it, keeping every public item. smith is
   left alone (smith/wall-clock runs on hetzner); jig's pins are
   client-api's to move; hearth's paused M1 plan takes skein at its next
   repin, and its SHA-256 step is this plan's skein 01.
4. **One pin, in temper-forge-forgejo** (setup.md 6.1; protocol.md 5):
   `VERSION`, `RUNNER_VERSION` and their checks move there from
   `testing/temper-throwaway-forgejo/src/binary.rs` (temper 03.1); the
   throwaway re-exports them. forgejo-17's rule that only the test crate
   names a version ends with this move.
5. **The shipped profile's new limits** (configuration.md 7 names them
   and gives no value): `setup.orgs` 8, `setup.deadline_ms` 120000,
   `setup.backoff_ms` 100, `setup.backoff_max_ms` 5000; `box.backoff_ms`
   500, `box.backoff_max_ms` 30000, `box.restarts` 5, `box.window_ms`
   300000, `box.grace_ms` 10000; `forge_component.hooks` 4,
   `hook_head_bytes` 8192, `hook_body_bytes` 1048576 (a push of many
   commits; the body streams, so it costs no memory), `hook_deadline_ms`
   10000, `hook_secret_bytes` 128 (reconcile draws 32 bytes, 64
   hexadecimal digits); `forge.client.settle_ms` 30000; the profile's
   sockets 48, so that `small` holds a document with both listeners
   (designer's answer 30). Each satisfies configuration.md 7's
   cross-checks. A test that needs another value
   names it beside the profile's (development.md 5.6).
6. **How the setup is checked** (the designer's answer of 2026-10-16;
   setup.md 8): temper-service hands `application.setup` and the
   `setup.`/`box.` overrides on undecoded, with the forges' and
   identities' members; the binary calls temper-forge-setup's and
   temper-box's checks for every command, `temper serve` included.
   `temper forge …`, `temper box` and `temper sign-in` read no private
   file to check.
7. **The lock and the exec** (the designer's answer; setup.md 6.4): one
   module of temper-forge-setup-shell, std's `File::try_lock` and
   `CommandExt::exec`, the lock handed to the exec'd process as its
   stdin. The module holds the lock and the exec alone; the
   preparation's commands run on skein-io's processes in the host's
   loop (the designer's answer 32). The boundary test that keeps
   `std::process` and file locks in that module lands with the module's
   first use (temper 08.1).
8. **The `state` file** is a skein-json document with a `version` member
   (1), the versions prepared, whether the first preparation ended, and
   Forgejo's secrets. Pre-release: a reader refuses another version.
9. **The box finds `temper`** by `std::env::current_exe()`, and gives
   each step the configuration's path made absolute against its own
   working directory.
10. **The upgrade comes last** (temper 15.1): a box at the pinned
    versions never meets it.
11. **Facts are named by group:** `setup-…` (admin routes, tokens, orgs,
    teams, hooks), `cli-…` (Forgejo's command line), `runner-…`,
    `delivery-…`, so "Done when" counts each group.
12. **The process tier aims at 180 s whole** (testing.md 2.6;
    setup.md 13.4), CI, webhooks, the box and the upgrade included. If it
    goes past, the slowest story becomes a test of its own, or the aim
    moves, by a ruling for the user.
13. **The suites' aims for the whole run:** at the base (temper
    0ff45022, x1) the default suite takes 4.43 s and the fuzzy one
    24.22 s (medians of three, `findings/measurements.md`); the run adds
    at most 1.0 s to the first and 3.0 s to the second. No session has a
    share. skein's suites take what skein 01 adds, at most 0.5 s to its
    default suite.
14. **The owned example is a file,** `docs/examples/owned.json`, the
    whole document, with its non-secret files in `docs/examples/owned/`
    (the designer's answer 12), which the guide and the tests read; the
    guide is `docs/guide/box.md`.
15. **A hint's key is not used to skip a read.** setup.md 10 has each
    echo cost the keep one read, which the keep does today
    (`keep.rs:638-641`); the domain drops what it already knows after
    that read.
16. **No live service.** The Forgejo and runner are the pinned local
    binaries; the process tier's LLM is skein's fake peer; no agent
    signs in. The guide's `temper sign-in` is the person's, and its
    check stops before it.

17. **Session 07 is split, and the upgrade is a session of its own.**
    The drafts gave temper 07 589 lines, past development.md 5.2's 400;
    `forges[].user` as a file (drafted as 10.1) must come before the
    first command, since setup.md 8 requires it for a set-up forge. So:
    temper 07 is the crate, Forgejo's files, the configuration (07.3:
    `forges[].user` as a file, `application.setup`) and the preparation's
    machine (07.4, which waits for temper 03.3, the configuration filling
    that wait); temper 08 is `temper forge prepare`, `run` and the
    process tier's Forgejo; temper 15 is the upgrade, which nothing waits
    on.
18. **temper 06 waits for temper 04.1,** so that the fake's hooks follow
    the order and the mergeability the conformance recorded, not an
    expectation (forge.md 9: the fake answers what the facts record).
19. **The fake's deliveries are not sent to the hook reconcile made in a
    world:** temper 11.1 proves Forgejo's delivery to reconcile's org
    hook at the process layer, and the fake's world tests the listener
    with the hook its scenario names.
20. **The process tier's person** gets a token made on the admin's routes
    (designer's answer 14), as the bot does; nothing signs in with a
    password through the API.
21. **No live service, in the example either:** temper 14's test and
    its guide's script hand in an LLM account for skein's fake peer, as
    the process tier does; the guide names the borrowed-login way for
    people (designer's answer 13).
22. **The increments whose Size is the process tier's support** (temper
    08.3, 10.2, 11.1, 11.2, 13.2, 14.1) count that support's lines
    (`code-organization.md`, 6), since the plan's count leaves `tests`
    aside.

23. **temper 10 starts after temper 08.3,** so that its proofs read the
    owned Forgejo through the throwaway's `attach` (08.3), not through a
    second HTTP helper of their own.
24. **`box.` overrides are refused as unknown** until temper 13.1 wires
    temper-box's checks into the binary's; no document of the plan
    names one before.
25. **The private org's CI clones with the job's token:** the runner is
    offline, so a workflow runs `git` itself, not `actions/checkout`;
    temper 03.3 records the fact that a job's token clones its private
    repository.
26. **The protocol world's late, reordered and lost deliveries wait**
    (forge.md 9, 10; `later.md`): the fake sends each delivery on time;
    the connector world's hinted stories (temper 06) and the process
    tier's hinted story (temper 11.2) carry hints with polling as the
    fallback.
27. **The base's lint idioms bind the sketches** (`findings/base-runs.md`):
    `#[expect(clippy::large_enum_variant, reason = …)]` on an enum that
    carries a `Document`, never a `Box`; no `Default` derive in step code
    (`User`'s literals name its two new fields).

## Rulings

101. **04.1's Size at 300** (plan defect, temper 04's block of 2026-10-16T22:10:01Z).
     Ruling 101, a plan defect: 04.1's Size of 250 under-counted the scenario the session file itself prescribes
     (the listener, eleven steps, two probes, 21 facts and their roll-up, "Facts, 21, each named `delivery-` …" in 04.1's
     Shape). 04.1's Size is now 300, the plan's largest (development.md, 5.2: "Every Size is at most about 300 lines"),
     so its tolerance is 600 by implementer.md's 8.6 ("An increment lands within twice its Size without asking").

     - 04.1 lands at its count of 523, plus what the recording's proof needs fixed, while the count by
       code-organization.md 6's command stays at most 600.
     - Past 600, stop on a question again, with the count by file.
     - Unchanged: 04.2's Size (220, codec); the rule against trimming a doc comment, a test or the formatting to meet a
       Size (implementer.md, 8.6); the proof, the recording whole, before landing.

102. **07.1's Size at 300; sizes within 2.6 times** (plan defect, temper 07's block of 2026-10-16T22:11:00Z).
     Ruling 102, a plan defect: 07.1's Size of 280 under-counted its own Shape (sketches 1-3, `problem::{Problems,
     lines}`, `limits::{apply, check}`, `check::members`' refusals of setup.md 8, `Step` and `steps`). Accept, do not split:
     07.1 is written, its 17 tests pass, and 07.2-07.4, temper 08, 09 and 12 wait on it.

     - 07.1's Size is now 300, the plan's largest (development.md, 5.2: "Every Size is at most about 300 lines").
     - The same block twice (04.1, ruling 101; 07.1) becomes a standing rule, in the README's "Rules of this plan":
       "An increment of this plan lands within 2.6 times its Size without asking, a Size marked "(codec)" within 3 times;
       its report gives the count by code-organization.md 6's command, by file. Past that, it stops on a question, as
       implementer.md 8.6 says." By it, 07.1 lands at its 724 and what its review needs, up to 780.
     - Unchanged: never trim a doc comment, a test or the formatting to meet a Size (implementer.md, 8.6); a split
       increment's parts count together; 07.2-07.4's Sizes.
     - For the retrospective: with Sizes at most about 300 and increments landing at about twice them (development.md,
       5.2), a tolerance of twice stops about half of them; this plan's two size blocks came in its first 11 minutes.

103. **04.1's deletion by forge.md 5; cancelled, interim** (design gap and plan question, temper 04's block of 2026-10-16T22:13:42Z).
     Ruling 103, interim on question 1, final on question 3 of temper/04-deliveries.md, "Questions to raise when
     reached".

     - **Question 3, the branch deletion: a plan question, answered by the design.** forge.md 5's table, row `push`:
       "of a branch, `Change::BranchMoved { branch, head, actor }`, … ; of a branch deleted, `Change::Branch(branch)`".
       So, as question 3 says, 04.2's document gains `Delivery::deleted()` (a push whose `after` is all zero), and the
       deleted branch's hint comes from that push. 05.2's Shape already reads it so (05-webhooks.md:168-169: "`Push`,
       `BranchMoved { … }`, or `Branch(branch)` when `after` is all zero"). `branch-deletion-sends-no-push` failing is
       allowed: 04's "Done when" counts "2, or 3 with branch-deletion-sends-no-push".
     - **Question 1, the cancelled job's delivery: a design gap, sent to the designer** (`question.md`, 22:14:42Z). Until
       the answer, go on as question 1 says: "A delivery not had within the Size is left unrecorded (no fact;
       `SUBSCRIBED` unchanged; the coverage test names it)". No web driver. The answer may change 04.2's coverage test
       and `SUBSCRIBED`; it reaches you as a later ruling.
     - **Question 2, the pull-ordering pair:** go on with your investigation. The recording rule holds (README, "The
       recording tools": once more after each fix aimed at a cause the output names; the third failure with the same
       symptom is a question). If both facts of the pair still fail then, block with session 04's question 2.
     - Unchanged: 04.1 lands only with `holds webhook-deliveries` and 04's "Done when" counts.

104. **04.1's cancelled job end, by the API's cancel** (designer's answer, temper 04's block of 2026-10-16T22:13:42Z).
     Ruling 104: the designer's answer, landed in temper fbe1ceaed, replaces ruling 103's interim on question 1:

     temper 04.1, the cancelled job's delivery: **record it through the API, with no web driver.** Landed in temper
     fbe1ceaed (forge.md 5 and 13; plans/forge-setup/temper/04-deliveries.md).

     The premise that no API route cancels a run does not hold. Forgejo v17's API description
     (`tmp/forgejo-17-swagger.json:6556`, `CancelActionRun`) says: `POST /repos/{owner}/{repo}/actions/runs/{run_id}/cancel`,
     "Pending or running jobs of the run are cancelled. A run that has already finished ... is left unchanged. In both cases
     the endpoint responds with HTTP 204." forge.md 13 now lists it: "A run is cancelled, its waiting and running jobs with
     it, by `POST actions/runs/{run_id}/cancel`: `204`, ...; temper never cancels one."

     - **04.1, step 10:** `deliveries.yaml` gains job `held`, `runs-on: temper-held`, which no runner serves; once
       `workflow_job_success`, `_failure` and `_skipped` are had, O cancels the run (`POST R/actions/runs/{job.run_id}/cancel`,
       `204`): `workflow_job_cancelled` (`held`).
     - **Facts, 22:** `job-ends-row` leaves the cancelled end aside; a new fact, `cancelled-job-end-is-delivered` (may fail),
       checks it as `job-ends-row` checks the others. "Done when" counts 22, and lets that fact fail.
     - **04.2:** `SUBSCRIBED` unchanged. `every_subscribed_event_has_a_recorded_delivery_the_decoder_knows` names
       `workflow_job_cancelled` as the one unrecorded event only while `delivery-cancelled-job-end-is-delivered` fails.
     - **If the cancel delivers nothing,** forge.md 5 now rules: "An event the tool records no delivery of stays as the
       sources name it, its fields those of the events whose payload it shares, and the tool names it unrecorded: a hint only
       brings a read sooner (`domain/forge.md`, 5.2), so a row that is wrong costs a poll's wait."
     - forge.md 13's to-check list gains "a job's cancelled end, from cancelling the run of a job no runner takes".
     - 05 (the component's hint for a job's end) is unchanged. Nothing is deferred: no `later.md` row.

105. **06's Shapes against main** (ready check of temper-06.1).
     Ruling 105, the ready check of temper-06.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/06-settle.md: `translate::hint` serves the forge world and the conformance world, the client world keeps
     ignoring hooks (`client/src/world.rs:125,467,645`); Files add fake `tests/next.rs`, whose `:32,33` read the
     harness's hooks by tuple position; `Config` has six literals, named, not five; the updated tests add
     `observations_and_webhooks_follow_branches_pull_requests_and_the_wiki` (`tests.rs:1720`) and the totals at
     `tests.rs:1431-1433`; `wiki.rs:45,71` call `Hook::wiki` now and push the observation alone once `Wiki` goes;
     the helpers moved to `changes.rs` add `change_step_with_release` (`simulation.rs:324`), `change_step_task` is
     `pub`, and `protocol.rs:63-66` keeps its copy. Sketches A to E compiled under clippy; no suite was reached. A further
     user of a reshaped `Hook` or `Change` found while working is updated, and named in the commit body.

106. **12's Shapes against main** (ready check of temper-12.1).
     Ruling 106, the ready check of temper-12.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/12-box-machine.md: jig-boundary is at the rev `Cargo.lock` locks (755e02e then, not 2900e9b); 12.2's
     `io_limits` gives `skein_io::Limits`'s five other fields (`sends` 1, `accepts` 1, `backlog` 1, `close_timeout` 5 s,
     `retry` 10 ms, as the terminal's `limits.rs:90-94`); the harness's root is `Sim::root(Handle::new(opened.raw()))`,
     `Sim::now` is `sim.rs:191`, and each ask goes to `skein_fake_machine::step` alone, never `serve`, the spawn's
     environment recorded from its `Ask::Spawn` first (`step` drops it, `face.rs:19`). The host in the `no_std` crate
     under the subset's lints is the design's: setup.md 596-598, "its host (ordinary Rust) on skein-io's processes; …
     one crate, under the subset's lints, its host written within the subset". Sketches 1-3 and the boundary test
     compiled under clippy; the boundary test passed, 1 of 1. A further stale pin or limit found while working is
     refreshed, and named in the commit body.

## For the user

- 1: the box lands on polling before webhooks reach it.
- 5: the limits' values.
- 10: the upgrade after the box.
- 14: the example's and the guide's places.
- 15: echoes cost a read each; no skip by key.
- `sha2` stays in temper-engine-domain-forge-issues (`later.md`).
- 17: the upgrade in a session of its own, last.
- 12: the process tier's 180 s, and a ruling if it goes past.
- 19: no fake-world story of deliveries to reconcile's hook (`later.md`).
- 26: the protocol world's late, reordered and lost deliveries wait
  (`later.md`).
- 20: the process tier's support reads `admin_token` (to attach and to
  make the person's token), which setup.md 9 reserves to reconcile in
  production.

## Answers received

- **The director,** 2026-10-16: temper's skein moves in temper 05.1
  once client-api's ruling lands (decision 2; ruling 154); temper builds
  on skein's `main` at the base (`findings/skein-main-in-temper.md`).
- **The designer,** 2026-10-16, landed in temper `326cbded`, `0d67b373`,
  `50c7dd0d`, `91ad7ef1`, `b05af1f7` and skein `121daa43`, `04c1562e`: 45 answers,
  each now in the design, which the session files cite. Among them: the
  lock and the exec by std, alone in one module (setup.md 6.4); the
  admin token's fresh name and scopes, a cut-short first preparation
  begun again from an empty layout, a later preparation taking Forgejo's
  lock alone (6.3); the bot's tokens on the admin's routes, its marker
  in its full name, private orgs (5); reconcile on plaintext loopback
  alone (5, 15); settle's window as its own wait, in the change's
  procedure (domain/forge.md 5.2); the box's signals, text and restarts
  (7.2; configuration.md 7); the listener's answers (forge.md 5); an
  absent option written `null`, never missing (configuration.md 3, 10);
  SHA-256's sweep in the fuzzy suite (lib.md 10).
