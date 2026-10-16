# Status

Updated 2026-10-16. How far temper's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| protocol | done 2026-10-14 | the protocol layer, standalone (`protocol.md`, section 12, steps 1 to 13): the codecs, `temper-service`, the binary's four commands, the client root and view and the terminal, the forge component and its worlds, the git component, the one-process world, the process and live tiers; the domain changes of `protocol.md`, section 5; the engine worlds on jig's store; the web crates retired | `docs/development/plans/protocol/` |
| web-implementation | done 2026-10-06 to W4; W5 to W10 superseded 2026-10-14 | `web/` in `temper-web-domain` and `temper-web-view`, W1 to W4: the two spines, the person and the client's world, the first slice. W5 to W10 are never built: temper's client is jig's client with temper's client root (`client.md`), which retires both crates (`client.md`, section 9) | `docs/plans/web-implementation/` |

The domain migration and jig extraction are complete; their plans and
adopted drafts have been retired. jig is now a separate repository,
pinned in `Cargo.lock` (`docs/development/local-jig.md`): jig
`d91a89c4`, with the smith and skein its lock holds, since 2026-10-14,
when temper's domain was reconciled with jig's hub, core and charters. Temper's
scenarios run through jig's conformance kit (`tests/conformance/README.md`),
and the system world composes the engine, worker and real Smith agents
(`tests/system/README.md`).

The protocol layer is designed for standalone mode (`protocol.md`); the
plan that builds it is `protocol` (`docs/development/plans/protocol/`).
"protocol" in the table below names it. The supervisor marks a plan
active and done in this file, and the designer keeps the rest (skein's
development.md, section 3.3).

The first milestone, a person's change from a chat to its landing on a
real forge, and what stands between the `protocol` plan and it, is
`draft/mvp.md`.

## 2. The design

| Document | Built | Left |
|---|---|---|
| `agent-domain.md` | no | describes the legacy agent, superseded by smith and `domain/agent.md`; to be removed: not planned |
| `channel.md` | no | all: temper's parts on jig's channel, the payload version, a git identity's value: protocol |
| `client.md` | partly | built: the forge's words and "Shown on the forge." (4.1, 4.2), its records (4.6). Left, waiting for jig's client API (jig's `STATUS.md`, "jig's client API"): the client root on `jig-client`, with two owners, temper's pages and the forge's side, routing by asker (3.1, 3.3), the pages after the mockups and their confirmations, on jig's helpers (3.5, 3.6), the forge's family as bytes and its form a page of temper's (4.3 to 4.7), `Owner` in the wire crate (4.6); the view on `skein-ui` (5); the terminal on skein's `skein-ui-text` over `jig-client-shell-native`, its host keeping the credential's file through `jig-client-shell-native` and signing in again with `Held` (6, 7); `temper-service` owning the clients' listener, jig's default HTTP mapping under `/jig` with its example guards, the link route's `GET` answered not found (8); `tests/client` with jig's check, and the rebound-Host attack story in the process tier (10); the crates `temper-protocol-client-wire` renamed `temper-client-protocol-wire` and `temper-shell-terminal` `temper-client-shell-terminal`, by the client naming rule; the client root, view root and native client copied from jig's retired. Not planned: the forge's documents and panels, the browser, the forge as a sign-in provider through `Mint` (section 11) |
| `configuration.md` | no | all but the `webhooks` member: protocol; `webhooks`: not planned; 3, 8: `application.clients` (the listener), `templates` in place of `chats`, the `documents.lines` override: wait for jig's client API (jig's `STATUS.md`, "jig's client API") |
| `credentials.md` | no | all: protocol; the forge as people's sign-in provider: not planned; 6: the terminal's credential kept by its host and given back with `Held`: waits for jig's client API (jig's `STATUS.md`, "jig's client API") |
| `engine-domain.md` | no | the legacy engine, superseded by `domain/`; still cited by the fake forge and the checkout; to be removed: not planned |
| `forge.md` | no | sections 1 to 4 and 6 to 13, Forgejo v16.0.5 polled, the fake forge's protocol layer, the cost checks through bytes: protocol; 5, webhooks, and 11, GitHub: not planned |
| `git.md` | no | all: protocol; containment between hosts: not planned |
| `llm.md` | partly | `temper-engine-smith` decodes and renders host tools and reads results; its place in the engine's routing: protocol |
| `performance.md` | no | section 3's two containers: protocol; sections 5 and 6, measurements once `temper serve` runs: not planned |
| `protocol.md` | no | standalone (sections 4 to 10, 12): protocol; between hosts (TLS, contained process trees, a worker-only process): not planned; 1 to 6, 8 to 14: the client crates by the client naming rule, `temper-service` owning the clients' listener and mounting jig's clients' component: wait for jig's client API (jig's `STATUS.md`, "jig's client API") |
| `testing.md` | partly | the domain and system worlds and the client's joint world (6.3) stand; section 2's tiers below the domain (2.2, 2.5 to 2.7), the fake forge's protocol layer (4.2) and the fake git program (4.3): protocol; a scenario's limits from the shipped profile in every world, fixture and process configuration (5.1), and the live story ending at its task's first terminal failure, with each run's summary (2.7): not planned; 2.1, 2.6: the client's tiers on `jig-client`, the rebound-Host attack story: wait for jig's client API (jig's `STATUS.md`, "jig's client API") |
| `worker-domain.md` | no | the worker before jig's hub, superseded by `domain/worker.md` and jig's `hosts.md`; still cited by the worker's crates and worlds; to be removed: not planned |
| `domain/README.md` | yes | the names and maps stand; section 5 points at the protocol documents |
| `domain/core.md` | yes | built in `jig-core` and its children under temper's root; the document still describes primitives owned by jig; to keep temper's application side and cite jig for the rest: not planned |
| `domain/tasks.md` | yes | built as `jig-core-tasks`; the document still names temper's crates; to keep temper's application side and cite jig's `tasks.md` for the rest: not planned |
| `domain/authority.md` | yes | built as `jig-core-authority`, landing rules translated to the forge's requirements; the same move: not planned |
| `domain/connectors.md` | partly | the contract is in `jig-core` and the forge meets it; section 14's second connector, test environments: not planned |
| `domain/engine.md` | partly | the root in jig's reference shape on `jig-core` and skein-lib's journal, notes in the store; 7.3's undecodable host call relayed with no commit, section 8's failed preparation reported as a fact of the attempt, section 14 below the domain: protocol; section 16's saved work (the root names no saved-work branch in an assignment, so no run saves): not planned |
| `domain/forge.md` | partly | the connector's four crates on the fake forge; section 4's adoption request, locate and the forge's host, and 17's protocol: protocol; sections 13 and 14 beyond the `Fork` role and participation news: not planned |
| `domain/people.md` | partly | `jig-core-people` and the root's routes; section 11, signing in by the deployment's link: protocol; through the forge: not planned |
| `domain/agent.md` | partly | the typed translation to smith (`temper-engine-smith`), the worker hosting smith, the system world; section 3's agent shell as temper's `agent` command, section 9's protocol tests and the charter's encoding: protocol |
| `domain/worker.md` | partly | the worker root on jig's hub (`jig-domain-host`), the checkout and smith's host domain, with whole-worker worlds; section 9 below the domain, its parts naming repositories by tag: protocol; section 11's contraction (a missing base branch created, a push with no expected head): not planned |
| `store/README.md` | no | all: temper's families on jig's store: protocol |
| `web/` | no | W1 to W4's crates retired by `client.md`, section 9 (protocol 11.4); `web/architecture.md`: temper's browser, `temper-client-shell-web`, on skein's `skein-ui-web` and jig's `jig-client-shell-web`, its sign-in by the printed link's page and cookie, its tier with jig's cross-site and framed-page stories: not planned; `web/mockups/`: the pages `client.md`, 3.5 builds; `web/ux/`, the forge's family's pages: not planned. `web/ux/` predates jig's client and is partly superseded by `client.md`, 3.5 and the mockups |

## 3. Not planned

- **What the reconciliation with jig `d91a89c4` left, and the protocol
  design does not take** (`domain/worker.md`, 11): jig's `Approval`
  contract for the forge's person gate, where temper keeps a verdict; a
  run's live text (below); a calendar for periods (`domain/authority.md`;
  `configuration.md`, section 11). The forge's client documents are the
  item below; a failed preparation's repository and reason are the
  protocol plan's (`domain/engine.md`, section 8).
- **The forge's client documents and panels** (`client.md`, section 4;
  `web/ux/forge.md`): panels, results, effects, outbox entries, holds, a
  project's and the deployment's forge parts, answered as unknown until
  then (jig's `domain/connectors.md`, 5.5). The first client has the
  forge's words and its adoption form only. Waits for a plan of its own
  after the protocol plan.
- **After standalone** (`protocol.md`, section 12, "After
  standalone"): between hosts
  (TLS on the engine's listeners, a worker-only process, contained
  process trees for agents and git), the forge's webhooks (`forge.md`,
  section 5), the forge as people's sign-in provider (`domain/people.md`,
  section 14), GitHub (`forge.md`, section 11). Wait for a deployment
  beyond one person's machine, and for skein's TLS server side and
  contained process trees (jig's `protocol/README.md`, section 10).
- **The browser** (`web/architecture.md`; `client.md`, section 11):
  temper's browser shell, `temper-client-shell-web`, on skein's
  `skein-ui-web` and jig's `jig-client-shell-web`, served by
  `temper-service`, with its page's policy and jig's cross-site and
  framed-page attack stories. Waits for temper on jig's client API
  (below) and jig's browser step.
- **A failed preparation's words on the task's page**
  (`domain/engine.md`, section 8): the root reports a permanent
  preparation's repository and reason as a fact of the attempt; people
  read them on the task's page once jig's core keeps an attempt's
  failure detail, which its answer's end, a class only, does not at
  temper's pinned revision.
- **A run's live text in the client** (jig's `protocol/channel.md`, 6.4):
  waits for smith's channel to carry it (jig's STATUS.md, "What jig asks
  of smith, later"); until then runs show their activity, and their
  words as each turn commits.
- **jig's charter and tools translations** (`llm.md`, section 7):
  temper's routing keeps `temper-engine-smith` for the charter, host
  tools and results. Waits for the root's tools to be reconciled with
  jig's core tool events.
- **Documents of deleted code** (`engine-domain.md`, `agent-domain.md`,
  `worker-domain.md`): to be removed. They wait for the code citing them
  to be repointed to `domain/` and jig's documents.
- **temper's `domain/` as an application of jig:** `core.md`, `tasks.md`,
  `authority.md`, `engine.md`, `people.md`, `worker.md` and `agent.md`
  still describe crates that are now jig's. Each keeps temper's side and
  cites jig's for the rest. It waits for jig's boundary to settle.
- **A second connector** (`domain/connectors.md`, section 14): test
  environments. It waits for continuous QA or the factory to be adopted
  (`draft/qa.md`, `draft/factory.md`).
- **Participating objects and upstream contributions**
  (`domain/forge.md`, sections 13 and 14): outside contributions
  triaged and reviewed, changes landed by others' merges, submissions
  from a fork. Not needed first; they wait for a project that wants them.
- **Performance's measurements** (`performance.md`, sections 5 and 6):
  allocator tuning and the byte budget, once `temper serve` runs.
- **smith's and skein's changed APIs** (smith's `host.md`, section 11):
  the typed agent face, smith's inline agent, removed modes. temper
  repins after smith's reliability plan
  (`~/src/rust/plans/reliability-plan/`, not versioned), which never
  edits temper.
- **temper on jig's client API** (`client.md`; `protocol.md`;
  `configuration.md`; `credentials.md`; `testing.md`; jig's
  `STATUS.md`, "jig's client API"): temper builds its own presentation
  and server on `jig-client`: its client root with temper's pages after
  the mockups and the forge's side, its view on `skein-ui`, its terminal
  on skein's text shell, `temper-service` mounting jig's clients'
  component under `/jig`; its copied client root, view root and native
  client retire, and with them the repin onto jig's pages that
  client-composition left. Its crates follow the client naming rule:
  `temper-protocol-client-wire` becomes `temper-client-protocol-wire`,
  `temper-shell-terminal` becomes `temper-client-shell-terminal`.
  `temper-service/src/family/refusals.rs` copies jig's private
  translation of refusals (`jig-protocol-engine-clients`'
  `translate/refusals.rs`); it goes when temper takes jig's typed
  answers. Before that, temper's repin onto jig past skein-ui (jig's
  `docs/development/plans/skein-ui/later.md`) changes:
  - the tree's renamed types, `State::Provisional` dropped, and its
    users of `jig_fake_person` (14 lines in 6 files at `8d27a682`) and
    `jig_shell_text` moved to `skein_fake_person` and `skein_ui_text`;
  - its four literals of skein-ui's `Limits`, which need `scene_marks`
    and `scene_bytes`: `crates/temper-client-view/src/tests/fixtures.rs:75`,
    `crates/temper-shell-terminal/src/tests/fixtures.rs:105`,
    `crates/temper-shell-terminal/tests/support/mod.rs:21`,
    `tests/client/src/fixtures.rs:111`;
  - its joint front on jig's client world: `Reply::SignOut` takes
    `sign_in` (`tests/client/src/front.rs:352`), its `match` on
    `packets::Incoming` (lines 127 to 141) takes `Incoming::SignedOut`,
    and its `Front` gets jig's `delivers` rule (jig's
    `client/domain/client.md`, section 12).

  Waits for jig's client API's plan, then a plan of temper's own.
  `draft/mvp.md`, section 6 then needs revising: it plans temper's
  client on jig's pages.

## 4. Drafts

| Draft | About | Next |
|---|---|---|
| `mvp.md` | the first milestone: the client against its design, and what stands between the `protocol` plan and a change landed for real | discuss; its section 6 names the plan after `protocol` |
| `factory.md` | the software factory: write, prove and run planes, the lab, on jig and os | not adopted: discuss; its section 13 changes `qa.md` |
| `qa.md` | continuous QA on a running build of main | not adopted: discuss; its order of building (section 8) waits for a running build |
