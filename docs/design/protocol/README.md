# The protocol layer

Provisional, 2026-10-03, revised 2026-10-20. temper's protocol layer:
how temper composes jig's protocol layer with components of its own (its
forge's, its checkout's, its client's) into the processes of one binary.
jig owns the layer's shape (jig's `protocol/README.md`) and the services'
(jig's `protocol/service.md`); this document says what temper adds and
the crates it is made of. The mechanics are those of skein's
`programming-model.md` (sections 2, 4 and 9); the entities it carries are
those of `../domain/`. Each boundary of temper's own is designed in a
document of its own (section 2). What is open across them is listed in
section 14.

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
- **What temper adds:** the forge's protocol (skein's forge component,
  with its webhooks' listener, handed temper's values and routed to the
  root and the forge connector); the checkout's io
  (the git component); its parts' codecs on jig's channel; its families
  in jig's store; its git identities; its host tools' translation; its
  client (its wire, its client root and view on `jig-client`, its
  terminal); the clients' listener, on which it mounts jig's clients'
  component and serves its forge routes; the setup host, skein's, in the
  engine (forge.md, 2.6); and the forge's setup's commands outside the
  services (setup.md). Each is a crate of its own (section 5).
- **Foreign protocols are stacks of skein's machines.** Forgejo's REST
  API and GitHub's are skein's forge kit, each a dialect of it, over
  skein's HTTP client and JSON (skein's `forge.md`); git's arguments in and text out are temper's decoders
  over io's processes. temper's records on jig's channel, in jig's store
  and on the client's wire are generated from schemas by skein's
  generator.
- **Standalone first.** `temper serve` runs the engine and one worker in
  one process, on one loop, joined by jig's in-process address; the
  worker's agents are child processes of temper's own binary (`temper
  agent`); a person's client reaches the engine on loopback; the forge is
  polled, and hinted by its webhooks on loopback when they are
  configured (section 8; forge.md, section 5). Hosts apart, webhooks from
  another host, the forge as a sign-in provider and the forge's client
  documents come after (section 14). The browser is a person's client
  on loopback too (`../client/browser.md`).
- **One binary:** `serve`, `agent`, `open`, `program-login`, the forge's
  setup's `forge prepare`, `forge run` and `box`, and
  the agents' `ctl`, which `../client/tools.md` designs (section 4).
- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines (section 5).
- **The forge's load follows its changes.** The calls a pass makes grow
  with what changed on the forge, never with its history, its labels or
  the items temper holds, and the worlds count them by HTTP request
  (forge.md, section 6).
- **Secrets never reach a domain.** A domain names a credential by number
  and generation; values live in the protocol layer's tables and in jig's
  secrets' store: LLM accounts' as jig keeps them, and each forge's
  records under temper's tag, from which its access and its git identity
  are made (section 10; credentials.md).
- **Tested at every tier:** codecs and parsers against golden bytes and
  transcripts of real peers; the forge component and the git component in
  worlds of their own; temper's client joined to the engine's root; the
  one-process world on skein's simulator; the real binary with an owned
  Forgejo, its runner and real git; a live tier with a real model
  (section 13; `../domain/testing.md`).

## 2. Reading order

1. **jig's `protocol/README.md` and `protocol/service.md`:** the layer
   temper composes, and the services temper copies.
2. **README.md:** this overview.
3. **configuration.md:** what `temper serve` and its other commands are
   told at start.
4. **credentials.md:** the secrets temper's processes hold: LLM accounts
   as jig's, the forges' records in jig's secrets' store, the forges'
   access, git identities, and the values clients' requests carry.
5. **channel.md:** temper's parts on jig's channel, and a git identity's
   value.
6. **store.md:** temper's families in jig's store.
7. **llm.md:** where LLM calls are made, host tools, the charter and a
   run's result.
8. **git.md:** the checkout's io: the git component.
9. **forge.md:** the forge's protocol layer on skein's forge kit: what
   temper hands the forge component, the setup host, its marks, its
   webhooks' address, and what it relies on of Forgejo and GitHub.
10. **setup.md:** the forge's setup: connected, managed and owned, the
    setup host in the engine, and `temper box`.
11. **`../client/README.md`:** temper's client: its client root on `jig-client`,
    its pages, the forge's client family, its view, the terminal, and the
    clients' listener of temper's server; then **`../client/tools.md`**, `temper
    ctl`, the agents' tools.
12. **`../domain/testing.md`:** the tiers below the domain.
13. **`../client/browser.md`:** the browser, the second shell of
    temper's client.

## 3. Names

jig's names keep their meaning here (jig's `protocol/README.md`, section
3; jig's `domain/README.md`, section 4): the hub, the core, the fleet,
the worker's link, credential grant, an application's credential,
payload version, profile, the clients' component, the default HTTP
mapping, the reference service (`jig-test-service`), the reference
binary (`jig-test`).

| Name | What it is |
|---|---|
| standalone | one process on one person's machine: `temper serve` with the engine and one worker joined in process (section 8) |
| the engine's service, the worker's service | each `iterate` with its root and protocol layer, in `temper-service` (sections 6 and 7) |
| the routing | the protocol layer's code between io, components and the root (jig's `protocol/service.md`, section 2) |
| the forge component | skein's `skein-forge-connection` (forge.md; skein's `forge-connection.md`) |
| the git component | `temper-worker-protocol-git` (git.md) |
| temper's parts | the workspace items, the saved-work branch, the work, the preparation (channel.md) |
| payload version, store version, client version | temper's three own versions: `PAYLOAD_VERSION`, `STORE_VERSION`, `CLIENT_VERSION` (section 9) |
| temper's families | the forge connector's records in jig's main store (store.md) |
| a forge's records | its credentials, ids and accepted installations: rows of jig's secrets' store under temper's tag (credentials.md, 2.1) |
| git identity | one of temper's application credentials, one per forge: Forgejo's git token, or a GitHub App's installations' tokens, with a login and an author (credentials.md, section 4) |
| the identities' table | the protocol layer's table of git identities' values, on the engine and on each worker |
| a forge's access | what the forge component authenticates with: Forgejo's API token, or a GitHub App's id and key and its installations' tokens; the component's only (credentials.md, section 5) |
| the setup host | skein's in-process host for the setup's plans, a child of the engine's protocol layer (forge.md, 2.6) |
| the forge's client family | temper's client side of the forge connector: its words, and the adoption form (`../client/README.md`, section 4) |
| the terminal shell | `temper-client-shell-terminal` (`../client/README.md`, section 6) |
| the browser shell | `temper-client-shell-web` (`../client/browser.md`, 2.1) |
| the bundle | the browser shell's module, glue and stylesheets, skein's bundler's output, which the clients' listener serves (`../client/browser.md`, 2.2) |
| the tools | the agents' interface to temper, `temper ctl`: `temper-client-tools` and `temper-client-shell-cli` (`../client/tools.md`, section 4) |
| the clients' listener | `temper-service`'s listener for clients, on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig` (`../client/README.md`, section 8) |
| the fake git program | git as the worker runs it, faked: its core `crates/temper-fake-git`, `temper fake git`, and its hosted half `testing/temper-fake-git-hosted` (`../domain/testing.md`, 4.3) |
| the throwaway Forgejo | a Forgejo started by a test script on loopback, with SQLite in a scratch directory, removed after (`../domain/testing.md`, 2.6) |
| locate | the forge connector's read of a repository's id by owner and name (skein's `forge.md`, 5.2) |
| the webhooks' listener | the forge component's own listener for Forgejo's signed deliveries, plaintext on loopback, one path per forge (skein's `forge-connection.md`, section 6; forge.md, section 5) |

The forge's setup's names are setup.md's: connected, managed and owned
(section 2), the forge's directory (section 3), reconcile, a plan of the
setup host's (section 5),
the pin (6.1), the preparation (6.3), the box and its steps (section 7).

## 4. The processes and the binary

One binary, `temper` (`crates/temper`), ordinary Rust: a shell, copied
from jig's reference binary. Its commands, `temper ctl`'s as
`../client/tools.md` designs them:

```
temper serve [DIRECTORIES]
temper init [DIRECTORIES]
temper agent AGENT.json
temper open DIRECTORY LINK_OR_ADDRESS
temper ctl DIRECTORY TOOL [ARGUMENT...] [--json]
temper ctl DIRECTORY --call FILE [--json]
temper ctl help [TOOL]
temper program-login [DIRECTORIES] NAME
temper forge prepare [DIRECTORIES] FORGE
temper forge run [DIRECTORIES] FORGE forgejo|runner
temper box [DIRECTORIES]
```

`DIRECTORIES` is `[--config DIR] [--data DIR] [--cache DIR]`, given
anywhere among the command's arguments, each with its variable and its
default; the document is `temper.json` in the configuration directory
(configuration.md, section 12).

- **`temper serve`:** reads the configuration (configuration.md,
  section 2), opens the store in the deployment's directory, and runs
  the engine's and the worker's services in one process on skein's
  `Host` and `drive` (jig's `protocol/service.md`, sections 4 and 10).
  - Its stdout carries only sign-in links, one per line: each the
    secret jig's clients' component gives, in jig's default mapping's
    link route under temper's prefix (jig's `protocol/client.md`,
    section 8; `../client/README.md`, section 7); its stderr carries startup
    notes and the reason of a failure.
  - A first termination signal drains it (jig's `protocol/service.md`,
    section 5); a second turns the drain into an abort.
- **`temper init`:** writes a deployment that starts, the default
  document among it, into the three directories, refusing when the
  configuration directory holds a document (setup.md, 8.2).
- **`temper agent AGENT.json`:** smith's agent shell,
  `smith_agent_shell::run`, on smith's agent configuration
  (configuration.md, section 5). The worker's protocol layer spawns it,
  one process per run; a person never does.
- **`temper open DIRECTORY LINK_OR_ADDRESS`:** a person's terminal client
  (`../client/README.md`, 6.1). It reads no configuration file.
- **`temper ctl DIRECTORY ...`:** the agents' tools, one call per
  process, over the same native transport as `temper open`
  (`../client/tools.md`, section 8). It reads no configuration file.
- **`temper program-login NAME`:** a program's login for
  the program source NAME, on this host: it checks the document and the
  agent's configuration, makes the source's configuration directory, and
  replaces itself by the program's own sign-in (credentials.md, section
  3). The engine need not run.
- **The forge's setup** (setup.md), each command reading and checking
  the whole document as `temper serve` does (configuration.md, section
  8), FORGE a forge's `number`; reconcile is no command, but a plan the
  engine's setup host runs (forge.md, 2.6):
  - `temper forge prepare`: an owned forge's preparation, before its
    Forgejo runs; it ends when done (setup.md, 6.3);
  - `temper forge run`: checks the binary and the preparation, takes
    the forge's lock, and replaces itself by `forgejo web` or
    `forgejo-runner daemon`, whose process a host supervises (setup.md,
    6.4);
  - `temper box`: the configuration's steps as child processes, in
    order, restarted when they exit and stopped in reverse order; its
    stdout is `temper serve`'s, its stderr each step's, line by line,
    prefixed by the step's name (setup.md, section 7).
- **Exit status:** 0 after a drained shutdown or a finished command; 1 on
  any failure (a refused start, a usage error, a failed commit, an
  abort, a failed step of the setup, the box giving up), the reason on
  stderr. smith's agent shell exits 0 or 1 the same way; `temper ctl`'s
  statuses are `../client/tools.md`, 6.3; `temper forge run`'s is Forgejo's or
  the runner's own, and `temper program-login`'s the program's sign-in's
  once it has replaced itself.

Standalone, one person's machine:

```
temper serve                           one process, one loop: temper-service's system module
├── the engine's service               section 6
│   ├── listens: the in-process name "workers"; the clients' listener and, when configured, the webhooks' listener, TCP on loopback
│   └── connects: each configured forge's API (HTTP; TLS unless the forge is on loopback)
├── the worker's service               section 7
│   └── dials: the in-process name "workers"
├── temper agent AGENT.json            a child process per run, in a process group of its own
└── git ...                              a child process per git invocation, in a process group of its own (git.md, section 7)
temper open DIRECTORY LINK_OR_ADDRESS  a process of its own, reaching the engine on loopback
temper ctl DIRECTORY TOOL ...          a process per call, reaching the engine on loopback
temper program-login NAME              run by hand on a host, replaced by the program's sign-in
```

A box runs `temper serve` as one of its steps, after the forge's setup's
commands, each a child of its own (setup.md, 7.1).

## 5. Components and crates

**jig's,** composed as they are (jig's `protocol/README.md`, section 5):

| Crate | In | What temper uses it for |
|---|---|---|
| `jig-protocol-config` | engine, worker | jig's parts of the configuration (configuration.md, section 2) |
| `jig-protocol-store` | engine | the store, temper's families beside jig's in the same commits (store.md) |
| `jig-protocol-worker-wire` | engine, worker | the channel's records; temper's parts ride in them as bytes (channel.md) |
| `jig-protocol-engine-workers` | engine | the worker's link, the engine's side: opening, the payload version, grants' values filled in |
| `jig-protocol-worker-link` | worker | the worker's link, the worker's side: dialling, grants' values taken out |
| `jig-protocol-accounts` | engine | LLM accounts on skein's accounts driver: signed-in records and keys from jig's secrets' store, a loopback listener for each sign-in's redirect, the access tokens and keys lent (credentials.md, section 3) |
| `jig-client-protocol-wire` | engine, client | jig's records for clients; the forge's client family's records ride in them as bytes (`../client/README.md`) |
| `jig-protocol-engine-clients` | engine | the clients' component: jig's messages with their credentials, the printed link's secret, sign-ins, credentials (`../client/README.md`, section 8) |
| `jig-protocol-engine-clients-http` | engine | jig's default HTTP mapping, its engine end, with its example guards, mounted on temper's clients' listener under `/jig` |
| `jig-client` | client | the client temper's client root holds: the link, watches by handle, the request table, kept work, families; jig's helpers (`../client/README.md`, section 3) |
| `jig-client-protocol-link` | client | `jig-client`'s half toward its transport: jig's messages and the small store's records as bytes (jig's `protocol/client.md`, 4.3) |
| `jig-client-protocol-http` | client | jig's default HTTP mapping, its client end, under `jig-client-shell-native` and `jig-client-shell-web` |
| `jig-client-shell-web` | client | the browser shell's adapter over skein's fetch, and the printed link's sign-in part (`../client/browser.md`, 2.1) |
| `jig-protocol-transcript` | client | turns' bytes, decoded below `jig-client` |
| `jig-client-shell-native` | client | `temper open`'s and `temper ctl`'s transport: HTTP on io, the credential `jig-client` holds sent as a bearer, the small store's file in its directory |

temper does not use `jig-protocol-charter` or `jig-protocol-tools`: its
root takes its own `Tool` vocabulary, translated by `temper-engine-smith`
(llm.md, sections 4 and 5). Nor does it use jig's reference UI
(`jig-client-view`, `jig-client-test-shell-web`, nor the look of
`jig-client-shell-web`): temper's pages are its own,
on skein's UI kit (`../client/README.md`, section 5).

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
`skein-json`, configuration and the forges' documents; `skein-http`, the
forge component's client and its webhooks' listener's server, the setup
host's client, and the fake forge's server; the forge kit,
`skein-forge` (Forgejo's and GitHub's documents and call plans), `skein-forge-client`,
`skein-forge-connection` (the forge component) and `skein-forge-setup`
(the setup host and its plans), with the fake forge and the throwaway Forgejo in its worlds
(skein's `forge.md`, section 2);
`skein-tls`, a forge's TLS; `skein-codec` and its generator, every codec
crate; `skein-kv`, under jig's store and temper's key encoders;
`skein-channel`, under jig's channel; `skein-oauth-accounts`, under
jig's accounts; `skein-ui`, under temper's view, and
`skein-ui-text`, the terminal's text shell. In the worlds: skein's
simulator, fake machine, fake LLM peer, fake issuer and fake person.

**temper's own,** each step code under the subset but the binary, the
terminal's host, `temper ctl`'s host, `temper-forge-setup-shell`, the
box's host and the fake git program's core:

| Crate | Kind | Holds | Depends on |
|---|---|---|---|
| `crates/temper-service` | services | the engine's and the worker's services and their one-process composition, in an engine, a worker and a system module, as jig's reference service is: each `iterate`, the routing, the clients' listener with jig's clients' component and its default HTTP mapping mounted, temper's forge routes, and the browser's page and bundle served (`../client/browser.md`, 2.2), startup from configuration's bytes, start and end; temper's profile `small` | the roots, `temper-engine-smith`, jig's components, smith's host half and codecs, temper's codec crates and components, `skein-forge-connection`, `skein-io`, `skein-json` |
| `crates/temper` | binary | the commands of section 4, copied from jig's reference binary; for each that reads the document, `temper-service`'s startup, with `setup` and the setup's and the box's limits checked by their own crates (configuration.md, section 2) | `temper-service`, `skein-shell`, `smith-agent-shell`, `smith-local-service`'s profile, `skein-oauth-accounts`, `jig-client-shell-native`, the client crates, `temper-forge-setup-shell`, `temper-box` |
| `crates/temper-forge-setup` | step code | the forge's setup, but for reconcile, which is skein's (`skein-forge-setup`): its types and checks, and the setup mapped onto reconcile's given values, with the kit's check of them; the preparation's machine; `app.ini` and the runner's files; the processes' specs; the configuration's steps, `steps` and `Step`. Names no host | setup.md, section 11 |
| `crates/temper-forge-setup-shell` | hosts | ordinary Rust: the hosts of `temper forge prepare` and `run`: commands, files, the forge's locks and the exec (setup.md, 6.4); HTTP on skein-http over skein-io | setup.md, section 11 |
| `crates/temper-box` | machine, host | the box's machine, step code, and its host, ordinary Rust, on skein-io's processes; nothing of the setup but its steps (setup.md, 7.3) | setup.md, section 11 |
| `crates/temper-fake-git` | program | the fake git program's core, ordinary Rust, which the binary runs as `temper fake git`: its checks, answers, step core, replay and two remotes (`../domain/testing.md`, 4.3) | setup.md, section 11 |
| `crates/temper-worker-protocol-git` | component | the git component: each `git::Op` of `temper-worker-domain-checkout` as a fixed sequence of git invocations, their arguments, environment, output and failures; the operations in flight, their processes, pipes and deadlines; `Make` as file operations (git.md) | `skein-lib`, `skein-io`, the checkout's vocabulary |
| `crates/temper-protocol-worker-wire` | codecs | temper's parts on jig's channel, and a git identity's value, from a schema; `PAYLOAD_VERSION` (channel.md) | `skein-lib`, `skein-codec` |
| `crates/temper-protocol-store` | codecs, keys | temper's families: key encoders, codecs from a schema, `STORE_VERSION`, the restart's interval (store.md) | `skein-lib`, `skein-codec`, `skein-kv`, `jig-protocol-store`, the forge's records |
| `crates/temper-client-protocol-wire` | codecs | the forge's client family's records: the adoption request and its outcome, the small store's record of a pending one; `FORGE`, the forge connector's number, 1; `Owner`, temper's owners; `CLIENT_VERSION` (`../client/README.md`, 4.6) | `skein-lib`, `skein-codec` |
| `crates/temper-engine-smith` | translation | smith's start from an assignment, host calls decoded into the root's `Tool` vocabulary, answers rendered, results and turns read; called by the engine's routing, keeping no state (llm.md) | the root's and its forge children's vocabularies, jig's and smith's domain vocabularies, `skein-lib`, `skein-json` |
| `crates/temper-client-domain` | client root | `jig-client` held, temper's pages, the forge's client side (`../client/README.md`, section 3) | `../client/README.md`, section 2 |
| `crates/temper-client-view` | view | temper's pages as skein-ui's tree, with the forge's view (`../client/README.md`, section 5) | `../client/README.md`, section 2 |
| `crates/temper-client-shell-terminal` | shell | the terminal: its host ordinary Rust, over skein's text shell and jig's native transport (`../client/README.md`, section 6) | `../client/README.md`, section 2 |
| `crates/temper-client-shell-web` | shell | the browser: a `cdylib` for wasm32, ordinary Rust, over skein's browser parts and jig's browser adapter (`../client/browser.md`, 2.1) | `../client/README.md`, section 2 |
| `crates/temper-client-tools` | client root, rendering | the agents' tools: the table, the runner, marks and conditions, the rendering, the tools' root (`../client/tools.md`, section 4) | `../client/tools.md`, section 4 |
| `crates/temper-client-shell-cli` | shell | `temper ctl`: its host ordinary Rust, one call per process over jig's native transport (`../client/tools.md`, section 8) | `../client/tools.md`, section 4 |

**temper's test crates, worlds and tiers** are `../domain/testing.md`'s
(sections 2, 4 and 7): the fake git program, the worlds below the
domain, the process and live tiers, and the tool that records git's
transcripts (git.md, section 9). The fake forge's protocol layer, the
throwaway Forgejo and the tool that records Forgejo's transcripts are
skein's (`fake-forge.md`, sections 4, 6 and 7).

**What temper's components share** is what every component of jig's
does (jig's `protocol/README.md`, section 1; programming-model.md,
sections 4 and 9): each a child of the protocol layer with its own
limits, worst case, deadlines and most outputs per entry point; its
connections or invocations its own entities, bound to calls by echoed
tokens; one exchange in flight per connection, with an output cap and a
room check before the next is parsed; a call with no room refused before
anything is set aside for it; each call's deadline the domain's, and
each connection's or invocation's its own, armed from its limits. Two
rules every temper component keeps:

- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines: for the forge,
  unavailable only when the call provably never left, and timed out
  otherwise (skein's `forge.md`, 5.4). A retry below the domain hides its cost from
  the budget, and adds load just when the peer is struggling.
- **Limits and the worst case.** Each component exports its `Limits` and
  `worst_case`, summed with jig's at start (configuration.md, section 7).

## 6. The engine's service

In the shape of jig's `protocol/service.md`, section 2:

```
temper-service, engine module           iterate: io ↔ components ↔ root
├── temper-engine-domain                the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── jig-protocol-store              commits, loads, transcripts, secrets; temper's families (store.md)
    ├── jig-protocol-engine-workers     the worker's link; temper's parts as bytes (channel.md)
    ├── jig-protocol-accounts           LLM accounts (credentials.md, section 3)
    │   └── skein-oauth-accounts
    ├── the clients' listener           temper's: TCP on loopback (../client/README.md, section 8)
    │   └── jig-protocol-engine-clients-http   jig's default HTTP mapping, under /jig, with its example guards
    ├── jig-protocol-engine-clients     the clients' component: jig's messages; the forge's client family as bytes (../client/README.md, section 8)
    ├── skein-forge-connection          the forge component: every configured forge's API; the webhooks' listener, TCP on loopback (forge.md)
    ├── the setup host                  skein-forge-setup's in-process host: the setup's plans (forge.md, 2.6)
    ├── the identities' table           git identities' values (credentials.md, section 4)
    └── timers                          the root's and every component's deadlines
```

Starting, ending, `iterate`'s passes, room and timers are jig's
(jig's `protocol/service.md`, sections 3 to 5). What temper adds:

- **The forge component,** speaking each forge's API, Forgejo's REST API
  under `/api/v1` or GitHub's in skein's dialect, polled at the domain's
  cadences (skein's `forge.md`, section 5). The root's
  `Request::Forge { call, repository, op }` goes to it; it answers each
  call with exactly one `client::Event::Answered { call, cost, result }`,
  which the routing hands the root as a forge event
  (`../domain/root.md`, 5.1). An event the root cannot take yet comes
  back to the component, which holds it and offers it again (jig's
  `protocol/service.md`, section 2, room); one the root returns during
  its drain stays with the component until it closes (forge.md, section
  2; `../domain/root.md`, 9.2). Each forge's access is the component's
  alone (credentials.md, section 5).
- **A forge's start check** (credentials.md, section 5; skein's
  `forge-connection.md`, section 3): when a forge refuses its access, or
  it answers as a user other than the forge's own, that forge stops,
  never the engine: the component gives `Stopped { forge, why }`, the
  deployment's page names it with Replace (`../client/forge.md`, 5.4),
  and its calls are answered `NotConnected`; the engine and every other
  forge run on (forge.md, 2.3). Records that do not fit the configured
  forge leave it not connected, named the same way (credentials.md,
  2.1). A failure of jig's secrets' store itself stops the engine
  (credentials.md, section 5).
- **The setup host,** skein's in-process host component, beside the
  forge component: the routing runs its plans (reconcile at each start
  of a managed or owned Forgejo, a connect's, an adoption's access) and
  carries what passes between it, the root and the forge's records. It
  holds an admin's token or a GitHub App's key only while a plan runs
  (forge.md, 2.6; setup.md, the setup host in the engine).
- **The webhooks' listener,** the forge component's own, when
  `application.webhooks` names it: each delivery verified as it streams,
  answered, then made a hint, which the routing hands the root as a forge
  event (forge.md, sections 2 and 5). A hint is never a fact: polling
  finds what a lost delivery told of (forge.md, section 6).
- **temper's parts.** As it encodes an assignment, the routing encodes
  the root's `ForgeWorkspace` as temper's workspace items, with
  `temper-protocol-worker-wire`, making each repository's remote from its
  forge's `git` base and the repository's owner and name
  (configuration.md, section 3), with the owner's account id on a GitHub
  forge, so workers need no forge configuration.
  As it decodes an answer, it decodes the work and the preparation into
  the root's `Parts` (channel.md, sections 4, 5 and 7;
  `../domain/root.md`, 7.4).
- **temper's families.** The root's forge records go to the store under
  jig's application tag, encoded by `temper-protocol-store`, in the same
  commits as jig's; the forge's restart step loads them as one interval
  (store.md).
- **The identities' table,** filled from each Forgejo forge's records at
  start and from each GitHub forge's minted installation tokens as they
  come, keeping each identity's two newest generations; the routing puts
  each value in the table, then gives the root one `Event::Credential`
  for it, before its restart for a Forgejo forge's, and
  `jig-protocol-engine-workers` fills grants' values from the table
  through the routing (credentials.md, section 4; `../domain/root.md`,
  7.1).
- **Host tools, the charter and results through `temper-engine-smith`,**
  called in the routing, with smith's codecs, in the step that routes
  them, keeping no state (llm.md, sections 4 and 5; `../domain/root.md`,
  5.2).
- **The clients' listener.** The service owns it, and mounts jig's
  clients' component on it with jig's default HTTP mapping under `/jig`,
  with the mapping's example guards (`../client/README.md`, section 8; jig's
  `protocol/service.md`, section 2). A path jig's component does not claim
  goes to temper's forge routes, `/forges/...`, a GitHub connect's
  browser steps, each a setup host's run (`../client/forge.md`, 9.1), then
  to the browser's page and bundle; every other path, and a `GET` of the
  printed link's route, is answered not found.
- **The client's application wire.** The forge's client family's
  requests are decoded by `temper-client-protocol-wire` in the routing
  and answered by the root; every other part of the forge's is answered
  as unknown (`../client/README.md`, sections 4 and 8). An adoption names a
  forge, an owner and a name; the routing completes it with the forge's
  host, the host of its configured `git` base with `:port` when it names
  one (configuration.md, section 3), and hands the root the party's keyed
  request; the root locates the repository, then makes jig's adoption
  request on the party's behalf (`../domain/root.md`, 5.4;
  `../domain/forge.md`, section 4).
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
  LLM grants go to smith's host half in its form, a credential's value
  with its scheme or a program source's name (jig's `protocol/llm.md`,
  section 6); git identities' go into the identities' table,
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
  own, which a stop signals whole (jig's `protocol/README.md`, section
  10); contained process trees are for hosts apart (section 14).
- **The forge outside,** reached over HTTP, and over TLS when it is not
  on loopback; its deliveries reach the webhooks' listener on loopback,
  so a forge on another host polls alone (forge.md, section 5).
- **The core knows its workers are in process:** the engine's workers'
  listener is the in-process name, from which jig's configuration tells
  the core so (jig's `protocol/configuration.md`, section 3;
  `../domain/root.md`, section 2).
- **Starting and ending together:** configuration for both, then the
  engine's start, then the worker's. A signal drains the engine; once it
  has drained, the service ends the worker at once, its agent processes'
  and git invocations' groups killed as io closes them. No worker
  outlives the engine, so the next start settles the worker's claims at
  once (jig's `domain/engine.md`, section 6; `../domain/root.md`, 9.1).
- **Its world** is `tests/one-process` (`../domain/testing.md`, 2.5).

## 9. Versions

jig's rules hold (jig's `protocol/README.md`, section 7): one version at a
time before release, a reader that refuses another and says so, and an
upgrade that stops the engine and its workers together. jig's versions
(its channel's, its store's format and families', its clients' per
owner) and smith's (its charter's, transcript's and result's) are as
jig's `protocol/README.md`, section 7 and smith's `protocol/README.md`,
section 8 give them. temper's sit beside them:

| Version | Owner | Covers | Checked |
|---|---|---|---|
| `PAYLOAD_VERSION` | `temper-protocol-worker-wire` | temper's parts: the workspace items, the saved-work branch, the work, the preparation; a git identity's value; and a delivery's evidence, smith's record as the smith temper links encodes it | the worker's credential at the link's opening: another is refused as *application* (channel.md, section 3) |
| `STORE_VERSION` | `temper-protocol-store` | temper's families | at the store's opening: a store written by another refuses to open (store.md, section 4) |
| `CLIENT_VERSION` | `temper-client-protocol-wire` | the forge's client family's records | per owner, as jig's: the engine serves exactly its own, and `temper open`, from the same binary, speaks it (`../client/README.md`, 4.6) |
| keys' markers | the forge connector | the keys temper writes into forge bodies | the key's version, its first byte; the marker has none of its own (forge.md, section 4) |
| configuration | none | — | unknown members refuse the start (configuration.md, section 2) |

- **Each of temper's three moves at most once per plan,** at the first
  landing in that plan that changes one of the encodings it covers; for
  `PAYLOAD_VERSION`, a repin of smith that changes a delivery's record
  counts (channel.md, section 3). Plans that run at once share one move:
  the plan that names the move in its own README makes it, and the
  others change their encodings in place in the schema `main` holds,
  before the move or after it.
- **Both channel components are given `PAYLOAD_VERSION`** when they are
  made; neither knows what it covers.

## 10. Secrets

The secrets temper's processes hold, where each comes from, who holds it
and what it reaches are credentials.md's (section 2); the files they are
read from are configuration.md's (section 6). Across the layer:

- **Names in the domain, values beside it** (jig's `protocol/README.md`,
  section 8): no domain state, fact, record of the main store or log
  holds a value. Values move between components through the routing,
  never through a root.
- **temper's forge secrets are in jig's secrets' store:** each forge's
  records (its credentials, ids and accepted installations) under
  temper's tag, read whole at start and written only by the engine's
  protocol layer (credentials.md, 2.1), and the values clients' requests
  carry, kept by jig until their run ends (credentials.md, 2.2). The
  files read at start are configuration.md's (section 6).
- **Nothing crosses a host** in standalone: the link is in process, the
  client and the webhooks on loopback, and a forge off the machine is
  reached over TLS only (configuration.md, section 3).
- **The forge's setup's secrets:** the engine's setup host holds an
  admin's token or a GitHub App's key only while a plan runs (setup.md,
  the setup host in the engine; credentials.md, section 5). The one setup
  file `temper serve` reads is an owned forge's handover `admin_token`,
  read once and removed after its `TakeOver` is done (forge.md, 2.6;
  configuration.md, section 6).

## 11. What temper uses from jig, smith and skein

temper's protocol layer uses jig, smith and skein as they are, at the
revisions temper's `Cargo.lock` pins.

- **jig:** the components of section 5; the reference service and binary,
  copied; `jig-client` and its native transport; jig's client test kit,
  its check, faults and attack stories, for temper's client tiers; the
  conformance kit, as the domain's worlds use it.
- **smith:** the agent shell, the host domain and host half, the codecs;
  the agent process over skein's LLM client, and its program backend,
  Claude Code (smith's `protocol/programs.md`); smith's fake `claude`, in
  the worlds.
- **skein:** io with its in-process names, processes and process groups;
  the startup kit and its private reads; lib's SHA-256 and its HMAC;
  JSON; HTTP's client and server; TLS's client; the codec generator; the
  key-value store; the framed channel; the accounts driver, under jig's
  accounts; the UI kit and its text shell; the simulator, the fake
  machine, the fake LLM peer, the fake issuer and the fake person.
- **What is temper's own:** its presentation, since presentation is the
  application's (jig's `client/domain/client.md`), on skein's UI kit and
  text shell (`../client/README.md`, sections 5 and 6); and, where something
  would be better upstream, as jig's design allows, the fake git
  program, on skein's fake checkout and fake forge; and `temper program-login`, the
  application's command a program source's holder runs on each host
  (jig's `domain/accounts.md`, section 5).
- **Not used:** `jig-protocol-charter` and `jig-protocol-tools` (llm.md);
  agents in a host's own process (`smith-inline-agent`; section 14);
  jig's reference UI (section 5).

## 13. Testing

`../domain/testing.md` applies skein's strategy (testing-strategy.md) to
temper: its tiers, fakes, layout and budgets. What checks this layer:

- **Step tests:** every codec against golden bytes, at its bounds and by
  fuzz targets; each translation both ways; git's argument and
  environment builders, and its output parsers against transcripts of
  real git (`tools/git-transcripts`); `skein-forge`'s decoders against
  transcripts of the throwaway Forgejo (skein's conformance,
  `fake-forge.md`, section 7);
  temper's pages as skein's text shell prints them (`../client/README.md`,
  section 10).
- **Machine worlds:** the forge component against a scripted HTTP peer
  (skein's `forge-connection.md`, section 10); the git component against
  scripted process ends and outputs; the terminal's host against a
  scripted stdin and stdout.
- **The forge's protocol world,** `tests/engine/protocol-forge`: the
  forge connector world's stories through bytes, over in-memory streams
  cut at random, with the cost checks by HTTP request (`../domain/testing.md`,
  2.2).
- **The client's joined world,** `tests/client`: temper's client root
  and view, with skein's fake person and jig's check, joined to temper's
  engine root (`../domain/testing.md`, 2.1 and 6.3).
- **The one-process world,** `tests/one-process`: `temper-service` on
  skein's simulator, killed and restarted at drawn moments, drained by a
  shutdown. Its referee: nothing a party, the worker or the forge saw
  depended on a lost commit; a drained shutdown ends with nothing
  waiting for a commit; no secret's value in a domain's state, a store
  record or an agent process's environment (`../domain/testing.md`, 2.5).
- **Reconcile's world** is skein's: reconcile against the fake forge's
  protocol layer, serving the setup's routes (skein's `forge-setup.md`,
  section 7; setup.md, 13.2).
- **The process tier and the live tier:** the real binary end to end,
  outside the two suites: a chat answered, a change landed on an owned
  Forgejo with CI and webhooks, the binary killed and restarted, `temper
  box`, `temper program-login` over smith's fake `claude`; then a real
  model (`../domain/testing.md`, 2.6 and 2.7; setup.md, 13.4).

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
  nothing of `temper serve`'s own; a box restarts it (setup.md, 7.2),
  and otherwise the person or the machine's
  service manager starts it again, and it recovers from the last
  durable commit.
- **Agents in the worker's process** (`smith-inline-agent`), with no
  containment, for one person's machine (jig's `protocol/service.md`,
  section 10). Default: agent processes, as `temper agent`.
- **After standalone,** each designed where it is named, and needed only
  beyond one person's machine: webhooks from another host, over TLS
  (forge.md, section 5); hosts apart: TLS on the engine's listeners,
  contained process trees, a process with a worker only (jig's
  `protocol/README.md`, section 10); the forge as people's sign-in
  provider (credentials.md, section 8); the forge's client documents and
  panels (`../client/forge.md`); and what the forge's setup leaves for later
  (setup.md, section 15). Default: standalone.
