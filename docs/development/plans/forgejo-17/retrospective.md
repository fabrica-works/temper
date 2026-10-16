# Retrospective: forgejo-17

2026-10-16. The run of temper's `forgejo-17` plan on x1, beside temper's
client-api run on carbon-forge, on the same `main`. It was claimed at
14:01:40Z. The last step landed at 15:58:55, and the coordinator wrote
`done.md` at 16:00:48. The supervisor's "Done when" commands ran from
16:03 to 16:12, and the run was released at 16:25:26. This retrospective
was written by a fresh session that took no part in the run
(development.md, section 15).

Six read-only sub-agents, one per source, read:
- the coordinator's Codex rollout;
- the nine implementer rollouts, in two sets:
  - temper 01, 01-s1 and 02, and skein 00;
  - temper 03, 04 and 05;
- the supervisor's Claude Code transcript and its ten sub-agents;
- the designer's two sessions;
- the planner's two sessions and the twelve sub-agents its
  `Planner-Session:` trailers name.

This session read:
- the plan: README, `decisions.md` (rulings 101 to 109), `later.md`,
  `code-organization.md` section 6, `findings/measurements.md`, and
  each increment's Proof;
- the run's ledger, `log.md`, `supervision.md`, `done.md`, `context.md`,
  `claude.tsv` and `logs/heavy.tsv`;
- temper's and skein's history, with the trailers;
- development.md, sections 5.2, 5.4, 5.5, 7.2, 9.1, 11.4 and 15, and
  supervisor.md, section 5;
- `numbers.sh`'s Proof parsing and `plan metrics --history`;
- the parity retrospective's sections 1, 8 and 10;
- protocol's and client-composition's sections 7, 9 and 10.

Times are UTC on 2026-10-16. Costs are at Opus 5.5's list prices.
This document proposes; the user decides, and the designer applies what
is adopted.

**Which process.** `supervision.md`'s `## Process` line names skein
`d047b45`. No commit touched `docs/foundation` or `scripts/dev` from it
to skein's `main`, so the run kept one process throughout. It holds
parity's decisions as `30bc10f` applied them, the text parts alone
(section 7).

## 1. In one page

- **Done in 2.0 running hours: every step landed, and every "Done when"
  item holds.**
  - 11 steps landed: the 9 planned, and 2 supplements from the
    designer's answer (skein-00.s1, temper-01.s1). None was dropped.
  - temper pins Forgejo 17.0.0 and forgejo-runner 13.2.0. On v17:
    - the conformance's 11 new facts hold, its 40 component facts hold,
      and the base's four `fails` remain;
    - git's facts hold.
  - `Read::Job` answers the failed step and its log's end, read by a
    suffix range. The repair's brief names them.
  - The suites stayed within their aims:
    - default 4.510 s, against 4.564 s at the reference;
    - fuzzy 23.360 s, against 22.919 s;
    - the aims were +0.5 s and +1.5 s.
  - `STORE_VERSION` did not move.
  - +1,266 production lines, and +1,654 lines of tests.
- **The one stop was a panic already on `main`, which the planning
  would have found.**
  - In the process tier, `temper open` panicked at
    `crates/temper-client-shell-terminal/src/terminal.rs:72`,
    `Print::new(view_limits).expect("print bounds")`.
  - That line came with client-api's f6f01026f at 12:03:58, ten minutes
    before the plan's base.
  - development.md 5.4 has the planner run every "Done when" command at
    the base. The planner never ran the process tier, which is item 9.
    It had v16.0.5 on disk and the base built by 12:32.
  - The run made the panic a fix for every `temper open`, as follows:
    - interim ruling 101 came 10 minutes after the block;
    - the designer answered 26 minutes after the question;
    - skein-00.s1 and temper-01.s1 landed, and the full process tier
      passed 6 of 6 at 15:22.
  - It cost a 10-minute block and two steps. Nothing else waited on it.
- **The planning cost more than the run, and its cuts fell on the cheap
  checks.**
  - The planning took 1 h 41 min and cost $78 in Claude Code. The run's
    supervisor and designer cost $55 together.
  - Five sub-agents wrote the session files, at 244K to 500K contexts,
    for $42.
  - The session files hold about 400 lines an increment. jig's, smith's
    and skein's plans hold 55 to 80. Three of the files were too long
    for one Codex read.
  - At 13:01:47 the user wrote: "it feels that a 2 hour design/planning
    session for a forgejo version upgrade is a lot".
  - The planner then dropped two checks:
    - the process tier at the base;
    - the sketches' clippy compile.

    These two would have caught the panic, and most of what the ready
    checks corrected.
- **The ready checks found what the planning left, at a price.**
  - Six Opus sub-agents ran 70 build-lane tasks (1,294 s) for five
    rulings of corrections (102, 105 to 108). That was about $19 of the
    supervisor's $37.
  - Two re-checks held a session after its needs had landed. 02.1 was
    held 9.5 minutes and 04.1 11 minutes, both on the run's chain.
- **Landings collided four times.**
  - 03.1, 03.2, 04.2 and temper-01.s1 each ran their build and gate
    again after another session landed first. This cost about 11.5
    minutes of their sessions.
  - test-budgets' proposal 4, adopted on 10-14, would make `land` hold
    its repository's turn. It still waits on the kit.
- **The end took 24 minutes for a cleanup.**
  - `plan downstream`'s three worktrees blocked `plan release`.
  - The supervisor watched nothing after `done:`. It then hit Opus's
    usage limit, which held it from 16:09 to 16:24.
- **19.1M tokens a step.** Codex's part was 6.2M a step, in line with
  earlier runs. Claude Code's part in the run was 12.9M a step, against
  1.6M to 3.3M in the other runs under 80 steps. A small plan carries
  the supervision's fixed costs over few steps.
- **Nine proposals** (section 8). The first two make the planning
  lighter where it was heavy, and complete where it was cut. The third
  lets the kit's adopted fixes reach runs.

## 2. Outcome

The supervisor ran the README's "Done when" once, from 16:03 to 16:12,
in `~/src/rust/temper` at `origin/main` `294de1303`. Its commands and
outputs are in `done.md`.

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 11 of 11 landed, none dropped. |
| 2 | The pin | Holds: `binary.rs` names 17.0.0 and 13.2.0. Nothing under `crates`, `testing`, `tools` or `tests` names 16.0.5 or 12.13.2. |
| 3 | The conformance's facts are v17's | Holds: the header names `17.0.0` and `v13.2.0`. The four `fails` are the base's, and there are no others. |
| 4 | The 11 facts of `forge.md`, section 13 | Holds: prints 11. The waiting job's fact is in `later.md`. |
| 5 | The component's facts | Holds: no `fails` or `unchecked`, and 40 `holds component-`. The 40th is 04.2's. |
| 6 | git's facts are v17's | Holds: both facts hold on 17.0.0, with git 2.53.0. |
| 7 | The answer and the brief name the failed step | Holds: `api.rs:156` has `step: Option<Step>`, and `Job log tail` is gone. |
| 8 | `STORE_VERSION` did not move | Holds: `JobAnswer` changed in place. |
| 9 | The process tier on v17 | Holds: 6 tests run, 6 passed, in 42 s. It needed temper-01.s1. |
| 10 | The plan's commits | Holds: the 9 planned steps and `temper-01.s1`. skein-00.s1 landed in skein (`267730eb2`). The empty line is the rulings' commits. |
| 11 | The test-time aims | Holds: default −0.054 s, fuzzy +0.441 s, against the reference at `fb381950e`. |
| 12 | No budget raised | Holds: `.config/nextest.toml` is untouched. |
| 13 | Nothing else checked | Holds. |

The reference for item 11 was taken at the parent of `b6a69057c`, the
designer's answers to the planning questions. Those answers carry `Plan:
forgejo-17`, so they are the plan's first commit by the README's
command. They hold no code, so the reference is the base the item
meant.

## 3. Numbers

`metrics.tsv` beside this file, against the runs nearest in size, and
temper's last run and the last run of all. Steps landed are planned plus
added.

| Metric | forgejo-17 | tls-cleanup | test-budgets | client-composition | skein-ui | protocol | parity |
|---|---|---|---|---|---|---|---|
| `wall_h` | 2.4 | 1.3 | 3.7 | 4.3 | 8.1 | 18.5 | 33.9 |
| `running_h` | 2.0 | 1.3 | 3.7 | 4.3 | 7.6 | 16.9 | 13.3 |
| steps landed | 9 + 2 | 11 + 2 | 36 + 3 | 36 + 0 | 70 + 5 | 112 + 3, 22 dropped | 47 + 11 |
| `step_h` | 0.18 | 0.10 | 0.09 | 0.12 | 0.10 | 0.15 | 0.23 |
| `escalations` | 3 | 12 | 24 | 8 | 9 | 71 | 36 |
| `escalation_wait_min` | 10.5 | 1.3 | 1.8 | 1.9 | 4.3 | not valid | 2.5 |
| `blocked_h` | 0.6 | 0.2 | 0.6 | 0.4 | 1.3 | not valid | 7.3 |
| `answers_reversed` | 0 | 2 | 3 | 0 | 0 | 5 | 1 |
| `steers` | 1 | 1 | 4 | 2 | 6 | 5 | 11 |
| `tokens_per_step` | 19.1M | 22.2M | 8.6M | 8.8M | 7.8M | 13.3M | 23.0M |
| Codex a step | 6.2M | 4.5M | 7.0M | 5.8M | 4.4M | 9.9M | 17.4M |
| Claude Code in the run, a step | 12.9M | — | 1.6M | 3.0M | 3.3M | 3.4M | 5.4M |
| `cost_claude` | $174 | — | — | $132 | $149 | $192 | $231 |
| of it, the planner | $78 | — | — | $55 | $38 | — | $45 |
| `gates_per_step` | 1.4 | 2.5 | 3.1 | 1.3 | 1.3 | 2.3 | 1.8 |
| `queue_wait_ratio` | 0.29 | 0.66 | 2.13 | 0.06 | 0.65 | 0.31 | 0.74 |
| `contexts`, p90 | 182K | — | 209K | 241K | 237K | 262K | 293K |
| handovers | 2 | — | 3 | 5 | 2 | 45 | 26 |
| `compactions` | 0 | 0 | 1 | 0 | 1 | 4 | 2 |
| `report_lines`, median | 12 | 10 | 11 | 10 | 10 | 13 | 11 |

What explains the largest differences:
- **`step_h` 0.18, almost twice the other small runs'.**
  - The plan is a chain (decision 11), with at most three sessions at
    once: 01.1, then 02.1, 03.1, 04.1 and 04.2.
  - About 42 minutes of the sessions' time went to waits that were not
    their own work, most of it on that chain:
    - the panic's block (10 minutes);
    - two re-checks after a need landed (9.5 and 11 minutes);
    - four gates run again (about 11.5 minutes).
  - The steps themselves took 4 to 18 minutes each. `land` was about a
    third of each session's time.
- **`tokens_per_step` and Claude Code's part.**
  - In the run, Claude Code read 141.8M tokens:
    - the supervisor's ten sub-agents, about 62M, all on Opus;
    - the supervisor itself, 35M;
    - the designer's two sessions, 44M.
  - The run has a fixed amount of supervision: a claim, two hourly
    checks, six ready checks, a design question and the "Done when"
    run. Over 11 steps, that is four times client-composition's share a
    step.
  - Codex's part was 6.2M a step: the coordinator 1.5M, the implementers
    4.6M. Earlier runs ranged 4.4M to 17.4M.
- **The planner's $78.** It is the highest of the runs that count one,
  for the smallest plan. The session writers cost $42, and the review and its fixer $12
  (section 5, item 3).
- **`escalation_wait_min` 10.5** is the median of three waits:
  - 2 minutes for the Size block;
  - 10.5 minutes for the panic, until the interim ruling;
  - 24 minutes for the release (section 5, item 8).
- **`gates_per_step` 1.4.** 15 land gates ran for 11 landings, and 4 of
  them ran again after `main` moved. None failed.
- **`contexts` p90 182K** is the lowest of any run with handovers.
  Parity's proposal 6 held (section 7).

Not comparable:
- **`proofs`.** The metric prints "process 2, 0 below plan".
  - Read by hand, the 11 landed proofs are 7 process, 1 joint, 2 world,
    and skein's gate for skein-00.s1. None is below plan.
  - The Proof lines start with "the real Forgejo", "the fake service
    over its HTTP bytes", "Joint", "the system world" or "gate".
    `numbers.sh` takes the first word and prints only the six layers'.
  - Parity's proposal 15 ("reads every Proof word") is not in the kit,
    and would not read these either (proposal 8).
- **`wall_h` 2.4.** It runs to `done.md`'s last write, the coordinator's
  "Release cleanup" at 16:25:40. `done.md` was written at 16:00:48, 2.0
  hours after the claim.
- **`tokens_claude`.** "Before the claim" is the planner alone. The
  designer's answers to the six planning questions count in the run:
  that session started 13 s after the claim, for $8.95.
- **`cost_claude`'s retrospective part,** $41, is this session's as far
  as it had gone when `plan metrics` last ran, before the landing. Each
  run's total holds its retrospective: $31 of client-composition's, $54
  of parity's.

`pin_lag`: temper is 3 commits behind skein's `main`, 163 behind smith's
and 28 behind jig's. This plan repins nothing (decisions.md, 12).
client-api moves temper's pins.

## 4. What worked

- **A design gap went to the designer, and the run went on.**
  - The panic blocked at 14:11:10. The supervisor diagnosed it with a
    read-only sub-agent, which reproduced it on `main` and computed
    257,411,441,911 bytes against `Print::new`'s u32.
  - The question went to a fresh designer at 14:19:38. Interim ruling
    101 landed at 14:21:23, letting 01.1 land with 4 of 6 process tests
    at 14:26.
  - The designer's answer (skein `054c06e1`, temper `551b1d2d`) put the
    fault in skein's reservation, not temper's limits. It amended
    client-api's plan files for its run.
  - Its two supplements landed by 15:22, while the line went on.
  - client-api's own proofs ran one layer down (its ruling 111) and
    could not have seen it.
- **Proofs on the real side.**
  - Seven of the 11 proofs ran at the process layer:
    - the recording tools on Forgejo 17 (01.1, 01.2, 02.1, 02.2);
    - the component on the throwaway Forgejo (04.1, 04.2);
    - the full process tier (temper-01.s1).
  - Each ran once. Only the first process tier failed, on the panic.
  - The planning's facts probe ran through Codex, for 82K tokens. It
    settled these before any Shape rested on them:
    - `files=false`;
    - suffix ranges;
    - v17's `bytes 0--1/0`.

    All 11 new facts held at their first recording.
- **Ready checks set early, mostly off the chain.**
  - The coordinator set each check 8 to 12 s after the session it
    needed had started. The checks then took 18 to 33 minutes while
    those ran.
  - 03.1 started 19 s after its ruling. 05.1 was set ready 18 s after
    04.1 landed.
  - The corrections that would have failed a step:
    - a job's end answering its row only under `Jobs`;
    - six `d::Row` matches and `walk.rs` missing from 02.1's Files;
    - `Row::Step` forbidden by a Don't that the Choices required;
    - functions at 96, 92 and 100 of clippy's 100 lines;
    - 04.1's `&step=1` request check.
- **Handovers between increments.** temper 02 handed over at 177K and
  temper 04 at 166K, each after a landing. The successors started 29 s
  and 15 s after the reports. The implementers read 4.6M tokens a step,
  against parity's 13.4M.
- **Disk on a laptop.**
  - Free disk stayed between 34 and 49 GB.
  - Every implementer removed its `target/` after its landing.
  - `kache gc` ran at every check.
  - One start, skein-00.s1, was held 5 minutes at the 40 GB bound, and
    there was no disk event.
  - The session acting for the user freed 13 GB outside the run at
    14:55.
- **Two runs on one `main`.** client-api landed only rulings in temper
  during the run. The one rebase conflict was 03.1's with 02.2, two
  steps of this plan.
- **The coordinator.**
  - It started each session 15 to 42 s after its ring.
  - It ran the four prerequisites in one call before the first spawn.
  - It never compacted, and peaked at 153K.
  - 162 requests in all.
- **No step went below its planned layer, no gate failed, and no answer
  was reversed.**

## 5. What did not

1. **The process tier was never run at the base.**
   - **Cause:** at 12:21 the planner judged that the process tier "can't
     be the proof here", having no CI story.
     - It prescribed the tier anyway, as decision 10 and "Done when"
       item 9.
     - Writer 01 reported at 12:56:47 that the planner should run it at
       the base. The planner went on "as scoped".
     - After the user's 13:01 message, the planner dropped it, with the
       sketches' compile.
     - 5.4 asks for every command the plan prescribes, at the base.
       f6f01026f was in the base, and the run with v16.0.5 was one
       command away.
   - **First visible:** 14:10:01, 01.1's proof. The implementer reported
     34 s after the failure.
   - **Acted on:** 14:21 (ruling 101) and 14:50 (ruling 104). The
     supplements landed at 15:09 and 15:22.
2. **The sketches were not compiled under clippy.**
   - **Cause:** at 13:18:58 the planner left the compile to the ready
     checks, "as 5.4 allows". 5.4 defers only sketches that name code
     an earlier session lands. Sessions 01, 02 and 05 named the base's.
     - Of the writers, one compiled with pedantic warnings only, one
       with plain `rustc`, and three not at all.
     - No transcript of the planning reads `clippy.toml`'s
       `disallowed-macros`.
   - **Cost:** rulings 102, 105 and 106 corrected:
     - `.unwrap()` and `format!`;
     - an `#[expect]` never met;
     - functions at the 100-line limit.

     The implementers still met these in their first clippy runs:
     - functions at 105 and 107 of 100;
     - `matches!`, `Option::map`, `vec!`, `slice::concat` and
       `copy_from_slice`, all disallowed;
     - `arithmetic_side_effects`.

     That was about 28 failed build rounds in all.
   - **First visible:** 14:18, the first ready check.
   - **Acted on:** at each ready check.
3. **The plan was long, and its writing was the planning's cost.**
   - **Cause:** five session writers wrote in parallel from a handover
     note.
     - They grew to 244K to 500K contexts and cost $42.
     - The files hold 3,589 lines for 9 increments, about 400 an
       increment. The largest are 02 (904 lines), 04 (853) and 03
       (796).
     - temper's protocol held 273 lines an increment. client-composition
       held 80, skein-ui 60, parity 57 and test-budgets 55.
     - The length bought no fewer defects. The run still needed five
       rulings of corrections.
   - **Cost:**
     - Codex cut each `cat` at about 10K tokens. w2 and w3 read
       `02-documents.md` again by pieces.
     - w7 read `04-job-read.md` four times. It was at 159K ten minutes
       into 04.1, and documents were 82K of the 176K it ended at.
     - The review and the fixer that followed cost $12 and 33 minutes.
   - **First visible:** 13:01:47, the user's message.
   - **Acted on:** no; the cuts fell on items 1 and 2.
4. **The six planning questions were answered after the claim.**
   - **Cause:** `design-questions.md` was complete at 13:20. The
     designer was started only at 14:01:51, after the plan landed and
     the run was claimed. 5.4 has the designer answer each before the
     run.
   - **Cost:**
     - The answers (`b6a69057c`, 14:17:51) rewrote sessions 02 to 05 by
       344 lines in and 283 out, after the plan's review.
     - They overtook 02.1's first ready check, and the re-check held
       02.1's start 9.5 minutes after 01.1 landed.
     - They changed 02.1's shape, and wrote "Size stays 290".
   - **Acted on:** no.
5. **Size: a missing sentence, and squeezed code.**
   - **Cause:** 5.2 has `code-organization.md` say that an increment
     lands within twice its Size without asking, and 2.5 times for a
     decoder. This plan's section 6 omits it, and the implementers read
     no other source of it.
     - 02.1's 290 was the sum of seven guessed parts, never counted
       against the code.
   - **Cost:**
     - 02.1 asked at 1.21 times its Size (ruling 103).
     - 04.2 deleted the doc comments of `Range::Head` and `Range::Tail`,
       and a blank line, to land at 140 of 140.
     - 02.2 rewrote struct literals with a script to land at 66 of 70.
     - 04.1 landed at 110 of 110.
   - **First visible:** 14:44:24.
   - **Acted on:** 14:46, for 02.1 alone.
6. **Re-checks held starts on the chain.**
   - **Cause:** 7.2 holds a session until its check, and the
     supervisor's re-check after a need lands is part of it.
     - The 04 re-check found one compile error and line numbers in 10
       minutes 49 s (ruling 107).
     - The supervisor's own re-check of temper-01.s1, with `git diff
       --stat` and greps, took 1 minute 13 s.
   - **Cost:** 02.1 started 9.5 minutes after 01.1 landed, and 04.1 11
     minutes after 03.1. That is about a sixth of the run.
   - **Acted on:** no.
7. **Landings collided.**
   - **Cause:** `land`'s push is its lock (9.1). A landing that loses
     the race builds and gates again.
     - 02.2 landed 34 s into 03.1's land, and temper-01.s1 40 s into
       03.2's.
     - 05.1 landed 20 s into 04.2's, and 03.1 before temper-01.s1's.
     - test-budgets' proposal 4 ("`land` holds its repository's turn
       from rebase to push") was adopted on 10-14. It waits on
       `kit-after-parity`.
   - **Cost:** 4 of 15 land gates ran again, about 11.5 minutes of
     sessions' time. `land` took 34% of temper 01's, 02's and 01-s1's
     time, and skein 00's.
   - **Acted on:** no.
8. **The release waited 24 minutes.**
   - **Cause:** the hourly check of 15:36 ran `plan downstream`. Its
     three worktrees (`wt/{smith,jig,temper}-downstream`) were left
     with a `Cargo.lock` moved to skein's `main`. `plan release`
     refuses while worktrees are left.
     - The coordinator blocked at 16:01:31.
     - The supervisor's `done:` watch was one-shot. Its "Done when"
       sub-agent saw `blocked.md` at 16:01:58, but reported it last.
     - At 16:09:33 a refused request stopped the supervisor until the
       limit reset at 16:20. The session acting for the user woke it at
       16:24:27.
   - **Cost:** 0.4 h of `blocked_h`'s 0.6. About 8 minutes were
     unattended by omission, and 15 by the limit.
   - **Acted on:** ruling 109, 20 s after the supervisor read the
     block.
9. **The supervisor's spend, and a stop that was not one.**
   - **Cause:** all ten of the supervisor's sub-agents ran on Opus. No
     `Agent` call passed `model`.
     - The six ready checks took about $19: 272K output, 846K fresh
       input, 45.6M cache reads.
     - The two hourly checks were surveys.
     - At 15:40:36 a usage notice arrived. The supervisor wrote a
       Handover ("this session stops"), and told the user its watch and
       cron would die.
     - Neither happened, and no successor was needed. The real stop
       came at 16:09:33, during the release (item 8).
   - **First visible:** 15:40:36.
   - **Acted on:** no.
10. **Waiting by polling, again.**
    - **Implementers:**
      - Polls took 25% of temper 01's, 02's and 01-s1's input, and skein
        00's.
      - They took 70% of temper 03's: 13 heavy launches at a 1-second
        yield, 65 waits of 10 s and five `sleep 60`. implementer.md,
        8.3, asks for the longest wait.
    - **The coordinator:**
      - Nine waits timed out after 5 minutes with nothing new. Each cost
        3 or 4 requests: heartbeat, `kache gc`, the log line, then the
        gc's output.
      - Together they cost 35 requests and 3.74M tokens, 22% of its
        input.
      - It also read development.md whole, then sections again. That put
        38K tokens into a context that 161 later requests carried.
    - **Acted on:** no.
11. **Smaller things.**
    - Steer 1 let 02.1 change three lines outside its Files with no
      landed ruling (supervisor.md, section 3). The text sits in
      `log.md` alone.
    - `rm -rf target` was refused by Codex's sandbox in four of five
      sessions of the first set. `rm -r` and `shutil.rmtree` worked.
    - skein 00 queried the code graph by the workspace `AGENTS.md`'s
      example name, `srv-data-git-runner-src-skein`, which was not
      there.
    - The designer's `plan join --help` wrote a role named `--help`
      into `claude.tsv`, and removed it by hand. protocol's proposal 12,
      in the kit, has `plan set` refuse a step that starts with `-`;
      `plan join` has the same gap.
    - The designer found that `plan wake supervisor` refuses a busy
      supervisor. It sent its message with SendMessage instead.

## 6. Decisions taken during the run

| Decision | What | In the design? |
|---|---|---|
| The six planning questions (`b6a69057c`, 14:17:51) | a step's long name carried empty; a job's log within `answer_bytes` less its name; one `STORE_VERSION` move shared by concurrent plans; the gather's cut of a CI section; a log read through bounded by `body_bytes_in`; a job's steps as streamed rows | Yes: `forge.md` 3.2 and 3.3, `store/README.md` 4, `domain/engine.md` 9 |
| Ruling 101 | 01.1's process proof without the two `temper open` tests | Plan only; ended by temper-01.s1 |
| Ruling 102 | 02's Shapes against `main` | Plan only (corrections) |
| Ruling 103 | 02.1's Size at most 370 | Plan only; landed at 352 |
| Ruling 104 | A print's room from its own tree: skein-00.s1 and temper-01.s1 | Yes: skein `054c06e1` (`ui-text.md` 3, 4, 8), temper `551b1d2d` (`client.md` 5.4, 6.8, 10); both `STATUS.md` files name it |
| Rulings 105 to 108 | 03's, 04's and 05's Shapes against `main` | Plan only. 105 noted that translate's read by id drops the attempt. 106 and 107 made 04.1 assert the attempt asked, which no world contradicts (section 9) |
| Ruling 109 | Remove `plan downstream`'s worktrees for the release | Process: proposal 6 |
| Steer 1 | A replaced phrase reads as English; three doc comments corrected | Not in any document. A taste rule for the implementer's card; one occurrence, not proposed |
| 04.1, in its commit | The raw-room test at 4,096 bytes, not the shipped 65,536 the Test shape named; the scripted peer could not deliver it | No: a test's choice (section 9) |
| 04.2, in its commit | A body of unknown length keeps `Raw`'s one-byte demand; the Shape's larger demand lost a short final chunk | No (section 9) |
| The designer's "Seen, not decided" | temper's client checks no configured memory at start (programming-model.md 6.3); jig's envelope's worst case is past any machine | No: in no `STATUS.md` row (section 9) |

## 7. The proposals adopted after the last retrospective

The last retrospective was parity's, decided on 10-16 at 07:26 (smith
`f4f7f675`). `30bc10f`, an ancestor of the run's `d047b45`, applied its
text. Its kit parts (5, 6's limit, 7's pulse line, 8, 11, 14's
`AGENTS.md`, and 15) are not on skein's `main`. On origin,
`kit-after-parity` holds one commit, of 10-14. So these were not in
force.

| Proposal | Meant to move | parity | forgejo-17 | Moved |
|---|---|---|---|---|
| 1. No role asks the user through a tool that waits | `blocked_h` | 7.3, 68% of it one question | 0.6; the supervisor asked nothing that waited. The planner asked twice before the run, the user present, answered in 2 minutes | Yes |
| 2. A live suite's pipeline proved early | `steps_added` | 11 | no live suite | Not tested |
| 3. A verdict plan settles its closing runs | design gaps at the end | — | no verdict | Not tested |
| 4. Deferred to smith's wall-clock design | — | — | — | Not here |
| 5. `plan idle` takes task ids; the watch loads the thread (kit) | hand loads; late wakes | 10; about 1 h | not in force. The thread never unloaded, and nine 5-minute waits woke to nothing (section 5, item 10) | Not in force |
| 6. Hand over past 150K between increments | `contexts` p90; implementers' tokens a step | 293K; 13.4M | 182K; 4.6M. Handovers at 177K and 166K, after landings | Yes |
| 7. `target/` after each landing; `kache gc` below 80 GB | infrastructure escalations; `blocked_h` | 4 | 0. One 5-minute start hold at 40 GB | Yes |
| 8. pueued restarts itself (kit) | `blocked_h` | 5 h | not in force; pueued stayed up | Not tested |
| 9. A design gap goes to the designer, with an interim ruling | wait to a gap's first answer; `steps_added` | 87 minutes to ask | asked 8.5 minutes after the block, interim ruling at 10, answered at 35 | Yes |
| 10. A bound's premise checked by a command in `findings/` | plan defects from a bound | 6 rulings | the suites' times measured at the base. No bound's premise failed | Held |
| 11. The watch's `check:` (kit); meanwhile, a late successor checks at once | hours without a check | — | not in force. Checks at 14:41 and 15:36 by cron; no successor | Not tested |
| 12. A check's report ends with the steer it proposes | debts at the end | — | both checks did ("Steers to send: 1", "none") | Yes |
| 13. With one live suite running, the check is `plan pulse` | sub-agent checks | — | no live suite | Not tested |
| 14. The designer and the planner in sessions of their own | comparability; planner cost | $97 of sub-agents | held: three sessions of their own, each named. The planner's writers are its sub-agents, and cost $42 (section 5, item 3) | Held |
| 15. `plan metrics` reads every Proof word (kit) | `proofs` | — | not in force; 9 of 11 proofs uncounted (section 3) | No |
| 16. A run keeps the process of its claim | comparability | — | `d047b45` throughout; no process change landed during the run | Yes |

Earlier adopted kit proposals met this run too:
- test-budgets' 4, `land` holding its repository's turn: four gates ran
  again (section 5, item 7);
- protocol's 12, `plan set` refusing a step that starts with `-`: the
  designer's `plan join --help` met the same gap.

The decision tables of six retrospectives send 30 rows, whole or in
part, to `kit-after-parity`. The oldest are from 10-14 (proposal 3).

## 8. Proposals

Ordered by expected effect. Proposals 1 and 3 have a kit part, and
proposal 6 is the kit's. Proposal 3 would let these land at once; until
then, the text parts stand.

1. **A plan lands finished: its design questions answered, and every
   command it prescribes (each Proof, each "Done when" item, each sketch
   under clippy) run once at its base, each output in `findings/`. When
   planning must be lighter, the writing is cut, never these runs.**
   - **Evidence:**
     - The process tier, "Done when" item 9, would have shown the panic
       at the base. It cost ruling 101, a design question and two
       supplements (section 5, item 1).
     - Uncompiled sketches brought three rulings of corrections, and
       about 28 failed build rounds (item 2).
     - Questions answered after the claim rewrote four session files
       after the review, and held 02.1 for 9.5 minutes (item 4).
     - After the user's 13:01 message, these were what the planner
       cut. The writing cost $42; the runs would have taken minutes.
   - **Changes:** development.md 5.4, and 6 (`plan check` refuses a plan
     whose `findings/` has no base runs, or whose questions are open:
     the kit's).
   - **Moves:**
     - `steps_added` from a failure already on `main` (2) to 0;
     - ready-check rulings (5) and the ready checks' build-lane time
       (1,294 s) down;
     - design questions during the run down.
2. **A session file gives each increment at most about 100 lines, and
   is read in one read (at most about 400 lines). It names the design,
   the code and the findings by section and symbol, and does not
   restate them.**
   - **Evidence:**
     - This plan's session files hold about 400 lines an increment, and
       temper's protocol 273. jig's, smith's and skein's plans hold 55 to
       80.
     - The longer files bought no fewer defects: 38 plan defects in
       protocol, and five rulings of corrections here, against 3 in
       client-composition.
     - Three of the files were too long for one Codex read, and w7 read
       one four times (section 5, item 3).
     - The writers cost $42 at 244K to 500K contexts. The user found
       the planning too heavy.
   - **Changes:** development.md 5.2 (the session file) and 4.5.
   - **Moves:**
     - the planner's cost ($78) and time (1 h 41 min) down;
     - `tokens_implementers` a step (4.6M) and `contexts` p90 (182K)
       down;
     - plan-defect rulings, not up.
3. **A run takes its kit from its claim, as it takes its process.
   `plan start` runs `plan`, `land` and `heavy` from a worktree of skein
   at the claimed commit, so a kit change lands as soon as it is
   decided.**
   - **Evidence:**
     - Thirty decision rows wait on `kit-after-parity`, some since
       10-14. They wait because a run's machine takes skein's `main`
       mid-run whenever it lands there (parity's section 10).
     - This run met three of them:
       - `land`'s turn: 4 gates again, 11.5 minutes;
       - the Proof words: 9 of 11 proofs uncounted;
       - an option guard like protocol's 12, which `plan join` lacks.
     - It also met a new one: `plan downstream`'s worktrees.
   - **Changes:** `plan` (`start`, `resume`, the roles' `PATH`);
     development.md 3.6, 6 and 12.
   - **Moves:**
     - adopted rows waiting on the kit (30) to 0;
     - `gates_per_step` toward 1.0 once `land` holds its turn.
4. **A re-check never holds a start. When a checked session's needs
   land, it starts at once. The supervisor re-checks what `main` moved,
   and sends what it finds as a steer.**
   - **Evidence:**
     - 02.1 started 9.5 minutes after its need landed, and 04.1 11
       minutes after.
     - The 04 re-check found one compile error and line numbers, which
       an implementer's first build finds in seconds.
     - The two holds were a sixth of the run (section 5, item 6).
     - Repins, handovers and added steps already start without a check.
   - **Changes:** development.md 7.2; supervisor.md, section 5, step 3.
   - **Moves:**
     - the time from a need's landing to the start (9.5 and 11 minutes)
       to under 1;
     - `step_h` down on chained plans.
5. **The supervisor's sub-agents run on Sonnet: ready checks, hourly
   checks, diagnoses and the "Done when" run. The supervisor stays on
   Opus.**
   - **Evidence:**
     - All ten sub-agents ran on Opus. The ready checks alone took about
       $19 of the supervisor's $37.
     - Their work is greps, compiles and test runs, against a written
       prompt.
     - Opus's usage limit stopped the supervisor for 15 minutes at the
       release (section 5, items 8 and 9).
   - **Changes:** supervisor.md, sections 5 and 6; development.md 14.1.
   - **Moves:**
     - `cost_claude`'s supervisor part ($37) down;
     - stops on a usage limit (1) to 0;
     - plan-defect escalations after a ready check, not up (one here:
       02.1's Size, which both of its checks missed).

     The next retrospective compares them.
6. **`plan downstream` removes its worktrees when its check ends, and
   after `done:` the supervisor watches until the run is released.**
   - **Evidence:**
     - Ruling 109.
     - The release waited 24 minutes: 8 of them unattended, because the
       `done:` watch was one-shot (section 5, item 8).
   - **Changes:** `plan` (`downstream`, the kit's); supervisor.md,
     section 7; development.md 7.6.
   - **Moves:**
     - process escalations at the release (1) to 0;
     - the time from `done.md` to the release (24 minutes) to a few.
7. **implementer.md states the Size tolerance: twice the Size, 2.5
   times for a codec or document increment. An increment never trims a
   doc comment, a test or the formatting to meet its Size.**
   - **Evidence:**
     - 5.2 asks each plan to copy the tolerance into
       `code-organization.md`. This one did not, and 02.1 asked at 1.21
       times (ruling 103).
     - 04.2 deleted two doc comments to land at 140 of 140, and 02.2
       rewrote struct literals with a script to land at 66 of 70
       (section 5, item 5).
   - **Changes:** implementer.md, 8.6; development.md 5.2.
   - **Moves:**
     - Size escalations below twice (1) to 0;
     - landings at exactly their Size (two) down.
8. **A Proof line starts with its layer's word: unit, world, joint,
   process or live. A second layer follows a semicolon.**
   - **Evidence:**
     - This plan's Proofs start with "the real Forgejo", "the fake
       service", "the system world", "two, each once".
     - The landed lines start with "real", "HTTP", "Joint", "gate".
     - `plan metrics` counted 2 of 11 (section 3).
     - Parity's 15 reads the words, but cannot map these to layers.
   - **Changes:** development.md 5.2 and 5.5; implementer.md, the
     `Proof:` trailer; the consistency review of 5.4 checks it.
   - **Moves:** `proofs` counted from 2 of 11 to 11 of 11.
9. **A usage notice changes nothing. A role stops, and writes its
   Handover, only when a request is refused. Then its last line in
   `supervision.md` gives the reset time, and the user, or the session
   acting for the user, resumes it then.**
   - **Evidence:**
     - At 15:40:36 the supervisor took a notice for a stop. It wrote a
       Handover, and told the user the watch and cron would die. They
       did not.
     - The real refusal came at 16:09:33, with no line saying when it
       would end (section 5, item 9).
   - **Changes:** supervisor.md, section 7; development.md 10.5.
   - **Moves:**
     - false Handovers (1) to 0;
     - the time from a limit's reset to the role's resume (4 minutes
       here) down.

## 9. Debts left in the code

temper keeps no `docs/development/tech-debt.md` yet, as protocol's
retrospective found. These are for one, ordered by what they cost the
next plan.

- **A body of unknown length is read a byte per demand.**
  - In `crates/temper-engine-protocol-forge/src/connection.rs:513-525`,
    `raw_demand` asks `Read::Fill(1)` for a `Raw` or `Tail` body without
    `Content-Length`.
  - 04.2 found that a larger demand lost a short final chunk. Its test
    `a_chunked_tail_read_keeps_the_last_bytes_across_pieces` read
    nothing where it should have read `[99, 100, 101]`.
  - The cause is not found, in temper's connection or in skein's HTTP
    machine. A demand can lose bytes at a body's end.
  - What one-byte demands cost on a long chunked log is not measured.
- **A stale job attempt asserts, rather than answering `MissingJob`.**
  - translate's read by job id drops the attempt (ruling 105).
  - 04.1's `job_answer` asserts that the fake's job is the attempt asked
    (rulings 106, 107). This holds only because no world sets
    `reruns`.
  - Re-runs, `later.md`'s first row, need translate to carry the
    attempt.
- **The client checks no configured memory at start.**
  - This is the designer's "Seen, not decided" (programming-model.md
    6.3). jig's envelope also puts its worst case past any machine.
  - It is in no `STATUS.md` row: a question for the designer first.
- **Two doc comments deleted to fit a Size:** `Range::Head` and
  `Range::Tail`, in `crates/temper-engine-protocol-forge/src/plan.rs:6-9`
  (04.2).
- **04.1's raw-room test runs at 4,096 bytes,** not the shipped 65,536
  its Test shape named, because the scripted peer could not deliver
  them.
- **Over-long tests and files:**
  - test functions over 40 lines:
    - three in 04.1 (52, 43 and 41 lines);
    - 04.2's suffix-range table (48);
    - the system story
      `a_smith_change_with_failed_ci_is_repaired_reviewed_and_landed`
      (120);
  - `tests/engine/domain/tests/forge_routes.rs` (2,875 lines) and
    `tests/engine/forge/client/tests/simulation.rs` (858) are past 800;
  - `testing/temper-fake-forge-domain/src/api.rs` is at 794.
- **The fake forge's Actions answers are not compared with v17's
  recordings** (`later.md`). The fake's jobs, steps and ranges follow
  the facts by hand.

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
