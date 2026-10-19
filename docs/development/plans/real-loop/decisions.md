# real-loop: decisions

The questions this plan raised and their answers first; then the
decisions this plan rests on, numbered; the supervisor's rulings during
the run follow them, from 101; the choices the user may veto close the
file. The director answered for the user (development.md, 2.7).

This plan is the second part of temper's tools plan
(`docs/development/plans/tools/`), whose decisions and answers it
builds on and does not repeat: below, "the tools plan's question N"
names them.

## Design questions

1. **An account first, with no tools to run it.** testing.md 2.4 ("An
   account first"), 2.6 (Credentials) and tools.md 10, 11.3 and 11.4 make
   every real-loop story, and the process tier's first act, add the
   deployment's key with `add-key` from a private file and wait for
   `wait accounts --until usable`. Those are area 5's tools, which
   `temper-client-tools` lacks and STATUS.md lists as not planned (LLM
   accounts' second part). Today the engine's credential is a
   `handed_in` file its configuration names
   (`llm.accounts[].source.handed_in.file`).
   Answer: until area 5's tools land, the real-loop world and the process
   tier hand the deployment's credential in as a `handed_in` file, as
   main does. The account-first step comes with area 5's tools: a
   `later.md` row, and the item under "Not planned". testing.md 2.4 and
   2.6 and tools.md 11.3 and 11.4 are reworded at the designer's next
   pass. By the director, 2026-10-19.

2. **The fake LLM's parts from the live-text and accounts plans.**
   testing.md 2.4 names skein's fake LLM with several credentials,
   failures per credential and observations of route, scheme and
   credential index (`fake-llm.md`, 2.2, 2.3 and 3), which those plans
   build.
   Answer: area 1's stories need one credential and whole scripted turns,
   both on skein's main, so the plan awaits no live-text or accounts
   step. The real loop's fake-LLM helper passes the peer's own types
   through, so that their parts are usable once they land, and a
   `later.md` row names the stories that will use them. By the director,
   2026-10-19.

3. **A signal to `temper ctl`.** tools.md 8.2 says "the tools catch
   none, so its exit status is the OS's", and the process tier cuts an
   act with SIGTERM and expects a status other than 0 (tools.md 11.4;
   the tools plan's question 25). skein's startup blocks termination
   signals and delivers them as `Shutdown { signal }` (skein's io.md, 7;
   shell.md, 6), so a skein host is never killed by one: what it does on
   `Shutdown` is its own.
   Answer: on `Shutdown`, the host ends the call at once: it sends nothing
   more, leaves DIRECTORY's kept requests as they are, prints nothing
   more, and exits with 128 + the signal's number (130 for SIGINT, 143
   for SIGTERM), as a shell shows a process the signal killed. tools.md
   8.2 is reworded at the designer's next pass, beside question 1's
   rewordings. By the director, 2026-10-19.

4. **The shared fake forge and the checked world.** In temper 13's
   forge stories, hosted fake git writes into skein's forge peer's shared
   `Forge` handle (skein's fake-forge.md, 5); memory git allocated there
   outlives git's exit, so skein-world's checked world (`Heap::release`)
   fails it, while tools.md 11.3 has the harness check memory ownership,
   outstanding operations and release.
   Answer: the forge stories run on skein-world's `World::new_controlled`,
   unchecked; every other real-loop story stays checked. tools.md 11.3
   says so at the designer's next pass. A `later.md` row asks skein for a
   checked world that knows a shared handle's owner, and the plan's
   landing puts it in skein's `STATUS.md` under "Not planned", so that
   the forge stories become checked once it lands. By the director,
   2026-10-19.

5. **Three failures with no words** (tools.md 6.3): the link down past
   `DEADLINE`, a DIRECTORY never signed in, and an absent DIRECTORY given
   to a call other than `sign-in`. Drafted on: only `sign-in` creates
   DIRECTORY; the others exit 1 with `temper ctl: the engine at ADDRESS
   did not answer within 30 s`, `temper ctl: DIRECTORY is not signed in`
   and `temper ctl: DIRECTORY does not exist` (temper 05.1, 05.6).
   Answer: as drafted. Only `sign-in` creates DIRECTORY, so an act on a
   mistyped path never leaves a directory behind. By the director,
   2026-10-19.

6. **`pending`'s first-sent words** (tools.md 5.6, "wall time"). No tools
   carrier prints a wall time (`findings/code-checks.md`, 3). Drafted on:
   `first sent HH:MM UTC` through the tools' `document::utc`
   (`pub(super)`, visible to a sibling `render::pending`), the clock
   words the documents print; JSON keeps jig's integer (temper 05.5).
   Answer: as drafted. By the director, 2026-10-19.

### For the designer

What the answers above ask of temper's design. temper `a959f6e10` folded
in questions 1, 3, 4, 5 and 6 and adopt's two `kept:` forms
(`findings/code-checks.md`, 1). Three remain for the designer's next
pass:
- `domain/testing.md` 2.4: the real loop's paces gain `hinted`
  (decision 11).
- `client/tools.md` 11.3 still names "the fake forge's host", now
  skein's forge peer (skein 05).
- `client/tools.md` 5.6 and 8.1 say `sign-out` "erases the token"; jig's
  native shell leaves a zero-byte `token`, read as signed out
  (decision 14).

## The plan's own

1. **Skein's forge peer, hosted in the real loop.** forge-kit's skein 05
   has landed whole, the peer's webhook delivery (skein-05.5) included,
   and the tools plan's temper-repin pins it; forge-kit's temper-05.2,
   which puts fake git on the peer's shared handle and teaches the
   one-process world the port-0 hooks address, has not. The real loop's
   forge stories wait for it, and for skein-05.5 in name, so that the
   wait reads as the two steps it rests on. They host
   `skein_fake_peers::forge::Peer` themselves, and extend skein's fake
   rather than build one in temper. The tools plan's draft 07, a temper
   host in `testing/`, is dropped, as forge-kit's decision 26 says. The
   rest of this plan waits for no forge-kit step.

2. **The hooks listener's bound address is the world's to learn.** The
   world reads it from io's completion of the engine's port-0 listen, as
   forge-kit's temper 05.2 does in the one-process world, and hands it to
   the forge peer. The engine gains no accessor: draft 08.1 is dropped.

3. **No upstream change, no repin.** The plan changes temper alone. A
   need upstream that a session finds is a question (development.md,
   10.1), not an edit.

4. **Sessions keep their drafts' numbers:** temper 05, 06, 08, 09, 10 and
   13, so that the drafts' cross-references hold. 07 is not used.

5. **The meeting point comes first** (development.md, 5.3). The real
   loop's first story through `temper ctl` (temper 08.3), a sign-in and
   a chat to its reply, with `temper ctl`'s host hosted as a program per
   call, needs only what temper 05 lands first: DIRECTORY, the native
   transport, `sign-in FILE`, a read and the `chat` act. temper 05 lands
   those before the rest of its acts, its waits and `pending`, which then
   prove in the real loop as well as by units.

6. **The person on Forgejo's web is the box's.** tools.md 10 has the QA
   harness sign in on Forgejo's web as the person, change the password
   and make a read-only token; tools.md 11.4 does not ask it of the
   process tier, and skein's throwaway Forgejo has no web sign-in. The
   process tier's landing keeps the person its support makes today.
   Drafts 10.1 and 10.2 are dropped, with a `later.md` row for the QA
   loop.

7. **One landing in the process tier, today's.** testing.md 2.6's
   landing story (`crates/temper/tests/process.rs`, the person's landing
   through the native client) is the tier's one landing on the owned
   Forgejo, against real git and CI (tools.md 11.4). The tools' `adopt
   --landing` does not join it: serve's printed link is spent by its
   first sign-in, so `temper ctl` and the native person cannot share that
   run, and the native client would lose its only landing. The tools'
   landing is the real loop's (temper 13); the process tier's cut act
   (temper 10.2) stops at the adoption, with no CI.

8. **A story without the forge gives the engine an unserved forge.**
   temper's startup takes at least one forge, and testing.md 2.4 gives
   the forge peer only to the stories that need it. Until temper 13, a
   real-loop story's configuration names a loopback listener the world
   binds on port 0 and never serves; temper 13's forge stories name the
   forge peer instead. If the engine's start or drain waits on that
   forge, temper 08.2 raises it (its Questions) before any other choice.

9. **Another party's kept request waits for area 3.** tools.md 11.3 has
   a story in which a DIRECTORY signed in as another party leaves the
   first party's kept request unsent, and sends it once the first party
   signs in again. Signing a second party in through the tools takes area
   3's `sign-in --service` (tools.md 5.5, 9), and the engine's printed
   link is spent by its first sign-in, so area 1 has no way to make the
   second party in the real loop. temper 05 proves the line
   (`kept: N for party P, unsent while it is not signed in here`) and the
   exit by units on its own fixtures; the real-loop story comes with
   area 3's tools, a `later.md` row.

10. **A wait across an engine restart waits for the real loop's faults.**
    tools.md 5.4 has a running wait outlive the engine's restart. The
    real loop gives each start a kernel-assigned port (testing.md 2.4),
    so a restarted engine listens elsewhere and DIRECTORY's address names
    the old one; the restart story (temper 09.1) keeps the work, not a
    call. The real-loop story comes with the faults row of `later.md`,
    which drops and restores the loopback links.

11. **The stories set `hinted` short too.** testing.md 2.4 has the real
    loop's polling run "at paces the stories' configuration sets short
    (`poll`, `poll_max` and `slow`)". `forge.client.hinted_ms`, 1 s
    shipped, re-arms a read after each delivery, so a landing would wait
    on it past its 1 s aim. temper 13's stories set it short with the
    other three, through the profile's builder and `admit`; testing.md
    2.4 names it at the designer's next pass.


12. **The fuzzy walking story's cost, fixed at its cause (temper 14).**
    The tools plan's run missed one aim (its ruling 139): temper's fuzzy
    suite's median is 9.2 s against 8 s, and the suite's wall time is
    one test's, `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`
    (`tests/engine/domain`), 7.8 s alone, the median of three
    (`findings/fuzzy-walking.md`). At the director's request, one step,
    which needs nothing, profiles it and fixes the cause in the code or
    the test (`findings/fuzzy-walking.md`), never by a budget, fewer
    cases than its claim needs, or optimized builds (testing-strategy.md,
    8). The profile, run in planning, found the cause upstream: 84% of the
    story is the `Debug` walk of both worlds at each iteration, and a
    structural compare needs `PartialEq` or `Hash` on skein-lib's
    collections and jig's `Core`, the state digest of testing-strategy.md
    6 (`later.md`). temper 14 therefore shards the story into eight tests
    of eight seeds, each under the fuzzy ceiling, with the same seeds and
    the same compare at every iteration, and names the cause in its doc;
    the CPU it costs is unchanged. Asked of the director (its inbox's
    question 7, 2026-10-19 12:55Z). Answer: (c), by the director, 2026-10-19: temper
    14 shards the story, and each shard's doc names the cause (the `{:?}`
    of both worlds at every iteration, 493 MB of text, 84% of the time);
    the digest is its own small plan, next, right after this one, starting
    from `findings/fuzzy-walking.md` (`later.md`; For the user, K).

13. **The tools plan's retrospective, followed where it costs little.**
    Its adopted proposals 1, 2, 5 and 6 (temper e139ccaf4): the steps
    were split into sessions before any file was refreshed; each printed
    field, cursor, bound and act temper 05 adds is mapped to the type
    that carries it (`findings/code-checks.md`); the longest chain is
    projected at about 24 minutes a step; no new test is named with
    `_and_`. Where tools.md lags the code on the tools plan's decisions
    M to Q, the sessions follow the code and those decisions.

14. **Sign-out leaves a zero-byte `token`, read as signed out.** jig's
    native `Shell::keep(None)` writes it so; temper 05.5 reads a
    zero-byte `token` as none, and changes no jig code (3).

## Rulings

101. **The code graph for temper, re-indexed** (steer of 2026-10-19T13:51:25Z).
     The code graph indexes temper again: re-indexed at 2026-10-19T13:51:25Z, at temper `ed725ce57`, past the claim. The README's rule "**The code graph** is not used for temper until the supervisor has re-indexed it past its `origin/main` (AGENTS.md) and a steer says so. Until then, search with `rg` in the worktree." is removed; AGENTS.md's rule stands in its place: "Check each name the graph gives with `rg` in the worktree before relying on it." The graph indexes the primary checkout's `main`, never a session's worktree, and lags behind what lands; the hourly check re-indexes it.

102. **temper 05.1: DIRECTORY opened before its private open; a zero-byte token** (plan defect, temper 05's block of 2026-10-19T13:52:58Z).
     Plan defect: 05.1's Shape prescribes `OpenPrivate`, which makes an absent root, for every call. tools.md 6.3: "`temper ctl: DIRECTORY does not exist`, for any other call on a DIRECTORY that is not there"; decisions.md 5: "Only `sign-in` creates DIRECTORY, so an act on a mistyped path never leaves a directory behind."

     - Every call but `sign-in` first sends `file::Request::OpenDirectory { root: parent, name: leaf, no_follow: true }`, which opens and never makes. Its `NotFound` exits 1 with `temper ctl: DIRECTORY does not exist`, before any `OpenPrivate`. Its `Opened` token is closed (`Close`), then `OpenPrivate`, `Lock` and `Load` go on as the Shape says, keeping io's private-root checks. `sign-in` sends `OpenPrivate` alone, as now.
     - The window between the two opens, in which another process removes DIRECTORY and `OpenPrivate` makes it again empty, is accepted: that call then finds no `token` and exits 1, "DIRECTORY is not signed in". An existing-only private open in skein's io would close it: `later.md`'s new row, not planned. No upstream change and no pin moves ("temper alone").
     - Also from temper 116d0a645 (tools.md 8.1, after the plan's base): `sign-out` "leaves `token` a zero-byte file, which a call reads as no token". 05.1: a zero-byte `token` exits 1 as an absent one, `temper ctl: DIRECTORY is not signed in`. New 05.1 tests: `an_absent_directory_is_never_made` (no `MakeDirectory` or `OpenPrivate` submitted; exit 1) and `a_zero_byte_token_is_not_signed_in`.
     - 05.5: `temper-client-tools`' `src/jig/rows.rs` joins its Files: `SIGN_OUT`'s `line`, "End sign-in and erase the token", becomes "End sign-in and leave the token empty" (tools.md 5.6: "leaves `token` empty, zero bytes, read as signed out"), with `tests/golden/help.txt`, which repeats it; its test `sign_out_waits_for_the_token_erase` is named `sign_out_waits_for_the_empty_token_stored`. Refresh any further copy of the old line, and list it in the commit body.

103. **temper 06.1: the presentation allowance at the person's preset** (plan defect, temper 06's block of 2026-10-19T13:58:19Z).
     Plan defect: Choice 1 keeps "`worst_case` stays one function of the limits", and Choice 2 gives the agent "the tools' preset", but the Shape did not foresee that `view::Limits::of` refuses that preset, so `Temper::worst_case(&limits::client())` panics at `fixtures.rs:76` before any heap peak exists. The proposal is adopted.

     - `Temper::worst_case(limits)` is `domain::worst_case(limits)`, of the limits it takes, plus `2 × view::worst_case` computed wholly at `fixtures::domain_limits()`, the person's preset (its `phrasebooks`, `view_bounds` and limits). Every person world takes `domain_limits()`, so the person's value is unchanged; the agent's root needs no presentation, and this allowance is its upper bound.
     - `agent::new`'s `View::Agent { limits }` is `fixtures::view_bounds(&fixtures::domain_limits(), ..).tree`; its tree stays empty.
     - The agent's root, its `Joint` limits and its codec stay the tools' preset (Choice 2); the Don't's "the person's limits for the agent" means those. No tools-root `worst_case`, no upstream change.
     - If `domain::worst_case` refuses the tools' preset, or the heap check fails, raise it with the error or the peak (06's Questions). Refresh any further copy of the presentation allowance computed at the supplied limits, and list it in the commit body.

## For the user

The choices below have long-term consequences; each is the planner's,
and the user, or the director, may veto it before the run:

- **A. The process tier's five proofs, the exceptions to the real loop**
  (temper 10; tools.md 11.4; design question 3). The real loop hosts
  `temper ctl`'s library as a program per call, never the executable,
  so five proofs stay on the process tier, each for what only the
  executable shows: `temper ctl help` (argv in, stdout out); a usage
  error's exit 1 (`main`'s conversion of the host's exit to the OS's
  status); `sign-in -`, then `me --json` (serve's printed link through
  process pipes into ctl's stdin); a wait's timeout, exit 3 with its
  digest (the OS's status beside the printed document); an adopt cut by
  `SIGTERM`, then made once (the OS's signal, the host's exit 143). The
  tier goes from 152.9 s (`findings/base-times.md`) to about 165 s for its 3 min aim
  (testing.md 2.6). No proof of this plan is on the live tier. A veto
  proves the five by units of `crates/temper` and leaves the executable
  to the tier's existing stories.
  Decision: accepted, by director, 2026-10-19 (each proof is what only
  the executable shows).
- **B. The forge stories run unchecked** (temper 13; design question
  4). Hosted fake git writes into skein's forge peer's shared handle, so
  temper 13's stories run on skein-world's `World::new_controlled`:
  the binaries `tests/git.rs` (13.1's four hosted-git tests) and
  `tests/forge.rs` (13.2's delivery, 13.4's adoption and its refusal,
  13.5's landing), eight tests. Every other real-loop story is checked. They become
  checked once skein's checked world knows a shared handle's owner
  (`later.md`; skein's STATUS.md, "Not planned").
  Decision: accepted, by director, 2026-10-19 (design question 4).
- **C. The real loop's cost** (testing.md 6.1). The plan adds sixteen
  real-loop stories to temper's focused suite, each aiming at 1 s,
  unoptimized; each increment that adds one runs it alone three times
  and gives the median in its commit body. A story over 1 s is made
  cheaper or moved to the fuzzy profile (`fuzzy_restart.rs`,
  `fuzzy_landing.rs`), never excepted. The plan's packages go from 9.90 s summed at the base
  to at most 37 s focused, the sum of the Proof bounds; about 20 s is
  expected (README, Pulse).
  Decision: accepted, by director, 2026-10-19 (the testing vision: the
  real loop over all of temper; a story's cost is cut or moved, never
  excepted).
- **D. Skein's forge peer, awaited from forge-kit** (1). temper 13
  waits for forge-kit's temper-05.2 (skein-05.5 has landed), run on
  another machine; the rest of the plan waits for no other plan. A veto builds a
  forge host in temper (the tools plan's draft 07), a second fake forge
  beside skein's.
  Decision: accepted, by director, 2026-10-19.
- **E. The person on Forgejo's web is the box's** (6). The process
  tier's landing keeps the person temper's support makes today; signing
  in on Forgejo's web, changing the password and making a read-only
  token come with the QA loop (`later.md`). A veto needs skein's
  throwaway Forgejo to sign in on the web (a session cookie, the form
  and its CSRF token: a skein plan and a repin) before temper 10.
  Decision: accepted, by director, 2026-10-19 (the web sign-in comes
  with the QA loop).
- **F. One landing in the process tier** (7). The native client's
  landing stays the tier's one landing on real git and CI; the tools'
  landing is the real loop's (temper 13), and the tier's cut act stops
  at the adoption. A veto adds a second landing to the tier, about 86 s
  alone, past its 3 min aim.
  Decision: accepted, by director, 2026-10-19.
- **G. Another party's kept request waits for area 3** (9).
  Decision: accepted, by director, 2026-10-19.
- **H. A wait across an engine restart waits for the faults row** (10).
  Decision: accepted, by director, 2026-10-19.
- **I. `hinted` set short in the stories** (11).
  Decision: accepted, by director, 2026-10-19.
- **J. Sign-out's zero-byte `token`** (14). A veto makes sign-out erase
  the file, a change to jig's native shell and a repin.
  Decision: accepted, by director, 2026-10-19.
- **K. The fuzzy walking story sharded until the state digest lands** (12):
  temper 14 spreads its cost over eight shards, each naming the cause,
  and the digest, the next plan, cures it.
  Decision: accepted, by director, 2026-10-19 (decision 12).
