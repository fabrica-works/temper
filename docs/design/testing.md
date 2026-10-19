# Testing temper

Provisional, 2026-10-03, revised 2026-10-19. How temper is tested. The
strategy is skein's `docs/foundation/testing-strategy.md`: the tiers,
faults, what every fake shares, what every world checks, scenarios and
the referee, the two suites, and where a failure is fixed. This document
keeps what is temper's: its tiers (section 2), its neighbours and their
fakes (sections 3 and 4), its scenarios and referee (section 5), what its
worlds check beyond the strategy (section 6), its layout (section 7),
and what is still open (section 10).

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
  simulator; the real-loop world on one ring; reconcile's world; the
  process tier, the real binary with an owned Forgejo, its runner and
  real git; and the live tier, with a real model.
- **Peers across a wire, the machine beneath.** LLM providers, their
  sign-in's issuers and the forge are peers, whose fakes grow into
  services and meet temper at the wire; Claude Code, the program smith's
  agent runs on a program's login, is faked by smith's fake `claude`
  (4.1); people use temper's client, and their fake is skein's fake
  person, in front of it. The machine (files, processes, and the
  programs they run: git, rg, sh, temper's own) stays one set of state
  machines, with a face for each layer of temper it can sit under.
- **The fake forge is Forgejo-shaped,** API and git in one store, so a
  branch the worker pushes is the head the engine reads. Its protocol
  layer serves Forgejo's API; its git is reached by the fake git program,
  never over HTTP.
- **Checks follow content.** The machine's sh is a small subset of POSIX
  sh, in which fake repositories write their checks (4.3), and the fake
  forge's CI follows a cue in the pushed tree (4.2), so a change that is
  wrong fails in the run and on the forge alike.
- **A referee in every engine world,** on skein's world harness, holds
  its scenarios' expectations (5.2).
- **No fake of temper's own components:** the system world runs the real
  engine and worker.
- **Two suites, and two tiers outside them.** The focused and the fuzzy
  suites keep their 15 s and 60 s, which nextest enforces; the process
  and live tiers run by their scripts, never at the gate.

## 2. temper's tiers

| Tier (testing-strategy.md, section 2) | In temper |
|---|---|
| step tests | every domain crate and every fake; every codec (golden bytes, bounds, fuzz targets) and each translation both ways; git's argument and environment builders, and git's output parsers against transcripts of real git (protocol/git.md, section 9); Forgejo's decoders, skein's, against the conformance's transcripts (skein's `forge.md`, section 11); the setup's machines, files and specs, and the box's machine (protocol/setup.md, 13.1); lib's are skein's |
| domain worlds | each child domain with the world as its parent, and each component's root domain with its child domains; most of temper's tests |
| system worlds | the engine, the worker and smith's agents together; temper's client joined to the engine's root (2.1) |
| machine worlds | the git component against scripted process ends and outputs; the terminal shell's loop against a scripted stdin and stdout; skein's, for the machines it ships, the forge component among them (skein's `forge-connection.md`, section 10) |
| protocol worlds | the forge's: the forge component against the fake forge's protocol layer, deliveries included; reconcile's is skein's (2.2) |
| io worlds | skein's: temper does not retest io |
| simulated worlds | the one-process world: `temper-service` on skein's simulator (2.3, 2.5) |
| real loop | the standalone service, hosted agents, native clients and fake peers in one loop over one real ring (2.4) |
| end to end | the process tier: the real binary, an owned Forgejo and its runner, and real git (2.6) |
| end to end, live | the live tier: the process tier's stories with a real model (2.7) |

Every child domain has a world of its own but the engine's rules, which
keep no state and are tested by their step tests alone: a departure from
testing-strategy.md, 2.2, since with no state there is no sequence for a
world to drive. The process tier departs from the strategy's end to end
(testing-strategy.md, 2.9) twice: its forge is an owned Forgejo with its
runner, never a fake, since temper provisions and runs it; and it runs
outside both suites, by its script, since starting Forgejo takes longer
than a suite's budget.

The tiers in the two suites keep the budgets of 6.1; the process and the
live tiers, outside them, keep their own (2.6, 2.7). The browser tier,
a suite of its own that temper's gate runs, proves only what a browser
alone shows; temper's stories stay in the worlds and the real loop
(client/browser.md, section 4).

### 2.1 System worlds

One runs the real engine driving the real worker: the composed system
world (`tests/system`), whose worker hosts smith's real agent domain over
skein's fake LLM, and whose engine and worker's git meet one fake forge.
It aims at what crosses components: the tree an agent left is the tree the
worker pushes, and the outcome the run accepted is the one the engine
records. Between the two roots it uses the protocol layer's own
translation, without bytes (protocol/channel.md, section 7).

temper's client joined to the engine's root, `tests/client`
(`temper-client-world`), is a system world too: temper's client root and
view, skein's fake person reading the view's tree, jig's client worlds'
faults and jig's check, and temper's engine root on the engine world's
store, hosts and forge (client/README.md, section 10). Section 6.3 says
why it is joined.

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
- **Reconcile's,** in skein (skein's `forge-setup.md`, section 7):
  reconcile's machine against the fake forge's protocol layer serving the
  setup's routes, through bytes: a forge converged from empty and again,
  each thing deleted made again, the refusals, no secret in a message or a
  log (protocol/setup.md, 13.2).
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

`tests/real-loop`, on `skein-world`'s real-world harness
(testing-strategy.md, 2.8). One test process, one thread and one loop
drive the standalone service, its agents, its clients and its fake peers
over one real io_uring ring. This tier checks temper's assembled stack
on the real kernel in the focused suite, within its budgets (6.1).

- **What runs:** `temper-service`'s system module, with the engine,
  store and one worker as standalone runs them; smith's agent hosts;
  the tools' root over jig's native client transport; and skein's fake
  LLM peer on loopback, latency zero. skein's fake issuer joins
  stories that sign in to an LLM provider, and skein's forge peer
  stories that need the forge. Configuration, the store and client
  credentials live in a scratch directory on the real filesystem.
  Listeners are on loopback, on a port the kernel assigns, passed to
  their clients once bound. A story never uses a fixed port.
- **Agents share the loop.** The harness answers the worker's spawn
  with a hosted smith agent, joined by real pipes, and drives its
  `iterate` in the same loop. The agent's LLM is skein's fake peer,
  which plays the story's scripted turns, or, on a program's source,
  smith's fake `claude` (4.1). git is the fake git program
  (4.3); other external programs, such as sh, run as real child
  processes. Launching the `temper agent` executable, with its own
  loop and ring, is checked by the process tier (2.6).
- **Reuse across worlds.** The host composition, fake state machines,
  scenario scripts and outside referee observations are shared with
  the one-process world wherever they apply. Backend setup supplies
  real descriptors, directories and time here. The real loop replays
  neither the simulator's schedule nor its crashes and power losses.
- **The stories:** sign-in, a scripted chat and reply, a wait
  satisfied by another client, and a drained shutdown; a service
  restarted on the same directory that retains the work; a call whose
  answer was lost, retried under its named key and made once (tools.md,
  11.3); the forge's API stories, and stories that clone, push or land,
  on the fake git program (below).
- **An account first.** A story whose agents run adds a key the fake
  LLM peer accepts, from the client, before its first task, and waits
  for its account to be usable: a key needs no sign-in to drive (jig's
  `domain/accounts.md`, 3.4; tools.md, 11.3).
- **LLM accounts' stories** (protocol/credentials.md, section 7),
  each scripted on the fake peers, never by chance:
  - **a sign-in:** on skein's fake issuer, the engine's redirect
    listener on a port the kernel assigns, and the redirect driven by
    the test, as a browser would; the account usable, then a run whose
    credential the fake LLM peer observes as that account's, by its
    scheme, never its value; a sign-in refused, and one past its
    deadline, each settled with no account usable;
  - **limited, refused and unauthorized:** a scripted turn answering
    with the provider's failure (exhausted with a reset and without,
    refused, unauthorized), the run's notice reaching jig's accounts
    child, and the account's state following (jig's
    `domain/accounts.md`, 3.5); the next task waits for its account
    (jig's `domain/authority.md`, 8.3) and runs once the reset passes,
    or once `retry` is asked; a notice of a fenced attempt dropped;
  - **a program's source:** declared from the client, smith's fake
    `claude` scripted signed in, the source verified, and a run through
    smith's program backend calling host tools the engine decides as a
    direct run's; a usage limit the fake reports making the source
    limited, and a login lost making it need sign-in (smith's
    `protocol/programs.md`, 5.6);
  - **after each:** no key, token or record's value in the engine's
    store, a fact, a request, an answer, a trace, the client's small
    store or a printed page (protocol/credentials.md, section 7).
- **Checks between iterations** are the strategy's for the real loop
  (testing-strategy.md, 2.8 and section 6), with the lower worlds'
  outside expectations; real deadlines bound waits, and the fake peers
  stay available until their clients have settled.
- **The forge and git:** skein's forge peer (skein's `fake-forge.md`,
  section 5), which the one-process world hosts too, serves the fake forge on a
  loopback listener and shares its forge with the fake git program
  (4.3). Stories that clone, push or land run on the fake git program
  over that forge, with neither an owned Forgejo nor its runner. Real
  git against the fake forge is an open question (section 10).
- **CI and webhooks.** CI follows the fake forge's content cue (skein's
  `fake-forge.md`, section 2); CI that runs a workflow's steps in the
  shell subset is an open question (section 10). The forge peer
  delivers a hooked repository's webhooks to the engine's webhooks'
  listener (protocol/forge.md, section 5), and polling stays the
  backstop, at paces the stories' configuration sets short (`poll`,
  `poll_max` and `slow`).
- **The person's terminal client.** The real loop also hosts temper's
  terminal client, in the loop as the service is: temper's client root
  and view under skein's text shell, over jig's native transport to the
  service's clients' listener, on a loopback port the kernel assigned.
  It is a general part of the tier, not of one story: a story that
  needs a person at a client in the real loop has one.
  - **The typist** stands where a person sits. It types whole commands
    to the shell's input, as the person would (client/README.md,
    section 6.5), and reads the pages the shell prints at its print
    interval (client/README.md, section 6.3): what it knows of the
    system is what the pages say. It is ordinary Rust in the test
    process, with the process tier's typist as its model
    (`crates/temper/tests/support/terminal.rs`) and no terminal device;
    the shell's input and output are the harness's pipes in the loop.
    It waits by the loop's deadlines, never by sleeping, and reads the
    page after the print that follows its command.
  - **The peers are fakes:** skein's fake LLM, the fake forge and the
    fake git program (4.1 to 4.3), as for the tier's other stories.
  - **The terminal is the real loop's test surface.** People use the
    browser, or a later full-screen terminal; the terminal client is the
    way a real-loop story reads what a person would see
    (client/README.md, section 6.9). The process tier's `temper open`
    tests stay smokes (2.6), outside the gate.
  - **A run's live text is one of its stories.** A person's message
    starts a chat turn, and the fake LLM plays a reply in pieces and
    holds it after its first piece (skein's `fake-llm.md`, section 2.2).
    The typist reads a printed page that shows that piece under
    "Writing…", before the turn commits. The story releases the hold;
    the reply completes, the turn commits, and the next printed page has
    the committed turn and no provisional text. A page printed while the
    reply is held shows the same piece or a longer one, never a shorter
    one; the story asserts that a piece it read is a prefix of the
    committed turn's text.

Most behavioural cases remain in the domain and system worlds. This
tier keeps a few stories of transport, persistence and lifecycle, within
6.1's budgets. `temper ctl`'s lock and DIRECTORY's files run here
(tools.md, 11.3). The process tier keeps executable startup, CLI
parsing, exit statuses, OS process launch and signal handling, the
owned Forgejo and runner, and production's containment (section 6).

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
    its domain, view and jig's native transport (client/README.md, section 2;
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
  sockets are io's own, out of the simulator's faults (skein's `io.md`,
  3.5), so the link's cuts and stalls are jig's channel world's, not
  this world's.
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
    stories, accepting a key the test makes; the owned example's LLM
    providers' endpoints are written at its address, and Claude Code's
    path at smith's fake `claude` (4.1);
  - an owned Forgejo and its runner (below);
  - real git, the machine's, at the absolute path configuration gives it
    (protocol/configuration.md, section 4; protocol/git.md, section 3);
  - the person: skein's fake person in front of temper's client, over
    jig's native transport (client/README.md, section 2); and, in a test of its
    own, `temper open` itself with stdin and stdout piped (client/README.md,
    sections 6 and 10).
- **Its Forgejo is owned,** provisioned by the setup's commands
  (protocol/setup.md, 13.4): the configuration's one forge is owned
  (protocol/setup.md, section 8), its binaries those `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name, which the commands check against temper's
  pins (protocol/setup.md, 6.1); its address, the clients' listener and
  the webhooks' listener are loopback addresses of the test's own. The
  test runs the commands itself, in the order `temper box` would
  (protocol/setup.md, 7.1), refuses to run without either variable, naming
  it, and stops every process it started after. Reconcile makes temper's
  user, its tokens, its id, the org's team and its hook; the test, as the
  person, makes the repository in the setup's org through Forgejo's API,
  with Actions and a workflow whose job runs the repository's check on the
  runner. Never an existing Forgejo.
- **CI on, webhooks on.** The repository is adopted with "It runs CI"
  checked, and the project's landing rule asks for CI and for no gate or
  approval (`"ci": true`, `"gates": []`, `"approvals": []`;
  protocol/configuration.md, section 3): the change lands once its job, run by
  the owned runner, has passed on its head. The webhooks' listener is
  configured, and the org's hook reaches it.
- **The story,** one test:
  1. the person signs in by the link the binary prints, and adds the
     test's key as the deployment's on the accounts page
     (client/README.md, 3.6), which shows its account usable;
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
  scratch directory of their own: tools.md, 11.4.
- **The server's guards,** a third test, against `temper serve`'s
  clients' listener: jig's one attack story that needs no browser, a
  rebound Host (jig's `protocol/browser.md`, section 15): a request
  naming a host off the list is answered `421`, and nothing it asked is
  made. The cross-site request and the framed page need a browser: they
  are the browser tier's (client/browser.md, section 4).
- **The hinted story,** a fourth test (protocol/setup.md, 13.4): the story with
  the keep's `poll`, `poll_max` and `slow` and the change's `stall`
  raised past the story's budget (protocol/configuration.md, section 7), and its
  CI's job outlasting `settle`, so that it lands within its budget only
  if its job's end is delivered (protocol/setup.md, section 10, "Checked").
- **`temper box`,** a fifth test, on a scratch directory of its own
  (protocol/setup.md, 13.4): `temper box` on the owned example prints the
  sign-in link on its stdout; Forgejo killed is restarted by the box, with
  the engine still serving; a stop leaves no process the box started.
- **`temper program-login`,** a sixth test, on a scratch directory of
  its own (protocol/credentials.md, section 3): on the owned example
  with Claude Code's path at smith's fake `claude`, `temper
  program-login CONFIG.json live` makes the programs directory and
  `live` in it, each `0o700`, and replaces itself with the fake's `auth
  login`, which observes `CLAUDE_CONFIG_DIR` at that directory and none
  of the program's credential variables, and writes its login there;
  the command's exit status is the fake's. A name out of bounds, and a
  programs directory readable by another user, are each refused, exit
  1, with nothing made. A deployment started after it, its program
  source declared as `live`, finds the source signed in.
- **The throwaway Forgejo** is the one in skein's `fake-forge.md`,
  section 6, never an existing Forgejo: temper's `tools/git-transcripts`
  records real git against it (protocol/git.md, section 9). temper's
  users give it the binaries `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name, and refuse to run without
  `TEMPER_FORGEJO` (and, for a story with CI, `TEMPER_FORGEJO_RUNNER`),
  naming it.
- **Its referee:** the person's script finishes within its deadline; the
  binary does not exit before it is killed, or its stderr is shown; the
  forge's state, read through Forgejo's API, shows the change; nothing
  the person saw before the kill is lost after it; no process the binary
  or the test started outlives it.
- **Credentials:** none borrowed. The fake LLM peer accepts a key the
  test makes; the person adds it from the client once signed in, as the
  deployment's, and the agents' tools' test with `temper ctl add-key`
  (tools.md, 11.4); it is kept in jig's secrets' store, and no file the
  configuration names holds an LLM account (protocol/credentials.md,
  section 3). The forge's API token, its user and the git identity's
  token are reconcile's, in the forge's directory, and the admin's token
  the preparation's, read by reconcile alone (protocol/setup.md,
  sections 3 and 9).
- **Outside the suites:** nextest profile `process`, run by
  `scripts/proof-one-process.sh`, which builds the binary first. It aims
  at 3 minutes; if CI takes it past, the story with CI becomes a test of
  its own (protocol/setup.md, 13.4).

### 2.7 The live tier

`crates/temper/tests/live.rs`, after jig's (jig's `protocol/service.md`,
section 8; testing-strategy.md, 2.9).

- **What runs:** the process tier's binary, owned Forgejo, real git and
  person, with a real model: the live configuration, `live.json`, is
  the owned example's (protocol/setup.md, 8.1), its LLM providers'
  endpoints the providers' own and Claude Code's path the machine's
  `claude`. Each run is a fresh deployment, with no LLM account until
  the script adds one, in place of the process tier's key (2.6, step
  1), with `temper ctl` signed in by a link the binary
  prints (tools.md, section 10); the script's argument says which
  account, Claude Code's login by default, since it needs nobody.
- **Claude Code, unattended** (the default). The live configuration's
  programs directory is `~/.local/state/temper/live/programs`. The
  developer runs `temper program-login live.json live` once on each
  machine that runs the tier, and signs in as Claude Code asks
  (protocol/credentials.md, section 3). Each run's deployment declares
  the program source `live` (`temper ctl DIRECTORY declare 2 claude_code
  live --key live-program`, tools.md, 5.5, area 5), then waits with
  `wait accounts --until usable`: the name, not the account's number,
  ties the source to the login, which the worker finds signed in.
  Claude Code renews its own login; temper never reads it, and the run
  needs nobody. Its runs are on smith's program backend (protocol/llm.md,
  section 2).
- **ChatGPT, signed in as the run starts.** `scripts/proof-live.sh`
  runs `temper ctl DIRECTORY connect 1 --key live-sign-in`, prints its
  address, and runs `wait accounts --until settled`, within the
  sign-in's deadline, while the developer approves it in a browser on
  the same machine. The account is the run's deployment's, and goes
  with it; its runs are on smith's direct backend.
- **A key, when neither is at hand:** `add-key PROVIDER FILE
  --deployment --key live-key`, from a private file the script's
  argument names, then `wait accounts --until usable`.
- **Nothing borrowed:** no other program's login is read (no
  `~/.codex`), no record or key is in a file the configuration names,
  and the script copies no credential; an account added for a run is in
  its deployment's store, removed with its scratch directory.
- **The stories:** the process tier's chat, answered by the real model;
  and one small change, made by the model in a repository of the owned
  Forgejo, adopted as in 2.6, and pushed by real git. The story
  ends once the change's branch on the forge is at the run's commit; it
  does not wait for the change to land. A story also ends at its task's
  first terminal failure, which fails the test at once, naming it: it
  never waits out the run's 15 minutes after its task has ended. An
  account limited, refused or needing sign-in fails the test the same
  way, naming the account's state and its reset, since its task would
  wait for it (jig's `domain/authority.md`, 8.3).
- **Its referee:** the process tier's, without the restart; and the
  completions, counted from the engine's transcripts. Each completion is
  a turn of main's conversation on either backend: on the program
  backend, each assistant message the program writes (smith's
  `protocol/programs.md`, 5.5), so the count holds for Claude Code's
  runs as for direct ones.
- **Budget:** at most 25 completions and 15 minutes in all. The live
  configuration's charters bound each run's turns so the two stories
  together make at most 25 completions, which smith's agent holds on
  either backend (on the program backend with `--max-turns` behind its
  own count; smith's `protocol/programs.md`, 5.7), the script stops the
  run at 15 minutes, and the test fails when the transcripts count more
  than 25.
  Each run's summary, a failed run's included, records its completions
  and its wall time, which a plan totals (skein's `development.md`, 5.5).
- **Outside the suites:** nextest profile `live`, run only by
  `scripts/proof-live.sh`, one at a time, never at the gate.

## 3. Neighbours and their faces

### 3.1 Peers

A peer's fake grows the layers that the temper component facing it
grows, and each tier joins the two at the lowest layer both have
(testing-strategy.md, 4.1):

```
smith's agent  domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   skein's fake LLM
engine         domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   skein's fake forge
```

People are no peer of temper's: they use temper's client, temper's own,
which reaches the engine over jig's client protocol, and their fake is
skein's fake person in front of it (4.4). A peer can be a client as well as
a server: the fake forge's protocol layer sends signed deliveries to the
engine's webhooks' listener (protocol/forge.md, section 5; skein's
`fake-forge.md`, section 4); and reconcile meets it as the engine does,
through the setup's routes (protocol/setup.md, 13.2).

### 3.2 The machine

The machine's fake grows no protocol or io layers, and shows a face to
whichever layer of temper sits just above it (testing-strategy.md,
4.3). Its domain face answers the tools' file operations, commands and
searches and the checkout's git operations; its io face, io's records,
a spawn with an argument vector among them. Every component's files and
processes go through it: the worker's workspaces and agent processes,
the agent's tools and checks, and the engine's store.

## 4. The fakes

A fake that more than one tier needs is a step crate under `testing/`;
what only one world needs stays in that world, as a script specialised to
what the world aims at. What every fake shares, from its own vocabulary to
faults drawn from the seed, is the strategy's (testing-strategy.md,
section 4).

### 4.1 LLM providers

temper has no fake provider of its own: the engine makes no LLM call, and
the worker's agents are smith's (protocol/llm.md, section 2). The fake is
skein's (skein's `fake-llm.md`):

- **Its domain,** `skein-fake-llm-domain` (skein's `fake-llm.md`,
  section 2), whose scripts cue on content (5.1); a scripted reply can
  come in pieces, held after a piece, so a world observes the reply
  partway (skein's `fake-llm.md`, 2.2).
- **Its peer,** in `skein-fake-peers` (skein's `fake-llm.md`, section 3),
  which the one-process and real-loop worlds host and the process tier
  runs as a process. Its faults move down with it, to the stream: a stream
  cut mid-event or between events, a malformed event, a slow trickle, a
  stall with pings only. One peer accepts several accounts' credentials; a
  scripted turn answers with a provider's failure (skein's `fake-llm.md`,
  2.3), so that a story drives an account's state; and its observations
  say which credential a call used, never a value.
- **Its issuer,** skein's fake issuer (`skein-fake-oauth`; skein's
  `oauth.md`, section 5), for sign-ins with an LLM provider; the test
  drives the redirect to the engine's loopback listener as a browser
  would (2.4).
- **Claude Code** is smith's fake `claude` (smith's
  `protocol/programs.md`, section 8), at the path the agent's
  configuration gives, scripted per story, with usage limits and a
  login lost. The real loop runs it under smith's agent hosts, and the
  process tier under `temper agent` and `temper program-login` (2.6).
- **What temper's stories need of them** is skein's and smith's to
  design (skein's `fake-llm.md`, and skein's `oauth.md`, section 5;
  smith's `protocol/programs.md`, section 8); temper's stories code
  against the fake LLM's script.

### 4.2 The forge

The fake forge is skein's (skein's `fake-forge.md`): its domain, with its
refusals, timing, faults and CI (section 2); its git, in the store the
API reads, so a branch the worker pushes is the head the engine reads
(section 3); its protocol layer, which serves Forgejo's API, signed
deliveries and the setup's routes (section 4); and its forge peer on
loopback (section 5). temper's worlds add:

- **What the engine's worlds need of it** (`domain/root.md`, section 12;
  `domain/forge.md`, section 18): CI that passes, fails or never
  reports; merges that conflict; webhooks that come late or not at all;
  requests that fail or hit a rate limit; people who comment and edit.
  The domain's faults give them, drawn from the world's seed.
- **Its fixtures.** The scenario gives the protocol layer each
  repository's id and its table of API tokens and the users they stand
  for (skein's `fake-forge.md`, section 4), and the repositories their
  files and checks (5.1).
- **Its costs.** The protocol layer counts the requests it serves, by
  route, which the engine's cost checks hold to ceilings
  (protocol/forge.md, section 10).
- **Its deliveries** go to the engine's webhooks' listener: the world
  writes each on a connection to it, as Forgejo sends one
  (protocol/forge.md, section 5), and the forge peer does so in the real
  loop (2.4).
- **Its setup routes** serve reconcile's tests (protocol/setup.md, 13.2).
- **Its referee** drains the forge's observations (skein's
  `fake-forge.md`, 2.4; here, 5.2).
- **Its git** is reached through its typed transport, never over HTTP:
  by the fake git program in the one-process world (4.3), as by the fake
  checkout in the domain worlds. Real git meets an owned Forgejo in the
  process and live tiers, and skein's throwaway Forgejo in temper's
  transcripts (2.6). The real-loop world's repository stories run on the
  fake git program too (2.4); a real-git bridge is an open question
  (section 10).
- **CI follows content,** by the fake forge's cue (skein's
  `fake-forge.md`, 2.3; here, 2.4). CI that runs a repository's checks
  in the shell subset (4.3) on the pushed tree, so that a change that is
  wrong fails on the forge as it fails in the run, is an open question
  (section 10).
- **GitHub's API** is an open question (protocol/forge.md, section 14).

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
reaches the owned Forgejo over HTTP. Working trees and their git
directories are the machine's, as skein's fake checkout keeps them, the
repositories are the fake forge's, and the transport carries git's calls
between them; the transport and the remote boundary are skein's
(skein's `fake-forge.md`, section 3, and skein's `fake-checkout.md`,
section 4).

**The fake git program,** `testing/temper-fake-git`, is git as the
machine runs it below the git component: a program on skein's fake
machine whose protocol is its command line. skein's fake machine runs
no program of an application's, so it is a program hosted by skein's
world harness (its `HostedProgram`, spawned by name), acting on the
fake machine's files. It stays temper's, as it fakes temper's git
component's program, and is built on skein's fake forge: it reaches the
forge through the typed transport of the fake forge's domain (skein's
`fake-forge.md`, section 3), and shares that forge with the forge peer
(skein's `fake-forge.md`, section 5) where a world hosts both (2.4, 2.5).

- **What it takes:** the invocations of protocol/git.md, section 4, with
  their settings (protocol/git.md, section 3) and their environment
  (protocol/git.md, section 5), and nothing else. Any other argument
  vector exits as git's usage error does, which fails the test that caused
  it.
- **What it does:** it acts on the repositories' files on the machine,
  keeping its git state in each repository's `.git` as skein's fake
  checkout keeps it, and reaches the fake forge through its typed
  transport. Its commits are the fake forge's counts, written as 40-digit
  ids (skein's `fake-forge.md`, section 4).
- **What it prints:** what real git prints for those invocations: the
  porcelain lines, the NUL-separated paths and the ids protocol/git.md
  reads, and git's exit statuses; and, for the fake forge's refusals and
  faults (a missing repository, branch or commit, a wrong token, a
  protected branch, the forge out of reach, drawn from the seed), the
  stderr lines real git prints for them, which protocol/git.md, section 6
  classifies.
- **What it checks,** as the providers' fakes check their clients: the
  credential only in `GIT_CONFIG_*`'s `http.extraHeader`, for an identity
  the fake forge knows, a wrong one answered as Forgejo answers it; no
  `PATH`; the settings present; the remote a `https` URL, or a loopback
  `http` one.
- **Checked against real git:** its answers to the transcripts of
  `tools/git-transcripts` match real git's, masked (protocol/git.md, section 9).
- **Where it runs:** the one-process world (2.5) and the real loop
  (2.4). The domain worlds keep the fake checkout at the domain face.

**The shell subset.** An LLM can type any command, and no fake can run a
real build. The machine's sh interprets a small subset of POSIX sh: a few
commands (`test`, `grep -q`, `cat`, `echo`, `exit`) joined by `&&`, `||`
and `;`. The fake LLM's scripts write their commands in it, and fake
repositories write their checks (`.temper/pre-pr`) in it. This gives
two things:

- **Checks follow content.** An edit is right or wrong, and the check and
  the run say so. A scenario can tell a run that fixed the bug from one
  that only said it did.
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
  front of temper's terminal client (client/README.md, section 2), in the
  one-process world and the process tier (client/README.md, section 10);
  in the real loop (2.4) a typist takes its place: ordinary Rust that
  types commands and reads the printed pages.
  temper has no fake person of its own (client/README.md, section 2).

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
  in-memory mode (protocol/store.md, section 6), and jig's fake
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
`temper-service`'s limits module (protocol/configuration.md, section 7),
in every world, fixture and process-tier configuration: none is set by
hand. A scenario that needs another value changes it by name on the
profile's, with the reason beside it. A limit set by hand hides what the
shipped profile refuses, and the shipped numbers drift toward the
fixtures'.

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
- **A failure in the real-loop, process or live tier** is reported with
  the trace and the files the run left, since the seed does not replay
  there.

The referee is built on skein's world harness (`skein-world`; skein's
`testing.md`, section 5), as ordinary Rust: each world writes its
scenario's expectations, with the stimuli it may inject, and the
harness judges them.

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
- **Secrets stay beside the domains:** in every world with the protocol
  layer, no git identity's value or API token appears in a domain's
  state, a fact, a store record or a log; the fake git program checks
  that a credential reaches git only through its environment (4.3).
- **The process tier** adds that what the binary started ends with it:
  after a drain or a kill, no agent or git process of its remains; and
  that production's containment holds: a stopped run's process tree is
  empty, and a command cannot write a git directory.

### 6.1 Focused tests and fuzzy tests

The two suites are the strategy's (testing-strategy.md, section 8), run
by the commands in `docs/development/workflow.md`. In temper:

- **Focused tests** are the strategy's, with facts changing nothing (a
  run is the same whether its facts are kept or dropped): every test
  binary of a world but its fuzzy ones, the real-loop world's stories
  among them (2.4).
- **Fuzzy tests** are each world's `tests/fuzzy_*.rs` binaries. A sweep
  settles each random world under every invariant, and reaches every
  ending among them.
- **Budgets.** The suites' 15 s and 60 s are rules, which nextest
  enforces. Within them, guides: the worlds below the domain (the
  protocol, machine, client, one-process and real-loop worlds) together
  at most 4 s of the default suite and 25 s of the fuzzy suite; a focused
  protocol-world test at most 0.5 s, and a one-process or real-loop story
  at most 1 s.
  A test that misses its guide is made cheaper or moved to the fuzzy
  suite, never merged with another and never named as an exception: a
  real-loop story with CI and webhooks (2.4) is held to the 1 s as any
  other.
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

### 6.3 The client and the engine, joined

A client's world on a scripted engine agrees with the client by
construction: only a world that joins the client to its engine shows
where either side has drifted from their contract. So `tests/client`
joins temper's client to temper's engine root, and there is no client
world of temper's on a scripted engine (client/README.md, section 10).

- **The translation** between the client's vocabulary and the engine's
  is exhaustive both ways, so a variant added on either side without
  the other is a build error.
- **The joint world is the client's acceptance.** Each client feature
  has its stories there, or extends one.
- **jig's scripted engine** is jig's; the forge's client family has no
  scripted engine of temper's, so its stories run against temper's
  engine root alone.
- **The agents' tools** run there too: their root, as a scripted agent
  beside the fake person (tools.md, 11.2).
- **A run's live text** has its story there, with a scripted host in
  place of an agent: the host sends a run's text in pieces, held after
  the first (jig's `protocol/channel.md`, 6.4), and the fake person
  reads the chat's provisional "Writing…" turn with that piece, then the
  rest of the held text, and then the committed turn, which replaces it.
  Retries, gaps and cuts are jig's views' to test (jig's
  `domain/engine.md`, section 11); this story checks temper's wiring
  from the engine's root to the chat's page. The same story over real
  agents runs in the real loop (2.4).

## 7. Layout

```
crates/*/src/tests.rs               step tests
tests/codecs                        temper-codecs-world: the generated codecs and the foreign documents' decoders
crates/temper/tests/process.rs      the process tier (2.6): the story, the terminal, the server's guards, the hinted story, `temper box`; nextest profile process
crates/temper/tests/live.rs         the live tier (2.7), nextest profile live
crates/temper-client-shell-terminal/tests  the terminal's machine world (client/README.md, section 10)
testing/temper-fake-git             the fake git program, on skein's fake machine (4.3)
tests/<component>/<child>           a child domain's world, temper-<component>-<child>-world
tests/<component>/domain            a component's world: the engine's, the worker's
tests/engine/forge                  the forge connector's world
tests/engine/protocol-forge         the forge's protocol world (2.2)
tests/worker/protocol-git           the git component's machine world (protocol/git.md, section 9)
tests/system                        the composed system world (2.1)
tests/conformance                   the engine root on jig's conformance harness
tests/client                        temper-client-world: temper's client with skein's fake person, joined to the engine's root (2.1, 6.3)
tests/one-process                   the one-process world (2.5)
tests/real-loop                     the real-loop world (2.4), in the focused suite
tests/*/*/tests/*.rs                a world's focused tests
tests/*/*/tests/fuzzy_*.rs          its fuzzy tests: random worlds, domains driven at random
tests/fake-checkout                 temper-fake-checkout: working trees' git, over skein's fake checkout
tools/git-transcripts               git's transcripts, from real git against skein's throwaway Forgejo (protocol/git.md, section 9)
scripts/proof-one-process.sh        the process tier
scripts/proof-live.sh               the live tier
```

- **The worlds and the harness** are skein's world harness
  (`skein-world`); each world's crate is a library other worlds may use
  (4.5).
- **The machine-level fakes are skein's:** the fake machine
  (`skein-fake-machine`), the fake checkout, the fake LLM and its peer,
  the fake issuer. temper's fakes sit on them.
- **The fake forge is skein's** (skein's `fake-forge.md`): its domain
  (`testing/skein-fake-forge-domain`), its protocol layer
  (`testing/skein-fake-forge-protocol`), its forge peer (in
  `testing/skein-fake-peers`), the throwaway Forgejo
  (`testing/skein-throwaway-forgejo`), its tests in ordinary Rust, and
  the conformance (`tests/forge/conformance`). Reconcile's world is
  in skein's `forge-setup.md`, section 7. temper's fake git program, in
  `testing/temper-fake-git`, sits on the fake forge (4.3).
- **Machine worlds live beside what they test:** the forge component's is
  in skein's `forge-connection.md`, section 10; the git component's is in
  `tests/worker/protocol-git` (protocol/git.md, section 9); the terminal's
  in `temper-client-shell-terminal`'s own tests (client/README.md, section
  10). temper's protocol world for the forge, over skein's crates, is
  `tests/engine/protocol-forge` (protocol/forge.md, section 10).
- **The setup's crates** test as step code does: `temper-forge-setup`'s
  files, specs and steps, and `temper-box`'s machine, by their step
  tests (protocol/setup.md, 13.1); reconcile's machine by skein's world;
  the hosts, `temper-forge-setup-shell` and `temper-box`'s own, by the
  process tier (2.6), as the binary's are.

## 10. Open questions

- **Real git against the fake forge.** How to connect the real-loop
  world's git subprocesses to the fake forge while retaining one
  authoritative repository state. A candidate is real bare repositories
  in the scratch directory, with an adapter for git access and the fake
  API's reads and merges. The design must keep object identities, refs,
  protection and merge results consistent, and reuse the fake's policies
  and scenario expectations. It is designed against skein's fake forge,
  in skein's `fake-forge.md`, section 3. Default: the real loop's
  clone, push and landing stories run on the fake git program over the
  shared forge (2.4), and real git meets a forge in the process tier
  (2.6); no story waits for the bridge.
- **What the real loop leaves out:** the real-git bridge (above),
  CI that runs a workflow's steps in the shell subset (how much sh,
  below), faults in the loop, and several real workers in one world.
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
- **The process tier's privileges:** the containment it shares with
  production needs user namespaces and a delegated cgroup on the test
  host. Default: process groups.
- **The machine's git in the process tier.** Default: whatever git the
  test host has, 2.41 or later, which protocol/git.md's start refuses below
  (protocol/git.md, section 3); transcripts are of 2.53.
