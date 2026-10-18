# Testing temper

Provisional, 2026-10-03, revised 2026-10-18. How temper is tested. The
strategy is skein's `docs/foundation/testing-strategy.md`: the tiers,
faults, what every fake shares, what every world checks, scenarios and
the referee, the two suites, and where a failure is fixed. This document
keeps what is temper's: its tiers (section 2), its neighbours and their
fakes (sections 3 and 4), its scenarios and referee (section 5), what its
worlds check beyond the strategy (section 6), its layout (section 7),
what is not built yet (section 9), and what is still open (section 10).

## 1. In one page

- **Behaviour is tested in the domain layer.** The engine and the worker
  are each a root domain with child domains, and each has worlds of its
  own; the system world runs the real engine and worker together, with
  smith's agents.
- **Below the domain, one tier per layer it adds:** step tests of every
  codec, translation and parser, against golden bytes and transcripts of
  real peers; machine worlds of the forge component, the git component
  and the terminal shell; the forge's protocol world; temper's client
  joined to the engine's root; the one-process world on skein's
  simulator; reconcile's world; the process tier, the real binary with
  an owned Forgejo, its runner and real git; and the live tier, with a
  real model.
- **Peers across a wire, the machine beneath.** LLM providers and the
  forge are peers, whose fakes grow into services and meet temper at the
  wire; people use temper's client, and their fake is skein's fake
  person, in front of it. The machine (files, processes, and the
  programs they run: git, rg, sh, temper's own) stays one set of state
  machines, with a face for each layer of temper it can sit under.
- **The fake forge is Forgejo-shaped,** API and git in one store, so a
  branch the worker pushes is the head the engine reads. Its protocol
  layer serves Forgejo's API; its git is reached by the fake git program,
  never over HTTP.
- **Checks follow content.** The machine's sh is a small subset of POSIX
  sh, in which fake repositories write their checks and their CI, so a
  change that is wrong fails in the run and on the forge alike.
- **The referee is built,** on skein's world harness, and every engine
  world holds its scenarios' expectations in one.
- **No fake of temper's own components:** the system world runs the real
  engine and worker.
- **Two suites, and two tiers outside them.** The focused and the fuzzy
  suites keep their 15 s and 60 s, which nextest enforces; the process
  and live tiers run by their scripts, never at the gate.

## 2. temper's tiers

| Tier (testing-strategy.md, section 2) | In temper |
|---|---|
| step tests | every domain crate and every fake; every codec (golden bytes, bounds, fuzz targets) and each translation both ways; git's argument and environment builders, and git's output parsers against transcripts of real git (protocol/git.md, section 9); Forgejo's decoders against transcripts of the throwaway Forgejo (protocol/forge.md, section 10); the setup's machines, files and specs, and the box's machine (protocol/setup.md, 13.1); lib's are skein's |
| domain worlds | each child domain with the world as its parent, and each component's root domain with its child domains; most of temper's tests |
| system worlds | the engine, the worker and smith's agents together; temper's client joined to the engine's root (2.1) |
| machine worlds | the forge component against a scripted HTTP peer; the git component against scripted process ends and outputs; the terminal shell's loop against a scripted stdin and stdout; skein's, for the machines it ships |
| protocol worlds | the forge's: the forge component against the fake forge's protocol layer, deliveries included; reconcile's, against the setup's routes (2.2) |
| io worlds | skein's: temper does not retest io |
| simulated worlds | the one-process world: `temper-service` on skein's simulator (2.3, 2.5) |
| real loop | none of its own: the process tier stands for it (2.4) |
| end to end | the process tier: the real binary, an owned Forgejo and its runner, and real git (2.6) |
| end to end, live | the live tier: the process tier's stories with a real model (2.7) |

Every child domain has a world of its own but the engine's rules, which
keep no state and are tested by their step tests alone: a departure from
testing-strategy.md, 2.2, since with no state there is no sequence for a
world to drive.

The tiers in the two suites keep the budgets of 6.1; the process and the
live tiers, outside them, keep their own (2.6, 2.7). A browser tier
comes with temper's browser shell (section 9; web/architecture.md).

### 2.1 System worlds

One is built with the real engine driving the real worker: the composed
system world (`tests/system`), whose worker hosts smith's real agent
domain over skein's fake LLM, and whose engine and worker's git meet one
fake forge. It aims at what crosses components: the tree an agent left is
the tree the worker pushes, and the outcome the run accepted is the one
the engine records. Between the two roots it uses the protocol layer's
own translation, without bytes (protocol/channel.md, section 7).

temper's client joined to the engine's root, `tests/client`
(`temper-client-world`), is a system world too: temper's client root
and view, skein's fake person reading the view's tree, jig's client
worlds' faults and jig's check, and temper's engine root on the engine world's
store, hosts and forge (client.md, section 10). Section 6.3 says why it
is joined from the first increment.

### 2.2 Protocol worlds

Each component's protocol layer meets the protocol layers of its peers'
fakes, joined by in-memory streams cut and joined at random, and the
machine sits there at its io face (3.2). They test what translation adds
in temper.

- **The forge's,** `tests/engine/protocol-forge`: the engine's forge
  component against the fake forge's protocol layer, running the forge
  connector world's stories (`tests/engine/forge`) through bytes, with
  the cost checks counted by HTTP request, and the fake forge's signed
  deliveries to the engine's webhooks' listener, late, out of order or
  never (protocol/forge.md, section 10).
- **Reconcile's,** `tests/forge-setup/reconcile`: reconcile's machine
  against the fake forge's protocol layer serving the setup's routes,
  through bytes: a forge converged from empty and again, each thing
  deleted made again, the refusals, no secret in a message or a log
  (protocol/setup.md, 13.2).
- **The worker's link and the client's connection** have their protocol
  worlds in jig (jig's `protocol/channel.md`, section 12; jig's
  `protocol/client.md`, section 14), and an agent's channel and its LLM
  calls theirs in smith and skein. temper's parts and records cross them
  as bytes those worlds never read; the one-process world carries
  temper's through bytes (2.5).
- **git has no protocol world:** it is a program, not a peer on a stream.
  Its tiers are the git component's machine world, the fake git program
  and real git in the process tier (protocol/git.md, section 9).

### 2.3 Simulated worlds

temper's simulated world is the one-process world (2.5). Every process
in a scenario runs its `iterate` on skein's simulator: `temper serve`'s
service, the agents it spawns, the fake forge as a service, the fake LLM
peer and the person's client. A spawn the worker asks for starts an agent
process or the fake git program, and a kill ends it. The machine sits
behind the simulator (3.2).

### 2.4 The real loop

temper has no real-loop world of its own. The process tier (2.6) runs
temper as it ships, as processes with their own loops, which the strategy
calls end to end (testing-strategy.md, 2.9), and stands for the real
loop: io_uring, signals, process groups that end, real git's output and
a real Forgejo's answers. The machine is real: a scratch directory, with
process groups as standalone has them (protocol/git.md, section 7). The
containment production will use (`domain/worker.md`, section 9) waits for
skein's contained process trees, and so do the checks that it holds
(section 6).

### 2.5 The one-process world

`tests/one-process`, after jig's (jig's `protocol/service.md`, sections 8
and 10).

- **What runs,** on skein's simulator and skein's fake machine:
  - `temper-service`'s system module, the engine's and one worker's
    services in one `iterate`, started from configuration's bytes as
    `temper serve` starts them, joined by the in-process name `workers`;
  - the store, jig's store component on skein-kv, on the simulator's
    crashing disk;
  - the worker's agents, smith's agent processes as `temper agent` runs
    them, hosted as programs of the world, over skein's fake LLM peer;
  - the fake git program for every git invocation (4.3);
  - the fake forge as a service, its protocol layer over its domain
    (4.2);
  - people: skein's fake person in front of temper's terminal client,
    its domain, view and jig's native transport (client.md, section 2;
    4.4).
- **Stories:**
  - a person signs in by the link `temper serve` prints, and a chat is
    answered;
  - a repository is adopted through temper's client, and a goal's change
    is prepared, committed and pushed on the fake forge through the fake
    git program;
  - a merge in progress is resolved and pushed with both parents;
  - a signal mid-run drains the engine, the worker ends, and the next
    start settles the worker's claims at once;
  - the service is killed, or the machine's power lost, at a drawn
    moment, then started again on the same disk, an adoption pending
    among what it finds.
- **Faults:** kills and power loss at drawn submissions; the fake
  forge's and the fake LLM's faults, drawn from the seed; the forge's
  refusals and failures as the fake git program prints them. Joined
  sockets are io's and never reach the simulator (skein's `io.md`, 3.5),
  so the link's cuts and stalls are jig's channel world's, not this
  world's.
- **Its referee:**
  - jig's conformance referee, through bytes, as jig's one-process world
    has it: nothing a party, a host or the forge saw depended on a commit
    that was lost, and a drained shutdown ends with nothing waiting for a
    commit;
  - the engine's referee over the fake forge (5.2): nothing lands on a
    default branch without green CI, and a run's change is the tree its
    agent left;
  - no git identity's value or API token in a domain's state, a store
    record or an agent process's environment.
- **Budget:** a story aims at 1 s (6.1); sweeps of drawn kills are fuzzy
  tests.

### 2.6 The process tier

`crates/temper/tests/process.rs`, after jig's reference binary's (jig's
`protocol/service.md`, section 8): temper as it ships, as processes.

- **What runs:**
  - the real `temper` binary: `temper forge prepare`, `temper forge run`
    for Forgejo and for its runner, `temper forge reconcile`, then
    `temper serve`, on a scratch directory, with a configuration the
    test writes from the owned example (protocol/setup.md, 8.1);
  - skein's fake LLM peer, a process of its own on loopback, over TLS
    with a test root the agent's configuration trusts, scripted for the
    stories;
  - an owned Forgejo and its runner (below);
  - real git, the machine's, at the absolute path configuration gives it
    (protocol/configuration.md, section 4; protocol/git.md, section 3);
  - the person: skein's fake person in front of temper's client, over
    jig's native transport (client.md, section 2); and, in a test of its
    own, `temper open` itself with stdin and stdout piped (client.md,
    sections 6 and 10).
- **Its Forgejo is owned,** provisioned by the setup's commands
  (protocol/setup.md, 13.4): the configuration's one forge is owned (protocol/setup.md,
  section 8), its binaries those `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name, which the commands check against
  temper's pins (protocol/setup.md, 6.1); its address, the clients' listener and
  the webhooks' listener are loopback addresses of the test's own. The
  test runs the commands itself, in the order `temper box` would
  (protocol/setup.md, 7.1), refuses to run without either variable, naming it,
  and stops every process it started after.
  Reconcile makes temper's user, its tokens, its id, the org's team and
  its hook; the test, as the person, makes the repository in the
  setup's org through Forgejo's API, with Actions and a workflow whose
  job runs the repository's check on the runner. Never an existing
  Forgejo.
- **CI on, webhooks on.** The repository is adopted with "It runs CI"
  checked, and the project's landing rule asks for CI and for no gate or
  approval (`"ci": true`, `"gates": []`, `"approvals": []`;
  protocol/configuration.md, section 3): the change lands once its job, run by
  the owned runner, has passed on its head. The webhooks' listener is
  configured, and the org's hook reaches it.
- **The story,** one test:
  1. the person signs in by the link the binary prints;
  2. a chat is answered;
  3. a repository is adopted through temper's client, as above;
  4. a goal is given whose change, its CI passed, lands on the owned
     Forgejo, pushed by real git;
  5. the binary is killed, and started again on the same directory;
  6. the person finds the work, and the change's branch on the forge is
     at the run's commit.

  The person reads the chat that gives the goal, finds the goal on the
  board and its plan on its page, and finds the same after the restart.
  The forge's state is the story's proof of the change, read through
  Forgejo's API.
- **The terminal,** a second test, on a scratch directory of its own:
  `temper open DIRECTORY LINK` with stdin and stdout piped signs in by
  the link `temper serve` printed, prints the inbox, adopts a repository
  as above by commands (the form's fields written with `n words`, then
  Adopt pressed with `n`), and exits 0 on `q` after a drained close. Its
  referee reads the printed pages (the inbox and the adoption's
  outcome) and the exit status.
- **The agents' tools,** `temper ctl` against `temper serve`, on a
  scratch directory of their own: tools.md, 11.3.
- **The server's guards,** a third test, against `temper serve`'s
  clients' listener: jig's one attack story that needs no browser, a
  rebound Host (jig's `protocol/browser.md`, section 15): a request
  naming a host off the list is answered `421`, and nothing it asked is
  made. The cross-site request and the framed page need a browser, and
  wait for temper's browser tier (section 9).
- **The hinted story,** a fourth test (protocol/setup.md, 13.4): the story with
  the keep's `poll`, `poll_max` and `slow` and the change's `stall`
  raised past the story's budget (protocol/configuration.md, section 7), and its
  CI's job outlasting `settle`, so that it lands within its budget only
  if its job's end is delivered (protocol/setup.md, section 10, "Checked").
- **`temper box`,** a fifth test, on a scratch directory of its own
  (protocol/setup.md, 13.4): `temper box` on the owned example prints the sign-in
  link on its stdout; Forgejo killed is restarted by the box, with the
  engine still serving; a stop leaves no process the box started.
- **The throwaway Forgejo** is the tools':
  `testing/temper-throwaway-forgejo`, v17.0 and forgejo-runner v13.2,
  the binaries `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name, with
  SQLite in a scratch directory, listening on loopback only. Its support
  starts it, makes the admin user, temper's forge user, their tokens and
  a repository, and stops and removes the instance after. It refuses to
  run without `TEMPER_FORGEJO`, or with a binary other than the pinned
  patch release (protocol/forge.md, section 10), naming it. Never an existing
  Forgejo. `tools/forgejo-conformance` and `tools/git-transcripts`
  start theirs with it (protocol/forge.md, section 10; protocol/git.md, section 9).
- **Its referee:** the person's script finishes within its deadline; the
  binary does not exit before it is killed, or its stderr is shown; the
  forge's state, read through Forgejo's API, shows the change; nothing
  the person saw before the kill is lost after it; no process the binary
  or the test started outlives it.
- **Credentials:** none borrowed. The fake LLM peer takes the account the
  test configures in a private file; the forge's API token, its user and
  the git identity's token are reconcile's, in the forge's directory,
  and the admin's token the preparation's, read by reconcile alone
  (protocol/setup.md, sections 3 and 9).
- **Outside the suites:** nextest profile `process`, run by
  `scripts/proof-one-process.sh`, which builds the binary first. It aims
  at 3 minutes; if CI takes it past, the story with CI becomes a test of
  its own (protocol/setup.md, 13.4).

### 2.7 The live tier

`crates/temper/tests/live.rs`, after jig's (jig's `protocol/service.md`,
section 8; testing-strategy.md, 2.9).

- **What runs:** the process tier's binary, owned Forgejo, real git and
  person, with a real model: the agent's endpoint is the provider's,
  reached through a borrowed Codex login.
- **The borrowed login:** `scripts/proof-live.sh` reads the access token,
  its expiry and the account's id from `~/.codex/auth.json`, read-only,
  and writes them, and nothing else, to a private file in the run's log
  directory, removed at exit. It refuses to start when the token expires
  within the run's span. The configuration hands that file in as the
  account's source (jig's `protocol/llm.md`, 5.4): the token is never
  refreshed, the refresh token is never copied, and neither an agent nor
  `temper sign-in` signs in.
- **The stories:** the process tier's chat, answered by the real model;
  and one small change, made by the model in a repository of the owned
  Forgejo, adopted as in 2.6, and pushed by real git. The story
  ends once the change's branch on the forge is at the run's commit; it
  does not wait for the change to land. A story also ends at its task's
  first terminal failure, which fails the test at once, naming it: it
  never waits out the run's 15 minutes after its task has ended.
- **Its referee:** the process tier's, without the restart; and the
  completions, counted from the engine's transcripts.
- **Budget:** at most 25 completions and 15 minutes in all. The live
  configuration's charters bound each run's turns so the two stories
  together make at most 25 completions, the script stops the run at 15
  minutes, and the test fails when the transcripts count more than 25.
  Each run's summary, a failed run's included, records its completions
  and its wall time, which a plan totals (skein's development.md, 5.5).
- **Outside the suites:** nextest profile `live`, run only by
  `scripts/proof-live.sh`, one at a time, never at the gate.

## 3. Neighbours and their faces

### 3.1 Peers

A peer's fake grows the layers that the temper component facing it
grows, and each tier joins the two at the lowest layer both have
(testing-strategy.md, 4.1):

```
smith's agent  domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   skein's fake LLM
engine         domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   temper's fake forge
```

People are no peer of temper's: they use temper's client, temper's own,
which reaches the engine over jig's client protocol, and their fake is
skein's fake person in front of it (4.4). A peer can be a client as well as
a server: the fake forge's protocol layer sends signed deliveries to the
engine's webhooks' listener (protocol/forge.md, sections 5 and 9); and reconcile
meets it as the engine does, through the setup's routes (protocol/setup.md,
13.2).

### 3.2 The machine

The machine's fake grows no protocol or io layers (testing-strategy.md,
4.3). It shows a face to whichever layer of temper sits just above it:

| temper is real down to | The machine's face | It answers |
|---|---|---|
| the domain | domain face | typed operations: the tools' file operations, commands and searches; the checkout's git operations |
| the protocol layer | io face | io's records: open, read, store, rename, spawn with an argument vector, bytes on a pipe, an exit |
| io | behind the simulator | the file and process operations the simulator passes on |
| everything | none | the real kernel, in a scratch directory |

Every component's files and processes go through it: the worker's
workspaces and agent processes, the agent's tools and checks, and the
engine's store.

## 4. The fakes

A fake that more than one tier needs is a step crate under `testing/`;
what only one world needs stays in that world, as a script specialised to
what the world aims at. What every fake shares, from its own vocabulary to
faults drawn from the seed, is the strategy's (testing-strategy.md,
section 4).

### 4.1 LLM providers

temper has no fake provider of its own: the engine makes no LLM call, and
the worker's agents are smith's (protocol/llm.md, section 2). The fake is skein's
(skein's `fake-llm.md`):

- **Its domain,** `skein-fake-llm-domain`, a provider seen from the
  inside: it answers each call after a drawn latency, by a world's script
  or at random (failures by configured chance, a number of tool rounds,
  then a final answer), and rejects conversations a real provider would
  reject. Its scripts cue on content (5.1).
- **Its peer,** in `skein-fake-peers`: the server side of each provider
  API smith's agents speak, over HTTP and server-sent events, which the
  one-process world hosts and the process tier runs as a process; with
  skein's fake issuer (`skein-fake-oauth`) for `temper sign-in`. Its
  faults move down with it: a stream cut mid-event or between events, a
  malformed event, a slow trickle, a stall with pings only.

### 4.2 The forge

The fake forge holds what temper meets on a forge: repositories, issues,
pull requests, comments, labels, reviews, CI, merges, wikis and webhooks;
and the git a worker clones, fetches and pushes. Both live in one store,
so a branch the worker pushes is the head the engine reads through the
API, and a merge the engine asks for moves the branch the next fetch
sees.

- **A domain first.** The engine's worlds need one (`domain/root.md`,
  section 12; `domain/forge.md`, section 18): CI that passes, fails or
  never reports; merges that conflict; webhooks that come late or not at
  all; requests that fail or hit a rate limit; people who comment and
  edit. Its git keeps git's rules: a push moves a branch only by a
  fast-forward, and another party can move a branch under a run.
- **Then a service:** its protocol layer,
  `testing/temper-fake-forge-protocol`, serves Forgejo's API over the
  domain, the routes temper uses, on skein's HTTP server, from the
  server side of `temper-forge-forgejo` (protocol/forge.md, section 9). The
  scenario gives it each repository's id, which it answers by, and a
  table of API tokens and the users they stand for; an unknown token
  is answered `401`. It keeps Forgejo's quirks (its default order, its
  page cap, its unpaged threads) and counts the requests it serves, by
  route, which the engine's cost checks hold to ceilings (protocol/forge.md,
  sections 9 and 10). The domain grows the reads its routes need that
  it lacks: Actions' runs, as a commit's run by its number, a run by
  its id and a run's jobs, with its CI's statuses linking their jobs
  as Forgejo's do (protocol/forge.md, section 9). For each of the domain's
  webhooks (`Request::Hook`) it renders a signed delivery, which the
  world writes on a connection to the engine's webhooks' listener, as
  Forgejo sends one (protocol/forge.md, sections 5 and 9). And it serves the
  setup's routes, for reconcile's world: admin users and their tokens,
  orgs, teams and their members, and org hooks (protocol/setup.md, sections 11
  and 13.2), from a store of its protocol layer's beside the routes,
  not its domain's; a hook reconcile makes there is the one its
  deliveries go to. GitHub's API comes after Forgejo's (protocol/forge.md,
  section 14).
- **No git over HTTP.** The fake forge's git is reached through its
  typed transport only: by the fake git program in the one-process world
  (4.3), as by the fake checkout in the domain worlds. Real git meets a
  real forge: an owned Forgejo in the process and live tiers, and the
  throwaway Forgejo for the tools (2.6).
- **CI follows content.** A repository's CI runs its checks in the shell
  subset (4.3) on the pushed tree, so a change that is wrong fails on the
  forge as it fails in the run.

The domain is built, as `testing/temper-fake-forge-domain`, a step crate
with a vocabulary of its own. Its rules:

- **Forgejo-shaped, not better than Forgejo.** It refuses what Forgejo
  refuses (too little permission, a label not defined, an empty title or
  body, a stale head on merge, a merge its base's protection does not
  allow), and keeps what Forgejo keeps and no more. Listings are
  Forgejo's: the items updated at or after a time, at a second's
  resolution, least recently updated first, paged by number, each page
  read from the order as it is when its call arrives, so an item that
  changes during a listing moves to its end and can shift another onto
  a page already read. An item's updated time moves for what moves
  Forgejo's (a comment, a review, its labels, its title or body, a push
  to its head, closing, reopening, merging), not for a status on its
  head nor for a comment edited or deleted; a configuration can make
  either move it. A pending review is hidden until it is submitted, and
  keeps the id it was started with. Stale approvals are dismissed only
  where the protection says so, off by default as on Forgejo. Anyone
  with write permission may edit or delete another's comment, as on
  Forgejo's web, so a person can mangle the engine's record.
- **Timing.** A call is decided as it arrives: refused for its user's
  rate, failed before it is made, made and then timed out, or made, its
  answer held for a drawn latency, sometimes late. A call may also time
  out first and land afterwards, as a request a client gave up on can
  still be acted on. The forge keeps a clock of its own, at its
  resolution and skewed from the world's, and every time it shows is on
  it. Webhooks name the branch and commit they are about, and come after
  a drawn latency, late, out of order, or never.
- **CI is drawn or cued by content.** When a commit becomes a head, each
  of its repository's check contexts goes pending, then passes, fails or
  never reports, by chance; or, where the repository configures a cue,
  passes on a commit whose file at a path holds a marker, as the stand-in
  until CI runs checks in the shell subset. A context may run again once
  after it reported, so a head's CI can move after it settled. People
  with write permission report statuses of their own.
- **Merges** happen at an exact head, as a squash: one commit on the
  base's tip, three-way from the newest commit head and base share, which
  conflicts where both changed a path differently. A repository has at
  most one protected branch, which takes a merge only with the statuses
  and approvals its protection asks for, and refuses pushes, creation and
  deletion. Deleting a branch closes the pull requests it was the head or
  base of.
- **Its git is the only forge git.** Commits live in the forge's one
  store, named by a count so a seed replays to the same names, each a
  parent and a tree; they carry no message. A working tree's git
  (`temper-fake-checkout`) reaches the forge through a typed transport,
  as real git reaches a remote across the network (4.3): the worker's and
  the agent's worlds route it to the fake forge directly, answered in the
  same instant, since io's latency already stands for the network and the
  forge.
- **Seen from outside.** Every change it makes is observed, content and
  all, and so is every write it refused, in a bounded queue a world
  drains for its referee (5.2); a world can also read its store at
  settle, count what it did and refused, and ask how much room is left,
  so that a forge that filled up is told from one that works. Needing
  room, it forgets the statuses of the oldest commit no head shows;
  commits it never forgets.

### 4.3 The machine, and the fake git program

The machine is the host temper's processes run on. It has three parts:

- **Files:** directories, files, links and special files; a version that
  changes with every change; roots that operations resolve beneath, as io
  does; and each process tree's view of them, writable or read-only.
- **Processes:** trees, with pipes, an environment, an exit and a kill;
  the time they take, on the world's clock; deadlines raced as io races
  them.
- **Programs,** what a process runs. Each is a small service whose
  protocol is its command line: a domain of what it does to the files, and
  a face that takes an argument vector and prints the bytes the real
  program prints. The programs are git, whose working trees are the
  machine's and whose remotes are on the fake forge; rg; sh; and temper's
  own. The agent is a program the worker spawns, with its channel on the
  program's pipes.

At the domain face, a world calls what a program does directly: a typed
fetch is the git program's fetch, with no argument vector. From the io
face down, every program runs through its command line. That is what
tests the protocol layer's invocations and its parsing, and only the
invocations temper makes need a face.

The forge's git is not the machine's. The git program reaches the fake
forge as real git does, across the network: through a typed transport
that the world or the simulator routes. In the process tier, real git
reaches the owned Forgejo over HTTP. That split is built:
`temper-fake-checkout` keeps working trees and their git directories, the
commits a tree cloned, fetched or made among its files, while the forge
keeps the repositories, and the transport carries git's calls between
them (where a repository's branches are, a fetch, a push, a branch
created for a base).

**The fake git program,** `testing/temper-fake-git`, is git as the
machine runs it below the git component: a program on skein's fake
machine whose protocol is its command line. skein's fake machine runs
no program of an application's, so it is a program hosted by skein's
world harness (its `HostedProgram`, spawned by name), acting on the
fake machine's files.

- **What it takes:** the invocations of protocol/git.md, section 4, with their
  settings (protocol/git.md, section 3) and their environment (protocol/git.md, section 5),
  and nothing else. Any other argument vector exits as git's usage error
  does, which fails the test that caused it.
- **What it does:** it acts on the repositories' files on the machine,
  keeping its git state in each repository's `.git` as skein's fake
  checkout keeps it, and reaches the fake forge through its typed
  transport. Its commits are the fake forge's counts, written as 40-digit
  ids (protocol/forge.md, section 9).
- **What it prints:** what real git prints for those invocations: the
  porcelain lines, the NUL-separated paths and the ids protocol/git.md reads, and
  git's exit statuses; and, for the fake forge's refusals and faults (a
  missing repository, branch or commit, a wrong token, a protected
  branch, the forge out of reach, drawn from the seed), the stderr lines
  real git prints for them, which protocol/git.md, section 6 classifies.
- **What it checks,** as the providers' fakes check their clients: the
  credential only in `GIT_CONFIG_*`'s `http.extraHeader`, for an identity
  the fake forge knows, a wrong one answered as Forgejo answers it; no
  `PATH`; the settings present; the remote a `https` URL, or a loopback
  `http` one.
- **Checked against real git:** its answers to the transcripts of
  `tools/git-transcripts` match real git's, masked (protocol/git.md, section 9).
- **Where it runs:** the one-process world (2.5). The domain worlds keep
  the fake checkout at the domain face.

The checkout's merge world independently predicts complete trees, both
parents and remote refs for clean and conflicted merges, unchanged merge
trees, saved work, stale expected heads and pushes that land but time out.
Each scenario settles all its typed promises, replays from its seed, and
has the same requests and effects with facts disabled. Its counted-memory
test includes construction and maximum conflict-path sets in retired
preparations alongside live pushes before reclaim.

**The shell subset.** An LLM can type any command, and no fake can run a
real build. The machine's sh interprets a small subset of POSIX sh: a few
commands (`test`, `grep -q`, `cat`, `echo`, `exit`) joined by `&&`, `||`
and `;`. The fake LLM's scripts write their commands in it, and fake
repositories write their checks (`.temper/pre-pr`) and their CI in it.
This gives two things:

- **Checks follow content.** An edit is right or wrong, and the check,
  the run and the forge's CI all say so. A scenario can tell a run that
  fixed the bug from one that only said it did.
- **Scenarios run on a real machine unchanged,** since real sh speaks the
  subset.

A command outside the subset fails as an unknown command does. Programs
a world scripts by name, such as a test suite that takes two hours, stay
as named programs of the machine.

### 4.4 People

People chat, accept and reject plans, release held work, adopt
repositories, and write and correct notes (`domain/root.md`, section 12),
on the forge and through temper's client. Their fake is a scripted actor,
drawn from the seed.

- **On the fake forge** it acts as a forge user.
- **Through temper's client** it is skein's: skein's fake person
  (skein's `ui.md`, 17.1), reading the tree temper's view makes and
  acting on it, with jig's client test kit's faults and check (jig's
  `client/domain/testing.md`), in `tests/client`; and the same person in
  front of temper's terminal client (client.md, section 2), in the
  one-process world and the process tier (client.md, section 10).
  temper has no fake person of its own (client.md, section 9).

In the engine's worlds people are scripted, ordinary Rust rather than a
domain: each story's person looks at the forge as observed, never at its
store, and does what the story calls for next, one thing at a time, so a
call lost to a restarting engine is simply tried again. Where no client
is in the world, they make the engine's own calls. The system world
reuses them.

### 4.5 Temper's own components

temper keeps no fake of its own components: the system world runs the
real engine and worker, joined by the protocol layer's translation
(protocol/channel.md, section 7).

- **The composed system world** (`tests/system`): the real engine root,
  worker root and smith's agent domain; skein's fake LLM, which scripts
  only completions' contents; jig's store component over skein-kv's
  in-memory mode, in place of jig's fake store once temper's families
  have their codecs (protocol/store.md, section 6), and jig's fake
  parties; one fake forge, serving the engine's API and the worker's git
  from one repository state; smith's agent world behind the worker's
  process face, its directories mounted from the trees the worker
  prepared.
- **The whole worker's world** (`tests/worker/domain`): the real worker
  root, its checkout and its hub, with smith's process boundary and
  scripted agents, the protocol layer's hand-outs played with smith's
  codecs, and the fake checkout and forge; it plays jig's worker link as
  jig's worker worlds do: a channel ready or lost, the grace passed.

The system world uses the engine's, the forge connector's and the
worker's worlds' crates as libraries: the store, the forge's protocol as
the world plays it, the referees, the people.

Scripted stand-ins inside a world are another matter, and stay. The
worker's worlds keep agents that hang, crash and break the channel's
rules, and the run's world keeps a host that cancels twice, because the
real ones never do.

## 5. Scenarios

### 5.1 Setting one up

A scenario (testing-strategy.md, section 7) is set up in the terms of
temper's fakes: each conversation's script for the LLM, the repositories
on the forge with their files and checks, the items and the people, and
the faults to inject. A scenario cues its scripts through content: a
repository's files say which script its run follows.

A scenario's limits are the shipped profile's, `small`, built by
`temper-service`'s limits module (protocol/configuration.md, section 7), in every
world, fixture and process-tier configuration: none is set by hand. A
scenario that needs another value changes it by name on the profile's,
with the reason beside it. A limit set by hand hides what the shipped
profile refuses, and the shipped numbers drift toward the fixtures'.

### 5.2 The referee

The referee is the strategy's (testing-strategy.md, section 7). temper's
expectations are of this kind: safety, such as nothing landing on a
default branch without green CI, or no forge credential reaching an
agent; liveness, such as a pull request opened, with green CI, within
the hour. What temper adds:

- **It steps on what it observes,** and on its own timers, not on every
  iteration, so its work per iteration stays bounded. Its earliest
  deadline is covered by the shell's one ring timeout, as every layer's is
  (skein's `programming-model.md`, section 9).
- **A failure in the process or live tier** is reported with the trace
  and the files the run left, since the seed does not replay there.

The referee is built on skein's world harness (`skein-world`), as
ordinary Rust. Each world writes its scenario's expectations: what
its referee observes, the names of what it expects to happen, the
stimuli it may inject, and what it checks of each observation. Through
them the referee checks safety and fails at once, arms a liveness
expectation as a deadline, meets it, or withdraws and re-arms it; a
liveness expectation whose deadline fires fails the test and is no
longer pending, so meeting it later counts for nothing. A stimulus comes
out when it is due, or at once from the observation that calls for it,
so a world can inject it between two things it does, such as a restart
between two writes. The verdict is passed, still open (and what is
pending), stopped early by the scenario, or failed (when, why, and what
was pending then).

Every engine world has a referee, the engine's own included, over what
the forge did and what the engine assigned. Its stimuli are an engine
restarting cold: at a drawn moment; at once as it posts an outcome, so
that it restarts while applying it; or at a moment a scenario chooses as
the forge shows it. The worker's worlds add a worker's channel dropping,
a worker vanishing and a worker told to shut down. The engine's referee
meets a story only when its item closes. The system world checks its
stories independently of the roots, against durable task and call
records and smith's own observations (`tests/system`). The conformance
world (`tests/conformance`) runs the engine root on jig's conformance
harness, under its outside referee. The worker's child domain worlds
keep their checks inline.

## 6. What the tiers check

Every world checks what the strategy lists (testing-strategy.md, section
6). In temper:

- **Contracts** include each operation's deadline and identity, and each
  component's own (`domain/worker.md`, section 10).
- **Memory** is measured per step: the counting allocator records the
  most a domain held in one step, against its worst case.
- **Replay** compares traces: a seed replays to the same trace. A digest
  of the state on replay is not built (section 9).
- **Secrets stay beside the domains:** in every world with the protocol
  layer, no git identity's value or API token appears in a domain's
  state, a fact, a store record or a log; the fake git program checks
  that a credential reaches git only through its environment (4.3).
- **The process tier** adds that what the binary started ends with it:
  after a drain or a kill, no agent or git process of its remains. Once
  contained trees exist, it adds that production's containment holds: a
  stopped run's process tree is empty, and a command cannot write a git
  directory.

### 6.1 Focused tests and fuzzy tests

The two suites are the strategy's (testing-strategy.md, section 8), run
by the commands in `docs/development/workflow.md`. In temper:

- **Focused tests** are the step tests, and each world's scenarios,
  referee tests, replay, facts changing nothing (a run is the same
  whether its facts are kept or dropped), and memory at the worst case:
  every test binary of a world but its fuzzy ones.
- **Fuzzy tests** are each world's `tests/fuzzy_*.rs` binaries. A sweep
  settles each random world under every invariant, and reaches every
  ending among them.
- **Budgets.** The suites' 15 s and 60 s are rules, which nextest
  enforces. Within them, guides: the worlds below the domain (the
  protocol, machine, client and one-process worlds) together at most 4 s
  of the default suite and 25 s of the fuzzy suite; a focused
  protocol-world test at most 0.5 s, and a one-process story at most 1 s.
  A test that misses its guide is made cheaper or moved to the fuzzy
  suite, never merged with another.
- **Outside both suites:** the process tier's binary `process` and the
  live tier's binary `live`, which both suites' filters leave out, run
  under the nextest profiles `process` and `live` by
  `scripts/proof-one-process.sh` and `scripts/proof-live.sh`. The gate
  never runs them.

A fuzzy test lives in its world's crate, beside the focused ones, and
uses that world: a world's settings, random ones included, are the
world's, not either suite's. The suites are told apart by binary name.
Each fuzzy file is a binary of its own, so a memory test's counting
allocator never runs under a sweep.

### 6.2 The worker's runtime world

`tests/worker/domain/src/runtime.rs` drives the real worker root and its
children against scripted engine and process records and deterministic
git terminals. Its independent boundary referee checks complete turn
replay, exact names and cumulative spend, read credit, identical answers,
separate push title and body, merge paths and expected heads, and release
at iteration reclaim points. Focused stories cover busy backoff,
reconnects, crossed acknowledgements, ordinary saved work and
snapshot-free merge parking. Replay compares digests of the full boundary
records and facts; dropping facts leaves every boundary record unchanged.
Referee tests inject a changed duplicate body and an unseen turn prefix
and require rejection. Counted memory in the root and its children
exercises saturated turn windows, transcripts and conflict paths; a small
deterministic sweep varies body lengths and one-to-three count and byte
windows.

### 6.3 The client and the engine, joined from the start

A client's world on a scripted engine agrees with the client by
construction: only a world that joins the client to its engine shows
where either side has drifted from their contract (jig's
`docs/development/tech-debt.md`, section 9). So temper's client is
joined to its engine from its first increment, and `tests/client` is
that joint world itself: there is no client world of temper's on a
scripted engine before it or beside it (client.md, section 10).

- **The translation** between the client's vocabulary and the engine's
  is written with the first types on both sides, exhaustive both ways.
  Every increment that adds a variant on either side extends it, so
  drift is a build error the day it happens, not a finding at the end.
- **One thin story end to end,** such as signing in and seeing the
  inbox, runs in the joint world as soon as the engine can answer it.
  Each increment extends it, or adds its own story there.
- **jig's scripted engine** keeps the contract of jig's documents, and
  jig checks it against jig's engine; the forge's client family has no
  scripted engine of temper's, so its stories run against temper's
  engine root alone.
- **The joint world is the client's acceptance.** A client increment is
  done when its stories pass there.
- **The agents' tools** run there too: their root, as a scripted agent
  beside the fake person (tools.md, 11.2).

## 7. Layout

```
crates/*/src/tests.rs               step tests
crates/temper/tests/process.rs      the process tier (2.6): the story, the terminal, the server's guards, the hinted story, `temper box`; nextest profile process
crates/temper/tests/live.rs         the live tier (2.7), nextest profile live
crates/temper-client-shell-terminal/tests  the terminal's machine world (client.md, section 10)
testing/temper-fake-forge-domain    the fake forge's domain (4.2)
testing/temper-fake-forge-protocol  the fake forge's protocol layer: Forgejo's API over skein's HTTP server
testing/temper-fake-git             the fake git program, on skein's fake machine (4.3)
testing/temper-throwaway-forgejo    the throwaway Forgejo and its runner, for the tools (2.6)
tests/<component>/<child>           a child domain's world, temper-<component>-<child>-world
tests/<component>/domain            a component's world: the engine's, the worker's
tests/engine/forge                  the forge connector's world
tests/engine/protocol-forge         the forge's protocol world (2.2)
tests/worker/protocol-git           the git component's machine world (protocol/git.md, section 9)
tests/forge-setup/reconcile         reconcile's world, temper-forge-setup-reconcile-world (2.2; protocol/setup.md, 13.2)
tests/system                        the composed system world (2.1)
tests/conformance                   the engine root on jig's conformance harness
tests/client                        temper-client-world: temper's client with skein's fake person, joined to the engine's root (2.1, 6.3)
tests/one-process                   the one-process world (2.5)
tests/*/*/tests/*.rs                a world's focused tests
tests/*/*/tests/fuzzy_*.rs          its fuzzy tests: random worlds, domains driven at random
tests/fake-checkout                 temper-fake-checkout: working trees' git, over skein's fake checkout
tests/fake-forge                    the fake forge's tests in ordinary Rust: its memory
tools/forgejo-conformance           Forgejo's facts and transcripts, from the throwaway Forgejo (protocol/forge.md, section 10)
tools/git-transcripts               git's transcripts, from real git against the throwaway Forgejo (protocol/git.md, section 9)
scripts/proof-one-process.sh        the process tier
scripts/proof-live.sh               the live tier
```

- **The worlds and the harness** are skein's world harness
  (`skein-world`); each world's crate is a library other worlds may use
  (4.5).
- **The machine-level fakes are skein's:** the fake machine
  (`skein-fake-machine`), the fake checkout, the fake LLM and its peer,
  the fake issuer. temper's fakes sit on them.
- **Machine worlds live beside what they test:** the forge component's
  in `tests/engine/protocol-forge`'s crate, beside its protocol world
  (protocol/forge.md, section 10); the git component's in
  `tests/worker/protocol-git` (protocol/git.md, section 9); the terminal's in
  `temper-client-shell-terminal`'s own tests (client.md, section 10).
- **The setup's crates** test as step code does: `temper-forge-setup`'s
  machines, files, specs and steps, and `temper-box`'s machine, by their
  step tests (protocol/setup.md, 13.1); reconcile's machine by its world; the
  hosts, `temper-forge-setup-shell` and `temper-box`'s own, by the
  process tier (2.6), as the binary's are.
- **No web crates:** neither `testing/temper-fake-person` nor any
  `tests/web/*` is temper's (client.md, section 9).

## 9. Not built yet

By tier:

- **Step tests:** the codec crates (`temper-protocol-worker-wire`,
  `temper-protocol-store`, `temper-client-protocol-wire`), Forgejo's
  documents, the routing's translations, git's builders and parsers.
- **Transcripts** of the real peers temper speaks to:
  Forgejo's answers, recorded by `tools/forgejo-conformance`, and git's,
  recorded by `tools/git-transcripts`, both against the throwaway
  Forgejo v17.0 that `TEMPER_FORGEJO` names (protocol/forge.md, section 10).
  LLM providers' are skein's.
- **Machine worlds:** the forge component's, the git component's and the
  terminal shell's.
- **System worlds:** `tests/client`; several real workers in one world
  (the engine's world places runs on scripted ones; `domain/worker.md`,
  section 12).
- **Protocol worlds:** the forge's, with the engine's cost checks through
  bytes (protocol/forge.md, section 10).
- **The real store in the domain worlds,** jig's component over
  skein-kv's in-memory mode in place of jig's fake store
  (protocol/store.md, section 6).
- **The one-process world,** with the fake forge as a service and the
  fake git program.
- **The process tier and the live tier,** with their scripts; the
  process tier on owned mode, with CI, webhooks, the hinted story and
  `temper box`'s test (protocol/setup.md, 13.4).
- **The setup's tiers:** its step tests and reconcile's world (protocol/setup.md,
  13.1 and 13.2).
- **The browser tier,** with temper's browser shell: temper's bundle,
  its pages and signing in, in Chromium, a suite of its own outside the
  focused and fuzzy suites (web/architecture.md, section 4).
- **Containment's checks,** which wait for skein's contained process
  trees.
- **The referee** in the worlds that still check their scenarios inline,
  so that those checks run at every tier.
- **Checks:** transition coverage; fuzzing, starting with the domains'
  step functions fed recorded events; and a digest of the state on
  replay.

By fake:

- **Forge:** inline review comments, statuses' descriptions and links,
  commit messages, more than one protected branch in a repository, and CI
  that runs checks in the shell subset; Actions' runs and their jobs,
  for its protocol layer; its protocol layer, Forgejo's API; its signed
  deliveries to the engine's webhooks' listener, and the setup's routes
  (4.2); GitHub's API later (protocol/forge.md, section 14).
- **Machine:** the fake git program; rg and sh as programs with
  command-line faces; the shell subset.
- **People:** none of temper's: skein's fake person, in front of
  temper's view and of temper's terminal client.

## 10. Open questions

- **How much sh:** which commands and syntax the subset takes. It stays a
  subset of POSIX, so that real sh agrees.
- **Where the shell's interpreter lives,** so that the machine and the
  forge's CI share it.
- **Differential checks:** whether a scenario must end the same way at
  every tier it runs at, as a check that the layers below the domain
  decide nothing.
- **What the referee sees in the process tier:** the machine is real
  there and reports nothing, so an expectation about it, such as what an
  agent's environment holds, needs facts that say so, or is checked only
  in the tiers below. Default: checked in the one-process world.
- **The process tier's privileges:** the containment it will share with
  production needs user namespaces and a delegated cgroup on the test
  host. Default: process groups, until skein's contained process trees
  exist.
- **The machine's git in the process tier.** Default: whatever git the
  test host has, 2.41 or later, which protocol/git.md's start refuses below
  (protocol/git.md, section 3); transcripts are of 2.53.
