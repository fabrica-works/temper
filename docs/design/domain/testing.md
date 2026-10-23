# Testing temper

Provisional, 2026-10-03, revised 2026-10-23. How temper is tested. The
strategy is skein's `docs/foundation/testing-strategy.md`: the tiers,
faults, what every fake shares, what every world checks, scenarios and
the referee, the two suites, and where a failure is fixed. This document
keeps what is temper's: its tiers (section 2), its neighbours and their
fakes (sections 3 and 4), its scenarios, referee and the reference case (section 5), what its
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
- **The fake forge speaks Forgejo's dialect and, through its GitHub
  layer, GitHub's** (skein's `fake-forge.md`, section 10), one world per
  forge kind, API and git in one store, so a branch the worker pushes is
  the head the engine reads. Its protocol layer serves the dialect's
  API; its git is reached by the fake git program, through the forge's
  own calls, never git's protocol over HTTP.
- **Checks follow content.** The machine's sh is a small subset of POSIX
  sh, in which fake repositories write their checks (4.3), and the fake
  forge's CI follows a cue in the pushed tree (4.2), so a change that is
  wrong fails in the run and on the forge alike.
- **A referee in every engine world,** on skein's world harness, holds
  its scenarios' expectations (5.2).
- **The reference case runs in the real loop,** step by step, on its
  cast, its scripts and an oracle over effects that every world running
  the case reuses (5.3).
- **No fake of temper's own components:** the system world runs the real
  engine and worker.
- **Two suites, and two tiers outside them.** The focused and the fuzzy
  suites keep their 30 s (15 s before the user's doubling of 10-22,
  provisional) and 120 s (60 s before the user's doubling of 10-22,
  provisional), which nextest enforces; the process and live tiers run by
  their scripts, never at the gate.

## 2. temper's tiers

| Tier (testing-strategy.md, section 2) | In temper |
|---|---|
| step tests | every domain crate and every fake; every codec (golden bytes, bounds, fuzz targets) and each translation both ways; git's argument and environment builders, and git's output parsers against transcripts of real git (../protocol/git.md, section 9); Forgejo's decoders, skein's, against the conformance's transcripts (skein's `forge.md`, section 11), and GitHub's, skein's (skein's `forge-github.md`); the forges' records and a git identity's value, golden bytes and fuzz targets (../protocol/forge.md, 2.1; ../protocol/credentials.md, section 4); `temper-service`'s routing of the forges: start, connect, stop, installations, generations and pasted values, and GitHub's three routes on the clients' listener (../protocol/forge.md, section 10; 2.1); the setup's machines, files and specs, and the box's machine (../protocol/setup.md, 13.1); lib's are skein's |
| domain worlds | each child domain with the world as its parent, and each component's root domain with its child domains; most of temper's tests |
| system worlds | the engine, the worker and smith's agents together; temper's client joined to the engine's root (2.1) |
| machine worlds | the git component against scripted process ends and outputs; the terminal shell's loop against a scripted stdin and stdout; skein's, for the machines it ships, the forge component among them (skein's `forge-connection.md`, section 10) |
| protocol worlds | the forge's: the forge component against the fake forge's protocol layer, deliveries included; reconcile's is skein's (2.2) |
| io worlds | skein's: temper does not retest io |
| simulated worlds | the one-process world: `temper-service` on skein's simulator (2.3, 2.5) |
| real loop | the standalone service, hosted agents, native clients and fake peers in one loop over one real ring; the cluster's engine alone and two workers alone, in the same loop (2.4) |
| end to end | the process tier: the real binary, an owned Forgejo and its runner, and real git; the cluster's processes on the fakes as processes (2.6) |
| end to end, live | the live tier: the process tier's stories with a real model (2.7) |

Every child domain has a world of its own but the engine's rules, which
keep no state and are tested by their step tests alone: a departure from
testing-strategy.md, 2.2, since with no state there is no sequence for a
world to drive. The process tier departs from the strategy's end to end
(testing-strategy.md, 2.9) twice: its forge is an owned Forgejo with its
runner, never a fake, since temper provisions and runs it, but in a
first run's story and the cluster's, whose forge is temper's fake forge
as a process; and it runs outside both suites, by its script, since
starting Forgejo takes longer than a suite's budget.

The tiers in the two suites keep the budgets of 6.1; the process and the
live tiers, outside them, keep their own (2.6, 2.7). The browser tier,
a suite of its own that temper's gate runs, proves only what a browser
alone shows; temper's stories stay in the worlds and the real loop
(../client/browser.md, section 4).

No tier calls github.com or uses a real credential. GitHub's tests run
on the fake's GitHub layer (skein's `fake-forge.md`, section 10). Live
calls to GitHub are only skein's opt-in conformance (skein's
`fake-forge.md`, section 11), run by hand on credentials from the person's private
directory, never in a suite or at the gate. The user's own token is never used.

### 2.1 System worlds

One runs the real engine driving the real worker: the composed system
world (`tests/system`), whose worker hosts smith's real agent domain over
skein's fake LLM, and whose engine and worker's git meet one fake forge.
It aims at what crosses components: the tree an agent left is the tree the
worker pushes, and the outcome the run accepted is the one the engine
records. Between the two roots it uses the protocol layer's own
translation, without bytes (../protocol/channel.md, section 7). With a
GitHub forge on the fake's GitHub layer, it checks the forge connector's
connected state, the picker and GitHub's merge queue as the branch's
landing queue (`forge.md`).

A run's live text has its story there: a chat's reply reaches the
party's run watch before its turn commits. The core asks for the chat's
text, and smith's start carries the ask. smith's agent domain gives its
reply; the world, playing smith's process protocol as it plays every
record of the agent's channel, sends that reply up the channel in three
pieces ahead of its turn, and holds the second piece, the third and then
the turn itself until the watch has read all the text before it. The
worker's root hands each piece to its hub, and feeds the agent's
watchdog with it; the engine's root keeps it as the run's text and
streams it to the watch, which the world opens and acknowledges as
temper's client does. Once the turn goes, it commits, and its text is
the text the watch read. smith's own producer, the fake LLM's named
holds and the bytes between the roots are jig's channel story's (jig's
`protocol/channel.md`, section 12); the same reply in temper's client is
6.3's and 2.4's.

A chat's reply of text alone has its story there, with its Finite
sibling. The chat, Open, is answered by skein's fake LLM with text and
no `finish` call: one model call, one committed reply, the chat parked
with no try spent, and past its idle wait no second call, assignment
or reply (`agent.md`, 4.6). A task ended by its result, as
every goal and delegation is, given the same text alone, is nudged
toward `finish` by smith and finishes with its report on its next
call, with no try spent. The parking and the nudge are smith's and
jig's own worlds'; this story holds temper's choice of mode for each
(`temper-engine-smith`'s start) to them.

temper's client joined to the engine's root, `tests/client`
(`temper-client-world`), is a system world too: temper's client root and
view, skein's fake person reading the view's tree, jig's client worlds'
faults and jig's check, and temper's engine root on the engine world's
store, hosts and forge (../client/README.md, section 10). Section 6.3 says
why it is joined. GitHub's three routes on the clients' listener (the
manifest page, the redirect and the setup address; ../client/forge.md,
9.1) are not tested here: they are `temper-service`'s own tests, as
requests through `fixtures::http::Peer` to the service's routes: each
answer, its headers, and a state used, unknown or past its life
refused.

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
  never (../protocol/forge.md, section 10). The routing is not here:
  start's load of each forge's records, a connect at runtime, a forge
  stopped, GitHub's installations, git's tokens' generations and pasted
  values are `temper-service`'s own tests (../protocol/forge.md, section
  10). The joint's GitHub stories hold the component's half of git's
  tokens under temper's settings: every accepted installation's token
  before the forge's first `Connected`, and each renewed before it
  expires on the fake's clock, at GitHub's hour (../protocol/credentials.md,
  section 4).
- **Reconcile's,** in skein (skein's `forge-setup.md`, section 7):
  reconcile's machine against the fake forge's protocol layer serving the
  setup's routes, through bytes: a forge converged from empty and again,
  each thing deleted made again, the refusals, no secret in a message or a
  log (../protocol/setup.md, 13.2).
- **The worker's link and the client's connection** have their protocol
  worlds in jig (jig's `protocol/channel.md`, section 12; jig's
  `protocol/client.md`, section 14), and an agent's channel and its LLM
  calls theirs in smith and skein. temper's parts and records cross them
  as bytes those worlds never read; the one-process world carries
  temper's through bytes (2.5).
- **git has no protocol world:** it is a program, not a peer on a stream.
  Its tiers are the git component's machine world, the fake git program
  and real git in the process tier (../protocol/git.md, section 9).

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
on the real kernel in the focused suite, within its budgets (6.1). It is
smoke, the testing pyramid's top, a way to climb in confidence: one
end-to-end path per feature, and each behaviour and corner case tested
in the lower world that owns it, never here.

- **What runs:** `temper-service`'s system module, with the engine,
  store and one worker as standalone runs them; smith's agent hosts;
  the tools' root over jig's native client transport; and skein's fake
  LLM peer on loopback, latency zero. skein's fake issuer joins
  stories that sign in to an LLM provider, and skein's forge peer
  stories that need the forge. Configuration, the store and client
  credentials live in a scratch directory on the real filesystem.
  Listeners are on loopback, on a port the kernel assigns, passed to
  their clients once bound. A story never uses a port the kernel did
  not give it; GitHub's stories reserve the clients' listener's before
  the service starts (below).
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
- **The stories,** one end-to-end path each: sign-in, a scripted chat
  and reply, a wait satisfied by another client, and a drained shutdown;
  a service restarted on the same directory that retains the work; a
  call whose answer was lost, asked again under its named key and made
  once; two acts on one DIRECTORY taking `temper ctl`'s lock in turn,
  and a signed-in DIRECTORY private to its user (../client/tools.md,
  11.3); a repository adopted, a hooked repository's webhook delivered,
  and a chat's change cloned, pushed and landed on the fake git program
  (below); a chat adding a landing dependency to its live report delegate,
  kept until the change lands and started after Met (`fuzzy_landing`); the
  first run's green light, a question answered in the person's inbox before
  the change it holds (`fuzzy_green_light`, below); the first run's
  repair, a change whose first head fails CI for its content, repaired
  and landed (`fuzzy_repair`, below); and GitHub's, one per feature of a
  GitHub forge (below).
- **The first run's green light,** one story (`fuzzy_green_light`), on
  the first run's cast (`../client/mockups/README.md`, "The first
  run"): the person asks their chat T1 "Add CSV export to widget"; T1
  delegates the work T2, an agent holding the change's authority; T2
  asks T1 whether to go ahead before it delegates anything, and jig
  puts the question in the inbox of T1's person (jig's
  `domain/people.md`, section 6); the person answers it there with
  `temper ctl answer`; T1, woken by the answer, relays it to T2
  (`message`, its form an answer to the question's number); T2 then
  delegates the change T3, whose coder edits widget's `src/main.rs`; it
  lands with CI green at its head, T2 reports, and T1 says so.
  - **The work that waits asks.** The change's own agents cannot reach
    the person: a question reaches a person's inbox only when the task
    asked is one the person requested, and a coder's requester is its
    change (jig's `domain/tasks.md`, 8.1 and 8.5). The chat cannot ask
    its own person (5.3, 4.4's row), so the question is T2's, asked of
    the chat, as the case's A asks the director (5.3, 4.1's row).
  - **Every judgement a script,** one call per response, each task's
    wakes one cause at a time, so each turn follows the count of replies
    said: T1 delegates T2, says so and waits; on the question, says it
    waits in the person's inbox and waits; on the answer, relays it,
    says so and waits; on T2's report, finishes with its own. T2 asks,
    says so and waits; on the answer, delegates the change and waits; on
    the change's result, reports. The coder reads, edits and finishes;
    the reviewer approves. The numbers are a fresh deployment's, named
    once: the tasks T1 to T5 and the question, message 1, which the
    story checks against T2's tool output ("Message 1 sent") and the
    inbox's entry, since a script cannot read a returned number.
  - **The person's acts,** by `temper ctl`, waiting on watches, never on
    repeated reads: sign in; adopt the repository first, as the first
    run's box adopts widget at its start (a home adopted while a chat is
    parked holds the chat's next run: an engine finding, not this
    story's); start the chat and a watch on its end; wait until the
    question waits in the inbox (`wait inbox --until waiting`) and T1 is
    at rest; read T1's tree, which holds T2 alone; read the question,
    T1's activity holding its words, and answer it; join the watch.
  - **What it checks:** the question in the inbox, on T1 and message 1, and
    in T1's activity in T2's words (ctl's inbox entry names the question,
    not its words), while T1's tree held no change; T1 told the person's
    answer after the question, and T2 told it as T1's answer before it
    delegated the change; the change merged with CI green at its head,
    widget's edit on main; T2's report and T1's; no question left in the
    inbox. The fake forge's one repository stands as widget, its tree
    holding widget's source, and its CI follows the edit (4.2); the story's
    fixture, scripts and checks are its own, in its test file, on the tier's
    shared hosts. It is fuzzy, past the 2 s guide (1 s guide before the
    user's doubling of 10-22, provisional) by the change procedure's landing
    path, as `fuzzy_landing` is, and its forge's records stay within the
    harness's bound.
- **The first run's repair,** one story (`fuzzy_repair`), on the first
  run's cast, its green light left to `fuzzy_green_light`: the person
  asks their chat T1 "Add CSV export to widget"; T1 delegates the change
  T2; its coder T3 turns widget's rows into CSV in `src/main.rs` and
  leaves the table's header as it was; the change pushes that head and
  opens its pull request, and CI fails there; the change delegates the
  repair T4, whose brief carries CI's failing output (forge.md, 8.4);
  T4 writes the CSV header on the change's branch and delivers a new
  head; CI passes there, the reviewer T5 approves at it, the change
  lands, and T1 says so.
  - **A real failure, by the fake's contract.** The forge's CI judges
    the pushed tree by its content cue (4.2): a head passes only when
    `src/main.rs` holds the CSV header line. The first head fails for
    what it holds, as a check of the program's output would; the story
    sets no status and names no head, and nothing compiles (CI that
    runs a workflow's steps is open, section 10).
  - **Every judgement a script,** one call per response: T1 delegates
    the change, says so and waits, and on its result reports. The coder
    reads, edits the rows and finishes with a change; the repair reads,
    edits the header and finishes with a change; the reviewer approves.
    The repair's script is cued by the engine's own task words, "Repair
    the failed CI on the change branch", which open its system text
    before anything of the change's: skein's fake LLM takes the
    earliest cue there, so were the coder's first, the repair would
    replay the coder's script and the story fail. The numbers are a
    fresh deployment's: the coder, the repair and the reviewer are T2's
    delegates, in that order, and no review runs at a head CI failed
    (forge.md, 8.3).
  - **The person's acts,** by `temper ctl`: sign in; adopt the
    repository first, as `fuzzy_green_light` does; start the chat and
    wait for its end; read the change T2 and the repair T4.
  - **What it checks:** CI at each of the change's heads, from the
    forge's records in order: the first head failed, the change's
    branch moved once, and the new head passed and merged at exactly
    that head. T4 is the change's one repair, its words the engine's; it
    was told why, its brief holding the failed check, its description,
    the failed step by name and its log's end, at the failing head
    (forge.md, 8.4), and it started from the change's branch, its read
    holding the first head's source. widget's CSV source whole on main.
    T2's result is the landing, naming the forge's merge commit: what
    the chat's card reads to draw the change Landed, through the
    failure and the repair, in which it stays Running
    (../client/README.md, 5.7, whose drawing the view's tests own); T1
    told that result before its report. Its fixture sizes the brief's
    CI section to carry the failure whole.
  - It is fuzzy, past the 2 s guide (1 s guide before the user's doubling of
    10-22, provisional) by the landing path and its repair, as
    `fuzzy_green_light` is, and its forge's records stay within the
    harness's bound. Its fixture, scripts and checks are its own, in its
    test file, on the tier's shared hosts.
- **An account first,** with area 5's tools (../client/tools.md, 5.5):
  a story whose agents run adds a key the fake LLM peer accepts, from
  the client, before its first task, and waits for its account to be
  usable: a key needs no sign-in to drive (jig's `domain/accounts.md`,
  3.4; ../client/tools.md, 11.3). The real loop and process tier (2.6)
  use runtime key adoption. Lower domain worlds first keep temper's real
  first-start rows at receipt one, then preseed restored fixture keys at receipt two; configuration
  names providers only, and no world hands a record in; ChatGPT's
  account comes from a sign-in alone, on the fake issuer below the live
  tier (2.6) and the real one in it (2.7).
- **LLM accounts' stories** (../protocol/credentials.md, section 7),
  one path per source beside the key every story adds first (above),
  each scripted on the fake peers, never by chance:
  - **a sign-in:** on skein's fake issuer, the engine's redirect
    listener on a port the kernel assigns, and the redirect driven by
    the test, as a browser would; the account usable, then a run whose
    credential the fake LLM peer observes as that account's, by its
    scheme, never its value;
  - **a program's source:** declared from the client, smith's fake
    `claude` scripted signed in, the source verified, and a run through
    smith's program backend calling host tools the engine decides as a
    direct run's;
  - **after each:** no key, token or record's value in the engine's
    store, a fact, a request, an answer, a trace, the client's small
    store or a printed page (../protocol/credentials.md, section 7).
- **The accounts' cases are tested below,** each in the world that owns
  it:
  - **a sign-in refused, and one past its deadline,** each settled with
    no account usable: `tests/client`, its answer scripted
    (../client/tools.md, 11.2); the issuer's side in skein's OAuth
    worlds (skein's `oauth.md`, sections 3 and 6.8) and jig's protocol
    world (jig's `protocol/llm.md`, section 8);
  - **a turn limited (exhausted with a reset and without), refused or
    unauthorized,** and the account's state following (jig's
    `domain/accounts.md`, 3.5): jig's protocol world, "failures of each
    class" and "a source limited and resumed" (jig's `protocol/llm.md`,
    section 8), and jig's accounts world (jig's `domain/testing.md`,
    section 3); the next task waiting for its account until the reset
    passes (jig's `domain/authority.md`, 8.3): jig's core world (jig's
    `domain/testing.md`, section 4); in temper, the limited notice, the
    next task waiting with `task N` saying why, then `retry` and the run
    started: `tests/client` (../client/tools.md, 11.2);
  - **a fenced attempt's notice dropped:** jig's fleet world, "Attempts
    are fenced" (jig's `domain/engine.md`, section 8; jig's
    `domain/testing.md`, section 3);
  - **a program source's usage limit and its lost login** (smith's
    `protocol/programs.md`, 5.6): smith's world over its fake `claude`
    (smith's `protocol/programs.md`, section 8), relayed as limited and
    as unauthorized in jig's channel world (jig's `protocol/channel.md`,
    section 12); `verify` answered as each host reports: `tests/client`
    (../client/tools.md, 11.2).
- **Checks between iterations** are the strategy's for the real loop
  (testing-strategy.md, 2.8 and section 6), with the lower worlds'
  outside expectations; real deadlines bound waits, and the fake peers
  stay available until their clients have settled.
- **The forge and git:** skein's forge peer (skein's `fake-forge.md`,
  section 5), which the one-process world hosts too, serves the fake forge on a
  loopback listener, its git route among its routes, which the fake
  git program reaches (4.3). Stories that clone, push or land run on the
  fake git program over that route, with neither an owned Forgejo nor
  its runner. Real git against the fake forge is an open question
  (section 10).
- **CI and webhooks.** CI follows the fake forge's content cue (skein's
  `fake-forge.md`, section 2); CI that runs a workflow's steps in the
  shell subset is an open question (section 10). The forge peer
  delivers a hooked repository's webhooks to the engine's webhooks'
  listener (../protocol/forge.md, section 5), and polling stays the
  backstop, at paces the stories' configuration sets short (`poll`,
  `poll_max`, `slow` and `hinted`, the read a delivery re-arms).
- **The person's terminal client.** The real loop also hosts temper's
  terminal client, in the loop as the service is: temper's client root
  and view under skein's text shell, over jig's native transport to the
  service's clients' listener, on a loopback port the kernel assigned.
  It is a general part of the tier (`tests/real-loop/src/terminal.rs`),
  not of one story: a story that needs a person at a client in the real
  loop has one.
  - **As `temper open` makes it.** The harness hosts the terminal's own
    composition, `temper-client-shell-terminal`'s client and terminal,
    as a program the harness spawns with its pipes for the terminal's
    input and output, as the tools' root is hosted for `temper ctl`.
    Its settings are `temper open`'s (../client/README.md, 6.2) but its
    print interval, which the tier sets short, at 20 ms, as it sets the
    engine's paces short: the shell prints at once after a command, and
    a story waits on what the pages say, never on the interval. The
    shipped interval's pacing is skein's text shell's to test (skein's
    `ui-text.md`, section 8).
  - **Signed in by the printed link,** as `temper open DIRECTORY LINK`
    is (../client/README.md, 6.1), on a DIRECTORY of its own in the
    scratch directory, never one of `temper ctl`'s
    (../client/tools.md, section 15). One link is live at a time: where
    `ctl` spent it first, the terminal's sign-in is refused, the service
    prints a fresh link (jig's `protocol/client.md`, section 8), and the
    harness opens the terminal again with it, as `ctl` signs in a second
    DIRECTORY. A story's key comes first through `ctl`, as for the
    tier's other stories (below).
  - **The typist** stands where a person sits. It types whole commands
    to the shell's input, as the person would (../client/README.md,
    section 6.5): a control's number from the last printed page, found
    by its label, with the words after it, and `q` to close. It reads
    the pages the shell prints (../client/README.md, sections 6.3 and
    6.4): what it knows of the system is what the pages say. It is
    ordinary Rust in the test process, with the process tier's typist
    as its model (`crates/temper/tests/support/terminal.rs`) and no
    terminal device; the shell's input and output are the harness's
    pipes in the loop. Its acts are a story's steps: open, type on a
    label, see a page that holds a text and lacks another, and close.
    It waits by the loop's deadlines, never by sleeping; a see reads
    only the pages printed after its last command; and every page it
    read is kept, in order, for the story's checks.
  - **Its smoke:** the person opens the terminal by the printed link,
    starts a chat from Chats with its words, and reads the committed
    reply on the chat's page; closing exits 0, and the credential stays
    in the terminal's DIRECTORY, private to its user.
  - **The peers are fakes:** skein's fake LLM, the fake forge and the
    fake git program (4.1 to 4.3), as for the tier's other stories.
  - **The terminal is the real loop's test surface.** People use the
    browser, or a later full-screen terminal; the terminal client is the
    way a real-loop story reads what a person would see
    (../client/README.md, section 6.9). The process tier's `temper open`
    tests stay smokes (2.6), outside the gate.
  - **A run's live text is one of its stories.** A person's message
    starts a chat turn, and the fake LLM plays a reply in two pieces,
    the second under a named hold, which waits before it (skein's
    `fake-llm.md`, section 2.2): the first piece is sent and the reply
    held. The typist reads a printed page that shows that piece under
    "Writing…", before the turn commits; only then does the story
    release the hold, through the fake LLM's owner in the loop, as the
    world releases its other holds. The reply completes, the turn
    commits, and the first page that shows the committed turn shows no
    provisional text. A page printed while the reply is held shows the
    same piece or a longer one, never a shorter one; the story asserts
    that each piece it read is a prefix of the committed turn's text.
    smith sends a piece by its flush deadline with no later delta to
    push it, and may join deltas: the story reads prefixes and never
    counts pieces.
- **GitHub's stories,** the real loop's; their code waits for the
  real-loop plan's end. One smoke story per feature, each on the fake's
  GitHub layer, which the forge peer hosts on loopback beside its
  Forgejo face (skein's `fake-forge.md`, section 10), over the same
  forge as the fake git program. No story calls github.com or holds a
  real credential.
  - **The configuration** points the GitHub forge's `api`, `transport`,
    `git` and `web` at the peer (../protocol/configuration.md, section
    3): plaintext on loopback, `web` the `api` origin, which serves the
    browser's pages too. The scenario and the configuration set the
    layer's waits (the delivery log's delay, mergeability's settling)
    and temper's `log_poll` short, as `poll` is above.
  - **The clients' listener's address is fixed** while a GitHub forge
    is configured (../protocol/configuration.md, section 3), since the
    App's addresses name it. The harness reserves a loopback port the
    kernel assigns before the service starts, and names it in the
    configuration.
  - **No browser in the real loop.** The story stands in for it with
    plain HTTP requests, following each `302` and the one link, to the
    service's clients' listener (skein's `fake-forge.md`, 10.1). It
    reads the manifest page's form on the clients' listener and posts
    its `manifest` field to `<web>/settings/apps/new?state=<s>`, which
    answers `200` with a small page holding one link, `Create GitHub
    App` (id `create`), to the redirect address with the code and the
    state; the story follows it as a person clicks GitHub's form.
    temper's redirect answers `302` to the install page, which the
    story asks, `<web>/apps/<slug>/installations/new?state=<s>`, with
    `account=<login>` and `repositories=all`, or
    `repository_ids=<a>,<b>`; it answers `302` to the setup address
    with `installation_id`, `setup_action=install` and the state.
  - **The harness,** the eight stories' own, in
    `tests/real-loop/src/github.rs`: skein's forge peer with its GitHub
    layer (skein's `fake-forge.md`, section 10) on the world's one fake
    forge, the GitHub forge's configuration (its identity by the forge,
    `login` and `author` null, the bot's), and the layer's record of its
    git calls read for the story, which holds no token. A story that
    starts connected has its App seeded before the service starts, by
    `testing/temper-github-app`: the App on the layer's first fixed
    test key, and its six records (`User`, `AppId`, `AppSlug`, `AppKey`,
    `Accepted`, `Origin`: ../protocol/credentials.md, 2.1) kept in jig's
    secrets' store in the scratch data directory, through jig's store
    opened as temper's start opens it; the process tier seeds the same
    (2.6). The worker's git in GitHub's stories is the hosted fake git
    program over the layer's git route (`tests/real-loop/src/git.rs`,
    the route's transport beside the typed one the other stories keep),
    so each clone and push carries the token the route checks.
  - **The stories:**
    1. the meeting point (`../draft/github.md`, 16.1): jig's secrets'
       store seeded with an App's records before the start, its key one
       of the layer's fixed test keys, and a change landed directly on
       the fake's GitHub;
    2. GitHub connected through the manifest flow from `temper ctl`,
       the browser's steps as the requests above;
    3. a repository added to a project later, from the picker;
    4. a landing through GitHub's merge queue, with an ejection
       repaired;
    5. an approval rule waiting on a person, then the App on the bypass
       list;
    6. git on GitHub's tokens (`tests/real-loop/tests/github.rs`): a
       chat's clone and push on a GitHub repository, each authenticated
       by its installation's token (`x-access-token`), which the layer's
       git route checks (skein's `fake-forge.md`, section 10). Two
       accounts on the layer, `acme` with the App installed on all its
       repositories and `umbra` with none, one repository each. The
       person adopts `acme/app`; a chat delegates a change whose coder
       reads, edits and finishes it, as `fuzzy_landing`'s scripts begin
       (no review, no landing), and the story ends on the push. The
       layer's record shows the clone and the push, each by a token
       minted for `acme`'s installation, the push a write, and the
       branch's head is the coder's commit. `umbra/tool`'s adoption is
       refused as not installed (`temper ctl adopt` exits 2, `NoAccess`
       on the client's wire until it names `NotInstalled`: forge.md,
       4.3), with no git call and no
       token minted for `umbra`. The token renewed mid-run is the
       one-process world's story (2.5): its moment is GitHub's expiry,
       in whole seconds at GitHub's hour, which a real clock can only
       wait for;
    7. a repository granted on GitHub appearing in the picker;
    8. Forgejo connected, managed and owned from the client; its code
       waits for jig's request carrying a value and a jig
       repin.
  - **After each:** no secret outside jig's secrets' store: none in a
    root, a fact, a store record but the secrets' store's, a request or
    an answer (../protocol/credentials.md, section 7); and no carried
    value left once its run has ended (../protocol/forge.md, 2.6).
  - **Each at most 1 s (0.5 s before the user's doubling of 10-22,
    provisional),** half the tier's 2 s (1 s before the user's doubling of
    10-22, provisional) story guide, or 3 s (1.5 s before the user's
    doubling of 10-22, provisional) for one that runs a whole chat (its
    agents, git and `ctl`), and the tier's sum within its 13 s (6.5 s before
    the user's doubling of 10-22, provisional) ceiling (6.1). Their cases
    are tested below: the records' and the identity's codecs in step tests,
    the routing and GitHub's routes on the clients' listener in
    `temper-service`'s own tests (../protocol/forge.md, section 10;
    ../client/forge.md, 9.1), the connector's state, picker and queue in the
    system worlds (2.1).

- **The cluster's services,** one story (../protocol/README.md, 15.6):
  an engine alone and two workers alone, three `temper-service`
  services in the one loop, each on three scratch directories of its
  own (../protocol/README.md, 15.1), the workers dialling the engine's
  workers' listener on loopback with their fixture secrets
  (../protocol/README.md, 15.3). Each worker has one slot: two tasks on
  the fake LLM peer and the fake git program, one placed on each worker,
  each run to its end. It shares the standalone stories' hosts, fakes
  and checks; the shapes' checks are step tests, and placement, a lost
  worker and its grace the engine's domain world's.
- **The reference case's stories** (5.3): its act 1, one story for each
  step's slice that temper can run, on the case's cast, scripts and
  oracle. The cluster's runner hosts them too: the case's fixture and
  fake forge under the engine and every worker, and a worker configured
  from the start but started at a step of the story, the person's act
  (../protocol/README.md, 15.2 and 15.6), and a worker stopped and
  started again at a step, the person's acts: its ending (15.4), then a
  new service on its own three directories in the same host, which links
  at its position. Every worker, a late one and a restarted one too, has
  the cluster story's link deadlines and grace, so no worker is lost in
  a story: a run under a worker that ends fails into a retry, and a lost
  worker and its grace stay the engine's domain world's.

What only the real kernel shows is this tier's: transport, persistence
and lifecycle, `temper ctl`'s lock and DIRECTORY's files
(../client/tools.md, 11.3), within 6.1's budgets. A failure found here
is rerun lower down as a scenario, where it replays (testing-strategy.md,
2.8). The process tier keeps executable startup, CLI
parsing, exit statuses, OS process launch and signal handling, the
owned Forgejo and runner, the cluster's processes, and production's
containment (section 6).

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
    its domain, view and jig's native transport (../client/README.md, section 2;
    4.4).
- **Stories:**
  - a person signs in by the link `temper serve` prints, and a chat is
    answered;
  - a person still signed in signs in again by a fresh link, with the
    same directory, and keeps their sign-in: its number, token and
    expiry, and the deployment's sign-ins unchanged
    (../client/README.md, section 7);
  - a repository is adopted through temper's client, and a goal's change
    is prepared, committed and pushed on the fake forge through the fake
    git program;
  - a merge in progress is resolved and pushed with both parents;
  - on GitHub's layer (skein's `fake-forge.md`, section 10), its App
    seeded as the real loop's are (2.4): a goal's change cloned by its
    installation's token, the token renewed `renew_before` ahead of
    GitHub's hour on the simulator's clock while the run holds its
    workspace, git's generation changing under it, and the push
    authenticated by the renewed token, as the layer's record of its git
    calls shows (../protocol/credentials.md, section 4);
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
- **Budget:** a story aims at 2 s (1 s before the user's doubling of 10-22,
  provisional) (6.1); sweeps of drawn kills are fuzzy tests.

### 2.6 The process tier

`crates/temper/tests/process.rs`, after jig's reference binary's (jig's
`protocol/service.md`, section 8): temper as it ships, as processes.

- **What runs:**
  - the real `temper` binary: `temper forge prepare`, `temper forge run`
    for Forgejo and for its runner, then `temper serve`, on a scratch
    directory, with a configuration the test writes from the owned
    example (../protocol/setup.md, 8.1);
  - skein's fake LLM peer, a process of its own on loopback, over TLS
    with a test root the agent's configuration trusts, scripted for the
    stories, accepting a key the test makes; the owned example's LLM
    providers' endpoints are written at its address, and Claude Code's
    path at smith's fake `claude` (4.1);
  - an owned Forgejo and its runner (below);
  - real git, the machine's, at the absolute path configuration gives it
    (../protocol/configuration.md, section 4; ../protocol/git.md,
    section 3);
  - the person: skein's fake person in front of temper's client, over
    jig's native transport (../client/README.md, section 2); and, in a
    test of its own, `temper open` itself with stdin and stdout piped
    (../client/README.md, sections 6 and 10).
- **Its Forgejo is owned,** provisioned by the setup's commands
  (../protocol/setup.md, 13.4): the configuration's one forge is owned
  (../protocol/setup.md, section 8), its binaries those `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name, which the commands check against temper's
  pins (../protocol/setup.md, 6.1); its address, the clients' listener and
  the webhooks' listener are loopback addresses of the test's own. The
  test runs the commands itself, in the order `temper box` would
  (../protocol/setup.md, 7.1), refuses to run without either variable, naming
  it, and stops every process it started after. At `temper serve`'s start
  the setup host's `Reconcile` plan makes temper's user, its tokens, its id, the org's team and its hook; the test, as the
  person, makes the repository in the setup's org through Forgejo's API,
  with Actions and a workflow whose job runs the repository's check on the
  runner. Never an existing Forgejo.
- **CI on, webhooks on.** The repository is adopted with "It runs CI"
  checked, and the project's landing rule asks for CI and for no gate or
  approval (`"ci": true`, `"gates": []`, `"approvals": []`;
  ../protocol/configuration.md, section 3): the change lands once its job,
  run by the owned runner, has passed on its head. The webhooks' listener
  is configured, and the org's hook reaches it.
- **The story,** one test:
  1. the person signs in by the link the binary prints, and adds the
     test's key as the deployment's on the accounts page
     (../client/README.md, 3.6), which shows its account usable (2.4);
  2. a chat is answered;
  3. a repository is adopted through temper's client, as above;
  4. a goal is given whose change, its CI passed, lands on the owned
     Forgejo, pushed by real git;
  5. the binary crashes: its whole tree is killed at once, as a
     supervisor ends a crashed service's (skein's `shell.md`, 13), and
     once nothing of it runs, it is started again on the same directory;
  6. the person finds the work, and the change's branch on the forge is
     at the run's commit.

  The person reads the chat that gives the goal, finds the goal on the
  board and its plan on its page, and finds the same after the restart.
  The forge's state is the story's proof of the change, read through
  Forgejo's API.
- **The first owner's deployment accounts,** a separate smoke on the
  fake-issuer sign-in configuration: the configured first owner signs in
  by the printed link, sees "Deployment accounts", adds an API key with
  "Use deployment accounts", and sees its deployment-held account there.
  It runs in under five seconds, without an LLM call or a real account.
- **Its endings.** The crash is the tier's one deliberate kill. It
  proves what a crash of every process keeps on disk, the page cache
  surviving it, not what a power loss does, which the one-process world
  draws (2.5). Every other end of `temper serve`, after the restart and
  in every other test of the tier, is a deployment's healthy stop
  (skein's `shell.md`, 13): the test sends `Terminate`, the engine
  drains (`root.md`, 9.2), the service closes what it started, and the
  binary exits 0 with none of its tree running. A process left at that
  exit is temper's failure, its shutdown's; one left by the crash's kill
  is skein's test kit's (skein's `examples.md`, 6). Neither is relaxed.
- **The terminal,** a second test, on a scratch directory of its own:
  `temper open DIRECTORY LINK` with stdin and stdout piped signs in by
  the link `temper serve` printed, prints the inbox, adopts a repository
  as above by commands (the form's fields written with `n words`, then
  Adopt pressed with `n`), and exits 0 on `q` after a drained close. Its
  referee reads the printed pages (the inbox and the adoption's
  outcome) and the exit status.
- **The agents' tools,** `temper ctl` against `temper serve`, on a
  scratch directory of their own: ../client/tools.md, 11.4. A call cut
  by `SIGINT` or `SIGTERM` sends nothing more, prints nothing more,
  leaves DIRECTORY's kept requests as they are, and exits 130 or 143
  (../client/tools.md, 8.2).
- **A sign-in kept,** on a scratch directory of its own, over five
  starts of `temper serve`, each printing a fresh link: `temper ctl
  DIRECTORY sign-in -` with each of three starts' links keeps the
  first's token, byte for byte, and each passes, though the
  configuration's pool holds two sign-ins (`people.sign_ins`); then the
  link's route, posted as a browser posts it, sets a cookie, and the
  next start's link posted with that cookie is answered signed in with
  no credential and no `Set-Cookie`, DIRECTORY's sign-in still live
  (../client/README.md, section 7; jig's `domain/people.md`, section 3).
  The cookie is proved here, below the browser, whose harness can't yet
  open a later printed link in the same context.
- **The server's guards,** a third test, against `temper serve`'s
  clients' listener: jig's one attack story that needs no browser, a
  rebound Host (jig's `protocol/browser.md`, section 15): a request
  naming a host off the list is answered `421`, and nothing it asked is
  made. The cross-site request and the framed page need a browser: they
  are the browser tier's (../client/browser.md, section 4).
- **The hinted story,** a fourth test (../protocol/setup.md, 13.4): the
  story with the keep's `poll`, `poll_max` and `slow` and the change's
  `stall` raised past the story's budget (../protocol/configuration.md,
  section 7), and its CI's job outlasting `settle`, so that it lands
  within its budget only if its job's end is delivered
  (../protocol/setup.md, section 10, "Checked").
- **`temper box`,** a fifth test, on a scratch directory of its own
  (../protocol/setup.md, 13.4): `temper box` on the owned example prints the
  sign-in link on its stdout; Forgejo killed is restarted by the box, with
  the engine still serving; a stop leaves no process the box started.
- **A Forgejo set up from the client,** a test of its own
  (../protocol/setup.md, 13.4, "A first run set up for real"): `temper
  box` on three empty scratch directories, forge 3's binaries placed
  where its fetch looks, then `temper ctl DIRECTORY forge-setup 3` and
  `wait forges --until settled`, the box's group run on the real Forgejo
  and runner; forge 3's row goes through "Setting up" and "Connecting"
  to "Connected as temper", with no restart, its first repository
  `work/hello` made and adopted into project 1. The owner's first task
  there, on the fake LLM, is cloned by its worker, its pull's run of the
  starter workflow passes on the owned runner, and it lands, all read
  through temper's own reads, with no credential. The handover file is gone,
  and forge 3's directory holds no `api_token`, `git_token`, `user` or
  `webhook_secret` file. A second box on the same directories starts the
  group by its record; preparation makes a new handover, and serve takes
  it over, connects forge 3 from its records and finds the task landed. Plaintext throughout:
  no TLS and no call off the machine.
- **A first run,** a test of its own (../protocol/setup.md, 13.4):
  `temper box` on three empty scratch directories writes the
  configuration (../protocol/setup.md, 8.2), says so on stderr, and
  prints a sign-in link that signs the owner in; the demo's suggested
  change, sent to the project's chat, is delegated, pushed by `temper
  fake git`, checked and landed into the fake's `main` on the demo
  model, and the chat says so, its card for the change reading it
  landed (../protocol/setup.md, 8.4 and 13.4; ../client/README.md, 5.7).
  The test reads the one hook on `playground/widget` at the box's webhooks'
  listener with the handover's token and prints each box run's time,
  asserting no time. A delivery's acceptance is shown by
  `tests/real-loop/tests/forge.rs`'s
  `a_forge_delivery_is_accepted_by_the_engines_listener`; its secret's
  matching is checked by the step tests. The demo's scripts are data,
  checked in `temper-init`'s step tests; this test alone plays them end
  to end.
- **`temper program-login`,** a sixth test, on a scratch directory of
  its own (../protocol/credentials.md, section 3): on the owned example
  with Claude Code's path at smith's fake `claude`, `temper
  program-login live`, given the test's three directories, makes the
  programs directory and `live` in it, each `0o700`, and replaces
  itself with the fake's `auth
  login`, which observes `CLAUDE_CONFIG_DIR` at that directory and none
  of the program's credential variables, and writes its login there;
  the command's exit status is the fake's. A name out of bounds, and a
  programs directory readable by another user, are each refused, exit
  1, with nothing made. A deployment started after it, its program
  source declared as `live`, finds the source signed in.
- **The throwaway Forgejo** is the one in skein's `fake-forge.md`,
  section 6, never an existing Forgejo: temper's `tools/git-transcripts`
  records real git against it (../protocol/git.md, section 9). temper's
  users give it the binaries `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name, and refuse to run without
  `TEMPER_FORGEJO` (and, for a story with CI, `TEMPER_FORGEJO_RUNNER`),
  naming it.
- **Its referee:** the person's script finishes within its deadline; the
  binary does not exit before the test stops or crashes it, or its
  stderr is shown; a stopped binary exits 0; the forge's state, read
  through Forgejo's API, shows the change; nothing the person saw before
  the crash is lost after it; no process the binary or the test started
  outlives it: none runs at a stopped binary's exit, and none of the
  crashed tree when the binary starts again.
- **Credentials:** none borrowed. The fake LLM peer accepts a key the test
  makes. The person adds it from the client once signed in (2.4),
  as the deployment's, and the agents' tools' test with `temper ctl
  add-key` (../client/tools.md, 11.4); it is then kept in jig's secrets'
  store, and no file the configuration names holds an LLM account
  (../protocol/credentials.md, section 3). The forge's API token, its user
  and the git identity's token are records the setup host's plans write
  in jig's secrets' store, and the admin's token the preparation's
  handover, `admin_token`, which the setup host takes once
  (../protocol/setup.md, sections 3 and 9).
- **ChatGPT's sign-in,** three tests of their own
  (`crates/temper/tests/support/sign_in.rs`;
  ../protocol/credentials.md, section 3), on skein's fake issuer and the
  fake LLM peer, with the sign-in's terms as `temper init` writes them
  (../protocol/setup.md, 8.1), only its addresses swapped for the
  fakes', plaintext, and the redirect's port the kernel's. The issuer's
  access token is as long as a real plan's, 4,096 bytes shaped as a
  JWT, so the bounds it crosses (the accounts' table, the grant, the
  model's request head) are a real one's; the fake LLM peer's request
  head takes it. The peer's replies carry a Content-Type where a plan's
  route sends none: skein-llm's own world holds that its client takes a
  reply without one (skein's `tests/llm`, `stream_head.rs`), and no
  layer above skein-llm sees the header:
  - *signed in, a chat that lasts:* `temper ctl connect 1` prints the
    sign-in's address and the accounts page, a native person's view,
    shows the sign-in under way; the test drives the redirect as a
    browser would; the page then shows the person's account, "ChatGPT ·
    DISPLAY · OpenAI", a plan, its source ready. A chat on gpt-6-luna
    is answered on it by text alone, with no `finish` call, as a plan's
    model answers a chat: the reply commits once and the chat parks
    (`agent.md`, 4.6). The person's next words wake it, and the
    peer's second answer carries a reasoning item at the model's
    allowance, so its turn passes jig's small profile's 64 KiB and
    stays within the bounds the document derives from smith's largest
    turn (../protocol/configuration.md, section 7): it commits once and
    the chat parks again. The person reads the conversation page by
    page, the long turn decoded whole. Close cancels the chat with the
    chat template's grace (jig's `shapes::close_chat`): one closing
    turn, and the task ends cancelled by its person, with no result.
    The peer has seen three requests, one for each of the person's
    words and the closing turn's, each answered, and the chat holds two
    replies, each once: no retry and no second reply. The peer
    observes a bearer, the grant's scheme, never its value;
  - *past its allowance:* the peer's reasoning item one byte past the
    model's allowance holds the chat at once, after one request, with
    the reason the tools' JSON names, the bound and its configured
    value (`worker.md`, 4.3); the person's release allows one
    more request, whose answer holds the chat again;
  - *stopped while signing:* the binary, told to stop with a sign-in
    under way, ends at once with exit 0, well before the sign-in's
    deadline, its client answered (jig's `domain/accounts.md`, section
    5: the accounts' close cancels each sign-in under way).
- **`temper program-login`,** in two stories of their own, each within ten
  seconds (five seconds before the user's doubling of 10-22, provisional)
  and with no engine, service or deployment secret: it execs smith's fake
  `claude`, which signs the source in; and the fake's refusal exits with the
  fake's status, while an exec that fails exits 1 with one final `temper
  program-login:` line, which only smith's secret-free `smith: ` diagnostics
  may precede (../protocol/credentials.md, section 3). The fake is smith's
  `smith_fake_claude::main()` at temper's pin, built as
  `tools/fake-claude`'s `fake-claude` binary and named by the absolute path
  in `TEMPER_TEST_CLAUDE`. The argument checks, the directories checked at
  each name and made private, and the command's arguments and environment
  before the exec are the shell's lower tests.
- **The cluster,** its own tests (../protocol/README.md, 15.6), on
  temper's fakes as processes, never on Forgejo: `lab/cluster/`, the
  lab's description of the cluster (../protocol/README.md, 15.5), read
  by `crates/temper/tests/support/cluster.rs`, which starts the fake
  forge, the fake LLM, the engine alone and two workers alone as
  processes, each in a cgroup of its own, awaits their ready lines and
  stops them in reverse. Its stories: a task on each worker; a worker
  lost, killed mid-run; a worker hung, stopped past the grace and
  continued; a worker started later, and one restarted; and a mixed
  document refused, exit 1. Each story's referee is the tier's: what
  the engine records, read through the client (the forge's state, from
  the fake forge's API, once it joins: ../protocol/README.md, 15.5), and
  no process of the cluster outliving it. Without delegated cgroups the cluster's tests
  fail at their start, saying why (section 10).
- **The reference case on the cluster** (5.3), stories of the cluster's
  own, each a step of the case run as the real loop runs it, on the
  case's library unchanged, by `crates/temper/tests/support/case.rs`:
  - **What runs:** `lab/cluster/`'s members (../protocol/README.md,
    15.5): the fake forge serving kit, the case's repository, at its
    `id` with its `tree` (4.2), and one inert repository (below); the
    fake LLM playing the step's scripts (`case/scripts.rs`), written
    into its document once the
    forge's address is known, its queries recorded (4.1); the engine
    alone; and the case's workers alone, named as the case names them,
    ws, s1 and s2 once a step adds it, one slot each. s2 is the lab's
    second worker's description copied under its own name, with its own
    directories and link secret, prepared with the others and started
    by the step's act. A step's cast is the fewest workers its Musts
    need: ws, s1 and s2 for 4.1, 4.3 and 4.6, whose Musts are where
    runs go and how they move; ws alone for 4.4 and 4.5, whose Musts
    are the deployment's accounts, their order and their states, which
    no worker holds: a worker's own sources (s2's switch, its sign-in)
    are undesigned (5.3), so a second worker would show nothing more,
    and ws's one slot keeps the real loop's order, the step's agents
    taking it in turn as on the standalone service's one worker. The
    forge runs in every step, kit unread in 4.4 and 4.5, so that the
    case's fixture and its accounts' numbers are the same in each. The
    case's
    fixture (`case.rs`) is applied once to the cluster's documents, as
    the real loop's cluster applies it to its own: to the engine's
    document and one worker's together, then split into the engine's
    and each worker's; the lab's base is brought first to the real
    loop's where the two differ (the chat charter's wait, its ending
    and its text), and its charters are the case's alone, so that a
    charter the lab adds never meets one the fixture adds, in one
    place that names them.
  - **The forge, reached as a Forgejo,** as serve reaches a fake one:
    the case's run makes the engine's forge 1 the fake's records-only
    setup entry (../protocol/setup.md, 6.5 and 8.4), at the forge
    member's address, its handover the token the fake wrote at its
    document's `admin_token`; serve takes the handover over at its
    start, reconciles with the webhooks' address it bound, and adopts
    the setup's repositories. That adoption walks the forge's ids from
    1, one per repository the setup lists, so the forge seeds kit
    first, at the id the scripts name, its first commit the draft, and
    then one inert repository at id 1 that no act names, which goes
    when adoption takes the forge's own ids. The setup entry and the
    inert repository are the case's departures from the description's
    forge, in one function of the harness.
  - **The person's acts** are the step's (`case/acts.rs`), each a real
    `temper ctl` call through `support/ctl.rs`, interpreted by one
    exhaustive match over the acts' steps, so that a step added to the
    library is a build error here the day it is added; a step no
    process story runs yet fails its story, naming it. The fake LLM's
    key is added as the deployment's, and its account waited usable,
    after the first sign-in and outside the acts' positions, as the
    real loop provisions it. A second key an act adds (4.4's) is the
    fake LLM's second accepted credential, the document's first
    `added_keys` (../protocol/setup.md, 8.4), written by the fake at
    the path the act names; a script's failing line (4.5's) is the
    document's `fail`, failing the credential the script names. And
    after the chat act, outside them too,
    the director's chat is waited running once, with its document,
    since it can end before an act reads it live. The cluster's steps
    are the harness's: a worker added is started (`Members::up`) and
    waited linked; a worker stopped is ended (`Members::stop_one`) and
    waited ended, its directories kept; a worker started again is
    restarted (`Members::restart`) on its own three directories and
    waited linked once more; a wait the acts watch (`Watching`) goes on
    once its background `temper ctl` call is running: no process
    shows when that call has its task's document, and none needs to,
    since `wait --until ended` without `--since` holds on whichever
    document it receives first, an ended task's being retained, so
    what it prints doesn't hang on when it subscribed, as the opener's
    two watches already run without the act; the real loop's world
    holds the act until its host has the delivery, to keep its hosts
    in one order. Where a script holds a reply until every worker holds a run
    (4.3's check, armed on `cluster::PLACED`), the harness releases the
    hold through the fake LLM's control (4.1) at each of its passes
    once every started worker has held a run, read from the tasks'
    documents and their channels' hosts, as the real loop's world
    releases it at each of its iterations; a release before the reply
    is held changes nothing, so it is sent again until the acts end,
    and, since no process shows when the reply is held and the
    referee's passes follow its processes' events, the referee passes
    at least every 20 ms while the release is due: the release is
    idempotent, and the cadence decides only how soon the held reply
    goes on, never whether a wait is met.
  - **Its referee:** the step's oracle, unchanged (5.3), over one
    `Observed` filled from the processes: the documents, `temper
    ctl`'s JSON by the acts that read them; the commands, the acts'
    verbs; what each agent was told, from the fake LLM's records; kit,
    from the fake forge's observations, the checks at each merged
    head, and main's tree at the last merge, read by the fake git
    route's `objects` before the forge stops. And the tier's: each act
    exits 0; where each task ran, from the documents the acts and that
    read show, a live task's channel, mapped to its worker by one read of
    `accounts`' `deployment_facts`' `program_hosts` after the acts,
    each worker having linked once so that the channels are the same
    links, or, where a step stops a worker and starts it again, one
    read before its stop and one after its next link, each mapping its
    own life's channels, and, with ws alone, none: every task can
    only have run there; no file and no document the harness gives a
    worker (its `temper.json`, its `agent.json`, its link secret) holds
    the fake forge's admin token or a token the forge minted, read from
    those files after the stop, so that the forge's credentials stay
    the engine's: a git call's credential reaches a worker's git only
    per call, which no file shows; and, after the stop, from the workers' roots, which only the
    harness sees: the worker whose workspace holds the change's tree
    made the change. A task neither shows goes unasserted and its
    step's row names it, since a task's document drops its channel when
    it ends and its activity keeps none (jig's `domain/tasks.md`), and
    the real loop's harness sees it on the worker's wire, in its own
    process; every process stopped exits 0 and none outlives the
    story.
  - **Its stories:**
    `the_opener_and_the_two_hour_check_run_on_three_workers_as_processes_without_pools_lending_or_host_moves`,
    4.1 then 4.3, on the real loop's acts for the two, as the real
    loop runs them, so that the opener runs once: ws and s1 linked
    before the acts, s2 added by its act once the benchmark is due;
    the directors' chats on ws, the design's change made on s1, B on
    s1 and the benchmark on s2, by the acts' reads and the workers'
    roots.
    `a_restarted_workstation_keeps_the_directors_question_and_answers_it_once_as_processes_without_a_hang_past_the_grace`,
    4.6: ws, s1 and s2 linked before the acts; the first attempt on
    ws and the next on s1 or s2, each by its own life's mapping; ws
    stopped and started again, its two lives each linked once, the
    second running nothing.
    `the_pace_question_reaches_the_person_and_its_answer_is_kept_as_processes_on_one_worker_without_plan_pacing`,
    4.4, and
    `a_refused_key_holds_its_work_until_it_is_replaced_as_processes_on_one_worker_without_a_sign_in`
    and `a_limited_account_holds_its_work_until_its_reset_as_processes_on_one_worker`,
    4.5's two: ws alone linked before the acts, every task's run on
    its one slot in turn. What each leaves out is its step's row in
    5.3.
  - **The scripts' rules.** On processes a message reaches an agent
    at its next `wait`, as in the real loop, but in an order the real
    loop's one ring never shows: kit's landing news can wake A, a
    goal, between its question and the green light. A script that
    such a wake would put out of step carries rule tables
    (`case/scripts.rs`, skein's `fake-llm.md`, 2.4), the same in the
    real loop, whose turns the tables pick as the count did: 4.1's A
    and director, each wake matched by the case's own words where it
    has them, a delegation's or a message's acknowledgement by smith's
    words, and any other delivery (the renderer's `": "` after its
    sender) answered by a `wait`. The other scripts count, since no
    wake but the expected one reaches them; one gains a table where a
    run shows it. A table can't tell a turn whose fresh input is the
    same as another's: 4.6's director's first activation and its retry
    both begin with smith's opening, so it counts, which the retry
    resumes in step.
  - **Past the deadline,** the story's referee names its stage, the act
    in flight and the next on stderr before the cleanup, so that a
    stall is read from its output.
- **GitHub's git,** a test of its own (../protocol/credentials.md,
  section 4), on temper's fakes as processes, never on github.com:
  `temper fake forge` serving the fake's GitHub layer (4.2), the fake
  LLM, an engine alone and one worker alone (../protocol/README.md,
  15.1), by `crates/temper/tests/support/github.rs` on the cluster's
  members: each started on its ready lines in a cgroup of its own, after
  the one before, and stopped in reverse. Not from `lab/cluster/`'s
  description: before the engine starts, the test seeds the App's
  records into the engine's data directory with
  `testing/temper-github-app`, as the real loop's GitHub stories do
  (2.4), a step the lab's format has no place for (../protocol/README.md,
  15.5). The engine's clients' listener is on a loopback port the test
  reserved (2.4), and the worker's git is the `temper` binary under the
  name `git`, `temper fake git` (4.3), over the layer's git route.
  - **The story:** the person signs in, adds the test's key (2.4) and
    adopts `acme/app`, the layer's one installed account's repository;
    a chat delegates a change whose coder reads, edits and finishes it,
    scripted as the real loop's story 6 is (2.4), on the worker; the
    story waits, on a watch, until the change's production has
    delivered, then stops the processes.
  - **Its referee** is the tier's: the coder's task ended on the worker;
    the forge's observations, which hold the layer's record of its git
    calls and never a token, show the clone and the push, each by a
    token minted for `acme`'s installation, the push a write, and the
    branch's head the coder's commit; no process outlives the story.
  - **It aims at 2 s (1 s before the user's doubling of 10-22,
    provisional),** as a world's story does (6.1).
- **Outside the suites:** nextest profile `process`, run by
  `scripts/proof-one-process.sh`, which builds the binary and the fake
  `claude` first and sets `TEMPER_TEST_CLAUDE`. It aims at 6 minutes (3
  minutes before the user's doubling of 10-22, provisional); if CI takes it
  past, the story with CI becomes a test of its own (../protocol/setup.md,
  13.4).

### 2.7 The live tier

Nothing is handed in: no input takes a record, no tool reads another
program's login, and a ChatGPT account comes only from a sign-in
(../protocol/credentials.md, section 3), on the owned example's
registration (../protocol/setup.md, 8.1).

`crates/temper/tests/live.rs`, after jig's (jig's `protocol/service.md`,
section 8; testing-strategy.md, 2.9).

- **What runs:** the process tier's binary, owned Forgejo, real git and
  person, with a real model: the live configuration, a configuration
  directory (../protocol/configuration.md, section 12), is the owned
  example's (../protocol/setup.md, 8.1), its LLM providers'
  endpoints the providers' own and Claude Code's path the machine's
  `claude`. Each run is a fresh deployment, with no LLM account until
  the script adds one, in place of the process tier's key (2.6, step
  1), with `temper ctl` signed in by a link the binary
  prints (../client/tools.md, section 10); the script's argument says which
  account, Claude Code's login by default, since it needs nobody.
- **Claude Code, unattended** (the default). The live configuration's
  programs directory is `~/.local/state/temper/live/programs`. The
  developer runs `temper program-login --config LIVE live`, `LIVE` the
  live configuration's directory, once on each
  machine that runs the tier, and signs in as Claude Code asks
  (../protocol/credentials.md, section 3). Each run's deployment declares
  the program source `live` (`temper ctl DIRECTORY declare 2 claude_code
  live --key live-program`, ../client/tools.md, 5.5, area 5), then waits with
  `wait accounts --until usable`: the name, not the account's number,
  ties the source to the login, which the worker finds signed in.
  Claude Code renews its own login; temper never reads it, and the run
  needs nobody. Its runs are on smith's program backend (../protocol/llm.md,
  section 2).
- **ChatGPT, signed in as the run starts.** `scripts/proof-live.sh`
  runs `temper ctl DIRECTORY connect 1 --key live-sign-in`, prints its
  address, and runs `wait accounts --until settled`, within the
  sign-in's deadline, while the developer approves it in a browser on
  the same machine, or through the port forwarded to it
  (../protocol/credentials.md, section 3). The account is the person's
  that `temper ctl` signed in, a plan's (jig's `domain/accounts.md`,
  3.2), in the run's deployment, and goes with it; its runs are on
  smith's direct backend.
- **A key, when neither is at hand:** `add-key PROVIDER FILE
  --deployment --key live-key`, from a private file the script's
  argument names, then `wait accounts --until usable`.
- **Nothing borrowed:** no other program's login is read (no
  `~/.codex`), no record or key is in a file the configuration names,
  and the script copies no credential; an account added for a run is in
  its deployment's store, removed with its scratch directory.
- **No GitHub:** its forge is the owned Forgejo alone. The tier never
  calls github.com; live calls to GitHub are only skein's opt-in
  conformance (skein's `fake-forge.md`, section 11), run by hand
  (section 2).
- **The stories:** the process tier's chat, answered by the real model; and
  one small change, made by the model in a repository of the owned Forgejo,
  adopted as in 2.6, and pushed by real git. The story ends once the
  change's branch on the forge is at the run's commit; it does not wait for
  the change to land. A story also ends at its task's first terminal
  failure, which fails the test at once, naming it: it never waits out the
  run's 30 minutes (15 minutes before the user's doubling of 10-22,
  provisional) after its task has ended. An account limited, refused or
  needing sign-in fails the test the same way, naming the account's state
  and its reset, since its task would wait for it (jig's
  `domain/authority.md`, 8.3).
- **Its referee:** the process tier's, without the restart; and the
  completions, counted from the engine's transcripts. Each completion is
  a turn of main's conversation on either backend: on the program
  backend, each assistant message the program writes (smith's
  `protocol/programs.md`, 5.5), so the count holds for Claude Code's
  runs as for direct ones.
- **Budget:** at most 25 completions and 30 minutes (15 minutes before the
  user's doubling of 10-22, provisional) in all. The live configuration's
  charters bound each run's turns so the two stories together make at most
  25 completions, which smith's agent holds on either backend (on the
  program backend with `--max-turns` behind its own count; smith's
  `protocol/programs.md`, 5.7), the script stops the run at 30 minutes (15
  minutes before the user's doubling of 10-22, provisional), and the test
  fails when the transcripts count more than 25. Each run's summary, a
  failed run's included, records its completions and its wall time, which a
  plan totals (skein's `development.md`, 5.5).
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
engine's webhooks' listener (../protocol/forge.md, section 5; skein's
`fake-forge.md`, section 4); and reconcile meets it as the engine does,
through the setup's routes (../protocol/setup.md, 13.2).

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
the worker's agents are smith's (../protocol/llm.md, section 2). The fake is
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
- **As a process,** `temper fake llm --document FILE`
  (../protocol/setup.md, 8.4) records what each agent was told when
  given `--observations FILE`, so that the reference case's oracle
  reads a process's queries as it reads a hosted peer's (5.3): FILE is
  written as the fake forge's is, created new at 0o600, a checkpoint
  first and at each pass that observed or lost, and one more at its
  stop, under the same write-failure rule, and each query the peer
  observed is one JSON line, in order: `{"observation": "query",
  "credential", "model", "system", "messages"}`, `credential` the
  index of the account's credential that the call presented, or null
  for none it holds, never a value; `model` and `system` the call's; `messages` its history, each
  `{"role", "parts"}`, `role` `user` or `assistant`, each part one of
  `{"text"}`, `{"opaque"}`, `{"tool_call": {"id", "name", "arguments"}}`
  and `{"tool_output": {"id", "output", "is_error"}}`. Every byte string
  is standard base64, since a provider's text is bytes, so the record
  is lossless. Its decoder is `temper-fake`'s, giving skein's messages
  back, so a reader holds no copy of the format. skein's peer keeps its
  records for its life, with no drain, so the process holds them under
  the caps the real loop's hosted peer runs the whole case under,
  smith's, 256 records and 4 MiB: at the count's cap the stream ends
  with `{"observation": "records_full", "capacity"}` and the fake
  exits non-zero before a record is lost, as the GitHub layer's git records do (4.2), and a
  record past the bytes' cap stops it non-zero. Without the flag the
  peer records nothing.
- **A script's rules and holds, as a process** (../protocol/setup.md,
  8.4): the LLM document gives a script its turn rules, its `inputs`
  (skein's `fake-llm.md`, 2.4), so that it plays the turn its fresh
  input selects rather than the one its count reaches; and a turn its
  pieces, held by name (skein's `fake-llm.md`, 2.2), which the
  process's `control` releases, a loopback listener of the test's
  alone. A hold without a `control` is refused at the start.
- **Its issuer,** skein's fake issuer (`skein-fake-oauth`; skein's
  `oauth.md`, section 5), for sign-ins with an LLM provider; the test
  drives the redirect to the engine's loopback listener as a browser
  would (2.4).
- **Claude Code** is smith's fake `claude` (smith's
  `protocol/programs.md`, section 8), at the path the agent's
  configuration gives, scripted per story, with usage limits and a
  login lost, which smith's and jig's worlds test (2.4). The real loop
  runs it signed in, under smith's agent hosts, and the process tier
  under `temper agent` and `temper program-login` (2.6).
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

- **What the engine's worlds need of it** (`root.md`, section 12;
  `forge.md`, section 18): CI that passes, fails or never
  reports; merges that conflict; webhooks that come late or not at all;
  requests that fail or hit a rate limit; people who comment and edit.
  The domain's faults give them, drawn from the world's seed.
- **Its fixtures.** The scenario gives the protocol layer each
  repository's id and its table of API tokens and the users they stand
  for (skein's `fake-forge.md`, section 4), and the repositories their
  files and checks (5.1).
- **Its costs.** The protocol layer counts the requests it serves, by
  route, which the engine's cost checks hold to ceilings
  (../protocol/forge.md, section 10).
- **Its deliveries** go to the engine's webhooks' listener: the world
  writes each on a connection to it, as Forgejo sends one
  (../protocol/forge.md, section 5), and the forge peer does so in the real
  loop (2.4). The world hands the peer that listener's address once it
  listens (skein's `fake-forge.md`, section 5). A hook due before then has
  no address, and is dropped and observed, as the hooks of a story's setup
  are, since the setup runs before any host. The adopted repository's
  story is the hooked one. Its setup's push is lost. The agent's push, which
  ends the story, is written: the story ends only once the peer holds an
  outcome for each of the story's own two hooks, one per push it makes, and
  never on a count of the hooks the forge has released so far, since a hook
  is scheduled at its push and released later. Its adoption, with the first
  lost, shows that a lost delivery "costs latency, never liveness"
  (`forge.md`, 5.2).
- **Its setup routes** serve reconcile's tests (../protocol/setup.md, 13.2),
  and on GitHub's dialect its browser's pages serve the real loop's
  connect (2.4). temper's own routes on the clients' listener are
  tested in `temper-service`'s tests, against no fake (2.1).
- **Its referee** drains the forge's observations (skein's
  `fake-forge.md`, 2.4; here, 5.2).
- **Its git** is reached through the fake forge's own calls, never
  git's protocol over HTTP: through its typed transport by the fake git
  program in the one-process world (4.3), as by the fake checkout in the
  domain worlds; and over its git route (skein's `fake-forge.md`,
  section 4) by the fake git program in the real loop (2.4), as in the
  QA loop's processes and the demo's (`../draft/qa-loop.md`, section
  11; ../protocol/setup.md, 8.4). Real git meets an owned Forgejo in the
  process and live tiers, and skein's throwaway Forgejo in temper's
  transcripts (2.6); a real-git bridge is an open question (section
  10).
- **CI follows content,** by the fake forge's cue (skein's
  `fake-forge.md`, 2.3; here, 2.4). CI that runs a repository's checks
  in the shell subset (4.3) on the pushed tree, so that a change that is
  wrong fails on the forge as it fails in the run, is an open question
  (section 10).
- **GitHub's API** is served by the fake's GitHub layer, over the same
  domain, with the browser's pages as plain HTTP at its `web` origin
  (skein's `fake-forge.md`, section 10); its stories are 2.4's.
- **GitHub's, as a process:** `temper fake forge --document FILE`
  serves the GitHub layer when the document has a `github` member: the
  App (`id`, `slug`, the bot's user id, and `key`, the index of one of
  the layer's fixed test keys, as `testing/temper-github-app` names it),
  the accounts (`login`, `id`, `kind`), each account's installation
  (`all` its repositories, or the names of some) or none, and
  `token_life_seconds`, GitHub's hour, 3600, unless the scenario says.
  The repositories are the document's own, as for Forgejo, each with its
  owner an account. Its `--observations` file holds, beside the forge's
  observations, the layer's record of its git calls (skein's
  `fake-forge.md`, 10.1): the installation and
  mint of the token that authenticated it, whether it wrote, and its
  status, never a token. Each new call is written as it appears; at the
  layer's `limits.git_records` bound, the stream ends with a git-call
  record-full observation and the fake exits non-zero before an eviction
  can go unnoticed. Its ready line names its address, which the
  engine's document gives as the forge's `api`, `git` and `web` (2.4).
- **A repository's id and tree, as a process:** in `temper fake forge
  --document FILE`'s repositories, each may name `id`, its id on the
  forge, a positive integer, and `tree`, the files of its first
  commit, `[{"path", "content"}]`, each path relative and distinct and
  each content text, within the domain's content limits, in place of
  the seed's three files. Either every repository names an `id`, each
  distinct, or none does and ids follow the document's order. So the
  reference case's kit is the forge's first repository, at the id the
  case's scripts name, its first commit the case's draft (5.3). Its
  git route's `objects` call gives a commit's tree, which the case's
  referee reads for main once the last merge is observed, before the
  forge stops.

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

**The fake git program** is git as the machine runs it below the git
component: a program whose protocol is its command line. Its step core
asks for file operations as io's kernel does, so the same core runs
three ways: hosted by skein's world harness (its `HostedProgram`,
spawned by name) on skein's fake machine, which runs no program of an
application's; hosted on the real file system by the real loop; and as
a process of its own, `temper fake git`, the demo's and the QA loop's
git. It stays temper's, as it fakes temper's git component's program,
and is built on skein's fake forge, whose domain is its only remote
state. It reaches that forge in one of two ways (skein's
`fake-forge.md`, section 3): through the typed transport of the fake
forge's domain, sharing the forge with the forge peer (skein's
`fake-forge.md`, section 5) where a world hosts both in one ring
(2.5); or over the forge's git route on the forge's listener (skein's
`fake-forge.md`, section 4), from the real loop (2.4) and from another
process.

- **What it takes:** the invocations of ../protocol/git.md, section 4, with
  their settings (../protocol/git.md, section 3) and their environment
  (../protocol/git.md, section 5), and `git version`, answered as git
  2.53 answers it (../protocol/git.md, section 3), and nothing else.
  Any other argument vector exits as git's usage error does, which fails
  the test that caused it.
- **What it does:** it acts on the repositories' files on the machine,
  keeping its git state in each repository's `.git` as skein's fake
  checkout keeps it, and reaches the fake forge through its typed
  transport. Its commits are the fake forge's counts, written as 40-digit
  ids (skein's `fake-forge.md`, section 4).
- **What it prints:** what real git prints for those invocations: the
  porcelain lines, the NUL-separated paths and the ids ../protocol/git.md
  reads, and git's exit statuses; and, for the fake forge's refusals and
  faults (a missing repository, branch or commit, a wrong token, a
  protected branch, the forge out of reach, drawn from the seed), the
  stderr lines real git prints for them, which ../protocol/git.md, section 6
  classifies.
- **What it checks,** as the providers' fakes check their clients: the
  credential only in `GIT_CONFIG_*`'s `http.extraHeader`, for an identity
  the fake forge knows, a wrong one answered as Forgejo answers it; no
  `PATH`; the settings present; the remote a `https` URL, or a loopback
  `http` one.
- **Checked against real git:** its answers to the transcripts of
  `tools/git-transcripts` match real git's, masked (../protocol/git.md,
  section 9).
- **Where it runs:** the one-process world (2.5) on the typed
  transport; the real loop (2.4) over the git route of the forge peer it
  hosts, so that the gate runs the route; `temper fake git` in the demo,
  the QA loop and the process tier. The domain worlds keep the fake
  checkout at the domain face.

**Over the git route,** the program is the same core with a remote of
its own:

- **Its calls:** skein's fake checkout asks its remote synchronously
  (skein's `fake-checkout.md`, section 4), and the route is across a
  network, so the program replays. It answers the kit from what it has:
  the objects it fetched, kept in a cache, and a log of the other
  calls' answers, in order. A call it lacks gets a placeholder answer
  (no commit, an empty tree or message, a refusal), and that run is
  thrown away; the program makes the call, then runs the command again
  from a fresh copy of the checkout it scanned. A run that replays must
  ask the logged calls in the same order, and the program asserts it
  does. A write (a push, a branch made, a commit, a merge or a commit's
  tree stored) is made once, never again by a replay. A command's calls
  share one connection.
- **Its network is io's:** the core makes each call through skein's
  HTTP client on the kernel's sockets, as it reads files, and never
  blocks, so a ring that hosts both the program and the forge peer, as
  the real loop's does, serves both.
- **Its forge:** the listener is the host of the remote URL on git's
  command line, for a command that names its remote; for a local one,
  the URL its clone wrote in `.git/temper-remote`, which holds no
  secret. The credential is the `Authorization` git's `http.extraHeader`
  carries (../protocol/git.md, section 5), forwarded as it is; the
  store's calls carry none (skein's `fake-forge.md`, section 4).
- **Its exits:** what real git exits, for what real git does or
  refuses; 128 and a `fatal:` line, as git's on a network failure, when
  the route is out of reach or fails; 125 and `temper fake git: broken:
  <rule>` on stderr for a spawn that breaks what it checks (above),
  which the hosted program asserts; and, for a remote whose host serves
  no git route (an unnamed 404 or 405, or an answer the route can't
  read), 128 and `fatal: <host> serves no fake git route`. A host that
  refuses the credential before it routes, as the fake LLM peer does,
  answers as the fake forge refuses an unknown credential, an unnamed
  401, and is git's 401 line. An assertion of skein's kit ends the
  process as a panic (101).
- **How the binary is it:** `temper` is `temper fake git` when its
  `argv[0]`'s last component is `git`, before it reads a command of its
  own: a link named `git` to the binary, the fake setup entry's `git`,
  which the worker spawns by the absolute path each of the forge's
  repositories' records names (../protocol/setup.md, 8.2;
  ../protocol/channel.md, section 4), with git's command line as it is.
  A person can run the same program as `temper fake git ARGS...`.
- **Its crates:** the core, `crates/temper-fake-git`, ordinary Rust
  (programming-model.md, 10.2) with no test dependency, since the
  binary ships it: the checks, the answers, the step core, the replay
  and both transports; over `skein-fake-checkout`, the fake forge's
  domain, protocol layer and peer, and `skein-http`, each in skein's
  `crates/` (skein's `testing.md`, section 6). Its hosted half,
  `testing/temper-fake-git-hosted`: the program as skein's world hosts
  it, from the world's `Inherited`, with the fixtures and scenarios the
  worlds share.
- **Checked:** the core's step tests, by a scripted remote: each call
  missed and replayed, a replay that asks out of order, each write made
  once, the listener found from the URL and from `.git/temper-remote`,
  and each exit. The transcripts run on the core whatever its remote.
  The real loop's stories run the route against skein's forge peer; the
  process tier (2.6) runs `temper fake git` from the demo's link.

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
repositories, and write and correct notes (`root.md`, section 12),
on the forge and through temper's client. Their fake is a scripted actor,
drawn from the seed.

- **On the fake forge** it acts as a forge user.
- **Through temper's client** it is skein's: skein's fake person
  (skein's `ui.md`, 17.1), reading the tree temper's view makes and
  acting on it, with jig's client test kit's faults and check (jig's
  `client/domain/testing.md`), in `tests/client`; and the same person in
  front of temper's terminal client (../client/README.md, section 2), in the
  one-process world and the process tier (../client/README.md, section 10);
  in the real loop (2.4) a typist takes its place: ordinary Rust that
  types commands and reads the printed pages.
  temper has no fake person of its own (../client/README.md, section 2).

In the engine's worlds people are scripted, ordinary Rust rather than a
domain: each story's person looks at the forge as observed, never at its
store, and does what the story calls for next, one thing at a time, so a
call lost to a restarting engine is simply tried again. Where no client
is in the world, they make the engine's own calls. The system world
reuses them.

### 4.5 Temper's own components

temper keeps no fake of its own components: the system world runs the
real engine and worker, joined by the protocol layer's translation
(../protocol/channel.md, section 7).

- **The composed system world** (`tests/system`): the real engine root,
  worker root and smith's agent domain; skein's fake LLM, which scripts
  only completions' contents; jig's store component over skein-kv's
  in-memory mode (../protocol/store.md, section 6), and jig's fake
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
`temper-service`'s limits module (../protocol/configuration.md, section 7),
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

### 5.3 The reference case

`../draft/reference-case.md` is one story that holds every move of the
owners' process. Its steps run as stories of the real loop (2.4), each
on the case's cast, its scripts and its oracle, written once and run
again by every world that runs the case: the real loop's cluster, and
the process tier's (../protocol/README.md, 15.6).

- **The cast is a fixture.** The person is a client's directory, signed
  in; the director is the person's chat in project 1, on a
  coordinator's charter; the owners (A, B, C) and their delegates are
  the agent and procedure tasks the scripts delegate; kit, and app when
  a step needs it, are repositories on the fake forge, kit's tree
  holding a draft and the process's documents; the workers are the
  standalone service's one, and ws, s1 and s2 once the case runs on the
  cluster. The tasks' numbers are a fresh deployment's, named once and
  checked against the documents the story reads.
- **Every judgement is a script.** Each choice the case gives an agent
  (the director's, an owner's, a delegate's) is a fake LLM script played
  word for word, cued by the task's words (5.1). Each act of the person
  is a `temper ctl` call. Nothing on the fakes judges, so the case's
  judgements are its scripts, and its oracle checks what temper did with
  them.
- **The oracle checks effects,** one checkpoint for each step, over
  what the step's Musts and Adds promise: authority and spend, a message
  or an answer delivered, work held and resumed, what landed. It reads
  only what every world that runs the case observes: the person's
  documents (`temper ctl`'s JSON, by the act that read them), the forge
  (its main and its checks at each merged head), and what each agent was
  told (the fake LLM's queries, by their cue). It never judges an
  agent's words, only their effects, so the same oracle checks a live
  model's run of the case. It reports every miss at once, each naming
  its step and its Must or Add.
- **Time keeps the case's order, not its ratio.** A story waits on
  conditions, `temper ctl`'s waits, never on time. A timer, deadline,
  cap, reset or grace that a step runs on is armed where the case arms
  it, at the shortest length the loop keeps in order, and the story
  waits for its effect. The case's hours are not mapped to a ratio.
- **Smoke, as the real loop is** (2.4): a step's story runs its path
  end to end, once; the cases each Must rests on are tested in the
  lower worlds that own them. Each story is held to 6.1's guide, and
  one past it after its cost has been cut is a fuzzy test (6.1).
- **What a gap holds is named.** A step whose Must needs what temper
  lacks (`../draft/reference-case.md`, section 9) runs the part that
  works; its story's name says what it shows, and the table below what
  it leaves out and what holds it. Behaviour that conflicts with the
  case is not changed here, and not counted as the case's.
- **A library of the real-loop world:** `tests/real-loop/src/case.rs`,
  the cast and its fixture, and `case/`: `scripts.rs`, the judgements
  for each step; `acts.rs`, the person's acts for each step; and
  `oracle.rs`, the checkpoints over one type of observations, which
  each world fills from what it observes. The stories are
  `tests/real-loop/tests/case_*.rs`, or `fuzzy_case_*.rs` past the
  guide.

Act 1 (`../draft/reference-case.md`, section 4):

| Step | Its story shows | It leaves out, and what holds it |
|---|---|---|
| 4.1 | `the_opener_and_the_two_hour_check_run_on_three_workers` (`fuzzy_case_4_1_4_3`, past the guide by the change procedure's landing path), its first part, on the cluster of ws, s1 and s2, one slot each (../protocol/README.md, 15.7), ws and s1 started: the person asks the director to schedule a draft at priority 1; the director delegates A, an agent on the model and effort it names, tracked as a goal at that priority, its brief naming the draft's commit, under authority within its own; A has its design delegated, a change of Markdown alone on kit, which lands through the change procedure with CI green at its head; A sends its design report to the director as a question, and the director answers it with a green light. The oracle: A tracked at its priority, on its model, with its brief, its authority within the director's, and its spend rolled up to the goal; the report delivered to the director, and the answer to A; kit's main holding the design, merged with CI green, and nothing but Markdown changed. The story asserts where each task ran, a fact only the harness sees: the director's chat, which waits in its run, keeps ws; A, and its design's coder and reviewer after it, run on s1. The engine places them by the cached workspace first, then free slots, and the story only checks where they went. `qa_case_4_1`'s misses run the opener alone, on the standalone service. On the process tier's cluster (2.6), `the_opener_and_the_two_hour_check_run_on_three_workers_as_processes_without_pools_lending_or_host_moves` runs this part with the same acts, scripts and oracle, ws and s1 being processes, and asserts the director's chat on ws, by the acts' reads, and the design's change made on s1, whose workspace alone holds it; A's and the director's scripts carry rule tables, in both worlds, so that kit's landing news, which on processes can wake A between its question and the green light, is answered by a wait (2.6) | A's survey before its design, one more report delegate than the director's own, for 6.1's guide; A's instructions and its pool (jig's `domain/tasks.md`, section 6), which the agent's `delegate` doesn't decode (`agent.md`, 4.3); which workers have room (`../draft/temper-on-temper.md`, 6.7); a host kind, which neither the agent's `delegate` nor jig's task spec or amendment names, and no worker declares: placement by free slots stands in; s2's part in the opener: the opener's agents run one at a time behind A's workspace, so it fills two workers, and s2 joins in 4.3; the director's chat waiting in its run, as the case's director keeps ws as its home, on a charter of the case's fixture for the person's chats (5.1): its threshold is past the story's patience, a mechanism never waited out, every run ends on its own conditions, and a run that parks is 4.4's and 4.5's, on the standalone service; the fast path for Markdown alone, which is the repository's CI's and not temper's (`../draft/temper-on-temper.md`, 4.3): the fake forge's CI passes at once; the goal's issue, with kit adopted without a home, since a goal whose issue write ends uncertain or refused is held from closing (`forge.md`, section 12), a finding of this story's; the director as a standing chat: its chat ends with its report once A has ended, so that the person's acts wait on it; A's report in 15 lines: its design report is the question it sends. Its process story leaves out the same, and where A and its design's reviewer ran: no act reads them while they run, an ended task's document names no worker, and neither leaves a tree of its own on its worker (2.6) |
| 4.2 | none | CI at the tip and the queue's priorities are the forge's worlds' (`forge.md`, sections 8 and 9); a failure at the tip that matches an open finding leaving the queue running is undesigned (`../draft/temper-on-temper.md`, 6.5), and today's pause and repair at a failed tip is the conflict the case names, so the step does not run as the case's |
| 4.3 | `the_opener_and_the_two_hour_check_run_on_three_workers` (`fuzzy_case_4_1_4_3`), its second part, after the opener: the person asks the director, in a second chat, to schedule B's benchmark and to run the two-hour check; the director delegates B, an owner, and B delegates its benchmark, each waiting in its run, so B keeps s1; the benchmark is due with no free slot; the person adds s2, which links, and the benchmark is placed there and waits in its run, holding s2: the stall; the director arms its check, a timer, once every worker holds a run; the timer's wake reaches the director, which cancels the benchmark with its reason; B, told of its delegate's cancelled ending, reports, and the director reports. The oracle: the director told its timer's wake after its subscription, and its activity recording the timer; the benchmark ended, cancelled by the director with the reason, its run's end cancelled; no run of the benchmark left open, and B told the cancelled ending before its report. The story asserts what only the harness sees: ws ran the directors alone, s1 A's tasks and B, s2 the benchmark alone; s2 started after ws and s1 each held a run; every worker drained, and no agent killed. The cancel is cooperative: the benchmark's run stops at its next turn (smith's stop), so the story shows a cancel that is heeded. On the process tier's cluster (2.6), `the_opener_and_the_two_hour_check_run_on_three_workers_as_processes_without_pools_lending_or_host_moves` runs this part with the same acts, scripts and oracle, ws, s1 and s2 being processes, s2 started by its act: the check's turn is held on the fake LLM process until every started worker has held a run, and released by its control (4.1); the story asserts the directors on ws, B on s1 and the benchmark on s2, by the acts' reads | a run that ignores the cancel, its worker holding it past a bound, which is the hung worker's, 4.6's; pools resized by amendment and a slot lent to A: there is no pool control on main, in temper's `ctl` or the agent's tools, and one slot per worker (../protocol/README.md, 15.7); a host kind amended for the next run, which jig's amendment doesn't carry (`../draft/temper-on-temper.md`, 6.4); the check's report of load, queue and disk, worker health being undesigned (6.4); the timer's wake stands in for it; B moved to s2 with its notes and branches, which is placement by host kind: here B keeps s1's one slot, so the benchmark stalls on s2 and not on s1, the case's order under one slot per worker; data that is neither git nor notes, a benchmark's quiet machine, disk floors, build lanes, caches and memory caps, the plane's (hearth and the lab); the two hours: the timer is due at once, `at` 0, the shortest length that keeps the case's order, its turn held until every worker holds a run; B and the benchmark waiting in their runs, on the person's chats' charter (5.1), as the director does: a run that parks frees its slot, and the stall is a slot held. Its process story leaves out the same: the pools, the lending and the host moves are in its name |
| 4.4 | `the_pace_question_reaches_the_person_and_its_answer_is_kept` (`fuzzy_case_4_4`, past the guide by its seven activations, S3's floor): the deployment holds two accounts, the second idle; the director delegates A on the owner's model, without its design; A asks the director the pace question, its recommendation first, and the question waits in the person's inbox, the director being the person's chat; the person ranks the second account first with `temper ctl rank` and answers the question with `temper ctl answer`; the director, woken by the answer, writes it into a note of the project and amends A's model to the default one, the amendment's reason relaying the answer to A, which wakes once; A, on its new model, has a survey delegated, which pays from the account now first; the director recalls the note before its report. The oracle: the question in the person's inbox, on the director's task and A's message, until it is answered; the person's answer in what the director was told, after A's question; the note holding the answer, and recalled; A's model the director's choice at its next spec revision, and A's next query on it; the accounts in the person's order, and the survey's run paying from the account now first while A keeps the account it started on. On the process tier's cluster (2.6), `the_pace_question_reaches_the_person_and_its_answer_is_kept_as_processes_on_one_worker_without_plan_pacing` runs it with the same acts, scripts, fixture and oracle, the engine and ws being processes: the second account's key is the fake LLM's second accepted credential (../protocol/setup.md, 8.4), written where the act reads it | the director as the asker: a task can't ask its own person, since a question comes from a delegate (jig's `domain/tasks.md`, 8.1), so A's question to the director, which waits for the director's person, stands in for it; a plan's windows, its resets and its sustainable pace as the account's facts (`../draft/temper-on-temper.md`, 6.3); B's model, one owner amended being enough; s2's sources, with one worker until the cluster's stories: the holder's order of its accounts stands in, and a task keeps the account it started on; the question not asked again, which the note's recall stands for; the director amending A's model alone: the story's amendment also changes A's spec, the fake LLM's need, since after a model change smith recaps A's conversation and the fake, which picks a turn by the count of replies said, can't follow a recap, and the oracle passes no judgement on the spec. Its process story leaves out the same: s2's sources stay out on the cluster too, a worker's own sources being undesigned, so it runs on ws alone (2.6); the plan's pacing is in its name |
| 4.5 | `a_refused_key_holds_its_work_until_it_is_replaced` (`fuzzy_case_4_5_refused`, past the guide by smith's retries of a refused key, about 1.1 s of backoff, and by its activations, about 1.1 s without them): A's first query is refused (the fake LLM's `Unauthorized`); A waits, its task naming its account and the refusal, its account to be signed in again, until the person replaces the key with `temper ctl replace-key`; A then resumes and finishes. `a_limited_account_holds_its_work_until_its_reset` (`fuzzy_case_4_5_limited`, past the guide by its reset): A's first query finds its account limited with a reset (the fake LLM's `Exhausted` with its retry); A waits, its task naming the limit and its reset, its account `later` until then, and resumes past the reset with no act. The oracle, for each: the waiting task's run wait naming its account and why, read while it waits; the account's state, and for a limit its reset the run wait's, within a millisecond, since each document projects the engine's one deadline to wall time as it is read; the same task resumed on the account it kept, no task recreated, and its conversation kept: the query after the resume holds what A was told before it; the work finished after the act or the reset. On the process tier's cluster (2.6), `a_refused_key_holds_its_work_until_it_is_replaced_as_processes_on_one_worker_without_a_sign_in` and `a_limited_account_holds_its_work_until_its_reset_as_processes_on_one_worker` run them with the same acts, scripts, fixtures and oracle, the engine and ws being processes: the refusal and the limit are the fake LLM's failing lines for its first credential, and the replacing key its second (../protocol/setup.md, 8.4) | the sign-in on s2, and credentials never copied between machines: one worker until the cluster's stories, and the real loop hosts no fake issuer for an LLM provider's sign-in (2.4), so a key's replacement stands in for a sign-in; B's tasks, one owner held being enough; the case's five hours: a reset is whole seconds on the provider's wire, so the shortest is one second, which puts the limited story past the guide; a session that never ends: the waiting task's run ends, and a new attempt resumes its committed conversation (jig's `domain/accounts.md`, 3.5 and 4.3). Its process stories leave out the same: the sign-in on s2 stays out on the cluster too, the engine hosting no fake issuer for an LLM provider's sign-in, so they run on ws alone (2.6), and the refused one's name says it |
| 4.6 | `a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` (`fuzzy_case_4_6`), a cluster story of its own, so that each of the case's cluster stories keeps its margin under the real loop's patience on a busy machine: the case's third part, on ws, s1 and s2, all started and each free: the person opens a chat with the director, whose run is placed on ws, the first free slot, and waits in it; the director delegates A's next phase, an owner that runs on another worker and sends the director its question; the director reads it, a read committed by a completed turn (a recall of the project's notes), and waits again with its answer not yet sent; the person stops ws, whose ending (../protocol/README.md, 15.4) ends the director's run, failed, into a retry; after the case's backoff the retry is placed on a free worker, resumes from the chat's committed turns with A's question kept, and answers it once; A, told the answer, reports, and the director reports; then the person starts ws again on its own three directories, and it links at its position and runs nothing. The oracle: the director's chat ran two attempts, the first ended failed and the next began after it and finished, never two open, in the journal's order; the director's read of the question committed and its run waiting before ws stopped; the next attempt told the question once, one answer sent across both attempts, and A told it once; A's phase ended with its report. The story asserts what only the harness sees: the first attempt ran on ws and the next on s1 or s2; ws's two lives each linked once and drained, the second running nothing; every worker drained and no agent killed. Time keeps the case's order: an agent's failure backs off 1 ms in this story's fixture (`tasks.retries.agent.base_ms`), the shortest whole length the configuration takes and never zero: the retry's path is the same and only its wait is short, the backoff is not what the story tests, and the order comes from the engine, which journals the ended attempt before it places the next, the order the oracle reads; the profile's own backoff stays the engine's domain world's, and every other case story keeps it. ws's ending waits on events: the director's committed read (polled in its activity, since no `ctl` wait names a committed turn), its run waiting (`ctl wait --until replied`), then ws's ring settling. ws starts again once the chat has ended, so that the retry, placed by the cached workspace first, cannot go back to it. On the process tier's cluster (2.6), `a_restarted_workstation_keeps_the_directors_question_and_answers_it_once_as_processes_without_a_hang_past_the_grace` runs it with the same acts, scripts, fixture and oracle, ws, s1 and s2 being processes: ws stopped by its ending (`Members::stop_one`) and started again on its own directories (`Members::restart`); the story asserts the first attempt on ws, by its live channel and ws's first life's mapping read while the run waits, before the stop; and the next on s1 or s2 by the acts' order, not a mapping: ws is ended from before the first attempt fails until after the chat has ended, so the retry, whose channel no read is sure to see before it ends (2.6), is placed while s1 and s2 are the only workers linked; ws's two lives each linked once, and no forge token in any file or document the harness gives a worker (2.6) | the hang past the grace, and its four hours (exact): here the link deadlines stay past the story's patience (2.4), so no grace runs out, ws goes away by its ending rather than a hang, and the director's run ends failed as an agent's, not lost; the loss and the grace are the engine's domain world's, in root_routes.rs `a_lost_workers_run_is_placed_on_the_other_worker_past_its_grace`, `a_worker_lost_mid_run_resumes_at_the_last_committed_turn` and `a_worker_frozen_past_its_grace_gets_no_next_attempt_until_its_sum_has_passed`, and forge_routes.rs `a_worker_frozen_past_its_grace_resumes_with_a_push_in_hand_and_lands_nothing_twice`, and the process tier's lost and hung workers (../protocol/README.md, 15.6); a fenced attempt still running when its worker returns, and two resumed copies of a session beside their originals: the process tier's hung worker, and jig's fleet world, where a fenced attempt's turns are acknowledged and dropped: here ws's run has stopped before ws returns; a restarted worker that takes new runs, the process tier's (../protocol/README.md, 15.4): here ws links and runs nothing, the case's work having ended; a stalled run held past its bound and retried (exact): smith's no-progress watchdog, in smith's host world, which temper's worker translates into an agent's failure and jig retries within its allowance; no composed world stalls an agent, since no fake agent hangs (`../draft/reference-case.md`, section 9); three of A's questions: one waits, since the limits that three questions take raise each worker's ring provision until three workers' together pass what one io_uring ring holds; no single point: one engine, whose ending ends the cluster (../protocol/README.md, 15.4), undesigned; the reboot's other faults, an orphaned completion checker and a Codex run hanging on its stdin, smith's and jig's worlds; the build queue's environment, hearth's (the plane); forge credentials never on a worker: the real loop's fake forge writes its token into each worker's staging secrets (`tests/real-loop/src/forge.rs`), so the story can't assert their absence: the director's backlog; its process story asserts it of the files and documents a worker is given, not of a git call's credential, which reaches the worker per call. Its process story leaves out the same but that check: the hang past the grace is in its name, and three of A's questions stay out, since the oracle, which is the real loop's, holds one |

## 6. What the tiers check

Every world checks what the strategy lists (testing-strategy.md, section
6). In temper:

- **Contracts** include each operation's deadline and identity, and each
  component's own (`worker.md`, section 10).
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
- **Budgets.** The suites' 30 s (15 s before the user's doubling of 10-22,
  provisional) and 120 s (60 s before the user's doubling of 10-22,
  provisional) are rules, which nextest enforces. Within them, guides:
  - the worlds below the domain other than the real loop (the protocol,
    machine, client, one-process and reconcile worlds) together at most 8 s
    (4 s before the user's doubling of 10-22, provisional) of the default
    suite;
  - the real-loop world's tests, its stories included, together at most 13 s
    (6.5 s before the user's doubling of 10-22, provisional) of the default
    suite: a standing ceiling, the same for each plan, sized to the smoke
    that 2.4 keeps the real loop to ("one end-to-end path per feature, and
    each behaviour and corner case tested in the lower world that owns it,
    never here"), which the user revisits when the sum reaches it;
  - the worlds below the domain, the real loop included, together at most 80
    s (40 s before the user's doubling of 10-22, provisional) of the fuzzy
    suite;
  - a focused protocol-world test at most 1 s (0.5 s before the user's
    doubling of 10-22, provisional), and a one-process or real-loop story at
    most 2 s (1 s before the user's doubling of 10-22, provisional);
  - a GitHub story of the real loop (2.4) at most 1 s (0.5 s before the
    user's doubling of 10-22, provisional), and one that runs a whole chat
    (its agents, git and `ctl`) at most 3 s (1.5 s before the user's
    doubling of 10-22, provisional), so that its eight stay within the real
    loop's ceiling with the rest; a whole-chat GitHub story that would pass
    the ceiling goes to the user with its numbers.

  A test that misses its guide is made cheaper or moved to the fuzzy suite,
  never merged with another and never named as an exception: a real-loop
  story with CI and webhooks (2.4) is held to the 2 s (1 s before the user's
  doubling of 10-22, provisional) as any other. If GitHub's stories would
  push the real loop's sum past 13 s (6.5 s before the user's doubling of
  10-22, provisional), the ceiling goes to the user, as below, and no
  real-loop story moves to the fuzzy suite to make room for them unless the
  user so decides.
- **The worlds' sums are measured as the suites' rules are:** each test's
  own time, run alone on the idle development machine (testing-strategy.md,
  section 8), with `--profile measure -j 1` through `heavy --gate`, the
  median of three runs. Beside another task, it measures that task.
- **The sums are the worlds' guides, never a landing's.** A test within its
  own guide lands. A plan that adds to these worlds gives both default-suite
  sums at its base and at its end: the other worlds' against 8 s (4 s before
  the user's doubling of 10-22, provisional), the real loop's against 13 s
  (6.5 s before the user's doubling of 10-22, provisional). When the other
  worlds together pass 8 s (4 s before the user's doubling of 10-22,
  provisional), or the real-loop world 13 s (6.5 s before the user's
  doubling of 10-22, provisional), their slowest tests are profiled, then
  made cheaper at the cause the profile shows or moved to the fuzzy suite,
  by steps a plan names. A plan whose end finds the real loop's sum at 13 s
  (6.5 s before the user's doubling of 10-22, provisional) or past it puts
  the ceiling to the user, as skein's `development.md`, 10.4, "What goes to
  the user", says; until the user moves it, the 13 s (6.5 s before the
  user's doubling of 10-22, provisional) stands.
- **Outside both suites:** the process tier's binary `process` and the
  live tier's binary `live`, which both suites' filters leave out, run
  under the nextest profiles `process` and `live` by
  `scripts/proof-one-process.sh` and `scripts/proof-live.sh`. The gate
  never runs them; `scripts/after-landing.sh` runs the process tier
  after each landing (skein's `owners.md`, 4.5).

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
world of temper's on a scripted engine (../client/README.md, section 10).

- **The translation** between the client's vocabulary and the engine's
  is exhaustive both ways, so a variant added on either side without
  the other is a build error.
- **The joint world is the client's acceptance.** Each client feature
  has its stories there, or extends one.
- **jig's scripted engine** is jig's; the forge's client family has no
  scripted engine of temper's, so its stories run against temper's
  engine root alone.
- **The agents' tools** run there too: their root, as a scripted agent
  beside the fake person (../client/tools.md, 11.2).
- **A run's live text** has its story there, with a scripted host in
  place of an agent: the host sends a run's text in pieces, held after
  the first (jig's `protocol/channel.md`, 6.4), and the fake person
  reads the chat's provisional "Writing…" turn with that piece, then the
  rest of the held text, and then the committed turn, which replaces it.
  The joint world's front plays the run's worker: at the chat run's
  assignment it sends the reply's first piece up the worker link, as
  the text kind at offset 0, and holds; at each of the story's releases
  it sends the next of its parts, the rest of the text as a second piece
  at its offset, then the committed turn with the whole text and the
  run's answer.
  Retries, gaps and cuts are jig's views' to test (jig's
  `domain/engine.md`, section 11); this story checks temper's wiring
  from the engine's root to the chat's page. The same story over real
  agents runs in the real loop (2.4).

## 7. Layout

```
crates/*/src/tests.rs               step tests
tests/codecs                        temper-codecs-world: the generated codecs and the foreign documents' decoders
crates/temper/tests/process.rs      the process tier (2.6): the story, the terminal, the server's guards, the hinted story, `temper box`, the cluster's stories, GitHub's git; nextest profile process
crates/temper/tests/support/github.rs   GitHub's git as processes (2.6): the fake GitHub, the fake LLM, an engine alone and a worker alone, the App seeded
crates/temper/tests/support/cluster.rs  the cluster's reader: `lab/cluster/` started, awaited and stopped on skein-world's `Binary` (../protocol/README.md, 15.5)
lab/cluster                         the cluster's description, in the lab's format, which the process tier and skein's lab runner both run
crates/temper/tests/live.rs         the live tier (2.7), nextest profile live
tests/client-browser                temper-client-browser-world: the browser tier (../client/browser.md, section 4), Chromium against temper serve on the bundle the gate builds; nextest profile browser
crates/temper-client-shell-terminal/tests  the terminal's machine world (../client/README.md, section 10)
crates/temper-fake-git              the fake git program's core: temper fake git (4.3)
testing/temper-fake-git-hosted      the fake git program hosted by skein's world, with its fixtures (4.3)
testing/temper-github-app           the App on the fake GitHub layer's first fixed key, its records, seeded in jig's secrets' store before a start (2.4, 2.6)
tests/<component>/<child>           a child domain's world, temper-<component>-<child>-world
tests/<component>/domain            a component's world: the engine's, the worker's
tests/engine/forge                  the forge connector's world
tests/engine/protocol-forge         the forge's protocol world (2.2)
tests/worker/protocol-git           the git component's machine world (../protocol/git.md, section 9)
tests/system                        the composed system world (2.1)
tests/conformance                   the engine root on jig's conformance harness
tests/client                        temper-client-world: temper's client with skein's fake person, joined to the engine's root (2.1, 6.3)
tests/one-process                   the one-process world (2.5)
tests/real-loop                     the real-loop world (2.4), in the focused suite
tests/*/*/tests/*.rs                a world's focused tests
tests/*/*/tests/fuzzy_*.rs          its fuzzy tests: random worlds, domains driven at random
tests/fake-checkout                 temper-fake-checkout: working trees' git, over skein's fake checkout
tools/git-transcripts               git's transcripts, from real git against skein's throwaway Forgejo (../protocol/git.md, section 9)
tools/gallery                       temper-client-gallery: the browser's pages in named states, printed for screenshots (../client/browser.md, section 7), its tests in the view's world
scripts/proof-one-process.sh        the process tier
scripts/proof-live.sh               the live tier
scripts/after-landing.sh            the process tier, after each landing
```

- **The worlds and the harness** are skein's world harness
  (`skein-world`); each world's crate is a library other worlds may use
  (4.5).
- **The machine-level fakes are skein's:** the fake machine
  (`skein-fake-machine`), the fake checkout, the fake LLM and its peer,
  the fake issuer. temper's fakes sit on them.
- **The fake forge is skein's** (skein's `fake-forge.md`): its domain
  (`crates/skein-fake-forge-domain`), its protocol layer
  (`crates/skein-fake-forge-protocol`), its forge peer (in
  `crates/skein-fake-peers`), the throwaway Forgejo
  (`testing/skein-throwaway-forgejo`), its tests in ordinary Rust, and
  the conformance (`testing/skein-forge-conformance`). Reconcile's world is
  in skein's `forge-setup.md`, section 7. temper's fake git program,
  its core in `crates/temper-fake-git` and its hosted half in
  `testing/temper-fake-git-hosted`, sits on the fake forge (4.3).
- **Machine worlds live beside what they test:** the forge component's is
  in skein's `forge-connection.md`, section 10; the git component's is in
  `tests/worker/protocol-git` (../protocol/git.md, section 9); the terminal's
  in `temper-client-shell-terminal`'s own tests (../client/README.md, section
  10). temper's protocol world for the forge, over skein's crates, is
  `tests/engine/protocol-forge` (../protocol/forge.md, section 10).
- **The setup's crates** test as step code does: `temper-forge-setup`'s
  files, specs and steps, and `temper-box`'s machine, by their step
  tests (../protocol/setup.md, 13.1); reconcile's machine by skein's world;
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
  in skein's `fake-forge.md`, section 3. What needs it: a real agent
  on the fake forge, which runs git in its working tree as a person
  does; nothing else does, since the fake git program reaches the fake
  forge across processes too (`../draft/qa-loop.md`, section 11), and
  real git's own behaviour is the transcripts' and the process tier's.
  A likely shape: the domain stays authoritative over refs and policy,
  the bare repositories mirror it, the protocol layer keeps a table of
  the domain's counts to git's ids, a push's ref updates are decided
  by the domain before git writes them and its tree imported as the
  domain's commit, and a merge the domain makes is written to the mirror
  with a fixed author and time, so its id is the same on every run.
  Default: the real loop's clone, push and landing stories run on the
  fake git program over the fake forge's git route (2.4), real git meets a forge
  in the process tier (2.6) and the QA loop's M2, and no story waits for
  the bridge.
- **What the real loop leaves out:** the real-git bridge (above),
  CI that runs a workflow's steps in the shell subset (how much sh,
  below), and faults in the loop. Several workers in one world are the
  cluster's services (2.4); the cluster's faults, a worker lost or hung,
  are the process tier's (2.6).
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
  host. Default: process groups, but the cluster's tests, which observe
  several processes and need skein-world's cgroup trees, delegated by
  the user's service manager, and fail at their start without them
  (2.6).
- **The machine's git in the process tier.** Default: whatever git the
  test host has, 2.41 or later, which ../protocol/git.md's start refuses below
  (../protocol/git.md, section 3); transcripts are of 2.53.
