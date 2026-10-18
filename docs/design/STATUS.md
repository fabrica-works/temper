# Status

Updated 2026-10-18. How far temper's code is from its design, and which
plans take it there. The designer keeps this file (skein's
development.md, section 3.3).

## 1. Plans

| Plan | State | Takes the design to | Where |
|---|---|---|---|
| client-api | active on carbon-forge since 2026-10-16 | temper on jig's client API, as far as temper's gate needs (the plan's `decisions.md`, 1(c)), beside jig's six steps of "jig's client API" and skein's harness (skein 01): its client crates renamed by the client naming rule, with jig's check of it; its terminal on `skein-ui-text` over `jig-client-shell-native`, the credential kept by its host and given back with `Held`; its client root on `jig-client` with two owners, the forge's request as bytes and `Owner` in its wire crate; `templates` in its configuration; the forge's family on the collapsed wire; `temper-service` owning the clients' listener and mounting jig's component under `/jig`, `application.clients`; the pages its stories use after the mockups (Inbox, Chats, Chat, the task's page, the adoption form) with their confirmations; `tests/client` on `jig-client-world` with jig's check; the three changes skein-ui's `later.md` left for temper's repin (temper 01); temper pinned to jig's last landing of the plan | `docs/development/plans/client-api/` |
| forge-setup | done 2026-10-18 | `protocol/setup.md`, sections 1 to 13: skein's lib with SHA-256, `sha256::Hmac` and `sha256::equal` (skein's `lib.md`, 6.2), skein's three copies on it; the setup's documents and facts both sides (admin users, the bot's tokens, orgs, teams, members, org hooks, Forgejo's command line and the runner, the deliveries); signed deliveries from the fake and the component's listener, `application.webhooks` (`protocol/forge.md`, 5); `settle` (`domain/forge.md`, 5.2); `temper-forge-setup` and `temper forge prepare\|run\|reconcile`; the process tier on an owned Forgejo with CI and webhooks, the hinted story; `temper-box` and `temper box`; the owned example and the setup guide; the upgrade | `docs/development/plans/forge-setup/` |
| forgejo-17 | done 2026-10-16 | Forgejo 17.0.0 and forgejo-runner 13.2.0 pinned (`protocol/forge.md`, 1, 10, 13), the conformance's and git's transcripts and facts recorded on them; v17's job with its steps and a step's log by a suffix range, the comparisons' `files` and `verification` (3.2); the fake forge's jobs with steps (9); `Read::Job` answering the failed step and its log's end, in the agent's `read_job` tool; the repair's brief naming them (`domain/forge.md`, 8.4); temper's terminal on skein's print by its own tree (`client.md`, 6.8 and 10; skein's `ui-text.md`, 3 and 4) | `docs/development/plans/forgejo-17/` |
| protocol | done 2026-10-14 | the protocol layer, standalone (`protocol/`): the codecs, `temper-service`, the binary's commands, the client root and view and the terminal, the forge component and its worlds, the git component, the one-process world, the process and live tiers; the engine worlds on jig's store; the web crates retired | `docs/development/plans/protocol/` |
| web-implementation | done 2026-10-06 (W1 to W4; W5 to W10 dropped 2026-10-14) | the web's first slice, in two crates `client.md` has since replaced | `docs/plans/web-implementation/` |

## 2. The design

| Document | Built | Left |
|---|---|---|
| `README.md` | partly | 2, steps 6 and 7 of the first milestone: a change's result and why it stopped, in the client in the forge's words: not planned (`web/ux/forge.md`, 2.5) |
| `domain/README.md` | yes | nothing; its decisions stand in the code |
| `domain/root.md` | partly | 6: a change's delegates' commits as text, in lowercase hexadecimal: client-api (`temper-repin`); 7.1: a saved-work branch named in an assignment (the root names none, so no run saves): not planned; 5.3: the forge's documents and reads, answered as unknown: not planned (`web/ux/forge.md`); 6: the configuration the root's construction asserts (the escalation role, the resume limit against a row's bytes, each charter's run against its bytes, the landing rules' and permission mappings' bytes, the judges' tables) refused at start rather than asserted, and a start refusing `fleet.engine_slots` above 0 or a charter not placed on the workers (2): not planned |
| `domain/forge.md` | partly | 8.3: a change's own gates as its parameters, amended while live; person gates (`GateKind::Person` is never made); a review's lens and a change's own charters (every forge task runs under the forge's one charter); a gate's verdict posted as a commit status: not planned; 8.1: a change's own bounds (every change takes `forge.change_policy`'s): not planned; 8.5: a release that retargets a pull request back: not planned; 4: a repository of an org a setup names adopted with no collaborator step: not planned; 12: findings' state in the store: not planned; 13 and 14 beyond the `Fork` role and participation news: not planned |
| `domain/worker.md` | partly | 4.2, 5: the worker creating no branch (the checkout still creates a missing base branch): not planned; 4.1: an assignment naming an expected head (the engine names none): not planned; 5: saved work, which waits for the root to name a saved-work branch (`domain/root.md`, 7.1): not planned; 9: agents in contained process trees: not planned (after standalone) |
| `domain/agent.md` | partly | 4.3: `delegate`'s answer naming each spec's revision and what an `AsAllowed` budget was given; `decide`, `withdraw` and `decide_escalation` saying how they decided; 4.4: a verdict's follow-ups; a chat's `deliver` before its end; 5: a message's sender labelled by name: not planned |
| `protocol/README.md` | partly | 3 to 5: `temper ctl`, `temper-client-tools` and `temper-client-shell-cli`: not planned (`tools.md`); 5: `jig-client-view` no longer a dependency of the client crates and the binary: client-api; 14: between hosts: not planned (after standalone) |
| `protocol/configuration.md` | partly | 7: the worker's check against the agent naming both limits (it names `workers[0].limits` alone): not planned (forge-setup's `later.md`); 7: a start refusing `forge.resources_per_task` below `checkout.repositories`: not planned (forge-setup's `later.md`); 3: a git identity's number held within jig's `accounts.credentials` at start (a grant past it is dropped on the worker): not planned; 11: `small`'s worst cases within a person's machine: not planned (forge-setup's `later.md`); 3, 8: `application.clients` (the listener), `templates` in place of `chats`, 7: the example's bounds on a name's segments, `documents.lines` for short lists alone, an authority's lists carried whole and held to temper's client's entries, the engine's watch rooms, 16 a party and 16 a stream, held to temper's client's watches: client-api |
| `protocol/credentials.md` | partly | 6: the terminal's credential kept by its host and given back with `Held`: client-api; 8: the forge as people's sign-in provider: not planned |
| `protocol/llm.md` | yes | nothing but its open questions |
| `protocol/channel.md` | yes | nothing but its open questions |
| `protocol/store.md` | yes | nothing but its open questions |
| `protocol/forge.md` | partly | 1, 5: webhooks from another host, over TLS: not planned (after standalone); 9, 10: the protocol world's late, reordered and lost deliveries, its stories run with hints and without: not planned (forge-setup's `later.md`); 9: the fake forge's deliveries to the org hook reconcile made, in a world: not planned (forge-setup's `later.md`); 10: the idle cost check, its bound computed from the limits: not planned; 14: a waiting job's fact: not planned (forgejo-17's `later.md`); 14: GitHub: not planned |
| `protocol/git.md` | yes | nothing but its open questions (containment between hosts, section 10) |
| `protocol/setup.md` | partly | 15, later: GitHub's setup, the binaries' digests, webhooks from another host, CI's isolation, provisioning at adoption, reconcile over TLS, the lock and the exec on skein's shell: not planned (forge-setup's `later.md`) |
| `client.md` | partly | built: the forge's words and "Shown on the forge." (4.1, 4.2), its records (4.6). Left: the client root on `jig-client`, with two owners, temper's pages and the forge's side, routing by asker (3.1, 3.3), the pages its stories use after the mockups (Inbox, Chats, Chat, the task's page, the adoption form) and their confirmations, on jig's helpers (3.5, 3.6), their cards temper's own, from the documents and `jig-client`'s `facts`, jig's words for a decision's facts under temper's phrasebook (1, 3.5, 5.1), the forge's family as bytes and its form a page of temper's (4.3 to 4.7), `Owner` in the wire crate (4.6); the view on `skein-ui` (5); the terminal on skein's `skein-ui-text` over `jig-client-shell-native`, its line bytes, its host adding skein's terminal's worst case and operations to its own (6.2, 6.8), a path's segments apart from the rows, a long path shown cut (5.4), an authority's entries apart from the rows, 12, a long authority shown in part (5.4), keeping the credential's file through `jig-client-shell-native` and signing in again with `Held` (6, 7), the root started before the sign-in, jig's client opening no watch until signed in (3.4); `temper-service` owning the clients' listener, jig's default HTTP mapping under `/jig` with its example guards, the link route's `GET` answered not found (8); `tests/client` with jig's check, given temper's phrasebook (10); the crates `temper-protocol-client-wire` renamed `temper-client-protocol-wire` and `temper-shell-terminal` `temper-client-shell-terminal`, by the client naming rule; the client root, view root and native client copied from jig's retired, the banner's word for each link state and temper's main while signed out or out of date (5.1, 6.6), the adoption's title for a project `me` does not name (4.4), the kept work dropped said in a line of the terminal's (3.4, 6.6): client-api; 6.8, 10: the terminal on skein's print by its own tree, and made from `limits::native()`'s limits in a unit test: forgejo-17; the board, the changes page, an agent's task page, a goal's page's history, budget, authority and acts, the confirmations no present story makes (3.5, 3.6), the rebound-Host attack story in the process tier (10), a chat's proposal at the call that made it, once jig's turns name what their calls made (3.5; jig's `domain/engine.md`, 7.2): not planned (client-api's `later.md`, temper's next client plan). Not planned: the forge's parts beyond adoption (`web/ux/forge.md`), the browser, the forge as a sign-in provider through `Mint` (section 11) |
| `testing.md` | partly | the domain and system worlds, the client's joint world (6.3), the tiers below the domain (2.2, 2.5 to 2.7), the fake forge's protocol layer, deliveries and setup routes (4.2), the fake git program (4.3), owned mode with CI, webhooks and the hinted story, the box's test and reconcile's world (2.6) stand; a scenario's limits from the shipped profile in every world, fixture and process configuration (5.1), and the live story ending at its task's first terminal failure, with each run's summary (2.7): not planned; 2.1, 2.6: the client's tiers on `jig-client`: client-api; 2.6: the rebound-Host attack story, and the process tier's person finding the goal on the board: not planned (client-api's `later.md`); 2.4: the focused real-loop world on one ring, with hosted agents, native clients and fake peers, and its real-git bridge: not planned; 2.4, 2.6 and 6.3, the agents' tools' stories: not planned (`tools.md`) |
| `tools.md` | no | all: not planned. jig's named key and its read of one turn come first (section 13); then area 1 (5.5), `temper ctl` for a chat to a landed change; areas 2 to 4 as the QA loop reaches them; the MCP endpoint and the tools in jig later (section 14) |
| `web/architecture.md` | no | all: temper's browser shell, `temper-client-shell-web`, on skein's `skein-ui-web` and jig's `jig-client-shell-web`, its sign-in by the printed link's page and cookie, its tier with jig's cross-site and framed-page stories: not planned |
| `web/ux/forge.md` | partly | built: "Shown on the forge." and the adoption form (5.1). Left: the forge's read family and its subject records (2.5, 9), a change's result and why it stopped (2.5), the changes page and landing queues (3), approvals at a head (4), repositories' health, their landing rules in a project's settings and the deployment's forge parts (5.2 to 5.4), holds in the forge's words (6), a goal's issue, landings as news and findings (7): not planned; the changes page: not planned (client-api's `later.md`, temper's next client plan) |

`web/mockups/` is reference material for the client's look and the
order of its pages, not contract (`client.md`, 3.5).

## 3. Not planned

- **The forge in temper's client, beyond adoption** (`web/ux/forge.md`;
  `client.md`, section 4): the forge's read family and its subject
  records; a change's result and why it stopped, the least the first
  milestone needs (`README.md`, section 2, steps 6 and 7); the changes
  page and landing queues, approvals, repositories, holds in the forge's
  words, a goal's issue, landings as news, findings. Until then the
  forge's parts say "Shown on the forge." Waits for a plan of its own.
- **temper's client beyond client-api** (`client.md`, 3.5, 3.6, 10;
  `protocol/configuration.md`, 7; `testing.md`, 2.6; `web/mockups/`):
  the board and reordering goals, an agent's task page, a goal's page's
  history, budget, authority and acts, the confirmations no present
  story makes, the rebound-Host attack story and the process tier's
  board story, the `documents.lines` override, a chat's proposal shown
  at the call that made it (jig's turns name no call's objects yet).
  These wait for temper's next plan of its client (client-api's
  `later.md`). `temper-service/src/family/refusals.rs` copies jig's
  private translation of refusals until temper takes jig's typed
  answers.
- **The browser** (`web/architecture.md`; `client.md`, section 11):
  temper's browser shell on skein's `skein-ui-web` and jig's
  `jig-client-shell-web`, served by `temper-service`, with its page's
  policy and jig's cross-site and framed-page attack stories. Waits for
  temper's next plan of its client.
- **The real-loop world** (`testing.md`, 2.4; `tools.md`, 11.3):
  a small focused suite on one ring, with the standalone service, hosted
  smith agents, native clients and fake peers. Chat, waits, persistence
  and teardown first; clone, push and landing wait for the real-git
  bridge (testing.md, section 10). Waits for a plan of its own.
- **The agents' tools, `temper ctl`** (`tools.md`): waits for jig's
  named key and its read of one turn (`tools.md`, section 13).
- **After standalone** (`protocol/README.md`, section 14): between hosts
  (TLS on the engine's listeners, a worker-only process, contained
  process trees for agents and git), the forge's webhooks from another
  host and reconcile over TLS (`protocol/forge.md`, section 5;
  `protocol/setup.md`, section 15), GitHub (`protocol/forge.md`,
  section 14). Waits for a deployment beyond one person's machine, and
  for skein's TLS server side and contained process trees (jig's
  `protocol/README.md`, section 10).
- **The forge as people's sign-in provider**
  (`protocol/credentials.md`, section 8): through jig's `Mint`. Waits for
  a plan of its own.
- **The change procedure's own contract** (`domain/forge.md`, 8.1, 8.3,
  8.5): a change's own gates and bounds as its parameters, amended while
  live; person gates and jig's `Approval` contract for them; a review's
  lens and a change's own charters; a gate's verdict posted as a commit
  status; a release that retargets a pull request back.
- **Participating objects and upstream contributions**
  (`domain/forge.md`, sections 13 and 14): outside contributions
  triaged and reviewed, changes landed by others' merges, submissions
  from a fork. Not needed first; they wait for a project that wants them.
- **Saved work and an expected head** (`domain/root.md`, 7.1;
  `domain/worker.md`, 4.1 and 5): the root names no saved-work branch and
  no expected head in an assignment, so no run saves and no push is
  held to a head; and the checkout still creates a missing base branch.
- **The agents' answers** (`domain/agent.md`, 4.3 to 5): the
  revisions and budgets a `delegate` made, how a decision decided, a
  verdict's follow-ups, a chat's delivery before its end, a sender
  labelled by name.
- **A failed preparation's words on the task's page**
  (`domain/root.md`, 7.4): the root reports a permanent preparation's
  repository and reason as a fact of the attempt; people read them on
  the task's page once jig's core keeps an attempt's failure detail.
- **A run's live text in the client** (jig's `protocol/channel.md`, 6.4):
  waits for smith's channel to carry it (jig's `STATUS.md`, "What jig
  asks of smith, later"); until then runs show their activity, and
  their words as each turn commits.
- **jig's charter and tools translations** (`protocol/llm.md`, section
  7): temper's routing keeps `temper-engine-smith` for the charter, host
  tools and results. Waits for the root's tools to be reconciled with
  jig's core tool events.
- **A calendar for periods** (`protocol/configuration.md`, section 11):
  one period per project at the first start until a calendar child is
  designed.
- **The start's checks** (`protocol/configuration.md`, sections 3 and 7;
  `domain/root.md`, section 6; forge-setup's `later.md`): the worker's
  check against the agent naming both limits, `forge.resources_per_task`
  below `checkout.repositories` refused, a git identity's number held
  within jig's `accounts.credentials`, the engine given no slots and its
  charters placed on the workers, and the configuration the root's
  construction asserts refused at start instead.
- **The owned example sized for a person's machine**
  (`protocol/configuration.md`, section 11; `protocol/setup.md`, 8.1 and
  section 16): `small`'s worst cases, and the example's agent, against
  what one person's machine and work need; its `views.` overrides, which
  the root does not take (it holds its watch rooms itself,
  `domain/root.md`, section 10).
- **A second connector** (`README.md`, section 8): test environments.
  Waits for continuous QA or the factory to be adopted (`draft/qa.md`,
  `draft/factory.md`).
- **Allocation under load:** allocator tuning and skein's byte budget
  (skein's `notes.md`), once `temper serve` runs real work.
- **smith's changed APIs** (smith's `domain/host.md`, section 11): the
  typed agent face, smith's inline agent, removed modes, which temper
  takes at its next repin of smith, after smith's reliability plan.
- **smith's parity changes, for temper's next repin of smith** (smith's
  parity plan, its `later.md`; skein's development.md, 5.7): smith's
  transcript version 4, and the tools domain's new `Call` and `Outcome`
  variants, for jobs, which temper's generated crates and its users of
  `smith-domain-tools` follow. Waits for the plan that moves temper's
  smith past them.

## 4. Drafts

| Draft | About | Next |
|---|---|---|
| `qa.md` | continuous QA on a running build of main | not adopted: discuss; where QA lives is open (`README.md`, section 8) |
| `factory.md` | the software factory: write, prove and run planes, the lab, the build daemon | not adopted: discuss. It crosses repositories, and hearth's design cites it as the factory hearth serves: where it lives is the user's to decide |
| `ci-on-forgejo-17.md` | Forgejo v17's Actions beyond what the design adopts: a failure the runner caused run again instead of repaired, a stall's reasons, the agents' step and search log tools | not adopted; its order is its section 7, re-runs first |
| `llm-setup.md` | how a person makes temper's agents run on their LLM subscriptions and keys: the configuration's providers, signing in to ChatGPT from the client, Claude through Claude Code and `temper program-login`, keys, several subscriptions, models chosen per task, the commands, and live tests on a subscription; after jig's `draft/llm-accounts.md` | not adopted: review; becomes design after jig's accounts are designed, in its section 10's documents |
| `forge-kit.md` | temper's side of skein's `draft/forge.md`: what moves to skein, what temper keeps, what its documents become, the top's seam for a later move, and the order | Review with skein's draft; promoted with it, before the extraction plan |
