# Retrospective: real-loop

2026-10-20. This is the run of temper's `real-loop` plan on debian-16gb-hel1-1, a machine with eight cores and 16 GB. The plan is the second of two that build temper's tools (`docs/design/client/tools.md`, area 1). It built `temper ctl`'s one-shot host (`temper-client-shell-cli`), fourteen client stories over the tools, the real-loop world (`tests/real-loop`) with fifteen focused stories and one fuzzy one, the process tier's five `temper ctl` proofs, and the fuzzy walking story in eight shards.

| Event | When (UTC) |
|---|---|
| Planning started (session 63f7b581) | 2026-10-19T08:46:12Z |
| Plan landed (temper `1db350240`) | 13:37:28 |
| For the user A, C, E and F recorded by the director (`e0bc5ebb9`) | 13:40:09 |
| Claim | 13:48:59 |
| First landing (temper-08.1, `2065dd47f`) | 13:57:11 |
| The user's decisions: the real loop is smoke (ruling 130), then L (`08e91e24a`) | 16:42:27, 17:32:40 |
| Paused: temper 13.1 to 13.5 wait for forge-kit's temper-05.2 | 17:45:59 |
| forge-kit's temper-05.2 landed (`4e16e4eba`) | 2026-10-20T07:40:03 |
| forge-kit's closing repin (`93f554438`) | 09:57:28 |
| Resumed, on the director's message | 10:05:07 |
| Last landing (temper-13.s2, `dc9e5c8e8`) | 15:41:49 |
| `done.md` written | 15:43:52 |
| "Done when" run (Codex, then the supervisor) | 15:47:40 to 16:05:14 |
| Release | 16:08:10 |

All 36 ledger steps settled: 35 landed, 26 planned and 9 added; temper-09.4 was dropped (ruling 130). The plan changed temper alone. This retrospective was written by a fresh session that took no part in the run (development.md, section 15).

**The last run under development.md.** During this run the director moved new work to owners (skein's `docs/foundation/owners.md`, an experiment since 2026-10-19). Its decision of 10-20 07:00, recorded in smith's accounts retrospective (section 10), says that development.md is being retired: "no new run starts under it; forge-kit and real-loop finish as they are". forge-kit finished at 09:57. So this run's numbers are the last of the old process, the ones owners.md 6 says the director compares the owners' closing notes with. Its proposals (section 8) target what owners and the shared kit still use: owners.md, `heavy`, `land`, `after-landing`. Where a lesson applies only to development.md, section 5 says so and proposes nothing.

**Who read what.** Eight read-only readers, one per source. Four ran on Codex gpt-6-astra at `high` through `tmp/director/codex-helper.sh`, and four were Sonnet sub-agents:
- Codex: `log.md`, with the ledger, `context.md`, `alerts.md`, `sessions/`, `tmp/` and the 84 reports;
- Codex: `decisions.md`, rulings 101 to 152 and "For the user", against temper's design on `main`;
- Codex: the coordinator's rollout and the 42 implementer threads it spawned;
- Codex: the Claude Code transcripts of the supervisor and its 7 sub-agents and 6 Codex helpers, the five designer sessions, and the planner and its 23 sub-agents;
- Sonnet: temper's git history from the plan to the release, with its trailers;
- Sonnet: `logs/heavy.tsv` and its 3,189 tasks;
- Sonnet: the earlier retrospectives and skein's process from `2bd29bd` to `main`;
- Sonnet: the debts, from `later.md` and the code on `main`.

This session read:
- development.md's sections 1, 5.2 ("Done when"), 7.6 and 15, and owners.md whole;
- the plan's README, `done.md` and `supervision.md` whole, and `decisions.md`'s "For the user" and rulings 150 to 152;
- the tools retrospective whole, and the accounts retrospective's section 10;
- the director's messages to the supervisor, and the supervisor's transcript from 22:19 on 10-19 to 10:06 on 10-20;
- the metrics and their history.

Not read: the design session `74881b0e` (temper `116d0a645`, among the Inputs), which is not on this machine.

**Conventions.** Times are UTC, on 2026-10-19 unless a date is given. Costs are at list prices for each message's model, as `plan metrics` prices them:

| Model | Input | 1-hour cache write | Cache read | Output |
|---|---|---|---|---|
| Opus 5.5 | $4 | $8 | $0.20 | $20 |
| Sonnet 5.5 | $2 | $4 | $0.10 | $10 |

This document proposes; the user, or the director, decides.

**Which process.** The run claimed under skein `2bd29bd` (`supervision.md`'s Process line). The kit moved four times:
- to `31ae5ab` at 15:06, by ruling 116, for the "no designer recorded" defect the run met at 14:27;
- to `74e48147` at the resume, 10:05 on 10-20, as `plan resume` does;
- to `9ab88f69` at about 12:40, by ruling 141: `land`'s exit 9 for a refusal, and heavy's load settle;
- to `a46635c8` at about 13:43, by ruling 147: heavy leaves credential-shaped names out of queued tasks.

## 1. In one page

- **35 steps landed and one was dropped. "Done when" passed but for three misses.**
  - Items 2 and 3 failed on lines other work wrote during the run: browser-1's second `global-timeout = "60s"` and `opt-level = 3`, and ten test names with `_and_` from the accounts-2 owners. Session 13's second command failed on two of its own literals: one no increment ever wrote, and one 13.5 renamed. Ruling 150 records each. None of the 113 tests this run added is named with `_and_`.
  - Item 5: the worlds below the domain other than the real loop sum to **4.04 s against 4 s** (ruling 151, for the user). This run's fourteen client stories added 0.47 s to that sum, as `later.md` had foreseen. The real loop sums to 3.23 s against its 6 s ceiling, and no story is past 1 s. The suites' medians are 14.44 s and 19.65 s, against aims of 37 s and 22 s.
  - The process tier passed 24 tests in 107.8 s, against 180 s.
- **25.9 hours from claim to `done.md`; 9.2 of them running.** For 16.3 hours the run was paused: temper 13 waited for forge-kit's temper-05.2 (For the user D).
  - The README projected its 12-step chain at 4.8 hours. It took 9.2 running hours.
  - Eleven of its steps took about 22 minutes each, against the 24 projected. The twelfth, temper 13.5, the landing through the loop, took 3 hours 12 minutes.
  - The wait for forge-kit cost two more running hours: 16:16 to the pause, and 10:05 to 13.1's start at 10:34.
- **The end of the wait woke nobody.** After its 22:19 check, the supervisor's last watch printed the next hourly event, then "process exited while detached". The session took no turn for 11.7 hours. temper-05.2 landed at 07:40, and the run resumed at 10:05, when the director wrote: 2 hours 25 minutes later.
- **The real loop found defects that the lower worlds had missed.**
  - Its stories found three, in code whose own worlds passed: a stream not freed when a call's peer ends (08.s1, temper-service); owner work stuck behind a refused input (09.s1, temper-service); and in the forge domain, a wake lost when it overtook a fresh read (13.s3). Running the story again found that 13.s3's fix emitted a stale decision (13.s4).
  - Each was fixed in its owner's crate, with a unit test there, as testing-strategy.md 2.8 asks.
  - After the run's landings, the process tier and the after-landing tier found two more in `temper ctl`'s own host: a sign-in answered `Again` was fatal, and many calls exhausted its sockets. Other owners fixed both outside the ledger (`e9bb42960`, `5761ae077`).
- **Plan defects came back.**
  - 20 rulings fixed one: 0.57 a step, where tools had 0.05 plan-defect blocks a step.
  - 14 of them fell on the sessions that started at the claim, which no ready check had compiled. temper 06.1 blocked four times in 27 minutes.
  - The ready-checked sessions (09, 10 and 13) met six, four of them found only by running a story.
  - The defects lay in how the harnesses behave over time: the referee's view, Online, `Steps`, the admission envelope, a parked run against a finished one. The planner's literal checks compared names and types, and could not see this.
- **Questions were answered fast, and reached the design.** 33 blocks took a median of 4.2 minutes.
  - Five designer sessions answered six questions. Each answer landed in the design within 11 to 16 minutes of its question.
  - The user's two decisions reached temper's `testing.md` within 14 and 11 minutes: the real loop is smoke, and it is counted apart under a standing 6 s.
  - The run had 52 rulings: 1.49 a step, against tools' 0.91.
- **15.3M tokens a step, against tools' 12.1M.**
  - The coordinator took 2.96M a step, against 1.90M. Its requests carried about 149K of context each, against 110K.
  - The implementers took 6.47M a step. Temper 13.5 alone took 42.1M, 19% of them.
  - Claude Code cost $217:
    - the planner $126.50, 65% of it spent refreshing, at Opus, six drafts that the tools planner had written for this plan;
    - the supervisor $65, of which its sub-agents took only $9.27, since its later checks ran on Codex;
    - the designers $24.40, for six questions.
- **The run's own queue was as calm as tools'.**
  - Its own tasks waited 0.82 seconds for each second they ran. After the resume, the accounts-2 owners ran 683 tasks on the same machine, and 82% of the run's waiting was behind them.
  - `heavy` writes every task on the machine into the active run's log. So `plan metrics`' gates per step (3.1), gate-lane tasks (84) and queue ratio (1.50) count other work. The run's own figures are 1.17, 34 and 0.82.
- **Test shape still took six rulings and two gate supplements** (09.s2 and 13.s2). `land`'s refusal of `_and_` names (tools' proposal 6) held for all 113 of this run's tests. But `land` applies it only to an active plan's branches, so owners' landings on the same `main` added ten such names in four hours.
- **Six proposals,** in section 8:
  1. `heavy`'s log names whose each task is;
  2. `land` checks test names on every landing;
  3. a diagnosis budget renews with each new symptom;
  4. a task's sketches run against `main` before it starts;
  5. a landing wakes whoever waits for it;
  6. Codex's `-o` takes an absolute path.

## 2. Outcome

`done.md` holds each command's output: the Codex helper's run at temper `9f444b30c` (15:47:40), then the supervisor's run of item 5's second part at `8f2228ce8` (16:03:56).

| Item | Result |
|---|---|
| 1. Every step landed or dropped | **Met.** 35 landed, temper-09.4 dropped by ruling 130 (the real loop is smoke). |
| 2. Each session's own "Done when" | **Met but for three commands, on other work's lines or the plan's own literals** (ruling 150). |
| | Session 13, second command: `cue: Some(` was never written by any commit (`git log -S` finds none); the cue is an assignment, `setup.checks.cue = Some(…)`. 13.5 renamed `@coding Small fix` to `@coding Fix`, "to keep the coding brief within the title bound". |
| | Session 14, seventh and eighth commands: each found two lines, not one. The second lines are browser-1's: its `[profile.browser]` timeout (`00e77f2cd`) and its `skein-ui-bundle` opt-level (`7719ba16a`). |
| | The other 33 session commands passed. |
| 3. No test this plan adds is named with `_and_` | **Met for this plan; the command failed** (ruling 150). It printed ten names, all from the accounts-2 owners' commits (`3ceceb8ff`, `a438acc2c`, `2a529b4f2`, `2a343d21e`), in directories this plan shares. None of this run's 113 test functions has `_and_`. |
| 4. The process tier within 3 minutes | **Met.** 24 passed in 107.8 s (base 152.9 s). The tier fell from 152–159 s on 10-19 to 105–109 s on 10-20. It fell while the run was paused and landed nothing, so other work's landings caused it. |
| 5. The test-time aims | **Missed by 0.04 s for the other worlds** (ruling 151, for the user). The rest was met. |
| | Suites, medians of three: focused 14.44 s (aim 37 s), fuzzy 19.65 s (aim 22 s); the base was 9.90 s and 11.86 s. |
| | Worlds below the domain, medians of the last three of four idle runs: the other worlds 4.04 s against 4 s; the real loop 3.23 s against 6 s; no story past 1 s. |
| | By world, from the base (ruling 112) to the end: client 0.92–0.99 s → 1.62 s (this run's 14 stories, 0.47 s); one-process 1.54–1.67 s → 1.80 s; forge protocol 0.61–0.67 s → 0.31 s (forge-kit); terminal 0.16 s → 0.17 s; git 0.15–0.18 s → 0.14 s; reconcile 0.26–0.30 s → gone to skein. |
| | The slowest of the other worlds: the one-process adoption story, 0.899 s, 22% of the sum. 13.s1 had cut it from 1.008 s to 0.724 s; it rose again during the run, for a cause not traced. |
| | 13.5's landing story, 4.1 s, went to `fuzzy_landing.rs`, as the README's rule says. |
| | The first measurement could not run: forge-kit's temper-07.1 (`4b074713d`) had removed `temper-forge-setup-reconcile-world` during the run. Ruling 150 dropped it from the list. |
| 6. Nothing else | **Met.** |

What the plan leaves for later:
- **The other worlds' 0.04 s,** with the adoption story at its cause (ruling 151): work for temper's owner.
- **testing.md 6.1 still names reconcile's world** among the 4 s worlds: `later.md`'s last row.
- **The rows the plan added to `later.md`:** an existing-only private open in skein's io (ruling 102), and jig's reference service freeing a stream at a peer's end (ruling 123).
- **The real loop's next stories,** about fifteen `later.md` rows, each one smoke path.

## 3. Numbers

`plan metrics` ran without error from skein's checkout and wrote `metrics.tsv`. The comparisons:
- tools, temper's last run, on this machine, the first part of the same area;
- accounts, smith's run of the same days;
- forge-kit, temper's run beside this one, on another machine;
- client-api, the largest run.

| Metric | real-loop | Notes | tools | accounts | forge-kit | client-api |
|---|---|---|---|---|---|---|
| `wall_h`, `running_h` | 25.9, 9.2 | The pause took 16.3 h. | 8.3, 8.5 | 12.8, 11.0 | 29.0, 18.3 | 63.5, 60.0 |
| `steps_planned`, `added`, `dropped` | 26, 9, 1 | Of the added: three from design answers (05.s2, 08.s1, 13.s1), one from a plan defect (05.s1), three for defects the stories found (09.s1, 13.s3, 13.s4), and two gate checks of test shape (09.s2, 13.s2). | 42, 1, 0 | 54, 2, 0 | 55, 14, 5 | 137, 22, 0 |
| `step_h` | 0.26 | The chain set the run again (section 5, item 2). | 0.20 | 0.20 | 0.27 | 0.38 |
| `chain_step_min`, `max` | 9.5, 37.4 | Not comparable as it stands: see below. | | 33.6, 116.4 | | |
| `escalations` | 33: cross-session scope 13, plan defect 12, process 5, design gap 3 | 0.94 a step. Of the 13 "cross-session scope", the rulings call 8 plan defects and 3 diagnosis budgets. | 13 | 15 | 37 | 194 |
| `escalation_wait_min` | 4.2 | To the interim answer. To a designer's answer, 14 to 26 minutes. | 4.2 | 3.5 | 6.2 | 2.7 |
| `blocked_h` | 2.3 | | 1.2 | 1.2 | 11.1 | 13.9 |
| `answers_reversed` | 11 | 11 `Reverses:` lines; 123 names two rulings. Five correct an earlier ruling's fix of the plan, two end an interim wait, and four change an instruction. | 9 | 0 | 44 | 9 |
| `steers` | 11 sent, 1 point refused | The refused point asked the coordinator to inspect tests; ruling 120 withdrew it. | 15, 0 | 26, 0 | 85, 0 | 64, 18 |
| `restarts` | 1 resume, 1 load | The pause's end. | none | 3 loaded, 2 reboots | many | crash 1, unknown 4 |
| `tokens_coordinator` | 103.8M in (98% cached), 0.17M out | 2.96M a step. | 81.9M; 0.14M | 108.3M; 0.20M | 286.3M; 0.50M | 947.0M; 1.58M |
| `tokens_implementers` | 226.6M in (97%), 0.77M out | 6.47M a step. | 242.7M; 0.93M | 428.8M; 1.35M | 385.7M; 1.37M | 3,349.8M; 8.29M |
| `tokens_claude` | 520.1M in (97%), 3.46M out: 305.8M before the claim, 201.7M in the run, 12.6M after | "After" is this retrospective, until the metrics ran. | 708.9M | 1,057.3M | 507.8M | 2,284.3M |
| `cost_claude` | $217: planner $126, supervisor $65, designer $24, retrospective $1 | $6.2 a step: planning $3.6, the run $2.5. | $287 | $375 | $164 | $795 |
| `supervisor` | 1 session, 26.3 h, 0.7M weighted tokens an hour, 43% in the 40 requests after each start or compaction, 13 compactions | Compactions at 167K to 172K. The 26.3 h include the 11.7 h in which the session took no turn. | 1, 8.5 h, 1.4M, 87%, 19 | 2, 12.1 h, 1.2M, 62%, 13 | Codex | 36, 65.0 h, 1.4M, 36%, 25 |
| `tokens_per_step` | 15.3M | Coordinator 2.96M, implementers 6.47M, the run's Claude Code 5.76M. | 12.1M | 15.2M | 9.8M | 41.5M |
| `coordinator_requests` | 697, 52 after a wait ran out | 19.9 a step. A ring took a median of 4 requests, p90 7. | 743, 47 | 943, 63 | 1,726, 60 | 4,655, 325 |
| `compactions` | 2 | At 182K (14:44) and 181K (16:10). | 2 | 3 | 5 | 10 |
| `gates_per_step` | 3.1 | **The run's own: 1.17**, 41 land gates for 35 steps, 3 failed. | 1.1 | 1.4 | 1.2 | 1.8 |
| `gate_lane_other` | 84 | **The run's own: 34**: 12 process-tier runs, 21 measurements, 1 kit test. | 25 | 11 | 64 | 37 |
| `queue_wait_ratio` | 1.50 | **The run's own: 0.82**: build 1.20, gate 0.25. Before the pause 0.31; after the resume 1.48. | 0.85 | 1.26 | 1.44 | 0.94 |
| `suites` | **temper:** focused 1,862 / 3.495 s → 1,727 / 3.974 s; fuzzy 50 / 7.536 s → 48 / 6.822 s; browser 10 / 14.363 s at the end | The default count fell after forge-kit moved worlds to skein (`4b074713d`). The browser profile is browser-1's. | | | | |
| `proofs` | unit 15, world 5, joint 15, process 3; 0 below plan | | unit 16, world 8, joint 19, process 3 | | | |
| `report_lines` | 12 median, 13 p90 | 83 reports, each copied whole into `log.md`, 37.6% of its bytes. | 12, 15 | 12, 13 | 12, 14 | 12, 18 |
| `lines` | code +2,439 −286; tests +7,047 −659; data +203 −8 | 2.9 test lines for each line of code. | code +17,016; tests +9,125 | | | |
| `contexts` | 122K median, 177K p90 of the sessions' peaks; 28 handovers | At its 30th request a thread's median context was 89K. 35 of 42 threads passed 100K. | 119K, 184K; 25 | 138K, 188K; 33 | 122K, 154K; 38 | 186K, 332K; 150 |
| `pin_lag` | temper: skein 88, smith 31, jig 11 | Fetched after the release, while the owners land. | | | | |

**Not comparable, and why:**
- **`gates_per_step`, `gate_lane_other`, `queue_wait_ratio`.** `heavy` writes every task on the machine into the active run's `logs/heavy.tsv`, and `numbers.sh`'s `queue_rows` reads all of it. 2,673 of its 3,177 rows (84%) were other work's: the accounts-2 owners 1,992, hearth's owners 657, after-landing 12, design 14. The figures in bold above count only tasks whose working directory is under this run's directory (proposal 1).
- **`chain_step_min`, `chain_step_max`.** A step's span starts at the ledger's thread, which is the last one for a step handed over or begun again after a block. The metric gives 13.5 10.3 minutes; it took 192, from its first thread. It gives 08.3 2 minutes, against about 98. The median of the first threads' spans is not recoverable for seven steps.
- **`wall_h`** includes the 16.3-hour pause.
- **`escalations` by kind** follow each block's Kind line. The implementers filed eight plan defects as "cross-session scope".
- **`cost_claude`** counts this retrospective only up to when the metrics ran, and the design session `74881b0e` at $0, as it is not on this machine.
- **`report_lines`' p90.** The log reader counts 12, not 13, by either convention of line endings. A small thing, noted so that the next comparison does not chase it.

**What explains the largest differences:**
- **Hours a step, 0.26 against tools' 0.20.**
  - The chain was the run, as in tools. Its 9.2 running hours were:
    - its own work for 7.3 hours: 13:50 to 16:16, and 10:34 to 15:25 on 10-20;
    - the wait for forge-kit inside the running time, 2.0 hours.
  - Within the work, 13.5 took 3.2 hours, and the eleven other steps about 22 minutes each.
- **The coordinator's 2.96M a step.**
  - **Context.** Its 697 requests averaged 149K of input, against tools' 110K. It compacted only at 182K and 181K, under its profile's 200K.
  - **Blocks and answers.** 33 blocks against 13. Handling unblocks took 126 requests (17.3M), blocks 51 (6.0M), steers 60 (8.6M).
  - **Bookkeeping.** `plan set`, `plan landed` and `plan log` took 163 requests (22.5M). There were 40 `plan landed` calls for 35 steps: four steps took two or three, for a ledger that named another branch, then a dirty tree.
  - **Python that writes files** was in 161 requests, holding 45.7% of its output tokens. That is an upper bound, by request; tools' 40% may have been counted otherwise.
- **The implementers' 6.47M a step,** against 5.64M.
  - **Diagnosis.** The three costliest threads took 48.1M (21.2%). w13 handled 06.2 to 06.4 through four plan-defect rulings. w27 diagnosed 09.2's stall and built 09.s1. w36 ran 13.5's first eight diagnostic runs.
  - **13.5 alone.** Its six threads took 42.1M in 406 requests.
  - **What did not move.** A thread's first prompt was 16.9K, as in tools. Its context at the 30th request was 89K, against 96K.
  - **Polls.** Requests that only polled a running command were 4.1% by tools' count (`write_stdin` alone), 6.4% with the cell waits.
- **Rulings, 1.49 a step against 0.91:** 20 plan defects, six test-shape rulings, six diagnosis budgets (two of them also plan defects), and four for the kit and the environment.

## 4. What worked

1. **The real loop found what the worlds below it missed, and each defect went down to its owner.**
   - 08.3's story found that temper-service kept a call's stream after its peer ended. 08.s1 fixed it in eleven lines, with a unit test.
   - 09.2's story found the service's owner work stuck behind a refused input. 09.s1 fixed it, with a lower scheduling proof.
   - 13.5's story found a forge-domain wake lost when it overtook a fresh read (13.s3), then the stale decision that fix emitted (13.s4). Both were fixed in `temper-engine-domain-forge`, with unit tests in its `src/tests/change.rs`.
   - 09.3's story showed that the lock ended before the process exited. The designer judged that right, and tools.md 8.3 now says it (`6a287983a`).
   - No real-loop test tests a corner case. The smoke rule (ruling 130, testing.md 2.4) dropped six stories, 09.4 among them, and moved each case to the world that owns it.
2. **Every design question was answered by a designer and reached the design, the same hour.** Tools' proposal 3 held.

   | Question | Asked | Answered in the design |
   |---|---|---|
   | The worlds' sum below the domain, from 08.2 | 14:25 | 14:37, testing.md 6.1, `2f97ab8bb` |
   | What frees a call's stream at its peer's end, from 08.3 | 15:11 | 15:27, jig `bfc3a82fc` and temper `200916de5` |
   | Where a FILE in a `--call` call is read from, from 05.6 | 15:15 | 15:27, tools.md 8.1, `200916de5` |
   | The real loop is smoke, the user's decision | 16:43 | 16:56, testing.md 2.4, `2f71813ad` |
   | When DIRECTORY's lock ends, from 09.3 | 17:14 | 17:26, tools.md 8.3, `6a287983a` |
   | The real loop counted apart, the user's L | 17:32 | 17:43, testing.md 6.1, `bab11df83` |

   - Interim rulings (109, 118, 119) let the work go on meanwhile.
   - `plan wake designer` refused at 14:27 ("no designer recorded"). The director joined one at 14:28, and skein `31ae5ab` fixed the defect by 14:36.
3. **Measured costs, at their cause.**
   - Ruling 112 showed that the 5.059 s "miss" below the domain was a build-lane measurement under load. Run idle, it was 3.91 s. testing.md 6.1 now names the measure.
   - 13.s1 profiled the one-process world and cut its adoption story from 1.008 s to 0.724 s, in the code, with no build setting changed.
   - Each of the nine increments that added a real-loop story gives its median of three runs alone in the commit body. The two that added world tests (08.1, 13.1) give single runs.
   - The 14 client stories take 0.47 s together, against the Pulse's 7 s.
4. **Ready checks stayed in their lane.** Tools' proposal 4 held:
   - Of the supervisor's 29 ready-check tasks, none ran the whole workspace or the fuzzy profile. Eleven ran `rdeps(…)` filters, and two were stress runs of one crate.
   - They were 13.4% of the run's build-lane waiting, against 51% in tools.
   - Rulings 105, 106 and 138 corrected sessions 10, 09 and 13 before they started, five points each.
5. **Test names.** Of the 113 tests this run added, none has `_and_`, under `land`'s refusal from 15:06 and the README's rule (tools had 19). The run also removed one such name, `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, in 14.1's shards. After 09.s2's gate, no story indexes `calls()` or passes a bare number to `task(`.
6. **The process tier stayed green.** It ran twelve times in the run's gate lane.
   - Its one failure, at 11:22 on 10-20 ("native signing failed: Again"), was traced within 9 minutes to a fix already on `main` (ruling 139).
   - Forgejo 17 and its runner were installed, so no Proof was "unavailable".
7. **Gates.** 41 land gates for 35 steps. Each of the three failures was fixed and landed within 5 to 11 minutes: 08.3's world name, ruling 127; 09.s1's order of two exits, ruling 134; and 13.s3's clippy doc lint.
8. **The pause cost little in tokens.** The coordinator made 3 requests between 18:07 and the resume. The supervisor's four quiet checks cost $2.82.
9. **Compaction in place.** One supervisor session ran from claim to release, with 13 compactions at 167K to 172K. Its restores, to its next `plan watch`, cost $13.67 together; that includes the rulings written in the same windows.
10. **The supervisor's Codex helpers** took the hourly checks after the resume, the diagnosis of 13.s4, 13's ready check and the "Done when" run: six runs, 45 minutes. Its Claude sub-agents cost $9.27, $0.26 a step, against about $1.06 in tools.

## 5. What did not

1. **Plan defects fell on the sessions that started at the claim, which nothing had run.**
   - **What.** 20 rulings fixed a plan defect, 0.57 a step, against tools' 0.05.
     - 14 fell on sessions 05, 06 and 08, which started at the claim. temper 06.1 blocked four times between 13:58 and 14:25 (rulings 103, 107, 108, 110), and session 06 met seven of the first thirteen blocks.
     - 08.3, the plan's first meeting point, took about 98 minutes across three blocks and a design answer. 06.2 took about 57.
     - The ready-checked sessions (09, 10, 13) met six. Four of them came up only when a story ran: 134's exit order, and 143, 145 and 148 in 13.5.
   - **Cause.** The sketches assumed how the harnesses behave over time, and only running them shows that:
     - `Root::new` makes a linked root (107); the referee needs the agent's tree to show its link (108) and the cards offered for release (114);
     - `Steps(1)` does not advance the clock (108); the admission envelope is needed before the first turn (104);
     - a chat's run parks rather than finishes (113), and reads its messages in one turn before it parks (115);
     - an ended task reads as historical (121), and a cut agent must be reloaded (122).

     The planner's two final checks found 14 and 27 mismatches of names and types; neither ran a sketch. development.md gives a ready check only to a session that does not start at the claim.
   - **When.** The first block came at 13:52, four minutes after the claim. The 14:50 check named session 06 as the most blocked. Each ruling came within 1 to 8 minutes of the supervisor seeing the block, but for one missed after a compaction (item 11).
   - Proposal 4.
2. **The landing through the loop, 13.5, took 3 hours 12 minutes: a quarter of the chain.**
   - **What.** Six threads (w35 to w41), five blocks, two engine fixes (13.s3, then 13.s4 to correct it) and 42.1M of input. It made 37 build-lane runs of one test, 35 of them failing, which waited up to 637 s each behind the accounts-2 owners. Its Size was 120 lines; it landed 127.
   - **Cause.**
     - The story was the first to run a landing whole, through every part, so each run showed the next failure: the tree's window, the decoder, the checkout roots, a title past its bound, the clock, the concurrency, the lost wake, the stale decision, the missing review rule, and the parent's `finish`.
     - A Size counts lines, and a meeting point's cost lies in the seams it crosses.
     - The diagnosis budget (ruling 110) was granted again by ruling after each new symptom: 140, 143, 149, and reset by 145 and 148. Ruling 148 told the implementer to report a symptom the README's budget already covered, which cost block 149 (the supervisor's proposal of 15:14).
     - The shape of 13.s3's fix was the supervisor's, from the implementer's proposal. It needed 13.s4 within the hour. The Codex diagnosis behind 13.s4 (`rl-diag-1417`) said the design states freshness, but not the handshake, and ruling 148 quotes the design as if it did.
   - **When.** From 13.5's first block at 12:25; each ruling came within 1.1 to 5.8 minutes of the block reaching the supervisor.
   - Proposal 3.
3. **The end of the wait woke nobody, for 2 hours 25 minutes.**
   - **What.** The supervisor's 22:19 check ended its turn with a watch running. That watch printed the next hourly event, then "[process exited while detached; exit code unknown]". The session took no turn until the director's message at 10:04 on 10-20. So no check ran for 11.7 hours.
   - The await was registered (`awaits.tsv`: forge-kit's skein-05.5 and temper-05.2, for the coordinator), but the supervisor's `plan watch` is what reads it, and the coordinator was paused until the supervisor resumed it.
   - temper-05.2 landed at 07:40. forge-kit's closing repin landed at 09:57. The director's message named both, and the run resumed at 10:05. temper 13 needed only temper-05.2 (README).
   - **Also.** skein-05.5, the other awaited step, had landed at 09:27 on 10-19, before the claim. The quiet checks still named it as awaited.
   - **Cause.** Within the run, only the supervisor's session read the await. When its process was gone, the director's message was the one way left.
   - **When.** First visible at the missed hourly check after 22:19; acted on at 10:04, by the director.
   - Proposal 5.
4. **After the resume, another owner's work filled the machine.**
   - **What.** From 10:05 to 15:44 on 10-20, the accounts-2 owners ran 683 of the machine's 828 tasks.
     - The run's tasks waited 8,182 s, 82% of it behind theirs, against 1,868 s before the pause.
     - The gate lane was busy 45% of the time, against 14% before.
     - heavy refused a 13.3 build at 11:20 with the disk at 37 GB, under the 40 GB floor. accounts-2's worktrees held 36, 32, 29 and 21 GB.
   - **Also.** A gate task holds the lane while other tasks finish and the load settles: 35% of the run's gate time was that hold. 13.4's first measurement held the gate lane for 256 s to run a test of 2 s.
   - **Cause.** The owners began on this machine during the pause, and nothing divided the lanes between a run and the owners.
   - **When.** The 12:27 check saw 6,442 s waiting against 5,551 s running, and named no steer.
   - No proposal beyond 1: the director orders owners' footprints (owners.md 4.5), and it knew the run was on this machine.
5. **The metrics count other work's tasks.**
   - **What.** `queue_rows` reads the run's whole `heavy.tsv`, and heavy writes every task on the machine there.
     - `gates_per_step` 3.1 counts 107 land gates: the run's 41, the accounts-2 owners' 48 (41 in jig, 7 in temper), hearth's 12 and the designers' 6 in skein.
     - The pulse's figures had the same fault. The 12:27 check wrote "48 land gates include other runs" and moved on.
   - **Cause.** Before the owners, a machine ran one plan, and every task was the run's.
   - **When.** The 12:27 check.
   - Proposal 1.
6. **"Done when" failed on other work's lines.**
   - **What.** Five commands failed at 15:47:
     - two on lines other plans wrote: browser-1's profile lines, and accounts-2's `_and_` names;
     - one on a package another plan removed: forge-kit 07.1's reconcile world;
     - two on the plan's own literals: one never written, one renamed by 13.5.
   - The first try of item 5's corrected run was refused, since the earlier run's logs lay in the checkout. development.md 5.4 has said since 13:43 (`1d105bc8`, six minutes after the plan landed) that a command must not write inside the worktree it measures.
   - **Cause.** development.md 5.2 forbids an item to name what a later session may move, but not what another plan may. The plan was written before the rule on logs, and `plan check` does not check it.
   - **When.** 15:47 to 16:03. Rulings 150 and 151 settled them within 18 minutes.
   - Not proposed: owners keep no "Done when" (the accounts decision, its proposal 6). The supervisor's proposals of 16:10 on these points stay with development.md.
7. **Test shape took six rulings, and the gate saw only part of it.**
   - **What.** The rulings, each from an hourly check:
     - 117, from the check of 14:50: act once and assert after;
     - 126, from 15:46: four points;
     - 133, from 16:52: named positions, with 09.s2's gate;
     - 137, from 17:57: named counts, and no `_` slice patterns;
     - 142, from 12:27 on 10-20: with 13.s2's gate;
     - 146, from 13:33: a module of its own, and a named deadline.

     About 15 steer points. Assertions on the way were sent three times before 13.s2's gate checked them, and bare positions twice before 09.s2's.
   - **What is left.**
     - Three `_` slice patterns over calls (`tests/real-loop/tests/lost.rs:30`; `crates/temper/tests/process.rs:61`, `:103`).
     - At least eight expectations computed by the code under test, two tables that branch on a row, five one-sided bounds and five assertions before the act. Most are in `crates/temper-client-shell-cli/src/tests.rs`, whose tests mostly landed before ruling 117 (15:06).
     - The gate's checks cover only `tests/real-loop/tests`.
   - **And outside the run.** `land` checks test names only when the branch carries the active plan's `Plan:` trailer (`land`'s `pulse_line`). The accounts-2 owners, landing on the same `main` with no plan, added ten `_and_` names from 11:16 to 15:34.
   - **When.** First at the 14:50 check.
   - Proposal 2.
8. **Two defects of `temper ctl`'s host showed after its landing, and others fixed them.**
   - **What.** A sign-in answered `Again` was fatal: 13.3's process proof failed on it at 11:22, and after-landing on 13.2's commit at 11:16. `e9bb42960` fixed it at 11:20, outside the ledger. Many calls exhausted its sockets: after-landing panicked on "bounded sockets" at 12:24. `5761ae077` replaced the run's in-memory transport with jig's `Transport` at 14:54.
   - **Cause.** 05.1 to 05.6 proved the host by unit tests, which neither answer a sign-in NotReady nor make many calls. The process tier reached `temper ctl` only from 10.1 (15:55), and the real loop only from 08.3 (16:05).
   - **When.** 11:16 on 10-20, by the after-landing tier: skein's `23e80d5` had `land` run it, and temper's script arrived at 11:09 (`4833827a6`).
9. **The coordinator's overhead rose.** 2.96M a step (section 3). Ruling 111's `Answer to` heading lacked its time, so `plan unblocked` left the block open, which cost block 113. "A ring is one command" (client-api's proposal 5, skein `2349030d`, 14:44) reached the run only at the resume: the supervisor kept the kit at `31ae5ab`, since the commit named no run. These belong to development.md's coordinator; not proposed.
10. **Implementers ran past 100K within an increment.** 35 of 42 threads passed 100K. 13.5's diagnosing threads peaked at 226K (w36) and 217K (w37). w27 was handed 09.3 and 09.s1 at about 161K, until ruling 133 sent them to a fresh thread; its handover line says 43K, after two compactions. Owners give each implementer one task (owners.md 4.2), so not proposed.
11. **Smaller things.**
    - **Release.** The supervisor's ready-check worktrees `wt/temper-09r` and `-10r` were dirty at the release, which removed them by force (ruling 152).
    - **Compactions and the watch.** After four of them the supervisor found its watch not running, though the notes put several stops down to events that had come in. After the fifth, 06.2's block of 15:09:06 went unanswered for 17 minutes (ruling 121).
    - **A lost report.** One Codex hourly report was lost: `-o` resolves against the shell's working directory, not `-C`'s (`rl-hourly-1333`). The supervisor recovered it from the log (proposal 6).
    - **A duplicate tier run.** The supervisor's tier run of 14:42 on `57b9c692e` repeated after-landing's run of the same commit, which had ended at 14:42:19.
    - **The claim came before the kit.** Tools' proposals 6 and 8 landed in skein at 14:04, after the claim, and reached the run with the kit at 15:06. live-text's decision said real-loop would be claimed once they were applied. The claim's `plan check` (`2bd29bd`) did not yet refuse a claim with an adopted proposal unapplied; `4134983` added that at 14:18.
    - **Queue reads.** The coordinator ran `pueue status --json` unfiltered, and implementers ran `pueue log`, while queued tasks still carried their callers' environment (ruling 144, until `a46635c8`). No read of another process's environment was found.
    - **Records.** `supervision.md` calls 13.s1's commit "13.1" (14:50 check). `decisions.md`'s "For the designer" still lists three points that landed in `116d0a645` before the claim. Ruling 150 says "36 ledger commits"; there are 35.

## 6. Decisions taken during the run

| Decision | Ruling | Reached the design? |
|---|---|---|
| The worlds' sum below the domain is measured idle, and is the worlds' guide, never a landing's | 109 (interim), 112 | **Yes,** testing.md 6.1 (`2f97ab8bb`). |
| The real loop counted apart, at a standing 6 s, each story at 1 s; the others keep 4 s (the user's L) | 136 | **Yes,** testing.md 6.1 (`bab11df83`). |
| A peer's end ends its unfinished call; the server frees the stream | 118 (interim), 123 | **Yes,** jig `protocol/client.md` 17.4 (`bfc3a82fc`); temper `client/README.md` 8 and tools.md 8.2 (`200916de5`). jig's reference service is a `later.md` row. |
| A call that `--call` reads gives no FILE argument; a FILE is read as given, under a root opened at the start | 111, 119 (interim), 123 | **Yes,** tools.md 8.1 (`200916de5`). |
| The real loop is smoke; its corner cases are tested below (the user's decision) | 130, 132 | **Yes,** testing.md 2.4, tools.md 11.1 and 11.3 (`2f71813ad`). |
| DIRECTORY's lock ends at the `Close` of its descriptor, before the call exits | 135 | **Yes,** tools.md 8.3 (`6a287983a`). |
| An absent DIRECTORY is opened before its private open; the window in which another process removes and remakes it is accepted | 102 | **No.** A `later.md` row (an existing-only private open in skein's io); no comment at `directory.rs:227`, no STATUS.md row. |
| A wake that overtakes a fresh-read chain is kept and replayed; an overtaken chain sends no decision | 145, 148 | **No.** domain/forge.md 5.3 states freshness; the rule is in `decisions.md` and the files of 13.s3 and 13.s4. |
| reconcile's world leaves the worlds below the domain | 150 | **No.** testing.md 6.1 still names it; `later.md`'s last row. |
| Owner work resumes behind a refused input | 131 | Already in the design (domain/root.md 4): a code defect. |
| Two exits after one commit have no order | 134 | Not design: a test's assertion. |
| The other worlds' 4.04 s | 151 | Not design: a measured miss, for the user and temper's owner. |
| Test shape: act once; independent expectations; named positions, counts and sizes; no `_` slice patterns; shared setup | 117, 126, 133, 137, 142, 146 | Not design: testing-strategy.md 10 says them. README rules, and two gate checks. |
| Diagnosis budgets | 110, 125, 129, 140, 143, 149 | Not design: process. |
| The coordinator checks no tests; kit moves; no other process's environment | 116, 120, 141, 144, 147 | Not design: process. |

For the user A to K were decided before the claim, by the director, at 13:40:09 (`e0bc5ebb9`) and in the plan's landing. L was the user's, at 17:32:40 (`08e91e24a`), as the director amended it.

## 7. The proposals adopted after the last retrospective

The last retrospective before the claim was temper's tools (`d222c5fd1`, 12:29:29), six minutes after smith's live-text. The director adopted all eight proposals at 12:30 (`e139ccaf4`).
- The text parts landed in development.md as skein `656ef69` (13:14), in the claim's kit.
- The tool parts landed after the claim: `ebf24d5` (14:04) for proposals 6, 7 and 8, and `0b006b8` (14:34) for 5. Both reached the run with the kit at 15:06.

Figures a step divide by landed steps: tools 43, real-loop 35.

| Proposal | Applied | Meant to move | tools | real-loop | Moved |
|---|---|---|---|---|---|
| 1. Split a design's steps into plans before drafting; draft only the first plan's sessions | `656ef69`: development.md 2.2, 5.1 | the planner's dollars a landed step, $4.0 to under $2 | $4.0 | $3.61 | **Little.** The plan was the second part, at 27 steps and a chain of 12. But $81.71 of the planner's $126.50 went to refreshing six drafts that the tools planner had written for this plan, at Opus; a new draft (session 14) cost $2.69. |
| 2. Map the design's outputs to the upstream types; a Size counts the types it covers | `656ef69`: 5.4 | design questions during a run, 8 to 2 or fewer; steps past twice their Size, 7 to 2 or fewer | 8; 7 | 4 from the run, and 2 decisions of the user; 5 | **In part.** The map (`findings/code-checks.md`) ran at 12:33, after the drafts; Sizes stayed line ceilings. The questions were about lifecycle (when a stream ends, when a lock ends, where a FILE is read, how a sum is measured), which no type map shows. Past twice their Size: 05.1, 06.2, 08.4, 10.1, 10.2. |
| 3. A decision taken in a run without the designer reaches the design | `656ef69`: 10.4, 2.7; M to Q into tools.md (`a959f6e10`, before the plan) | decisions in force but not in the design at the end, 5 to 0 | 5 | 2 rules of behaviour (102, 145 and 148) and 1 wording (150) | **Yes, mostly.** A designer answered each question here, and each answer landed in the design within 16 minutes of its question. |
| 4. A ready check runs only its step's crates (`rdeps`), in `<repo>-<NN>r` | `656ef69`: supervisor.md 5 | `queue_wait_ratio`, 0.85 to under 0.5; the supervisor's share of build-lane waiting, 51% to under 20% | 0.85; 51% | the run's own 0.82: 0.31 before the pause, 1.48 after; 13.4% | **Yes for the share,** and for the ratio until the owners came. No ready check ran the whole workspace. |
| 5. Project the chain at a chain step's own time | `656ef69`: 5.2; `0b006b8`: `chain_step_min` | the projection within 25% of `wall_h` | 23% to 35% | 4.8 h against 9.2 running hours (52%) | **No, but the step figure held:** 11 chain steps took about 22 minutes each, against the 24 projected. 13.5 took 192. The new metric reads 9.5 minutes because it counts each step's last thread (section 3). |
| 6. `land` refuses a test named with `_and_` | `ebf24d5`: `land`, from 15:06 | test-shape steer points, 8 to 3 or fewer; new tests named with "and", 19 to 0 | 8; 19 | about 15; 0 | **Names yes, steers no.** `land` checks only a plan's branches: the owners added 10 (proposal 2). |
| 7. A check that reads nothing fails; a pin check matches the repository | `656ef69`: 4.5; `ebf24d5`: `plan check` | "Done when" items that pass without reading, 1 to 0 | 1 | 0 | **Yes.** Item 5's third command failed, rather than passing, when reconcile's world was gone; the designer had added the check to it (`bab11df83`). |
| 8. `running_h` ends where `wall_h` does | `ebf24d5`: `numbers.sh` | `running_h` ≤ `wall_h` | 8.5 > 8.3 | 9.2 ≤ 25.9 | **Yes.** |

**Proposals of earlier retrospectives that reached the run late or not at all:**
- client-api's proposal 5, a ring as one command (`2349030d`, 14:44), reached the run at the resume, with `74e48147`.
- live-text's proposal 3 (threads get the code graph) is not built.
- client-api's proposal 8 (btrfs checks) is not built; this machine is ext4.

**Changes to the process that landed during the run** (skein's `docs/foundation` and `scripts/dev`, 34 commits from `2bd29bd` to the release):

| Commit | Landed | What | Did the run take it? |
|---|---|---|---|
| `ebf24d5`, `0b006b8`, `4134983`, `2cedfab`, `baaebb7`, `e002687`, `e1053a7` | 14:04 to 14:34 | Tools', client-api's and live-text's tool parts: `land`'s name refusal, `running_h`, chain spans, the refusal of a claim with a proposal unapplied, waits timed by their return, the aims line, the fresh designer's reading | **Yes,** with `31ae5ab` at 15:06. |
| `31ae5ab` | 14:36 | `plan wake` starts a fresh designer | **Yes,** ruling 116: the defect the run met at 14:27. |
| `2349030d` | 14:44 | A ring is one command | **At the resume:** "names no run" (`supervision.md`, 15:46 check). |
| `7360708` to `bc6c16c`, then the owners' commits of 10-20 | 16:29 to 10-20 10:54 | owners.md, the owners experiment | Not a role of this run. |
| `153ce4a` | 10-20 08:14 | heavy's disk-floor refusal names what fills the disk (accounts' proposal 3) | **Yes,** at the resume; it named accounts-2's worktrees at 11:20. |
| `23e80d5` | 08:50 | `land` runs the after-landing tier | **Yes,** at the resume; it found the sign-in `Again` defect at 11:16. |
| `c381220`, `94bffac`, `42b2a84` | 09:10 to 09:29 | kache pinned, set up in the gate lane, an old store cleared | **Yes,** at the resume. |
| `7227d90`, `88a5e38` | 10:40, 10:52 | `land` exit 9 for a refusal; heavy lets the load settle | **Yes,** ruling 141, met at 13.3 and 13.4. |
| `a46635c` | 13:17 | heavy leaves credentials out of queued tasks | **Yes,** ruling 147, met at ruling 144. |
| `e408e2c` | 15:32 | after-landing releases x1's lane target | No: x1 only, and the run was at its end. |

**What this says of the loop.** Every adopted proposal of tools reached the run, within 77 minutes of the claim.
- Proposals 3, 4, 7 and 8 moved their metric.
- Proposals 2 and 6 moved one of their two.
- Proposal 1 moved a little, and proposal 5 not at all. Both are about planning, and missed for one reason: the planner's figures covered the steps the plan could see, while the cost lay in drafts written before the split, and in a meeting point no step measured.

## 8. Proposals

The supervisor wrote twelve proposals (`supervision.md`, 14:50, 15:46, 17:57, 15:14 and 16:10). Two are carried here: proposals 3 and 6. Nine concern development.md's own roles and tools (`plan ruling`, the pulse, the coordinator's steers, `plan status`, the watch, "Done when", the ready check's worktrees), which no new work uses. One, Done-when logs written outside the checkout, is already in development.md 5.4.

1. **`heavy` writes, in each row of its log, whose task it is (the run or the owner whose directory it runs in). Every count of the queue reads only its own rows: `plan metrics`, the pulse, and an owner's closing note.**
   - **Evidence:** section 5, items 4 and 5.
     - 84% of the run's `heavy.tsv` was other work's.
     - `gates_per_step` read 3.1 against the run's own 1.17, and `queue_wait_ratio` 1.50 against 0.82.
     - owners.md 6 has the director compare owners' notes with the old runs; that comparison needs counts of each one's own tasks.
   - **Changes:** skein's `scripts/dev/heavy` (the row), `numbers.sh` and `metrics.sh` (the reading); owners.md 4.5 and 6.
   - **Moves:** comparability. This run's three queue metrics read as the recount gives them, 1.17, 34 and 0.82; an owner's closing note can state its queue time.
2. **`land` refuses an added test named with `_and_` on every landing, whether the branch belongs to a plan, an owner or neither; a plan's Pulse may only add to the pattern.**
   - **Evidence:** section 5, item 7.
     - None of this run's 113 new tests has `_and_`, under the refusal.
     - On the same `main`, owners' landings, which the refusal does not see, added ten from 11:16 to 15:34.
     - testing-strategy.md 10's one behaviour per test binds every repository, not only plans.
   - **Changes:** skein's `scripts/dev/land` (`pulse_line`'s default); owners.md 4.4.
   - **Moves:** new `_and_` test names on temper's `main`, 10 a day to 0.
3. **A task names its diagnosis budget, in runs and minutes, for each new symptom. A new symptom starts a new budget without asking, and the implementer stops only for a fix outside its files, or for a symptom whose cause the budget did not find.**
   - **Evidence:** section 5, item 2.
     - Rulings 129, 140 and 149 answered a block only by granting ruling 110's budget again; 143, 145 and 148 reset it beside their fixes.
     - Ruling 148's "report it" cost block 149, which the README's rule already answered (the supervisor's proposal of 15:14).
     - Two of 13.5's five blocks waited on a ruling that restated a rule.
   - **Changes:** owners.md 4.3 (the task's "on a doubt") and 4.2 (an investigation's task).
   - **Moves:** blocks answered only by a rule the implementer already had, 3 to 0.
4. **Before an implementer starts a task, the owner runs the task's test sketches against `main` in a scratch worktree: compiled, and each new story's harness driven to its first assertion. The task goes out only with the sketches that ran.**
   - **Evidence:** section 5, item 1.
     - The sessions that started at the claim, which nothing had run, met 14 plan-defect blocks; 06.1 met four in 27 minutes.
     - The ready checks of 09, 10 and 13 compiled their sketches and corrected five points each before the steps began.
     - The defects lay in the harnesses' lifecycles, which only running a sketch shows. The planner's literal checks found 41 mismatches of names and types, and missed them.
   - **Changes:** owners.md 4.1 ("Settle first") and 4.3 (the task's sketches).
   - **Moves:** plan-defect blocks a step, 0.57 here (by the rulings), to under 0.1; an owner's note counts the tasks it changed after they started.
5. **A wait on another's landing is ended by the landing, not by the waiting session: `land`'s after-landing hook reads each waiter's awaits (a plan and step, or a path) and wakes the waiter whose await the landed commit meets.**
   - **Evidence:** section 5, item 3.
     - temper-05.2 landed at 07:40, and the run resumed at 10:05, on the director's message.
     - The supervisor's session had taken no turn since 22:20, its watch's process gone, and the await was read only by that session's `plan watch`.
     - Owners wait on each other's landings too ("Upstream first", owners.md 4.1 and 4.5), and are woken today by the director's message.
   - **Changes:** skein's `scripts/dev/after-landing` (or `land`), with an awaits file under each owner's notes; owners.md 4.5.
   - **Moves:** minutes from an awaited landing to the waiter's resume, 145 to under 10.
6. **A Codex task's `-o` report path is absolute.**
   - **Evidence:** section 5, item 11. `rl-hourly-1333`'s report was lost: `-o` resolved against the shell's working directory, not `-C`'s (the supervisor's proposal of 15:14). owners.md 4.2 has every implementer write its report with `-o`.
   - **Changes:** owners.md 4.2; `tmp/director/codex-helper.sh`'s pattern already does it.
   - **Moves:** Codex reports lost, 1 to 0.

## 9. Debts left in the code

For temper, which has no `docs/development/tech-debt.md`, and for its owner. "Listed" says whether `later.md`, a STATUS.md or a debt file already has it. Ordered by what they cost the next work.

1. **The other worlds below the domain are 0.04 s past their 4 s.**
   - `tests/one-process`'s `an_adopted_repository_gets_its_change_pushed_through_git` takes 0.899 s, 22% of the sum. 13.s1 had cut it to 0.724 s, and it rose again during the run; forge-kit's changes to `tests/one-process/src/world.rs` (`4e16e4eba`, `20d88b6bb`, `5dec8f6f0`) are candidates, not traced.
   - Next: `a_person_signs_in_by_the_printed_link_and_a_chat_is_answered` 0.264 s, and `a_signal_mid_run_drains_and_the_next_start_settles_the_claims` 0.253 s.
   - Listed: ruling 151; `later.md`'s 4 s row, whose wording ("if temper 06 … lands") is now past.
2. **The landing story costs a quarter of the fuzzy sum.** `tests/real-loop/tests/fuzzy_landing.rs:8`, `a_chat_lands_its_change_on_the_forge`, takes 4.1 to 4.6 s: four agent lifetimes, seven LLM turns, 23 hosted git calls. Its world's `tests/real-loop/src/landing.rs:72-107` sets 33 limit overrides, 2 of them explained, so each new forge story retunes them by trial. Not listed.
3. **testing.md 6.1 names reconcile's world** among the 4 s worlds, though it is skein's since `4b074713d`. Listed: `later.md`'s last row.
4. **Test shape left in this run's tests.**
   - `_` slice patterns over calls: `tests/real-loop/tests/lost.rs:30` (with a bare `2` at `:36`); `crates/temper/tests/process.rs:61`, `:103`.
   - Expectations computed by the code under test, in `crates/temper-client-shell-cli/src/tests.rs`:
     - `help_opens_nothing` at `:520-548`, which retypes `host/input.rs`'s help render;
     - `:608-614` (`root::next_deadline`), `:184-193` (`* 2 + 5`, as `host.rs:697`) and `:648-665` (the backoff modelled in the test);
     - `crates/temper/tests/process.rs:104-111`, which takes `first_sent` from the output under test.
   - Tables that branch on their row: `tests.rs:11-23`, `:477-493`.
   - One-sided bounds: `tests.rs:108`, `:191`, `:292`, `:304`, `:333`.
   - Assertions before the act: `tests.rs:136-145`, `:224-244`, `:345-355`; `tests/real-loop/tests/tools.rs:28-34`.
   - A port literal the "Done when" grep cannot see: `tests/real-loop/tests/landing.rs:13`, `([127, 0, 0, 1], 41_234)`, never bound.
   - `scripts/check.sh:9-17` checks only `tests/real-loop/tests`, and has no rule for slice patterns.
   - Not listed.
5. **Two call drivers, and their copies.**
   - `crates/temper/tests/support/ctl.rs` (532 lines: `Calls`, `Step`, `Call`) and `tests/real-loop/src/calls.rs` (640 lines: `Caller`, `Step`, `Argv`) are parallel vocabularies. Each defines `OUTPUT: usize = 65_536`, and the literal recurs about 12 times unnamed.
   - The adopt argv is typed in six places, the chat argv in two.
   - `tests/client`'s `call` helper is in `agent.rs:17` and `agent_recovery.rs:40`, and inlined at `agent_turn.rs:27-28`.
   - The standalone tree fixture is in three places: `crates/temper/src/tests.rs:127`, `tests/support/mod.rs:37`, and `tests/real-loop/src/fixtures.rs:83` (the last on purpose).
   - Not listed.
6. **A dev-dependency cycle.** `crates/temper/Cargo.toml:43` dev-depends on `../../tests/real-loop`, which depends on `temper`, since 13.3 shared `landing.rs` with the process tier (`48543c5ac`). `temper`'s tests build about 33 more crates. The line is not `workspace = true`, unlike its siblings. Not listed.
7. **Files near or past 800 lines.**
   - `tests/client/src/fixtures.rs`: 527 → 762 lines (+226 from this run). Every client story edits it.
   - `crates/temper-service/src/tests/fixtures.rs`: 841 → 840, already past; 09.s1 had to put its 35-line setup in `tests/engine.rs:171`.
   - `crates/temper-engine-domain-forge/src/domain.rs`: 3,594 lines, +24 from this run. Listed in protocol's `later.md`.
   - `tests/real-loop/src/calls.rs`: 640 lines, and about fifteen `later.md` rows will add to it.
8. **Decisions in force that the design does not say** (section 6):
   - the two-open window of ruling 102 (`crates/temper-client-shell-cli/src/directory.rs:227` has no comment on it);
   - the overtaken chain's rule of 13.s3 and 13.s4.

   Listed: `later.md` for the first only.
9. **Magic numbers in `temper ctl`'s host.**
   - `host.rs:94` (`* 3`), `:96` (`16`), `:106` (`+ 8192`), `:697` (`* 2 + 5`), and `limits.rs:77`, `:80`, with no comment.
   - The 30 s deadline is written again beside `limits::DEADLINE`: "within 30 s" at `host.rs:485`, and `Duration::from_secs(30)` at `directory.rs:61`.
   - Not listed.
10. **Stale rows.**
    - temper's `docs/design/STATUS.md` still names real-loop as the owner of delivered work: lines 15, 32, 33, 42 and 45.
    - The dropped 09.4 has no `later.md` row; ruling 130 names the lower tests that cover it.
11. **Not code.**
    - 39 local `real-loop/*` branches in temper. 37 are merged. Two are not:
      - `real-loop/temper-09-4-unread` (`9b9cba32a`), the only copy of the dropped 09.4's 247 lines;
      - `real-loop/temper-10-2-signals` (`ed5325404`), which ruling 128 left unlanded.
    - The designers' worktrees `temper/.claude/worktrees/real-loop-{lock,smoke,sums}`, all merged, and the planner's `tmp/planning/real-loop`.
    - The run's directory, 174 MB, keeps the two scratch patches of ruling 152.

## 10. Decisions

| Proposal | Decision | Where |
|---|---|---|
| 1 | accepted: heavy names each row's task, and each count reads its own rows | o/hygiene (skein's heavy) |
| 2 | accepted: land refuses `_and_` names a landing adds, on every landing | o/hygiene (skein's land), after o/accounts-2 renames the ten |
| 3 | declined: under owners.md the owner judges a diagnosis's length | — |
| 4 | accepted | skein 1303838e, owners.md 4.3 |
| 5 | declined: owners report the landings others await, and the director relays them | — |
| 6 | accepted | skein 1303838e, owners.md 4.2 |
