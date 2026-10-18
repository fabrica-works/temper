# The forge connector

Provisional, 2026-10-04, revised 2026-10-18. How temper drives a forge,
Forgejo first: the forge as a connector on jig's contract (jig's
`domain/connectors.md`), with its resources, effects, topics, procedures,
requirements and reads; adopting repositories; keeping up; a change's
way to landing and the requirements its merge meets; landing queues;
updates and conflicts; issues; and the ways temper takes part in what
others own. Its protocol layer, the component that makes each call to
Forgejo and hears its webhooks, is `../protocol/forge.md`. What is still
open is listed in section 20.

## 1. In one page

- **What the forge holds:** git; pull requests as the unit of a change;
  CI; branch protection, as a second net; and a record people can read.
  From what temper owns there, only CI, people's approvals and drift
  come back: no comment on it is an input, and no label marks what
  temper tracks.
- **Live work comes from the store.** The connector holds a working set
  of what the forge owns of the resources live tasks name (pull
  requests' heads and states, CI on those heads, landing branches'
  tips), kept up by listing what changed, sooner on a webhook's hint.
- **The change procedure** takes a change from produced to landed:
  produce, open, CI, gates, its turn in its landing branch's queue, an
  update from its base when it lacks the base's tip, and a merge at
  exactly its head. Agents are woken for what needs judgement: producing,
  repairs, reviews, conflicts.
- **Landing rules are requirements.** A project's rules say what a merge
  into a branch needs: CI passed, the head up to date with the branch,
  gates, people's approvals. The forge judges each from its own facts at
  the merge's exact head, which Forgejo's merge is conditional on (8.7).
- **Landing queues.** Changes ready to land in a branch wait in line, by
  their goal's priority then by when each became ready; only the first
  is brought up to date and merged.
- **Updates are lazy and mechanical.** A change takes its base when it
  is first in line, or about to get a run, through the forge's own
  update; an LLM only resolves a conflict, in a workspace where the
  merge is in progress.
- **Every landing is news** for every goal working in that repository;
  it wakes a goal's coordinator only when it overlaps the files of that
  goal's changes.
- **Issues stay, one per goal,** as a projection people can read: the
  goal, its plan as a task list, and its milestones. Pull requests refer
  to it.
- **Plugged in as it is.** temper adopts repositories with whatever
  access it is given; webhooks, labels, protection and CI are all
  optional. Nothing is forced: pushes are fast-forwards, and merges are
  made at exactly the head decided on.

## 2. What it gives

- **Resources,** named `forge` `<host>` `<owner>` `<repo>`, then:
  - `branch` and the branch's own segments: `… branch temper 7 c42`;
  - `pull` and a number; `issue` and a number;
  - nothing beyond, for the repository itself.

  The host, the owner and the name are those the repository was adopted
  with (section 4).
- **Holds** (jig's `domain/connectors.md`, 3.3): a branch under temper's
  prefix takes an exclusive hold, and a hold already taken makes its
  task wait, never refuses it (jig's `domain/tasks.md`, 6.2). Landing
  branches, pull requests, issues and the repository itself are shared
  and take none: merging into a landing branch is ordered by its queue
  (section 9).
- **Effects:** open, edit, update, merge, close and reopen a pull
  request; post a review, a comment or a commit status; create, edit,
  close and reopen an issue; set reviewers; create and delete a branch
  (section 6). A run's push is the worker's, over git, under its task's
  hold (`worker.md`, section 5).
- **Kinds,** for authority (jig's `domain/authority.md`, section 4),
  numbered: 1, reads; 2, a run's push; 3 to 15, the effects, one per row
  of section 6's table. No kind implies another: each write a task means
  to make needs a grant of its own kind. Landing rules govern kind 4,
  the merge (8.7). None forces a push (8.7).
- **Topics:** landings on a branch; CI on a head; a pull request's
  state; the comments and reviews on a participating object (section 7).
- **Procedures:** the change (section 8).
- **Requirements:** CI passed, up to date, a gate, an approval, which
  landing rules name as judges of the merge (8.7).
- **Reads an agent may call:** each of the forge client's reads but
  locating a repository: a page of the issues and pull requests changed
  since a time; an issue or a pull request and its comments; a pull
  request, or the one for a head and a base, its reviews, a review's
  remarks, and its files; a branch's head, and the branches; the
  statuses and checks on a commit, and a failed job's log; a comparison
  of two commits; a file at a head; a repository's protection, settings
  and collaborators, and a user's permission. Each answer is cut to its
  limit. The host tools that serve them, and those for effects and
  topics, are `agent.md`, 4.3.
- **Effects offered to agents** (jig's `domain/connectors.md`, 4.5):
  creating an issue, posting a comment on an issue or a pull request,
  posting a commit status. What the change procedure owns (opening,
  updating, merging, reviewing as a gate) is not offered: a run that
  wants a change landed delegates it.
- **Brief sections,** of three kinds, each at the change's head: the pull
  request, its files and what differs from its base, or, for a
  resolution and a semantic repair, what landed in the base, with the
  files in conflict for a resolution; CI's failures, each with its
  failed step and the end of that step's output; reviews' remarks.
- **Workspace items:** for each repository a run works in (root.md,
  7.1), a checkout: where it starts (a landing branch or the
  repository's default, a branch, saved work, or a merge in progress),
  whether it may be written and the branch it pushes, and the git
  identity it is fetched and pushed with (`worker.md`, 4.1;
  `../protocol/credentials.md`, section 4).
- **Documents and parts** (jig's `domain/connectors.md`, 5.5): what
  temper's client shows of the forge, made from the connector's records
  and working set: a change's way to landing, its result and its hold,
  the queues, the repositories and their landing rules, goals' issues,
  findings and the forge itself (`../client/forge.md`, section 9).
- **What temper owns there** (jig's `domain/core.md`, 8.1): what it
  creates, recorded in the store by key. On the forge, ownership shows
  as authorship, by temper's own user, and as temper's branch prefix,
  which branch protection can reserve to it.

## 3. Structure

```
temper-engine-domain-forge              the connector's top: resources and holds, adoption, topics, judges, briefs, routing; the effects and their identities
├── skein-forge-client                  the working set, keeping up, fresh reads, the request budget, effects' keys, write lanes and recovery (skein's forge-client.md)
├── skein-forge                         the vocabulary the top and the client share: calls, answers, errors, hints (skein's forge.md, section 3)
├── temper-engine-domain-forge-change   the change procedure and the landing queues: decisions over facts
└── temper-engine-domain-forge-issues   the goals' issues: what each projects, when to write it
```

- **The top** keeps the forge's resources as the project knows them
  (roles, what temper may do there), the holds on branches, the topics
  and their subscribers, the changes' procedure states, the issues'
  projection states, and the landing rules' gates and judges' parameters.
  It routes between its children and translates to and from the root.
  It names the client's live resources, owns the effects and their
  identities (section 6), and hands the client their bytes and its
  namespace, the deployment's id, before any restored entry or live
  resource (section 16).
- **`skein-forge-client` is a capability:** it knows the forge's API as
  `skein-forge`'s vocabulary states it, the working set, the request
  budget, and how a write is made once. Its owner, how it is driven and
  what it never knows are skein's `forge-client.md`, sections 2 and 9.
- **`change` and `issues` are policy:** given a change's state and the
  facts, or a goal's state, they say what is due. They keep nothing
  between calls; their states are the top's, committed in the store.

## 4. Adopting repositories

- **A repository joins a project with a role** (jig's `domain/core.md`,
  6.1): *owned*, where temper pushes its branches, opens pull requests
  and lands; a *fork* of an upstream (section 14); or *context*, read
  only. An owner of the project adopts it in the client, through the
  adoption form (`../client/README.md`, 4.4). A repository temper writes
  belongs to one project; others may adopt it as context.
- **A repository is named by its owner and name, and kept by the forge's
  id for it.** The adoption names the forge, the repository's owner and
  its name; the connector first locates it, a read whose answer is the
  forge's own id for the repository, and names the repository by that id
  from then on (`Repository { forge, repository }`), so a rename or a
  transfer on the forge keeps it the same repository. A repository the
  forge does not have refuses the adoption as not found.
- **The adoption request is the forge's own,** keyed as any party's
  request (`../client/README.md`, 4.4): the root takes it from the protocol
  layer typed, and the connector refuses a forge the deployment does not
  configure; then locates the repository (`Read::Locate { owner, name }`,
  answered `Answer::Located { repository }`); then the root makes jig's
  adoption request on the person's behalf (jig's `domain/people.md`,
  5.1), in which the connector reads what temper may do there (below)
  before the core commits the adoption (jig's `domain/connectors.md`,
  12). Its outcome is the adoption's: adopted, refused by the core with
  jig's reasons, or refused by the forge with the connector's. The
  connector keeps it under the people child's token until its retention
  ends, to give again when the request is sent again (jig's
  `domain/connectors.md`, 5.5; `../protocol/store.md`, section 2).
- **The forge's host** in a repository's resource path, and in what a
  person reads of it, is the forge's as configured: the host of its `git`
  base, with `:port` when it names one (`../protocol/configuration.md`,
  section 3). The adoption does not ask a person for it.
- **What temper may do is read at adoption:** its user's permission on
  the repository; the repository's settings (the merge styles it allows,
  its default branch); the branch protection on the landing branch; and,
  but for context, the branches already under temper's prefix. Write
  permission, or admin, gives every kind of effect on an owned or fork
  repository, merging only where the repository allows a merge style;
  read gives reads alone; no permission refuses the adoption as
  forbidden. Forgejo refuses protection reads to a writer who is not an
  admin (skein's `forge.md`, section 9): adoption records
  protection as unknown, never as absent. A write the forge refuses
  narrows what temper may do there, for every later decision (jig's
  `domain/connectors.md`, 12).
- **The project's ceiling follows** (jig's `domain/authority.md`,
  section 6) from each repository's role and what was read: on an owned
  repository, pushing under temper's prefix, opening pull requests and
  landing into its landing branches; on a fork, pushing into the fork;
  on a context repository, reading. Protection that keeps temper from
  merging makes landing a wait for someone else's merge (section 13).
- **One deployment's prefix.** Adoption refuses a repository where
  branches under temper's prefix exist that the store records no making
  of: they are another deployment's (jig's `domain/connectors.md`, 12).
- **People the forge knows.** Adoption reads the repository's
  collaborators, each with its permission (read, write or admin), and
  gives them to the core as parties known by their forge user's id. The
  project's configuration maps each permission to a role, given to those
  with no role in the project yet (jig's `domain/people.md`, section 4):
  temper's usual mapping is admin to maintainer, write to member, read to
  observer. After that, roles are the core's: a permission that changes
  on the forge changes no role.
- **A repository of an org a setup names** (`../protocol/setup.md`,
  section 5) is adopted with no collaborator step: temper's user is a
  member of the org's team, which has write on every repository the org
  has or will have, so the adoption reads that write access as any
  other; and the org's hook, made when the deployment has a webhooks'
  listener, delivers its hints. Any other repository, a person's own
  among them, needs a person to give temper's user its access, as a
  collaborator or through a team, and to add its hook or leave it to
  poll.
- **The adoption names, per repository:** its landing branch; temper's
  branch prefix, `temper/` by default, which protection can reserve to
  temper's user; whether it is the project's home, where its goals'
  issues are kept (section 12), one per project and never a context
  repository; and whether it runs CI, and if not, the checks its changes
  are gated on (8.3). The merge style is the repository's default
  (skein's `forge.md`, section 12).
- **Nothing is required:**
  - no webhook: polling is the backstop, webhooks only cut the latency,
    and the client shows a repository whose hints never arrive;
  - no label: temper writes none, and decides nothing by one;
  - no protection: temper's rules are its enforcement, protection a second
    net;
  - no CI: a repository without it gates its changes on checks its
    adoption names (8.3).
- **Provisioning** (creating a repository, its webhook, its protection)
  is an optional set of effects a person may ask for, never a step of
  adoption (jig's `domain/connectors.md`, 12).

## 5. What it holds, and keeping up

The working set, the keep, fresh reads and the budget are the client's
(skein's `forge-client.md`, sections 3 to 6). This section keeps what
temper puts in them and the members that set them: the client's limits
are `forge.client.<limit>`, with durations in `_ms`
(`../protocol/configuration.md`, section 7).

### 5.1 The working set

Held by skein's `forge-client.md`, section 3. temper's part:

- **The live resources, and nothing else** (jig's
  `domain/connectors.md`, 3.4). A resource enters when a live task names
  it, and leaves when none does. The top names them to the client.
- **What temper watches:**
  - per repository: the landing branches' tips, and the CI on each tip;
  - per pull request temper owns that a live task names: its head and
    base, its state (open, closed, merged), its mergeability, CI on its
    head (each context's latest status, and the combined state), and,
    when a decision wants them, its files at its head;
  - per branch a live task holds: its tip as temper last pushed or
    updated it, to tell drift from temper's own moves;
  - per participating object: its comments and reviews not yet given to
    its task (section 13);
  - per issue of a live goal: what temper last wrote into it, so an
    unchanged projection is not written again.
- **Its capacity:** `forge.client.resources` live resources and
  `forge.client.repositories` repositories. A batch naming resources past
  it is refused.

### 5.2 Keeping up

Held by skein's `forge-client.md`, section 4: the listing per
repository since the forge's time, a pull request's backoff, the hints,
the landing branches' tips, the slow pass and the forge's clock. temper's
part:

- **Its values:** `forge.client.poll_ms`, `poll_max_ms`, `hinted_ms`,
  `slow_ms` and `settle_ms`. The paces are what they are with no
  webhook, so a delivery lost, refused or never sent costs latency,
  never liveness (skein's `forge-client.md`, section 4).
- **Hints** come from the forge's webhook deliveries
  (`../protocol/forge.md`, section 5). The client reads on every hint it
  holds a resource for. temper, the owner, drops its own echoes, by key
  and by pusher: a creation's key for an item, the pusher for a push
  (skein's `forge-client.md`, section 4).
- **What Forgejo settles after it delivers** is read again sooner, by
  the change procedure, which reads its pull request afresh (5.3), not
  by the keep. Forgejo checks a pull request's mergeability after a push
  moves its head or base, and answers it not mergeable while it checks;
  and temper relies on no order between a job's end being delivered and
  its commit status being written, only on both settling within `settle`
  (skein's `forge.md`, section 9). So, within `settle` of a move of
  its head or base (known by a hint, a read or temper's own push), a
  pull request that reads not mergeable is read again at `hinted`,
  doubling, until it reads mergeable or `settle` has passed; and a
  commit a job's hint named whose CI reads pending is read again the
  same way. These rereads are temper's fresh reads. `settle` is a limit
  of the forge client's, which checks only that it is above 0 and runs
  no window (skein's `forge-client.md`, sections 4 and 5).
- **The window is the change procedure's own wait.** Until `settle` has
  passed since the move, not mergeable is not known: the change waits
  until the window's end at most, never stalls on it, and is never taken
  as conflicted on it; once the window has passed, not mergeable is a
  conflict (8.2). A move long before, such as a head pushed before a
  long CI, leaves no window open. The window's start is the later of the
  change's own last push, which it keeps, and the last move a hint or a
  read told of, which it does not: after a restart, only its own push
  opens a window.

### 5.3 Fresh reads

Held by skein's `forge-client.md`, section 5: a fresh read goes first
in the request budget (5.4). temper's part is what it asks for. Before a
decision depends on a fact, the change child domain asks for it afresh:
the branch's head, the base's tip, the pull request, CI on the exact
head and on the base's tip, the comparison of the head with the base's
tip, the pull request's reviews, and the files of the pull requests an
overlap compares. These are also the facts the forge's requirements are
judged on (8.7).

### 5.4 The request budget

Held by skein's `forge-client.md`, section 6: the four queues in their
order, the charge of each call and the wait after a refusal for the
rate. The order is the one jig's `domain/connectors.md`, section 13
gives: fresh reads first, a write's own checks and finds among them;
then writes; then keeping up; then the slow pass. temper's values:
`forge.client.rate` requests per `forge.client.window_ms`, with
`forge.client.reserve` of each window kept for keeping up and the slow
pass.

## 6. Effects

Every effect is committed into the outbox and made by `client`, in
commit order per lane (a pull request, an issue, a repository) and one
at a time on each (jig's `domain/connectors.md`, 4.2). The top owns the
effects and their identities; the client's keys, its write lanes and its
recovery are skein's `forge-client.md`, section 7. temper's effects:

| Kind | Effect | What it is | Recovery class (jig's `domain/connectors.md`, 4.3) | Found again after an uncertain failure |
|---|---|---|---|---|
| 3 | open a pull request | a creation: from a branch into its landing branch, with a title, a body and a key | conditional: only while no pull request for its branches is open; Forgejo keeps it, closed or not | by its branches: the pull request open for them, else the oldest |
| 10 | edit a pull request's title, body or base | a set | idempotent | written again |
| 11 | update a pull request from its base | a transition: the base's tip merged into its branch, by the forge | unrecoverable | its head contains the base tip it asked for |
| 4 | merge a pull request | a transition, conditional on its exact head | conditional | merged, and at which commit |
| 12 | close, reopen a pull request | a set | idempotent | written again |
| 5 | post a review | a creation, with a key | unrecoverable | by its key, among the reviews after the entry's start |
| 7 | post a comment | a creation, with a key | unrecoverable | by its key, among the comments after the entry's start |
| 6 | post a commit status | a set: a context on a commit | idempotent | written again |
| 8 | create an issue | a creation, with a key | unrecoverable | by its key, among the issues changed since the entry's start |
| 13 | edit, close, reopen an issue | a set | idempotent | written again |
| 14 | set a pull request's reviewers | a set | idempotent | written again |
| 9 | create a branch at a commit | a creation, named | unrecoverable: another hand can delete it, and nothing then shows it was made | the branch exists at that commit |
| 15 | delete a branch | a set: temper's own branches only | idempotent | it is gone |

- **Each row is a kind of its own** (jig's `domain/connectors.md`,
  3.1): a policy that grants writing issues grants each kind it means.
- **Classes follow what Forgejo offers** (skein's `forge-client.md`,
  section 7.3; the facts, skein's `forge.md`, section 9). It keeps no
  key it would refuse a second creation by, so no kind is keyed. A pull
  request is opened only while none is open for its branches, and
  Forgejo keeps every pull request, closed or not, so the
  state it leads to is always found, closed by another hand included,
  which is drift (8.5). A change opens one only when none exists for its
  branches, and reopens its own when it was closed (8.5), so the oldest
  for its branches is the one it follows; one another hand opened while
  that one is closed is the open one, found first. A branch is created
  only while none of its name exists, but another hand can delete it, and
  nothing then shows it was made: its creation is unrecoverable.
- **An unrecoverable creation is looked for first:** an uncertain review,
  comment, issue, update or branch found by its key, or by the state it
  leads to, is made; one not found by its deadline holds its task for a
  person, and is never sent again by temper on its own. The client makes
  the find (skein's `forge-client.md`, section 7).
- **Keys are carried** in what temper creates: a marker at the head of a
  body, a comment or a review, hidden where the forge renders it (skein's
  `forge.md`, section 6; temper's word and namespace,
  `../protocol/forge.md`, section 4). A pull request's key is its
  branches, which temper alone names. An entry's start, and the find that
  reads only what came after it, are the client's (skein's
  `forge-client.md`, section 7).
- **An effect's identity** is its purpose, and the top encodes it. The
  client takes the bytes whole and never inspects them (skein's
  `forge-client.md`, section 7). A key is the client's frame (version 1, the namespace's
  length as two bytes big-endian, the namespace) and then the purpose's
  bytes: a tag, and its fields in order, each number big-endian. The
  namespace is the deployment's id, 16 bytes. A purpose is derived from
  what the effect is for, never from when it was asked or its entry's
  number, so asking again for the same thing gives the same bytes (jig's
  `domain/connectors.md`, 4.4):

  | Tag | Purpose | Fields, with their bytes | Bytes with the tag |
  |---|---|---|---|
  | 1 | a call: one agent's call in one task attempt | task (8), attempt (8), completion (4), position (4) | 25 |
  | 2 | a step: one purpose within a task's procedure | task (8), purpose (2) | 11 |
  | 3 | a projection: one part of a goal's issue | goal (8), part (1), number (8) | 18 |
  | 4 | a release: cleanup of a task's named resource | task (8), the resource's length (2), the resource | 11 and the resource |
  | 5 | a history milestone: one task's milestone in a goal's issue | family (1), goal (8), task (8), position (8) | 26 |

  A part of a projection is the issue's open (1), its body at a local
  revision (2), the plan accepted (3), a plan revision by its history
  number (4), a change landed (5), a change held (6), a report kept (7),
  the goal finished (8) and the issue's close (9); the number is the
  revision, the history number or the change's, and 0 for the others. A
  history milestone's family is a task's lifecycle position (1) or a
  task's semantic revision (2, whose position is the revision). So a
  call's key is 44 bytes, and a key is refused when it passes the
  client's `op_bytes` or the namespace is empty.
- **Conditions travel with the effect:** a merge names the head its
  decision saw, which the forge checks, and a stale head is refused, not
  merged. It names the base too: the pull request is read afresh first,
  and one retargeted since the decision is not merged.
- **Failures:** a merge refused for a conflict, an update refused for a
  conflict, a pull request with nothing to merge, a protected branch and
  a forbidden write are each typed, and go to the procedure that asked,
  never retried blind.

## 7. Topics, and landings as news

### 7.1 Topics

| Topic | Published when | Read by |
|---|---|---|
| landings on a repository's branch | a landing branch's tip moves | goals working in that repository; landing queues |
| CI on a head | a head's combined state changes | changes; tasks that subscribe |
| a pull request's state | opened, closed, merged, its head moved, its mergeability known | changes; tasks that subscribe |
| comments and reviews on a participating object | another party writes there | the task dealing with it (section 13) |

The change child domain reads its facts directly and needs no
subscription; topics are for tasks whose executors are agents, and for
goals.

### 7.2 Every landing is news

- **For two kinds of goal:** the goal whose change landed, whose step is
  done; and every other active goal with changes open or planned in the
  same repository, whose base has moved. Concurrent plans meeting this
  way is the ordinary case, not an exception for outside activity.
- **A goal hears of landings** through the forge's topic for goals
  (jig's `domain/connectors.md`, 5.4): the landings on the branches its
  subtree's changes target, which the core subscribes it to as the first
  change into each is made, until the goal ends.
- **A landing temper did not make** is news like any other: what landed
  is read, a pull request or a range of commits.

### 7.3 Which landings wake

Not every piece of news wakes. The connector classifies each landing for
each subscribed goal (jig's `domain/connectors.md`, 5.3), without an
LLM:

- **its own change landed:** kept; what comes after it is due
  mechanically, through the goal's dependencies;
- **another landing overlaps its work:** wakes. Overlap is the files the
  landing touched against those of the goal's own open pull requests at
  their heads, and, for changes not produced yet, the paths their specs
  name, as a hint;
- **no overlap:** kept, for its next run to read;
- **overlap unknown** (a landing too large to list, a read that failed):
  wakes, since a wasted turn is cheaper than a missed conflict.

The files are read from the forge (a pull request's files, or the
comparison of the branch's old and new tips), cached per head and cut at
a limit past which the overlap is unknown. Forgejo pages a pull
request's files but answers a comparison whole
(skein's `forge.md`, section 9): the protocol layer bounds the
comparison's bytes, files and commits before the domain holds them, and
a bound reached, an unusable answer or an uncertain completeness makes
the overlap unknown and wakes every affected subscriber. A pull
request's pages count only while fresh reads before and after them name
the same head; a head that moved voids the listing.

## 8. The change procedure

### 8.1 What a change is

A change task asks for one change to one repository, landed into one
branch. Its parameters:

- **the repository and the landing branch;**
- **the branch,** held by the task: one it is handed, already pushed (a
  chat's small fix, handed over by the chat that made it: jig's
  `domain/tasks.md`, 6.4), or its own, named after its tree
  (section 11), which a producing task pushes;
- **what to produce,** when nothing is pushed yet: the spec and charter
  of its producing task;
- **its gates** (8.3);
- **the pull request's title and body,** or what to make them from, and
  the goal it is part of;
- **its bounds:** repairs, resolutions, updates, and the stall of each
  wait.

A goal across repositories is several change tasks, ordered by their
dependencies: skein's change lands before temper's that bumps its lock.

### 8.2 Its way to landing

```
producing ─► opening ─► checking ─► gating ─► queued ─► first ─► landing ─► landed
                          ▲   │        │                  │
                          │   └────────┴──► repairing ────┤
                          │                               ├─ lacks the base's tip ─► updating
                          └───────────────────────────────┤                          │
                                                          └──── a conflict ◄─────────┘
                                                                      │
                                                                  resolving
```

1. **Producing:** a task for an agent, with the charter the change
   names, whose workspace starts from its landing branch (or its saved
   work) and pushes the change's branch (`agent.md`, section 6). Its
   result is the head it pushed. An attempt whose answer was lost may
   have pushed all the same: its writer slot is freed only once the
   branch has been read afresh (jig's `domain/tasks.md`, 6.3), and a
   head found there then is the change's own (the branch is held by this
   change alone), so the change cancels its producing task, whose work
   is done, and goes on to open it, in the same commit that frees the
   slot, before any retry could claim it. A change handed a pushed
   branch starts at the next step.
2. **Opening:** its pull request into its landing branch, keyed by its
   branches, its body naming its goal's issue ("Part of
   `ai/temper#12`").
3. **Checking:** CI runs on its head. A failure is a repair (8.4): a task
   given the failing output. A repository without CI is checked by its
   checks (8.3).
4. **Gating:** each gate without a verdict valid at this head runs: a
   task at the exact head. A blocking review or check that asks for
   changes is a repair, given its remarks; a blocking approval a person
   declines holds the change (8.3).
5. **Queued:** ready (CI passed and every blocking gate holds, at its
   head) and waiting its turn in its landing branch's queue (section 9).
6. **First:** its turn, which it keeps through what follows unless it
   leaves the queue (section 9). If its head lacks the base's tip, it is
   **updated** (section 10): cleanly, and its new head is checked again,
   its gates carried over where they count clean updates (8.3); or in
   conflict, and it leaves the queue to be
   **resolved**, its new head checked again and its gates asked again.
   A pull request whose mergeability has not settled (5.2) is neither:
   the change waits for it, until the window's end at most, and the
   stall does not count that wait.
7. **Landing:** merged at exactly its head, in the repository's merge
   style, automatically: what more a landing needs is a landing rule's,
   which the core checks at the merge (8.7). Merged, the change is
   **landed**: its result is the landing (the pull request, the merge
   commit), and it closes, its branch deleted as its hold is released
   (section 11).

Lazy, too, for runs: before a repair or a resolution is created, the
change takes its base if its head lacks the base's tip, so a repair
starts from current code. It never updates only because its base moved.

### 8.3 Gates

- **A gate** is a verdict a change asks for before it lands, at its exact
  head:
  - *a review:* a task for an agent with a lens (memory safety,
    performance, a design), whose verdict is approve or request changes,
    with remarks;
  - *a person's approval,* through a person task addressed to the
    project's maintainers and answered in the client (jig's
    `domain/people.md`, section 8): the change approved at the head, or
    declined as "cannot do it", with a reason;
  - *a check:* one of the checks a repository without CI is adopted
    with, a task at exactly the head.
- **Blocking or advisory.** A blocking gate's verdict holds the landing;
  an advisory one's is read, and reported, and holds nothing.
- **A blocking gate that says no:** a review that asks for changes, or a
  check that fails, is a repair, given its remarks (8.4); a declined
  approval holds the change, with the person's reason. The verdict
  stands at that head, so a release alone holds the change again there:
  a person amends the change to drop the gate, then releases it, or
  cancels it.
- **Asked for by the change, the project and the plan.** A change's own
  gates are its parameters, amended while it is live (a coordinator
  adds a memory-safety review to one change); the landing rules add
  theirs to every change into a branch they cover (8.7). A gate added
  while a change is queued takes it out of the queue until it has its
  verdict.
- **Valid at a head, and carried over clean updates.** A verdict is given
  at a head. Each gate says which later heads it still counts at
  (`freshness`): *exact*, the head itself; *clean*, every head reached
  from it by updates temper made that merged cleanly, since the change's
  own diff is unchanged. A check is always exact. Any other new head (a
  repair, a resolved conflict) asks for every gate again. The change
  keeps, in its state, the heads its clean updates led through, which is
  the fact the requirements read (8.7).
- **Posted, if the project wants:** a gate's verdict as a commit status,
  such as `temper/review/memory-safety`, which protection can require
  independently of temper.
- **When they run:** once CI passes on the head, by default, so a head
  CI fails costs no review; the change may ask to run them at once.

### 8.4 Repairs, resolutions and stalls

- **A repair** is a task for an agent whose workspace starts from the
  change's branch, its brief carrying why: CI's failing output on the
  head; a gate's remarks; or, for a clean update whose CI then fails (a
  semantic conflict), the base merged in and what landed in it. It
  pushes a new head, and the change is checked again.
  For each failed check, the brief carries its context, description and
  link, then the step that failed, by name, and the end of that step's
  log, where a step tells its failure; the job's whole log's end when no
  step failed. The log is read at the job attempt that failed on this
  head, within the brief's byte budget, and its truncation is said; a
  read that fails is said in its place (skein's `forge.md`, 5.2).
- **A resolution** is a task for an agent whose workspace starts from a
  merge in progress: the base merged into the branch, the conflicting
  files marked. The agent edits them and runs the checks, with no git
  writes of its own; the worker commits the tree as the merge, with both
  parents, and pushes it (`worker.md`, section 5).
- **Bounded apart:** repairs past their limit, resolutions past theirs,
  updates past theirs, hold the change, saying why, and its requester
  hears.
- **Stalls:** each wait on the forge (CI that never reports, a
  mergeability never known, a gate's task stuck) is bounded by a stall
  counted from the head's push or the last release; past it, the change
  is held.
- **A release** lifts what it releases: repairs, resolutions and updates
  count again from it, its stall is counted from it, and the change
  decides afresh.

### 8.5 Drift

While its branch's writer slot is taken (jig's `domain/tasks.md`, 6.3),
by a run of one of its delegates, by a run that handed it the branch and
has not yet answered, or by an update it asked for, the change decides
nothing on its branch's head: forward moves by temper's identity are
taken as that writer's (jig's `domain/connectors.md`, section 11), and
the change reads the head afresh once the slot is freed. Otherwise:

- **Its pull request closed** by someone else: held, saying who and
  when. A release reopens it; a cancel closes the change.
- **Its branch moved** while nothing of temper's writes it, or by another
  identity, or not forward: held, the head kept as it is. A release
  takes the new head as the change's and checks it, gates asked again.
- **Its branch deleted:** held. A release creates the branch again at
  the head temper last knew, which the forge keeps under the pull
  request, and reopens the pull request.
- **Its pull request retargeted** to another base: held. A release
  retargets it back, or takes the new base as its landing branch if its
  requester amends the change so.
- **Merged by someone else:** landed, like any other landing.

Nothing is written over: a held change makes no effect until it is
released.

### 8.6 Cancelled

A cancelled change closes as every task does (jig's `domain/tasks.md`,
5.1): its live delegates close first, their runs ended; an entry already
sent, such as a merge, settles, and what landed counts; then its hold is
released (jig's `domain/tasks.md`, 6.5): its pull request closed with a
comment saying why, and its branch deleted, its commits still readable
under the closed pull request. A change whose merge landed meanwhile
ends landed, not cancelled.

### 8.7 The forge's requirements

The forge judges four requirements on its own facts (jig's
`domain/connectors.md`, section 7), which landing rules attach to its
merge, kind 4, on a pattern of landing branches (jig's
`domain/authority.md`, section 10):

| Requirement | Its parameters | Met | Wait | Refuse | Guarded or observed |
|---|---|---|---|---|---|
| 1, CI passed | none | CI passed at the head; for a repository without CI, every one of its checks passed at exactly the head | CI pending or unknown; a check with no verdict at the head | CI failed; a check failed; a repository with neither CI nor checks | guarded |
| 2, up to date | none | the head contains the base's tip | the comparison unread or failed | the head lacks the base's tip | observed, freshness 0 |
| 3, a gate | its number; `exact` or `clean` (8.3) | its verdict, at the head or, where `clean`, at a head the change's clean updates led through, approves | no such verdict yet, or one pending | such a verdict asks for changes | guarded |
| 4, an approval | a role; how many people; `exact` or `clean` | that many distinct people holding the role in the project approved the pull request at the head, or where `clean` at a head its clean updates led through | fewer have; the reviews too many to read whole | one of them asked for changes at such a head | guarded |

- **Judged at the merge's head.** The core asks the forge for each
  verdict as it checks the merge (jig's `domain/root.md`, section 9),
  and the forge answers from the fresh reads its step made for that
  decision (5.3). Facts read for another head are wait.
- **Guarded or observed.** Forgejo applies a merge only at the head it
  names (section 6; skein's `forge.md`, section 9), so a fact
  pinned to that head still holds as the
  merge lands: CI, gates and approvals are guarded, and are marked as
  requirements that must be guarded. Up to date is observed: the base
  may move between the decision and the merge, and Forgejo does not
  check it then. Its freshness is zero, so only the reads of the step
  that decides the merge count. A base that moves in between lands the
  change on a tip it was not checked against, which the branch's own CI
  then checks (section 9).
- **An approval,** the requirement, is a review a person posts on the
  pull request on the forge, as themselves, apart from a person's gate
  answered in the client (8.3). The forge reads the pull request's
  reviews, keeps each author's latest, and counts the distinct people
  among them whom the core knows by their forge user's id with the role
  in the project (section 4); a review that only comments counts for
  nothing.
  It is the one thing temper reads of what people write on its pull
  requests (section 13). A project has approvals only if its rules ask
  for them: temper's own default has none (`../README.md`, section 2).
- **Landing rules** come from the configuration, the deployment's and
  each project's (`../protocol/configuration.md`, section 3, `landing`),
  each covering a pattern of landing branches with CI, up to date, its
  gates and its approvals. A merge meets every rule that covers its
  branch: a project's rules add to the deployment's, and never replace
  them; gates a plan asks for on one change are the change's own (8.3),
  and add to the rules', never replace them.
- **The usual rule** for `main`: CI passed at the head (or the
  repository's checks, without CI), the head up to date with the branch,
  and an agent's review as a gate. A project adds what it wants on every
  landing (a review with a lens, people's approvals) to its rule.
- **Enforced, and waited for.** A rule marked `enforce` makes each of its
  criteria a requirement the core checks at the merge, whatever any
  task's authority says: the rule. Every change also waits, on its own,
  for CI, its blocking gates and its base's tip (8.2): the mechanics. A
  rule not marked `enforce` leaves its criteria to those waits alone. A
  requirement that waits keeps the change at its turn until it is asked
  again; one that refuses holds the change, saying which, and never
  lands it. Approvals are waited for at the merge alone; for what the
  procedure waits for itself, a refusal at the merge is a bug in the
  procedure, counted (jig's `domain/authority.md`, section 10).
- **Force-pushing does not exist.** No kind forces a push or moves a
  branch backwards: a run's push is a fast-forward
  (`../protocol/git.md`, 4.8), an update merges the base in
  (section 10), and a branch found moved backwards is drift (8.5).

## 9. Landing queues

One per landing branch that has changes ready. A repository belongs to
one project (another may read it as context), so its goals' priorities
are comparable.

- **In the queue:** a change is in the queue once it is ready (CI passed
  and every blocking gate holds, at its head). It keeps the time it
  first became ready while it stays live, so leaving and coming back
  does not send it to the back.
- **Ordered by priority,** its goal's (jig's `domain/core.md`, 6.2), a
  change outside any goal taking its project's default, **then by that
  time.** Priorities move with the goals'. **Aging bounds starvation:**
  a change ready for longer than the deployment's queue window
  (`forge.queue_window_ms`, `../protocol/configuration.md`, section 7)
  goes before every change that became ready after it, whatever their
  priorities.
- **The first owns the queue.** Only the first is brought up to date and
  merged; the rest wait. It keeps its turn through a clean update, CI on
  its new head and its merge, so no CI run is wasted by a change
  overtaking it. It **leaves the queue** on a conflict, on a check that
  fails (CI, a gate asked again), on a gate added, or when held or
  closing, and the next becomes first; it comes back when it is ready
  again, at its place by its first time. A resolution never holds the
  queue.
- **Derived, not kept.** A queue is the ready changes into a branch,
  their priorities and their times; nothing of it is stored beyond the
  changes' own states, so a restart rebuilds it from them.
- **The branch's own CI.** A landing temper did not make may break the
  branch. Once CI fails on the branch's tip, the queue pauses and one
  repair is made: a change task into that branch, first in its queue,
  requested by the deployment on the project's behalf with the
  authority its policy gives such repairs, funded from the project's
  spend (jig's `domain/authority.md`, section 6), and news for every goal
  with changes waiting there. Every change in the queue waits for it
  rather than repairing a failure that is not its own. The pause ends
  when CI passes on the tip; repairs are made one at a time, up to a
  bound, past which the queue is held for a person, who may also release
  it as it is (a flaky check).
- **Where temper may not merge,** the queue still orders what is ready,
  for people to see; each change's landing waits for someone else's
  merge (section 13).

## 10. Updates: lazy and mechanical

- **Lazy:** a change takes its base when it is first in its queue, or
  about to get a run for another reason (8.2). It never does only
  because its base moved.
- **Mechanical:** the update merges the base into the change's branch,
  with no LLM. Forgejo makes it itself (its pull request's update, by a
  merge), so it is an effect, as a merge is. A merge rather than a
  rebase: the branch only moves forward, so it is never forced, and the
  heads before the update stay the change's own.
- **A conflict** makes the forge refuse the update, and the change is
  resolved (8.4).
- **A semantic conflict,** a clean update whose CI fails while CI passes
  on the base's tip, is a repair, whose brief names the base merged in
  and what landed in it. While CI fails on the base's tip, the failure is
  not the change's: its queue is paused (section 9), and the change
  waits for the branch's repair instead.
- **Gates carry over** a clean update temper made, where their
  `freshness` is clean (8.3); after a conflict an LLM resolved, they are
  asked for again.

## 11. Branches

- **Named after the tree.** Every branch temper makes is under the
  repository's prefix (`temper/` by default, one deployment's on a
  repository: section 4) and its tree's root task:
  - a change's own branch: `temper/<root>/c<task>`;
  - a task's saved work: `temper/<root>/s<task>`, never the change's
    branch, so it starts no CI and moves no pull request;
  - an agent's task's own branch, for runs that push outside a change,
    as a chat making a small fix does: `temper/<root>/r<task>`, in each
    repository its runs may write, the project's home among them, which
    the task holds from the commit that makes it and hands to the change
    task that lands it.

  They are siblings under `temper/<root>/`, so no name is the prefix of
  another as git refs forbid, and one grant covers a tree's branches;
  a task's own branches are named in its batch symbolically and resolved
  once it has its number (jig's `domain/authority.md`, section 4).
- **Held by the change** that lands it, and written by its delegates one
  at a time (jig's `domain/tasks.md`, 6.3).
- **Landing branches are never made by a push.** A landing branch must
  exist when it is adopted or named; a goal's own branch (11.1) is
  created at its base's tip by the change that lands it, as an effect.
- **Deleted as their tasks close,** as the release of their holds (jig's
  `domain/tasks.md`, 6.5): a landed change's branch, a cancelled one's,
  and a task's saved work. A failed change's branch is kept for a person
  to read, and deleted as its tree's root closes.

### 11.1 A branch per goal

Whether a goal's changes land into a branch of its own, which then lands
whole, or each into the shared base, is the plan's choice like any
other: a change's landing branch is its parameter. Landing each into the
base is the usual answer: plans meet early and in small pieces; a goal
that needs another's work has it once that lands; and repositories that
pin each other's commits (temper locks skein's in `Cargo.lock`) pin a
commit of the base, which no later squash of a goal's branch replaces.
A goal's own branch suits work likely to be dropped; a goal
checked whole before it lands (by an end-to-end run, by a review of the
whole); intermediate states that would break the base; and upstream
contributions (section 14). Such a branch is short-lived: it is a change
of its own, landed by the change procedure like any other, and it takes
the base's landings that touch its area as they come, by an update. A
project's rule may require or forbid it.

## 12. Issues

The forge stays readable: issues show goals and their milestones, pull
requests their changes, for people who prefer to read there and for the
history they keep if temper is unplugged. Issues are projections (jig's
`domain/core.md`, 8.1), written and never read back:

- **One per goal,** in its project's home (jig's `domain/core.md`, 6.1),
  created when the task becomes tracked. Its body is the goal and its
  plan as a task list, which the forge renders with its progress,
  rewritten as the plan changes, at most once per `interval` and only
  when what it projects changed.
- **Milestones as comments,** each once: the plan accepted; each revision
  and its reason, from the tree's history; each change landed or held;
  each report the goal's agents mark as worth keeping ("why not library
  X"), and the design agreed, in their words; the goal finished, after
  which the issue is closed.
- **Pull requests refer to it** ("Part of `ai/temper#12`"), so the forge
  links each change to its goal, across repositories too.
- **Findings as issues:** a bug found while exploring, a follow-up a
  review raised, a flaky test, created by an agent's `effect_forge` tool
  (`agent.md`, 4.3; jig's `domain/connectors.md`, 4.5), keyed. Their
  state (open, taken up by a goal, dropped) is the store's, shown in the
  client (`../client/forge.md`, 7.3); taking one up is a goal that
  names it.
- **A projection, not a procedure** (jig's `domain/connectors.md`,
  section 8). The `issues` child domain is fed each goal's state, its
  plan and its milestones by the root: it is part of tracking, not work
  anyone asked for, so it has no task of its own. Its effects are keyed
  by the goal and what they write, made with the authority the project's
  policy gives its goals' projections (jig's `domain/authority.md`,
  section 6), in its home; the last ones, the closing milestone and
  closing the issue, are made as the goal closes.
- **Issues others open** in an adopted repository are participating
  objects, which may become intake later.

## 13. Participating objects

Not needed first, and kept possible with one mechanism (jig's
`domain/core.md`, 8.2), for three cases: outside contributions to a
project temper works on; a repository where temper may not merge; and
temper's own contributions upstream (section 14).

- **What people write on what temper owns is no input.** A comment on
  temper's pull request or issue reaches no task; the pull request's
  body says where to reach temper instead. The one fact temper reads
  there is a review's verdict by a person a landing rule's approval
  counts (8.7).
- **Outside contributions** to a project temper works on: their issues
  and pull requests, which a task may triage and review. A task given
  one subscribes to its comments and reviews; what others write reaches
  its inbox as news, in order, from a position the connector keeps.
  Outside contributors are not the project's parties unless adoption
  seeded them (section 4) or they sign in: what they write is a
  participant's words.
- **A repository where temper may not merge:** a change there is ready
  when its CI and gates hold, and then waits for someone else's merge,
  which is a landing like any other. People's reviews on its pull request
  are news for the change; a request for changes is a repair, given its
  remarks.
- **What temper writes** there (a reply, a review) is a participant's:
  drafted by an agent's task, and, where the project's policy says so,
  proposed to a person before it is posted.

## 14. Upstream contributions

Not needed first, and kept possible:

- **A fork,** on the upstream's forge where it can be, so that upstream's
  CI runs on it unchanged and the pull request is native there. Its main
  follows upstream's, fast-forward only, and temper's own changes land
  into a work branch in the fork: a goal's own branch (11.1).
- **The submission is a task:** brought up to date with upstream, its
  commits shaped to upstream's conventions, described and signed off,
  and accepted by a person before anything is posted, often under that
  person's own identity.
- **Once posted,** its pull request is a participating object
  (section 13): a task reads the maintainers' reviews, updates the
  branch, and drafts replies a person approves.

## 15. A goal and its landings, step by step

### 15.1 A goal, as tasks

A person, P, reaches a plan for adding OAuth login to temper:

```
P, a person
├─ T0  a chat with P about temper          agent, a chat
└─ T1  add OAuth login                     agent, a coordinator's charter; tracked;
   │                                       T0 proposed it, P accepted it
   ├─ T2  spike: library X                 agent, read-only: a report
   ├─ T3  spike: a hand-rolled flow        agent, read-only: a report
   ├─ T4  choose       after T2, T3        person P: a choice
   ├─ T5  design       after T4            the forge's change, a design review as its gate
   ├─ T6…T9  changes   T5 as their input   the forge's change, made by T1
   │                                       once it read T5's result
   │     T7 amended while live: a memory-safety review as a gate
   └─ T10 try it       after T6…T9         agent: a report on what landed
```

- **T1 is proposed, explicitly.** Its authority (a budget, landing into
  main) is beyond T0's, so T0's `delegate` is refused, naming what it
  lacked, and T0 proposes it (jig's `domain/tasks.md`, section 9). P
  accepts it and funds it from P's pool. A goal a chat proposes is
  requested by the person who accepts it (jig's `domain/core.md`, 6.2),
  so T1 is P's, and outlives T0, which keeps a reference to it. Within
  T1's authority, its own delegations need nobody's acceptance.
- **T1 is a goal.** Its issue shows the plan as T1 revises it
  (section 12); its changes queue for main at its priority (section 9);
  landings in temper are news for it (section 7).
- **Results flow along dependencies:** T2's and T3's reports reach T4,
  which P answers in the client; T4's choice reaches T5's brief.
- **T6…T9 are a revision,** made by T1 once T5 landed, within its
  authority; T5 has ended, so its result reaches them as an input, not a
  dependency.
- **The memory-safety review is one amendment:** T1 adds a gate to T7
  (8.3). The change procedure runs a review task at each head of T7 its
  verdict does not carry over to, and holds the merge until it passes.
- **Under each change,** the procedure makes its own tasks: producing,
  repairs, reviews, resolving conflicts (section 8).
- **Had P asked for a small fix,** T0 would have made it itself and
  handed its branch to one change task (8.1), and there would be no T1.
- **Had P closed T0** while T1 ran, T1 would go on: it is P's.

How T6…T9 land while another goal lands into the same branch is 15.2.

### 15.2 Landing, step by step

One repository, temper, with main at M0, and two goals in flight:

- **goal A:** change A1 is ready to land (CI green on its head a1,
  reviewed), and A has steps still to come;
- **goal B:** change B1 is ready to land (CI green on b1, reviewed), and
  change B2 is being produced by a run that started from M0.

A1 and B2 both change `limits.rs`; nothing else overlaps. A has the
higher priority.

| When | Who | Does | Afterwards |
|---|---|---|---|
| t0 | Engine | A1 is first in main's queue, and its head holds M0: merges A1 at exactly a1, through Forgejo | main is M1 |
| t0 | Engine | news for A (its own step landed) and B (A1 landed in temper, overlapping neither B1 nor anything B has pushed): neither wakes | B1 and B2 untouched |
| t1 | Engine | B1 is first, and its head lacks M1: asks Forgejo to update it | |
| t1 | Forgejo | merges M1 into B1's branch, cleanly | B1's head is b1′ |
| t1–t2 | CI | passes on b1′ | |
| t2 | Engine | B1's review carries over the clean update; merges B1 at exactly b1′ | main is M2 |
| t2 | Engine | news for B (its own step) and A (B1 overlaps nothing of A's): neither wakes | |
| t3 | Agent, worker | B2's producing run finishes; the worker pushes it | B2's head is b2, built on M0 |
| t3 | Engine, CI, agent | opens B2's pull request; CI passes on b2; a review task approves it | B2 is ready |
| t4 | Engine | B2 is first, and its head lacks M2: asks Forgejo to update it | |
| t4 | Forgejo | refuses: B2 and A1 both changed `limits.rs` | a conflict |
| t4 | Engine | B2 is resolving: a task for an agent; news for B, which does not wake it while the task resolves it | |
| t4 | Worker, agent | the worker prepares the merge in progress; the agent resolves `limits.rs` and runs the checks; the worker commits the merge, with both parents, and pushes it | B2's head is b2′ |
| t5 | CI, agent | CI passes on b2′; a review, asked again since an LLM resolved a conflict, approves it | |
| t5 | Engine | merges B2 at exactly b2′ | main is M3 |
| t5 | Engine | news for B (its last change landed: its coordinator's policy wakes on the last of its delegates done) and A (no overlap: kept) | |
| t5 | Coordinator B | its one run: reads its inbox (A1 and B1 landed, the conflict resolved) and finishes goal B with its report, which the issue's last milestone carries | goal B is done |

## 16. Restart

At a restart (jig's `domain/engine.md`, section 6; jig's
`domain/connectors.md`, section 14) the connector is given its records
from the store: the projects' repositories and what temper may do there,
the live tasks' resources and holds, the changes' and issues' states, and
its outbox's entries with their attempts. The deployment's id is given
to the client first, as its namespace, before any restored entry or live
resource. The client's part is skein's `forge-client.md`, section 8: what
it reads afresh before anything else steps, how it settles an outbox
entry as its class says, and where its listings start. temper's steps:

1. it reads afresh each repository's landing branches' tips and each
   live pull request (head, state, mergeability, CI), before any change
   steps;
2. it settles its outbox, each entry as its class says (section 6): a
   pull request found by its branches, a merge by its pull request's
   state, a creation by its key after the entry's start, a set written
   again; one not found waits for the retry deadline its last attempt
   committed, and an unrecoverable one holds its task for a person;
3. it steps every change once, and every goal's issue.

Its listings start from the newest time the store kept for each
repository, inclusive, so what changed while the engine was down is
read once.

## 18. The world

The connector's world, `tests/engine/forge`, runs its subtree with the
fake forge below it (`../testing.md`, 4.2) and the root scripted above
it; the tiers that run it through bytes and on a real Forgejo are
`../testing.md`, section 2. Its stories: a change produced, opened,
checked, gated, queued and landed; CI failing, repaired; a gate asking
for changes, repaired, its verdict given again; a gate added while
queued; two changes into one branch, the second updated cleanly and its
review carried over; an approval carried over a clean update and asked
again after a repair; a change that leaves the queue for a conflict and
comes back at its place; a low-priority change aging past the window
ahead of later ones; a conflict resolved from a merge in progress; a
semantic conflict repaired; the branch's own CI broken by a landing
temper did not make, the queue paused and repaired; its pull request
closed by someone else, held, released and reopened; a landing that
overlaps a goal's change, and one that does not; a goal's issue
written, revised and closed; a merge whose answer is lost, found merged
after a restart; a mergeability unsettled within the window, then
settled; a forge preloaded with ten times the history, idle at the same
cost.

Its referee, beyond jig's for every connector (jig's
`domain/connectors.md`, section 15): nothing lands without every
requirement its branch's enforced rules name, guarded ones met at the
exact head merged and up to date met on the reads of the decision that
merged it; merges land exactly the heads decided; temper never writes to
what it does not own, nor forces a push; a held change makes no effect;
a landing wakes a goal exactly when it overlaps; no change waits in a
queue past the aging window behind changes that became ready after it;
temper's own pushes are never taken for drift; the calls made while
idle are bounded by the limits.

## 20. Open questions

- **A queue's throughput:** landings into one branch wait on CI one after
  another. If that binds, landings are batched: tested together, and
  bisected when they fail.
- **Conflicts found before either side lands:** temper sees every open
  change, and could try them against each other and warn both goals.
- **Checks without an LLM:** a repository's checks run as agents' tasks
  at the exact head (8.3); a checks-only run on a worker, running the
  project's commands with no model, would cost no turns (`worker.md`,
  section 12).
- **A person's approval as a rule:** temper takes it two ways, a person
  gate answered in the client (8.3) and a review the person posts on the
  forge, which a landing rule counts (8.7). Whether an approval given in
  the client should also be posted on the forge as the person, so that
  the forge's protection requires it independently, needs a forge
  credential of the person's (jig's `domain/people.md`, section 12).
- **A failing check's repair brief:** a check gate that says no is
  repaired as a review that says no is, and its repair's brief carries
  the pull request's reviews section (8.4). A check's verdict is not a
  review on the forge: whether the brief should carry the check's own
  output instead is open.
- **A failure that is the runner's, not the change's:** a job whose
  set-up step failed, or whose runner was lost, is a repair like any
  other. Running the job again first, through the API, is
  `../draft/ci-on-forgejo-17.md`, with the stalls' reasons and the
  agents' log tools.
- **Two Forgejo behaviours no fact records yet** (`../protocol/forge.md`,
  section 14): that CI after a clean update runs on exactly the head the
  update made (section 10); and where a merge lands when its pull
  request is retargeted after the fresh read before it, since Forgejo's
  merge is conditional on its head alone (section 6).
- **Intake:** issues others open as a source of goals.
- **A resource refused as forbidden:** whether the connector should hear
  when it is readable again, rather than read it on a backoff.
- **A merge style per repository,** named by the project rather than the
  repository's default (skein's `forge.md`, section 12). Default: the
  repository's.
- **GitHub,** later: the same connector over another protocol layer
  (`../protocol/forge.md`, section 14), its pull request update and
  merge queue mapped onto the same effects.
