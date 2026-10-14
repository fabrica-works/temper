# Status

Updated 2026-10-14. How far temper's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| protocol | active on x1 since 2026-10-14 | the protocol layer, standalone (`protocol.md`, section 12, steps 1 to 13): the codecs, `temper-service`, the binary's four commands, the client root and view and the terminal, the forge component and its worlds, the git component, the one-process world, the process and live tiers; the domain changes of `protocol.md`, section 5; the engine worlds on jig's store; the web crates retired | `docs/development/plans/protocol/` |
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
| `client.md` | partly | built: the client root and view over jig's (3, 5), the forge's words and "Shown on the forge." (4.1, 4.2), its records (4.6), the native client and its sign-in (6.2's client, 7), the web crates retired (9), `tests/client`'s first stories (10). Left: the forge's ask and address, the adoption form, its page and navigation link (4.3 to 4.7, 5.2, 5.3), the engine's side (8), the terminal's renderer, command reader and loop, `temper open` (6), the one-process and process tiers (10): protocol; the forge's documents and panels, the browser (section 11): not planned |
| `configuration.md` | no | all but the `webhooks` member: protocol; `webhooks`: not planned |
| `credentials.md` | no | all: protocol; the forge as people's sign-in provider: not planned |
| `engine-domain.md` | no | the legacy engine, superseded by `domain/`; still cited by the fake forge and the checkout; to be removed: not planned |
| `forge.md` | no | sections 1 to 4 and 6 to 13, Forgejo v16.0.5 polled, the fake forge's protocol layer, the cost checks through bytes: protocol; 5, webhooks, and 11, GitHub: not planned |
| `git.md` | no | all: protocol; containment between hosts: not planned |
| `llm.md` | partly | `temper-engine-smith` decodes and renders host tools and reads results; its place in the engine's routing: protocol |
| `performance.md` | no | section 3's two containers: protocol; sections 5 and 6, measurements once `temper serve` runs: not planned |
| `protocol.md` | no | standalone (sections 4 to 10, 12): protocol; between hosts (TLS, contained process trees, a worker-only process): not planned |
| `testing.md` | partly | the domain and system worlds and the client's joint world (6.3) stand; section 2's tiers below the domain (2.2, 2.5 to 2.7), the fake forge's protocol layer (4.2) and the fake git program (4.3): protocol |
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
| `web/` | no | W1 to W4's crates retired by `client.md`, section 9 (protocol 11.4); `web/architecture.md`, the browser: not planned; `web/ux/`, the forge's family's pages: not planned. `web/ux/` predates jig's client and is partly superseded by jig's `pages.md` and `objects.md` (`draft/mvp.md`, section 3) |

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
  temper's client in jig's browser shell. Waits for jig's browser shell
  and the client's wasm32 builds (jig's STATUS.md, "Not planned").
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

## 4. Drafts

| Draft | About | Next |
|---|---|---|
| `mvp.md` | the first milestone: the client against its design, and what stands between the `protocol` plan and a change landed for real | discuss; its section 6 names the plan after `protocol` |
| `factory.md` | the software factory: write, prove and run planes, the lab, on jig and os | not adopted: discuss; its section 13 changes `qa.md` |
| `qa.md` | continuous QA on a running build of main | not adopted: discuss; its order of building (section 8) waits for a running build |
