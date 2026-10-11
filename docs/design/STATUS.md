# Status

Updated 2026-10-11. How far temper's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| web-implementation | done 2026-10-06 to W4; W5 to W10 ready | `web/` in `temper-web-domain` and `temper-web-view`. W1 to W4 are built: the two spines, the person and the client's world, the first slice. W5 to W10 are left: the joint world, inbox, conversation, task page, board, forge panels. The plan now requires translation to the real engine from the first increment and acceptance in the joint world | `docs/plans/web-implementation/` |
| store plan | ready | `store/README.md` on skein-kv, S1 to S9. Written before jig's store part (jig's `README.md`, section 8) | `docs/design/store/plan.md` |

The domain migration and jig extraction are complete; their plans and
adopted drafts have been retired. jig is now a separate repository,
pinned in `Cargo.lock` (`docs/development/local-jig.md`). Temper's
scenarios run through jig's conformance kit (`tests/conformance/README.md`),
and the system world composes the engine, worker and real Smith agents
(`tests/system/README.md`).

No plan is active. The designer marks plans active and done in this file
(skein's development.md, sections 3.3 and 10.5).

## 2. The design

| Document | Built | Left |
|---|---|---|
| `agent-domain.md` | no | describes the legacy agent, deleted on 2026-10-07; superseded by smith and `domain/agent.md`; to be removed: not planned |
| `channel.md` | no | `temper-channel` was deleted on 2026-10-07; skein's channel supersedes it, and the engine's channel is jig's (jig's `hosts.md`, section 10): not planned |
| `credentials.md` | no | `temper-oauth` was deleted on 2026-10-07; skein's OAuth client supersedes it; secrets move to the store (`domain/README.md`, section 5): not planned |
| `engine-domain.md` | no | the legacy engine, deleted on 2026-10-07 and superseded by `domain/`; still cited by the fake forge and the checkout; to be removed: not planned |
| `forge.md` | no | `temper-forge-forgejo` was deleted on 2026-10-07; the Forgejo v16.0.5 protocol with the calls of `domain/forge.md`, section 17: not planned |
| `llm.md` | partly | providers moved to smith and skein's LLM client; of temper's half, the engine tools' schemas and decoding are in `temper-engine-smith`, rendering in a protocol layer is not built: not planned |
| `performance.md` | no | section 3's doubling buffer and `ByteRing` (no `ByteRing` in skein-lib); sections 4 to 6 wait for the protocol layer and a shell: not planned |
| `protocol.md` | no | the protocol layer, deleted on 2026-10-07, with the revisions of `domain/README.md`, section 5: not planned |
| `testing.md` | partly | the domain and system worlds stand; the conformance scenarios and composed engine/worker/Smith world are built; section 7 still describes deleted protocol worlds; section 9's tiers: not planned |
| `worker-domain.md` | no | the worker before `jig-host`, superseded by `domain/worker.md` and jig's `hosts.md`; still cited by the worker's crates and worlds; to be removed: not planned |
| `domain/README.md` | partly | the names and maps stand; section 5's revisions of the protocol documents are not made: not planned |
| `domain/core.md` | yes | built in `jig-core` and its children under temper's root; the document still describes primitives owned by jig; to keep temper's application side and cite jig for the rest: not planned |
| `domain/tasks.md` | yes | built as `jig-core-tasks`; the document still names temper's crates; to keep temper's application side and cite jig's `tasks.md` for the rest: not planned |
| `domain/authority.md` | yes | built as `jig-core-authority`, landing rules translated to the forge's requirements; the same move: not planned |
| `domain/connectors.md` | partly | the contract is in `jig-core` and the forge meets it; section 14's second connector, test environments: not planned |
| `domain/engine.md` | partly | the root in jig's reference shape on `jig-core` and skein-lib's journal, notes in the store; section 14 below the domain and the configuration: not planned |
| `domain/forge.md` | partly | the connector's four crates on the fake forge; sections 13 and 14 beyond the `Fork` role and participation news, and section 17's protocol: not planned |
| `domain/people.md` | partly | `jig-core-people` and the root's routes; section 11, the web and signing in through the forge: not planned |
| `domain/agent.md` | partly | the typed translation to smith (`temper-engine-smith`, `jig-charter`), the worker hosting smith, the system world; section 9's protocol tests and the charter's encoding: not planned |
| `domain/worker.md` | partly | the worker root on `jig-host`, the checkout and smith's host domain, with whole-worker worlds; section 11's contraction (a missing base branch created, a push with no expected head) and section 9 below the domain: not planned |
| `store/` | no | the store over skein-kv (skein's `kv.md`): store plan; turns are still records keyed per turn, so even its S1 is not built |
| `web/` | partly | W1 to W4 built; W5 to W10: web-implementation; the wire, the protocol layer and the shells: not planned |

## 3. Not planned

- **The protocol and io layers** (`protocol.md`, `channel.md`,
  `credentials.md`, `forge.md`; `domain/engine.md`, section 14): temper
  below the domain, deleted on 2026-10-07 to be planned afresh on
  skein's channel, OAuth client, HTTP and JSON. They wait for jig's layers
  below the domain (jig's `README.md`, section 12), which temper's compose.
- **The tiers below the domain** (`testing.md`, section 9): protocol
  worlds, simulated worlds and the real loop. They wait for the protocol
  layers and a shell.
- **Configuration** (`domain/engine.md`, section 14): rules, connectors'
  addresses and tokens, LLM endpoints and prices, charters, seeded
  projects. It waits for the protocol layer and `main`.
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
- **Performance's fixes** (`performance.md`, sections 3 to 6): two
  skein-lib containers before the first machine that joins pieces, copy
  accounting with the protocol layer, allocator tuning once a shell runs.
- **smith's and skein's changed APIs** (smith's `host.md`, section 11):
  the typed agent face, smith's inline agent, removed modes. temper
  repins after smith's reliability plan
  (`~/src/rust/plans/reliability-plan/`, not versioned), which never
  edits temper.

## 4. Drafts

| Draft | About | Next |
|---|---|---|
| `factory.md` | the software factory: write, prove and run planes, the lab, on jig and os | not adopted: discuss; its section 13 changes `qa.md` |
| `qa.md` | continuous QA on a running build of main | not adopted: discuss; its order of building (section 8) waits for a running build |
