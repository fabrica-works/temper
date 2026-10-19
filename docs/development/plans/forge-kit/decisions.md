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
    the real loop (temper's domain/testing.md 4.2).

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
    skein's forge peer (domain/testing.md 4.2) and drops its draft 07's host
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

109. **skein 05's Shapes against main** (ready check of skein-05.1).
     Amends: skein/05-fakes.md 05.1 Shape and 05.4 Files/temper's side. Reverses: nothing.

     The ready check found two lint adaptations and omitted accounting/compatibility users. The domain's moved tests/memory.rs adds `#![cfg(test)]` and explicitly typed u32 ranges in Measured::call and fill. This preserves the no-exception lint rule and existing memory algorithms, limits and assertions.

     Include skein-fake-forge-protocol/src/service.rs::worst_case in 05.4's snapshot accounting. Expand the next temper repin's compatibility scope to testing/temper-fake-git/Cargo.toml (skein-fake-peers dependency), its already named source users, and tests/one-process/src/stories.rs::{forge,prepare_merge_branch} plus src/referee/forge.rs::green. They follow skein_fake_peers::forge::Forge and its constructor; prepare_merge_branch uses Forge::next_reply and reserves bounded pending Queue room before stepping/firing. Preserve story inputs and assertions. Refresh further copies and derived values and list further users/repairs in the commit body. Temper 05.2 retains its peer migration and deletions.

     Checked starting skein 0db7b1fc7307aa50980ce75637e686e384adc4e4 and temper c4f89032047e4980b9f946ce8abf1c2c81653ba8; downstream searches cover smith, jig and hearth. Verified memory's untyped ranges, protocol service::worst_case, fake-git's manifest, the stories' Forge constructor/pending pushes, and green's temper_fake_git::Forge parameter against that source. No further missed consumer was found.

     Corrected scratch clippy passes for skein-fake-forge-domain, protocol and peers. Sequential focused suites pass 100 domain tests (including 3 memory), 58 protocol probe tests and 8 existing peer tests; fuzzy/browser select zero in those packages. The probe confirms the written accounting values 4_645_153, 4_663_809 and 4_663_949. Full interface A, remaining future protocol units, peer capture/sharing/delivery, and downstream migrations remain unrun until declared implementations; temper 01.2 is the expected prerequisite. Evidence: pump/ready-skein-05.md and scratch-ready-skein-05/suite-results.tsv.

110. **Restore the disk floor and release completed check targets** (infrastructure, skein 02's block of 2026-10-19T05:51:53Z).
     The disk block is resolved: filesystem availability is 46,001,192,960 bytes (about 43 GiB), above the 40 GB start floor. The coordinator may start eligible sessions after its normal resource check; development.md 11.4's floors and six-session limit below 60 GB remain in force.

     Development.md 11.4 permits removing an inactive session's target once none of its heavy tasks is queued or running. All six supervisor checks had ended and the queue held no unfinished task. Removed nine scratch build targets from those completed checks, retaining source, patches, reports and logs. Their apparent 29 GiB reclaimed only about 1 GiB because Btrfs shares cache extents.

     After the ten-minute GC interval, kache gc --max-age 1h found older entries still cloned into targets and reclaimed nothing. Its tracked-target dry-run found 36 build targets unused for at least an hour, estimating 11.4 GiB reclaimable. The subsequent tracked cleanup removed those disposable outputs and restored the floor. Exact preview and cleanup output are retained in pump/disk-clean-preview.json and pump/disk-clean.out. No source checkout or retained check evidence was removed.

     The README now requires completed supervisor checks to release scratch targets, including custom CARGO_TARGET_DIRs, once their workers and heavy tasks finish. Measure filesystem availability afterward; preview further inactive tracked targets if needed. Preserve source and evidence, and record further build-output paths cleaned. This supplements the existing session cleanup rule without changing start floors or GC cadence. The last GC was 05:56:06Z; the usual ten-minute interval still applies. Kache now reports 222.1 GiB private and 41.9 GiB cloned; its next eligible normal GC can reclaim cache formerly retained by stale outputs.

111. **Refresh stale primary graphs** (hourly check of 2026-10-19T06:09:28Z).
     Reverses: graph permission of 2026-10-19T05:06:14Z, temporarily.
     Amends: README, Rules of this plan.

     The hourly check finds both primary indexes older than landed source. Until the supervisor refreshes and verifies them, the coordinator and implementers use source searches instead of either graph. The README records that temporary restriction. Existing checks continue; no start is held. The kit remains d6d4abe with the director's adapter.

     Checked with codebase-memory-mcp index_status: srv-data-git-runner-src-skein indexed_at 2026-10-19T05:04:07Z; srv-data-git-runner-src-temper indexed_at 2026-10-19T05:04:37Z, both ready at their exact primary roots. git log -1 --format='%H %cI' origin/main gives skein 7436461f0fce6d0178c483f3b631e1a4db5fc343 at 2026-10-19T06:11:49Z, temper 037991de0b1b14a1e474bc9ca2766f4d49befcd6 at 2026-10-19T06:20:59Z. Both postdate their indexes. Commands plan ruling/ring are present in kit/scripts/dev/plan, dispatch lines 669 and 671.

112. **temper 02's Shapes against main** (ready check of temper-02.1).
     Reverses: nothing.
     Amends: temper/02-client.md Needs at start and 02.1 Needs; README's temper 02 session row.

     Temper 02.1 requires temper 01.3 as well as skein 02.2. The measure cutover in its repin changes protocol size expectations only after temper's component uses skein's call plans. Retain the existing three expectation changes, bounds, assertions and four-commit shape. Refresh further dependency copies if found and list them in the commit body.

     Checked skein 0db7b1fc7307aa50980ce75637e686e384adc4e4 and temper 2a141e6d5d33a527be329a0aa0f8a9f771948023. git show temper origin/main:crates/temper-engine-protocol-forge/src/reads/size.rs shows files_size using size_of::<api::File>(); contents.rs::Plan::row likewise accumulates the local file width. The existing temper/01-forge.md 01.3 deletes those local plan modules and imports skein_forge::forgejo::plans::plan::{Plan,Step,Bounds}. That prerequisite supplies the plans changed by skein 02.2; it introduces no dependency cycle. Searches across all five repositories found no additional consumer correction.

     The scratch planned-value probe reproduces the absent prerequisite: two contents assertions fail, 52 versus 68, and the 4032-byte side remains accepted. Restored protocol baseline passes 117 focused tests; fuzzy selects none. Clippy passes for four reached packages, and the client/connector/client-world suites pass 130 focused and 2 fuzzy tests. Future key/measure implementations, actual migration/deletion, full joint/process/gate/browser proofs remain unrun until their declared steps. Evidence: pump/ready-temper-02.md and scratch-ready-temper-02/evidence/report.txt. This check does not hold any already running session.

113. **Primary graphs refreshed** (hourly check of 2026-10-19T06:09:28Z).
     Reverses: ruling 111 of 2026-10-19T06:21:24Z and its graph-restriction steer of 2026-10-19T06:22:09Z.
     Amends: README, Rules of this plan, removing the temporary restriction.

     Both exact primary-root indexes are refreshed and ready; the temporary graph restriction ends. Use the graphs for discovery and verify each name in the working checkout, as workspace AGENTS.md requires. The first temper refresh aborted before publication, preserving the earlier index; its retry succeeded. No implementation or start was held.

     Checked codebase-memory-mcp index_status: srv-data-git-runner-src-skein is ready, root /srv/data/git/runner/src/skein, indexed_at 2026-10-19T06:22:53Z; srv-data-git-runner-src-temper is ready, root /srv/data/git/runner/src/temper, indexed_at 2026-10-19T06:25:11Z. git -C /srv/data/git/runner/src/skein log -1 --format='%H %cI' origin/main gives 7436461f0fce6d0178c483f3b631e1a4db5fc343 at 06:11:49Z; the same command for temper gives 280efe4fc91554f5fb226ea3c0b85090cb90b47b at 06:23:32Z. Both HEADs equal those origin/main commits; git diff --name-only HEAD origin/main -- crates testing tests tools Cargo.toml Cargo.lock prints nothing in both. Indexing postdates those snapshots. Preserve source searches for documented parser gaps and later worktree changes. The kit remains d6d4abe.

114. **temper 04's Shapes against main** (ready check of temper-04.1).
     Reverses: nothing.
     Amends: temper/04-setup.md, sketch A, temper_forge_setup_shell::reconcile::Settings::documents.

     Correct the sketch's documents field to skein_forge::forgejo::limits::Limits. Temper 01.2 has deleted the old dialect crate; the landed shell already uses that skein type. No other field, behavior, bound, assertion or prerequisite changes. Refresh any further copy and list it in the commit body.

     Checked skein 7436461f0fce6d0178c483f3b631e1a4db5fc343 and temper 52eba416314f30a384bdf24902ec31cb08c00ad5. Graph search_graph(project=srv-data-git-runner-src-temper,name_pattern=documents,file_pattern=*reconcile.rs) locates Settings::documents in crates/temper-forge-setup-shell/src/reconcile.rs. rg -n 'documents|limits::Limits' there confirms `pub documents: skein_forge::forgejo::limits::Limits`. The all-increment name/copy/derived-value search across five repositories found no omitted consumer. Missing setup APIs belong to declared skein 04.1 or temper 04.1–04.3 additions.

     Scratch exact-commit repin and workspace/all-target check pass without source repairs. Corrected fragment and five reached packages pass clippy; sequential suites pass 237 focused and 1 fuzzy, 0.887/0.579 s; the seven-command startup test passes in 0.242 s in the exclusive lane. Future behavior, complete cutovers, full gate and process/live proofs remain unrun. Evidence: pump/ready-temper-04.md and scratch-ready-temper-04/report.txt. Scratch source/logs remain; completed targets were released.

115. **skein 06's moved inputs against main** (started re-check of skein-06.1).
     Reverses: ruling 108 of 2026-10-19T05:39:47Z only for the qualification of shipped's profile helper; its remaining corrections stand.
     Amends: skein/06-conformance.md, 06.1 pin re-export wording, 06.2 runner-pin users and 06.3 shipped helper.

     After temper's pin cutover, the copied throwaway has no runner-pin re-exports to retain. Skein 06.1 adds the designed `pub use skein_forge::forgejo::pin::{RUNNER_VERSION,names_runner}`. When 06.2 moves the conformance, switch registration.rs's RUNNER_VERSION, runner.rs::version's names_runner and tests.rs::a_listed_runner_is_found_by_its_uuid's version reference from the dialect pin to skein_throwaway_forgejo's re-exports. This implements the plan's existing fake-forge 6 exception to its general no-re-export rule.

     In 06.3's crates/skein-forge/tests/forgejo_deliveries.rs::shipped, call `forgejo::profile_limits()` through `use common::forgejo`; the longer qualification triggers unused_qualifications under skein's lints. Preserve all 58 moved test names and assertions. Refresh further copies and derived users and list them in the commit body.

     Checked skein 7436461f0fce6d0178c483f3b631e1a4db5fc343 and temper 52eba416314f30a384bdf24902ec31cb08c00ad5. search_code of the exact temper graph locates the three runner users. git -C /srv/data/git/runner/src/temper show origin/main:tools/forgejo-conformance/src/registration.rs contains `use skein_forge::forgejo::pin::RUNNER_VERSION`; runner.rs and tests.rs likewise name that pin. rg -n 'RUNNER_VERSION|names_runner' testing/temper-throwaway-forgejo/src/*.rs finds no remaining re-export. The planned 06.1 explicitly supplies both exports. The corrected scratch integration file imports common::forgejo and clippy passes with `fn shipped() -> Limits { forgejo::profile_limits() }`. Searches across all five repositories found no additional repair.

     Sequential heavy proofs: temper clippy passes including the 06.4 sketch; 167 focused pass (1 skipped) and 13 fuzzy. Skein clippy passes after the corrections; 142 focused and 3 fuzzy pass, browser selects none. A transplant-script-only Malformed qualification error was corrected before those passes. Future Host/attach, recorder and mask/boundary proofs remain unrun; unchanged HTTP/live proofs were not repeated. Evidence: pump/recheck-skein-06-start.md and scratch-recheck-skein-06-start/results.txt. No running step is held.

116. **temper 06's Shapes against main** (ready check of temper-06.1).
     Reverses: nothing.
     Amends: temper/06-throwaway.md, 06.1 Files and 06.3 Files/Shape.

     Include tools/forgejo-conformance/src/component/hosted.rs::Hosted::new in 06.1's crate-name cutover; its Token parameter becomes skein_throwaway_forgejo::Token. Include src/cli.rs::tokens in 06.3's captured-body read migration, covering all three crate::json reads. Correct the Json read sketch to `v.root().get("a").get("b").str()`: Json supplies root and At supplies get. Preserve payload interpretation, assertions and all existing prerequisites; refresh further copies and derived users and list them in the commit body.

     Checked skein 7436461f0fce6d0178c483f3b631e1a4db5fc343 and temper 52eba416314f30a384bdf24902ec31cb08c00ad5. rg -n 'Token|fn new' tools/forgejo-conformance/src/component/hosted.rs confirms Hosted::new's temper_throwaway_forgejo::Token parameter. rg -n 'crate::json' tools/forgejo-conformance/src/cli.rs finds the three body reads in tokens. The planned json.rs interface in this session has Json::root and At::get; the materialized scratch interface compiles with the corrected read. All-increment path/name/content/derived-value searches across the applicable downstream repositories found no further missing user.

     Through heavy, exact repin, workspace all-target check, formatting and sketch clippy pass. The git-transcripts and forgejo-conformance suites pass 42 tests, zero skipped, including the JSON and refusal probes; quiet execution is 0.115 s, JSON units 2–5 ms and refusal tests 5 ms. Fuzzy selects none successfully. Missing skein 06.1 is a declared prerequisite, not a reason to keep the ready check open: the coordinator starts the session after it lands. Actual kit swap/owned::attach, complete reads/writes, full suites and process/recording proofs remain unrun. Evidence: pump/ready-temper-06.md and scratch-ready-temper-06/report.txt; source/patch/logs retained and completed targets released.

117. **temper 05's Shapes against main** (ready check of temper-05.1).
     Reverses: nothing.
     Amends: temper/05-fakes.md, 05.2 constructor inventory, factory Shape, Test shape and relative Proof counts.

     Rename the existing tests/one-process/src/world.rs::peer LLM factory to llm_peer and update run_phase's spawn_restartable function item before adding the planned forge peer() factory. The private factory has only that caller. Correct the constructor inventory: testing/temper-fake-git/src/fixtures.rs::named has one Forge::new; tests/one-process/src/stories.rs::forge is separate.

     Apply testing-strategy.md 10, One behaviour per test, to the new hooks-address unit: a_port_zero_bind_names_the_hooks_address checks the successful port-zero bind; a_fixed_port_bind_is_not_a_hooks_address checks the fixed-port refusal; a_failed_bind_is_not_a_hooks_address checks the failed-bind refusal. Preserve all three original cases and literal outcomes. The new unit and focused-count deltas become +3, with fuzzy counts unchanged; existing moved tests retain their identities. Refresh further copies and derived counts and list them in the commit body.

     Checked skein 7436461f0fce6d0178c483f3b631e1a4db5fc343 and temper 52eba416314f30a384bdf24902ec31cb08c00ad5. search_graph in the exact temper project locates world::{peer,run_phase} and fake-git fixtures::named. rg -n '\bpeer\b' tests/one-process/src/world.rs finds only the existing definition and run_phase's spawn_restartable use. rg -n 'Forge::new|fn named' testing/temper-fake-git/src/fixtures.rs confirms the single constructor. The ready check covers both increments, names/content/derived users across downstream repositories; no other missing caller was found.

     Exact repin, workspace/all-target check, clippy and prebuilds pass through heavy, including the factory correction. Ten reached packages pass 455 focused tests (4.660 s) and 39 fuzzy (23.388 s); one-process has 12 focused / 4 fuzzy and fake-git 14 focused; the three new units take 10–11 ms. The unchanged Adopt story takes 1.801 s under this check's load. The plan already records its 0.953 s baseline above the 0.5 s ceiling and requires its cost/cause report; that requirement and suite budgets remain. This is not a new baseline miss of the whole-run measured suite aims. Its final peer/delivery execution and cost diagnosis remain to be proved at the assigned implementation step. Future fake/peer APIs, actual migration, hand_hooks and delivery assertions, full gate and process/live proofs were unrun. Evidence: pump/ready-temper-05.md and scratch-ready-temper-05/. Source and results retained; completed targets released.

118. **Sampled tests follow the testing strategy** (hourly check of 2026-10-19T06:09:28Z).
     Reverses: nothing.
     Amends: README session table and Rules; adds skein/01.s1-test-shapes.md.

     Add the test-only supplement skein-01.s1 after skein-01.4, before the closing temper repin, to correct hourly-01 points 2–5. Its named preservation exception permits splitting the five sampled unit tests while retaining every assertion input and outcome in an old/new mapping. It shares the repeated repository/Comments setup and names sampled capacities from fixture content. New tests use testing-strategy.md 10; other moved tests retain their preservation rules.

     The same supplement audits all callers/corpus/boundaries of the sampled unit defaults before reducing them. The finding is partial: string_bytes is 1024, whereas tokenizer string and body_bytes are 65,536. No smaller numeric default is authorized by guesswork. Derive named tiny defaults and explicit larger boundary fixtures from the complete audit; retain a necessary larger value with its justification. The audit is bounded to 20 minutes and three targeted compiler/test runs, with independent shape work continuing if it blocks. Production behavior, assertion cases, shipped/profile values and budgets remain unchanged. Refresh further copies and derived counts reached by these fixes and list them in the commit body.

     Checked at skein origin/main 50f743621439da9376b871ba0e366a4deacbffea: rg of the five function names in crates/skein-forge/src/{tests.rs,forgejo/tests.rs} finds each named function. The locate table has Located(7)/true, Located(0)/false and Done/false; the path, RFC3339, string/body and target tests combine the distinct outcomes listed in pump/hourly-test-shapes.md. rg/sed of the two comment tests confirms identical Request::Comments with fixture/specimen and number 3; no repository/comments helper exists in fixtures.rs. The file's limits() contains json.string/body_bytes 65_536, json/writer length 4_194_304, chunk/target_bytes 4_096 and string_bytes 1024. Capacities 13 and 12/11 occur in the sampled path/target tests. The graph locates fixtures::{limits,small}; source confirms them. These are observed defaults, not replacement values. plan status reports the closing temper-repin waiting, so no repeat closing repin is needed; no other closing repin is landed in the ledger. The kit's plan set/ruling/ring dispatch is present.

     The read-only follow-up verified the concrete findings but did not finish a sketch/proof. Its bare-bash heavy task 762 waited on pipe input with an empty log; the supervisor stopped only that owned task. No corrected-code pass or +13 test delta is claimed. The supplement's ready check and implementation must prove the actual mapping/count and clippy/profile results. Retained evidence: pump/hourly-test-shapes.md and scratch-hourly-test-shapes/. No source or kit change was made by supervision.

     Instructions: register skein-01.s1 from skein/01.s1-test-shapes.md, after skein-01.4, as a separate supplement session 01.s1. Its first step undergoes the normal ready check. Make the existing waiting temper-repin depend on it as well as all original skein steps. No additional closing repin step is required because none has landed. Pass the README rule to running skein test implementers; they keep the named preserved rewrites for this supplement.

119. **Include sampled fixture callers** (hourly check of 2026-10-19T06:09:28Z).
     Reverses: ruling 118 only insofar as its Files/Don't prevented necessary explicit boundary-fixture selection in other callers; its preservation and audit requirements stand.
     Amends: skein/01.s1-test-shapes.md, Files, Test shape and Don't.

     The fixture-limit audit also reaches skein-forge/src/forgejo/tests/{tokens,hooks,orgs,delivery,accounts}.rs. Permit those callers only to select a justified explicit boundary fixture when reducing a shared default requires it. Keep their test names, input bytes, assertions and outcomes. This does not expand the five-test split exception. Refresh further callers and derived fixture bounds found by the audit and list them in the commit body.

     Checked with rg -l 'fixtures::(limits|small)|fixtures::\*|fixtures::\{' /srv/data/git/runner/src/skein/crates/skein-forge/src/forgejo -g '*.rs': tests.rs and precisely those five modules are direct fixture users. The ready check must verify any additional aliases or newer users against its own snapshots. The audit's bounded scope and proofs remain as ruling 118.

120. **skein 03 Shapes against main** (ready check of skein-03.1).
     Reverses: nothing.
     Amends: skein/03-connection.md, 03.1 sizing and temper-side inventory, named body fixtures, 03.2 token test/guard, 03.3 routing fixture and relative proof counts.

     03.1 must account for retained Settings.word, Settings.user_agent and plaintext names as required by forge-connection.md 7.4 (the worst case fits u64). Add checked settings::owned_bytes and thread its byte count into worst_case, check and each reached caller. Inventory temper-service/src/system.rs::memory, tools/forgejo-conformance/src/component/hosted.rs::Hosted::new and the moved tests.rs/tests/hooks.rs bound callers. The complete old fixture sum is 2,812,325 bytes, a baseline rather than the final expected sum. Remeasure after the full owner-value adaptation and refresh further copied/derived bounds, listing each in the commit body.

     The next temper repin also repairs tests/engine/protocol-forge/src/fixtures.rs::{settings,shared,protocol_shared}: supply owner word/agent and flatten the five client numbers. Keep temper's six-byte word and 1045-byte body bound until the skein 08 move. Include crates/temper/src/lib.rs::forges transport consumer. The existing rule leaves these temper edits to its next repin.

     Name the new fixture word, agent, operation cap and body bounds: kit and kit/0.0.0, OP_BYTES 512, BODY_ROOM 1042 = 2*512+3+15, BODY_ROOM_SHORT one below, and MAX_WORD_BODY_ROOM 1055 = 2*512+16+15. The supplied body-bound sketch uses them.

     Split the new token boundary proof: api_tokens_stop_before_the_listener_range starts at the listener prefix and must panic on one allocation; the_last_api_token_is_still_issued independently accepts its predecessor. Name LISTENER_OWNER = 1_u64 << 59_u32 and LAST_API_OWNER one below; use MAX_OUT.below for queue capacity. Guard before issuing/incrementing, retain checked increment. This prevents should_panic from hiding rejection of the valid predecessor. 03.2 now adds seven tests (122 total); 03.3 reaches 128, or 112 preserved identities plus 3+7+6 new. Inherited assertions/names remain.

     03.3's two-forge harness must map configured endpoint addresses to forge numbers in tests/fixtures.rs::Harness::{with,drain}; route Request::Connect.addr instead of assigning every peer FORGE. Give the two endpoints distinct fixture ports. An unknown-forge test with no total connects uses no configured forges; otherwise compare connect count around the call.

     Checked: ready-skein-03 examined all three increments and downstream users at skein 50f743621439da9376b871ba0e366a4deacbffea and temper a6f34f6af3f484e9f24b58fed9feab1d8ac04aa2. Exact-project search_graph locates worst_case/check/open_start, Harness::drain and world settings/shared/protocol_shared; rg confirms each in main. sed of open_start shows allocation then increment then 2^60 assertion; hooks.rs reserves 2^59. sed of Harness::drain shows Connect { owner, .. } and forge: FORGE. rg finds op_bytes 512/body_bytes 1045 and the startup/conformance/test bound callers; system.rs defines memory. New helper/constants/tests belong to the named future increments. The ready check's copy/derived sweeps found no affected smith/jig/hearth forge user.

     Proof: pinned-kit heavy baseline clippy and suites passed 523 focused/11 fuzzy; token counterexample independently failed valid-bound acceptance while the original panic test passed. Corrected sketch clippy and suites passed 531 focused/11 fuzzy (2.817/6.258 s); new probes 4–7 ms. Skein forge/http clippy and 275 focused passed; fuzzy selected none. All 117 current inherited identities are retained. Actual owner-value, head and page-measure implementation proofs await their named prerequisites; no full gate/process/live result is claimed. Budgets/lints unchanged. Evidence: scratch-ready-skein-03/report.md, corrections.patch, results.tsv and derivations.md; completed targets released.

     Instructions: mark skein-03.1 ready after this ruling, retaining its checked commits; normal prerequisites still govern its start. Future declarations are required implementation work, with complete worst-case and relative test-count proofs. The kit's plan ready/ruling/ring dispatch is present.

121. **Refresh hourly graph indexes** (hourly check of 2026-10-19T07:15:01Z).
     Reverses: ruling 113 of 2026-10-19T06:26:26Z only as to permission to use stale graph indexes; its primary-project identity and worktree verification rules stand.
     Amends: README Rules of this plan, temporary hourly-02 graph restriction.

     Both primary indexes are older than origin/main. Until refreshed, do not use the code graph for skein or temper; search with rg. This follows development.md 14.1 step 2. The supervisor refreshes the primary checkout indexes and lands the lifting rule after checking the result. Worktree source remains authoritative; empty/stale graphs prove no absence.

     Checked: index_status reports srv-data-git-runner-src-skein at /srv/data/git/runner/src/skein indexed 2026-10-19T06:22:53Z and srv-data-git-runner-src-temper at /srv/data/git/runner/src/temper indexed 2026-10-19T06:25:11Z. git log -1 origin/main reports skein 283a7ec9144a4db7ffed00aedeaf0871cde12aca at 07:06:15Z and temper ab7ef9b9403f4f7041f6c678c8ea7e483dbbbece at 07:06:22Z, both later. The known hostile JSON parse gaps remain source-search cases. Kit plan ruling/ring dispatch is present; no kit move is authorized.

     Instructions: coordinator passes the README restriction word for word to all running implementers; new sessions read it there. Continue implementation using source searches. The supervisor's refresh does not hold a ready/start check or any session.

122. **Use refreshed primary graphs** (hourly check of 2026-10-19T07:15:01Z).
     Reverses: ruling 121 of 2026-10-19T07:16:00Z and steer of 2026-10-19T07:16:38Z, only their temporary graph restriction.
     Amends: README Rules, removes the temporary hourly-02 graph restriction; decisions.md ruling 121 metadata corrects ruling 113's time to its verified 06:26:26Z commit time.

     The primary skein and temper indexes have been refreshed. Resume code-graph discovery for exact projects srv-data-git-runner-src-skein and srv-data-git-runner-src-temper, verifying each name in the actual worktree before relying on it. Use source searches for newer worktree changes and declared coverage gaps. The restriction is lifted; no kit move or proof-budget change.

     Checked: index_repository fast completed on /srv/data/git/runner/src/skein and /srv/data/git/runner/src/temper. index_status reports ready at 2026-10-19T07:17:38Z (26381 nodes) and 2026-10-19T07:18:49Z (24063 nodes). git rev-parse HEAD origin/main agrees in skein at 283a7ec9144a4db7ffed00aedeaf0871cde12aca and temper at eab32d2657fcd3de16520a0ffcc97250a7145d23. The index's partial/hostile JSON and excluded files remain source-search cases, not evidence of absent symbols. git show -s --format=%cI 774074daac99c7c10f52d516e747476bccf391f4 reports 2026-10-19T07:26:26+01:00, correcting the earlier typed timestamp. The kit's plan ring/ruling dispatch is present.

     Instructions: coordinator lifts the temporary instruction in running sessions and passes the exact primary-project/worktree-verification rule word for word. Future sessions use the README's remaining graph rule. Hourly-02's other findings are still being checked.

123. **Refresh later skein source and wait by notification** (hourly check of 2026-10-19T07:15:01Z).
     Reverses: ruling 122 of 2026-10-19T07:19:58Z and its lifting steer only for skein, whose source advanced after the refresh. Temper's graph permission remains.
     Amends: README Rules, temporary skein-only graph restriction and wait-discipline rule.

     Refresh skein's primary graph after its later source landing. Until verified, do not use the code graph for skein; search with rg. Temper retains the refreshed graph with worktree verification: its only intervening commit is ruling122, excluded Markdown. This is development.md14.1's current-source check, not a claim that a newer index contains an unindexed later source commit.

     Apply development.md7.4's wait discipline to running and future implementers: prefer completion notifications; use supported work-completion waits of at least300 seconds, and identify tool-imposed exceptions instead of repeated expiry/polling. Incoming messages and required operational checks remain prompt. The hourly pulse reports29 short session waits; coordinator count iszero. No proof or implementation-budget change.

     Checked: hourly-02 index/source comparison gives skein refreshed07:17:38Z (artifact07:17:53Z), later origin/main4c0895d676c9c80ebc5e95c1f03d686f076e0b0c at07:22:31Z. Temper's index07:18:49Z predates only0ece5ea545b923e16ed3c5f4ecedc9c2810c2ad8 at07:19:58Z, the supervisor's Markdown ruling122. Its changed paths are plan README/decisions. The run kit has plan ruling/ring/bg and heavy's supported pueue wait; the development7.4 rule names300000ms wait_agent. Actual tool limits must be reported, never guessed.

     Instructions: coordinator passes both README rules to running implementers. Supervisor refreshes only primary skein, then removes its temporary rule with a verified lifting steer. Hourly-02 findings2–6 have a bounded scratch follow-up for a concrete test-only supplement and detectable gate checks; those corrections remain pending, not waived.

124. **Use skein graph after setup landing** (hourly check of 2026-10-19T07:15:01Z).
     Reverses: ruling123 of2026-10-19T07:27:29Z and its following graph-restriction steer, only the temporary skein prohibition; its wait-discipline rule stands.
     Amends: README Rules removes the hourly-02 follow-up graph restriction.

     The later skein primary source has been reindexed; resume graph discovery with exact project srv-data-git-runner-src-skein and worktree verification. Temper keeps its refreshed exact-primary graph; newer worktree changes and parse/excluded gaps use source search. The wait-discipline rule and all proof budgets remain.

     Checked: index_repository fast succeeded; index_status reports ready at2026-10-19T07:28:55Z, root/srv/data/git/runner/src/skein,26593nodes. git rev-parse HEAD origin/main agrees at4c0895d676c9c80ebc5e95c1f03d686f076e0b0c. Temper's intervening ruling122/123 changes are Markdown only, so its07:18:49Z index still covers the indexed source. The kit's plan ruling/ring commands are present.

     Instructions: coordinator passes the lifting graph instruction to running implementers, retaining worktree source verification and source fallbacks. Hourly-02 contract findings and unfinished process/downstream results remain under their bounded follow-ups, not waived.

125. **Keep only the setup decoder's used result fields** (ready check of skein-07.1).
     Reverses: nothing.
     Amends: skein/07-worlds.md, 07.3 Shape.

     ## Decision

     The private setup world's decoder result carries only the fields its exchange reads: `rows` and `document`. In skein 07.3 omit `Decoded.steps` from `tests/forge/setup/src/decode.rs`, discard the tokenizer driver's returned count, and preserve its execution bound and all decoder outcomes. The original `tests/codecs/src/forgejo.rs::Decoded.steps` and its fuzzy bound assertion stay in temper. No test case or moved test name changes.

     The ready-check prototype's `#[expect(dead_code)]` is not adopted: development.md 14.3's Weight requires “no lint exception in tests.” Do not add a setup assertion merely to read the field. Refresh any further private copy reached by this move and list it in the commit body; retain counters that an actual assertion consumes.

     The static ready check found no other mismatch across 07.1–07.4. Its compiler/test tasks never started before the timebox; no executable proof is claimed. The increment's clippy and focused/fuzzy proofs remain required.

     ## Checked

     - Graph: `search_graph(project=srv-data-git-runner-src-temper, file_pattern=tests/codecs/src/forgejo.rs, name_pattern=Decoded|decode_answer)` found Decoded62–66 and decode_answer74–94; verified in the primary source.
     - `sed -n '62,94p' temper/tests/codecs/src/forgejo.rs`: Decoded has rows/document/steps; decode_answer stores drive's usize count. The driver is bounded and has no must_use annotation.
     - `sed -n '166,184p' temper/tests/forge-setup/reconcile/src/exchange.rs`: decode_answer's result reads document and rows only. `rg -n '\.steps\b' temper/tests/forge-setup/reconcile temper/tests/codecs` finds only tests/codecs/tests/fuzzy_forgejo.rs:15, outside this move.
     - Ready report pump/ready-skein-07.md checked skein283a7ec9144a4db7ffed00aedeaf0871cde12aca and temperab7ef9b9403f4f7041f6c678c8ea7e483dbbbece, all four increments and downstream consumers; tasks813/817/820/821 were cancelled while queued, with no targets created.
     - Pinned kit/scripts/dev/plan declares ruling at line30 and ready at31; kit remains d6d4abe with the director adapter.

     ## Instructions

     Mark skein-07.1 ready with those checked commits. Implement the revised private result in 07.3 and run its existing proofs; its capability dependencies and the coordinator's landing checks still apply.

126. **Correct the new client contract tests and check recurring shapes** (hourly check of 2026-10-19T07:15:01Z).
     Reverses: nothing; the preserved-test exceptions of rulings 118/119 keep their scope.
     Amends: README session inventory/Rules; skein/02-client.md final Done when; new skein/02.s1-contract-tests.md and findings/contracts-assertions.md.

     ## Decision

     Register skein-02.s1 as the test/gate supplement in `skein/02.s1-contract-tests.md`, after skein-02.4 and before the closing temper-repin. No closing repin has landed, so no repeated repin is added. It resolves hourly-02 findings 2–6 without changing production or any original assertion case. The permanent assertion map records the 60 checked cases and eight added row-success assertions.

     Repeated pull setup and unnamed capacities recur after the earlier hourly steer. Under development.md 14.4 their rules become the README's Client contract fixtures rule and scoped Rust source checks in the repository gate. The checker must demonstrate positive and negative controls; the scratch prototype remains uncompiled and is evidence for implementation, not approved code.

     Two mixed tests become eighteen focused tests, a relative +16. At the checked 02.2 base that is client84->100; after planned 02.4,114->130 (74 preserved+56 new). Only final Done when count consumers change; historical increment proof counts remain. Reconcile any intervening contract additions and refresh further reached copies/count consumers in the supplement, listing them in its commit body.

     ## Checked

     - Bounded report pump/hourly-02-contracts.md and its source/assertion inventories checked skein origin/main4c0895d676c9c80ebc5e95c1f03d686f076e0b0c; no primary edits. Rust task829 never started before cancellation: no compiler, checker-control or suite pass claimed.
     - `rg -n '^fn |^#\[test\]' skein/crates/skein-forge-client/src/tests/contracts.rs`: ten tests, including the stored/answer combined tests. `rg` finds identical pull.head/base/reviewers assignments in both, capacity65535 and expected length65539.
     - Source map derives five-row byte widths80/80/46/83 and four-row64/64/38/67 independently; byte room83 isolates the four row limits. The map preserves twelve scheduling read/cap cases and all namespace/key/binding assertions.
     - `sed` of skein/scripts/check.sh shows Rust fmt/clippy and sequential default/fuzzy/browser suites, but no contract source checker. New checker is Rust; no kit change or new language.
     - `rg '114|74 preserved|40 new'` plus the bounded all-repository consumer sweep finds only skein/02-client.md final Done when count consumers; its historical02.4 proof remains114. `plan status`: temper-repin waiting,11/61 landed.
     - Pinned kit/scripts/dev/plan declares set/ruling/ready/ring; kit d6d4abe retained. Supplement dependencies and registration follow development.md10.3/10.4.

     ## Instructions

     Register skein-02.s1 from skein/02.s1-contract-tests.md after skein-02.4; make temper-repin await it. Pass the Client contract fixtures rule to the running skein02 session and later implementers. Preserve all independent work and existing proof obligations; implement and validate the supplement against its actual post02.4 source.

127. **Refresh primary graphs after the next source landings** (graph comparison at the 08:12 ready events).
     Reverses: ruling124 of2026-10-19T07:30:20Z and ruling122 of2026-10-19T07:19:58Z only as permission to use stale primary indexes; exact-project/worktree verification remains.
     Amends: README Rules, standing Graph freshness rule and current temporary hold.

     ## Decision

     Both primary indexes lag new source landings. Until the supervisor reports each refresh complete, use source searches for skein and temper rather than their graphs. Refresh only the primary checkouts, retaining exact project identities and worktree source verification. The README's Graph freshness rule persists after this temporary hold lifts; empty/stale coverage never proves absence. No implementation, ready check or unrelated work waits for the refresh when source searches suffice. Kit d6d4abe stays pinned with its adapter.

     ## Checked

     - index_status: srv-data-git-runner-src-skein root/srv/data/git/runner/src/skein indexed07:28:55Z; srv-data-git-runner-src-temper root/srv/data/git/runner/src/temper indexed07:18:49Z, both ready but stale.
     - git log -1 origin/main: skein5aaba05472924164e6fb6f07ea779036dbe4c153 at08:08:45Z; temper e83e552e4591ad6f73d35a615f27c10085d98e17 at08:11:18Z. git rev-parse HEAD agrees with each primary origin/main.
     - Pinned kit/scripts/dev/plan declares ruling/ring; no watch or kit move is needed. Ready-check backgrounds ready-skein-10 and ready-skein-01s1 already carry source fallback for the stale indexes.

     ## Instructions

     Pass the current two-repository graph hold and continuing source-search permission to running sessions. Supervisor refreshes the exact primary projects; lift the temporary hold after status/source comparison, retaining the standing rule.

128. **Investigate archived-write reset within a bounded diagnostic scope** (cross-session scope, temper 06.2's block of 2026-10-19T08:13:14Z).
     Authorize the new06.2 diagnostic exception: scratch only,20minutes and three executable diagnostic attempts, at most one full recording among them. Compare the unchanged archived-write request with pre-06.2 code and report its failure layer/proposed repair location or remaining uncertainty in at most15lines. This assigns diagnostic scope without presuming the JSON migration is independent.

     The plan's “Behaviour unchanged” and Don't's “change a transcript, a delivery or facts.txt” still hold. No outside-scope repair lands under this authorization; the next ruling assigns upstream scope. Preserve the JSON implementation checkpoint and all proof checks; do not swallow the reset. Independent sessions continue.

129. **Use refreshed primary graphs after the next landings** (graph refresh after the 08:12 ready events).
     Reverses: ruling127 of2026-10-19T08:15:49Z and its steer of2026-10-19T08:16:18Z, only the temporary two-repository graph hold.
     Amends: README Graph freshness removes the hold; ruling127 metadata corrects ruling124's reference from the steer time07:31:24Z to verified commit time07:30:20Z.

     ## Decision

     Both primary graphs are refreshed; resume discovery for exact projects srv-data-git-runner-src-skein and srv-data-git-runner-src-temper. Keep README Graph freshness, worktree source verification and source search for newer/unindexed/partial code. No kit move or proof-budget change. Correct the earlier metadata timestamp as named above; no substantive ruling changes.

     ## Checked

     - index_repository fast/persistent succeeded; index_status reports skein root/srv/data/git/runner/src/skein ready08:17:08Z,26724nodes; temper root/srv/data/git/runner/src/temper ready08:18:00Z,22399nodes.
     - git rev-parse HEAD origin/main agrees in skein at5aaba05472924164e6fb6f07ea779036dbe4c153 and temper atb7569d47472fcfe4b0132b50f4e0cfd1a755161a. Since the indexed temper source e83e552e, intervening changes are Markdown only (git diff --stat), so its indexed code is current.
     - Known partial parse counts remain3/2; hostile JSON and excluded paths use source. git show of49949acedcca187bbecaa9dac3586a01d480b3f5 gives2026-10-19T08:30:20+01:00, hence07:30:20Z.
     - The recorded graph steer is2026-10-19T08:16:18Z; pinned kit plan ruling/ring commands exist. Kit d6d4abe remains.

     ## Instructions

     Lift only the temporary graph hold in running sessions; retain the README rule, exact primary project identities, worktree verification and source fallbacks. Temper06.2's bounded diagnostic authorization remains unchanged.

130. **Permit the two shared Comments setup substitutions** (ready check of skein-01.s1).
     Reverses: rulings118 of2026-10-19T06:46:02Z and119 of2026-10-19T06:47:11Z only insofar as their preservation clauses prohibited the two named Comments setup substitutions; their five split exceptions, assertions and fixture audit remain.
     Amends: skein/01.s1-test-shapes.md Test shape/Don't.

     ## Decision

     Permit the two named test callers, `forgejo/tests.rs::{a_comment_at_or_below_after_is_skipped_body_unread,a_cut_thread_counts_the_comments_it_skipped}`, to replace their repeated Comments construction with the supplement's `fixtures::{repository,comments}` helpers. Preserve their names, request/input bytes and assertions. This makes its required shared setup possible; it authorizes no additional test split or general preserved-body rewrite. Other permitted boundary-fixture selections and the20minute/3run audit budget stand. Refresh any further helper copy reached by this substitution and list it in the commit body, retaining the same inputs/outcomes.

     The corrected scratch sketch retains36assertion cases in five tests split into21 (+16); snapshot package102->118. Keep the plan's relative accounting against the actual post01.4 inventory rather than replacing it with that snapshot total. No smaller default is established by this ready check; perform the existing caller/boundary audit before lowering one.

     ## Checked

     - ready-skein-01s1 checked all named files/callers and downstream snapshots: skein5aaba05472924164e6fb6f07ea779036dbe4c153, temper e83e552e4591ad6f73d35a615f27c10085d98e17; smith e79b9a6b, jig01b53884, hearth d0a552d8. Missing fixtures belong to this supplement;01.4 remains its prerequisite.
     - Exact-primary search_graph finds both callers in crates/skein-forge/src/forgejo/tests.rs; rg confirms identical Request::Comments blocks with fixture/specimen and number3. repository/comments helpers are new additions of this supplement, not missing upstream APIs.
     - Two heavy attempts: first found a scratch u64/u32 sketch error, corrected clippy passes; nextest skein-forge118pass0.056s; fuzzy/browser zero selected with --no-tests=pass. Source/mapping/logs retained in scratch-ready-skein-01s1; no primary edits or targets remain.
     - Existing session file requires shared Comments setup but previously restricted other preserved bodies to boundary-fixture selection; the two names are within its existing Files. Pinned plan ruling/ready commands exist, kit d6d4abe unchanged.

     ## Instructions

     Mark skein-01.s1 ready with the recorded skein/temper commits. At its actual post01.4 start retain all existing audit/proof obligations, preserve every original assertion case and report the relative count. No other session waits for this ready check.

131. **Hourly graph freshness after source landings** (hourly check of 2026-10-19T08:20:55Z).
     Reverses: ruling129 of 2026-10-19T08:19:26Z and steer of 2026-10-19T08:19:54Z only their current graph-use permission; the standing freshness rule remains.
     Amends: README Rules, Graph freshness.

     Follow README Graph freshness. Both primary indexes now precede source landings, so use source searches for skein and temper until the supervisor reports their exact primary refreshes. No worktree indexing; the kit stays d6d4abe. Further source landings require the same comparison rather than relying on an earlier permission.

     ## Checked

     - index_status(exact primary project): skein indexed_at08:17:08Z, root /srv/data/git/runner/src/skein; temper08:18:00Z, root /srv/data/git/runner/src/temper.
     - git log origin/main: skein source7a7867bb,6e290792,49725eca follow5aaba054; temper sourcea6b1c197 at08:28:17Z follows its refreshed e83e552e source.
     - git show -s --format=%cI e2985c1: ruling129 time2026-10-19T09:19:26+01:00; log.md records lift steer08:19:54Z. Kit scripts/dev/plan has ring and ruling dispatch; no command from newer main is required.

     ## Instructions

     Apply the temporary source-search instruction to running and ready-check sessions. Supervisor refreshes only the two exact primary indexes, then reports commits and partial/excluded coverage before lifting this temporary restriction. Check every graph name against the actual worktree and use source searches for every further uncovered copy.

132. **Skein 10 Shapes against main** (ready check of skein-10.1).
     Reverses: the original skein10 test-name preservation and fixed5/14/22/25/28 counts only as the explicit assertion mapping now records; no earlier ruling or steer is reversed.
     Amends: skein/10-plans.md Rules, Shapes, Files, Test shapes, Proof and Done when.

     Use the corrected skein10 plan. Its Run dispatches through the landed request/error decoders and Raw/Empty/Document body kinds, retains encoded request bodies, and can select the repository passed to Plan::new. Remarks is locally sliced, with no wire limit; include PullFor fallback and Checks among paged wire listings. Side-overflow tests first admit metadata and then exercise the raw shortage. Preserve the write resolver's range-before-equality order while moving owner caching after validation.

     The explicit old/new assertion map separates independent behaviours while preserving all literal cases and budgets: increments add8/9/13/11/7 tests, cumulative8/17/30/41/48. This path-filtered total replaces28; package totals stay relative. Repair the full headerless PullFiles fixture with explicit X-HasMore:false in the connection copy after03.1 and in temper on its next repin while present; preserve its side-advance expectations. Refresh any further content copy and list it in the commit body.

     ## Checked

     - Agent checked all five increments at skein5aaba05472924164e6fb6f07ea779036dbe4c153 and tempere83e552e4591ad6f73d35a615f27c10085d98e17; smith e79b9a6b, jig01b53884, hearthd0a552d8. Name/content/derived-user searches found no other required downstream repair.
     - rg primary skein: client.rs has Encoded.body:Option<Box<[u8]>>, decoder(request,limits), error_decoder(limits); plans.rs Body::Document/Raw/Empty; plan.rs Plan::new(repository,op); measure::FILE_BYTES=64. reviews.rs ReviewComments target appends /comments and no page.
     - rg primary temper contents.rs: multiple_changed_files_each_advance_to_their_own_sides uses a two-row fixtures::ok page and expects more:false. The future03.1 copy is named, not claimed landed. writes.rs caches keep_document before unequal-id rejection.
     - Scratch heavy attempts844/853/859 completed; final clippyPASS, skein109PASS with fuzzy/browser0selected; temper160PASS and fuzzy6PASS. Future Run and10.4/10.5 behaviours/machine proofs remain unverified. Sources/patches/validation retained under scratch-ready-skein-10; owned completed targets removed.
     - Python sum([8,9,13,11,7])=48 checks the explicit planned test inventory; no package count is inferred. Kit plan has ruling/ready dispatch; all compile commands used pinned heavy with explicit script files.

     ## Instructions

     Mark skein-10.1 ready immediately with the two checked full commits; the coordinator keeps its01.4 dependency. Preserve every mapped assertion case and report any further necessary split with its relative count and named consumer. Only10.5 awaits08.1; retain existing proof budgets and future-behaviour verification.

133. **Hourly primary graph refresh completed** (hourly check of 2026-10-19T08:20:55Z).
     Reverses: ruling131 of2026-10-19T08:30:16Z and steer2026-10-19T08:30:42Z only their temporary graph hold.
     Amends: no graph rule; correct skein/10-plans.md's two residual narrative counts to its ruling132 inventory (eight10.1 tests; six pure10.3 tests).

     The two primary source graphs are refreshed; lift the temporary hold under README Graph freshness. Use exact projects srv-data-git-runner-src-skein and srv-data-git-runner-src-temper, verifying each result in the actual worktree. Skein is covered through source6e29079218b9aa0bced12e8fe62b6c1ea02fb2c6; its later49725ecab135926f1f783bc30f70ab0ea1449b6d changes only docs/design/oauth.md. Temper is covered through source e3430f46ffeef64f62f44fbf8c11fdff82522f47 (checkout caada357b8e5fbf5de522720ed074b588c4c8a54). Later source landings require the same comparison. Use source searches for excluded paths, partial parses, unavailable graph tools and newer code. The kit remains d6d4abe.

     ## Checked

     - index_repository exact primary roots, fast/persistence:true; index_status: skein08:31:21Z26828nodes, temper08:34:22Z22399nodes. Temper refreshed again after concurrent tools source landing to avoid inferring coverage from its timestamp.
     - git rev-parse HEAD origin/main: skein both49725eca after clean fast-forward; git diff --name-only6e290792..origin/main prints only docs/design/oauth.md. Temper bothcaada357b; e3430f46source precedes its last refresh, then132Markdown only.
     - Partial parses remain3skein/2temper;8hostileJSON files unusable in skein; excluded docs/tools/scripts require source coverage. No absence inferred from graph gaps. Primary checkout code is untouched by the supervisor.
     - log.md records steer08:30:42Z point1 acknowledged08:30:51Z, passed to all six sessions. Kit plan ring/ruling dispatch verified. No process or kit update taken.

     ## Instructions

     Pass restored graph permission to the running sessions; retain source-search permission and actual-worktree checks. Future sessions follow README Graph freshness. Refresh any further uncovered copy by source and name it in the work's report. Ruling132's cumulative counts and all proof budgets remain.

134. **Bound the upstream OneShot cause and repair** (cross-session scope, temper06.2 block of 2026-10-19T08:38:28Z).
     Assign the upstream cause and evidenced driver repair to new skein-06.s1 in skein/06.s1-oneshot.md, under its explicit15minute/3probe budget and existing contracts. Complete reply receipt is unproved. io.md3.3 says “Failed may come at any time” and nothing follows but Closed; no io failure-semantics change is authorized. Distinguish a complete buffered reply from an incomplete reply before choosing a OneShot scheduling fix. A required contract change returns to the designer.

     Temper06.2 retains its clean JSON checkpoint; recording proof awaits the landed evidenced repair and a compatible skein repin/all-target check. No full recording during diagnosis. Existing required recording/facts/transcript assertions and budgets remain. The closing temper repin also awaits skein-06.s1; none has landed, so no repeated repin step is added.

135. **Correct recovery test shapes and check recurring setup** (hourly check of 2026-10-19T08:20:55Z).
     Reverses: nothing. Ruling126 keeps its contract-only scope; this follow-up amends the original02.4 fresh-success-per-rejection prose and final02-client counts after02.s1.
     Amends: README sessions/Rules, skein/02-client.md02.4 and Done when; adds skein/02.s2-recovery-tests.md and findings/recovery-{assertions,checker}.md.

     Assign skein-02.s2 after landed02.s1 for hourly03 findings2–6. Its assertion map preserves all72 original locations/cases and21 rejected causes. The21 fresh successes all use the same generic paired record, independently of the rejection input: one separately named generic success preserves that behavior. The failed instance's rejection, Keep Busy and persistence stay with every cause; Held and Remake get independent observations through one assertion-free setup.

     Add the nine independently derived exact-bound accepting records and precise4097-byte pull refusal; retain the old4101-byte pull refusal. Name value-identical clocks/counts from fixtures/design bounds. No production, limit, saved deadline, failure semantic, process proof or budget changes. Counts stay relative: recovery24->30; observed post-s1 client130->136,74 preserved+62new. Historical proofs keep their numbers.

     Repeated setup/size findings require the bounded Rust AST checker and explicit positive/negative controls in the supplement. Its controls cover only the reached mechanical shapes; row data, semantic behavior separation and true bound pairing remain manual mapped checks. Preserve02.s1's separate four-file checker. Refresh any further reached copy/count consumer and name it in the commit body.

     ## Checked

     - rg primary skein recovery.rs finds oversized_hints_read_nothing label dispatch, remake_releases_a_held_entry pre-Remake Held assertion; fixtures/recovery.rs::failed_restore constructs the same fresh paired(live()) for every rejection. LIMITS is rows4,inbox16,answer_bytes4096,op_bytes256,lifetime10s,margin1s,resources4,repositories2.
     - Python test-attribute count on primary6ab44a32a6e09e47bfdd88cc0e0ee7fee02b62dd: recovery24,client130. git diff6e290792..origin/main -- the two recovery paths is empty. The follow-up's all-five-repository name/content/count searches found only these behavior owners and the final02-client Done when count consumers; downstream restart tests are separate behavior consumers.
     - Follow-up scratch-hourly-03-recovery maps72/72 original assertion locations with unresolved0. Task876 compiled the AST checker and passed6 positive/7 negative controls. Task877 failed on three moved fixture:: prefixes; corrected sketch is UNCOMPILED, profiles UNRUN. No successful final implementation proof is inferred; owned workers ended and targets released.
     - Kit plan dispatch contains ruling/ready/ring; pinned testing-strategy.md10 supplies Rows carry data, One behaviour per test, The setup the assertion reads, A setup written once, Named fixtures and sizes, A bound's two sides. plan status shows closing temper-repin waiting, not landed: no extra closing repin is needed.

     ## Instructions

     Register skein-02.s2 using skein/02.s2-recovery-tests.md after skein-02.s1, with the normal ready check, a fresh implementation session and closing temper-repin dependency. Keep02.s1 and01.s1 scope unchanged. The recovered candidate/map/checker controls are review artifacts, not tested final code; the implementer must validate the corrected post-s1 source. No additional repin is added because no closing repin has landed.

136. **Recovery supplement Shapes against main** (ready check of skein-02.s2).
     Reverses: nothing; ruling135's scope, preservation mapping and budgets stay.
     Amends: skein/02.s2-recovery-tests.md Gate/Proof and findings/recovery-checker.md; no production change.

     Use the post-s1 ready check's corrected artifacts for the recovery supplement. Wire the bounded checker exactly once after the existing contract checker. In Shape::visit_item_fn, recognize assertions and Event::Remake through AST path/macro visits, not token-string matching; retain the six positive/seven negative controls and two additional string-decoy regressions. Apply formatting to both reached recovery files.

     The corrected client passes136 tests, retaining all72 mapped assertion locations,21 rejected causes, the single independent generic fresh-success case, nine exact accepts and both4097/4101 pull refusals. Counts remain N+6. The shared-load20–24ms observations do not establish the new cache/page/byte units'20ms ceiling: obtain isolated proof under pinned heavy --gate during implementation before claiming compliance or landing. Report any remaining miss without changing the ceiling, cases or budgets.

     ## Checked

     - Ready check snapshots: skein6ab44a32a6e09e47bfdd88cc0e0ee7fee02b62dd and temper8517cd677ae0e0a95a6aaafc3048163b01ee0137. No Files/symbol mismatch; the new helper/checker names belong to02.s2. All-five-repository name/content/derived sweeps found no additional code copy requiring repair.
     - rg scratch-hourly-03-recovery/gate-wiring.patch shows duplicate recovery checker invocations; gate-wiring.corrected.patch has one. Shape::visit_item_fn originally searched stmt token strings; corrected StatementShape visits AST paths/macros and passes two string probes. These are scratch artifacts, not landed source.
     - Heavy906PASS compiled checker and6 positive/7 negative controls, rejected original files and accepted correction. Heavy909PASS client all-target clippy; default136/136PASS0.320s, fuzzy0/browser0PASS sequentially. Recovery24->30; client130->136,74 preserved+62new. New bodies3–18lines; timing20–24ms under concurrent load remains open. No full gate/process/live or isolated timing proof claimed.
     - Kit heavy documents --gate at lines5/39 and parses it at119; kit plan provides ready/ruling/ring. All owned workers ended; only own completed targets removed. Artifacts retained at scratch-ready-skein-02s2.

     ## Instructions

     Mark skein-02.s2 ready immediately with the checked full hashes. Its implementer validates the corrected post-s1 source and isolated timing before land; normal landing gate remains. Keep all135 cases/controls,01.s1 scope and closing-repin dependency. Refresh any further reached copy/count consumer and list it in the commit body.

137. **OneShot supplement Shapes against main** (ready check of skein-06.s1).
     Reverses: nothing; ruling134's cause-before-repair, diagnostic budget and recording limits stay.
     Amends: skein/06.s1-oneshot.md Test shape, clarifying valid fixture feeding and the proof boundary.

     Build demand-correct HTTP fixtures: http/host/tests.rs::feed retains each pending Read and uses skein_lib::Intake::meet to supply only what that demand accepts. Preserve the original fragments/order; never invent End to finish incomplete input. Keep actual submitted operation tokens/buffers and valid completion order in the driver regression.

     The six scratch sketches prove fixture/public-boundary feasibility and leave the eight existing library identities intact. They do not prove the archived-write cause or the future OneShot scheduling repair. All134 diagnostic/repair conditions,15minute/three-probe limit, genuine incomplete-reply failure, scope boundaries and consumer recordings remain. Package counts stay relative; no final regression count is inferred from the feasibility sketches. Refresh any further reached driver copy and list it in the commit body.

     ## Checked

     - Ready check at skein6ab44a32a6e09e47bfdd88cc0e0ee7fee02b62dd / tempercfc50c4adcc0d8ca80515ac21bd767e5d4b2bf9f found no missing landed Files/signature and no extra driver repair target across all five repositories. host::exchange/wait and ExchangeHost interfaces are present; downstream uses need no public API/capture change.
     - Exact primary graph finds Intake::meet in crates/skein-lib/src/intake.rs132–153; rg verifies pub fn meet(&mut self, read:Read)->Option<Box<[u8]>>. Existing skein-http body/tests satisfy pending reads with intake.meet(read). Corrected scratch host/tests.rs::feed retains pending until meet succeeds; original whole-fragment fixture failed, not the production driver.
     - Pinned heavy904 exposed that fixture error after all-target clippy passed. Corrected917 passed all-target clippy and sequential --lib default14 (8 existing+6 sketches), fuzzy0/browser0 with --no-tests=pass; new sketches4–10ms. No live HTTP, cause investigation, repair, recordings, full gate or repin proof is claimed. Sources/patches/logs retained under scratch-ready-skein-06s1; owned workers ended and only own target removed.
     - Kit plan provides ruling/ready/ring. The source/config changes since checked6ab44a32 are fake peer/protocol only; throwaway/io/HTTP/lib files read by this check remain unchanged. Existing source fallback remains available for graph exclusions.

     ## Instructions

     Mark skein-06.s1 ready immediately with the checked full commits. Start its fresh upstream implementation under134's bounded diagnostic scope; feasibility sketches do not spend or enlarge its diagnostic probe allowance. Temper06.2 remains waiting for the evidenced landed repair before its repin/recording proof. Retain closing temper-repin dependency and all existing case/budget limits.

138. **Validate the corrected signed-number fixture** (process, skein01.s1 block of2026-10-19T09:02:55Z).
     Authorize final validation of the corrected two-byte numeric fixture under ruling 138. The plan requires preserving every input and expected assertion case; the existing signed-id input -1 has two bytes and must reach Malformed::Range { field: b"id" }. Task916 instead returned Json(NumberTooLong) with the one-byte fixture. The corrected NUMBER_BYTES=2 is a test-fixture derivation from that preserved input; its success remains unproved.

     The existing audit is complete and its three targeted runs are spent. Allow each of the four existing Proof commands once in the prescribed order: package clippy, then default, fuzzy and browser package nextest profiles. Stop on failure and report it; this grants no additional fixture investigation or iterative retry. The extension permits 15 minutes of command execution, excluding queue waiting. Continue the normal dependent checks and landing gate once these pass. Keep all36 mapped assertion cases, relative+16 count, shared-fixture ownership and existing per-test/suite ceilings; record any newly reached fixture caller or derived bound and its preservation in the commit body. Production and shipped profiles remain outside this test-only correction.

139. **One fresh archived-write receipt probe** (process, skein06.s1 block of2026-10-19T09:15:28Z).
     Authorize exactly one fresh-scratch pre-repair archived-write process probe under ruling 139. The session's Shape still requires proving whether the remaining archived-write reply reached io before Reset. Task934 establishes a driver defect for a complete supplied reply with submitted Recv tokens/buffers; it does not establish actual archived-write receipt. Tasks938/945 stopped during startup/nonempty-scratch validation, so neither supplies that missing evidence.

     Use the retained targeted cargo-test diagnostic through pinned heavy, outside the default unit suite, with a fresh run-owned instance directory. Preserve the prior scratch/evidence. Allow10minutes of command execution excluding queue, with this process bounded to180seconds and no retry or full recording. Keep the existing binary fingerprint and exact masked request comparison before the exchange; capture typed receive completions and framing evidence sufficient to decide whether the complete reply entered io before Reset. Remove temporary diagnostic code/prints afterward.

     If that trace establishes complete receipt and the same scheduling loss, the session's existing conditional OneShot repair and separate complete/incomplete/close/deadline regressions apply. Otherwise return the remaining uncertainty before changing production. Keep the original one post-repair archived-write comparison and temper06.2's later two-recording proof; this extra pre-repair probe replaces neither. Keep io/HTTP/shell contracts, request bytes, deadlines, bounds, public APIs and all test/profile ceilings.

     Add the README Diagnostic process attempts rule: a bounded process probe preflights a fresh instance path, its proper process harness and the exact request comparison; preserve prior evidence, count setup failures as attempts, and report budget exhaustion before another attempt. Refresh any further reached driver copy or diagnostic fixture and list it in the commit body. Propose this prerequisite rule for the retrospective, rather than changing the pinned kit during the run.

140. **Hourly04 graph coverage and named page fixtures** (steer of2026-10-19T09:26:37Z).
     Reverses: nothing.
     Amends: README Rules, graph-refresh hold and named page fixtures; skein/10-plans.md10.2.

     Hourly04 found new source beyond both reported primary graph refreshes. Index status reports skein09:00:04Z through6df8817b and temper08:57:37Z through3fe3cf80; checked primary HEAD skeinbd503f9e38ffdee4207224fcb0bd2219c1fccff9 and temper24756a9d0e03ecbd23b6fd4fa28a01c24ffe29bd, with temper origin/maina8e697c54c46e17cb76269a711d781d06a5290e1. Both worktrees are clean. Hold graph use for those repositories while the supervisor updates exact clean primaries and indexes them; scoped source discovery remains available. Report exact source coverage before lifting this hold.

     The new calls.rs page fixtures repeat a named-size finding. Testing-strategy.md10, Named fixtures and sizes, applies: correct page_size_is_passed_to_each_listing and remarks_are_sliced_without_wire_paging in skein10.2 using PAGE_SIZE and a named derived one-row fixture consistently in table sizes and expected slices. Preserve the byte strings b"1"/b"2" and every row/input/outcome; ordinary domain IDs remain data. Explicit conversions must preserve the u32 page-size and usize slice-bound types under existing lints. No test count, production behavior or budget changes. A bounded follow-up scopes the required recurring-mechanical AST gate supplement under development.md14.4; it will be registered with its own normal ready check before implementation.

     Checked: scoped rg and sed in primary forgejo/plans/tests/calls.rs find page_size_is_passed_to_each_listing's (1,b"1") row and remarks_are_sliced_without_wire_paging's REMARKS[..2]/[..1] rows. Shared forgejo/tests/fixtures.rs::PAGE_SIZE is u32=2, ROWS u32=8. Existing10.2 Files already include calls.rs; its current tests and fixture caller are the complete reached users. Refresh any further reached copy or derived size found by the correction and list it in the commit body. Pinned plan supports ruling/ring; the kit remainsd6d4abe and its --gate tool contract remains unchanged.

     The pinned numbers.sh::record_check at lines538–542 excludes .sN headings; the five pulse flags are supplement headings, not malformed coordinator reports. Record a retrospective proposal with ordinary-session/supplement/malformed-heading controls; keep the kit fixed. Process974 and three downstream checks remain pending and unverified; completion director806964 owns them. Hearth975 already passes. No other proof is repeated by this ruling.

141. **Hourly04 primary graph refresh verified** (steer of2026-10-19T09:36:44Z).
     Reverses: ruling140 of2026-10-19T09:26:37Z only its temporary graph-refresh hold; its named page fixture rule remains.
     Amends: README Rules, removes Graph refresh pending (hourly04).

     Both exact primary checkouts are now refreshed and verified clean at the indexed source commits. Lift the temporary graph hold: use project srv-data-git-runner-src-skein through f1a6219445776335612dc9a04c925aec388ea1bf, indexed09:35:55Z with27401nodes/202973edges; use srv-data-git-runner-src-temper through b0bce4541ffdb80535bf6581ce532e3d3d531c6f, indexed09:36:44Z with22791nodes/152502edges. Verify graph results against each actual session worktree. Later source changes require a new comparison under the permanent README freshness rule; later docs-only commits do not change source coverage.

     Checked: fetched origins, then git status --porcelain empty before fast-forward; git merge --ff-only origin/main updated both exact primaries. Following fast index_repository, index_status reports ready at the timestamps/counts above, and git rev-parse HEAD origin/main returns the same covered pair, with both working trees clean. search_graph finds both new plan-call test functions; scoped primary rg verifies those names and PAGE_SIZE:u32=2.

     Coverage gaps remain source fallbacks: skein3 partial files and8 intentionally hostile JSON parses, plus excluded docs/scripts/lock/binary artifacts; temper2 partial files and excluded docs/tools/scripts/lock/binary artifacts. Missing graph output proves no absence. No worktree index or kit move occurred. Keep ruling140's page-fixture correction and bounded check follow-up. Refresh any further reached symbol/copy using the actual source and record it in the implementation commit body.

142. **Compatible landed smith in the temper repin** (cross-session scope, temper05.2's block of2026-10-19T09:41:21Z).
     Authorize temper05.2's existing first repin commit to take the earliest compatible smith revision landed on origin/main when available, alongside the full skein update. Retain an already newer compatible smith source. This changes temper's Cargo.lock; decision13 still leaves smith's own skein pin to smith's next repin.

     The failure is an upstream interface mismatch: smith-session-world's Event::Call initializer lacks call, and its Request match lacks Piece/Hold. Latest fetched smith f925b18034c344f314adad1e3ac5907188d2dc2c still has the old initializer; no compatible revision is certified here. Independent peer migration continues. If compatibility remains unavailable, report the owning upstream plan/Step trailer for the coordinator's existing cross-machine await. Preserve all assertions, bytes, budgets and the unrelated jig design question.

     Keep one revision per git source and the full skein update. The existing workspace/all-target check and gate establish compatibility before the repin commit. Name old/new smith pins and the landed repair in that commit; refresh any further reached copy following that repair and list it in the body. Follow README's compatible smith source rule on subsequent temper repins; do not patch Cargo's dependency checkout.

143. **Named page fixtures and scoped recurrence gate** (hourly04 bounded check of2026-10-19T09:26:37Z).
     Reverses: ruling140 of2026-10-19T09:26:37Z only its pending supplement assignment; its named-page fixture correction remains.
     Amends: README session table and Named page fixtures rule; new skein/10.s1-named-page-fixtures.md; closing temper-repin dependency.

     Add skein-10.s1, session10.s1 in skein/10.s1-named-page-fixtures.md, after skein-10.5. It preserves the two named plan-call test functions and adds the bounded AST recurrence gate to the existing syn executable in separate listing-sizes mode. Its Files include calls.rs, scripts/recovery-shape/src/{main,listing_sizes}.rs and scripts/check.sh. Keep ruling140's correction if it already landed; no duplicate change to tests, counts or assertions. Normal ready check precedes implementation. Skein10's current session and increments remain unheld.

     The closing temper-repin waits for skein-10.s1. plan status and ledger show temper-repin waiting, so no repeat closing repin is added. Register the new step and dependency now; send it through normal check/ready before start.

     Checked: search_graph on project srv-data-git-runner-src-skein returns page_size_is_passed_to_each_listing and remarks_are_sliced_without_wire_paging in forgejo/plans/tests/calls.rs; git show origin/main of that path with scoped rg confirms three size rows2/8/1, eight listings and exactly four bare size/slice positions. The checker main's --controls dispatch and scripts/check.sh's existing recovery invocation exist; listing_sizes module/mode are the supplement's new additions. Scripts are outside graph coverage and were checked directly.

     Read-only bounded evidence: scratch-hourly-04-named/results.md and candidate.patch, skeinbd503f9e38ffdee4207224fcb0bd2219c1fccff9, temper8c481125bc8ab472a0e82535f71eaa3551336344. Pinned-heavy984, one attempt,21s active: rustfmt, checker/package all-target clippy -D warnings PASS; original source rejected exactly4 violations and corrected source accepted;7 negative/4 positive listing controls PASS; existing6 positive/7 negative/2 decoy recovery controls PASS; package135 default tests PASS0.163s, fuzzy/browser0 successful sequentially. No full gate, process/live proof, future10.2–10.5 proof or repin claimed. Scratch evidence/source/patch/logs retained; owned worker/target released.

     The checker claims only those bounded mechanical positions; assertion cases, semantic row behavior and fixture derivations retain explicit source review. The normal readiness audit re-resolves all post10.5 owners, copies and derived count users. Refresh any further reached copy and list it in the commit body. Existing budgets, production behavior and recovery/contract gates remain. Pinned kit plan supports ruling/ready/ring/bg and no kit move is authorized.

144. **Await the assigned live-text smith repair** (cross-session scope, temper06.2's block of2026-10-19T09:46:34Z).
     Ruling142 and README's compatible smith source rule already authorize temper06.2's forward smith source in its existing repin commit, subject to a landed compatible repair and unchanged all-target/gate checks. The same call/Piece/Hold mismatch in reports/temper-06-20261019T094528Z.md is covered; no new repair in a Cargo dependency checkout is authorized.

     The upstream repair is assigned already: live-text's smith-02.1. Its README rule6 and smith/02-best-effort.md02.1 name tests/session/src/world.rs, every fake-LLM Event/Request match and Event::Call.call. Register plan await coordinator live-text/smith-02.1 once for both temper05.2 and06.2. After that trailer lands, take the earliest compatible landed smith revision under142, retain a newer compatible source and the full skein main update, and establish compatibility with the existing proofs. Preserve all assertions, bytes and budgets; mechanical callsites from that landed repair follow README's repin repair rule, with further copies named in the commit body. A structural/behavioral difference returns a scoped report.

     Independent peer and JSON work/checkpoints remain authorized while the repair is pending. Required recordings wait for compatibility and retain their fresh scratch/facts/transcript proof. No duplicate upstream supplement is assigned because the live-text step owns the exact sites.

145. **Skein10.s1 Shapes against main** (ready check of skein-10.s1).
     Reverses: ruling143 of2026-10-19T09:26:37Z only the proposed SINGLE_ROW_PAGE/checked_sub prescription; retain the landed fixture and every other scope/dependency.
     Amends: skein/10.s1-named-page-fixtures.md Shape, names/derivation retained from landed calls.rs.

     The normal ready check found ruling140's fixture correction already landed with ONE_ROW_PAGE=PAGE_SIZE.saturating_sub(1) and page_rows/one_row usize::try_from conversions. Preserve those symbols and derivation; do not introduce SINGLE_ROW_PAGE, rename the fixture or alter either function's bytes. The supplement adds only the absent listing_sizes module, main dispatch and gate invocation unless a fresh reached-copy audit names a further user. All27 assertion cases (24 listing plus3 remarks), test identities, wire bytes and counts remain.

     Checked: git show b985f45b6226e0e285282f190d3f6ffd006df6c8:crates/skein-forge/src/forgejo/plans/tests/calls.rs filtered by ONE_ROW_PAGE/PAGE_SIZE/page_rows/one_row and the two test names confirms the actual constant and conversions. The read-only checker saw no executable downstream copies in smith/jig/hearth/temper; reached temper plan/count consumers are recorded in scratch-ready-skein-10s1/copies-and-counts.txt. New listing_sizes module/mode remain10.s1 additions; later10.3–10.5 behavior remains owned by its increments.

     Ready snapshots: skeinb985f45b6226e0e285282f190d3f6ffd006df6c8, temper2fffea684c940b9edf612c4a170ae6f559cb58e7; kitd6d4abeb22ae0c3a5e12e3c7a8978a30bdfb6421 unchanged. Pinned-heavy825 one attempt, wait1s/run4s: rustfmt and package/checker all-target clippy -D warnings PASS; listing mode7 negative/4 positive controls PASS, preserved original source rejected exactly4 scoped violations, current source accepted; recovery6 positive/7 negative/2 decoys PASS. Package144 default tests PASS0.073s, fuzzy/browser0 successful sequentially. Named existing tests8ms/4ms. Calls source remained byte-identical. Evidence, mappings, negative fixture, patch and logs retained under scratch-ready-skein-10s1; own worker ended/reaped, only own completed target removed.

     Mark skein-10.s1 ready at these snapshots immediately; its existing skein-10.5 and closing temper-repin dependencies remain. Proof reaches checker/package only: no future increment behavior, full gate, process/live or downstream build is claimed. Relative N->N and historical counts keep their meaning. Re-resolve any later moved inputs/copies at start, refresh further reached copies and list them in the implementation commit body. Kit plan's ruling/ready commands are supported; no kit move or extra gate run is required by this readiness ruling.

146. **Apply the designer's jig boundary answer** (design answer, temper02.1 block of2026-10-19T08:37:27Z).
     Apply the designer's answer landed in jig `9e9c350529327f5aa7b2bfad8569c9868b5a5a72` (design) and temper `b72efd046f8c020969ee4410c178ccc31a8e891c` (plan).


     The candidate's policy is the design. A dependency's name is evidence of a
     client crate only under a client prefix of jig's or of the checked project
     (by its whole name), less the records both ends read, and for skein's UI
     kit, by its crates' names. No other UI kit is inferred: one counts once
     section 15 names its crates. A `-client` in any other namespace,
     `skein-forge-client`'s among them, never seeds. jig `9e9c350` adds this
     to jig's `docs/design/README.md`, section 15 (and STATUS.md):

     > A dependency is a client crate by its name in two cases only:
     > - under a prefix this section gives, `<project>-client` or
     >   `<project>-<application>-client`, where `<project>` is jig or the
     >   checked project, by its whole name, … but not the records both ends
     >   read …;
     > - one of the UI kit's crates a client is built and tested on, by its
     >   name: `skein-ui`, `skein-ui-text`, `skein-ui-web`, `skein-fake-person`
     >   and `skein-fake-person-browser`; not the bundler, a tool.
     >
     > A name in any other namespace, skein's among them, follows its own
     > repository's rules, and says nothing here: a `<kit>-<service>-client`
     > there is a client of a service, which a crate in the engine's process
     > may use, and makes no crate that depends on it a client crate.

     It rests on section 15: "The project is jig, or an application under its
     own name"; "A crate that runs in the engine's process keeps the engine's
     name, even when it serves clients". For the run, temper `b72efd0` (README,
     Rules: "The boundary check's jig repair"; `temper/02-client.md`, 02.1):
     - The repair is jig's one change in this plan, a step the supervisor adds,
       on the diagnosis's candidate (`client_names.rs`, its eight fixtures): no
       system or forge word in jig; `breaches(root, project, allowed)` kept.
     - temper 02.1 takes it in its repin (`heavy cargo update -p jig-boundary
       --precise <the repair's commit>`); its migration, deletion and clippy go
       on meanwhile; its naming proof and landing wait.
     - `tests/client/tests/names.rs` does not change: empty `breaches`, the same
       two exempted crates, none exempted for using the forge client.
     - Nothing is deferred: no `later.md` row.

147. **Skein03 references after the component's start cutover** (started recheck of skein-03.1).
     Reverses: nothing; ruling120's owner-byte accounting, fixture bounds, token guard, route mapping and relative115/122/128 proofs remain.
     Amends: skein/03-connection.md03.1 landed marker/body sizing, historical/current fixture baselines and next-repin ownership; 03.2 hook-driver/signature and start-result references.

     Preserve the landed checked marker::length helper rather than resurrecting the old body formula. Ruling120's historical2,812,325-byte baseline remains historical; the changed pre-owner-value fixture measures2,812,377 in the recheck. Skein03.1 still measures the final owner-value sum after retaining the new owner bytes and holders; neither earlier number establishes its final bound. Plaintext's configured name is temper03.1's next-repin work, as temper01.4 says, rather than a landed unit-variant field.

     The new hook test driver retains full Settings and passes it to Hooked::pump; use a local lowercase digit table in signed_head because the former crate marker path is removed and upstream hex is private. The direct start test consumes forgejo::plans::start::Heard. Preserve request bytes, assertions, bounds, test identities/counts and deadlines. Refresh further reached copies/derived fixture consumers under the existing Files and list them in the commit body.

     Checked: recheck-skein-03-start/report.md and evidence/increment-audit.md cover03.1–03.3 at skein40d85476df1b696a9e4967c54b7c34cceed70163 and temper9a2eda75e47cd37749778df52ab691d38982e96c. Five-repository source/copy sweeps found no additional affected smith/jig/hearth forge user. Current inherited component identities112 remain; future additions stay assigned to03.1/.2/.3. Exact source verification: git grep marker::length in temper origin/main limits.rs gives line188; git show hooks/connection.rs232–253 gives pump(...,settings:&Settings,...); git grep enum Transport shows unit Plaintext in component/service; git grep Heard in skein plans/start.rs gives public enum Heard. temper/01-forge.md explicitly leaves configured plaintext name to temper03.1. Ruling120 has the same next-repin ownership rule.

     Proof: pinned-heavy reached component/world focused165 pass; consumer focused363 and fuzzy5 pass; earlier component/world fuzzy6 pass; skein-forge focused136 pass with empty fuzzy successful. The old-baseline assertion exposed the changed fixture sum and its corrected exact assertion passed. Scratch-only lint/fixture errors consumed the three-attempt budget; final lint and late hook-driver corrections remain unchecked. This report supplies no final clippy, future owner-value implementation, post-repin integration, full-gate/live/process or TLS contract proof. Sources, patch, exact commands and logs retained in scratch-recheck-skein-03-start; workers ended and only owned targets removed.

     Instructions: pass these plan corrections to skein03 and the temper03 ready checker. Normal implementation proofs cover the corrected references; no fourth supervisor executable attempt is authorized by this ruling. The separate TLS Name/IP Host question still blocks its dependent implementation/landing, and these corrections do not decide or authorize that public contract. Kit d6d4abe unchanged; rg kit/scripts/dev/plan confirms ruling --text and ring commands.

148. **temper03's Shapes against main** (ready check of temper-03.1).
     Reverses: nothing; rulings 120 and 147 retain the owner-byte requirement and historical/current baselines, and 142/144/146 retain the assigned smith/jig prerequisites.
     Amends: temper/03-connection.md, Needs at start, Choices, 03.1 Files/Shape and 03.2 maximum-head derivation.

     Make the existing migration inventory explicit: startup/application.rs::{Resolved,resolve} joins Files, and its two Webhooks occurrences plus forge::settings name the kit type directly. reconcile_settings uses temper_service::forge::USER_AGENT, as the sketch defines it. The component fixture called the shell's forges is crates/temper/src/lib.rs::forges; setup-shell tests/support/mod.rs::settings keeps reconcile Settings, its host and fixture agent, changing only component-limits paths.

     Carry skein03.1's checked retained-owner byte accounting into temper-service system::{Service::new,startup_worst_case,memory} and conformance component/hosted.rs::Hosted::new before consuming Settings. Follow the actual landed settings::owned_bytes/worst_case seam, retaining checked failure handling and separate TLS-root accounting. Refresh further reached startup/memory fixtures and derived copies and list changed users and the complete sum in the commit body; neither historical sum is the future total. This makes ruling120's existing requirement concrete in the consumer step and invents no absent signature.

     Correct the 03.2 derivation: the 253-byte configured name plus colon and five port digits gives a 259-byte Host and a 5,071-byte shipped worst head. Keep the fixture's 1,048,345-byte target edge, all admission limits, five then two new units, inherited test identities and relative gate counts. Remove the stale parenthesis saying README requires only03.2: it already requires skein03.3. No production change, new test or new step is authorized by these plan corrections.

     Checked: pump/ready-temper-03.md and scratch-ready-temper-03/{report.md,evidence/increment-audit.md} record full checked hashes: temper f7f0e0a5cf92394f14da7901df117b72af8ce1a6; skein 196f1330bd4624a579fd4133fcdefe1c6827fd8c; jig 9e9c350529327f5aa7b2bfad8569c9868b5a5a72; smith 256f4bc352ef81a76eb9948c92993f2841b01493; hearth 3dfe544d686f263d3c346a745a31507ef6ba1a95. Five-repository path/symbol/copy/derived-value searches found no further omitted forge consumer. All112 component identities remain; service forge proofs remain11+5=16 then+2=18.

     Checked source commands and exact outputs are retained in pump/ready-temper-03-premises.txt. git -C temper grep -n settings::Webhooks origin/main -- crates/temper-service/src/startup/application.rs crates/temper-service/src/forge.rs reports forge.rs:24 and application.rs:599/705. The scoped worst_case search reports system.rs:594 and hosted.rs:132, with startup_worst_case:532 and memory:550. The scoped reconcile/forges search reports forge.rs:224/261 and lib.rs:279. git -C jig grep -n 253 origin/main -- crates/jig-protocol-config/src/address.rs reports read::text(...,253) at54; skein host.rs:53/55 appends a colon and Decimal port except80/443. Arithmetic is259/5071/1048345. README row82 already names03.3; the setup-shell source returns reconcile::Settings with host and user_agent test. The graph lookup found no matching nodes; scoped source checks establish these premises. Kit scripts/dev/plan exposes ruling --text and ready; kit d6d4abe stays fixed.

     Proof: three pinned-heavy attempts ended. Corrected temper sketches passed all-target clippy, focused494/494 in5.796s and sequential fuzzy18/18 in14.043s; preceding baseline493/18 passed. The landed Host-helper oracle and skein-forge clippy/default158 passed; fuzzy/browser selected zero and passed. Exact checked skein repin succeeded, then clippy failed in smith-session-world for Event::Call.call and Request::{Piece,Hold}; this is the repair already assigned to live-text/smith-02.1 by144, not a new or waived failure. Post-update suites, future connection integration/deletion, final owner sum, head admission, recordings/process/full gate are unproved. Loaded25–62ms sketch timings do not certify the unchanged20ms unit ceiling. Sources, patches, scripts and logs remain; all owned workers ended and only their two completed targets were removed.

     Instructions: mark temper-03.1 ready with these checked hashes after this ruling lands. Readiness does not waive its skein03.1/temper02.1 needs, the already assigned compatible smith repair/pin, jig01.s1 naming/landing prerequisite, or the unanswered TLS Name/IP Host design contract. Normal implementation retains every proof and budget; no fourth supervisor attempt or upstream dependency-source patch is authorized.

149. **hourly05 graph refresh pending** (hourly check of 2026-10-19T10:32:03Z).
     Reverses: nothing; the permanent Graph freshness rule and all existing implementation prerequisites remain.
     Amends: README Rules, temporary Graph refresh pending (hourly05).

     The primary graph indexes precede newer main source. Until the supervisor reports their refreshed commits, do not use the code graph for skein, temper or jig; search with rg and verify in the actual worktree. The supervisor refreshes only the exact primary roots, not session or scratch worktrees. The temporary rule changes no step dependency, proof, budget or kit.

     Checked: index_status for srv-data-git-runner-src-skein reports root /srv/data/git/runner/src/skein, ready, indexed2026-10-19T09:35:55Z; temper reports its matching primary root, ready,09:36:44Z; jig reports its matching root, ready,2026-10-14T15:14:39Z. The last reported covered source commits were skeinf1a62194 and temperb0bce454. Scoped git rev-parse HEAD origin/main and git status --short show clean main checkouts with equal pairs: skein5c8ae967570b8cfc33f10c83cdfc5495e3b42e99, temperb9d0ebaedd32856d99067b396369483a687bec9e, jigc415911cc6e73d2359a30db896dc87e00d28b202. Since the last report, moved source includes the plans tests, temper owner adapters and landed jig classifier repair. Current index diagnostics retain partial/excluded-file fallbacks. Kit scripts/dev/plan exposes ruling --text and ring; no kit move.

     Instructions: pass the README temporary source-search rule to the coordinator and any resumed implementer until the supervisor reports its removal. The hourly checker already uses source fallback for later source; no additional test, gate or executable attempt follows from this rule.

150. **hourly05 primary graphs refreshed** (hourly check of 2026-10-19T10:32:03Z).
     Reverses: ruling149, hourly check of2026-10-19T10:32:03Z, only its temporary graph-refresh hold. All other rules and dependencies remain.
     Amends: README Rules, removes Graph refresh pending (hourly05).

     The exact primary indexes have refreshed. Lift the temporary hold, using srv-data-git-runner-src-skein through5c8ae967570b8cfc33f10c83cdfc5495e3b42e99, indexed10:35:04Z with27598nodes/204634edges; srv-data-git-runner-src-temper through89d2db94520ac6807c559797a32fd908787fd1a5, indexed10:35:36Z with23285nodes/157063edges; srv-data-git-runner-src-jig throughc415911cc6e73d2359a30db896dc87e00d28b202, indexed10:36:18Z with57765nodes/480989edges. Verify every result in the actual session worktree. Later source changes require comparison under the permanent Graph freshness rule; source searches remain available.

     Checked: pump/hourly05-graph-snapshots.json records exact roots, clean trees and HEAD=origin/main before indexing; a repeated git rev-parse HEAD origin/main and git status --porcelain after all requests returned the same three clean pairs. index_repository(mode=fast,persistence=true) completed for those primary roots only; index_status reports each ready at the timestamps/counts above. Structured diagnostics are in pump/hourly05-graph-results.json. search_graph(project=srv-data-git-runner-src-jig,file_pattern=tests/boundary/src/client_names.rs,name_pattern=under_client|seed) now returns under_client54–61 and seed64–68; primary rg independently reports the definitions at54/64.

     Coverage gaps still require source fallback: skein3 partial files and8 intentionally hostile JSON documents; temper2 partial files; jig skips crates/jig-client-protocol-wire/src/generated/v9.rs on a parse timeout. The tools also exclude directories and lock/binary artifacts by policy. No graph absence proves source absence. No worktree index, code change, test retry or kit move occurred. The capped functions.wait calls during indexing were tool-yield handling, not coordinator work-completion polling.

     Instructions: pass the refreshed commit coverage and remaining gaps to the coordinator and resumed implementers; remove only149's temporary restriction. Keep all test/proof layers, named fixtures and upstream prerequisites. Refresh further reached symbols/copies from actual source and record changed consumers in the implementation commit body.

151. **Hourly05 plan contracts and exhaustive repository matches** (hourly05 bounded check of 2026-10-19T10:32:03Z).
     Reverses: nothing; historical increment proofs remain. Adds two independent contract tests to final coverage.
     Amends: README Sessions, Rules and Repin; skein/10-plans.md delivery wording and final count consumer; new skein/10.s2-plan-contracts.md and findings/plan-shapes.md.

     Add skein-10.s2, session10.s2 in skein/10.s2-plan-contracts.md, after landed skein-10.4, with normal ready check before start. It owns hourly05’s three findings, under testing-strategy.md10 “One behaviour per test” and “The setup the assertion reads”, and programming-model.md10.2: “match, exhaustive” and “if let and let … else on Option and Result only.” The README’s Plan contracts and repository matches rule is standing under development.md14.4. It does not wait for10.5/08.1. Serialize shared checker/gate edits with10.s1; prefer10.s2 first without adding a hold to existing starts.

     In edges.rs split the literal subscription equality into subscriptions_match_the_literal_contract; keep the delivery-mapping loop. Move all three positive valid_word assertions into marker_words_at_both_bounds_are_valid; retain marker sizing/encoding/decoding and refusal cases. Preserve the landed assertion map in findings/plan-shapes.md:22 subscription names in one equality;15 delivery rows over13 headers with two assertions per row;3 validity cases;9 marker assertions,3 exact-fit expects and3 one-short refusal rows. The parent plan’s “22 events” wording is clarified as these separate literal-list and mapping contracts; this correction adds no delivery behavior. Relative count is N+2; parent historical41/48 proofs remain, later runtime/final checks include +2.

     In reads/done.rs and writes.rs::Plan::check_repository replace only the two sampled partial own-Document branches with explicit exhaustive matches, preserving read validation before keep_document, write resolving gate, overflow/error precedence, mismatch refusals, Locate and Q12 cache outcomes. Add the bounded syn plan_shapes module/--plan-shapes mode only for the four named test functions, with semantic AST assertions/calls and mechanical/decoy controls. Retain existing recovery/listing modes and controls; no generic style scanner. All files, assertion map, scoped controls, proof and limits are in the supplement. Refresh further reached copies/count consumers and name them in the implementation commit body.

     The closing temper-repin also waits for skein-10.s2. plan status/ledger show that repin waiting, so no repeat repin is added. Normal readiness remains required, and feasibility is not proof of future10.5 or a waiver of any required increment proof.

     Checked: root search_graph at primary project srv-data-git-runner-src-skein found Document, reads::Plan::done/check_repository, writes::Plan::check_repository and both named tests; git grep -n on origin/main verifies these locators. Exact commands/output are retained in pump/hourly05-shapes-premises.txt. Checked skein5c8ae967570b8cfc33f10c83cdfc5495e3b42e99, smith e2b9f45afb4687125fabef12a8cd280f7ba25fe4, jigc415911cc6e73d2359a30db896dc87e00d28b202, hearth3dfe544d686f263d3c346a745a31507ef6ba1a95, tempercd08d58382e443830978ad7b1c06dbec60a245f2. Both new test names and plan_shapes mode/module are supplement additions, not absent landed prerequisites.

     Checked: scratch-hourly05-plan-shapes/{report.txt,assertion-map.md,reconciliation.md,copies-and-counts.txt,candidate.patch} retain all mapped cases and downstream searches. No executable downstream copies were found; active final-count consumers are README, skein/10-plans.md and the skein10 handoff. Refresh the handoff’s final-count instruction, preserving its historical41 proof. Historical base findings are not rewritten. The scratch combined driver exercised both pre-existing checker modes and the new mode; implementation retains10.s1’s listed ownership if its dispatch has not landed yet.

     Checked proof: two pinned-heavy attempts ended; the first exposed function-length clippy and helper extraction corrected it. Second: checker/package all-target clippy and rustfmt PASS; original file rejected for exactly2 scoped violations; new6 negative/6 positive controls PASS; recovery6 positive/7 negative/2 decoys and listing7 negative/4 positive PASS. Sequential skein-forge default170/170,0skip,0.065s; fuzzy/browser0 selected PASS, maximum unit12ms. Package168->170 and calls/edges41->43; no future10.5 proof. Own workers ended and target removed; no primary/run-state/kit/index edits. Full gate at implementation land, existing ceilings retained.

     Checked commands: kit/scripts/dev/plan lists set/ready/ruling/ring/wake; kit/scripts/dev/heavy supplies the compilation/test wrapper. Root exact rg outputs are retained in pump/hourly05-shapes-premises.txt. Kit stays d6d4abeb22ae0c3a5e12e3c7a8978a30bdfb6421.

     Instructions: register skein-10.s2 (skein session10.s2) in check for skein/10.s2-plan-contracts.md, Needs skein-10.4, before implementation; add it to existing closing temper-repin dependencies. Request its normal ready check immediately. Pass on the README rule and the landed assertion map to skein10 and the supplement implementer; refresh the yielded skein10 handoff’s future count and shared-file ownership. Keep existing10.5/10.s1 prerequisites and budgets. No extra repin step or kit move.

152. **Skein10.s2 standalone checker readiness** (ready check of skein-10.s2).
     Reverses: only ruling151 and its steer of2026-10-19T10:52:58Z insofar as they unconditionally require listing mode/controls before10.s1 lands. No other finding, scope, assertion, ordering or count changes.
     Amends: README Plan contracts and repository matches rule; skein/10.s2-plan-contracts.md Gate/Controls/Proof/ordering; reciprocal mode preservation in skein/10.s1-named-page-fixtures.md.

     Skein10.s2 stays independent after10.4. It adds only its named Files and plan-shapes mode, preserving actual landed recovery. Preserve and re-run listing mode/controls only when present from10.s1; do not introduce listing_sizes.rs or --listing-sizes as a prerequisite or extra implementation. When10.s1 later edits the dispatcher/gate, preserve and run any present plan-shapes mode/controls. Serialize those shared files and rebase retaining every present mode. Preserve the README rule, assertion map, exhaustive repository matches, historical41/48 proofs, relative+2 and final50, all ceilings and closing-repin dependency.

     Checked: primary skein origin/main5c8ae967570b8cfc33f10c83cdfc5495e3b42e99; git show origin/main:scripts/recovery-shape/src/main.rs | rg -n 'fn controls|fn main|--controls|listing|plan_shapes' prints controls148/main162/--controls165 only. git show origin/main:scripts/check.sh | rg -n 'recovery-shape|listing|plan-shapes' prints only the recovery invocation14. listing module/mode are10.s1 additions; plan_shapes mode/module, the two contract tests and reads::Plan::check_repository_document are10.s2 additions.

     The fresh ready check checked all of this single increment at skein5c8ae967570b8cfc33f10c83cdfc5495e3b42e99, smithe2b9f45afb4687125fabef12a8cd280f7ba25fe4 (then53b9d6db52ff6cbe999212160e49056cb8ded44e; bounded source delta contained no reached copy), jigc415911cc6e73d2359a30db896dc87e00d28b202, hearth3dfe544d686f263d3c346a745a31507ef6ba1a95, temperbb1f788ddc7cff37f4d63c578dbec8a0ad9eb928. The agent verified graph locators in scratch; adopted prior unchanged assertion mapping/source searches, not the combined-driver executable result.

     Checked proof: scratch-ready-skein-10s2/report.txt and plan-amendments.patch retain the precise correction. Its one new pinned-heavy attempt824,20261019T105444-824.log, PASS (0s queued/3s run) used standalone actual recovery plus only10.s2 Files, with no listing module/dispatch or manifest/lock change. Rustfmt and checker/package all-target clippy -D warnings PASS. Corrected plan-shapes PASS;6 positive/6 negative controls PASS; original exactly2 violations; recovery6 positive/7 negative/2 decoys PASS. Sequential package default170/170,0.052s, fuzzy/browser0 selected PASS with --no-tests=pass; maximum11ms. Four existing Q12 tests passed freshly. No future10.5, full gate/process/live/recording/downstream proof claimed. Worker ended before own target cleanup.

     Checked run: ledger has skein-10.s2 in check and temper-repin waiting with10.s2 dependency; coordinator acknowledged all four151 points at2026-10-19T10:52:58Z. The later smith move was not a readiness input for this source correction; it creates no new smith compatibility claim. Kit/scripts/dev/plan lines22/669/671/672 supply ready/ring/ruling. No kit move or duplicate gate.

     Mark skein-10.s2 ready at these checked snapshots immediately after this ruling. Send the reciprocal dispatch/controls instruction to coordinator, skein10 and supplement implementer, and refresh the skein10 checkpoint's future-count/ownership notes. Closing temper-repin dependency remains; no repeat repin. Refresh reached copies/count consumers and list them in the implementation commit body. Preserve historical base findings and unchanged10.5/10.s1 Needs.

153. **TLS Name typed reading and IP Host answer** (design answer to skein03.1 block of2026-10-19T09:47:53Z).
     Apply the designer's answer in skein1908a365559463adf86582cce7ca0ff88c684387 and temper10e0a568d4dc2ad2407dddd15e4ab1b63794ec9e.

     Landed: skein 1908a365 (design: tls.md 2, 3.4, 6; forge.md 5.1;
     forge-connection.md 3.1, 3.2, 7.5; STATUS.md) and temper 10e0a568 (plan:
     skein/03-connection.md 03.1, and 03.2's head test).

     1. The accessor is typed and borrowed, not owned text. tls.md, 3.4:
        "Whoever holds a name reads it back with `Name::server`, as what it
        is, borrowed: `Server::Dns`, the DNS name's text as it was given, or
        `Server::Ip`, core's `IpAddr`. A name is one or the other, and reading
        it allocates nothing". The listing: `Name::server(&self) -> Server<'_>`.
        `ServerName::to_str` is not used (tls.md, 6).
     2. A TLS IP name is the Host, written as an address is. forge.md, 5.1:
        "A TLS server name that is an IP address is written as the address's
        IP is, with the address's port, whichever IP the address holds: IPv4
        in dotted decimal, IPv6 in brackets as its eight groups in lowercase
        hexadecimal without leading zeros, never compressed
        (`[2001:db8:0:0:0:0:0:7]`); a DNS name is written as it was given."
     3. One path for check and send. forge-connection.md, 3.1: "a DNS name
        goes to `host` as the name, and an IP name in place of the address's
        IP, with the address's port … An IP name that differs from the
        address's IP is the `Host`, as a DNS name is: the name names the
        forge, and the address only reaches it. Start's check and each request
        make it by the same function, from the same values, so they agree."
        3.2: "A TLS name is refused by none of them".
     4. Accounting, 7.5: "each forge's credential, secret and name, a TLS
        name's DNS text or a plaintext name (an IP name holds none); and the
        word and the user agent". 03.1's `owned_bytes` counts them.
     5. Unchanged: TLS IP names stay admitted; `skein_forge::host`, decision
        25's port rule and the plaintext name's refusals; temper's seam (it
        hands `Transport::Tls { server_name, trust }` and reads nothing back).
     6. 03.1 now names `crates/skein-tls/src/{config,lib}.rs` and
        `src/tests/config.rs`; `Name` keeps which of the two it holds so that
        `server` is total over rustls's non-exhaustive `ServerName`; the wiring
        check adds 192.0.2.7 on 127.0.0.1:3000 -> `192.0.2.7:3000`, 2001:db8::7
        on :443 -> `[2001:db8:0:0:0:0:0:7]`, ::1 -> `[0:0:0:0:0:0:0:1]`.
        Nothing is deferred.

154. **Settled smith adapters at the next compatible repin** (shared cross-session scope, temper02.1/05.2/06.2 blocks of2026-10-19T10:57:10Z–10:58:25Z).
     The three reports expose one already settled contract. Smith's live-text decisions2.2 says "Those paths become v3 in jig's repin". Its Design questions4 accepts "live: None and an arm that ignores pieces", answered(a) by the director on2026-10-19; For the user9 confirms that choice. Its later.md assigns these exact adapters to the tools closing repins "or their next repin of smith". Channel design3 says an absent ask "asks for nothing, and nothing is sent"; host design2 keeps live content outside the domain. No new live policy or design question is needed.

     Add forge-kit/jig-01.s2, jig/01.s2-smith-adapters.md, with normal readiness before implementation. It is jig's next compatibility repin, after the already-landed tools/jig-repin: move smith and skein together to compatible landed sources, one revision each, retaining a newer compatible source. Repair the four channel-v2 consumer files to v3, both Start literals to live:None, and worker_route::{process,channel} to explicitly ignore only OpenEvent::Live alongside the existing ignored variants. Keep fields, cases, assertions and limits; refresh only upstream-version framing/codec-derived values that the landed contract requires. Its complete Files, proofs and copy rule are in the supplement.

     Assign temper's same next-repin share once, to the existing temper02.1 repin commit: its three Start literals, worker_route::{process,channel}, and the service encode_start-derived proof, as named in README and02.1 Files. This uses the existing next-repin assignment rather than waiting for tools/temper-repin or adding another temper supplement. Retain live-text/smith-02.1's compatible repair, the full skein update, jig01.s1 naming repair, one revision per source and every existing proof. No live display, persistence or domain forwarding is implemented.

     Temper02.1's first compatible repin waits for jig01.s2. Temper05.2 and06.2 retain their independent peer/JSON checkpoints and wait for both jig01.s2 and temper02.1, then rebase the single landed adapter sweep before their own all-target checks and required proofs. The schema/path changes in this ruling authorize no bounds increase, omitted assertion, bypassed gate, or suppression of another event. A further structural/design difference returns a scoped report.

     The closing temper-repin is still waiting and also needs jig01.s2; no completed closing repin of this run requires repetition. The tools run's future closing temper repin observes the source landing and avoids duplicating this sweep. Existing live-display deferrals in live-text/later.md stay; this ruling defers no new work.

155. **Complete the shared adapter session copies** (derived plan copy correction for ruling154).
     Reverses: nothing; completes the derived session-copy updates of ruling154, with its scope and ordering unchanged.
     Amends: temper/02-client.md02.1 compatibility Files/repin Shape; temper/05-fakes.md05.2 Needs; temper/06-throwaway.md06.2 repin rule.

     Ruling154's landed README and jig01.s2 supplement already state the full settled next-repin policy. Its three derived temper session edits were omitted after an exact Markdown-context assertion failed before writing them. This landing applies those copies:02.1 names its four compatibility Files and settled adapter sweep;05.2/06.2 wait for jig01.s2 and temper02.1 and retain every existing proof/checkpoint. No new API policy, implementation scope, test expectation or budget is added.

     Checked: git show --stat7d328f67a93f123bd95fa82f795264a495c64266 names README.md, decisions.md and new jig/01.s2-smith-adapters.md, with no temper session changes. Fresh scoped reads found02.1's unindented Shape label,05.2's original Needs and06.2's old smith-repair ownership sentence. All replacement contexts were checked before any of these three writes; git diff --check passes. The source/copy evidence remains ruling154's scratch-shared-adapters/evidence.txt; this correction changes no source premise.

     Follow ruling154 and these completed session copies; keep the new jig step in normal check, not ready, until its fresh check/ruling. Give the checkpoint sessions their updated plan text alongside154. The existing unblock/Instructions remain authoritative and no second unblock or duplicate proof is needed.

156. **jig01.s2 Shapes against main** (ready check of jig-01.s2).
     Reverses: nothing; completes ruling154/155's mechanical-copy inventory without changing its contract or dependency order.
     Amends: jig/01.s2-smith-adapters.md Files, framing Shape and Size.

     The fresh jig01.s2 ready check found one further framing fixture: testing/jig-test-service/src/tests/worker.rs::start_on_pipes constructs its ScriptedPeer at channel version2. Add it to Files and change only that argument to3 alongside the four channel-v3 consumers; retain every other argument and schema-derived control. The supplement now names seven Rust files plus Cargo.lock. Its existing copy rule remains: refresh any further reached copy or derived value within the settled contract and name it in the commit body; report a further structural/design difference before expanding scope.

     Checked: git -C jig grep -n -A15 -B3 'fn start_on_pipes' c415911cc6e73d2359a30db896dc87e00d28b202 -- testing/jig-test-service/src/tests/worker.rs finds the helper at164, schema-derived kind.largest+8 sizing at174 and ScriptedPeer::new(...,2,...) at179. The primary graph query returned no matching node; exact landed source supplies this premise. scratch-ready-jig-01s2/full.diff changes that argument2 to3 with all other arguments preserved. Bounded plan-tree search found the old six-file count only in this supplement; the existing README rule already covers further framing copies. Downstream temper's three Start literals, two Live matches and encode_start proof remain assigned to temper02.1.

     Validation: scratch-ready-jig-01s2/report.txt records all five full snapshots, three completed pinned-heavy attempts, exact compatible smith/skein updates and one revision per upstream source. Workspace/all-target check, four affected packages' all-target clippy -D warnings and formatting pass. Reached16-package suites preserve all identities/counts:1173 default pass8.903s,95 fuzzy pass24.237s, empty browser passes sequentially with --no-tests=pass. CallName remains16 bytes, smallest Start42→43 via None tag, charterv2/119 unchanged. Non-lock delta+17/-14; no limit, assertion, behavior or count waiver.

     Accept the check with this plan correction and mark jig01.s2 ready using its recorded snapshots. The full repository gate stays at implementation landing; process/live/recordings, wasm/browser application tier and downstream temper compilation were not run or claimed. All checker workers ended before its own14GB target was removed. Other sessions' gates/proofs, checkpoints and154 dependency order remain unchanged.

157. **Hourly06 primary graph refresh** (hourly check of 2026-10-19T11:37:40Z).
     Reverses: nothing; temporary application of the README Graph freshness rule to newly stale source and a missing index.
     Amends: README Rules, Graph freshness, until the hourly06 refresh is reported.

     Hourly06 compares each exact primary root with its graph coverage. Do not use the code graph for skein, smith, jig, hearth or temper; search with rg until the supervisor reports refreshed coverage. Existing starts and all proof/limit/ownership requirements continue. The supervisor refreshes only these primary repositories, never worktrees or dependency checkouts; later source still needs verification in the actual worktree.

     Checked: pump/hourly06-graph-pre.json retains list_projects pages0–500 matching exact absolute roots and index_status diagnostics. Four required primary-name indexes exist: skein/temper indexed10:35, jig10:36, smith2026-10-13; primary origin/main source has moved since. No index matches the hearth primary root, and its exact primary project returns not indexed. Alias indexes for some other primary roots do not replace the workspace's exact project names. pump/hourly06-graph-snapshots.json records clean primary HEADs lagging their origin/main and exact paths. Pinned plan-check.sh27–40 authorizes fast-forwarding a clean main that is only behind; development9.6 forbids moving backwards. Before indexing, apply only that clean ancestry-checked fast-forward to existing origin/main and record the commit indexed; never discard local work or create a source commit. If cleanliness/ancestry fails, leave that checkout unchanged and retain source fallback.

     Use supported index_repository on the resolved primary roots, explicitly preserving project names srv-data-git-runner-src-{skein,smith,jig,hearth,temper}; report readiness, gaps and pre/post HEADs. No kit or source behavior changes. Hourly/process checks continue independently and no extra suite is justified by index metadata updates.

158. **Director's coordinated Cargo org cutover** (director instruction of 2026-10-19).
     Reverses: nothing; the director’s 2026-10-19 org/source instruction governs future Cargo URL changes, including older literal examples in the plan.
     Amends: README Rules, The director’s Cargo org cutover.

     Record the director’s instruction as the standing rule for this run. GitHub’s org is now fabrig-dev and primary remotes already point there, but Cargo treats the forgetic and fabrig-dev URLs as distinct sources. The director owns one coordinated change of smith’s, jig’s and temper’s Cargo.toml and Cargo.lock after temper’s tools part 1 ends; pins stay unchanged. Keep forgetic in all Cargo source lines until that change lands; use fabrig-dev in all such lines after it. New dependencies copy their manifest’s existing URL. On a source-line rebase conflict, take main’s Cargo file and rerun the step’s authorized repin. A two-skein-source gate error means a forgetic line remains and must be renamed. No supervisor Cargo edit or pin change is part of this ruling; existing gates, source-unification checks and settled adapter ownership continue.

     Checked: pump/director-org-url-check.json records git remote get-url origin and git grep -n -E 'github.com/(forgetic|fabrig-dev)/' origin/main -- Cargo.toml Cargo.lock for all five primaries. All remotes use fabrig-dev. smith/jig/temper currently have respectively 53/105/204 forgetic source matches and zero fabrig-dev; hearth has16/0 and skein0/0. These checks confirm the current pre-cutover phase, not a completed Cargo migration. The director’s instruction names smith/jig/temper as the coordinated change and does not authorize an independent supervisor migration of any repository.

     Checked commands: pinned kit/scripts/dev/plan contains plan ruling --text at21/499 and plan ring at19/669. No new build, gate or repin command is introduced. The new README rule covers every current and later dependency addition and source-line rebase; refresh any further literal source copy reached by the director’s cutover, and list it in that change’s commit body.

159. **Hourly06 primary graph coverage refreshed** (graph refresh after ruling157 of 2026-10-19T11:40:58Z).
     Reverses: ruling157’s temporary graph hold (2026-10-19T11:40:58Z), lifted only through the refreshed source inputs below; its standing source-verification/freshness rule remains.
     Amends: README Rules, Graph freshness: remove the temporary hourly06 prohibition.

     All five exact primary projects are ready after supported index_repository refreshes; use the graphs as locators through their recorded source inputs and verify every result in the actual worktree. The primary-only restriction, source fallback for gaps and later changes, and missing-node-is-not-absence rule continue. No worktree/dependency checkout was indexed and no kit, production behavior, proof or budget changed.

     Recorded primary inputs and index_status readiness (UTC; nodes/edges):
     - srv-data-git-runner-src-skein:92e177c3165f937df5d51138c2d0974e15e033e0,11:42:09,28262/211509.
     - srv-data-git-runner-src-smith:7b4dfc07cc64c125514f2ce5caa662442789f4ff,11:43:03,27156/193724.
     - srv-data-git-runner-src-jig:eceadcaf8d5b19a2495c5bf81355232f512c9f1f,11:45:13,57767/481008.
     - srv-data-git-runner-src-hearth:292f8a60670315a58e0aec77281dc7e51cbe92f2,11:45:50,12696/75142.
     - srv-data-git-runner-src-temper:cd472781947898ab3be0c57635c22f97f68ca5c9,11:52:06,23600/159782.

     Checked: pump/hourly06-graph-index-inputs.json records clean ancestry-checked fast-forwards before the initial refresh; graph-results.json and graph-post.json retain results/diagnostics. The first temper refresh raced nine later paths; detect_changes from7a992b10f6a32b3450b9103dfeaba57e0c272383 listed those paths, and a second supported refresh at clean cd4727819 resolves that known movement (pump/hourly06-temper-reindex.json). Other primary heads stayed at the recorded inputs in graph-post-heads.json. Later source is not claimed covered.

     Known gaps remain: skein3 partial files and8 intentionally hostile JSON parses; smith1 partial reference patch; jig generated/v9.rs parse timeout; hearth12 partial/14 unusable Jinja fixtures; temper2 partial mockup/proof-script ranges. Deliberately omitted lock/golden/binary files remain source-only; diagnostics are not completeness guarantees. graph-post.json records exact paths/ranges. Preserve source fallback for any additional omission and list a newly relevant gap in the session’s report.

     Checked locators/source: search_graph found skein-tls config.rs::Name::server/Server; smith-agent-service service.rs::Service::lost_live_bytes; jig-test-service tests/worker.rs::start_on_pipes; hearth-domain-builder lib.rs::Builder; temper-service system.rs::memory. Exact rg checks in each primary confirmed lines205/185,753,164,23,550 respectively. A search for encode_start as a function returned zero, while rg found its call at temper-service tests/worker.rs:53; this is recorded as a locator gap, never absence. Locator results are in pump/hourly06-graph-locators.json.

     Pinned plan still supports ruling --text and ring (scripts/dev/plan21/499 and19/669). The coordinator passes the refreshed input bounds and fallback rule to running/resumed sessions; no extra suites or repeated completed checks follow from index metadata alone.

160. **Derive temper02 owned-example memory from the complete migrated service** (plan scope, temper 02.1 block of 2026-10-19T12:24:09Z).
     Authorize the derived owned-example correction in temper02.1's existing users commit. This is a plan-scope conflict, resolved by the design's explicit definition: setup.md8.1 says “Its `memory_bytes` hold its worst cases wherever a person lays it and whatever its forge's files hold”, with each variable string at its bound. Configuration.md7 requires the complete service sum to fit and refuses the start otherwise. The example allowance must follow that sum; operational limits and their refusal checks stay.

     The checked cause is skein02.3's landed `Domain::identity_used` field, implementing deployment binding. On switching client implementations, the inline client grows from 592 to 600 bytes; the containing Service grows from 29,920 to 29,928. `system::memory` counts that holder once. At identical startup inputs the complete `startup_worst_case` sum is 78,675,187,547,016 before the users cutover and 78,675,187,547,024 after it. The payload and all subsequent terms are unchanged. This is a traced representation change, with no accounting duplication or client regression found.

     Add `docs/examples/owned.json` and the unchanged `serve_prepares_once_on_the_owned_example` regression to 02.1's coverage. Change only the derived `engine.memory_bytes` allowance to the freshly calculated complete sum; do not change operational caps, settings, assertions, accounting code or test identities. If rebasing changes the calculation, rederive the complete sum and explain its changed terms rather than adding a stale eight-byte delta. The README makes this evidence requirement standing; an unexplained size/profile difference remains a question. Record old/new full sums and cause in the users commit; refresh further reached copies/derived values and list every user in its body.

     The scratch proof changed only that JSON value and the existing regression passed in 0.07s. Its users are the owned support helper, startup/configuration/process tests and box guide's copy command; no additional executable numeric copy was found across the five repositories. Keep their behavior and assertions. Complete remaining reverse dependents, process preparation/proof and gate before landing. No proof is waived: the 986 originally unrun reverse-dependent cases and full gate/process remain unproved by this diagnosis. Rulings148's later owner accounting and158's director-owned Cargo cutover remain separate and unchanged.

161. **Refresh hourly07 primary graph coverage** (hourly07 graph comparison of 2026-10-19T12:43:08Z).
     Reverses: nothing; the standing Graph freshness rule applies to new source movement after ruling159's recorded inputs.
     Amends: README Rules, Graph freshness, with a temporary hourly07 source-search requirement.

     All five primary repositories have advanced beyond ruling159's recorded index inputs. Until the supervisor reports their refreshed coverage, do not use the code graph for skein, smith, jig, hearth or temper; search with rg. Existing starts, proof requirements, limits and ownership continue. Refresh only the primary checkout indexes, preserving their exact project names; never index a worktree or dependency checkout. Verify every later locator against the actual worktree, and use source searches for gaps or later changes.

     Checked: pump/hourly07-graph-pre.json records exact-root index_status diagnostics; its git.head_sha is the current checkout, not proof of index coverage. Ruling159 records the last actual refreshed source inputs. Fresh origin/main heads are skein9d444e33d1997003b6f2935f86fbd27622f12643, smith7ec31926b1d7f7ce45a344e73545d9d7f03073ef, jig149970b999bf2ffb639c2e3398b842084525afab, hearth5efba43e0a036628119cb5892fe5c6de2a2eb319 and temperfc728645e8ba82502d4e2150147631d7967252be; all primary checkouts are clean main and ancestors of those heads. Before each refresh, recheck cleanliness/ancestry and apply only the supported fast-forward in pinned scripts/dev/plan-check.sh27–40; record the exact indexed commit. Preserve local work if any check fails.

     Use index_repository on the resolved primary roots with names srv-data-git-runner-src-{skein,smith,jig,hearth,temper}. Report readiness and known parser/coverage gaps, with pre/post commits. No kit move or production change follows from metadata maintenance, and no extra suite is justified by it. Pinned plan supports ruling --text and ring at scripts/dev/plan671/669.

162. **Diagnose pinned Jig UTF-8 provenance before further process proof** (cross-session scope, temper02.1 block of 2026-10-19T12:44:46Z).
     Authorize the bounded scratch-only diagnosis in 02.1's new exception, under ruling162. The preserved retry proves that invalid UTF-8 reached pinned Jig's documents::small::text; it does not yet prove which layer supplied the bytes or which code must change. Trace that provenance from the actual failing tree and pins, comparing the last passing source, and seek a deterministic lower-layer reproducer before assigning a repair. Budget: 20 minutes and three executable diagnostic attempts through pinned heavy and explicit bash script files; compile/setup failures count, queue waits grant no extra attempts. No further full process run, recording or gate precedes the diagnostic report and next scope ruling. No production repair lands under this exception, and no UTF-8 assertion, input payload, error, bound or proof is relaxed. Preserve both original process attempts and the four-commit implementation; map any further reached caller or copy in the report. Required process and gate proofs remain outstanding. A fresh implementer thread performs this diagnostic only, then returns the evidence and proposed owner/location or uncertainty; any design gap goes to the designer under development.md10.4.

163. **Report hourly07 refreshed primary graph coverage** (hourly07 graph refresh completion).
     Reverses: ruling161's temporary graph prohibition, 2026-10-19T12:49:00Z, and steer2026-10-19T12:50:00Z point1, only through the refreshed source inputs below; the standing freshness and source-verification rule remains.
     Amends: README Rules, Graph freshness, removing the temporary hourly07 prohibition.

     All five exact primary projects are ready after supported primary-only index_repository refreshes. Use the graphs as locators through the recorded inputs, checking every result in the actual worktree; source searches remain required for gaps and later source changes. No worktree or dependency checkout was indexed, and no production, kit, proof or budget changed.

     Recorded primary inputs and index_status readiness (UTC; nodes/edges):
     - srv-data-git-runner-src-skein:dae2ab6bc9ea8eb16b86fcaf9ae95cd062ab2da5,12:52:27,28409/212268.
     - srv-data-git-runner-src-smith:83eaa2ae173ba34c2d68b2cb06d175cd97136af2,12:52:49,27207/193775.
     - srv-data-git-runner-src-jig:149970b999bf2ffb639c2e3398b842084525afab,12:53:29,57767/481008.
     - srv-data-git-runner-src-hearth:5efba43e0a036628119cb5892fe5c6de2a2eb319,12:53:43,12696/75142.
     - srv-data-git-runner-src-temper:efe462a251ea681f1d193830c32d7b95b3a6d48f,12:54:02,23613/159797.

     Checked: pump/hourly07-graph-index-inputs2.json records the clean ancestry-checked primary fast-forwards; graph-refresh-results.json and graph-post.json retain refresh and readiness diagnostics. graph-post-heads.json confirms the same clean heads afterward. The git.head_sha field alone was not used as index evidence; the recorded source inputs bracket the explicit refreshes. Later origin/main landings are outside this coverage claim.

     Known gaps remain: skein3 partial files and8 intentionally hostile JSON files; smith1 partial reference patch; jig1 skipped generated parser file; hearth12 partial/14 unusable template fixtures; temper2 partial mockup/proof-script ranges. Omitted lock/golden/binary files remain source-only. Diagnostics are not completeness guarantees: preserve source fallback for any further omission and name a newly reached gap in the session report.

     Checked graph/source locators: search_graph found skein connection.rs::start_backoff_doubles_to_its_maximum, jig documents/small.rs::text, smith service.rs::lost_live_bytes, hearth lib.rs::Builder and temper system.rs::startup_worst_case; exact rg checks confirmed lines506,10,753,23,532. pump/hourly07-graph-locators.json retains results. Pinned plan supports ruling --text and ring at scripts/dev/plan671/669. Pass the refreshed input bounds to running/resumed sessions without repeating completed suites for metadata changes.

164. **Scope hourly07 connection test-shape corrections** (hourly07 follow-up of 2026-10-19T13:10Z).
     Reverses: nothing. Amends README’s Rules/Sessions/closing-repin prerequisite, skein03’s final count and test-shape handoff; adds skein/03.s1-connection-shapes.md and findings/connection-shapes.md.

     Add skein03.s1 for hourly07’s two sampled shape findings. testing-strategy.md10’s “One behaviour per test” separates backoff progression from terminal immutability; “Named fixtures and sizes” puts the sampled capacities beside their premises. Preserve19 assertion sites/83 evaluations, every input/outcome/wire value and inherited test. The scoped AST recurrence check follows development.md14.4 and preserves all landed checker modes/controls.

     This authorizes a plan step and its normal readiness, not implementation before readiness. The preparatory checker/controls passed; its final package correction is uncompiled, and default/fuzzy/browser remain unchecked after three attempts. A fresh ready check gets at most20 minutes/two pinned-heavy explicit-bash attempts to resolve these. No process, recording, live run or extra downstream compatibility rebuild is in that readiness scope.

     Checked: `rg -n 'fn (each_listing_uses_its_largest_answer_row|start_backoff_doubles_to_its_maximum|an_idle_connection_is_reassigned_to_another_forge)'` on frozen skein sources finds the three sampled functions; graph locators were checked against those sources. `assertion-audit.txt` reports19 sites/83 evaluations, six→seven identities. `changed-files.txt` lists exactly the six scoped files, including new connection_shapes.rs. The new terminal function/constants belong to this supplement.
     Checked: hourly07-followups commands/results identify each of three heavy attempts; checker controls passed, package clippy failed before the final source-only correction. Bounded five-repository consumer audit found no sampled-test copies; temper’s112 inherited identities/deletion delta−107 stay. Ledger shows forge-kit/temper-repin waiting, so no additional closing repin is needed.
     Checked: pinned kit plan dispatch contains ruling/ready/bg at671/672/685; heavy and explicit-bash commands are retained in the three attempt scripts. No mutable Cargo, production or dependency source edit belongs to this supplement.

     Instructions: coordinator adds skein-03.s1 in check with its new session file after skein-03.3; normal readiness precedes implementation. Closing forge-kit/temper-repin also waits for it after its existing prerequisites. Preserve historical increment counts; final connection count gains relative+1. Refresh further reached copies/derived counts and name each in the commit body. Existing starts and temper’s deletion proof remain unheld.

165. **Record skein03.s1 readiness against current main** (ready check of skein-03.s1, 2026-10-19T13:20Z).
     Reverses: nothing. Amends skein/03.s1-connection-shapes.md’s readiness evidence and findings/connection-shapes.md; scope and budgets stay as ruling164.

     The normal ready check found no blocking plan mismatch. Its final fixture correction independently compiles; preserve the nineteen-site/eighty-three-evaluation map,128→129 relative+1 and temper’s112 inherited identities/deletion delta−107. Historical preparation failures and128 proofs remain historical. The gate is scripts/check.sh; no scripts/recovery-shape/check.sh exists or is required.

     Checked: report pump/ready-skein-03s1.md freezes skein9fbda6b7076a20b5dcf387bbbc47453a3cea84a0, smith9d4430e956b558571687642291769afd884e520b, jig39c1e4db9dc923b0a2063468fa5533fa3c50113a, hearth61ab3d16477c4dc97b10d4b7dc2ac6e0cddf7ef0 and temper79eff1cddcfe45e9584415c5338d8449cd73904e. Name/source/copy audit found no additional sampled-test consumer; new names belong to03.s1.
     Checked: one pinned-heavy explicit-bash attempt20261019T131645-824 exits0; format, both all-target clippy commands, connection5+/9-/decoys, original exactly3violations, recovery and plan-shapes controls pass. Sequential package profiles129/0/0 pass; focused0.102s, sampled units4/3/3/13ms, ceilings unchanged. No full gate/process/recording/live/downstream build was claimed.
     Checked: pinned plan supports ready/ruling at672/671. Readiness worker ended before removing only its completed target; detailed commands/ownership retained in scratch-ready-skein-03s1.

     Instructions: mark skein-03.s1 ready now with these snapshots; coordinator starts after its landed03.3 prerequisite. Preserve all shared modes/controls present at implementation, rechecking listing if it lands meanwhile; serialize common-file edits without holding other starts. Refresh further reached copies/derived counts and list each in the commit body. Existing closing forge-kit/temper-repin waits for03.s1, with no additional repin.

166. **Apply the bounded landing-result text answer** (designer answer, temper02.1 block of2026-10-19T12:59:50Z).
     Apply the designer's answer landed in temper6d4741a4b1f1f3655f615c9bc9cdc8a7ff9cbeb5 and jig8cc7e79f9ba72d64eac30597b8fe07f4f628eeb6:


     Landed with `land --docs`: temper 6d4741a4b1f1f3655f615c9bc9cdc8a7ff9cbeb5 (design
     root.md 6, forge.md 8.2, STATUS.md; plan temper/02-client.md 02.1; later.md) and
     jig 8cc7e79f9ba72d64eac30597b8fe07f4f628eeb6 (contract.md 5.3, STATUS.md).

     1. The words are the merge's lowercase hex id via `commit_text`, shortened to
        the task's own bound. temper root.md 6: "A landed change's result is a change
        of the forge's connector and kind, its resource the pull request, and its
        words the merge commit's id as text: in lowercase hexadecimal, as a
        delegate's commits are (below), shortened to its first digits, as a short id
        is, when the task's contract bounds its words under the id's 40 or 64. …
        never the commit's bytes; the whole id stays the change's and its pull
        request's (`forge.md`, 8.2). The short id keeps any bound the task's creator
        chose: a bound under the id's digits is not refused, and the result never
        passes it."
     2. Owner: temper's translation, `change_decision` (forge_route.rs), through a new
        `landing_words(merge, bound)` beside `commit_text`; `bound` is the `words` of
        the task's own `Contract::Change` (read as `link.rs:26` reads it). jig's
        assertion stays.
     3. Bound: 64 digits fit the ceiling (`result_bytes`, 512 in jig's small profile;
        `batch.rs:426`) but not every task: the creator picks the bound, no floor, and
        the forge-kit tests use 32, where a whole id is refused (`invalid_result`) and
        the task stays due. No bound, test value, encoding or `STORE_VERSION` changes.
     4. Retained raw words: test artifacts only (the run's attempt stores and logs);
        temper's formats are pre-release, no deployment's store must outlive a
        release. Keep the failed artifacts as they are; no fixed tree restores them.
     5. 02.1 gains commit 5, "The landing's words", on the four-commit checkpoint:
        unit `landing_words_are_the_merge_id_cut_to_the_contract_bound`; the
        frozen-worker forge_routes.rs test asserts its words are the first 32 digits
        of `State::Landed { merge }`; joint "plus 6 (927 from the base's 921)",
        check.sh "less 70 (… 1809 …)", new proof `heavy cargo nextest run -p
        temper-engine-domain forge_route::tests` "plus 1 (2 from the base's 1)"
        (`git grep -c '#\[test\]' origin/main --
        crates/temper-engine-domain/src/route/forge_route/tests.rs` prints `…:1`).
        Don't: more digits than the bound, a raised bound, `logs/temper-02/` touched.
     6. Deferred: jig 5.3 now lists "a task's result's texts … refused as a result its
        contract does not admit wherever the tasks child admits a result"; its check
        has a later.md row and jig STATUS.md's.

167. **Require inherited licenses in new workspace members** (director instruction, 2026-10-19).
     The director’s 2026-10-19 instruction requires every new skein or smith workspace member to put `license.workspace = true` in `[package]`, beside its other workspace-inherited fields. This applies to moved crates as well as newly written crates; no gate checks it yet. Jig and temper follow when accounts licenses their workspaces.

     Add README’s standing **Workspace license inheritance** rule and state the manifest requirement in every remaining member-adding increment: skein06.2 (`tests/forge/conformance/Cargo.toml`), 07.1 (`tests/forge/client/Cargo.toml`), 07.3 (`tests/forge/setup/Cargo.toml`), 08.1 (`tests/forge/machine/Cargo.toml`) and 09.1 (`tests/forge/protocol/Cargo.toml`). Forge-kit adds no smith member. The already-landed skein members were included in the director’s license change; their historical creation steps need no replay. Preserve inherited metadata when refreshing a destination manifest, refresh any further reached member and list it in the commit body.

     This is plan metadata scope only: no new step, gate, repin, runtime behavior, proof count, budget or start hold. Existing Cargo source rules, independent proof obligations and the unanswered process-tree question remain in force.

     Checked:
     - `git -C skein show --format=fuller --no-patch 1222e7b` identifies `1222e7b73ff33d446e68cd364b8370a6df5207a9`, “License the workspace and all members under MIT”, with `Plan: accounts`, `Step: skein-01.1`; `git -C skein merge-base --is-ancestor 1222e7b origin/main` exits 0.
     - `git -C skein grep -n -E '^\[workspace.package\]|^license = ' 1222e7b -- Cargo.toml` prints `59:[workspace.package]`, `60:license = "MIT"`. `git -C skein show 1222e7b:crates/skein-forge/Cargo.toml` shows `[package]` with `license.workspace = true` beside version/edition/rust-version/publish inheritance.
     - Scoped `rg -n 'Cargo\.toml|\[package\]|new crate|new package|[Mm]anifest'` over all forge-kit skein session files, reconciled with `ledger.tsv`, finds the five remaining member additions above; later increments edit those existing packages. The checker remains a standalone excluded workspace, not a new workspace member.
     - `git -C <supervisor docs worktree> diff --check` exits 0; only README and the four session documents change before this ruling. Markdown-only landings use `land --docs` under temper’s workflow1; no Rust invocation is required for these plan edits.

168. **Apply the designed healthy-stop and whole-tree crash endings** (designer answer to process tier question of 2026-10-19T13:27:18Z).
     Apply designer commits skein f4dc2e8b88dd0b3deb71ae1145008f114d5c86b7 and temper e7f32d1253932c5e7ebf10bfe6a3f1024033f89d through ruling 168.

     Written fresh: the Q3 answer was consumed with its question. Landed with
     `land --docs`: skein f4dc2e8b88dd0b3deb71ae1145008f114d5c86b7 (shell.md 13,
     examples.md 6) and temper e7f32d1253932c5e7ebf10bfe6a3f1024033f89d
     (domain/testing.md 2.6, protocol/setup.md 7.4, STATUS.md; plan temper/02-client.md 02.1).

     1. Two endings. The restart is a **whole owned-tree crash**, not a leader
        crash. testing.md 2.6 step 5: "the binary crashes: its whole tree is
        killed at once, as a supervisor ends a crashed service's (skein's
        `shell.md`, 13), and once nothing of it runs, it is started again on the
        same directory". Every other end is a healthy stop: "the test sends
        `Terminate`, the engine drains (`root.md`, 9.2), the service closes what
        it started, and the binary exits 0 with none of its tree running."
     2. Why whole: production cannot end a crashed leader's children (no
        parent-death signal). skein shell.md 13: "A crash closes nothing … A
        supervisor that restarts a crashed service first ends its whole tree".
     3. Descendants and verdicts. At a stopped exit, none may run
        (`EndsWithBinary`). A leftover is temper's failure, in its shutdown (the
        drain, the worker's end, io's group close). After the crash, examples.md 6:
        "A process its kill leaves keeps the tree from settling and fails the
        test at its deadline, the kit's failure, not the binary's". No-orphans
        stays whole: "none runs at a stopped binary's exit, and none of the
        crashed tree when the binary starts again" (2.6, the referee).
     4. Seams: the harness, `crates/temper/tests/support`. 02.1 gains commit 6,
        "The stories' endings", on the five-commit checkpoint. `run_story` takes
        `Ending::{Stop, Crash}`: `run` stops, `run_until_landed` crashes.
        `ChatJudge::act` sends `Signal::Terminate` (mod.rs:396), and `observe`
        asserts `Exit::Code(0)`. The crash starts with
        `Binary::start_with_tree(…, Expectation::Supervise, false)`, sends
        `signal_tree(Signal::Kill)`, and asserts `!stderr_overflowed()` and
        `counts()` before the restart. `sign_in.rs:257` sends the engine
        Terminate. No test is added or renamed. This authorizes exactly that
        tree signal, that expectation and that substitution.
     5. Proof: 02.1's process proof as before (19 pass), plus a Done-when:
        `heavy rg -n '\.signal\(Signal::Kill\)' crates/temper/tests` prints
        nothing. A stopped leftover is a finding, never rerun as load. Don't kill
        a healthy leader, change a stop's expectation, or add a backstop or
        time. Nothing is deferred.

169. **Hourly08 graph source fallback** (hourly check of 2026-10-19T13:50:58Z).
     Hourly08's fresh comparison found all five recorded graph inputs older than fetched origin/main. Until the supervisor refreshes the exact primary indexes, do not use the code graph for skein, smith, jig, hearth or temper; search with rg. The temporary README rule records this fallback for running and future sessions. This does not hold implementation or alter proof requirements.

     Checked: hourly08-graph-preflight.json records git rev-parse HEAD/origin/main, git status --porcelain, the previous recorded index inputs and git diff --name-only for each repository. All five primary checkouts are clean main branches and can fast-forward to the fetched origin/main. index_status confirms each exact project's primary root; its live head_sha is not treated as the index's input. The pinned plan script provides ruling --text and ring.

     The root supervisor owns refresh and removal of the temporary rule; implementers retain source verification for every graph locator and any further reached copy.

170. **Hourly08 graph refresh and accounts compatibility owners** (hourly check of 2026-10-19T13:50:58Z).
     The five primary graph refreshes completed successfully. Remove ruling169's temporary Hourly08 graph refresh rule; the permanent Graph freshness rule remains. Refreshed source inputs are skein 2bd29bd29d63d733005396132f94731465c1a64c, smith 014b134b905696fcc8162880928a84a2ab9801a4, jig 8cc7e79f9ba72d64eac30597b8fe07f4f628eeb6, hearth 61ab3d16477c4dc97b10d4b7dc2ac6e0cddf7ef0 and temper d272f68d14dec9c8002ea40e729257c29bf99558. Use exact primary project names and verify every locator in the actual worktree. Later source landings require comparison and source fallback. Parser gaps, including jig's generated wire v9 file, still require direct source searches; a missing result proves no absence.

     Hourly08 separately found jig's OAuth literal incompatibilities at crates/jig-protocol-accounts/src/handed_in.rs, crates/jig-protocol-config/src/profile/small.rs and crates/jig-protocol-accounts/src/tests/fixtures.rs. These are already named, with tests/protocol-accounts/src/fixtures.rs, by accounts/skein-05.2's Files and Downstream; accounts' closing jig-repin owns the downstream sweep. Report this evidence to that existing owner through the director. Keep the accounts/smith-01.3 await for smith's separate interfaces; its landing alone is not downstream compatibility proof. This assigns no OAuth production repair to forge-kit, adds no speculative retry and preserves independent work and all existing checks. Refresh any further reached copy or derived value within the owning repin and list every user.

     Checked: hourly08-graph-inputs.json records clean main fast-forwards before indexing; hourly08-graph-refresh-results.json and hourly08-graph-post.json record all five successful exact-root refreshes. Post-input HEAD comparisons are empty; the index-created untracked hearth persistence artifacts were removed after completion, with their checksums recorded, leaving primary source trees clean. Hourly08's jig-check.log and temper-check.log contain E0063 at the three named jig literals; source verification confirms them. Accounts' README closing-repin table and skein/05-oauth.md 05.2 explicitly inventory the repair. Smith's archive-only missing-git build-script error is a check setup limitation, not a production finding. No process change is adopted and kit d6d4abeb remains pinned.

171. **Bounded diagnosis of the pinned HTTP call table** (cross-session scope, temper02.1 block of 2026-10-19T14:01:32Z).
     Ruling 171 assigns a supervisor-owned, scratch-only lower-layer diagnosis
     under pinned supervisor.md9 and development.md10.4's authority over scope.
     The commit-6 process proof passed two stories, failed the restart story before
     its requested ending, and left sixteen unchecked; this is a new HTTP call-table
     finding, with no established shared cause with the earlier UTF-8 or child findings.
     Preserve the clean six-commit checkpoint92425da290227502bc853359bc0925c374deca09
     and all original instances, stores and logs. It has the same tree as the process
     source8afc00ec8862245661dd72f06e5dcce0331be07c. The actual Jig pin is
     eceadcaf8d5b19a2495c5bf81355232f512c9f1f; a new main diagnosis alone does not
     establish its historical cause.
     The checker owns twenty minutes and at most two executable diagnostic attempts,
     using pinned heavy with explicit bash script files. Setup/compile failures
     consume attempts; queue waits authorize none. Trace actual pinned call admission,
     lifetime, terminal events and capacity between server and mount, compare the last
     passing source, and prefer a deterministic lower-layer Rust reproducer that
     respects the server's event contract. An arbitrary over-capacity input is not
     cause proof. Return at most twenty lines with full hashes/pins, established cause
     or uncertainty, smallest valid reproducer, proposed repair owner/location, exact
     commands/results and task/target ownership. Map further reached copies and derived
     values for any proposed repair and list them; retain active targets until workers end.
     No repair, dependency repin, full process retry, recording or gate is authorized by
     this diagnosis. Preserve all limits, assertions, payloads, errors, deadlines,
     existing proof requirements and the original failed artifacts. The six-commit
     step stays held until the report and a further scope ruling; use a fresh
     continuation after W49's114K checkpoint when implementation is authorized.

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
