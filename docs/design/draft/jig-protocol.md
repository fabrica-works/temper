# temper on jig's protocol layer

Draft, 2026-10-12. A brief for temper's next design pass: what jig's
protocol layer now gives an application (jig's `protocol/`, revised and
completed with its store on 2026-10-12), what of temper's own design
moves to jig, what stays temper's, which of temper's documents change,
and the decisions temper must take. Nothing here is adopted: temper's
documents and jig's describe what is decided. What is still open is
listed in section 7.

## 1. In one page

- **jig's protocol layer is designed whole for temper's shape:** agents on
  workers, a web client, a store in the engine, and a standalone mode
  with the engine and its workers in one process, a browser on loopback
  (jig's `protocol/README.md`, section 4; jig's `protocol/service.md`,
  section 10). jig ships the components; temper composes them, with its
  forge's own, in its protocol layers, copying jig's reference service
  and binary (`jig-test-service`, `jig-test`).
- **Most of temper's protocol design becomes jig's:** the store (but its
  forge's families), the workers' channel (but its payloads), LLM
  accounts and grants, configuration's shape, the client's wire,
  connection and shells. temper's documents for them shrink to what is
  temper's, and cite jig's.
- **temper's web client moves onto jig's client.** jig's client domain,
  wire (records from schemas, not JSON), link and shells replace
  `temper-web-domain`'s and its planned wire: temper adds its forge's
  families, its wire crate for them, and its client root and view root.
  The web plan's W5 to W10 are not built on `temper-web-domain`.
- **Standalone first:** one process, the engine and one worker joined by
  an in-process address, the worker's agents as child processes (smith's
  agent shell as a command of temper's own binary), a browser on loopback
  signing in by the link the engine prints, the forge polled. Webhooks,
  the forge's sign-in, TLS between hosts and contained process trees
  come after.
- **temper waits for jig's protocol plan** (jig's STATUS.md: jig-protocol),
  then repins jig, reconciles its domain with jig's (section 5), and
  builds its protocol layer on jig's components.

## 2. What becomes jig's

| temper's design | Becomes | What temper keeps |
|---|---|---|
| `store/README.md`, `store/plan.md` | jig's `protocol/store.md`: skein-kv in the engine, keys and intervals, codecs, commits, loads, transcripts as files (one per task), secrets' store, opening, retention and fill as mechanisms, the store's world | its forge's families under its tag, their key encoders and codecs, its store version, its indexes, its numbers (horizons, budgets) as its configuration |
| `channel.md` | jig's `protocol/channel.md`, on skein's framed channel, records from schemas | its payloads inside jig's records, under its payload version: the workspace (repositories, remotes, starts, identities), what landed, saved work, a push's evidence |
| `credentials.md` | jig's `protocol/llm.md`, sections 5 and 6 (accounts on skein's driver, grants), and jig's `domain/engine.md`, 12.1 (an application's credentials) | the git identity, the forge's API token, the webhook secret, each as one of its credentials, from its own source |
| `llm.md` | nothing in the engine: a worker's agents make their own calls; jig's `protocol/llm.md`, sections 1 to 4, only if temper later runs agents in a host's process | — |
| `protocol.md` | jig's `protocol/README.md` (the layer, components, versions, secrets) and `protocol/service.md` (iterate, main, start, end, one process) | its forge's protocol, its own members of configuration, its own routing as jig's reference service has it |
| `web/architecture.md`, sections 5 and 6 | jig's `protocol/client.md` (wire per owner, one connection, server-sent events, sign-ins, the small store, shells, served by the engine) | its forge's families' documents, requests and views; its sign-in provider for the forge (a component jig's clients' side routes to) |
| `temper-engine-smith`'s host tools | jig's `jig-protocol-tools` for the core's tools (jig's `protocol/channel.md`, 6.3) | the forge's reads and effects as host tools: schemas, decoding, rendering, in its protocol layer |

## 3. What stays temper's

- **The forge connector's protocol:** its system's client, its API token,
  its webhooks on a listener of its own (jig's `protocol/README.md`,
  section 6), polling for liveness (`forge.md`, section 6).
- **The worker's root and workspace:** git checkouts, prepared, saved,
  delivered and released; the worker's root composes jig's hub, smith's
  host domain and the workspace (jig's `domain/hosts.md`, section 4).
- **Its charters,** in charter v2's terms: each model's window, output
  and prices, budgets with a reserve (jig's `domain/hosts.md`, section
  8).
- **Its configuration's own members:** the forge's address and token
  file, the webhook secret, the git identity's source, procedures'
  settings, the workspace's cache, a periods' calendar (jig's
  `protocol/configuration.md`, section 5).

## 4. temper's documents to revise

- **`STATUS.md`:** the protocol and io rows cite jig's
  `protocol/README.md`; the channel row stops citing jig's `hosts.md`,
  section 10; the credentials row stops moving LLM secrets to the store;
  the web plan's W5 to W10 wait for the move onto jig's client; the
  store plan is superseded but for temper's families.
- **`store/`:** cut to temper's families, their encodings and indexes,
  citing jig's `protocol/store.md`.
- **`channel.md`, `credentials.md`, `llm.md`, `protocol.md`:** cut to what
  section 3 keeps, citing jig's.
- **`web/architecture.md`:** temper's client root over jig's client
  domain and view, its families, its wire crate.
- **`domain/worker.md`:** the git identity granted with the assignment
  (4.1), never configured per worker (section 11 says otherwise).
- **`domain/agent.md`, section 3:** in one process, agents are child
  processes.
- **`domain/*.md` that restate jig's** (core, tasks, authority,
  connectors, engine, people): keep temper's side, cite jig's.

## 5. Reconciling temper's domain with jig's

What the repin from jig `2860e63` brings, before the protocol layer:

- **The client:** jig's client domain, level-triggered watches, the
  request table, the view's slots; temper's forge connector owes
  documents for its panels (jig's `domain/connectors.md`, 5.5).
- **Views:** level-triggered, held until durable, a run's text by turn,
  retry and offset, from hosts that give it.
- **People:** numbered providers, services as parties, the printed link,
  explicit read positions, chats forgotten.
- **Accounts and grants:** accounts on skein's driver; grants resent at a
  hello and an adoption; an application's credentials.
- **Runs:** the hello with its agent's bounds; refusals carried with
  turns; the start carrying the inbox; windows; the reserve.
- **Periods:** a calendar child of temper's root.
- **Stopping and restart:** the engine stopping; in-process workers'
  claims settled at once.
- **Host tools:** decoded outside the root.
- **Proposals and approvals:** a proposal nobody may accept, refused;
  approvals of an exact opaque state, the head commit for the forge's
  gate.
- **smith's main:** charter v2, transcript v3, the parent face (no
  `Bounce`), smith's payloads handed out by the worker's root to be
  decoded and encoded.

## 6. Decisions temper takes

- **Chats with no repository.** A chat that names no repository has no
  host today (`domain/worker.md`, 4.1 refuses an empty workspace). Either
  a worker prepares an empty workspace for it, or temper runs such chats
  in its engine with `smith-inline-agent` (jig's `domain/hosts.md`, 5.2;
  not planned in jig).
- **The forge's sign-in:** a provider component of temper's, to which
  jig's clients' side routes (jig's `protocol/client.md`, section 8),
  after standalone mode.
- **One binary:** temper's binary runs smith's agent shell as its
  `agent` command, so standalone mode is one file to install.

## 7. Open questions

- **A run's live text in the chat page** waits for smith's channel to
  carry it (jig's STATUS.md, "a run's live text and named calls from
  workers").
- **Webhooks on the clients' origin,** if one listener is wanted (jig's
  `protocol/README.md`, open questions).
