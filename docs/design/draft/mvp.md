# The first milestone

Draft, 2026-10-14, revised 2026-10-16. What temper's first milestone is
(the smallest product that works end to end, for real), how far the
client is from its design, what the `protocol` plan gives the milestone,
and what no plan gives it yet. It is an assessment of temper at `main`
95622baa, with jig pinned at `d91a89c4`, and nothing here is adopted:
`client.md` and the documents it cites describe the client. What is
still open is listed in section 7.

## 1. In one page

- **The person directs; agents own the code.** People decide at the
  level of goals, designs and plans, and steer where agents cannot
  decide; agents write, review and land the code. A person's approval of
  a change is a workflow choice the person makes when giving direction
  ("every pull request of this plan waits for me"), never temper's
  default, and not part of the milestone (section 5.4).
- **The milestone:** one person, on their own machine, with a terminal,
  a real Forgejo repository and a real model. They adopt the
  repository, ask a chat for a change, follow the work at the chat's
  level, and see what landed; the change is checked by CI and an agent's
  review and lands with no person in between (section 2).
- **The client's domain layer is on its design, for what is built.**
  temper's client root and view (`temper-client-domain`,
  `temper-client-view`) are thin by design: jig's client domain and view
  hold the pages, the cards and every act but adoption. What is built
  matches `client.md`, sections 3 and 5; the adoption form, the terminal
  and `temper open` are the `protocol` plan's steps still to land
  (section 3).
- **jig's client does almost everything the milestone needs:** the
  inbox, conversations, the task page with each committed turn, goals,
  decisions and person tasks, policy and roles. It has no
  terminal (temper's), no live text (smith's channel carries none), and
  no forge parts (temper's, section 4).
- **The `protocol` plan is the milestone's spine.** When it ends, the
  binary serves, an agent runs, a person signs in through `temper open`,
  adopts a repository, and a change lands on a throwaway Forgejo with
  real git (section 4).
- **Two gaps stand between that plan and the milestone** (section 5):
  1. **Nothing has run for real.** Every tier runs on the throwaway
     Forgejo, with no CI; the real model runs only in the live tier,
     which stops at the push. How a person sets temper up is
     `setup.md`'s, on a box, not built yet.
  2. **What came of the work, and why it stopped, are not in the
     client.** The forge's results and holds say "Shown on the forge.",
     with no link: the person cannot see what landed, or why a change is
     held, which is where the person is needed.
- **Not for the milestone:** a person's approval of changes, the
  browser, a run's live text, the forge's panels in full (queues,
  health, drift), GitHub, several people, findings, recurring tasks
  (section 6). Webhooks come with the forge's setup (`setup.md`,
  section 10).

## 2. The milestone

One story, on one person's machine:

1. `temper serve CONFIG.json` on a deployment directory, configured for
   one Forgejo (with Actions, or with project checks), one git identity,
   one worker and one LLM account signed in by `temper sign-in`.
2. `temper open DIRECTORY LINK` signs the person in by the printed link
   and shows the inbox.
3. The person adopts a repository into the configured project through
   the adoption form (`client.md`, section 4).
4. The person starts a chat and asks for a change; the chat delegates
   the change procedure (`domain/forge.md`, section 8).
5. Agents do the rest: the change is produced, its pull request opened,
   checked by CI and by an agent's review (a gate, `domain/forge.md`,
   8.3), repaired when either fails, and landed. No person acts on it.
6. The person follows it from the chat: its delegates and their phases
   in words, and, if the change is held, why, with what may be done
   (released, amended, cancelled).
7. The person sees what landed: the chat's update names the pull
   request and the merge commit, with a link to each on the forge.
8. `temper serve` is stopped and started again; nothing the person saw
   is lost, and work in flight goes on.

Steps 1 to 5 and 8 are the `protocol` plan's process tier, on a
throwaway Forgejo with no CI, a project check in its place; on an owned
Forgejo with CI it is the forge's setup's (`testing.md`, 2.6; `setup.md`,
13.4). Running them for real, and steps 6 and 7, are this draft's gaps.

## 3. The client against its design

At `main` 95622baa; "plan" is the `protocol` plan, by step.

| `client.md` | State |
|---|---|
| 2, the crates | `temper-client-domain`, `temper-client-view`, `temper-protocol-client-wire`, `temper-shell-terminal` (`client`, `route`, `limits`): built. `terminal`, `render` and `command`, and the `temper` binary: plan 10.3, 16.1 to 16.4 |
| 3, the client root | built (11.1) with jig's child, the forge's side and the request table; jig's navigation configured. The forge's ask, owner and address (`Adopt`) are missing: plan 15.1a (running), 15.1b |
| 4.1, 4.2, the forge's words; "Shown on the forge." | built (11.2), in the view. The forge's side holds its parts per page generation |
| 4.3 to 4.5, 4.7, the adoption form | not built: plan 15.1b, 15.2 |
| 4.6, the records | built (03.2): `Adopt`, its outcome and its reasons in `temper-protocol-client-wire`. The routing that carries them: plan 15.1a |
| 5, the view root | built (11.2): jig's frame and pages, the forge's fragments in slots. The navigation slot is filled empty, and `act` decodes no binding of the forge's: plan 15.1b |
| 6, the terminal | the native client (12.1a, 12.1b) only. The renderer, the command reader, the loop and `temper open`: plan 16.1 to 16.4, 16.6 |
| 7, signing in and the small store | built in the native client, on `jig-shell-native` (12.1). `temper sign-in`, the LLM account's: plan 16.5 |
| 8, the engine's side | jig's clients' component and the forge family in `temper-service`: plan 08.5a, 08.5b |
| 9, what goes | done (11.4): the web crates, the fake person and the web world retired |
| 10, testing | `tests/client` stands (11.3, 11.4): signing in, a chat started and answered, a double press made once, a held chat released, a sign-out mid-request, and a fuzzy sweep. The adoption's stories, the terminal's world, the one-process world and the process tier: plan 15.2, 16.3, 17, 10.4, 12.2, 15.3, 16.4, 16.6 |

**What jig's client gives, at the pin.** Every page of jig's
`client/domain/pages.md` is built in jig's client domain and view: the
frame, the inbox, conversations, the task page with its six sections,
a project's goals, tasks, notes and settings, the deployment, and the
pages that are not about the work. A person can start a chat, write to
it, answer questions and person tasks (approvals among them), decide
proposals and escalations, stop, cancel, release and amend, set and
order goals, edit notes, and change policy, roles and pools. The task
page shows each committed turn: text, tool calls folded, results. jig
offers no request to adopt (an application's), to create a project
(projects come from the configuration), or to invite a person; a
project's resources and recurring tasks, and the deployment, are read
only.

**What it does not give.** A terminal: jig's native shell has no view,
and jig's view leaves a text shell to applications. A run's live text:
jig's client and view draw it, but nothing in production makes it,
since smith's channel carries facts without content (jig's STATUS.md,
"What jig asks of smith, later"). The forge's parts: panels, results,
effects, holds, a project's and the deployment's, which are temper's to
serve (section 5.1).

**The older web documents.** `web/ux/` predates jig's client; most of
it is now jig's `pages.md` and `objects.md`. Superseded there: signing
in through the forge as the first slice (`web/ux/README.md`, 9;
`chats.md`, 9), streams in place of level-triggered views (README, 2.1
and 7; `tasks.md`, 12), "offers to propose instead" (README, 5.3), two
requests where jig makes one (`inbox.md`, 9; `chats.md`, 10), and the
old names (Chats, Board, System, the web). What no current document
carries, and the milestone needs, is section 5.1's and 5.2's. The rest
(findings, a goal's issue and its landings as news, queues, health,
drift, amending a change's gates, recurring tasks, a goal's landing
scope, participation) waits for a later plan.

## 4. What the `protocol` plan gives the milestone

- **At this draft's date,** 57 of 108 steps have landed, 3 run. The
  longest chain left is the engine's service, the worker's service, the
  binary, a chat answered by the binary, a change landed, and adopting
  by commands (08.4, 08.5 → 09 → 10 → 12.2 → 15.3 → 16.6).
- **Its process tier** is section 2's steps 1 to 4 and 8 against the
  throwaway Forgejo: no Actions runner, the project's landing rule with
  no CI, no gate and no approval, a fake LLM peer; the forge's setup puts
  it on an owned Forgejo with CI (`testing.md`, 2.6; `setup.md`, 13.4).
  Its live tier (2.7) adds a real model, and stops once the change is
  pushed.
- **What it does not prove,** and the milestone needs: CI from Forgejo's
  Actions in a run (decoded from transcripts, judged in the worlds on
  the fake forge's CI); a real model's change through to its landing;
  `temper sign-in` against a real provider (it runs against skein's fake
  issuer); a forge over TLS; a real repository's size and history.

## 5. The gaps

### 5.1 Running for real

- **A person's setup** is `setup.md`'s (section 12): the document
  written from the owned example (`configuration.md`, section 10), with
  the project's landing rule (CI and an agent's review) and the LLM
  account; `temper sign-in`; `temper box`, which runs Forgejo, its
  runner, reconcile and `temper serve`; a repository with Actions made in
  an org on Forgejo's web, then adopted through `temper open`. What is
  left is the setup document that walks a person through it, written on
  the box.
- **A dogfood run:** section 2's story by hand, on a box, with Actions on
  the owned runner and a real repository, with a real model, through to
  the landing. Its findings are written as world stories, fake
  corrections or charter changes (`draft/qa.md`, section 1, says why).
  It is evidence, not a tier.

### 5.2 What came of the work, and why it stopped

- **Today:** a change's task shows on jig's task page, with its turns and
  its delegates, and a held task shows jig's hold card. But the
  change's result, its effects, its panel and its hold are the forge's
  parts, and each says "Shown on the forge." An adopted repository is
  "repository ID on forge F". The person learns what landed only from
  the agent's words, and why a change is held not at all: they can
  release it only blind.
- **The least that serves the milestone:** one read family of the
  forge's in `temper-protocol-client-wire`, beside `Adopt`, for a
  change's result and its hold: landed, with its pull request's number,
  its merge commit, and the address of each on the forge; held, with its
  reason in `web/ux/forge.md`'s words (repairs or resolutions spent, a
  stall, a refusal) and the head and failure it is about. The forge's
  view draws them as words and links, in the slots of the task page,
  the chat's update and the inbox's entry. A repository's owner and name
  in place of its numbers (`client.md`, section 11) comes from the same
  reads.
- **Not needed for it:** the change's way to landing step by step,
  queues, health, drift, the outbox, the deployment's forge parts.

### 5.3 The story in the worlds

`tests/client` stops at a chat answered and, after plan 15.2, a
repository adopted; the system world lands changes with no person's
view. Section 2's steps 4 to 7 belong in `tests/client`, joined to the
engine's root: a chat's change landed by agents, its result read in the
chat, and a held change's reason read and released.

### 5.4 A person's approval, as a workflow choice

Not a gap, and not the milestone's; noted so that it stays the
person's choice. The design already lets direction carry it: a change's
gates are its parameters, asked for by the change, the project and the
plan (`domain/forge.md`, 8.3), and one kind of gate is a person's
approval. So a plan given "every pull request waits for my approval"
adds that gate to its changes. The configuration's `approvals` member
(`configuration.md`, section 3) sets the same for every change into a
branch, and the standalone example leaves it empty. When used, the
engine counts a Forgejo review only from a person whose identity is the
forge's user id under provider 0, which `configuration.md` does not yet
say.

## 6. In what order

1. **The `protocol` plan to its end.** Nothing else serves the milestone
   sooner, and its chain is long.
2. **The forge's setup** (`setup.md`, section 14), after it: webhooks,
   owned's preparation, reconcile, `temper box` and the owned example.
   **The dogfood run** (5.1), on the box, with the setup document
   written as it goes. Its findings feed the next plan.
3. **A plan for 5.2 and 5.3,** temper only, on the adoption's pattern: a
   design pass on `client.md` (the forge's read family, its words and
   slots), then the records, the engine's reads, the view, and the
   stories in `tests/client` and the process tier.
4. **After the milestone,** each waiting for what its document names:
   the browser, a run's live text, the forge's panels in full, GitHub,
   several people, repinning jig, smith and skein; and the
   person's own workflow, from a brainstorm through a design and a plan
   to its supervised run, as temper's goals and plans.

## 7. Open questions

- **CI for the milestone** is answered by `setup.md`, section 12:
  Actions on the owned runner.
- **The milestone's model:** the account `temper sign-in` signs in,
  against which provider, and its registration (authorization URL,
  redirect, scope, client). Default, until it is settled: the owned
  example hands its account in, as the live tier does with a borrowed
  Codex login (credentials.md, section 3; testing.md, 2.7), with the
  live tier's model.
- **Where a person's approval is set** (5.4): as a gate the plan adds to
  its changes when the person asks, or the configuration's landing
  rule. Default: the plan's; the configuration's `approvals` stays
  empty.
