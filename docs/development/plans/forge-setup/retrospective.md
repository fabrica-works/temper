# Retrospective: forge-setup

2026-10-18. The run of temper's `forge-setup` plan on x1, a laptop with
eight cores. It was claimed at 2026-10-16T22:00:19Z. 37 of its 44 steps
landed by 02:29:34 on 10-17. The run then waited 30 hours for client-api,
a plan run on carbon-forge on the same temper `main`. The last planned step
landed at 15:57:36 on 10-18, and the coordinator wrote `done.md` at
16:00:19. The supervisor's sub-agent ran the "Done when" commands from
16:01 to 16:28, before the release, and seven items failed. Ruling 156
added a step, which landed at 16:56:47; a rerun passed at 17:42:27, and the
coordinator released the run at 17:44:55. The plan gave temper its owned
Forgejo, set up by `temper forge prepare|run|reconcile`; `temper box`;
webhooks over loopback; and skein's SHA-256 and HMAC. This retrospective
was written by a fresh session that took no part in the run (development.md,
section 15).

Fourteen read-only sub-agents, on Sonnet, one per source, read:
- the plan, its session files, `later.md`, `findings/`,
  `code-organization.md` and the plan directory's history;
- `decisions.md`: the planner's decisions and rulings 101 to 156;
- `log.md`, in three parts, with the ledger, `context.md`, `sessions/`,
  `done.md` and `reports/`;
- `supervision.md`, `alerts.md`, `handed-over`, `claude.tsv`, and the
  director's records of this run;
- the git history of temper and skein, with their trailers;
- `logs/heavy.tsv`, the 968 task logs, the "Done when" outputs and pueue's
  record;
- the coordinator's Codex rollout;
- the 38 implementer rollouts, in two groups;
- the six supervisor transcripts with their 35 sub-agents;
- the three planner sessions with their 89 sub-agents, the design session
  with its 5, and the designer's ten sessions;
- the earlier retrospectives, the metrics history, and skein's process from
  `c0268a8` to `main`.

This session read:
- development.md, sections 2.2, 4.3, 5.1, 7.4, 7.6 and 15, at `c0268a8`
  and at `main`;
- the plan's README; `done.md`, `status.md`, `claude.tsv` and `threads`;
- the metrics' code: the running time, the roles and the prices
  (`metrics.sh`, `numbers.sh`);
- `plan`'s `docs_worktree` and `ruling` at `main`;
- 14.1's landing report, and two session files' "Questions to raise when
  reached" as the plan landed them;
- the workspace `AGENTS.md`, the director's decisions 39 to 45, and x1's
  boot list;
- the render retrospective's shape, and forgejo-17's sections 8 and 10;
- the files the plan grew, for the debts (section 9).

Every transcript this retrospective counts is on this machine. Not read:
client-api's own run records, on carbon-forge, and the director's
transcript.

Times are UTC. A time without a date is on the day last named. Costs are at
the list prices of each message's model, as `plan metrics` prices them:
Opus 5.5 at $4 input, $8 a 1-hour cache write, $0.20 a cache read and $20
output; Sonnet 5.5 at $2, $4, $0.10 and $10. The readers priced their
sessions at higher rates; every dollar here is the kit's. This document
proposes; the user, or the director, decides, and the designer applies what
is adopted.

**Which process.**
- The run's kit was skein `c0268a8` ("Compact the coordinator at about 200K
  tokens", 21:54:57 on 10-16), from the claim to the release. `kit/` never
  moved.
- 51 commits changed skein's `docs/foundation` or `scripts/dev` during the
  run:
  - the compaction experiment and a resume fix, 12:00 to 12:36 on 10-17;
  - wall-clock's decisions, `b30cc95`, at 20:05;
  - kit-batch's 29 kit steps and its decisions, 23:48 to 04:09 on 10-18;
  - render's decisions and kit, `b1d2276` and nine kit commits, 07:53 to
    08:02;
  - two STATUS.md rules, at 16:34 and 16:39.
- The run took parts of `b30cc95` and `b1d2276`, by the steer of 11:34:25 on
  10-18 and by the supervisor's own practice. Section 7 judges what was in
  force.
- The roles read the process from two places. Implementers w20 to w30 read
  `main`'s card, and those after the 11:34 steer read the kit's. The last
  two supervisors read both.

## 1. In one page

- **44 steps landed, 40 planned and 4 added, none dropped. "Done when"
  failed seven ways before the release, and the run was released green.**
  - Items 1, 2, 4, 5, 6 and 8 pass. Item 7 misses its aim for temper's
    default suite. Item 3's session items failed in seven groups.
  - The gate failed in the "Done when" worktree, whose path is longer. The
    owned example's `memory_bytes` was the least for the path it had been
    measured in. Ruling 156 added `temper-14.s2`, which landed 27 minutes
    after the results; the rerun passed.
  - Five failures were stale checks: items that read the history, count a
    file, or name a test that a later session renamed or moved (ruling 155).
  - The release waited for the run only because the 11:34 steer took
    render's proposal 2. Under the run's kit, it would have come at about
    16:00, with the gate red in a longer path.
- **Two runs on one `main`: 37 steps in 4 h 20 min, then 30 hours waiting,
  then 7 steps in 7.6 hours.**
  - The first night, 22:09 to 02:29, landed 8.5 steps an hour, with up to
    six sessions, the disk's cap.
  - Five increments needed client-api's temper 05.1 and 05.2. The plan knew
    it; its README expected them at about 13Z and 14Z on 10-18. They landed
    at 07:44 and 08:11, and nobody woke the run for 25 minutes.
  - On the second day, client-api's landings twice broke this run's process
    tier on the shared `main`:
    - the landing story, red from at the latest 08:56, possibly from 10:36
      on 10-17, until 13:31;
    - three serve stories, from 13:10 to 14:42.
  - 11.1 was held 4 h 25 min. The process tier runs in no gate, and
    client-api's proofs of it were "unavailable" (proposals 1 and 2).
- **The kit stood still while `main` fixed what the run met.** 51 process
  commits landed during the run, most of them while it was idle (proposal
  3). Among them:
  - loading the coordinator before a ring; the supervisor loaded it by hand
    six times;
  - passing the coordinator its compaction limit; it reached 378K;
  - the pulse's test-time aims; the default suite passed its aim unseen;
  - removing `plan downstream`'s worktrees; four were moved aside at the
    release.
- **Waiting cost the most.**
  - `wall_h` 43.8 and `running_h` 7.9: the Codex threads were active for
    7.9 hours.
  - Three landings waited for an hourly check to see them: 25, 55 and 41
    minutes (proposal 2).
  - The coordinator waited 60 s at a time, against the kit's 300 s. 243
    waits ran out, and the routine cycles after them were 37% of its 1,303
    requests. Implementers polled in 42% of the first night's requests
    (proposal 7).
  - The coordinator spent 5.7M tokens a step, the most on record.
    `tokens_per_step` is 23.4M, against render's 14.4M.
- **The owned example took the last session.**
  - Three design questions, five rulings and two supplements came from one
    file, `docs/examples/owned.json`, which no one started before 14.1.
  - Decision 5's limits were arithmetic only. "One run reports every
    problem" was false. `small` did not fit smith's agent, and four of its
    constants changed.
  - "The least the checks accept" made the numbers depend on the checkout's
    path. That showed at 12:32 in 14.1's first box proof, and in
    client-api's `acc7496e2` at 13:48. It failed the gate at 16:29
    (proposal 5).
- **The plan's numbers ran low again, and its planning was the second
  costliest.**
  - Eight Size blocks. The tolerance went from twice to 2.6 times, then to
    3.5 times (4.5 for a codec), within 2 h 18 min. Codec increments landed
    at 3.8 to 6 times.
  - The planning cost $203 and the design $22, uncounted by `plan metrics`:
    the `Planner-Session:` lines are again not trailers. Only render's
    planning cost more.
  - Planner 2 drafted every session file before asking 45 of the 50 design
    questions: $81 of drafts, then $69 of revisions and fixes (proposal 9).
- **Supervision: $112, six sessions, four handovers by context, 56
  rulings.**
  - $2.55 a landed step, against render's $1.64.
  - It found the `plan ruling` defect within the hour, bisected four of
    client-api's regressions, and ran "Done when" before the release.
- **Twelve proposals** (section 8):
  - the process tier checked on a shared `main` every hour;
  - waits for a landing as watch events;
  - a paused run resumes on `main`'s kit;
  - "Done when" items as properties of `main`;
  - tests that do not depend on the checkout's path;
  - test-time aims on the plan's own packages;
  - Codex roles wait 300 s;
  - steers become README rules, and prompts name them;
  - the design traced before the drafting;
  - `plan ruling` keeps committed edits;
  - the workspace `AGENTS.md` brought up to date;
  - every replacement carries `Reverses:`.

## 2. Outcome

"Done when", item by item. The supervisor's sub-agent, on Sonnet, ran it
once, from 16:01 to 16:28, at temper `58c5321c1` and skein `f7a4e70`,
through `heavy`, in fresh worktrees `wt/donewhen-<repository>`. It reran the
gate's items at 17:39 to 17:42, at temper `61bae3871` (`done.md`).

1. **Every step landed or dropped: passes.** `status.md` shows 44 of 44
   landed. The sub-agent read `status.md` rather than `plan status`.
2. **The plan's commits: passes.**
   - temper's `Step:` trailers list the 37 planned temper steps, and skein's
     list skein-01.1 to 01.3.
   - The four supplements add a line each.
3. **Each session file's "Done when": fails as written, in seven groups;
   holds after 14.s2, with six misses recorded by ruling 155.**
   - **The gate** (temper 01.7, 03.7, 06.5 and 10.1, with 07.1, 14.2 and
     15.4, which run the same test). `scripts/check.sh` exits 100 at
     `serve_prepares_once_on_the_owned_example`
     (`crates/temper/tests/startup.rs:328`) with "workers[0].memory_bytes:
     the worst case passes memory_bytes by 8 bytes". Every landing's gate
     had passed in its own, shorter worktree. After 14.s2 the rerun passed:
     1,833 tests and 50 fuzzy tests, 106 s.
   - **Five stale checks:**
     - temper 01.6 greps the plan's history for `actual`. 02.1 added three
       lines with it, and 02.2 removed them 25 minutes later. No later
       commit can make the item pass.
     - 03.1 counts `"17.0.0"` lines. A third, a test vector of 15.1's,
       sits in `crates/temper-forge-setup/src/tests/upgrade.rs:32`.
     - 04's `git diff $B --stat -- transcripts` expects nothing. Sessions
       01 and 02 added 120 `setup-*` captures there.
     - 08.4b's filter finds 1 test, not 4, because 15.1 renamed three.
     - 10.3d's `grep -c '"ci":true'` prints 0, because 14.1 moved the tier's
       CI rule into the owned example.
   - Not run, because they name reports: temper 02.5, the last paragraph of
     04, and 05.7.
4. **The box, on polling and with webhooks, end to end: passes.**
   `scripts/proof-one-process.sh` ran 19 tests, all passed, in 138.2 s,
   within 180 s. The hinted story alone took 84.4 s.
5. **The conformance's facts: passes.** The seven `fails` lines, and no
   `unchecked`.
6. **temper's skein holds `sha256::Hmac`: passes.** One revision,
   `8a51e10b8`, a descendant of skein-01.2.
7. **The test-time aims: temper's default suite fails; its fuzzy suite and
   skein pass.**
   - temper's default suite: the median of three is 5.991 s for 1,816
     tests, against 3.928 s for 1,306 at the reference `eab91b13d`. That is
     +2.06 s, against an aim of +1.0 s.
   - client-api's tests are inside that figure, and item 7's command cannot
     separate them (ruling 155).
   - temper's fuzzy suite: 21.977 s against 24.745 s.
   - skein: +0.30 s, against +0.5 s.
   - As written, the default run stopped at the gate's failure, so the
     sub-agent measured again with `--no-fail-fast`.
   - The three slowest default tests are not this plan's: they came from
     protocol's tests and from earlier plans.
8. **No budget raised: passes.** `.config/nextest.toml` has no
   `Plan: forge-setup` change, in temper or skein.
9. **These are the only checks.**

The four added steps came from the plan's own code or its "Done when", none
from a fault already on `main`:
- **temper-15.s1** (ruling 129): 07.4 drew Forgejo's `JWT_SECRET` as 64 hex
  digits, which Forgejo decodes as base64. A re-check against the real
  binary found it while 15.1 was in `land` (ruling 128).
- **temper-08.s1** (ruling 133): three races in 08's process-test support,
  which read state before it was written, and failed 1 to 5% of gate runs.
  09.3's gate failed once on one of them. A $0.98 sub-agent diagnosis found
  the class, and the step's stress run passed 100 times in 100.
- **temper-14.s1** (ruling 147): 07.4's `CreatePerson` stored Forgejo's
  whole line ("generated random password is '…'") as the password. 03.2
  had recorded that line. The fixtures used a bare password, so only 14.2's
  first sign-in by web found it, 37 hours after the code landed.
- **temper-14.s2** (ruling 156): the owned example's `memory_bytes`,
  measured with each string a copy changes at its bound.

## 3. Numbers

`plan metrics` ran without error, from the kit at `main`, and wrote
`metrics.tsv`. The comparisons:
- forgejo-17, the last retrospective's run: temper, 11 steps;
- render, the latest run: hearth and skein, 168 steps;
- wall-clock, the other run that waited most of its wall clock;
- protocol, temper's largest.

| Metric | forge-setup | Notes | forgejo-17 | render | wall-clock | protocol |
|---|---|---|---|---|---|---|
| `wall_h`, `running_h` | 43.8, 7.9 | From the claim to `done.md`'s last change. `running_h` is the active span of the coordinator's rollout; every Codex thread together is active the same 7.9 h. | 2.4, 2.0 | 13.8, 13.4 | 29.1, 5.6 | 18.5, 16.9 |
| `steps_planned`, `added`, `dropped` | 40, 4, 0 | 37 temper and 3 skein increments; 15.s1, 08.s1, 14.s1, 14.s2. | 9, 2, 0 | 151, 17, 0 | 20, 0, 1 | 112, 3, 22 |
| `step_h` | 0.18 | The first night: 37 steps in 4 h 20 min. The second day: 7 in 7.6 h, held by client-api's regressions and the owned example. | 0.18 | 0.08 | 0.28 | 0.15 |
| `escalations` | 29: plan defect 17, cross-session scope 5, process 4, design gap 3 | 0.66 a step. Eight were Size stops. 56 rulings in all. | 3 | 48 | 12 | 71 |
| `escalation_wait_min` | 1.7 | The first night's first 13 blocks took a median of 112 s, the next 8 of 72 s. 11.1's two blocks waited 2.5 h and 1.6 h for client-api's fixes. | 10.5 | 2.7 | 2.3 | (−153) |
| `blocked_h` | 3.2 | Most of it is 11.1, from 11:20 to 15:38 on 10-18. | 0.6 | 2.6 | 1.0 | (−277.7) |
| `answers_reversed` | 2 | 104 over 103, and 135 over 134. At least nine more replacements carry no `Reverses:` line (proposal 12). | 0 | 2 | 0 | 5 |
| `steers` | 13 sent, 1 point refused | The refused point told the coordinator to run a Proof (ruling 137). One steer was the user's pause. | 1, 0 | 27, 1 | 10, 1 | 5, 0 |
| `restarts` | loaded 6, reboot 1, the user's request 1 | Six hand loads of an unloaded coordinator thread; x1's one reboot, at 11:51 on 10-17; the resume after the user's pause. | | | | the 1 |
| `tokens_coordinator` | 249.7M in (99% cached), 0.37M out | 5.7M a step, the most on record. | 16.6M; 0.04M | 401.3M; 0.70M | 82.6M; 0.14M | 271.0M; 0.62M |
| `tokens_implementers` | 320.8M in (97%), 0.95M out | 7.3M a step. | 51.1M; 0.15M | 1,122.6M; 4.40M | 143.4M; 0.45M | 870.4M; 3.41M |
| `tokens_claude` | 667.6M in (97% cached), 4.37M out; 0 before the claim, 456.9M in the run, 210.7M after | "0 before the claim" misses 912.9M of planning and design input on this machine. "After" is this retrospective's, at the time of the metrics. | 439.4M | 880.9M | 511.6M | 520.1M |
| `cost_claude` | $197: supervisor $112, designer $32, retrospective $52 | With the planners ($203) and the design session ($22), about $421. | $174 | $312 | $233 | $192 |
| `supervisor` | 6 sessions, 43.6 h, 0.4M weighted tokens an hour, 17% in the 40 requests after each start or compaction, 0 compactions | Four handovers by context, at 288K to 300K, and the reboot's resume. The idle hours dilute the hourly rate. | | 15, 13.4 h, 2.7M, 63%, 28 | | |
| `tokens_per_step` | 23.4M | Coordinator 5.7M, implementers 7.3M, the run's Claude sessions 10.4M. | 19.1M | 14.4M | 23.6M | 13.3M |
| `coordinator_requests` | 1,303, 243 after a wait ran out | 29.6 a step, against render's 11.7. All 331 waits used 60 s. | 162, 9 | 1,961, 32 | 461, 26 | 1,404, 60 |
| `compactions` | 2 | At 357K (00:52 on 10-17) and 378K (12:00 on 10-18) of a 475K window. The kit's 200K limit never reached the thread. | 0 | 4 | 1 | 4 |
| `gates_per_step` | 1.5 | 66 land gates, 59 passed. Four were skein kit landings from `tmp/design/`, through this run's queue. The run's own 62 make 1.41. | 1.4 | 1.3 | 1.2 | 2.3 |
| `gate_lane_other` | 148 | The process tier 80, recordings 27, "Done when" 18, the guide script 11. 40 gate-lane runs bisected client-api's regressions. | 14 | 21 | 30 | 0 |
| `queue_wait_ratio` | 1.34 | The build lane waited 11.4 h for 4.7 h of running, 79% of it while a gate held the lane. | 0.29 | 0.74 | 0.97 | 0.31 |
| `suites` | temper: 1,306 tests / 3.660 s and 46 / 23.138 s first; 1,816 / 6.398 s and 50 / 22.230 s last. skein: 2,469 / 5.649 s first; 2,476 / 5.543 s last | temper's last figure includes client-api's tests. | | | | |
| `proofs` | unit 18, world 17, joint 3, process 25; 0 below plan | 44 Proof lines, each first word a layer's; 19 with a second layer. | process 2 | unit 52, world 61, joint 11, process 71, live 10 | world 3, process 5, live 7 | unit 67, world 28, joint 8, process 11, live 1, none 77 |
| `report_lines` | 16 median, 24 p90 | The longest on record: each report lists its Size by file (ruling 102). 81 reports for 44 steps. | 12, 15 | 12, 16 | 11, 18 | 13, 16 |
| `lines` | code +11,733 −1,002; tests +12,443 −1,020; data +10,712 −40 | Data is mostly the conformance's recordings. | code +1,266 −332; tests +1,654 −339 | code +45,176 −3,776; tests +72,259 −3,985 | code +106,583 −67,728; tests +4,547 −1,681 | code +85,628 −23,242; tests +40,123 −6,166 |
| `contexts` | 152K median, 208K p90; 19 handovers | All 13 first-night handovers were past the kit's 150K (median 175K). Six came on 10-18, after the 100K steer; five were still past 100K. | 113K, 182K; 2 | 172K, 223K; 66 | 101K, 199K; 7 | 225K, 262K; 45 |
| `pin_lag` | smith: skein 69; jig: skein 91, smith 196; hearth: skein 42; temper: skein 90, smith 196, jig 20 | temper took skein once, in 05.1, as planned. | | | | |

**Not comparable, and why:**
- **`running_h`, `step_h`.**
  - development.md 15.2 defines `running_h` by the heartbeat. `metrics.sh`
    measures the coordinator's rollout, less its gaps past ten minutes.
  - This coordinator ended its turn with `plan idle` while it waited, so
    the figure leaves out more than the 30-hour idle. It also leaves out
    the network loss (09:33 to 11:16 on 10-18), the wait for client-api's
    fix (14:04 to 15:39), and the "Done when" run (16:01 to 16:28). No
    Codex thread was active in any of them.
  - The time some role worked, or waited on an answer, is about 13 hours.
- **`tokens_claude`, `cost_claude`.** No planning or design is counted.
  - The plan's landing, `05fe185c3`, writes its three `Planner-Session:`
    lines in a paragraph apart from its trailers. It is the sixth plan
    landing with that blank line. `114db4d` made `land` refuse such lines
    at 07:57 on 10-18, after this plan had landed.
  - The README's Inputs name `setup.md`, not `docs/design/setup.md`, so the
    metrics find no design session.
  - Added to a scratch copy of `claude.tsv`, the four sessions come to
    912.9M input before the claim: planners $202.85 and the design session
    $22.48.
  - forgejo-17 and protocol priced Sonnet at Opus's rates.
- **`gates_per_step`, `gate_lane_other`.** Skein's own kit landings of 10-17
  and 10-18, from `tmp/design/`, went through this run's queue. The bisects of another
  run's regressions count among this run's gate-lane tasks.
- **`supervisor`.** Only render and kit-batch have it. This kit hands over
  by context, and 30 idle hours sit in its 43.6.
- **`answers_reversed`** counts `Reverses:` lines. The rulings' text shows at
  least eleven replacements.

**What explains the largest differences:**
- **Hours per step.**
  - The first night ran at 0.12 hours a step, near render's 0.08. Six
    sessions ran at once, under the six-session cap that a disk below 60 GB
    sets (development.md 11.4); disk stayed at 39 to 52 GB.
  - A session started 8 to 22 s after its need landed. Blocks were answered
    in a median of 112 s.
  - The second day had at most one implementer at a time from 12:00, and
    one running 28% of the time. Most of the rest was spent waiting: for
    the designer on the owned example, for client-api's two fixes, for the
    network, and for an hourly check to see a landing.
- **Tokens per step.**
  - The coordinator's 5.7M a step is the 60 s wait. 218 routine cycles of
    timeout, `touch heartbeat`, `kache gc`, `df` and wait again took 482
    requests and 94M input.
  - Its contexts were large: 63.5% of its requests were past 150K, and
    they carried 80.9% of its input, because the compaction limit never
    reached the thread.
  - The run's Claude sessions are 10.4M a step. The supervisor watched for
    43.6 hours, and four handovers each started a fresh context.
  - The implementers' 7.3M a step is near render's 6.7M and protocol's
    7.6M. Waiting on a running command was 45% of the first night's
    implementer input and 30% of the second day's.
- **Queue waits.**
  - The build lane waited 2.4 times its running time. The gate's
    exclusivity caused most of it: 9.0 of the 11.4 hours of waiting fell
    while a gate held the lane.
  - The two worst windows came from three land gates back to back (22:16 to
    22:24 and 23:48 to 23:59 on 10-16), each draining the lane for up to 327
    s before running alone.
  - The gate's own build had nothing left to redo: its clippy took 3.5 s on
    average and its test build 0.2 s. The build lane's prebuild works.
- **Report lines.** Ruling 102 asked each report for its count by file. A
  report of 01.3 lists 40 files.

## 4. What worked

1. **The first night.**
   - Five sessions ran 90 s after the claim. The first step landed at
     22:09:19, nine minutes in.
   - 37 steps landed in 4 h 20 min.
   - A session started 8 to 22 s after the landing it needed (forgejo-17's
     proposal 4). A handover's successor started 8 to 22 s after the
     handover.
   - The coordinator recorded a landing a median 15 s after its report.
2. **Fast answers.**
   - The first night's 21 blocks took a median of 112 s, then 72 s, from the
     block to the ruling.
   - The designer answered four questions in 2.5 to 6.3 minutes, each with a
     design commit.
   - Interim rulings let sessions go on while the designer worked (103,
     145).
3. **The recordings caught the plan's wrong guesses about Forgejo.**
   - Three facts the planner wrote before recording were wrong, and the real
     Forgejo said so: a nameless team edit answers 200, not 422 (ruling
     119); an edit replaces units with an explicit `none` (127); a narrowing
     hook PATCH keeps the `workflow_job_*` events (131).
   - The fixture's HTTP 500 and the recordings' byte-stability each became a
     README rule within two minutes of their second sighting (rulings 114
     and 124).
   - A re-check against the real binary found the JWT encoding (15.s1).
4. **"Done when" before the release.**
   - The 11:34 steer took render's proposal 2, and the coordinator held the
     worktrees and the release for the supervisor's "done when ran".
   - The gate's failure had a design answer 15 minutes after the question,
     and a landed step 8 minutes after that. The run was released green.
5. **Implementers and the coordinator stopped where they should.**
   - Every Size past its tolerance stopped before landing.
   - Twice the coordinator refused to start unnamed follow-up work after a
     landing, and asked for a named step (01:16 and 02:16 on 10-17).
   - It refused the one steer point outside its role, to run a Proof, and
     ruling 137 agreed in 37 s.
   - w36 found two red stories beyond ruling 152's list, and stopped rather
     than touch the person or the engine. Ruling 153 confirmed a new
     regression.
6. **The supervisor's checks found real faults cheaply.**
   - The first hourly sub-agent found, 50 minutes after the first loss,
     that `plan ruling` had dropped eight rulings' plan edits. The same
     check recovered them.
   - A $0.98 diagnosis turned a gate flake into step 08.s1.
   - Its bisects localized four of client-api's regressions, and the
     director routed them to client-api, which fixed each within 100
     minutes of hearing.
7. **The plan landed finished, as forgejo-17's proposal 1 asks.**
   - 111 commands run at the base, and 84 sketches compiled under clippy.
   - No step came from a fault already on `main` at the base, and no ruling
     found a plan command malformed.
   - Every Proof started with its layer's word, and none fell below its
     plan.
8. **The idle itself was cheap.**
   - The supervisor's 29 quiet-hour checks cost about $4. A cron at 59
     minutes kept its cache warm.
   - The coordinator made 25 requests in 28 hours.
   - The reboot and the user's pause were recovered within minutes, with
     nothing in flight.

## 5. What did not

1. **Thirty hours waiting on another run, claimed.**
   - **Cause.**
     - 05.3, 11.1, 11.2, 14.1 and 14.2 needed client-api's temper 05.1 and
       05.2, the listeners and `application.clients`. The README expected
       them at about 13Z and 14Z on 10-18 (lines 92 to 99).
     - Decision 1 put the box first, on polling. So 35 of the 40 planned
       increments landed before the wait, and the run stayed claimed through
       it.
     - The kit has no event for another run's landing. The supervisor's
       02:28 note says "no rule for a plan whose last steps wait on another
       run's step".
   - **First visible.** In the README. At the supervisor's second check
     (23:50:55 on 10-16). At 05.3's yield (00:57:10 on 10-17), after a
     `git fetch` found no trailers.
   - **Acted on.**
     - By design, never. client-api's 05.1 and 05.2 landed 5.3 and 5.8 hours
       earlier than the README said, at 07:44:46 and 08:11:17.
     - The 08:36 hourly check rang the coordinator 25 minutes after 05.2.
       05.3 started 47 minutes after it, and landed at 09:18:47.
   - **What the wait cost.**
     - The kit stayed at `c0268a8` while 51 process commits landed (item 7;
       section 7).
     - The coordinator's thread unloaded after about 30 to 45 minutes idle,
       and the supervisor loaded it by hand at six rings. Two cold wakes, at
       230K and 279K of context, read 509K tokens uncached, a quarter of the
       coordinator's uncached input.
     - The supervisor's context grew from 107K to 183K over 28 wakes, so its
       third handover came 3.3 hours into the second day.
     - Session files drifted against `main`. Ruling 138 made about 40
       corrections to 05.3's file, and ruling 139 about 45 to 11's and 14's.
   - Proposals 2 and 3.
2. **Another run's landings broke this run's process tier, unseen.**
   - **Cause.**
     - The process tier runs "where a Proof line or 'Done when' says,
       never in the gate" (README). The pinned Forgejo and runner are on
       x1, and client-api's landings recorded their process proofs as
       "unavailable".
     - During the idle, the quiet checks ran nothing. So a story turned red
       on `main` only when this run's next step ran the tier.
   - **The landing story,
     `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`.**
     - It passed at `d4fe894a9` (02:29 on 10-17) and failed at `64f610ae6`
       (08:56 on 10-18).
     - The supervisor's bisect named three client-api commits: `fe8fcdf6b`
       (temper-03.5, 10:36 on 10-17), `e461dfd8d` (04.3, 05:38 on 10-18) and
       `3f8b3df6f` (05.1, 07:44).
     - client-api's `fe4978f22` fixed it at 13:31. It was red for at least
       4.5 hours, and perhaps 27.
   - **The serve stories** (the signed 204, the unknown 404, the drain).
     client-api's `3262df8f9` broke them at 13:10:53, and `46796e8df` fixed
     them at 14:42:28. The supervisor found the fix at 15:37: its wait had
     watched the bisected commit's files.
   - **First visible:** 09:22, ruling 139, from 11.1's ready check.
   - **Acted on.**
     - The bisect began at 09:19, was cut off by the network loss, was
       redone from 11:18 to 11:54, and went to client-api through the
       director at 11:55.
     - The director asked client-api, at 14:03, to run the whole tier before
       each remaining landing. client-api's gate ran it from 14:59.
   - **Cost.**
     - 11.1 was held from 11:20:58 to 15:45:31, across two blocked threads
       and two fresh restarts.
     - Five rulings: 139, 141, 152, 153 and 154.
     - From 09:00 to 14:05, 83 gate-lane tasks, 48 of them failing.
   - **This run did it too.** 14.1 dropped the tier's `root.brief.gather_ms`
     from `forgejo::configuration` while running only the box stories. The
     12:37 check found it, and 11.1 restored it at 15:45, so it was off
     `main` for 3 hours. Rulings 148 and 149 followed.
   - Proposal 1.
3. **"Done when" failed seven ways, after every step had landed.**
   - **Five stale checks** (section 2, item 3).
     - Cause: each item pins a state that a later session's Files change. It
       reads the history (01.6), counts a file (03.1), diffs a directory
       others write to (04), or names tests and members others rename or
       move (08.4b, 10.3d).
     - The README forgave only growing counts ("read as 'at least'").
     - The plan's reviews had flagged the class (literal-A 9, literal-B 1).
     - Ruling 128 knew of 08.2's split at 01:14 on 10-17, and said "Done
       when's counts unchanged".
     - First visible: when 02.2 (01:41), 15.1 (01:15) and 14.1 (12:36)
       landed, and on reading the plan.
     - Acted on: 16:35 on 10-18, ruling 155. Proposal 4.
   - **The gate in a longer path** (item 4 below). Proposal 5.
   - **The default suite's aim.**
     - Cause: client-api's tests ran in the same suite, and item 7 measures
       the workspace. On the first night, when only this plan landed, its
       382 default tests added about 0.75 s.
     - Single gates swung 1.2 to 1.65 s at the same test count, more than
       the aim.
     - The kit's pulse prints the budgets, not the aims. `171ca57`, which
       names landings past an aim, is not in the kit.
     - First visible: 03:49 on 10-17, with one gate at 5.9 s. At the 11:19
       check on 10-18, +1.85 s, and no steer.
     - Acted on: 16:35, recorded for the user by ruling 155. Proposal 6.
4. **The owned example: three design rounds and two supplements in the last
   session.**
   - **Cause.**
     - Decision 5's limits were checked by arithmetic. No one started the
       example at the base: it did not exist. The design's example could not
       start, whatever its grants (ruling 142).
     - 14.1's Shape said "one run reports every problem: at most three
       reruns". `startup.rs` returns after each of its five stages (ruling
       143), so each run shows one stage, one problem deeper.
     - `small` did not fit smith's agent bounds. Four of its production
       constants changed to fit the example (ruling 146).
     - Ruling 143's "the least each check accepts" made the numbers depend
       on the scratch directory under the checkout. The worker counts its
       agent's program and its configuration path, and the engine its
       token and git base.
   - **First visible.**
     - 09:22, 14.1's ready check: "eight reruns found no passing document".
     - The path, at 12:32: 14.1's first box proof needed 32 bytes more,
       which its report of 12:37:18 gives as "an additional 32 bytes for
       its longer scratch path".
     - The path again at 13:48, in client-api's `acc7496e2`: "32 for the
       box's longer scratch".
   - **Acted on.** 11:42 (ruling 142), 12:03 (143), 12:28 (146), and 16:48
     (156, after the gate failed). Five commits of two runs set or raised the
     numbers between 12:36 and 16:56, three of them for a checkout's path.
   - **Cost.**
     - The fifth question waited 2 h 19 min, across the network loss.
     - 14.1 was open from 09:22 to 12:36, over four threads, and two of them
       landed nothing.
     - 14.s1 and 14.s2.
   - Proposal 5.
5. **The guide script's runs: three, then six, then ten.**
   - **Cause.**
     - 14.2's Proof bounded the runs of a first script against Forgejo's
       web pages, which no one had probed. Each run met a new cause:
       - the version's build suffix;
       - no `_csrf` on `/user/login`;
       - a redirect of `/user/change_password`;
       - a 303 that becomes a 404;
       - the script's own CI branch, which adoption refuses.
     - Run 4 failed because the coordinator's `cargo clean`, at 13:09:10
       when the turn ended, had removed the binary the checkpoint said was
       built.
   - **First visible:** 12:41. **Acted on:** 13:10 (ruling 150) and 13:18
     (151). 14.2 landed at 13:20, after eight runs of 2 to 42 s.
   - The supervisor's 13:17 note: "a first script against a pinned binary's
     pages meets one new cause per run".
6. **Waiting by polling.**
   - **Cause.**
     - All 331 of the coordinator's `wait_agent` calls used `timeout_ms:
       60000`. Section 7.4 says 300000. 243 ran out.
     - On each timeout the coordinator ran its wake-up command and `kache
       gc`. Section 11.4 asks for `kache gc` at every disk check below 80 GB,
       and x1 stayed at 39 to 52 GB.
     - The implementers yielded their long commands at 30 to 60 s and polled
       again. The two threads that began a cell with `// @exec:
       {"yield_time_ms": 300000}` polled in 12% of their requests. The others
       polled in 32% to 57%.
   - **Evidence.**
     - The routine cycles were 482 requests, 37% of the coordinator's.
     - 145 of 382 `kache gc` runs evicted nothing. 423 `Resources` lines are
       9% of `log.md`.
     - Polls were 42% of the first night's implementer requests.
   - **First visible:** 22:03 on 10-16. **Acted on:** never.
   - Proposal 7.
7. **The kit froze what `main` had fixed.**
   - **Cause.** forgejo-17's proposal 3 holds a run to its claim's kit, so
     that no change reaches it mid-step. This run idled 30 hours on
     `c0268a8`.
   - **The fixes it met without having them:**
     - `fea416f`, load the coordinator before ringing: six hand loads, three
       of them after the resume of 08:37.
     - `37c676e`, pass the coordinator's profile to its thread: no
       compaction at 200K. The coordinator compacted at 357K and 378K, and
       63.5% of its requests were past 150K.
     - `171ca57`, the pulse names landings past the test-time aims: the
       11:19 check saw +1.85 s and sent nothing.
     - `a5994d2`, `plan downstream` removes its worktrees: four dirty
       worktrees blocked `plan release` at 17:44:06. The coordinator moved
       them to `preserved-worktrees/`, unasked.
     - `7779af1`, `plan resume` loads the thread: the director resumed the
       run by hand after the reboot, and the resume of 13:08 met "thread not
       found" again.
     - `1dfc098` and `b1d2276` point 12, implementers hand over past 100K:
       taken by the steer, though the kit's `plan contexts` flagged at 150K.
       Five of the six later handovers were past 100K.
     - `92633c1` and `b1d2276` point 2, the release after "Done when": taken
       by the steer.
   - **First visible:** 03:49 on 10-17, the first hand load. **Acted on:**
     in part, by the 11:34 steer on 10-18, 15.5 hours after `b30cc95` and
     3.7 hours after `b1d2276`. The quiet checks carried no process line.
   - Proposal 3.
8. **The Sizes ran low again.**
   - **Cause.** The codec and recorder increments enumerate their arms, and
     their tests, more than the session files counted. 02.1 landed 1,153
     lines against 250, and 02.2 was allowed 900 against 150.
   - **Evidence.**
     - Eight Size blocks: 101, 102, 113, 116, 121, 124, 126 and 130.
     - The tolerance went from twice to 2.6 times at 22:12 (102), and to 3.5
       times (4.5 for a codec) at 00:30 (125).
     - The steer that carried ruling 125 said "from their next increment",
       which stopped 05.2 mid-way (126).
     - Ruling 102's own note: "a tolerance of twice stops about half".
   - **First visible:** 22:10 on 10-16, ten minutes in. **Acted on:** by
     standing rulings. render's proposal 5, adopted at 07:53 on 10-18, lets
     an increment past its Size land within its Files; the run's kit lacked
     it.
9. **`plan ruling` dropped committed plan edits.**
   - **Cause.** The supervisor committed a ruling's plan edits before
     calling `plan ruling`. The tool's docs worktree is switched back to
     `origin/main` whenever it is clean (`docs_worktree`, `git switch -C`).
   - **Evidence.**
     - Rulings 101, 102, 105 to 109 and 112 landed their text without their
       edits.
     - Temper 12 started on an uncorrected file (ruling 113).
     - `d8bb411bc` recovered them at 23:02.
   - **First visible:** 22:11:08. **Acted on:** found at 23:01, and kept
     since as a rule in a memory file. `main`'s `plan` still switches the
     branch.
   - Proposal 10.
10. **The planning drafted before it asked.**
    - **Cause.**
      - Planner 2 launched 15 drafting sub-agents at 17:24, three minutes
        after it started, for $80.68.
      - The drafts raised 45 of the 50 design questions, which went to the
        designer in four rounds from 17:52 to 18:26.
      - 17 revisions ($49.12) and 22 fixes ($20.23) followed.
    - **Evidence.**
      - The planning cost $203: forgejo-17's cost $78, wall-clock's $44 and
        parity's $45. Only render's cost more, at $221 on hetzner alone,
        uncounted too.
      - Two design gaps known at the landing were left to the run, though
        the design could settle them, against forgejo-17's proposal 1:
        - 04.1's cancelled job ("no API route to cancel a run is known"):
          the pinned swagger had one, which the designer found in two
          minutes;
        - 09.1's existing bot ("the design names only the marker").
    - **First visible:** at the planning. **Acted on:** never.
    - Proposal 9.
11. **Rulings and re-checks that lost the race to a landing.**
    - **Evidence.**
      - Ruling 128 reached 15.1 while it was in `land`, and 15.s1 followed.
      - Ruling 134 reached 10.1 five minutes after it landed, and ruling 135
        moved the work to 05.3, which landed 31 hours later.
      - 13.1's re-check reported after 13.1 had landed.
    - **Cause.** The supervisor's 02:28 note: "a re-check should start when
      the needs land, not later". Cost: one step and two blocks.
12. **Smaller faults.**
    - **Test-shape rules at the third sighting.** Ruling 140 made two README
      rules from sightings at 23:03, 02:28 and 11:19. That was 36.5 hours
      after the first, across the idle. development.md 14.4 says the second
      sighting.
    - **A stale premise.** Ruling 148 declined render's proposal 6, a
      test-shape check in the gate, because the workspace `AGENTS.md` says
      "temper has no `scripts/check.sh` yet". temper has had one since
      `796d50279`, on 10-13. Ruling 152 corrected the premise 61 minutes
      later, and added no check (proposal 11).
    - **`rm -rf target`.** It was refused in 25 of the first 28 implementer
      threads, each time with the `plan log` of its report in the same
      command. The card prescribed it; `b30cc95`'s `cargo clean` came with
      the 11:34 steer.
    - **The network loss**, 09:30 to 11:00 on 10-18.
      - No task ended for 1 h 43 min.
      - The supervisor's requests failed from 09:41, and it resumed only at
        the director's message at 11:18, about 18 minutes after the network
        returned.
      - The designer and two sub-agents were cut off and restarted.
    - **Local times.** `uptime -s` prints x1's local time.
      - The coordinator first wrote the reboot an hour late.
      - The director recorded a second reboot "at 12:50:56Z". The boot list
        has one boot, at 11:51:14.
    - **`done.md`.** The coordinator's "Supervisor final verification:
      Pending" and "Rulings for these: not yet" stayed after the supervisor's
      sections answered them.
    - **Disk.** Starts held below 40 GB at 01:13 on 10-17 (2 minutes), at
      08:58 on 10-18 (29 GB), and at 11:36 (36 GB). The director found
      something building in temper's primary checkout, and freed 9.7 GB.

## 6. Decisions taken during the run

56 rulings, 101 to 156. By kind:
- 19 plan defects and 1 plan question;
- 14 ready checks and re-checks against `main`, which client-api kept
  moving;
- 8 design answers;
- 7 process;
- 3 interims, 2 test-shape rules, 1 unblock;
- 1 "Done when" record.

| Decision | Rulings | Reached the design? |
|---|---|---|
| A cancelled job's delivery is recorded through the API's cancel | 103, 104 | Yes, `fbe1ceaed` (forge.md 5, 13) |
| A pull's check at its push's delivery varies; the fake always checks | 111 | Yes, `edcd1954b` (forge.md 9, 13) |
| Reconcile never edits an existing bot | 118 | Yes, `231582c05` (setup.md 1, 5, 8) |
| A prepared forge takes a runner added or removed; the runner's lock | 123 | Yes, `13e35d82e` (setup.md 6.3, 6.4, 13.1); `done.md` omits it |
| The owned example's engine limits, stated whole | 142 | Yes, `a975025ad` (configuration.md 7, 10) |
| The hinted story proves CI's end delivery, with the stall raised; long CI deferred | 144 | Yes, `3a323b3b2` (setup.md 10, 13.4; testing.md 2.6; `later.md`) |
| `small` gives four constants; the example's worker limits | 145, 146 | Yes, `6545cb474` (configuration.md 7, 10) |
| The owned example's `memory_bytes` at the bounds of what a copy changes | 156 | Yes, `13154c9dd` (configuration.md 7, 10) |
| Forgejo's answers: a nameless team edit, explicit `none`, a narrowing hook keeps `workflow_job_*` | 119, 127, 131 | No: the conformance's `facts.txt` holds them |
| `JWT_SECRET` as URL base64 of 32 bytes | 128, 129 | No |
| `person_password` holds the password alone; forms post `_csrf` when a page has one | 147 | No |
| The hook secret's bound wired in 05.3 | 134, 135 | No: the plan's |
| 06.1 lands with 06.2 | 115 | The plan's |
| Size tolerance 2.6 times, then 3.5 (codec 4.5), increments in progress included | 102, 125, 126 | The plan's README. implementer.md 8.6 at `main` now lets an increment land past its Size |
| Byte-stable recordings; the fixture's HTTP 500 retried | 114, 121, 124 | The plan's README |
| No rerun limit; each number the least its check accepts | 143 | The plan's; replaced for `memory_bytes` by 156 |
| The guide script's run bound, 3 to 6 to 10 | 150, 151 | The plan's |
| One behaviour per test with a literal; named sizes and both sides of a bound; a setup written once; a changed support function's users run | 117, 140, 148, 149 | The plan's README; testing-strategy.md 10 at `main` says some |
| The implementer runs a renamed Proof; the coordinator checks with git | 137 | development.md 2.3 and 7.3 already say it |
| Regressions of another run's on `main`: hold, permit named reds, unblock | 139, 141, 152, 153, 154 | No: in no document (proposal 1) |
| The six "Done when" misses, each with the command that shows the property | 155 | The plan's (proposal 4) |

**The reversals.**
- **104 over 103's interim**, 4 minutes later: the designer found an API
  route that cancels a run.
- **135 over 134's `Refused::words()`**, 3 minutes later: 10.1 had landed.
- **More replacements, without a `Reverses:` line:** 123 over 120, 125 over
  102, 126 over the 00:30 steer, 129 over 128's parts, 146 over 145, 149
  over 148, 152 over 139 and 141, 154 over 152's list, and 156 over 143
  (whose unblock says "Reverses: nothing"). Proposal 12.

**For the user.**
- decisions.md lists ten numbered decisions, and one unnumbered, "for the
  user", none with a `Decision:` line. The run built each as written.
  kit-batch's `5f26abb` refuses such a plan at `plan check`, but the run's
  kit lacked it.
- Decision 13, the test-time aims, is not among them. Its miss, item 7, is
  recorded for the user by ruling 155.

## 7. The proposals adopted after the last retrospective

The last retrospective before the claim is forgejo-17's. The director
decided it at 17:19 on 10-16, and the designer applied it in skein
`ce02728` and `e6fd9fb`, both in `c0268a8`. Its kit parts, proposal 1's
refusal and proposal 6's cleanup, went to the next kit batch, which landed
after the claim. smith's parity decisions (`30bc10f`) were in `c0268a8` as
text; their kit parts came with that batch too.

| Proposal | Decision | Meant to move | forgejo-17 | forge-setup | Moved |
|---|---|---|---|---|---|
| 1. A plan lands finished: questions answered, every command run at its base | adopted | steps added from a fault already on `main`; ready-check rulings; design questions in the run | 2; 5; 6 | 0 of 4; 14 (12 against a `main` client-api moved); 8, two of them known at the landing | In part |
| 2. A session file at most about 100 lines an increment, 400 a file | adopted | the planner's cost; implementers' tokens a step; `contexts` p90; plan-defect rulings, not up | $78; 4.6M; 182K; 5 | $203; 7.3M; 208K; 18. Six files at exactly 400 lines; 75 lines an increment | No |
| 3. A run takes its claim's kit | adopted | rows waiting on the kit; `gates_per_step` | 30; 1.4 | the kit stayed `c0268a8`; 1.41 the run's own. It also kept seven fixes away (section 5, item 7) | Yes, at a cost (proposal 3) |
| 4. A re-check never holds a start | adopted | the time from a need's landing to the start | 9.5 and 11 min | 8 to 22 s on the first night. Three re-checks lost the race to a landing instead (section 5, item 11) | Yes |
| 5. The supervisor's sub-agents on Sonnet | adopted | the supervisor's cost; usage stops | $37 (Opus-priced); 1 | all 35 on Sonnet, $60 of the $112; $2.55 a step against $3.4; no stop | Yes |
| 6. `plan downstream` removes its worktrees; the supervisor watches until the release | adopted | process escalations at the release; minutes from `done.md` to the release | 1; 24 | 0, but four dirty worktrees moved aside to release; 105, with "Done when", a design answer and a step between | In part: the kit part was not in the kit |
| 7. A Size tolerance of twice; nothing trimmed to fit | adopted | Size escalations below twice; landings at exactly their Size | 1; 2 | 0, the lowest at 2.09 times; not counted | Yes, but the Sizes were low (section 5, item 8) |
| 8. A Proof starts with its layer's word | adopted | proofs counted | 2 of 11 | 44 of 44 | Yes |
| 9. A usage notice changes nothing | adopted | false handovers | 1 | 0; no notice met | Not tested |

**Changes to the process that landed during the run** (skein, 22:00 on
10-16 to 17:45 on 10-18):

| Commits | Landed | What | Did the run take it? |
|---|---|---|---|
| `7779af1` | 12:00, 10-17 | `plan resume` after a reboot | No: "runs claimed from here". The resume of 13:08 met "thread not found" again |
| `71f6c38`, `1b83a43`, `2f317f5` | 12:26 to 12:36 | Compaction in place, as an experiment; the `supervisor` metric | No: the kit is older. The supervisor handed over by context, four times |
| `b30cc95` | 20:05 | wall-clock's decisions | 2 and 6 (`cargo clean`, `python3`) by the 11:34 steer on 10-18, 15.5 hours later; 3 (the hourly check as one-time cron jobs) by the last supervisor, from 16:38; not 4 (the quiet checks named no process change) |
| `e28122c` to `c611973`, 29 commits | 23:48 to 01:58 | kit-batch's steps: rules restored, `plan landed`, the watch's events, loading before a ring, the coordinator's profile, downstream cleanup, the metrics' fixes | No: the kit stayed `c0268a8` (section 5, item 7) |
| `9c57a6b` | 03:50, 10-18 | kit-batch's decisions | 9, the graph project's name, through the workspace `AGENTS.md`; 1 and 6 named by no check |
| `2a9bcd3`, `5f26abb`, `92633c1`, `23cf08d` | 03:52 to 04:09 | kit: test-plan's budget, the "For the user" refusal, a released run's status, the shell kit counted as code | No. This retrospective's `plan metrics` ran with them |
| `b1d2276` | 07:53 | render's decisions | 2, 3 and 12 by the 11:34 steer, 3.7 hours later; 8 as its premise (steer point 5 withheld the stale graph); 1 and 13 by the supervisor; 6 declined by ruling 148 on a false premise, and still not added after 152; 10 not, for this kit hands over by context |
| `441c41a`, `b9c4b96`, `1dfc098`, `e4a754f`, `114db4d`, `171ca57`, `094a27c`, `0cfe3b5`, `867095d` | 07:57 to 08:02 | render's kit | No: "take this only by a ruling that moves `kit/`" |
| `9f50d9a`, `c9bd775` | 16:34, 16:39 | A done plan's items leave STATUS.md | No: their bodies say forge-setup keeps its process |

**The experiment of compaction in place** (development.md 13, `71f6c38`)
was not in the run's kit, and this run tells nothing of it.

## 8. Proposals

Ordered by expected effect.

1. **While another run lands on the same repository's `main`, each hourly
   check of a run that has a process tier runs it once on `origin/main`,
   whenever `main` has moved since its last run, quiet hours included. A
   story that turned red goes, with its range of commits, to the run that
   landed them.**
   - **Evidence:** section 5, item 2.
     - The landing story was red on `main` for at least 4.5 hours, and
       perhaps 27, before 11.1's ready check saw it. The serve stories were
       red for 1.5 hours.
     - 11.1 was held 4 h 25 min, and 40 gate-lane runs bisected the cause.
     - The 29 quiet checks ran nothing.
     - client-api's proofs of the tier were "unavailable"; its binaries are
       on x1.
     - The tier takes 138 s alone.
     - This run's own 14.1 also removed an override the tier needed, unseen
       for 3 hours.
   - **Changes:** development.md 12 and 14.1; supervisor.md 6; the plan's
     README (its process tier's command).
   - **Moves:**
     - hours from a landing that turns a story red to its owner hearing, at
       least 4.5 to under 1.5;
     - hours a step is held on another run's regression, 4.4 to less.
2. **A wait for a landing, another run's or this run's, is a watch event:
   the waiting role names the `Step:` trailers it waits for, and `plan
   watch` rings it when they reach `origin/main`.**
   - **Evidence:** section 5, items 1 and 2, and section 4's end.
     - 05.2 landed at 08:11:17; the ring came at 08:36:43.
     - `46796e8df` landed at 14:42:28 and was found at 15:37:09; the wait
       watched the bisected commit's files.
     - 14.s2 landed at 16:57:54 and was found at 17:39:05; the kit's watch
       has no event for a landing.
     - 121 minutes in all.
     - The README's estimate of client-api's landings was five hours off.
   - **Changes:** development.md 7.4 and 10.5; supervisor.md 2 and 6; `plan
     idle` and `plan watch`.
   - **Moves:** minutes from an awaited landing to the next action, 25, 55
     and 41 to under 2.
3. **When a run has nothing running and nothing that can start for an hour,
   the supervisor pauses it. `plan resume` then resumes it with skein's
   `main` as its kit and process, recorded as a new `## Process` line.**
   - **Evidence:** section 5, item 7, and section 7.
     - The run idled 30 hours on `c0268a8` while 51 process commits landed.
     - At the resume of 08:37, `main` had fixes for six of the faults the
       run then met: the hand loads, the coordinator's compaction, the aims
       in the pulse, the downstream worktrees, the 100K handover, and the
       release after "Done when".
     - Nothing ran across the pause, so no session would have met a change
       of kit mid-step. That was forgejo-17's reason for proposal 3, which
       stands for a running run.
   - **Changes:** development.md 3.6, 7.6 and 12; supervisor.md 6; `plan
     resume`.
   - **Moves:**
     - fixes on `main` that a run meets without having them, seven to 0;
     - "Loaded" events, 6 to 0.
4. **A "Done when" item reads a property of `origin/main`. It never greps
   the history, counts a whole file, or names a test, a file or a member
   that a later session's Files may rename or move. The plan's literal
   review checks each item against the later sessions' Files.**
   - **Evidence:** section 2, item 3, and section 5, item 3.
     - Five of the seven failures; ruling 155.
     - Ruling 128 left a count it knew to be stale.
     - The planner's reviews named the class, and only growing counts were
       fixed.
     - render's run had nine stale counts.
   - **Changes:** development.md 5.2 ("Done when") and 5.4 (the literal
     review).
   - **Moves:** "Done when" items that fail as stale text, 5 to 0.
5. **A test's expected number never depends on the checkout's path: what it
   measures takes each path it lays at its bound. The supervisor's "Done
   when" run uses worktree paths longer than any session's.**
   - **Evidence:** section 5, item 4.
     - The owned example's minima moved with the scratch directory: 32
       bytes at 14.1's first box proof, 32 in client-api's `acc7496e2`, and
       8 at the "Done when" path.
     - Step 14.s2 followed. The longer path found it by chance.
   - **Changes:** skein's testing-strategy.md (a test's inputs); the plan's
     Shapes, through development.md 5.5; supervisor.md 8 (the "Done when"
     prompt).
   - **Moves:**
     - changes of a shipped number for a checkout's path, 3 to 0;
     - "Done when" failures that come from the checkout, 1 to 0.
6. **When another run lands on the same `main`, a plan's test-time aim
   covers the packages its sessions change, measured as their tests' summed
   time, not as the workspace's wall time.**
   - **Evidence:** section 2, item 7, and section 5, item 3.
     - Item 7 missed by 2.06 s with client-api's 510 tests inside, "not
       separable".
     - This plan's own tests added about 0.75 s while only it landed.
     - One gate swung more than the aim at the same test count.
   - **Changes:** development.md 5.2 (the aims) and 5.4; the pulse's
     `aims:` line.
   - **Moves:** aim verdicts that the run cannot attribute, 1 to 0.
7. **Every Codex role waits 300 s at a time, and the pulse counts the waits
   that ran out in less.**
   - The coordinator's `wait_agent` uses 300000, as 7.4 says.
   - An implementer starts each long command with a 300 s yield.
   - `kache gc`, and its `Resources` line, come at most every ten minutes.
   - **Evidence:** section 5, item 6.
     - 331 waits at 60 s; 482 routine requests, 37% of the coordinator's.
     - The `// @exec` threads polled in 12% of their requests, the others
       in 32% to 57%.
     - 145 empty `kache gc` runs; 423 `Resources` lines.
     - The rule is already written for the coordinator. It was not
       followed, and nothing saw that it was not.
   - **Changes:** development.md 7.4, 11.4 and 14.2; implementer.md 8.3;
     the pulse (`numbers.sh`).
   - **Moves:**
     - the coordinator's requests a landed step, 29.6 to under 15;
     - the share of implementers' requests that only wait, 42% to under
       15%.
8. **A steer point that later sessions must keep lands as a README rule,
   by ruling. A spawn prompt names the README's rules, which every thread
   reads first, takeovers included, instead of carrying the steers' text.**
   - **Evidence:**
     - Spawn prompts grew from 0.86 KB to 14.6 KB (w36), as 7.2's "Pass on"
       texts accumulated. Their median was 2.4 KB on the first night and
       9.3 KB on 10-18. render's grew from 1.1 KB to 10 KB.
     - Of seven test shapes the steers later named, four came from takeover
       threads that never read the README. The README had listed the
       forbidden words since the landing.
   - **Changes:** development.md 7.2 and 10.3; implementer.md (its first
     reads).
   - **Moves:**
     - spawn prompts, p90 about 10 KB to under 4 KB;
     - test shapes steered after the README has their rule, to 0.
9. **The planner traces the design, and has every gap answered, before it
   drafts the session files. A session file's "Questions to raise when
   reached" keeps only questions that depend on what the run records.**
   - **Evidence:** section 5, item 10.
     - $81 of drafts, then $69 of revisions and fixes after 45 questions
       the drafts raised.
     - The planning cost $203, second only to render's.
     - Two design gaps were left to the run, one of them answered by the
       pinned swagger in two minutes.
   - **Changes:** development.md 2.2 and 5.4.
   - **Moves:**
     - the planner's cost, $203 down;
     - the share of it spent revising drafts, 34% down;
     - gaps a session file leaves to the run, 2 to 0.
10. **`plan ruling` refuses a docs worktree whose branch has commits that
    `origin/main` lacks, instead of switching it back to `origin/main`.**
    - **Evidence:** section 5, item 9.
      - Eight rulings landed without their edits, and one session started
        on an uncorrected file.
      - `main`'s `docs_worktree` still switches the branch.
      - The workaround lives in a memory file, not in supervisor.md.
    - **Changes:** `plan` (`docs_worktree`); supervisor.md 3.
    - **Moves:** rulings whose edits are lost, 8 to 0.
11. **The workspace `AGENTS.md` brings its section "Until the reliability
    plan ends" up to date, as that section says it should once the plan's
    retrospective has landed: temper's gate is `scripts/check.sh`.**
    - **Evidence:**
      - Ruling 148 declined render's proposal 6 on "temper has no
        `scripts/check.sh` yet". temper has had one since `796d50279`
        (10-13), and every landing of this run ran it.
      - The reliability retrospective landed on 10-14.
      - The projects table still gives temper's gate as its workflow's
        section 1.
    - **Changes:** skein's `scripts/dev/workspace/AGENTS.md`.
    - **Moves:** rulings resting on a stale line of `AGENTS.md`, 1 to 0.
12. **A ruling or a steer that replaces any part of an earlier one names it
    on its `Reverses:` line.**
    - **Evidence:** section 6. `answers_reversed` counts 2, and the
      rulings' text shows at least eleven.
    - **Changes:** development.md 10.3; supervisor.md 3.
    - **Moves:** `answers_reversed` against the replacements the rulings
      state, 2 of 11 to all.

## 9. Debts left in the code

temper has no `docs/development/tech-debt.md`. These belong in one, with
what was seen, why it matters and what pays it back. Paths are at temper
`main`.

1. **The owned example's numbers are kept by hand.**
   - A landing that raises a worst case raises `engine.memory_bytes` or
     `workers[0].memory_bytes` in `docs/examples/owned.json` by the N of
     T3's refusal (14-example.md, the block of 16:47:23).
   - Two runs edited them five times in four hours, and client-api holds its
     edits until 14.s2 (its ruling 377).
   - A test that derives them would end it. Medium.
2. **`small`'s worst cases are about 71.5 TiB (engine) and 1.8 TiB
   (worker)** at their bounds, so `memory_bytes` bounds nothing a person
   has (`later.md`). The designer's. Medium.
3. **A change's CI is read only at its `stall`, 30 s as shipped, when no
   hint arrives.** A box whose CI takes longer meets Stalled (ruling 144,
   `later.md`). It is due before dogfood. Large.
4. **Three gaps of the example's limits** (`later.md`, `6545cb474`):
   - the worker's check names one of two limits;
   - a start that passes, but whose hello never arrives, when
     `forge.resources_per_task` is below `checkout.repositories`;
   - the example's agent is not sized for a person's work.

   Medium.
5. **Four `small` constants changed to fit the example** (ruling 146,
   `crates/temper-service/src/limits.rs`):
   - `checkout.repositories` 3 to 2;
   - `forge.resources_per_task` 4 to 2;
   - `checkout.message_bytes` 256 to 4096;
   - `forge.name_bytes` 256 to 64.

   Whether they suit a person is open. The designer's. Medium.
6. **The guide's walk lives in the run's directory.** 14.2's Proof runs
   `logs/temper-14.2/follow.sh`. Once the run's directory goes, nothing
   checks the setup guide against the pinned Forgejo again. Medium.
7. **Two tests assert the whole usage text:**
   `sign_in_without_two_arguments_prints_its_usage` and
   `forge_without_its_arguments_prints_its_usage`
   (`crates/temper/src/tests.rs:125`, `:127`). Each failed a gate when a
   sibling session's command grew the usage under a clean rebase. Small.
8. **Files this plan brought near or past the 800-line aim:**

   | File | At the base | Now |
   |---|---|---|
   | `testing/temper-fake-forge-domain/src/api.rs` | 794 | 800 |
   | `crates/temper-protocol-store/src/tests/fixtures.rs` | 797 | 798 |
   | `crates/temper-engine-protocol-forge/src/reads.rs` | 758 | 793 |
   | `testing/temper-fake-forge-protocol/src/response.rs` | 741 | 789 |

   `crates/temper-service/src/tests/fixtures.rs` is at 840, mostly from
   client-api. Small.
9. **A version named in a test:**
   `crates/temper-forge-setup/src/tests/upgrade.rs:32` names `b"17.0.0"`.
   code-organization.md's "Don't" forbids a version outside the pin, and
   ruling 140's literal rule asks for one. The two rules disagree. Small.
10. **Forgejo behaviour held only as facts:** a nameless team edit answers
    200 and keeps the name; a narrowing hook PATCH keeps the
    `workflow_job_*` events; `/user/login` and
    `/user/settings/change_password` hold no `_csrf` (rulings 119, 131,
    147). The design says nothing of them. Small.

skein's, from this run:
- **The browser test on x1.**
  - `chromium_survives_a_page_that_reloads_itself` timed out once on x1
    (22:43 on 10-16), and passed on the retry.
  - With `a_page_of_another_origin_is_opened_by_its_step` filtered, x1's
    skein gate is not `scripts/check.sh` as written.
- **jig and temper do not build against skein's `main`:** the hourly
  downstream builds failed six times on 10-18, on `skein_io`'s new `Locked`
  and `Busy`. Not this plan's change.

The kit's, for skein:
- `plan ruling`'s switch of the docs branch (proposal 10).
- **`heavy.tsv`:**
  - It has no header, no kind, and no time alone. A gate's "ran" includes
    its pause.
  - pueue reuses ids (id 45 is in 234 rows).
  - Three finished tasks have no row: 37, 43 and 44.
- **Local times:** `uptime -s`, pueue's timestamps and file times give x1's
  local time where the documents expect UTC.
- **`plan idle`** ran in a script after the script's `context.md` write had
  failed (11:36:01). `context.md`'s 200-character lines ran words together.
- **`plan status`** prints "last check: Reasoning".
- **The pulse's "binary file matches"** for goldens, carried from earlier
  retrospectives.

## 10. Decisions

2026-10-18. The director, acting for the user with their authority
(development.md, section 2.7), adopted each proposal. The designer
applied the text parts to development.md, its cards, testing-strategy.md
and the workspace `AGENTS.md` in skein 49e7eb6, and the kit parts of
proposals 2, 3, 7 and 10 in skein d450346. Proposal 2's wait is a command
of its own, `plan await`, beside `plan idle`. Proposal 6's kit part, the
pulse's `aims:` verdict summing the test times of the packages an aim
names, goes to the next kit batch; until then the hourly check and "Done
when" sum them from the gate's log.

temper's client-api run, on carbon-forge, keeps its claim's kit, skein
841f487, and the process in it (development.md, section 12). That kit has
no move at a pause's resume, so this batch reaches the run only by its
supervisor's ruling: the practice of proposals 1, 5, 7, 8 and 12, and the
kit of proposals 2, 3, 7 and 10 by moving its `kit/` to d450346 or later
(section 3.6). Proposals 4, 6 and 9 are planning rules, for the plans
after it. The workspace `AGENTS.md` (proposal 11) is read from
carbon-forge's primary checkout, not from `kit/`: it reaches the run's
sessions once that checkout is fast-forwarded.

| Proposal | Decision | Where |
|---|---|---|
| 1 | adopted | skein 49e7eb6: development.md, 5.2 (the README's Process tier), 12, 14.1 and 14.5; supervisor.md, 6 |
| 2 | adopted | skein d450346: `plan await` and the watch's `landed:` event and coordinator turn; skein 49e7eb6: development.md, 3.5, 3.6, 7.4 and 10.5; supervisor.md, 2 and 6 |
| 3 | adopted | skein d450346: `plan resume` of a paused run moves `kit/` to skein's `origin/main` and appends its Process line; skein 49e7eb6: development.md, 3.6, 4.4, 7.6, 12 and 14.1; supervisor.md, 1, 2 and 6 |
| 4 | adopted | skein 49e7eb6: development.md, 5.2 ("Done when") and 5.4 (the literal reading) |
| 5 | adopted | skein 49e7eb6: testing-strategy.md, 10; development.md, 5.6, where the test rules are; supervisor.md, 8 |
| 6 | adopted | skein 49e7eb6: development.md, 5.2 (the aims) and 5.4; the pulse's `aims:` verdict over the named packages, in the next kit batch |
| 7 | adopted | skein d450346: the pulse's short waits; skein 49e7eb6: development.md, 7.4, 11.4 and 14.2; implementer.md, 8.3 |
| 8 | adopted | skein 49e7eb6: development.md, 7.2 and 10.3; supervisor.md, 3; implementer.md, 8.1 |
| 9 | adopted | skein 49e7eb6: development.md, 2.2, 5.2 and 5.4 |
| 10 | adopted | skein d450346: `plan docs` and `plan ruling` refuse a worktree whose branch has commits `origin/main` lacks; skein 49e7eb6: development.md, 3.6 and 10.3; supervisor.md, 3 |
| 11 | adopted | skein 49e7eb6: the workspace `AGENTS.md` gives smith's and temper's gate as `scripts/check.sh`, and its section "Until the reliability plan ends" goes |
| 12 | adopted | skein 49e7eb6: development.md, 10.3; supervisor.md, 3 |
