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

104. **temper 08.2: ServiceHost admitted with preparation's envelope** (plan defect, temper 08's block of 2026-10-19T14:01:41Z).
     Plan defect: the Choice admits `ServiceHost` before the caller and prepares it once the referee hands it the peer's address, but the checked world records a root's `worst_case` right after its spawn closure and collects its `operations` before the first turn, and no public builder gives the service's envelope before preparation (`startup_worst_case` is `pub(crate)`, `startup_operations` private). The envelope is preparation's own, taken before the address, since the documents' limits do not name it.

     - `crates/temper` gains `pub fn envelope_in(directory: &Path, config: &[u8]) -> Result<(u64, u32), String>`: `prepare` of `config` in `directory`, as `prepare_in` runs it, then the service's `worst_case()` and `operations()`, the service dropped and every descriptor `prepare` opened closed. Its unit test in `src/tests.rs`: equal to a prepared service's on the same documents, nothing left open. This is the one edit to `crates/temper` (the Rules' exception), beside 05.6's in the Hotspots.
     - `ServiceHost`'s spawn closure writes the documents with the held forge listener's address standing in for the peer's, takes `envelope_in`, and its `Host::worst_case` and `operations` return those two throughout. Once handed the peer's address, it writes the documents again and calls `prepare_in`, as the Shape says, and asserts the prepared service's `worst_case()` and `operations()` equal the admitted ones.
     - Nothing typed, no check switched off, no upstream edit, no copy of startup. If the two differ, if `prepare` refuses the stand-in address, or if dropping a prepared service leaves a descriptor open, raise it with the values or the descriptor. Refresh any further copy of the admission envelope, and list it in the commit body.

105. **temper 10's Shapes against main** (ready check of temper-10.1).
     Ready check of temper 10 against temper 60e76617d (none of 10.1's or 10.2's Files moved since the claim, ed725ce57). Its corrections:

     - 10.1's Shape: `Op::Wait { reap: false }`, then `Op::Wait { reap: true }`, then `Close` of the pidfd (skein-io `kernel::Op::Wait`, "Observes a child exit, or reaps it after observation"); `Spawn.root` the scratch's root, declared among `Calls`' descriptors, with `dir` empty, since `dir` is relative to `root`; `OUTPUT` is a `const` of `ctl.rs`.
     - 10.2's reference test: the kept line is `{"operation": "adopt", "outcome": adopted}`, the act's whole JSON (`crates/temper-client-tools/src/forge.rs`, `print`), not `adopted["outcome"]`, which is null.
     - 05.5's Files: the goldens and the help test are `crates/temper-client-tools/src/tests/golden/{help,help_pending}.txt` and `src/tests/help.rs`.

     The check compiled the corrected `ctl.rs` fixture and the five tests under clippy (clean) and ran `rdeps(temper)`: 59 passed. None of them ran against the shipped binary: `Start`, `Until` and `Cut` need temper 05. 10.1 and forge kit's 02.1 both edit `Process`, `ChatJudge::observe` and `process.rs`: the second to land rebases. Refresh any further copy of these lines, and list it in the commit body.

106. **temper 09's Shapes against main** (ready check of temper-09.1).
     Ready check of temper 09 against temper 28aef8f70 (08.1 landed as 2065dd47f; rulings 102 and 103 touch only 05 and 06): no name of 09 mismatches landed code. Its corrections:

     - 09.4's Shape: `Unread`'s `Event` match names every variant (`wildcard_enum_match_arm` is denied), and its `Debug` is written by hand, since `jig_client_shell_native::Shell` has none (`missing_debug_implementations` is denied); the key's name is `temper_client_tools::jig::key::Name`.
     - 09.4's Files: `temper-client-tools` comes with 09.2, `skein-io` and `skein-lib` with 08.2.
     - 09.2's Files gain `tests/real-loop/src/fixtures.rs`: D2's parent is `fixtures::write`'s, as D1's; if 08.2's `write` makes D1's alone, 09.2 makes D2's there.

     The check compiled 09.4's `unread::record()` and `Unread` sketch and 09.1's landed names under clippy (clean) and ran `rdeps(temper-real-loop-world)`: 3 passed. 09's stories cannot compile before 08.2 to 08.4 and temper 05. Refresh any further copy of these names, and list it in the commit body.

107. **temper 06.1: the tools root built unlinked** (plan defect, temper 06's block of 2026-10-19T14:05:56Z).
     Plan defect: 06.1's Shape builds the agent by `root::Root::new(&agent::limits(), seed)`, and Choice 1 steps `StartUnlinked { kept }`, then `SignIn { with: With::Transport }`; but `Root::new` is `with_client(Client::new(..))`, whose client holds a link, and jig-client refuses `StartUnlinked` on it ("Up and StartUnlinked require an unlinked client"). The proposal is adopted.

     - `crates/temper-client-tools/src/root.rs` gains `pub fn unlinked(limits: &Limits, seed: u64) -> Result<Self, jig_client::Unusable>`, `Root::new` on `Client::unlinked(&limits.domain.client, seed)`, with `new`'s `# Errors`. `with_client` stays `pub(crate)`; the root gains nothing else (06.1's Don't: no `worst_case`).
     - `agent::new` calls `Root::unlinked`. The tools' unit fixture (`src/tests/fixtures.rs`, `Root::with_client(Client::unlinked(..))`) may take it; refresh any further copy of that construction, and list it in the commit body.
     - No other session edits `root.rs`; no upstream change and no pin moves.

108. **temper 06.1: the agent's tree shows its link** (plan defect, temper 06's block of 2026-10-19T14:17:55Z).
     Plan defect, the proposal adopted. The Rules say "jig's check and Made once hold through `Verdict::Passed`", and jig's referee rearms `Online` for every client and meets it, and `Offline`, only from `Seen::Link`, which `Temper::seen` reads from a `Status` named `Live` or `Offline` in the region at `view::frame::link_region()`. Choice 1's empty tree could never meet it. No design text speaks of the agent's tree: it is the harness's.

     - **The agent's tree is the link alone.** `View::Agent { render, limits }`: `render` is `skein_ui::View::new(&limits)`, `limits` the person preset's tree bound (ruling 103). Its tree: `Page` > `Banner` > `Region` at `view::frame::link_region()` > one `Status` named `Live` for `Link::Live`, `Offline` for `Link::Offline`, none for the other states; then an empty `Main`, so that `skein_ui::outline::check` holds (a `Page` root and one `Main`). `build` rebuilds it when that state changes (`render.stale` of a count per state, `render.finish` emitting the patches), from `root.client().link().state`; `tree` is `render.tree()`. The two names are consts of `app.rs` that `Temper::seen` matches too. No bindings; `act` returns `None`; `take` does nothing.
     - **`Until::Steps(2)`, not `Steps(1)`,** in `fixtures::answered`, `fixtures::give` and 06.5's `fixtures::cut`: a `Steps(1)` run stops before world time moves (jig's `World::run`), so repeated runs never reach a callback. `ANSWER_STEPS` stays the loop's guard.
     - **Size:** ~285 lines of `tests/client/src`, the link tree's ~25 added.
     - If the link tree breaks the heap check, or `Online` is still unmet with it, raise it with the peak or the referee's line. Refresh any further copy of the empty agent tree or of `Steps(1)` in these fixtures, and list it in the commit body.

109. **temper 08.2: lands while the 4 s aggregate goes to the designer** (design gap, interim, temper 08's block of 2026-10-19T14:19:53Z).
     Interim, a design gap: the question has gone to the designer (`question.md`), whether testing.md 6.1's "the worlds below the domain … together at most 4 s of the default suite" binds this plan, now that `main` misses it before 08.2, and which steps aim at it.

     - **08.2 lands now.** Its story is 0.072 s (median of three), within its 1 s guide ("a one-process or real-loop story at most 1 s"), and the aggregate is missed without it (4.973 s of 5.059 s), so no answer would move or cut it. 08's Question ("raised … before any merge") is met by this raise.
     - **Every later step goes on** within its own guide and Proof bound, and gives its measures as its session file says. A step that would move a test to the fuzzy suite, or make an existing one cheaper, for the aggregate's sake, waits for the answer: that is the answer's to name.
     - Size: 08.2's 458 lines against ~300 land without asking (development.md, 8.6's rule: within its Files and ruling 104's edit).

110. **temper 06.1: a budget to find why Online is unmet** (plan defect, temper 06's block of 2026-10-19T14:25:21Z).
     Plan defect: ruling 108 told 06.1 to raise an unmet `Online` rather than find its cause, and implementer.md, 8.7 allows a failing test "at most three times for it, or within the budget a step to find a cause names": 06.1's three runs went to the `Steps(1)` stall. The same block twice (`Online(ClientId(0))` unmet, 14:17:55Z and 14:25:21Z) becomes the README's rule "A world failure's cause, within a budget": eight more runs and 40 minutes per unnamed symptom, in its world through `heavy`, diagnostics removed before the commit; a fix within the step's Files and rulings needs no question.

     - **06.1 finds why the agent's link is not shown `Live` by 0.5 s,** within that budget, from now. Where to look first, in this order: whether `root.client().link().state` ever leaves `SignedOut`/`Starting` after `StartUnlinked` and `SignIn { with: With::Transport }` (the person's arm also steps `domain::Event::Start` after them, the agent's `Event::Start` only those two); whether the agent's requests reach the `Joint` as the person's do (`duty` of its `Request::Client`, the front's bind for its `ClientId` and party); whether `build` emits patches when the state changes (the world shows a tree only then, after `take`, in each callback).
     - A fix within 06.1's Files and rulings 103, 107 and 108 lands without a question. Past the budget, or for a fix outside them (jig, the tools' crate beyond `Root::unlinked`), raise it with the cause found, or each run's finding.

111. **temper 08.3: sign-in reads FILE as given** (plan defect, temper 08's block of 2026-10-19T14:35:28Z).
     Plan defect: tools.md 8.1 says `sign-in` reads the link "from FILE", and a FILE given to a command is the path as given, absolute or relative to the working directory; 05.3's Shape read it by the directory's `FileIo`, whose only root is DIRECTORY's parent, and the kernel resolves every path beneath its root, refusing an absolute one with `Escape`. The fix is session 05's, as the block proposes.

     - **A new step, `temper-05.s1`,** `temper/05.s1-file-root.md`: `arguments::file_root` names the directory of the FILE a call reads; the entry point opens it, as `Startup.file_parent`; the host adopts it as a second root of the directory's `FileIo` and loads FILE's leaf beneath it. It runs in session 05 after 05.4; its proof is a unit test, and an absolute FILE on the real kernel is 08.3's sign-in story.
     - **08.3 waits for it,** its Shape kept: `sign-in <scratch>/link`, absolute. Its world gives a call that names a FILE a second startup root, `file_parent`, from `arguments::file_root`; its README Needs and order name the step.
     - **05.6 reads `--call FILE` the same way,** beneath `file_parent`, and `ctl::run` opens it by `skein_shell::open_root`.
     - Any further place that builds `Startup` or loads a FILE argument beneath DIRECTORY's parent takes `file_parent` the same way: list it in the commit body.

112. **temper 08.2: the worlds' sum is the worlds' guide, and the one-process world's cost a step** (design answer, temper 08's block of 2026-10-19T14:19:53Z).
     Design answer, temper 2f97ab8bb (testing.md 6.1; the plan's README, Rules and Done when 5; 06, 08, 09 and 13's Questions; later.md; decisions.md, For the user L). Point 5's step is `temper-13.s1` (`temper/13.s1-one-process-cost.md`), its Files the README's Hotspots exception; it measures by instrumentation, since `perf` is not installed and `kernel.perf_event_paranoid` is 3.


     1. **`main` does not miss the 4 s.** 6.1's guides sit "within" the suites' rules, which are "wall time on the development machine, run idle" (testing-strategy.md 8). The 5.059 s was measured in the build lane, beside another task. I measured idle through `heavy --gate`, at `94df159d3` (08.2 landed), with four runs after one build. The runs took 4.189, 3.972, 3.845 and 3.908 s; the median of the last three is **3.91 s**. Per world: client 0.92–0.99 s (not 2.0), one-process 1.54–1.67, forge protocol 0.61–0.67, reconcile 0.26–0.30, terminal 0.16, git 0.15–0.18, real loop 0.09–0.10. No test is past its own guide.
        `heavy --gate -- sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3 4; do cargo nextest run -p temper-client-shell-terminal -p temper-client-world -p temper-engine-protocol-forge-world -p temper-forge-setup-reconcile-world -p temper-one-process-world -p temper-real-loop-world -p temper-worker-protocol-git-world --profile measure -j 1 >below-$i.log 2>&1; done'`
     2. **testing.md 6.1 now says how the sum is measured and whose it is:**
        - "each test's own time, run alone on the idle development machine … with `--profile measure -j 1` through `heavy --gate`, the median of three runs. Beside another task, it measures that task."
        - "The sum is the worlds' guide, never a landing's. A test within its own guide lands. A plan that adds to these worlds gives the sum at its base and at its end, with the real loop's part apart. When the worlds together pass the sum, their slowest tests are profiled, then made cheaper at the cause the profile shows or moved to the fuzzy suite, by steps a plan names."
        - Reconcile's world joins the list (section 1 places it below the domain, and 08.2 counted it).
     3. **The sum binds this plan as a guide, and the plan's stories will pass it.** The headroom is 0.09 s, and the plan adds sixteen real-loop and fourteen client stories (README, Pulse). The real loop joined the 4 s on 2026-10-18 (`a1c9f50b8`), and the 4 s stayed. Counting the real loop apart would raise a guide, and "Raising a budget is an explicit decision, not a fix" (testing-strategy.md 8). So it is the user's: decisions.md, For the user L, now with the director. I recommend (b): the real loop is counted apart, its sum is each plan's Pulse aim as the user accepts it (for this plan, C), and the other worlds keep 4 s.
     4. **Until L is decided** (README, Rules, "The worlds' sum"; 06, 08, 09 and 13's Questions):
        - Each step lands within its own tests' guides.
        - No test is moved, cut or merged for the sum.
        - Done when 5 measures the sum at the end, with the real loop apart and the three slowest tests named.
        - later.md has the row.
        - Ruling 109's hold on "a step that would … make an existing one cheaper, for the aggregate's sake" is lifted for point 5.
     5. **The step that aims at the sum: a supplement step, which the supervisor names, before 13.4.**
        - Why the one-process world: it is 40% of the sum. `an_adopted_repository_gets_its_change_pushed_through_git` takes 0.87 s and two stories take 0.24 s each. On skein's simulator that time is CPU, so its cause is in the code or the test. Its 0.95 s came from larger chunks (eb8abb6aa: 3.3 s → 0.95 s), which cut iterations; no profile was ever run.
        - Why before 13.4: 13.4 and 13.5 drive the same adoption through fake git on the real loop ("near the real-loop story's 1 s aim", findings/base-times.md).
        - Scope: profile the three stories (frame pointers are on; `perf record -g` on the unoptimized test binary). Fix the cause in the code or the test, never in a build setting. Give each story's time before and after, idle.
        - Where to look first, from a reading (no profile yet):
          - `Outcome.iterations` and `procs.len()` per story: finished git, sh and agent processes stay in `procs`, and `Referee::observe` (`tests/one-process/src/referee.rs:310`) runs after each turn, O(processes).
          - The referee's per-iteration scans and per-commit `rows.clone_from` and `snapshot`.
          - Fake git's per-invocation `Checkout` rebuild and tree clones (`testing/temper-fake-git/src/answer.rs`, `program.rs`).
        - Its Files cross 08's "nothing of `tests/one-process`" and 13's "No edit to … `testing/temper-fake-git`". That exception is the supervisor's ruling.
        - No step for the client world: 0.95 s for 37 tests is about 26 ms each.

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
- **L. How testing.md 6.1 counts the real loop** (the designer's answer
  of 2026-10-19). 6.1 counts the real-loop world among the worlds below
  the domain, together at most 4 s (the real loop joined that sum on
  2026-10-18, and the 4 s stayed); they take 3.91 s at the base,
  measured idle, and this plan adds
  sixteen real-loop and fourteen client stories (C). The choices: (a)
  keep the 4 s with the real loop in, and make cheaper or move to fuzzy
  what passes it; (b) count the real loop apart: the other worlds keep
  4 s, and the real loop's sum is each plan's Pulse aim, accepted by the
  user (for this plan C's 16 s, about 8 s expected), each story at most
  1 s; (c) no sum: the per-test guides and the 15 s rule alone. The
  designer recommends (b). Until it is decided: measured, never moved,
  cut or merged for the sum (README, Rules, "The worlds' sum").
  Decision: pending, with the director.
