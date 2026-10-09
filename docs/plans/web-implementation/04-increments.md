# Increments

Provisional, 2026-10-05. The increments of README.md, section 7, still
to build (W1 to W4 are built), in order: what each lands in the domain,
the view and the worlds; the stories that show it; what its joint-world
stories need of the engine; the design documents it updates. Each is a
branch that passes the gate and the review of README.md, section 6, and
adds no vocabulary it does not use.

## W5 The joint world

**Lands** `tests/web/engine` (03-testing.md, section 5): the translation,
the first slice's stories against the real root, the referee unchanged,
`fuzzy_slice.rs`.

**Needs of the engine,** beyond the routes it has (README.md, section 5):

- **who is signed in,** as a watch: the person, their projects and roles,
  the inbox's counts (zero until the inbox is built), for the frame;
- **a task's watch,** or at least its read: its chip, header and phase,
  its held escalation, its result once ended, for the chat's page.

W5 is scheduled when the engine has both. Should they come late, the
increment may start with the client's requests and reads alone, and the
frame's snapshot made by the translation from the root's sign-in answer
and the configured projects: a stand-in in the world, named as such, and
deleted when the route lands.

**Updates** `ux/README.md`, section 10, with anything the joint world
finds missing that it does not list.

## W6 The inbox

`ux/inbox.md`. **Lands** `Page::Inbox` (needs you, oldest first; updates,
newest first; filters by project, kind and goal), the proposal,
question and person task objects and cards, the refusal card,
`Ask::Decide` for proposals (accept, reject with a reason, pass up),
`Ask::Answer` (a choice with why, words; approvals at a head wait for
W10), `Ask::Read` for the read position, `Watch::Inbox`, `Query::Inbox`
for what is past the engine's limit, `Action::Choose`, `Filter` and
`More`, and the frame's counts. Home becomes the inbox.

- **Stories:** a proposal accepted in the inbox leaves the chat open in the
  other tab (one object, one card); rejected with a reason; a choice
  answered with why, its two reports side by side; decided elsewhere first
  (another tab, a task above), the card saying by whom and leaving; refused
  for a pool since spent, staying with its reason; beyond the person's
  authority and proposable, offering to propose; a proposal passed up
  while read; updates marked new until opened, the read position moving;
  paging past the window.
- **Needs of the engine:** the inbox as a watch, across projects
  (`ux/inbox.md`, section 9); deciding proposals; answering questions and
  person tasks; the read position.

## W7 The conversation

`ux/chats.md`, sections 4 to 8. **Lands** the transcript
(`task/transcript.rs`: a window of turns from the end, runs and their
headings, the provisional turn from `Watch::Run`, folds), words and
their states (sending, sent, read), each task's composer and its
draft, inline cards (a delegate's
chip, a proposal, a result), the chat's line in words, its spend;
`Ask::Write`, `Stop`, `Release`, `Close` (with each live delegate's
disposal), `Cancel`; words then release for a stopped chat, as two
requests; `Query::Transcript`. In the view, the conversation and the
activity-led page of `agent-task.html`.

- **Stories:** words sent, then sent, then read; a reply streamed, then
  replaced by its committed turn; a worker lost mid-turn, its streamed
  text gone; stopping a chat, writing, releasing; closing a chat with a
  live delegate, cancelled or made the person's; cancelling, told what it
  ends; scrolling back through earlier pages; typing in the composer while
  turns stream in, its value never written over (the model DOM shows no
  `Value` patch).
- **Needs of the engine:** a run's watch; transcripts read a page at a
  time; writing, stopping, releasing, closing and cancelling a task;
  making oneself a delegate's requester.

## W8 The task page

`ux/tasks.md`. **Lands** the universal page's parts: header with its
lineage, overview, activity grouped by run, the plan as a tree
(`task/tree.rs`: folds, counts per phase, held below, held only and live
only), history for the task or its subtree, budget and authority; which
requests fit the phase (`task/steer.rs`); `Ask::Amend` with its before
and after, `TakeOver`, amend then release; release by each hold's reason;
`Watch::Tree`, `Query::History`, `Query::Ended`. In the view, the
plan-led page of `task.html`.

- **Stories:** a task held for each reason of `ux/tasks.md`, section 7,
  saying why and offering what fits; amending, then releasing; a plan
  with many tasks folded into counts, a held task deep down counted on
  its folded line; an ended branch opened and loaded; history paged, for
  the task and its subtree; taking a delegate over before cancelling its
  parent; where a budget went.
- **Needs of the engine:** a tree's watch; history and ended delegates
  read a page at a time; amending and taking over; funders' numbers.

## W9 The board

`ux/projects.md`, section 3. **Lands** `Page::Board` (the period's
numbers, goals by priority, live, waiting and recently ended), raising
and lowering a goal as a pending request (`Ask::Prioritise`), setting a
goal (`Ask::SetGoal`, a form with `Action::Number`, beyond the person's
pool said before it is sent), `Watch::Goals`.

- **Stories:** a goal raised, pending, then in place; a goal set within
  the pool; one beyond it, refused with what is left, or proposed; the
  board's filters; a goal's held count following its plan.
- **Needs of the engine:** a project's goals as a watch; prioritising;
  setting a goal.

## W10 The forge's panels

`ux/forge.md`. **Lands** `forge.rs` (01-domain.md, section 9): a change's
body for its card and its panel (its way to landing, pull request, CI,
gates and the head each was given at, heads, bounds); `Page::Changes`
(each landing branch's queue and rule, not ready, held, the branch's
health, landings paged); the task page's `Panel` with its forge variant;
approvals as person tasks at a head, replaced when the head moves;
amending a change's gates; drift's card and what release does;
`Watch::Changes`, `Query::Landings`.

- **Stories:** a change followed step by step to landed; an approval at a
  head; the head moving while the card is open, the card saying it was
  replaced; the queue's order following a goal raised on the board; a
  paused queue and its repair; a drifted change released.
- **Needs of the engine:** the forge connector's reads of changes, queues
  and branches (`domain/forge.md`); a project's changes as a watch;
  answering approvals.

## After W10

A project's tasks, findings, notes, recurring tasks, settings and spend,
and the system's page (`ux/projects.md`, sections 4 to 8), planned here
once the board is built. Several of their requests are not in the domain
yet (`ux/README.md`, section 10), so their plan starts with what the
engine will offer.
