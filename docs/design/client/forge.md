# The forge in temper's client

Provisional, 2026-10-05, revised 2026-10-20. What the forge connector
adds to temper's client: a change's way to landing and its result, a
project's landing queues, approvals at a head, the forge's holds in
words, repositories with their health and landing rules, the picker of
repositories to adopt, goals, findings and landings as the forge shows
them, and the forge's part of the deployment, where each configured
forge is connected. jig's pages and cards (jig's `client/domain/pages.md`
and jig's `client/domain/objects.md`) leave a slot for each part a
connector shows (jig's `client/domain/root.md`, section 6), and
temper's pages and cards, its own after them, show the same parts
(`README.md`, section 1); this document says what the forge shows in
those slots and on the pages only temper has, `Changes`, `Adopt` and
`Forges` (`README.md`, 3.5), in temper's words. The forge's client
family, its words for resources and kinds, its adoption form and its
requests are `README.md`, sections 4 and 8. Each part shows what `../domain/forge.md` keeps or reads,
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
  changes about. Where a branch lands through GitHub's merge queue,
  temper's queue decides only what enters it and when, and the panel
  says "in GitHub's merge queue, n" (2.1).
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
  moved, a job's log on a host temper does not read (5.4).
- **Forges are connected from the client.** Configuration names each
  forge, never its access; on the deployment's `Forges` page (5.4) a
  person whose role holds the deployment's connectors connects it:
  GitHub through an App made from temper's manifest, or one of their
  own, its key pasted; Forgejo with its bot's tokens pasted, or a site
  admin's token from which temper makes its bot. The page says what
  temper is there, what stopped it, and offers Replace.
- **Adoption picks from what temper can reach** (5.1): per connected
  forge, the repositories temper can reach there, a page at a time by
  number, and "Not listed" for one named by owner and name, with what
  would make it reachable. A forge not connected offers none.
- **The forge's refusals are said with their remedy** (5.5): not
  connected, not installed on an account, not granted to temper's App
  (linked to the installation's page on GitHub), stopped and why.
- **The browser's steps of a connect are temper's routes** on the
  clients' listener (9.1): the manifest's page, GitHub's redirect and
  its setup, each authenticated by a one-use state alone. The client
  prints a connect's link and never opens it.

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
| queued | its place in its landing branch's queue, and who is ahead; handed to GitHub's merge queue, "in GitHub's merge queue, n", n its place there |
| first | first in the queue: updating from its base, CI on the updated head, or merging |
| repairing | why (CI's failure, a gate's remarks, a semantic conflict, taken out of GitHub's merge queue and GitHub's reason), the repair as a chip, and how many repairs are left |
| updating | the base it takes in |
| resolving | the conflicting files, the resolution as a chip, and how many are left |
| recreating, reopening | after drift released (6.2): its branch made again at the head temper last knew, or its pull request reopened or retargeted back |
| landing | merging at its head |
| landed | its merge commit, and when |

Where the landing branch requires GitHub's merge queue
(`../domain/forge.md`, on merging through it), temper's own queue
decides only which change enters GitHub's and when: the first change's
merge hands it to GitHub's queue at the head its decision saw, and
until it lands or leaves, the panel says "in GitHub's merge queue, n",
n the position the forge reads for its pull request (`Pull.queued`) as
each read gives it. GitHub builds the merge group, runs CI on it and lands
it; the landed step names the commit the group landed (`Pull.merged`),
which is not the head temper saw. A pull request GitHub takes out of
its queue for any reason but its merge is repaired, as a failed CI is,
the repairing step naming GitHub's reason. Forgejo has no such queue.

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
  request's number and title, its merge commit (through GitHub's merge
  queue, the commit its group landed), and the address of each on the
  forge; or ended otherwise, with what became of its pull request
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
its changes are gated on without it. The merge style is the forge's for
the repository, which the form shows before the adoption is confirmed:
on GitHub the first the repository allows of merge, squash and rebase,
on Forgejo its default (`../domain/forge.md`, what adoption says of the
branch's rules). A repository where another
deployment's branches already live under the prefix is refused, saying
so (jig's `domain/connectors.md`, section 12).

**The picker.** Below the forge's choice, for the chosen forge, the form
lists the repositories temper can reach there (the `Reachable` request,
`README.md`, section 8), a page at a time by number from 1, each page
at most `picker.page` rows (`../protocol/configuration.md`, its limits)
clipped to the forge's own page cap, with "Next" while the answer says
more follow, which it says when the page is full. The request names no
project, so the project's own repositories are among the rows; adopting
one again is refused as any adoption is (section 10). Each row is
OWNER/NAME, marked "private" where it is; the repository's id and its
owner's id the row carries ride in the adoption request and are never
shown. Choosing a row fills the owner and the name. What temper can
reach:

| The forge | Listed |
|---|---|
| GitHub | one accepted installation's repositories of temper's App (GitHub's `GET /installation/repositories`, on a token the setup's host mints for the page and drops), the installation shown by its account's login and asked for by its id, which the picker's request carries, the installations as the deployment's page reads them (5.4), marked "selected repositories" where its installation has chosen repositories and "suspended" where GitHub suspended it; read by the setup's host's `Reachable { forge, installation, page }` plan (skein's `forge-setup.md`, 4.8) |
| Forgejo, connected | the repositories its API token can push to, read by the same plan, with no installation |
| Forgejo, managed or owned | every repository the site admin can see, read by the setup's host's `Reachable { forge, installation?, page }` plan on the admin's token (skein's `forge-setup.md`, 4.8), by id ascending, its `reachable` rows given from `picker.page`, since adopting one grants the bot its access (`../protocol/forge.md`, the setup's host's `Reachable` and `Adopt`); the admin's token stays the setup's host's, and the picker gets the rows alone. The plan runs beside the forge's one writing run; a second read of the picker while one runs is refused busy, and so is one past `setup.runs`, which counts both. Its own refusals are the admin's token's, as the reconcile's first step has them (`forge-setup.md`, 4.3): "the admin's token is missing" and "NAME is not a site admin" |

A forge not connected offers none: the picker says "Forge F is not
connected." and links to the deployment's page (5.4), and the "Adopt"
button stays disabled for it. A suspended installation's repositories are
listed and marked; adopting one is refused as no access until GitHub
unsuspends it.

**Not listed.** Below the list, "Not listed" takes a repository's owner
and name, and the form says what would make it reachable, by the forge
and what its read answered:

| Where | The form says |
|---|---|
| GitHub, an account whose installation has selected repositories | "Grant OWNER/NAME to temper's App on GitHub's installation page.", linked to that installation's page (5.5) |
| GitHub, an account with no installation of temper's App | "temper's App is not installed on OWNER.", with "Install on another account" (5.4) |
| Forgejo, connected | "Make LOGIN a collaborator with write on OWNER/NAME.", LOGIN the bot's |
| Forgejo, managed or owned | nothing: adopting it grants the bot its access |
| any forge | a repository its credential can read may be adopted as context |

Once the forge says so (an installation's repositories changed, a
collaborator added), the repository is listed on the picker's next
page, with no restart.

**The rules, in words** (`../domain/forge.md`, on what adoption reads):
the landing branch's rules as the forge applies them, each with what it
means for landing: a merge queue (temper enters it, 2.1), required
approvals (a person approves, or temper's App may bypass), required
status checks (CI as the landing rule's), code owners (a person
reviews), signed commits (temper's commits are unsigned, so where the
forge refuses its merge for it, its landings wait), linear history (a
merge style that keeps it), and whether
temper's App may bypass the rules. Where a rule keeps temper from
landing, the form names the one setting on the forge that would let it
land, for the person to choose; until it is changed, its ready changes
wait for someone else's merge (section 3). temper never changes a
repository's rules. On GitHub temper writes no commit status
(`../domain/forge.md`, 4.4), so every GitHub adoption says "Commit
statuses stay temper's own: GitHub's rules cannot require them."
Forgejo's reads answer none of the merge queue,
signed commits, linear history and code owners, and no bypass, so on
Forgejo the form says its branch protection alone.

### 5.2 Health

Wherever a repository is shown (the project's page, its settings, the
deployment's page):

- **what temper may do** there, and whether that narrowed since adoption
  ("a merge was refused: temper may no longer land into main");
- **its webhooks:** when they were last heard, on GitHub the App's
  deliveries as its delivery log records them (`../protocol/forge.md`,
  a GitHub App's delivery log); none, or none for days, means changes
  are seen by polling alone, more slowly;
- **its landing branches:** their tips and CI on them;
- **on GitHub, its installation:** the account's, marked suspended
  while GitHub suspends it; a repository that left the installation is
  marked "not granted to temper's App", linked to the installation's
  page (5.5);
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
`client/domain/pages.md`, section 7), on temper's `Forges` page
(`README.md`, 3.5), read for a person whose role on project 0 is
`Owner`, in standalone the deployment's owner; any other's read is
refused, in jig's words (`README.md`, section 8). A row for each
configured forge, by its number, its kind and its `web` address, says:

- **Its connection,** one of:

  | State | The row says |
  |---|---|
  | not connected | "Not connected", with Connect; where its records say why, "Not connected: its records are unreadable" or "Not connected: its records were made for KIND at ADDRESS", and connecting again overwrites them (`../protocol/forge.md`, at start) |
  | not set up | an owned forge not prepared yet (`../protocol/setup.md`, 6.5): under the box, "Not set up", with "Set up a Forgejo here"; without it, "Not set up: run `temper forge fetch N`, `temper forge prepare N`, `temper forge run N forgejo` and `temper forge run N runner`, then restart `temper serve`", N its number, with no action |
  | setting up | the box runs its steps, asked: "Setting up: fetching", "Setting up: preparing" or "Setting up: starting", by the box's last word of the step it runs, else "Setting up" (`../protocol/setup.md`, 7.1); then "Connecting", as its handover is taken and reconciled (`../protocol/setup.md`, 5.1) |
  | set up failed | "Set up failed: WHY", the box's reason in words (a step that exited, a digest that differs, a version refused, its port taken), with "Set up again"; the engine and every other forge run on |
  | connecting | "Connecting": a run holds the forge, and what it waits for: a managed or owned Forgejo's reconcile, the forge's answers to a check, or the person's browser on GitHub until the link's state ends (`setup.state_life_ms`) |
  | connected | "Connected as LOGIN (ID)", the user the records hold: the bot's on Forgejo, the App's on GitHub |
  | stopped | "Stopped: WHY", the component's reason in words (its key or token refused, a user other than its own, an access it finds invalid, a refusal that asking again would not mend), with Replace; the engine and every other forge run on |

  A connect that ends otherwise is said on the row until the next one:
  refused (the person's role, the forge's kind or mode, "busy": the
  forge's one writing run is taken, or `setup.runs` runs are running),
  "the pasted value changed", "the API token
  and the git token answer as LOGIN and LOGIN", "LOGIN is not temper's
  bot: add `temper:bot` to its full name on Forgejo" (5.5), the forge
  unreachable, a `Manifest` or an `Install` that
  stopped, its failure said here and never in its answer (`README.md`,
  section 8), or "interrupted; paste again" after a restart. A GitHub
  App lacking what temper needs connects all the same (the setup's
  host's `Lacking` is done): the row says it beside "Connected as LOGIN
  (ID)", each permission and event named (5.5), until the forge's next
  check of the App ends: a paste's (a connect or a Replace) or the
  reconcile after each connect, the one at each start among them
  (`../protocol/setup.md`, 17.5), which clears the line or names what
  the App still lacks. The line is kept in memory alone: a restart
  drops it, and the start's reconcile names it again while the App
  still lacks it. A forge other than a fake, connected while a worker's
  `git` is null (`../protocol/configuration.md`, section 4), says beside
  "Connected as LOGIN (ID)": "No git on this machine: install git 2.41
  or later and set workers[].git, then restart"; its repositories'
  commands fail on that worker, naming the forge's host, until it has
  one (`../protocol/git.md`, section 3).
- **Its mode** (`../protocol/setup.md`, the modes): owned, where the
  configuration's setup owns the forge; managed, where temper keeps a
  site admin's token for it; connected otherwise. Owned and managed say
  "admin's token kept": the setup's host alone reads it, never the
  forge's component.
- **On GitHub, temper's App:** its slug, and its installations, each
  with its account's login, "all repositories" or "selected
  repositories" (linked to the installation's page, 5.5), "suspended"
  while GitHub suspends it, and a stray, an installation never accepted
  through temper's link, "uninstalled at the next reconcile".
- **A job's log on an origin temper does not read:** "A job's log is on
  ORIGIN, which temper does not read: add it to this forge's
  `log_origins` and restart." (`../protocol/configuration.md`, the
  forges' members), until a start names it: the forge's `Fact::Origin`
  (`../protocol/forge.md`, 3.6), kept per forge in memory, at most
  `log_origins` of them, and forgotten when the forge next connects. The
  same line is in the engine's log, once until a start names the
  origin.
- **Its load:** its request budget and what uses it (fresh reads,
  writes, keeping up, the slow pass; `../domain/forge.md`, 5.4); a
  pause for its rate, and until when; its outbox, the effects waiting
  and those uncertain, being looked for; and when each repository's
  webhooks were last heard.

Its actions, each a request of the forge's family (`README.md`,
section 8), offered by kind, mode and state:

| Action | Offered | What it does |
|---|---|---|
| Connect | GitHub, not connected or stopped | "Connect GitHub", for the person's account or an organization they name: the answer is a link to the manifest's page on the clients' listener (9.1), shown as text for the person to open; on GitHub they create the App, then install it on an account, and come back to temper. Or "Connect an App of your own": its id, and its private key pasted as GitHub gives it; no webhook secret is asked, since temper points the App's hook at `https://SLUG.invalid/` and hears changes from its delivery log |
| Connect | Forgejo not owned, not connected; a Forgejo of the connected mode, stopped | its bot's API token, its git token and, where the person made hooks to temper's listener, their secret, pasted together; both tokens must answer as one user |
| Connect | Forgejo not owned, not connecting | "Connect as its admin": a site admin's token pasted; temper makes its bot (or takes over the connected one, 5.5) and the forge becomes managed |
| Replace | GitHub, connected or stopped; a Forgejo of the connected mode, connected or stopped | a new App key, or a new API token, git token or both, pasted, which must answer as the forge's user (on GitHub, the same App); the old is erased once the new is kept |
| Install on another account | GitHub, connected | a fresh link to install temper's App, shown as text; the person picks the account and its repositories, the page advising selected ones over all |
| Forget the admin's token | managed | the admin's token erased; the forge is connected, its bot and tokens kept, and adopting a repository grants nothing more: the person makes the bot a collaborator |
| Set up a Forgejo here | owned, not set up, under the box | the box asked to run the forge's steps (`../protocol/setup.md`, 7.1): its binaries fetched, its preparation, Forgejo and its runner started, then its handover taken and reconciled; the person signs in to it as their login, with the password its preparation wrote (`person_password`, `../protocol/setup.md`, section 9) |
| Set up again | owned, set up failed | the same ask again: what the last one left is kept, a fetched binary hashed and not fetched again |

A pasted value is a field of its own (`README.md`, 3.6 and section 8),
emptied as the request leaves and shown nowhere, then or after; the
page shows that it was given, never what. Two refusals are the page's
own, before anything is sent, the fields kept for the person to fix: a
field the action needs left empty is marked invalid, "Required." (a
Replace of a Forgejo's tokens with both empty: "Paste a new API token,
git token or both."); and the bundle of the action's values past 4 KiB,
jig's bound on a carried value (`tools.md`, section 12), shows "The
pasted values are past 4 KiB, the most a request carries." under the
action's button. The link a connect or an
install answers is shown as text, and the client never opens it
(`README.md`, 6.5).

### 5.5 How the forge's refusals show

What the forge's protocol layer answers a call, or tells the root
(`../protocol/forge.md`, a forge stopped, GitHub's installations,
what the vocabulary adds and the setup's host's outcomes), wherever a
part shows it: the deployment's
page, the adoption form and its outcome, a change's panel and its hold.

| The forge answers or tells | It shows |
|---|---|
| `NotConnected` | on the forge's row, "Not connected", with Connect (5.4); the adoption form lists none of its repositories, and an adoption on it is refused, "Forge F is not connected." |
| `NotInstalled`, no accepted installation on the owner | "temper's App is not installed on OWNER.", with "Install on another account"; a push to it is refused, as "temper's App is not installed on the account owning NAME." (`../protocol/channel.md`, section 6) |
| `Missing`, on a repository whose account has an installation of selected repositories | "OWNER/NAME is not granted to temper's App.", linked to the installation's page on GitHub: `WEB/settings/installations/ID` for a person's account, `WEB/organizations/ORG/settings/installations/ID` for an organization's, WEB the forge's configured `web` address and ID the installation's |
| `Missing` otherwise | not found, as `README.md`, 4.7 says |
| `Stopped { why }`, told to the root | on the forge's row, "Stopped: WHY", with Replace (5.4); its calls answer `NotConnected` until its next connect, shown as that row says |
| an installation suspended | "suspended" on the installation, on the forge's row and on each repository of it (5.2); its calls are refused as no access |
| a stray installation | on the forge's row, "uninstalled at the next reconcile" |
| `Lacking`, a GitHub App's `Paste` (a connect of an App of the person's own, or a Replace) whose App lacks permissions or events temper needs | on the forge's row, connected, "temper's App lacks WHAT: grant them in the App's settings, then paste its key again.", WHAT each permission, with its level, and each event, as GitHub names them, until the forge's next check of the App ends (5.4); meanwhile a call that needs what the App lacks is refused by GitHub, as no access; with Replace where the forge is connected, Connect where it is not, and a link to the App's settings, `WEB/settings/apps/SLUG`, or `WEB/organizations/ORG/settings/apps/SLUG` for an organization's App |
| `Unmarked`, connecting as the admin a forge whose user lacks temper's mark | "LOGIN is not temper's bot: add `temper:bot` to its full name on Forgejo, then connect again." |

The installation's page and the App's settings are built by the engine
from the configured `web` address, never from an address a forge's
answer gave, so a link the client shows points at the configured forge
alone. SLUG is the App's slug, as the forge's row shows it (5.4).

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
The first part served is the deployment's, its forges (5.4); a change's
result and its hold (2.5) follow.

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
| the picker (5.1) | temper's `Adopt` page, through the `Reachable` request (`README.md`, section 8) | `../protocol/forge.md`, the setup's host's `Reachable`; `../domain/forge.md`, section 4 |
| each forge's connection, its mode, installations and actions (5.4, 5.5) | temper's `Forges` page; the deployment's part | `../protocol/forge.md`, at start, connecting at runtime, a forge stopped, GitHub's installations and the setup's host; `../protocol/credentials.md`, the forges' records |
| the browser's steps of a connect (9.1) | none of jig's: temper's routes on the clients' listener | `../protocol/forge.md`, the setup's host |

### 9.1 The browser's steps, on the clients' listener

A GitHub connect's browser steps are temper's routes on the clients'
listener (`README.md`, section 8): temper's server asks jig's clients'
component first whether it claims a path (jig claims `/jig/...`), then
routes `/forges/...`, then serves the browser's bundle. GitHub sends
the person's browser to them, and nothing of temper's client calls
them.

| Route | What it does | Answers |
|---|---|---|
| `GET /forges/N/github/manifest?state=S` | the page the setup's host's `Manifest` run produced for that state, held with it: one form posting temper's manifest to `WEB/settings/apps/new?state=S`, or `WEB/organizations/ORG/settings/apps/new?state=S` for an organization, one button, no script; the manifest's redirect and setup addresses are whole URLs on the clients' listener, `http://LISTENER/forges/N/github/redirect` and `http://LISTENER/forges/N/github/setup`, and its `url`, the App's homepage, `http://LISTENER/` | `200`, the page |
| `GET /forges/N/github/redirect?code=C&state=S` | `Convert`: the code exchanged for the App's id, slug and key, each kept before the browser is answered, and its user; the App's hook points at `https://SLUG.invalid/`, so no webhook secret is kept on GitHub; then `Install` for a fresh state, whose link answers the browser, and the forge connects | `302` to the install page, `Install`'s link for that state |
| `GET /forges/N/github/setup?installation_id=I&setup_action=A&state=S` | `Accept`: the installation checked and accepted, then the forge's installations handed to its component again | `200`, a short page: "temper has the installation; you may close this tab." |

N is the forge's `number`, WEB its configured `web` address, ORG the
organization the connect named and LISTENER the clients' listener's
address, as temper builds the manifest from it
(`../protocol/forge.md`, the setup's host).

- **The manifest's page's headers:** `Content-Security-Policy:
  default-src 'none'; form-action WEB`, `Cache-Control: no-store` and
  `Referrer-Policy: no-referrer`. It loads nothing, runs nothing, and
  may post only to the configured forge.
- **The state is the only authenticator.** A route takes no cookie and
  no credential of jig's: the state, 128 bits or more drawn by the
  setup's host for one connect or one install, is good once, for its
  life (`setup.state_life_ms`), for that forge alone. The host keeps at
  most `setup.states` of them, in memory, so a restart forgets them
  (`../protocol/forge.md`, the setup's host).
- **Refused:** a state used, unknown or past its life, or a code GitHub
  refuses, is answered `410` and a short page, "This link has ended.
  Start again from temper.", with `Cache-Control: no-store` and
  `Referrer-Policy: no-referrer`, and nothing is kept. So is a run that
  stops otherwise: `Convert`'s `Stop`, whatever its failure, an
  `Install` that stops, and `Accept`'s `Stop`.
- **A visit with no state** to the setup route is GitHub's, after a
  person changed the installation's repositories on GitHub: for an
  installation accepted already it is a refresh, answered `200`, "temper
  has the installation; you may close this tab.", the routing handing
  the root `Repositories` for it so the picker lists them again, and
  nothing reaches the forge's component or the store; for any other
  installation it is refused, `410`.
- **Bounds and others:** a query malformed or past `routes.query_bytes`
  is answered `400`, in the refused page's words and headers; a method
  other than `GET` is answered `405`; a forge not configured, or one
  that is not GitHub, `404`; any other path under `/forges/`, not
  found.
- **A fixed address with a GitHub forge.** The App's `redirect_url` and
  `setup_url` are those whole URLs on the clients' listener, and
  GitHub's API cannot change them, so a deployment configuring a GitHub
  forge gives its clients' listener a fixed address, never port 0; the
  engine
  checks it at start and refuses to start, naming it
  (`../protocol/configuration.md`, the clients' listener). GitHub takes
  `http://127.0.0.1:PORT` for both, so a standalone deployment needs no
  public address: the browser that follows the link runs where that
  address reaches (section 10).
- **Tested without GitHub.** The fake forge's manifest post answers
  `200` with a small page holding one link, `Create GitHub App` (id
  `create`), to the manifest's `redirect_url?code=C&state=S`, which a
  test follows as a person clicks GitHub's form (a link, not a
  redirect, so the page's `form-action` is not crossed), and its install
  page answers `302` to the `setup_url` (skein's `fake-forge.md`, 10.1);
  so the three steps run as plain requests on the clients' listener.
  The routes' tests are `temper-service`'s own, requests through
  `fixtures::http::Peer`; the bytes through the fake's GitHub layer are
  the real loop's (`../domain/testing.md`, 2.4), whose browser's tier
  opens the manifest's page once to see its form post to the configured
  `web`; no test calls github.com.

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
- **A browser on another machine** for GitHub's connect. Default: the
  link names the clients' listener's fixed address, and the person
  opens it where that address reaches (in standalone, the engine's
  machine) or forwards the port themselves; temper makes no tunnel.
- **Mending a stopped managed or owned forge.** Default: Replace is
  offered for what a person pasted (a GitHub App's key, a connected
  Forgejo's tokens); a managed forge is mended by connecting as its
  admin again, reconcile making the bot's tokens anew for the same
  user; an owned one by its preparation and a restart.
- **Disconnecting a forge** from the client. Default: later; a forge's
  records are overwritten by connecting again, and those of a forge the
  configuration stops naming are named once in the engine's log.
- **The picker's pages kept.** Default: none; each page is a fresh read
  of the forge through the setup host's `Reachable` plan, asked by
  number, so a
  repository added or removed between two pages can move a row across
  them, seen twice or missed until the list is read again from its
  first page, and a page past the last answers no rows, as "Next" after
  a last page that came back full does.
- **The project's own repositories in the picker:** `Reachable` names no
  project, so neither the engine nor the client can leave out the
  repositories the project holds, as `../domain/forge.md`, what the
  picker offers, says the list does. Default: every reachable
  repository is listed, and adopting one the project holds is refused
  by the engine as any adoption; the request naming the project is the
  candidate, and the domain's page is asked to say which.
