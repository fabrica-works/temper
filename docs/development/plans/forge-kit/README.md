# forge-kit: the forge kit's extraction

2026-10-19. Start commits: skein `d120820` ("The forge kit: five designs
from draft/forge.md"), temper `27995fb75` ("Point temper's forge
documents to skein's forge kit"); jig `b2d920e43` and smith `a277e922`
are read only. Every command of this plan ran once at these commits
(`findings/base-runs.md`, and each session's `findings/base-<session>.md`);
each session starts from its repository's `origin/main`.

temper's forge crates move to skein, made neutral, as skein's forge kit:
the vocabulary, Forgejo's dialect and its call plans (`skein-forge`), the
client (`skein-forge-client`), the component (`skein-forge-connection`),
reconcile's machine (`skein-forge-setup`), the fake forge, its forge peer,
the throwaway Forgejo, the conformance and the kit's worlds. Upstream
first, one crate at a time, in temper's design's order (`draft/forge-kit.md`,
section 5): each crate is a skein landing, then a temper step that repins
temper onto it, moves temper's users onto it, hands it temper's values
and deletes temper's crate (development.md, 5.7). The move closes the two
gaps the designer named: the head check counts the user agent (skein
03.2), and the start's two reads are the dialect's plans (skein 01.4).

## Inputs

- **skein's design,** at `d9f3e39` (`d120820` with the designer's
  answers, decisions.md): `forge.md` (2 to 9, 11),
  `forge-client.md` (2 to 10), `forge-connection.md` (2 to 10),
  `forge-setup.md` (2 to 4, 6 to 8), `fake-forge.md` (2, 4 to 8);
  `testing-strategy.md`, 8.
- **temper's design,** at `b4cfb999`: `protocol/forge.md`,
  `protocol/setup.md`, `domain/forge.md`, `testing.md` (what stays
  temper's: the owner's values, the connector, the stories);
  `draft/forge-kit.md` (the connector's neutral top, later).
- **`decisions.md`:** the plan's decisions, what the user may veto, and
  the design's answers.
- **`findings/`:** the code audit at the base, one file per area
  (`survey-01-forge.md` to `survey-07-pins.md`), the design trace per
  document (`trace-*.md`), `base-runs.md` (each command of this plan, run
  once at the base), `drafters.md`.
- **`code-organization.md`:** what moves where, names, owners, fixtures,
  the "Don't" list, sizes and the count.
- **`later.md`:** what the design names that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"temper 01.2" means that increment has landed on temper's `origin/main`.

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-forge.md` | skein 01: `skein-forge`: the vocabulary, Forgejo's dialect, its call plans on plain bounds with keys' markers on the owner's word, the start's two reads, credentials' kinds and `host` | nothing | 01.3: temper 01.1, 01.2; 01.4: temper 01.3 |
| `skein/02-client.md` | skein 02: `skein-forge-client`: the client moved, effects' keys on the owner's namespace; the measure, the dialect's and the client's (decision 24); the design's differences; the tests section 10 asks for | temper 01.1 | 02.2: skein 01.3 |
| `skein/03-connection.md` | skein 03: `skein-forge-connection`: the component moved, on plain numbers, the owner's values and the transport's name; the head check counts the user agent; the checks at their bounds; page sizes on the measure | temper 01.4 | 03.3: skein 02.2 |
| `skein/04-setup.md` | skein 04: `skein-forge-setup`: reconcile's machine moved, on the owner's given values and the kit's item; the design's differences; section 7's step tests | temper 01.2 | 04.1: skein 01.3 |
| `skein/05-fakes.md` | skein 05: the fake forge's domain and protocol layer moved, with their tests; the forge peer in `skein-fake-peers` | temper 01.2 | 05.4: temper 05.1 |
| `skein/06-conformance.md` | skein 06: the throwaway Forgejo on skein's HTTP client; the conformance on the kit, with its transcripts, and its script; the dialect's document tests on the transcripts | temper 01.2 | 06.2: temper 03.1, temper 06.4 |
| `skein/07-worlds.md` | skein 07: the client's and setup's worlds, `tests/forge/client` and `tests/forge/setup`, moved, with the stories the design asks for | temper 02.1, temper 05.1 | 07.1: skein 02.2; 07.3: temper 04.1, skein 08.1; 07.4: skein 04.2 |
| `skein/08-machine-world.md` | skein 08: the component's machine and listener world, `tests/forge/machine`, moved, with plaintext and TLS faults, deadlines, closing and the counted worst case (forge-connection.md 10; decision 23) | temper 03.1, temper 05.1, skein 06.2 | |
| `skein/09-protocol-world.md` | skein 09: the protocol world's component and fake tests, `tests/forge/protocol`, moved, with its owner, stories and invariants on skein's client and component (forge-connection.md 10; decisions 22, 23) | skein 07.1, skein 08.1, temper 03.1, temper 05.1, skein 06.2 | |
| `skein/10-plans.md` | skein 10: `skein-forge`'s plans' pure tests on canned exchanges, and the plans' differences from the design (answers 7 to 12 and 35), each its own increment (decision 28) | skein 01.4 | 10.5: skein 08.1 |

### temper

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-forge.md` | temper 01: temper on `skein-forge`: the vocabulary's and the dialect's users moved, temper-forge-forgejo deleted, the component on skein's plans and start reads, temper's word handed | skein 01.1; the tools plan's first repin (Rules) | 01.2: skein 01.2; 01.3: skein 01.3; 01.4: skein 01.4 |
| `temper/02-client.md` | temper 02: temper on `skein-forge-client` and the measure; temper's client deleted; effects' purposes in temper's connector | skein 02.2, temper 01.3 | |
| `temper/03-connection.md` | temper 03: temper on `skein-forge-connection`; the component's crate deleted; temper-service hands the word, the user agent, the transport's name and the limits; the new refusals mapped | skein 03.1, temper 02.1 | 03.2: skein 03.3 |
| `temper/04-setup.md` | temper 04: temper on `skein-forge-setup`; reconcile's modules deleted; temper's setup hands its values | skein 04.1 | |
| `temper/05-fakes.md` | temper 05: temper on skein's fakes, temper's deleted; the one-process world on skein's forge peer | skein 05.2 | 05.2: skein 05.5 |
| `temper/06-throwaway.md` | temper 06: temper's process tier, tools and conformance on skein's throwaway; temper's deleted; the conformance on skein-json (decision 27) | skein 06.1 | |
| `temper/07-worlds.md` | temper 07: the moved worlds, the dialect's tests and the conformance with its transcripts deleted from temper; the protocol world keeps temper's own tests (decision 22) | skein 06.3; temper 06.1 | the client's world: skein 07.1; setup's: skein 07.3; the component's: skein 08.1 and 09.1; 07.1: temper 05.2 |

17 sessions: skein 10, temper 7. The brief's fifth step, the fakes and
the conformance, is skein 05 to 09 and temper 05 to 07, by size. The
increments per session are in each file.

**A copy follows temper's code onto skein's.** A skein step copies
temper's code only once that code names skein's crates for everything
the copy uses: skein 02 copies temper's client after temper 01.1 has put
it on skein's vocabulary, skein 03 the component after temper 01.4, its
worlds (skein 08 and 09) after temper 03.1, 05.1 and skein 06.2, skein 04 reconcile and skein 05 the fakes
after temper 01.2, the forge peer after temper 05.1, the conformance
after temper 06.4 has put it on skein-json, and each world after temper has put it on skein's
crates. The copy is then a move of files and a rename of paths
(code-organization.md, 6), and temper deletes its own once skein's has
landed.

**How many at once:** as many as the dependencies and the machine allow.
At the start one needs nothing: skein 01; temper 01 starts when skein
01.1 lands. Then, as they become ready, first what others wait on: skein
03 (temper 03, skein 06.2, skein 08), skein 02 (temper 02, skein 07, 09),
skein 05 (temper 05, skein 07 to 09), skein 04, skein 06, skein 10; the temper
sessions as their skein steps land; skein 07 to 09, then temper 07
last.

**The critical path:** skein 01.1 → 01.2 → temper 01.2 → skein 01.3 →
temper 01.3 → skein 01.4 → temper 01.4 → skein 03.1 → temper 03.1 →
skein 06.2 → skein 08.1 → skein 09.1 → temper 07.1 → 07.2: fourteen
steps to temper's last session. skein 08.2 → … → 08.5 and 09.2 → 09.3
→ 09.4 reach fifteen and may land after it; the closing `temper-repin`
follows every skein step: sixteen in all.
Beside it, skein 03.2 → 03.3 after skein 03.1, then temper 03.2;
temper 06.1 → … → 06.4 after skein 06.1, before skein 06.2 (decision
27); and skein 10.1 → … →
10.4 after skein 01.4, with 10.5 after 10.4 and skein 08.1. temper 07.1
also waits for temper 05.2, behind skein 05.5 (decision 26): a shorter
chain.

## Rules of this plan

- **Primary code graphs temporarily unavailable:** do not use the skein or
  temper graph until the supervisor confirms refreshed primary indexes in a
  landed ruling. Search source with `rg` meanwhile and verify names in the
  working checkout. This supersedes the graph permission of 2026-10-19T05:06:14Z.


- **The tools plan runs beside this one,** on another machine, on the
  same temper `main` (temper's tools, area 1, first part: temper
  `1093544c3`). Its jig 01 and temper 01.1 repin jig and temper onto
  skein's and smith's `main`; temper 01.1 here starts after its temper
  01.1 has landed: temper's `Cargo.lock` names smith at or after
  `b415f737` (`findings/survey-07-pins.md`). Its hotspots, the files this
  plan copies, deletes or edits that the tools plan also changes, each
  with the steps that hold it:
  - temper's `Cargo.lock` and root `Cargo.toml`: every repin of both
    plans; its temper 02.1 adds a member, and each temper session here
    deletes some.
  - `tests/client/Cargo.toml`: its temper 01.1, before temper 01.1 and
    02.1 here, so a rebase only.
  - The forge client's importers that its temper 03.1 changes:
    `crates/temper-service/src/tests/{fixtures,family}.rs`,
    `crates/temper-protocol-store/src/tests/records.rs`,
    `tests/conformance/src/scenarios.rs`; temper 01.1 and 02.1 here
    change their forge-client imports.
  - `testing/temper-fake-forge-domain`, `testing/temper-fake-forge-protocol`:
    no step of the tools plan changes them. They are copied by skein 05.1
    and 05.2 and deleted by temper 05.1; from skein 05.1's start to temper
    05.1's landing a change to them waits, and after it lands in skein's
    `testing/skein-fake-forge-*` (the designer's ordering rule: a change
    to a fake forge crate lands where the crate lives on `main`).
  - `tests/one-process/src/forge.rs` (`ForgeHost`) and
    `testing/temper-fake-git/src/transport.rs` (its shared forge handle):
    the tools plan leaves the forge host, its delivery and the real-loop
    world to its next part, unplanned (its `later.md`; drafts in its
    `findings/next/`, whose 07 moves the host to
    `testing/temper-fake-forge-host` with delivery). skein 05.4 copies the
    base host into skein's forge peer, skein 05.5 adds delivery there,
    and temper 05.2 deletes the host (decision 26), so that the next
    part's real loop hosts skein's peer (testing.md 4.2). From skein
    05.4's start to temper 05.2's landing a change to the host waits.
  - For the next part, when it is planned: its drafts also change
    `testing/temper-throwaway-forgejo` and
    `crates/temper/tests/support/{owned,forgejo}.rs` (its 10.1, 10.2;
    skein 06.1 copies them, temper 06.1 deletes them),
    `tests/client/src/{front,fixtures,translate}.rs` (its 06; temper 02.1
    here) and the webhooks' listener,
    `temper-engine-protocol-forge/src/{hooks,component}.rs` with
    `tests/engine/protocol-forge/{src,tests}/hooks.rs` (its 08.1; skein
    03.1 copies them, temper 03.1 deletes them).
  `temper ctl adopt`'s forge-facing flags (its temper 02.3, 11.1 and
  11.2: `temper-client-tools`, `temper-client-domain`'s `forge/`,
  `temper-client-view`) and its temper 03.1 and 03.2's
  `crates/temper-service/src/family.rs` and `src/family/refusals.rs`
  (its README's hotspots name them for the forge kit) touch none of this
  plan's files: temper 03.1 here changes temper-service's `forge.rs`
  and its neighbours, not the family. By the director's order for this
  run, it starts before the tools plan's temper 11 lands. The tools run
  is on `debian-16gb-hel1-1`, on the same temper `main`. The coordinator
  must register `plan await coordinator tools/temper-01.1` and must not
  start temper 01.1 here until that step is on `origin/main`. No other
  hotspot above adds a wait for tools temper 11; rebase conflicts are
  resolved as `land` and the implementer's card say. The fake and host
  copy windows above still hold until their named landings.
- **Each temper increment that needs a new skein step repins first,** in
  a commit of its own: `heavy cargo update -p skein-lib`, then `heavy
  cargo check --workspace --all-targets`. It moves every skein crate,
  jig's and smith's included (one revision per source); skein's new
  crates and modules break neither (Repin). **`Cargo.lock` is never
  merged by hand:** a conflict takes `origin/main`'s, and the update runs
  again. A repin takes all of skein's `main`, and repairs in its own
  commit what a skein step since temper's last repin broke, as
  development.md 5.7 allows: names and paths that moved, and the tests
  an upstream change of behaviour broke, adapted as the skein commit
  adapted its own. A skein increment that changes an interface temper
  uses after its move names temper's lines it breaks in its Shape
  ("temper's side"), so that whichever temper increment repins next
  knows the repair.
- **The first repin's jig repair** (temper 01.1): if its jig source is
  still `719bd70d4`, the first skein repin also takes landed tools
  `jig-02.1`, `b2eebe6ab91af9eca121588f2d44072fcbdf3b9c`, with
  `heavy cargo update -p jig-client-domain --precise b2eebe6ab91af9eca121588f2d44072fcbdf3b9c`.
  This repairs the RequestTable admission-number interface. Keep one jig
  revision per source, retain the skein update, and run the existing
  `heavy cargo check --workspace --all-targets` before the repin commit.
  If another landing has already moved jig forward compatibly, keep it;
  this rule authorizes no backward pin. The remaining tools steps add no
  wait to this repair. Refresh derived fixtures reached by the normal check
  and name further copies and repairs in the repin commit body.
- **Mechanical repairs during repins** belong to the existing repin commit.
  Follow landed upstream renames, paths, signatures and generated-version
  moves through their whole sweep, including source, tests, schema comments,
  copied values and derived fixtures, as implementer 8.2 allows. Keep fields,
  behaviour, test identities and assertions; list further users and repairs
  in the commit body. Use the increment's existing all-target check and gate.
  A straightforward successor-name repair needs no further scope block;
  report a structural or behavioural difference beyond the landed change.
  For temper 01.1's authorized jig pin, replace `people_v3` with `people_v4`
  in `crates/temper-protocol-store/src/records/application.rs`,
  `src/records/application/refusal.rs`, `src/tests/records.rs` and the comment
  in `schema/store-v2.schema` (all under that crate). This brings forward
  only the tools plan's temper 03.1 store-family name repair; its remaining
  work follows the existing hotspot/rebase rule.
- **A move copies files and renames paths** (code-organization.md, 6):
  besides that, it changes only what its new home forces, each named in
  the increment's Shape: temper's names out of skein's code (the owner
  hands them), and a type of a crate downstream of the new home made
  plain values. A behaviour change, and every difference the audit
  found, is a later increment's, with its own test.
- **A moved test keeps its name, its budget and its time.** A moved
  test over its ceiling (the protocol world's four `fuzzy_machine`
  tests, 7.5 to 7.7 s each at the base, `findings/base-runs.md`, moved
  by skein 08.1) moves as it is, its time and its cost's cause in the
  increment's report, and is named in decisions.md (11) for the user; none is split, trimmed or hidden by a filter.
- **Sizes.** A move is sized "about N lines moved, M new": the moved
  lines are those code-organization.md 6's command finds unchanged but
  for paths, and the increment's tolerance applies to the new lines
  alone.
- **A completed supervisor check releases its build outputs.** Once its worker
  has ended and no queued/running heavy task uses its paths, remove its scratch
  build targets, including a custom CARGO_TARGET_DIR, while retaining source,
  patches, reports and logs. Count available filesystem bytes after cleanup:
  Btrfs targets may share cache extents, so du totals do not measure reclaimed
  space. When those targets are insufficient, preview tracked targets unused
  for at least an hour and remove only inactive build outputs, after checking
  the queue. Keep development.md 11.4's floors, concurrency limit and GC interval.
- **The process tier runs where a Proof line or "Done when" says,**
  once, never in the gate, with the pinned binaries named:
  `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  heavy --gate scripts/proof-one-process.sh` in temper, with `WS` the
  workspace: `~/src/rust`, or `~/src` on a machine that keeps its
  checkouts there (AGENTS.md). skein's throwaway and conformance take
  the same paths as flags (skein 06). The tier is load-sensitive: two
  base runs failed beside other builds, three passed alone
  (`findings/base-runs.md`); a failure is rerun once alone before it is
  read as the step's.
- **The suites' aims for the whole run** (decisions.md): skein's default
  suite takes at most 1.5 s more than at the base, and its fuzzy suite
  at most 6 s more; temper's suites take no more than at the base, by
  package (the tools run lands beside it): the deleted crates' tests
  leave, the moved worlds' with them. No session has a share.

## Live services

None. The throwaway Forgejo and the conformance run the pinned local
binaries on loopback. At the base, temper's tests read `TEMPER_FORGEJO`
and `TEMPER_FORGEJO_RUNNER`, and they keep reading them. From skein
06.2 the conformance takes `--forgejo`, `--scratch`, `--out` and an
optional `--runner`, and reads no variable. The kit's ignored tests
read `SKEIN_FORGEJO` and `SKEIN_FORGEJO_RUNNER` in their test files,
and fail naming the one that is unset (decisions.md, Design questions
31). The conformance is a tool outside the suites, run by its script.

## Repin

temper's, in each temper increment that needs a new skein step (Rules),
and the closing `temper-repin` once every skein step has landed,
dropped or split (development.md, 5.7): temper 07.1's repin may land
before skein's last tests do. skein's changes are new crates, a module
of `skein-fake-peers` and new test crates, so nothing of smith, jig or
hearth breaks; they take skein at their next repins (decisions.md, 13):
`later.md`, and each repository's `STATUS.md`, under "Not planned", in
the designer's next landing.

## Pulse

```
words: actual|genuine|legacy|compat|workaround
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

Each command runs once, after `done.md`, in a worktree of temper (or
skein, for skein's) at `origin/main`, the plan's worktrees removed.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. The plan's commits: `git log origin/main --grep='^Plan: forge-kit$'
   --format='%(trailers:key=Step,valueonly,separator=)' | sort -u` in
   skein lists the skein steps the session files number, and the same in
   temper the temper steps; a supplementary step (`.s<k>`) adds a line.
3. Each session file's "Done when" holds, in the sessions table's order.
4. temper holds none of the moved crates: `ls crates testing tools`
   names none of `temper-forge-forgejo`,
   `temper-engine-domain-forge-client`, `temper-engine-protocol-forge`,
   `temper-fake-forge-domain`, `temper-fake-forge-protocol`,
   `temper-throwaway-forgejo`, `forgejo-conformance`; `ls tests` names
   none of the moved worlds (code-organization.md, 1); and `rg -l
   'skein_forge' Cargo.toml crates/*/Cargo.toml` lists temper's users.
5. skein's kit names no system: `rg -n -i 'temper' crates/skein-forge*
   testing/skein-fake-forge-* testing/skein-throwaway-forgejo
   tests/forge testing/skein-fake-peers/src/forge*` prints nothing.
6. The process tier: in a worktree of temper at `origin/main`, with the
   binaries named as Rules say,
   `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
   TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
   heavy --gate scripts/proof-one-process.sh` exits 0.
7. The conformance: skein 06.4's stable-fact check, its "Done when"'s
   `heavy bash` line, prints the five timing outcomes, then
   `facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0` and the
   base's digest, `d1f7ba2b…18347d` (`findings/base-skein-06.md`).
8. The test-time aims (Rules), measured once: `heavy --gate -- sh -c
   'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run
   --workspace 2>&1 | grep Summary; cargo nextest run --workspace
   --profile fuzzy 2>&1 | grep Summary; done'` in skein, against the
   same at skein's parent of `skein-01.1`; in temper, the per-package
   times of the moved crates' users against the base. An aim missed is
   recorded with the three slowest tests by name in decisions.md, for
   the user; the run is done either way.
9. No budget was raised: `git log origin/main --grep='^Plan: forge-kit$'
   -p -- .config/nextest.toml` prints nothing, in skein and in temper.
