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
