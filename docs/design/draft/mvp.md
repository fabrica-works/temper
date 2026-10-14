# The first milestone

Draft, 2026-10-14. What temper's first milestone is (the smallest
product that works end to end, for real), how far the client is from
its design, what the `protocol` plan gives the milestone, and what no
plan gives it yet. It is an assessment of temper at `main` 95622baa,
with jig pinned at `d91a89c4`, and nothing here is adopted:
`client.md` and the documents it cites describe the client. What is
still open is listed in section 7.

## 1. In one page

- **The milestone:** one person, on their own machine, with a terminal,
  a real Forgejo repository and a real model. They adopt the
  repository, ask a chat for a change, see the change on its way, approve
  it, and see it land (section 2).
- **The client's domain layer is on its design, for what is built.**
  temper's client root and view (`temper-client-domain`,
  `temper-client-view`) are thin by design: jig's client domain and view
  hold the pages, the cards and every act but adoption. What is built
  matches `client.md`, sections 3 and 5; the adoption form, the terminal
  and `temper open` are the `protocol` plan's steps still to land
  (section 3).
- **jig's client does almost everything the milestone needs:** the
  inbox, conversations, the task page with each committed turn, goals,
  decisions, approvals as person tasks, policy and roles. It has no
  terminal (temper's), no live text (smith's channel carries none), and
  no forge parts (temper's, section 4).
- **The `protocol` plan is the milestone's spine.** When it ends, the
  binary serves, an agent runs, a person signs in through `temper open`,
  adopts a repository, and a change lands on a throwaway Forgejo with
  real git (section 4).
- **Three gaps stand between that plan and the milestone** (section 5):
  1. **A change is invisible in the client.** Every forge part but a
     word says "Shown on the forge.", with no link: the person cannot
     find the change, follow it, or see it land from temper.
  2. **A person's approval has no proven path.** The engine judges
     approvals from Forgejo reviews by a person whose identity is the
     forge user, under provider 0; nothing says how an owner signed in by
     the link gets that identity, and no story proves it.
  3. **Nothing has run for real.** Every tier runs on the throwaway
     Forgejo, and the real model only in the live tier, which stops at
     the push. No document says how a person sets temper up.
- **Not for the milestone:** the browser, a run's live text, the forge's
  panels in full (queues, health, drift), webhooks, GitHub, several
  people, findings, recurring tasks (section 6).

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
5. The person sees, in the client: the change's task, its step in words,
   its pull request (named, with a link to it on the forge), its head and
   its CI.
6. The person approves the change: on the forge, as a review, or in the
   client (section 5.2); asking for changes sends the change back for a
   repair.
7. The change lands, and the person sees it landed: the chat's update,
   the merge commit named.
8. `temper serve` is stopped and started again; nothing the person saw
   is lost, and work in flight goes on.

Steps 1 to 4 and 8 are the `protocol` plan's process tier (`testing.md`,
2.6), on a throwaway Forgejo, with no approval and no CI. Steps 5 to 7
are this draft's gaps.

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
- **Its process tier** (`testing.md`, 2.6) is section 2's steps 1 to 4
  and 8 against the throwaway Forgejo: no Actions runner, the project's
  landing rule with no CI, no gate and no approval, a fake LLM peer. Its
  live tier (2.7) adds a real model, and stops once the change is
  pushed.
- **What it does not prove,** and the milestone needs: CI from Forgejo's
  Actions in a run (decoded from transcripts, judged in the worlds on
  the fake forge's CI); a person's approval anywhere outside the
  engine's domain world; `temper sign-in` against a real provider (it
  runs against skein's fake issuer); a forge over TLS; a real
  repository's size and history.

## 5. The gaps

### 5.1 A change in the client

- **Today:** a change's task shows on jig's task page, its turns and its
  delegates, but its panel, its result and its effects are the forge's
  parts, and every one says "Shown on the forge." An adopted repository
  is "repository ID on forge F". The person learns the pull request's
  number only from the agent's words, and has no link to it.
- **The least that serves the milestone:** one read family of the
  forge's in `temper-protocol-client-wire`, beside `Adopt`, for a
  change's panel and result: its step in `web/ux/forge.md`, 2.1's words
  (producing to landed, repairs and updates among them), or held and
  why; its pull request's number, owner, name and address on the
  forge; its head; CI's state at that head; the gates' verdicts; and,
  landed, the merge commit. The forge's view draws it as words and one
  external link, in the task page's panel slot, the chat's delivery
  line and the inbox's update. A repository's owner and name in place of
  its numbers (`client.md`, section 11) comes from the same reads.
- **Not needed for it:** queues, health, drift, holds' details beyond
  their reason, the outbox, the deployment's forge parts.

### 5.2 A person's approval

- **Today:** a landing rule's `approvals` (`configuration.md`,
  section 3) is judged by the forge connector from the pull request's
  reviews. The root maps each review's author to a person by the
  identity provider 0 and the forge user's id, and counts approvals by
  people holding the rule's role (`domain/forge.md`, 8.3). The engine's
  domain world proves the judgement. Nothing says that an owner's
  configured identity must be provider 0 and their forge user's id for
  their reviews to count, the standalone example names no approval, and
  no story approves through the client or a real Forgejo.
- **The cheapest path for the milestone:** approve on the forge. The
  configuration names the owner by their forge identity; section 5.1's
  link takes the person to the pull request; the process tier gains a
  story with one approval required, given as a Forgejo review by the
  owner's forge user.
- **The path the design names:** a person task with jig's `Approval`
  contract (`domain/people.md`, section 8; jig's objects.md, section 7),
  shown in the inbox with the change's panel (section 5.1) and answered
  in the client. temper keeps a verdict of its own instead (STATUS.md,
  "Not planned"). It needs section 5.1 first, since an approval of a
  head the person cannot see is no approval (`web/ux/inbox.md`, 4.4).
- **Asking for changes** is open either way: a review that requests
  changes refuses the approval criterion, but no document says that its
  remarks become a repair's brief, as an agent gate's do
  (`domain/forge.md`, 8.3 and 8.4).

### 5.3 Running for real

- **A person's setup:** `configuration.md` is the reference, and its
  example is complete, but no document walks a person through a first
  deployment: a Forgejo user and tokens for temper, the owner's
  identity, the project's landing rule, the git identity, the LLM
  account and `temper sign-in`, then `temper serve` and `temper open`.
- **A dogfood run:** section 2's story once, by hand, on a real Forgejo
  with Actions and a real repository, with the findings written as
  world stories, fake corrections or charter changes (`draft/qa.md`,
  section 1, says why). It is evidence, not a tier.

### 5.4 The story in the worlds

`web/ux/README.md`, section 8, walks a change from a chat to its
landing; no current document does. `tests/client` stops at a chat
answered and, after plan 15.2, a repository adopted; the system world
lands changes without the person's view. Section 2's steps 4 to 7
belong in `tests/client`, joined to the engine's root, and in the
process tier once sections 5.1 and 5.2 exist.

## 6. In what order

1. **The `protocol` plan to its end.** Nothing else serves the milestone
   sooner, and its chain is long.
2. **A plan for sections 5.1, 5.2 (on the forge), 5.4 and 5.3's setup
   document,** temper only, on the pattern of the adoption: a design pass
   on `client.md` (the forge's read family, its words and slots), the
   configuration's identity rule, then the records, the engine's reads,
   the view, and the stories in `tests/client` and the process tier.
3. **The dogfood run** (5.3), its findings into the worlds.
4. **After the milestone,** each waiting for what its document names:
   approving in the client (5.2), the browser, a run's live text, the
   forge's panels in full, webhooks, GitHub, several people, repinning
   jig, smith and skein.

## 7. Open questions

- **Where the person approves for the milestone:** on the forge, or in
  the client through jig's `Approval` person task. Default: on the
  forge.
- **Asking for changes:** whether a person's review that requests
  changes becomes a repair with its remarks, or holds the change.
  Default: a repair, as an agent gate's request does.
- **CI for the milestone:** Forgejo's Actions with a runner, or the
  project's checks on the worker. Default: Actions, since a real
  repository has it.
- **The milestone's model:** the account `temper sign-in` signs in,
  against which provider. Default: the provider the live tier uses.
