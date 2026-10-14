# protocol: decisions

The decisions this plan rests on, each with its reason; the supervisor's
rulings during the run follow them, numbered from 101; the last section
lists what the user may veto. The planner took these on 2026-10-14 under
the user's delegation (development.md, 2.2 and 10.4).

## The plan's decisions

1. **One plan, temper only, the design's thirteen steps.** It builds
   protocol.md, section 12, steps 1 to 13, and the domain changes of its
   section 5 ("Kept"), and moves the engine worlds onto jig's real store
   (store/README.md, section 6; testing.md, 4.5). It changes no other
   repository: the design needs nothing upstream (protocol.md, section
   11), and jig's test-budgets plan moves jig's, smith's and skein's
   mains on another machine. temper keeps its pins, jig `d91a89c4`,
   smith `dfc0fe57`, skein `b5d3a2c3`; a step that would need more is a
   question, never a repin.
2. **The plan is named `protocol`,** the name `STATUS.md` already gave
   it, in `docs/development/plans/protocol/`.
3. **The audit's design gaps were closed first,** in a Markdown-only
   landing before the plan (`Design: close the gaps the protocol plan's
   code audit found`): a failed preparation reported as a fact of the
   attempt, since jig's core keeps no attempt's failure detail; only the
   identity's token is secret in its `Debug`; the adoption's outcome kept
   durably in a sixteenth family, `Application`, at byte 15, as jig asks
   of a connector's keyed requests; jig-view's `Cell` role and a status
   with no text in the terminal's table; the fake git program hosted by
   skein's world harness; the sweep's `trash/<n>` counted from a number
   drawn at each start, its failed pass resumed at the next Make or
   start, and `sweep_bytes` at least the 255 bytes io's listing needs.
   `findings/README.md` maps every finding. The session writers met
   smaller gaps where the design is silent on a mechanism; each is a
   question to raise with its default in its session file (for example
   06's cap on a list inside one answer, 13's missing connect and
   handshake deadlines, 08's `PartyInput::SignOut`).
4. **Eighteen sessions, 81 increments; at most three sessions at once**
   on `x1` (8 cores, 15 GB, about 8 GB free; pueue's gate lane 1, build
   lane 2). The README orders them for short slots. 81 steps is above
   the 60 that jig-protocol's retrospective (proposal 3) suggests for a
   plan; the dependency structure lets the supervisor stop at a session's
   end, and `later.md` holds nothing that this plan's steps need. The
   session writers split increments past about 600 lines (04.4, 07.3,
   08.2, 08.5, 09.1, 10.1, 15.1, into `a` and `b`, or four for 08.2) and added
   16.6 (the terminal's adoption, after the landed change). Session 08
   keeps its nine increments in one session: the second half needs the
   first, so two sessions would run them in the same order, and the
   handover by context (README) bounds each thread.
5. **Adding a crate of jig, smith or skein keeps the pins** by three
   `cargo update --precise` commands, jig first, then smith, then skein
   (code-organization.md, section 3). Cargo otherwise moves a git
   source to its newest commit when a package it does not hold yet is
   added; the planner saw it move all three, and the three commands in
   that order put them back.
6. **Crate kinds.** Every crate is step code but the binary `temper`
   (ordinary Rust, after jig's `jig-test`), `temper-shell-terminal`
   (ordinary Rust, after `jig-shell-native`; its `route`, `render`,
   `command` and `limits` modules are step code held to the subset by
   code-organization.md, section 6's check, since one crate has one
   lints table), the test support crates, the tools and the worlds.
7. **The git component's file operations are kernel operations.**
   skein-io's file layer renames within one directory only
   (`file_layer.rs:576-583` at the pin), while git.md, 4.1 moves a
   workspace into `trash/`. The component issues `skein_io::kernel::Op`
   `Rename`, `Remove`, `MakeDirectory`, `Open` and `List` itself, through
   the worker's submissions, as jig's reference worker does for its
   root's close. It spawns git through io's records.
8. **temper's application parts plug in where jig's reference leaves
   them empty:** the four `parts` sites of jig's reference service, the
   store's `Put`, `Remove` and application interval, the clients'
   application family, the application credentials' table
   (`findings/jig-service.md`, section 1). temper writes the application
   member's parser with `jig_protocol_config::read`, and its own
   formatter of problems, one line each with its path.
9. **`temper-service` takes typed settings.** `startup` (10.1) parses
   configuration's bytes into `temper_service::settings`, and the engine
   (08) and the worker (09) are built from those values, so their tests
   need no document. This is the interface between parsing and
   composition, not a transitional constructor.
10. **Transcripts are raw bytes, one directory per case:** Forgejo's as
    `request` and `response` (HTTP/1.1 as sent and received, masked),
    git's as `argv`, `exit`, `stdout` and `stderr`. Consumers read them
    with no parser beyond their own. The tools that record them are
    ordinary Rust and may use `serde_json` to mask JSON bodies; no step
    crate links it.
11. **The live tier's login is borrowed by a Rust tool,**
    `tools/live-login`, which `scripts/proof-live.sh` runs: jig's script
    read it with Python, which this plan does not write. Its budget is
    the design's per run (25 completions, 15 minutes) and the README's
    for the plan (3 runs, 50 completions); it runs only after the same
    stories passed against the fake peer.
12. **The throwaway Forgejo is v16.0.5,** downloaded by the planner from
    codeberg.org's release page and verified against its published
    SHA-256 (`2c548bf1bfa09a441a3a674b1dd4959d77576816ad50534c23496908a3ea5a19`),
    kept at `~/src/rust/tmp/temper/forgejo/v16.0.5/`; it started on
    loopback with SQLite in a scratch directory in about two seconds.
    The Actions facts' runner is forgejo-runner v12.13.2, verified the
    same way (`2b4c3751fc2f9a60f753ef97414949de25587dfa2b921870b0cb6cb7c9bf7080`),
    the last release before the runner's v13, whose breaking changes
    postdate Forgejo v16. The user's own Forgejo instance and its 7.0.12
    binary are never touched.
13. **Scratch directories are never under `/tmp`:** tests use
    `CARGO_TARGET_TMPDIR`; the tools take their directory as an argument.
    jig's process tests used the system's temporary directory; temper's
    do not.
14. **temper writes its own world driver for jig's store** (18.1): jig
    ships none that is not tied to its test application. It reuses
    `jig_test_store_files::Files` for the transcripts' file requests the
    component makes in memory mode.
15. **The forge component replaces the routing's unavailable answers in
    one place** (14.1). Until then the engine's routing answers every
    `Request::Forge` with `Error::Unavailable` at cost 0, as protocol.md,
    section 12, step 2 says; 14.1 removes that arm.
16. **The client is copied from ops's,** as client.md, section 3 and
    jig's `client/domain/root.md`, section 12 say: ops's client root and
    view root, its `tests/ops/src/client/app.rs` for `tests/client`'s
    `App`, and its `tests/ops/src/joint/` for the `Front` over temper's
    engine root, on `jig-client-world`'s kit. The forge connector's
    number is given at construction, as client.md, 3.1 says.
17. **The native client is temper's own** (12.1): jig's `NativeClient`
    is test code bound to jig's test application and panics on sign-in
    paths temper needs (`findings/jig-client.md`, Discrepancies 1 to 3).
18. **The new worlds' shares of the suites** are code-organization.md,
    section 5's guides, summing to 3.8 s of the focused suite and 24 s
    of the fuzzy one, within testing.md, 6.1's 4 s and 25 s. The base
    gate ran 682 focused tests in 1.236 s and 31 fuzzy in 7.849 s
    (`heavy --gate ./scripts/check.sh` at `cc2697cf`, 2026-10-14).
19. **Sizes:** every increment lands within 1.5 times its Size without
    asking, a codec or document increment within twice it
    (jig-protocol's retrospective, proposal 6, and its rulings 132 to
    214).
20. **The web crates go in one increment with the stories that replace
    them** (11.4), as client.md, section 9 says ("in the same change").
21. **The design's open questions keep their defaults:** repositories
    keyed by Forgejo's id; one period per project at the first start;
    a renamed repository keeps its path and remote; the merge style is
    the repository's default; the side that hands the core a delivery
    encodes its evidence; `Landed` capped by the forge's limits; a run's
    live text waits for smith's channel; nothing of temper's restarts
    `temper serve`; two clients on one directory are not supported; no
    run saves (the root names no saved-work branch, so the routing sends
    neither `save`).
22. **The large files are not split in this plan** (later.md): an
    increment adds at most 40 lines to each, and its new code goes in a
    new module beside it (README, "Large files").
23. **The adoption form's defaults** are client.md, 4.4's: the forge by
    number (1), home checked, prefix `temper/`, CI checked.
24. **The plan's commands, run once at the base** (development.md,
    5.4), on 2026-10-14 at `cc2697cf` with this plan's Markdown on top,
    through `heavy`:
    - the gate, `heavy --gate ./scripts/check.sh`: passed, 682 focused
      tests in 1.236 s, 31 fuzzy in 7.849 s, 25 s alone;
    - "Done when" 1: `plan: no active run on this machine`, as expected
      before the run; 2: no commit yet carries `Gate-Result:
      scripts/check.sh passed` (temper's last code landing predates
      `land`'s current trailer), as expected; 3: 22 paths missing, all
      of them, as expected; 4: 133 lines name the web crates, as
      expected; 5: the two Python files, as expected; 6: holds, the
      three pins; 7: 4 manifests name `jig-fake-store`, as expected; 8:
      `scripts/proof-one-process.sh` does not exist yet, as expected;
      9: no `temper-15.4` commit, as expected; 10: no `facts.txt`, as
      expected; 11: nextest refuses the filter, "no packages matched
      this" (exit 94), as expected; 12: holds at the base (no Rust file
      reads an environment variable or spawns a thread; `Cargo.lock` is
      unchanged);
    - the pins' commands of code-organization.md, section 3, after
      adding 30 crates of jig, smith and skein to a scratch crate: the
      check printed the three pinned revisions;
    - the throwaway Forgejo's smoke start (decision 12); `git --version`
      2.53.0, the version git.md's transcripts name;
    - every interface sketch, compiled under clippy with `-D warnings`
      in a scratch crate of a scratch worktree of temper at `cc2697cf`
      (`~/src/rust/tmp/plan/temper-sketches`, `heavy cargo clippy -p
      plan-sketches --all-targets -- -D warnings`); the sketches of
      ordinary-Rust code are marked "not compiled" in their files.

## Rulings during the run

The supervisor's rulings follow, numbered from 101 (development.md,
10.4).

101. **The store crate links the forge's three children** (temper 04.1,
     blocked 2026-10-14T03:26:27Z; plan defect). `Stored` and `Key` hold
     types of `temper-engine-domain-forge-client` (`api::Repository`,
     `api::Commit`, `Key`, ...), `-change` (`Status`, `GateReport`, ...) and
     `-issues` (`Projected`, `GoalView`), which the forge names through
     private `use ... as client` aliases in `boundary.rs:6-8` and does not
     re-export. `temper-protocol-store` lists all three in its
     `[dependencies]`, with the workspace entries that exist at
     `Cargo.toml:77-79`; it adds no `pub use` to the forge. They are
     domain crates of one connector, not a root, so "no codec crate links
     an engine or worker root" holds, and 04's check
     (`grep -cE '^temper-(engine|worker)-domain[ .=]'`) still prints 0.
     code-organization.md, section 3 and session 04 say so.

102. **05.3 lands as one increment, at most 800 lines** (temper 05.3,
     blocked 2026-10-14T03:57:07Z; plan defect). Its Size is 450, so 675
     with 1.5 times; the implementer counted about 740 production lines
     already written (`facts.rs` 490, `runner.rs` 222, about 30 more),
     most of it the 63 facts of forge.md, section 13 that the Shape lists
     one by one. A split into 05.3a and 05.3b, proposed as the
     alternative, would add a gate and a landing for code already
     written, and leave `facts.txt` with six Actions facts unchecked
     between them. The bound is code-organization.md, section 8's count
     on the increment's commit, at most 800; past 800 it splits as the
     implementer proposed. Its Files, Test shape, Proof (one process run,
     63 facts, 83 transcripts) and README's 10 lines are unchanged, and
     06.4's and 14.5's dependencies on 05.3 stay as they are.

103. **Forgejo v16.0.5's four `fails` lines** (temper 05.3, blocked
     2026-10-14T04:06:04Z; design gap). 05.3 lands `facts.txt` as
     observed, its Proof's "the run goes on". Two lines have the
     design's fallback, and need nothing:
     - `fails effective-protection-name-for-a-writer:
       effective_branch_protection_name=""`: forge.md, 3.2's
       `Read::Protection` row reads "`GET R/branch_protections` when it
       names none", answered "a `403` (a writer who is not an admin)
       `Forbidden`".
     - `fails delete-missing-branch-is-404: HTTP 500: ""`: 3.4's "`5xx`"
       is `Timeout`, and the find after it reads the branch
       (`outbox.rs:310-312`), whose `404` is `Missing`.

     Two are the designer's, in the run's `question.md` of this time:
     `fails pull-for-names-the-newest-of-several: 12` (the direct lookup
     names the oldest, closed pull) and `fails open-pull-no-changes-is-422:
     HTTP 201: null` (Forgejo creates a pull with no changes). The
     designer answered at 2026-10-14T04:27:56Z, landed in `ba448afd`
     ("Design: PullFor finds the open pull; OpenPull compares first"):
     `PullFor` answers the open pull request for the head and base, from
     the open listing when the lookup names a closed one, else the one
     the lookup named; `OpenPull` compares the branches' commits first
     and answers `NothingToMerge` with nothing written when
     `total_commits` is 0. forge.md, 3.2, 3.4, 9 and 13, and sessions
     06.3, 06.5, 13.3, 13.4, 14.2, 14.3 and 14.5 say so; 05's
     `facts.txt` keeps its four `fails` lines as the record.

105. **A fix a session's own "Done when" needs is no new step**
     (temper 05.4, blocked 2026-10-14T04:33:15Z; plan defect). Session
     05's check `git grep -nE 'Authorization: (token|Basic)
     [0-9A-Za-z+/=]{20,}' -- tools` matched 05.4's test literal
     `Basic d3JpdGVyOnNlY3JldA==` (`writer:secret`, no issued token).
     The implementer shortened it to `writer:t` and landed it as
     `2838a7a8`, test-only, under `Step: temper-05.4`. That stands. From
     now: a fix that the session's own "Done when" needs, at most 20
     lines, in that session's files, every test's name and expected
     behaviour kept, lands under its last step's `Step:` trailer, with
     the "Done when" item it serves named in the commit body; the
     coordinator records it in that step's ledger note, and no step is
     added. Anything larger, or outside the session's files, is a
     question.

106. **Sizes: twice, and 2.5 times for a codec or document increment**
     (temper 06.2, blocked 2026-10-14T04:35:36Z; plan defect; the second
     size block after ruling 102). 06.2 (Size 550, allowance 1100) is
     complete at 1287 production lines (`client/items.rs` 440,
     `server/items.rs` 275, `decode/walk.rs` 263, ...), both sides of
     six request families, and lands as it is. The plan's Sizes run low
     for these increments: 04.3's parts summed 2.35 times its Size, 04.4a
     1.99, 06.1 1.98, 06.2 2.34, 05.3 1.85 (a tool). As a standing rule,
     replacing decision 19's 1.5 and twice and the steer of
     2026-10-14T04:21:44Z's point 1: an increment lands within twice its
     Size without asking, a codec or document increment within 2.5 times
     it; a split increment's parts count together; each report names
     "Size: <increment> <count> of <allowance>"; past the allowance it
     asks. README, code-organization.md, section 8, and sessions 03, 04
     and 06 say so.

107. **The codecs world keeps its own store limits** (temper 04.5,
     blocked 2026-10-14T04:35:36Z; plan defect). The fuzzy store test
     needs a `temper_engine_domain_forge::Limits`; the store's
     `fixtures::LIMITS` (`src/tests/fixtures.rs:31`) is in a unit test
     module a world cannot import. code-organization.md, section 5's
     "never redefined" binds a crate's own fixtures; a world keeps its
     own, as `tests/codecs/src/forgejo.rs:50`'s `limits()` does beside
     `temper-forge-forgejo`'s unit `fixtures::limits()`. So
     `tests/codecs/tests/fuzzy_store.rs` defines its limits as one
     constant at its top, the unit fixture's values copied, and
     `tests/codecs/Cargo.toml` adds the forge crates it needs as
     dev-dependencies; both are in 04.5's Files. The unit fixture stays
     where it is; nothing is moved or included by path.

108. **A session's own earlier files are in its later increments' Files**
     (temper 06.3, blocked 2026-10-14T04:43:10Z; plan defect). 06.3's
     Files omit `decode/walk.rs`, `client/items.rs`, `server/items.rs`,
     `client/start.rs` and `server/start.rs`, which 06.1 and 06.2 created
     and which its Shape must extend (the walker's tables and dispatch;
     `PullBranch::repo_id` in 06.2's pull), and
     `tests/codecs/tests/forgejo_items.rs`, whose pull literals gain
     `repo_id`. As a standing rule, the README's "the session's own
     earlier code, replaced" becomes "replaced or extended": a file the
     session's own earlier increments created is in each later
     increment's Files, its lines counted in the Size, and an expected
     literal there gains a field the Shape adds, with the value its
     transcript or fixture gives, no other value changed. A file another
     session created stays outside.

109. **`owner-permission` is a `403`** (temper 06.3, blocked
     2026-10-14T04:43:37Z; plan defect). 05.2 recorded it as `HTTP 403`
     with `{message, url}`, no permission. 06.3's rows assert its status
     and its error body is decoded by 06.5's `error_decoder`, in 06.5's
     failure table; `Permission::Owner` is read from a literal unit
     document, and 06.3's Proof counts eleven documents. forge.md, 3.4
     reads a `403` as `Forbidden` ("| `401`, `403` | `Forbidden` |").

110. **The store's codec files are in the Files of an increment that
     changes a stored value** (temper 02.1, blocked 2026-10-14T04:48:10Z;
     cross-session scope). 02.1 adds `Answer::Located`, which
     `Stored::Call` holds, so the README's "A durable record's codec
     follows its record" already asks it to change "the schema, the
     generated code and the translation in the same increment". Its
     files were not named in any Files: they now are, in that README
     rule, for every increment that changes a stored value (02.1, 02.4,
     and any other). The hand-written lines count in its Size; the
     schema, generated code and goldens do not.

111. **A commit status's creator may be `null`** (temper 06.4, blocked
     2026-10-14T04:58:18Z; design gap, in part). 05.3's
     `actions-combined-status` transcript has two statuses with
     `"creator":null`, written by Actions. 06's documents "carry what
     Forgejo says, typed, not translated", so `CommitStatus.creator`
     becomes `Option<User>`: none for `null`, the key still required,
     and the server writes none as `null`. What the component makes of a
     status with no creator (the domain's `api::Status.author` is a
     `u64`), and the job link's shape (forge.md, 3.2 says
     `<html_url>/actions/runs/<n>/jobs/<i>`; v16.0.5 writes
     `/fixture/specimen/actions/runs/1/jobs/0`, relative to the origin),
     are the designer's, in the run's `question.md` of this time. The
     designer answered at 2026-10-14T05:04:58Z, landed in `b65b41ef`
     ("Design: Actions statuses carry no author, and their links are
     relative"): `api::Status` loses `author`, which nothing reads (13.3's
     Files, with the store's codec by ruling 110); `job_link` reads the
     prefix as `html_url` whole or its path, and refuses `//`, another
     scheme or host; the component makes an origin-relative link absolute
     with `html_url`'s origin (forge.md, 3.2, 3.5, 9, 13; sessions 06.4,
     13.3, 13.4, 14).

112. **Disk below 40 GB: the cache past its cap** (blocked
     2026-10-14T05:08:02Z; infrastructure). `df -h ~/src` showed 38 GB
     after the coordinator removed the finished sessions' `target/`.
     `kache stats` showed its store at 40.9 GiB of a 23.3 GiB cap (176%),
     its daemon reporting a version mismatch; its own eviction then
     brought it to 20.9 GiB, and `kache gc` evicted nothing more. The
     supervisor removed three idle `target/` directories outside the run
     (`~/src/rust/temper/target`, `~/src/rust/tmp/plan/temper-protocol/target`,
     `~/src/rust/tmp/plan/temper-sketches/target`). `df` then showed 53
     GB, and new starts resumed. If disk falls below 40 GB again, the
     coordinator runs `kache stats | head -1` once and names the store's
     size in its block.

113. **The small profile's numbers pass every worst case** (temper 08.1,
     blocked 2026-10-14T05:18:05Z; plan defect). 08.1 takes `small`'s
     constants from the engine worlds' fixtures, with `name_bytes` 256
     and the client's `answer_bytes` 65,536, beside jig's small children;
     the root's worst case refuses that mix
     (`temper-engine-domain/src/limits.rs:233-243`:
     `forge.named_calls < call_records`,
     `journal.result_bytes < people.words`,
     `journal.transcript_bytes < row_bound`, and the journal's room for
     deliveries). configuration.md, section 7 fixes no number ("Its
     numbers are constants in `temper-service`'s limits module, each
     documented with what it bounds"). So the implementer's profile
     stands: root journal `{commits: 3, held: 1536, writes: 6144,
     deliveries: 512, transcript_bytes: 65536, result_bytes: 2048,
     run_bytes: 16384}`, root loads `{loads: 2, rows: 1, bytes: 65536,
     reply_bytes: 65536, transcript_bytes: 65536}`, `forge.named_calls`
     32, the other fixture values kept; each raised constant's doc line
     names the check that needs it, and
     `the_small_profile_passes_each_domains_worst_case` is the check.
     Tighter numbers wait for startup's memory measure. `writes` was 4096
     in the first answer; 08.1's proof measured the root's worst case
     at 6048 (`route_bound` 6016 with jig small's `tasks::max_out` 724,
     and `call_records` 32), so it is 6144 (blocked
     2026-10-14T05:24:23Z).

114. **Make's rename needs no probe** (temper 07.3b, blocked
     2026-10-14T05:23:47Z; plan defect). Session 07's Choices promise
     "a Make whose rename meets a name anyway ends `Broken`", while
     skein's pinned `kernel::Op::Rename` reads "replacing what `to`
     named" (`kernel.rs:447-448`) and git.md, 4.1 names two operations.
     The pin submits a plain `renameat` (`skein-shell/src/ring.rs:827-831`,
     no flags), and a workspace is a directory: rename(2) refuses to
     replace a directory that holds anything (`ENOTEMPTY`) and replaces
     an empty one. Checked on x1 with `mv -T ws trash/7` onto a
     non-empty `trash/7`: "cannot overwrite 'trash/7': Directory not
     empty", exit 1; onto an empty one, exit 0. So Make keeps git.md's
     two operations, with no probe, and the promise holds where a
     collision would lose anything. Session 07 says so.

115. **After a reboot, `plan resume` must load the coordinator's
     thread** (2026-10-14; infrastructure). The machine rebooted for an
     apt upgrade at about 05:26Z and was back at 06:56Z. `plan resume`
     failed: `{"code":-32600,"message":"thread not found:
     01a1386e-..."}`, "the Codex daemon did not start the coordinator's
     turn". After a restart the daemon's `thread/read` answers the thread
     with `"status":{"type":"notLoaded"}`, while `plan`'s `resume()`
     (skein `scripts/dev/plan:812-818`) calls `thread/resume` only when
     `thread/read` fails. The supervisor called `thread/resume` with
     `plan`'s parameters (sandbox `danger-full-access`, approval
     `never`); the thread read `idle`, and `plan resume` then started the
     coordinator's turn (log.md, "Coordinator resumed,
     2026-10-14T07:03:22Z"). The fix, `resume()` also calling
     `thread/resume` when the status is `notLoaded`, is skein's kit, a
     proposal for the retrospective.

116. **`PartyInput::SignOut` lands in 11.3** (temper 11.3, blocked
     2026-10-14T07:17:06Z; plan defect, cross-session). 11.3's joint
     world copies ops's `Front`, whose `sign_out` sends
     `people::Event::SignOut { reply_to, sign_in }`; temper's root has no
     input for it. 08.5a's file already plans it ("add
     `PartyInput::SignOut`, field for field, routed to the people child
     as the core takes it"), but 08 is at 08.2a. So 11.3 adds the variant
     and its routing in `crates/temper-engine-domain/src/{boundary.rs,
     route.rs}`, with one test of the routing, and 08.5a translates
     jig's event to it. Sessions 11 and 08 say so. 11.3's Size, world
     code only, takes those production lines: at most 20 (8 at
     2026-10-14T07:25:24Z, `boundary.rs` 5 and `route.rs` 3).

117. **The core's restore ranges and the census are jig's** (temper
     08.2a, blocked 2026-10-14T07:21:33Z; plan defect crossing the root's
     load seam, put to the designer). The designer answered at
     2026-10-14T07:29:45Z, landed in `f5e4672d` ("Design: the core's
     restart ranges and the census are jig's"): `CoreRange::Tasks` is
     jig's `[Core, Tasks]`, filtered, a page that keeps nothing asked
     again; the census is jig's index walk, its rows in walk order,
     repeated owners kept, its cursor the last index row's key; the
     root's part (`store.rs`, `loads.rs` for `Range::Census` only) and
     the engine world's fake census join 08.2a, whose Size is at most
     570 (store/README.md, 5; session 08).

118. **Disk below 40 GB, the second time** (blocked 2026-10-14T07:50:47Z;
     infrastructure). `kache stats` showed its store at 34.7 GiB of its
     23.3 GiB cap (149%); the coordinator removed finished sessions'
     `target/` and disk came back to 50 GB; by 07:52Z the daemon had
     evicted to 20.9 GiB (90%), its version mismatch of ruling 112 gone.
     kache keeps its cap, late: between its passes it overshoots by 11 to
     17 GiB. As a standing rule (README, Rules), below 40 GB the
     coordinator removes the `target/` of sessions not running, logs
     `kache stats | head -1`, and blocks only if disk stays below 40 GB.
     kache's late eviction is a proposal for the retrospective.
     At 2026-10-14T08:28:33Z disk was 37.6 GB again, the store 37.6 GiB
     (161%), no idle `target/` left; the supervisor's `kache gc`
     printed "Evicted 2890 entries to stay under size limit" and disk
     came back to 51 GB, so the rule adds `kache gc` once when the
     store is past its cap.

120. **A world-only increment may land the README's 20-line fix**
     (temper 18.1, blocked 2026-10-14T08:35:59Z; plan defect; the second
     after ruling 116's 11.3). On jig's real store, a goal's transcript
     continuation fails: the root's `archive.rs:377-385` derives
     `TaskTranscript.from` from the preceding page's last turn, so the
     opening window (1,1) becomes (1,2), which jig's pinned
     `transcripts/read.rs:605-619` answers `Unread`. store/README.md, 5:
     "A root cursor is a key of the root's range, and names the jig
     interval that holds it". The fix, about 6 lines, keeps the opening
     window and carries `after` as the cursor. As a standing rule
     (README), the README's "a fix the change, its Proof or the gate
     needs, at most 20 lines" in an increment whose Size is world code
     only counts against 20 production lines, with no question.

121. **The forge connection's deadlines are its holder's** (temper
     13.2, blocked 2026-10-14T08:38:38Z; design gap, session 13's
     questions). The designer answered at 2026-10-14T08:41:34Z, landed
     in `0be9dd34`: a start read's deadline is the component's
     `deadline` from when it is asked; connect and handshake have no
     limit of their own; `idle` is 4 s in the profile `small`, below
     Apache's 5 s `KeepAliveTimeout` default (forge.md, 7 and 8; session
     13, its limits table, 13.2).

122. **The native client's limits rise only as far as the pin accepts**
     (temper 12.1a, blocked 2026-10-14T08:49:46Z; plan defect). Session
     12's `text` 4,096, `document_bytes` 1,048,576, `rows` 64, `watches`
     16 are refused at jig `d91a89c4`: `link/limits.rs` copies `rows`
     into wire fields whose ceiling is 32, and jig-view's
     `Limits::compose` returns none with `text` 4,096 at every `rows`
     from 64 to 2 (`logs/temper-12/12.1a-preset-diagnosis.log`).
     configuration.md and client.md fix no number. So each starts from
     jig's ops world's value, which jig's tests run with, and rises toward
     the plan's target only as far as `link::Limits::new` and
     `Limits::compose` accept, `rows` at most 32; no repin.

123. **A changed type's users are in the increment's Files** (temper
     18.2, blocked 2026-10-14T09:18:48Z; plan defect). 18.2's Shape
     removes `observations::Record`'s wrapper; 29 uses of
     `.map(|record| &record.0)` in `tests/conformance/src/stories.rs`
     (14) and `src/scenarios.rs` (15) follow it, past the README's
     20-line fix. code-organization.md, 6 already asks it: "it changes
     the type in place, and its users in the same increment". As a
     standing rule (README, edits outside Files), a type's users are
     edited to its new form with no other change, in any number of
     lines, every assertion, input, expected value and name kept.

119. **02.4 lands before 02.3** (temper 02.3, blocked
     2026-10-14T07:52:40Z; plan defect, order). 02.3 moves the
     conformance worlds' adoption to `PartyInput::Application`; after a
     crash cut with the people's token durable, the outcome is gone on
     restart until 02.4 keeps it durably, so eight conformance fuzzy
     stories fail (`tests/conformance/src/lib.rs:641`; the gate at
     `6693b866`, 8 of 35 fuzzy failed). 02.4 needs only 02.2's kept
     outcomes. So 02.4 runs next and lands first, and 02.3's commit is
     replayed on it, its assertions kept; 08.5b's and 17.4's needs are
     unchanged.

104. **A combined status with no status** (temper 06.2, blocked
     2026-10-14T04:16:35Z; plan defect). Forgejo v16.0.5 answers
     `{"state":"","sha":"","total_count":0,"statuses":null,...}`
     (05.2's transcript `combined-status-empty`). `CombinedStatus.state`
     becomes `Option<StatusState>`, none for `""`, as forge.md, 3.2 reads
     `base.sha` ("none when empty"); the server's
     `Answer::CombinedStatus` takes `sha: Option<ObjectId>` the same way,
     and writes no statuses as `null`. `StatusState` gains no variant: it
     is also a commit status's and `PostStatus`'s word, where `""` is
     none of Forgejo's. The decoder takes `statuses` `null` or `[]` as no
     rows, and checks nothing across fields: "a `total_count` of 0 is
     `None`, whatever the state says" (forge.md, 3.2) is the component's
     reading (13.3).

124. **An adoption's scope is the confirmed party** (temper 15.1b,
     blocked 2026-10-14T09:42:59Z; plan defect, not design). 15.1b
     said "one `Request::Send` through the table with the project as
     its scope"; no design text names the project. jig's
     `client/domain/root.md`, 4, at the pin: "a scope per request: an
     opaque number its asker gives, which `jig-domain-client` names: the
     party the request is for. Only the requests of the party confirmed
     are ever sent"; temper's client.md, 3.3 keeps it ("restored with
     `Kept { scope }`, parked until their party is confirmed"). So the
     forge side does as jig's ops sides do at the pin: the root passes
     `jig_domain_client::Request::Confirmed { party }` to the side as
     `forge::Event::Confirmed { party }`, beside `root.table.confirmed`
     (ops `route.rs:193-196`); the side keeps it (`infrastructure.rs:348-352`);
     `Adopt` sends `scope: party`, and nothing while no party is
     confirmed (`infrastructure.rs:657-659`, `693-695`). The project
     stays in the `Ask`'s `Adoption`. 15.1b's Files hold it
     (`forge.rs`, `forge/form.rs`, `route.rs`); its fixture
     `on_adoption_page` confirms a party, and
     `adopt_sends_one_keyed_request_and_a_second_press_none` asserts the
     sent scope is it. Its Size (550), its ten tests' names and its
     Proof are unchanged; no upstream edit or repin.

125. **An out-of-date family's requests stay in the table, unsent**
     (temper 15.1b, blocked 2026-10-14T09:46:45Z; plan defect, not
     design). The design says what happens: jig's `protocol/client.md`,
     9, at the pin, "A client outside a range sends nothing more of that
     owner, shows that it is out of date and how to update ..., and
     keeps what is pending in its small store; updated, it sends what is
     pending under its keys"; temper's client.md, 4.4, "the button is
     disabled, and a pending adoption stays in the small store". skein's
     pinned `RequestTable` (`skein-lib/src/requests.rs`) parks by link
     and scope only, not by owner, and 15.1b named no mechanism. So,
     with no change to the table and no repin: the root's `take`
     (`temper-client-domain/src/domain.rs`) pushes no `Request::Send`
     whose ask is `Ask::Forge(_)` while the forge side is out of date
     (the ask, not the owner, so a kept adoption is held too); the
     attempt stays unanswered until the table parks it at a link or
     party change and offers it again, and it is held again. The side
     shows its pending request's progress as `Parked` while out of date.
     Retention still retires it as unknown (jig's
     `client/domain/root.md`, 4: "A request first sent longer ago than
     that, less a margin, is never sent again"), the 4.7 row "unknown
     past the retention". `domain.rs` joins 15.1b's Files;
     `out_of_date_disables_adopt_and_keeps_the_pending_request` also
     asserts that after the out-of-range hello and a link down and up,
     the root's `take` pushes no forge `Send` and the table still holds
     the request. No other test; 15.1b's Size and allowance (550, 1100,
     its parts 15.1ba and 15.1bb summed) are unchanged.

126. **13.3 within 2100 lines; 13.4's Size 1000** (temper 13.3b,
     blocked 2026-10-14T10:06:56Z; plan defect, size). 13.3 (Size 600,
     allowance 1200) counts 1819: 13.3a landed 356 (`1f433ab1`), 13.3b
     holds 1463 (`reads.rs` 540, `component/calls.rs` 349,
     `reads/done.rs` 319, `reading.rs` 253, `reads/size.rs` 123,
     `component.rs` 106, `connection.rs` 80, the rest 31). Each part is
     in 13.3's Scope: fifteen reads' plans, the resolution by id, the
     answer's running size, a call run when a connection is free (an
     idle one, a new one, or another forge's closed), and the reads'
     failures. Its Size ran low, as 13.1's (998 of 1100) and 13.2's
     (962 of 1100) nearly did. So 13.3 lands within 2100, its parts
     summed, with no further split; past 2100 it asks. 13.4's Scope is
     as dense (five reads, twelve writes, 3.4's classification), so its
     Size becomes 1000 before it starts, its allowance 2000 by ruling
     106. Its Scope, Files, Test shape and Proof are unchanged. Session
     13 says so.

127. **The worker's secret comes from the `Files` `Worker::new` takes**
     (temper 09.1b, blocked 2026-10-14T10:16:19Z; plan defect). jig's
     pinned `config::Worker.secret` is a file's path; 09.1b's sketch gave
     `Worker::new(settings, agent, limits, runtime, seed)` no file's
     bytes. 09.1b copies jig's reference `worker.rs`, whose
     `Worker::new(config, agent, files: &Files, ...)` reads the secret
     with `files.contents(&config.secret)` and checks it against
     `limits.link.secret` (`testing/jig-test-service/src/worker.rs:60-68`
     at the pin), its startup passing the files it read
     (`startup.rs:600`); 08.1's `settings::Files` is that type, "the
     private files jig's parts name, by path, as jig's reference takes
     them", already in `settings::Engine`. So `Worker::new` takes
     `files: &settings::Files` before `runtime`, as the reference does,
     and 10.1b passes the files its `check_files` read; configuration.md,
     6: a worker's secret is "Named by jig's parts", "Read by jig's
     components". `settings::Worker` stays `{ jig, root, git }`
     (decision 9), and no `Secret` is added to it. Sessions 09 and 10
     say so.

128. **New starts hold below 30 GB of disk on x1** (blocked
     2026-10-14T10:39:14Z; infrastructure; the fifth disk block, after
     rulings 112 and 118 and the unblocks of 08:29Z and 10:20Z). At
     10:40Z `du -sh --total ~/.cache/kache wt/*/target` printed 47G:
     kache 21G (its store 27.9 GiB before the coordinator's `kache gc`,
     a cap of 23.3 GiB), session 08's `target/` 15G and 09's 12G,
     beyond development.md, 11.4's "1 to 8 GB"; the rest of the volume
     holds about 69 GB that is not the run's. With three sessions built
     out, `df -h ~/src` shows 38 to 44 GB, so the 40 GB hold blocked
     every 20 to 60 minutes with nothing idle left to remove. A session
     that ends frees its `target/` (12 to 15 GB) before the next starts;
     a start at 30 GB, its `target/` grown to 15 GB and kache past its
     cap by 5 GB, leaves about 10 GB above the stop at 15 GB. So, for
     this plan on x1, 30 GB replaces 40 GB in development.md, 11.4's
     hold and in ruling 118's rule (README); the stop at 15 GB, the
     memory bounds, three sessions at once and kache's cap are
     unchanged. That x1's run leaves too little disk for three sessions'
     `target/`, and that 11.4's 1 to 8 GB is now 12 to 15 GB in temper,
     is a proposal for the retrospective.

129. **A node's text is its text nodes' runs, printed once** (temper
     16.1, blocked 2026-10-14T11:45:30Z; plan defect, read from the
     design). client.md, 6.4 prints "every node in preorder, by its
     role", a heading as "its text", a button as `[n] NAME`; at jig's pin
     a node's text lives in text-node children (`jig-view` `Builder::run`
     admits only `Role::Text`, `builder.rs:134-135`), and `frame::control`
     gives a button both a name and a text child with the same label
     (`frame.rs:98-111`), so a literal walk prints every label twice.
     jig's `client/domain/view.md`, 3 decides it: a node carries "text,
     on text nodes only, as runs" and "a name, where its text does not
     give it". So a node's text is the runs of the text nodes under it
     that no nearer node of another role holds, NAME is its name or else
     its text, and the text nodes a line took print nothing more; a text
     node nobody took prints its own line, and a node of another role
     under a text-bearing one prints its own line. Session 16's 16.1
     Shape says so; its Size (900), Files (the golden `control_text`
     among them) and tests are unchanged.

130. **Startup checks the service's worst case through `system.rs`'s
     own** (temper 10.2, blocked 2026-10-14T12:07:40Z; cross-session
     scope). 10.2 checks "the two worst cases against `memory_bytes` with
     `jig_protocol_config::fits`" before anything is made; the service's
     envelope is 09.3's private `memory(settings, limits, store_limits,
     accounts, queue, operations)` (`crates/temper-service/src/system.rs:503`),
     which `Service::new` calls after the worker exists
     (`system.rs:55-71`, `worker.operations()`). So `system.rs` gains one
     `pub(crate) fn startup_worst_case`, taking the worker's operations
     as a number, that computes the queue, the operations and `memory`
     as `Service::new` does; `Service::new` calls it in place of its own
     lines, so the envelope is computed in one place (14.1 adds the
     forge's share there once). At most 30 lines change in `system.rs`,
     counted in 10.2's Size; every test of `system` keeps its name and
     expected values. The worker's operations come from its settings by
     the same function `Worker::operations` uses, made visible within
     the crate if needed, never copied. Session 10 says so.
     At 2026-10-14T12:10:13Z 10.2 found that formula across two crates
     (`Component::operations`, `temper-worker-protocol-git/src/component.rs:432`,
     `limits.operations + 1`; `Worker::operations` adds 1): so
     `temper-worker-protocol-git/src/limits.rs` gains `pub fn
     operations(limits: &Limits) -> Option<u32>`, which
     `Component::operations` calls, and `temper-service/src/worker_settings.rs`
     a settings-only `operations`, which `Worker::operations` and startup
     call; about 10 lines in `limits.rs`, `component.rs`, `worker.rs` and
     `worker_settings.rs`. A further shared helper ruling 130 needs, at
     most 20 lines, is the README's 20-line fix, with no question.

131. **The forge component's small values fit the client's answers**
     (temper 14.1, blocked 2026-10-14T12:13:36Z; plan defect). forge.md,
     7 checks at start "for each listing, at least one of its largest
     rows fits the client's `answer_bytes`, so no page size is 0", and
     fixes no number: `string_bytes` "at least the forge's
     `name_bytes`" (256), `body_bytes` "at least the client's `op_bytes`
     with a marker" (`2 × 512 + 21` = 1,045). Session 13's table chose
     `body_bytes` 65,536, `string_bytes` 1,024 and `inner` 64, which make
     `Items`' largest row (`body + body/2 + string + inner × (string +
     16) + Summary`, the implementer's reading of `limits::largest_row`)
     about 166 KB, past the small client's `answer_bytes` 65,536 (ruling
     113), so its page size is 0. Raising `answer_bytes` to 262,144
     breaks jig's small store preset and still gives a page of 1. So the
     component's small values fall, within forge.md's floors:
     `string_bytes` the larger of 512 and `name_bytes`, `body_bytes` the
     larger of 16,384 and `2 × op_bytes + 21`, `inner` 32; `Items`' row is
     then about 42 KB. A body past 16,384 bytes reads as `Long` (forge.md,
     3.3), as any body past `body_bytes` does. If a listing's page size is
     still 0, `inner` halves until it is not, at least 8, and the report
     names the values. The client's, the root's and the store's presets
     are unchanged; a larger profile, later, raises them together.

132. **One process's io is its parts' io, summed** (temper 10.2,
     blocked 2026-10-14T12:16:25Z; plan defect). 09's and 10's default,
     "one io from the engine's limits", refuses every small document:
     jig's small `io.sockets` is 16 (`jig-protocol-config/src/profile/small.rs`),
     the pinned parser has no `io.*` override (`overrides.rs:8-35`), and
     one process needs at least 24 (the implementer's count: the client
     listener and 4 client connections, the worker listener, the accepted
     worker and the worker's link, `slots × 4` agents, `git_component.operations
     × 4`). jig's reference startup composes one process's io from each
     part's: the engine's `limits.io`, then each worker's `sockets` and
     `names` added and `intake`, `receive`, `output` and `sends` the
     larger (`testing/jig-test-service/src/startup.rs:547-600`), and jig's
     worker part has `limits.io` of its own (`worker.rs:17`). So temper
     does the same, in one function `Service::new` and startup's
     envelope share (ruling 130's `startup_worst_case` takes it); 10.2's
     check counts the entities the configuration enables (the count
     above, the accounts' connections when accounts are configured)
     against the summed `sockets`, refusing at `engine.limits.io.sockets`
     "by how much". No profile constant and no parser change. Session
     10 says so (09 has ended).
     At 2026-10-14T12:19:43Z 10.2 asked for the means: the crate-internal
     `Service::new` takes the composed `io::Limits` as a parameter
     (`Service::new` holds a built `Worker`, which keeps no settings),
     startup computes it once with `checks::process_io` and passes it to
     `startup_worst_case` and `Service::new`, and the fixtures' callers
     pass it; no second constructor. From then, any change rulings 130
     and 132 need inside `temper-service` and the git component (a
     crate-internal function's signature, its callers, a fixture's call),
     every test's name and expected value kept and no public type's
     fields changed, is in 10.2's Files, named in its commit body, with
     no question.

133. **A TLS forge's trust roots are priced as the accounts' are**
     (temper 14.1, blocked 2026-10-14T12:22:01Z; plan defect, a boundary
     type). skein-tls's client worst case leaves out the shared trust
     configuration, "counted once by its service"
     (`skein_tls::roots_worst_case(count, each)`); 14's sketch gave
     `ForgeRuntime` its number, address and transport only, so a TLS
     forge's roots go unpriced, or are priced at the reader's cap (1,024
     roots of 65,536 bytes). `AccountsRuntime` already carries
     `trust_roots: u32` and `trust_root_bytes: u32`
     (`crates/temper-service/src/accounts.rs:26-27`), checked at
     `accounts.rs:82` and priced at `system.rs:556`. So `ForgeRuntime`
     gains the same two fields, filled by whoever reads the trust roots
     for it as for the accounts; `system`'s memory adds
     `roots_worst_case` for each TLS forge, as for the accounts; a TLS
     forge with no roots is refused as `accounts.rs:82` refuses; a
     plaintext forge carries 0 and 0. Session 14 says so.

134. **A native shell idle before it loaded is a refused directory**
     (temper 16.3b, blocked 2026-10-14T12:35:21Z; plan defect). Session
     16's default printed a line "on 12.1's `DirectoryRefused` notice";
     at jig `d91a89c4`, `jig-shell-native`'s `files.rs:230-256` meets a
     failed private open by marking the failure and closing its roots,
     and `Shell::is_empty` also needs `closing`, so no notice reaches the
     native client, which stalls (findings/jig-client.md, Discrepancy 4;
     later.md's row). A malformed token or store file is met the same
     way. So the native client reads a shell that has not loaded and
     holds no work, deadline, submission or completion as
     `DirectoryRefused`, closes it, and `temper open` prints `temper
     open: DIRECTORY: refused (not a private directory of this user's,
     or its records unreadable)` and exits 1: client.md, 6.1's exit 1,
     with no cause the pin cannot tell apart. The terminal does not
     check the directory itself (no second reading of what the shell
     reads). One test, `a_refused_directory_exits_one_with_its_line`
     (a directory with group bits set), joins 16.3's Test shape. Saying
     why stays jig's (later.md).

135. **A push that lost a ref-lock race is tried once more** (temper
     16.3a, blocked 2026-10-14T12:38:18Z; process). `land` exited 7
     ("fetching or pushing failed") when its push met `cannot lock ref`,
     another push to `main` at the same moment (here `2dec7fb5`), with
     the gate passed and nothing wrong in the commit. development.md,
     8.5 stops on 7, as it should for a failure that needs a person; this
     one is main moving, as 6 is. As a standing rule (README): on exit 7
     whose message names `cannot lock ref`, the implementer waits 60
     seconds and runs `land` once more; a second exit 7 is a block, kind
     infrastructure. Whether `land` should report this race as 6 is a
     proposal for the retrospective (skein's kit).

136. **The binary advertises raw message bytes, the domain keeps
     rendered ones** (temper 10.4, blocked 2026-10-14T13:10:30Z; plan
     defect). 10.4's first real-binary proof refused `workers[0].limits`:
     `temper-worker-domain/src/limits.rs` checks `host.event_bytes +
     SEPARATOR <= agent.message_bytes` (`:84`, `SEPARATOR` 2 at `:39`)
     and `bounds.message_bytes <= host.event_bytes` (`:89`), while 10.3's
     `agent()` (`crates/temper/src/lib.rs:134-175`), copied from jig's
     reference, advertises `bounds.message_bytes =
     limits.domain.run.message_bytes`, the rendered cap; smith renders a
     message as its label, two bytes and its text. So no derived profile
     could start; 09's and 10's fixtures set the three apart and hid it.
     The hello's bound and the host's event bound are raw bytes, the
     domain's `message_bytes` rendered ones, and both checks stay (a hello
     must not promise more than the domain keeps). So the binary's
     `agent()` advertises `run.message_bytes - SEPARATOR` (checked; a
     profile where it underflows is refused naming the agent's
     `message_bytes`), with smith's host limits as they are, and the
     fixtures use raw and rendered values two bytes apart (the worker's
     `hub.event_bytes` 598 beside `tool_payload` 600). Those lines in
     `crates/temper/src/lib.rs` are in 10.4's Files, counted in its Size;
     the check is 10.4's Proof, the real binary starting.

137. **A fake forge write that Forgejo answers with a document reads it
     back through a call** (temper 14.3b, blocked 2026-10-14T13:21:42Z;
     plan defect). 14.3 makes "each other route ... one domain `Call`",
     "its reply encoded by 06's `server::encode`", and its Don't forbids
     answering "from the domain's store but through a call". The fake
     forge domain answers writes with ids (`Created(number)`,
     `Commented(id)`, `Reviewed(id)`, `Done`;
     `testing/temper-fake-forge-domain/src/api.rs:428-442`), while
     Forgejo answers them with documents the component reads
     (`Review.commit_id`, an issue's user, times, title and body, a
     pull's `head.sha`, `base.sha` and `merge_base`;
     `temper-engine-protocol-forge/src/writes.rs:588`). So such a write
     makes its one write `Call`, then one read `Call` of the document
     Forgejo returns, and answers with what the read returns: nothing
     made up, the write made once, the HTTP request counted once in the
     tally, and a fault drawn after the write as before. The fake
     domain's boundary types are unchanged. Session 14 says so.

## For the user

Long-term choices the user may veto; each is the default the plan runs
with.

- **The designer's defaults, taken:** repositories keyed by Forgejo's
  repository id; the first client is a terminal (`temper open`), the
  browser waits for jig's browser shell, and temper's web crates are
  deleted; temper keeps its own charter and tools translation
  (`temper-engine-smith`) instead of jig's; git credentials ride in
  git's environment (readable by an agent of the same user in
  standalone); configuration is one strict JSON file with temper's
  members under `application`; the live tier uses a throwaway local
  Forgejo, never the user's forge.
- **A failed preparation's words are not on the task's page yet**
  (decision 3): jig's core keeps no attempt's failure detail at temper's
  pin, so the root reports them as a fact until jig's core does. The
  alternative is a jig change, which this plan does not make.
- **The plan has 81 steps** (decision 4), above the retrospective's 60.
  The alternative was two plans run in turn (the domain, codecs and
  service first; the forge's worlds, the client's tiers and the
  one-process world second).
- **`serde_json` in tools** (decision 10): the transcript tools mask JSON
  with it, a crates.io dependency outside skein, smith and jig, in
  ordinary-Rust tools only, as jig's process tests already use it.
- **forgejo-runner v12.13.2** for the Actions facts (decision 12), not
  the newer v13 line.
- **The live tier's budget for the plan:** at most 3 runs and 50
  completions of the user's Codex account (README, "Live services").
- **No repin of jig,** although jig's `main` is ahead of temper's pin;
  a later plan repins once jig's test-budgets plan ends.
- **An adopted repository is not listed on the project's resources**
  (the designer's answer to temper 15.2, 2026-10-14). jig's core, at
  temper's pin and at jig's `main`, lists on a project's page only the
  roles a live task's resources narrowed, and never an owned one
  (`jig-domain-core/src/resources.rs:170-182` keeps narrowings, not
  adoptions), though jig's `domain/core.md`, 6.1 and
  `client/domain/pages.md`, 6.5 ask for every adopted resource. temper's
  client.md, 4.3 and 10 and testing.md, 2.6 now say so, and its tests
  check the outcome's sentence and the engine's record of the adoption
  in place of the listing. The fix is jig's core's (keep each adoption's
  role, owned included, and list it), then a repin of jig in temper; the
  alternative, a temper-side list in jig's project document, puts
  temper's records in a document jig's core owns. jig's `STATUS.md`
  records the shortfall.
