# Retrospective: tools

2026-10-19. This is the run of temper's `tools` plan on debian-16gb-hel1-1, a machine with eight cores and 16 GB. The plan is the first of two that build temper's tools (`docs/design/tools.md`, area 1). skein's request table took a key given by the asker. jig moved to skein's and smith's `main`, sent under a named key, read one turn, and took a deadline as a span. jig's focused budget went back to 15 s. temper built `temper-client-tools` as a library: the table and a call's argv, the root and the runner with the reads and acts, the waits, the forge's parts and `adopt`, help and a call's JSON. Closing repins moved smith, jig, hearth and temper.

| Event | When (UTC) |
|---|---|
| Planning started (session 123b69d5) | 2026-10-18T21:45:44Z |
| Plan landed (temper `1093544c3`) | 2026-10-19T02:25:09Z |
| Decisions A to I recorded by the director (`d2d6c9cb9`) | 03:25:57 |
| Claim | 03:28:49 |
| First landing (skein-01.1, `7ee1ac86`) | 03:36 |
| Last landing (temper-repin, `8c02b3b9e`) | 11:47:43 |
| `done.md` written | 11:48:42 |
| "Done when" run by the supervisor's Sonnet sub-agent | 11:52:47 to 11:57 |
| Release | 11:59:08 |

All 43 steps landed: 2 in skein, 22 in jig (the repins included), 17 in temper, and one repin each in smith and hearth. This retrospective was written by a fresh session that took no part in the run (development.md, section 15).

**Who read what.** Eight read-only readers, one per source. While Claude credit is tight, the director's rule of 10-18 sends complex reading to Codex, so four of them ran on Codex gpt-6-astra at `high` through `tmp/director/codex-helper.sh`, and four were Sonnet sub-agents:
- Codex: `log.md`, with the ledger, `context.md`, `alerts.md`, `sessions/` and the 79 reports;
- Codex: `decisions.md`, rulings 101 to 139 and "For the user", with `findings/drafters.md` and the plan's growth;
- Codex: the coordinator's rollout and the 42 implementer threads it spawned;
- Codex: the supervisor's transcript and its 27 sub-agents, and the planner's two transcripts and their 34 sub-agents;
- Sonnet: the git history of the five repositories, with its trailers;
- Sonnet: `logs/heavy.tsv` and the 748 logs;
- Sonnet: the earlier retrospectives (client-api, render, kit-batch, wall-clock, forge-setup), and skein's process from `54c5a8b` to `main`;
- Sonnet: the debts: `later.md`, the tech-debt files, the code on `main`.

This session read:
- development.md's sections 2.6, 3.4, 3.5, 5.1, 5.2, 5.3 (Size), 10.4, 13 and 15;
- the plan's README, `done.md` and `supervision.md` whole;
- the director's messages to the planner and the supervisor (`tmp/director/msg-tools-*`, `msg-codex-*`, `inbox/tools-done-when.md`);
- the metrics and their history;
- client-api's retrospective, sections 1, 3, 7, 8 and 10.

Not read: the design's 15 sessions and the director's, which are not on this machine.

**Conventions.** Times are UTC, all on 2026-10-19 unless a date is given. Costs are at list prices for each message's model, as `plan metrics` prices them:

| Model | Input | 1-hour cache write | Cache read | Output |
|---|---|---|---|---|
| Opus 5.5 | $4 | $8 | $0.20 | $20 |
| Sonnet 5.5 | $2 | $4 | $0.10 | $10 |

The metric prices a five-minute cache write below a one-hour one. The transcript reader priced every write at the one-hour rate. Its dollars run about 10% to 15% above the metric's: $84 against $76 for the supervisor, $200 against $174 for the planner. Shares below come from its counts; totals are the metric's.

This document proposes; the user, or the director, decides, and the designer applies what is adopted.

**Which process.** The run kept skein `54c5a8b` (`supervision.md`'s Process line; `kit/` from the claim). Two commits changed `docs/foundation` or `scripts/dev` before the release, both about the GitHub organization's new name (section 7). Neither touched the run. client-api's nine proposals were adopted at 01:43 and none had landed in skein. Three reached the run by other ways: the planner applied proposal 1, the README gave proposal 3's wait, and the director's note at 03:29 gave the supervisor proposals 3, 6 and 7.

## 1. In one page

- **43 steps landed: 42 planned, 1 added, none dropped. "Done when" passed but for item 13.**
  - temper's fuzzy suite missed its aim, a median of 9.210 s against 8 s (ruling 139). The suite is as long as its slowest test, `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, 7.86 s alone. Its median at the base was already 7.04 s.
  - Item 11 passed for hearth vacuously. hearth's `Cargo.lock` moved to the `fabrig-dev` organization at 11:45, and the item's grep names `forgetic`. The supervisor checked hearth's pin by hand.
- **8.3 hours, 0.20 h a step.** The longest chain was the run. Of its 21 steps, 20 landed one after another at a median of 24 minutes each, 7.9 hours in all; temper 04.2 ran beside 04.1. The README projected 1.9 to 2.9 hours, from development.md 5.2's 0.09 to 0.14 h a step. That is a throughput figure; a chain step's own time was three to four times it. On average 1.66 implementers were at work.
- **12.1M tokens a step,** against client-api's 41.5M and render's 14.4M.
  - Implementers took 5.6M a step (21.1M). Polling-only requests fell from 28.5% to 4.7%, under the README's `yield_time_ms` rule (client-api's proposal 3).
  - The coordinator took 1.9M a step (5.96M): it compacted at 184K and 189K, and had 19 heartbeat-only cycles, against 230.
  - The supervisor cost $76. It compacted in place 19 times; the restores cost $6.5 together. All 27 of its sub-agents ran on Sonnet.
- **The planning's Claude Code cost more than the run's: $174 against the supervisor's $76.** That is $4.0 a step, the most on record.
  - Sixteen Opus drafting sub-agents took about 70% of it.
  - The plan was split in two at 01:46, when client-api's proposal 1 gave the bound, three hours after the drafting began. About $62 went to drafts of sessions that moved to the next plan, and $41 to three drafts lost in the 23:20 outage and drafted again on Codex.
- **Eight design questions, J to Q. The director decided each as its default, in a median of about 7 minutes, and no designer was asked.**
  - Each was a mismatch between tools.md and jig's contract that the types show: a bound past jig's wire ceiling, two cursors where tools.md prints one, an act with no tool, a recipient jig does not keep.
  - Five of them (M to Q) came up in temper 03 within an hour, as it wrote the documents.
  - J, K and L reached tools.md (`dba3c3fcb`). M to Q have not: tools.md contradicts the code on five behaviours, and the next plan is being written against it.
- **Plan defects were found before the steps started.** Seventeen ready checks and re-checks led to 14 rulings correcting nine session files, nearly all before their steps began. Only two plan defects blocked a session, 0.05 a step against client-api's 0.47. Rulings came to 0.91 a step, against 1.9.
- **The Sizes of temper's document steps were off by up to 20 times.** temper 03.2 changed 6,008 production lines against a Size of 300: 23 hand-written JSON encoders, one per type of jig's contract. Seven temper steps passed twice their Size. Their threads were among the costliest; temper 03.2's and 03.3's compacted twice.
- **The supervisor's own checks held the queue.** Its 193 scratch tasks were half of the build lane's waiting, with 17 whole-workspace test runs among them. 77% of the waiting fell in the first 90 minutes, when four ready checks and then four re-checks built beside the sessions.
- **Test shape took eight steer points and still slipped.** Three README rules came by steer (rulings 106, 116, 120). The findings left at the end are in jig's `deadline_spans.rs` (two files), `read_activity.rs`, `requests.rs` and `job_stories.rs`, and temper's `acts.rs` and `help.rs`.
- **Another run broke a closing repin.** live-text's smith changed channel v2 to v3 at 09:34, after this run's jig repin. Ruling 138 added `jig-repin.s1`; temper-repin landed 23 minutes after its block.
- **Experiment 7 (defects by drafter) does not separate Opus from Codex.** Opus files had 4 plan-defect corrections in 9 increments, Codex files 8 in 14, mixed files 7 in 15. The numbers are small and the files differ.
- **Eight proposals,** in section 8: settle a plan's split before drafting; map the design's outputs to the upstream types before landing; a decision made in a run reaches the design before the next plan; ready checks test their step's crates only; project the chain at a chain step's own time; refuse a new test named with "and"; a check that extracts nothing fails; `running_h` ends where `wall_h` does.

## 2. Outcome

`done.md`'s "Done when, run 2026-10-19T11:52:47Z" holds each command's output. A Sonnet sub-agent of the supervisor ran them once, from `origin/main`, and ran item 13 through `heavy --gate`. `done.md`'s first part, at 11:48:42, lists only which steps cover each item.

| Item | Result |
|---|---|
| 1. Every step landed or dropped | **Met.** 43 of 43. |
| 2. skein's table takes a given key and refuses a held one | **Met.** It prints 2 and 0. |
| 3. jig sends under a name | **Met.** It prints 1. |
| 4. jig reads one turn, at one wire version | **Met.** One line, `crates/jig-client-protocol-wire/src/lib.rs:12`, `VERSION: u32 = 9`; `Turn { task, attempt, turn }` once. |
| 5. jig takes a deadline as a span | **Met.** `NewDeadline` and `tasks: 4, people: 4` once each. |
| 6. jig's budgets are 15 s and 60 s | **Met.** `15s 60s`. jig's `tech-debt.md` still says 30 s (section 9, item 12). |
| 7. No profile changed | **Met.** |
| 8. temper's tools exist as a library | **Met.** |
| 9. The usage lines of `close`, `decide` and `adopt` | **Met.** |
| 10. The naming checks | **Met.** |
| 11. One revision of each source; skein with given keys | **Met, by hand for hearth.** hearth's lock names `github.com/fabrig-dev/skein.git` since its `5efba43` at 11:45. The item's `forgetic` pattern skipped it. In the second loop `$s` was empty, so `git show ":crates/…"` read skein's index and passed. The supervisor then checked hearth's pin `49725eca` by hand: it has `ask_given`. |
| 12. temper holds jig's wire version 9 | **Met.** temper pins jig `eceadcaf`, jig-repin.s1's landing. |
| 13. The test-time aims | **Missed for temper fuzzy**, recorded in ruling 139. Medians of three: jig focused 5.404 s (aim 7), jig fuzzy 6.993 s (8), temper focused 3.501 s (5), temper fuzzy 9.210 s (8). One more temper fuzzy run at 11:57 took 7.860 s. Its three slowest tests are named in `done.md`. |
| 14. Nothing else | **Met.** |

What the plan leaves for later:
- **The next plan,** `temper ctl`'s host, the client stories, the fake forge's host, the real-loop world and the process tier. Its drafts are in `findings/next/`.
- **The design text of decisions M to Q,** `later.md` rows 40, 42 and 43 (section 6).
- **jig's test mends** that ruling 128 deferred, `later.md` row 41.
- **setup.md's figures for the owned example's memory,** `later.md` row 36 (ruling 109).

## 3. Numbers

`plan metrics` ran without error from skein's checkout and wrote `metrics.tsv`. The comparisons:
- client-api, the last run, and temper's;
- forge-setup, temper's run before it;
- render, the fastest large run;
- test-budgets, a run of about this size (39 steps).

| Metric | tools | Notes | client-api | forge-setup | render | test-budgets |
|---|---|---|---|---|---|---|
| `wall_h`, `running_h` | 8.3, 8.5 | `running_h` exceeds `wall_h`. `numbers.sh` sums the coordinator's rollout events, gaps up to ten minutes, with no end: they ran to the release at 11:59, while `wall_h` ends at `done.md`'s line at 11:48 (proposal 8). | 63.5, 60.0 | 43.8, 7.9 | 13.8, 13.4 | 3.7, 3.7 |
| `steps_planned`, `added`, `dropped` | 42, 1, 0 | The added step is `jig-repin.s1` (ruling 138). temper-04.4 landed twice: `bd949d171`, then `71fbd2b12` for the `Deadlines` its Shape asked. | 137, 22, 0 | 40, 4, 0 | 151, 17, 0 | 36, 3, 0 |
| `step_h` | 0.20 | The chain was the run (section 5, item 1). | 0.38 | 0.18 | 0.08 | 0.09 |
| `escalations` | 13: design gap 7, plan defect 2, process 2, infrastructure 1, cross-session scope 1 | 0.30 a step. Ruling 101 calls the first "process" block a plan defect. Ruling 137 calls its "design gap" none: tools.md 5.4 answered it. Questions J and L came from ready checks, outside the count. | 194 | 29 | 48 | 24 |
| `escalation_wait_min` | 4.2 | To the interim ruling. To the final answer of M to Q, the median is 6.0 minutes and the longest 12.1 (O). | 2.7 | 1.7 | 2.7 | 1.8 |
| `blocked_h` | 1.2 | The two temper-01.1 blocks overlapped. | 13.9 | 3.2 | 2.6 | 0.6 |
| `answers_reversed` | 9 | Six end an interim ruling's wait (127, 130, 131, 133, 134, 136), and 110 with 111 ends ruling 108's. Two change an instruction: 116 removes ruling 113's `env!`, and 128 withdraws 122's mend. | 9 | 2 | 2 | 3 |
| `steers` | 15 sent, 0 points refused | 20 points, eight of them on test shape. | 64, 18 | 13, 1 | 27, 1 | 4, 0 |
| `restarts` | none | One coordinator thread throughout. | crash 1, unknown 4 | loaded 6, reboot 1, user 1 | | |
| `tokens_coordinator` | 81.9M in (99% cached), 0.14M out | 1.90M a step. | 947.0M; 1.58M | 249.7M; 0.37M | 401.3M; 0.70M | 71.8M; 0.16M |
| `tokens_implementers` | 242.7M in (97%), 0.93M out | 5.64M a step: client-api 21.1M, forge-setup 7.3M, render 6.7M. The three costliest threads (jig 05.2/05.3, temper 03.2, 03.3) took 26%. | 3,349.8M; 8.29M | 320.8M; 0.95M | 1,122.6M; 4.40M | 201.9M; 0.48M |
| `tokens_claude` | 708.9M in (96%), 5.75M out; 457.9M before the claim, 191.5M in the run, 59.5M after | The planning is counted, from this machine. The design's 15 sessions are elsewhere, counted as 0. "After" is this retrospective, up to when the metrics ran. | 2,284.3M | 667.6M | 880.9M | 129.4M |
| `cost_claude` | $287: planner $174, supervisor $76, designer $21, retrospective $16 | $6.7 a step: planning $4.0, the run $1.8. | $795, no planning | $197 | $312 | |
| `supervisor` | 1 session, 8.5 h, 1.4M weighted tokens an hour, 87% in the 40 requests after each start or compaction, 19 compactions | Compactions every 27 minutes, at 116K to 120K. At that rate the 40-request share covers most of the session and says nothing (section 5, item 9). | 36, 65.0 h, 1.4M, 36%, 25 | 6, 43.6 h, 0.4M, 17%, 0 | 15, 13.4 h, 2.7M, 63%, 28 | |
| `tokens_per_step` | 12.1M | Coordinator 1.9M, implementers 5.6M, the run's Claude Code 4.5M. | 41.5M | 23.4M | 14.4M | 8.6M |
| `coordinator_requests` | 743, 47 after a wait ran out | 17.3 a step. A ring took a median of 4 requests, p90 10. | 4,655, 325 | 1,303, 243 | 1,961, 32 | 398, 9 |
| `compactions` | 2 | At 184K (05:42) and 189K (09:26), under the profile's 200K. | 10 | 2 | 4 | 1 |
| `gates_per_step` | 1.1 | The lowest on record. 48 land gates, 2 failed: a browser story that missed its text in 6.5 s, where it passes in 2 (04:17), and a golden that temper-04.1 had not seen (10:55). | 1.8 | 1.5 | 1.3 | 3.1 |
| `gate_lane_other` | 25 | 10 of jig 04's measurements, 10 process-tier runs, 3 for "Done when", and 2 of the next plan's planner after `done.md`. | 37 | 148 | 21 | |
| `queue_wait_ratio` | 0.85 | Build lane 1.14, gate lane 0.24. 77% of the build lane's waiting fell before 05:00 (section 5, item 6). | 0.94 | 1.34 | 0.74 | 2.13 |
| `suites` | **jig:** focused 4,270 / 6.960 s → 4,358 / 5.934 s; fuzzy 111 / 7.635 → 7.155 s; browser 13 / 14.859 → 14.483 s; gate 131 → 57 s. **temper:** 1,880 / 3.627 s → 1,861 / 3.131 s; fuzzy 50 / 9.481 → 7.771 s. **skein:** 2,582 / 4.366 s → 2,588 / 2.963 s. | temper's count fell while its new crate added tests: forge-kit moved temper's forge crates, with their tests, to skein. | | | | |
| `proofs` | unit 16, world 8, joint 19, process 3; 0 below plan | None "unavailable". temper 01.1's process proof ran on Forgejo 17.0.0. | | | | |
| `report_lines` | 12 median, 15 p90 | 79 reports, each copied whole into `log.md`. | 12, 18 | 16, 24 | 12, 16 | 11, 14 |
| `lines` | code +17,016 −1,623; tests +9,125 −1,118; generated +1,146 −372; data +1,038 −559 | temper-03.2 alone added 6,002 lines of production code. | | | | |
| `contexts` | 119K median, 184K p90 of the sessions' peaks; 25 handovers | At its 30th request a thread's median context was 96K, as in client-api. | 186K, 332K; 150 | 152K, 208K; 19 | 172K, 223K; 66 | 142K, 209K; 3 |
| `pin_lag` | smith: skein 12; jig: skein 3, smith 5; hearth: skein 30; temper: skein 4, smith 3, jig 2 | Fetched after the release, while forge-kit and live-text land. | | | | |

**Not comparable, and why:**
- **`running_h`** counts the coordinator's events to its last, here the release, and `wall_h` ends at `done.md`, so here the first exceeds the second (proposal 8). section 15.2 defines it by the heartbeat; `numbers.sh` uses the rollout.
- **`supervisor`**'s share in the 40 requests after a compaction. At 19 compactions in 8.5 hours it counts most requests. Render's proposal 11 replaces it with each restore's cost; that part of the kit is not built.
- **`cost_claude`** counts the planning here; client-api's was on x1, counted as 0. Neither counts the director.
- **The machine.** client-api and forge-setup ran on carbon-forge, so `wall_h`, `suites` and the aims differ by machine.
- **`queue_wait_ratio`.** This run's kit writes `heavy`'s row inside the task (`817b15c`). The figure is true; client-api's 0.94 was overstated.
- **`contexts`.** Implementers hand over past 100K here (render's proposal 12). client-api handed over later.
- **Step names repeat across plans.** skein-01.1 and 01.2, temper-01.1, 04.1 and 04.2 are also forge-kit's or live-text's steps on the same `main`s. Only the `Plan:` trailer tells them apart.

**What explains the largest differences:**
- **Hours a step, 0.20 against client-api's 0.38.**
  - The plan was a third of client-api's size, and blocks held the chain for 1.2 hours in all, against 13.9.
  - But the chain still set the run. Implementers at work, measured over the run: none for 9 minutes, one for 4.6 hours, two for 2.3 hours, three or more for 1.2 hours. A wider plan would not have been faster; a shorter chain, or faster chain steps, would.
- **Implementers' 5.6M a step.**
  - **Polls.** 126 of 2,675 requests only polled a running command: 4.7%, 12.1M input. In client-api it was 28.5%.
  - **Handovers at 100K.** Peaks were 119K at the median, against 186K.
  - **What did not move.** A thread's first request held 16.9K. By its 30th it held 96K, as in client-api: the session file, `decisions.md` items and code it read. 28 of 42 threads `cat` their session file whole.
- **The coordinator's 1.9M a step.**
  - **Compaction under 200K,** set by its profile (`37c676e`), against 330K to 385K in client-api.
  - **Rings of 4 requests** at the median, against 6 to 7, and 19 heartbeat-only cycles, against 230. client-api's proposal 5, one-command rings, is not built.
  - **Python scripts that write files** are still 40% of its output (172 requests), against client-api's 29% for copying.
- **The supervisor's $76.**
  - **Sub-agents.** All 27 ran on Sonnet, about 54% of its cost. By purpose: nine ready checks 37%, eight re-checks 35%, seven hourly checks 24%, two diagnoses 3%, "Done when" 1%.
  - **The main session.** The ready checks and their rulings took about 37% of it, questions J to Q 21%, the restores after compactions 17%, the hourly checks 9%.
- **The planning's $174** (section 5, item 4).

## 4. What worked

1. **A plan of the size client-api's proposal 1 asked.** The planner split area 1 in two (decisions.md, 16): 38 increments and 4 repins, a chain of 21, against 68 steps and a chain of about 31 whole. The results: 0.91 rulings a step (client-api 1.9), 0.05 plan-defect blocks a step (0.47), and a plan that grew 18% during the run (5,020 to 5,908 lines), against client-api's 82%.
2. **Ready checks found the plan's defects before the steps started.**
   - Nine ready checks and eight re-checks, all Sonnet, led to 14 rulings (102 to 125) that corrected nine session files. All but three landed before their step began; 112, 117 and 123 reached theirs in its first minutes.
   - The plan's defects by cause, a ruling counted under each cause it fixed: Files that missed users a `git grep` at the base finds (7); API forms that did not exist or did not match (5); Shapes that broke `clippy.toml`'s bans (5); version, count and budget literals (3); encoded sizes not carried to their bounds (4).
   - Only two plan defects reached a running session: the owned example's bounds (ruling 109) and the store profile's growth (ruling 118).
3. **The yield rule.** The README's `// @exec: {"yield_time_ms": 300000}` held. Polling-only requests fell to 4.7%. Only two polls returned before 60 s on a running command, both asking 30 s (temper 03.3, 09:49).
4. **Fast answers kept the work going.**
   - The director decided each of J to Q within 2 to 38 minutes of the question; for the six that held a step, 2 to 9.
   - Interim rulings (126, 129, 131, 132, 135) let 03.2, 03.3 and 03.4 go on with the rest of their work. No step waited more than about 12 minutes on a question.
5. **A prompt coordinator.** Each session but one started 34 to 62 seconds after its Needs landed; temper 02 took 3.1 minutes, behind the scratch worktree of item 6 in section 5. Mean concurrency was low because the plan's dependencies allowed no more, not because the coordinator held ready work. It compacted twice and never restarted.
6. **The hourly process tier** (forge-setup's proposal 1). It ran at each check from 07:00. The one failure, at 08:00, was alerted at 08:19, with the range of commits it could come from. No story turned red on `main` unseen, and no Proof said "unavailable": Forgejo 17 and its runner were installed on the machine (ruling 109).
7. **Compaction in place for the supervisor.**
   - One session ran the whole run. Each of its 19 restores took 1 to 8 requests and about $0.34, $6.5 in all.
   - render's rule (development.md, 10.5 and 13) held: no handover by count, none needed. The director's "hand over at 150K" was never reached.
8. **Gates.** 48 land gates for 43 steps; two failures, both retried or fixed within three minutes. A gate held across Markdown landings: temper 03.3 rebased past docs and kept its gate.
9. **jig 04** brought jig's focused budget back to 15 s (Done when 6). At the end jig's focused median was 5.4 s.
10. **The closing repins absorbed another run's break.** `jig-repin.s1` was added, run and landed in 11 minutes, and temper-repin landed 23 minutes after its block (section 5, item 7).

## 5. What did not

1. **The run took 8.3 hours against a projection of 1.9 to 2.9.**
   - **Cause.** The README projects the longest chain at the last runs' `step_h`, 0.09 to 0.14 hours a step (development.md, 5.2). `step_h` divides a run's hours by all its steps, so it falls as more steps run at once. A chain step's own time, from its start to its landing, was a median of 24 minutes here: 20 chain steps took 7.9 of the run's 8.3 hours. Steps off the chain had a median of 15 minutes.
   - **Where a chain step's time went.** Of three typical threads' wall time: landing (rebase, prebuild and gate) 20% to 44%, reading 10% to 23%, editing 15% to 27%, building and testing 16% to 31%. jig's land prebuild and gate took about 4 minutes of each jig step. Most chain steps began in a new thread, which read its rules and session file again: 15 of the 25 handovers were in the chain's sessions.
   - **When.** The 06:51 check re-projected 2.6 to 5.6 hours more; the run took 5.0 more. Nothing could act on it: the chain was the plan's.
   - Proposal 5.
2. **Eight design questions came up during the run, and five never reached the design.**
   - **What they were.** Each was a mismatch between tools.md and jig's contract, which the types show:
     - J: `TaskQuery.order` has no `Option`, and tools.md names no default;
     - K: tools.md's 50 rows, against the wire's ceiling of 32 in jig's `CEILINGS`;
     - L: `Read::Tasks` answers two groups with two cursors, and tools.md prints one;
     - M: 13 of jig's 23 `ActKind`s have no tool;
     - N: the inbox's two groups, two cursors;
     - O: two first pages within 32 rows;
     - P: `Read::Activity` takes no row count, so a cut page cannot read on;
     - Q: `Outcome::Proposed` keeps no recipient.
   - **When.** J at 04:57 and L at 07:04, from ready checks; K at 05:31, from temper 02.1's block; M to Q from 08:58 to 10:01, five blocks in an hour as temper 03 wrote the documents.
   - **Who answered.** No designer was joined (`plan wake designer` refused at 05:39). The supervisor wrote each under "For the user" with its default, and the director decided each as the default. development.md 10.4 has the designer decide a design gap "always", and amend the design in the same landing.
   - **The design.** A designer session on another machine wrote J, K and L into tools.md at 08:41 (`dba3c3fcb`). M to Q are `later.md` rows for "the designer's next pass". tools.md now contradicts the code on five behaviours, and the next plan's planner is reading it.
   - forge-setup's proposal 9 (the planner traces the design first) was adopted. It did not reach the upstream contract's types and bounds, where all eight lay.
   - Proposals 2 and 3.
3. **Sizes were off by up to 20 times on temper's document steps.**
   - Seven steps passed twice their Size, all in temper: 02.2 (2.6 times), 02.3 (4.5), 03.2 (20.0), 03.3 (7.9), 03.4 (4.8), 04.1 (2.9), 11.2 (2.3). jig's largest was 05.3 at 1.6.
   - **Cause.** temper 03.2 writes "concrete integer JSON for the client contract": one hand-written encoder per type of `jig_client::contract`, 5,112 lines in 23 files under `jig/render/document/`. 03.3 and 03.4 did the same for the reads and the acts' answers. Each Size was set at about 300 with no count of the types it covers. development.md 5.2 splits a step that needs more than about 300 lines; a count at the base would have split these.
   - **Cost.** Three of the four costliest threads were these: temper 03.2 19.2M, 03.3 17.7M, 03.4 10.9M, each compacting once or twice. They were also the chain's slowest temper steps, 41, 36 and 26 minutes. They landed without asking, under "Past the Size".
   - **When.** First at temper 02.2's landing, 06:00. The 10:07 check noted it for the retrospective.
   - Proposal 2: the map that finds the questions also counts the types.
4. **The planning cost $174, more than the run's supervisor.**
   - **Cause.** The planner launched sixteen Opus drafting sub-agents from 22:29 on 10-18, one per session file, for the whole of area 1 (68 steps, a chain of about 31). At 22:50 the director moved drafting to Codex; the running Opus drafters were let finish. Six of the Opus drafts, about $62, became the next plan's `findings/next/`, which its planner must check again against this run's `main`.
   - **Also.** The account's outage at 23:20 lost three drafts (jig 03, temper 05, temper 08), about $41, and Codex drafted them again. The Codex revision waves at `xhigh` took its week from 35% to 51% in two hours, and the director stopped them at 01:43.
   - **When.** No bound on a plan's size existed when the drafting began. client-api's proposal 1 gave one at 01:43, and the planner split the plan at 01:46. With the bound in hand, the step list alone would have shown the split before the first draft.
   - **Drafters.** Experiment 7 found no difference in defects between Opus and Codex drafts (section 1), so the drafter can be chosen by cost.
   - Proposal 1.
5. **Test shape took eight steer points and still slipped.**
   - **What.** The hourly checks found the same classes at 04:40, 05:45, 06:51, 07:56, 09:02, 10:07 and 11:13: a setup typed again, a row that carries a flag, a bare size, two behaviours in one test, a test found by position.
   - **What was done.** Three README rules by steer (rulings 106, 116, 120), and steers at 04:53, 05:53, 07:05, 08:18 and 10:20. Ruling 122 asked for a mend at the file's next edit. No increment edited it again, and ruling 128 withdrew the mend at 09:14.
   - **What is left.** jig's `tests/core/tests/deadline_spans.rs` and `tests/client-protocol/tests/deadline_spans.rs`, `read_activity.rs`, `requests.rs` and `job_stories.rs`; temper's `src/tests/jig/acts.rs` and `help.rs` (section 9).
   - **Cause.** render's proposal 6 (a rule sent twice becomes a gate check) was adopted, and no check was built. The supervisor wrote at 07:56 that only a check on the diff would serve: bare `from_nanos(` is in 89 jig test files, and test names with "and" in a third of them.
   - The new crate's tests carry `_and_` in 19 names; some are the plan's own (12.1's, kept by ruling 120).
   - Proposal 6.
6. **The supervisor's checks held the build lane.**
   - **What.** Its 193 scratch tasks waited 8,651 s for 3,595 s of running: 51% of all build-lane waiting. They include 17 whole-workspace test runs, among them the fuzzy and browser profiles.
   - **When.** Four ready checks started at 03:36 and four re-checks at 04:40, beside the sessions' own tasks. The build lane waited 1.97 times its running in the first half hour and 1.35 in the next hour, and 77% of all its waiting fell before 05:00.
   - **What the queue reported.** The 05:45 check saw 6,666 s waiting against 4,695 s running; nothing acted on it.
   - **The worth.** The checks were worth having (section 4, item 2). Their test runs were not: a step's own land gate runs the workspace anyway.
   - **Also.** A ready check made its scratch worktree at a session's own path, `wt/temper-02`, and blocked temper 02.1 at 05:26 (ruling 111, the `r` suffix).
   - Proposal 4.
7. **Another run's change broke the closing repin, and was seen 76 minutes before it was acted on.**
   - **What.** live-text's smith-01.1 (`f925b180`, 09:34) replaced channel v2 with v3 in place and added `Start.live`. jig-repin had landed at 08:29. `plan downstream` at 10:13 showed smith, jig and temper failing.
   - **When.** The 10:07 check alerted it, and left it for the repins under the director's note of 06:58. temper-repin met it at 11:25; ruling 138 added `jig-repin.s1` at 11:29. A step added at 10:20 could have run beside temper 04, off the chain, though live-text's later smith steps (10:28 to 11:13) might have needed it again.
   - **Cost.** About 20 minutes at the end of the chain.
8. **"Done when" item 11 passed without checking hearth.**
   - **Cause.** The GitHub organization was renamed during the run. hearth's `5efba43` moved its lock to `fabrig-dev` at 11:45. The item and the README's Pins rule grep `github.com/forgetic/`. The loop read an empty revision, and `git show ":path"` read skein's index instead, which passed.
   - **When.** The "Done when" run saw it at 11:52; the supervisor checked hearth's pin by hand at 11:59.
   - smith, jig and temper still name `forgetic`. When they move, the Pins rule and items 11 and 12 pass for all of them, checking nothing.
   - Proposal 7.
9. **The supervisor compacted at 117K, not at about 180K.**
   - All 19 compactions were automatic, at 116K to 120K. development.md 10.5 says `--autocompact 200000` compacts "at about 180K". The planner's sessions compacted at 167K to 170K.
   - The cause was not found. The cost was small: $6.5 of restores.
   - **Two effects.** The `supervisor` metric's 40-request share reads 87% and means nothing (section 3). The director's 150K handover, kept in each compaction's summary, could never be reached.
   - **Also.** `supervision.md`'s own compaction numbers restart: the 15th compaction is written "compaction 7". The transcripts' count, 19, is the metric's.
10. **The temper fuzzy aim was missed, by a test known slow since client-api.**
    - **What.** temper's fuzzy suite runs as long as `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, 7.86 s alone at the end. The suite's median at the base was 7.04 s on this machine, and the plan's aim, 8 s, left under 1 s for noise. Temper's fuzzy gate ran above 8 s on 8 of 18 landings.
    - **Cause.** client-api's retrospective (section 9, item 3) named it: `run_replayed` formats both worlds with `Debug` at up to 600 iterations (`tests/engine/domain/src/walking.rs:832-833`). No plan has a step for it.
    - **When.** First at temper-01.1's gate, 05:24 (9.481 s). Each check called the excess noise.
11. **A process-tier story failed once, and its cause was not sought.**
    - **What.** At 08:00, `a_change_lands_by_deliveries_with_every_poll_raised` panicked in the engine, at jig-domain-core `documents/small.rs:11`: "source text is UTF-8 at the engine entrance". It passed the next three runs.
    - The alert named forge-kit's range, and no one diagnosed it. A reachable `expect` at the engine's text entrance is a debt (section 9, item 8).
12. **Smaller things.**
    - **temper 04.4 landed twice.** 04.3 had missed its Shape's `skein_lib::Deadlines`. 04.4's own check found it, and a second commit landed at 11:29.
    - **temper 12.2 waited.** It started at 06:29. Ruling 117, at 06:32, made it need 03.2's golden helper, and it waited 2.5 hours for it; a second thread then reconciled it. The README's row for temper 12 still names no increment that needs more.
    - **The Claude login expired** at about 05:05 and stopped three re-checks for 14 minutes.
    - **Hourly-check sub-agents read `supervision.md` whole,** two of seven, through `cat`; one read the README whole in two ranges. client-api's proposal 6, which the director had sent, says no.
    - **Two Evidence lines lost a space** ("passed207 tests"), from 11:10 on.

## 6. Decisions taken during the run

| Decision | Ruling | Reached the design? |
|---|---|---|
| J: `tasks` without `--order` reads by number; `--ended` changes no order; `Changed` and `Ended` have no argv form | 110 | **Yes,** tools.md 5.2 (`dba3c3fcb`, 08:41). |
| K: the tools' rows are 32, jig's wire ceiling | 114 | **Yes,** tools.md 12 (`dba3c3fcb`). `done.md` calls it "decision L". |
| L: a `tasks` call reads one group, the live one or with `--ended` the ended one; jig's cursor | 121 | **Yes,** tools.md 5.2 (`dba3c3fcb`). `later.md` rows 37 to 39 still say "next pass" for J, K and L. |
| M: an offered act with no tool prints its name and ` (no tool)`, with no call | 127 | **No.** `later.md` row 40. Code: `jig/render/acts.rs:44-47`. |
| N: the inbox's printed cursor carries a group byte before jig's | 130 | **No.** `later.md` row 42. Code: `render/lists.rs:195-226`, `argv.rs:523-537`. |
| O: the inbox's first page prints needs first, and updates on it only when both fit in 32 rows | 133 | **No.** `later.md` row 42. Code: `render/lists.rs:150-193`. |
| P: a page cut by `--limit` reads on by replaying the page whole; the cursor stays jig's | 134 | **No.** `later.md` row 42. Code: `render/activity.rs:9-41`. |
| Q: a restored proposal's kept line prints its identity without a recipient | 136 | **No.** `later.md` row 43. Code: `render/outcome.rs:213-226`. |
| An inbox read cut before Updates digests the watch's first window from all the rows read | 137 | No new design: tools.md 5.4 answered it. |
| The owned example's `memory_bytes` rise by each refusal's N | 109 | **No.** setup.md 8.1's figures are the designer's, `later.md` row 36. |
| jig 05.2 and 05.3 may grow the store profile's value, chunk and page by measurement | 118 | Not design: a plan allowance. |
| Joint stories share the fixture; a setup written once; rows carry data; one behaviour per test; no lint exception for a plan's words | 106, 116, 120 | Not design: testing-strategy.md 10 says them. They are README rules. |
| The `deadline_spans.rs` mend at its next edit, then withdrawn | 122, 128 | Not design. `later.md` row 41. |
| Scratch worktrees take an `r` suffix | 111 | Not design: the supervisor's practice. |
| `jig-repin.s1`, past live-text's smith | 138 | Not design: a repin step. |
| temper's fuzzy aim missed | 139 | Not design: for the user (section 9, item 2). |

Decisions A to I were recorded at 03:25:57, before the claim. The director vetoed C's fallback and installed Forgejo 17 and its runner instead.

## 7. The proposals adopted after the last retrospective

The last retrospective before the claim was temper's client-api, written at 23:52 on 10-18 (`62d50e0b8`). The director adopted all nine proposals at 01:43 (`1c918a77c`), "to apply" by a process designer. None had landed in skein by the end of this run. The run took three of them by other ways: the planner applied proposal 1, the README gave proposal 3's wait, and the director's note to the supervisor at 03:29 gave proposals 3, 6 and 7.

The figures a step divide by landed steps: client-api 159, tools 43.

| Proposal | Decision | Meant to move | client-api | tools | Moved |
|---|---|---|---|---|---|
| 1. Long plans run as consecutive plans | adopted; applied by the planner (decisions.md, 16), not in development.md | plan-defect blocks a step, < 0.25; rulings a step, < 1; `step_h`, < 0.15 | 0.47; 1.9; 0.38 | 0.05; 0.91; 0.20 | **Yes** for the first two. `step_h` moved, but not to its target: the chain set it (section 5, item 1). The README cites "development.md's bound", which is not in development.md. |
| 2. The rules every thread reads stay under about 300 lines | adopted; not applied | context at a thread's 30th request, < 60K; implementers' tokens a step, < 10M; steer points refused, 0 | 96K; 21.1M; 18 | 96K; 5.6M; 0 | **The context did not move.** The README's rules were about 100 lines, so the 96K is the session file, the `decisions.md` items and the code each thread read. The tokens and refusals moved, for other causes: polls, handover at 100K, steers that became README rules. |
| 3. A wait counts by when it returned; `yield_time_ms` is the one way to wait | adopted; the README gave the wait | polling-only requests, < 10%; implementers' tokens a step | 28.5%; 21.1M | 4.7%; 5.6M | **Yes.** The pulse's `short_waits` still counts by the asked value: not built. |
| 4. The kit moves to a fix for a defect the run meets | adopted; not applied | infrastructure and process blocks with a fix on `main`, 0; coordinator compaction, ≤ 200K; longest check gap, < 1.5 h | 5; 330K to 385K; 4.2 h | 0; 184K and 189K; 1.2 h (claim to the first check), else 65 to 66 minutes | **Not tested:** no kit fix landed during the run. The compaction moved through the profile (`37c676e`). |
| 5. A ring is one command | adopted; not built | coordinator requests a step, < 15; coordinator tokens a step, < 3M | 29.3; 5.96M | 17.3; 1.90M | **In part, without the kit.** Rings took 4 requests, the compaction moved, and timed-out waits fell. Python scripts that write files are still 40% of the coordinator's output. |
| 6. Sub-agents read ranges, never whole files | adopted; the director's note | the supervisor's sub-agent dollars a step, < $1; the designer's dollars a question, < $5 | $2.13; $6.5 | about $1 ($1.06 at the reader's prices); no designer | **Nearly.** Two of 27 sub-agents read `supervision.md` whole, and one the README. |
| 7. No Proof lands "unavailable" | adopted; the director's note; Forgejo installed | hours a regression stays on `main` unseen, < 1.5; regressions found by another run, 0 | 27; 3 | the one failure was seen within 19 minutes; 0 | **Yes.** |
| 8. btrfs's unallocated space is a resource | adopted; not applied | infrastructure blocks, 0; crash restarts, 0 | 6; 1 | 1 (Forgejo's runner); 0 | **Not tested:** this machine is ext4. |
| 9. `plan metrics` maps kinds and proof words | adopted; not built | unmapped kinds and words, misread landings, 0 | 4 kinds, 4 words, 1 landing | 0 | **Held by chance:** no kind ended in a full stop, every Proof's word was a layer's, and the plan's landing was its only commit with `Planner-Session:`. |

**Changes to the process that landed during the run** (skein's `docs/foundation` and `scripts/dev`, 03:28 to 11:59):

| Commit | Landed | What | Did the run take it? |
|---|---|---|---|
| `92e177c` | 11:38 | development.md 3.1 and the workspace `AGENTS.md` name `github.com/fabrig-dev` | No, and it needed none. The plan's Pins rule and "Done when" items 11 and 12 still name `forgetic` (section 5, item 8). |
| `2432239` | 11:53, after `done.md` | `plan check`'s `insteadOf` hint names `fabrig-dev` | No: a later `plan check`. |

**What this says of the loop.** Adopted proposals reach a run only by a planner's choice or the director's note, until a process designer lands them. Here that worked for 1, 3 and 7, and their metrics moved. Proposal 2's premise did not hold: the 96K is not the rules.

## 8. Proposals

1. **A planner splits a design's steps into plans, from the step list and its longest chain, before it drafts any session file, and drafts only the first plan's sessions.**
   - **Evidence:** section 5, item 4.
     - Sixteen Opus drafters took about 70% of the planner's $174, from 22:29.
     - The split came at 01:46, once client-api's proposal 1 gave the bound. About $62 of drafts became the next plan's `findings/next/`, to be checked again against this run's `main`.
     - The Codex revisions at `xhigh` took its week from 35% to 51% in two hours.
   - **Changes:** development.md, 2.2 and 5.1, in the text that applies client-api's proposal 1: it says when the split is made.
   - **Moves:** the planner's dollars a landed step, $4.0 to under $2 (forge-setup $1.8, client-composition $1.5).
2. **Before a plan lands, the planner maps each thing the design's output prints (each field, cursor, bound and act) to the upstream type that carries it, in `findings/`. Each with no carrier, or past the upstream bound, goes to the designer. Each step's Size counts the types it covers.**
   - **Evidence:** section 5, items 2 and 3.
     - All eight design questions (J to Q) were such mismatches: no default for a field with no `Option`, 50 rows past a ceiling of 32, two cursors for one, an act with no tool, a recipient not kept, a read with no row count.
     - Five came up in temper 03 within an hour.
     - temper 03.2's Size of 300 covered 23 encoders, one per type of jig's contract: 6,008 lines.
     - forge-setup's proposal 9 (trace the design first) was adopted; this carries it to the upstream contract.
   - **Changes:** development.md, 2.2 and 5.4 (before the run).
   - **Moves:** design questions during a run, 8 to 2 or fewer; steps past twice their Size, 7 to 2 or fewer.
3. **A design question decided during a run without the designer reaches the design before the next plan on that design is planned: the decision names the design text it changes, and the designer lands it.**
   - **Evidence:** section 5, item 2, and section 6.
     - No designer was joined. The director decided all eight as their defaults; development.md 10.4 has the designer decide "always" and amend the design.
     - M to Q are in the code and not in tools.md, and the next plan's planner is reading tools.md now.
   - **Changes:** development.md, 10.4 and 2.7.
   - **Moves:** decisions in force but not in the design at the run's end, 5 to 0.
4. **A ready check or re-check compiles its step's sketches and runs only the tests of the crates its Files name (`rdeps(…)`), never the whole workspace or its fuzzy and browser profiles, in a worktree named `<repository>-<NN>r`.**
   - **Evidence:** section 5, item 6.
     - The supervisor's 193 scratch tasks were 51% of the build lane's waiting, with 17 whole-workspace runs.
     - The build lane waited 1.97 times its running in the first half hour.
     - A scratch worktree at a session's path blocked temper 02.1 (ruling 111).
     - Implementers ran `rdeps` filters already. Each step's land gate runs the workspace.
   - **Changes:** supervisor.md (the ready check); development.md, 10.4 (what a ready check finds).
   - **Moves:** `queue_wait_ratio`, 0.85 to under 0.5; the supervisor's share of build-lane waiting, 51% to under 20%.
5. **A plan's README projects its longest chain at the median time from a chain step's start to its landing in the last runs (24 minutes in tools), not at `step_h`.**
   - **Evidence:** section 5, item 1.
     - Projected 1.9 to 2.9 hours; the chain took 7.9 of the run's 8.3.
     - `step_h` is the run's hours over all its steps, and falls as steps run at once. A chain step's own time does not.
   - **Changes:** development.md, 5.2 (the README's longest chain); `plan metrics`, a `chain_step_min` metric (the median start-to-landing minutes of the chain's steps) for the next plans to read.
   - **Moves:** the README's projection against `wall_h`, from 23% to 35% of it here to within 25% either way.
6. **`land` refuses a branch that adds a test function whose name the Pulse's `test-names:` pattern matches, `_and_` by default, and a plan names no test so.**
   - **Evidence:** section 5, item 5.
     - Eight steer points on test shape, three README rules, one mend withdrawn, and findings left in seven files.
     - render's proposal 6 (a rule sent twice becomes a gate check) was adopted, and none was built.
     - The supervisor wrote at 07:56 that only a check on the diff would serve. A name is the one part of testing-strategy.md 10's "one behaviour per test" that a script can see.
   - **Changes:** development.md, 9.1 (`land`'s refusals) and 5.2 (the Pulse); `land`.
   - **Moves:** steer points on test shape, 8 to 3 or fewer; new tests named with "and", 19 to 0.
7. **A "Done when" command or a plan's check that reads a value out of a file fails when it reads none, and a pin check matches the repository's name, not its organization.**
   - **Evidence:** section 5, item 8. Item 11 passed for hearth on an empty revision, reading skein's index. After the organization's rename, the Pins rule and items 11 and 12 would pass for every repository.
   - **Changes:** development.md, 5.2 ("Done when") and 4.5.
   - **Moves:** "Done when" items that pass without reading what they check, 1 to 0.
8. **`plan metrics` ends `running_h` where `wall_h` ends, at `done.md`'s `Recorded` line.**
   - **Evidence:** section 3. `running_h` 8.5 exceeds `wall_h` 8.3. `numbers.sh`'s `running_seconds` sums the coordinator's rollout events with no end, so the 11 minutes from `done.md` to the release count as running.
   - **Changes:** development.md, 15.2; `numbers.sh`.
   - **Moves:** comparability: `running_h` ≤ `wall_h` in every run.

## 9. Debts left in the code

For jig's `docs/development/tech-debt.md`, and for temper and skein, which have none. "Listed" says whether a debt file or `later.md` already has the entry. They are ordered by what they cost the next plans.

1. **tools.md lacks decisions M to Q.**
   - The code (all under `crates/temper-client-tools/src/jig/`):
     - M: `render/acts.rs:44-47`;
     - N: `render/lists.rs:195-226`, `argv.rs:523-537`, `runner/reads.rs:248-266`;
     - O: `render/lists.rs:150-193`, `runner/reads.rs:152-163`;
     - P: `render/activity.rs:9-41`;
     - Q: `render/outcome.rs:213-226`.
   - tools.md has no text for any of them.
   - `later.md` rows 37 to 39 (J, K, L) are stale: those are in tools.md.
   - Listed in `later.md`, rows 40, 42 and 43.
2. **temper's fuzzy suite is one test.**
   - `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure` takes 7.86 s of about 8 s. `tests/engine/domain/src/walking.rs:817-840` (`run_replayed`) formats both worlds with `{:?}` over 64 seeds of up to 600 iterations.
   - The fix needs `PartialEq` on jig's `Core` and temper's `engine::Domain` (client-api's retrospective, section 9, item 3).
   - Any plan that grows the worlds misses temper's fuzzy aim. Listed: ruling 139 only.
3. **temper's hand-written JSON mirrors jig's contract.**
   - `jig/render/document/` holds 5,112 lines in 23 files, one `encode_*` per contract type or variant. `act_answers.rs` alone has 23 `unreachable!` arms ("dispatched outcome"), and the crate 55.
   - Each change to `jig_client::contract` needs a matching edit here, found only by the compiler's match arms.
   - Not listed.
4. **Tool positions are bare numbers in production code.**
   - 23 sites compare `call.tool` with a literal: `jig/render.rs:291`, `:308`; `render/activity.rs:18`, `:22`, `:34`, `:42`; `render/document/read_sets.rs:16`, `:75`, `:79`, `:84`; `render/turn.rs:37`, `:63`, `:93`, `:99`; `runner/acts.rs:28`, `:377`; `runner/reads.rs:152`; and others. Watch slots are bare too (`local == 5`, `pages.get(&1)`).
   - The next plan adds tools (`remove`, `order`, `check`, `resume`, `ctl`'s verbs), which moves every position silently.
   - Not listed.
5. **jig's test shape, deferred by ruling 128.**
   - `tests/core/tests/deadline_spans.rs`: a loop that branches on `expected` (`:67-76`), the signed-in opener three times (`:15-16`, `:33-34`, `:51`), three behaviours in `an_aged_span_is_beyond_its_accepter_and_passed_up` (`:78-103`), bare seeds, keys and `from_nanos(15_000_000_000)`.
   - `tests/client-protocol/tests/deadline_spans.rs:172-204`: `a_named_proposal_keeps_its_span_across_client_and_engine_restart`, three behaviours.
   - `crates/jig-domain-core/src/tests/read_activity.rs:145-162`: a `tiny` flag that switches the setup, and its opener four times.
   - `crates/jig-client-domain/src/tests/requests.rs:7-12`, `:163-168`, `:234`: the save-then-send destructuring three times.
   - `tests/client/src/joint/stories/job_stories.rs:6`, `:16`, `:27`: its own `call`, `result` and `turn` beside `fixture.rs`'s `turn`, and bare `256` at `:124-125` and `:133`.
   - Listed: `later.md` row 41 has the first three; `requests.rs` and `job_stories.rs` are in no list.
6. **temper's test shape in the new crate.**
   - `src/tests/jig/acts.rs`: one opener typed five times (`:156`, `:164`, `:172`, `:187`, `:224`); `chat_checks_both_sides_of_its_bounds` (`:64-90`) branches; `decide_sends_its_two_forms` and `proposal_names_preserve_task_or_party` send one call and assert one value.
   - `src/tests/help.rs`: bare table positions in `GOLDENS` (`:15-22`) and `USAGES` (`:54-60`); `each_derivation_names_every_tool` (`:61-108`), the plan's own, with four behaviours.
   - 19 test names with `_and_`.
   - Not listed.
7. **Figures and goldens refreshed by hand.**
   - `docs/examples/owned.json:192` and `:825` were refreshed in three commits of this run (`4ef11dee3`, `e3430f46f`, `8c02b3b9e`). The worker's grew by 366,843,064,659 at 01.1.
   - jig's small profile, `crates/jig-protocol-config/src/profile/small.rs:321`, `:329` and `:330`, and its copy in `testing/jig-test-service/src/tests.rs:25`.
   - 40 goldens in `crates/temper-client-tools/src/tests/golden/`, with no write mode.
   - Listed: `later.md` row 36 for the owned figures; client-api's retrospective, items 4 and 16, for the rest; no debt file.
8. **An engine panic on text that is not UTF-8.** `crates/jig-domain-core/src/documents/small.rs:11` calls `.expect("source text is UTF-8 at the engine entrance")`. A process story reached it once at 08:00 (`logs/20261019T080041-3833.log`, lines 285-286). Not listed.
9. **Two GitHub organizations.**
   - hearth's lock names `fabrig-dev`; smith's, jig's and temper's name `forgetic`, in their `Cargo.toml`s too (25, 45 and 90 places). Cargo treats them as two sources, so they must move together.
   - Still `forgetic`: this plan's README (Pins, items 11 and 12) and five session files; temper's `docs/development/local-jig.md:19`, `:53`, `:55`, `:57`, whose `[patch."https://github.com/forgetic/jig.git"]` stops matching once `Cargo.toml` moves; smith's `docs/development/workflow.md:34` and README.
   - Not listed.
10. **jig's `tech-debt.md` is stale.**
    - `:197-198`, `:217`: the focused budget is 15 s since jig 04.4, not 30 s.
    - `:507-511`: the smith goldens' pin is `ff623361`, not `a277e922`. `crates/jig-protocol-transcript/tests/data/README.md:2` says `a277e922` too.
    - `:526-528`: `v9.rs` has 106,639 lines.
    - `:289`: 281 `unreachable!`, not 278.
11. **Files near or past 800 lines.**
    - No hand-written file crossed 800 in this run.
    - jig's over-800 files grew: `jig-domain-core/src/tests.rs` 2,974 to 3,003, `page/routing.rs` 1,103 to 1,130, `tests/acts.rs` 975 to 1,002, the scripted engine's `tests.rs` 2,257 to 2,283, `documents/objects.rs` 930 to 955.
    - skein's `requests.rs` is at 743 and its test `request_table.rs` at 752.
    - Listed: jig's `tech-debt.md:374`, with old figures.
12. **Not code.**
    - 41 local `tools/*` branches remain: 19 in temper, 22 in jig.
    - temper has no `docs/development/tech-debt.md`; items 2 to 4, 6 and 7 need one.

## 10. Decisions

| Proposal | Decision | Where |
|---|---|---|
| 1 | adopted | skein 656ef69: development.md, 2.2 and 5.1 |
| 2 | adopted | skein 656ef69: development.md, 2.2 and 5.4 |
| 3 | adopted (M to Q go into tools.md in the design pass on hel8, 10-19) | skein 656ef69: development.md, 2.7 and 10.4, the process rule |
| 4 | adopted | skein 656ef69: supervisor.md, 5; development.md, 3.5 and 10.4 |
| 5 | adopted | skein 656ef69: development.md, 5.2 and 15.2; `plan metrics`' `chain_step_min` goes to the next kit batch |
| 6 | adopted | skein's branch design/process-7, to land: `land`; development.md, 5.2 and 9.1 |
| 7 | adopted (the org rename to fabrig-dev makes it urgent) | skein 656ef69: development.md, 4.5, 5.2 and 5.4; `plan check`'s refusal of an organization in a pin check is on skein's branch design/process-7, to land |
| 8 | adopted | skein's branch design/process-7, to land: numbers.sh; development.md, 15.2 |

The kit's commit on skein's branch design/process-7 waits for a gate
that passes: on hetzner, test-plan runs 48 s, past its 40 s and its 5 s
margin, as `main`'s own does (10-19).
