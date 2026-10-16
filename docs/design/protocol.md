# The protocol layer

Provisional, 2026-10-03, revised 2026-10-16. temper's protocol layer:
how temper composes jig's protocol layer with components of its own (its
forge's, its checkout's, its client's) into the processes of one binary.
jig owns the layer's shape (jig's `protocol/README.md`) and the services'
(jig's `protocol/service.md`); this document says what temper adds, the
crates it is made of, and the order it is built in. The mechanics are
those of skein's `programming-model.md` (sections 2, 4 and 9); the
entities it carries are those of `domain/`. Each boundary of temper's own
is designed in a document of its own (section 2). What is open across
them is listed in section 14.

## 1. In one page

- **Translation, and nothing else.** The protocol layer turns bytes into
  the domains' entities and back, and decides nothing about the domain
  (programming-model.md, section 4). The domain decides deadlines,
  retries and what a failure means; the protocol layer runs the call, its
  timer and its one attempt.
- **jig's protocol layer, composed.** jig ships the components for its
  own vocabularies: the store, the workers' channel, LLM accounts,
  clients, configuration (jig's `protocol/README.md`, section 5).
  temper's services own them as children beside smith's, skein's and
  temper's own components, and route between them, io and the roots, as
  jig's reference service does (jig's `protocol/service.md`, section 2).
- **What temper adds:** the forge's protocol (the forge component, over
  Forgejo's documents, with its webhooks' listener); the checkout's io
  (the git component); its parts' codecs on jig's channel; its families
  in jig's store; its git identities; its host tools' translation; its
  client (its wire, its client root and view on `jig-client`, its
  terminal); the clients' listener, on which it mounts jig's clients'
  component; and the forge's setup, commands outside the services
  (setup.md). Each is a crate of its own (section 5).
- **Foreign protocols are stacks of skein's machines** with temper's
  decoders on top: Forgejo's REST API over skein's HTTP client and JSON;
  git's arguments in and text out over io's processes. temper's records
  on jig's channel, in jig's store and on the client's wire are generated
  from schemas by skein's generator.
- **Standalone first.** `temper serve` runs the engine and one worker in
  one process, on one loop, joined by jig's in-process address; the
  worker's agents are child processes of temper's own binary (`temper
  agent`); a person's client reaches the engine on loopback; the forge is
  polled, and hinted by its webhooks on loopback when they are
  configured (section 8; forge.md, section 5). Hosts apart, webhooks from
  another host, the forge as a sign-in provider, the forge's client
  documents and the browser come after (section 12, "After standalone").
- **One binary:** `serve`, `agent`, `open`, `sign-in`, and the forge's
  setup's `forge prepare`, `forge run`, `forge reconcile` and `box`
  (section 4).
- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines (section 5).
- **The forge's load follows its changes.** The calls a pass makes grow
  with what changed on the forge, never with its history, its labels or
  the items temper holds, and the worlds count them by HTTP request
  (forge.md, section 6).
- **Secrets never reach a domain.** A domain names a credential by number
  and generation; values live in the protocol layer's tables and in
  private files: LLM accounts as jig keeps them, git identities, the
  forges' API tokens and their webhooks' secrets as temper does (section
  10; credentials.md).
- **Upstream as it is.** temper uses jig, smith and skein at its pinned
  revisions; nothing in this design waits for a change upstream but the
  webhooks' HMAC-SHA256 and its comparison in constant time, in skein's
  lib (section 11).
- **Tested at every tier:** codecs and parsers against golden bytes and
  transcripts of real peers; the forge component and the git component in
  worlds of their own; temper's client joined to the engine's root; the
  one-process world on skein's simulator; the real binary with an owned
  Forgejo, its runner and real git; a live tier with a real model
  (section 13; testing.md).

## 2. Reading order

1. **jig's `protocol/README.md` and `protocol/service.md`:** the layer
   temper composes, and the services temper copies.
2. **protocol.md:** this overview.
3. **configuration.md:** what `temper serve` and its other commands are
   told at start.
4. **channel.md:** temper's parts on jig's channel, and a git identity's
   value.
5. **git.md:** the checkout's io: the git component.
6. **forge.md:** the forge component, Forgejo's API as temper uses it,
   its webhooks, and the fake forge's protocol layer; then
   **setup.md**, the forge's setup: connected, managed and owned.
7. **store/README.md:** temper's families in jig's store.
8. **credentials.md:** LLM accounts, git identities, the forges' API
   tokens.
9. **llm.md:** where LLM calls are made, host tools, the charter and a
   run's result.
10. **client.md:** temper's client: its client root on `jig-client`,
    its pages, the forge's client family, its view, the terminal, and the
    clients' listener of temper's server.
11. **testing.md:** the tiers below the domain.
12. **web/architecture.md**, the browser, a later shell of temper's
    client; **performance.md**, copies and allocation.

## 3. Names

jig's names keep their meaning here (jig's `protocol/README.md`, section
3): the hub, the core, the fleet, the deployment's link, credential
grant, application credential, payload version, profile, the reference
service (`jig-test-service`), the reference binary (`jig-test`).

| Name | What it is |
|---|---|
| standalone | one process on one person's machine: `temper serve` with the engine and one worker joined in process (section 8) |
| the engine's service, the worker's service | each `iterate` with its root and protocol layer, in `temper-service` (sections 6 and 7) |
| the routing | the protocol layer's code between io, components and the root (jig's `protocol/service.md`, section 2) |
| the forge component | `temper-engine-protocol-forge` (forge.md) |
| the git component | `temper-worker-protocol-git` (git.md) |
| temper's parts | the workspace items, the saved-work branch, the work, the preparation (channel.md) |
| payload version, store version, client version | temper's three own versions: `PAYLOAD_VERSION`, `STORE_VERSION`, `CLIENT_VERSION` (section 9) |
| temper's families | the forge's records in jig's store (store/README.md) |
| git identity | one of temper's application credentials: a forge token, a login and an author (credentials.md, section 4) |
| the identities' table | the protocol layer's table of git identities' values, on the engine and on each worker |
| the API token | a forge's token for its API, the engine's protocol layer's only (credentials.md, section 5) |
| the forge's client family | temper's client side of the forge connector: its words, and the adoption form (client.md, section 4) |
| the terminal shell | `temper-client-shell-terminal` (client.md, section 6) |
| the clients' listener | `temper-service`'s listener for clients, on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig` (client.md, section 8) |
| the fake git program | `testing/temper-fake-git` (testing.md, 4.3) |
| the throwaway Forgejo | a Forgejo v17.0 started by a test script on loopback, with SQLite in a scratch directory, removed after (testing.md, 2.6) |
| locate | the forge connector's read of a repository's id by owner and name (forge.md, 3.2) |
| the webhooks' listener | the forge component's own listener for Forgejo's signed deliveries, plaintext on loopback, one path per forge (forge.md, section 5) |
| connected, managed, owned | the three modes of a forge's setup: a person runs and sets up Forgejo and gives temper its bot's tokens and id; a person runs it and gives temper a site admin's token; temper runs Forgejo and its runner (setup.md, section 2) |
| reconcile | `temper forge reconcile`: a managed or owned forge's bot, its tokens, its id, each org's team and org hook, converged at a start with the admin's token (setup.md, section 5) |
| the preparation | `temper forge prepare`: an owned forge's work path, settings, migrations, admin and runner's registration, made or upgraded before Forgejo runs (setup.md, 6.3) |
| the forge's directory | a set-up forge's private directory, where each layer of its setup writes the files the next reads (setup.md, section 3) |
| the pin | the Forgejo and runner versions temper runs and the conformance checks, constants of `temper-forge-forgejo` (setup.md, 6.1) |
| the box | `temper box`: one arrangement of a configuration's steps, wiring only (setup.md, section 7) |
| a step | a command the box runs: once, done when it exits 0, or long, run until the box stops (setup.md, 7.2) |

## 4. The processes and the binary

One binary, `temper` (`crates/temper`), ordinary Rust: a shell, copied
from jig's reference binary. Its commands, exactly:

```
temper serve CONFIG.json
temper agent AGENT.json
temper open DIRECTORY LINK_OR_ADDRESS
temper sign-in CONFIG.json ACCOUNT
temper forge prepare CONFIG.json FORGE
temper forge run CONFIG.json FORGE forgejo|runner
temper forge reconcile CONFIG.json FORGE
temper box CONFIG.json
```

- **`temper serve CONFIG.json`:** reads the configuration (configuration.md,
  section 2), opens the store in the deployment's directory, and runs
  the engine's and the worker's services in one process on skein's
  `Host` and `drive` (jig's `protocol/service.md`, sections 4 and 10).
  - Its stdout carries only sign-in links, one per line: each the
    secret jig's clients' component gives, in jig's default mapping's
    link route under temper's prefix (jig's `protocol/client.md`,
    section 8; client.md, section 7); its stderr carries startup notes
    and the reason of a failure.
  - A first termination signal drains it (jig's `protocol/service.md`,
    section 5); a second turns the drain into an abort.
- **`temper agent AGENT.json`:** smith's agent shell,
  `smith_agent_shell::run`, on smith's agent configuration
  (configuration.md, section 5). The worker's protocol layer spawns it,
  one process per run; a person never does.
- **`temper open DIRECTORY LINK_OR_ADDRESS`:** a person's terminal client
  (client.md; configuration.md, section 9). It reads no configuration
  file.
- **`temper sign-in CONFIG.json ACCOUNT`:** an LLM account's sign-in at a
  terminal (credentials.md, section 3). The engine need not run.
- **The forge's setup** (setup.md), each command reading and checking
  the whole document as `temper serve` does (configuration.md, section
  8), FORGE a forge's `number`:
  - `temper forge prepare`: an owned forge's preparation, before its
    Forgejo runs; it ends when done (setup.md, 6.3);
  - `temper forge run`: checks the binary and the preparation, takes
    the forge's lock, and replaces itself by `forgejo web` or
    `forgejo-runner daemon`, whose process a host supervises (setup.md,
    6.4);
  - `temper forge reconcile`: a managed or owned forge's reconcile, the
    only command that reads an admin's token; it ends when converged
    (setup.md, section 5);
  - `temper box`: the configuration's steps as child processes, in
    order, restarted when they exit and stopped in reverse order; its
    stdout is `temper serve`'s, its stderr each step's, line by line,
    prefixed by the step's name (setup.md, section 7).
- **Exit status:** 0 after a drained shutdown or a finished command; 1 on
  any failure (a refused start, a usage error, a failed commit, an
  abort, a failed sign-in, a failed step of the setup, the box giving
  up), the reason on stderr. smith's agent shell exits 0 or 1 the same
  way; `temper forge run`'s is Forgejo's or the runner's own.

Standalone, one person's machine:

```
temper serve CONFIG.json               one process, one loop: temper-service's system module
├── the engine's service               section 6
│   ├── listens: the in-process name "workers"; the clients' listener and, when configured, the webhooks' listener, TCP on loopback
│   └── connects: each configured forge's API (HTTP; TLS unless the forge is on loopback)
├── the worker's service               section 7
│   └── dials: the in-process name "workers"
├── temper agent AGENT.json            a child process per run, in a process group of its own
└── git ...                              a child process per git invocation, in a process group of its own (git.md, section 7)
temper open DIRECTORY LINK_OR_ADDRESS  a process of its own, reaching the engine on loopback
temper sign-in CONFIG.json ACCOUNT     a process of its own, run by hand
```

A box, one arrangement of the forge's setup's commands, here for a
managed forge 1 and an owned forge 2 (setup.md, 7.1); `temper serve`'s
tree is the one above:

```
temper box CONFIG.json                    a process of its own: each step a child, in a process group of its own
├── temper forge prepare CONFIG.json 2    once, per owned forge
├── forgejo web                           long: temper forge run CONFIG.json 2 forgejo
├── forgejo-runner daemon                 long: temper forge run CONFIG.json 2 runner
├── temper forge reconcile CONFIG.json 1  once, per managed or owned forge
├── temper forge reconcile CONFIG.json 2
└── temper serve CONFIG.json              long: its own tree, above
```

## 5. Components and crates

**jig's,** at temper's pin, composed as they are (jig's
`protocol/README.md`, section 5):

| Crate | Kind | In | What temper uses it for |
|---|---|---|---|
| `jig-protocol-config` | parsing | engine, worker | jig's parts of the configuration (configuration.md, section 2) |
| `jig-protocol-store` | component, codecs | engine | the store, temper's families beside jig's in the same commits (store/README.md) |
| `jig-protocol-worker-wire` | codecs | engine, worker | the channel's records; temper's parts ride in them as bytes (channel.md) |
| `jig-protocol-engine-workers` | component | engine | the worker's link, the engine's side: opening, the payload version, grants' values filled in |
| `jig-protocol-worker-link` | component | worker | the worker's link, the worker's side: dialling, grants' values taken out |
| `jig-protocol-accounts` | translation, table | engine | LLM accounts on skein's accounts driver; the access tokens lent (credentials.md, section 3) |
| `jig-client-protocol-wire` | codecs | engine, client | jig's records for clients; the forge's client family's records ride in them as bytes (client.md) |
| `jig-protocol-engine-clients` | component | engine | the clients' component: jig's messages with their credentials, the printed link's secret, sign-ins, credentials (client.md, section 8) |
| `jig-protocol-engine-clients-http` | mapping | engine | jig's default HTTP mapping, its engine end, with its example guards, mounted on temper's clients' listener under `/jig` |
| `jig-client` | client | client | the client temper's client root holds: the link, watches by handle, the request table, kept work, families; jig's helpers (client.md, section 3) |
| `jig-client-protocol-http` | mapping | client | jig's default HTTP mapping, its client end, under `jig-client-shell-native` |
| `jig-protocol-transcript` | decoding | client | turns' bytes, decoded below `jig-client` |
| `jig-client-shell-native` | shell | client | `temper open`'s transport: HTTP on io, the credential `jig-client` holds sent as a bearer, the small store's file in its directory |

temper does not use `jig-protocol-charter` or `jig-protocol-tools`: its
root takes its own `Tool` vocabulary, translated by `temper-engine-smith`
(llm.md, sections 4 and 5). Nor does it use jig's reference UI
(`jig-client-view`, `jig-client-shell-web`): temper's pages are its own,
on skein's UI kit (client.md, section 5).

**smith's:** `smith-host-domain`, the worker root's agent child;
`smith-host-protocol`, smith's host half, one channel per agent process
over its pipes (smith's `protocol/hosts.md`, sections 2 and 3);
`smith-protocol-channel`, `smith-channel`, `smith-charter` and
`smith-transcript`, smith's codecs, which the routing calls to decode and
encode smith's payloads; `smith-agent-shell`, the `agent` command; and
`smith-local-service`'s profile, from which `temper serve` reads the
agent's host limits at start, as jig's reference binary does.

**skein's:** `skein-lib` and `skein-io` everywhere, lib's HMAC-SHA256
checking the deliveries' signatures (skein's `lib.md`, 6.2), io's
processes under the box; `skein-shell`, the binary's startup kit
(`read_file`, `read_private_file`, seeds, `Host`, `drive`);
`skein-json`, configuration and Forgejo's documents; `skein-http`, the
forge component's client and its webhooks' listener's server,
reconcile's client, and the fake forge's server;
`skein-tls`, a forge's TLS; `skein-codec` and its generator, every codec
crate; `skein-kv`, under jig's store and temper's key encoders;
`skein-channel`, under jig's channel; `skein-oauth-accounts`, under
jig's accounts and `temper sign-in`; `skein-ui`, under temper's view, and
`skein-ui-text`, the terminal's text shell. In the worlds: skein's
simulator, fake machine, fake LLM peer, fake issuer and fake person.

**temper's own,** each step code under the subset but the binary, the
terminal's host, `temper-forge-setup-shell` and the box's host:

| Crate | Kind | Holds | Depends on |
|---|---|---|---|
| `crates/temper-service` | services | the engine's and the worker's services and their one-process composition, in an engine, a worker and a system module, as jig's reference service is: each `iterate`, the routing, the clients' listener with jig's clients' component and its default HTTP mapping mounted, startup from configuration's bytes, start and end; temper's profile `small` | the roots, `temper-engine-smith`, jig's components, smith's host half and codecs, temper's codec crates and components, `skein-io`, `skein-json` |
| `crates/temper` | binary | the commands of section 4, copied from jig's reference binary; for each that reads the document, `temper-service`'s startup, with `setup` and the setup's and the box's limits checked by their own crates (configuration.md, section 2) | `temper-service`, `skein-shell`, `smith-agent-shell`, `smith-local-service`'s profile, `skein-oauth-accounts`, `jig-client-shell-native`, the client crates, `temper-forge-setup-shell`, `temper-box` |
| `crates/temper-forge-forgejo` | documents | Forgejo's documents, both sides: each request's method, path, query and JSON body as a sized encoder; each answer's decoder over skein-json's tokens; error bodies; the deliveries' headers and payloads' decoders (forge.md, section 5); the setup's documents, admin users and their tokens, orgs, teams and their members, org hooks; the pin (setup.md, 6.1). No temper domain type | `skein-lib`, `skein-json` |
| `crates/temper-engine-protocol-forge` | component | the forge component: one per deployment, serving every configured forge; calls in flight, connections, each `Read` and `Write` as a plan of one or a few exchanges, costs, failures, the caches of repository paths and users; the webhooks' listener, each delivery verified as it streams and made a hint (forge.md) | `skein-lib`, `skein-io`, `skein-http`, `skein-tls`, `skein-json`, `temper-forge-forgejo`, the forge client's vocabulary |
| `crates/temper-forge-setup` | step code | the forge's setup: its types and checks; reconcile's machine; the preparation's machine; `app.ini` and the runner's files; the processes' specs; the configuration's steps, `steps` and `Step`. Names no host (setup.md, section 11) | `temper-forge-forgejo`, `jig-protocol-config`, `skein-lib`, `skein-json` |
| `crates/temper-forge-setup-shell` | hosts | ordinary Rust: the hosts of `temper forge prepare`, `run` and `reconcile`: commands, files, the forge's locks and the exec, in one module on std's (setup.md, 6.4); HTTP on skein-http over skein-io | `temper-forge-setup`, `temper-forge-forgejo`, `skein-io`, `skein-http`, `skein-tls`, `skein-json`, `skein-shell` |
| `crates/temper-box` | machine, host | the box's machine, step code, and its host, ordinary Rust, on skein-io's processes; nothing of the setup but its steps (setup.md, 7.3) | `temper-forge-setup`'s `steps` and `Step` alone, `skein-lib`, `skein-io`, `skein-shell` |
| `crates/temper-worker-protocol-git` | component | the git component: each `git::Op` of `temper-worker-domain-checkout` as a fixed sequence of git invocations, their arguments, environment, output and failures; the operations in flight, their processes, pipes and deadlines; `Make` as file operations (git.md) | `skein-lib`, `skein-io`, the checkout's vocabulary |
| `crates/temper-protocol-worker-wire` | codecs | temper's parts on jig's channel, and a git identity's value, from a schema; `PAYLOAD_VERSION` (channel.md) | `skein-lib`, `skein-codec` |
| `crates/temper-protocol-store` | codecs, keys | temper's families: key encoders, codecs from a schema, `STORE_VERSION`, the restart's interval (store/README.md) | `skein-lib`, `skein-codec`, `skein-kv`, `jig-protocol-store`, the forge's records |
| `crates/temper-client-protocol-wire` | codecs | the forge's client family's records: the adoption request and its outcome, the small store's record of a pending one; `FORGE`, the forge connector's number, 1; `Owner`, temper's owners; `CLIENT_VERSION` (client.md, 4.6) | `skein-lib`, `skein-codec` |
| `crates/temper-engine-smith` | translation | kept: smith's start from an assignment, host calls decoded into the root's `Tool` vocabulary, answers rendered, results and turns read; called by the engine's routing (llm.md) | as it is |
| `crates/temper-client-domain` | client root | `jig-client` held, temper's pages, the forge's client side (client.md, section 3) | client.md, section 2 |
| `crates/temper-client-view` | view | temper's pages as skein-ui's tree, with the forge's view (client.md, section 5) | client.md, section 2 |
| `crates/temper-client-shell-terminal` | shell | the terminal: its host ordinary Rust, over skein's text shell and jig's native transport (client.md, section 6) | client.md, section 2 |

**temper's test crates and tiers** (testing.md, sections 2 and 7):
`testing/temper-fake-forge-protocol`, the fake forge's server side over
skein's HTTP server, counting requests by route, serving the setup's
routes too and sending deliveries (forge.md, section 9; setup.md,
section 11); reconcile's world, on it (setup.md, 13.2);
`testing/temper-fake-git`, the fake git program on skein's fake machine
(testing.md, 4.3); the worlds `tests/engine/protocol-forge`,
`tests/worker/protocol-git`, `tests/client` (`temper-client-world`) and
`tests/one-process`; the
process and live tiers in `crates/temper/tests/process.rs` and
`crates/temper/tests/live.rs`, under nextest's `process` and `live`
profiles, run by `scripts/proof-one-process.sh` and
`scripts/proof-live.sh`; `tools/forgejo-conformance`, which captures
Forgejo's transcripts from the throwaway Forgejo (forge.md, section 10);
and `tools/git-transcripts`, which captures git's from real git against
the throwaway Forgejo (git.md, section 9).

**Kept:** the domains' crates (`temper-engine-domain` and its forge
children, `temper-worker-domain`, `temper-worker-domain-checkout`), the
fake forge's domain, and the domains' worlds, with the changes this
design owns:

- the forge client's locate (forge.md, 3.2; `domain/forge.md`,
  section 4);
- the adoption as the forge's own keyed request, which the root takes
  typed from the routing, locates, then makes as jig's adoption request
  (client.md, section 8; `domain/forge.md`, section 4);
- the root's `Parts` carrying a failed preparation (channel.md,
  section 5; `domain/engine.md`, section 8);
- the worker's parts going up naming each repository by its tag, each
  saved repository saying whether it started from saved work, and a
  failed preparation naming its repository's tag (channel.md, section 5;
  `domain/worker.md`, section 9);
- the root relaying, with no commit and no record, the error answer to
  a host call that did not decode (llm.md, section 4; `domain/engine.md`,
  7.3).

**Retired,** in the plan that lands `temper-client-domain` and its
world: `temper-web-domain`, `temper-web-view`,
`testing/temper-fake-person`, `tests/web/domain` (client.md, section 9).

**What temper's components share,** as every component of jig's does
(jig's `protocol/README.md`, section 5):

- **Children of the protocol layer,** each with its own limits, worst
  case, deadlines and most outputs per entry point, which the service
  reports and fires (programming-model.md, section 9).
- **Connections are the component's own entities** (programming-model.md,
  4.1): a socket or a pipe below, a connection or an invocation here, a
  call or an operation above, bound by echoed tokens (4.2).
- **One exchange in flight per connection,** with an output cap and a
  room check before the next is parsed: a peer that stops reading stops
  being read from.
- **Refusals at the entrance.** A call with no room is refused before
  anything is set aside for it.
- **Deadlines.** The domain gives each call its deadline; each connection
  or invocation arms its own from its limits. Machines keep none
  (programming-model.md, section 9).
- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines: for the forge,
  unavailable only when the call provably never left, and timed out
  otherwise (forge.md, 3.4). A retry below the domain hides its cost from
  the budget, and adds load just when the peer is struggling.
- **Limits and the worst case.** Each component exports its `Limits` and
  `worst_case`, summed with jig's at start (configuration.md, section 7).

## 6. The engine's service

In the shape of jig's `protocol/service.md`, section 2:

```
temper-service, engine module           iterate: io ↔ components ↔ root
├── temper-engine-domain                the root, with the journal (domain/engine.md)
└── the protocol layer
    ├── jig-protocol-store              commits, loads, transcripts, secrets; temper's families (store/README.md)
    ├── jig-protocol-engine-workers     the worker's link; temper's parts as bytes (channel.md)
    ├── jig-protocol-accounts           LLM accounts (credentials.md, section 3)
    │   └── skein-oauth-accounts
    ├── the clients' listener           temper's: TCP on loopback (client.md, section 8)
    │   └── jig-protocol-engine-clients-http   jig's default HTTP mapping, under /jig, with its example guards
    ├── jig-protocol-engine-clients     the clients' component: jig's messages; the forge's client family as bytes (client.md, section 8)
    ├── temper-engine-protocol-forge    the forge component: every configured forge's API; the webhooks' listener, TCP on loopback (forge.md)
    ├── the identities' table           git identities' values (credentials.md, section 4)
    └── timers                          the root's and every component's deadlines
```

Starting, ending, `iterate`'s passes, room and timers are jig's
(jig's `protocol/service.md`, sections 3 to 5). What temper adds:

- **The forge component,** speaking Forgejo v17.0's REST API under
  `/api/v1`, polled at the domain's cadences (forge.md, section 3). The
  root's `Request::Forge { call, repository, op }` goes to it; it
  answers each call with exactly one
  `client::Event::Answered { call, cost, result }`, which the routing
  hands the root as a forge event. An event the root cannot take yet
  comes back to the component, which holds it and offers it again (jig's
  `protocol/service.md`, section 2, room); one the root returns during
  its drain stays with the component until it closes (forge.md, section
  2). The API tokens are the component's alone (credentials.md, section
  5).
- **A forge's start check.** The component's first exchange with each
  forge checks the API token's user; another user, or a refusal of the
  token, stops the engine: the service drains as at a shutdown, and
  `temper serve` exits 1, naming the forge (credentials.md, section 5;
  forge.md, 3.5).
- **The webhooks' listener,** the forge component's own, when
  `application.webhooks` names it: plaintext HTTP on loopback, one path
  per forge with a `webhook_secret`; each delivery verified as it
  streams, answered, then made a hint, which the routing hands the root
  as a forge event (forge.md, sections 2 and 5). A hint is never a fact:
  polling finds what a delivery lost told of (forge.md, section 6).
- **temper's parts.** As it encodes an assignment, the routing encodes
  the root's `ForgeWorkspace` as temper's workspace items, with
  `temper-protocol-worker-wire`, making each repository's remote from its
  forge's `git` base and the repository's owner and name
  (configuration.md, section 3), so workers need no forge configuration.
  As it decodes an answer, it decodes the work and the preparation into
  the root's `Parts` (channel.md, sections 4, 5 and 7).
- **temper's families.** The root's forge records go to the store under
  jig's application tag, encoded by `temper-protocol-store`, in the same
  commits as jig's; the forge's restart step loads them as one interval
  (store/README.md).
- **The identities' table,** filled from configuration at start; the
  routing gives the root one `Event::Credential` per identity before its
  restart, and `jig-protocol-engine-workers` fills grants' values from the
  table through the routing (credentials.md, section 4).
- **Host tools, the charter and results through `temper-engine-smith`,**
  called in the routing, with smith's codecs, in the step that routes
  them, keeping no state (llm.md, sections 4 and 5).
- **The clients' listener.** The service owns it, and mounts jig's
  clients' component on it with jig's default HTTP mapping under `/jig`,
  with the mapping's example guards (client.md, section 8; jig's
  `protocol/service.md`, section 2); every other path, and a `GET` of
  the printed link's route, is answered not found.
- **The client's application wire.** The forge's client family's
  requests are decoded by `temper-client-protocol-wire` in the routing
  and answered by the root; every other part of the forge's is answered
  as unknown (client.md, sections 4 and 8). An adoption names a forge, an
  owner and a name; the routing completes it with the forge's host, the
  host of its configured `git` base with `:port` when it names one
  (configuration.md, section 3), and hands the root the party's keyed
  request; the root locates the repository, then makes jig's adoption
  request on the party's behalf (`domain/forge.md`, section 4).
- **Periods,** from the smallest source, as jig's reference service gives
  them (configuration.md, section 8).

## 7. The worker's service

In the shape of jig's `protocol/service.md`, section 6:

```
temper-service, worker module           iterate
├── temper-worker-domain                the worker's root: jig's hub, smith's host domain, the checkout
└── the protocol layer
    ├── jig-protocol-worker-link        the link to the engine; temper's parts as bytes (channel.md)
    ├── smith-host-protocol             one channel per agent process, over its pipes; LLM grants filled into its start
    ├── agent processes                 temper agent, each in a process group of its own
    ├── the identities' table           git identities' values, from grants (credentials.md, section 4)
    └── temper-worker-protocol-git      the git component: the checkout's io (git.md)
```

No store; starting, ending and smith's payloads are jig's (jig's
`protocol/service.md`, section 6). What temper adds:

- **Agents run as `temper agent`:** the agent program is temper's own
  binary, with the arguments configuration gives (configuration.md,
  section 5), its environment holding no credential.
- **temper's parts.** The routing decodes the workspace items into the
  root's `wire::Workspace` before the root sees the assignment, and
  encodes the root's `wire::Work` and `wire::Preparation` beside an
  answer (channel.md, sections 4, 5 and 7).
- **Credentials.** `jig-protocol-worker-link` takes grants' values out:
  LLM accounts' go to smith's host half in its form (jig's
  `protocol/llm.md`, 5.3); git identities' go into the identities' table,
  from which the routing hands the git component an identity's value
  with each operation that names it; the component never reads the table
  (git.md, sections 2 and 5).
- **The checkout's io.** Each `git::Op` the root asks for goes to the git
  component, which answers it with exactly one terminal; `Make` is file
  operations under the worker's root directory (git.md, section 4).
- **Starting:** configuration, the components, the worker's root
  directory opened, then the link dialled at once (jig's
  `protocol/service.md`, section 6).

## 8. One process

Standalone is jig's one process (jig's `protocol/service.md`, section 10):

- **One service, one loop.** `temper-service`'s system module owns the
  engine's protocol layer and the worker's as children, each with its own
  root and journal, sharing io and nothing else; nothing crosses between
  them but bytes.
- **Joined by the in-process name `workers`:** the engine's workers'
  listener and the worker's link (configuration.md, section 2). The
  link's opening, authentication, liveness, relays and grants run as
  between hosts, in plaintext, with no TLS.
- **A client on loopback,** over TCP to the clients' listener: `temper
  open`, signing in by the link `temper serve` prints at start.
- **Agents and git as child processes,** each in a process group of its
  own, until skein's contained process trees exist (jig's
  `protocol/README.md`, section 10).
- **The forge outside,** reached over HTTP, and over TLS when it is not
  on loopback; its deliveries reach the webhooks' listener on loopback,
  so a forge on another host polls alone (forge.md, section 5).
- **Starting and ending together:** configuration for both, then the
  engine's start, then the worker's. A signal drains the engine; once it
  has drained, the service ends the worker at once, its agent processes'
  and git invocations' groups killed as io closes them. The next start
  settles the worker's claims at once (jig's `domain/engine.md`, section
  6).
- **Its world** is `tests/one-process` (testing.md, 2.5).

## 9. Versions

jig's rules hold (jig's `protocol/README.md`, section 7): one version at a
time before release, a reader that refuses another and says so, and an
upgrade that stops the engine and its workers together. temper's three
versions sit beside jig's:

| Version | Owner | Covers | Checked |
|---|---|---|---|
| jig's channel version | jig | jig's records on the channel | at the link's opening (jig's `protocol/channel.md`, 4.1) |
| `PAYLOAD_VERSION` | `temper-protocol-worker-wire` | temper's parts: the workspace items, the saved-work branch, the work, the preparation; a git identity's value; and a delivery's evidence, smith's record as temper's pinned smith encodes it | the worker's credential at the link's opening: another is refused as *application* (channel.md, section 3) |
| smith's charter, transcript and result versions | smith | smith's payloads | by smith's codecs (smith's `protocol/README.md`, section 8) |
| jig's store format and families' versions | jig | jig's records | at the store's opening (jig's `protocol/store.md`, 5.3) |
| `STORE_VERSION` | `temper-protocol-store` | temper's families | at the store's opening: a store written by another refuses to open (store/README.md, section 4) |
| jig's client versions | jig | jig's documents and requests | per owner, a range served; another answered with jig's typed refusal, naming the ranges served (jig's `protocol/client.md`, section 9) |
| `CLIENT_VERSION` | `temper-client-protocol-wire` | the forge's client family's records | per owner, as jig's: the engine serves exactly its own, and `temper open`, from the same binary, speaks it (client.md, 4.6) |
| keys' markers | the forge connector | the keys temper writes into forge bodies | the key's version, its first byte; the marker has none of its own (forge.md, section 4) |
| configuration | none | — | unknown members refuse the start (configuration.md, section 2) |

- **Each of temper's three is 1 at first** and moves at most once per
  plan, at the first landing in that plan that changes one of the
  encodings it covers; for `PAYLOAD_VERSION`, a repin of smith that
  changes a delivery's record counts (channel.md, section 3).
- **Both channel components are given `PAYLOAD_VERSION`** when they are
  made; neither knows what it covers.

## 10. Secrets

| Secret | Owner | Held by | Kept | Reaches |
|---|---|---|---|---|
| an LLM account's record | jig | skein's accounts driver; the access tokens lent, jig's accounts' table | the driver's private files, one per account (jig's `protocol/llm.md`, 5.4) | the refresh token, nobody; the access token, the worker's agent processes, through credential grants and smith's host half |
| a git identity: its token, login and author | temper | the identities' table on the engine; the worker's identities' table, from grants | a private token file per identity, read at start (configuration.md, section 6) | git, in each invocation's environment only (git.md, section 5) |
| a forge's API token | temper | the forge component | a private file per forge, read at start | that forge's API, in each request's `Authorization` header |
| a worker's secret | jig | the worker; its digest, the engine | configuration's files | the link's opening, over the in-process name |
| a sign-in's secret | jig | `jig-client` in `temper open`, which keeps it in its directory; its digest, the engine | jig's secrets' store | each of the client's exchanges, as a bearer token (client.md, section 7) |
| a forge's webhooks' secret | temper | the forge component's webhooks' listener | a private file per forge, read at start (configuration.md, section 6) | the signature check of each delivery to that forge; the forge's hooks, given it by reconcile or the person (setup.md, sections 4 and 5) |

- **Names in the domain, values beside it** (jig's `protocol/README.md`,
  section 8): no domain state, fact, record of the main store or log
  holds a value. Values move between components through the routing,
  never through a root.
- **No temper secret in jig's secrets' store** in standalone: every
  temper secret is a file read at start. jig keeps sign-ins' and
  services' digests there.
- **Nothing crosses a host** in standalone: the link is in process, the
  client and the webhooks on loopback, and a forge off the machine is
  reached over TLS only (configuration.md, section 3).
- **The forge's setup's secrets** (a site admin's token, the passwords,
  the runner's secret, Forgejo's own: setup.md, section 9) are the
  setup's commands' alone, kept in the forge's directory or nowhere;
  `temper serve` reads none of them.

## 11. What temper uses from jig, smith and skein

temper's protocol layer uses jig, smith and skein as they are, at
temper's pinned revisions (jig `d91a89c4`, smith `dfc0fe57`, skein
`b5d3a2c3`). One thing in this design is a prerequisite upstream: the
webhooks' HMAC-SHA256 and its comparison in constant time, `sha256::Hmac`
and `sha256::equal` in skein's lib (skein's `lib.md`, 6.2; setup.md,
section 14).

- **jig:** the components of section 5; the reference service and binary,
  copied; `jig-client` and its native transport; jig's client test kit,
  its check, faults and attack stories, for temper's client tiers; the
  conformance kit, as the domain's worlds use it.
- **smith:** the agent shell, the host domain and host half, the codecs;
  the agent process over skein's LLM client.
- **skein:** io with its in-process names, processes and process groups;
  the startup kit and its private reads; lib's SHA-256 and its HMAC;
  JSON; HTTP's client and server; TLS's client; the codec generator; the
  key-value store; the framed channel; the accounts driver and its
  sign-in; the UI kit and its text shell; the simulator, the fake
  machine, the fake LLM peer, the fake issuer and the fake person.
- **What is temper's own:** its presentation, since presentation is the
  application's (jig's `client/domain/client.md`), on skein's UI kit and
  text shell (client.md, sections 5 and 6); and, where something would
  be better upstream, as jig's design allows, `temper sign-in`, copied
  from smith's local shell's (smith's `shell.md`, 4.5), and the fake git
  program, on skein's fake machine.
- **Not used:** `jig-protocol-charter` and `jig-protocol-tools` (llm.md,
  section 7); agents in a host's own process (`smith-inline-agent`);
  jig's reference UI (section 5).

## 12. Order of building

The order a plan can build in, each step proved where the step names.
The first end-to-end story is step 5: a chat answered by the real
binary.

1. **Codec crates:** `temper-protocol-worker-wire`,
   `temper-protocol-store`, `temper-client-protocol-wire`, and
   `temper-forge-forgejo`'s documents, with Forgejo's decoders fed the
   transcripts `tools/forgejo-conformance` records by plain requests to
   the throwaway Forgejo (forge.md, section 10). Proved by step tests:
   golden bytes, bounds, fuzz targets (testing.md, section 2).
2. **`temper-service` on jig's reference:** the engine (the store with
   temper's families, accounts, the workers' side, the clients'
   listener with jig's clients' component and its default HTTP mapping
   mounted, the identities' table, host tools through
   `temper-engine-smith`); the
   worker (the link, smith's host half, the git component); one process.
   Until step 6 the routing answers every forge call unavailable, since
   it provably never left. Proved by the routing's step tests, git's
   builders and parsers against the transcripts of
   `tools/git-transcripts`, and the git component's machine world,
   `tests/worker/protocol-git` (git.md, section 9).
3. **The binary's `serve` and `agent`,** copied from jig's reference
   binary, with configuration's startup (configuration.md).
4. **The client root and view,** `temper-client-domain` on
   `jig-client` and `temper-client-view` on skein's UI kit: temper's
   pages and the forge's words, joined to the engine's root in
   `tests/client` from their first types (client.md, section 10;
   testing.md, 6.3); and the terminal's host,
   `temper-client-shell-terminal`'s `host` module (client.md, section 2),
   through which the process tier's fake person reads their tree.
5. **A chat answered end to end, in the process tier:** the real binary
   on a scratch directory, skein's fake LLM peer on loopback, skein's
   fake person through temper's client, signing in by the printed
   link and asking a chat that names no repository. A run with no
   repository has no workspace (`domain/worker.md`, 4.1), so it needs no
   git and no forge; the configuration names one forge, which no call
   reaches.
6. **The forge component:** its machine world against a scripted HTTP
   peer; then against the fake forge's protocol layer in
   `tests/engine/protocol-forge`, with locate and the cost checks by
   HTTP request; then against the throwaway Forgejo, driven by
   `tools/forgejo-conformance` (forge.md, sections 9, 10 and 13).
7. **The adoption form,** a page of temper's, linked from its
   navigation; the forge's client family on both sides; and the root's
   adoption request (client.md, 4.3, 5.3 and section 8;
   `domain/forge.md`, section 4), in `tests/client`.
8. **A change landed, in the process tier:** a repository adopted through
   temper's client, a goal whose change lands on the throwaway Forgejo
   with real git, the binary killed and restarted, the work found
   (testing.md, 2.6); owned mode takes the Forgejo's place in the forge's
   setup's order (setup.md, section 14).
9. **The terminal shell** and `temper open` (client.md, section 6):
   temper's pages as skein's text shell prints them, fixed by golden
   text; its host, by its machine world; and `temper open` itself, stdin
   and stdout piped, in the process tier (client.md, section 10;
   testing.md, 2.6).
10. **`temper sign-in`,** against skein's fake issuer (credentials.md,
    section 3).
11. **The one-process world,** `tests/one-process`, with the fake git
    program and the fake forge as a service (testing.md, 2.5).
12. **The live tier,** `crates/temper/tests/live.rs` (testing.md, 2.7).
13. **Retiring `temper-web-*`,** `testing/temper-fake-person` and
    `tests/web/domain`, in the same plan as step 4, at its end at the
    latest (client.md, section 9).

**The forge's setup,** after the steps above, in setup.md, section 14's
order: Forgejo's facts; skein lib's HMAC-SHA256; the webhooks, on
loopback (forge.md, section 5); owned's preparation; reconcile; `temper
box`; the owned example (configuration.md, section 10).

**After standalone,** designed but not built first, each waiting for
what its document names: webhooks from another host, over TLS (forge.md,
section 5); hosts apart: TLS for the engine's listeners, contained
process trees, a process with a worker only (jig's `protocol/README.md`,
section 10); the forge as people's sign-in provider (credentials.md,
section 6); the forge's client documents and panels (client.md, 4.2 and
section 11); the browser (web/architecture.md); and what the forge's
setup leaves for later (setup.md, section 15).

## 13. Testing

As testing.md applies skein's strategy (testing-strategy.md):

- **Step tests:** every codec (golden bytes, bounds, fuzz targets), each
  translation both ways, git's argument and environment builders, git's
  output parsers against transcripts of real git (recorded by
  `tools/git-transcripts`), Forgejo's decoders against transcripts of
  the throwaway Forgejo (recorded by `tools/forgejo-conformance`),
  temper's pages as skein's text shell prints them (testing.md, section
  2; client.md, section 10).
- **Machine worlds:** the forge component against a scripted HTTP peer;
  the git component against scripted process ends and outputs; the
  terminal's host against a scripted stdin and stdout (testing.md,
  sections 2 and 7).
- **The forge's protocol world,** `tests/engine/protocol-forge`: the
  forge component against the fake forge's protocol layer, over
  in-memory streams cut at random, the forge connector world's stories
  through bytes, and the cost checks by HTTP request (testing.md, 2.2).
- **The client's joined world,** `tests/client`, a system world:
  temper's client root and view with skein's fake person and jig's
  check, joined to temper's engine root from the first increment
  (testing.md, 2.1 and 6.3).
- **The one-process world,** `tests/one-process`, temper's simulated
  world: `temper-service` on skein's simulator, killed and restarted at
  drawn moments, drained by a shutdown (testing.md, 2.5).
- **Reconcile's world:** reconcile against the fake forge's protocol
  layer, serving the setup's routes (setup.md, 13.2).
- **The process tier** and **the live tier,** end to end, outside the
  two suites, under nextest's `process` and `live` profiles, run by
  `scripts/proof-one-process.sh` and `scripts/proof-live.sh` (testing.md,
  2.6, 2.7 and 6.1). The process tier runs an owned setup's commands,
  with CI and webhooks, the hinted story and `temper box`'s test
  (setup.md, 13.4).
- **Budgets:** the suites' 15 s and 60 s are rules, which nextest
  enforces. Within them, guides: the worlds below the domain (the
  protocol, machine, client and one-process worlds) together at most
  4 s of the default suite and 25 s of the fuzzy suite; a focused
  protocol-world test at most 0.5 s, and a one-process story at most
  1 s. A test that misses its guide is made cheaper or moved to the
  fuzzy suite, never merged with another (testing.md, 6.1). The process
  tier aims at 3 minutes (testing.md, 2.6); the live tier makes at most
  25 completions in at most 15 minutes in all, and is stopped past
  either (testing.md, 2.7).

## 14. Open questions

Each document keeps its own; these cross them:

- **Names while running.** io connects to addresses, resolved at
  startup, while a forge's address may move behind its CDN. Default: a
  restart resolves names again; skein's DNS client would mend it.
- **HTTP/1.1 only.** Forgejo serves it; a forge that required HTTP/2
  would need skein to grow it. Default: HTTP/1.1.
- **Generating the routing** (jig's `protocol/service.md`, section 9).
  Default: copied from jig's reference service and filled.
- **Supervision** (jig's `protocol/service.md`, section 9): what
  restarts `temper serve` after a failed commit or a crash. Default:
  nothing of `temper serve`'s own; a box restarts it, after its
  reconciles (setup.md, 7.2), and otherwise the person or the machine's
  service manager starts it again, and it recovers from the last
  durable commit.
- **Agents in the worker's process** (`smith-inline-agent`), with no
  containment, for one person's machine (jig's `protocol/service.md`,
  section 10). Default: agent processes, as `temper agent`.
