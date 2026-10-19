# forge-kit: decisions

The decisions this plan rests on, numbered. The supervisor's rulings
during the run follow them, from 101. Then the design's questions with
their answers, and the choices left to the user. The user is away, and
the director acts for them. The planner took each decision below as the
planner's, choosing the most reversible option where the choice could
be the user's (development.md, 10.4).

## The plan's own

1. **One crate per skein session, upstream first.** temper's design
   (the forge kit's side, section 5) orders the work as skein-forge, the
   client, the component, reconcile, then the fakes and the conformance.
   The plan follows that order, and the last item takes five sessions:
   skein 05 the fakes, skein 06 the throwaway, the conformance and the
   dialect's document tests, skein 07 the client's and setup's worlds,
   and skein 08 and 09 the component's worlds. Each skein session up to 06 is
   followed by the temper session of its number, which repins and
   deletes temper's copy (development.md 5.7); temper 07 deletes the
   worlds that skein 07 to 09 moved. The component's worlds read the
   conformance's transcripts and deliveries and drive the fake forge's
   delivery, so they come after skein 05 and 06 (decision 23).

2. **A copy follows temper's code onto skein's.** A skein step copies
   temper's code from temper's `origin/main` at the step's start, and
   only once that code already uses the skein crates moved before it.
   For example, skein 02.1 copies the client after temper 01.1 has put
   it on skein-forge's vocabulary. The commit body names the temper
   commit it copied from, and temper deletes its own copy in the temper
   step that follows. So no part has two live copies that drift, and
   the moved code is the code that ran.

3. **A move changes only what its new home forces.** That is: temper's
   names out, handed by the owner; a downstream crate's type made plain
   values; serde made skein-json; `skein_io::digest` made
   `skein_lib::sha256`. Each such change is named in the move's Shape. A
   difference between the code and the design is a later increment's,
   with its own test: skein 10.4 and 10.5, 02.2, 03.2 and 04.2.

4. **`api.rs` splits across two crates by its move.** The vocabulary
   (lines 1 to about 417) goes to skein-forge (forge.md 3);
   `answer_bytes_unbounded` stays the client's until the measure
   replaces it in skein 02.2 (decision 24). The answer check (`accepts`) goes to the
   client (forge-client 2.3). `locate::bytes` and `locate::accepts`
   become `pub`, so that the client can reach them.

5. **The effect key is bytes.** skein's client has
   `effect_key(namespace, identity, max)` over opaque bytes.
   `EffectPurpose` and its tags stay temper's connector's (forge.md 2.3).
   temper 02.1 pins golden vectors (tags 1 to 5, the call key's 44 bytes
   and the history key's 45), so that no key at rest changes.

6. **All the call plans move in one increment, skein 01.3.** The
   plans call each other: reads, writes, contents, actions, plan,
   failure, tables, revision and `hooks/hint`. `Heard` and `Body` move
   out of the component's `connection.rs` into the dialect with them.

7. **`Host` is the dialect's; the user agent is the owner's.** By the
   designer's answer 1, skein-forge's pure `host(address, name)` makes
   `Host` for the component and setup alike, and moves with the start's
   reads in skein 01.4; a plaintext transport carries its forge's name.
   The rest of `conventions` (the user agent) becomes the owner's value
   in skein 03.1. The start's two reads become dialect plans in skein
   01.4: this is the design's first code gap. The head check counts the
   user agent in skein 03.2: the second.

8. **The 68 plan tests move with the component (skein 03.1).** They
   drive a `Component` through a harness. skein 10 writes the plans'
   pure `begin`/`heard` tests in skein-forge.

9. **The fake forge's memory tests become the domain crate's own
   `tests/` (skein 05.1).** `tests/forge/machine` is the component's
   world (skein 08.1), not the fake's.

10. **The dialect's document tests go in skein-forge's `tests/` (skein
    06.3).** They read the transcripts that the conformance carries into
    `tests/forge/conformance` (skein 06.2). This covers the codecs
    world's `forgejo_*` tests, 55 focused and 3 fuzzy, which are not
    serde. The rest of the codecs world stays temper's.

11. **The over-ceiling tests move as they are.** The protocol world's
    four `fuzzy_machine` tests take 7.5 to 7.7 s each at the base
    (`findings/base-runs.md`). That is over the 5 s fuzzy aim, inside the
    60 s budget. They move unchanged to `tests/forge/machine` in skein
    08.1. Its report names what each test costs and why (testing-strategy
    8: understood, never hidden). Making them cheaper would rewrite a
    moved part, so it is not this plan's.

12. **serde becomes skein-json** in the throwaway and the conformance,
    about 15 files (skein's dependency allowlist), as their moves
    require. The throwaway's HTTP goes onto skein-http under `drive`.

13. **The repin is temper's alone.** Each temper increment that needs
    a new skein step repins in its first commit, and repairs what skein's
    steps since temper's last repin broke (README, Rules); the closing
    `temper-repin` follows the last skein step (development.md, 5.7).
    The tools plan's first repin (jig and temper onto skein's and smith's
    `main`) is assumed landed before temper 01.1; if it has not, temper
    01.1 waits for it (README, "The tools plan"). smith, jig and hearth
    take skein at their next repins, as forge-setup's decision 3 did:
    skein's changes are new crates and modules, which break none of them
    (`later.md`).

14. **The owner's word.** skein's kit takes the owner's word
    (`[a-z0-9-]`, 1 to 16 bytes) for the marker, and the owner's user
    agent and namespace. skein's tests use `kit`. temper hands `temper`,
    and temper tests pin the bytes (`<!-- temper:key <hex> -->\n`,
    `temper/0.0.0`), so that nothing temper writes on the wire or at
    rest changes.

15. **The fake forge peer replaces temper's `ForgeHost` (skein 05.4 and
    05.5, temper 05.2).** The one-process world hands the peer's `address()`
    to the engine, where it had 127.0.0.1:3000 fixed. In the simulated
    world, whose engine is configured at spawn and whose restarted peer
    rebinds the engine's address, both are given one constant,
    `fixtures::FORGE`, and the referee asserts `address()` is `Some(FORGE)`
    once the peer listens; the kernel-assigned address is handed over in
    the real loop (temper's testing.md 4.2).

16. **The conformance's masks are made whole, in two steps.** The
    actions transcripts hold four stale UUID occurrences and 9 unquoted
    log times. skein 06.2, the move, records once on the pinned binaries
    with the base's mask, whose UUID rule clears the four; skein 06.4
    changes the mask for the unquoted times, records once more, and its
    test, `recorded_corpus_has_no_unmasked_uuid_or_log_time`, checks
    that no unmasked UUID or log time remains. No transcript is edited
    by hand.

17. **No live service.** The Forgejo and runner binaries are the pinned
    local ones, on loopback, as temper's process tier runs them today.

18. **The suites' aims for the whole run.** At the base
    (`findings/base-runs.md`), skein's default suite may grow by at most
    1.5 s and its fuzzy suite by at most 6 s. temper's suites take no
    longer than at the base, package by package, for what stays. A
    moved test keeps its name, budget and time. The aims are the
    suites' wall times, as the README's Done when 8 measures them; a
    session's sum of its tests' times is not that, and is reported, not
    held to them.

19. **The over-800 files split as they next change.** `keep.rs` (1377
    lines) and `outbox.rs` (1079) split in skein 02.2, which changes
    them. `api.rs` (850) splits by its move (4).

20. **The hotspots with the tools plan** are in the README, Rules. A
    real-loop story stays temper's, on temper's forge host, until temper
    05.2. A change to a fake forge crate lands where the crate lives on
    `main`; between skein 05.1 and temper 05.1, such a change waits for
    temper 05.1's repin.

21. **The design's answers are in the plan before it lands.** The
    session files were drafted with each recommendation below in place.
    Each answer is patched into them, and each [shape] answer before
    any draft is reviewed.

22. **A world's test of temper's own policy stays temper's.** Of
   `tests/engine/protocol-forge`'s remainder, the tests that drive
   temper's connector, change and issue engines test what forge.md 2.3
   leaves temper's, and skein never depends on temper. They stay in
   temper's world, on skein's component and fakes, after temper 07; only
   the tests of the component and the fakes move (skein 09.1).

23. **The component's worlds are two sessions of their own, skein 08
   and 09.** The machine and listener world (`tests/forge/machine`)
   reads the conformance's transcripts (`case_bytes`) and deliveries,
   and its hooks part imports the fake protocol layer's `delivery`, so
   its copy needs temper 05.1 and skein 06.2 (the copy rule, decision
   2). It moves after them, with its new tests, in skein 08. The
   protocol world's remainder, on skein's client and the machine
   world's joint, is skein 09. skein 03 ends at the component's own
   changes; skein 07 keeps the client's and setup's worlds. One session
   for both worlds would pass five increments, and its increments about
   300 new lines, tests counted (code-organization.md 6).

24. **The measure is one increment, skein 02.2.** forge.md 3.7's
   measure is one rule: the dialect's `measure::op` and `measure::answer`
   in skein-forge, and the client's `measure::stored`, `measure::effect`
   and the rows check beside them, with the widths asserted at build. It
   lands whole in skein 02.2, which changes skein-forge as well as the
   client, rather than as a sixth increment of skein 01; it retires
   `answer_bytes` and `answer_bytes_unbounded`, and the plans' running
   measure (skein 01.3's) takes its widths. Its "temper's side" names
   temper's callers, which the next temper repin repairs. skein 03.3's
   page size needs it.

25. **The Host's port follows answer 1 as written.** skein-forge's
   `host(address, name)` leaves the port off when it is 443 or 80,
   whatever the transport. The base chose the default by whether a TLS
   server name was given (`conventions.rs:56`), which a named plaintext
   forge would break, and the new signature carries no transport. So a
   TLS forge on port 80 and a plaintext forge on port 443 now send no
   port; skein 01.4 tests both edges. The alternative, a `host` that
   takes the transport and drops only its own default port, is a
   designer's change.

26. **The forge peer is built from the base host (skein 05.4, 05.5).**
    The tools plan as landed (temper `1093544c3`, area 1's first part)
    does not move temper's forge host: the host's move to `testing/`, its
    webhook delivery and the real-loop world are its next part, unplanned
    (its `later.md`, drafts in its `findings/next/`). So skein 05.4 copies
    the base host, `tests/one-process/src/forge.rs` (268 lines, no
    delivery), with fake git's shared handle, and a new skein 05.5 builds
    delivery in skein's peer (about 520 new lines in the two, from
    `findings/base-skein-05.md`); temper 05.2, which needs 05.5, deletes
    the host. The next part, planned after this, hosts its real loop on
    skein's forge peer (testing.md 4.2) and drops its draft 07's host
    move. The alternative, waiting for that host before skein 05.4, would
    hold this plan's last steps (skein 05.4, temper 05.2, temper 07) on a
    plan not yet written.

27. **The conformance leaves serde in temper, before skein copies it
    (temper 06.2 to 06.4).** Its 17 source files use serde_json: about
    700 production lines change, 350 of them reads rewritten in place
    (`findings/base-temper-03.md`). skein has no serde, so a skein copy
    would replace it in the move, and skein 06.2 would then pass 300 new
    lines. By the copy rule (decision 2), temper 06.2 to 06.4 put the
    conformance on skein-json's `Document` and `writer::Encoder` first
    (the module and the byte leaves, the reads, the writes), each proved
    by its unit tests and the stable-fact check, and skein 06.2 copies
    it renamed. They need nothing outside temper and follow temper
    06.1's repoint onto skein's throwaway, so they run beside temper 03,
    off the critical path. The reads stay one increment: a
    half-converted family would hold two trees. The
    throwaway's five serde lines and its 121-line HTTP client stay skein
    06.1's, which is about 200 new lines with them.

28. **skein-forge's plan tests and the settled differences are a
    session of their own, skein 10.** In skein 01 they were one
    increment of about 1,100 new lines (220 production, 880 tests),
    with five behaviour changes in it. skein 10 splits them: the
    harness and the read calls (10.1), the write calls and the failure
    order (10.2), deliveries, markers and the statuses the base already
    has (10.3), then answers 7, 9 and 12 (10.4) and 8 and 10 (10.5).
    Nothing waits for it but the run's end, so it is off the critical
    path. 10.5 needs skein 08.1: the Items-without-Date change breaks a
    machine test whose scripted Items head has no Date, and skein 08.1
    has moved that test to skein, where 10.5 repairs it; temper's next
    repin repairs temper's copy while temper still has it.

## Design questions

The inbox's questions (`~/src/tmp/director/inbox/forge-kit-planner.md`,
2026-10-19), each with the recommendation the drafts carry and the step
where its answer lands.

| # | Question | Recommended | Lands in | Answer |
|---|---|---|---|---|
| 1 | [shape] who builds `Host` | the owner hands it, the component derives it by default | skein 01.4, 03.1, 03.2 | answered: skein-forge's pure `host(address, name)`; a plaintext transport carries its name; the owner hands no `Host` |
| 2 | control bytes in the token or user agent | start refuses them, a new `Unusable` | skein 03.2 | answered: as recommended, with the existing `Unusable`: a credential or user agent that is empty or holds a byte below a space other than tab, or DEL, and a user id of 0 |
| 3 | the signature header's shape | not 64 lowercase hex digits is no signature, 403 from the head | skein 03.2 | answered: as recommended (forge-connection.md 6.3) |
| 4 | 6.3's "a `4xx`" for 501 and 505 | "the server's own answer" | design only | answered: "skein-http's own answer: a 4xx, or 501 or 505" |
| 5 | the component's token space | stated in 2.1 | design only | answered: tokens below 2^60; API connections from 1, the listener at 2^59, hook connections above |
| 6 | `Retry-After` at start; no `X-Forgejo-Event` | honoured as 3.3 says; 204 stated | skein 03.2 | answered: honoured however long (the rate holds, the sum saturates, the calls end at their deadlines, Unavailable); a verified delivery without the event header is 204, no hint |
| 7 | a write's list past `inner` | `Timeout` | skein 10.4 | answered: TooLarge on a read, Timeout on a write; the client's own check of a write's answer stays InvalidAnswer; a body past `body_bytes_in`, the job log's read-through included, stays malformed (forge.md 5.3) |
| 8 | which `404`/`3xx` drop the path | every one, stated | skein 10.5 | answered, B: a 404 that ends the call (MissingJob included) drops the path; one 5.2 calls an answer keeps it |
| 9 | `File` past the room left | cuts like `Job` | skein 10.4 | answered: File and Job cut at the room left, `truncated`; a side of PullFiles or Compare is never cut |
| 10 | absent answer headers | a missing `Date` on `Items` is malformed | skein 10.5 | answered: Items without Date is malformed; a 429 without Retry-After is `RateLimited { after: 0 }`; a listing without X-HasMore has more only when full |
| 11 | success statuses | a status column in 5.2 | skein 10.3 | answered: a Success note in 5.2; any other 2xx is malformed |
| 12 | `repositories/{id}` answering another id | `InvalidAnswer` for reads too | skein 10.4 | answered: in 5.3's malformed list |
| 13 | the page size's row measure | the client's answer measure, with 14 | skein 03.3 | answered with 14 |
| 14 | [shape] the byte measure | fixed widths in 2.3; the three functions named | skein 02.2, 03.3 | answered: one measure, the vocabulary's (forge.md 3.7), fixed widths; `measure::op`, `measure::answer` in skein-forge, `measure::stored`, `measure::effect` in the client |
| 15 | which of two refusals wins | the code's order, written down | skein 02.2 | answered: the code's order, numbered for Keep (3), Read (5) and Make (7.2); a test per pair (forge-client.md 10) |
| 16 | owner misuse | a Preconditions list, asserted | design only | answered: a Preconditions bullet in 2 |
| 17 | [shape] Restore before a namespace | `bind_deployment` returns false | skein 02.2 | answered: false, changing nothing, once any `Restore`, `Keep`, `Make` or `Remake` was taken |
| 18 | a rate-refused write sent again | each send is an attempt | skein 02.3 | answered: each send is an attempt, its deadline from the send |
| 19 | an issue's provenance | issues settle by writer and key | design only | answered: comments and reviews need provenance; an issue settles by its key, alone |
| 20 | an answered `Busy` | `InvalidAnswer` | skein 02.2 | answered: InvalidAnswer |
| 21 | a withdraw that does nothing | stated as a no-op | skein 02.3 | answered: silent |
| 22 | after a failed restore | terminal; the owner builds a new client | skein 02.3, 07.1 | answered: `restoration_failed()` is final, Keep is Busy from then on, the owner builds a new client; "every table full" is a world story |
| 23 | [shape] the given values | a `Place` enum; one `Option<owner>` | skein 04.1 | answered: a `Place` per given value and limit (an org by index); `Option<owner>`, None stops with `OrgMissing` |
| 24 | empty `orgs` | the check refuses zero | skein 04.2 | answered: refuses zero orgs |
| 25 | the bot's marker | a boundary match | skein 04.2 | answered: the mark matched whole, both sides |
| 26 | `BotIsAdmin` and `Unmarked` both | `Unmarked` first | skein 04.2 | answered: Unmarked first, tested |
| 27 | empty items | empty means none for every item | skein 04.2 | answered: empty is none, for every item |
| 28 | a kept token answered 403 | remade like a 401 | skein 04.2 | answered: 401 or 403 remakes; other statuses and Down stop |
| 29 | temper's setup.md | the mapping test in the shell; the edge; `must_change_password` | temper 04.1 | answered, B: temper-forge-setup's step code maps the setup and runs the kit's check for every command, temper serve's too, no shell; a refused Place named by setup.md 5's path |
| 30 | [shape] `Instance::attach` | moves with the admin's login as an argument | skein 06.1 | answered: `attach(child, port, scratch, admin_token, &Settings)` reads the login and id from `GET /api/v1/user` |
| 31 | [shape] the binaries' paths | `--forgejo`, `--runner`; `SKEIN_FORGEJO(_RUNNER)` in tests only | skein 06.1, 06.2 | answered: the conformance's `--forgejo`, `--scratch`, `--out`, optional `--runner`; the kit's ignored tests read `SKEIN_FORGEJO(_RUNNER)` |
| 32 | [shape] where the peer delivers | `host` is the loopback `ip:port` | skein 05.5, temper 05.2 | answered: as recommended; the owner of a port-0 listener sets the endpoint once known; no address, dropped and observed |
| 33 | fake-forge.md 4's small ones | each stated | skein 05.2, 05.5 | answered: each stated (fake-forge.md 4): every review renders pull_request_approved; `new` refuses a none worst_case; a write's answer makes one more read on the same request; the 8 rendered named |
| 34 | forge.md 4.2: a plan's end and its cost | the component counts the cost, as forge-connection.md and the base do; 4.2's sentence amended | skein 01.3 | answered: as recommended; the component counts one per request whose first byte it wrote; `Done` carries only the result; forge.md 4.2 says the plan counts nothing, and the plans' step tests drop the cost |
| 35 | a 429 on Merge's read after the merge | `Timeout`, as forge.md 5.4 now says (the base's `failure::classify` says RateLimited) | skein 10.3 | answered: the designer's open item in its second answers |
| 36 | forge-connection.md 10: the requests the fake served against the costs reported, when a cut falls inside a request head | A: the requests whose first byte reached the fake equal the costs, counted at the world's pipe; the fake's served count at most that | skein 09.4 | answered: A, made exact: in every run the costs equal the requests whose first byte the component wrote, counted at the world's streams, and the fake served those less the ones cut inside their head |

## Rulings

101. **Start beside tools; retain its first-repin dependency** (director's order at the claim).
     Reverses: nothing.
     Amends: README, Rules of this plan, tools-plan ordering and temporary graph rule; temper/01-forge.md, Needs at start.

     The director authorizes this run to start before tools temper 11 lands.
     The README says that tools temper 11's forge-facing flags “touch none of this
     plan's files”; remove its contrary statement that the director waits for 11.
     Temper 01.1 here still starts only after `tools/temper-01.1` reaches the shared
     temper `origin/main`. The coordinator registers `plan await coordinator
     tools/temper-01.1`. The tools run is on debian-16gb-hel1-1. Other hotspots
     are handled by rebasing as `land` and the implementer's card say; the named
     fake and host copy windows remain in force. Apply the same rebase rule to
     further overlapping files and name them in the landing's commit body.

     The claim's skein and temper graph indexes are older than `origin/main`.
     Until their refresh is complete and a ruling removes the README's temporary
     rule, use `rg` for these repositories.

     ## Checked

     - Ordering authority: director's message received this turn; recorded 2026-10-19T05:02:59Z.
     - `git -C temper show origin/main:docs/development/plans/tools/temper/11-tools-forge.md`: 11.1 moves client-view words to client-domain::forge::words; 11.2 is tools adopt. README's hotspot list says those files do not overlap this plan.
     - `rg -n 'plan await ROLE|plan ruling|plan ring' kit/scripts/dev/plan`: lines 17, 19, 21 document these commands; `plan await` accepts `[PLAN/]STEP`.
     - MCP index_status: srv-data-git-runner-src-skein indexed 2026-10-13T09:30:32Z; srv-data-git-runner-src-temper indexed 2026-10-11T09:45:04Z; both ready, primary roots matched.
     - `git show -s --format='%h %cI' origin/main`: skein d6d4abe 2026-10-19T05:46:18+01:00; temper f10b03ea7 2026-10-19T04:57:21Z.

102. **Refresh skein and temper graph indexes** (claim-time index refresh).
     Reverses: 2026-10-19T05:02:59Z (ruling 101's temporary graph restriction only).
     Amends: README, Rules of this plan, removes Graph refresh at the claim.

     The primary-checkout indexes are refreshed. Remove the temporary rule
     added by ruling 101 and resume the workspace's graph-first discovery, using
     `srv-data-git-runner-src-skein` for `/srv/data/git/runner/src/skein` and
     `srv-data-git-runner-src-temper` for `/srv/data/git/runner/src/temper`.
     Check graph names in each working checkout with `rg`, as workspace guidance
     says. The director's tools-plan ordering and other rules remain in force.

     ## Checked

     - MCP index_repository returned indexed, then index_status returned ready for both primary roots: skein indexed_at 2026-10-19T05:04:07Z; temper indexed_at 2026-10-19T05:04:37Z.
     - Run kit numbers.sh graph_index, with root=/srv/data/git/runner/src: `graph_index skein` -> `srv-data-git-runner-src-skein current`; `graph_index temper` -> `srv-data-git-runner-src-temper current`.
     - Completed 2026-10-19T05:05:27Z. Existing malformed JSON transcripts remain direct-read coverage gaps in skein; no temper Rust parse gaps were reported.

103. **temper 01's Shapes against main** (ready check of temper-01.1).
     Reverses: nothing.
     Amends: temper/01-forge.md, 01.3 and 01.4 Files, Shape and proof.

     Temper 01's ready check corrects the Settings and Forge boundary sketches to
     use settings.rs's existing Box and List imports. It adds docs/examples/owned.json
     and its existing startup regression to 01.3/01.4's coverage. These increments
     price newly held state in worst_case already; their complete cutovers must
     refresh the derived engine.memory_bytes using system::startup_worst_case at
     the existing startup fixture's bounds. The word-only sketch measured +32
     bytes; that is evidence of the missed derived value, not a shipped literal
     for either complete cutover. Refresh further copies and derived values found
     and name them in the commit body.

     The corrected runnable sketch passed clippy and both suites. The initial
     32-byte shortfall is resolved by this plan correction; a concurrent focused
     run's timeout cleared when run alone, within the existing budget. No suite
     remains failing in the check, and no budget or ignore changes are authorized.
     Missing skein symbols belong to the named upstream increments. The ready
     check may be released while temper 01.1 continues to wait for the director's
     tools/temper-01.1 dependency and its skein dependency.

     ## Checked

     - Check report: pump/ready-temper-01.md; starting origin/main skein 70cc62419296055c341a06f2d831d65306fb6d86, temper 10e5f18b84482c8620189e1b7200d86a18bcc42f.
     - `git -C temper show origin/main:crates/temper-engine-protocol-forge/src/settings.rs | rg -n 'use alloc::boxed::Box|use skein_lib|pub (struct Settings|forges|host)'`: Box imported at 2, List/Secret at 4, Settings at 9, forges: List at 11, host: Box at 27.
     - `git -C temper show origin/main:crates/temper-service/src/system.rs | rg -n 'fn startup_worst_case'`: 532: pub(crate) fn startup_worst_case(. Graph search independently located this function and the startup test.
     - `git -C temper show origin/main:crates/temper/tests/startup.rs | rg -n 'fn serve_prepares_once_on_the_owned_example'`: 314; same-path baseline 1 PASS, corrected fixture passes too.
     - `git -C temper show origin/main:docs/examples/owned.json | rg -n '78669036813526'`: 192, engine.memory_bytes.
     - `rg -n 'serve_prepares_once_on_the_owned_example|78669036813526|78669036813558|startup_worst_case'` over temper, skein, smith, jig, hearth sources and examples: only temper's owned.json, startup test, system.rs and startup.rs caller. The check's downstream search found no affected users.
     - Check's `heavy cargo nextest run --workspace`: sketch 69 passed/1 failed, corrected isolated 1,879 passed in 3.536 s, one skipped; corrected fuzzy 50 passed in 31.647 s. Clippy all targets passed.
     - Kit plan line 22 documents `plan ready STEP [NOTE]`; existing Proof lines already use heavy cargo nextest. Recorded 2026-10-19T05:20:20Z.

104. **Take tools jig-02.1's landed repair in temper's first repin** (cross-session scope, temper 01.1's block of 2026-10-19T05:26:26Z).
     Authorize the compatible jig repair in temper 01.1's existing first repin.
     The README says a repin “repairs in its own commit what a skein step since
     temper's last repin broke”; extend this repin to its required jig consumer.
     Tools `jig-02.1`, commit `b2eebe6ab91af9eca121588f2d44072fcbdf3b9c`, has
     already landed on jig's shared origin/main. It retains the RequestAdmission
     and passes its number to RequestTable::not_kept. Use that landed revision
     for the currently locked jig 719bd70d4, by running
     `heavy cargo update -p jig-client-domain --precise b2eebe6ab91af9eca121588f2d44072fcbdf3b9c`
     in temper's worktree, then the existing
     `heavy cargo check --workspace --all-targets` before committing the repin.
     Keep the skein update, one revision per git source, and any already newer
     compatible jig pin that main has acquired; do not move such a pin backward.
     The selected repair precedes the tools plan's later wire-version change;
     this ruling adds no dependency on its remaining steps. Refresh further
     revision entries and derived fixtures reached by the normal checks, naming
     the copies and repairs in the repin commit body. A remaining incompatible
     interface is reported with its compiler error. Vocabulary work continues.

105. **Complete the authorized jig repin's people_v4 sweep** (cross-session scope, temper 01's block of 2026-10-19T05:32:59Z).
     Complete the people_v4 successor-name sweep in temper 01.1's existing, separate repin commit. The authorized jig b2eebe6ab includes tools jig-05.1's store-family move; its people schema differs only in the family version header. The tools plan's temper 03.1 Shape already says “people_v3 becomes people_v4 in the three temper-protocol-store files; nothing in temper names tasks_v3.” Bring forward that mechanical repair and its schema comment only.

     The checked consumers are crates/temper-protocol-store/src/records/application.rs (encode/decode), src/records/application/refusal.rs (module import), src/tests/records.rs (Refusal worst-case assertion), and schema/store-v2.schema (comment), all under temper-protocol-store. Replace people_v3 with people_v4 throughout them. Preserve fields, behaviour, temper's store-v2 family, test names and assertions; use the repin's existing all-target check and normal increment proofs/gate. Keep ruling 104's precise compatible jig pin and skein update.

     The README now makes mechanical repin repairs a standing rule. Implementer 8.2 already permits “a removed or renamed name replaced by its successor” and “a version's move with its whole sweep”. Follow those landed upstream changes through source, tests, comments, copies and derived fixtures within the existing repin; no further scope block is needed for a straightforward rename. Report structural or behavioural differences beyond the landed change. Refresh any further copy and list every further user and repair in the commit body. Tools temper 03.1's remaining work keeps the director's existing hotspot/rebase ordering.

106. **skein 04's Shapes against main** (ready check of skein-04.1).
     Amends: skein/04-setup.md 04.2 Files and temper's side. Reverses: nothing.

     The ready check found a downstream fixture omitted from the strengthened org admission repair. Add temper-forge-setup-shell/tests/support/mod.rs::settings to the next temper repin's repair scope; populate its Entry::orgs with `Box::new([b"acme".as_slice().into()])`. Keep the shell stories and assertions. Refresh further copies and derived expectations reached by this repair and name them in the commit body.

     Checked skein 0db7b1fc7307aa50980ce75637e686e384adc4e4 and temper c4f89032047e4980b9f946ce8abf1c2c81653ba8. `settings` at that temper commit supplies `orgs: Box::new([])`; 04.2's new admission refuses zero orgs before any clone/need. The check searched files, symbols, copies and derived values across all five repositories and found no other mismatch. Missing future APIs/tests remain declared dependencies.

     The corrected scratch fixture passes clippy for temper-forge-setup, its shell and reconcile world; their focused suite passes 145 tests in 1.377 s and fuzzy passes 1 in 1.126 s. Skein's declaration sketch passes clippy; skein-forge plus sketch pass 86 tests, with no fuzzy/browser tests selected. Runtime Given and future 17/33/54-test proofs await their implementations. Evidence: pump/ready-skein-04.md and scratch-ready-skein-04/temper/.ready-check/fixture.patch.

107. **skein 02's Shapes against main** (ready check of skein-02.1).
     Amends: skein/02-client.md 02.4 Test shape, invalid_reads_make_no_call. Reverses: nothing.

     The invalid-read table covers six paged reads: skein-forge/src/api.rs::Read::{Items,Reviews,Statuses,Remarks,PullFiles,Collaborators}, each at page zero. Correct “seven” to “six” and name the variants in the table. Keep File's empty-path cases and File/Job byte-limit cases, existing outcome and zero-Call assertions. No consumer correction was found; refresh further copies/derived tables if encountered and list them in the commit body.

     Checked skein 0db7b1fc7307aa50980ce75637e686e384adc4e4 and temper c4f89032047e4980b9f946ce8abf1c2c81653ba8. The Read enum has exactly those six page fields; the check searched all increments and downstream repositories. Other absent names belong to declared future increments. The corrected scratch table compiles under clippy and passes within the 78-test temper client suite (0.159 s); client world passes 29 focused tests (0.041 s) and 2 fuzzy (0.329 s). Skein-forge passes clippy and 86 tests (0.148 s); client fuzzy selects no tests. Future opaque-key, measure, echo and 81/84/90/114-test proofs await their APIs. Evidence: pump/ready-skein-02.md.

108. **skein 06's Shapes against main** (ready check of skein-06.1).
     Amends: skein/06-conformance.md 06.1 Shape, 06.2 Proof, 06.3 Shape, 06.4 Proof and Done when. Reverses: nothing.

     The ready check corrects five plan details. In moved throwaway `http.rs::request`, absent bodies use skein-http `Body::None`; `Body::Length(0)` emits Content-Length and changes the existing GET golden. Present bodies retain Length(n).

     Moved skein-forge integration targets need test configuration under skein's lint policy: `#![cfg(test)]`, scoped and reasoned test-harness `#[expect]` attributes, and `common` exposed within test targets for shared helpers. Adapt redundant qualifications in `tests/common/forgejo.rs::{decode_refusal,decode_body,transcript_cases}` and `tests/forgejo_setup.rs::OrgAnswer::request` to their imports. Keep production lints and test assertions. Also replace `tests/forgejo_deliveries.rs::shipped`'s temper-service lookup with `common::forgejo::profile_limits`; the existing delivery fuzzy test uses the same helper. Refresh further copies and derived values and list them and lint adaptations in the commit body.

     The committed corpus contains 218 facts, with 55 setup facts and 40 component facts, all holding. Correct the three “36 setup” references to 55; retain the stable-outcome digest and timing comparison unchanged.

     Checked skein 0db7b1fc7307aa50980ce75637e686e384adc4e4 and temper c4f89032047e4980b9f946ce8abf1c2c81653ba8. Verified Body::{None,Length} in skein-http/src/client/request.rs, the temper profile lookup in tests/codecs/tests/forgejo_deliveries.rs::shipped, and counted setup/component prefixes in tools/forgejo-conformance/facts.txt. All-increment symbol/copy/derived-value searches across five repositories found no further consumers.

     The corrected scratch probe passes clippy for the throwaway/conformance/codecs and skein-forge/http packages. Sequential suites pass: temper 98 focused (1 skipped), 6 fuzzy; the existing opt-in throwaway test passes (13 skipped); skein forge/http/world pass 384 focused and 10 fuzzy, no browser tests selected. The initial future-mask sketch predictably fails against today's masker: 06.4 already owns that implementation; its sketch compiles, and baseline suites pass after restoring it. No new design question is exposed. Future attached fixtures, recorder/document conversion proofs, five boundary tables and generated fuzzy sweep remain unrun until their declared implementations. Evidence: pump/ready-skein-06.md and scratch-ready-skein-06/.

## For the user

- 1: the fakes, the conformance and the worlds take five sessions
  (05 to 09).
  Decision: accepted, by director, 2026-10-19.
- 2: skein copies temper's code only after temper uses skein's crates,
  so the two repositories alternate on the critical path.
  Decision: accepted, by director, 2026-10-19.
- 9 and 10: where the fake's memory tests and the dialect's document
  tests live.
  Decision: accepted, by director, 2026-10-19.
- 11: the four `fuzzy_machine` tests move at 7.5 to 7.7 s each, over
  the 5 s aim. Making them cheaper is later work.
  Decision: accepted, by director, 2026-10-19: the report names each test's cost and why; making them cheaper goes to later.md.
- 13: only temper repins. smith, jig and hearth keep their pins of
  skein until their next repin (`later.md`).
  Decision: accepted, by director, 2026-10-19.
- 16: the actions transcripts are recorded again, on the pinned
  binaries, to mask their UUIDs and times.
  Decision: accepted, by director, 2026-10-19.
- 18: the aims, +1.5 s and +6 s on skein's suites.
  Decision: accepted, by director, 2026-10-19.
- 22: the protocol world's tests of temper's connector, change and issue
  engines stay temper's.
  Decision: accepted, by director, 2026-10-19.
- 23: the component's worlds are two sessions of their own, skein 08
  and 09.
  Decision: accepted, by director, 2026-10-19.
- 24: the measure lands whole in skein 02.2, changing skein-forge too.
  Decision: accepted, by director, 2026-10-19.
- 25: the Host leaves off ports 443 and 80 whatever the transport.
  Decision: accepted, by director, 2026-10-19.
- 26: skein's forge peer is built from the base host, with its delivery
  new in skein 05.5, rather than waiting for the tools plan's next part
  to move the host; that part then builds its real loop on skein's peer.
  Decision: accepted, by director, 2026-10-19: the tools plan's next part builds its real loop on skein's forge peer.
- 27: temper 06.2 to 06.4 put the conformance on skein-json before
  skein 06.2 copies it; 06.3's reads, about 350 lines rewritten in
  place, count as moved and stay one increment.
  Decision: accepted, by director, 2026-10-19.
- 28: skein-forge's plan tests and the settled differences are a session
  of their own, skein 10, beside the critical path.

  Decision: accepted, by director, 2026-10-19.
## Answers received

- **The director,** 2026-10-19: the questions went to the forge kit's
  designer at 00:40Z, the [shape] ones (1, 14, 17, 23, 30, 31, 32) to be
  answered first; the drafts carry the recommendations, marked pending.
  The planner's decisions 9 and 10 stand.
- **The designer,** 2026-10-19, the [shape] answers (1, 13 with 14, 17,
  23, 30, 31, 32), landed in skein `d9f3e39` and temper `b4cfb999`:
  `~/src/tmp/director/forge-kit-answers-1.md`; each in the table above,
  and in the session files that build it. The design's inputs are now
  at those commits; the code is the base's.
- **The designer,** 2026-10-19, the rest of round 1 (2 to 12, 15, 16,
  18 to 22, 24 to 29, 33), landed in skein `8938e16` and temper
  `dd12b8e`: `~/src/tmp/director/forge-kit-answers-2.md`. Two differ
  from the recommendation: 8 (B) and 29 (temper's step code, not the
  shell). Its open item, a 429 on Merge's read after the merge, is row
  35. The design's inputs are now at those commits; the code is the
  base's.
- **The designer,** 2026-10-19, question 34, landed in skein `25d2551`:
  `~/src/tmp/director/forge-kit-answers-3.md`. Every design question is
  answered.
- **The designer,** 2026-10-19, question 36, landed in skein `064c55f`:
  `~/src/tmp/director/forge-kit-answers-4.md`. A, made exact, so that
  skein 09.4 checks it in every run. Every design question is answered.
