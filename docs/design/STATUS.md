# Status

Updated 2026-10-18. How far temper's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| client-api | active on carbon-forge since 2026-10-16 | temper on jig's client API, as far as temper's gate needs (the plan's `decisions.md`, 1(c)), beside jig's six steps of "jig's client API" and skein's harness (skein 01): its client crates renamed by the client naming rule, with jig's check of it; its terminal on `skein-ui-text` over `jig-client-shell-native`, the credential kept by its host and given back with `Held`; its client root on `jig-client` with two owners, the forge's request as bytes and `Owner` in its wire crate; `templates` in its configuration; the forge's family on the collapsed wire; `temper-service` owning the clients' listener and mounting jig's component under `/jig`, `application.clients`; the pages its stories use after the mockups (Inbox, Chats, Chat, the task's page, the adoption form) with their confirmations; `tests/client` on `jig-client-world` with jig's check; the three changes skein-ui's `later.md` left for temper's repin (temper 01); temper pinned to jig's last landing of the plan | `docs/development/plans/client-api/` |
| forge-setup | done 2026-10-18 | `setup.md` whole, in its section 14's order (section 15 aside, the plan's `later.md`): skein's lib with SHA-256, `sha256::Hmac` and `sha256::equal` (skein's `lib.md`, 6.2), skein's three copies on it; the setup's documents and facts both sides (admin users, the bot's tokens, orgs, teams, members, org hooks, Forgejo's command line and the runner, the deliveries); signed deliveries from the fake and the component's listener, `application.webhooks` (`forge.md`, 5); `settle`; temper-forge-setup and `temper forge prepare|run|reconcile`; the process tier on an owned Forgejo with CI and webhooks, the hinted story; temper-box and `temper box`; the owned example and the setup guide; the upgrade | `docs/development/plans/forge-setup/` |
| forgejo-17 | done 2026-10-16 | Forgejo 17.0.0 and forgejo-runner 13.2.0 pinned (`forge.md`, 1, 10, 13), the conformance's and git's transcripts and facts recorded on them, with section 13's facts but a waiting job's (the plan's `later.md`); v17's job with its steps and a step's log by a suffix range, the comparisons' `files` and `verification` (3.2); the fake forge's jobs with steps (9); `Read::Job` answering the failed step and its log's end, in the agent's `job` tool; the repair's brief naming them (`domain/forge.md`, 8.4); temper's terminal on skein's print by its own tree, its skein moved to forgejo-17's skein step (`client.md`, 6.8 and 10; skein's `ui-text.md`, 3 and 4) | `docs/development/plans/forgejo-17/` |
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
| `client.md` | partly | built: the forge's words and "Shown on the forge." (4.1, 4.2), its records (4.6). Left: the client root on `jig-client`, with two owners, temper's pages and the forge's side, routing by asker (3.1, 3.3), the pages its stories use after the mockups (Inbox, Chats, Chat, the task's page, the adoption form) and their confirmations, on jig's helpers (3.5, 3.6), the forge's family as bytes and its form a page of temper's (4.3 to 4.7), `Owner` in the wire crate (4.6); the view on `skein-ui` (5); the terminal on skein's `skein-ui-text` over `jig-client-shell-native`, its line bytes, its host adding skein's terminal's worst case and operations to its own (6.2, 6.8), a path's segments apart from the rows, a long path shown cut (5.4), an authority's entries apart from the rows, 12, a long authority shown in part (5.4), keeping the credential's file through `jig-client-shell-native` and signing in again with `Held` (6, 7), the root started before the sign-in, jig's client opening no watch until signed in (3.4); `temper-service` owning the clients' listener, jig's default HTTP mapping under `/jig` with its example guards, the link route's `GET` answered not found (8); `tests/client` with jig's check (10); the crates `temper-protocol-client-wire` renamed `temper-client-protocol-wire` and `temper-shell-terminal` `temper-client-shell-terminal`, by the client naming rule; the client root, view root and native client copied from jig's retired, the banner's word for each link state and temper's main while signed out or out of date (5.1, 6.6), the adoption's title for a project `me` does not name (4.4), the kept work dropped said in a line of the terminal's (3.4, 6.6): client-api; 6.8, 10: the terminal on skein's print by its own tree, and made from `limits::native()`'s limits in a unit test: forgejo-17; the board, the changes page, an agent's task page, a goal's page's history, budget, authority and acts, the confirmations no present story makes (3.5, 3.6), the rebound-Host attack story in the process tier (10), a chat's proposal at the call that made it, once jig's turns name what their calls made (3.5; jig's `domain/engine.md`, 7.2): not planned (client-api's `later.md`, temper's next client plan). 10's process tier on an owned Forgejo, with CI checked: not planned, the forge's setup. Not planned: the forge's documents and panels, the browser, the forge as a sign-in provider through `Mint` (section 11) |
| `configuration.md` | no | all but the `webhooks` member: protocol; `webhooks`, each forge's `webhook_secret`, `user` from a file, `application.setup` and its checks, the owned example, its `engine.limits` (7) and its owner's grants (10), `small` starting as it stands against smith's agent, the worker's check against the agent, the example's worker `limits` and its agent's profile (7, 10), the example's `memory_bytes` at the bounds of what a copy changes (7, 10): not planned, the forge's setup; 7: the worker's check against the agent naming both limits (it names `workers[0].limits` alone): not planned (forge-setup's `later.md`); 7: `small`'s worst cases within a person's machine (the owned example's are about 71.5 TiB for the engine and 1.8 TiB for the worker): not planned (forge-setup's `later.md`); 3, 8: `application.clients` (the listener), `templates` in place of `chats`, 7: the example's bounds on a name's segments, `documents.lines` for short lists alone, an authority's lists carried whole and held to temper's client's entries, the engine's watch rooms, 16 a party and 16 a stream, held to temper's client's watches: client-api |
| `credentials.md` | no | all: protocol; the webhooks' secret and the setup's credentials (2, 5): not planned, the forge's setup; the forge as people's sign-in provider: not planned; 6: the terminal's credential kept by its host and given back with `Held`: client-api |
| `engine-domain.md` | no | the legacy engine, superseded by `domain/`; still cited by the fake forge and the checkout; to be removed: not planned |
| `forge.md` | no | sections 1 to 4 and 6 to 13, the fake forge's protocol layer, the cost checks through bytes: protocol, on Forgejo v16.0.5; Forgejo v17.0 and forgejo-runner v13.2 (1, 10, 13: the pin, the throwaway Forgejo and runner, the conformance's transcripts and facts), 3.2's failed step's end for `Read::Job` and its comparisons' `files` and `verification`, 9's jobs with steps in the fake forge: forgejo-17; 13's waiting job: not planned (forgejo-17's `later.md`); 5, webhooks on loopback, made by the setup's reconcile, with 2, 7 to 10 and 12's parts of them and 13's facts of their deliveries: not planned, the forge's setup (`setup.md`, section 14); 5 from another host, over TLS, and 11, GitHub: not planned |
| `git.md` | no | all: protocol; `tools/git-transcripts`' records and facts again on Forgejo v17.0: forgejo-17; containment between hosts: not planned |
| `llm.md` | partly | `temper-engine-smith` decodes and renders host tools and reads results; its place in the engine's routing: protocol |
| `performance.md` | no | section 3's two containers: protocol; sections 5 and 6, measurements once `temper serve` runs: not planned |
| `protocol.md` | no | standalone (sections 4 to 10, 12): protocol; the `temper forge` commands, `temper box`, the setup's crates and the webhooks' listener (3 to 5, 10 to 14): not planned, the forge's setup; between hosts (TLS, contained process trees, a worker-only process): not planned; 1 to 6, 8 to 14: the client crates by the client naming rule, `temper-service` owning the clients' listener and mounting jig's clients' component: client-api; `temper ctl` and the tools' crates (4, 5): not planned, tools.md |
| `setup.md` | no | all: the modes' layers and their files, reconcile, owned's preparation and running, `temper box`, the configuration's `setup`, the crates, the tests and the facts of 13.3: forge-setup, in section 14's order; section 15, later: not planned |
| `testing.md` | partly | the domain and system worlds and the client's joint world (6.3) stand; 2.6 on owned mode with CI, webhooks and the hinted story, the box's test, the fake forge's deliveries and the setup's routes (3.1, 4.2), reconcile's world: not planned, the forge's setup; section 2's tiers below the domain (2.2, 2.5 to 2.7), the fake forge's protocol layer (4.2) and the fake git program (4.3): protocol; a scenario's limits from the shipped profile in every world, fixture and process configuration (5.1), and the live story ending at its task's first terminal failure, with each run's summary (2.7): not planned; 2.1, 2.6: the client's tiers on `jig-client`: client-api; 2.6: the rebound-Host attack story, and the process tier's person finding the goal on the board: not planned (client-api's `later.md`); 2.6 and 6.3, the agents' tools' stories: not planned, tools.md |
| `tools.md` | no | all: not planned. Its order is its section 12: after client-api, jig's named key and read of one turn with skein's request table admitting a given key (section 13), then area 1 (5.5), `temper ctl` for a chat to a landed change, with which the QA loop starts; areas 2 to 4 as the QA loop reaches them; the MCP endpoint and the extraction into jig later (section 14) |
| `worker-domain.md` | no | the worker before jig's hub, superseded by `domain/worker.md` and jig's `hosts.md`; still cited by the worker's crates and worlds; to be removed: not planned |
| `domain/README.md` | yes | the names and maps stand; section 5 points at the protocol documents |
| `domain/core.md` | yes | built in `jig-core` and its children under temper's root; the document still describes primitives owned by jig; to keep temper's application side and cite jig for the rest: not planned |
| `domain/tasks.md` | yes | built as `jig-core-tasks`; the document still names temper's crates; to keep temper's application side and cite jig's `tasks.md` for the rest: not planned |
| `domain/authority.md` | yes | built as `jig-core-authority`, landing rules translated to the forge's requirements; the same move: not planned |
| `domain/connectors.md` | partly | the contract is in `jig-core` and the forge meets it; section 14's second connector, test environments: not planned |
| `domain/engine.md` | partly | the root in jig's reference shape on `jig-core` and skein-lib's journal, notes in the store; 7.3's undecodable host call relayed with no commit, section 8's failed preparation reported as a fact of the attempt, section 14 below the domain: protocol; section 9's gather cutting a section in its kind's order, a CI section from its start: forgejo-17; section 16's saved work (the root names no saved-work branch in an assignment, so no run saves): not planned |
| `domain/forge.md` | partly | the connector's four crates on the fake forge; section 4's adoption request, locate and the forge's host, and 17's protocol: protocol; 8.4's brief with the failed step and the end of its log: forgejo-17; sections 13 and 14 beyond the `Fork` role and participation news: not planned; section 4, a configured org's repository adopted with no collaborator step, 5.2, hints from the forge's deliveries and what Forgejo settles after it delivers read again within `settle`, and 8.2's wait on an unsettled mergeability: not planned, the forge's setup |
| `domain/people.md` | partly | `jig-core-people` and the root's routes; section 11, signing in by the deployment's link: protocol; through the forge: not planned |
| `domain/agent.md` | partly | the typed translation to smith (`temper-engine-smith`), the worker hosting smith, the system world; section 3's agent shell as temper's `agent` command, section 9's protocol tests and the charter's encoding: protocol |
| `domain/worker.md` | partly | the worker root on jig's hub (`jig-domain-host`), the checkout and smith's host domain, with whole-worker worlds; section 9 below the domain, its parts naming repositories by tag: protocol; section 11's contraction (a missing base branch created, a push with no expected head): not planned |
| `store/README.md` | no | all: temper's families on jig's store: protocol |
| `web/` | no | W1 to W4's crates retired by `client.md`, section 9 (protocol 11.4); `web/architecture.md`: temper's browser, `temper-client-shell-web`, on skein's `skein-ui-web` and jig's `jig-client-shell-web`, its sign-in by the printed link's page and cookie, its tier with jig's cross-site and framed-page stories: not planned; `web/mockups/`: the pages `client.md`, 3.5 builds: Inbox, Chats, Chat, the task's page and the adoption form: client-api; the board, the changes page and an agent's task page: not planned (client-api's `later.md`); `web/ux/`, the forge's family's pages: not planned. `web/ux/` predates jig's client and is partly superseded by `client.md`, 3.5 and the mockups |

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
  process trees for agents and git), the forge's webhooks from another
  host (`forge.md`, section 5), the forge as people's sign-in provider (`domain/people.md`,
  section 14), GitHub (`forge.md`, section 11). Wait for a deployment
  beyond one person's machine, and for skein's TLS server side and
  contained process trees (jig's `protocol/README.md`, section 10).
- **The browser** (`web/architecture.md`; `client.md`, section 11):
  temper's browser shell, `temper-client-shell-web`, on skein's
  `skein-ui-web` and jig's `jig-client-shell-web`, served by
  `temper-service`, with its page's policy and jig's cross-site and
  framed-page attack stories. Waits for the client-api plan, which
  builds jig's browser step, and temper's next plan of its client
  (below).
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
- **smith's parity changes, for temper's next repin of smith** (smith's
  parity plan, its `later.md`; skein's development.md, 5.7): smith's
  transcript version 4, and the tools domain's new `Call` and `Outcome`
  variants, for jobs, which temper's three generated crates and its
  users of `smith-domain-tools` follow. Against smith's `main` of
  2026-10-15, temper's own code fails with 10 errors
  (`smith_transcript::v3`, `Request::Job`, `Turn`'s `jobs`). client-api
  moves temper to jig's pin of smith, `29fa97c9`, before them. Waits for
  the plan that moves temper past it.
- **temper's client beyond client-api** (`client.md`, 3.5, 3.6, 10;
  `configuration.md`, 7; `testing.md`, 2.6; `web/mockups/`): the board
  and reordering goals, the changes page, an agent's task page, a goal's
  page's history, budget, authority and acts, the confirmations no
  present story makes, the rebound-Host attack story and the process
  tier's board story, the `documents.lines` override, a chat's proposal
  shown at the call that made it (jig's turns name no call's objects
  yet). The client-api
  plan builds the rest of temper on jig's client API, and the three
  changes skein-ui's `later.md` left for temper's repin (temper 01);
  these wait for temper's next plan of its client (client-api's
  `later.md`). `temper-service/src/family/refusals.rs` copies jig's
  private translation of refusals; client-api's temper 04.2 removes it
  if jig 14 makes that translation callable, and otherwise it stays
  until temper takes jig's typed answers. `draft/mvp.md`, section 6
  then needs revising: it plans temper's client on jig's pages.

## 4. Drafts

| Draft | About | Next |
|---|---|---|
| `mvp.md` | the first milestone: the client against its design, and what stands between the `protocol` plan and a change landed for real | discuss; its section 6 names the plan after `protocol` |
| `factory.md` | the software factory: write, prove and run planes, the lab, on jig and os | not adopted: discuss; its section 13 changes `qa.md` |
| `qa.md` | continuous QA on a running build of main | not adopted: discuss; its order of building (section 8) waits for a running build |
| `ci-on-forgejo-17.md` | Forgejo v17's Actions beyond what the design adopts: a failure the runner caused run again instead of repaired, a stall's reasons, the agents' step and search log tools | not adopted; its order is its section 7: re-runs first, after the plan that moves temper to v17 |
| `llm-setup.md` | how a person makes temper's agents run on their LLM subscriptions and keys: the configuration's providers, signing in to ChatGPT from the client, Claude through Claude Code and `temper program-login`, keys, several subscriptions, models chosen per task, the commands, and live tests on a subscription; after jig's `draft/llm-accounts.md` | not adopted: review; becomes design after jig's accounts are designed, in its section 10's documents |
