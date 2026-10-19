# tools: decisions

The questions this plan raised and their answers first; then the
decisions this plan rests on, numbered; the supervisor's rulings during
the run follow them, from 101; the choices the user may veto close the
file. The designer answered the design questions (rounds 1 and 2, and
questions 33 and 34); the user answered 18a and 20. The planner took the
rest, the most reversible option where the choice would have been the
user's (development.md, 10.4).

temper's `docs/design/client.md` is `docs/design/client/README.md`
since temper `6f342f624`, its sections numbered as before: below,
temper's "client.md N" is that file's section N; jig's `client.md` is
jig's `docs/design/client/domain/client.md`.

## Design questions

These were raised while planning and answered by the designer (rounds 1 and 2) or by the user (18a, 20).

1. **A given key in skein's request table.** lib.md 12 draws keys from the seed. Open: when a given
   key is admitted, how long it is held, a typed refusal apart from "limit", `restore` against a
   live key, and a stale `NotKept` retiring a reused name's new holder.
   Answer: skein lib.md 12: a given key is admitted while unheld and held until `reclaim`. `ask` and
   `restore` refuse `Full` or `Held`. Each admission is numbered and returned with its key, and
   `not_kept` names the key and the number, so a stale one is dropped. skein 52d6750.
2. **jig's named send.** client.md 4, 6, 20: a client-side name conflict is an `assert!` panic in
   `send`. Open: the same name and ask under another application key, the same name and another ask,
   a name held by a restored record, the outcome for "name held", and family asks. The planner
   recommended that the first case join.
   Answer: not a join. A `Send` may carry a name, which is the engine key (contract.md 3.1;
   client.md 4: `Send { key, name, request }`). When the table already holds the key (another
   application key, another ask, a restored record), the client answers a new `Outcome::Held` at
   once, keeping and sending nothing, until the holder's record is erased. The assert stays for the
   same application key with another request. Family asks are covered. jig f50c3f63.
3. **A rerun in a fresh process.** tools.md 7, 8.2, 13. Open: the kept request settling first, or
   still parked and colliding with the call, and a repeat past the retention.
   Answer: tools.md 7, 8.2: step 4 ends only once each kept request is answered and erased, so a
   call never meets its own name held, and a rerun under a kept name gets the engine's first answer;
   there is no "kept: pending". A link down past 30 s at step 3, 4 or 5 exits 1 and the records stay
   kept. Past the retention `me` gives, the name is free. temper a7a67b05b.
4. **The turn read.** contract.md 5.1, 6, 7.1; store.md 7.3, 8.3. Open: its row, subject and wire
   shape; its answers (decoded, undecodable, not found); a turn too large for one read, which lost
   its size; tools.md 5.2's "412 KB".
   Answer: jig contract.md 6: `turn{n,attempt,turn}` is a read, never a subject, with
   `activity{n}`'s access. Its answers are Unknown, Gone (past the transcript horizon), Busy,
   Malformed and `TooLarge{bytes}` (the engine's, past one read's bound; the client's, past
   `document_bytes`). Pages keep wire bytes: `TurnProblem::TooLarge{bytes}`. The 412 KB example
   needs a profile whose turn bound passes the page bound (tools.md 11.2). jig f50c3f63.
5. **jig STATUS.md on the one-shot mode.** It lists client.md 20 as "not planned" while tools.md 13
   treats it as designed.
   Answer: jig STATUS.md: the one-shot mode is designed and `temper ctl` is its first host, in this
   plan. The service mode and the one-shot client's story stay unplanned. jig f50c3f63.
6. **The repin takes smith's main.** Moving skein to main forces smith's main (transcript v5, jobs)
   on jig and temper. Open: how they follow `Outcome::{Running,Job}`,
   `CallProblem::{JobsFull,UnknownJob}`, `NotRun`, `Turn.jobs`, and temper's `Request::Job`.
   Answer: jobs never reach temper's engine; temper-engine-smith's `turn` answers non-turn requests
   `Problem::Type` and stays (temper agent.md 4.3). jig client.md 10 words jobs as call outcomes,
   "started as job N", never "is running". jig-protocol-transcript follows v5; `Turn.jobs` is not
   shown. Both STATUS files say this plan moves the pins. jig f50c3f63, temper a7a67b05b.
7. **`adopt` cannot build its record.** client.md 4.4, 4.6 and forge.md 4 need landing, prefix, CI
   or checks, and home; tools.md 5.3 has no flags.
   Answer: tools.md 5.3: `adopt --project P [--forge F] --owner O --name N --landing B`, then
   `[--role owned|fork|context] [--prefix X] [--no-home] [--no-ci] [--check G]...`, with the form's
   defaults. `--role context` turns home off; `--no-ci` without `--check` is a usage error. temper
   a7a67b05b.
8. **Marks.** tools.md 5.4. Open: which bytes are digested, which reads print a mark, and what a
   composite read digests.
   Answer: tools.md 5.4: "mark" is renamed DIGEST (`--since DIGEST`, a `digest:` line): SHA-256 over
   the subject's document as `--json` prints it, without the digest field, 16 hex. Only watch
   subjects print one: `me`; `inbox` and `tasks` without a cursor; `task N` (one per subject with
   `--tree` or `--run`); `run N`. `chat`, `activity`, `turn` and `--activity` lines print none. With
   `--since` and `--until`, the wait is met at the first delivery that differs and meets the
   condition. temper a7a67b05b.
9. **Waiting for a chat's reply.** No condition names it, a chat never ends, and the `phase:` words
   are unlisted.
   Answer: tools.md 5.4: `--until replied` (on `task N`, a chat) is met when the chat's status is
   parked, waiting for you, waiting for them, held, closing or ended. A chat wakes on its person's
   words (jig tasks.md 8.3), so a wait begun after `message` is not met by an earlier reply.
   `phase:` takes the client's 14 phases, lower-case and hyphenated. temper a7a67b05b.
10. **Provisional text.** Agent processes give none (llm.md 7), but 5.2's `chat` and `run` rows do
    not say so.
    Answer: tools.md 5.2: the rows give provisional text only where the run's host gives it; an
    agent process gives none and nothing is printed. temper a7a67b05b.
11. **The three turn bounds.** tools.md 5.2, 6.4, 6.3: the page bound, the client's limit and the
    tools' window. Open: their values, which one wins, and the exit status past the client's limit.
    Answer: the new tools.md section 12, "The bounds", names the page bound (engine), one read's
    bound (engine), the client's `document_bytes` and the tools' window: 50 rows; 32 KiB result
    text; 16 KiB turn part; 1 MiB document; 30 s link deadline; 60 s lock wait at 100 ms. A turn
    past the read bound exits 0 with its size and "past the limit; report it". The step test keeps
    each tools' ceiling within the client limit. temper a7a67b05b.
12. **Missing rows and formats.** Rows for `sign-in`, `sign-out`, `pending` and `help`; the JSON
    field names; the two `decide` forms in `--call`; the `kept:` line; TEXT from stdin.
    Answer: tools.md 5.6: `sign-in FILE|-` (the link is never an argument), `sign-out`, `pending`
    (reads the store file only: no lock, no client, nothing sent) and `help [TOOL]`. tools.md 6.2:
    JSON is one object in snake case; `help TOOL --json` prints the table's record; `decide` takes
    the proposal, or the task and hold, then outcome and reason; `kept:` lines are one JSON object
    per line on stderr. TEXT `-` reads stdin, once per call, not with `--call -`. temper a7a67b05b
    (29 and 33 changed `kept:` and `pending`).
13. **Open exit statuses.** An act whose link stays down, a client-side key conflict, a watch that
    ends mid-wait, and `adopt`'s refusals.
    Answer: tools.md 6.3: 1 is a link down past the deadline (record kept), `Held`, an address off
    loopback, a lock busy past 60 s, a watch ended by sign-out. 2 is an engine key conflict, or a
    wait's subject no longer visible. 3 is a timeout, or a watch ended with its condition unmet.
    `adopt`: adopted 0; unknown past the retention 1; every other 4.7 outcome 2. Questions 24 and 30
    amended two rows. temper a7a67b05b.
14. **The lock.** tools.md 8.3; client.md 11. Open: the mechanism, the wait and exit status under
    contention, and which tools take it. The planner recommended `Op::Lock` on the store.
    Answer: tools.md 8.3: skein's `Lock` on DIRECTORY itself, not the store (renamed on write),
    tried every 100 ms; past 60 s exit 1, "DIRECTORY is busy with another call". Acts, `sign-in` and
    `sign-out` take it; `pending`, `help`, reads and waits do not. Reads and waits use an empty
    in-memory store, never DIRECTORY's. temper a7a67b05b.
15. **The address.** tools.md 8.2 keeps an address file in DIRECTORY; client.md 6.1 passes it as an
    argument. Open: name, format, loopback check.
    Answer: tools.md 8.1: `address` is temper-client-shell-cli's: one line, `http://HOST:PORT`, mode
    0o600, written by `sign-in`. Every call checks 127.0.0.0/8 or [::1] before sending the token,
    else exit 1, "the engine's address must be on loopback". temper a7a67b05b.
16. **Which jig-client limits `temper ctl` sets.** client.md 5.4 against tools.md 6.4.
    Answer: tools.md 12: `ctl` has its own `Limits` preset in temper-client-tools' limits, not the
    terminal's: rows 50, document 1 MiB, segments 9, authority 12, the rest as the terminal's.
    temper a7a67b05b.
17. **The agent's credentials.** How `ctl` signs in on the box (link and password, setup.md 6.5, or
    a script token, setup.md 16), and the forge token the agent needs to read the landed change.
    Answer: `ctl` signs in by the box's printed link on stdin (`sign-in -`). The harness plays the
    person (setup.md 6.5): it signs in on Forgejo's web, changes the password, makes the repository
    to adopt in an org, and makes a person token with read-only scopes (setup.md 16). The agent
    reads the forge with that token, never `admin_token` (setup.md 9; testing.md 2.6).
    `tests/support/owned.rs` still reads admin_token as the proofs' read handle: a plan that touches
    it moves it. temper a7a67b05b.
18. **The real loop to a landed change.** testing.md 2.4, 10; tools.md 11.3. No tier lands a change
    through the HTTP fake forge, the real engine and worker, and an agent on a real kernel. Open:
    (a) the bridge, fake git on the shared in-memory forge or real bare repositories, (b) authority
    and merging if the latter, (c) CI, (d) timing, (e) the tools' root, (f) a lost answer on a real
    ring, (g) the chat's reply, (h) scope.
    Answer: (a) decided by the user and the director: option 1, the fake git program on the shared
    in-memory fake forge, so (b) is moot. (c) CI from a content marker; the sh subset later. (d) The
    forge host delivers signed webhooks to the engine's listener, with a polling backstop at short
    paces; a story past the 1 s guide is made cheaper or moved to fuzzy, never excepted. (e) The
    tools' root is hosted over the native transport; the executable stays with 11.4. (f) Drop the
    hosted root once the engine holds the request, before the answer; a fresh root on another
    DIRECTORY, same party, reruns under the name and gets the first answer, one chat. (g) A hosted
    smith agent on the fake LLM's scripted turns. (h) In this plan: `tests/real-loop`, its fixtures,
    and the hosted service, worker, agent, fake LLM, forge host, fake git and tools' root; later:
    real git, sh CI, faults, several workers. The forge host moves to
    `testing/temper-fake-forge-host` (testing.md 7). temper aeff31a (tools.md 11.3, 11.4; testing.md
    2.4, 4.2, 9, 10).
19. **tools.md has no section 12.** Cosmetic.
    Answer: answered with question 11: tools.md section 12, "The bounds", now exists. temper
    a7a67b05b.
20. **jig's test builds at opt-level 2.** Can they take their dependencies at
    `[profile.dev.package."*"] opt-level = 2`, given testing-strategy.md 8? The planner recommended
    allowing it, with an amendment.
    Answer: first allowed (skein e9cc93b), then replaced by the user's ruling of 10-19: not allowed.
    skein b4c6241 reverts e9cc93b; testing-strategy.md 8 says "A slow test is understood, never
    hidden": test builds stay unoptimized, dependencies included; a slow test's cause is found with
    a profile and fixed; a per-crate override is for one hot leaf crate only, with a profile and
    sign-off cited beside it. jig 04 becomes a profiling pass, then restores 15 s.
21. **Digests of `inbox` and `tasks`.** tools.md 5.4: jig's watches deliver a document unlike the
    read's (no cursor), so a read's digest cannot equal the wait's and `--since DIGEST` breaks.
    Answer: accepted. tools.md 5.4: a read and a wait digest one form, the watch's document: for
    `inbox` and `tasks`, the rows of the first window, in its order, plus whether more follow,
    within the tools' page of rows. The read prints its cursor outside the digest. A subject takes
    only what its watch takes (`inbox` with no flag; `tasks` without `--after` or `--ended`); else a
    usage error, exit 1. temper 4e14c2b5.
22. **`phase:` conditions never met.** tools.md 5.4: the engine ends a task watch when the task
    ends, and a chip's phase is never done, failed or cancelled.
    Answer: accepted. tools.md 5.4: `phase:ended|done|failed|cancelled` are usage errors, exit 1,
    naming `--until ended`. `ended` is met by a delivery showing the subject ended, or by its
    watch's end; for `task N` the answer is the task as a read gives it once ended, so it says how
    it ended. temper 4e14c2b5.
23. **`tree N --until waiting-on-me`.** tools.md 5.4: tree nodes carry no "waits on". The planner
    recommended reading task N's own watch, a second watch kept beside the tree's.
    Answer: the designer differs from the planner. `tree N` takes `ended` only; `waiting-on-me` is
    dropped there, with no second watch. A task's waits-on is computed over its tree (jig
    domain/tasks.md 3), so `wait task N --until waiting-on-me` covers the tree. tools.md 5.4. temper
    4e14c2b5.
24. **Exit 1 for "watch ended by the engine's stop".** tools.md 6.3: jig reopens a watch the engine
    dropped, so no `Ended` arrives.
    Answer: accepted. An engine that stops ends no watch: jig's client reopens what it dropped
    (client.md 8.1). The wait goes on and fails as a link down past its deadline, exit 1; "the
    engine's stop" leaves the exit table. tools.md 5.4, 6.3. temper 4e14c2b5.
25. **A call cut by a signal.** tools.md 6.3 gives no status.
    Answer: accepted. The tools catch no signal: a cut call's status is the OS's, never 0, and a
    rerun under its name gets its first answer. The process tier asserts "not 0", then makes the
    rerun once. tools.md 6.3, 8.2, 11.4. temper 4e14c2b5.
26. **The runner's asker.** jig client.md 4.6 gives the tools' runner none, and temper root.md 15
    reads "one asker per owner, all in one enum, never reused". The planner recommended one asker
    for the runner, each call's number in the application key's high bits.
    Answer: accepted, except on the owner, where the designer differs in part from the planner
    ("Owner gains no variant" in the draft): temper's `Owner` gains `Runner` = 3 (jig client.md
    4.6). The call's number goes in the high 32 bits of each key's and handle's number; calls are
    numbered from one past the highest among the keys kept at `Start`. tools.md 4. temper 4e14c2b5.
27. **A named rerun and the wall clock.** tools.md 7: `shapes::chat` takes the clock, so a rerun in
    a fresh process builds other bytes and jig answers a key conflict, not the first answer.
    Answer: accepted, in jig's domain (jig 62bcf71e, design only; its STATUS rows say "the
    downstream repository's tools plan"). A deadline given as a span means "this long after the task
    is made": the engine makes it a wall from the wall time of the decision that makes the task (a
    `Create`'s admission, or the acceptance of the proposal that asks for it); a proposal keeps the
    span until accepted (jig contract.md 3.1, 3.2, 3.6; client.md 19, where `shapes::chat` takes no
    clock; authority.md 7; tasks.md 4; engine.md 4). temper tools.md 7: a named request is built
    from the call and `me` alone; a rerun after `me` changed what it is built from is another ask, a
    key conflict. temper 4e14c2b5.
28. **`close` takes no REASON.** tools.md 5.3, but `shapes::close_chat` needs one.
    Answer: accepted. tools.md 5.3: `close N [--move D]... [--reason TEXT]`, the reason empty unless
    given, as jig's reference UI sends it. temper 4e14c2b5.
29. **What a `kept:` line names.** tools.md 6.2: jig-client exposes no engine key. The planner
    recommended the name when the call has one, and the application key always.
    Answer: the designer differs from the planner, and corrected a first answer (temper fe0ec316). A
    kept line prints no name and no key: a request's record keeps its engine key, for a named
    request the name's digest (first 16 bytes of SHA-256), never the name (jig protocol/client.md
    10; client.md 11). Text: `kept: ` + the operation (jig's, or the forge's stop or adopt) + the
    outcome as an act prints. JSON: `operation`, `outcome`, no `name`. The runner keeps no records
    of its own. tools.md 6.1, 6.2.
30. **A refusal by jig's client itself.** tools.md 6.3: Unconfirmed, a client-side key conflict, a
    limit. The planner recommended exit 2, as the engine's key conflict.
    Answer: the designer differs from the planner. Exit 2 is the engine's answer alone. The client's
    own answers exit 1: not kept, held, unknown past the retention, a read past its limits.
    Unconfirmed and the client-side key conflict never reach a call (the runner sends nothing before
    `me` is live; each call's keys are its own): either is the tools' bug, exit 1, to be reported as
    a finding (11.5). The plan's draft 03 ("a client limit exits 2") becomes exit 1, except the
    sized turn, which stays 0. tools.md 6.3. temper 4e14c2b5.
31. **`decide P/K`.** tools.md 5.3: jig's `ProposalKey.proposer` is `Task(u64)` or `Party(u64)`.
    Answer: accepted. `decide P/K`, P as `tN` or `pN`: `t41/3`, `p3/17`. 6.1's example reads
    `proposed: p3/17, to maintainers`. tools.md 5.3, 6.1. temper 4e14c2b5.
32. **The forge's words, and skein-ui in the crate list.** Cosmetic, tools.md 4.
    Answer: skein-ui is added to tools.md 4's crate list. The forge's words moving to
    temper-client-domain is temper 11; the design already says so. temper 4e14c2b5.
33. **What `pending` prints, and records the tools cannot read.** tools.md 5.6: with 29, a pending
    row has no key or name; and a kept record of another store version, or of a family or owner the
    tools do not declare, had no defined output.
    Answer: tools.md 5.6, 6.1, 6.2, 6.3, 7, 8.2, 11.3. A row prints the operation, `party N`, and
    the first sending or `not sent`; JSON `{"pending": [...], "unread": N}`, a row having
    `operation`, `party`, `first_sent`. A record the tools cannot read is counted, never listed
    (jig's client drops it at start and gives only `Dropped { count }`): `pending` prints after the
    rows "N kept records these tools cannot read; a call that sends drops them", exit 0; a file that
    is not a small store's exits 1. A sending call prints `dropped: N ...` on stderr; step 4 sends
    only `me`'s party's requests and says "kept: N for party P, unsent while it is not signed in
    here". 11.3 adds two real-loop checks. temper fe0ec316.
34. **A deadline span against an authority's wall bounds.** The engine's checks (a `Create`,
    `propose::allowed`, `goal_accept_allowed`, the acts a document offers) and the client's
    `shapes::set_goal`, which uses `now`.
    Answer: engine side, the planner's (a), stated generally: each check judges a span as its own
    decision's wall plus the span (admission, each re-decision of where a proposal waits,
    acceptance). A waiting proposal's span ages: a later acceptance beyond the accepter's authority
    is answered as any (authority.md 8.1), and past the ceiling it is refused. No new commit kind,
    no time-driven re-routing. Client side differs from all three options: `set_goal` loses `now`, a
    span is within both of a draft's bounds as `AsAllowed` is, and the engine judges it (contract.md
    5.3; client.md 19; authority.md 7). jig 6ad8bb40.

## The plan's own

1. **jig and temper move to smith's main as well as skein's.** Forced:
   smith `29fa97c` does not build on skein's main, and the first smith
   that does, `b415f737`, carries transcript v5 and jobs
   (`findings/base-repin.md`). This plan is "the plan that moves jig's
   smith" of jig's and temper's `STATUS.md`.
2. **The first pins are fixed commits:** skein `b4c6241` and smith
   `a277e922`, the mains at the plan's start (skein's later commits are
   the forge kit's design and STATUS marks), so jig 01 does not wait
   for skein 01. jig 02.1 moves jig's skein to skein 01's landing;
   temper 01.1 pins jig 01's landing; temper 03.1 moves temper's jig to
   the newest landing of jig 02, 03 and 05. The closing repins (17)
   take every repository to the mains the run ends with.
3. **The turn read moves jig's wire to version 9,** nothing translated
   (protocol/README.md 7, contract.md 8; the design names no number).
   jig 03.2, 03.3, 05.2 and 05.3 change version 9 in place; there is no
   version 10 in this plan.
4. **The named send changes no wire:** the name is the engine key the
   wire carries already. jig 02 changes no wire version.
5. **For the next plan:** `temper ctl`'s one-shot host answers the
   client's Save and Erase in memory for reads and waits (tools.md 8.3)
   and drops the loaded records; jig's native Shell is not changed (it
   has no memory store; tools.md 15 keeps the host temper's). This
   plan's library therefore leaves the store to its host: the root and
   the runner take the kept records they are given and touch no file.
6. **For the next plan:** temper-client-shell-cli's host is a skein host
   with no main of its own, and the real loop hosts it as a program per
   call, so that DIRECTORY's lock and files and a call's exit run on the
   real kernel (`findings/next/05-ctl.md`, `08-real-loop.md`).
7. **Proofs by tier.** The user's direction of 10-18 22Z puts area 1's
   stories in the real loop (testing.md 2.4) wherever it can carry them.
   The real loop is the next plan's (16), so this plan proves the tools
   as a library, in `temper-client-tools`' unit tests, and jig's changes
   in jig's worlds and joint stories; the process tier carries only the
   repins' own checks (jig 01.3, temper 01.1). The tools' stories
   through a hosted call are the next plan's, in `tests/client` and
   `tests/real-loop` ("For the user", B).
8. **For the next plan:** the real loop needs no skein addition:
   `real::World` kills a hosted program (`Op::Signal{pidfd, Kill}`) and
   spawns it again after `Op::Wait`.
9. **For the next plan:** the fake forge's host moves to
   `testing/temper-fake-forge-host` in a session of its own, extending
   the existing fake forge (`ForgeHost`, temper-fake-forge-protocol's
   delivery) on a kernel-assigned port; no new fake.
10. **Forge- and LLM-facing work in increments of its own** (the user's
    addendum 2 of 10-18 22Z): temper 11 holds the forge's words and
    `adopt`; temper 03.1's forge arm is a repin's compile repair. The
    README's Hotspots list the files the forge kit and the LLM accounts
    work would also touch.
11. **jig 04, the test cost, is in this plan,** as the director asked
    (10-19, after the user's ruling on question 20): a profiling pass on
    jig's focused suite (`findings/jig-test-profile.md`). It starts at
    once, beside jig 01; each increment fixes one cause, in the code or
    the test, named in its commit; 04.4 reverts ruling 275's commit (the
    focused budget back to 15 s) once jig 01 has landed, so the time is
    measured at the pins the plan keeps, and the 15 s binds jig 02, 03
    and 05.
12. **Areas 2 to 4, the MCP endpoint and the tools in jig** (tools.md 14)
    are in `later.md`; the QA loop is not in this plan.
13. **A named send under the same application key with another name is
    a key conflict,** loud as today's (the assert kept and extended): one
    application key names one request (client.md 4); the design does not
    say. A named send before `me` is answered Unconfirmed first
    (client.md 11).
14. **jig 05, deadline spans, is in this plan:** jig `62bcf71e`'s design
    (a deadline given as a span, made a wall by the engine; shapes read
    no clock) names "the downstream repository's tools plan" in its
    STATUS rows, and tools.md 7's named requests need it (a name sent
    again makes the same ask; question 27). Question 34's answer fixes
    the engine's checks (the decision's wall, the span at each check).
15. **Sessions over section 5's 400 lines are split,** the new halves
    numbered after the last, with no renumbering: temper 11, the tools'
    forge parts and `adopt` (from 03); temper 12, help and a call's JSON
    (from 02); temper 13, the real loop's landing (from 09, drafted for
    the next plan).
16. **Area 1 runs as two consecutive plans** (the director's rust-d1,
    Proposal 1: a longest chain past about 20 increments, or more than
    about 80 steps, runs as consecutive plans). Area 1 whole had 68
    steps and a longest chain of about 31. This plan, the first, has
    38 steps and 4 closing repins, with a longest chain of 21 (jig 01,
    03, 05.2 to 05.5, temper 03, 04, temper-repin): section 13, and the
    tools as a library. The next plan starts from this one's `main`:
    `temper ctl`'s host (tools.md 5.5, 8), the client stories, the fake
    forge's host, the real-loop world and its stories, and the process
    tier; `later.md`'s first row names it, and its drafts are in
    `findings/next/`, with their numbers (temper 05 to 10, 13), as its
    planner's input. Section 13 first, as the brief asks, and 5.5 next:
    the split falls between them.
17. **Closing repins** (development.md 5.7): `smith-repin` once skein 01
    has landed; `jig-repin` once smith-repin and every jig step have;
    `hearth-repin` once jig-repin has and hearth has a checkout on the
    run's machine; `temper-repin` once hearth-repin and every temper step
    have. Neither smith nor hearth uses skein's request table, so their
    repins change pins and absorb only what skein's main changed.
18. **The Pulse's aims are this machine's:** the run's machine
    (debian-16gb-hel1-1) ran each suite three times at the plan's start
    (`findings/base-times.md`): jig's focused 5.23 s, fuzzy 6.72 s;
    temper's focused 2.74 s, fuzzy 7.04 s. The aims are jig 7 8 and
    temper 5 8: each suite's median plus what the run may add (the
    README's Pulse), rounded up to the second.

## Rulings


101. **jig 01.1's skein pin after smith's update** (plan defect, jig 01's block of 2026-10-19 03:31:33 UTC).
     Ruling 101: a plan defect. The Pins rule fixes the result, "prints two lines: `skein.git#b4c6241…`; `smith.git#a277e922…`" (jig/01-repin.md, Rules that bind it), and the Shape's order cannot reach it once skein's `main` has moved past `b4c6241`: smith's update moves the skein packages that only smith's crates use to skein's `main`. temper 01.1's Shape already says "If a source still holds two revisions, run its command once more." jig 01.1 does the same: after smith's command, run `heavy cargo update -p skein-io --precise b4c6241aefea0e434b6e1128754168d02bae0706` once more, then check the Pins rule's two lines; a second revision still there after that is a question. The README's Pins rule now says it for every step: a source left with two revisions by a later command gets its own command once more, last.

102. **jig 05's Shapes against main** (ready check of jig-05.1).
     jig 05's ready check (skein 7ee1ac86, smith a277e922, jig dca2bbfb, temper 3119ae3a) found users its Files missed; jig/05-deadline-spans.md now names them. 05.1: `crates/jig-protocol-store/src/tests/{format,commits}.rs` join its Files, and its Test shape names the version cases that follow (want 4; the people case of `another_family_is_refused_naming_the_first_difference` finds 5, an arm per family; the other-version people row writes 5; `memory.rs` and `fuzzy_decode.rs` say core 3, tasks and people 4). 05.2: `tests/client/tests/stories.rs` and `tests/client-wire/tests/fuzzy_client_wire.rs`. 05.3: `acts/requests.rs` and `routing/people.rs` of jig-domain-core; its protocol sketch takes `.expect("first task")` (clippy denies `unwrap_used`), and its core sketch's `use` is in rustfmt's form. 05.4 and the Other jig users: the client world's `src/{stories,stories/goals,harness/referee/offers}.rs` and the ops world's `src/client/stories.rs` and `tests/client_stories/creating.rs`, which call `shapes::{chat,set_goal,fall}`. With 05.1's corrections, the store suites passed, 310 of 310 and 14 of 14 under fuzzy, and the workspace's 4272. A further user found: change it as its increment's Shape says, and list it in the commit body.

103. **temper 01's Shapes against main** (ready check of temper-01.1).
     temper 01's ready check (skein b3146c74, smith a277e922, jig b2aded80, temper 8f09dc12) compiled and ran only the crates that do not depend on jig: 120 passed, 3 under fuzzy, clippy clean, with T1 to T6 applied. temper/01-repin.md now says: `smith-transcript` is a dependency of `tests/client`, not a dev-dependency, and `tests/client/Cargo.toml` and `Cargo.lock` join T6's Files; `Limits::new(bytes, turn)` is on jig's `main` already (`pub const fn new(bytes: u32, turn: TurnLimits)`), so Shape 6 is Choices' form, and a JIG01 without it is a question; `turn` joins the `use crate::{…}` of `crates/temper-engine-smith/src/tests.rs`; step 7's memory, if reached, changes `docs/examples/owned/agent.json`'s `"memory"` (2147483648) too, which the owned deployment tests load through `SHIPPED`. The check's worry that temper's jig pin trails jig's `main` does not hold: `git diff --name-only bbea9408 origin/main` in jig, outside `docs/`, names only `crates/jig-client-view/src/{owner_tree,sections/project/owner,sizes/pages}.rs`, jig 04.1's. The jig-dependent crates are checked again when 01.1 starts, against JIG01. A further copy of a changed value: change it as its step says, and list it in the commit body.

104. **jig 02's Shapes against main** (ready check of jig-02.1).
     jig 02's ready check (skein b3146c7 with 7ee1ac86 pinned and a local `ask_given` stand-in, smith a277e922, jig b2aded80, temper 8f09dc122) built jig-client-domain with 02.1 and 02.2 applied: clippy and fmt clean, nextest 44/44 after 02.1 and 53/53 after 02.2. jig/02-named-send.md now says: 02.1's `restore_record` takes the admission as `self.table.restore(..).ok()`, as clippy's `manual_ok_err` wants, the exhaustive-match rule covering a `match` only; 02.2's assert is `held.request == request && named`, with `let named = match name { None => true, Some(name) => name == held.admission.key };`, since jig's clippy.toml disallows `Option::is_none_or` ("no closures: match"); `a_name_is_admitted_again_once_its_holder_is_answered` drains `c.downs()` as well as `c.told()` after the `Final` answer; jig-client's `Event::Send` takes no doc line in the sketch; `cargo fmt` runs after `send`'s signature is pasted. Not built by the check, as jig 01 was not in it: jig-client, jig-client-view, ops, the test domain and the protocol world, the fuzzy and browser profiles, `named.rs` and the Done-when greps; they are checked again when 02.1 starts. A further site the compiler or clippy names: fix it as the step's Shape says, and list it in the commit body.

105. **jig 03's Shapes against main** (ready check of jig-03.1).
     jig 03's ready check (skein 93052c6b, smith a277e922, jig dca2bbfb, temper 3119ae3a) applied 03.1 in a scratch worktree and, with the further sites its diagnosis found, ran jig's gate at version 9: drift with and without `SKEIN_CODEGEN_WRITE=1`, the workspace's 4270 tests, fuzzy's 111, clippy (native and wasm32) and fmt clean, the browser tier's 13; the proof `an_engine_outside_jig_s_version_leaves_the_client_out_of_date_sending_nothing` passed in 0.019 s. jig/03-turn-read.md now says: the focused budget is `.config/nextest.toml`'s 30 s, not 15 s; 03.1's jig-version literals are those a wider `rg` finds, 32 files at jig `1899792f`, named in its Files: among them the scripted engines' `versions().contains(8)` in `tests/client/src/scripted.rs` and `tests/ops/src/client/transport.rs` (left at 8, about 68 client stories read "Client out of date"), three `version(), 8` asserts, `SERVICE_ANSWER`'s version byte and `Owner::Jig, 8, 8`; family numbers stay; the `a_v8_*` tests become `a_v9_*`, `v8` leaves three test files' panic texts, and tech-debt.md names v9; `client-v9.schema`'s first line is `family jig-client-wire 9`; the out-of-date story serves `serve_jig(8)` and expects `VersionRange { first: 8, last: 8 }`, the restart story serves 9 to 9, and `cargo fmt` runs after; 03.2's held answer is `Held::Read` (`Held::ReadAnswer` does not exist), and the client `ReadNone` matches in `examples/ops/client-domain/src/family.rs`, `tests/client-protocol/src/family.rs` and `tests/client/src/scripted.rs` join its Files; `ReadValue`'s matches in `tests/client/src/{observe,scripted,harness/referee,harness/referee/offers}.rs` join 03.3's; `tests/client/src/joint/wire/pages.rs` joins 03.4's; the last Done-when command, which passes on `main` already, now greps `tests/client-protocol/tests/turn_read.rs` for `a_turn_read_after_restart_keeps_its_archive`. temper's users of v9, `TurnProblem`, `ReadValue` and `ReadNone` are in temper 03.1's Files already. The browser story `under_a_prefix_the_inbox_s_links_carry_the_prefix` failed once under load and passed three reruns, unrelated to the version: if land's gate fails it, report the log as an infrastructure block, without a retry loop. A further jig-version literal: change it to 9, and list it in the commit body.

## For the user

The choices below have long-term consequences; each is the planner's,
and the user, or the director, may veto it before the run:

- **A. jig's 15 s budget, and what jig 04 can win** (11). The budget is
  a wall-clock time and depends on the machine: the run's machine
  (debian-16gb-hel1-1, 8 vCPU) runs jig's focused suite in 5.23 s
  already, carbon-forge (4 cores) in 14.4 s. So jig 04's proofs also
  read the slowest tests' own times before and after each fix
  (nextest's per-test times), which carry from one machine to the next.
  The profile (`findings/jig-test-profile.md`: 4,270 tests, 59.4 s
  summed) gives three fixes, about 2.5 to 3.6 s summed (page bounds
  reused, the vocabulary's capacity kept by its book, the drain's bound
  reused), then the restore to 15 s. The largest cause, the phrasebook's
  first load in each test process (about 2.7 s summed), is not fixed
  here: a cache across tests would cross the process boundary, and data
  prepared at build time needs a skein form for it (`later.md`). A veto
  keeps the wall-clock alone, or drops jig 04 and leaves the budget at
  30 s.
  Decision: accepted, by director, 2026-10-19.
- **B. Two plans, and the real loop in the second** (7, 16). This plan
  builds section 13 and the tools as a library, proven by unit tests in
  temper and by jig's worlds and joint stories; `temper ctl`, the client
  stories, the fake forge's host, the real-loop world (testing.md 2.4,
  tools.md 11.3) and its stories, and the process tier are the next
  plan's, drafted in `findings/next/`. The real loop, as the general
  layer the user asked for, is built there whole: its world, the hosted
  service, worker and agent, the scripted fake LLM, the fake forge's
  host over the existing fake forge, signed deliveries, and the cli host
  hosted per call. `later.md` lists what it still lacks to cover the
  rest of temper. Exceptions to "the real loop wherever it can": none
  of this plan's tools' proofs, because no real loop exists before the
  next plan; jig 01.3 and temper 01.1 (C) prove repins on the process
  tier, which only the executables show. A veto runs area 1 as one plan
  of about 68 steps with a longest chain of about 31.
  Decision: accepted, by director, 2026-10-19. Two plans, with the real loop built whole in the second.
- **C. The process proofs, and where they run** (7). jig 01.3 drops a
  declared memory in jig's process tests; temper 01.1's process proof
  runs `a_native_chat_reaches_the_model`, which needs `TEMPER_FORGEJO`
  (the Forgejo binary's path). It is unset on the run's machine. Until
  the machine has it, temper 01.1's process proof runs where Forgejo is
  installed, or the supervisor rules it one layer down (implementer.md
  8.8). A veto installs Forgejo on the run's machine before the run.
  Decision: vetoed, by director, 2026-10-19. Forgejo 17.0.0, the pinned build (sha256 5a3b07e1…42ca), is installed on the run's machine, debian-16gb-hel1-1, with TEMPER_FORGEJO set in its login profile and Claude's environment, so temper 01.1's process proof runs there (the client-api retrospective's proposal 7: no Proof lands "unavailable").
- **D. The user's drafts** (temper `draft/llm-setup.md`, jig
  `draft/llm-accounts.md`): area 1 chooses nothing they must undo. The
  points that touch them:
  - tool names: area 1's are plain verbs; the accounts tools may want
    `remove`, `order`, `check` or `resume`, and area 1 takes none;
  - a secret never rides in a kept request: the token is a file of its
    own, never in the store (tools.md 8.1, the next plan's);
  - a call prints one result; a sign-in that waits for a browser (the
    drafts' OAuth on port 1455) would be a wait tool, not a change to
    that rule;
  - no config file (tools.md 8.1);
  - the named key (SHA-256 of `"temper-tools"`, a zero byte, the name)
    is generic: keyed account acts can reuse it.
  Decision: accepted, by director, 2026-10-19.
- **E. jig's jobs read as call outcomes** (jig 01.2): "Started as job
  N", "Job N ended: exit C", "Not run: after a failed write"; jobs never
  reach temper's engine (`Request::Job` answered `Problem::Type`).
  Decision: accepted, by director, 2026-10-19.
- **F. The repin absorbs smith's parity and wall-clock changes** (1):
  transcript v5, jobs, event vocabulary v3 and the larger shipped
  memory, into jig and temper, in jig 01 and temper 01.
  Decision: accepted, by director, 2026-10-19.
- **G. The forge kit** (skein `draft/forge.md`, temper
  `draft/forge-kit.md`) (10): temper 11 moves the forge's words from
  temper-client-view to temper-client-domain and builds `adopt` on them;
  the kit, which would move the forge client, will rebase them. The
  README's Hotspots list each file and step. The fake forge's host is
  the next plan's.
  Decision: accepted, by director, 2026-10-19. The forge kit's run follows this one's temper 11, on another machine, by its own hotspot list.
- **H. hearth's repin** (17): hearth has no checkout on the run's
  machine, and its own run on hel8 is paused. `hearth-repin` waits for
  a checkout and runs after jig-repin; if hearth's run resumes first,
  the two runs' pins meet in hearth's lock. A veto drops hearth-repin
  from this plan and leaves hearth on its own pins.
  Decision: accepted, by director, 2026-10-19. hearth is cloned on debian-16gb-hel1-1; hearth's own run stays paused.
- **I. The aims are this machine's** (18): jig 7 8 and temper 5 8, from
  three runs each on debian-16gb-hel1-1. A run on another machine
  measures new base times first.
  Decision: accepted, by director, 2026-10-19. The run goes on debian-16gb-hel1-1.
