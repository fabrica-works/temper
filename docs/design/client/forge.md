# The forge in temper's client

Provisional, 2026-10-05, revised 2026-10-18. What the forge connector
adds to temper's client: a change's way to landing and its result, a
project's landing queues, approvals at a head, the forge's holds in
words, repositories with their health and landing rules, goals,
findings and landings as the forge shows them, and the forge's part of
the deployment. jig's pages and cards (jig's `client/domain/pages.md`
and jig's `client/domain/objects.md`) leave a slot for each part a
connector shows (jig's `client/domain/root.md`, section 6), and
temper's pages and cards, its own after them, show the same parts
(`README.md`, section 1); this document says what the forge shows in
those slots and on the pages only temper has, `Changes` and `Adopt`
(`README.md`, 3.5), in temper's words. The forge's client family, its
words for resources and kinds, and its adoption form are `README.md`,
section 4. Each part shows what `../domain/forge.md` keeps or reads,
and links to the forge for code. What is still open is listed in
section 10.

## 1. In one page

- **In jig's slots, in the forge's words.** jig's pages show tasks,
  cards and holds; the forge fills the parts they name (a panel, a
  result, an effect, an outbox entry, a hold, a project's part, the
  deployment's part) on a task's page, in a chat, in the inbox, on a
  project's pages and on the deployment's (section 9). A part the
  forge's side does not read says "Shown on the forge." (`README.md`,
  4.2).
- **The first milestone needs two of them** (`../README.md`, section
  2): a change's result and its hold, in the forge's words, with a link
  to each on the forge (2.5).
- **A change is shown as its way to landing,** one step at a time, each
  with its reason in words: CI running, a review asked for, second in
  the queue, updating from main, resolving a conflict.
- **Queues are shown, not managed.** A landing branch's queue is its
  ready changes in order, on temper's `Changes` page; a person changes
  the order through goals' priorities on the board, never by moving
  changes about.
- **Code stays on the forge.** Parts link to a pull request, its diff,
  its files and its failing jobs; the client shows states and verdicts,
  never code, and has no code review of its own.
- **Approvals are at a head,** given in the client as a person task or
  on the forge as a review (section 4): each counts at the head it was
  given at, and the panel says when a later head still carries it.
- **A hold says what happened on the forge,** and what each act would
  do; nothing on the forge is written over until the person decides
  (section 6).
- **What the forge did not tell temper is said:** a repository with no
  webhooks, protection temper could not read, a branch someone else
  moved.

## 2. A change's panel

On a change task's page, as its panel: a procedure's activity is its
connector's panel (jig's `client/domain/pages.md`, 5.2). Folded to its
step in words, it is the change's phase wherever its chip shows (jig's
`client/domain/objects.md`, section 3): a chat's conversation, a goal's
plan, the inbox, the `Changes` page.

### 2.1 Its way to landing

The steps of `../domain/forge.md`, 8.2, with the one it is at and
why:

```
producing ─ opening ─ checking ─ gating ─ queued ─ first ─ landing ─ landed
                 repairing · updating · resolving · recreating · reopening
```

| Step | It says |
|---|---|
| producing | T21 producing it, as a chip |
| opening | opening its pull request |
| checking | CI on its head, and for how long |
| gating | which gates are running, as chips, and which requirements wait |
| queued | its place in its landing branch's queue, and who is ahead |
| first | first in the queue: updating from its base, CI on the updated head, or merging |
| repairing | why (CI's failure, a gate's remarks, a semantic conflict), the repair as a chip, and how many repairs are left |
| updating | the base it takes in |
| resolving | the conflicting files, the resolution as a chip, and how many are left |
| recreating, reopening | after drift released (6.2): its branch made again at the head temper last knew, or its pull request reopened or retargeted back |
| landing | merging at its head |
| landed | its merge commit, and when |

### 2.2 What it shows

- **Its pull request:** number, title, head and base, state,
  mergeability, or that it is not known yet while the forge settles it
  (`../domain/forge.md`, 5.2); links to it, its diff and its files on
  the forge.
- **CI on its head:** the combined state and each context's, with a link
  to a failing job, and the failed step and the end of its log a repair
  was given, cut as the repair got it (`../domain/forge.md`, 8.4).
- **Its gates** (`../domain/forge.md`, 8.3): each with its kind (a
  review by a lens, a person's approval, checks), blocking or advisory,
  who asked for it (the change, the project's landing rule, its plan),
  its verdict and the head it was given at, and whether it carries over
  to the current head; the task running it, as a chip. An advisory gate
  that failed is reported and holds nothing, and the panel says so.
- **Its requirements:** the landing rule's, each met, waiting (and for
  what) or refused at the current head: CI at the head, up to date with
  the branch, its blocking gates, approvals on the forge by so many
  people of a role (section 4; `../domain/forge.md`, 8.7).
- **Its heads,** in order, each with what made it (produced, a repair, a
  clean update, a resolution) and its CI and verdicts, so the person can
  see why a gate is asked again: a repair's head asks every gate again,
  a clean update's carries them.
- **Its bounds:** repairs, resolutions and updates used of their limits;
  each wait against its stall: "CI has not reported for 25 minutes; held
  at 60".
- **Its delegates:** producing, repairs, reviews, resolutions, as the
  task page's plan (jig's `client/domain/pages.md`, 5.2).

### 2.3 What may be done

- **Amend its gates:** add a review by a lens or a person's approval, or
  drop one the change or its plan asked for; a landing rule's gate
  stays. A change's gates are its parameters, amended
  through the forge's form in the panel, the forge's own request (jig's
  `client/domain/root.md`, section 6). A gate added while it is queued
  takes it out of the queue until its verdict, which the form says.
- **Release** it, held (section 6; jig's `client/domain/objects.md`,
  section 5).
- **Cancel** it (jig's `client/domain/objects.md`, section 11): the
  panel says what the forge will do: its pull request closed with a
  comment saying why, its branch deleted, its commits still readable
  under the closed pull request; a merge already sent may still land,
  and then it ends landed (`../domain/forge.md`, 8.6).

### 2.4 A chat's small change

A chat that makes a small change itself delivers it to its run's own
branch, and delegates a change task handed that branch
(`../domain/forge.md`, 8.1 and section 11). In the conversation the
delegation shows as the change's chip (jig's `client/domain/objects.md`,
section 10), its phase the panel's step in words, following it through
CI, its gates and its queue to landing while the chat carries on. The
change's result reaches the chat as its delegate's result (2.5), and the
chat tells the person.

### 2.5 Its result, and why it stopped

What the first milestone needs of the forge's parts (`../README.md`,
section 2, steps 6 and 7):

- **Its result,** a connector's result (jig's
  `client/domain/objects.md`, section 8): landed, with its pull
  request's number and title, its merge commit, and the address of each
  on the forge; or ended otherwise, with what became of its pull request
  and its branch. It shows on the change's page, in the chat that
  delegated it, as its delegate's result, and in the update of the
  person who asked for it.
- **Its hold,** the forge's hold part (jig's `client/domain/objects.md`,
  section 5): its reason in section 6's words, and the head and the
  failure it is about, each with a link on the forge, on the held card
  wherever it shows: the change's page, the chat, the inbox's
  escalation.
- **A repository named by owner and name,** in place of its numbers
  (`README.md`, 4.1), from the same reads.

Beyond it: the way to landing step by step (2.1 to 2.3), the queues
(section 3), health (5.2), what each act on drift would do (6.2), the
outbox and the deployment's part (5.4).

## 3. Changes and landing queues

temper's `Changes { project }` page (`README.md`, 3.5): a
project's changes, by landing branch (`../domain/forge.md`, section
9):

- **The queue:** the ready changes in order, each with its goal and
  priority and when it became ready; the first says what it is doing
  (updating, CI on its updated head, merging); a change past the queue's
  window is marked as going ahead of those ready after it.
- **The order's rule,** in a line: "by goal priority, then by when each
  became ready; ready for over WINDOW goes first", WINDOW the forge's
  queue window (`../protocol/configuration.md`, section 7).
- **Not ready yet,** by step: producing, checking, gating, repairing,
  resolving.
- **Held,** each with its reason (section 6).
- **The branch:** its tip, CI on it, and the landings recently made,
  paged.
- **Paused:** when CI fails on the branch's tip, "main fails at its tip
  since 10:02; T88 repairs it, first in the queue; the rest wait", with
  the repair as a chip. Past its repairs' bound, the queue is held for a
  person, whose card offers to release it as it is (a flaky check) or to
  cancel the repair.
- **Where temper may not merge,** the queue is still ordered for people
  to see, each ready change "waiting for someone else's merge".

## 4. Approvals

A person approves a change in one of two ways, as its gates and its
project's landing rule ask (`../domain/forge.md`, 8.3 and 8.7).
Neither is temper's default: a person's approval is a gate a plan or a
change is given when a person asks for it, or a project's rule
(`../README.md`, section 2).

- **In the client:** a person task addressed to the project's
  maintainers, one of whom takes it, whose contract is an approval of
  the change at a head (jig's `client/domain/objects.md`, section 7;
  jig's `domain/people.md`, section 8). Its card, in the inbox and on
  the change's page, says:
  - *what is asked:* approve this change at this head, and who asked
    for the approval: the change, its plan or a landing rule;
  - *the change at that head,* the forge's part: its pull request, CI at
    the head, the other gates' verdicts, what changed since an earlier
    approval if there was one, and a link to its diff on the forge;
  - *the answer:* approve, naming the head; or cannot do it, with a
    reason, which holds the change when the approval is blocking, the
    reason in the person's words (6.1).
- **On the forge:** reviews by people of a role, counted by the landing
  rule at the head or, where the rule allows, a clean update of it. The
  panel shows each approval counted, by whom and at which head, and how
  many more the rule asks; the person reviews through the panel's link.

If the change's head moves while the card is open, the card says so. A
clean update temper made carries the approval where its gate counts
clean updates (its freshness, `../domain/forge.md`, 8.3); any other
new head asks again, as a new person task at that head, and the old card
says it has been replaced.

## 5. Repositories, and the forge itself

### 5.1 Adopting one

The adoption form is `README.md`, 4.4, on temper's `Adopt` page,
and its outcome 4.7. With the forge's documents served, the form chooses
among the configured forges by name, says whether the project has a
home, and, before the adoption is confirmed, shows what temper found
there (`../domain/forge.md`, section 4): temper's permission, the
merge styles the repository allows and so whether it may merge, the
protection on the landing branch (or that it could not read it), its
webhooks or none, CI or none, the people it knows there with the role
each would be given, and the ceiling that follows, in words.

What the adoption names there: the repository's role (owned, where
temper pushes and lands; a fork of an upstream; or context, read only),
its landing branch, temper's branch prefix (`temper/` by default),
whether it is the project's home, and whether it runs CI or the checks
its changes are gated on without it; the merge style is the
repository's default. A repository where another deployment's branches
already live under the prefix is refused, saying so (jig's
`domain/connectors.md`, section 12).

### 5.2 Health

Wherever a repository is shown (the project's page, its settings, the
deployment's page):

- **what temper may do** there, and whether that narrowed since adoption
  ("a merge was refused: temper may no longer land into main");
- **its webhooks:** when they were last heard; none, or none for days,
  means changes are seen by polling alone, more slowly;
- **its landing branches:** their tips and CI on them;
- **what is held** on it for drift (6.2).

### 5.3 In a project's settings

- **Resources:** each repository of the project, as the forge's part in
  the settings' resources (jig's `client/domain/pages.md`, 6.5): its
  role, whether it is the project's home, its landing branch, temper's
  branch prefix there, its merge style, CI or the checks that stand for
  it, what temper may do there in `README.md`, 4.7's words, and its
  health (5.2). Adopting one is the form of 5.1; where the form lives is
  `README.md`, section 11's.
- **The landing rules,** in the policy's requirements (jig's
  `client/domain/pages.md`, 6.5), per pattern of landing branches, the
  deployment's and the project's, in words: "into main in acme/shop: CI
  passed at the head; up to date with main; a review for memory safety,
  at the head or a clean update of it; two maintainers' approvals on the
  forge" (`../domain/forge.md`, 8.7).

### 5.4 The deployment's page

The forge's part of the deployment's connectors (jig's
`client/domain/pages.md`, section 7), for each configured forge: its
request budget and what uses it (fresh reads, writes, keeping up, the
slow pass; `../domain/forge.md`, 5.4); a pause for its rate, and
until when; its outbox, the effects waiting and those uncertain, being
looked for; and when each repository's webhooks were last heard.

## 6. Holds, in the forge's words

A held change's card (jig's `client/domain/objects.md`, section 5)
shows the forge's hold part: what happened, the head and the failure it
is about, by whom and when where another hand did it, a link to each on
the forge, and what each act will do. A release resets the change's
repairs, resolutions and updates, and counts its stall from the release.
Nothing on the forge is written over until the person decides
(`../domain/forge.md`, 8.4 and 8.5).

### 6.1 The change's own

| Held because | The card says | A release |
|---|---|---|
| stalled | the wait, and since when: CI that never reported on its head, a mergeability never known, a gate's task stuck | takes the change back to the step it was at |
| repairs spent | the repairs made of their limit, and the last failure: CI's failed step, a gate's remarks | takes it back, its repairs counted again; amending its gates or its producing charter's instructions first, where the failure is not the change's |
| resolutions spent | the conflicts resolved of their limit, and the base it conflicts with | takes it back, its resolutions counted again |
| updates spent | the updates made of their limit | takes it back, its updates counted again |
| an approval declined | who declined it, at which head, and why | amended to drop the approval and released, in one request; released alone, it is held again at the same head |
| a requirement refused at the merge (`../domain/forge.md`, 8.7) | which of its landing rule's requirements refused, at which head: CI failed, a gate asked for changes, a person's review on the forge asked for changes, by whom | takes it back, judged again at its next merge: held again while the refusal stands |
| a delegate or a write failed for good | which: its producing task's failure, or the forge's refusal of a write (a protected branch, a forbidden write) | takes it back, to decide afresh |
| a write uncertain, of a kind that cannot be recovered (`../domain/forge.md`, section 6) | the write, that it may or may not have been made, and where to look on the forge: the branch, the pull request's reviews or comments, the issue | makes it again, or goes on without it, as the person says (jig's `client/domain/objects.md`, section 5) |

### 6.2 Drift

A held change's card for drift (`../domain/forge.md`, 8.5; jig's
`domain/connectors.md`, section 11): what changed on the forge, by whom
and when, and a link to it there.

| What changed | Release | Otherwise |
|---|---|---|
| its pull request closed by someone else | reopens it | cancel: the change closes |
| its branch moved by another identity, backwards, or while nothing of temper's wrote it | takes the new head as the change's, checks it and asks every gate again | cancel |
| its branch deleted | creates it again at the head temper last knew, and reopens the pull request | cancel |
| its pull request retargeted | retargets it back | amend its landing branch to the new base, then release; cancel |
| merged by someone else | not held: it is landed, like any landing | |

## 7. Goals, findings and news on the forge

### 7.1 A goal's issue

- **In its project's home repository** (`../domain/forge.md`, section
  12): the goal's page links to it, says when it was last written, and
  that it is a projection: what people write there is not read, and the
  client is where temper is reached.
- **Pull requests name their goal's issue,** so a person reading the
  forge finds the goal, and from it temper's client.

### 7.2 Landings as news

On a goal's page, with what it hears (jig's `client/domain/pages.md`,
5.2): each landing in its repositories, marked as its own, overlapping
its work (it woke the goal), not overlapping (kept), or unknown, as for
a landing too large to compare (it woke the goal)
(`../domain/forge.md`, 7.3).

### 7.3 Findings

The issues agents filed (`../domain/forge.md`, section 12): a bug
found while exploring, a follow-up a review raised, a flaky test. They
are the forge's part of a project's page (jig's
`client/domain/pages.md`, 6.1):

- **Each** shows its title, its repository and issue, the task that
  filed it as a chip, when, and its state, which the store keeps: open,
  taken up by a goal, dropped.
- **Taking one up** sets a goal that names it, the board's form with its
  words filled in (jig's `client/domain/pages.md`, 6.2), or starts a
  chat about it, with the finding named in the chat's first words, so
  its brief carries it (jig's `client/domain/pages.md`, section 4).
- **Dropping one** gives a reason, the forge's own request (jig's
  `client/domain/root.md`, section 6).

## 8. Taking part

What the forge shows of work on what others own
(`../domain/forge.md`, sections 13 and 14), possible with the parts
above:

- **A participating object** (an outside contribution, an issue others
  opened) shows on the page of the task dealing with it: what others
  wrote there, as the news it heard, and what temper would write, a
  reply or a review, as a proposal card with its exact text, when the
  project's policy has a person accept it first.
- **Where temper may not merge,** a ready change waits for a maintainer's
  merge, and people's reviews on its pull request show as its news.
- **An upstream submission** is a proposal a person accepts before
  anything is posted, showing exactly what will be.

## 9. What each part is served from

Each part is the forge's family's (`README.md`, section 4): read
or watched by the forge's client side through the family's own reads
and subjects, under the forge's owner number, each naming its scope, a
project or the deployment (jig's `client/domain/contract.md`, 7.5; jig's
`client/domain/root.md`, section 7), their records in
`temper-client-protocol-wire` beside `Adopt` (`README.md`, 4.6).
The first are a change's result and its hold (2.5).

| The client | jig's part and slot | The domain |
|---|---|---|
| a change's way to landing, its gates, requirements and heads | a panel, on its task's page | `../domain/forge.md`, section 8 |
| a change's result | a result, on its task's page, in a chat, in an update | `../domain/forge.md`, 8.2 and 8.6 |
| its holds | a hold, on the held card | `../domain/forge.md`, 8.4 and 8.5; jig's `domain/connectors.md`, 4.3 and section 11 |
| what a proposed write would do; the outbox | an effect, on a proposal card; an outbox entry | `../domain/forge.md`, section 6 |
| queues, paused and held | temper's `Changes` page | `../domain/forge.md`, section 9 |
| approvals | a person task's card; a panel | `../domain/forge.md`, 8.3 and 8.7; jig's `domain/people.md`, section 8 |
| adoption, repositories, health, landing rules | temper's `Adopt` page; a project's part | `../domain/forge.md`, sections 4 and 5, and 8.7; jig's `domain/connectors.md`, section 12; jig's `domain/authority.md`, section 10 |
| goals' issues, landings as news, findings | a project's part; a goal's page | `../domain/forge.md`, sections 7 and 12 |
| the forge itself | the deployment's part | `../domain/forge.md`, 5.4 |

## 10. Open questions

- **A change's files in the client:** whether the panel lists the files a
  change touches, from the connector's read, short of showing the diff.
- **Conflicts seen early** (`../domain/forge.md`, section 20):
  warning two goals whose open changes conflict before either lands, as
  a panel on both.
- **A queue's throughput:** if landings are ever batched, how a batch and
  its bisection show.
- **A person's approval as a rule,** posted on the forge as the person
  (`../README.md`, section 8), which the approval's card would then
  say.
