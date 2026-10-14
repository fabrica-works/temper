# Retrospective: protocol

2026-10-14. The run of temper's `protocol` plan on x1, claimed at
2026-10-14T03:22:17Z and done at 21:50:16Z (`done.md`; released at
21:51:06Z). Written by a fresh session that took no part in the run
(development.md, section 15).

Ten read-only sub-agents, one per source, read:
- the plan, with `decisions.md`, `later.md` and `findings/`;
- the ledger, `log.md`, `context.md`, `done.md` and the sessions'
  checkpoints;
- `supervision.md`, with the escalations and their answers;
- temper's git history, with its trailers;
- `logs/heavy.tsv` and the run's `logs/`;
- the coordinator's Codex rollout;
- the 82 implementer rollouts;
- the Claude Code transcripts of the three supervisors, the eight
  designer sessions and the session that designed and planned the run;
- the earlier retrospectives (tls-cleanup, jig-protocol, test-budgets,
  reliability, client-composition), their metrics, and the history of
  the process documents and the kit;
- temper's code on `main`, for its debts.

This session read development.md as x1 had it and as skein's
`origin/main` has it, with `implementer.md` and `supervisor.md`, the
plan's README and `done.md`. It ran the "Done when" commands that need no
build, and `plan metrics`. Times are UTC. This document proposes; the
user decides, and the designer applies what is adopted.

**Which process.** The run followed development.md and the kit as of
skein `9539e00` (02:50Z), which x1's checkout kept to the end. This
retrospective follows section 15 as skein's `origin/main` (`5171bbf`)
writes it now, with its section 10. Its `metrics.tsv` is `plan metrics`
from `origin/main`'s kit, whose definitions changed during the run
(section 3).

## 1. In one page

- **Done in 18.5 hours, 16.9 of them running, and every "Done when" item
  holds;** item 12 only as ruling 164 reads it.
  - 80 of the 81 planned increments landed, as 110 steps: 61 whole, and
    19 split by their own sessions into 49 parts. 04.4b was dropped by
    the README's own condition.
  - Five steps were added: 12.2a and 15.3a by steer, 17.s1 to 17.s3 by
    rulings. 115 steps landed in all.
  - The suites ended at 1,265 tests in 4.05 s and 46 in 15.3 s, against
    budgets of 15 and 60. Each of the 11 gate failures was a real bug:
    no flake, no timeout.
  - Every increment proved at the layer its session file named. The
    process tier passed its six tests in 43 s. The live tier passed on
    its second run, 7 completions in 41 s, 12 of the plan's 50 in all.
  - The web crates and the Python are gone, and the pins held.
- **No proposal of the last retrospectives reached this run.**
  - test-budgets' retrospective landed at 03:13Z, nine minutes before the
    claim, with its 14 proposals open.
  - Their adoption (skein `479f836` at 08:16Z to `63fff2d` at 09:24Z),
    and reliability's and client-composition's after it, went to skein's
    `origin/main` only. x1's checkout stayed at `9539e00`: every role
    read the old text and ran the old kit.
  - This run is the last measure of the old process. Several of its
    faults are answered on `main` already (section 7).
- **The first 8.6 hours made 83 code landings; the last 9.8 made 33.**
  - From 13:00Z the process tier joined the service's halves for the
    first time. It found 8 faults in production code that no lower layer
    had met (rulings 136, 140, 144, 146, 150, 153, 157, and 12.2a's), and
    about as many in tests and fixtures.
  - Three of the eight, and two of the fixture faults, came from limits
    that the worlds and fixtures set by hand, apart from the shipped
    profile.
  - Implementers diagnosed them by running the proof again: 99 process
    and live runs for 12 steps, 3,711 s alone on the machine, and nine
    rulings, each a budget of runs for one step.
  - 15.3, 17.3 and 17.4 took 235M input tokens, 27% of the implementers',
    for 3 planned steps and 4 added.
- **Tokens: 13.3M a landed step,** counted once. client-composition and
  test-budgets used 8.8M and 8.7M; jig-protocol and reliability 21.5M
  and 27.6M.
  - **Implementers, 7.6M a step.** Threads reached their first change at
    105K and grew 50K an increment, not section 5.3's 30K. 82 threads
    served 18 sessions, 1.4 steps each.
  - **The coordinator, 2.4M.** Its median request carried 199K, since it
    compacted only at 349K to 395K. A tenth of its input went to
    code-graph queries that section 2.3 does not allow.
  - **Claude Code, 3.4M.** The designer was forked seven times, to 646K,
    and 82% of its input was copied history. A supervisor ran five hours
    past its own handover note.
  - **The planning,** 1,050M input tokens before the claim, is counted
    nowhere.
- **The channel was fast, and plan defects did not fall.**
  - 72 blocks, answered in a median of 2.2 minutes; 7 design questions,
    in 2 to 21 minutes. Five answers reversed, 0.04 a step, the fewest so
    far.
  - 38 plan defects, 0.33 a step, as in every run of the old process
    (0.28 to 0.38). About 21 were the kind section 5.4's checks find, and
    6 were Sizes set below what every earlier run had landed.
- **The coordinator's record decayed.**
  - Its prose lost its spaces from 04:16Z, its markers their shape from
    09:50Z, and `context.md` ended with a line of 17,255 characters. Four
    metrics broke on it.
  - Rings and `plan resume` reach it as the user's words, so it logged
    the reboot as "the user's request" and the end as "the user's
    explicit instruction".
- **An apt upgrade rebooted x1 at 05:26Z.** The run stood still for 98
  minutes, until the user came back, and `plan resume` could not load the
  coordinator's thread (ruling 115).
- **Fourteen proposals** (section 8), by expected effect. The first three
  move the tail and the threads: a failed proof is diagnosed below its
  layer, increments are sized from the record, and every test takes its
  limits from the shipped profile. The rest bound the coordinator's
  context and bookkeeping, survive a reboot, and make the metrics true.

## 2. Outcome

The plan's "Done when", on temper's `origin/main` at `0ffa7b13`, whose
code is `b1994cdf`'s, the last code step. This session ran items 2 to 7,
9, 10 and 12 again. Items 1, 8 and 11 are the record's: session 17's last
thread ran them in `wt/temper-done` at `b1994cdf`
(`logs/temper-17/done-report.md`).

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 115 landed, 22 dropped. 21 of the 22 are split parents whose parts landed (section 3). One, 04.4b, was dropped by the README's condition: 02.4 took the `Application` family's codec. |
| 2 | The gate passes within budgets | Holds: `b1994cdf`, `scripts/check.sh passed: 1265 tests / 4.050 s; 46 tests / 15.276 s; 53 s in all`. The highest of the 116 code landings were 5.665 s and 19.914 s (14.2, `ae79f386`). |
| 3 | The crates and worlds exist | Holds: the loop prints nothing. |
| 4 | The web crates are gone | Holds, since 11.4 (`27197d49`). |
| 5 | No Python | Holds. |
| 6 | The pins held | Holds: jig `d91a89c4`, smith `dfc0fe57`, skein `b5d3a2c3`. |
| 7 | The engine worlds on jig's real store | Holds. |
| 8 | The process tier passes | Holds by the record: six tests in 43.0 s (`logs/20261014T214342-0.log`). |
| 9 | The live tier within its budget | Holds: `Completions: 7`, `Wall: 41 s`. The plan's first live run used 5 completions and 849 s, and failed on the model's words. Its numbers are only in the commit body's prose, which the item does not read (section 5, item 9). |
| 10 | Forgejo's facts recorded | Holds: the four `fails` lines are ruling 103's. |
| 11 | The new worlds' share | Holds by the record: 2.0 s focused and 11.2 s fuzzy, against 4.0 and 25.0. |
| 12 | The code rules | Holds as ruling 164 reads it. The variables and the threads hold as written. The lock's new names include 16 that the item's list lacks: transitive dependencies of `serde_json`, `serde`, `rustls` and smith's `smith-local-shell`. No temper manifest names any of them. Run at the base, as decision 24 did, the command diffs the base with itself and prints nothing, so it could not find this (section 5, item 4). |
| 13 | Only these aims | Holds. |

## 3. Numbers

`metrics.tsv` comes from `origin/main`'s kit, which counts each Claude
Code message once across sessions, each escalation once, and the suites
by repository. client-composition was counted the same way. For the
other runs, the table takes the values their retrospectives corrected by
hand where they did. Per-step values divide by the steps landed: 115
here, 36 in client-composition, 39 in test-budgets, 217 in jig-protocol
and 108 in reliability.

| Metric | protocol | client-composition | test-budgets | jig-protocol | reliability | Notes |
|---|---|---|---|---|---|---|
| `wall_h`, `running_h` | 18.5, 16.9 | 4.3, 4.3 | 3.7, 3.7 | 29.8, 29.5 | 36.0, 35.8 | The reboot took 1.6 h (05:25Z to 07:02Z). |
| steps landed | 115: 110 planned, 5 added | 36 | 39 | 217 | 108 | The metric prints 112, 3 and 22 dropped (below). |
| `step_h` | 0.15 | 0.12 | 0.09 | 0.14 | 0.33 | 0.085 to 12:00Z; 0.30 after. |
| escalations a step | 0.63 (72) | 0.22 | 0.54 | 0.97 | 1.71 | Plan defect 38, cross-session scope 11, design gap 9, process 9, infrastructure 5. |
| plan defects a step | 0.33 | 0.08 | 0.28 | 0.30 | 0.91 | client-composition had the ready check. |
| `escalation_wait_min` | 2.2 | 1.9 | 1.8 | 1.6 | 1.8 | The metric prints −153.0 (below). |
| `blocked_h` | 4.4 (2.8 without the reboot's block) | 0.4 | 0.6 | 8.2 | — | The metric prints −277.7. |
| `answers_reversed` a step | 0.04 (5) | 0 | 0.08 | 0.17 | 0.10 | All five narrowed the supervisor's own earlier answer. |
| `steers` | 9 sent, 0 refused | 2 | 4 | 24 | 10 | The metric prints 5. |
| `restarts` | 1, a reboot | none | none | none | 1 | The metric prints "the 1". |
| `tokens_per_step` | 13.3M | 8.8M | 8.7M | 21.5M | 27.6M | Counted once. |
| — coordinator a step | 2.36M | 1.16M | 1.84M | 3.22M | 5.0M | 271.0M in, 98% cached. |
| — implementers a step | 7.57M | 4.68M | 5.18M | 12.7M | 14.8M | 870.4M in, 98% cached. Without 15.3, 17.3 and 17.4: 5.9M. |
| — Claude Code a step (the run) | 3.35M | 2.98M | 1.61M | 5.46M | 7.7M | 385.6M: supervisors 283.3M, designer 105.4M. "After" in `metrics.tsv` is this retrospective's own, 134.5M and $61 when it was written. |
| `cost_claude`, the run | $131: supervisor $82, designer $49 | $46 | — | — | — | Planning not counted here (below). |
| `coordinator_requests` a step | 11.7 (1,346) | 8.1 | 10.2 | 16.4 | 26.0 | The metric prints 1,404, 60 after a wait ran out. |
| `compactions` | 4 | 0 | 1 | 9 | 7 | At 349K to 395K. |
| land gates a step | 1.17 (135) | 1.28 | 1.67 | 1.91 | 1.75 | The metric prints 2.3 (below). |
| `gate_lane_other` | 125 | 1 | — | — | — | 91 process-tier runs, 8 live, 26 others. The metric prints 0. |
| `queue_wait_ratio` | 0.31 | 0.06 | 2.13 | 2.24 | 0.93 | 1.22 h waited in all, 38 s a step. |
| `suites`, temper | 693 / 1.712 s; 32 / 7.769 s → 1,265 / 4.050 s; 46 / 15.276 s | | | | | Base: 682 / 1.236 s; 31 / 7.849 s. |
| `proofs` | unit 67, world 29, joint 8, process 11, live 1; none below plan | unit 13, world 18, joint 3 | | | | By code commit. The metric prints "none 77, 1 below plan" (below). |
| `report_lines` | 13 median, 16 p90 | 10, 10 | 11, 14 | 10, 11 | 10, 12 | The README asked each report for its suites' times. |
| `lines` | code +38,928 −7,150, tests +39,637 −6,166 | | | | | Hand-written. The metric prints code +85,628 −23,242 (below). |
| `contexts` | 225K median, 262K p90; 61 handovers | 189K, 241K; 5 | 142K, 209K; 3 | 238K, 319K; 70 | 245K, 348K; 41 | The metric counts 45 of the 61 `Handover,` lines. |
| `pin_lag`, temper | skein 26, smith 148, jig 119 | | | | | Against each upstream's `origin/main` after a fetch. |

**What explains the largest differences:**

- **Tokens a step were half again test-budgets' and client-composition's,
  for three reasons.**
  - **The tail.** 15.3, 17.3 and 17.4 took 235M of the implementers'
    870M, for 7 of 115 steps (section 5, item 2). Without them the
    implementers used 5.9M a step, near test-budgets' 5.2M.
  - **Thread size.** Threads started their first change at a median of
    105K and grew 50K an increment. 68 of 82 threads went past 200K, and
    the requests above 200K carried 34% of the implementers' input
    (section 5, item 3).
  - **The long-lived roles ran the old way.** The coordinator's median
    request was 199K. The designer's forks and the second supervisor's
    late handover carried about 120M of copied or avoidable context
    (section 5, items 5 and 10).
- **`step_h` was 0.085 until noon and 0.30 after.** The morning landed
  the codecs, the components and the worlds at the pace of
  client-composition. The afternoon met the process tier (section 5,
  items 1 and 2).
- **The queue waited a seventh of what test-budgets' and jig-protocol's
  did.** x1 ran at most three sessions, and temper's suites are small:
  the median gate took 43 s. Most waiting was two-second builds behind a
  gate. One wait of 792 s, behind the failed live run, was 18% of the
  total.
- **Plan defects held at 0.33 a step, against client-composition's
  0.08.** client-composition checked each session file against `main`
  before it started (the ready check). This run had no such check
  (section 5, item 4).

**Not comparable, and why:**

- **`metrics.tsv` uses `origin/main`'s definitions,** like
  client-composition's. jig-protocol's, test-budgets' and reliability's
  used the old ones. The table uses their once-counted corrections.
  - The old kit counted this run's Claude Code input as 698.7M. Seven
    forks of the designer copied 309M of earlier messages, and it counted
    them again. Counted once, the run used 385.6M.
- **`tokens_claude` "0 before the claim":** the plan's commit `0777f0b7`
  has no `Planner-Session:` trailer. The adoption that asks for one came
  after it.
  - The session that designed and planned the run, `7edb90b4`, used
    1,844M input tokens before the claim, with 34 sub-agents:
    - the design, 325M;
    - the plan, 725M, of which the 7 session writers took 434M, the 7
      audits 104M and the 2 reviews 41M;
    - and 794M of other work: reconciling temper with jig's `main` before
      the design began.
  - Counted with the design and the plan, `tokens_per_step` would be
    22.4M.
- **`escalation_wait_min` (−153.0) and `blocked_h` (−277.7)** pair each
  block with the last `## Blocked and unblocked, <time>` heading above
  it. From 09:50:55Z the coordinator wrote that heading without its
  space, and from 11:45:11Z without its `## `. So 46 later blocks were
  timed from 09:46:45Z. Counted by hand, the waits have a median of 2.15
  minutes and a p90 of 3.8. Blocks were open for 4.44 hours; 2.76 without
  08.1's block, which spanned the reboot.
- **`restarts` "the 1"** keeps one word of the cause. The one restart's
  heading reads "the user's request". It was a reboot (section 5, item
  7).
- **`steers` 5:** four of the nine steers were logged as `# Steer`, with
  no `## Steered,` heading.
- **`steps_planned` 112, `steps_added` 3, `steps_dropped` 22:**
  - 19 increments were split by their sessions, as code-organization.md,
    section 8 asks. The ledger keeps their parents as "dropped", and
    counts their 49 parts as planned. Two parts that were split again
    are dropped too.
  - 12.2a and 15.3a, added by steer, are named like parts, so they count
    as planned.
  - Truly: 81 planned increments, 80 landed as 110 steps, 5 added, and 1
    dropped.
- **`gates_per_step` 2.3 and `gate_lane_other` 0:** this run's `heavy`
  predates the `land` label. So the kit counts all 260 gate-lane tasks as
  land's gates.
  - By command: 135 gates, which is 1.17 a landed step.
    - 116 landed.
    - 11 failed.
    - 7 passed, and ran again after `main` moved.
    - 1 was `plan check`'s, after the reboot.
  - And 125 others.
- **`proofs`:**
  - "none 77" are the plan's Markdown commits.
  - "1 below plan" is 05.4's one-line fix, `2838a7a8`.
  - One "World" with a capital is not counted.
  - By increment, every one proved at its planned layer: unit 38, world
    25, joint 8, process 11, live 1.
- **`lines`:** the metric's code includes the generated codecs (+40,612
  −15,869), Forgejo's and git's transcripts, `Cargo.lock` and Markdown.
- **`contexts`:** 61 `Handover,` lines in `log.md`; the metric reads 45,
  since 16 lost their space.
- **`coordinator_requests` 1,404:** these are `token_count` events, 58 of
  which repeat the one before. The trace has 1,346 requests, and the 4
  compactions' 1.5M input tokens are outside `tokens_coordinator`.
- **`pin_lag`** reads each upstream's `origin/main` as the checkout last
  fetched it. x1's smith and jig had not fetched since the claim.

## 4. What worked

1. **Transcripts first, and the layers open early.**
   - Session 05 recorded Forgejo's transcripts and facts from 03:34Z.
     Forgejo's four surprises (rulings 103, 104, 109, 111) came by
     04:58Z, before session 13 began. The designer settled two in
     `ba448afd` and `b65b41ef`, and the design already answered the
     others.
   - The process tier opened at 05.1 (03:34Z) and the joint layer at 11.3
     (07:30Z). `scripts/proof-one-process.sh` was kept in the repository
     at 10.4 and reused by 12.2, 15.3 and 16.6.
2. **Fast answers that held.**
   - 72 blocks, a median of 2.15 minutes; 67 were answered within 5.
   - The designer answered 7 questions in a median of 6.8 minutes from
     its wake, each with a design commit.
   - Five answers reversed, 0.04 a step, all of them narrowing the
     supervisor's own.
3. **A gate that told the truth.**
   - Each of the 116 code landings was gated on its own tree. Section
     9.2's Markdown reuse spared 8 of them a gate, across 75 ruling
     commits.
   - All 11 failures were real: one of an increment's own code, three of
     crates the session had not built, seven in tests outside its
     focused runs. No flake, no timeout, no gate run twice on one tree.
   - The suites ended at 27% and 25% of their budgets.
4. **Sessions split their own increments, and standing rules ended
   repeats.**
   - 19 increments were split by their sessions before they started,
     with no ruling (code-organization.md, section 8).
   - At the second sighting, rulings 106 (sizes), 118 and 128 (disk), and
     120 and 123 (Files) became README rules. No disk block came after
     10:41Z, and no size block after 15:23Z.
5. **Steers that split a step freed the chain.** 12.2a (14:35Z) and
   15.3a (17:34Z) landed what others waited on, while the rest of their
   steps stayed in diagnosis.
6. **The implementer's card held where it is written plainly.**
   - No gate before `land`, and no whole suite outside one.
   - No edit outside a worktree, nor of the design or the plan.
   - No observers, manifests or polling scripts, and no sub-agents.
   - Reports had a median of 11 lines in `log.md` and none past 40.
   - 18 of the 22 threads past 250K committed their work, wrote their
     checkpoint and ended within 9 requests, as the README asked.
7. **The work survived the reboot.**
   - Every worktree's uncommitted work was kept for the fresh threads,
     and ruling 114's lost unblock was recovered from the supervisor.
   - The supervisor loaded the coordinator's thread by hand 2 minutes
     after the user pasted `plan resume`'s failure.
8. **The Claude Code roles waited on events.**
   - The designer joined, ended its turn in 9 seconds, and cost nothing
     between questions.
   - Each of the supervisors' 74 watch events had something to do.
   - The first supervisor handed over by itself at 09:21Z, and stopped
     its own cron and watch.
9. **The live tier kept its budget.** 2 of 3 runs and 12 of 50
   completions. The retry followed the README's clause as written: the
   first run failed on the model's words (it delegated instead of
   reading and editing), and ruling 163 added one charter sentence to the
   process configuration only.

## 5. What did not

1. **The process tier was the first layer to join the service's halves,
   with its real limits.**
   - **What:**
     - 8 production faults surfaced between 13:10Z and 20:23Z:
       - limits: the hello's message bytes (136), documents the native
         client refuses (140), and the shipped profile's
         `conflicts: 0`, which refused every merge (153);
       - wiring: a worker start with no paths (144), a change with no
         resource (146);
       - behaviour: Forgejo's `changed` status (150), Adopt enabled
         before a party was confirmed (157), 12.2a's `projects`.
     - About as many test faults came with them (147's world, 151, 155,
       159, 160, 161).
     - In the last third, 16 steps landed in 6.7 hours.
   - **Cause:**
     - The worlds and fixtures set their limits by hand.
       `temper-service/src/limits.rs` has 61 comments that give a test
       fixture as the source of a shipped number.
     - No story ran the engine, the worker and the binary together on
       the shipped profile before 15.3.
     - Each increment proved at its planned layer: 38 of 83 at the unit
       layer, 25 at the world layer.
   - **First visible:** 13:10Z (ruling 136). The supervisor named the
     pattern at 18:36Z.
   - **Acted on:** 17.s1 (ruling 153) for `conflicts` only.
     - skein `e30fe58` (10:31Z, on `main` only) since asks that parts
       that meet connect from the start, and that each proof run above its
       tests.
     - Nothing yet says where tests take their limits from.
2. **Implementers found those faults by running the proof again.**
   - **What:**
     - 99 process and live runs for 12 steps, 3,711 s with the machine to
       themselves.
     - 15.3: 38 runs, of which 37 failed, across 12 threads and about
       124M tokens, 15:24Z to 20:49Z, with 11 blocks.
     - 12.2: 20 runs. 16.5: 17 runs.
     - Session 17 did the same in its world, in the build lane: 66 runs
       for 17.3, at about 4 s each.
     - Nine rulings set a budget of runs for one step each:
       - 145 and its amendment;
       - 147 to 150, 156, 158, 159 and 161.
       - They averaged 215 words, up to 455 (147).
       - Two had to be read again: 147 withdrew 145's amendment, and 158
         said that 145 counts failures by symptom.
   - **Cause:**
     - Section 8.8 says to fix a failing proof before landing, not where
       to look for its cause.
     - The process script was the only thing that ran the stories.
   - **First visible:** 13:04Z, 10.4's seven runs in 16 minutes. The
     supervisor noted it at 13:35Z.
   - **Acted on:**
     - Ruling 145 at 16:36Z, for 15.3 only; 15.3 still ran 13 more times.
       Ruling 148 for 17.3, ruling 158 at 20:41Z.
     - Section 10.4's "the same block twice becomes a standing rule"
       never made one.
3. **Threads were too small for their increments.**
   - **What:**
     - A median of 105K at the first change, from:
       - 25K of code;
       - 16K of the session file, whose 11 KB an increment are twice
         jig-protocol's;
       - 11K of plan documents and 7K of standing ones;
       - 437 outputs cut at the tool's cap, and read again.
     - Then 50K an increment (p75 66K), so a thread landed 1.4 steps.
     - 68 of 82 threads passed 200K, 22 passed 250K, and there were 61
       handovers.
     - Six size rulings: 102, 106, 126, 139, 142 and 143.
   - **Cause:**
     - Section 5.3 plans "near 100,000 tokens" and "about 30,000 with each
       increment".
     - The Sizes were set at 1.5× and 2× (decision 19), and 13 of them at
       600 lines, the split threshold. jig-protocol had landed at 2.4×
       its Sizes, and this run landed at 1.85× to 2.35×.
   - **First visible:**
     - 03:48Z: w2 handed over after three parts, at 201K.
     - 03:57Z: ruling 102.
   - **Acted on:**
     - Ruling 106 at 04:37Z set 2×, and 2.5× for codecs.
     - Rulings 126, 139, 142 and 143 raised single allowances.
     - Nothing resized a thread.
4. **Plan defects stayed at a third a step.**
   - **What:** 38 plan-defect blocks.
     - Six were Sizes.
     - Six were Files lists that missed a user or a copy. Ruling 123 is
       exactly section 5.4's search for the users of a changed type.
     - 15 were Shapes or numbers that did not match the pinned code or
       Forgejo. Ruling 134's Shape contradicted the plan's own
       `findings/jig-client.md`.
     - Two were "Done when" items: 105, and 164, whose command was empty
       at the base.
   - **Cause:**
     - Decision 24 records only the commands run once at the base. It
       records no consistency review, literal reading or search for
       users.
     - No session file was checked against `main` before its session
       started.
     - About 21 of the 38 are what those checks find.
   - **First visible:** 03:26Z (ruling 101).
   - **Acted on:** ruling 123 at 09:19Z, a README rule. `main`'s ready
     check and planner's search (test-budgets' proposal 2) since.
5. **The coordinator carried a large context, and typed what the kit
   could.**
   - **What:**
     - A median request of 199K. The 4 compactions came at 349K to 395K.
       Hour 10, at a mean of 347K, cost 28.1M; hour 11, after a
       compaction, cost 7.5M.
     - Bookkeeping requests (`log.md`, `context.md`, `plan set`, git
       checks) took 43% of its input, 117M. It retyped 182 reports.
     - Before 80 of its 82 spawns it queried the code graph: 146 graph
       requests, 29.0M, 10.7% of its input, with 39 re-indexes of
       temper. Its spawn prompts had a median of 5,912 characters,
       against section 7.2's 450.
     - In 34 commands it logged "on-main" and set the step `landed` in
       the same command as the checks, before it saw their result.
   - **Cause:**
     - Its profile sets no compaction limit.
     - `~/.codex/AGENTS.md`, the machine's: "Whenever spawning a subagent,
       include … relevant graph/coverage evidence in its task".
     - Section 7.3's checks and records are a sequence of the
       coordinator's own commands.
   - **First visible:** 03:22:40Z (the graph), 03:34:07Z (the checks).
   - **Acted on:**
     - Never, during the run.
     - `main` since has implementers log their own reports (`plan log`),
       and forbids graph evidence in spawn prompts. Both are in words,
       for the coordinator.
6. **The coordinator's record decayed.**
   - **What:**
     - Spaces went from 04:16Z ("Memory7 GB, disk46 GB"). By 20:30Z there
       were 0.8 spaces per 100 characters ("Resumed17/w78context196K…").
     - Markers went from 09:50:55Z (`## Blocked and unblocked,2026-…`),
       then lost their `## ` from 11:45:11Z.
     - From 15:00Z `context.md` met its 60 lines by merging lines. A
       script error at 18:07Z spelt one ruling a character a line. It
       ended at 30 lines and 27.6 KB, the longest line 17,255
       characters, and `done.md` copied it whole.
     - Twice, 21 and 37 s after a compaction, it ran `plan set --help`,
       which added a `--help` step. It removed both rows by rewriting the
       ledger by hand.
   - **Cause:**
     - Fixed lines typed by the model, which nothing checks.
     - A 60-line limit with no rule for what goes.
     - The model's drift: the prose recovered for an hour after the
       11:03Z compaction.
   - **First visible:** 04:16:35Z.
   - **Acted on:**
     - Partly, by the coordinator, at 21:19Z.
     - `main`'s section 7.5 since asks for spaces, lines of at most 200
       characters, and `log.md`'s shapes, in words.
7. **The coordinator could not tell the supervisor and the kit from the
   user.**
   - **What:** it wrote:
     - "Coordinator resumed, 07:03:22Z: the user's request" for a reboot;
     - "the user's explicit instruction" in `context.md` and `done.md` for
       the supervisor's ring after ruling 164.
     - One ring stamped its steer 19:39:00Z, at 19:36:17Z.
   - **Cause:**
     - `plan ring` and `plan resume` reach the thread in the user's role,
       unmarked.
     - The resume prompt names no cause.
   - **First visible:** 07:03:22Z. The supervisor saw it and left it.
   - **Acted on:** never.
8. **A reboot stopped the run for 98 minutes.**
   - **What:**
     - An apt upgrade rebooted x1 at about 05:26Z. It was back at 06:56Z.
     - The supervisor runs on the same machine, so nothing could alert
       the user.
     - When the user ran `plan resume`, it could not start the
       coordinator, whose thread was `notLoaded` (ruling 115).
     - Three implementer threads were lost. After the reboot two hourly
       crons fired.
   - **Cause:**
     - Unattended upgrades that reboot.
     - `plan resume` loads a thread only when reading it fails.
   - **First visible:** 06:59Z, when the user asked what to run.
   - **Acted on:**
     - 07:03Z: the supervisor loaded the thread by hand.
     - 07:29Z: it deleted the duplicate cron.
     - client-composition's proposal 4 since asks that `plan ring` and
       `plan resume` load the thread.
9. **The live run waited out its deadline.**
   - **What:**
     - The first real run took 849 s of the script's 900, long after its
       coding task had failed.
     - A process proof waited 792 s behind it, and hour 21's queue ratio
       was 0.60.
     - Its 5 completions and 849 s are in the body of `b1994cdf` as prose.
       Only the passing run's `Completions:` and `Wall:` lines are what
       "Done when" 9 reads, and git does not read them as trailers either.
   - **Cause:** the live story stops only at its deadline.
   - **First visible:** 21:27Z.
   - **Acted on:** ruling 163 at 21:31Z, for the retry only.
10. **The Claude Code roles carried their history.**
    - **What:**
      - `plan wake` resumed the last designer session each time, which
        forks it. The seven wakes grew from 46K to 646K, and 86.6M of
        the designer's 105.4M was history sent again.
      - The second supervisor wrote its handover note at 12:35Z, at 390K.
        It wrote "Starting that session needs you", and ran on to 570K
        until the user asked at 17:34Z.
        - Its 129 requests after 12:37Z cost 63M.
        - Its last instructions needed correcting: the amendment of 145
          by ruling 147, and the 17:34Z steer's wording by ruling 148.
      - The old pulse counted the forks' copies, so the 09:20Z handover
        cited "245M" for the run's Claude Code tokens. Counted once, it
        was 150M.
    - **Cause:**
      - The old `plan wake`.
      - No command to start a successor supervisor.
    - **First visible:** 05:00Z (the designer at 295K); 12:35Z (the
      handover).
    - **Acted on:**
      - The user, at 17:34Z.
      - `main` since starts a fresh designer from a brief past 150K or an
        hour idle, and has `plan handover supervisor` at 300K.
11. **Disk ran short five times.**
    - **What:** blocks at 05:08Z, 07:50Z, 08:28Z, 10:19Z and 10:39Z.
      - kache was at 176% of its cap.
      - Each session's `target/` took 12 to 15 GB, against section 11.4's
        "1 to 8 GB".
    - **Cause:**
      - Section 11.4's figures.
      - kache's late eviction.
    - **First visible:** 05:08Z.
    - **Acted on:**
      - Ruling 118 at 07:51Z: free the targets of finished sessions, then
        `kache gc` past the cap, before blocking.
      - Ruling 128 at 10:41Z: hold new starts at 30 GB.
      - No block after that.
12. **Sessions resumed after hours idle carried their old context.**
    - **What:**
      - w26 was resumed at 171K after 4 h 13 min idle.
        - It made 10 process runs and passed 320K, at 21.7M the run's
          dearest thread.
        - It handed over only when the coordinator asked, 31 minutes
          later.
      - w45 was resumed at 122K after 3 h 05 min, and reached 252K in 14
        minutes.
    - **Cause:** section 7.3 tests a resumed session's context, not how
      long it stood idle, by which time its cache has gone.
    - **First visible:** 13:28Z.
    - **Acted on:** the coordinator's message at 14:26Z.
13. **Implementers waited in short polls.**
    - **What:**
      - 481 heavy commands were started with a one-second yield, and 139
        `write_stdin` polls used one second.
      - Requests whose only trigger was a poll took 37.9M, 4.35% of
        input.
    - **Cause:**
      - The README's "one wait of 300,000 ms … never sends empty input"
        cannot be followed in Codex as written: an empty `write_stdin` is
        how one waits on a running command.
      - `main`'s card now says "60 seconds on Codex". Here a cell `wait`
        of 300,000 ms blocked past 61 s four times, the longest 263 s
        (w81, 21:14Z, Codex 0.162.1).
    - **First visible:** 03:40Z.
    - **Acted on:** never.
14. **Two of three deferrals left no `later.md` row.** The designer's
    answers deferred two things:
    - pages of 2 rows and 128-byte texts (`b5dd62bc`);
    - no page of the forge's work until jig takes pages (`318a21ba`).

    Both are under "For the user", and only the second is in `STATUS.md`.
    Neither is in `later.md`; only the adopted repository's listing got a
    row (`2b2e9214`).
15. **A finished thread ran the final checks.**
    - **What:** at 21:41:51Z the coordinator gave "Done when" to session
      17's finished thread w82, with `followup_task`.
    - **Cause:** the README says the commands run "in a worktree … after
      the last step". The old section 7.6 says the coordinator runs
      nothing, and names no one who runs them.
    - **Acted on:** `main`'s section 7.6 since gives them to the
      supervisor. The check found ruling 164's case, so it was worth
      running.

## 6. Decisions taken during the run

Rulings 101 to 164, 9 steers and 7 design answers. The supervisor
classified each in `done.md`.

| Rulings | What | Belongs in | Proposal |
|---|---|---|---|
| 103, 111, 117, 121, 122 and 140, and the answers for 15.2 and 15.3 | Forgejo's facts; statuses with no author; the core's ranges; the forge connection's deadlines; documents that fit the native client; adopted repositories; the forge's work in the inbox | In the design: `ba448afd`, `b65b41ef`, `f5e4672d`, `0be9dd34`, `b5dd62bc`, `e76cf0b2` (and jig `7b65bd1e`), `318a21ba` | — |
| 104, 109, 114, 124, 125, 127, 129, 144, 146, 160 | The code's means for what the design says | The design already says it | — |
| 132 | One process's io is its parts' io, summed | configuration.md, 7, beside performance.md, 6's "summed over its root and every component" | For the designer |
| 133 | A TLS forge's trust roots, priced as the accounts' are | configuration.md, 7 (the worst case), with the `transport` row | For the designer |
| 134 | A native shell idle before it loaded is a refused directory | client.md, the native shell; jig's part is in `later.md` | For the designer |
| 136 | The hello advertises raw message bytes; the domain keeps rendered ones | configuration.md, where the hello's states come from | For the designer |
| 137 | The fake forge reads a written document back through a call | testing.md, 4.2 | For the designer |
| 150's fix | Forgejo's `changed` file status is a modification | forge.md, 3.2, which names no status words | For the designer; section 9 |
| 153 | The shipped profile admits a merge's conflicts (4 paths of 255 bytes) | configuration.md, 7, as a requirement on the profile | 3; for the designer |
| 157 | Adopt is disabled until a party is confirmed | client.md, 4.4, with the other disabled states | For the designer |
| 102, 106, 126, 139, 142, 143, steer 04:21 | Sizes and split parts | development.md, 5.3 | 2 |
| 101, 107, 108, 110, 120, 123 | Edits outside Files | `main`'s `implementer.md`, 8.2 has each since (jig-protocol's proposal 5) | — |
| 145, 147 to 150, 156, 158, 159, 161 | Diagnostic runs, one budget per step | implementer.md, 8.8 | 1 |
| 112, 118, 128 | Disk | development.md, 11.4 | 10 |
| 115 | `plan resume` on an unloaded thread | The kit | 8 |
| 135 | A push that lost a ref-lock race is tried once more | `land` | 12 |
| 152 | The one-process share is measured serially | The plan | — |
| 105, 164 | A session's "Done when" fix; item 12 read as its rule | The plan's shape, development.md, 5.2 | — |
| 113, 116, 119, 130, 131, 138, 141, 151, 154, 155, 162, 163 | Order, fixtures, the live retry, other plan corrections | The plan | — |
| Steers 05:21, 08:21, 11:36, 19:39 | Helpers in the fixtures module; table rows; named numbers | Rules that exist: development.md, 5.2 and 5.6; `main`'s implementer.md, 8.2 | — |
| Steers 04:29, 05:06, 14:35, 17:34 | Session files amended; 12.2a and 15.3a split off | Expired | — |

**Decided by the user during the run:** nothing was asked. The user
resumed the run after the reboot (07:00Z), and asked for the
supervisor's handover (17:34Z).

**Still with the user** (`decisions.md`, "For the user"):
- the plan's seven defaults, which the planner took under the user's
  delegation;
- and three designer answers:
  - an adopted repository is not listed until jig's core lists it;
  - pages of 2 rows and texts of 128 bytes;
  - no page of the forge's work until jig takes pages.

## 7. The proposals adopted after the last retrospective

test-budgets' retrospective, the last before the claim, landed at 03:13Z
with 14 proposals open. Its decisions (jig `7db77430` at 08:18Z and
`f4f0feba` at 09:24Z) and their application (skein `479f836`, `63fff2d`)
came during this run. With them came parts of jig-protocol's 1, 2, 4, 5,
10, 15 and 16, reliability's text-only proposals (`7f4b0e8`, 09:22Z), the
user's change to proofs (`e30fe58`, 10:31Z), and four of
client-composition's (`f80521b`, 17:55Z).

**None reached the run.**
- The coordinator and the implementers read `~/src/rust/skein`, and
  `land`, `heavy` and `plan` link into it.
- On x1 it stayed at `9539e00`, since nothing landed in skein from x1.
- Nothing in the run's record cites `implementer.md`, `supervisor.md`,
  `plan log` or `plan ruling`.

The plan's README carried some rules in by hand:
- jig-protocol's 5, 6, 8, 9 and 11;
- the trailers' shape;
- "a design gap is a question".

`main` would now refuse such a claim: `plan check` refuses a plan while
the last retrospective has a proposal open (test-budgets' 1).

So this run measures the old process, and client-composition (carbon-forge,
12:34Z to 16:51Z) is the first run under the new one. The table gives
each adopted proposal's metric, this run's value as the baseline, and
test-budgets' before it.

| Proposal (test-budgets) | Decision | Meant to move | test-budgets | protocol | Moved |
|---|---|---|---|---|---|
| 1. Decide before the next run | adopted | proposals that recur | 6 of 17 | none decided at the claim. Five of test-budgets' items recurred: plan defects, retyped reports, a late supervisor handover, short polls, and the metrics (section 5, items 4, 5, 10 and 13; section 3) | Not in force |
| 2. Search a changed thing's users; the ready check | adopted | plan defects a step | 0.28 | 0.33 | Not in force. client-composition: 0.08 |
| 3. Generated code proves itself downstream | adopted | `steps_added` | 3 | 5 | Not applicable: no downstream generation |
| 4. `land` holds its repository's turn | adopted; on `kit-after-parity` | land gates a step | 1.67 | 1.17 | Not in force; 7 gates ran again after `main` moved |
| 5. Nothing unready enters the gate lane | adopted | failed gate-lane runs | 5 | 11 failed gates, 3 of them in crates the session never built | Not in force |
| 6. The prompt says what to read and how to wait | adopted | `tokens_implementers`, `contexts` | 5.18M; 142K, 209K | 7.57M; 225K, 262K | Not in force. Its "60 seconds on Codex" does not hold here (section 5, item 13) |
| 7. `land` refuses unreadable trailers | adopted; on `kit-after-parity` | trailers git cannot read | 1 | 0, by the README's text rule | Not in force |
| 8. The landed check reads the branch tip | adopted | process escalations a step | 0.10 | 0.08 | Not in force |
| 9. Implementers log their reports | adopted in part | `tokens_coordinator`, `compactions` | 1.84M; 1 | 2.36M; 4 | Not in force |
| 10. The supervisor hands over at 300K | adopted | Claude Code tokens a step | 1.61M | 3.35M; 68% of supervisor input above 300K | Not in force |
| 11. `plan metrics` counts what 15.2 says | adopted | comparability | — | escalations once, messages once, suites by repository: right here; the rest of section 3's faults remain | In part, on `main`'s kit |
| 12. A design gap goes to the designer | adopted | behaviour left unchecked against the design | 1 | Of 9 design-gap blocks, 4 went to the designer and 4 were answered from the design's text. 16.3b's (ruling 134) was ruled a plan defect, and its behaviour is not in the design | Not in force |
| 13. A finding not verified is verified first | adopted | `steps_added` | 3 | 5 | Not in force |
| 14. smith's counting tests | adopted | — | — | — | Not applicable |

From jig-protocol's proposals adopted in the same commit:

| Proposal (jig-protocol) | Meant to move | jig-protocol | protocol | Moved |
|---|---|---|---|---|
| 1. A fresh designer past 150K or an hour idle | Claude Code tokens | 514M for 36 questions | 105M for 7, 82% copied history | Not in force |
| 4. `land` builds and lints in the build lane first | `queue_wait_ratio` | 2.24 | 0.31 | Not in force |
| 15. The pulse prints cost by role | `tokens_per_step` | — | the supervisors quoted the old pulse's double count | Not in force |

**What to do before the next run on x1.** x1's skein checkout is 11
commits behind `origin/main`, so `plan check` will refuse there until it
moves. Reliability's proposal 20 would have `plan check --fix` do it. It
was deferred until after smith's parity run.

## 8. Proposals

Each gives a rule in one sentence, its evidence, the documents it
changes, and the metric it should move. They are ordered by expected
effect. Each was checked against skein's `origin/main`, so none repeats
what is adopted there.

1. **A proof that fails is reproduced below its layer, in a world or a
   focused test in the build lane, and runs again only after a change
   aimed at a cause its log names; the same symptom a third time is a
   question.**
   - **Evidence:**
     - 99 process and live runs for 12 steps, 3,711 s alone on the
       machine (section 5, item 2).
     - 15.3's 38 runs, 37 of them failed, about 124M tokens.
     - Session 17's world runs took about 4 s each, in the build lane.
     - Nine rulings, each a budget for one step, of which two had to be
       read again. This is section 10.4's "same block twice" made a
       standing rule.
   - **Changes:** implementer.md, 8.8 and 8.7; development.md, 5.5. It
     takes in rulings 145 and 158.
   - **Moves:**
     - `gate_lane_other` from 1.09 a step toward one run per proof.
     - `tokens_implementers` a step down: the tail took 235M.
     - Process escalations down.
2. **The planner sizes increments from the record: a Size of at most
   about 300 lines, landed within 2× (2.5× for codecs and documents),
   with section 5.3's arithmetic at 105,000 tokens at the first change
   and 50,000 an increment.**
   - **Evidence:**
     - Landings at 1.85× to 2.35× their Size here, 2.4× in jig-protocol
       and 1.6× in reliability; six size rulings.
     - 13 increments planned at the split threshold, 11 of them split.
     - 1.4 steps a thread, 68 of 82 threads past 200K, 61 handovers
       (section 5, item 3).
   - **Changes:** development.md, 5.2 (the Size line) and 5.3; the
     size limits of code-organization.md's template.
   - **Moves:** size escalations from 6 to 0; `contexts` p90 down from
     262K; handovers a step down from 0.53; `tokens_implementers` a step
     down.
3. **Every test, world and process-tier configuration takes its limits
   from the shipped profile's builder, and a test that needs another
   value names it beside the profile's, with the reason.**
   - **Evidence:**
     - Rulings 136, 140 and 153, three production faults met only at the
       process tier.
     - 147's world and 155's fixture agent, two fixture faults.
     - The 61 comments in `limits.rs` that cite a fixture as a shipped
       number's source (section 5, item 1; section 9).
   - **Changes:** development.md, 5.6, whose Test shape names where a
     world's limits come from; development.md, 5.4, where the planner
     runs each sketched number through its pinned check; temper's
     testing.md, for the designer.
   - **Moves:**
     - Escalations met in the last third of a run down; `steps_added`
       down.
     - `step_h` in the tail down. It complements `e30fe58`'s "parts that
       meet are connected from the start".
4. **The coordinator's Codex profile compacts at about 200,000 tokens.**
   - **Evidence:**
     - A median request of 199K, and compactions at 349K to 395K.
     - Hour 10 at a mean of 347K cost 28.1M; hour 11, after a
       compaction, 7.5M.
     - The four compactions cost 1.5M in all, and `context.md` carries
       the state across them (section 5, item 5).
   - **Changes:** skein's `workspace/coordinator.config.toml`
     (`model_auto_compact_token_limit = 200000`, which Codex 0.162.1
     reads); development.md, 7.5.
   - **Moves:** `tokens_coordinator` down by about a third (an estimate);
     `compactions` up, to about ten a run of this length.
5. **In a run, the coordinator has no code graph, and implementers query
   the project named in their prompt, and never list projects, check
   status or coverage, or index.**
   - **Evidence:**
     - The coordinator: 146 graph requests, 10.7% of its input, and 39
       re-indexes.
     - The implementers: 594 graph-triggered requests, 7.7% of input,
       and 465 bookkeeping calls. 9 of them re-indexed the primary
       checkout from a worktree.
     - Both came from the machine's `~/.codex/AGENTS.md` (section 5,
       item 5). `main`'s rule against graph evidence in spawn prompts is
       in words only.
   - **Changes:** skein's `workspace/coordinator.config.toml` (the
     codebase-memory server off); the workspace `AGENTS.md`;
     implementer.md, 8.1. `~/.codex/AGENTS.md` is the user's own.
   - **Moves:** `tokens_coordinator` down about a tenth;
     `tokens_implementers` down about a twentieth.
6. **The kit writes the coordinator's fixed lines: `plan landed <step>
   <commit>` runs section 7.3's git checks and sets the step only when
   they pass, and `plan` writes "Blocked and unblocked", "Steered",
   "Handover" and "Coordinator resumed" with their times.**
   - **Evidence:**
     - Markers lost their shape from 09:50Z, which broke four metrics.
     - 34 commands logged "on-main" before seeing it.
     - Bookkeeping was 43% of the coordinator's input.
     - Two `--help` steps right after compactions (section 5, items 5
       and 6).
     - `main`'s section 7.5 asks for the shapes in words.
   - **Changes:** development.md, 7.3 to 7.6 and 10.3; `plan`.
   - **Moves:** `coordinator_requests` a step down from 11.7;
     `tokens_coordinator` down. `escalation_wait_min`, `blocked_h`,
     `steers`, `restarts` and `contexts` read true.
7. **A session that has stood idle for over an hour continues in a fresh
   thread from its checkpoint, whatever its context.**
   - **Evidence:** w26, resumed at 171K after 4 hours, the run's dearest
     thread; w45, at 122K after 3 hours, at 252K 14 minutes later (section
     5, item 12). It is the designer's hour on `main`, for sessions.
   - **Changes:** development.md, 7.3, step 4.
   - **Moves:** `contexts` p90 down; `tokens_implementers` down.
8. **`plan ring` and `plan resume` name who speaks, and a run resumes by
   itself after a reboot.**
   - A ring's message begins "From the supervisor, by plan ring:".
   - `plan resume` gives the cause, a reboot when the machine booted
     after the heartbeat. It loads an unloaded thread
     (client-composition's 4), and writes the "Coordinator resumed"
     heading itself.
   - `plan start` enables a user unit that runs `plan resume` at boot,
     and `plan release` disables it.
   - `plan check` warns when the machine may reboot unattended (apt's
     `Unattended-Upgrade::Automatic-Reboot`).
   - **Evidence:** 98 minutes stopped, ruling 115, a reboot logged as
     the user's request, and an end logged as the user's instruction
     (section 5, items 7 and 8).
   - **Changes:** development.md, 6, 7.4, 7.6 and 10.5; `plan`.
   - **Moves:** `running_h` up toward `wall_h` (1.6 h lost here);
     `restarts` by their true cause.
9. **The implementer's card names the wait that blocks on the Codex in
   use, a cell `wait` of 300,000 ms here, and every heavy command starts
   with it; `test-plan --live` checks it on each new Codex.**
   - **Evidence:**
     - Four waits blocked past 61 s, the longest 263 s, against the
       card's "60 seconds on Codex".
     - 481 starts used a one-second yield, and poll-only requests took
       4.35% of input (section 5, item 13).
   - **Changes:** implementer.md, 8.3; `test-plan`.
   - **Moves:** the implementers' poll-only share down.
10. **Every plan takes ruling 118's disk rule, and section 11.4 says a
    session's `target/` takes up to 15 GB.**
    - **The rule:** free the targets of finished sessions, run `kache gc`
      when the store is past its cap, and only then block.
    - **Evidence:** five disk blocks before ruling 118, none after
      ruling 128; kache at 176% of its cap; targets of 12 to 15 GB
      (section 5, item 11).
    - **Changes:** development.md, 11.4.
    - **Moves:** infrastructure escalations down, from 5 here.
11. **`plan metrics` counts what section 15.2 says:**
    - escalations from the rulings `plan ruling` lands, or each
      `Since:` paired with the answer of its own section;
    - restarts with their whole cause;
    - a step its session split is a `split` row in the ledger, whose
      parts count as planned, and steps added by a steer are named
      `.s<k>`, as section 7.3 says, and count as added;
    - proofs and lines from code steps only, with a `Step:` trailer and
      a gate that ran:
      - layers read in lower case;
      - an increment judged at its last part;
      - generated code, goldens, transcripts and lock files apart from
        code;
    - land's gates by label, or, with none, by `check.sh` in the command;
    - the coordinator's requests from its usage records, with the
      compactions' tokens;
    - `pin_lag` after fetching each upstream.
    - **Evidence:** section 3, "Not comparable".
    - **Changes:** the kit; development.md, 7.3 (the `split` state) and
      15.2 (`lines`' parts).
    - **Moves:** comparability. This run's values become the ones
      section 3 counted by hand.
12. **`land` treats a push that lost a ref-lock race as `main` moving,
    exit 6; `plan set` refuses a step that starts with `-`.**
    - **Evidence:** ruling 135, which the README carried for the run;
      the two `--help` steps (section 5, item 6).
    - **Changes:** `land`, `plan`; development.md, 9.1.
    - **Moves:** infrastructure escalations down; no hand edits of the
      ledger.
13. **A live story ends at its task's first terminal failure, and a live
    step's `Completions:` and `Wall:` are trailers that total every live
    run of the plan.**
    - **Evidence:** 849 s of 900 after the task had failed; a 792 s
      wait behind it; the first run's numbers only in prose (section 5,
      item 9).
    - **Changes:** temper's testing.md, 2.7, for the designer, and
      `scripts/proof-live.sh`; development.md, 8.4.
    - **Moves:** `queue_wait_ratio` down; live usage counted.
14. **An answer that defers work lands with its `later.md` row.**
    - **Evidence:** two of three deferrals have none (section 5, item
      14).
    - **Changes:** development.md, 10.4.
    - **Moves:** deferrals missing from `later.md`, from 2 to 0.

## 9. Debts left in the code

temper keeps no `docs/development/tech-debt.md` yet. These are for one,
in jig's shape, ordered by what they cost the next plan.

- **Facts go nowhere.**
  - The service's engine pops its root's facts and only counts them
    (`crates/temper-service/src/engine.rs:100-107`).
  - The worker drops its link's facts (`worker.rs:331`).
  - Nothing pops the worker root's facts, so its queue fills and the
    rest count as `lost` (`temper-worker-domain/src/domain.rs:227-231`).
  - The design says nowhere where facts go: a question for the designer
    first.
- **The shipped limits are copied from test fixtures.** 61 comments in
  `crates/temper-service/src/limits.rs` give a fixture as a number's
  source, and `reserve: 0` stays. Ruling 153 found one of them refusing
  every merge. Paid by proposal 3, and by deriving each number from its
  worst case (configuration.md, 7).
- **The socket count is a hand count** (`checks.rs:267-291`).
  - It has no term for the forge component's connections.
  - Its refusal reports sockets as "33 bytes" (`tests/checks.rs:185`).
- **Forgejo's file-status words** (`temper-forge-forgejo/src/client/files.rs:143`).
  - `changed` is read as a modification (ruling 150).
  - The transcripts hold only `added`, and Forgejo's per-commit lists
    also use `removed`, which `status()` may refuse.
  - Paid by a transcript of each word.
- **The store's generators live outside the repository.**
  `logs/temper-04/*.mjs` generated the store's schema and translations,
  rewritten for each increment. The next change to a stored record will
  redo them by hand.
- **`tests/codecs/tests/fuzzy_forgejo.rs:4` takes a median of 10.9 s**
  (4.2 to 16.0), against about 5. It runs 200 mutations a transcript
  case, and grew with each case.
- **44 of temper-service's 153 unit tests pass 20 ms.** Each builds a
  whole `Service` (`tests/fixtures.rs:393-475`).
- **The binary reads its configuration twice.** `lib.rs:264-300` reads
  the forges again with `.expect("checked …")`, and `lib.rs:206-229`
  reads the trust file again.
- **Copies of jig's reference service:** `process_io`
  (`checks.rs:292-304`) and `agent()` (`crates/temper/src/lib.rs:146-195`).
- **Rulings' code without a test:**
  - 133's pricing of TLS trust roots (`temper-service/src/forge.rs:296-320`),
    where every fixture carries no roots;
  - 136's separator underflow (`lib.rs:172-178`).
- **Two guesses.**
  - Ruling 134's "idle before it loaded is a refused directory"
    (`temper-shell-terminal/src/client.rs:350-357`), until jig says why it
    refused.
  - Ruling 137's read-back, a second call that another call could
    precede (`testing/temper-fake-forge-protocol/src/server.rs:153-170`).
- **Parallel types:** `startup/application.rs`'s `Transport` (line 32)
  repeats `settings.rs:31`. Its `Forge` (line 21) differs from
  `settings.rs:7` only in `api_token`. Their `trust` is a file name in
  one and the file's bytes in the other.
- **Six lint exceptions in tests,** which the plan's "Don't" list forbids
  and no ruling allowed:
  - `temper-service/src/tests/fixtures/startup.rs:1` and
    `src/tests/startup.rs:1`;
  - `temper-shell-terminal/tests/support/mod.rs:1`;
  - `tests/one-process/src/forge.rs:88`;
  - `tests/one-process/src/referee.rs:181` and `:483`.

  The run added 42 `#[expect(` in all, 23 of them `clippy::manual_map` in
  the store's records. It also added one `too_many_lines`
  (`temper-service/src/limits/names.rs:10`).
- **77 `unreachable!` arms in production code,** 45 of them in
  `temper-forge-forgejo`, from encoders and decoder tables that match the
  whole enum. Not audited.
- **The one-process world's person host** holds story-specific branches
  and two `thread_local!` cells (17.4; supervision.md, 21:38Z).
- **The live story waits out its 900 s after its task fails**
  (`crates/temper/tests/support/live.rs`). Proposal 13.
- **Files near the limit:**
  - `tests/system/src/composed.rs` 795;
  - `temper-protocol-store/src/tests/fixtures.rs` 790;
  - `temper-fake-forge-domain/src/api.rs` 785;
  - `temper-engine-domain/src/decision.rs` 785;
  - `temper-engine-domain-forge/src/boundary.rs` 775.

  Also the 15 large files the plan named, unsplit (`later.md`).
- **From `later.md`:** `ByteRing`, `PushDiagnostic` defined twice, and
  `Role::Adopted`.
- **Design text owed** for the eight rulings "not in the design yet"
  (section 6).

## 10. Decisions

| Proposal | Decision | Where |
|---|---|---|
| 1 | open | |
| 2 | open | |
| 3 | open | |
| 4 | open | |
| 5 | open | |
| 6 | open | |
| 7 | open | |
| 8 | open | |
| 9 | open | |
| 10 | open | |
| 11 | open | |
| 12 | open | |
| 13 | open | |
| 14 | open | |
