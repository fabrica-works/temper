# A person's first run

Draft, 2026-10-20. What a person sees the first time they run temper,
and the general pieces it is made of. It records a design session with
the user on 2026-10-20, which started from the assessment of where the
first run stands once the accounts and GitHub work land
(`~/src/rust/tmp/temper-first-time-ux-2026-10-20.md`). It crosses skein
(the arguments' rule, the fakes as product crates) and temper (its
directories, `temper init`, the box, the client) on purpose; an owner
splits it. It builds on `draft/github.md` as it lands: forges connected
from the client, credentials taken at runtime and the forges' records
in jig's secrets' store.

The first run is sugar. Every piece below has a place of its own in the
layering and is useful without the first run; the first run only
arranges them. Nothing here shapes a piece for the first run's sake.

## 1. In one page

- **One command, no argument:** `temper box`. With an empty
  configuration directory it writes a default document there, starts
  what that document names, and prints one link (section 2).
- **The link signs the person in** and opens the default project's
  chat, on a demo repository, with a demo model and a demo forge:
  temper's fakes, which run as the box's steps (sections 3.5, 3.6).
- **Real backends are added, never swapped in.** Codex or Claude is an
  LLM account beside the demo model; GitHub or a Forgejo is a forge
  beside the demo forge. The demo project stays on its fakes (2.2).
- **Three directories,** XDG's: configuration, data and cache, each a
  flag, an environment variable and a default, in that order of
  precedence (3.1, 3.2).
- **`temper init`** writes the default document, a command of its own,
  which answers setup.md's open question of a command that writes the
  document (3.3).
- **GitHub and an owned Forgejo are in the default document,** so
  connecting either needs no restart. An owned Forgejo there is dormant
  until the person asks for it; then temper fetches its pinned binaries
  into the cache and starts it (3.7, 3.8). A Forgejo elsewhere is at an
  address no default knows, and so is a restart (section 4).
- **The name stays `temper box`,** provisional. The binary's commands as
  a whole (engine, worker, standalone, the box) want a session of their
  own (section 8).

## 2. The person's way

### 2.1 The first run

```
$ temper box
box: wrote the configuration, ~/.config/temper/temper.json
box: demo forge, demo model, temper serve: started
Open http://127.0.0.1:<port>/#sign-in=...
```

1. `temper box` finds `$XDG_CONFIG_HOME/temper` empty, runs `temper
   init` on it (3.3), and starts the document's steps (3.9).
2. The person opens the link in a browser on the same machine. It signs
   them in as the deployment's owner (`../client/README.md`, section 7)
   and lands on the default project's chat.
3. The project has the demo repository adopted, on the demo forge, and
   the demo model as its account (3.6). A banner says it is a demo and
   offers to connect a real model and a real forge.
4. The person tries a suggested request; the demo model plays its
   script and the change goes through temper's procedure on the demo
   forge. A link to the forge's pull request opens a page the demo
   forge does not serve, for now (section 7).

The mockup of what the person sees is `../client/mockups/first-run.html`.

### 2.2 Going real

- **A model.** The accounts page, as the accounts work leaves it:
  Continue with ChatGPT, use a Claude subscription, or an API key. The
  new account becomes the deployment's default; the demo model stays an
  account and stays the demo project's.
- **A forge.** The Forges page, as `draft/github.md` leaves it: Connect
  GitHub (managed, by the manifest flow, or connected, an App pasted),
  Set up a Forgejo here (owned, 3.8), or a Forgejo elsewhere (managed
  or connected, section 4). A forge connected, the person adopts a
  repository into a new project, or into the default one.
- **The demo** can be hidden; its forge and model stay in the document,
  so a demo is always there to show someone.

## 3. The pieces

| Piece | Its home | Useful alone for |
|---|---|---|
| A command's arguments, environment and defaults (3.1) | skein: `shell.md`, 6.5 | every binary of every repository |
| Configuration, data and cache directories (3.2) | temper: `protocol/configuration.md`; jig for its parts' paths | every deployment, with or without the box |
| `temper init` (3.3) | temper: `protocol/setup.md`, section 8 | a person, a test or QA starting a deployment |
| The fakes as product crates (3.5) | skein: `crates/`, from `testing/` | QA, the real loop, demos, a person trying a charter |
| The fakes as the box's steps (3.5) | temper: `protocol/setup.md`, section 7 | QA's "temper in a box" (`qa.md`, 4.1) |
| Adoption from a client tool (3.6) | temper: `../client/tools.md` | scripts and agents adopting repositories |
| `temper forge fetch` (3.7) | temper: `protocol/setup.md`, 6.1; skein: the pins | any owned Forgejo |
| A dormant owned forge (3.8) | temper: `protocol/setup.md`, sections 5 to 7 | any deployment offering an owned forge later |

### 3.1 Arguments, environment and defaults

skein's rule for every binary (skein's `shell.md`, 6.5, landed
2026-10-20): a flag over an environment variable, `TEMPER_<NAME>`, over
a default; directories defaulting to XDG's base directories. temper's
commands adopt it with their next change.

### 3.2 The directories

| Directory | Flag, variable | Default | Holds | Backed up |
|---|---|---|---|---|
| configuration | `--config`, `TEMPER_CONFIG` | `$XDG_CONFIG_HOME/temper` | the document, `temper.json`; the charters' texts and the files the document names as configuration | as dotfiles are |
| data | `--data`, `TEMPER_DATA` | `$XDG_DATA_HOME/temper` | the store, jig's secrets' store, the worker's secret and digest, each set-up forge's directory, programs' login directories | yes: it is the deployment |
| cache | `--cache`, `TEMPER_CACHE` | `$XDG_CACHE_HOME/temper` | fetched binaries (3.7) | no: refetched |

- **Every command that reads the document** takes the three: `temper
  serve`, `temper forge`, `temper box`, `temper init`, `temper
  program-login`. A positional `CONFIG.json` goes.
- **Every path in the document says its directory** by its type: a
  configuration path, a data path or a cache path, each relative to its
  directory, or absolute. Today paths are relative to the working
  directory (configuration.md, section 2); the root's own paths and
  jig's parts' are both concerned, so jig's parts take the same three
  roots.
- **Secrets never sit in the configuration directory,** which people
  copy, sync and publish; the worker's secret moves to data.
- **The box passes its three on** to each step, as flags.

### 3.3 `temper init`

`temper init [--config DIR] [--data DIR] [--cache DIR]` writes a
deployment that starts:

- **The document,** from the default of 3.4, and the charters' texts it
  names; refused, naming the file, when the configuration directory
  holds a document already.
- **The worker's secret and its digest,** in the data directory, private.
- **The clients' listener's port, chosen once:** the default port when
  it can be bound, or one the kernel assigns, written into the
  document. The document always names a fixed port, as a configured
  GitHub forge requires (configuration.md, section 8); a later start on
  a port taken refuses, naming it and the document, as any start does.
  An owned forge's port is chosen the same way.
- **Nothing else:** no account, no forge credential, no repository,
  which are runtime's.

`temper box` with an empty configuration directory runs it first, and
says so; otherwise it never writes the document.

### 3.4 The default document

The first default is a working minimum, for the mechanics and the
experience; good defaults for a person's real work are open (section 8).

- **LLM providers:** OpenAI's, as the owned example's (setup.md, 8.1),
  with no account; and the demo model, an endpoint on loopback in
  plaintext, as the real loop's fake LLM peer is reached.
- **Forges,** each by its stable number: 1 the demo forge, 2 GitHub, 3
  an owned Forgejo, dormant (3.8). GitHub's addresses are its defaults;
  it starts not connected (`draft/github.md`, section 4).
- **A project,** number 1, with landing rules that land on green CI
  with no human approval, and one chat charter.
- **The box's steps:** the demo model and the demo forge, `temper
  serve`, and the owned forge's steps, dormant.

### 3.5 The fakes, as product

- **skein's fake LLM peer and fake forge move to `crates/`,** product
  crates on the same footing as the others, which the worlds and the
  real loop use as now. They keep their contracts (skein's
  `fake-llm.md`, `fake-forge.md`); only their place, and the guarantee
  that they ship, change.
- **temper serves each as a command,** `temper fake llm` and `temper
  fake forge`, a long step of the box, its address and its script or
  seed from the document. The setup's `mode` gains `fake` beside
  `managed` and `owned`: a forge the box runs from temper's own binary,
  which writes a handover at its start as an owned forge's preparation
  does, so that the engine's setup host connects and reconciles it as
  an owned forge's (setup.md, 6.5; `draft/github.md`, section 13).
  Reconcile, the bot and adoption's access run against the fake forge
  as they would against Forgejo.
- **The demo model's script** is the document's: suggested requests
  each with its scripted turns, and an answer for anything else, which
  says it is the demo model and offers to connect a real one.
- **The demo's state** resets at every start of the box (but see
  section 8).

### 3.6 The demo's seed

- **The repository** is the fake forge's seed: `demo/widget`, a few
  files and a check workflow the fake's CI answers.
- **Adoption is the client's,** never configuration's: repositories are
  never configured (configuration.md, section 8). Once `temper serve`
  is up, the box runs one client tool, `temper ctl` (`../client/tools.md`),
  which adds the demo model's key and adopts `demo/widget` into project
  1, as the real loop's stories do (`../domain/testing.md`, 2.4). Run
  again, each act finds itself done.

### 3.7 `temper forge fetch`

- **Fetches the pinned binaries** of Forgejo and its runner into the
  cache directory, when missing: `skein-forge`'s `VERSION` and
  `RUNNER_VERSION` (setup.md, 6.1), from their releases.
- **Checks each one's SHA-256,** pinned beside its version: setup.md
  15's digests move from later to now, required for fetched binaries.
- **A once step** of the box, before an owned forge's preparation, and a
  command of its own. An owned forge's binaries become a path, as now,
  or `"fetched"`, which names the cache's.

### 3.8 A dormant owned forge

- **An owned forge in the document may be dormant:** the box knows its
  steps and starts none until the person asks, and the client shows Set
  up a Forgejo here.
- **The person asks from the client;** the engine asks the box. The box
  is the engine's parent, so the ask goes over a pipe the box gives
  `temper serve` at spawn, carrying one request: start a dormant forge
  the document names. The box runs fetch, prepare and run, records the
  forge as started in its data directory, so that later starts run it,
  and answers. Run without the box (setup.md, 7.4), the client names
  the commands to run instead.
- **The engine takes the handover at runtime,** through the same
  connect a pasted managed admin's token takes (`draft/github.md`,
  12.2), where today it takes it only at its start.

### 3.9 The box, changed

- **`temper box [--config] [--data] [--cache]`,** no positional
  argument; init when empty (3.3).
- **Its steps,** from the document: the fakes' (3.5), fetch, prepare
  and run for each started owned forge, then `temper serve`; a dormant
  forge's when asked (3.8). Its policy is section 7.2's, unchanged.
- **Its stdout** is still `temper serve`'s, the link among it.

## 4. What needs a restart

| Change | Restart? |
|---|---|
| An LLM account added, any provider in the document | no: jig's accounts at runtime |
| GitHub connected, managed or connected | no: in the document, connected at runtime (`draft/github.md`, section 4) |
| An owned Forgejo set up | no: dormant in the document (3.8) |
| A Forgejo elsewhere, managed or connected | yes: its address is the document's. The client says what to add and that temper restarts |
| Anything else in the document | yes, until a hot restart (section 7) |

## 5. Order

1. **The arguments' rule:** its function in skein's shell (its design
   landed, `shell.md`, 6.5); each binary adopts it with its own next
   change.
2. **Directories, `temper init`, the box changed,** on the default
   document without the fakes: the person signs into an empty project.
   Waits for the accounts' second part and the GitHub work to land.
3. **The demo, mechanics first:** the fakes moved and served, the
   `fake` mode, the seed and the box's client tool. Its bar is a chat
   that works on the demo model, and the demo forge registered and
   working as far as today's pieces allow: the fake git program on it
   (8), its state gone at a restart as it happens to be. Section 8's demo questions,
   the defaults and the content are its follow-up.
4. **Going real from the client:** fetch, the dormant owned forge, the
   box's pipe, the handover at runtime, and the client's banner and
   wording.

## 6. Testing

- **The process tier** (setup.md, 13.4) runs `temper box` on empty
  scratch directories with every directory flag given, and checks the
  link signs in and the demo project's chat answers a suggested
  request. No test uses the defaults under `$HOME`.
- **`temper init`** in step tests: the document it writes checks as
  configuration.md, section 8 says, port chosen and port fallen back.
- **The arguments' rule** in skein's own tests: flag, variable, default,
  an empty variable.
- **Fetch** against a local file server with a wrong digest, refused.

## 7. Later

- **The demo forge's pages:** its pull request's link now answers 404.
  Either the demo forge serves a few read-only pages, or temper shows a
  change's diff itself, which would also show more of a change's
  progress on any forge (`../client/forge.md`).
- **A hot restart:** the process stays, its listeners and connections
  with it, and the domain is built again from a new document as a start
  builds it. It would make section 4's last rows no restart. An
  investigation of the code (2026-10-20,
  `hot-restart-report.md`) finds it
  feasible, medium for a first version, large for any change at any
  time:
  - **The seam exists:** `Service` keeps io, the store and the worker
    apart from `Engine`'s domain, in temper as in jig's reference
    service; skein's loop needs nothing.
  - **What is new is a lifecycle,** pause, drain, rebuild, recover,
    rebind, distinct from shutdown, which stops clients and workers,
    and from a cold `Restart`, which jig's core takes only once.
  - **"No task in flight" is not needed,** since recovery already
    resumes durable work; but jig's adoption treats an in-process
    worker's attempts as lost at a start, so a worker that lives on
    needs a rule of its own, and a first version would require no agent
    attempt running.
  - **Watches are not stored,** so the clients' watches are rebound
    from their full subjects, which jig's protocol does not carry yet.
  - **The first version:** policy, charters, templates and limits
    within the same capacity; the same deployment, listeners, workers
    and endpoints. jig owns the lifecycle and its tests; temper its
    configuration's diff and its wiring.

## 8. Open questions

- **The demo's reset against a lasting store.** The fake forge's state
  is in memory, but the engine's store lasts: after a restart it holds
  an adoption, tasks and pull requests the fresh fake forge lacks.
  Either the fake forge keeps its state in its data directory, or a
  start forgets the demo forge and its project's work and seeds them
  again. Default: as it falls in the first iteration (5, item 3);
  answered in its follow-up.
- **git on the demo forge.** The fake forge's git is reached through a
  typed transport, never over HTTP (`fake-forge.md`, section 3), and
  the real loop runs a fake git program (`../domain/testing.md`, 4.3).
  The demo needs a git that reaches the fake forge across processes,
  not real git: its agent is the fake LLM, and nothing else reads its
  working trees. Default: the fake git program as a program of its
  own, with its transport over the network, as the QA loop needs it
  (`qa-loop.md`, section 11, item 2); the demo then needs no git on
  the machine. Real git against the fake forge is real-loop's later
  real-git bridge, for real agents on the fake forge (`../STATUS.md`).
- **The default port.** Default: one unassigned port, chosen with the
  default document.
- **Good defaults** for a person's real work: charters, landing rules,
  limits, the memory bound (setup.md, section 16). Default: the first
  default of 3.4, refined once the mechanics run.
- **The demo's content:** the repository, its CI, the suggested
  requests and their scripts. Default: one request, refined later.
- **The binary's commands:** engine alone, worker alone, standalone,
  the box; their names and how the box names what it starts. Default:
  `temper box` and today's commands, until a session of its own.
- **A Forgejo elsewhere without a restart:** the client writing the
  document and the box restarting `temper serve`, or the hot restart.
  Default: the person adds it and restarts.
