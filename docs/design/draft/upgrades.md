# Upgrades

Draft, 2026-10-23. How a deployment moves to a newer temper and keeps
its history: its people, projects and accounts, its tasks and goals, live
and ended, its landings and what agents wrote. It takes
`first-release.md`'s section 6 further, from the investigation of
2026-10-23 (Codex, read-only;
`~/src/rust/tmp/roadmap-3/store-upgrades.md`) and a session with the
user that day. It crosses skein, smith, jig and temper on purpose; an
owner splits it. Nothing here is adopted, and nothing changes today's
development: when the first cut comes is unknown, and until then the
rules of section 4.1 hold. Commands' spellings are placeholders.

## 1. In one page

- **Offline:** stop, translate, start (section 3). `temper upgrade`, run
  by the new binary on a stopped deployment, translates its store into a
  new generation and switches to it; the new binary then starts. A start
  on an older store is refused, naming the command. Rolling upgrades, an
  engine and workers of different wires, are out.
- **`temper box` upgrades itself** as it starts (3.2): a person who
  downloads a newer box expects it to open.
- **Versions freeze at cuts** (section 4). A cut is a release. Between
  cuts the next version of each encoding is open, and changes edit its
  schema and its one translator in place, as plans share a version
  today. Only frozen versions are promised; the first cut is the
  baseline, and stores from before it, or from between cuts, are made
  again. How often to cut is a knob.
- **A generation per upgrade** (section 5): the deployment's directory
  holds generations and a small file naming the active one. Translation
  works on a copy, never on the old generation, and switches by one
  atomic replacement once the new one is whole and checked.
- **Going back** is supported until the new generation's first start
  (3.3); after it, it is a backup's restore, with effects that may be
  made again.
- **Records first, the disk format later** (6.3). As PostgreSQL tells
  its own upgrade between versions of its files on disk (`pg_upgrade`)
  from an application's migrations of its schema and data, an upgrade
  here has two layers: skein-kv's formats, and jig's and temper's records
  over them. This draft is the second. skein-kv's formats have not moved
  since it was written, and translation uses skein-kv as it is.
- **Translation is a chain of frozen steps** (section 6), one per cut,
  each owner stepping its own families and the values of its own nested
  in another's. It works on a copy of the old generation by ordinary
  commits, so identities, counters and commit positions carry on;
  indexes and transcripts' offsets are rebuilt; every join is checked
  before the switch.
- **Everything recorded survives** (section 7). An agent resumes its
  conversation when smith's turns kept their version; when they moved,
  its next attempt starts from a fresh brief, its old turns readable.
- **An upgrade starts settled** (section 8): no external effect in
  flight. The engine's stop settles first; the tool checks it from the
  store and refuses an unsettled one.
- **The engine alone may be upgraded while workers keep their runs**
  (section 9), when the workers' wire is unchanged and the window fits
  their grace. Built second.
- **The configuration is never rewritten** (section 10): each release
  reads its own members and the previous release's.
- **The gate holds the policy** from the baseline (section 12): frozen
  fingerprints, a step for every stored change, golden stores of every
  cut upgraded into the candidate.

## 2. What an upgrade carries

| Part | Where | Its formats' owners | Across an upgrade |
|---|---|---|---|
| the records | the deployment's `store/` | jig's component and families; temper's families; over skein-kv's frames and snapshot | translated in the new generation, by ordinary commits (section 6); skein-kv's formats unchanged (6.3) |
| the transcripts | the deployment's `transcripts/` | jig's frames, smith's turns and results inside | rewritten, their offsets rebuilt; smith's bytes kept as written (6.4) |
| the secrets | the deployment's `secrets/` | jig's records; temper's forge records; skein's OAuth records | translated, every number kept (6.5) |
| the configuration | the configuration directory: `temper.json`, `agent.json`, charters' instructions | the operator's | read as written (section 10) |
| small files | the data directory: each forge's setup `state`, the box's records, the demo's, a worker's secret and digest | temper | read in any frozen version (section 10) |
| an owned Forgejo | its forge's directory | Forgejo, and temper's preparation | its own upgrade, at its preparation (section 11) |
| program logins | each worker's programs directory | each program's | kept as they are; a login a program no longer reads is signed in again |
| a worker's root | each worker's data directory | temper | rebuilt at will: checkouts and workspaces |
| what temper wrote on a forge | the forge | temper's connector | read in every frozen version (section 11) |
| a client's kept requests | the person's browser or shell | jig's client | as jig's client says (7.4) |
| the cache | the cache directory | — | never carried |

The directories are `../protocol/configuration.md`'s (section 12); what
the store holds is jig's `protocol/store.md` (4.4) and
`../protocol/store.md`'s (section 2).

**What must keep its identity:** the deployment's id, which names its
branches and keys' markers on the forge (`../protocol/forge.md`, section
4); the installation's id; every number the domain gives (tasks,
attempts, messages, sources, accounts, issued program names), so that
none is given twice; and the commit position, which the deployment's
header and projections name (6.3). A worker holds no authoritative state
of its own: what it acknowledged is in the engine or on the forge.

## 3. The operator's view

### 3.1 Under a service manager

A deployment of `temper serve`, or of an engine alone and workers alone
(`../protocol/README.md`, section 15), each a service:

1. **Rehearse,** optionally: `temper upgrade --check`, by the new binary,
   on a copy, such as last night's backup. It translates into a scratch
   directory and switches nothing, and reports how long it took, what it
   refused and whether the workers may stay (section 9).
2. **Stop** the engine and its workers, or the engine alone (section 9).
   The stop settles first (section 8).
3. **`temper upgrade`,** by the new binary, with the deployment's three
   directories. It plans the chain (6.1), refusing before it writes
   anything when it cannot finish; translates; checks (6.6); and
   switches. It prints each step and what it counted, and exits 0. Any
   failure leaves the old generation active, says why, and exits
   non-zero; running it again starts the translation afresh.
4. **Start** the new binary.
5. **Drop** the old generation once satisfied: `temper upgrade --drop`.

- **A start on an older store** is refused, naming its version, the
  binary's, and `temper upgrade`. A start on a newer one is refused too:
  there are no translators backwards.
- **A development store** (4.2), written before the baseline or between
  cuts, is refused by both, saying it is to be made again.

### 3.2 The box

`temper box` finds an older store as it starts and runs the upgrade's
steps before its own, printing one line, then a line per step as section
3.1's command does. A refused translation stops the box with the old
generation untouched, saying why. The box keeps the previous generation
and drops older ones at its next upgrade, and its stop settles, as every
stop does (section 8). Its owned Forgejo upgrades as today, at its
preparation (section 11).

### 3.3 Going back

- **Before the new generation's first start:** `temper upgrade --back`
  makes the previous generation active again, and the old binary starts
  on it. Nothing has happened since the stop, so nothing is lost or made
  twice. The tool knows the new generation has started when its commit
  position has moved past the one its manifest records: every start
  commits, the wire versions it speaks first (section 9), and every
  effect is committed before it is made. No other mark is kept.
- **After it:** going back is restoring a backup. The new version may
  have acted on the forge since, and the old one, restored, may act
  again; an owned Forgejo migrated since needs its dump restored with it
  (`../protocol/setup.md`, 6.3). The tool refuses `--back` and says so.

### 3.4 Backups

A backup is a copy of the data directory with the deployment stopped
(jig's `protocol/store.md`, section 16; `../protocol/setup.md`, section
16). Generations make an upgrade its own backup of the store, not of the
forge or the programs' logins. Online backups stay open (section 15).

## 4. Versions and cuts

### 4.1 Before the baseline

Today's rules hold, unchanged: one version of each encoding at a time, a
reader that refuses another and says so, and nothing translated (jig's
`protocol/README.md`, section 7; skein's `codec.md`, section 5); each of
temper's versions moves at most once per plan, plans running at once
sharing the move (`../protocol/README.md`, section 9); and a deployment
whose store breaks is made again (`../protocol/store.md`, section 4).
So a version names no one layout yet: plans may change a schema under a
shared version, and no translator could tell their stores apart. No
promise reaches back before the baseline, and nothing in this draft asks
anything of a change landed before it.

### 4.2 A cut

- **A cut is a release.** A cut of temper pins cuts of skein, smith and
  jig, made at the commits it pins: each repository freezes its own
  formats.
- **What freezes:** every encoding whose schema changed since the last
  cut. Its schema, its generated code and its translator from the
  previous frozen version are kept from then on, never edited, beside
  the current; and its fingerprint, a digest of its schema, keys and
  codec, enters its owner's registry of frozen formats.
- **Between cuts the next version is open.** The first change after a
  cut opens it, and every later change edits its schema and extends its
  one translator in place: a version moves at most once per cut, as it
  now moves at most once per plan.
- **Only frozen versions are promised.** A store names, beside its format
  row (jig's `protocol/store.md`, 5.3), the fingerprint of every format
  it was written in. The tool finds it among the frozen cuts'; one that
  is none of them is a development store, which upgrades to nothing and
  is made again, as today. Translating a frozen store into an open
  version is refused unless asked for, since its result is a development
  store.
- **The baseline is the first cut,** expected at v0.1.0
  (`first-release.md`). Every store of a frozen version from the
  baseline on upgrades to every later cut, skipping cuts between by
  chaining through them.
- **The cadence is a knob.** Cutting costs a freeze, a fingerprint and a
  golden store (section 13), since the translator was written as the
  changes landed. A deployment that wants `main`'s changes sooner asks
  for a cut.

### 4.3 What changes a version

From the baseline, any change to what is stored: its bytes, its keys, a
nested payload's decoding, or what an existing value means. An added
field is a change. A family no change touched takes the identity step,
which no one writes. A change to a wire alone (the clients',
the workers') moves its version and needs no translator, unless what it
carries is also stored.

## 5. Generations

```
<deployment>/
  active                   the active generation's number
  generations/
    1/  store/  transcripts/  secrets/  manifest
    2/  store/  transcripts/  secrets/  manifest
```

- **jig's deployment directory holds them** (jig's `protocol/store.md`,
  section 2): its open reads `active` and opens that generation, and a
  first start makes generation 1. The layout is the baseline's from the
  start, so no upgrade ever moves today's.
- **The manifest** names the generation it was translated from, the
  fingerprints it was written in, each family's count before and after
  the translation, and the commit position translation ended at.
- **The new generation starts as a copy.** The old generation's files
  are copied whole into the new one's directory and synced; translation
  opens the copy, never the original, so whatever recovery or
  translation writes, the old generation stays as it was.
- **One atomic switch.** Once the new generation is whole, synced and
  checked (6.6), `active` is replaced by io's replacement sequence:
  written beside, synced, renamed over, the directory synced (skein's
  `io.md`). Never two renames standing for one.
- **A crash before the switch** leaves the old generation active and the
  new one partial; the next `temper upgrade` removes the partial one and
  starts again. After the switch, the new one is whole.
- **Room for two:** the disk holds two generations until the old is
  dropped. In memory, translation works in one map, where a record whose
  key changes is held twice until its old key is erased: at most about
  twice the engine's map budget, while the engine is stopped.

## 6. Translation

### 6.1 The chain

- **A step per cut and family,** written by the family's owner: a
  function from the frozen types of one cut to the next's, over the
  code generated from their schemas, decoding with the older's codec
  and encoding with the newer's.
- **Planned before anything is written.** The tool reads the source's
  fingerprints and format row, its secrets' payloads' versions and its
  transcripts' frames' versions, and plans the steps from each to the
  current. A development store, a newer version, or a step missing
  refuses, naming each.
- **All owners step together, cut by cut,** because families nest: the
  core's records hold the tasks' records, and temper's hold jig's
  refusals. A value nested in another's family steps by its own owner's
  step, which that owner exposes: temper's step for a family that holds
  jig's refusals calls jig's.
- **Keys move freely.** A step may change keys and ranges: a record
  whose key changed is put at its new key and erased at its old in the
  same commit. Each versioned record names its family's version, so a
  record already stepped is passed over, and no reader but the tool
  ever sees the two side by side.
- **Indexes are rebuilt** from the records, never translated (jig's
  `protocol/store.md`, 7.4).

### 6.2 What a step may not do

- **Invent.** A new field the old record holds no evidence for takes a
  default its step documents, or the step refuses the record, naming
  it. Unknown is not zero: smith's usage became optional so that a turn
  with none is not one that cost nothing.
- **Give a number twice.** Every number and counter is kept; the next
  number given after the upgrade is one no record holds.
- **Repeat or forget an outcome.** Keyed requests' answers, connectors'
  keyed outcomes and decisions are kept, so a request sent again after
  the upgrade is answered as before.

### 6.3 skein-kv as it is

- **The copy opens as any start opens a store:** skein-kv recovers its
  snapshot and the log's longest valid prefix (skein's `kv.md`, section
  3), and jig's checks its format row and the deployment's header (jig's
  `protocol/store.md`, section 10).
- **Translation is ordinary commits** through jig's store, each within
  skein-kv's bounds and each writing the deployment's header, as every
  commit does. So the commit numbering carries on from the old
  generation's, the header's check holds at the next open, and every
  reference to an earlier commit still names it. The format row and the
  fingerprints are the last commit's.
- **Nothing new of skein-kv.** Its frames and snapshot have been at
  version 1 since it was written (skein 288c5423), and this pass leaves
  them so. The gate fingerprints them as it does every frozen format
  (section 12), so a change is seen; changing them, after the baseline,
  waits for an upgrade of their own: a tool of skein's, as `pg_upgrade`
  is PostgreSQL's, later (section 15).

### 6.4 Transcripts

- **Rewritten into the new generation,** a file per task (jig's
  `protocol/store.md`, section 8), when jig's frames moved, and copied
  otherwise; the committed lengths and each window's and attempt's
  offsets are rebuilt in the map either way.
- **smith's bytes stay as written,** in their smith version, never
  translated. smith keeps frozen readers of its turns and results by
  version, and clients render old turns through them.

### 6.5 Secrets

- **Translated as the records are,** in their own map with its own
  commits (jig's `protocol/store.md`, section 9): jig's credentials'
  digests and LLM accounts' records, temper's forge records, skein's
  OAuth records by skein's frozen readers.
- **Their numbers kept:** sources', accounts' and the carried values'
  counter, so that the start's sweep, which erases the secrets no record
  names, erases nothing it should keep.

### 6.6 Checked before the switch

- **Every join:** each record's references resolve: tasks to projects
  and parties, accounts to sources, transcripts to tasks, temper's
  repositories and landings to their forges.
- **Each family's rules,** as a restart checks them, but strictly: a
  record breaking its child's rules, which a restart counts and leaves
  (jig's `domain/engine.md`, section 6), refuses the translation.
- **An ordinary open:** the current binary's open, read-only, over the
  new generation, with the deployment's limits: a live set past them
  refuses, as a start would.
- **The counts** per family before and after, in the manifest.

## 7. What survives

### 7.1 Records

Everything recorded: people, roles, projects and their policies,
sign-ins, LLM accounts and their sources, issued program names; tasks,
live and ended, with their activity, goals, proposals, decisions and
notes; requests' keys and answers within their retention; temper's
repositories adopted, its landings, its forges' records and keyed
outcomes; the deployment's and installation's ids.

### 7.2 Agents and runs

- **Every live task keeps its definition,** its inbox, its holds and its
  place in its queue.
- **Its next attempt resumes its window** when smith's turns kept their
  version, the usual case once smith's formats settle.
- **When they moved,** translation closes each live window, and the next
  attempt starts fresh from a brief carrying the transcript's tail,
  rendered by smith's frozen readers, as jig starts one whose window is
  past the resume limit (jig's `domain/engine.md`, 7.2): a transition,
  not a failed attempt.
- **Old turns stay readable,** whole, through smith's frozen readers.
- **Runs end with a full upgrade,** as at any stop: the workers stop
  first, their runs stopped once the engine has every turn and answer
  (`../protocol/README.md`, 15.4), and the tasks retry after the start.
  An upgrade of the engine alone keeps them (section 9).

### 7.3 Sign-ins

- **People stay signed in:** a credential's digest is a record, kept.
- **LLM accounts stay signed in;** one expired or revoked since is
  signed in again, as at any start.
- **Program logins** are the programs' own, kept on their hosts.

### 7.4 Clients

- **Pages reload.** The bundle is the new binary's; a page of the old
  version learns from the stream's hello that its versions are not
  served, and holds its requests unsent until it reloads (jig's
  `client/domain/client.md`, 8.2).
- **A kept request of another version is dropped,** and the person told
  (jig's `protocol/client.md`, section 10). Its outcome may have been
  committed before the stop, so the notice says it may have been made.
  Translating them is open (section 15).

### 7.5 What is lost

- What retention erased before the upgrade (jig's `protocol/store.md`,
  11.1; temper's bounded `Landed`, `../protocol/store.md`, section 7). A
  deployment that keeps history long sets its retention first.
- A run's unsaved state on its worker, in a full upgrade, as at any
  stop.
- A development store, whole: it is made again (4.2).

## 8. A settled stop

- **An upgrade starts settled:** no external effect in flight. Every
  outbox entry made or given up, no delivery half-way, no effect
  proposed and half-made. The version that started an effect finishes
  it, so no recovery class (jig's `domain/connectors.md`, 4.3) and no
  key's derivation has to span two versions.
- **The tool checks it from the store,** which is the record: it reads
  the outboxes and refuses an unsettled store, naming each effect and its
  task, and saying to start the previous version and stop it again. No
  mark is kept to go stale.
- **Every stop settles first.** Told to stop, before it stops taking
  commits (jig's `domain/engine.md`, 6.1), the engine starts no effect,
  places no run and steps no procedure, answers busy a run's call that
  would start an effect, and commits the outcomes of the effects in
  flight within their deadlines. Then it stops as today. Past a bound, it
  stops unsettled, naming what remains, and a later start of the same
  version recovers it as today.
- **Every stop, not only an upgrade's,** since an operator does not
  always know beforehand, and the box's person never does. Its cost is a
  stop that waits up to an effect's deadline; a second signal aborts it,
  as jig's service does (jig's `protocol/service.md`, section 5).

## 9. The engine alone

- **The workers may stay** when the new engine speaks their channel's
  version (jig's `protocol/channel.md`, 4.1), temper's payload version
  (`../protocol/README.md`, section 9) and writes smith's charter in the
  version their agents read. Translation already keeps live claims and
  attempts, since it keeps every record.
- **The tool says so.** The store keeps the wire versions of the engine
  that last opened it, beside its format row; `temper upgrade --check`
  compares them with the new binary's and says whether the workers may
  stay. `temper versions` prints a binary's formats and wires.
- **The window must fit the workers' grace:** the settled stop, the
  translation and the start, within the grace the workers' configuration
  sets (jig's `protocol/channel.md`, 10.1). The rehearsal (3.1) measures
  the translation.
- **Meanwhile, the workers carry on:** they keep the turns and answers
  not acknowledged and redial, and the new engine's restart adopts each
  run as its worker says hello (jig's `domain/engine.md`, section 6,
  step 3). A run's call that would start an effect waits, answered busy.
- **The workers follow later,** one at a time if the wire allows, each
  stop ending its runs, which retry elsewhere.
- **Built second,** after the full upgrade. At today's churn most
  upgrades would change the workers' wire; the share falls as it
  settles.

## 10. Configuration and small files

- **The configuration is never rewritten.** It is the operator's, which
  they may keep in their own tools. Each release reads its own members
  and the previous release's: a member added is optional, with a
  default; a member removed or renamed is accepted for one release with a
  warning naming its replacement, then refused as unknown, as today
  (`../protocol/configuration.md`, section 2). A deployment that skips a
  release may find a removed member refused, by name.
- **Runtime records shrink it** (`first-release.md`, 7.1): templates,
  charters and policies in the store, where translation carries them;
  the configuration keeps what a start fixes.
- **Small files outside the generation,** a forge's setup `state`, the
  box's records, the demo's, a worker's secret and digest: each carries
  its version, is read in any frozen version from the baseline on, and is
  written in the current at its next write. None is written before the
  new generation's first start, so going back (3.3) finds them as they
  were.

## 11. The forge

- **An owned Forgejo upgrades at its preparation,** as today
  (`../protocol/setup.md`, 6.3): a dump before the migration, pins that
  refuse a downgrade. A temper release may move its pins; the box
  prepares at its start, a service deployment by `temper forge prepare`
  before the engine's start. Going back past a migration needs its dump.
- **What temper wrote on the forge outlives every version:** its keys'
  markers in what it creates, the key's first byte its version
  (`../protocol/forge.md`, section 4), and the branches in the
  deployment's namespace. From the baseline the connector reads every
  frozen version of a marker, and the deployment's id is kept, so the
  namespace and the markers stay its own.
- **The forge is no backup.** Its issues and pull requests cannot rebuild
  the engine's records; the generations are the record.

## 12. The policy and the gate

From the baseline:

- **A stored change brings its step,** in the landing that makes it: the
  open version's translator extended, with a step test of an old record
  and its expected new form. Additive changes too (4.3).
- **Frozen is frozen.** A frozen schema, codec or step is never edited;
  section 15 holds the exception for a bug.
- **The gate checks:**
  - each owner's registry: a changed fingerprint under a frozen version
    fails, and so does an open version whose fingerprint differs from
    the last frozen one's with no step written;
  - the golden stores of the baseline and of the previous cut, upgraded
    into the candidate and asserted (section 13), their expectations
    never regenerated from today's encoders;
  - mutations that must fail it: a step deleted, a frozen schema edited
    without a new version.
- **Meaning is reviewed.** A change of what a value means moves no
  fingerprint: its review and a scenario hold it.
- **Its cost:** the investigation guessed 1 to 2 milestones for an
  additive change and 2 to 4 for one of meaning, against a churn of 26
  moves of the engine's stored families in the four weeks to 2026-10-23
  (20 jig's, 6 temper's; 21 mostly additive, 2 repackings, 3 that
  reshape concepts). Cuts shrink it: every change in a cut shares one
  step per family and one golden store.
- **`../protocol/README.md`, section 9, changes** at the baseline: a
  version moves at most once per cut, not per plan.

## 13. Proving it

On the existing tiers (skein's `testing-strategy.md`, section 2;
`../domain/testing.md`, section 2), storage mechanics tested once, below
the domains:

- **Step tests:** each step over golden records, old in, expected new
  out; the registries' fingerprints.
- **A golden store per cut,** whole: records, secrets, transcripts, the
  configuration and the small files, made by that cut's binary from a
  story and kept for good. Upgraded into the candidate, directly and
  through skipped cuts, then asserted: identities and counters, policies,
  outcomes, rendered turns.
- **A story through an upgrade,** in the one-process world
  (`../domain/testing.md`, 2.5): a project, a person and an account; a
  goal and a change through a landing; a chat paused and a task
  mid-run; settle, stop, upgrade, start, and go on. The referee: the
  promised history equal before and after, no effect made twice, kept
  requests answered as before, the next numbers colliding with none, and
  the task going on, resumed or fresh by smith's version.
- **Power loss** on skein's crashing disk at every step of the copy,
  every commit of the translation, the manifest and the switch: the
  active generation is wholly the old or wholly the checked new, and the
  translation starts again from a fresh copy.
- **Faults:** a full disk, a short write, a failed sync, a malformed
  record, a limit too small, a step missing, an unsettled store.
- **The engine alone:** the engine upgraded while a worker's run goes on,
  adopted at its hello; a window past the grace ending the run.
- **The process tier** (`../domain/testing.md`, 2.6): stop, upgrade,
  start and going back, on fakes, with an owned Forgejo's migration as
  its tests have it.

## 14. Who builds what

The investigation's 16 milestones of 2 to 3 hours, regrouped by owner
and adjusted for sections 3 to 10, less its two of skein-kv (an import
at a kept commit position, and its faults):

| Owner | What |
|---|---|
| skein | skein-codec alone: frozen versions' generated code beside the current, fingerprints, a registry's gate; skein-oauth's frozen readers. Nothing of skein-kv |
| smith | frozen readers of turns and results by version, rendering old turns; its registry |
| jig | generations and `active` at its open; its families' steps, chain planner and nested steps for applications; transcripts rewritten and offsets rebuilt; secrets' numbers; the joins' check; closing live windows when smith's turns move; the settled stop's core; the engine's wire versions kept |
| temper | its families' steps; `temper upgrade` (check, switch, back, drop) and `temper versions`; the box's upgrade at start; the configuration's previous release; small files; every marker's version read; the settled stop's connector |
| all | golden stores per cut; the story through an upgrade; power loss; the process drill; the gate's mutation checks |

About 28 to 42 hours for the full upgrade, then about two milestones for
the engine alone. Built as the baseline approaches: the full upgrade,
then the box's, then the engine alone.

## 15. Open questions

- **When the baseline is cut.** Unknown; until then, section 4.1.
- **A bug in a frozen step,** found after deployments ran it. Default:
  fixed in place for those yet to cross it, the fix named in the
  registry; a repair step at the next cut for those that crossed it,
  where the old evidence allows.
- **The client's kept requests,** translated so that one survives an
  update. Default: dropped with a notice (7.4).
- **Effects in flight,** translated, lifting the settled stop. Default:
  settled.
- **Online backups.** Default: a copy with the deployment stopped (3.4).
- **Each repository's cadence:** whether skein, smith and jig cut on
  their own, or only with temper's. Default: with temper's.
- **Generations kept.** Default: until the operator drops them; the box
  keeps the previous one alone.
- **Stores larger than twice the map's budget.** Default: one map,
  translated in place in the copy; a streaming translation if a
  deployment outgrows it.
- **skein-kv's own formats,** an upgrade of the disk format below the
  records. Default: none until they first change; then skein's tool,
  run before the records' translation, as `pg_upgrade` runs before an
  application migrates.
- **Rolling upgrades,** an engine and workers of different wires, by
  negotiated ranges. Out.
