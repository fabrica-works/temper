# Retrospective: client-api

2026-10-18. This is the run of temper's `client-api` plan on carbon-forge, a laptop with eight cores. The plan made jig's client presentation-agnostic: the client crates renamed, the engine's plumbing, the wire collapsed, `jig-client` with the reference UI on it, the application's server, and the browser shell. temper moved onto it with a client of its own, and one skein increment extended the browser harness.

| Event | When (UTC) |
|---|---|
| Plan landed (temper `d9b68d483`) | 2026-10-16T07:05Z |
| Claim | 07:48:33Z |
| First landing | 08:04 |
| Last landing | 2026-10-18T22:45:45Z |
| `done.md` written | 23:00:05Z |
| "Done when" run by the supervisor's sub-agent | 23:02 to 23:14 |
| Release | 23:18:07Z |

All 159 steps landed: 119 in jig, 7 in skein and 33 in temper. This retrospective was written by a fresh session that took no part in the run (development.md, section 15).

**Who read what.** Ten read-only sub-agents on Sonnet each read one source:
- `log.md`, with the ledger, `status.md`, `alerts.md`, `context.md` and `handed-over`;
- `decisions.md`, rulings 101 to 250;
- `decisions.md`, rulings 251 to 402 and "For the user";
- `supervision.md`, with the `facts-*.md` and `recheck-*.md` files;
- the git history of jig, skein and temper, with its trailers;
- `logs/heavy.tsv` and the logs it names;
- the coordinator's Codex rollout and the 203 threads it spawned;
- the 66 Claude Code transcripts in `claude.tsv` and their 158 sub-agents;
- the earlier retrospectives, the metrics history, and skein's process from `15ddb75` to `main`;
- the debts: the 430 reports, `later.md`, the code on `main` and jig's `tech-debt.md`.

This session read:
- development.md's sections 2.6, 3.6, 4.3, 5.4 and 15;
- the plan's README, its growth over the run, and `done.md`;
- the metrics and their history;
- skein's reflog for the checkout the tools ran from;
- implementer.md's waits and handover, at `841f487` and at `main`;
- the pulse's `short_waits` (`numbers.sh`);
- the last retrospectives' sections 8 and 10.

Not read: the planner's and the design's transcripts. They are on x1 (section 3).

**Conventions.** Times are UTC. A time without a date is on the day last named. Costs are at the list prices of each message's model, as `plan metrics` prices them:

| Model | Input | 1-hour cache write | Cache read | Output |
|---|---|---|---|---|
| Opus 5.5 | $4 | $8 | $0.20 | $20 |
| Sonnet 5.5 | $2 | $4 | $0.10 | $10 |

This document proposes; the user, or the director, decides, and the designer applies what is adopted.

**Which process.**
- **The claim.** The run was claimed at skein `15ddb75`, before `e6fd9fb` gave runs a `kit/`. Until ruling 295, `plan`, `land`, `heavy` and the documents the roles read came from skein's primary checkout. Other sessions fast-forwarded that checkout eleven times before the pin. It reached `841f487` at 17:52:56 on 10-17, carrying `71f6c38`'s compaction in place.
- **The pin.** Ruling 295 created `kit/` at `841f487` at 23:43:29 on 10-17, at the director's request, five minutes before kit-batch's first landing. No ruling moved it after that.
- **What `supervision.md` recorded.** Its Process lines say the run kept `15ddb75` and took only the points the supervisor named from five later commits. That was true of the practice those points describe. It was not true of the tools (section 5, item 9; section 7).

## 1. In one page

- **159 steps landed: 137 planned, 22 added, none dropped. "Done when" passed except item 11.**
  - Item 11 missed all four test-time aims, each by under 5 s and every suite within its budget (ruling 402).
  - jig's old client crates are gone, and the ten new ones exist.
  - The wire carries no shape of work. jig's component owns no listener.
  - temper's client is its own, pinned to jig `bbea94081`, the plan's last jig landing.
- **The longest run on record: 63.5 hours, 0.38 h a landed step.**
  - The first 24 hours landed 86 steps, with up to eight sessions at once.
  - After 06:00 on 10-17 it ran one or two sessions at a time, along the client track and temper's follow-ups.
  - Six of the 22 added steps came in its last 11 hours.
- **The plan was rewritten as it ran.**
  - There were 302 rulings, 1.9 a landed step. 75 of the 194 blocks were plan defects, and 43 rulings corrected a Size.
  - `decisions.md` grew from 12 KB to 568 KB, and the README from 29 KB to 82 KB.
  - The session files were written at the base, up to 54 KB each, and every landing moved what the later ones named (proposal 1).
- **Context was the cost.** The run took 4.3 billion Codex input tokens and $795 of Claude Code: 41.5M tokens a landed step, the most on record.
  - **Implementers: 21.1M a step.** A thread had read about 96K tokens by its 30th request, mostly the README's rules, its session file and the design (proposal 2). 28.5% of their requests only polled a running command, most of them returning at 31 s (proposal 3).
  - **The coordinator: 5.96M a step.** It compacted at 330K to 385K, ten times; the 200K limit never reached its thread. A doorbell ring took six or seven requests (proposals 4 and 5).
  - **The supervisor: $604, $3.80 a step.**
    - Its 155 sub-agents cost $338, and re-read `log.md`, `supervision.md` and development.md whole (proposal 6).
    - Restoring context after its 36 starts and 25 compactions cost $117.
- **The tools moved under the run, then stood still.**
  - Before the pin, `71f6c38`'s word "hand over at the second compaction" reached six supervisors, although the run had "not taken" that commit.
  - After the pin, the fixes for defects the run was meeting stayed out: the lost ref lock, the coordinator's compaction limit, loading a thread before a ring, pueued's restart and the 65-minute check. The run met each by hand (proposal 4).
- **The run broke temper's process tier, and no one saw it for 27 hours.**
  - temper 03.5 landed at 10:36 on 10-17 with its process Proof "unavailable", while ruling 163 had the landing story ignored. 04.3 and 05.1 broke it further.
  - forge-setup's director found it at 11:55 on 10-18. Fixing it took two added steps and 13 rulings, and every temper landing then ran the whole tier (proposal 7).
- **btrfs ran out of metadata room at 04:04 on 10-18, with `df` showing 98 GiB free.**
  - pueued and Codex's daemon died, and four implementer threads were lost.
  - The process tier lost its cgroup for three hours.
  - The supervisor balanced the filesystem by hand from 04:13 to 12:36 (proposal 8).
- **What held:**
  - blocks were answered in a median of 2.7 minutes;
  - 32 design questions were answered in a median of 15 minutes, each landed in the designs;
  - ready checks caught defects before sessions started;
  - the release came 18 minutes after `done.md`, with nothing dirty discarded.
- **Nine proposals** (section 8):
  - long plans run in parts;
  - the rules every thread reads stay short;
  - a wait counts by when it returned;
  - a kit fix for a defect the run meets reaches it;
  - a ring is one command;
  - sub-agents and designers read ranges, not whole files;
  - no Proof lands "unavailable";
  - btrfs space is a resource the kit keeps;
  - the metrics map their words.

## 2. Outcome

`done.md`'s "Done when, run 2026-10-18T23:14:55Z" holds each command's output. A Sonnet sub-agent of the supervisor ran them once, from `origin/main`, and ran item 11 through `heavy --gate`.

| Item | Result |
|---|---|
| 1. Every step landed or dropped | **Met.** 159 of 159 landed. |
| 2. jig's old client crates gone | **Met.** `ls-tree` prints nothing. |
| 3. jig's ten new client crates | **Met.** It prints 10. |
| 4. The naming checks | **Met.** `PENDING: [&str; 0]`; `client_crates_carry_the_client_prefix` passes in jig and in temper. |
| 5. No shape of work on the wire | **Met.** The grep prints nothing. |
| 6. jig's component owns no listener | **Met.** |
| 7. jig's gate has the browser suite and the wasm32 builds | **Met.** Four lines of `scripts/check.sh`. |
| 8. temper's client is its own | **Met.** No `jig-client-view`, `jig-domain-client` or `jig-view` in temper's manifests. temper's `Cargo.lock` still reaches `jig-client-view` through `jig-client-world` (section 9, item 16). |
| 9. temper pins jig's last landing | **Met.** `bbea94081`, which is jig-22.s1's. |
| 10. The plan's Step trailers | **Met.** All 137 planned steps are listed. Other lines: the 22 supplements, and `temper-06.s3`, which ruling 335 counts as 06.3; one empty line each in jig and temper, from a commit with no Step trailer; and four `temper-NN.N-repin` names, from ungated repin commits within steps. |
| 11. The test-time aims | **Missed**, recorded in ruling 402 with the three slowest tests of each suite. Medians of three: jig focused 14.283 s against 13.3, fuzzy 26.789 s against 24.91; temper focused 6.476 s against 6.0, fuzzy 23.885 s against 19.36. Every run was within its budget. |
| 12. No budget raised | **Met, as ruling 401 amended it at 22:53**, 21 minutes before the run. jig shows the browser profile's lines and ruling 275's focused 15 s to 30 s. temper shows ruling 326's scheduling override, which the item had not allowed for. |
| 13. Each store family moved once | **Met.** It prints 3. |
| 14. One definition of the wire's version | **Met.** One line, `crates/jig-client-protocol-wire/src/lib.rs:12`, `VERSION: u32 = 8`. |
| 15. The client's transitional pieces gone | **Met.** Both commands print nothing. |
| 16. No other aim | **Met.** |

What the plan names but leaves for later:
- **The browser's back and forward.** Ruling 393 deferred them to temper's later.md, since skein's pin was capped. The browser tier ends at 13 stories.
- **The brief's 64-digit commit spelling.** Ruling 399 deferred it to temper's later.md and STATUS.md.
- **The rows `later.md` already held.**

The six steps of jig's STATUS.md, section 3, are on `main`.

## 3. Numbers

`plan metrics` ran without error from skein's checkout at `e9cc93b` and wrote `metrics.tsv`. The comparisons:
- jig-protocol, the largest run: 217 steps;
- render, the latest large run: 168 steps;
- parity, the last retrospective's run;
- forge-setup, the run that shared temper's `main` with this one;
- protocol, temper's earlier large run.

| Metric | client-api | Notes | jig-protocol | render | parity | forge-setup | protocol |
|---|---|---|---|---|---|---|---|
| `wall_h`, `running_h` | 63.5, 60.0 | The longest run on record. The coordinator's thread was active almost throughout. | 29.8, 29.5 | 13.8, 13.4 | 33.9, 13.3 | 43.8, 7.9 | 18.5, 16.9 |
| `steps_planned`, `added`, `dropped` | 137, 22, 0 | All `.s<k>`: 12 in jig, 4 in skein, 6 in temper. Six were added after 12:00 on 10-18. | 166, 51, 0 | 151, 17, 0 | 47, 11, 0 | 40, 4, 0 | 112, 3, 22 |
| `step_h` | 0.38 | The highest on record. The first 24 h landed 86 steps; the last 39.5 h landed 73. | 0.14 | 0.08 | 0.23 | 0.18 | 0.15 |
| `escalations` | 194: plan defect 75, process 48, cross-session scope 46, design gap 19, infrastructure 6 | 1.22 a step, against jig-protocol's 1.18. `metrics.tsv` splits "plan defect." (6), "process." (5), "design gap." (1) and "infrastructure." (1) from their kinds; they are added in here. `log.md` holds 196 distinct blocks. | 256 | 48 | 36 | 29 | 71 |
| `escalation_wait_min` | 2.7 | p90 12.6 min; 8 waits over 30 min. | 1.6 | 2.7 | 2.5 | 1.7 | (−153) |
| `blocked_h` | 13.9 | The most on record: blocks overlapped while up to eight sessions ran. | 8.2 | 2.6 | 7.3 | 3.2 | (−277.7) |
| `answers_reversed` | 9 | Undercounted: 42 of rulings 251 to 402 and 39 of 101 to 250 replace earlier text. Only 9 name what they reverse (section 5, item 14). | 38 | 2 | 1 | 2 | 5 |
| `steers` | 64 sent, 18 points refused | 15 of the 18 points were refused because their run-long text was not yet in the README (ruling 167). | 24, 0 | 27, 1 | 11, 0 | 13, 1 | 5, 0 |
| `restarts` | crash 1, unknown 4 | The crash is the btrfs fault's. Of the four "unknown", `log.md` gives a cause for two: the network outage at 16:12 on 10-18, and an explicit resume prompt at 22:55. | | | | loaded 6, reboot 1, the user's request 1 | the 1 |
| `tokens_coordinator` | 947.0M in (99% cached), 1.58M out | 5.96M a step, the most on record. | 699.1M; 1.50M | 401.3M; 0.70M | 233.1M; 0.41M | 249.7M; 0.37M | 271.0M; 0.62M |
| `tokens_implementers` | 3,349.8M in (98%), 8.29M out | 21.1M a step: 1.7 times jig-protocol's 12.7M and 3.2 times render's 6.7M. | 2,761.3M; 9.85M | 1,122.6M; 4.40M | 779.1M; 1.69M | 320.8M; 0.95M | 870.4M; 3.41M |
| `tokens_claude` | 2,284.3M in (97% cached), 13.84M out; 0 before the claim, 2,281.3M in the run, 3.0M after | "0 before the claim": the design and the planning were done on x1, and their transcripts are not on this machine. | 5,160.9M | 880.9M | 565.6M | 667.6M | 520.1M |
| `cost_claude` | $795: supervisor $604, designer $189, retrospective $1; cache reads $355, writes $229, output $210 | $5.00 a step; the supervisor's share is $3.80, 2.3 times render's $1.64. Planning on x1 counts as $0, under 41 named sessions. The pulse's figure at 22:52 on 10-18 was $1,004: the kit's pulse priced Sonnet at Opus's rates. | | $312 | $231 | $197 | $192 |
| `supervisor` | 36 sessions, 65.0 h, 1.4M weighted tokens an hour, 36% in the 40 requests after each start or compaction, 25 compactions | Four handover rules in turn (section 5, item 8). Only render and forge-setup have this metric. | | 15, 13.4 h, 2.7M, 63%, 28 | | 6, 43.6 h, 0.4M, 17%, 0 | |
| `tokens_per_step` | 41.5M | Coordinator 6.0M, implementers 21.1M, the run's Claude Code sessions about 14.4M. | 39.8M | 14.4M | 23.0M | 23.4M | 13.3M |
| `coordinator_requests` | 4,655, 325 after a wait ran out | 29.3 a step. All 921 waits asked for 300 s. | 3,567, 67 | 1,961, 32 | 1,169, 48 | 1,303, 243 | 1,404, 60 |
| `compactions` | 10 | Each at 330K to 385K (section 5, item 7). | 9 | 4 | 2 | 2 | 4 |
| `gates_per_step` | 1.8 | 285 land gates, 186 passed. 99 failed, all on a test or a timeout; none on clippy, a build or the machine. | 2.2 | 1.3 | 1.8 | 1.5 | 2.3 |
| `gate_lane_other` | 37 | 15 measurements, 11 process-tier runs, 4 temper gates outside `land`, 3 probes, 2 scratch checks, 2 for "Done when". | | 21 | 103 | 148 | 0 |
| `queue_wait_ratio` | 0.94 | The build lane waited 60.5 h for 56.7 h of running. 87% of the waiting was in the first 24 h. The kit's `heavy` writes its row after the task, so the figure may overstate (wall-clock's retrospective). | 2.24 | 0.74 | 0.74 | 1.34 | 0.31 |
| `suites` | **jig:** focused 3,568 / 9.935 s → 4,270 / 13.190 s; fuzzy 109 / 16.919 s → 111 / 24.072 s; browser 13 / 18.033 s new; gate 36 s → 165 s. **temper:** 1,254 / 5.016 s → 1,879 / 7.557 s; fuzzy 46 / 20.452 s → 50 / 24.690 s. **skein:** 153 s → 76 s. | jig's focused suite grew 33% in time for 20% more tests. | | | | | |
| `proofs` | unit 25, world 43, joint 90, process 26, gate 5, browser 4, one 2, conformance 1, none 4; 0 below plan | "gate", "browser", "one" and "conformance" are no layer's words, and the metric counts them anyway. Three Proofs say "unavailable", all for temper's process tier (section 5, item 10). | unit 92, world 71, joint 15, process 1, live 1 | unit 52, world 61, joint 11, process 71, live 10 | unit 29, world 11, process 2, live 12 | unit 18, world 17, joint 3, process 25 | unit 67, world 28, joint 8, process 11, live 1, none 77 |
| `report_lines` | 12 median, 18 p90 | 430 reports for 159 steps. | 10, 11 | 12, 16 | 11, 16 | 16, 24 | 13, 16 |
| `lines` | code +40,800 −24,584; tests +69,605 −30,102; generated +70,095 −31,721; data +5,252 −3,851 | 165 gated landings. jig-21.1 alone added 24,681, 23,925 of them generated. | code +355,989 −129,879; tests +114,950 −18,650 | code +45,176 −3,776; tests +72,259 −3,985 | code +74,784 −93,312; tests +9,804 −1,518 | code +11,733 −1,002; tests +12,443 −1,020 | code +85,628 −23,242; tests +40,123 −6,166 |
| `contexts` | 186K median, 332K p90 of the sessions' peaks; 150 handovers | Handovers at a median of 193K; 113 were past 150K and 16 past 300K. | 238K, 319K; 70 | 172K, 223K; 66 | 201K, 293K; 26 | 152K, 208K; 19 | 225K, 262K; 45 |
| `pin_lag` | temper: jig 2, skein 97, smith 197; jig: skein 98, smith 197 | skein repins were capped at `2733882` (rulings 154 and 296). | | | | | |

**Not comparable, and why:**
- **`escalations`** splits four kinds by a trailing full stop (proposal 9).
- **`tokens_claude` and `cost_claude`** count no design and no planning.
  - The design (jig `28901d98`, temper `c7095516`) and the plan (temper `d9b68d483`, 40 Planner-Session trailers) were written on x1, in one session, 738807d8, and its agents. The planner wrote the plan in the design's session, as parity's proposal 14 was being adopted (section 7).
  - `numbers.sh` takes the last commit with a raw `Planner-Session:` line as the plan's landing. That is `5cfc5a2ba`, a README fix whose line is not a trailer, so the window for design sessions is wrong too (proposal 9).
  - The 41 sessions it names as missing are all on x1.
- **`proofs`** counts first words that are no layer's.
- **`restarts`** leaves "unknown" where `log.md` gives the cause.
- **`supervisor`**'s 25 compactions are the transcripts'. `supervision.md` records 22.
- **`queue_wait_ratio`**: wall-clock's retrospective measured 0.97 printed against 0.17 true, with the same `heavy`. `817b15c`, which writes the row inside the task, was not in this run's kit.
- **jig-protocol's `cost_claude` and `supervisor`** were never computed. protocol's escalation wait and blocked hours are negative from an old parse.

**What explains the largest differences:**
- **Hours per step, 0.38.**
  - **The chain.** The README's longest chain is 22 jig increments on the client track (01 → 11 → 12 → 13 → 21 → 14 → 22 → 15 → 16 → 17 → 18 → 19 → 20). temper's repins follow it, and step 2's sessions, jig 02 to 09, run beside it.
  - **After the first day.** Step 2's 43 increments had all landed by 03:59 on 10-17. After that only the chain and temper's sessions behind it could run. Implementers running, sampled at each block, had a median of 7 and 6 in the first two windows, then 4, then 1 from 06:00 on 10-17 on.
  - **Blocks along the chain.** temper 03 had 21 blocks, jig 21.2 nine in 1 h 44 min, temper-04.s1 seven. Each block stopped the only work there was.
  - **Late additions.** Steps added late extended it: jig-15.s1 and temper-07.s1 at 19:58 on 10-18 (ruling 394), and jig-22.s1 at 22:12 (ruling 399).
- **Implementers' tokens, 21.1M a step.** Three things carried them:
  - **Contexts.** The reading done before work, carried by every later request (section 5, item 5).
  - **Polls.** Requests that only polled a running command: 1,006.6M, 30% of their input.
  - **Size.** Long sessions over large session files: jig-16 used 278.8M over 10 threads, jig-21 269.8M.
- **The coordinator's 5.96M a step.**
  - **Contexts.** Half its requests were past 199K, and those past 200K carried 68% of its input.
  - **Rings.** Each of 314 rings took a median of six to seven requests: read the file, copy it into `log.md` with a script, send, set the ledger, wait. From each ring to the next wait took 1,870 requests and 384M, 41% of its input.
  - **Timed-out waits.** 230 of the 325 waits that ran out were followed by the heartbeat and another wait, nothing else: 460 requests and 99M.
- **The supervisor's $604:**
  - **Sub-agents, $338.** The hourly checks cost $133 (52 agents), the ready checks $107 (43), the re-checks $54 (27), and diagnoses and plan corrections $44. The 39 Opus sub-agents before the switch to Sonnet (17:55 on 10-16) cost $121 of the $338.
  - **Main threads, $266.** $117 of it went to the 40 requests after each start or compaction, and $125 to the turns in which a ruling landed.
  - **By phase:**

    | Phase | Sessions | Handover rule | $ an hour |
    |---|---|---|---|
    | 10-16 07:48 to 17:54 | 5 | about 300K | $17.3 |
    | 10-16 17:54 to 10-17 07:56 | 20 | the director's 150K | $10.7 |
    | 10-17 07:56 to 20:15 | 4 | 300K | $8.6 |
    | 10-17 20:15 to 10-18 07:01 | 6 | the second compaction | $8.7 |
    | 10-18 07:01 to 23:19 | 1 | none, 13 compactions | $4.9 |

    The first phase cost $3.00 a ruling and the last $1.02. Sonnet sub-agents, and fewer of them a ruling, explain most of the fall.
- **Queue waits.**
  - **The first 24 hours.** 87% of the build lane's 60.5 hours of waiting fell then, with 12 to 14 sessions queuing tasks in each six-hour window, and up to 14 tasks waiting at once.
  - **Gates.** 32% of all waiting fell while a gate held the lane. Land gates spent 4.2 of their 7.6 hours draining running build tasks before they started.
  - **Wasted passes.** On the first morning, 7 of 14 passing gates were thrown away because `main` moved while they queued (ruling 119).

## 4. What worked

1. **The plan's aim was met.** 159 of 159 steps landed, none dropped, and 15 of 16 "Done when" items passed.
   - jig's client went from the old crates to the ten new ones, with the browser shell and its 13 stories in jig's gate.
   - temper's pages are its own.
   - The design's six steps are on `main`.
   - All 165 gated landings passed their gate. The 99 failing gates landed nothing.
2. **Blocks were answered fast, at a scale no run had met.**
   - 196 blocks had a median wait of 2.7 minutes and a p90 of 12.6; 8 waited over 30 minutes.
   - The longest two, 84 and 86 minutes, waited on a designer whose network failed (EAI_AGAIN).
3. **The design moved with the run, and the questions were real.**
   - 32 questions reached the designer, in 29 sessions. The median wait was 15 minutes from question to the last design commit, and the longest 84 minutes.
   - Each answer landed in the designs: jig's and temper's, and four times skein's. All 29 design rulings name their commits, and each commit exists (section 6).
   - 13 of the 32 answers differed from the supervisor's own reading, so routing meaning to the designer (parity's proposal 9) paid.
4. **Ready checks and re-checks caught defects before work started.**
   - 53 rulings came from them.
   - jig-13.1's check found 17 items, among them a fuzzy test's compile break.
   - jig-23.1's re-check found that the Proof left out `jig-ops-world`, where its prototype failed 15 tests.
   - jig-14's check found that `people::Member` had no deadline, which went to the designer before the session started (ruling 224).
5. **Bisects found regressions within minutes once something ran.**
   - temper 04.s1's gate failed at 11:33 on 10-18. Ruling 339 bisected, and it named jig `7d3becac7`, jig-17.s2's landing of 07:24.
   - 05.s1's whole process tier found `3262df8f9`'s broken signed delivery 39 minutes after it landed.
   - jig-19.5's bisect named `73335dab7`.
6. **The disk fault was diagnosed in 18 minutes.**
   - First sight was 04:04 on 10-18, the three balances ran from 04:13 to 04:16, and ruling 309 landed at 04:22.
   - The coordinator was back by 04:20. Nothing landed was lost, and the four lost threads were restarted from their checkpoints.
7. **The supervisor's sub-agents on Sonnet** (forgejo-17's proposal 5, from 17:55 on 10-16):
   - a sub-agent cost $1.66 to $2.02, against $3.10, with the same number of requests;
   - sub-agent dollars per ruling fell from $2.09 in the first phase to $0.31 in the last.
8. **One supervisor, compacting in place, held the last 16.3 hours.**
   - The session was `b6c57fcc`, with 13 compactions and no handover, after ruling 331's point 10 (render's proposal 10).
   - It cost $4.9 an hour and $1.02 a ruling, the cheapest phase.
9. **The 100K handover steer (08:40 on 10-18) moved the implementers.**
   - Handover contexts went from a median of 213K (58 of 98 past 200K) to 140K (9 of 52).
10. **The end was clean.**
    - `done.md` came at 23:00:05, the supervisor's "Done when" from 23:02 to 23:14, and ruling 402 recorded item 11's misses.
    - The release came at 23:18:07, after "done when ran" (render's proposal 2).
    - 54 clean worktrees were removed, and six dirty ones were moved intact to `preserved-worktrees/`.

## 5. What did not

1. **The run took 63.5 hours, and two thirds of them ran one or two sessions at a time.**
   - **Cause: the plan's shape.**
     - The client track is a chain of 22 jig increments, and temper's sessions each wait on one of its landings.
     - Step 2's 43 increments, the only work beside the chain, had all landed by 03:59 on 10-17.
     - Steps added late extended the tail: jig-15.s1 and temper-07.s1 at 19:58 on 10-18, and jig-22.s1 at 22:12.
   - **Long stretches with no landing.** There were three of over two hours:
     - 05:52 to 08:08 on 10-17: jig 21.2's nine blocks and five handovers.
     - 08:09 to 10:21 on 10-17: temper 03.5's seven blocks and four handovers in 63 minutes.
     - 15:01 to 17:04 on 10-18: jig 19.5 and temper 06.6, both waiting on the designer.
   - **First visible:** from 06:00 on 10-17, when the median of running sessions fell to one. The coordinator's idle notes say "no independent session eligible".
   - **Acted on:**
     - The README already put the chain first.
     - Two steers started temper 04.1 and jig 14.1 at once.
     - temper 04.1 had been held 14 hours by a README row that read "temper 03" where ruling 143 said 03.4, until the director asked at 07:53 on 10-17 (ruling 231).
   - Proposal 1.
2. **The plan was rewritten while it ran.**
   - **Rulings.** There were 302, 1.9 a landed step, against forge-setup's 1.27. 130 corrected the plan:
     - Shapes against a `main` that had moved, 23 in rulings 251 to 402 alone. Some sessions were re-checked twice: 15.1, 16.5 and 17.1.
     - Files lists missing a file or its users, 39 in rulings 101 to 250.
     - Shapes naming a symbol, path or state `main` lacked, 10 in rulings 101 to 250.
     - Sketches failing the workspace's lints, 11.
     - The order of sessions, 14.
   - **Cause: the session files.**
     - 31 files, 976 KB, were written at the base for a run of 159 landings: 31 KB on average and up to 54 KB, about 13K tokens. By the end the largest was 67 KB.
     - Each landing moved the lines, the users and the states a later file named. The plan's compiled sketches (parity's proposal 10) were right only at the base.
   - **Growth.** The plan grew from 1.1 MB to 2.0 MB. `decisions.md` went from 12 KB to 568 KB and the README from 457 to 1,217 lines.
   - **First visible:** ruling 101 at 08:09 on 10-16, 21 minutes after the claim.
   - **Acted on:**
     - 43 ready checks and 27 re-checks, which cost $161 of sub-agents.
     - A re-check never held a start (forgejo-17's proposal 4).
   - Proposal 1.
3. **The plan's Sizes were low.**
   - **The count.** 43 rulings moved a Size or a stop. The worst:
     - jig 21.2: 250, then 950, then 4,500; it landed at 4,352.
     - jig 21.3: 320 to 2,500.
     - temper 07.4: 140, then 1,000, then 2,000, ending in a design gap at 4,442 copied lines (ruling 394).
     - temper 06.3 and 06.5: 603, then 650, then 700.
   - **Splits.** Seven requests to split were refused.
   - **The cause** is the one forge-setup met: Sizes estimated from the design, not measured against comparable landings.
   - **The fix not taken.** render's proposal 5 (an increment past its Size lands without asking, as long as it keeps to its Files) was adopted at 07:53 on 10-18. This run's process never took it, and seven more Size rulings followed (330, 342, 345, 359, 365, 388, 389).
   - No new proposal: render's 5 applies to the next run.
4. **Steers became README rules, and the rules every thread reads grew to 886 lines.**
   - **How the rules grew.**
     - Ruling 167, at 19:44 on 10-16, put each run-long steer text into the README so that new threads would read it. forge-setup's proposal 8 adopted the same rule.
     - 32 rulings then turned steers into rules, and 38 rulings of 101 to 250 changed the README.
     - "Rules of this plan" ran from line 175 to line 1,061 by the end.
   - **Refused points.** The coordinator refused 15 steer points because their text had not yet reached the README (rulings 223, 259, 341, 379).
     - Rulings 252 and 311 withdrew steers that repeated rules already written.
     - Rulings 387 and 395 restated earlier ones.
   - **The cost.** Every implementer thread starts by reading this.
     - Median context is 53K at the thread's 10th request, 78K at its 20th and 96K at its 30th.
     - The fixed prefix, plus the context built in a thread's first 30 requests and carried by every later one, is 56% of implementers' input: 1,872M.
     - The supervisor's main threads read the README 242 times, and 71 of its sub-agents read it.
   - **First visible:** ruling 167 itself, at 19:44 on 10-16, whose cost no check measured.
   - **Acted on:** not.
   - Proposal 2.
5. **Implementers' contexts were never within their limit.**
   - **The handovers.** 150 handovers, at a median of 193K; 113 were past 150K and 67 past 200K.
   - **Over 200K.** 85 threads went past 200K and 29 past 300K. Their requests above 200K carried 38.5% of implementers' input. 21 threads compacted, at 301K to 404K.
   - **The limits.**
     - The card's limit was 150K until the 08:40 steer on 10-18, then 100K.
     - 183 of 197 threads passed 100K by their 31st request, a median of five minutes in, so the 100K limit binds at the first landing at the latest.
     - The steer of 21:01 on 10-16 ("past 150K hands over at its landing") changed little: the median afterwards was 220K.
   - **First visible:** `plan contexts` from the first night. Contexts at handover were logged on every `Handover` line.
   - **Acted on:** the 08:40 steer (section 4, item 9).
   - Proposal 2: the reading is what fills them.
6. **28.5% of implementers' requests only polled a running command.**
   - **The count.** 6,643 requests, 1,006.6M of input. 72.7% of the polls found the command still running: 716.6M, 21.4% of implementers' input, and 67.6 hours of waiting.
   - **Cause.** On Codex, a `write_stdin` call returns at 31 s whatever it asks for. Only the `// @exec: {"yield_time_ms": 300000}` line waits longer. 1,951 polls asked for 60 s or 300 s without the line and returned at 31.0 s.
   - **The pulse missed it.** `short_waits` counts a wait by what it asked for, so the polls that asked for 300 s counted as compliant.
     - forge-setup's proposal 7 adopted "every Codex role waits 300 s, and the pulse counts the waits that ran out in less".
     - The card on `main` says the threads without `// @exec` "waited in 32% to 57% of their requests".
   - **First visible:** never, as a breach.
   - **Acted on:** not.
   - Proposal 3.
7. **The coordinator compacted at 330K to 385K, and a ring took six or seven requests.**
   - **The compactions.** Ten, each back to about 25K. Three requests later the context was at 38K to 58K, from re-reading `context.md` and the kit.
     - `c0268a8` (21:54 on 10-16) set `model_auto_compact_token_limit = 200000` for runs claimed after it.
     - `37c676e` (01:36 on 10-18) passed it into a resumed thread.
     - Neither reached this thread: the first came after the claim, the second after the pin.
   - **Rings.** A ring was: the wake-up command, a read of `unblock.md` or `steer.md`, a Python script copying it into `log.md`, the send, `plan set`, the wait.
     - Python scripts were 462K of its output tokens, 29% of it.
     - From each ring to the next wait took 40% of its requests and 41% of its input.
   - **Timed-out waits.** 230 of them were followed only by the heartbeat and the next wait.
   - **First visible:** the pulse's `contexts` line, 328K at 22:28 on 10-16.
   - **Acted on:** not.
   - Proposals 4 and 5.
8. **The supervisor cost $604 under four handover rules.**
   - **The sessions.** 36, with no crash and no usage stop.
     - **Sessions 1 to 4** handed over at about 300K.
     - **Sessions 5 to 25** handed over at 127K to 161K under the director's 150K message of 17:49 on 10-16. That made 20 sessions in 14 hours, each restoring 80K to 90K: 57% of the main threads' dollars went to restores. The director's note in `2f317f5` (12:36 on 10-17) measured it: 44% of weighted tokens in the first 40 requests, against 16% before.
     - **Sessions 26 to 29** went back to about 300K, and `supervision.md` never says why.
     - **Sessions 30 to 35** handed over after their second compaction. That was `71f6c38`'s word, which reached them through the drifting tools (item 9).
     - **Session 36** ran 16.3 hours with 13 compactions, once ruling 331's point 10 ended handover by count.
   - **Sub-agents, $338.** They re-read the run's largest files whole:
     - `log.md` (2 MB), by 53 agents, 1.64 MB in all;
     - `supervision.md` (596 KB), by 54, 1.50 MB;
     - development.md, by 56, 1.12 MB;
     - the README, by 71, 1.11 MB;
     - `decisions.md`, by 60.
   - **The designers, $189.** They read `supervision.md` 123 times, 967 KB, for earlier answers. The late questions needed 80K to 217K of reading and up to four compactions.
   - **First visible:** the director's note of 12:36 on 10-17.
   - **Acted on:**
     - the switch to Sonnet;
     - the end of handovers by count (ruling 331, 08:39 on 10-18).
   - Proposal 6.
9. **The run's tools moved under it, then the pin kept their fixes out.**
   - **Before the pin.** The run had no `kit/`. `plan`, `land`, `heavy` and the documents the roles read came from skein's checkout, which other sessions fast-forwarded eleven times before the pin:
     - `c1453bd` at 08:21 on 10-16;
     - `d047b45` at 13:27, with `de0f5ba`'s repository table;
     - `1f71424` at 20:02, with `e6fd9fb`;
     - `4321f97` at 22:03, with `c0268a8`;
     - `841f487` at 17:52 on 10-17, with `7779af1`, `71f6c38` and `1b83a43`.
     - `supervision.md`'s Process lines said the run did "not take" commits its tools were running.
   - **The pin.** Ruling 295 pinned `kit/` at `841f487` at 23:43 on 10-17, at the director's request, so that kit-batch's 29 commits, landing from 23:48, would not reach a run in flight. Ruling 310 confirmed it.
   - **The defects then met by hand**, each with its fix on `main` by then:
     - **The lost ref lock.** Ruling 315 at 05:47 on 10-18; `013f9b4` was 5.9 hours older.
     - **The compaction limit.** The coordinator compacted at 330K to 361K six more times; `37c676e` landed at 01:36.
     - **Hand loads.** Four hand loads of a coordinator thread "not found", 03:50 to 04:19; `fea416f` and `0b23b6b`, 01:03 and 01:22.
     - **pueued's restart.** pueued was restarted by hand at 04:17, outside its cgroup, and the process tier was unavailable until 07:17 (ruling 324); `7ee9672`, 00:38.
     - **The hourly alarm.** A check gap of 251 minutes, 04:05 to 08:16, during handovers 33 to 35; `0d605f5`, 01:34.
     - **Step accounting.** `plan landed`, which checks a landing's Step trailer, came in `2c4db1d` (00:22). Without it a commit trailered `temper-06.s3` landed unchecked at 09:04, and ruling 335 had to count it as 06.3.
   - **First visible:** ruling 310 at 04:23 on 10-18 named the trade.
   - **Acted on:** by hand, each time.
   - Proposal 4.
10. **The run broke temper's process tier for 27 hours, and another run found it.**
    - **The cause.** Ruling 163 (18:36 on 10-16) had the landing story `#[ignore]`d until temper 04.4.
      - temper 03.5 (`fe8fcdf6b`, 10:36 on 10-17) landed with "Proof: … unavailable under ruling 163; joint fallback passed".
      - 04.3 (`e461dfd8d`, 05:38 on 10-18) and 05.1 (`3f8b3df6f`, 07:44) broke the story further. Their process Proofs ran a smoke test.
      - temper-01.1 and 06.1 also landed with "unavailable" process Proofs.
    - **First visible:** forge-setup's director, at 11:55 on 10-18, on the shared `main`. forge-setup's retrospective puts the story red from 08:56 at the latest, possibly from 10:36 on 10-17.
    - **Acted on:**
      - Ruling 347 at 12:17 added temper-05.s1, which landed at 13:48.
      - 04.s1's `3262df8f9` (13:10) then broke the signed delivery. Ruling 367 added 05.s2, which landed at 14:42.
      - 13 rulings, from 339 to 372, dealt with these regressions, and temper-04.s1 alone blocked seven times. Ruling 369 made the whole tier run before every temper landing.
    - **The cost to forge-setup.** It waited on these fixes for 4 h 25 min.
    - Proposal 7.
11. **btrfs ran out of metadata room.**
    - **What failed.** At 04:04 on 10-18, `/srv` had 1 MiB unallocated, its metadata at 3.80 of 4.32 GiB, while `df` showed 98 GiB free.
      - mold's linking died with Bus errors (SIGBUS) and "Disk full?".
      - pueued and Codex's daemon died, at 03:47:45 and about 04:13:42.
      - Four implementer threads were lost: 161 requests and 15.8M tokens, redone.
      - pueued, restarted by hand, lost the cgroup the process tier needs for three hours.
    - **Acted on, by hand:**
      - the supervisor's three balances (04:13 to 04:16), then a balance below 5 GiB unallocated at each disk check (ruling 309);
      - a dozen more balances, 04:40 to 12:36;
      - the trigger raised to 10 GiB (ruling 331), and `-dusage=75` when data was 90% used (ruling 334).
    - **The gap.** development.md 11.4 bounds disk and memory, not a filesystem's unallocated space, and `plan check` does not read it.
    - Proposal 8.
12. **The test-time aims were missed, and jig's focused budget was raised.**
    - **jig.**
      - The focused suite went from 9.9 s to 14.3 s (+44%) for 702 more tests (+20%).
      - Five land gates timed out at 15 s from 17:01 to 17:09 on 10-17, every test passing. Two of them coincided with the check's own un-niced pulse and a downstream run.
      - Ruling 275 raised the focused `global-timeout` to 30 s for the rest of the run, by the user's rule of 2026-10-13. It is still 30 s on `main`, and "For the user" F has no answer (section 6).
    - **The repair step.** jig-14.s2 (ruling 269) was added on ruling 267's premise, which ruling 268 reversed. It had no measured cause, and it left the suite 0.56 s slower.
    - **temper.** Ruling 326 made temper's aims guides. It also made "Done when" item 12 false, until ruling 401 amended the item 21 minutes before the end.
    - **SLOW flags.** jig's "Done when" runs flagged 20 to 22 focused tests SLOW with the machine alone. Ruling 400 made a flag under a test's period report-only.
    - No new proposal. testing-strategy.md 8 ("a slow test is understood, never hidden", `b4c6241`) landed at the run's end, and the pass after the run is the debt of section 9.
13. **Landing was used as a test runner, and gates failed by the dozen.**
    - **The count.** 99 of 285 land gates failed, 35%, using 1.75 hours of the gate lane.
      - jig-13 failed 19 of 26 gates between 04:30 and 05:16 on 10-17, each in 11 to 41 s.
      - jig-21 failed 11 of 15, and jig-19 9 of 15.
      - 14 failures were jig-boundary guards, and 6 jig's text-page goldens.
    - **What would have caught them.** Those suites run in seconds alone, before `land`.
    - No proposal of its own.
14. **The record disagrees with itself.**
    - **`Reverses:`.** `decisions.md` has no `Reverses:` line. They are in the unblock texts in `log.md`.
      - Of rulings 251 to 402, 42 replace or narrow earlier text and 7 name it.
      - Of rulings 101 to 250, 39 replace text and 2 name it.
      - forge-setup's proposal 12 ("every replacement names it") was adopted after this run's claim.
    - **Times.** Ruling headings carry no time. Designer headings were off by up to an hour. Two steers, and ruling 402's title, carry the wrong time.
    - **Trailers.**
      - 10 of the 29 designer sessions wrote `Designer-Session:` in a paragraph git does not read as trailers, before `114db4d` made `land` refuse that (07:57 on 10-18). temper `3a323b3b2` (12:05) still did.
      - `temper/03-s2-repin-sweep.md` (ruling 199) has no row in the README's sessions table.
    - **Pin.** temper's pin of jig went backwards within one landing (`e461dfd8d`, 05:38 on 10-18), and the next repin moved it forward.
    - Proposal 9 covers the metrics' side.
15. **The hourly checks drifted.**
    - **Gaps.** 23 of 47 gaps between checks exceeded 65 minutes, and six exceeded 100:
      - 251 minutes, during handovers 33 to 35;
      - 183 minutes, a network outage from 09:32 to 11:15 on 10-18, with no alert raised;
      - 116 minutes, `b30cc95`'s one-time cron printing local time that the scheduler read as UTC.
    - **A sub-agent's slip.** One check's sub-agent ran `plan downstream --help`, which started a second downstream run.
    - parity's proposal 11 (the 65-minute alarm, `0d605f5`) was not in the kit. Proposal 4 bears on it.

## 6. Decisions taken during the run

302 rulings, 101 to 402, one commit each in temper, 08:09 on 10-16 to 23:14 on 10-18.

| Kind | Rulings | Reached the design? |
|---|---|---|
| Plan defect: Shapes, Files, Size, stale facts, lints, order | 130 | Plan only. Their Shapes changed session files. |
| Process: bounded diagnosis, steer plumbing, proof discipline | 33 | No design text, by kind. |
| Steer made a README rule | 32 | Plan only (section 5, item 4). |
| Design gap answered by the designer | 29, with 9 interim rulings | **Yes, every one** (the table below). |
| Budget and test time | 20 | No design text. Ruling 275's budget is in later.md. |
| Cross-session scope | 12 | Plan only. |
| Infrastructure | 10 | No design text: 309 and 334 (btrfs), 324 (pueued's cgroup), 315 (`land`'s ref lock), 295 (the kit), 296 (skein's cap). |
| Supplementary step added | 9 | The steps added by design rulings are counted there. |
| Step reordered | 6 | Plan only. |
| Regression of a landed step | 6 | Plan only. |
| "Done when" amended | 4 | Plan only: 271, 383, 401, 402. |
| Other | 2 | 154, 293. |

The design answers. Each names its commits, all on `main`:

| Ruling | Question | jig | temper | skein |
|---|---|---|---|---|
| 106 | What temper's census does with `EndedRead` rows | `9a24ded8a` | `97315175` | |
| 125 | A tracked conversation; a second grace | `a70de2906` | `ed6593899` | |
| 134 | How the host bounds the terminal's lines | | `3d695094` | `d047b45a` |
| 143 | The terminal's worst case; a path's segments; templates as JSON | `7efaeef4d`, `413726589` | `0625a5fa` | `8907ea3a` |
| 150 | What a run watch holds | `bab2e89cb` | `9ae5fca0` | |
| 165 | A forget lost in a crash | `134cbc189` | `4f86717c` | |
| 166 | A mark on a position not yet durable | `09a600c43` | `f92fddd0` | |
| 170 | A parked `NotStored` | `3b8bf1263` | `b694e960` | `1f71424d` |
| 180 | The owner's versions | `b09e2239e` | `21c0ec9a` | |
| 184 | A compound's outcomes, numbered | `a20cf96c4` | `90ed12e0` | |
| 190 | The funder of an accepted amendment | `1ee6c02ac` | `cf9493c5` | |
| 194 | What the bridge tells; the wall, `SignIn`, `Fact` | `64e1ab93b` | `8a14b8ed` | |
| 210 | Words that release a task | `5a9a97485` | `3cd97ab2` | |
| 224 | jig 14.4 and 14.5 against `main`'s people types | `b58a49c39` | `bc7c1376` | |
| 235 | Who keeps a read's page | `de4f80564` | `892f4e88` | |
| 246 | A start before the sign-in | `8c832e517` | `c3d0c0b9` | |
| 251 | What a cancel counts | `360aabcc6` | `63803dd15` | |
| 253 | A sign-in page after the engine ended the sign-in | `bbb6303aa` | `5b2faba9d` | |
| 262 | Whose project a role executor is | `ad3b26d04` | `ec988d053` | |
| 274 | The chip's status; sources from `me` | `6155e83e0` | `d55ffe2b2` | |
| 303 | The board's move once `Prioritise` goes | `ad465272b` | `baa38bb0a` | |
| 316 | Whether a watch holds its room until it ends | `544a57091`, `cd2cac35f` | `bf33cf174` | |
| 319 | Whether a reader lacking Amend loses a whole document | `35ab18732` | `405924109` | |
| 321 | temper's out-of-date `main`; an unnamed project's title | | `872fda7aa` | |
| 373 | One stream per client | `e444994d4` | `8e2e88622` | |
| 374 | A click on a button's text child | | `f5d63eaee` | `f7a4e7022` |
| 376 | Where a chat's waiting proposals show | `3150da02d` | `ffa4c1a20` | |
| 394 | Whose renderers temper's cards use | `2bd1a5cbb` | `076c659a5` | |
| 399 | Whether a byte parameter is text | `479a8f42d` | `e1e56152b` | |

**Not in the design yet:**
- **The server's eight connections.** Ruling 362 set them, and no design text names a count.
- **The browser's back and forward.** Ruling 393 deferred them, as a later.md row.
- **The brief's commit spelling.** Ruling 399 deferred the 64-digit spelling, in later.md and STATUS.md.
- **The busy answer to a reader past its bound.** Ruling 319 deferred it to later.md.
- **The fuzzy walking sweep.** Ruling 122 left its cost unrepaired.
- **The test-cost pass that restores 15 s.** Ruling 275 left it as a later.md row.

**"For the user"** (`decisions.md`, from line 3251):
- **A to D.** temper's scope, the naming check's home, the plan's home, and the two items of jig's step 5 that wait. The director read them and vetoed none, so the defaults stood.
- **E.** The process tier's cgroup (ruling 111). It was done at 14:58 on 10-16, lost at the hand restart of 04:17 on 10-18, and restored at 07:17 (ruling 324).
- **F.** jig's focused budget at 30 s (ruling 275). **It has no recorded answer from the user or the director.** Ruling 275 and later.md say 15 s comes back after the test-cost pass that follows the run, once the median of three gate-lane runs is at most 12 s. "Done when" measured 14.283 s.

## 7. The proposals adopted after the last retrospective

The last retrospective added before the claim was smith's parity (20:51 on 10-15). Its decisions were applied as text in skein `30bc10f`, at 07:25 on 10-16, 23 minutes before the claim. Its kit parts waited on `kit-after-parity` and landed in kit-batch from 23:48 on 10-17, after the run's pin.

The figures a step divide by planned and added steps: parity 58, client-api 159.

| Proposal | Decision | Meant to move | parity | client-api | Moved |
|---|---|---|---|---|---|
| 1. No role asks the user through a tool that waits | adopted | `blocked_h` | 7.3 | 13.9; no question waited on a tool. "For the user" E was pushed at once, and held nothing | No, for another cause (section 5, item 1) |
| 2. A live verdict suite planned as its own increment | adopted | | | No live suite | Not tested |
| 3. A chance verdict settles, or the run is done either way | adopted | | | Item 11's "done either way" held | Taken |
| 4. A change to what the model is sent reports its effect | deferred | | | | |
| 5. `plan idle` takes queue ids; ring and resume load the thread | adopted; kit | hand loads | 10 | 4, all in the 03:50 to 04:19 outage of 10-18 | In part: the kit part was not in the kit |
| 6. Hand over between increments past 150K | adopted | `contexts` p90; implementers' tokens a step | 293K; 13.4M | 332K; 21.1M. Median 213K until the 100K steer, then 140K | No |
| 7. `target/` goes at each landing; disk bounds | adopted in part | infrastructure escalations | 4 | 6. No stop at low `df`, but btrfs metadata ran out at 98 GiB free | Not comparable (proposal 8) |
| 8. pueued restarts itself; the watch alerts | adopted; kit | `blocked_h` | 7.3 | pueued died at 04:17 on 10-18 and was restarted by hand, outside its cgroup. `alerts.md` was written by hand | No: not in the kit |
| 9. A term's meaning goes to the designer | adopted | design-gap escalations a step | 0.07 | 0.12; 32 questions; 13 answers differed from the supervisor's reading | Taken. The run asked more because it had more to ask |
| 10. A plan's premises checked by a command | adopted | plan-defect escalations a step | 0.22 | 0.47. The plan was landed with compiled sketches at the base, which `main` then moved past | No (proposal 1) |
| 11. The watch's 65-minute check alarm | adopted; kit | the longest gap between checks | 6.0 h | 4.2 h; 23 of 47 gaps over 65 minutes | No: the kit part was not in the kit |
| 12. A check ends with the steer it proposes | adopted | | | Every check ends "Steered" and "Not steered", with reasons | Taken |
| 13. An hour with only a live suite | adopted | | | No live suite | Not tested |
| 14. The design and the plan in sessions of their own | adopted in part | planning cost | $45 | The plan landed at 07:05, 20 minutes before the decision. It was written in the design's session, 738807d8, on x1 | Not in force |
| 15. `plan metrics` and section 15.2 change together | adopted; kit | comparability | | `metrics.tsv` came from `main`'s kit, priced per model. Kinds, proof words, restarts and the plan's landing still misread | In part (proposal 9) |
| 16. A run keeps its claim's process | adopted | | | Kept for the practice the supervisor named; not for the tools, which followed skein's checkout until ruling 295 | In part (section 5, item 9) |

**Changes to the process that landed during the run** (skein's `docs/foundation` and `scripts/dev`, 07:48 on 10-16 to 23:18 on 10-18):

| Commits | Landed | What | Did the run take it? |
|---|---|---|---|
| `de0f5ba` | 10:57, 10-16 | hearth in the workspace | Not as process; its `plan` table reached the tools at 13:27 through the checkout |
| `d21d90d` | 13:25 | skein design | No |
| `e6fd9fb` | 17:18 | A run takes its kit at its claim | Not for this run, claimed before. It had no `kit/` until ruling 295 |
| `ce02728` | 17:18:55 | forgejo-17's decisions | Points 4, 5, 6 and 9, and the director (2.7), by the 17:49 steer: a re-check never holds a start, Sonnet sub-agents, a usage notice is no stop, watch until the release |
| `c0268a8` | 21:54 | The coordinator compacts at 200K | No: for runs claimed after it. The thread compacted at 330K to 385K |
| `7779af1`, `71f6c38`, `1b83a43`, `2f317f5` | 12:00 to 12:36, 10-17 | Resume after a reboot; compaction in place, as an experiment; the `supervisor` metric | "Not taken" (`supervision.md`, line 2314), but the tools ran them from 17:52, the checkout at `841f487`. Sessions 30 to 35 compacted, then handed over at the second compaction |
| `b30cc95` | 20:05 | wall-clock's decisions | Points 2 and 6 by steer; 3 and 4 as the supervisor's own. Point 3's cron printed local time and lost a check |
| `e28122c` to `c611973`, 29 commits | 23:48, 10-17, to 01:58, 10-18 | kit-batch's steps: rules restored, `plan landed`, the watch's events, loading before a ring, the coordinator's profile, downstream cleanup, the metrics | No: ruling 295 pinned `kit/` five minutes before; ruling 310 |
| `9c57a6b`; `2a9bcd3`, `5f26abb`, `92633c1`, `23cf08d` | 03:50 to 04:09 | kit-batch's decisions and four kit commits | Points 1 and 6 by the supervisor (rulings 306, 307); point 9 steered at 04:10 and withdrawn (ruling 311). Not the kit |
| `b1d2276` | 07:53 | render's decisions | Points 2, 3, 8, 10 and 12, by ruling 331 and the 08:40 steer. Not 5 (an increment past its Size lands), which would have spared seven Size rulings |
| `441c41a` to `867095d`, 8 commits | 07:57 to 08:02 | render's kit | No |
| `9f50d9a`, `c9bd775`; `d450346`, `49e7eb6` | 16:34 to 19:30 | A done plan leaves STATUS.md; forge-setup's decisions and kit | No: the run keeps its process |
| `e9cc93b`, `b4c6241` | 22:38, 23:04 | testing-strategy.md 8 | No |

**What this says of the loop.** Of the commits that landed during the run:
- the text the supervisor named reached the run within 9 to 46 minutes;
- the kit reached it by accident before the pin, and not at all after;
- render's proposal 10 (no handover by count) is the one process change whose effect shows in the numbers (section 4, item 8).

## 8. Proposals

1. **A plan whose longest chain passes about 20 increments, or whose steps pass about 80, runs as consecutive plans. Each plan is written once the plan before it has landed, against the `main` it left.**
   - **Evidence:** section 5, items 1 and 2.
     - 159 steps, 0.38 h a step: one or two sessions at a time after the first day.
     - 302 rulings, 130 of them correcting the plan. The Shapes drifted with each landing, and 70 ready checks and re-checks cost $161.
     - The plan grew from 1.1 MB to 2.0 MB.
     - render ran 168 steps at 0.08 h a step with a short chain, so length alone is not the cause.
   - **Changes:** development.md, 5.1 and 5.3 (a plan's size and its chain); 2.2 (the planner splits a design's steps across plans).
   - **Moves:**
     - plan-defect escalations a step, 0.47 to under 0.25;
     - rulings a step, 1.9 to under 1;
     - `step_h`, 0.38 to under 0.15.
2. **The rules every implementer thread reads stay under about 300 lines.**
   - **What the rule says:**
     - A steer that refines a rule replaces its text, never adds beside it.
     - A rule for one kind of session goes into those sessions' files.
     - A successor reads its increment's section of the session file and the checkpoint, not the whole file.
   - **Evidence:** section 5, items 4 and 5.
     - The README went from 457 to 1,217 lines, its rules to 886.
     - Threads reach 96K by their 30th request. The prefix and the first 30 requests' context are 56% of implementers' input.
     - Each 10K removed from those requests saves about 233M, 5.4%.
     - 15 steer points were refused for text not yet in the README.
     - forge-setup's proposal 8, adopted, sends every run-long steer there.
   - **Changes:** development.md, 5.2 (the README) and 10.3 (steers); implementer.md, 8.1 (the first reads) and 8.6 (the checkpoint).
   - **Moves:**
     - implementers' median context at their 30th request, 96K to under 60K;
     - `tokens_implementers` a step, 21.1M to under 10M;
     - steer points refused, 18 to 0.
3. **The pulse counts a wait by when it returned, not by what it asked for, and implementer.md names `// @exec: {"yield_time_ms": 300000}` as the one way to wait on a running command on Codex.**
   - **Evidence:** section 5, item 6.
     - 1,951 polls asked for 60 s or 300 s through `write_stdin` and returned at 31.0 s.
     - 4,830 polls found the command still running: 716.6M, 21.4% of implementers' input.
     - `short_waits` (`numbers.sh`) matches the asked value, so a poll that asked for 300 s and returned at 31 s counts as compliant.
   - **Changes:** implementer.md, 8.3; development.md, 14.2; the pulse.
   - **Moves:**
     - the share of implementers' requests that only poll, 28.5% to under 10%;
     - `tokens_implementers` a step, down.
4. **When skein lands a kit fix for a defect the run has met, the supervisor's next hourly check moves `kit/` to it by ruling, once `test-plan` passes there. It keeps the kit only for a reason it writes in the ruling.**
   - **Evidence:** section 5, items 7, 9 and 15. Five defects met by hand had their fixes on `main`, 2 to 6 hours older than the workaround:
     - the ref lock;
     - the coordinator's compaction limit;
     - loading before a ring;
     - pueued's restart;
     - the 65-minute alarm.
   - **The other risk.** Forgejo-17's proposal 3 kept the kit from moving under a run. Before the pin, this run showed the opposite risk: commits it had not taken drove six supervisor handovers.
   - **Changes:** development.md, 3.6 and 12; supervisor.md, 3.
   - **Moves:**
     - infrastructure and process escalations met with a fix on `main`, to 0;
     - the coordinator's compaction context, 330K to 385K to at most 200K;
     - the longest check gap, 4.2 h to under 1.5 h.
5. **A ring is one command: `plan unblocked` and `plan steered` log the file, set the ledger and print the message the coordinator sends. A wait that runs out is followed by the next wait alone, since the watch keeps the heartbeat.**
   - **Evidence:** section 5, item 7.
     - Rings took a median of six to seven requests, and 41% of the coordinator's input.
     - Copying scripts were 29% of its output.
     - 230 heartbeat-only cycles took 460 requests and 99M.
   - **Changes:** development.md, 7.3, 7.4 and 10.3; `plan`.
   - **Moves:**
     - `coordinator_requests` a step, 29.3 to under 15;
     - `tokens_coordinator` a step, 5.96M to under 3M.
6. **No sub-agent of the supervisor, and no designer woken for a question, reads `log.md`, `supervision.md`, `decisions.md` or development.md whole. Its prompt names the sections and line ranges it needs; earlier answers come from `decisions.md`'s design rulings.**
   - **What a ready check reads:**
     - the increment's part of the session file;
     - the symbols it names, on `main`;
     - `git diff` of its Files since the plan's base.
   - **Evidence:** section 5, item 8.
     - The sub-agents re-read 2 MB of `log.md` 53 times, `supervision.md` 54 times, development.md 56 times and the README 71 times.
     - They cost $338 of the supervisor's $604.
     - The designers read `supervision.md` 123 times for earlier answers, and their late questions needed up to 217K of reading.
   - **Changes:** development.md, 2.1, 10.4 and 14.1; supervisor.md, 6 and 8; `plan`'s designer prompt.
   - **Moves:**
     - the supervisor's sub-agent dollars a step, $2.13 to under $1;
     - the designer's dollars a question, $6.5 to under $5.
7. **No Proof lands "unavailable". A step whose layer cannot run blocks as infrastructure. A ruling that ignores a test names the step that restores it, and each hourly check runs the ignored test on `main` until then.**
   - **Evidence:** section 5, item 10.
     - Three temper Proofs said "unavailable".
     - The ignored landing story broke for 27 hours unseen, and another run's director found it.
     - Fixing it took two added steps and 13 rulings.
   - **Related.** forge-setup's proposal 1, adopted, checks the process tier hourly while another run shares `main`. This rule also holds for a run alone.
   - **Changes:** development.md, 5.5, 10.4 and 14.1; implementer.md, 8.4 (the Proof line); supervisor.md, 6.
   - **Moves:**
     - the hours a regression stays on `main` before a check sees it, 27 to under 1.5;
     - regressions found by another run, 3 to 0.
8. **On btrfs, the filesystem's unallocated space is a resource bound beside disk and memory.**
   - **What the kit does:**
     - `plan check`, the coordinator's disk check and the pulse read it;
     - below 10 GiB, the kit runs the balance that ruling 334 settled;
     - pueued is restarted only through `plan resume`, which keeps its cgroup.
   - **Evidence:** section 5, item 11.
     - 1 MiB unallocated with 98 GiB free.
     - Two daemons died and four threads were lost.
     - The process tier lost its cgroup for three hours.
     - A dozen balances were run by hand.
   - **Changes:** development.md, 11.4 and 14.1; `plan check`, `plan resume` and the pulse.
   - **Moves:**
     - infrastructure escalations, 6 to 0;
     - `restarts` by crash, 1 to 0.
9. **`plan metrics` maps escalation kinds and proof words onto section 15.2's sets and names what it could not map. It finds the plan's landing by parsed trailers only, and takes a restart's cause from the body of its `log.md` entry.**
   - **Evidence:** section 3, "Not comparable":
     - four kinds split by a full stop;
     - "gate", "browser", "one" and "conformance" counted as layers;
     - "unknown" restarts whose entries name a cause;
     - `5cfc5a2ba`, whose `Planner-Session:` is no trailer, taken as the landing.
   - **Changes:** development.md, 15.2; `numbers.sh` and `metrics.sh`.
   - **Moves:** unmapped kinds and words, and misread landings, to 0, so that the next comparisons hold.

## 9. Debts left in the code

For jig's `docs/development/tech-debt.md`, and for temper, which has none yet. "Listed" says whether jig's file already has the entry. They are ordered by what they cost the next plans.

1. **skein's pins are frozen.**
   - jig pins skein `0f093bb` and temper `8a51e10b`, under the `2733882` cap (rulings 154, 296). skein's `main` is 89 commits past it.
   - hearth's `skein_io` changes (`089ed83`: `Locked`, `Busy`, `Lock`, `ListPage`) gave 15 errors in jig and 19 in temper at the downstream check of 23:22 on 10-17.
   - The browser's back and forward wait on skein too (ruling 393), as does a click on the nearest control (`skein-ui-web/src/page/web.rs:76`). Until then jig and temper name their controls and hold no node in them (ruling 374).
   - Needs a jig repin and a temper repin, first in the next plans. Not listed.
2. **jig's focused budget is at 30 s.**
   - `.config/nextest.toml:10` (ruling 275); its header and `workflow.md` still say 15 s.
   - The median is 14.283 s against the base's 10.33 s. 15 s comes back once a gate-lane median is at most 12 s, so at least 2.3 s (16%) must go.
   - The slowest focused tests under load are protocol-world stories at 0.88 s, 0.31 s alone.
   - Listed (section 4), without these figures.
3. **The test-time aims missed** (ruling 402), with their slowest tests:
   - **jig fuzzy:**
     - `random_people_over_bytes_keep_the_referee_s_rules`: 13.7 s under load, 4.968 s alone, within 1% of its 5 s period;
     - `every_shared_story_survives_every_commit_and_seeded_random_cuts`: 9.7 s, 3.3 s alone;
     - `p_story_survives_seeded_commit_crashes_and_random_presses`: 8.4 s, 4.08 s alone.
   - **temper focused:** `an_adopted_repository_gets_its_change_pushed_through_git`, 3.171 s, 0.947 s alone.
   - **temper fuzzy:** `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, 24.69 s, which is the whole suite's median.
     - It formats both worlds with `Debug` at up to 600 iterations (`tests/engine/domain/src/walking.rs`, `run_replayed`; ruling 122).
     - A fix needs `PartialEq` on jig's `Core` and temper's `engine::Domain`.
   - Not listed, except the conformance cuts in general.
4. **Worst-case literals refreshed by hand** (rulings 110, 112, 354, 356, 363, 398).
   - **jig.**
     - `crates/jig-protocol-config/src/tests/domain.rs:195` pins the small profile's memory sum: 11,900,377,351 at the base, 18,576,825,183 now (+56%).
     - `ENGINE_SMALL` went from 16 to 32 GiB.
     - `testing/jig-test-service/src/tests.rs:22` pins store operations.
   - **temper.** `docs/examples/owned.json`'s `engine.memory_bytes` is exact, so each worst-case growth refused the owned startup and needed a commit of its own.
   - **Needs:**
     - a write mode, like the drift tests' (ruling 131);
     - a ruling on whether "small" may need 18.6 GB.
   - Not listed.
5. **SLOW flags are noise.** The "Done when" runs flagged 20 to 22 focused and 13 to 14 fuzzy jig tests SLOW with the machine alone. jig-22.s1's 26 flags are in `logs/jig-22-s1/slow-workflow.md`. Needs periods set from times alone, or the flag taken from a serial run. Partly listed.
6. **Bounds the client meets as silent cuts.** All are later.md rows, and need the design first (contract.md, 5.3 and 6). Not listed.
   - `documents/small.rs:133` fills `OfferedCharter.goals` to `documents.lines`, while the wire allows 4, so the frame is refused `Bound` (ruling 227).
   - `acts/bounds.rs:47` cuts `me`'s creating offer, or refuses `Limit { Authority }`.
   - `reads.rs:407` and `502` answer `Missing::TooLarge` as `Why::Busy`, which the client reads as "retry later".
7. **A widening's funder.** `jig-domain-tasks/src/control.rs` (`can_resize`, `resize`) resizes the task's present funder, while `acts/bounds.rs:358` counts the amender's pool. jig-03.5 passes `funder: None` on direct widenings (ruling 190). Needs a jig plan on `domain/authority.md` 7 and `tasks.md` 7. Not listed.
8. **Board moves while a goal is closing.** `page/project/moving.rs` renumbers every line in one compound. The goals query includes closing goals, so one closing goal refuses the whole compound. Not listed.
9. **Point loads before durable commits.** `Now::HistoricalProposal`, `HistoricalEscalation`, `HistoricalPersonTask` and `PartyInputs` (`jig-domain-core/src/routing/mod.rs:382`, `556`) go out as `Request::Now`, against engine.md 4.2. Not listed.
10. **A placement race.** `routing/run_preparation.rs:185` picks `fleet.free_kind` for two claims prepared in one commit, so one waits while a worker is free (ruling 129). Not listed.
11. **Decided proposals are never forgotten.** `forget.rs` has `Owner::of_row` and `of_result` only (rulings 175, 177), so decided proposals grow the store. Not listed.
12. **temper's brief spells a SHA-1 commit as 64 digits.** `temper-engine-domain-forge/src/brief.rs` (`append_hex`) disagrees with `route/forge_route.rs`'s `commit_text`, which trims the padding (ruling 399). Fix before an agent resolves or gates on a SHA-1 repository.
13. **A second kind of saved record.** The stream-name record (key `b"stream"`) is special-cased in `jig-client-protocol-link/src/store.rs:176`, and skipped by three checkers:
    - `tests/client-protocol/src/client.rs:130` (ruling 375);
    - temper's `tests/one-process/src/referee/client/cache.rs:143` (ruling 397);
    - `jig-client-test-shell-web/src/records.rs:30`.
    A third kind will need the same skips. Not listed.
14. **The listener's stopgaps.**
    - `loopback: true` is hard-coded (`temper-service/src/clients.rs:640`, `jig-test-service/src/clients.rs:703`), and the cookie has no `Secure`.
    - `jig-test-service` builds its components at `Time::ZERO` (`clients.rs:137`, `144`); ruling 325 fixed only temper's copy.
    - The server's 8 connections (ruling 362) are in no design text.
    - Not listed.
15. **Files past 800 lines.** jig has 66, from 65 at the base; temper 17, from 15.
    - Crossed during the run: `jig-domain-tasks/src/control.rs` (644 to 883), `testing/jig-test-connector/src/boundary.rs` (747 to 892), temper's `temper-service/src/tests/fixtures.rs` (753 to 840) and `tests/system/src/composed.rs` (795 to 819).
    - Needs the consolidation pass (code-organization.md, 7).
    - Partly listed (jig's section 7).
16. **Two vocabularies for jig's executors and outcomes.**
    - temper keeps its own phrases until `jig-client::words` says them (temper-07.s1).
    - temper's `Cargo.lock` still reaches `jig-client-view` through `jig-client-world`.
    - `temper-client-view/src/cards.rs:165`, `238` and `cards/proposal.rs:88`, `167` take parameters nothing reads.
    - 14 terminal goldens were recorded by hand (ruling 396), with no write mode.
    - Not listed.
17. **Terminal text cut at 128 bytes.** `temper-client-shell-terminal/src/limits.rs:55`: the view's `compose` overflows skein-ui's `u32` `text_bytes` at 256 (rulings 160, 163). Needs jig's view.md 10 and skein's ui.md 15 before temper's pages grow. Partly listed (jig's section 10).
18. **Wire fields nothing fills.** `translate/refusals.rs:85` sends `executor`, `needed`, `asked`, `latest`, `reason`, `party` and `how` as `None`. `jig-protocol-transcript/src/lib.rs:159` builds `Call.objects` and `Call.parts` empty. Listed for `refusals.rs` (section 3).
19. **Lints and words.** jig's `manual_map` allowances went from 191 to 233, 52 of them in `examples/ops`. The forbidden word `actual` is still in 839 places, mostly `tests/client/src/joint`. Comments cite rulings in `client-v8-part1.schema:49`, `56` and temper's `pages/inbox.rs:1`. Partly listed (jig's section 7).
20. **Not code.**
    - 172 local `client-api/*` branches remain (jig 122, temper 43, skein 7).
    - The drafts `temper-07-4-renderers-draft` and `temper-07-4-removal-draft` must never land (ruling 394).
    - Six dirty worktrees sit in the run's `preserved-worktrees/`.
    - The process tier needs pueued in its cgroup, which a reboot loses.

## 10. Decisions

2026-10-19. The director, acting for the user with their authority
(development.md, section 2.7), adopted each proposal. A process designer
applies them: the text to development.md, its cards and implementer.md,
and the kit parts of proposals 3, 4, 5, 8 and 9 to `plan`, the pulse,
`numbers.sh` and `metrics.sh`. "Where" names the commits once they land.

Two reach the plans being written now, before the batch lands. The tools
and forge kit planners check their plans against proposal 1's sizes, and
split any that pass them. The tools plan's README gives its implementers
proposal 3's wait (`// @exec: {"yield_time_ms": 300000}`) from its first
step.

| Proposal | Decision | Where |
|---|---|---|
| 1 | adopted | skein 656ef69: development.md 5.1 (a plan's size); 2.2 already had the split; skein 1d105bc: 5.3 (a chain of about 20 increments) |
| 2 | adopted | skein 1d105bc: development.md 5.2 (the rules under about 300 lines) and 10.3 (a steer replaces a rule's text); implementer.md 8.1 and 8.6 |
| 3 | adopted | skein 1d105bc: implementer.md 8.3 (`// @exec`, `write_stdin`'s 31 s); the pulse's count by return, with 7.4 and 14.2: skein's kit, to land |
| 4 | adopted | skein 1d105bc: development.md 3.6, 12 and 14.1; supervisor.md 3 and 6. No kit: the supervisor runs `test-plan` in `kit/` |
| 5 | adopted | skein's kit, to land: `plan unblocked` and `plan steered`, with development.md 7.3, 7.4 and 10.3 |
| 6 | adopted | skein 1d105bc: development.md 2.1 and 14.1; supervisor.md 4, 5, 6 and 8; `plan`'s designer prompt, with 10.4: skein's kit, to land |
| 7 | adopted | skein 1d105bc: development.md 5.5 and 14.1; implementer.md 8.4, 8.7 and 8.8; supervisor.md 6; testing-strategy.md 8 |
| 8 | adopted | skein 1d105bc: development.md 11.4, by hand; `plan check`, `plan resume` and the pulse: the next kit batch (which filesystem, sudo, and a btrfs machine to test on) |
| 9 | adopted | skein's kit, to land: `numbers.sh` and `metrics.sh`, with development.md 15.2 |
